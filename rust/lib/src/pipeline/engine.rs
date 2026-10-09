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
    Backend, BackendInfo, Builder, Features, FeaturesOrRuled, FileType, Input, Options, Runtime,
    Session,
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
/// [`EngineSession`]. Options are given with each call, so one engine serves any options.
pub struct Engine {
    cpu: Prepared,
    gpu: Option<Gpu>,
}

struct Gpu {
    prepared: Prepared,
    /// Whether the GPU was requested explicitly, so that failing to prepare it is an error.
    required: bool,
}

/// A runtime being prepared on its own thread.
struct Prepared {
    /// Set by the preparing thread, once.
    runtime: Arc<OnceLock<Result<Runtime>>>,
    /// Disconnected once `runtime` is set.
    resolved: Receiver<()>,
}

/// One thread's way to identify with an [`Engine`].
///
/// It spawns a session on a backend the first time it identifies there, so that a short run
/// never pays for the sessions of the backend it does not reach.
pub struct EngineSession<'a> {
    engine: &'a Engine,
    /// The session of each backend, by [`Backend`] value.
    sessions: [Option<Session>; 2],
}

impl Engine {
    /// Starts preparing the CPU and, unless the builder asks for the CPU only, the GPU.
    ///
    /// With `Backend::Gpu`, failing to prepare the GPU is an error at the first identification;
    /// with no backend, the CPU keeps identifying.
    pub fn new(builder: Builder) -> Result<Self> {
        let cpu = builder.clone().with_backend(Backend::Cpu);
        let cpu = Prepared::start("magika-cpu", cpu, None)?;
        let gpu = match builder.backend() {
            Some(Backend::Cpu) => None,
            backend => {
                // Preparing a GPU competes with preparing the CPU, which a short run waits for,
                // so it starts once the CPU is ready.
                let gpu = builder.with_backend(Backend::Gpu);
                let prepared = Prepared::start("magika-gpu", gpu, Some(cpu.resolved.clone()))?;
                Some(Gpu { prepared, required: backend == Some(Backend::Gpu) })
            }
        };
        Ok(Engine { cpu, gpu })
    }

    /// Returns the backend that identifies a full batch now.
    ///
    /// This waits for the CPU, or for the GPU when it is required, and fails if it cannot be
    /// prepared, so it also checks an engine before use.
    pub fn backend_info(&self) -> Result<BackendInfo> {
        Ok(self.runtime(GPU_MIN_BATCH)?.backend_info())
    }

