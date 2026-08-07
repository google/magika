# Magika runtime feasibility benchmark

This standalone, non-production crate measures the existing ONNX Runtime path against a minimal
tract NNEF path. It intentionally does not change the published `magika` library or CLI.

The converter is pinned to tract `0.23.4` and writes a deterministic gzip-compressed NNEF/tract-OPL
archive. During conversion it replaces the model's one-hot-plus-matmul embedding with an equivalent
gather, avoiding a large temporary tensor in tract's CPU path:

```sh
rust/tract-bench/scripts/convert-model.sh
rust/tract-bench/scripts/convert-model.sh --check
```

The current NNEF artifact is specialized to a batch of one. tract 0.23.4 cannot resolve the ONNX
graph's one-hot broadcast while retaining the symbolic batch dimension, so this first harness
measures the common single-file path and treats dynamic batching as an integration blocker.

Verify score and winning-label parity without collecting timings:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- --verify
```

The crate sets development builds to `opt-level = 3`, so an ordinary `cargo run` is suitable for
fast iteration. Recorded comparisons still use `--release`, which additionally enables full LTO,
one codegen unit, and stripping.

The CPU backend reuses one tract execution state and defaults its matrix-multiplication executor to
the smaller of two threads and the host's available parallelism. Four threads made the operators
faster in isolation on the M5 Max, but two gave the best end-to-end latency and tail behavior for
this batch-one harness. Use `--tract-threads N` to reproduce the thread-count comparison.

Collect cold preparation, first inference, and warm timing samples:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- \
  --batch 1 --iterations 100
```

Repeat some trials with `--reverse` so backend order does not systematically favor the first or
second runtime through cache, thermal, or sustained-load effects.

On Apple platforms, compile Metal into the comparison explicitly:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release \
  --features metal -- --batch 1 --iterations 100
```

Build runtime-specific binaries with `--no-default-features` plus either `ort-runtime` or
`tract-runtime` when comparing deployable executable size. The default features compile both
runtimes so the parity comparison uses identical feature tensors in one process.
