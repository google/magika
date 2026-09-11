// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Small ownership wrapper around the Vectorscan C API. Conditions execute in Vectorscan.

use std::ffi::{c_char, c_int, c_uint, c_void, CStr, CString};
use std::ptr;
use std::sync::Arc;

use anyhow::{bail, ensure, Context, Result};
use libloading::Library;

use super::compiler::{Expressions, Program, Streams};
use super::engine::Decision;
use super::preprocess::Synthetic;
use crate::ContentType;

type Free = unsafe extern "C" fn(*mut c_void) -> c_int;
type Alloc = unsafe extern "C" fn(*const c_void, *mut *mut c_void) -> c_int;
type Callback = unsafe extern "C" fn(c_uint, u64, u64, c_uint, *mut c_void) -> c_int;
type Scan = unsafe extern "C" fn(
    *const c_void,
    *const *const c_char,
    *const c_uint,
    c_uint,
    c_uint,
    *mut c_void,
    Option<Callback>,
    *mut c_void,
) -> c_int;
/// `hs_compile_ext_multi`: the only compile entry point, so every expression can carry
/// extended parameters; a null `hs_expr_ext_t*` means none.
type Compile = unsafe extern "C" fn(
    *const *const c_char,
    *const c_uint,
    *const c_uint,
    *const *const ExpressionExt,
    c_uint,
    c_uint,
    *const c_void,
    *mut *mut c_void,
    *mut *mut CompileError,
) -> c_int;

#[repr(C)]
struct CompileError {
    message: *const c_char,
    expression: c_int,
}

const HS_EXT_FLAG_MIN_OFFSET: u64 = 1;
const HS_EXT_FLAG_MAX_OFFSET: u64 = 2;

/// `hs_expr_ext_t`: `min_offset`/`max_offset` bound the inclusive end offset of a match.
#[repr(C)]
struct ExpressionExt {
    flags: u64,
    min_offset: u64,
    max_offset: u64,
    min_length: u64,
    edit_distance: c_uint,
    hamming_distance: c_uint,
}

impl ExpressionExt {
    fn bounded(bounds: super::compiler::Bounds) -> Self {
        Self {
            flags: HS_EXT_FLAG_MIN_OFFSET | HS_EXT_FLAG_MAX_OFFSET,
            min_offset: bounds.min_end_offset,
            max_offset: bounds.max_end_offset,
            min_length: 0,
            edit_distance: 0,
            hamming_distance: 0,
        }
    }
}

#[repr(C)]
#[derive(Default)]
struct Platform {
    tune: u32,
    cpu_features: u64,
    reserved1: u64,
    reserved2: u64,
}

pub(super) struct Api {
    scan: Scan,
    alloc: Alloc,
    free_db: Free,
    free_scratch: Free,
    platform: Platform,
    version: String,
    library: std::mem::ManuallyDrop<Library>,
}

fn load_library(path: &std::ffi::OsStr) -> Result<Library> {
    #[cfg(feature = "yara-rules")]
    let _startup_span = crate::startup_trace::span("native_library_dlopen");

    #[cfg(windows)]
    {
        use libloading::os::windows::{
            Library as WindowsLibrary, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
        };
        let path = std::path::Path::new(path);
        let mut flags = LOAD_LIBRARY_SEARCH_DEFAULT_DIRS;
        let resolved;
        let path = if path.components().count() > 1 {
            resolved = std::fs::canonicalize(path)?;
            flags |= LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR;
            resolved.as_os_str()
        } else {
            path.as_os_str()
        };
        // Exclude CWD and PATH for both the engine and its dependencies. An explicit
        // path additionally permits dependencies beside that deliberately selected DLL.
        Ok(unsafe { WindowsLibrary::load_with_flags(path, flags) }?.into())
    }
    #[cfg(not(windows))]
    Ok(unsafe { Library::new(path) }?)
}

