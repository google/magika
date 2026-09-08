// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0
use sha2::{Digest, Sha256};
use std::{hint::black_box, time::Instant};

fn main() {
    let mut args = std::env::args().skip(1);
    let pack = args.next().expect("mapped pack path");
    let mode = args.next().expect("sha256, blake3 or blake3-rayon");
    let data = std::fs::read(pack).unwrap();
    let n = u32::from_le_bytes(data[9..13].try_into().unwrap()) as usize;
    let manifest: serde_json::Value = serde_json::from_slice(&data[13..13+n]).unwrap();
    let metadata = serde_json::to_vec(&(&manifest["key"], &manifest["engine"], &manifest["labels"])).unwrap();
    let payload = &data[(13+n).next_multiple_of(4096)..];
    let mut durations = Vec::new();
    let mut expected = None;
    for _ in 0..21 {
        let start = Instant::now();
        let digest: [u8; 32] = match mode.as_str() {
            "sha256" => {
                let mut h = Sha256::new();
                h.update(black_box(&metadata));
                h.update(black_box(payload));
                h.finalize().into()
            }
            "blake3" | "blake3-rayon" => {
                let mut h = blake3::Hasher::new();
                h.update(black_box(&metadata));
                if mode == "blake3-rayon" { h.update_rayon(black_box(payload)); }
                else { h.update(black_box(payload)); }
                *h.finalize().as_bytes()
            }
            _ => panic!("unknown mode"),
        };
        durations.push(start.elapsed().as_nanos() as u64);
        let digest = black_box(digest);
        if let Some(expected) = expected { assert_eq!(digest, expected); }
        expected = Some(digest);
    }
    println!("{}", serde_json::json!({"mode":mode, "payload_bytes":payload.len(),
        "metadata_bytes":metadata.len(), "durations_ns":durations, "digest":expected.unwrap()}));
}
