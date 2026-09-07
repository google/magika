// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Assemble canonical YARA sources for checkout and packaged-crate builds.

#[cfg(feature = "yara-rules")]
use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "yara-rules")]
#[path = "src/rules/metadata.rs"]
mod metadata;

fn main() -> anyhow::Result<()> {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let packaged = manifest.join("rulesets");
    let canonical = manifest.join("../../rules/rulesets");
    // In a checkout, packaging leftovers must not override the maintained sources.
    // An isolated source distribution has only its staged rulesets directory.
    let root = if canonical.is_dir() { canonical } else { packaged };
    println!("cargo:rerun-if-changed={}", root.display());
    let mut source = String::new();
    for bucket in ["full", "partial", "notworking"] {
        let directory = root.join(bucket);
        println!("cargo:rerun-if-changed={}", directory.display());
        let mut files = std::fs::read_dir(&directory)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        files.sort();
        for file in files.iter().filter(|file| file.extension().is_some_and(|ext| ext == "yar")) {
            println!("cargo:rerun-if-changed={}", file.display());
            let text = std::fs::read_to_string(file)?;
            #[cfg(feature = "yara-rules")]
            validate(&text, bucket, file)?;
            source.push_str(&text);
            source.push('\n');
        }
    }
    #[cfg(feature = "yara-rules")]
    {
        let ast = yara_x_parser::ast::AST::from(source.as_str());
        let mut identifiers = std::collections::HashSet::new();
        for rule in ast.rules() {
            anyhow::ensure!(identifiers.insert(rule.identifier.name), "duplicate bundled rule ID");
        }
    }
    let output = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("bundled-rules.yar");
    if std::fs::read_to_string(&output).ok().as_deref() != Some(&source) {
        std::fs::write(output, source)?;
    }
    Ok(())
}

#[cfg(feature = "yara-rules")]
fn validate(source: &str, bucket: &str, file: &Path) -> anyhow::Result<()> {
    use anyhow::{ensure, Context};
    use yara_x_parser::ast::{Item, RuleFlags, AST};
    let ast = AST::from(source);
    ensure!(ast.errors().is_empty(), "invalid YARA in {}: {:?}", file.display(), ast.errors());
    for item in ast.items() {
        let Item::Rule(rule) = item else {
            anyhow::bail!("imports and includes are not supported")
        };
        ensure!(!rule.flags.contains(RuleFlags::Global), "global rules are not supported");
        metadata::enforced(rule, Some(bucket)).with_context(|| file.display().to_string())?;
    }
    Ok(())
}
