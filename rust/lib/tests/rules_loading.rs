// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Rule initialization errors must remain observable to library callers.

#![cfg(feature = "yara-rules")]

#[test]
fn builder_reports_missing_bundled_rules_library() {
    const CHILD: &str = "MAGIKA_TEST_RULES_LOADING_CHILD";
    if std::env::var_os(CHILD).is_some() {
        let builder =
            || magika::Runtime::builder().with_backend(magika::Backend::Cpu).with_max_batch(1);
        assert!(builder().build().is_ok());
        let error = builder()
            .with_rules_mode(magika::RulesMode::Enforce)
            .build()
            .err()
            .expect("missing rules library silently ignored");
        let error = format!("{error:#}");
        assert!(error.contains("rules") && error.contains("Vectorscan"), "{error}");
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "builder_reports_missing_bundled_rules_library", "--nocapture"])
        .env(CHILD, "1")
        .env("MAGIKA_RULES_CACHE", "")
        .env("MAGIKA_VECTORSCAN_LIBRARY", "/nonexistent-magika-regression-library")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
