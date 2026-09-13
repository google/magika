//! End-to-end scans over byte samples, including the engine tests ported from #1447.

use magika_rules::{Input, Outcome, RuleSet, Source, PREFIX_LIMIT};

const PACK: &str = r#"
rule png { meta: label = "png" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = { 89 50 4E 47 0D 0A 1A 0A } condition: $a at 0 and filesize >= 8 }
rule png_twin { meta: label = "png" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = "PNG" condition: $a at 1 }
rule gif { meta: label = "gif" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = "GIF8" condition: $a at 0 and uint8(4) == 0x37 }
rule clash { meta: label = "clash" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = "GIF89a" condition: $a at 0 }
rule gif_any { meta: label = "gif" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = "GIF8" condition: $a at 0 and uint8(4) == 0x39 }
rule off { meta: label = "elf" enforced = false class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = { 7F 45 4C 46 } condition: $a at 0 }
rule mp4 { meta: label = "mp4" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = "ftyp" condition: $a in (4..4) and uint32be(0) >= 8 and uint32be(0) <= filesize }
"#;

fn compile(text: &str) -> RuleSet {
    RuleSet::compile(&Source::parse(text).unwrap()).unwrap()
}

fn scan(rules: &RuleSet, bytes: &[u8]) -> Outcome {
    scan_sized(rules, bytes, bytes.len() as u64)
}

fn scan_sized(rules: &RuleSet, prefix: &[u8], size: u64) -> Outcome {
    rules.scan(Input { prefix, size, tail: None })
}

fn matched(outcome: Outcome) -> bool {
    matches!(outcome, Outcome::Match(_))
}

#[test]
fn matches_agreeing_rules_and_reports_conflicts() {
    let rules = compile(PACK);
    assert_eq!(rules.labels(), ["png", "gif", "clash", "mp4"]);
    assert_eq!(scan(&rules, b"\x89PNG\r\n\x1a\nrest"), Outcome::Match(0));
    assert_eq!(scan(&rules, b"GIF87a...."), Outcome::Match(1));
    assert_eq!(scan(&rules, b"GIF89a...."), Outcome::Conflict, "gif_any and clash disagree");
    assert_eq!(scan(&rules, b"nothing"), Outcome::NoMatch);
    assert_eq!(scan(&rules, b"\x7fELF...."), Outcome::NoMatch, "unenforced rules never fire");
}

#[test]
fn integer_reads_and_filesize_guards() {
    let rules = compile(PACK);
    assert_eq!(
        scan(&rules, b"\x00\x00\x00\x14ftypisom\x00\x00\x00\x00\x00\x00\x00\x00"),
        Outcome::Match(3)
    );
    assert_eq!(scan(&rules, b"\x00\x00\x00\x04ftyp"), Outcome::NoMatch, "box size below 8");
    assert_eq!(scan(&rules, b"\x00\x00\xff\xffftyp"), Outcome::NoMatch, "box size above filesize");
}

#[test]
fn prefix_must_be_exactly_the_bounded_head() {
    let rules = compile(PACK);
    assert_eq!(scan_sized(&rules, b"", 0), Outcome::InsufficientInput);
    assert_eq!(scan_sized(&rules, b"GIF87a", 100), Outcome::InsufficientInput);
    let big = vec![0u8; PREFIX_LIMIT];
    assert_eq!(scan_sized(&rules, &big, 1 << 40), Outcome::NoMatch);
}

#[test]
fn rules_are_send_sync_and_shareable() {
    let rules = std::sync::Arc::new(compile(PACK));
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let r = rules.clone();
            std::thread::spawn(move || (0..1000).all(|_| scan(&r, b"GIF87a") == Outcome::Match(1)))
        })
        .collect();
    assert!(handles.into_iter().all(|h| h.join().unwrap()));
}

