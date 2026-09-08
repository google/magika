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

The runtime consumes the `.nnef.tgz` file directly. Gzip is therefore the release codec. For a
codec-neutral size comparison, the size script also compresses equivalent raw ONNX and NNEF tar
representations with zstd-19:

```sh
rust/tract-bench/scripts/measure-size.sh
```

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

### Linux end-to-end CLI comparison

On 2026-08-21, release builds of `origin/main` (ONNX Runtime, commit `94ffd1d`) and this
branch (tract) were run over the same 4,000-file corpus on an 8-core/16-thread Intel Xeon
2.8 GHz Google Cloud VM. The 76 MiB corpus was made by cycling the repository's 185
`tests_data` fixtures into distinct files. Each binary processed the corpus recursively with its
production defaults and stdout discarded, so the measurement includes startup, traversal, feature
extraction, batching, inference, and ordered output.

Both binaries were warmed once. Seven measured pairs alternated which runtime ran first, and the
machine was checked for competing build or test processes before and after each block.

| Runtime | Median wall time | Range | Median throughput |
| --- | ---: | ---: | ---: |
| `origin/main`, ONNX Runtime | 5.181 s | 5.061–5.533 s | 772 files/s |
| this branch, tract CPU | 3.730 s | 3.711–3.786 s | 1,072 files/s |

The tract CLI delivered 1.39 times the throughput, or 28.0% lower wall time, end to end.

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
