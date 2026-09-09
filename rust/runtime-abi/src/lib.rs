//! Internal, versioned C boundary for the deferred inference backends.
//!
//! Callers supply valid, aligned, non-overlapping buffers with the stated lengths.
//! Handles must come from this API; runtime handles allow shared access, session
//! handles require exclusive access and stay on their creation thread. Each allocation is freed by its own library.
//! The library and runtime must outlive their sessions. Errors are UTF-8 bytes,
//! zero terminated when the error buffer is nonempty. No Rust types cross the ABI.
use std::ffi::c_void;

pub const ABI_VERSION: u32 = 1;
pub const FEATURE_SIZE: usize = 2048;
pub const NUM_LABELS: usize = 214;
pub const CPU: u32 = 1;
pub const METAL: u32 = 2;
pub const CUDA: u32 = 3;

/// Changes to layout or semantics require a new entry-point symbol and version.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Api {
    pub version: u32,
    pub size: u32,
    pub backend: u32,
    pub feature_size: usize,
    pub num_labels: usize,
    /// A zero maximum requests the original Runtime::new batch classes.
    pub create: unsafe extern "C" fn(usize, *mut *mut c_void, *mut u8, usize) -> i32,
    pub destroy: unsafe extern "C" fn(*mut c_void),
    pub session: unsafe extern "C" fn(*const c_void, *mut *mut c_void, *mut u8, usize) -> i32,
    pub destroy_session: unsafe extern "C" fn(*mut c_void),
    pub run: unsafe extern "C" fn(
        *mut c_void,
        *const i32,
        usize,
        usize,
        *mut f32,
        usize,
        *mut u8,
        usize,
    ) -> i32,
}
