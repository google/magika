# Magika runtime feasibility benchmark

This standalone, non-production crate measures the existing ONNX Runtime path against a minimal
tract NNEF path. It intentionally does not change the published `magika` library or CLI.

The converter is pinned to tract `0.23.4` and writes a deterministic gzip-compressed NNEF/tract-OPL
archive:

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
