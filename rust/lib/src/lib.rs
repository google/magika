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

//! Determines the content type of a file with deep-learning.
//!
//! # Examples
//!
//! ```rust
//! use magika::{Features, Label, Session};
//!
//! # fn main() -> magika::Result<()> {
//! let magika = Session::new()?;
//! let features = Features::extract_sync(&b"#!/bin/sh\necho hello"[..])?;
//! let result = magika.identify_one_sync(&features)?;
//! assert_eq!(result.label(), Label::Shell);
//! # Ok(())
//! # }
//! ```

#![cfg_attr(feature = "_doc", feature(doc_auto_cfg))]

pub use crate::builder::Builder;
pub use crate::error::{Error, Result};
pub use crate::input::{AsyncInput, Features, SyncInput};
pub use crate::label::Label;
pub use crate::output::Output;
pub use crate::session::Session;

mod builder;
mod error;
mod future;
mod input;
mod label;
mod output;
mod session;
