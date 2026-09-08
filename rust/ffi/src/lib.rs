// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! C API bindings for Magika.

use std::ffi::c_char;

mod content;

/// Status and error codes returned by Magika C API functions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum MagikaStatus {
    /// Operation completed successfully.
    Ok = 0,
    /// An argument was invalid or a required pointer was null.
    InvalidArgument = -1,
    /// An I/O error occurred while reading the file.
    IoError = -2,
    /// An error occurred during neural network inference.
    InferenceError = -3,
    /// An internal panic was caught across the FFI boundary.
    Panic = -4,
}

/// The kind of identified file type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum MagikaFileTypeKind {
    /// The file is a directory.
    Directory = 0,
    /// The file is a symbolic link.
    Symlink = 1,
    /// The file is a regular file and was identified using deep learning inference.
    Inferred = 2,
    /// The file is a regular file and was identified using rules.
    Ruled = 3,
}

/// Reason why an inferred content type was overwritten.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum MagikaOverwriteReason {
    /// The inferred content type was not overwritten.
    None = 0,
    /// The model score was below the confidence threshold for the inferred type.
    LowConfidence = 1,
    /// The inferred type was mapped to another canonical type.
    OverwriteMap = 2,
}

/// Content type information.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct MagikaTypeInfo {
    /// The unique label identifying this file type (null-terminated UTF-8 string).
    pub label: *const c_char,
    /// The MIME type of the file type (null-terminated UTF-8 string).
    pub mime_type: *const c_char,
    /// The group of the file type (null-terminated UTF-8 string).
    pub group: *const c_char,
    /// The human-readable description of the file type (null-terminated UTF-8 string).
    pub description: *const c_char,
    /// Null-terminated array of null-terminated extension strings.
    pub extensions: *const *const c_char,
    /// Whether the file type is text.
    pub is_text: bool,
}

unsafe impl Sync for MagikaTypeInfo {}
unsafe impl Send for MagikaTypeInfo {}

/// Result of a file identification.
///
/// All pointers are to static memory, so no cleanup function is needed.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct MagikaResult {
    /// The kind of identified file type.
    pub kind: MagikaFileTypeKind,
    /// Resolved content type information (never null, points to static storage).
    pub info: *const MagikaTypeInfo,
    /// Confidence score between 0.0 and 1.0 (1.0 for directory, symlink, or ruled).
    pub score: f32,
    /// Raw model output before overwrite rules were applied (null if not inferred).
    pub inferred_info: *const MagikaTypeInfo,
    /// Reason why the inferred type was overwritten (None if not overwritten).
    pub overwrite_reason: MagikaOverwriteReason,
}

unsafe impl Sync for MagikaResult {}
unsafe impl Send for MagikaResult {}

/// Hardware backend to use for neural network inference.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[repr(C)]
pub enum MagikaBackend {
    /// Automatically select the best available backend.
    #[default]
    Auto = 0,
    /// Use CPU inference.
    Cpu = 1,
    /// Use GPU inference.
    Gpu = 2,
}

/// Options for configuring a Magika runtime.
#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct MagikaRuntimeOptions {
    /// The backend to use for inference.
    pub backend: MagikaBackend,
    /// The maximum batch size to optimize for (0 for runtime default).
    pub max_batch: usize,
}

/// Shared Magika inference runtime (thread-safe).
pub struct MagikaRuntime {
    inner: magika::Runtime,
}

/// Magika identification session (not thread-safe, one per thread).
pub struct MagikaSession {
    inner: magika::Session,
}

/// Features extracted from a file or buffer for neural network inference.
pub struct MagikaFeatures {
    inner: magika::Features,
}

unsafe fn catch_unwind<T>(
    f: impl FnOnce() -> anyhow::Result<T>, g: impl FnOnce(T),
) -> MagikaStatus {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(Ok(x)) => {
            g(x);
            MagikaStatus::Ok
        }
        Ok(Err(err)) => {
            if err.downcast_ref::<std::io::Error>().is_some()
                || err.root_cause().is::<std::io::Error>()
            {
                MagikaStatus::IoError
            } else {
                MagikaStatus::InferenceError
            }
        }
        Err(_) => MagikaStatus::Panic,
    }
}

