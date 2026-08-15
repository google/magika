# Magika runtime feasibility benchmark

This release-engineering crate measures the former ONNX Runtime path against the tract NNEF runtime
used by the Rust `magika` library and CLI. The shared fixed-plan implementation lives in
`rust/tract-runtime`; this crate retains the ONNX parity harness, converter, profiling controls, and
release gates.

## Model release process

`convert-model` has one contract: it accepts an ONNX file and writes a portable, optimized NNEF
archive that the Rust tract runtime can load directly. It is pinned to tract `0.23.4`, performs
tract's typed portable-graph optimization, restores a symbolic leading batch dimension, and writes
a deterministic gzip-compressed tar archive. Semantic graph matching replaces one-hot matrix
multiplication with `Gather`, fuses canonical tanh-GELU math, and lowers dynamic BatchNorm to
portable arithmetic so both the v2 fast/standard models and the v3 standard models pass through the
same converter.

Convert any compatible Magika ONNX model:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml \
  --no-default-features --features convert --bin convert-model -- \
  path/to/model.onnx path/to/model.nnef.tgz
```

The `.nnef.tgz` output is the release artifact. Gzip is used because
`tract_nnef::nnef().model_for_path(...)` consumes the compressed archive directly. The size-analysis
script also recompresses the unpacked NNEF tar with zstd-19 for an equal-codec comparison; that
zstd file is a measurement artifact, not the Rust runtime package.

For the checked Magika model, regenerate or verify the artifact and then run the release gate:

```sh
rust/tract-bench/scripts/convert-model.sh
rust/tract-bench/scripts/convert-model.sh --check
rust/tract-bench/scripts/verify-model-conversion.sh
```

The release gate converts every bundled standard, fast, and beginning-only model twice, requires
byte-identical output, validates each gzip/tar, reloads each artifact through Rust, and compares
every model's scores and winning labels with its source ONNX at fixed batch classes 1, 4, 8, 16,
32, and 64. It additionally requires the release model's direct CPU and LayerNorm fusions to match and
execute.

Compare ONNX and NNEF model sizes using identical raw, gzip, and zstd representations:

```sh
rust/tract-bench/scripts/measure-size.sh
```

The NNEF artifact retains a symbolic batch dimension only for portable storage. Rust binds `N` to
the selected fixed class before tract performs target-specific optimization and kernel selection,
so there is no symbolic shape on the inference hot path. The same archive therefore supplies the
prebuilt 1, 4, 8, 16, 32, and 64 plans instead of shipping duplicate copies of the weights.

The corresponding Rust loading sequence is:

```rust
use tract_core::prelude::{Framework as _, ToDim as _};
use tract_core::runtime::{DefaultRuntime, Runtime as _};

let mut model = tract_nnef::nnef().model_for_path("model.nnef.tgz")?;
let n = model.symbols.get("N").expect("converter emits batch symbol N");
model = model.set_symbols(&std::collections::HashMap::from([(n, 8.to_dim())]))?;
let runnable = tract_core::runtime::DefaultRuntime.prepare(model)?;
let mut state = runnable.spawn()?;
```

NNEF stores the optimized portable graph. CPU and Metal code generation deliberately happens in
Rust after binding the concrete batch, because those target-specific kernels are not portable NNEF.

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
this batch-one harness. Use `--tract-threads N` to reproduce the thread-count comparison. ONNX
Runtime is pinned independently with `--ort-intra-threads N` (default: the same as tract) and
`--ort-inter-threads N` (default: 1), so owner-topology measurements never hide an unbounded ORT
thread pool. Before
model preparation the harness defaults `TRACT_LAZY_IM2COL_MIN_KERNEL` to `5`, selecting tract's
lazy convolution lowering for Magika's width-five kernel at batch one. An environment value set by
the caller takes precedence. tract 0.23.4 cannot use this lowering when batch is greater than one.

For fixed large CPU batches, `--direct-fused-conv` replaces Magika's
Conv1D -> GELU -> max-over-position chain with a `DirectFusedConvMax1D` operator. It
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
axis, constants, layouts, and concrete facts. Requesting this release optimization is fail-closed:
a non-matching model fails preparation instead of silently running the slower graph.

`--fused-layer-norm` independently replaces the release model's true-LayerNorm expansion with one
safe CPU operator. It validates the complete mean, variance clamp, epsilon, normalization, scale,
and bias graph before rewriting it, and requires a concrete fixed-batch plan. The operator uses two
contiguous activation passes and no `unsafe`; requesting it is fail-closed if the expected graph is
absent. The production GPU preparation path uses the same structural validation to lower both
LayerNorm forms to tract's GPU RMS-normalization primitive.

The high-performance release artifact is `standard_v3_3`. The v2 models remain conversion and
compatibility test inputs, but their exact-erf GELU does not match the approximate-GELU fused
kernel. They are deliberately not presented as the same CPU performance tier.

Collect cold preparation, first inference, and warm timing samples:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- \
  --batch 1 --fixed-batch --iterations 100
```

Inspect the target-specific optimized operator plan and profile one CPU inference:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- \
  --backend cpu --batch 8 --fixed-batch --direct-fused-conv --fused-layer-norm \
  --plan-summary --profile-plan --verify
