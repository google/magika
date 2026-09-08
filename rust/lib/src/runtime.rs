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

use anyhow::Result;
use magika_tract_runtime::BackendRequest;

use crate::{BackendInfo, Builder, Session};

/// Shared prepared inference plans.
///
/// Create one runtime, share it between threads, and call [`Self::session`] inside each thread.
pub struct Runtime {
    inner: magika_tract_runtime::Runtime,
}

impl Runtime {
    /// Creates a default runtime.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Initializes a new Magika runtime builder with default values.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Returns the resolved CPU or GPU implementation.
    pub fn backend_info(&self) -> BackendInfo {
        self.inner.backend_info().into()
    }

    /// Spawns private mutable inference state for the current thread.
    pub fn session(&self) -> Result<Session> {
        Ok(Session { inner: self.inner.session()? })
    }

    pub(crate) fn new_internal(backend: BackendRequest, max_batch: Option<usize>) -> Result<Self> {
        let inner = match max_batch {
            None => magika_tract_runtime::Runtime::new(backend)?,
            Some(max_batch) => magika_tract_runtime::Runtime::with_max_batch(backend, max_batch)?,
        };
        Ok(Self { inner })
    }
}
