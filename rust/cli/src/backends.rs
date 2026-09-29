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

//! The inference runtimes of a run: the CPU at once, the GPU as soon as it is ready.
//!
//! Preparing a GPU takes about 90 ms on an Apple M-series machine, more than a short run takes on
//! the CPU, which is ready in a few milliseconds. So the workers start identifying files on the CPU
//! while the GPU is prepared on its own thread, and move to the GPU once it is ready.

use std::sync::{Arc, OnceLock};

use anyhow::{Context, Result};
use magika::{Backend, Builder, Runtime};

use crate::{default_inference_threads, BackendChoice};

/// Batches smaller than this stay on the CPU in automatic mode: a GPU only pays off on full ones.
const GPU_MIN_BATCH: usize = 8;

pub(crate) struct Backends {
    cpu: Runtime,
    gpu: Option<Gpu>,
    workers: usize,
    /// How many workers start at once; the others only start when the GPU cannot be used.
    starting: usize,
}

struct Gpu {
    /// Set by the preparing thread, once.
    prepared: Arc<OnceLock<Result<Runtime>>>,
    /// Disconnected once `prepared` is set.
    resolved: crossbeam_channel::Receiver<()>,
    /// Whether the GPU was requested explicitly, so that failing to prepare it is an error.
    required: bool,
}

impl Backends {
    /// Prepares the CPU, and starts preparing the GPU unless the CPU was requested.
    ///
    /// `threads` overrides the number of workers.
    pub(crate) fn start(
        builder: Builder, choice: BackendChoice, threads: Option<usize>,
    ) -> Result<Self> {
        let cpu = builder.clone().with_backend(Backend::Cpu).build()?;
        let workers = threads.unwrap_or_else(|| default_inference_threads(Backend::Cpu));
        let (gpu, starting) = match choice {
            BackendChoice::Cpu => (None, workers),
            // The CPU competes with preparing the GPU, which a long run waits for, so the workers
            // leave it a third of the host until it is ready. A GPU is as fast with as many workers.
            BackendChoice::Auto | BackendChoice::Gpu => {
                let gpu = Gpu::start(builder, choice == BackendChoice::Gpu)?;
                let starting = match threads {
                    Some(threads) => threads,
                    None => (workers * 2 / 3).max(default_inference_threads(Backend::Gpu)),
                };
                (Some(gpu), starting.min(workers))
            }
        };
        Ok(Backends { cpu, gpu, workers, starting })
    }

    /// Returns how many workers start at once.
    pub(crate) fn starting(&self) -> usize {
        self.starting
    }

    /// Returns how many more workers to start, once known: all remaining ones if the GPU cannot be
    /// used, none if it can or if `done` disconnects first because the run ended.
    pub(crate) fn more_workers(&self, done: &crossbeam_channel::Receiver<()>) -> usize {
        let Some(gpu) = &self.gpu else { return 0 };
        crossbeam_channel::select! {
            recv(gpu.resolved) -> _ => match gpu.prepared.get() {
                Some(Err(_)) => self.workers - self.starting,
                _ => 0,
            },
            recv(done) -> _ => 0,
        }
    }

    /// Returns the runtime with which to identify a batch of `len` files.
    pub(crate) fn runtime(&self, len: usize) -> Result<&Runtime> {
        let Some(gpu) = &self.gpu else { return Ok(&self.cpu) };
        match gpu.prepared.get() {
            Some(Ok(runtime)) if gpu.required || len >= GPU_MIN_BATCH => Ok(runtime),
            Some(Err(error)) if gpu.required => {
                Err(anyhow::anyhow!("{error:#}")).context("preparing the GPU")
            }
            _ => Ok(&self.cpu),
        }
    }
}

impl Gpu {
    fn start(builder: Builder, required: bool) -> Result<Self> {
        let prepared = Arc::new(OnceLock::new());
        let (resolve, resolved) = crossbeam_channel::bounded(0);
        std::thread::Builder::new().name("magika-gpu".to_string()).spawn({
            let prepared = prepared.clone();
            move || {
                #[cfg(feature = "_trace")]
                let start = std::time::Instant::now();
                let runtime = builder.with_backend(Backend::Gpu).build();
                #[cfg(feature = "_trace")]
                match &runtime {
                    Ok(_) => eprintln!("trace  gpu ready after {:?}", start.elapsed()),
                    Err(error) => {
                        eprintln!("trace  gpu failed after {:?}: {error:#}", start.elapsed())
                    }
                }
                let _ = prepared.set(runtime);
                drop(resolve);
            }
        })?;
        Ok(Gpu { prepared, resolved, required })
    }
}
