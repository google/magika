// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(feature = "bundled", doc = include_str!("../README.md"))]
#![cfg_attr(
    not(feature = "bundled"),
    doc = "Bounded format rules: a YARA subset evaluated in pure Rust. See README.md."
)]
#![forbid(unsafe_code)]

mod error;
mod eval;
mod ir;
mod lower;
mod matcher;
mod source;

pub use error::Error;
pub use source::{Bucket, Class, RuleInfo, Source};

/// Bytes of input a rule may inspect.
pub const PREFIX_LIMIT: usize = 4096;

/// What one scan looks at.
#[derive(Clone, Copy, Debug)]
pub struct Input<'a> {
    /// The first `min(size, PREFIX_LIMIT)` bytes of the input, exactly.
    pub prefix: &'a [u8],
    /// The size of the whole input.
    pub size: u64,
    /// The input's trailing window, read only by facts rules; pass `None` until they land.
    pub tail: Option<&'a [u8]>,
}

/// The pack's verdict for one input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every matching enforced rule agrees on this index into [`RuleSet::labels`].
    Match(usize),
    /// No enforced rule matched.
    NoMatch,
    /// Enforced rules with different labels matched.
    Conflict,
    /// The prefix is empty or is not exactly `min(size, PREFIX_LIMIT)` bytes.
    InsufficientInput,
}

/// A compiled pack. `Send + Sync`; share it through `Arc`.
pub struct RuleSet {
    program: ir::Program,
    labels: Vec<String>,
    rules: Vec<RuleInfo>,
}

impl RuleSet {
    /// Compiles every enforced rule of `source`.
    pub fn compile(source: &Source) -> Result<Self, Error> {
        let (program, labels) = lower::lower(source)?;
        Ok(RuleSet { program, labels, rules: source.rules().to_vec() })
    }

    /// Distinct labels of the enforced rules, in source order; [`Outcome::Match`] indexes it.
    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    /// Metadata of every non-private rule in the source, enforced or not.
    pub fn rules(&self) -> &[RuleInfo] {
        &self.rules
    }

    /// Whether scans read [`Input::tail`]. Always `false` until facts rules land.
    pub fn needs_facts(&self) -> bool {
        false
    }

    /// Scans one input. Never panics. Masked patterns and integer reads do not allocate;
    /// regex patterns take a search cache from `regex-automata`'s pool, which allocates one
    /// only when none is free.
    pub fn scan(&self, input: Input<'_>) -> Outcome {
        eval::scan(&self.program, input)
    }
}
