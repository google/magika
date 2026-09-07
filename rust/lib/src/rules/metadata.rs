// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, ensure, Result};
use yara_x_parser::ast::{MetaValue, Rule, RuleFlags};

// Metadata declares the caller's evaluation results; it does not establish them.
// Validate before lowering, including terminals reached through rule references.
pub(super) fn enforced(rule: &Rule<'_>, bucket: Option<&str>) -> Result<bool> {
    let (mut enabled, mut enforced, mut class, mut fp, mut fn_rate) =
        (None, None, None, None, None);
    for meta in rule.meta.iter().flatten() {
        match (meta.identifier.name, &meta.value) {
            ("enabled" | "enforced", MetaValue::Bool((value, _))) => {
                let slot =
                    if meta.identifier.name == "enabled" { &mut enabled } else { &mut enforced };
                ensure!(slot.replace(*value).is_none(), "duplicate enforcement metadata");
            }
            ("class", MetaValue::String((value, _))) => {
                ensure!(class.replace(*value).is_none(), "duplicate class metadata");
            }
            ("fp_rate" | "fn_rate", value) => {
                let value = match value {
                    MetaValue::Integer((value, _)) => Some(*value as f64),
                    MetaValue::Float((value, _)) => Some(*value),
                    MetaValue::String(("unmeasured", _)) => None,
                    _ => bail!("error rates must be numeric fractions"),
                };
                ensure!(
                    value.is_none_or(|v| v.is_finite() && (0.0..=1.0).contains(&v)),
                    "error rates must be in [0, 1]"
                );
                let slot = if meta.identifier.name == "fp_rate" { &mut fp } else { &mut fn_rate };
                ensure!(slot.replace(value).is_none(), "duplicate error rate metadata");
            }
            ("enabled" | "enforced" | "class", _) => {
                bail!("enforcement must be boolean and class a string")
            }
            _ => (),
        }
    }
    ensure!(
        enabled.is_none() || enforced.is_none() || enabled == enforced,
        "enabled and enforced disagree"
    );
    let active = enforced.or(enabled).unwrap_or(rule.flags.contains(RuleFlags::Private));
    if let Some(bucket) = bucket.filter(|_| !rule.flags.contains(RuleFlags::Private)) {
        ensure!(
            matches!(
                (bucket, class, active),
                ("full", Some("full"), true)
                    | ("partial", Some("partial"), true)
                    | ("notworking", Some("not-working"), false)
            ),
            "rule {} metadata disagrees with its {bucket} directory",
            rule.identifier.name
        );
    }
    if !active || rule.flags.contains(RuleFlags::Private) {
        return Ok(active);
    }
    ensure!(fp.flatten() == Some(0.0), "enforced rules require fp_rate = 0");
    ensure!(
        matches!((class, fn_rate.flatten()), (Some("full"), Some(0.0)))
            || matches!((class, fn_rate.flatten()), (Some("partial"), Some(rate)) if rate > 0.0 && rate < 1.0),
        "enforced rules require class full with fn_rate = 0, or partial with 0 < fn_rate < 1"
    );
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yara_x_parser::ast::AST;

    #[test]
    fn bucket_membership_requires_matching_evidence_and_enablement() {
        for (bucket, class, active, fp, missed, valid) in [
            ("full", "full", true, 0.0, 0.0, true),
            ("partial", "partial", true, 0.0, 0.25, true),
            ("notworking", "not-working", false, 0.1, 0.5, true),
            ("full", "partial", true, 0.0, 0.25, false),
            ("full", "full", true, 0.01, 0.0, false),
            ("partial", "partial", true, 0.0, 1.0, false),
            ("notworking", "full", true, 0.0, 0.0, false),
            ("full", "full", false, 0.0, 0.0, false),
        ] {
            let source = format!(
                "rule example {{ meta: label = \"png\" class = \"{class}\" enforced = {active} fp_rate = {fp} fn_rate = {missed} condition: uint8(0) == 1 }}"
            );
            let ast = AST::from(source.as_str());
            assert!(ast.errors().is_empty());
            assert_eq!(enforced(ast.rules().next().unwrap(), Some(bucket)).is_ok(), valid);
        }
    }
}
