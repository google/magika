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

use std::borrow::Borrow;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use anyhow::{Context, Result, anyhow};
use crossbeam_channel::Receiver;

use crate::{
    Backend, BackendInfo, Builder, Features, FeaturesOr, FileType, Input, Options, RegularFileType,
    Runtime, Session,
};

/// Batches smaller than this stay on the CPU in automatic mode: a GPU only pays off on full ones.
const GPU_MIN_BATCH: usize = 8;

/// Shared inference from any thread, on the CPU at once and on the GPU as soon as it is ready.
///
/// Preparing a GPU takes about 90 ms on an Apple M-series machine, more than a short run takes on
/// the CPU. So the engine prepares both on background threads, identifies on the CPU while the GPU
/// is prepared, and moves full batches to the GPU once it is ready. Creating an engine returns at
/// once; the first identification waits for the CPU only.
///
/// The engine is shared between threads, and each thread identifies through its own
/// [`EngineSession`].
pub struct Engine {
    cpu: Prepared,
    gpu: Option<Gpu>,
    options: Options,
}

struct Gpu {
    prepared: Prepared,
    /// Whether the GPU was requested explicitly, so that failing to prepare it is an error.
    required: bool,
}

/// A runtime prepared on its own thread, or deferred until waited on when `use_model` is disabled.
enum Prepared {
    Running {
        /// Set by the preparing thread, once.
        runtime: Arc<OnceLock<Result<Runtime>>>,
        /// Disconnected once `runtime` is set.
        resolved: Receiver<()>,
    },
    Deferred {
        runtime: OnceLock<Result<Runtime>>,
        builder: Builder,
    },
}

/// One thread's way to identify with an [`Engine`].
///
/// It spawns a session on a backend the first time it identifies there, so that a short run
/// never pays for the sessions of the backend it does not reach.
pub struct EngineSession<'a> {
    engine: &'a Engine,
    /// The session of each backend, by [`Backend`] value.
    sessions: [Option<Session>; 2],
    options: Options,
}

impl Engine {
    /// Starts preparing the CPU and, unless the builder asks for the CPU only, the GPU.
    ///
    /// With `Backend::Gpu`, failing to prepare the GPU is an error at the first identification;
    /// with no backend, the CPU keeps identifying.
    pub fn new(builder: Builder) -> Result<Self> {
        let options = builder.options.clone();
        if !options.use_model {
            let cpu = Prepared::deferred(builder.clone().with_backend(Backend::Cpu));
            let gpu = match builder.backend {
                Some(Backend::Gpu) => Some(Gpu {
                    prepared: Prepared::deferred(builder.with_backend(Backend::Gpu)),
                    required: true,
                }),
                _ => None,
            };
            return Ok(Engine { cpu, gpu, options });
        }
        let cpu = builder.clone().with_backend(Backend::Cpu);
        let (cpu, cpu_resolved) = Prepared::start("magika-cpu", cpu, None)?;
        let gpu = match builder.backend {
            Some(Backend::Cpu) => None,
            backend => {
                // Preparing a GPU competes with preparing the CPU, which a short run waits for,
                // so it starts once the CPU is ready.
                let gpu = builder.with_backend(Backend::Gpu);
                let (prepared, _) = Prepared::start("magika-gpu", gpu, Some(cpu_resolved))?;
                Some(Gpu { prepared, required: backend == Some(Backend::Gpu) })
            }
        };
        Ok(Engine { cpu, gpu, options })
    }

    /// Returns the backend that identifies a full batch now.
    ///
    /// This waits for the CPU, or for the GPU when it is required, and fails if it cannot be
    /// prepared, so it also checks an engine before use.
    pub fn backend_info(&self) -> Result<BackendInfo> {
        Ok(self.runtime(GPU_MIN_BATCH)?.backend_info())
    }

