// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! The bundled rulesets expose enforced rules.

#![cfg(feature = "bundled")]
use magika_rules::RuleSet;

#[test]
fn bundled_rules_have_enforced_rules() {
    assert!(RuleSet::bundled().rules().iter().any(|r| r.enforced));
}
