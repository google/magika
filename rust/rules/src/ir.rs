// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! The compiled form of a pack: one condition tree per enforced rule over shared patterns.

use crate::matcher::Pattern;

pub(crate) struct Program {
    pub(crate) patterns: Vec<Pattern>,
    /// Enforced rules only, in source order.
    pub(crate) rules: Vec<Rule>,
}

pub(crate) struct Rule {
    /// Index into `RuleSet::labels`.
    pub(crate) label: usize,
    pub(crate) cond: Cond,
}

/// Integer read from the prefix; out-of-range reads are `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Read {
    U8,
    U16Le,
    U16Be,
    U32Le,
    U32Be,
}

/// Integer operand. Values are compared as `i128`, so every size and read is exact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Int {
    Const(i64),
    /// The whole input's size: `filesize` or `original_size`.
    FileSize,
    /// Bytes available in the prefix: `prefix_size`.
    PrefixSize,
    Read(Read, Box<Int>),
    /// `x % 2^k`, lowered as `x & (2^k - 1)`.
    And(Box<Int>, Box<Int>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cmp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// Boolean condition. `At`/`In` index `Program::patterns`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Cond {
    True,
    False,
    Not(Box<Cond>),
    And(Vec<Cond>),
    Or(Vec<Cond>),
    Cmp(Cmp, Int, Int),
    /// `$p at off`
    At {
        pattern: usize,
        offset: Int,
    },
    /// `$p in (lo..hi)`: a match whose start lies in `lo..=hi`.
    In {
        pattern: usize,
        lo: Int,
        hi: Int,
    },
}
