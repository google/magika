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

//! The --rules flag of the CLI.

use std::collections::BTreeMap;
use std::process::Command;

/// Samples whose directory label their bytes refute, with the label the bytes prove.
const REFUTED: &[(&str, &str)] = &[
    // Both are armored PGP public keys (`-----BEGIN PGP PUBLIC KEY BLOCK-----`), not PEM.
    ("basic/pem/doc.pem", "pgp"),
    ("basic/pem/doc.pub", "pgp"),
];

/// Returns the label of every file under `tests_data/basic`, by path.
fn labels(rules: &str) -> BTreeMap<String, String> {
    let output = Command::new(env!("CARGO_BIN_EXE_magika"))
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests_data"))
        .args([&format!("--rules={rules}"), "--format=%p %l", "--recursive", "basic"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let output = String::from_utf8(output.stdout).unwrap();
    let labels: BTreeMap<_, _> = output
        .lines()
        .map(|line| {
            let (path, label) = line.rsplit_once(' ').unwrap();
            (path.to_string(), label.to_string())
        })
        .collect();
    assert!(labels.len() > 100, "{} files", labels.len());
    labels
}

#[test]
fn bundled_rules_never_contradict_sample_labels() {
    let off = labels("off");
    let enforce = labels("enforce");
    let only = labels("only");
    assert_eq!(off.keys().collect::<Vec<_>>(), enforce.keys().collect::<Vec<_>>());
    let mut decided = 0;
    for (path, label) in &enforce {
        let expected = match REFUTED.iter().find(|(refuted, _)| refuted == path) {
            Some((_, proven)) => proven,
            None => path.split('/').nth(1).unwrap(),
        };
        // Enforcing rules never turns a right answer wrong, and rules only never guess.
        if label != &off[path] {
            assert_eq!(label, expected, "{path}: model says {}", off[path]);
        }
        if only[path] != "unknown" {
            decided += 1;
            assert_eq!(&only[path], label, "{path}");
        }
    }
    assert_eq!(only["basic/rust/code.rs"], "unknown");
    // The empty sample counts too: it needs neither rules nor the model.
    assert!(decided >= 10, "rules only names {decided} samples");
    eprintln!("rules only names {decided} of {} samples", enforce.len());
}
