// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Bounded, optional format rules with per-rule evaluation metadata.

use anyhow::Result;

use crate::ContentType;

/// Whether the selected format rules may bypass inference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RulesMode {
    /// Preserve the existing identification pipeline (the default).
    #[default]
    Off,
    /// Use the selected pack, or the bundled allowlist. Requires `yara-rules`.
    ///
    /// An empty allowlist or any scan failure falls through to the existing pipeline.
    /// Runtime construction returns an error if the selected rules cannot be loaded.
    Enforce,
}

impl RulesMode {
    pub(crate) fn check(self) -> Result<()> {
        if self == Self::Enforce && !cfg!(feature = "yara-rules") {
            anyhow::bail!("rules enforcement requires the yara-rules Cargo feature");
        }
        Ok(())
    }
}

pub(crate) const PREFIX_LIMIT: usize = 4096;
/// Size of the stream A header: `original_size` and `prefix_size`, big-endian.
#[cfg(feature = "yara-rules")]
const EXTERNAL_BYTES: usize = 2 * std::mem::size_of::<u64>();

pub(crate) fn identify(prefix: &[u8], original_size: u64, mode: RulesMode) -> Option<ContentType> {
    #[cfg(feature = "yara-rules")]
    if mode == RulesMode::Enforce {
        return engine::scan_promoted(prefix, original_size);
    }
    let _ = (prefix, original_size, mode);
    None
}

/// Bundled YARA source, including disabled rules retained for evaluation.
pub const DEFAULT_RULES: &str = include_str!(concat!(env!("OUT_DIR"), "/bundled-rules.yar"));

#[cfg(feature = "yara-rules")]
mod cache;
#[cfg(feature = "yara-rules")]
mod compiler;
#[cfg(feature = "yara-rules")]
mod engine;
#[cfg(all(feature = "yara-rules", unix))]
mod mapped;
#[cfg(feature = "yara-rules")]
mod metadata;
#[cfg(feature = "yara-rules")]
mod native;
#[cfg(feature = "yara-rules")]
mod preprocess;

/// A compiled, shareable YARA pack executed by Vectorscan.
///
/// Terminal rules require a canonical `label` and `enforced = true` (`enabled` is an alias).
/// They also require `fp_rate = 0`, plus `class = "full"` with `fn_rate = 0`, or
/// `class = "partial"` with `0 < fn_rate < 1`. Rates are fractions: FP / negatives
/// and FN / positives. Untested rules must remain unenforced. Metadata records the
/// caller's evidence; Magika cannot verify its truth when loading a custom pack.
/// Enabling custom rules is the caller's decision; Magika has not qualified their precision.
#[cfg(feature = "yara-rules")]
#[derive(Clone)]
pub struct RuleSet {
    database: Option<std::sync::Arc<native::Database>>,
    loaded_from_cache: bool,
}

#[cfg(feature = "yara-rules")]
impl std::fmt::Debug for RuleSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuleSet").field("empty", &self.database.is_none()).finish()
    }
}

#[cfg(feature = "yara-rules")]
impl RuleSet {
    /// Loads the bundled rules, returning initialization errors to the caller.
    /// Successful loads share the compiled database and use the normal user cache.
    pub fn bundled() -> Result<Self> {
        engine::bundled().cloned()
    }

