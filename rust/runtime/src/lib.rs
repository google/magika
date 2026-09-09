//! Deferred runtime loading for CPU and GPU inference.
use anyhow::{Context, Result, ensure};
use libloading::Library;
use magika_runtime_abi as abi;
use std::{
    ffi::c_void,
    path::{Path, PathBuf},
    ptr::NonNull,
    sync::{Arc, Mutex, OnceLock},
};

pub use abi::{FEATURE_SIZE, NUM_LABELS};
pub const BATCH_CLASSES: [usize; 6] = [1, 4, 8, 16, 32, 64];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BackendRequest {
    #[default]
    Auto,
    Cpu,
    Gpu,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Backend {
    Cpu,
    Gpu,
}
#[derive(Clone, Copy, Debug)]
pub struct BackendInfo {
    backend: Backend,
    implementation: &'static str,
}
impl BackendInfo {
    pub fn backend(self) -> Backend {
        self.backend
    }
    pub fn implementation(self) -> &'static str {
        self.implementation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Cpu,
    Metal,
    Cuda,
}
impl Kind {
    fn code(self) -> u32 {
        match self {
            Self::Cpu => abi::CPU,
            Self::Metal => abi::METAL,
            Self::Cuda => abi::CUDA,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Metal => "metal",
            Self::Cuda => "cuda",
        }
    }
    fn info(self) -> BackendInfo {
        BackendInfo {
            backend: if self == Self::Cpu { Backend::Cpu } else { Backend::Gpu },
            implementation: match self {
                Self::Cpu => "tract-cpu",
                Self::Metal => "tract-metal",
                Self::Cuda => "tract-cuda",
            },
        }
    }
}
fn gpu_kind() -> Kind {
    if cfg!(target_os = "macos") { Kind::Metal } else { Kind::Cuda }
}

fn select<T>(request: BackendRequest, mut load: impl FnMut(Kind) -> Result<T>) -> Result<T> {
    match request {
        BackendRequest::Cpu => load(Kind::Cpu),
        BackendRequest::Gpu => load(gpu_kind()),
        BackendRequest::Auto => load(gpu_kind()).or_else(|_| load(Kind::Cpu)),
    }
}

fn validate(
    (version, size, backend, features, labels): (u32, u32, u32, usize, usize), kind: Kind,
) -> Result<()> {
    ensure!(
        version == abi::ABI_VERSION && size as usize == size_of::<abi::Api>(),
        "incompatible inference backend ABI"
    );
    ensure!(backend == kind.code(), "incorrect inference backend kind");
    ensure!(
        features == FEATURE_SIZE && labels == NUM_LABELS,
        "incompatible inference model dimensions"
    );
    Ok(())
}

struct Loaded {
    api: abi::Api,
    _library: Library,
}
impl Loaded {
    fn open(path: &Path, kind: Kind) -> Result<Self> {
        // The backend is executable code, like the executable itself. Only the
        // explicit deployment directory is searched, never cwd or the OS path.
        let library =
            unsafe { Library::new(path) }.with_context(|| format!("loading {}", path.display()))?;
        unsafe {
            let entry =
                library.get::<unsafe extern "C" fn() -> *const abi::Api>(b"magika_runtime_v1\0")?;
            let pointer = NonNull::new(entry().cast_mut()).context("null inference backend API")?;
            // Read the stable two-u32 prefix before reading the complete table.
            let prefix = pointer.as_ptr().cast::<u32>();
            ensure!(
                *prefix == abi::ABI_VERSION && *prefix.add(1) as usize == size_of::<abi::Api>(),
                "incompatible inference backend ABI"
            );
            let api = *pointer.as_ptr();
            validate((api.version, api.size, api.backend, api.feature_size, api.num_labels), kind)?;
            Ok(Self { api, _library: library })
        }
    }
}

fn backend_path(kind: Kind) -> Result<PathBuf> {
    // This override names trusted, caller-installed native code, not model data.
    let directory = if let Some(path) = std::env::var_os("MAGIKA_RUNTIME_DIR") {
        PathBuf::from(path)
    } else {
        std::env::current_exe()?.parent().context("executable has no parent")?.join("lib")
    };
    let directory = directory.canonicalize().context("locating inference backend directory")?;
    Ok(directory.join(format!(
        "{}magika_runtime_{}{}",
        std::env::consts::DLL_PREFIX,
        kind.name(),
        std::env::consts::DLL_SUFFIX
    )))
}

fn load(kind: Kind) -> Result<Arc<Loaded>> {
    // Pin successful native loads for the process lifetime: Metal/ObjC may keep
    // callbacks after a session is dropped. Failed loads remain retryable.
    // Independent locks mean CPU loading never waits for a GPU library load.
    static CPU: OnceLock<Mutex<Option<Arc<Loaded>>>> = OnceLock::new();
    static GPU: OnceLock<Mutex<Option<Arc<Loaded>>>> = OnceLock::new();
    let cache = if kind == Kind::Cpu { &CPU } else { &GPU };
    let mut slot = cache
        .get_or_init(|| Mutex::new(None))
        .lock()
        .map_err(|_| anyhow::anyhow!("backend loader lock poisoned"))?;
    if let Some(loaded) = slot.as_ref() {
        return Ok(loaded.clone());
    }
    let loaded = Arc::new(Loaded::open(&backend_path(kind)?, kind)?);
    *slot = Some(loaded.clone());
    Ok(loaded)
}

fn call(f: impl FnOnce(*mut u8, usize) -> i32) -> Result<()> {
    let mut error = [0u8; 2048];
    let status = f(error.as_mut_ptr(), error.len());
    let end = error.iter().position(|b| *b == 0).unwrap_or(error.len());
    ensure!(status == 0, "inference backend: {}", String::from_utf8_lossy(&error[..end]));
    Ok(())
}

struct RuntimeHandle {
    pointer: NonNull<c_void>,
    loaded: Arc<Loaded>,
}
// The ABI requires Runtime: Send + Sync and the plugin asserts these bounds.
unsafe impl Send for RuntimeHandle {}
unsafe impl Sync for RuntimeHandle {}
impl Drop for RuntimeHandle {
    fn drop(&mut self) {
        unsafe { (self.loaded.api.destroy)(self.pointer.as_ptr()) };
    }
}

pub struct Runtime {
    handle: Arc<RuntimeHandle>,
    info: BackendInfo,
}
impl Runtime {
    pub fn new(request: BackendRequest) -> Result<Self> {
        Self::build(request, 0)
    }
    pub fn with_max_batch(request: BackendRequest, maximum: usize) -> Result<Self> {
        ensure!(maximum > 0, "maximum batch must be positive");
        Self::build(request, maximum)
    }
    fn build(request: BackendRequest, maximum: usize) -> Result<Self> {
        select(request, |kind| {
            let loaded = load(kind)?;
            let mut pointer = std::ptr::null_mut();
            call(|e, n| unsafe { (loaded.api.create)(maximum, &mut pointer, e, n) })?;
            let pointer = NonNull::new(pointer).context("backend returned null runtime")?;
            Ok(Self { handle: Arc::new(RuntimeHandle { pointer, loaded }), info: kind.info() })
        })
    }
    pub fn backend_info(&self) -> BackendInfo {
        self.info
    }
    pub fn session(&self) -> Result<Session> {
        let mut pointer = std::ptr::null_mut();
        call(|e, n| unsafe {
            (self.handle.loaded.api.session)(self.handle.pointer.as_ptr(), &mut pointer, e, n)
        })?;
        Ok(Session {
            pointer: NonNull::new(pointer).context("backend returned null session")?,
            runtime: self.handle.clone(),
            info: self.info,
        })
    }
}

/// Thread-affine inference state.
///
/// ```compile_fail
/// fn send<T: Send>() {}
/// send::<magika_runtime::Session>();
/// ```
pub struct Session {
    pointer: NonNull<c_void>,
    runtime: Arc<RuntimeHandle>,
    info: BackendInfo,
}
// Tract sessions are thread-affine. NonNull keeps this !Send and !Sync,
// so creation, inference and destruction stay on the same worker thread.
impl Session {
    pub fn backend_info(&self) -> BackendInfo {
        self.info
    }
    pub fn run(&mut self, input: &[i32], batch: usize) -> Result<Vec<f32>> {
        ensure!(
            batch > 0 && batch.checked_mul(FEATURE_SIZE) == Some(input.len()),
            "invalid inference input shape"
        );
        let count = batch.checked_mul(NUM_LABELS).context("inference output size overflow")?;
        let mut output = vec![0f32; count];
        call(|e, n| unsafe {
            (self.runtime.loaded.api.run)(
                self.pointer.as_ptr(),
                input.as_ptr(),
                input.len(),
                batch,
                output.as_mut_ptr(),
                output.len(),
                e,
                n,
            )
        })?;
        Ok(output)
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        unsafe { (self.runtime.loaded.api.destroy_session)(self.pointer.as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_never_attempts_gpu_loading() {
        let mut visited = Vec::new();
        let result = select(BackendRequest::Cpu, |kind| {
            visited.push(kind);
            if kind == Kind::Cpu { Ok(7) } else { panic!("CPU loaded GPU") }
        });
        assert_eq!(result.unwrap(), 7);
        assert_eq!(visited, [Kind::Cpu]);
    }

    #[test]
    fn auto_falls_back_but_explicit_gpu_propagates_failure() {
        let mut visited = Vec::new();
        let mut load = |kind| {
            visited.push(kind);
            if kind == Kind::Cpu { Ok(7) } else { anyhow::bail!("GPU unavailable") }
        };
        assert_eq!(select(BackendRequest::Auto, &mut load).unwrap(), 7);
        assert!(select(BackendRequest::Gpu, &mut load).is_err());
        assert_eq!(visited, [gpu_kind(), Kind::Cpu, gpu_kind()]);
    }

    #[test]
    fn rejects_incompatible_backend_headers() {
        let good =
            (abi::ABI_VERSION, size_of::<abi::Api>() as u32, abi::CPU, FEATURE_SIZE, NUM_LABELS);
        assert!(validate(good, Kind::Cpu).is_ok());
        assert!(validate((99, good.1, good.2, good.3, good.4), Kind::Cpu).is_err());
        assert!(validate((good.0, 0, good.2, good.3, good.4), Kind::Cpu).is_err());
        assert!(validate(good, gpu_kind()).is_err());
        assert!(validate((good.0, good.1, good.2, good.3, 1), Kind::Cpu).is_err());
    }

    #[test]
    fn missing_backend_is_an_error() {
        assert!(
            Loaded::open(std::path::Path::new("/nonexistent/magika-backend.dylib"), Kind::Cpu)
                .is_err()
        );
    }
}
