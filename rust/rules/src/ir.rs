// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! The compiled form of a pack: one condition tree per enforced rule over shared patterns.

use crate::matcher::Pattern;

pub(crate) struct Program {
    pub(crate) patterns: Vec<Pattern>,
    /// Enforced rules only, in source order.
    pub(crate) rules: Vec<Rule>,
    /// Whether a rule reads a fact or a view.
    pub(crate) facts: bool,
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
    /// A fact about an archive or an executable, zero when the input is neither.
    Fact(Fact),
}

/// Integer facts about an archive or an executable; see `facts` for how each is derived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fact {
    ZipValid,
    ZipFlags,
    ZipEntries,
    ZipNamesEntries,
    ZipNamesLen,
    ZipCommentLen,
    ZipCdSize,
    ZipFirstEntryLen,
    PeValid,
    PeFlags,
    PeMachine,
    PeCharacteristics,
    PeSubsystem,
    PeEntryPoint,
    PeDllCharacteristics,
    PeSections,
    PeMagic,
    PeClr,
    PeSigned,
    PeOverlay,
    PeIsDll,
    PeIsExecutableImage,
}

impl Fact {
    /// Every fact, by the identifier rules name it with.
    pub(crate) const NAMES: [(&'static str, Fact); 22] = [
        ("zip_valid", Fact::ZipValid),
        ("zip_flags", Fact::ZipFlags),
        ("zip_entries", Fact::ZipEntries),
        ("zip_names_entries", Fact::ZipNamesEntries),
        ("zip_names_len", Fact::ZipNamesLen),
        ("zip_comment_len", Fact::ZipCommentLen),
        ("zip_cd_size", Fact::ZipCdSize),
        ("zip_first_entry_len", Fact::ZipFirstEntryLen),
        ("pe_valid", Fact::PeValid),
        ("pe_flags", Fact::PeFlags),
        ("pe_machine", Fact::PeMachine),
        ("pe_characteristics", Fact::PeCharacteristics),
        ("pe_subsystem", Fact::PeSubsystem),
        ("pe_entry_point", Fact::PeEntryPoint),
        ("pe_dll_characteristics", Fact::PeDllCharacteristics),
        ("pe_sections", Fact::PeSections),
        ("pe_magic", Fact::PeMagic),
        ("pe_clr", Fact::PeClr),
        ("pe_signed", Fact::PeSigned),
        ("pe_overlay", Fact::PeOverlay),
        ("pe_is_dll", Fact::PeIsDll),
        ("pe_is_executable_image", Fact::PeIsExecutableImage),
    ];

    pub(crate) fn from_name(name: &str) -> Option<Fact> {
        Fact::NAMES.iter().find(|(x, _)| *x == name).map(|(_, fact)| *fact)
    }
}

/// Text derived from an archive, which rules search with `contains` and `startswith`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum View {
    /// The central directory names, each line `\n`-led, closed by a final `\n`.
    ZipNames,
    /// The first entry's name, `\n`, then its data.
    ZipFirstEntry,
}

impl View {
    pub(crate) fn from_name(name: &str) -> Option<View> {
        match name {
            "zip_names" => Some(View::ZipNames),
            "zip_first_entry" => Some(View::ZipFirstEntry),
            _ => None,
        }
    }
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

impl Cond {
    /// Whether the condition reads a fact or a view.
    pub(crate) fn uses_facts(&self) -> bool {
        match self {
            Cond::True | Cond::False => false,
            Cond::Not(x) => x.uses_facts(),
            Cond::And(xs) | Cond::Or(xs) => xs.iter().any(Cond::uses_facts),
            Cond::Cmp(_, a, b) => a.uses_facts() || b.uses_facts(),
            Cond::At { offset, .. } => offset.uses_facts(),
            Cond::In { lo, hi, .. } => lo.uses_facts() || hi.uses_facts(),
            Cond::View { .. } => true,
        }
    }
}

impl Int {
    fn uses_facts(&self) -> bool {
        match self {
            Int::Const(_) | Int::FileSize | Int::PrefixSize => false,
            Int::Read(_, x) => x.uses_facts(),
            Int::And(a, b) => a.uses_facts() || b.uses_facts(),
            Int::Fact(_) => true,
        }
    }
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
    /// `view contains "text"`, or `view startswith "text"` when `start`.
    View {
        view: View,
        text: Vec<u8>,
        start: bool,
    },
}
