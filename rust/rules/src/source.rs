// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Parsed, validated YARA text.

use std::collections::HashSet;

use yara_x_parser::ast::{Item, MetaValue, Rule, RuleFlags, AST};

use crate::Error;

/// Declared coverage class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// `class = "full"`.
    Full,
    /// `class = "partial"`.
    Partial,
    /// No `class` metadata.
    Untested,
}

/// One rule's metadata, in source order (private rules excluded).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleInfo {
    /// Rule identifier.
    pub id: String,
    /// The label the rule assigns; required when enforced.
    pub label: Option<String>,
    /// Declared coverage class.
    pub class: Class,
    /// Whether the rule takes part in scans.
    pub enforced: bool,
}

/// Validated YARA text.
#[derive(Clone, Debug)]
pub struct Source {
    text: String,
    rules: Vec<RuleInfo>,
}

impl Source {
    /// Largest accepted source.
    pub const MAX_BYTES: usize = 4 * 1024 * 1024;

    /// Parses and validates `text`.
    pub fn parse(text: &str) -> Result<Self, Error> {
        if text.len() > Self::MAX_BYTES {
            return Err(Error::Parse("source exceeds 4 MiB".into()));
        }
        let ast = AST::from(text);
        if !ast.errors().is_empty() {
            let errors: Vec<_> = ast.errors().iter().map(|e| syntax_error(text, e)).collect();
            return Err(Error::Parse(errors.join("; ")));
        }
        let mut seen = HashSet::new();
        let mut rules = Vec::new();
        for item in ast.items() {
            let Item::Rule(rule) = item else {
                return Err(Error::Unsupported {
                    rule: String::new(),
                    reason: "imports and includes".into(),
                });
            };
            let id = rule.identifier.name.to_string();
            if rule.flags.contains(RuleFlags::Global) {
                return Err(Error::Unsupported { rule: id, reason: "global rules".into() });
            }
            if !seen.insert(id.clone()) {
                return Err(Error::Metadata { rule: id, reason: "duplicate rule ID".into() });
            }
            if rule.flags.contains(RuleFlags::Private) {
                continue;
            }
            rules.push(Self::info(rule)?);
        }
        Ok(Source { text: text.to_string(), rules })
    }

    /// The validated YARA text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Metadata of every non-private rule, in source order.
    pub fn rules(&self) -> &[RuleInfo] {
        &self.rules
    }

    fn info(rule: &Rule<'_>) -> Result<RuleInfo, Error> {
        let id = rule.identifier.name.to_string();
        let fail = |reason: &str| Error::Metadata { rule: id.clone(), reason: reason.into() };
        let metadata = metadata::Metadata::read(rule).map_err(fail)?;
        let enforced = metadata.enforced().map_err(fail)?;
        let mut labels = Vec::new();
        for meta in rule.meta.iter().flatten().filter(|meta| meta.identifier.name == "label") {
            let MetaValue::String((value, _)) = &meta.value else {
                return Err(fail("label must be a string"));
            };
            labels.push(value.to_string());
        }
        if labels.len() > 1 {
            return Err(fail("duplicate label"));
        }
        let label = labels.pop();
        if enforced && label.is_none() {
            return Err(fail("enforced rule needs a label"));
        }
        Ok(RuleInfo { class: metadata.class, id, label, enforced })
    }
}

/// A rule's explicit `enabled`/`enforced` value, if any. Private helpers default to active.
pub(crate) fn explicit_enforcement(rule: &Rule<'_>) -> Option<bool> {
    rule.meta.iter().flatten().find_map(|meta| match (meta.identifier.name, &meta.value) {
        ("enabled" | "enforced", MetaValue::Bool((value, _))) => Some(*value),
        _ => None,
    })
}

mod metadata {
    //! Ported from `rust/lib/src/rules/metadata.rs` of #1447. Metadata declares the
    //! caller's evaluation results; it does not establish them.

    use yara_x_parser::ast::{MetaValue, Rule};

    use super::Class;

    pub(super) struct Metadata {
        pub(super) class: Class,
        active: bool,
        fp_rate: Option<f64>,
        fn_rate: Option<f64>,
    }

