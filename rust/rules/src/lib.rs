// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(feature = "bundled", doc = include_str!("../README.md"))]
#![cfg_attr(
    not(feature = "bundled"),
    doc = "Bounded format rules evaluated in pure Rust. See README.md."
)]
#![forbid(unsafe_code)]

mod error;
mod facts;
mod label;
mod magic;
mod source;
#[cfg(feature = "bundled")]
mod table;

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
    /// The input's last bytes, which only zip facts read: the last [`RuleSet::tail_len`] bytes,
    /// or from [`RuleSet::tail_start`] to the end. `None` leaves an archive's directory unknown.
    pub tail: Option<&'a [u8]>,
}

/// The pack's verdict for one input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Distinct indices into [`RuleSet::labels`] of the matching enforced rules, in source order.
    Match(Vec<usize>),
    /// The prefix is empty or is not exactly `min(size, PREFIX_LIMIT)` bytes.
    InsufficientInput,
}

/// A compiled pack. `Send + Sync`; share it through `Arc`.
pub struct RuleSet {
    labels: Vec<String>,
    rules: Vec<RuleInfo>,
}

impl RuleSet {
    /// Compiles every enforced rule of `source`.
    ///
    /// Always returns [`Error::Unsupported`] because the native rule engine does not compile
    /// custom rules at runtime.
    pub fn compile(_source: &Source) -> Result<Self, Error> {
        Err(Error::Unsupported {
            rule: String::new(),
            reason: "custom rules are not supported by the native rule engine".into(),
        })
    }

    /// The rules shipped with this crate.
    #[cfg(feature = "bundled")]
    pub fn bundled() -> Self {
        let (labels, rules) = table::bundled();
        RuleSet { labels, rules }
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
        true
    }

    /// How many of the last bytes of an input a scan should get as [`Input::tail`]: none unless
    /// `prefix` opens a zip archive whose end it does not hold, and only when the prefix alone
    /// does not already decide the subtype (`apk_prefix` / `epub_prefix`).
    pub fn tail_len(&self, prefix: &[u8], size: u64) -> usize {
        if !facts::wants_tail(prefix)
            || size <= prefix.len() as u64
            || magic::decides_zip_from_prefix(prefix)
        {
            return 0;
        }
        size.min(facts::ZIP_TAIL_BYTES as u64) as usize
    }

    /// Where the tail should start instead, when the one read holds the end of a zip archive
    /// whose central directory starts before it: reading from there makes the directory known.
    pub fn tail_start(&self, tail: &[u8], size: u64) -> Option<u64> {
        facts::directory_start(tail, size)
    }

    /// Scans one input through the layered native dispatch cascade. Never panics.
    pub fn scan(&self, input: Input<'_>) -> Outcome {
        magic::scan(input)
    }
}

