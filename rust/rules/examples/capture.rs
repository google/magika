// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Differential parity and tight-loop benchmark harness for `magika-rules`.

use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use magika_rules::{scan_native_names, Input, Outcome, RuleSet, PREFIX_LIMIT};

struct Sample {
    path: String,
    prefix: Vec<u8>,
    size: u64,
    tail: Option<Vec<u8>>,
}

impl Sample {
    fn as_input(&self) -> Input<'_> {
        Input { prefix: &self.prefix, size: self.size, tail: self.tail.as_deref() }
    }
}

fn collect_paths(root: &Path, out: &mut Vec<PathBuf>) {
    if root.is_file() {
        out.push(root.to_path_buf());
        return;
    }
    let Ok(entries) = std::fs::read_dir(root) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_paths(&p, out);
        } else if p.is_file() {
            out.push(p);
        }
    }
}

fn main() {
    let mut native = false;
    let mut bench = false;
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--native" => native = true,
            "--bench" => bench = true,
            "--list" => {
                let list_path = args.next().expect("--list requires a FILE argument");
                let text = std::fs::read_to_string(list_path).expect("failed to read --list file");
                for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
                    roots.push(PathBuf::from(line));
                }
            }
            _ => roots.push(PathBuf::from(arg)),
        }
    }

    let mut paths = Vec::new();
    for root in &roots {
        collect_paths(root, &mut paths);
    }
    paths.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

    let rules = RuleSet::bundled();
    let mut samples = Vec::with_capacity(paths.len());
    for path in paths {
        let Ok(bytes) = std::fs::read(&path) else { continue };
        let size = bytes.len() as u64;
        let prefix = bytes[..bytes.len().min(PREFIX_LIMIT)].to_vec();
        let mut start = bytes.len() - rules.tail_len(&prefix, size);
        if let Some(earlier) = rules.tail_start(&bytes[start..], size) {
            start = earlier as usize;
        }
        let tail = (start < bytes.len()).then(|| bytes[start..].to_vec());
        samples.push(Sample { path: path.to_string_lossy().into_owned(), prefix, size, tail });
    }

    if bench {
        run_bench(&rules, &samples, native);
        return;
    }

    // Warm up once across the sample set before timing per-file scans.
    for sample in &samples {
        if native {
            black_box(scan_native_names(black_box(sample.as_input())));
        } else {
            black_box(rules.scan(black_box(sample.as_input())));
        }
    }

    let mut times_ns = Vec::with_capacity(samples.len());
    for sample in &samples {
        let input = sample.as_input();
        let t0 = Instant::now();
        let formatted = if native {
            match scan_native_names(black_box(input)) {
                Some(labels) if labels.is_empty() => "@NONE".to_string(),
                Some(labels) => labels.join(","),
                None => "@INSUFFICIENT".to_string(),
            }
        } else {
            match rules.scan(black_box(input)) {
                Outcome::Match(indices) if indices.is_empty() => "@NONE".to_string(),
                Outcome::Match(indices) => indices
                    .into_iter()
                    .map(|i| rules.labels()[i].as_str())
                    .collect::<Vec<_>>()
                    .join(","),
                Outcome::InsufficientInput => "@INSUFFICIENT".to_string(),
            }
        };
        let dt = t0.elapsed().as_nanos() as u64;
        times_ns.push(dt);
        println!("{}\t{}", sample.path, formatted);
    }

    if !times_ns.is_empty() {
        times_ns.sort_unstable();
        let n = times_ns.len();
        let sum: u64 = times_ns.iter().sum();
        let mean = sum as f64 / n as f64;
        let median = times_ns[n / 2];
        let p90 = times_ns[n * 90 / 100];
        let p99 = times_ns[n * 99 / 100];
        eprintln!(
            "capture ({}) over {n} files: mean {mean:.1} ns, median {median} ns, p90 {p90} ns, p99 {p99} ns",
            if native { "native" } else { "scan" }
        );
    }
}

fn run_bench(rules: &RuleSet, samples: &[Sample], native: bool) {
    let inputs: Vec<Input<'_>> = samples.iter().map(Sample::as_input).collect();
    if inputs.is_empty() {
        eprintln!("no inputs to benchmark");
        return;
    }

    for inp in &inputs {
        if native {
            black_box(scan_native_names(black_box(*inp)));
        } else {
            black_box(rules.scan(black_box(*inp)));
        }
    }

    let rounds = (200_000 / inputs.len().max(1)).max(500);
    let t0 = Instant::now();
    for _ in 0..rounds {
        for inp in &inputs {
            if native {
                black_box(scan_native_names(black_box(*inp)));
            } else {
                black_box(rules.scan(black_box(*inp)));
            }
        }
    }
    let total: Duration = t0.elapsed();
    let total_scans = rounds * inputs.len();
    let ns_per_scan = total.as_nanos() as f64 / total_scans as f64;
    println!(
        "bench ({}): {} files x {} rounds = {} scans in {:?} ({:.1} ns/scan)",
        if native { "native" } else { "scan" },
        inputs.len(),
        rounds,
        total_scans,
        total,
        ns_per_scan
    );
}
