// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Compile and scan budgets. The assertions are CI ceilings, not targets, and hold for
//! optimized builds only: `test.sh` runs this test with `--release`.

#![cfg(all(feature = "bundled", not(debug_assertions)))]

use std::hint::black_box;
use std::time::Instant;

use magika_rules::{Input, Outcome, RuleSet, Source};

#[test]
fn compile_and_scan_stay_within_budget() {
    let t = Instant::now();
    let rules = RuleSet::compile(&Source::bundled()).unwrap();
    let compile = t.elapsed();

    // Worst case for anchored rules: a prefix that starts like nothing, so every rule is tried.
    let junk: Vec<u8> = (0..magika_rules::PREFIX_LIMIT).map(|i| (i * 7919 % 251) as u8).collect();
    let input = Input { prefix: &junk, size: 1 << 20, tail: None };
    for _ in 0..1000 {
        black_box(rules.scan(black_box(input)));
    }
    let t = Instant::now();
    let n = 20_000;
    for _ in 0..n {
        assert_eq!(black_box(rules.scan(black_box(input))), Outcome::NoMatch);
    }
    let per_scan = t.elapsed() / n;

    eprintln!(
        "perf: compile {compile:?}, scan {per_scan:?}, {} rules, {} labels",
        rules.rules().len(),
        rules.labels().len()
    );
    assert!(compile.as_millis() < 200, "compile {compile:?}");
    assert!(per_scan.as_micros() < 500, "scan {per_scan:?}");
}
