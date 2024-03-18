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

//! A tool to determine the content type of a file with deep-learning.
//!
//! TODO(release): Add some description and possibly disclaimer about readiness.

#![forbid(unsafe_code)]
#![warn(missing_docs, unreachable_pub, unused)]

pub use crate::builder::MagikaBuilder;
use crate::config::MagikaConfig;
pub use crate::error::{MagikaError, MagikaResult};
pub use crate::input::{MagikaFeatures, MagikaInput};
pub use crate::output::MagikaOutput;
pub use crate::session::MagikaSession;

mod builder;
mod config;
mod error;
mod input;
mod output;
mod session;
