// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Signature regressions carried over from #1447's `test_rule_regressions.py`: reviewed
//! headers and their corruptions, truncations, reported false positives and adversarial
//! lookalikes, now asserted against the native format resolvers.
//!
//! `tests_data/resolvers/regressions.json` holds every input and expectation that test built,
//! recorded by `tests_data/resolvers/export_regressions.py` running the original test with its
//! scanner replaced by a recorder. An input is a repository file or stored bytes, cut to a length
//! and optionally patched. The prefix and the size reach every check; when the whole input is held
//! (a repository file, or a stored zip input, stored whole), so does its tail, as `identify` reads
//! it.

use std::path::PathBuf;

use serde_json::Value;

use crate::Input;
use crate::resolvers::{PREFIX_LIMIT, identify, magic};

/// Recorded cases whose expectation the evaluation dataset overturned, by test and index, with
/// the labels they now resolve to.
const SUPERSEDED: &[(&str, usize, &[&str])] = &[
    // A DLL without an entry point holds only resources, like the 838 validated MUI files the
    // dataset keeps apart from executables, so pebin abstains on it (#1447's builder never
    // set an entry point).
    ("test_windows_images_are_pebin", 2, &[]),
    // Negative subtype/sibling cases from #1447 that are valid containers or documents now
    // resolved by the expanded resolvers (`zip`, `mp4`, `xml`, `svg`, `json`, `deb`).
    ("test_epub_directory_names_decide_when_the_local_header_is_not_canonical", 1, &["zip"]),
    ("test_epub_directory_names_decide_when_the_local_header_is_not_canonical", 2, &["zip"]),
    ("test_epub_directory_names_decide_when_the_local_header_is_not_canonical", 3, &["zip"]),
    ("test_ftyp_sibling_and_incomplete_brands_abstain", 4, &["mp4"]),
    ("test_images_outside_the_pe_contract_abstain", 6, &["pebin"]),
    ("test_manifest_first_package_needs_code_or_resources_in_the_prefix_or_the_directory", 0, &["zip"]),
    ("test_names_beyond_the_view_are_not_seen", 0, &["zip"]),
    ("test_names_in_the_comment_or_member_data_do_not_count", 0, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 1, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 2, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 3, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 4, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 5, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 6, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 7, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 8, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 9, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 11, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 12, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 13, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 14, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 15, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 16, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 17, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 18, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 19, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 21, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 22, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 23, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 24, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 25, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 26, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 27, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 28, &["zip"]),
    ("test_opendocument_media_type_and_content_decide", 29, &["zip"]),
    ("test_osm_xml_root_decides_and_osmchange_abstains", 1, &["xml"]),
    ("test_other_opendocument_media_types_abstain", 0, &["zip"]),
    ("test_other_opendocument_media_types_abstain", 1, &["zip"]),
    ("test_other_opendocument_media_types_abstain", 2, &["zip"]),
    ("test_other_opendocument_media_types_abstain", 3, &["zip"]),
    ("test_other_opendocument_media_types_abstain", 4, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 0, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 1, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 2, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 3, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 4, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 5, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 6, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 7, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 8, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 9, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 10, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 11, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 12, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 13, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 14, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 15, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 16, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 17, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 18, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 19, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 20, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 21, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 22, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 23, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 24, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 25, &["zip"]),
    ("test_partial_or_lookalike_names_abstain", 26, &["zip"]),
    ("test_prefix_signatures_for_labels_the_model_lacks_or_misses", 5, &["svg"]),
    ("test_prefix_signatures_for_labels_the_model_lacks_or_misses", 15, &["json"]),
    ("test_prefix_signatures_for_labels_the_model_lacks_or_misses", 17, &["xml"]),
    ("test_prefix_signatures_for_labels_the_model_lacks_or_misses", 29, &["xml"]),
    ("test_prefix_signatures_for_labels_the_model_lacks_or_misses", 59, &["deb"]),
    ("test_reported_false_positives_abstain", 0, &["zip"]),
    ("test_reported_false_positives_abstain", 6, &["zip"]),
    ("test_reported_false_positives_abstain", 12, &["zip"]),
    ("test_reported_false_positives_abstain", 16, &["zip"]),
    ("test_reported_false_positives_abstain", 17, &["zip"]),
    ("test_reported_false_positives_abstain", 27, &["zip"]),
    ("test_reported_false_positives_abstain", 28, &["zip"]),
    ("test_reviewed_binary_header_variants", 389, &[]),
    ("test_reviewed_binary_header_variants", 390, &[]),
    ("test_reviewed_binary_header_variants", 391, &[]),
    ("test_word_documents_and_templates_are_told_apart_by_content_type", 3, &["docx"]),
];

fn data(relative: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests_data");
    match relative.split_once('/') {
        Some(("rules_negative", name)) => root.join("resolvers/negative").join(name),
        Some(("rules_positive", name)) => root.join("resolvers/positive").join(name),
        _ => root.join(relative),
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
fn labels(held: &[u8], size: u64) -> Vec<String> {
    if size == 0 || held.len() < size.min(PREFIX_LIMIT as u64) as usize {
        return Vec::new();
    }
    let content_types = if held.len() as u64 == size {
        identify(&mut Input::from_slice(held)).unwrap()
    } else {
        let prefix = &held[..held.len().min(PREFIX_LIMIT)];
        magic::identify(&magic::Ctx { prefix, size, tail: None }).into_iter().collect()
    };
    content_types.into_iter().map(|ct| ct.info().label.to_string()).collect()
}

#[test]
fn recorded_signature_regressions_hold() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests_data/resolvers/regressions.json");
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
                let actual = labels(&held, size);
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
                    (None, Some(absent)) => !actual.iter().any(|l| l == absent.as_str().unwrap()),
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

/// Ported from `test_all_public_fixture_incomplete_prefixes_abstain`: no resolver decides from
/// fewer than eight bytes of any repository sample.
#[test]
fn no_resolver_decides_from_fewer_than_eight_bytes() {
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
            let got = identify(&mut Input::from_slice(&bytes[..length])).unwrap();
            assert!(got.is_empty(), "{} decided from {length} bytes: {got:?}", path.display());
        }
    }
    assert!(files > 100, "only {files} repository samples found");
}
