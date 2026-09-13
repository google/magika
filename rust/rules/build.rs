// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Concatenates `rulesets/{full,partial,notworking}/*.yar` and the third-party notices into
//! `OUT_DIR/bundled.yar`. Validation here is structural (syntax, imports, globals, duplicate
//! ids); `Source::bundled` runs the full metadata checks at first use.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use yara_x_parser::ast::{Item, RuleFlags, AST};

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let output = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("bundled.yar");
    let source = if std::env::var_os("CARGO_FEATURE_BUNDLED").is_some() {
        bundle(&manifest.join("rulesets"), &manifest.join("LICENSES"))
    } else {
        String::new()
    };
    if std::fs::read_to_string(&output).ok().as_deref() != Some(source.as_str()) {
        std::fs::write(&output, source).unwrap();
    }
}

fn bundle(root: &Path, licenses: &Path) -> String {
    println!("cargo:rerun-if-changed={}", root.display());
    let mut source = String::new();
    let mut ids = HashSet::new();
    for bucket in ["full", "partial", "notworking"] {
        let directory = root.join(bucket);
        println!("cargo:rerun-if-changed={}", directory.display());
        let mut files = std::fs::read_dir(&directory)
            .unwrap_or_else(|e| panic!("{}: {e}", directory.display()))
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "yar"))
            .collect::<Vec<_>>();
        files.sort();
        for file in files {
            println!("cargo:rerun-if-changed={}", file.display());
            let text = std::fs::read_to_string(&file).unwrap();
            validate(&text, &file, &mut ids);
            source.push_str(&text);
            source.push('\n');
        }
    }
    // Keep the notices with any source exported from a binary.
    println!("cargo:rerun-if-changed={}", licenses.display());
    source.push_str("\n// Third-party signature notices\n");
    for line in std::fs::read_to_string(licenses).unwrap().lines() {
        source.push_str("// ");
        source.push_str(line);
        source.push('\n');
    }
    source
}

fn validate(text: &str, file: &Path, ids: &mut HashSet<String>) {
    let ast = AST::from(text);
    assert!(ast.errors().is_empty(), "invalid YARA in {}: {:?}", file.display(), ast.errors());
    for item in ast.items() {
        let Item::Rule(rule) = item else {
            panic!("{}: imports and includes are not supported", file.display())
        };
        let id = rule.identifier.name;
        assert!(!rule.flags.contains(RuleFlags::Global), "{}: global rule {id}", file.display());
        assert!(ids.insert(id.to_string()), "{}: duplicate rule ID {id}", file.display());
    }
}
