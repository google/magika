# Cargo-option audit

The original CLI build enabled opt-level=3, LTO and one codegen unit. Its retained
Cargo output confirms `blake3_neon` and its fingerprint confirms no target CPU
override. The first experiment did not enable Rayon.

This small harness measures the exact same manifest and mapped payload for SHA-256,
BLAKE3 and BLAKE3 with Rayon. All builds enable SHA assembly and BLAKE3 NEON.
The native build adds both Rust `-C target-cpu=native` and C `-mcpu=native`;
setting Rust flags alone would miss the C NEON implementation. Rayon runs with
four threads, including thread pool creation in the first timed call.

[Generated results](results/report.md) and [raw samples](results/measurements.json)
show that native tuning alone did not improve sequential BLAKE3 here. Rayon did
reduce hash time. These memory-resident hash timings exclude file reads, manifest
serialization, CLI startup and shutdown; the actual CLI needs its own measurement.

Reproduce from this directory with separate target directories:

```sh
CARGO_TARGET_DIR=/tmp/magika-hash-generic cargo build --release --locked -vv
RUSTFLAGS='-C target-cpu=native' CFLAGS='-mcpu=native' \
  CARGO_TARGET_DIR=/tmp/magika-hash-native cargo build --release --locked -vv
python measure.py /tmp/magika-hash-generic/release/magika-hash-options \
  /tmp/magika-hash-native/release/magika-hash-options MAPPED_PACK NEW_RESULT_DIRECTORY
```

Verbose build logs and original CLI backend-selection records are retained.

The follow-up [Rust-only native results](results-rust-only/report.md) sets exactly
`RUSTFLAGS='-C target-cpu=native'`, leaving CFLAGS unset. Sequential BLAKE3 warm
medians were 0.6755 ms with that flag and 0.6759 ms without it. Its four-thread Rayon
first-call median was 0.4112 ms; the SHA-256 first-call median was 0.5019 ms.
Both runs retain their individual raw samples. Reproduce with a third target
directory, then pass `--rust-only` to `measure.py` with that binary.
