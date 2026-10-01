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

use crate::{Backend, Options, PredictionMode, Runtime};

/// Configures and creates a Magika runtime.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    backend: Option<Backend>,
    max_batch: Option<usize>,
    options: Options,
}

impl Builder {
    /// Selects a specific backend for inference.
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

    /// Sets options to identify files.
    ///
    /// These options can be later modified at the runtime and session level. Sessions inherit
    /// options from the runtime when created.
    pub fn with_options(mut self, options: Options) -> Self {
        self.options = options;
        self
    }

    /// Returns the current options.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Configures whether to use rules.
    pub fn with_rules(mut self, use_rules: bool) -> Self {
        self.options.use_rules = use_rules;
        self
    }

    /// Configures whether to use the model.
    pub fn with_model(mut self, use_model: bool) -> Self {
        self.options.use_model = use_model;
        self
    }

    /// Configures how confident the model needs to be.
    pub fn with_prediction_mode(mut self, prediction_mode: PredictionMode) -> Self {
        self.options.prediction_mode = prediction_mode;
        self
    }

    /// Configures whether to follow symlinks.
    pub fn with_follow_symlink(mut self, follow_symlink: bool) -> Self {
        self.options.follow_symlink = follow_symlink;
        self
    }

    /// Consumes the builder to create a Magika runtime.
    pub fn build(self) -> Result<Runtime> {
        Runtime::new_internal(Backend::to_request(self.backend), self.max_batch, self.options)
    }
}