/// Initializes the shared Magika inference runtime (loads models and plans).
///
/// If `options` is NULL, the default configuration is used.
///
/// # Safety
///
/// `out_runtime` must point to a valid, writable pointer to `MagikaRuntime`.
#[no_mangle]
pub unsafe extern "C" fn magika_runtime_new(
    options: *const MagikaRuntimeOptions, out_runtime: *mut *mut MagikaRuntime,
) -> MagikaStatus {
    if out_runtime.is_null() {
        return MagikaStatus::InvalidArgument;
    }
    *out_runtime = std::ptr::null_mut();
    catch_unwind(
        || {
            let mut builder = magika::Runtime::builder();
            if !options.is_null() {
                let opts = &*options;
                match opts.backend {
                    MagikaBackend::Auto => (),
                    MagikaBackend::Cpu => {
                        builder = builder.with_backend(magika::Backend::Cpu);
                    }
                    MagikaBackend::Gpu => {
                        builder = builder.with_backend(magika::Backend::Gpu);
                    }
                }
                if opts.max_batch > 0 {
                    builder = builder.with_max_batch(opts.max_batch);
                }
            }
            builder.build()
        },
        |runtime| *out_runtime = Box::into_raw(Box::new(MagikaRuntime { inner: runtime })),
    )
}

/// Frees a Magika runtime. Passing NULL is a safe no-op.
///
/// # Safety
///
/// If non-null, `runtime` must have been returned by `magika_runtime_new` and not previously freed.
#[no_mangle]
pub unsafe extern "C" fn magika_runtime_free(runtime: *mut MagikaRuntime) {
    if !runtime.is_null() {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            drop(Box::from_raw(runtime));
        }));
    }
}

/// Spawns a new identification session from the runtime.
///
/// # Safety
///
/// - `runtime` must point to a valid `MagikaRuntime`.
/// - `out_session` must point to a valid, writable pointer to `MagikaSession`.
#[no_mangle]
pub unsafe extern "C" fn magika_session_new(
    runtime: *const MagikaRuntime, out_session: *mut *mut MagikaSession,
) -> MagikaStatus {
    if runtime.is_null() || out_session.is_null() {
        return MagikaStatus::InvalidArgument;
    }
    *out_session = std::ptr::null_mut();
    catch_unwind(
        || (*runtime).inner.session(),
        |session| *out_session = Box::into_raw(Box::new(MagikaSession { inner: session })),
    )
}

/// Frees a Magika session. Passing NULL is a safe no-op.
///
/// # Safety
///
/// If non-null, `session` must have been returned by `magika_session_new` and not previously freed.
#[no_mangle]
pub unsafe extern "C" fn magika_session_free(session: *mut MagikaSession) {
    if !session.is_null() {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            drop(Box::from_raw(session));
        }));
    }
}

/// Frees Magika features. Passing NULL is a safe no-op.
///
/// # Safety
///
/// If non-null, `features` must have been returned by a Magika features extraction
/// function and not previously freed.
#[no_mangle]
pub unsafe extern "C" fn magika_features_free(features: *mut MagikaFeatures) {
    if !features.is_null() {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            drop(Box::from_raw(features));
        }));
    }
}

fn file_type_to_c(file_type: &magika::FileType) -> MagikaResult {
    match file_type {
        magika::FileType::Directory => MagikaResult {
            kind: MagikaFileTypeKind::Directory,
            info: &content::DIRECTORY,
            score: 1.0,
            inferred_info: std::ptr::null(),
            overwrite_reason: MagikaOverwriteReason::None,
        },
        magika::FileType::Symlink => MagikaResult {
            kind: MagikaFileTypeKind::Symlink,
            info: &content::SYMLINK,
            score: 1.0,
            inferred_info: std::ptr::null(),
            overwrite_reason: MagikaOverwriteReason::None,
        },
        magika::FileType::Ruled(ct) => MagikaResult {
            kind: MagikaFileTypeKind::Ruled,
            info: content::content_type_info(*ct),
            score: 1.0,
            inferred_info: std::ptr::null(),
            overwrite_reason: MagikaOverwriteReason::None,
        },
        magika::FileType::Inferred(inferred) => {
            let info = content::content_type_info(inferred.content_type());
            let inferred_info = content::content_type_info(inferred.inferred_type);
            let overwrite_reason = match inferred.content_type {
                None => MagikaOverwriteReason::None,
                Some((_, magika::OverwriteReason::LowConfidence)) => {
                    MagikaOverwriteReason::LowConfidence
                }
                Some((_, magika::OverwriteReason::OverwriteMap)) => {
                    MagikaOverwriteReason::OverwriteMap
                }
            };
            MagikaResult {
                kind: MagikaFileTypeKind::Inferred,
                info,
                score: inferred.score,
                inferred_info,
                overwrite_reason,
            }
        }
    }
}

