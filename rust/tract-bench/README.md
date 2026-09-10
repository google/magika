# Magika tract release tools

This crate contains release tooling for the Rust tract runtime:

- `convert-model` converts ONNX to a deterministic, optimized, gzip-compressed NNEF archive.
- `verify-model` compares an arbitrary converted NNEF model with its ONNX source.
- `magika-runtime-bench` measures the exact `magika-tract-runtime` implementation shipped by the
  Rust library and CLI. It does not rebuild or approximate the production graph.

## Model release

Convert a compatible ONNX model:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml \
  --no-default-features --features convert --bin convert-model -- \
  model.onnx model.nnef.tgz
```

For the embedded release model, use the checked script:

```sh
rust/tract-bench/scripts/convert-model.sh
rust/tract-bench/scripts/convert-model.sh --check
rust/tract-bench/scripts/verify-model-conversion.sh
```

The release conversion also writes `model.probe.f32le`, the batch-one CPU score vector used to
reject a GPU that loads successfully but computes incorrect results. The NNEF archive must
regenerate byte-for-byte on every target. Floating-point probe scores can vary slightly by
architecture, so the gate compares the checked probe numerically with the current CPU runtime
instead. It also checks every bundled historical model against ONNX at batches 1, 8, 16, 32, and 64
and runs the production CPU and GPU graph contracts.

The production runtime loads `model.graph.json` and `model.weights`, with `model.probe.f32le`
for GPU startup validation. The crate also includes `model.nnef.tgz` for release tooling and
source-versus-artifact tests. The current model files occupy 3,213,905 bytes for production
artifacts and 2,917,050 bytes for NNEF, or 6,130,955 bytes combined before crate compression.
These are model-file sizes, not executable or compressed crate sizes.

The size script compares ONNX and NNEF representations under raw, gzip and zstd-19 codecs.
Its output does not measure the production artifacts or the total packaged model size:

```sh
rust/tract-bench/scripts/measure-size.sh
```

## ONNX reference dependency

The optional ONNX reference uses `ort` and `ort-sys` 2.0.0-rc.12 from the lockfile.
That dependency includes a target-specific table of ONNX Runtime 1.24.2 download
URLs and SHA-256 hashes. Its build script checks the downloaded archive against
that table before accepting it into the cache. A fresh reference build needs
network access unless compatible libraries are already provided; Cargo's crate
cache alone does not provide the native library. For an offline reference build,
provision the matching native libraries and set `ORT_LIB_PATH` as described in
[ort's linking documentation](https://ort.pyke.io/setup/linking).

The production library and CLI do not depend on `ort`. CPU/GPU benchmark commands
with `--no-default-features` also omit this reference dependency.

## Runtime benchmark

Each invocation measures one backend in an isolated process. CPU and GPU use the production
`Runtime` and `Session`; ONNX Runtime uses independently pinned intra- and inter-op thread counts.
`--threads` means resident inference threads, matching the production CLI.

```sh
# Production CPU runtime.
cargo run --release --manifest-path rust/tract-bench/Cargo.toml \
  --no-default-features -- --backend cpu --batch 8 --threads 4 --iterations 100

# Production Metal runtime on macOS (or the compiled GPU implementation on another target).
cargo run --release --manifest-path rust/tract-bench/Cargo.toml \
  --no-default-features -- --backend gpu --batch 8 --threads 4 --iterations 100

# ONNX Runtime reference with explicit thread ownership.
cargo run --release --manifest-path rust/tract-bench/Cargo.toml -- \
  --backend ort --batch 8 --threads 4 --iterations 100 \
  --ort-intra-threads 1 --ort-inter-threads 1
```

The benchmark reports shared runtime preparation, per-thread session preparation and explicit
warm-up, timed wall duration, and files per second. Use `/usr/bin/time -lp` around one backend
invocation when recording peak memory. Alternate backend order across short trials to reduce cache
and thermal bias.

### The machine must be idle

Every inference thread runs flat out, so anything else competing for a core is subtracted straight
from the result, and it is rarely subtracted evenly from both backends. Results collected while a
build or test suite is running are not comparable and must be discarded.

Before quoting a number, check that nothing is building: no `cargo` or `rustc` process, and a load
average near zero. Take at least two trials with the backend order reversed and compare medians. A
single sample distinguishes nothing here; differences below about 5% are within run-to-run drift.

### Retuning the x86_64 convolution tile

`MAGIKA_DIRECT_TILE_COLUMNS` overrides how many output columns the fused convolution packs at a
time. The compiled default is sized so a tile stays inside a 256KiB L2, since throughput measured
flat across a wide band of tile sizes on the one machine available and the safe end of a flat band
travels better than its middle. On a machine with a larger private L2, or one whose L2 is not
shared between hyperthreads, sweeping this is the way to check whether the default still fits:

```sh
for columns in 24 48 96 144 192; do
  MAGIKA_DIRECT_TILE_COLUMNS=$columns cargo run --release \
    --manifest-path rust/tract-bench/Cargo.toml --no-default-features -- \
    --backend cpu --batch 8 --threads "$(nproc)" --iterations 100
done
```

End-to-end CLI results must be measured separately because traversal, feature extraction, global
batch accumulation, ordered output, startup, and backend auto-selection are intentionally outside
the compute-only benchmark.

## File classification and rules

The [cross-tool benchmark](../../rules/benchmarks/README.md) measures whole-process
CLI accuracy, coverage and timing with shipping defaults. The separate
[rule evaluation tool](../../rules/README.md#run-the-benchmark-and-report) qualifies
rule precision and compares explicit backend/worker configurations.