#[test]
fn unsupported_conditions_are_rejected_at_compile() {
    for cond in [
        "#a > 1",
        "@a[1] == 0",
        "all of them",
        "for any i in (1..2): (true)",
        "$a",
        "$a in (0..filesize)",
    ] {
        let text = format!(
            r#"rule r {{ meta: label = "png" enforced = true class = "full" fp_rate = 0 fn_rate = 0 strings: $a = "x" condition: {cond} }}"#
        );
        assert!(RuleSet::compile(&Source::parse(&text).unwrap()).is_err(), "{cond}");
    }
}

// Ported from #1447 `rust/lib/src/rules.rs` native tests.

fn rule(id: &str, label: &str, patterns: &str, condition: &str) -> String {
    format!("rule {id} {{ meta: label = \"{label}\" enabled = true class = \"full\" fp_rate = 0 fn_rate = 0 {patterns} condition: {condition} }}")
}

#[test]
fn bundled_structural_rules_preserve_identifying_fields() {
    let bundled = Source::bundled();
    let structural: String = bundled
        .text()
        .split("\nrule ")
        .skip(1)
        .filter(|source| source.starts_with("xz_v1 {") || source.starts_with("sqlite_v1 {"))
        .map(|source| format!("rule {source}"))
        .collect();
    let pack = compile(
        &structural
            .replace("enforced = false", "enforced = true")
            .replace("class = \"not-working\"", "class = \"full\"")
            .replace("\"unmeasured\"", "0"),
    );
    for (path, expected, identifying_byte) in [
        ("../../tests_data/mitra/xz/xz.xz", "xz", 8),
        ("../../tests_data/previous_missdetections/sqlite/test-gh-616.db", "sqlite", 21),
    ] {
        let label = pack.labels().iter().position(|l| l == expected).unwrap();
        let mut data = std::fs::read(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let size = data.len() as u64;
        let prefix = data.len().min(PREFIX_LIMIT);
        assert_eq!(scan_sized(&pack, &data[..prefix], size), Outcome::Match(label));
        let mut misaligned = data.clone();
        misaligned.push(0);
        assert_eq!(
            scan_sized(&pack, &misaligned[..misaligned.len().min(PREFIX_LIMIT)], size + 1),
            Outcome::NoMatch
        );
        data[identifying_byte] ^= 1;
        assert_eq!(scan_sized(&pack, &data[..prefix], size), Outcome::NoMatch);
    }
}

#[test]
fn alignment_guards_preserve_unsigned_reads() {
    for (function, width, be) in [
        ("uint8", 1, false),
        ("uint16", 2, false),
        ("uint16be", 2, true),
        ("uint32", 4, false),
        ("uint32be", 4, true),
    ] {
        for (divisor, remainder) in [(1, 0), (4, 0), (4, 3), (256, 255), (65536, 256)] {
            let pack = compile(&rule(
                "aligned",
                "xz",
                "",
                &format!("{function}(0) % {divisor} == {remainder}"),
            ));
            for value in (0..=256_u32).chain([65535, 65536, u32::MAX]) {
                let bytes = if be { value.to_be_bytes() } else { value.to_le_bytes() };
                let bytes = if be { &bytes[4 - width..] } else { &bytes[..width] };
                let actual = value as u64 & ((1_u64 << (width * 8)) - 1);
                assert_eq!(
                    matched(scan(&pack, bytes)),
                    actual % divisor == remainder,
                    "{function}: {actual} % {divisor}"
                );
                for end in 0..width {
                    assert!(!matched(scan(&pack, &bytes[..end])));
                }
            }
        }
    }
    let prefix = [0; PREFIX_LIMIT];
    for variable in ["original_size", "prefix_size"] {
        let pack = compile(&rule("size", "xz", "", &format!("{variable} % 4 == 0")));
        for size in [1, 4, 31, 32, 4095, 4096, 4097, u32::MAX as u64 + 1, (1 << 40) + 1] {
            let available = size.min(PREFIX_LIMIT as u64);
            let value = if variable == "original_size" { size } else { available };
            assert_eq!(
                matched(scan_sized(&pack, &prefix[..available as usize], size)),
                value % 4 == 0
            );
        }
    }
}

#[test]
fn ascii_alternatives_preserve_byte_semantics() {
    for (pattern, alternatives) in [
        ("{ ( 00 | 01 | 3F | 7F ) 00 }", &[0, 1, 63, 127][..]),
        (r"/(\x00|\x01|\x3f|\x7f)\x00/", &[0, 1, 63, 127][..]),
        (r"/(a|b)\x00/", &b"ab"[..]),
        ("{ ( 00 | FF ) 00 }", &[0, 255][..]),
    ] {
        let pack = compile(&rule(
            "alternatives",
            "unknown",
            &format!("strings: $a = {pattern}"),
            "$a at 0",
        ));
        for value in 0..=255_u8 {
            assert_eq!(
                matched(scan(&pack, &[value, 0])),
                alternatives.contains(&value),
                "{pattern}: {value}"
            );
            assert!(!matched(scan(&pack, &[value])));
            assert!(!matched(scan(&pack, &[value, 1])));
            assert!(!matched(scan(&pack, &[128, value, 0])));
        }
    }
}

#[test]
fn integer_predicates_and_actual_length() {
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
            let pack = compile(&rule("a", "png", "", &format!("{read}(0) {op} {threshold}")));
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
                    matched(scan(&pack, &bytes)),
                    predicate(value, threshold),
                    "{read} {op} {threshold}, input {value}"
                );
            }
            for len in 1..width {
                assert!(!matched(scan(&pack, &vec![0; len])));
            }
        }
    }
    let pack = compile(&rule(
        "a",
        "png",
        "strings: $a = \"AB\"",
        "$a at 0 and original_size == 5000 and prefix_size == 4096",
    ));
    let mut bytes = vec![0; 4096];
    bytes[..2].copy_from_slice(b"AB");
    for size in [5000, 6000, 4096, 5000, u64::MAX, 5000] {
        assert_eq!(
            scan_sized(&pack, &bytes, size),
            if size == 5000 { Outcome::Match(0) } else { Outcome::NoMatch }
        );
    }
}