    impl Metadata {
        /// Reads `enabled`/`enforced`, `class`, `fp_rate` and `fn_rate` of a non-private rule.
        pub(super) fn read(rule: &Rule<'_>) -> Result<Self, &'static str> {
            let (mut enabled, mut enforced, mut class, mut fp, mut fn_rate) =
                (None, None, None, None, None);
            for meta in rule.meta.iter().flatten() {
                match (meta.identifier.name, &meta.value) {
                    ("enabled" | "enforced", MetaValue::Bool((value, _))) => {
                        let slot = if meta.identifier.name == "enabled" {
                            &mut enabled
                        } else {
                            &mut enforced
                        };
                        if slot.replace(*value).is_some() {
                            return Err("duplicate enforcement metadata");
                        }
                    }
                    ("class", MetaValue::String((value, _))) => {
                        let value = match *value {
                            "full" => Class::Full,
                            "partial" => Class::Partial,
                            _ => return Err("class must be full or partial"),
                        };
                        if class.replace(value).is_some() {
                            return Err("duplicate class metadata");
                        }
                    }
                    ("fp_rate" | "fn_rate", value) => {
                        let value = match value {
                            MetaValue::Integer((value, _)) => Some(*value as f64),
                            MetaValue::Float((value, _)) => Some(*value),
                            MetaValue::String(("unmeasured", _)) => None,
                            _ => return Err("error rates must be numeric fractions"),
                        };
                        if !value.is_none_or(|v| v.is_finite() && (0.0..=1.0).contains(&v)) {
                            return Err("error rates must be in [0, 1]");
                        }
                        let slot =
                            if meta.identifier.name == "fp_rate" { &mut fp } else { &mut fn_rate };
                        if slot.replace(value).is_some() {
                            return Err("duplicate error rate metadata");
                        }
                    }
                    ("enabled" | "enforced" | "class", _) => {
                        return Err("enforcement must be boolean and class a string")
                    }
                    _ => {}
                }
            }
            if enabled.is_some() && enforced.is_some() && enabled != enforced {
                return Err("enabled and enforced disagree");
            }
            Ok(Metadata {
                class: class.unwrap_or(Class::Untested),
                active: enforced.or(enabled).unwrap_or(false),
                fp_rate: fp.flatten(),
                fn_rate: fn_rate.flatten(),
            })
        }

        /// Whether the rule is enforced, checking its declared error rates and class.
        pub(super) fn enforced(&self) -> Result<bool, &'static str> {
            if !self.active {
                return Ok(false);
            }
            if self.fp_rate != Some(0.0) {
                return Err("enforced rules require fp_rate = 0");
            }
            let consistent = match (self.class, self.fn_rate) {
                (Class::Full, Some(rate)) => rate == 0.0,
                (Class::Partial, Some(rate)) => rate > 0.0 && rate < 1.0,
                _ => false,
            };
            if !consistent {
                return Err("enforced rules require class full with fn_rate = 0, \
                            or partial with 0 < fn_rate < 1");
            }
            Ok(true)
        }
    }
}

/// One parser error, as `line L, column C: message` counting from 1, so an author finds it.
fn syntax_error(text: &str, error: &yara_x_parser::ast::Error) -> String {
    use yara_x_parser::ast::Error as E;
    let (message, span) = match error {
        E::SyntaxError { message, span }
        | E::InvalidInteger { message, span }
        | E::InvalidFloat { message, span }
        | E::InvalidRegexpModifier { message, span }
        | E::InvalidEscapeSequence { message, span } => (message.as_str(), span),
        E::InvalidUTF8(span) => ("invalid UTF-8", span),
        E::UnexpectedEscapeSequence(span) => ("unexpected escape sequence", span),
    };
    let before = &text.as_bytes()[..span.start().min(text.len())];
    let line_start = before.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
    let line = before.iter().filter(|&&b| b == b'\n').count() + 1;
    format!("line {line}, column {}: {message}", before.len() - line_start + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enforcement_requires_consistent_error_rates_and_class() {
        for (class, active, fp, missed, valid) in [
            (Some("full"), true, 0.0, 0.0, true),
            (Some("partial"), true, 0.0, 0.25, true),
            (None, false, 0.1, 0.5, true),
            (Some("full"), true, 0.01, 0.0, false),
            (Some("partial"), true, 0.0, 1.0, false),
            (None, true, 0.0, 0.0, false),
            (Some("full"), false, 0.0, 0.0, true),
        ] {
            let class = class.map_or(String::new(), |c| format!("class = \"{c}\" "));
            let source = format!(
                "rule example {{ meta: label = \"png\" {class}enforced = {active} \
                 fp_rate = {fp} fn_rate = {missed} condition: uint8(0) == 1 }}"
            );
            assert_eq!(Source::parse(&source).is_ok(), valid, "{source}");
        }
    }
}
