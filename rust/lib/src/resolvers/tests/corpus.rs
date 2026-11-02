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

//! Every sample under `tests_data/basic/<label>/` must resolve to no match or to its own label.
//! A sample listed in `MISLABELED` must resolve to the label its bytes prove instead.

use std::path::Path;

use crate::cached_input::CachedInput;
use crate::resolvers::identify;

/// Samples whose directory names the wrong label, with the label their bytes prove. Each
/// still has to resolve to that label. Relabel them in `tests_data` and drop the entry.
const MISLABELED: &[(&str, &str)] = &[
    // OpenPGP ASCII armor (RFC 9580, `-----BEGIN PGP PUBLIC KEY BLOCK-----`), not PEM (RFC 7468).
    ("pem/doc.pem", "pgp"),
    ("pem/doc.pub", "pgp"),
];

#[test]
fn resolvers_never_mislabel_a_sample() {
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
            files += 1;
            let mut input = CachedInput::new(std::fs::File::open(&path).unwrap()).unwrap();
            let got = identify(&mut input).unwrap();
            match &got[..] {
                [ct] if ct.info().label == expected => hits += 1,
                [ct] => {
                    errors.push(format!("{}: resolvers say {}", path.display(), ct.info().label));
                }
                [] if expected != label => {
                    errors.push(format!("{}: no longer resolves to {expected}", path.display()));
                }
                [] => {}
                cts => {
                    let names: Vec<_> = cts.iter().map(|ct| ct.info().label).collect();
                    errors.push(format!("{}: conflict: {}", path.display(), names.join(", ")));
                }
            }
        }
    }
    eprintln!("corpus: {files} files, {hits} resolver hits, {} false positives", errors.len());
    assert!(errors.is_empty(), "false positives:\n{}", errors.join("\n"));
    assert!(hits > 0, "no resolver matched any sample; resolvers are not wired");
}