unsafe fn identify(
    session: *mut MagikaSession, out_result: *mut MagikaResult,
    f: impl FnOnce(&mut magika::Session) -> anyhow::Result<magika::FileType>,
) -> MagikaStatus {
    catch_unwind(|| f(&mut (*session).inner), |file_type| *out_result = file_type_to_c(&file_type))
}

unsafe fn parse_path<'a>(path: *const c_char) -> Option<&'a std::path::Path> {
    if path.is_null() {
        return None;
    }
    let c_str = std::ffi::CStr::from_ptr(path);
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        Some(std::path::Path::new(std::ffi::OsStr::from_bytes(c_str.to_bytes())))
    }
    #[cfg(not(unix))]
    {
        c_str.to_str().ok().map(std::path::Path::new)
    }
}

/// Identifies the content type of a file on disk.
///
/// # Safety
///
/// - `session` must point to a valid `MagikaSession`.
/// - `path` must point to a null-terminated C string.
/// - `out_result` must point to a valid, writable `MagikaResult` struct.
#[no_mangle]
pub unsafe extern "C" fn magika_identify_file(
    session: *mut MagikaSession, path: *const c_char, out_result: *mut MagikaResult,
) -> MagikaStatus {
    if session.is_null() || out_result.is_null() {
        return MagikaStatus::InvalidArgument;
    }
    let Some(path_ref) = parse_path(path) else { return MagikaStatus::InvalidArgument };
    identify(session, out_result, |sess| sess.identify_file(path_ref))
}

/// Identifies the content type of an in-memory buffer.
///
/// # Safety
///
/// - `session` must point to a valid `MagikaSession`.
/// - `data` must point to at least `len` readable bytes (may be NULL only if `len == 0`).
/// - `out_result` must point to a valid, writable `MagikaResult` struct.
#[no_mangle]
pub unsafe extern "C" fn magika_identify_content(
    session: *mut MagikaSession, data: *const u8, len: usize, out_result: *mut MagikaResult,
) -> MagikaStatus {
    if session.is_null() || out_result.is_null() || (data.is_null() && len > 0) {
        return MagikaStatus::InvalidArgument;
    }
    let bytes = if len == 0 { &[][..] } else { std::slice::from_raw_parts(data, len) };
    identify(session, out_result, |sess| sess.identify_content(bytes))
}

unsafe fn extract_features(
    out_features: *mut *mut MagikaFeatures, out_result: *mut MagikaResult,
    f: impl FnOnce() -> anyhow::Result<magika::FeaturesOrRuled>,
) -> MagikaStatus {
    catch_unwind(f, |result| match result {
        magika::FeaturesOrRuled::Features(features) => {
            *out_features = Box::into_raw(Box::new(MagikaFeatures { inner: features }));
        }
        magika::FeaturesOrRuled::Ruled(content_type) => {
            if !out_result.is_null() {
                *out_result = file_type_to_c(&magika::FileType::Ruled(content_type));
            }
        }
    })
}

