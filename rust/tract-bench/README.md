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

The benchmark reports shared runtime preparation, per-thread session preparation and warming, timed
wall duration, and files per second. Use `/usr/bin/time -lp` around one backend invocation when
recording peak memory. Alternate backend order across short trials to reduce cache and thermal bias.

End-to-end CLI results must be measured separately because traversal, feature extraction, global
batch accumulation, ordered output, startup, and backend auto-selection are intentionally outside
the compute-only benchmark.
