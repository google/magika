// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Small ownership wrapper around the Vectorscan C API. Conditions execute in Vectorscan.

use std::ffi::{c_char, c_int, c_uint, c_void, CStr, CString};
use std::ptr;
use std::sync::Arc;

use anyhow::{bail, ensure, Context, Result};
use libloading::Library;

use super::compiler::Program;
use super::engine::Decision;
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
type Compile = unsafe extern "C" fn(
    *const *const c_char,
    *const c_uint,
    *const c_uint,
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
    library: Library,
}

fn load_library(path: &std::ffi::OsStr) -> Result<Library> {
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
                library,
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

    pub(super) fn compile(self: &Arc<Self>, program: Program) -> Result<Arc<Database>> {
        let (compile, free_error) = unsafe {
            (
                *self.library.get::<Compile>(b"hs_compile_multi")?,
                *self.library.get::<Free>(b"hs_free_compile_error")?,
            )
        };
        let expressions: Vec<_> = program
            .expressions
            .iter()
            .map(|x| CString::new(x.as_str()))
            .collect::<Result<_, _>>()?;
        let pointers: Vec<_> = expressions.iter().map(|x| x.as_ptr()).collect();
        let ids: Vec<_> = (0..expressions.len() as u32).collect();
        let mut db = ptr::null_mut();
        let mut error: *mut CompileError = ptr::null_mut();
        // All arrays have equal length. Compile for exactly the CPU target in the cache key.
        let code = unsafe {
            compile(
                pointers.as_ptr(),
                program.flags.as_ptr(),
                ids.as_ptr(),
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
        Ok(Arc::new(Database { db, api: self.clone(), outputs: program.outputs }))
    }

    pub(super) fn deserialize(
        self: &Arc<Self>, bytes: &[u8], outputs: Vec<Option<ContentType>>,
    ) -> Result<Arc<Database>> {
        type Deserialize = unsafe extern "C" fn(*const c_char, usize, *mut *mut c_void) -> c_int;
        let deserialize = unsafe { self.library.get::<Deserialize>(b"hs_deserialize_database")? };
        let mut db = ptr::null_mut();
        let code = unsafe { deserialize(bytes.as_ptr().cast(), bytes.len(), &mut db) };
        ensure!(code == 0 && !db.is_null(), "Vectorscan database deserialization failed: {code}");
        Ok(Arc::new(Database { db, api: self.clone(), outputs }))
    }
}

pub(super) struct Database {
    db: *mut c_void,
    outputs: Vec<Option<ContentType>>,
    // Keep the loaded functions alive until every database and worker has been freed.
    api: Arc<Api>,
}

// Vectorscan databases are immutable and may be shared; scratch is private to a worker.
unsafe impl Send for Database {}
unsafe impl Sync for Database {}

impl Database {
    #[cfg(test)]
    pub(super) fn compile(program: Program) -> Result<Arc<Self>> {
        Api::load()?.compile(program)
    }

    pub(super) fn serialize(&self) -> Result<Vec<u8>> {
        type Size = unsafe extern "C" fn(*const c_void, *mut usize) -> c_int;
        let size = unsafe { self.api.library.get::<Size>(b"hs_database_size")? };
        let mut allocated = 0;
        ensure!(
            unsafe { size(self.db, &mut allocated) } == 0
                && allocated <= super::cache::MAX_DATABASE_SIZE,
            "compiled database exceeds cache size limit"
        );
        type Serialize = unsafe extern "C" fn(*const c_void, *mut *mut c_char, *mut usize) -> c_int;
        let serialize = unsafe { self.api.library.get::<Serialize>(b"hs_serialize_database")? };
        let mut bytes = ptr::null_mut();
        let mut length = 0;
        let code = unsafe { serialize(self.db, &mut bytes, &mut length) };
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

    pub(super) fn worker(self: &Arc<Self>) -> Result<Worker> {
        let mut scratch = ptr::null_mut();
        let code = unsafe { (self.api.alloc)(self.db, &mut scratch) };
        ensure!(code == 0 && !scratch.is_null(), "Vectorscan scratch allocation failed: {code}");
        Ok(Worker { database: self.clone(), scratch })
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        unsafe {
            (self.api.free_db)(self.db);
        }
    }
}

pub(super) struct Worker {
    pub(super) database: Arc<Database>,
    scratch: *mut c_void,
}

impl Drop for Worker {
    fn drop(&mut self) {
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
    pub(super) fn scan(&mut self, prefix: &[u8], original_size: u64) -> Decision {
        if original_size > i64::MAX as u64
            || prefix.is_empty()
            || prefix.len() != original_size.min(super::PREFIX_LIMIT as u64) as usize
        {
            return Decision::InsufficientInput;
        }
        // Two vectors form one logical input. Original bytes are neither copied nor rewritten.
        // The compiler reserves bytes 0..16 for these refreshed, big-endian external values.
        let mut header = [0_u8; super::EXTERNAL_BYTES];
        header[..8].copy_from_slice(&original_size.to_be_bytes());
        header[8..].copy_from_slice(&(prefix.len() as u64).to_be_bytes());
        let buffers = [header.as_ptr().cast(), prefix.as_ptr().cast()];
        let lengths = [header.len() as u32, prefix.len() as u32];
        let mut matches = Matches { outputs: &self.database.outputs, decision: Decision::NoMatch };
        let code = unsafe {
            (self.database.api.scan)(
                self.database.db,
                buffers.as_ptr(),
                lengths.as_ptr(),
                2,
                0,
                self.scratch,
                Some(matched),
                (&mut matches as *mut Matches<'_>).cast(),
            )
        };
        if code == 0 {
            matches.decision
        } else {
            Decision::EngineError
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn terminated_scan_discards_matches_and_releases_worker() {
        // Exercise an actual native scan termination through an invalid output ID. A valid
        // match in the same scan must never escape after any callback or engine failure.
        let mut database = Database::compile(Program {
            expressions: vec!["^.{16}A".into(), "^.{16}A".into()],
            flags: vec![8, 8],
            outputs: vec![Some(ContentType::Png), None],
        })
        .unwrap();
        assert_eq!(super::super::engine::scan(&database, b"A", 1), Decision::EngineError);
        assert_eq!(Arc::strong_count(&database), 1, "failed worker must be discarded");
        Arc::get_mut(&mut database).unwrap().outputs[1] = Some(ContentType::Png);
        assert_eq!(
            super::super::engine::scan(&database, b"A", 1),
            Decision::Match(ContentType::Png)
        );
        assert_eq!(Arc::strong_count(&database), 2, "successful worker should remain cached");
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