#[test]
fn pattern_placement_respects_at_and_in_bounds() {
    // The exact `at` placement and the inclusive `in` range, on fixed and variable widths.
    let at = compile(&rule("exact", "png", "strings: $a = \"AB\"", "$a at 5"));
    for (data, expected) in [
        (&b"xxxxxAB"[..], true),
        (b"xxxxAB", false),
        (b"xxxxxxAB", false),
        (b"xxxxxABxx", true),
        (b"ABxxxAB", true),
        (b"xxxxxA", false),
    ] {
        assert_eq!(matched(scan(&at, data)), expected, "{data:?}");
    }
    let range = compile(&rule("ranged", "png", "strings: $a = \"AB\"", "$a in (3..5)"));
    for placement in 0..8 {
        let mut data = vec![b'x'; 10];
        data[placement..placement + 2].copy_from_slice(b"AB");
        assert_eq!(
            matched(scan(&range, &data)),
            (3..=5).contains(&placement),
            "placement {placement}"
        );
    }
    // Bounds still apply when the same literal also appears elsewhere, and a
    // combination needs every operand satisfied within its own bounds.
    let both = compile(&rule(
        "both",
        "png",
        "strings: $a = \"AB\" $b = { 43 [1-2] 44 }",
        "$a at 0 and $b in (2..3)",
    ));
    for (data, expected) in [
        (&b"ABCxD"[..], true),
        (b"ABxCxxD", true),
        (b"ABxxCxD", false),
        (b"xABCxD", false),
        (b"ABABCxD", false),
    ] {
        assert_eq!(matched(scan(&both, data)), expected, "{data:?}");
    }
}

#[cfg(feature = "bundled")]
#[test]
fn bundled_pack_compiles() {
    let rules = RuleSet::compile(&Source::bundled()).unwrap();
    assert!(!rules.labels().is_empty());
}