/// Runs the native magic cascade directly and returns the matched label names, or `None` when
/// [`Input`] fails the prefix window invariant (`Outcome::InsufficientInput`).
#[doc(hidden)]
pub fn scan_native_names(input: Input<'_>) -> Option<Vec<&'static str>> {
    magic::scan_names(input)
}

#[cfg(all(test, feature = "bundled"))]
mod tests {
    use super::*;

    #[test]
    fn bundled_table_matches_enforced_labels_and_content_types() {
        let bundled = RuleSet::bundled();
        assert_eq!(bundled.rules().len(), 192);
        assert_eq!(bundled.labels().len(), 137);

        let mut derived = Vec::new();
        for rule in bundled.rules().iter().filter(|r| r.enforced) {
            let lbl = rule.label.as_ref().expect("enforced rule has label");
            if !derived.contains(lbl) {
                derived.push(lbl.clone());
            }
        }
        assert_eq!(bundled.labels(), &derived[..]);

        let enum_labels: Vec<String> =
            label::Label::ALL.iter().map(|l| l.name().to_string()).collect();
        assert_eq!(bundled.labels(), &enum_labels[..]);

        let mut sorted = derived;
        sorted.sort();
        let content_types = format!("{}\n", sorted.join("\n"));
        assert_eq!(content_types, include_str!("../../gen/rules/content_types"));
    }

    #[test]
    fn notworking_verifiers_match_disabled_table_and_never_fire() {
        let bundled = RuleSet::bundled();
        let disabled_rows: Vec<_> = bundled.rules().iter().filter(|r| !r.enforced).collect();
        assert_eq!(disabled_rows.len(), magic::notworking::DISABLED.len());

        for (row, &(id, lbl, _)) in disabled_rows.iter().zip(magic::notworking::DISABLED.iter()) {
            assert_eq!(row.id, id);
            assert_eq!(row.label.as_deref(), Some(lbl));
            assert_eq!(row.class, Class::NotWorking);
            let has_enforced =
                bundled.rules().iter().any(|r| r.enforced && r.label.as_deref() == Some(lbl));
            if !has_enforced {
                assert!(!bundled.labels().iter().any(|l| l == lbl), "{lbl} should not be enforced");
            }
        }

        let find_verifier = |id: &str| {
            magic::notworking::DISABLED
                .iter()
                .find_map(|&(rid, _, f)| (rid == id).then_some(f))
                .unwrap_or_else(|| panic!("missing disabled verifier {id}"))
        };

        // Bare OLE2 header: matches disabled doc/ppt/outlook, never matched by enforced scan.
        let mut ole2 = vec![0u8; 520];
        ole2[..8].copy_from_slice(b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1");
        ole2[512..516].copy_from_slice(b"\xFD\xFF\xFF\xFF");
        assert!(find_verifier("taxonomy_doc")(&ole2, ole2.len() as u64));
        assert!(find_verifier("taxonomy_ppt")(&ole2, ole2.len() as u64));
        assert!(find_verifier("taxonomy_outlook")(&ole2, ole2.len() as u64));
        assert_eq!(
            bundled.scan(Input { prefix: &ole2, size: ole2.len() as u64, tail: None }),
            Outcome::Match(vec![])
        );

        // Bare ASF header GUID: matches disabled asf/wma/wmv, never matched by enforced scan.
        let mut asf = vec![0u8; 64];
        asf[..16]
            .copy_from_slice(b"\x30\x26\xB2\x75\x8E\x66\xCF\x11\xA6\xD9\x00\xAA\x00\x62\xCE\x6C");
        asf[24] = 1;
        asf[28..30].copy_from_slice(&[0x01, 0x02]);
        assert!(find_verifier("taxonomy_asf")(&asf, asf.len() as u64));
        assert!(find_verifier("taxonomy_wma")(&asf, asf.len() as u64));
        assert!(find_verifier("taxonomy_wmv")(&asf, asf.len() as u64));
        assert_eq!(
            bundled.scan(Input { prefix: &asf, size: asf.len() as u64, tail: None }),
            Outcome::Match(vec![])
        );

        // DocType-less EBML header: matches disabled webm, never matched by enforced scan.
        let mut ebml = vec![0u8; 64];
        ebml[..4].copy_from_slice(b"\x1A\x45\xDF\xA3");
        assert!(find_verifier("taxonomy_webm")(&ebml, ebml.len() as u64));
        assert_eq!(
            bundled.scan(Input { prefix: &ebml, size: ebml.len() as u64, tail: None }),
            Outcome::Match(vec![])
        );
    }

    /// The bundled label of `bytes`, given its tail when `tail`.
    fn label<'a>(rules: &'a RuleSet, bytes: &[u8], tail: bool) -> Option<&'a str> {
        let (prefix, size) = (&bytes[..bytes.len().min(PREFIX_LIMIT)], bytes.len() as u64);
        let tail_len = rules.tail_len(prefix, size);
        let tail = (tail && tail_len > 0).then(|| &bytes[bytes.len() - tail_len..]);
        match rules.scan(Input { prefix, size, tail }) {
            Outcome::Match(labels) => match &labels[..] {
                [label] => Some(&rules.labels()[*label]),
                _ => None,
            },
            Outcome::InsufficientInput => None,
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
        let valid = |tail| facts::Facts::analyze(prefix, size, Some(tail)).is_valid();
        assert!(!valid(tail));
        assert!(valid(&bytes[start as usize..]));
    }
}
