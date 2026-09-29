// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Signature regressions carried over from #1447's `test_rule_regressions.py`: reviewed
//! headers and their corruptions, truncations, reported false positives and adversarial
//! lookalikes, now asserted against this engine.
//!
//! `data/regressions.json` holds every input and expectation that test built, recorded by
//! `data/export_regressions.py` running the original test with its scanner replaced by a
//! recorder. An input is a repository file or stored bytes, cut to a length and optionally
//! patched. The prefix and the size reach every scan; when the whole input is held (a
//! repository file, or a stored zip input, stored whole), so does its tail, as a caller reads
//! it.

#![cfg(feature = "bundled")]

use std::path::PathBuf;

use magika_rules::{Input, Outcome, RuleSet, PREFIX_LIMIT};
use serde_json::Value;

/// Recorded cases whose expectation the evaluation dataset overturned, by test and index, with
/// the labels they now scan to.
const SUPERSEDED: &[(&str, usize, &[&str])] = &[
    // A DLL without an entry point holds only resources, like the 838 validated MUI files the
    // dataset keeps apart from executables, so pebin abstains on it (#1447's builder never
    // set an entry point).
    ("test_windows_images_are_pebin", 2, &[]),
];

fn data(relative: &str) -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match relative.split_once('/') {
        Some(("rules_negative", name)) => manifest.join("tests/data/negative").join(name),
        Some(("rules_positive", name)) => manifest.join("tests/data/positive").join(name),
        _ => manifest.join("../../tests_data").join(relative),
    }
}

fn hex(text: &str) -> Vec<u8> {
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap()).collect()
}

/// A stored prefix: hex runs, and `[byte, count]` for repeated bytes.
fn blob(segments: &Value) -> Vec<u8> {
    let mut bytes = Vec::new();
    for segment in segments.as_array().unwrap() {
        match segment {
            Value::String(text) => bytes.extend(hex(text)),
            Value::Array(run) => {
                let byte = hex(run[0].as_str().unwrap())[0];
                bytes.extend(std::iter::repeat_n(byte, run[1].as_u64().unwrap() as usize));
            }
            _ => panic!("malformed blob segment {segment}"),
        }
    }
    bytes
}

/// The labels of an input of `size` bytes, of which `held` are known: all of them, or at least
/// the prefix.
fn labels(rules: &RuleSet, held: &[u8], size: u64) -> Vec<String> {
    let prefix = &held[..held.len().min(PREFIX_LIMIT)];
    let mut tail = None;
    if held.len() as u64 == size {
        // The tail a caller reads: the last `tail_len` bytes, back to `tail_start` if needed.
        let mut start = held.len() - rules.tail_len(prefix, size);
        if let Some(earlier) = rules.tail_start(&held[start..], size) {
            start = earlier as usize;
        }
        tail = (start < held.len()).then(|| &held[start..]);
    }
    match rules.scan(Input { prefix, size, tail }) {
        Outcome::Match(i) => vec![rules.labels()[i].clone()],
        Outcome::Conflict => vec!["<conflict>".to_string()],
        Outcome::NoMatch | Outcome::InsufficientInput => Vec::new(),
    }
}

#[test]
fn recorded_signature_regressions_hold() {
    let rules = RuleSet::bundled();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/regressions.json");
    let recorded: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let blobs: Vec<Vec<u8>> = recorded["blobs"].as_array().unwrap().iter().map(blob).collect();
    let (mut checked, mut failures) = (0usize, Vec::new());
    for (test, cases) in recorded["tests"].as_object().unwrap() {
        for (index, case) in cases.as_array().unwrap().iter().enumerate() {
            let source = match (case.get("file"), case.get("blob")) {
                (Some(file), None) => std::fs::read(data(file.as_str().unwrap())).unwrap(),
                (None, Some(blob)) => blobs[blob.as_u64().unwrap() as usize].clone(),
                _ => panic!("{test}[{index}]: malformed input"),
            };
            let (first, last) = match (case.get("len"), case.get("lens")) {
                (Some(len), None) => (len.as_u64().unwrap(), len.as_u64().unwrap()),
                (None, Some(lens)) => (lens[0].as_u64().unwrap(), lens[1].as_u64().unwrap()),
                _ => panic!("{test}[{index}]: malformed length"),
            };
            for size in first..=last {
                let available = (size as usize).min(PREFIX_LIMIT);
                assert!(available <= source.len(), "{test}[{index}]: input shorter than {size}");
                // The whole input when it is held, its prefix otherwise.
                let held = if source.len() as u64 >= size { size as usize } else { available };
                let mut held = source[..held].to_vec();
                for patch in case.get("patch").and_then(Value::as_array).into_iter().flatten() {
                    let offset = patch[0].as_u64().unwrap() as usize;
                    let bytes = hex(patch[1].as_str().unwrap());
                    held[offset..offset + bytes.len()].copy_from_slice(&bytes);
                }
                let actual = labels(&rules, &held, size);
                let superseded = SUPERSEDED.iter().find(|x| x.0 == test && x.1 == index);
                let holds = match (case.get("labels"), case.get("absent")) {
                    _ if superseded.is_some() => actual == superseded.unwrap().2,
                    (Some(expected), None) => {
                        let expected: Vec<&str> = expected
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|l| l.as_str().unwrap())
                            .collect();
                        actual == expected
                    }
                    (None, Some(absent)) => {
                        !actual.iter().any(|l| l == absent.as_str().unwrap() || l == "<conflict>")
                    }
                    _ => panic!("{test}[{index}]: malformed expectation"),
                };
                checked += 1;
                if !holds {
                    failures.push(format!("{test}[{index}] size {size}: {case} -> {actual:?}"));
                }
            }
        }
    }
    eprintln!("regressions: {checked} cases, {} failures", failures.len());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(checked, 15169, "the recording changed; regenerate it deliberately");
}

/// Ported from `test_all_public_fixture_incomplete_prefixes_abstain`: no rule decides from
/// fewer than eight bytes of any repository sample.
#[test]
fn no_rule_decides_from_fewer_than_eight_bytes() {
    let rules = RuleSet::bundled();
    let mut pending = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests_data")];
    let mut files = 0;
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            pending.extend(std::fs::read_dir(&path).unwrap().map(|e| e.unwrap().path()));
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        files += 1;
        for length in 0..=bytes.len().min(7) {
            let outcome =
                rules.scan(Input { prefix: &bytes[..length], size: length as u64, tail: None });
            assert!(
                matches!(outcome, Outcome::NoMatch | Outcome::InsufficientInput),
                "{} decided from {length} bytes: {outcome:?}",
                path.display()
            );
        }
    }
    assert!(files > 100, "only {files} repository samples found");
}
