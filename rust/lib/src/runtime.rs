// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{BackendInfo, BackendRequest, Error, Result, Session};

/// Shared prepared inference plans.
///
/// Create one runtime, share it between threads, and call [`Self::session`] inside each thread to
/// allocate its private mutable tract state.
pub struct Runtime {
    inner: magika_tract_runtime::Runtime,
}

impl Runtime {
    pub(crate) fn new(backend: BackendRequest) -> Result<Self> {
        let inner = magika_tract_runtime::Runtime::new(backend).map_err(Error::inference)?;
        Ok(Self { inner })
    }

    /// Returns the resolved CPU or GPU implementation.
    pub fn backend_info(&self) -> BackendInfo {
        self.inner.backend_info()
    }

    /// Spawns private mutable inference state for the current thread.
    pub fn session(&self) -> Result<Session> {
        let inner = self.inner.session().map_err(Error::inference)?;
        Ok(Session { inner })
    }
}
