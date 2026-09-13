//! Parsing and metadata validation of rule sources.

use magika_rules::{Bucket, Class, Error, Source};

const OK: &str = r#"
rule png { meta: label = "png" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = { 89 50 4E 47 0D 0A 1A 0A } condition: $a at 0 and filesize >= 8 }
"#;

#[test]
fn parses_an_enforced_full_rule() {
    let source = Source::parse(OK).unwrap();
    let rule = &source.rules()[0];
    assert_eq!(rule.id, "png");
    assert_eq!(rule.label.as_deref(), Some("png"));
    assert_eq!(rule.class, Class::Full);
    assert!(rule.enforced);
    assert_eq!(rule.bucket, None);
}

#[test]
fn rejects_syntax_errors() {
    assert!(matches!(Source::parse("rule { }"), Err(Error::Parse(_))));
}

#[test]
fn rejects_imports_includes_and_global_rules() {
    assert!(matches!(Source::parse("import \"pe\"\n"), Err(Error::Unsupported { .. })));
    assert!(matches!(
        Source::parse("global rule g { condition: true }"),
        Err(Error::Unsupported { .. })
    ));
}

#[test]
fn rejects_duplicate_rule_ids() {
    let text = format!("{OK}\n{}", OK.replace("$a", "$b"));
    assert!(matches!(Source::parse(&text), Err(Error::Metadata { .. })));
}

#[test]
fn enforced_rules_need_zero_fp_and_class_consistent_fn() {
    let bad = OK.replace("fp_rate = 0", "fp_rate = 0.01");
    assert!(matches!(Source::parse(&bad), Err(Error::Metadata { .. })));
    let partial_with_zero_fn = OK.replace("class = \"full\"", "class = \"partial\"");
    assert!(matches!(Source::parse(&partial_with_zero_fn), Err(Error::Metadata { .. })));
}

#[test]
fn enforced_rules_need_a_label() {
    let bad = OK.replace("label = \"png\" ", "");
    assert!(matches!(Source::parse(&bad), Err(Error::Metadata { .. })));
}

#[test]
fn unenforced_rules_are_kept_but_flagged() {
    let off = OK.replace("enforced = true", "enforced = false");
    assert!(!Source::parse(&off).unwrap().rules()[0].enforced);
}

#[test]
fn bucket_check_rejects_a_full_rule_in_partial() {
    assert!(matches!(Source::parse_in_bucket(OK, Bucket::Partial), Err(Error::Metadata { .. })));
    assert!(Source::parse_in_bucket(OK, Bucket::Full).is_ok());
}
