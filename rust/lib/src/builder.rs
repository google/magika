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

use ort::session::builder::GraphOptimizationLevel;

use crate::{Result, Session};

/// Configures and creates a Magika session.
#[derive(Debug, Default)]
pub struct Builder {
    inter_threads: Option<usize>,
    intra_threads: Option<usize>,
    optimization_level: Option<GraphOptimizationLevel>,
    parallel_execution: Option<bool>,
}

impl Builder {
    /// Configures the number of threads to parallelize the execution of the graph.
    pub fn with_inter_threads(mut self, num_threads: usize) -> Self {
        self.inter_threads = Some(num_threads);
        self
    }

    /// Configures the number of threads to parallelize the execution within nodes.
    pub fn with_intra_threads(mut self, num_threads: usize) -> Self {
        self.intra_threads = Some(num_threads);
        self
    }

    /// Configures the session optimization level.
    pub fn with_optimization_level(mut self, opt_level: GraphOptimizationLevel) -> Self {
        self.optimization_level = Some(opt_level);
        self
    }

    /// Configures the session parallel execution.
    pub fn with_parallel_execution(mut self, parallel_execution: bool) -> Self {
        self.parallel_execution = Some(parallel_execution);
        self
    }

    /// Consumes the builder to create a Magika session.
    pub fn build(self) -> Result<Session> {
        let mut session = ort::session::Session::builder()?;
        let Builder { inter_threads, intra_threads, optimization_level, parallel_execution } = self;
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
        let session = session.commit_from_memory(include_bytes!("model.onnx"))?;
        Ok(Session { session })
    }
}