impl Api {
    pub(super) fn load() -> Result<Arc<Self>> {
        #[cfg(feature = "yara-rules")]
        let _startup_span = crate::startup_trace::span("native_api_load_total");

        let name = if cfg!(target_os = "macos") {
            "libhs.dylib"
        } else if cfg!(target_os = "windows") {
            "hs.dll"
        } else {
            "libhs.so.5"
        };
        let path = std::env::var_os("MAGIKA_VECTORSCAN_LIBRARY").unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| {
                    let bundled = exe.parent()?.join("lib").join(name);
                    bundled.is_file().then(|| bundled.into_os_string())
                })
                .unwrap_or_else(|| name.into())
        });
        // Loading executable libraries requires a trusted system installation or explicit path.
        let library = load_library(&path).with_context(|| {
            format!("load Vectorscan {path:?}; set MAGIKA_VECTORSCAN_LIBRARY to its library path")
        })?;
        let mut platform = Platform::default();
        // Resolve symbols before allocating native resources. Platform and version describe the
        // actual loaded engine, not a guessed CPU model or the library's filename.
        unsafe {
            let populate = library
                .get::<unsafe extern "C" fn(*mut Platform) -> c_int>(b"hs_populate_platform")?;
            ensure!(populate(&mut platform) == 0, "Vectorscan platform detection failed");
            let version = library.get::<unsafe extern "C" fn() -> *const c_char>(b"hs_version")?;
            Ok(Arc::new(Self {
                scan: *library.get::<Scan>(b"hs_scan_vector")?,
                alloc: *library.get::<Alloc>(b"hs_alloc_scratch")?,
                free_db: *library.get::<Free>(b"hs_free_database")?,
                free_scratch: *library.get::<Free>(b"hs_free_scratch")?,
                version: CStr::from_ptr(version()).to_string_lossy().into_owned(),
                platform,
                library: std::mem::ManuallyDrop::new(library),
            }))
        }
    }

    pub(super) fn identity(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}:{}",
            self.version,
            std::env::consts::OS,
            std::env::consts::ARCH,
            cfg!(target_endian = "little"),
            self.platform.tune,
            self.platform.cpu_features
        )
    }

    /// Compiles both streams; a stream without a terminal rule has no database.
    pub(super) fn compile(self: &Arc<Self>, program: Program) -> Result<Arc<Database>> {
        let streams = program.try_map(|expressions| self.compile_stream(&expressions))?;
        Ok(Arc::new(Database::new(self.clone(), streams)?))
    }

    fn compile_stream(self: &Arc<Self>, program: &Expressions) -> Result<Option<Stream>> {
        if program.outputs.iter().all(Option::is_none) {
            return Ok(None);
        }
        let (compile, free_error) = unsafe {
            (
                *self.library.get::<Compile>(b"hs_compile_ext_multi")?,
                *self.library.get::<Free>(b"hs_free_compile_error")?,
            )
        };
        let count = program.expressions.len();
        ensure!(
            program.flags.len() == count
                && program.bounds.len() == count
                && program.outputs.len() == count,
            "malformed rules program"
        );
        let expressions: Vec<_> = program
            .expressions
            .iter()
            .map(|x| CString::new(x.as_str()))
            .collect::<Result<_, _>>()?;
        let pointers: Vec<_> = expressions.iter().map(|x| x.as_ptr()).collect();
        let ids: Vec<_> = (0..expressions.len() as u32).collect();
        // Extended parameters are owned here for the duration of the call; the pointer
        // array is built only after `extended` is complete so it never reallocates.
        let extended: Vec<_> =
            program.bounds.iter().map(|x| x.map(ExpressionExt::bounded)).collect();
        let ext: Vec<*const ExpressionExt> = extended
            .iter()
            .map(|x| x.as_ref().map_or(ptr::null(), |x| x as *const ExpressionExt))
            .collect();
        let mut db = ptr::null_mut();
        let mut error: *mut CompileError = ptr::null_mut();
        // All arrays have equal length. Compile for exactly the CPU target in the cache key.
        let code = unsafe {
            compile(
                pointers.as_ptr(),
                program.flags.as_ptr(),
                ids.as_ptr(),
                ext.as_ptr(),
                ids.len() as u32,
                4,
                (&self.platform as *const Platform).cast(),
                &mut db,
                &mut error,
            )
        };
        if code != 0 {
            let message = if error.is_null() {
                format!("status {code}")
            } else {
                let message = unsafe { CStr::from_ptr((*error).message) }.to_string_lossy();
                let message = format!("expression {}: {message}", unsafe { (*error).expression });
                unsafe {
                    free_error(error.cast());
                }
                message
            };
            bail!("Vectorscan compilation failed: {message}");
        }
        ensure!(!db.is_null(), "Vectorscan returned a null database");
        Ok(Some(Stream { db, outputs: program.outputs.clone(), owned: true, api: self.clone() }))
    }

    /// Pins both stream images inside one mapping; an empty range is an absent stream.
    #[cfg(all(feature = "yara-rules", unix))]
    pub(super) fn map_image(
        self: &Arc<Self>, mapping: super::mapped::Mapping,
        streams: Streams<(std::ops::Range<usize>, Vec<Option<ContentType>>)>,
    ) -> Result<Arc<Database>> {
        let bytes = mapping.bytes();
        let streams = streams.try_map(|(range, outputs)| {
            let payload = bytes.get(range).context("truncated native image")?;
            self.map_stream(payload, outputs)
        })?;
        let mut database = Database::new(self.clone(), streams)?;
        database.mapping = Some(mapping);
        Ok(Arc::new(database))
    }

    #[cfg(all(feature = "yara-rules", unix))]
    fn map_stream(
        self: &Arc<Self>, payload: &[u8], outputs: Vec<Option<ContentType>>,
    ) -> Result<Option<Stream>> {
        if payload.is_empty() {
            return Ok(None);
        }
        // The pinned Vectorscan database header uses a relative bytecode offset.
        // Before passing mapped bytes to C, bound every region C will checksum/read.
        ensure!(payload.len() >= 104, "truncated native image header");
        let word = |at| u32::from_ne_bytes(payload[at..at + 4].try_into().unwrap()) as usize;
        let length = word(8);
        let bytecode = word(36);
        ensure!(word(0) == 0xdbdbdbdb, "invalid native image magic");
        ensure!(length.checked_add(104) == Some(payload.len()), "invalid native image length");
        ensure!(
            bytecode == 64 && bytecode + length <= payload.len(),
            "invalid native image alignment/offset"
        );
        let db = payload.as_ptr() as *mut c_void;
        ensure!((db as usize).is_multiple_of(64), "unaligned native image");
        type Size = unsafe extern "C" fn(*const c_void, *mut usize) -> c_int;
        let size = unsafe { self.library.get::<Size>(b"hs_database_size")? };
        let mut reported = 0;
        ensure!(
            unsafe { size(db, &mut reported) } == 0 && reported == payload.len(),
            "incompatible native image"
        );
        // Borrowed from the mapping: the image is read-only and never freed.
        Ok(Some(Stream { db, outputs, owned: false, api: self.clone() }))
    }

    /// Deserializes both streams; empty bytes are an absent stream.
    pub(super) fn deserialize(
        self: &Arc<Self>, streams: Streams<(&[u8], Vec<Option<ContentType>>)>,
    ) -> Result<Arc<Database>> {
        #[cfg(feature = "yara-rules")]
        let _startup_span = crate::startup_trace::span("native_deserialize");
        type Deserialize = unsafe extern "C" fn(*const c_char, usize, *mut *mut c_void) -> c_int;
        let deserialize = unsafe { self.library.get::<Deserialize>(b"hs_deserialize_database")? };
        let streams = streams.try_map(|(bytes, outputs)| {
            if bytes.is_empty() {
                return Ok(None);
            }
            let mut db = ptr::null_mut();
            let code = unsafe { deserialize(bytes.as_ptr().cast(), bytes.len(), &mut db) };
            ensure!(
                code == 0 && !db.is_null(),
                "Vectorscan database deserialization failed: {code}"
            );
            Ok(Some(Stream { db, outputs, owned: true, api: self.clone() }))
        })?;
        Ok(Arc::new(Database::new(self.clone(), streams)?))
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        #[cfg(feature = "yara-rules")]
        let _startup_span = crate::startup_trace::span("native_library_dlclose");
        unsafe {
            std::mem::ManuallyDrop::drop(&mut self.library);
        }
    }
}

