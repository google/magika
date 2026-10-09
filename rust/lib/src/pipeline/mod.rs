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

//! Helpers to identify many files fast, shared by the Magika command line, Python and C.
//!
//! The rest of the crate is the API to build any pipeline: [`crate::FeaturesOrRuled`] extracts
//! features, and [`crate::Session::identify_features_batch`] runs the model. These helpers are one
//! such pipeline, written with that API only:
//!
//! - [`Engine`] identifies from any thread, on the CPU at once and on the GPU once it is ready;
//! - [`Pipeline`] walks paths, reads them in parallel, and returns their results in order.

pub use self::engine::{Engine, EngineSession};
pub use self::paths::{DirectoryCycle, Identified, Pipeline, PipelineOptions};

mod engine;
mod paths;
