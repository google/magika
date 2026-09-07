// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_magika"))
}

#[test]
fn rules_mode_is_explicit_and_batch_order_is_preserved() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests_data");
    let paths = [root.join("basic/png/magika_test.png"), root.join("basic/pdf/magika_test.pdf")];
    // Use two existing files, repeated, to exercise both full and partial batches.
    let paths: Vec<_> = paths.iter().cycle().take(9).collect();
    let run = |mode: &str| {
        command()
            .args([
                "--rules",
                mode,
                "--batch-size",
                "8",
                "--threads",
                "2",
                "--readers",
                "2",
                "--backend",
                "cpu",
                "--jsonl",
            ])
            .args(&paths)
            .output()
            .unwrap()
    };
    let baseline = run("off");
    assert!(baseline.status.success(), "{}", String::from_utf8_lossy(&baseline.stderr));
    let rows: Vec<serde_json::Value> = String::from_utf8(baseline.stdout.clone())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(rows.len(), paths.len());
    for (row, path) in rows.iter().zip(&paths) {
        assert_eq!(row["path"].as_str().unwrap(), path.to_str().unwrap());
    }
    let enforced = run("enforce");
    if cfg!(feature = "yara-rules") {
        assert!(enforced.status.success());
        let actual: Vec<serde_json::Value> = String::from_utf8(enforced.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(actual.len(), rows.len());
        for (actual, expected) in actual.iter().zip(&rows) {
            assert_eq!(actual["path"], expected["path"]);
            assert_eq!(actual["result"]["value"]["output"], expected["result"]["value"]["output"]);
        }
    } else {
        assert!(!enforced.status.success());
        assert!(String::from_utf8_lossy(&enforced.stderr).contains("yara-rules"));
    }
}

#[test]
fn invalid_mode_is_rejected() {
    assert!(!command().args(["--rules", "candidates", "-"]).output().unwrap().status.success());
}

#[test]
fn partial_batch_before_directories_completes() {
    partial_batch_completes(false);
}

#[cfg(feature = "yara-rules")]
#[test]
#[ignore = "requires a native Vectorscan compiler library"]
fn partial_batch_before_rule_hits_completes() {
    partial_batch_completes(true);
}