/// One compiled stream: its native database and the label of each expression ID.
///
/// A stream compiled or deserialized in memory owns its database and releases it when
/// dropped, including when the other stream of the same pack fails to build. A stream
/// pinned inside a read-only image borrows the mapping's bytes instead: the [`Database`]
/// keeps that mapping alive and nothing ever passes the image to `hs_free_database`.
struct Stream {
    db: *mut c_void,
    outputs: Vec<Option<ContentType>>,
    owned: bool,
    // Keep `hs_free_database` loaded for as long as this stream may call it.
    api: Arc<Api>,
}

impl Drop for Stream {
    fn drop(&mut self) {
        if !self.owned {
            return;
        }
        #[cfg(feature = "yara-rules")]
        let _startup_span = crate::startup_trace::span("native_database_free");
        #[cfg(test)]
        FREED.set(FREED.get() + 1);
        unsafe {
            (self.api.free_db)(self.db);
        }
    }
}

#[cfg(test)]
thread_local! {
    /// Native databases released through `hs_free_database` on this thread.
    pub(super) static FREED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The native databases of one pack: one per scan stream, sharing one engine.
///
/// `streams.prefix` scans stream A, `[16-byte size header][original prefix]`, for every
/// input. `streams.facts` scans stream B, `[facts header][zip_names view]`, and only when
/// preprocessing produced something for it. A pack has at least one stream; each present
/// stream has its own expression IDs and output table, and both report into one decision.
pub(super) struct Database {
    streams: Streams<Option<Stream>>,
    // Keep the loaded functions alive until every database and worker has been freed.
    api: Arc<Api>,
    // Owns the image every mapped stream borrows; declared last so it outlives them.
    #[cfg(all(feature = "yara-rules", unix))]
    mapping: Option<super::mapped::Mapping>,
}

// Vectorscan databases are immutable and may be shared; scratch is private to a worker.
unsafe impl Send for Database {}
unsafe impl Sync for Database {}

impl Database {
    fn new(api: Arc<Api>, streams: Streams<Option<Stream>>) -> Result<Self> {
        ensure!(streams.prefix.is_some() || streams.facts.is_some(), "empty native pack");
        Ok(Database {
            streams,
            api,
            #[cfg(all(feature = "yara-rules", unix))]
            mapping: None,
        })
    }

    #[cfg(test)]
    pub(super) fn compile(program: Program) -> Result<Arc<Self>> {
        Api::load()?.compile(program)
    }

    fn present(&self) -> impl Iterator<Item = &Stream> {
        [&self.streams.prefix, &self.streams.facts].into_iter().flatten()
    }

    /// Whether any rule scans the facts stream, and so whether inputs need preprocessing.
    pub(super) fn uses_facts(&self) -> bool {
        self.streams.facts.is_some()
    }

    /// Serializes each stream; an absent stream serializes to nothing.
    pub(super) fn serialize(&self) -> Result<Streams<Vec<u8>>> {
        self.streams
            .as_ref()
            .try_map(|stream| stream.as_ref().map_or(Ok(Vec::new()), |x| self.serialize_stream(x)))
    }

    fn serialize_stream(&self, stream: &Stream) -> Result<Vec<u8>> {
        type Size = unsafe extern "C" fn(*const c_void, *mut usize) -> c_int;
        let size = unsafe { self.api.library.get::<Size>(b"hs_database_size")? };
        let mut allocated = 0;
        ensure!(
            unsafe { size(stream.db, &mut allocated) } == 0
                && allocated <= super::cache::MAX_DATABASE_SIZE,
            "compiled database exceeds cache size limit"
        );
        type Serialize = unsafe extern "C" fn(*const c_void, *mut *mut c_char, *mut usize) -> c_int;
        let serialize = unsafe { self.api.library.get::<Serialize>(b"hs_serialize_database")? };
        let mut bytes = ptr::null_mut();
        let mut length = 0;
        let code = unsafe { serialize(stream.db, &mut bytes, &mut length) };
        ensure!(code == 0 && !bytes.is_null(), "Vectorscan serialization failed: {code}");
        // Vectorscan's default miscellaneous allocator is malloc. This wrapper never changes
        // engine-global allocators; callers sharing libhs must not replace them while in use.
        // The engine and Rust must share the malloc/free allocator domain. In particular,
        // a Windows DLL with a private/static CRT is incompatible; Windows is not qualified.
        struct Serialized(*mut c_char);
        impl Drop for Serialized {
            fn drop(&mut self) {
                unsafe {
                    libc::free(self.0.cast());
                }
            }
        }
        let owned = Serialized(bytes);
        ensure!(
            length <= super::cache::MAX_DATABASE_SIZE,
            "compiled database exceeds cache size limit"
        );
        Ok(unsafe { std::slice::from_raw_parts(owned.0.cast(), length) }.to_vec())
    }

    /// Builds the relocatable image of each stream; an absent stream has none.
    #[cfg(all(feature = "yara-rules", unix))]
    pub(super) fn image(&self) -> Result<Streams<Vec<u8>>> {
        self.serialize()?.try_map(|serialized| {
            if serialized.is_empty() {
                return Ok(Vec::new());
            }
            self.image_of(&serialized)
        })
    }

    #[cfg(all(feature = "yara-rules", unix))]
    fn image_of(&self, serialized: &[u8]) -> Result<Vec<u8>> {
        type Size = unsafe extern "C" fn(*const c_char, usize, *mut usize) -> c_int;
        type At = unsafe extern "C" fn(*const c_char, usize, *mut c_void) -> c_int;
        let size = unsafe { self.api.library.get::<Size>(b"hs_serialized_database_size")? };
        let at = unsafe { self.api.library.get::<At>(b"hs_deserialize_database_at")? };
        let mut length = 0;
        ensure!(
            unsafe { size(serialized.as_ptr().cast(), serialized.len(), &mut length) } == 0
                && length <= super::cache::MAX_DATABASE_SIZE,
            "invalid image size"
        );
        let mut pointer = ptr::null_mut();
        ensure!(
            unsafe { libc::posix_memalign(&mut pointer, 64, length) } == 0,
            "image allocation failed"
        );
        struct Aligned(*mut c_void);
        impl Drop for Aligned {
            fn drop(&mut self) {
                unsafe {
                    libc::free(self.0);
                }
            }
        }
        let aligned = Aligned(pointer);
        ensure!(
            unsafe { at(serialized.as_ptr().cast(), serialized.len(), aligned.0) } == 0,
            "image construction failed"
        );
        Ok(unsafe { std::slice::from_raw_parts(aligned.0.cast(), length) }.to_vec())
    }

    pub(super) fn worker(self: &Arc<Self>) -> Result<Worker> {
        #[cfg(feature = "yara-rules")]
        let _startup_span = crate::startup_trace::span("native_scratch_allocate");

        // One scratch serves both streams: each allocation grows it to fit the new database
        // while keeping it valid for the previous ones.
        let mut scratch = ptr::null_mut();
        for stream in self.present() {
            let code = unsafe { (self.api.alloc)(stream.db, &mut scratch) };
            if code != 0 || scratch.is_null() {
                if !scratch.is_null() {
                    unsafe {
                        (self.api.free_scratch)(scratch);
                    }
                }
                bail!("Vectorscan scratch allocation failed: {code}");
            }
        }
        Ok(Worker { database: self.clone(), scratch })
    }
}

pub(super) struct Worker {
    pub(super) database: Arc<Database>,
    scratch: *mut c_void,
}

impl Drop for Worker {
    fn drop(&mut self) {
        #[cfg(feature = "yara-rules")]
        let _startup_span = crate::startup_trace::span("native_scratch_free");

        unsafe {
            (self.database.api.free_scratch)(self.scratch);
        }
    }
}

struct Matches<'a> {
    outputs: &'a [Option<ContentType>],
    decision: Decision,
}

