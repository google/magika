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

use magika_tract_runtime::BackendRequest;

/// Resolved device class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Backend {
    /// CPU inference.
    Cpu,
    /// GPU inference.
    Gpu,
}

/// Resolved runtime information suitable for diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendInfo {
    backend: Backend,
    implementation: &'static str,
}

impl BackendInfo {
    /// Returns whether inference runs on CPU or GPU.
    pub fn backend(self) -> Backend {
        self.backend
    }

    /// Returns the selected tract implementation name.
    pub fn implementation(self) -> &'static str {
        self.implementation
    }
}

impl Backend {
    pub(crate) fn to_request(request: Option<Backend>) -> BackendRequest {
        match request {
            None => BackendRequest::Auto,
            Some(Backend::Cpu) => BackendRequest::Cpu,
            Some(Backend::Gpu) => BackendRequest::Gpu,
        }
    }
}

impl From<magika_tract_runtime::Backend> for Backend {
    fn from(value: magika_tract_runtime::Backend) -> Self {
        match value {
            magika_tract_runtime::Backend::Cpu => Backend::Cpu,
            magika_tract_runtime::Backend::Gpu => Backend::Gpu,
        }
    }
}

impl From<magika_tract_runtime::BackendInfo> for BackendInfo {
    fn from(value: magika_tract_runtime::BackendInfo) -> Self {
        BackendInfo { backend: value.backend().into(), implementation: value.implementation() }
    }
}
