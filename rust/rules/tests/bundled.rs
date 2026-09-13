//! The bundled rulesets parse, sit in the right buckets, and enforce no facts rule yet.

#![cfg(feature = "bundled")]
use magika_rules::{Bucket, Source};

#[test]
fn bundled_source_parses_and_has_enforced_rules() {
    assert!(Source::bundled().rules().iter().any(|r| r.enforced));
}

#[test]
fn every_ruleset_file_matches_its_bucket() {
    for (dir, bucket) in
        [("full", Bucket::Full), ("partial", Bucket::Partial), ("notworking", Bucket::NotWorking)]
    {
        for entry in
            std::fs::read_dir(format!("{}/rulesets/{dir}", env!("CARGO_MANIFEST_DIR"))).unwrap()
        {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "yar") {
                Source::parse_in_bucket(&std::fs::read_to_string(&path).unwrap(), bucket)
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            }
        }
    }
}

#[test]
fn no_enforced_rule_uses_facts_yet() {
    let text = Source::bundled().text().to_string();
    for block in text.split("\nrule ").skip(1) {
        let uses_facts = block.contains("zip_") || block.contains("pe_");
        let enforced = block.contains("enforced = true") || block.contains("enabled = true");
        assert!(!(uses_facts && enforced), "facts rule enforced before Part D:\nrule {block}");
    }
}