unsafe extern "C" fn matched(id: u32, _: u64, _: u64, _: u32, context: *mut c_void) -> i32 {
    // Only invoked synchronously by scan with this live stack context. Never allocate or panic.
    let context = unsafe { &mut *context.cast::<Matches<'_>>() };
    let Some(Some(label)) = context.outputs.get(id as usize) else {
        context.decision = Decision::EngineError;
        return 1;
    };
    context.decision = match context.decision {
        Decision::NoMatch => Decision::Match(*label),
        Decision::Match(previous) if previous == *label => Decision::Match(previous),
        _ => Decision::Conflict,
    };
    0
}

impl Worker {
    /// Scans one input. `synthetic` is the preprocessed stream B, which a caller may skip
    /// building when the database has no facts stream.
    pub(super) fn scan(
        &mut self, synthetic: Option<&Synthetic>, prefix: &[u8], original_size: u64,
    ) -> Decision {
        #[cfg(feature = "yara-rules")]
        let _startup_span = crate::startup_trace::span("native_scan");

        if original_size > i64::MAX as u64
            || prefix.is_empty()
            || prefix.len() != original_size.min(super::PREFIX_LIMIT as u64) as usize
        {
            return Decision::InsufficientInput;
        }
        // Both streams report into one decision, so a facts rule and a prefix rule agreeing
        // on a label match, and disagreeing ones conflict, exactly like two prefix rules.
        let mut matches = Matches { outputs: &[], decision: Decision::NoMatch };
        // Stream A: two vectors form one logical input. Original bytes are neither copied nor
        // rewritten. The compiler reserves bytes 0..16 for these refreshed, big-endian values.
        if let Some(stream) = &self.database.streams.prefix {
            let mut header = [0_u8; super::EXTERNAL_BYTES];
            header[..8].copy_from_slice(&original_size.to_be_bytes());
            header[8..].copy_from_slice(&(prefix.len() as u64).to_be_bytes());
            matches.outputs = &stream.outputs;
            if !self.run(stream, [header.as_slice(), prefix], &mut matches) {
                return Decision::EngineError;
            }
        }
        // Stream B: the facts header and the views, only when preprocessing produced any.
        let active = synthetic.filter(|synthetic| synthetic.is_active());
        if let (Some(stream), Some(synthetic)) = (&self.database.streams.facts, active) {
            matches.outputs = &stream.outputs;
            let buffers = [
                synthetic.facts.as_slice(),
                synthetic.names.as_slice(),
                synthetic.first_entry.as_slice(),
            ];
            if !self.run(stream, buffers, &mut matches) {
                return Decision::EngineError;
            }
        }
        matches.decision
    }

    /// Scans `buffers` as one logical stream; false on any engine or callback failure.
    fn run<const N: usize>(
        &self, stream: &Stream, buffers: [&[u8]; N], matches: &mut Matches<'_>,
    ) -> bool {
        let pointers = buffers.map(|x| x.as_ptr().cast());
        let lengths = buffers.map(|x| x.len() as u32);
        let code = unsafe {
            (self.database.api.scan)(
                stream.db,
                pointers.as_ptr(),
                lengths.as_ptr(),
                buffers.len() as u32,
                0,
                self.scratch,
                Some(matched),
                (matches as *mut Matches<'_>).cast(),
            )
        };
        code == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn invalid_output_id_from_native_terminates_scan_and_discards_worker() {
        // Exercise an actual native scan termination through an invalid output ID. A valid
        // match in the same scan must never escape after any callback or engine failure.
        let mut database = Database::compile(Program {
            prefix: Expressions {
                expressions: vec!["^.{16}A".into(), "^.{16}A".into()],
                flags: vec![8, 8],
                bounds: vec![None, None],
                outputs: vec![Some(ContentType::Png), None],
            },
            facts: Expressions::default(),
        })
        .unwrap();
        assert!(database.streams.facts.is_none());
        let synthetic = super::super::preprocess::prepare(&super::super::preprocess::Blocks {
            prefix: b"A",
            size: 1,
            tail: None,
        });
        assert_eq!(
            super::super::engine::scan(&database, Some(&synthetic), b"A", 1),
            Decision::EngineError
        );
        assert_eq!(Arc::strong_count(&database), 1, "failed worker must be discarded");
        let stream = Arc::get_mut(&mut database).unwrap().streams.prefix.as_mut().unwrap();
        stream.outputs[1] = Some(ContentType::Png);
        assert_eq!(
            super::super::engine::scan(&database, Some(&synthetic), b"A", 1),
            Decision::Match(ContentType::Png)
        );
        assert_eq!(Arc::strong_count(&database), 2, "successful worker should remain cached");
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn failed_facts_stream_frees_the_compiled_prefix_stream() {
        let valid = || Expressions {
            expressions: vec!["^.{16}A".into()],
            flags: vec![8],
            bounds: vec![None],
            outputs: vec![Some(ContentType::Png)],
        };
        let invalid = Expressions {
            expressions: vec!["(".into()],
            flags: vec![8],
            bounds: vec![None],
            outputs: vec![Some(ContentType::Png)],
        };
        let before = FREED.get();
        let error = match Database::compile(Program { prefix: valid(), facts: invalid }) {
            Ok(_) => panic!("an invalid facts expression compiled"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("Vectorscan compilation failed"), "{error}");
        assert_eq!(FREED.get(), before + 1, "the prefix database leaked when the facts failed");
        // A complete database releases each of its streams exactly once.
        let database = Database::compile(Program { prefix: valid(), facts: valid() }).unwrap();
        let before = FREED.get();
        drop(database);
        assert_eq!(FREED.get(), before + 2);
    }
}

#[cfg(all(test, windows))]
mod loading_tests {
    use super::*;

    #[test]
    fn current_directory_dll_is_not_loaded_implicitly() {
        const CHILD: &str = "MAGIKA_TEST_DLL_DIRECTORY";
        const NAME: &str = "magika_untrusted_probe.dll";
        if let Some(directory) = std::env::var_os(CHILD) {
            let path = std::path::PathBuf::from(directory);
            assert!(load_library(std::ffi::OsStr::new(NAME)).is_err(), "loaded DLL from CWD");
            // The fixture is a real, loadable DLL; an invalid PE file cannot make this pass.
            assert!(load_library(path.join(NAME).as_os_str()).is_ok());
            std::fs::write(path.join("checked"), b"checked implicit and explicit loads").unwrap();
            return;
        }
        let temp = tempfile::tempdir().unwrap();
        let system = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap());
        std::fs::copy(system.join("System32/version.dll"), temp.path().join(NAME)).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "rules::native::loading_tests::current_directory_dll_is_not_loaded_implicitly",
            ])
            .env(CHILD, temp.path())
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            std::fs::read(temp.path().join("checked")).unwrap(),
            b"checked implicit and explicit loads"
        );
    }
}