    /// Returns a session to identify with this engine from the current thread.
    pub fn session(&self, options: Option<Options>) -> EngineSession<'_> {
        let options = options.unwrap_or_else(|| self.options.clone());
        EngineSession { engine: self, sessions: [None, None], options }
    }

    /// Returns whether the engine may still move to a GPU.
    pub(crate) fn has_gpu(&self) -> bool {
        self.gpu.is_some()
    }

    /// Sends `batch` to an idle worker unless `stop` is signaled, or returns `Err(batch)` if
    /// preparing the GPU failed first.
    pub(crate) fn send_or_gpu_failed<T>(
        &self, work_sender: &crossbeam_channel::Sender<T>, stop: &Receiver<()>, batch: T,
    ) -> std::result::Result<bool, T> {
        if let Some(Gpu { prepared: Prepared::Running { runtime, resolved }, .. }) = &self.gpu
            && !matches!(runtime.get(), Some(Ok(_)))
        {
            crossbeam_channel::select! {
                send(work_sender, batch) -> _ => return Ok(true),
                recv(stop) -> _ => return Ok(false),
                // A preparing thread that panicked set nothing: the GPU failed too.
                recv(resolved) -> _ => {
                    if !matches!(runtime.get(), Some(Ok(_))) {
                        return Err(batch);
                    }
                }
            }
        }
        crossbeam_channel::select! {
            send(work_sender, batch) -> _ => Ok(true),
            recv(stop) -> _ => Ok(false),
        }
    }

    /// Returns the runtime with which to identify a batch of `len` files.
    fn runtime(&self, len: usize) -> Result<&Runtime> {
        if let Some(gpu) = &self.gpu {
            if gpu.required {
                return gpu.prepared.wait().context("preparing the GPU");
            }
            if let Some(Ok(runtime)) = gpu.prepared.get()
                && len >= GPU_MIN_BATCH
            {
                return Ok(runtime);
            }
        }
        self.cpu.wait().context("preparing the CPU")
    }
}

impl EngineSession<'_> {
    /// Identifies a file.
    pub fn identify_file(&mut self, path: impl AsRef<Path>) -> Result<FileType> {
        match FeaturesOr::extract_file(path, &self.options)? {
            FeaturesOr::Ruled(file_type) => Ok(file_type),
            FeaturesOr::Features(features) => {
                let [file_type] =
                    self.identify_features_batch(&[features])?.try_into().ok().unwrap();
                Ok(file_type.into())
            }
        }
    }

    /// Identifies content.
    pub fn identify_content(&mut self, input: impl Input) -> Result<RegularFileType> {
        match FeaturesOr::extract_content(input, &self.options)? {
            FeaturesOr::Ruled(file_type) => Ok(file_type),
            FeaturesOr::Features(features) => {
                let [file_type] =
                    self.identify_features_batch(&[features])?.try_into().ok().unwrap();
                Ok(file_type)
            }
        }
    }

    /// Identifies files from their features, on the GPU if it is ready and the batch is full.
    pub fn identify_features_batch(
        &mut self, features: &[impl Borrow<Features>],
    ) -> Result<Vec<RegularFileType>> {
        if features.is_empty() {
            return Ok(Vec::new());
        }
        self.session(features.len())?.identify_features_batch(features)
    }

    fn session(&mut self, len: usize) -> Result<&mut Session> {
        let runtime = self.engine.runtime(len)?;
        let session = match &mut self.sessions[runtime.backend_info().backend() as usize] {
            Some(session) => session,
            slot => {
                let mut session = runtime.session()?;
                *session.options_mut() = self.options.clone();
                slot.insert(session)
            }
        };
        Ok(session)
    }
}

impl Prepared {
    /// Defers preparing a runtime until it is explicitly waited on.
    fn deferred(builder: Builder) -> Self {
        Prepared::Deferred { runtime: OnceLock::new(), builder }
    }

    /// Starts preparing a runtime, once `after` disconnects if given.
    fn start(
        name: &str, builder: Builder, after: Option<Receiver<()>>,
    ) -> Result<(Self, Receiver<()>)> {
        let runtime = Arc::new(OnceLock::new());
        let (resolve, resolved) = crossbeam_channel::bounded(0);
        std::thread::Builder::new().name(name.to_string()).spawn({
            let runtime = runtime.clone();
            move || {
                if let Some(after) = after {
                    let _ = after.recv();
                }
                let _ = runtime.set(builder.build());
                drop(resolve);
            }
        })?;
        Ok((Prepared::Running { runtime, resolved: resolved.clone() }, resolved))
    }

    fn get(&self) -> Option<&Result<Runtime>> {
        match self {
            Prepared::Running { runtime, .. } => runtime.get(),
            Prepared::Deferred { runtime, .. } => runtime.get(),
        }
    }

