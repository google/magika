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

use std::path::Path;
use std::sync::{Mutex, OnceLock};

use onnxruntime::environment::Environment;
use onnxruntime::{GraphOptimizationLevel, LoggingLevel};

use crate::{MagikaConfig, MagikaResult, MagikaSession};

/// Configures and creates a Magika session.
#[derive(Debug)]
pub struct MagikaBuilder {
    name: String,
    logging_level: LoggingLevel,
    optimization_level: GraphOptimizationLevel,
    number_threads: i16,
}

impl MagikaBuilder {
    /// Initializes a new Magika session builder with default values.
    pub fn new() -> Self {
        MagikaBuilder {
            name: "default".to_string(),
            logging_level: LoggingLevel::Warning,
            optimization_level: GraphOptimizationLevel::Basic,
            number_threads: 1,
        }
    }

    /// Configures the environment name.
    ///
    /// Defaults to `"default"` when unset.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Configures the environment logging level.
    ///
    /// Defaults to `LoggingLevel::Warning` when unset.
    pub fn with_logging_level(mut self, logging_level: LoggingLevel) -> Self {
        self.logging_level = logging_level;
        self
    }

    /// Configures the session number of threads.
    ///
    /// Defaults to `1` when unset.
    pub fn with_number_threads(mut self, number_threads: i16) -> Self {
        self.number_threads = number_threads;
        self
    }

    /// Configures the session optimization level.
    ///
    /// Defaults to `GraphOptimizationLevel::Basic` when unset.
    pub fn with_optimization_level(mut self, optimization_level: GraphOptimizationLevel) -> Self {
        self.optimization_level = optimization_level;
        self
    }

    /// Consumes the builder to create a Magika session.
    pub fn build(self, model_dir: impl AsRef<Path>) -> MagikaResult<MagikaSession> {
        // The onnxruntime crate mentions that only one ONNX environment can be created per process,
        // but doesn't provide a way to access their singleton. We hold our own singleton to their
        // singleton. This simplifies lifetime issues by using `'static` everywhere.
        static ENVIRONMENT: OnceLock<Environment> = OnceLock::new();
        // TODO: Once OnceLock::get_or_try_init() is stable, we should use it to propagate the error
        // instead of panicking.
        let environment = ENVIRONMENT.get_or_init(|| {
            Environment::builder()
                .with_name(self.name)
                .with_log_level(self.logging_level)
                .build()
                .unwrap()
        });
        let model_dir = model_dir.as_ref();
        let session = environment
            .new_session_builder()?
            .with_number_threads(self.number_threads)?
            .with_optimization_level(self.optimization_level)?
            .with_model_from_file(model_dir.join("model.onnx"))?;
        let session = Mutex::new(session);
        let config = MagikaConfig::parse(model_dir.join("model_config.json"))?;
        Ok(MagikaSession { session, config })
    }
}