fn partial_batch_completes(use_rules: bool) {
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let directory = std::env::temp_dir()
        .join(format!("magika-partial-batch-{}-{use_rules}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests_data");
    let first = root.join("basic/png/magika_test.png");
    let later = if use_rules { directory.join("hit.bin") } else { root };
    let mut cli = command();
    if use_rules {
        let pack = directory.join("test.yar");
        std::fs::write(
            &pack,
            r#"rule fixture { meta: label = "png" enabled = true class = "full" fp_rate = 0 fn_rate = 0
            strings: $a = "MAGIKA_BATCH_TEST" condition: $a at 0 }"#,
        )
        .unwrap();
        std::fs::write(&later, b"MAGIKA_BATCH_TEST").unwrap();
        cli.args(["--rules=enforce", "--rules-file"]).arg(pack);
    }
    let output = directory.join("output.jsonl");
    // Keep stdout draining independently of the exit deadline. One ML input followed by
    // 512 deterministic results exceed the bounded traversal window at batch size 8.
    let mut child = cli
        .args(["--jsonl", "--backend=cpu", "--batch-size=8", "--threads=1", "--readers=1"])
        .arg(&first)
        .args(std::iter::repeat_n(&later, 512))
        .stdout(std::fs::File::create(&output).unwrap())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "{status}");
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("partial inference batch stranded ordered output");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let rows: Vec<serde_json::Value> = std::fs::read_to_string(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(rows.len(), 513);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(
            row["path"].as_str().unwrap(),
            if index == 0 { &first } else { &later }.to_str().unwrap()
        );
        if use_rules && index > 0 {
            assert_eq!(row["result"]["value"]["output"]["label"], "png");
            assert_eq!(row["result"]["value"]["score"], 1.0);
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(unix)]
#[test]
fn closing_output_releases_traversal() {
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests_data");
    // The first input needs inference; later directory results can fill the reorder window.
    let mut child = command()
        .args([
            "--jsonl",
            "--rules=off",
            "--backend=cpu",
            "--batch-size=1",
            "--threads=1",
            "--readers=1",
        ])
        .arg(root.join("basic/png/magika_test.png"))
        .args(std::iter::repeat_n(&root, 512))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "{status}");
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("CLI did not stop after its output pipe closed");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn exports_real_yara_without_initializing_inference() {
    let directory = std::env::temp_dir().join(format!("magika-export-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("promoted.yar");
    let result = command().arg("--write-default-rules").arg(&path).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), magika::DEFAULT_RULES);
    assert!(!command().arg("--write-default-rules").arg(&path).output().unwrap().status.success());
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "yara-rules")]
#[test]
#[ignore = "requires a native Vectorscan compiler library"]
fn edited_pack_is_loaded_and_mixed_batches_keep_order() {
    let directory = std::env::temp_dir().join(format!("magika-pack-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let pack = directory.join("custom.yar");
    let hit = directory.join("hit.bin");
    std::fs::write(&hit, b"MAGIKA_TEST_RULE!").unwrap();
    let miss = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests_data/basic/pdf/magika_test.pdf");
    let source = r#"rule synthetic { meta: label = "png" enabled = true class = "full" fp_rate = 0 fn_rate = 0
        strings: $a = "MAGIKA_TEST_RULE!" condition: $a at 0 }"#;
    std::fs::write(&pack, source).unwrap();
    let paths: Vec<_> = [&hit, &miss].into_iter().cycle().take(257).collect();
    let run = || {
        command()
            .env("MAGIKA_RULES_CACHE", directory.join("cache"))
            .args(["--rules=enforce", "--rules-file"])
            .arg(&pack)
            .args(["--jsonl", "--batch-size=8", "--readers=2", "--threads=2", "--backend=cpu"])
            .args(&paths)
            .output()
            .unwrap()
    };
    let result = run();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let entries: Vec<_> = std::fs::read_dir(directory.join("cache"))
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|x| x.extension().is_some_and(|x| x == "hsdb"))
        .collect();
    assert_eq!(entries.len(), 1);
    let modified = std::fs::metadata(&entries[0]).unwrap().modified().unwrap();
    let repeated = run();
    assert!(repeated.status.success());
    assert_eq!(repeated.stdout, result.stdout);
    assert_eq!(std::fs::metadata(&entries[0]).unwrap().modified().unwrap(), modified);
    let rows: Vec<serde_json::Value> = String::from_utf8(result.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(rows.len(), paths.len());
    for (i, (row, path)) in rows.iter().zip(&paths).enumerate() {
        assert_eq!(row["path"].as_str().unwrap(), path.to_str().unwrap());
        assert_eq!(
            row["result"]["value"]["output"]["label"],
            if i % 2 == 0 { "png" } else { "pdf" },
            "{row}"
        );
    }
    // Reload the installed file in a new invocation: metadata selects individual rules.
    std::fs::write(&pack, source.replace("enabled = true", "enabled = false")).unwrap();
    let disabled = run();
    let baseline = command()
        .args(["--jsonl", "--batch-size=8", "--readers=2", "--threads=2", "--backend=cpu"])
        .args(&paths)
        .output()
        .unwrap();
    assert!(disabled.status.success());
    assert_eq!(disabled.stdout, baseline.stdout);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn compiled_empty_export_is_feature_gated_and_needs_no_native_library() {
    let directory =
        std::env::temp_dir().join(format!("magika-compiled-export-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("empty.yar"), "// Empty pack fixture.\n").unwrap();
    let compile = || {
        command()
            .current_dir(&directory)
            .env("MAGIKA_VECTORSCAN_LIBRARY", directory.join("missing-library"))
            .args(["--compile-rules", "empty.yar"])
            .output()
            .unwrap()
    };
    let result = compile();
    if cfg!(feature = "yara-rules") {
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        assert!(directory.join("empty.hsdb").is_file());
        let original = std::fs::read(directory.join("empty.hsdb")).unwrap();
        assert!(!compile().status.success());
        assert_eq!(std::fs::read(directory.join("empty.hsdb")).unwrap(), original);
    } else {
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("yara-rules"));
        assert!(!directory.join("empty.hsdb").exists());
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "yara-rules")]
#[test]
#[ignore = "requires a native Vectorscan compiler library"]
fn installed_source_pack_and_library_are_discovered() {
    // Build a relocatable installation, then remove the native-library override. This checks
    // actual executable-relative discovery rather than relying on a developer's search path.
    let directory =
        std::env::temp_dir().join(format!("magika-installed-pack-{}", std::process::id()));
    std::fs::create_dir_all(directory.join("rules")).unwrap();
    std::fs::create_dir_all(directory.join("lib")).unwrap();
    let binary = directory.join(if cfg!(windows) { "magika.exe" } else { "magika" });
    std::fs::copy(env!("CARGO_BIN_EXE_magika"), &binary).unwrap();
    let library = std::env::var_os("MAGIKA_VECTORSCAN_LIBRARY")
        .expect("test needs an explicit native library path");
    let library_name = if cfg!(target_os = "macos") {
        "libhs.dylib"
    } else if cfg!(windows) {
        "hs.dll"
    } else {
        "libhs.so.5"
    };
    std::fs::copy(library, directory.join("lib").join(library_name)).unwrap();
    let source = directory.join("rules/promoted.yar");
    let text = r#"rule fixture { meta: label = "png" enabled = true class = "full" fp_rate = 0 fn_rate = 0
        strings: $a = "MAGIKA_INSTALLED_TEST" condition: $a at 0 }"#;
    std::fs::write(&source, text).unwrap();
    let input = directory.join("input.bin");
    std::fs::write(&input, b"MAGIKA_INSTALLED_TEST").unwrap();
    let mut compile = Command::new(&binary);
    let result = compile
        .env_remove("MAGIKA_VECTORSCAN_LIBRARY")
        .arg("--compile-rules")
        .arg(&source)
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let run = || {
        Command::new(&binary)
            .env_remove("MAGIKA_VECTORSCAN_LIBRARY")
            .env("MAGIKA_RULES_CACHE", directory.join("cache"))
            .args(["--rules=enforce", "--label", "--backend=cpu"])
            .arg(&input)
            .output()
            .unwrap()
    };
    let matched = run();
    assert!(matched.status.success(), "{}", String::from_utf8_lossy(&matched.stderr));
    assert!(String::from_utf8_lossy(&matched.stdout).trim_end().ends_with(": png"));
    assert!(!directory.join("cache").exists());
    std::fs::write(&source, text.replace("\"png\"", "\"gif\"")).unwrap();
    let edited = run();
    assert!(edited.status.success());
    assert!(String::from_utf8_lossy(&edited.stdout).trim_end().ends_with(": gif"));
    assert!(directory.join("cache").is_dir());
    std::fs::remove_dir_all(directory).unwrap();
}