    /// Returns a session to identify with this engine from the current thread.
    pub fn session(&self) -> EngineSession<'_> {
        EngineSession { engine: self, sessions: [None, None] }
    }

    /// Returns whether the engine may still move to a GPU.
    pub(crate) fn has_gpu(&self) -> bool {
        self.gpu.is_some()
    }

    /// Waits until the GPU is prepared or `done` disconnects, and returns whether it failed.
    pub(crate) fn gpu_failed(&self, done: &Receiver<()>) -> bool {
        let Some(gpu) = &self.gpu else { return false };
        crossbeam_channel::select! {
            // A preparing thread that panicked set nothing: the GPU failed too.
            recv(gpu.prepared.resolved) -> _ => !matches!(gpu.prepared.runtime.get(), Some(Ok(_))),
            recv(done) -> _ => false,
        }
    }

    /// Returns the runtime with which to identify a batch of `len` files.
    fn runtime(&self, len: usize) -> Result<&Runtime> {
        if let Some(gpu) = &self.gpu {
            if gpu.required {
                return gpu.prepared.wait().context("preparing the GPU");
            }
            if let Some(Ok(runtime)) = gpu.prepared.runtime.get()
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
    pub fn identify_file(&mut self, path: impl AsRef<Path>, options: &Options) -> Result<FileType> {
        self.identify(FeaturesOrRuled::extract_file(path, options)?, options)
    }

    /// Identifies content.
    pub fn identify_content(&mut self, input: impl Input, options: &Options) -> Result<FileType> {
        self.identify(FeaturesOrRuled::extract_content(input, options)?, options)
    }

    /// Identifies files from their features, on the GPU if it is ready and the batch is full.
    pub fn identify_features_batch(
        &mut self, features: &[impl Borrow<Features>], options: &Options,
    ) -> Result<Vec<FileType>> {
        if features.is_empty() {
            return Ok(Vec::new());
        }
        let runtime = self.engine.runtime(features.len())?;
        let session = match &mut self.sessions[runtime.backend_info().backend() as usize] {
            Some(session) => session,
            slot => slot.insert(runtime.session()?),
        };
        *session.options_mut() = options.clone();
        session.identify_features_batch(features)
    }

    fn identify(&mut self, extracted: FeaturesOrRuled, options: &Options) -> Result<FileType> {
        match extracted {
            FeaturesOrRuled::Ruled(file_type) => Ok(file_type),
            FeaturesOrRuled::Features(features) => {
                let [result] =
                    self.identify_features_batch(&[features], options)?.try_into().unwrap();
                Ok(result)
            }
        }
    }
}

impl Prepared {
    /// Starts preparing a runtime, once `after` disconnects if given.
    fn start(name: &str, builder: Builder, after: Option<Receiver<()>>) -> Result<Self> {
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
        Ok(Prepared { runtime, resolved })
    }

    fn wait(&self) -> Result<&Runtime> {
        let _ = self.resolved.recv();
        match self.runtime.get() {
            Some(Ok(runtime)) => Ok(runtime),
            Some(Err(error)) => Err(anyhow!("{error:#}")),
            None => Err(anyhow!("the preparing thread panicked")),
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
            Prepared { runtime: Arc::new(OnceLock::from(runtime)), resolved }
        }

        /// A preparation still running until the returned sender is dropped.
        fn running() -> (Self, crossbeam_channel::Sender<()>) {
            let (resolve, resolved) = crossbeam_channel::bounded(0);
            (Prepared { runtime: Arc::default(), resolved }, resolve)
        }
    }

    fn cpu() -> Prepared {
        Prepared::done(Runtime::builder().with_backend(Backend::Cpu).build())
    }

    fn features(count: usize) -> Vec<Features> {
        let options = Options::default();
        let content = std::fs::read("../../tests_data/basic/rust/code.rs").unwrap();
        (0..count)
            .map(|_| match FeaturesOrRuled::extract_content(&content[..], &options).unwrap() {
                FeaturesOrRuled::Features(features) => features,
                FeaturesOrRuled::Ruled(_) => unreachable!(),
            })
            .collect()
    }

    #[test]
    fn a_gpu_still_preparing_leaves_the_cpu_identifying() {
        let (prepared, resolve) = Prepared::running();
        let engine = Engine { cpu: cpu(), gpu: Some(Gpu { prepared, required: false }) };
        let results = engine.session().identify_features_batch(&features(8), &Options::default());
        assert_eq!(results.unwrap()[0].info().label, "rust");
        assert_eq!(engine.backend_info().unwrap().backend(), Backend::Cpu);
        let (done, finished) = crossbeam_channel::bounded::<()>(0);
        drop(done);
        assert!(!engine.gpu_failed(&finished), "the run ended first");
        drop(resolve);
    }

    #[test]
    fn a_failed_gpu_is_an_error_only_when_required() {
        let failed = || Prepared::done(Err(anyhow!("no GPU here")));
        let (_done, running) = crossbeam_channel::bounded::<()>(0);
        let automatic =
            Engine { cpu: cpu(), gpu: Some(Gpu { prepared: failed(), required: false }) };
        assert!(automatic.gpu_failed(&running));
        let results =
            automatic.session().identify_features_batch(&features(8), &Options::default());
        assert_eq!(results.unwrap().len(), 8);
        let required = Engine { cpu: cpu(), gpu: Some(Gpu { prepared: failed(), required: true }) };
        let error = required.session().identify_features_batch(&features(1), &Options::default());
        assert_eq!(format!("{:#}", error.unwrap_err()), "preparing the GPU: no GPU here");
    }

    #[test]
    fn an_engine_is_shared_between_threads() {
        let engine = Engine::new(Builder::default().with_backend(Backend::Cpu)).unwrap();
        assert!(!engine.has_gpu());
        std::thread::scope(|scope| {
            for _ in 0..8 {
                scope.spawn(|| {
                    let mut session = engine.session();
                    let options = Options::default();
                    let rust = "../../tests_data/basic/rust/code.rs";
                    assert_eq!(session.identify_file(rust, &options).unwrap().info().label, "rust");
                });
            }
        });
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn full_batches_move_to_a_ready_gpu() {
        let engine = Engine::new(Builder::default().with_backend(Backend::Gpu)).unwrap();
        let results = engine.session().identify_features_batch(&features(8), &Options::default());
        assert_eq!(results.unwrap()[7].info().label, "rust");
        assert_eq!(engine.backend_info().unwrap().backend(), Backend::Gpu);
    }
}
