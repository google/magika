// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! CPU or GPU implementation of Magika's internal C runtime ABI.
use anyhow::{Result, ensure};
use magika_runtime_abi as abi;
use magika_tract_runtime::{BackendRequest, Runtime, Session};
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
};

#[cfg(all(feature = "metal", feature = "cuda"))]
compile_error!("build one GPU implementation per backend library");
#[cfg(all(feature = "metal", not(target_os = "macos")))]
compile_error!("Metal requires macOS");
#[cfg(all(feature = "cuda", target_os = "macos"))]
compile_error!("CUDA backend is not supported on macOS");

const BACKEND: u32 = if cfg!(feature = "metal") {
    abi::METAL
} else if cfg!(feature = "cuda") {
    abi::CUDA
} else {
    abi::CPU
};

const _: () = {
    assert!(abi::FEATURE_SIZE == magika_tract_runtime::FEATURE_SIZE);
    assert!(abi::NUM_LABELS == magika_tract_runtime::NUM_LABELS);
};

// These bounds justify the host's opaque-handle Send/Sync implementations.
fn assert_traits() {
    fn shared<T: Send + Sync>() {}
    shared::<Runtime>();
}

unsafe fn guarded(error: *mut u8, capacity: usize, f: impl FnOnce() -> Result<()>) -> i32 {
    let result = catch_unwind(AssertUnwindSafe(f));
    let message = match result {
        Ok(Ok(())) => return 0,
        Ok(Err(e)) => format!("{e:#}"),
        Err(_) => "inference backend panicked".to_owned(),
    };
    if capacity > 0 && !error.is_null() {
        let n = message.len().min(capacity - 1);
        unsafe {
            ptr::copy_nonoverlapping(message.as_ptr(), error, n);
            *error.add(n) = 0;
        }
    }
    1
}

unsafe extern "C" fn create(
    maximum: usize, out: *mut *mut c_void, error: *mut u8, capacity: usize,
) -> i32 {
    unsafe {
        guarded(error, capacity, || {
            assert_traits();
            ensure!(!out.is_null(), "null runtime output");
            *out = ptr::null_mut();
            let request =
                if BACKEND == abi::CPU { BackendRequest::Cpu } else { BackendRequest::Gpu };
            let runtime = if maximum == 0 {
                Runtime::new(request)?
            } else {
                Runtime::with_max_batch(request, maximum)?
            };
            *out = Box::into_raw(Box::new(runtime)).cast();
            Ok(())
        })
    }
}

unsafe extern "C" fn destroy(handle: *mut c_void) {
    // Never unwind across C, including from a destructor. A destructor panic
    // cannot be recovered safely; keep Rust's normal abort-on-FFI-unwind behavior.
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle.cast::<Runtime>())) };
    }
}

unsafe extern "C" fn session(
    handle: *const c_void, out: *mut *mut c_void, error: *mut u8, capacity: usize,
) -> i32 {
    unsafe {
        guarded(error, capacity, || {
            ensure!(!handle.is_null() && !out.is_null(), "null session argument");
            *out = ptr::null_mut();
            *out = Box::into_raw(Box::new((&*handle.cast::<Runtime>()).session()?)).cast();
            Ok(())
        })
    }
}

unsafe extern "C" fn destroy_session(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle.cast::<Session>())) };
    }
}

unsafe extern "C" fn run(
    handle: *mut c_void, input: *const i32, length: usize, batch: usize, output: *mut f32,
    output_length: usize, error: *mut u8, capacity: usize,
) -> i32 {
    unsafe {
        guarded(error, capacity, || {
            ensure!(
                batch > 0 && batch.checked_mul(abi::FEATURE_SIZE) == Some(length),
                "invalid input shape"
            );
            ensure!(
                batch.checked_mul(abi::NUM_LABELS) == Some(output_length),
                "invalid output shape"
            );
            ensure!(
                length <= isize::MAX as usize / size_of::<i32>()
                    && output_length <= isize::MAX as usize / size_of::<f32>(),
                "buffer too large"
            );
            ensure!(
                !handle.is_null() && !input.is_null() && !output.is_null(),
                "null inference argument"
            );
            let values = (&mut *handle.cast::<Session>())
                .run(std::slice::from_raw_parts(input, length), batch)?;
            ensure!(values.len() == output_length, "unexpected backend output shape");
            ptr::copy_nonoverlapping(values.as_ptr(), output, output_length);
            Ok(())
        })
    }
}

static API: abi::Api = abi::Api {
    version: abi::ABI_VERSION,
    size: size_of::<abi::Api>() as u32,
    backend: BACKEND,
    feature_size: abi::FEATURE_SIZE,
    num_labels: abi::NUM_LABELS,
    create,
    destroy,
    session,
    destroy_session,
    run,
};

/// Only exported interface; valid for the lifetime of this loaded library.
#[unsafe(no_mangle)]
pub extern "C" fn magika_runtime_v1() -> *const abi::Api {
    &API
}
