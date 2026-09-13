// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Parsed, validated YARA text.

use std::collections::HashSet;

use yara_x_parser::ast::{Item, MetaValue, Rule, RuleFlags, AST};

use crate::Error;

/// Which ruleset directory a rule ships in; checked against its metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bucket {
    /// `rulesets/full`: enforced, no observed false positive or false negative.
    Full,
    /// `rulesets/partial`: enforced, no observed false positive, some false negatives.
    Partial,
    /// `rulesets/notworking`: kept for reference, never enforced.
    NotWorking,
}

/// Declared coverage class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// `class = "full"`.
    Full,
    /// `class = "partial"`.
    Partial,
    /// `class = "not-working"`.
    NotWorking,
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
    /// The bucket the rule was validated against, if any.
    pub bucket: Option<Bucket>,
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
        Self::parse_inner(text, None)
    }

    /// Like `parse`, also checking every rule's metadata agrees with `bucket`.
    pub fn parse_in_bucket(text: &str, bucket: Bucket) -> Result<Self, Error> {
        Self::parse_inner(text, Some(bucket))
    }

    /// The rules shipped with this crate. `build.rs` validated them; a failure is a build bug.
    #[cfg(feature = "bundled")]
    pub fn bundled() -> Self {
        Self::parse(include_str!(concat!(env!("OUT_DIR"), "/bundled.yar")))
            .expect("bundled rules validated at build time")
    }

    /// The validated YARA text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Metadata of every non-private rule, in source order.
    pub fn rules(&self) -> &[RuleInfo] {
        &self.rules
    }

    fn parse_inner(text: &str, bucket: Option<Bucket>) -> Result<Self, Error> {
        if text.len() > Self::MAX_BYTES {
            return Err(Error::Parse("source exceeds 4 MiB".into()));
        }
        let ast = AST::from(text);
        if !ast.errors().is_empty() {
            return Err(Error::Parse(format!("{:?}", ast.errors())));
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
            rules.push(Self::info(rule, bucket)?);
        }
        Ok(Source { text: text.to_string(), rules })
    }

    fn info(rule: &Rule<'_>, bucket: Option<Bucket>) -> Result<RuleInfo, Error> {
        let id = rule.identifier.name.to_string();
        let fail = |reason: &str| Error::Metadata { rule: id.clone(), reason: reason.into() };
        let metadata = metadata::Metadata::read(rule).map_err(|reason| fail(reason))?;
        let enforced = metadata.enforced(bucket).map_err(|reason| fail(&reason))?;
        let mut label = None;
        for meta in rule.meta.iter().flatten() {
            match (meta.identifier.name, &meta.value) {
                ("label", MetaValue::String((value, _))) => {
                    if label.replace(value.to_string()).is_some() {
                        return Err(fail("duplicate label"));
                    }
                }
                ("label", _) => return Err(fail("label must be a string")),
                _ => {}
            }
        }
        if enforced && label.is_none() {
            return Err(fail("enforced rule needs a label"));
        }
        Ok(RuleInfo { class: metadata.class, id, label, enforced, bucket })
    }
}

mod metadata {
    //! Ported from `rust/lib/src/rules/metadata.rs` of #1447. Metadata declares the
    //! caller's evaluation results; it does not establish them.

    use yara_x_parser::ast::{MetaValue, Rule};

    use super::{Bucket, Class};

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
                            "not-working" => Class::NotWorking,
                            _ => return Err("class must be full, partial or not-working"),
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

        /// Whether the rule is enforced, checking its evidence and, when given, its bucket.
        pub(super) fn enforced(&self, bucket: Option<Bucket>) -> Result<bool, String> {
            if let Some(bucket) = bucket {
                let agrees = matches!(
                    (bucket, self.class, self.active),
                    (Bucket::Full, Class::Full, true)
                        | (Bucket::Partial, Class::Partial, true)
                        | (Bucket::NotWorking, Class::NotWorking, false)
                );
                if !agrees {
                    return Err(format!("metadata disagrees with its {bucket:?} directory"));
                }
            }
            if !self.active {
                return Ok(false);
            }
            if self.fp_rate != Some(0.0) {
                return Err("enforced rules require fp_rate = 0".into());
            }
            let consistent = match (self.class, self.fn_rate) {
                (Class::Full, Some(rate)) => rate == 0.0,
                (Class::Partial, Some(rate)) => rate > 0.0 && rate < 1.0,
                _ => false,
            };
            if !consistent {
                return Err("enforced rules require class full with fn_rate = 0, \
                            or partial with 0 < fn_rate < 1"
                    .into());
            }
            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_membership_requires_matching_evidence_and_enablement() {
        for (bucket, class, active, fp, missed, valid) in [
            (Bucket::Full, "full", true, 0.0, 0.0, true),
            (Bucket::Partial, "partial", true, 0.0, 0.25, true),
            (Bucket::NotWorking, "not-working", false, 0.1, 0.5, true),
            (Bucket::Full, "partial", true, 0.0, 0.25, false),
            (Bucket::Full, "full", true, 0.01, 0.0, false),
            (Bucket::Partial, "partial", true, 0.0, 1.0, false),
            (Bucket::NotWorking, "full", true, 0.0, 0.0, false),
            (Bucket::Full, "full", false, 0.0, 0.0, false),
        ] {
            let source = format!(
                "rule example {{ meta: label = \"png\" class = \"{class}\" enforced = {active} \
                 fp_rate = {fp} fn_rate = {missed} condition: uint8(0) == 1 }}"
            );
            assert_eq!(Source::parse_in_bucket(&source, bucket).is_ok(), valid, "{source}");
        }
    }
}