```

Compare the direct large-batch CPU kernel with tract's built-in eager-im2col lowering:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- \
  --backend cpu --batch 32 --fixed-batch --direct-fused-conv --fused-layer-norm \
  --iterations 10 --plan-summary --verify
```

Use release mode for this comparison: full LTO materially improves the custom panel-generation hot
loop. The direct path is not the batch-one default; tract's built-in batch-one `LazyIm2col` remains
faster for that compute class.

Prepare and warm every fixed class up front with a resident plan pool:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release --features metal -- \
  --backend metal --plan-pool 1,4,8,16,32,64 --pool-routing exact \
  --batch-sweep --iterations 5
```

Each class is independently bound, transformed with `MetalTransform`, optimized, spawned, and
warmed. Switching classes is then only a lookup of an already resident mutable state; no model
conversion, optimization, or pipeline compilation occurs on the request path. Exact routing avoids
padding by composing classes in descending order: request 10 becomes `8+1+1`, while 16 uses the
single class-16 state. `--pool-routing ceil` retains the padded single-plan comparison.

Before `MetalTransform`, the loader replaces the exact supported NHWC width-five convolution with
five spatial slices and five `PrefixMatMul` nodes. tract then selects its tiled Metal GEMM instead
of the generic direct-convolution kernel, whose threads repeatedly loaded the same 256-by-5 input
window. The release gate fails if this graph lowering no longer matches. Production uses tract's
default Metal GEMM for every class. GGML remains an explicit benchmark choice only: real class-1
feature vectors exposed a correctness failure that the synthetic benchmark vector did not.

The macOS throughput baseline is batch 8 with four Metal owners. The runnable for every resident
class is prepared once and shared; each owner spawns and warms its own private mutable state and
thread-local Metal command stream. The owner and resident-pool modes can be combined directly:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release --features metal -- \
  --backend metal --compute-owners 4 --plan-pool 1,4,8,16,32,64 \
  --pool-routing exact --batch 8 --iterations 100
```

A 60-cell release grid covered batches 1, 4, 8, 16, and 32; owners 1, 2, and 4; and all four GEMM
choices. A sustained three-pass follow-up covered batches 4 and 8 with 2, 4, 6, and 8 owners. The
batch-8/four-owner Metal median was 4,683 files/s, versus 2,880 for the identically configured fused
CPU harness. Batch 4 favored six Metal owners at a 4,813 files/s median, but four remains the
normal-operation setting because it is the batch-8 winner and consumes fewer host resources. These
are M5 Max measurements, not portable constants. The combined four-owner, six-class process used
80.6 MB maximum RSS and a 181.4 MB macOS peak memory footprint in an isolated batch-8 run.

This Metal-only mode does not use the custom CPU convolution and does not send tails to CPU. It
keeps CPU capacity available for extraction and unrelated work. The CPU pool remains available as a
separate fallback benchmark; with `--direct-fused-conv`, it uses built-in `LazyIm2col` for class 1
and direct fused convolution/max reduction for classes 8 and larger.

Measure the compute stage after accumulating representative CLI batches. The harness constructs
each contiguous input before timing and reports batch latency, mean latency per file, and files per
second. `--feature-size` defaults to the current model's 2048 values; the release verification
passes 1024 or 4096 for historical models whose checked configs require it:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- \
  --batch-sweep --iterations 200
```

Benchmark the new bounded accumulate/compute design with dedicated owner threads. tract prepares one
immutable runnable shared by the owners, while every owner constructs and retains its own mutable
state; inference never runs on a Tokio executor thread:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release -- \
  --backend cpu --compute-owners 4 --batch 8 --fixed-batch --iterations 200 \
  --tract-threads 1 --direct-fused-conv --fused-layer-norm
```

`--backend` is honored in owner mode, so CPU and ORT throughput and RSS can be measured in separate
processes. An exhaustive three-pass grid across owners 1-18 and fixed batches 1, 8, 16, 32, and 64
puts the sustained optimum between four and six owners on the measured M5 Max. Normal operation uses
batch 8 with four single-thread owners: that is the measured batch-8 winner and leaves CPU capacity
for extraction and other work. Larger workloads can raise both values explicitly with `--batch` and
`--compute-owners`; six owners remain a measured cross-class throughput option. Treat these as
platform measurements, not mathematical constants, and repeat the grid on other CPU topologies.

Repeat some trials with `--reverse` so backend order does not systematically favor the first or
second runtime through cache, thermal, or sustained-load effects.

On macOS, compile Metal into the comparison explicitly:

```sh
cargo run --manifest-path rust/tract-bench/Cargo.toml --release \
  --features metal -- --batch 128 --fixed-batch --iterations 10
```

Use `--backend metal` for isolated runs and `--metal-gemm mlx|mfa|ggml` to compare tract's Metal
GEMM kernels. Ten iterations at batch 128 represent ten consecutive accumulated compute calls and
1,280 classified files; a single fixed batch 1,280 can be measured separately rather than assumed
to have equivalent throughput.

`tract-metal`, the Metal loader, and Metal CLI execution are all compiled only when
`target_os = "macos"`. Other operating systems build the portable CPU runtime without the Metal
dependency; selecting `--backend metal` there returns an explicit platform error.

Build runtime-specific binaries with `--no-default-features` plus either `ort-runtime` or
`tract-runtime` when comparing deployable executable size. The default features compile both
runtimes so the parity comparison uses identical feature tensors in one process.
