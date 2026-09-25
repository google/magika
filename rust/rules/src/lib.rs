// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(feature = "bundled", doc = include_str!("../README.md"))]
#![cfg_attr(
    not(feature = "bundled"),
    doc = "Bounded format rules: a YARA subset evaluated in pure Rust. See README.md."
)]
#![forbid(unsafe_code)]

#[cfg(test)]
mod codegen;
mod error;
mod eval;
mod facts;
mod ir;
mod lower;
mod matcher;
mod source;

pub use error::Error;
pub use source::{Bucket, Class, RuleInfo, Source};

/// Bytes of input a rule may inspect.
pub const PREFIX_LIMIT: usize = 4096;

// Here rather than in `source.rs`, which `build.rs` compiles too, before `OUT_DIR` holds the rules.
#[cfg(feature = "bundled")]
impl Source {
    /// The rules shipped with this crate. `build.rs` validated them; a failure is a build bug.
    pub fn bundled() -> Self {
        Self::parse(include_str!(concat!(env!("OUT_DIR"), "/bundled.yar")))
            .expect("bundled rules validated at build time")
    }
}

/// What one scan looks at.
#[derive(Clone, Copy, Debug)]
pub struct Input<'a> {
    /// The first `min(size, PREFIX_LIMIT)` bytes of the input, exactly.
    pub prefix: &'a [u8],
    /// The size of the whole input.
    pub size: u64,
    /// The input's last bytes, which only zip facts read: the last [`RuleSet::tail_len`] bytes,
    /// or from [`RuleSet::tail_start`] to the end. `None` leaves an archive's directory unknown.
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

    /// The rules shipped with this crate, as [`RuleSet::compile`] compiles [`Source::bundled`].
    ///
    /// They were compiled when this crate was built, so this parses no YARA and builds no regex:
    /// each regex is built the first time a scan needs it.
    #[cfg(feature = "bundled")]
    pub fn bundled() -> Self {
        include!(concat!(env!("OUT_DIR"), "/bundled.rs"))
    }

    /// Distinct labels of the enforced rules, in source order; [`Outcome::Match`] indexes it.
    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    /// Metadata of every non-private rule in the source, enforced or not.
    pub fn rules(&self) -> &[RuleInfo] {
        &self.rules
    }

    /// Whether a rule reads a fact about an archive or an executable.
    pub fn needs_facts(&self) -> bool {
        self.program.facts
    }

    /// How many of the last bytes of an input a scan should get as [`Input::tail`]: none unless
    /// a rule reads facts and `prefix` opens a zip archive whose end it does not hold.
    pub fn tail_len(&self, prefix: &[u8], size: u64) -> usize {
        if !self.program.facts || !facts::wants_tail(prefix) || size <= prefix.len() as u64 {
            return 0;
        }
        size.min(facts::ZIP_TAIL_BYTES as u64) as usize
    }

    /// Where the tail should start instead, when the one read holds the end of a zip archive
    /// whose central directory starts before it: reading from there makes the directory known.
    pub fn tail_start(&self, tail: &[u8], size: u64) -> Option<u64> {
        facts::directory_start(tail, size)
    }

    /// Scans one input. Never panics. Masked patterns and integer reads do not allocate;
    /// regex patterns take a search cache from `regex-automata`'s pool, which allocates one
    /// only when none is free; the facts of an archive or an executable allocate their views.
    pub fn scan(&self, input: Input<'_>) -> Outcome {
        eval::scan(&self.program, input)
    }
}

#[cfg(all(test, feature = "bundled"))]
mod tests {
    use super::*;

    #[test]
    fn bundled_rules_are_the_compiled_bundled_source() {
        let source = Source::bundled();
        let (program, labels) = lower::lower(&source).unwrap();
        let expected = codegen::rule_set(&program, &labels, source.rules());
        assert!(expected == include_str!(concat!(env!("OUT_DIR"), "/bundled.rs")));
        let bundled = RuleSet::bundled();
        let actual = codegen::rule_set(&bundled.program, &bundled.labels, &bundled.rules);
        assert!(actual == expected);
    }

    /// The bundled label of `bytes`, given its tail when `tail`.
    fn label<'a>(rules: &'a RuleSet, bytes: &[u8], tail: bool) -> Option<&'a str> {
        let (prefix, size) = (&bytes[..bytes.len().min(PREFIX_LIMIT)], bytes.len() as u64);
        let tail_len = rules.tail_len(prefix, size);
        let tail = (tail && tail_len > 0).then(|| &bytes[bytes.len() - tail_len..]);
        match rules.scan(Input { prefix, size, tail }) {
            Outcome::Match(label) => Some(&rules.labels()[label]),
            _ => None,
        }
    }

    #[test]
    fn facts_rules_identify_archives_and_executables() {
        use facts::{archive, pe32, stored};
        let rules = RuleSet::bundled();
        assert!(rules.needs_facts());
        let docx = archive(
            &[
                stored(
                    b"[Content_Types].xml",
                    b"<Types>application/vnd.openxmlformats-officedocument.wordprocessingml.\
                      document.main+xml</Types>",
                ),
                stored(b"word/document.xml", b"<w:document/>"),
            ],
            b"",
        );
        assert_eq!(label(&rules, &docx, true), Some("docx"));
        // Larger than the prefix, so that its central directory is only in the tail.
        let jar = archive(
            &[stored(b"META-INF/MANIFEST.MF", &[b' '; 8192]), stored(b"A.class", b"")],
            b"",
        );
        assert_eq!(rules.tail_len(&jar[..PREFIX_LIMIT], jar.len() as u64), jar.len());
        assert_eq!(label(&rules, &jar, true), Some("jar"));
        assert_eq!(label(&rules, &jar, false), None);
        assert_eq!(label(&rules, &pe32().build(), false), Some("pebin"));
        // A DLL without an entry point holds only resources, like a MUI file: abstain.
        let resources = facts::Image { characteristics: 0x2102, entry_point: 0, ..pe32() };
        assert_eq!(label(&rules, &resources.build(), false), None);
        // Other inputs never need a tail.
        assert_eq!(rules.tail_len(b"%PDF-1.7", 1 << 20), 0);
    }

    #[test]
    fn a_tail_extends_back_to_a_large_central_directory() {
        use facts::{archive, stored};
        let rules = RuleSet::bundled();
        let names: Vec<_> = (0..400).map(|i| format!("{i:0>60}")).collect();
        let entries: Vec<_> = names.iter().map(|name| stored(name.as_bytes(), b"")).collect();
        let bytes = archive(&entries, b"");
        let (prefix, size) = (&bytes[..PREFIX_LIMIT], bytes.len() as u64);
        let tail = &bytes[bytes.len() - rules.tail_len(prefix, size)..];
        let start = rules.tail_start(tail, size).unwrap();
        assert!(start < size - tail.len() as u64);
        let valid = |tail| facts::Facts::analyze(prefix, size, Some(tail)).int(ir::Fact::ZipValid);
        assert_eq!(valid(tail), 0);
        assert_eq!(valid(&bytes[start as usize..]), 1);
    }

    #[test]
    fn build_script_uses_the_same_prefix_limit() {
        let build = include_str!("../build.rs");
        assert!(build.contains(&format!("const PREFIX_LIMIT: usize = {PREFIX_LIMIT};")));
    }
}
