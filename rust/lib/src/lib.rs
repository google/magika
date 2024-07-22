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

//! Determines the type of a file with deep-learning.
//!
//! # Examples
//!
//! ```rust
//! # fn main() -> magika::Result<()> {
//! // A Magika session can be used multiple times across multiple threads.
//! let magika = magika::Session::new()?;
//!
//! // Files can be identified from their path.
//! assert_eq!(magika.identify_file_sync("src/lib.rs")?.info().label, "rust");
//!
//! // Contents can also be identified directly from memory.
//! let result = magika.identify_content_sync(&b"#!/bin/sh\necho hello"[..])?;
//! assert_eq!(result.info().label, "shell");
//! # Ok(())
//! # }
//! ```

#![cfg_attr(feature = "_doc", feature(doc_auto_cfg))]

pub use crate::builder::Builder;
pub use crate::content::{ContentType, MODEL_NAME};
pub use crate::error::{Error, Result};
pub use crate::file::{FileType, InferredType, RuledType, TypeInfo};
pub use crate::input::{AsyncInput, Features, FeaturesOrRuled, SyncInput};
pub use crate::session::Session;

mod builder;
mod config;
mod content;
mod error;
mod file;
mod future;
mod input;
mod model;
mod session;
