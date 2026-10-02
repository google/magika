// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Compiles `rulesets/` into `src/bundled.rs` and `../gen/rules/content_types`.

use std::path::Path;

fn main() {
    let (bundled, labels) = magika_rules::generate(Path::new("rulesets"));
    std::fs::write("src/bundled.rs", bundled).unwrap();
    std::fs::create_dir_all("../gen/rules").unwrap();
    std::fs::write("../gen/rules/content_types", labels).unwrap();
}
