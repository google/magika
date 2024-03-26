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
use std::sync::Mutex;

use ort::{GraphOptimizationLevel, Session};

use crate::{MagikaResult, MagikaSession};

/// Configures and creates a Magika session.
#[derive(Debug)]
pub struct MagikaBuilder<Config> {
    config: Config,
    builder: Builder,
}

#[derive(Debug, Default)]
struct Builder {
    inter_threads: Option<i16>,
    intra_threads: Option<i16>,
    optimization_level: Option<GraphOptimizationLevel>,
    parallel_execution: Option<bool>,
}

impl<Config> MagikaBuilder<Config> {
    /// Initializes a new Magika session builder with default values.
    pub fn new(config: Config) -> Self {
        let builder = Builder::default();
        MagikaBuilder { config, builder }
    }

    /// Configures the number of threads to parallelize the execution of the graph.
    pub fn with_inter_threads(mut self, num_threads: i16) -> Self {
        self.builder.inter_threads = Some(num_threads);
        self
    }

    /// Configures the number of threads to parallelize the execution within nodes.
    pub fn with_intra_threads(mut self, num_threads: i16) -> Self {
        self.builder.intra_threads = Some(num_threads);
        self
    }

    /// Configures the session optimization level.
    pub fn with_optimization_level(mut self, opt_level: GraphOptimizationLevel) -> Self {
        self.builder.optimization_level = Some(opt_level);
        self
    }

    /// Configures the session parallel execution.
    pub fn with_parallel_execution(mut self, parallel_execution: bool) -> Self {
        self.builder.parallel_execution = Some(parallel_execution);
        self
    }

    /// Consumes the builder to create a Magika session.
    pub fn build(self, model_dir: impl AsRef<Path>) -> MagikaResult<MagikaSession<Config>> {
        let model_dir = model_dir.as_ref();
        let mut session = Session::builder()?;
        let MagikaBuilder { config, builder } = self;
        let Builder {
            inter_threads,
            intra_threads,
            optimization_level,
            parallel_execution,
        } = builder;
        if let Some(num_threads) = inter_threads {
            session = session.with_inter_threads(num_threads)?;
        }
        if let Some(num_threads) = intra_threads {
            session = session.with_intra_threads(num_threads)?;
        }
        if let Some(opt_level) = optimization_level {
            session = session.with_optimization_level(opt_level)?;
        }
        if let Some(parallel_execution) = parallel_execution {
            session = session.with_parallel_execution(parallel_execution)?;
        }
        let session = session.with_model_from_file(model_dir.join("model.onnx"))?;
        let session = Mutex::new(session);
        Ok(MagikaSession { session, config })
    }
}
