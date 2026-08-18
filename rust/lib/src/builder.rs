// Copyright 2024 Google LLC
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

use crate::{BackendRequest, Result, Runtime, Session};

/// Configures and creates a Magika session.
#[derive(Clone, Copy, Debug, Default)]
pub struct Builder {
    backend: BackendRequest,
    max_batch: Option<usize>,
}

impl Builder {
    /// Retained for source compatibility; tract inference threads use one CPU executor each.
    #[deprecated(
        note = "tract manages execution threads; configure application inference threads instead"
    )]
    pub fn with_inter_threads(self, _num_threads: usize) -> Self {
        self
    }

    /// Retained for source compatibility; tract inference threads use one CPU executor each.
    #[deprecated(
        note = "tract manages execution threads; configure application inference threads instead"
    )]
    pub fn with_intra_threads(self, _num_threads: usize) -> Self {
        self
    }

    /// Retained for source compatibility; tract always prepares its optimized fixed-shape graph.
    #[deprecated(note = "tract always applies its release graph optimization pipeline")]
    pub fn with_optimization_level<T>(self, _optimization_level: T) -> Self {
        self
    }

    /// Retained for source compatibility; each tract session executes synchronously.
    #[deprecated(note = "use one Magika session per application inference thread")]
    pub fn with_parallel_execution(self, _parallel_execution: bool) -> Self {
        self
    }

    /// Selects automatic, CPU-only, or GPU-required inference.
    pub fn with_backend(mut self, backend: BackendRequest) -> Self {
        self.backend = backend;
        self
    }

    /// Declares the largest batch this session will ever be asked to identify.
    ///
    /// Plans are prepared and warmed per batch class at build time, so declaring a smaller maximum
    /// skips the classes that can never be reached and makes startup proportionally cheaper.
    /// Passing a batch larger than this later still works: it is decomposed over the resident
    /// classes.
    pub fn with_max_batch(mut self, max_batch: usize) -> Self {
        self.max_batch = Some(max_batch);
        self
    }

    /// Prepares the model and creates one thread-private Magika session.
    pub fn build(self) -> Result<Session> {
        self.build_runtime()?.session()
    }

    /// Prepares shared fixed-shape plans for spawning multiple inference sessions.
    pub fn build_runtime(self) -> Result<Runtime> {
        match self.max_batch {
            Some(max_batch) => Runtime::with_max_batch(self.backend, max_batch),
            None => Runtime::new(self.backend),
        }
    }
}
