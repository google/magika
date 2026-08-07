# Magika runtime feasibility benchmark

This standalone, non-production crate measures the existing ONNX Runtime path against a minimal
tract NNEF path. It intentionally does not change the published `magika` library or CLI.

The converter is pinned to tract `0.23.4` and writes a deterministic gzip-compressed NNEF/tract-OPL
archive. During conversion it replaces the model's one-hot-plus-matmul embedding with an equivalent
gather and replaces both exporter-expanded GELU chains with tract's fused `GeluApproximate` op:

```sh
rust/tract-bench/scripts/convert-model.sh
rust/tract-bench/scripts/convert-model.sh --check
```

Compare ONNX and NNEF model sizes using identical raw, gzip, and zstd representations:

```sh
rust/tract-bench/scripts/measure-size.sh
```

The checked NNEF artifact retains a symbolic batch dimension only as a portable storage artifact.
Performance runs use `--fixed-batch`, which binds that symbol before tract's target-specific
optimization/codegen phase. This produces a concrete CPU or Metal plan; there is no symbolic shape
on the hot path. A concrete artifact can also be exported for inspection:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml \
  --no-default-features --features convert --bin convert-model -- \
  --batch 128 assets/models/standard_v3_3/model.onnx /tmp/model.fixed-128.nnef.tgz
```

The converter performs the embedding and GELU rewrites before serialization, replaces
exporter-generated shape inputs with equivalent tract dimensions, and removes two full-range
slices. `--check` verifies deterministic conversion. tract's portable NNEF stores the decluttered
typed graph; CPU/Metal kernel selection is intentionally performed after load and is not serialized.

Export ONNX Runtime's Level-3 CPU-optimized ONNX for comparison with the tract input pipeline:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release \
  --bin optimize-onnx -- assets/models/standard_v3_3/model.onnx /tmp/model.ort-level3.onnx
```

Verify score and winning-label parity without collecting timings:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- --verify
```

Verify changing and partial batch shapes through the same prepared runtime state:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- --verify-batches
```

The crate sets development builds to `opt-level = 3`, so an ordinary `cargo run` is suitable for
fast iteration. Recorded comparisons still use `--release`, which additionally enables full LTO,
one codegen unit, and stripping.

The CPU backend reuses one tract execution state and defaults its matrix-multiplication executor to
the smaller of two threads and the host's available parallelism. Four threads made the operators
faster in isolation on the M5 Max, but two gave the best end-to-end latency and tail behavior for
this batch-one harness. Use `--tract-threads N` to reproduce the thread-count comparison. Before
model preparation the harness defaults `TRACT_LAZY_IM2COL_MIN_KERNEL` to `5`, selecting tract's
lazy convolution lowering for Magika's width-five kernel at batch one. An environment value set by
the caller takes precedence. tract 0.23.4 cannot use this lowering when batch is greater than one.

For fixed large CPU batches, `--direct-fused-conv` replaces Magika's exact
Conv1D -> GELU -> max-over-position chain with a benchmark-only `DirectFusedConvMax1D` operator. It
packs the constant weights once, generates batched convolution panels on demand, adds bias inside
tract's optimized matrix-multiplication kernel, applies SIMD GELU in place, and updates the final
per-channel maxima before releasing each bounded tile. It therefore has no eager `Im2col`
allocation and never materializes the full `[batch, positions, channels]` activation. This path
uses the current tract executor, so its matrix multiplication remains multithreaded. It deliberately
requires `--fixed-batch`; it is not a new dynamic execution path.

The default tile contains four batch items, about 4 MiB for `standard_v3_3`, regardless of the
fixed plan's full batch. `MAGIKA_DIRECT_TILE_BATCHES=N` is a preparation-time benchmark knob.
Values from 4 through 64 produced equivalent batch-64 throughput on the M5 Max, so four remains the
memory-bounded default. The rewrite derives sequence and channel dimensions from the typed graph
and activates only after checking the exact convolution, canonical approximate GELU, reduction
axis, constants, layouts, and concrete facts. A non-matching future model remains on ordinary tract
rather than failing model preparation.

The checked-in converter currently targets the JAX-exported `standard_v3_3` graph. Historical
`standard_v2_1` and `fast_v2_1` use different converter node names and exact-erf GELU graphs; they
need a separate conversion/fusion adapter before this benchmark can claim accelerated NNEF support.

Collect cold preparation, first inference, and warm timing samples:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- \
  --batch 1 --fixed-batch --iterations 100
```

Inspect the target-specific optimized operator plan and profile one CPU inference:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- \
  --backend cpu --batch 128 --fixed-batch --plan-summary --profile-plan --verify
```

Compare the direct large-batch CPU kernel with tract's built-in eager-im2col lowering:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- \
  --backend cpu --batch 128 --fixed-batch --direct-fused-conv \
  --iterations 10 --plan-summary --verify
```

Use release mode for this comparison: full LTO materially improves the custom panel-generation hot
loop. The direct path is not the batch-one default; tract's built-in batch-one `LazyIm2col` remains
faster for that compute class.

Prepare and warm every fixed class up front with a resident plan pool:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release --features metal -- \
  --backend metal --plan-pool 1,8,16,32,64 --pool-routing exact \
  --batch-sweep --iterations 5
```

Each class is independently bound, transformed with `MetalTransform`, optimized, spawned, and
warmed. Switching classes is then only a lookup of an already resident mutable state; no model
conversion, optimization, or pipeline compilation occurs on the request path. Exact routing avoids
padding by composing classes in descending order: request 10 becomes `8+1+1`, while 16 uses the
single class-16 state. `--pool-routing ceil` retains the padded single-plan comparison.

This Metal-only mode does not use the custom CPU convolution and does not send tails to CPU. It
keeps CPU capacity available for extraction and unrelated work. The CPU pool remains available as a
separate fallback benchmark; with `--direct-fused-conv`, it uses built-in `LazyIm2col` for class 1
and direct fused convolution/max reduction for classes 8 and larger.

Measure the compute stage after accumulating representative CLI batches. The harness constructs
each contiguous `[batch, 2048]` input before timing and reports batch latency, mean latency per
file, and files per second:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- \
  --batch-sweep --iterations 200
```

Benchmark the new bounded accumulate/compute design with dedicated owner threads. tract prepares one
immutable runnable shared by the owners, while every owner constructs and retains its own mutable
state; inference never runs on a Tokio executor thread:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- \
  --compute-owners 2 --batch 8 --iterations 200 --tract-threads 4
```

Treat owner count and tract threads per owner as one CPU budget. Compare combinations such as
`1 x 8`, `2 x 4`, and `4 x 2`; do not derive owner count from the ORT CLI's task default.

Repeat some trials with `--reverse` so backend order does not systematically favor the first or
second runtime through cache, thermal, or sustained-load effects.

On Apple platforms, compile Metal into the comparison explicitly:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release \
  --features metal -- --batch 128 --fixed-batch --iterations 10
```

Use `--backend metal` for isolated runs and `--metal-gemm mlx|mfa|ggml` to compare tract's Metal
GEMM kernels. Ten iterations at batch 128 represent ten consecutive accumulated compute calls and
1,280 classified files; a single fixed batch 1,280 can be measured separately rather than assumed
to have equivalent throughput.

Build runtime-specific binaries with `--no-default-features` plus either `ort-runtime` or
`tract-runtime` when comparing deployable executable size. The default features compile both
runtimes so the parity comparison uses identical feature tensors in one process.
