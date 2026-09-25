// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Concatenates `rulesets/{full,partial,notworking}/*.yar` and the third-party notices into
//! `OUT_DIR/bundled.yar`. Validation here is structural (syntax, imports, globals, duplicate
//! ids); `Source::bundled` runs the full metadata checks at first use.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use yara_x_parser::ast::{Item, RuleFlags, AST};

// The crate's own parsing, validation and lowering compile the bundled rules ahead of time.
#[allow(dead_code, unreachable_pub)]
#[path = "src/codegen.rs"]
mod codegen;
#[allow(dead_code, unreachable_pub)]
#[path = "src/error.rs"]
mod error;
#[allow(dead_code, unreachable_pub)]
#[path = "src/ir.rs"]
mod ir;
#[allow(dead_code, unreachable_pub)]
#[path = "src/lower.rs"]
mod lower;
#[allow(dead_code, unreachable_pub)]
#[path = "src/matcher.rs"]
mod matcher;
#[allow(dead_code, unreachable_pub)]
#[path = "src/source.rs"]
mod source;

use error::Error;
use source::{RuleInfo, Source};

/// Must equal `magika_rules::PREFIX_LIMIT`; the crate's tests fail otherwise.
const PREFIX_LIMIT: usize = 4096;

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let (text, compiled) = if std::env::var_os("CARGO_FEATURE_BUNDLED").is_some() {
        let text = bundle(&manifest.join("rulesets"), &manifest.join("LICENSES"));
        let source = Source::parse(&text).unwrap_or_else(|e| panic!("bundled rules: {e}"));
        let (program, labels) =
            lower::lower(&source).unwrap_or_else(|e| panic!("bundled rules: {e}"));
        let compiled = codegen::rule_set(&program, &labels, source.rules());
        (text, compiled)
    } else {
        (String::new(), String::new())
    };
    write_if_changed(&out.join("bundled.yar"), &text);
    write_if_changed(&out.join("bundled.rs"), &compiled);
}

fn write_if_changed(path: &Path, content: &str) {
    if std::fs::read_to_string(path).ok().as_deref() != Some(content) {
        std::fs::write(path, content).unwrap();
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