    /// Reads a YARA file, reusing a compatible compiled database from the user cache when present.
    /// Existing instances retain their loaded contents. Cache failures use in-memory compilation.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        Self::from_file_with_cache(path, cache::default_directory().as_deref())
    }

    /// Loads a YARA file using a selected writable cache directory, or none when `None`.
    /// A paired `.hsdb` file is still considered. Sources and packs must be trusted configuration.
    pub fn from_file_with_cache(
        path: impl AsRef<std::path::Path>, directory: Option<&std::path::Path>,
    ) -> Result<Self> {
        let path = path.as_ref();
        Self::load(&Self::read_source(path)?, directory, Some(&path.with_extension("hsdb")))
    }

    /// Compiles source in memory without disk caching. Unsupported enabled conditions return an error.
    ///
    /// Native compilation requires Vectorscan's compiler library on the library search path,
    /// or an explicit trusted path in `MAGIKA_VECTORSCAN_LIBRARY`.
    pub fn from_source(source: &str) -> Result<Self> {
        Self::load(source, None, None)
    }

    /// Compiles a YARA file into a distributable pack, refusing to overwrite an existing file.
    /// Ship it beside the source with the same basename and `.hsdb` extension.
    /// Empty packs are portable and need no native library; nonempty packs target this engine/CPU.
    pub fn compile_file(
        path: impl AsRef<std::path::Path>, output: impl AsRef<std::path::Path>,
    ) -> Result<()> {
        let source = Self::read_source(path.as_ref())?;
        cache::export(&source, output.as_ref())
    }

    fn read_source(path: &std::path::Path) -> Result<String> {
        #[cfg(feature = "yara-rules")]
        let _startup_span = crate::startup_trace::span("rules_source_read");

        use std::io::Read;
        let mut source = String::new();
        std::fs::File::open(path)?.take(4 * 1024 * 1024 + 1).read_to_string(&mut source)?;
        Ok(source)
    }

    /// Reports whether this load reused a compiled pack from disk (including an empty pack).
    /// This describes initialization only; scans never read or lock the cache.
    pub fn loaded_from_cache(&self) -> bool {
        self.loaded_from_cache
    }

    fn load(
        source: &str, directory: Option<&std::path::Path>, shipped: Option<&std::path::Path>,
    ) -> Result<Self> {
        cache::load(source, directory, shipped)
    }

    /// Identifies an input using only the selected signatures, without loading a model.
    ///
    /// Reads at most the first 4096 bytes and preserves the original file size for guards.
    /// A miss, conflict or scan failure returns `None`; input errors remain errors.
    /// Empty and short text inputs receive no implicit fallback classification.
    pub fn identify_input(&self, mut input: impl crate::Input) -> Result<Option<ContentType>> {
        #[cfg(feature = "yara-rules")]
        let _input_read = crate::startup_trace::span("input_prefix_read");
        let size = input.length()?;
        let mut prefix = vec![0; size.min(PREFIX_LIMIT as u64) as usize];
        if !prefix.is_empty() {
            input.read_at(&mut prefix, 0)?;
        }
        #[cfg(feature = "yara-rules")]
        drop(_input_read);
        Ok(self.identify(&prefix, size, None))
    }

    /// Scans one input: the bounded prefix, its original size and, when the caller has read
    /// one, the trailing block. Preprocessing derives the synthetic stream views from them.
    pub(crate) fn identify(
        &self, prefix: &[u8], size: u64, tail: Option<&[u8]>,
    ) -> Option<ContentType> {
        let database = self.database.as_ref()?;
        // Preprocessing only feeds stream B: a pack without facts rules never pays for it.
        let synthetic = database
            .uses_facts()
            .then(|| preprocess::prepare(&preprocess::Blocks { prefix, size, tail }));
        engine::scan(database, synthetic.as_ref(), prefix, size).content_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_and_unpromoted_never_override() {
        assert_eq!(identify(b"\x89PNG\r\n\x1a\n", 100, RulesMode::Off), None);
        assert_eq!(identify(b"\x89PNG\r\n\x1a\n", 100, RulesMode::Enforce), None);
        assert!(RulesMode::Off.check().is_ok());
        assert_eq!(RulesMode::Enforce.check().is_ok(), cfg!(feature = "yara-rules"));
    }
}

#[cfg(all(test, feature = "yara-rules"))]
mod native_tests {
    use super::*;

    fn rule(id: &str, label: &str, patterns: &str, condition: &str) -> String {
        format!("rule {id} {{ meta: label = \"{label}\" enabled = true class = \"full\" fp_rate = 0 fn_rate = 0 {patterns} condition: {condition} }}")
    }

