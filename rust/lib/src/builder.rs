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

use anyhow::Result;

use crate::{Backend, Runtime};

/// Configures and creates a Magika runtime.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    backend: Option<Backend>,
    max_batch: Option<usize>,
}

impl Builder {
    /// Selects automatic, CPU-only, or GPU-required inference.
    pub fn with_backend(mut self, backend: Backend) -> Self {
        self.backend = Some(backend);
        self
    }

    /// Declares the largest batch this session will ever be asked to identify.
    ///
    /// Declaring a smaller maximum skips unreachable fixed plans and makes startup cheaper. On an
    /// x86_64 CPU, smaller tails are padded through the largest resident optimized plan;
    /// elsewhere, requests are decomposed over the original resident classes.
    pub fn with_max_batch(mut self, max_batch: usize) -> Self {
        self.max_batch = Some(max_batch);
        self
    }

    /// Consumes the builder to create a Magika runtime.
    pub fn build(self) -> Result<Runtime> {
        Runtime::new_internal(Backend::to_request(self.backend), self.max_batch)
    }
}
