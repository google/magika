// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

/// Everything that can go wrong before a scan. Scans never fail; see `Outcome`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The YARA text is syntactically invalid.
    Parse(String),
    /// A rule's metadata is missing, malformed, or inconsistent with its bucket.
    Metadata {
        /// Rule identifier, empty when the error is not tied to a rule.
        rule: String,
        /// What is wrong.
        reason: String,
    },
    /// A construct outside the supported YARA subset.
    Unsupported {
        /// Rule identifier, empty when the error is not tied to a rule.
        rule: String,
        /// The unsupported construct.
        reason: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "invalid YARA: {e}"),
            Error::Metadata { rule, reason } => write!(f, "rule {rule}: {reason}"),
            Error::Unsupported { rule, reason } => write!(f, "rule {rule}: unsupported: {reason}"),
        }
    }
}

impl std::error::Error for Error {}
