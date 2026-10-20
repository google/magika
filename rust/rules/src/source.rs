// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Rule metadata and source types.

use crate::Error;

/// Which ruleset directory a rule ships in; checked against its metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bucket {
    /// `rulesets/full`: enforced, no observed false positive or false negative.
    Full,
    /// `rulesets/partial`: enforced, no observed false positive, some false negatives.
    Partial,
    /// `rulesets/notworking`: kept for reference, never enforced.
    NotWorking,
}

/// Declared coverage class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// `class = "full"`.
    Full,
    /// `class = "partial"`.
    Partial,
    /// `class = "not-working"`.
    NotWorking,
    /// No `class` metadata.
    Untested,
}

/// One rule's metadata, in source order (private rules excluded).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleInfo {
    /// Rule identifier.
    pub id: String,
    /// The label the rule assigns; required when enforced.
    pub label: Option<String>,
    /// Declared coverage class.
    pub class: Class,
    /// Whether the rule takes part in scans.
    pub enforced: bool,
    /// The bucket the rule was validated against, if any.
    pub bucket: Option<Bucket>,
}

/// Validated rule source text.
///
/// The native rule engine uses compiled Rust verifiers and does not parse custom rules at
/// runtime; [`Source::parse`] and [`Source::parse_in_bucket`] always return
/// [`Error::Unsupported`].
#[derive(Clone, Debug)]
pub struct Source {
    text: String,
    rules: Vec<RuleInfo>,
}

impl Source {
    /// Largest accepted source.
    pub const MAX_BYTES: usize = 4 * 1024 * 1024;

    /// Parses and validates `text`.
    ///
    /// Always returns [`Error::Unsupported`] because the native rule engine does not compile
    /// custom rules at runtime.
    pub fn parse(_text: &str) -> Result<Self, Error> {
        Err(Error::Unsupported {
            rule: String::new(),
            reason: "custom rules are not supported by the native rule engine".into(),
        })
    }

    /// Like `parse`, also checking every rule's metadata agrees with `bucket`.
    ///
    /// Always returns [`Error::Unsupported`] because the native rule engine does not compile
    /// custom rules at runtime.
    pub fn parse_in_bucket(_text: &str, _bucket: Bucket) -> Result<Self, Error> {
        Err(Error::Unsupported {
            rule: String::new(),
            reason: "custom rules are not supported by the native rule engine".into(),
        })
    }

    /// The validated rule text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Metadata of every non-private rule, in source order.
    pub fn rules(&self) -> &[RuleInfo] {
        &self.rules
    }
}