    #[test]
    fn rules_only_reads_one_bounded_prefix_and_preserves_input_errors() {
        struct Probe {
            size: u64,
            reads: Vec<(u64, usize)>,
            fail: bool,
        }
        impl crate::Input for Probe {
            fn length(&self) -> Result<u64> {
                Ok(self.size)
            }
            fn read_at(&mut self, buffer: &mut [u8], offset: u64) -> Result<()> {
                self.reads.push((offset, buffer.len()));
                anyhow::ensure!(!self.fail, "input read failed");
                buffer.fill(b'A');
                Ok(())
            }
        }
        let pack = RuleSet::from_source("// Empty test pack.").unwrap();
        for size in [0, 1, 4096, 4097, u64::MAX] {
            let mut input = Probe { size, reads: Vec::new(), fail: false };
            assert_eq!(pack.identify_input(&mut input).unwrap(), None);
            assert_eq!(
                input.reads,
                if size == 0 { vec![] } else { vec![(0, size.min(4096) as usize)] }
            );
        }
        let mut broken = Probe { size: 8192, reads: Vec::new(), fail: true };
        assert!(pack
            .identify_input(&mut broken)
            .unwrap_err()
            .to_string()
            .contains("input read failed"));
    }

    #[test]
    fn enforcement_requires_consistent_evaluation_metadata() {
        let source = |meta: &str| {
            format!("rule test {{ meta: label = \"png\" {meta} condition: uint8(0) == 1 }}")
        };
        for meta in [
            r#"enforced = true class = "full" fp_rate = 0 fn_rate = 0"#,
            r#"enforced = true class = "partial" fp_rate = 0 fn_rate = 0.25"#,
            r#"enabled = true class = "partial" fp_rate = 0 fn_rate = 0.25"#,
            r#"enforced = false class = "not-working" fp_rate = "unmeasured" fn_rate = "unmeasured""#,
        ] {
            assert!(compiler::compile(&source(meta)).is_ok(), "{meta}");
        }
        for meta in [
            r#"enforced = true"#,
            r#"enabled = true"#,
            r#"enforced = true class = "partial" fp_rate = 0.000001 fn_rate = 0.25"#,
            r#"enforced = true class = "full" fp_rate = 0 fn_rate = 0.25"#,
            r#"enforced = true class = "partial" fp_rate = 0 fn_rate = 0"#,
            r#"enforced = true class = "partial" fp_rate = 0 fn_rate = 1"#,
            r#"enforced = true class = "not-working" fp_rate = 0 fn_rate = 0"#,
            r#"enforced = true class = "full" fp_rate = "unmeasured" fn_rate = 0"#,
            r#"enabled = false enforced = true class = "full" fp_rate = 0 fn_rate = 0"#,
            r#"enforced = true class = "full" fp_rate = 0 fp_rate = 0 fn_rate = 0"#,
            r#"enforced = true class = "full" fp_rate = -1 fn_rate = 0"#,
        ] {
            assert!(compiler::compile(&source(meta)).is_err(), "{meta}");
        }
        let referenced = format!("{} rule outer {{ meta: label = \"png\" enforced = true class = \"full\" fp_rate = 0 fn_rate = 0 condition: test }}", source(r#"enforced = true class = "partial" fp_rate = 0.01 fn_rate = 0.2"#));
        assert!(compiler::compile(&referenced).is_err());
    }

    #[test]
    fn bundled_source_policy() {
        for notice in [
            "Copyright (c) Ian F. Darwin",
            "Copyright (c) 2013-2025 Chris Griffith",
            "Contains public sector information licensed under the Open Government Licence v3.0.",
            "Copyright 2007-2026 The Apache Software Foundation",
            "Apache License",
            "The MIT License (MIT)",
        ] {
            assert!(DEFAULT_RULES.contains(notice), "missing bundled attribution: {notice}");
        }
        assert!(compiler::compile(DEFAULT_RULES)
            .unwrap()
            .prefix
            .outputs
            .iter()
            .any(Option::is_some));
        assert!(RuleSet::from_source(
            "rule a { meta: label = \"png\" enabled = false condition: filesize > 0 }"
        )
        .unwrap()
        .database
        .is_none());
        for condition in [
            "filesize > 0",
            "uint32(4094) == 0",
            "uint8(original_size - 1) == 0",
            "not true",
            "original_size == uint32(0)",
            "original_size % 0 == 0",
            "original_size % 3 == 0",
            "original_size % 4 >= 0",
            "unknown",
            "for all i in (0..1000): (true)",
            // Views support only `contains`/`startswith` with a literal, inside a view.
            "zip_names icontains \"a\"",
            "zip_names istartswith \"a\"",
            "zip_names endswith \"a\"",
            "zip_names iendswith \"a\"",
            "zip_names matches /a/",
            "zip_names contains zip_names",
            "zip_names contains \"\"",
            "zip_names == \"a\"",
            "pe_machine contains \"a\"",
            "\"a\" contains \"b\"",
            // Preprocessor facts cannot be combined with prefix bytes in one rule.
            "uint8(0) == 1 and pe_valid == 1",
            "uint8(0) == 1 or zip_names contains \"a\"",
        ] {
            assert!(
                RuleSet::from_source(&rule("bad", "png", "", condition)).is_err(),
                "{condition}"
            );
        }
        assert!(RuleSet::from_source(&rule(
            "bad",
            "png",
            "strings: $a = \"AB\"",
            "$a at 0 and zip_valid == 1"
        ))
        .is_err());
        for patterns in
            ["strings: $a = /a.*/", "strings: $a = /^abc/", "strings: $a = { 41 [-] 42 }"]
        {
            assert!(compiler::compile(&rule("bad", "png", patterns, "$a at 0")).is_err());
        }
        for source in [
            "include \"elsewhere.yar\"",
            "import \"pe\"",
            "rule a { meta: label = \"unknown-label\" enabled = true condition: true }",
        ] {
            assert!(RuleSet::from_source(source).is_err());
        }
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn native_bundled_structural_rules_preserve_identifying_fields() {
        let structural: String = DEFAULT_RULES
            .split("\nrule ")
            .skip(1)
            .filter(|source| source.starts_with("xz_v1 {") || source.starts_with("sqlite_v1 {"))
            .map(|source| format!("rule {source}"))
            .collect();
        let pack = RuleSet::from_source(
            &structural
                .replace("enforced = false", "enforced = true")
                .replace("class = \"not-working\"", "class = \"full\"")
                .replace("\"unmeasured\"", "0"),
        )
        .unwrap();
        for (path, expected, identifying_byte) in [
            ("../../tests_data/mitra/xz/xz.xz", ContentType::Xz, 8),
            (
                "../../tests_data/previous_missdetections/sqlite/test-gh-616.db",
                ContentType::Sqlite,
                21,
            ),
        ] {
            let mut data = std::fs::read(path).unwrap();
            let size = data.len() as u64;
            let prefix = data.len().min(PREFIX_LIMIT);
            assert_eq!(pack.identify(&data[..prefix], size, None), Some(expected));
            let mut misaligned = data.clone();
            misaligned.push(0);
            assert_eq!(
                pack.identify(&misaligned[..misaligned.len().min(PREFIX_LIMIT)], size + 1, None),
                None
            );
            data[identifying_byte] ^= 1;
            assert_eq!(pack.identify(&data[..prefix], size, None), None);
        }
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn native_alignment_guards_preserve_unsigned_reads() {
        for (function, width, be) in [
            ("uint8", 1, false),
            ("uint16", 2, false),
            ("uint16be", 2, true),
            ("uint32", 4, false),
            ("uint32be", 4, true),
        ] {
            for (divisor, remainder) in [(1, 0), (4, 0), (4, 3), (256, 255), (65536, 256)] {
                let pack = RuleSet::from_source(&rule(
                    "aligned",
                    "xz",
                    "",
                    &format!("{function}(0) % {divisor} == {remainder}"),
                ))
                .unwrap();
                for value in (0..=256_u32).chain([65535, 65536, u32::MAX]) {
                    let bytes = if be { value.to_be_bytes() } else { value.to_le_bytes() };
                    let bytes = if be { &bytes[4 - width..] } else { &bytes[..width] };
                    let actual = value as u64 & ((1_u64 << (width * 8)) - 1);
                    assert_eq!(
                        pack.identify(bytes, width as u64, None).is_some(),
                        actual % divisor == remainder,
                        "{function}: {actual} % {divisor}"
                    );
                    for end in 0..width {
                        assert!(pack.identify(&bytes[..end], end as u64, None).is_none());
                    }
                }
            }
        }
        let prefix = [0; PREFIX_LIMIT];
        for variable in ["original_size", "prefix_size"] {
            let pack =
                RuleSet::from_source(&rule("size", "xz", "", &format!("{variable} % 4 == 0")))
                    .unwrap();
            for size in [1, 4, 31, 32, 4095, 4096, 4097, u32::MAX as u64 + 1, (1 << 40) + 1] {
                let available = size.min(PREFIX_LIMIT as u64);
                let value = if variable == "original_size" { size } else { available };
                assert_eq!(
                    pack.identify(&prefix[..available as usize], size, None).is_some(),
                    value % 4 == 0
                );
            }
        }
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn native_rule_only_output_survives_cache() {
        // Synthetic markers exercise canonical output transport, not format predicates.
        let labels = [
            "qoi",
            "3dsx",
            "arrow",
            "avro",
            "bam",
            "beam",
            "berkeleydb",
            "blend",
            "bzip3",
            "cram",
            "duckdb",
            "flatgeobuf",
            "gguf",
            "hdf4",
            "llvm_bitcode",
            "lmdb",
            "luabytecode",
            "mat",
            "netcdf",
            "orc",
            "pcapng",
            "postgres_dump",
            "rdata",
            "redis_rdb",
            "sas",
            "spirv",
            "spss",
            "stata",
            "uf2",
            "fbx",
            "fits",
            "gltf",
            "wad",
            "3dsm",
            "access",
            "cinema4d",
            "dbase",
            "filemaker",
            "lightwave",
            "paradox",
            "shapefile",
            "sketchup",
        ];
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("custom.yar");
        let source: String = labels
            .iter()
            .enumerate()
            .map(|(index, label)| {
                rule(
                    &format!("custom_{index}"),
                    label,
                    &format!("strings: $a = \"{label}!\""),
                    &format!("$a at 0 and original_size == {}", label.len() + 1),
                )
            })
            .collect();
        std::fs::write(&path, source).unwrap();
        for cached in [false, true] {
            let pack = RuleSet::from_file_with_cache(&path, Some(directory.path())).unwrap();
            assert_eq!(pack.loaded_from_cache(), cached);
            for label in labels {
                let marker = format!("{label}!");
                let content_type =
                    pack.identify(marker.as_bytes(), marker.len() as u64, None).unwrap();
                assert_eq!(Some(content_type), ContentType::from_label(label));
                assert_eq!(crate::FileType::Ruled(content_type).info().label, label);
            }
            assert_eq!(pack.identify(b"miss", 4, None), None);
        }
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn native_coordination_and_thread_reuse() {
        let pack = RuleSet::from_source(&format!(
            "{}\n{}",
            rule(
                "a",
                "png",
                "strings: $a = \"AB\" $b = { 43 44 } $c = /E[F-G]/ $d = \"HI\"",
                "original_size >= 4 and (($a at 0 and $b at 2) or ($c at 0 and $d at 2))"
            ),
            rule("b", "gif", "strings: $a = \"ABCD\"", "$a at 0 and original_size == 6")
        ))
        .unwrap();
        for (data, expected) in [
            (&b"ABCD"[..], Some(ContentType::Png)),
            (b"EGHI", Some(ContentType::Png)),
            (b"ABHI", None),
            (b"EGCD", None),
            (b"ABCDxx", None),
            (b"ABC", None),
            (b"xABCD", None),
        ] {
            assert_eq!(pack.identify(data, data.len() as u64, None), expected, "{data:?}");
        }
        assert_eq!(pack.identify(b"ABCD", 5000, None), None); // Caller did not supply the available prefix.
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let pack = pack.clone();
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        assert_eq!(pack.identify(b"ABCD", 4, None), Some(ContentType::Png));
                        assert_eq!(pack.identify(b"ABCDxx", 6, None), None);
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
        let other =
            RuleSet::from_source(&rule("other", "gif", "strings: $a = \"ABCD\"", "$a at 0"))
                .unwrap();
        assert_eq!(other.identify(b"ABCD", 4, None), Some(ContentType::Gif));
        assert_eq!(pack.identify(b"ABCD", 4, None), Some(ContentType::Png));
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn native_ascii_alternatives_preserve_byte_semantics() {
        for (pattern, alternatives) in [
            ("{ ( 00 | 01 | 3F | 7F ) 00 }", &[0, 1, 63, 127][..]),
            (r"/(\x00|\x01|\x3f|\x7f)\x00/", &[0, 1, 63, 127][..]),
            (r"/(a|b)\x00/", &b"ab"[..]),
            ("{ ( 00 | FF ) 00 }", &[0, 255][..]),
        ] {
            let pack = RuleSet::from_source(&rule(
                "alternatives",
                "unknown",
                &format!("strings: $a = {pattern}"),
                "$a at 0",
            ))
            .unwrap();
            for value in 0..=255_u8 {
                assert_eq!(
                    pack.identify(&[value, 0], 2, None).is_some(),
                    alternatives.contains(&value),
                    "{pattern}: {value}"
                );
                assert_eq!(pack.identify(&[value], 1, None), None);
                assert_eq!(pack.identify(&[value, 1], 2, None), None);
                assert_eq!(pack.identify(&[128, value, 0], 3, None), None);
            }
        }
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn native_integer_predicates_and_actual_length() {
        for (op, predicate) in [
            ("==", (|a, b| a == b) as fn(u32, u32) -> bool),
            ("!=", |a, b| a != b),
            ("<", |a, b| a < b),
            ("<=", |a, b| a <= b),
            (">", |a, b| a > b),
            (">=", |a, b| a >= b),
        ] {
            for (read, width, be, threshold) in [
                ("uint8", 1, false, 129),
                ("uint16", 2, false, 32769),
                ("uint16be", 2, true, 32769),
                ("uint32", 4, false, 0x80000100),
                ("uint32be", 4, true, 0x80000100),
            ] {
                let pack = RuleSet::from_source(&rule(
                    "a",
                    "png",
                    "",
                    &format!("{read}(0) {op} {threshold}"),
                ))
                .unwrap();
                let values: Vec<u32> = if width <= 2 {
                    (0..1 << (width * 8)).collect()
                } else {
                    vec![0, 1, threshold - 1, threshold, threshold + 1, u32::MAX]
                };
                for value in values {
                    let bytes = if be {
                        value.to_be_bytes()[4 - width..].to_vec()
                    } else {
                        value.to_le_bytes()[..width].to_vec()
                    };
                    assert_eq!(
                        pack.identify(&bytes, width as u64, None).is_some(),
                        predicate(value, threshold),
                        "{read} {op} {threshold}, input {value}"
                    );
                }
                for len in 1..width {
                    assert_eq!(pack.identify(&vec![0; len], len as u64, None), None);
                }
            }
        }
        let pack = RuleSet::from_source(&rule(
            "a",
            "png",
            "strings: $a = \"AB\"",
            "$a at 0 and original_size == 5000 and prefix_size == 4096",
        ))
        .unwrap();
        let mut bytes = vec![0; 4096];
        bytes[..2].copy_from_slice(b"AB");
        for size in [5000, 6000, 4096, 5000, u64::MAX, 5000] {
            assert_eq!(
                pack.identify(&bytes, size, None),
                (size == 5000).then_some(ContentType::Png)
            );
        }
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn native_pattern_placement_respects_at_and_in_bounds() {
        // The exact `at` placement and the inclusive `in` range, on fixed and variable widths.
        let at =
            RuleSet::from_source(&rule("exact", "png", "strings: $a = \"AB\"", "$a at 5")).unwrap();
        for (data, expected) in [
            (&b"xxxxxAB"[..], Some(ContentType::Png)),
            (b"xxxxAB", None),
            (b"xxxxxxAB", None),
            (b"xxxxxABxx", Some(ContentType::Png)),
            (b"ABxxxAB", Some(ContentType::Png)),
            (b"xxxxxA", None),
        ] {
            assert_eq!(at.identify(data, data.len() as u64, None), expected, "{data:?}");
        }
        let range =
            RuleSet::from_source(&rule("ranged", "png", "strings: $a = \"AB\"", "$a in (3..5)"))
                .unwrap();
        for placement in 0..8 {
            let mut data = vec![b'x'; 10];
            data[placement..placement + 2].copy_from_slice(b"AB");
            assert_eq!(
                range.identify(&data, data.len() as u64, None),
                (3..=5).contains(&placement).then_some(ContentType::Png),
                "placement {placement}"
            );
        }
        // Bounds still apply when the same literal also appears elsewhere, and a
        // combination needs every operand satisfied within its own bounds.
        let both = RuleSet::from_source(&rule(
            "both",
            "png",
            "strings: $a = \"AB\" $b = { 43 [1-2] 44 }",
            "$a at 0 and $b in (2..3)",
        ))
        .unwrap();
        for (data, expected) in [
            (&b"ABCxD"[..], Some(ContentType::Png)),
            (b"ABxCxxD", Some(ContentType::Png)),
            (b"ABxxCxD", None),
            (b"xABCxD", None),
            (b"ABABCxD", None),
        ] {
            assert_eq!(both.identify(data, data.len() as u64, None), expected, "{data:?}");
        }
    }

    /// Scans with a hand-built stream B, bypassing the preprocessors.
    fn scan(pack: &RuleSet, synthetic: &preprocess::Synthetic, data: &[u8]) -> Option<ContentType> {
        let database = pack.database.as_ref().unwrap();
        engine::scan(database, Some(synthetic), data, data.len() as u64).content_type()
    }

    fn synthetic(data: &[u8]) -> preprocess::Synthetic {
        preprocess::prepare(&preprocess::Blocks {
            prefix: data,
            size: data.len() as u64,
            tail: None,
        })
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn native_facts_rules_scan_only_an_active_facts_stream() {
        let pack = RuleSet::from_source(&rule(
            "document",
            "docx",
            "",
            "zip_valid == 1 and zip_names contains \"word/document.xml\"",
        ))
        .unwrap();
        let mut zip = synthetic(b"PK");
        assert_eq!(scan(&pack, &zip, b"PK"), None);
        preprocess::write_fact(&mut zip.facts, preprocess::fact("zip_valid").unwrap(), 1);
        assert_eq!(scan(&pack, &zip, b"PK"), None);
        zip.names[..19].copy_from_slice(b"\nword/document.xml\n");
        assert_eq!(scan(&pack, &zip, b"PK"), Some(ContentType::Docx));
        zip.facts[16] = 0;
        assert_eq!(scan(&pack, &zip, b"PK"), None);
        // Nothing preprocessed: stream B is not scanned at all, even for a facts rule whose
        // atoms an all-zero header would satisfy.
        let hollow = RuleSet::from_source(&rule(
            "hollow",
            "png",
            "",
            "pe_valid == 0 and original_size >= 0",
        ))
        .unwrap();
        assert_eq!(scan(&hollow, &synthetic(b"PK"), b"PK"), None);
        assert_eq!(hollow.identify(b"PK", 2, None), None);
        let mut active = synthetic(b"PK");
        active.names[0] = b'\n';
        assert_eq!(scan(&hollow, &active, b"PK"), Some(ContentType::Png));
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn native_prefix_only_packs_skip_preprocessing() {
        let signature = rule("signature", "png", "strings: $a = \"AB\"", "$a at 0");
        let prefix_only = RuleSet::from_source(&signature).unwrap();
        let empty = RuleSet::from_source("// Empty test pack.").unwrap();
        let facts = RuleSet::from_source(&rule("fact", "gif", "", "pe_valid == 1")).unwrap();
        let before = preprocess::PREPARATIONS.get();
        assert_eq!(prefix_only.identify(b"AB", 2, None), Some(ContentType::Png));
        assert_eq!(prefix_only.identify(b"xx", 2, None), None);
        assert_eq!(empty.identify(b"AB", 2, None), None);
        assert_eq!(preprocess::PREPARATIONS.get(), before, "no facts stream, no preprocessing");
        assert_eq!(facts.identify(b"AB", 2, None), None);
        assert_eq!(preprocess::PREPARATIONS.get(), before + 1);
        let both = RuleSet::from_source(&format!(
            "{signature}{}",
            rule("fact", "gif", "", "pe_valid == 1")
        ))
        .unwrap();
        assert_eq!(both.identify(b"AB", 2, None), Some(ContentType::Png));
        assert_eq!(preprocess::PREPARATIONS.get(), before + 2);
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn native_view_membership_respects_view_boundaries() {
        let contains =
            RuleSet::from_source(&rule("view", "png", "", "zip_names contains \"needle\""))
                .unwrap();
        let starts =
            RuleSet::from_source(&rule("view", "png", "", "zip_names startswith \"needle\""))
                .unwrap();
        // A nonempty view starts with its `\n` separator; only `startswith` sees offset 0.
        for at in [0, 1, 100, preprocess::ZIP_NAMES_BYTES - 6] {
            let mut names = synthetic(b"x");
            names.names[0] = b'\n';
            names.names[at..at + 6].copy_from_slice(b"needle");
            assert_eq!(scan(&contains, &names, b"x"), Some(ContentType::Png), "{at}");
            assert_eq!(scan(&starts, &names, b"x"), (at == 0).then_some(ContentType::Png), "{at}");
        }
        // Cut off by the end of the view.
        let mut cut = synthetic(b"x");
        cut.names[0] = b'\n';
        cut.names[preprocess::ZIP_NAMES_BYTES - 3..].copy_from_slice(b"nee");
        assert_eq!(scan(&contains, &cut, b"x"), None);
        // Inside the facts header only: active, but not part of the view.
        let mut header = synthetic(b"x");
        header.facts[20..26].copy_from_slice(b"needle");
        assert!(header.is_active());
        assert_eq!(scan(&contains, &header, b"x"), None);
        // Straddling the header and the view.
        let mut straddling = synthetic(b"x");
        straddling.facts[61..].copy_from_slice(b"nee");
        straddling.names[..3].copy_from_slice(b"dle");
        assert_eq!(scan(&contains, &straddling, b"x"), None);
        assert_eq!(scan(&starts, &straddling, b"x"), None);
        // The prefix is never the view.
        assert_eq!(contains.identify(b"needle", 6, None), None);
        let mut active = synthetic(b"needle");
        active.names[0] = b'\n';
        assert_eq!(scan(&contains, &active, b"needle"), None);
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn native_streams_report_into_one_decision() {
        let signature = rule("signature", "png", "strings: $a = \"AB\"", "$a at 0");
        let agreeing = RuleSet::from_source(&format!(
            "{signature}{}",
            rule("fact", "png", "", "pe_valid == 1")
        ))
        .unwrap();
        let disagreeing = RuleSet::from_source(&format!(
            "{signature}{}",
            rule("fact", "gif", "", "pe_valid == 1")
        ))
        .unwrap();
        let mut executable = synthetic(b"AB");
        preprocess::write_fact(&mut executable.facts, preprocess::fact("pe_valid").unwrap(), 1);
        assert_eq!(scan(&agreeing, &executable, b"AB"), Some(ContentType::Png));
        assert_eq!(scan(&disagreeing, &executable, b"AB"), None, "conflict across streams");
        assert_eq!(scan(&disagreeing, &synthetic(b"AB"), b"AB"), Some(ContentType::Png));
        assert_eq!(scan(&disagreeing, &executable, b"xx"), Some(ContentType::Gif));
        assert_eq!(scan(&disagreeing, &synthetic(b"xx"), b"xx"), None);
    }

    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn native_two_stream_packs_survive_the_cache() {
        let signature = rule("signature", "png", "strings: $a = \"AB\"", "$a at 0");
        let fact = rule("fact", "gif", "", "pe_valid == 1 and zip_names startswith \"\\n\"");
        let directory = tempfile::tempdir().unwrap();
        let mut executable = synthetic(b"AB");
        preprocess::write_fact(&mut executable.facts, preprocess::fact("pe_valid").unwrap(), 1);
        executable.names[0] = b'\n';
        for (name, source, prefix, facts) in [
            ("both", format!("{signature}{fact}"), Some(ContentType::Png), Some(ContentType::Gif)),
            ("prefix", signature.clone(), Some(ContentType::Png), None),
            ("facts", fact.clone(), None, Some(ContentType::Gif)),
        ] {
            let path = directory.path().join(format!("{name}.yar"));
            std::fs::write(&path, source).unwrap();
            for cached in [false, true] {
                let pack = RuleSet::from_file_with_cache(&path, Some(directory.path())).unwrap();
                assert_eq!(pack.loaded_from_cache(), cached, "{name}");
                assert_eq!(pack.identify(b"AB", 2, None), prefix, "{name} {cached}");
                assert_eq!(scan(&pack, &executable, b"xx"), facts, "{name} {cached}");
                assert_eq!(scan(&pack, &synthetic(b"xx"), b"xx"), None, "{name} {cached}");
            }
        }
    }
}
