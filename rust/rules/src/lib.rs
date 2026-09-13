// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Bounded format rules: a YARA subset evaluated in pure Rust over the first [`PREFIX_LIMIT`]
//! bytes of an input. The crate has no native dependency, never reads environment variables,
//! holds no global state and never names a Magika content type; callers map
//! [`RuleSet::labels`] to their own label type.

#![forbid(unsafe_code)]

mod error;
mod ir;
mod matcher;
mod source;

pub use error::Error;
pub use source::{Bucket, Class, RuleInfo, Source};

/// Bytes of input a rule may inspect.
pub const PREFIX_LIMIT: usize = 4096;
