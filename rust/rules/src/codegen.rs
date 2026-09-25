// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Writes a compiled rule set as the Rust expression that builds it again.
//!
//! `build.rs` compiles the bundled rules with this crate's own lowering and writes them with
//! [`rule_set`], so that [`crate::RuleSet::bundled`] neither parses YARA nor builds a regex at
//! startup. A test checks that the generated code is what compiling the bundled source gives.

use std::fmt::Write as _;

use crate::ir::{Cond, Int, Program};
use crate::matcher::Pattern;
use crate::RuleInfo;

/// Returns a Rust block expression that evaluates to the rule set with these parts.
pub(crate) fn rule_set(program: &Program, labels: &[String], rules: &[RuleInfo]) -> String {
    let mut out = String::from(
        "{\n    use crate::ir::{Cmp, Cond as C, Int as I, Read};\n    \
         use crate::matcher::{Pattern, RegexPattern};\n    \
         use crate::{Class, RuleInfo};\n    \
         crate::RuleSet {\n        program: crate::ir::Program {\n            patterns: vec![\n",
    );
    for pattern in &program.patterns {
        out.push_str("                ");
        match pattern {
            Pattern::Masked { bytes, mask } => {
                write!(out, "Pattern::Masked {{ bytes: vec!{bytes:?}, mask: vec!{mask:?} }}")
            }
            Pattern::Regex(regex) => write!(
                out,
                "Pattern::Regex(RegexPattern::new({:?}, {}, {}, {}))",
                regex.source, regex.case_insensitive, regex.dot_matches_new_line, regex.max_len
            ),
        }
        .unwrap();
        out.push_str(",\n");
    }
    out.push_str("            ],\n            rules: vec![\n");
    for rule in &program.rules {
        out.push_str("                crate::ir::Rule { label: ");
        write!(out, "{}, cond: ", rule.label).unwrap();
        cond(&mut out, &rule.cond);
        out.push_str(" },\n");
    }
    writeln!(
        out,
        "            ],\n            facts: {},\n        }},\n        labels: vec![",
        program.facts
    )
    .unwrap();
    for label in labels {
        writeln!(out, "            {label:?}.to_string(),").unwrap();
    }
    out.push_str("        ],\n        rules: vec![\n");
    for rule in rules {
        writeln!(
            out,
            "            RuleInfo {{ id: {:?}.to_string(), label: {}, class: Class::{:?}, \
             enforced: {}, bucket: {} }},",
            rule.id,
            match &rule.label {
                Some(label) => format!("Some({label:?}.to_string())"),
                None => "None".to_string(),
            },
            rule.class,
            rule.enforced,
            match rule.bucket {
                Some(bucket) => format!("Some(crate::Bucket::{bucket:?})"),
                None => "None".to_string(),
            },
        )
        .unwrap();
    }
    out.push_str("        ],\n    }\n}\n");
    out
}

fn cond(out: &mut String, cond: &Cond) {
    match cond {
        Cond::True => out.push_str("C::True"),
        Cond::False => out.push_str("C::False"),
        Cond::Not(x) => {
            out.push_str("C::Not(Box::new(");
            self::cond(out, x);
            out.push_str("))");
        }
        Cond::And(xs) | Cond::Or(xs) => {
            out.push_str(if matches!(cond, Cond::And(_)) { "C::And(vec![" } else { "C::Or(vec![" });
            for x in xs {
                self::cond(out, x);
                out.push_str(", ");
            }
            out.push_str("])");
        }
        Cond::Cmp(op, a, b) => {
            write!(out, "C::Cmp(Cmp::{op:?}, ").unwrap();
            int(out, a);
            out.push_str(", ");
            int(out, b);
            out.push(')');
        }
        Cond::At { pattern, offset } => {
            write!(out, "C::At {{ pattern: {pattern}, offset: ").unwrap();
            int(out, offset);
            out.push_str(" }");
        }
        Cond::In { pattern, lo, hi } => {
            write!(out, "C::In {{ pattern: {pattern}, lo: ").unwrap();
            int(out, lo);
            out.push_str(", hi: ");
            int(out, hi);
            out.push_str(" }");
        }
        Cond::View { view, text, start } => write!(
            out,
            "C::View {{ view: crate::ir::View::{view:?}, text: vec!{text:?}, start: {start} }}"
        )
        .unwrap(),
    }
}

fn int(out: &mut String, int: &Int) {
    match int {
        Int::Const(value) => write!(out, "I::Const({value})").unwrap(),
        Int::FileSize => out.push_str("I::FileSize"),
        Int::PrefixSize => out.push_str("I::PrefixSize"),
        Int::Read(read, at) => {
            write!(out, "I::Read(Read::{read:?}, Box::new(").unwrap();
            self::int(out, at);
            out.push_str("))");
        }
        Int::Fact(fact) => write!(out, "I::Fact(crate::ir::Fact::{fact:?})").unwrap(),
        Int::And(a, b) => {
            out.push_str("I::And(Box::new(");
            self::int(out, a);
            out.push_str("), Box::new(");
            self::int(out, b);
            out.push_str("))");
        }
    }
}