/// Extracts features from a file on disk for neural network inference.
///
/// If the file does not require neural network inference (for example, if it is empty
/// or identified by rules):
/// - `*out_features` is set to NULL.
/// - If `out_result` is non-null, `*out_result` is populated with the identification result.
///
/// If the file requires neural network inference:
/// - `*out_features` is set to a newly allocated `MagikaFeatures` (which must be freed
///   with `magika_features_free`).
/// - `out_result` is left unchanged.
///
/// # Safety
///
/// - `path` must point to a null-terminated C string.
/// - `out_features` must point to a valid, writable pointer to `MagikaFeatures`.
/// - `out_result` may be NULL, or must point to a valid, writable `MagikaResult` struct.
#[no_mangle]
pub unsafe extern "C" fn magika_features_extract_file(
    path: *const c_char, out_features: *mut *mut MagikaFeatures, out_result: *mut MagikaResult,
) -> MagikaStatus {
    if out_features.is_null() {
        return MagikaStatus::InvalidArgument;
    }
    *out_features = std::ptr::null_mut();
    let Some(path_ref) = parse_path(path) else { return MagikaStatus::InvalidArgument };
    extract_features(out_features, out_result, || {
        let file = std::fs::File::open(path_ref)?;
        magika::FeaturesOrRuled::extract(file)
    })
}

/// Extracts features from an in-memory buffer for neural network inference.
///
/// If the buffer does not require neural network inference (for example, if it is empty
/// or identified by rules):
/// - `*out_features` is set to NULL.
/// - If `out_result` is non-null, `*out_result` is populated with the identification result.
///
/// If the buffer requires neural network inference:
/// - `*out_features` is set to a newly allocated `MagikaFeatures` (which must be freed
///   with `magika_features_free`).
/// - `out_result` is left unchanged.
///
/// # Safety
///
/// - `data` must point to at least `len` readable bytes (may be NULL only if `len == 0`).
/// - `out_features` must point to a valid, writable pointer to `MagikaFeatures`.
/// - `out_result` may be NULL, or must point to a valid, writable `MagikaResult` struct.
#[no_mangle]
pub unsafe extern "C" fn magika_features_extract_content(
    data: *const u8, len: usize, out_features: *mut *mut MagikaFeatures,
    out_result: *mut MagikaResult,
) -> MagikaStatus {
    if out_features.is_null() || (data.is_null() && len > 0) {
        return MagikaStatus::InvalidArgument;
    }
    *out_features = std::ptr::null_mut();
    let bytes = if len == 0 { &[][..] } else { std::slice::from_raw_parts(data, len) };
    extract_features(out_features, out_result, || magika::FeaturesOrRuled::extract(bytes))
}

/// Identifies the content type of a file from its extracted features.
///
/// # Safety
///
/// - `session` must point to a valid `MagikaSession`.
/// - `features` must point to a valid `MagikaFeatures`.
/// - `out_result` must point to a valid, writable `MagikaResult` struct.
#[no_mangle]
pub unsafe extern "C" fn magika_identify_features(
    session: *mut MagikaSession, features: *const MagikaFeatures, out_result: *mut MagikaResult,
) -> MagikaStatus {
    if session.is_null() || features.is_null() || out_result.is_null() {
        return MagikaStatus::InvalidArgument;
    }
    identify(session, out_result, |sess| sess.identify_features(&(*features).inner))
}

/// Identifies the content types of multiple files from their extracted features in a batch.
///
/// # Safety
///
/// Unless `count` is 0:
/// - `session` must point to a valid `MagikaSession`.
/// - `features` must point to an array of at least `count` valid, non-null `MagikaFeatures`
///   pointers.
/// - `out_results` must point to an array of at least `count` writable `MagikaResult` structs.
#[no_mangle]
pub unsafe extern "C" fn magika_identify_features_batch(
    session: *mut MagikaSession, features: *const *const MagikaFeatures, count: usize,
    out_results: *mut MagikaResult,
) -> MagikaStatus {
    if count == 0 {
        return MagikaStatus::Ok;
    }
    if session.is_null() || features.is_null() || out_results.is_null() {
        return MagikaStatus::InvalidArgument;
    }
    for i in 0..count {
        if (*features.add(i)).is_null() {
            return MagikaStatus::InvalidArgument;
        }
    }
    catch_unwind(
        || {
            let feats = (0..count).map(|i| &(*(*features.add(i))).inner);
            (*session).inner.identify_features_batch(feats)
        },
        |file_types| {
            for (i, file_type) in file_types.into_iter().enumerate() {
                *out_results.add(i) = file_type_to_c(&file_type);
            }
        },
    )
}
