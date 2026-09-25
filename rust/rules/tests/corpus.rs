// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Every sample under tests_data/basic/<label>/ must scan to NoMatch or to its own label.
//! Labels in tests_data use Magika's canonical names, which are also the rule labels.
//! A sample listed in `MISLABELED` must scan to the label its bytes prove instead.

#![cfg(feature = "bundled")]

use std::path::Path;

use magika_rules::{Input, Outcome, RuleSet, PREFIX_LIMIT};

/// Samples whose directory names the wrong label, with the label their bytes prove. Each
/// still has to scan to that label. Relabel them in tests_data and drop the entry.
const MISLABELED: &[(&str, &str)] = &[
    // OpenPGP ASCII armor (RFC 9580, `-----BEGIN PGP PUBLIC KEY BLOCK-----`), not PEM (RFC 7468).
    ("pem/doc.pem", "pgp"),
    ("pem/doc.pub", "pgp"),
];

#[test]
fn bundled_rules_never_mislabel_a_sample() {
    let rules = RuleSet::bundled();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests_data/basic");
    let (mut files, mut hits, mut errors) = (0usize, 0usize, Vec::new());
    for dir in std::fs::read_dir(&root).unwrap() {
        let dir = dir.unwrap().path();
        if !dir.is_dir() {
            continue;
        }
        let label = dir.file_name().unwrap().to_str().unwrap().to_string();
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if !path.is_file() {
                continue;
            }
            let relative = format!("{label}/{}", path.file_name().unwrap().to_str().unwrap());
            let expected = MISLABELED
                .iter()
                .find(|(sample, _)| *sample == relative)
                .map_or(label.as_str(), |(_, proven)| proven);
            let bytes = std::fs::read(&path).unwrap();
            let (prefix, size) = (&bytes[..bytes.len().min(PREFIX_LIMIT)], bytes.len() as u64);
            // The tail a caller reads: the last `tail_len` bytes, back to `tail_start` if needed.
            let mut tail = &bytes[bytes.len() - rules.tail_len(prefix, size)..];
            if let Some(start) = rules.tail_start(tail, size) {
                tail = &bytes[start as usize..];
            }
            let tail = (!tail.is_empty()).then_some(tail);
            files += 1;
            match rules.scan(Input { prefix, size, tail }) {
                Outcome::Match(i) if rules.labels()[i] == expected => hits += 1,
                Outcome::Match(i) => {
                    errors.push(format!("{}: rules say {}", path.display(), rules.labels()[i]))
                }
                Outcome::Conflict => errors.push(format!("{}: conflict", path.display())),
                _ if expected != label => {
                    errors.push(format!("{}: no longer scans to {expected}", path.display()))
                }
                Outcome::NoMatch | Outcome::InsufficientInput => {}
            }
        }
    }
    eprintln!("corpus: {files} files, {hits} rule hits, {} false positives", errors.len());
    assert!(errors.is_empty(), "false positives:\n{}", errors.join("\n"));
    assert!(hits > 0, "no rule matched any sample; the pack is not wired");
}