    fn wait(&self) -> Result<&Runtime> {
        let result = match self {
            Prepared::Running { runtime, resolved } => {
                let _ = resolved.recv();
                runtime.get().ok_or_else(|| anyhow!("the preparing thread panicked"))?
            }
            Prepared::Deferred { runtime, builder } => {
                runtime.get_or_init(|| builder.clone().build())
            }
        };
        match result {
            Ok(runtime) => Ok(runtime),
            Err(error) => Err(anyhow!("{error:#}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Prepared {
        /// A preparation that already ended with `runtime`.
        fn done(runtime: Result<Runtime>) -> Self {
            let (resolve, resolved) = crossbeam_channel::bounded::<()>(0);
            drop(resolve);
            Prepared::Running { runtime: Arc::new(OnceLock::from(runtime)), resolved }
        }

        /// A preparation still running until the returned sender is dropped.
        fn running() -> (Self, crossbeam_channel::Sender<()>) {
            let (resolve, resolved) = crossbeam_channel::bounded(0);
            (Prepared::Running { runtime: Arc::default(), resolved }, resolve)
        }
    }

    fn cpu() -> Prepared {
        Prepared::done(Runtime::builder().with_backend(Backend::Cpu).build())
    }

    fn features(count: usize) -> Vec<Features> {
        let options = Options::default();
        let content = std::fs::read("../../tests_data/basic/rust/code.rs").unwrap();
        (0..count)
            .map(|_| match FeaturesOr::extract_content(&content[..], &options).unwrap() {
                FeaturesOr::Features(features) => features,
                FeaturesOr::Ruled(_) => unreachable!(),
            })
            .collect()
    }

    #[test]
    fn a_gpu_still_preparing_leaves_the_cpu_identifying() {
        let (prepared, resolve) = Prepared::running();
        let engine = Engine {
            cpu: cpu(),
            gpu: Some(Gpu { prepared, required: false }),
            options: Options::default(),
        };
        let results = engine.session(None).identify_features_batch(&features(8));
        assert_eq!(results.unwrap()[0].info().label, "rust");
        assert_eq!(engine.backend_info().unwrap().backend(), Backend::Cpu);
        let (work_sender, _work_receiver) = crossbeam_channel::bounded::<()>(0);
        let (stop, stopped) = crossbeam_channel::bounded::<()>(1);
        drop(stop);
        assert_eq!(engine.send_or_gpu_failed(&work_sender, &stopped, ()), Ok(false));
        drop(resolve);
    }

    #[test]
    fn a_failed_gpu_is_an_error_only_when_required() {
        let failed = || Prepared::done(Err(anyhow!("no GPU here")));
        let (work_sender, _work_receiver) = crossbeam_channel::bounded::<()>(0);
        let (_stop, running) = crossbeam_channel::bounded::<()>(0);
        let automatic = Engine {
            cpu: cpu(),
            gpu: Some(Gpu { prepared: failed(), required: false }),
            options: Options::default(),
        };
        assert_eq!(automatic.send_or_gpu_failed(&work_sender, &running, ()), Err(()));
        let results = automatic.session(None).identify_features_batch(&features(8));
        assert_eq!(results.unwrap().len(), 8);
        let required = Engine {
            cpu: cpu(),
            gpu: Some(Gpu { prepared: failed(), required: true }),
            options: Options::default(),
        };
        let error = required.session(None).identify_features_batch(&features(1));
        assert_eq!(format!("{:#}", error.unwrap_err()), "preparing the GPU: no GPU here");
    }

    #[test]
    fn disabled_model_defers_runtime_until_needed() {
        let options = Options { use_model: false, ..Options::default() };
        let builder = Builder::default().with_backend(Backend::Cpu).with_options(options);
        let engine = Engine::new(builder).unwrap();
        let mut session = engine.session(None);
        let rust = "../../tests_data/basic/rust/code.rs";
        assert_eq!(session.identify_file(rust).unwrap().info().label, "txt");
        assert!(engine.cpu.get().is_none(), "ruled identification must not build the runtime");
        assert_eq!(engine.backend_info().unwrap().backend(), Backend::Cpu);
        assert!(engine.cpu.get().is_some());
    }

    #[test]
    fn an_engine_is_shared_between_threads() {
        let engine = Engine::new(Builder::default().with_backend(Backend::Cpu)).unwrap();
        assert!(!engine.has_gpu());
        std::thread::scope(|scope| {
            for _ in 0..8 {
                scope.spawn(|| {
                    let mut session = engine.session(None);
                    let rust = "../../tests_data/basic/rust/code.rs";
                    assert_eq!(session.identify_file(rust).unwrap().info().label, "rust");
                });
            }
        });
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn full_batches_move_to_a_ready_gpu() {
        let engine = Engine::new(Builder::default().with_backend(Backend::Gpu)).unwrap();
        let results = engine.session(None).identify_features_batch(&features(8));
        assert_eq!(results.unwrap()[7].info().label, "rust");
        assert_eq!(engine.backend_info().unwrap().backend(), Backend::Gpu);
    }
}
