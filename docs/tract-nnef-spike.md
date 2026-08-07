# tract/NNEF feasibility spike

Status: active exploration on branch `tract`.

## Decision to make

Determine whether the Rust Magika implementation should replace ONNX Runtime (`ort`) with
[`tract`](https://github.com/sonos/tract), ship the model as NNEF/tract-OPL, and optionally use
Metal on Apple platforms. The first phase is deliberately a spike: prove prediction parity,
measure latency and footprint, and stop before production integration unless the evidence is
favorable.

The portable baseline is tract CPU. Metal is an optional, separately measured backend rather
than a requirement: Magika's network and small inputs may not contain enough work to amortize GPU
setup, synchronization, and transfer costs.

## Current Magika map

The relevant production path is the Rust library and CLI:

1. `rust/lib/src/input.rs` extracts 2,048 integer tokens: 1,024 bytes from the beginning and 1,024
   from the end, padded with token 256. Empty and undersized inputs can be classified by rules
   without invoking a model.
2. `rust/lib/src/session.rs` stacks features into an `ndarray::Array2<i32>` with shape
   `[batch, 2048]`, sends the tensor under the ONNX input name `bytes`, removes the output named
   `target_label`, and converts the scores into public `FileType` results.
3. `rust/lib/src/builder.rs` owns ONNX Runtime configuration and constructs a session from the
   model embedded by `include_bytes!("model.onnx")`.
4. `rust/lib/src/future.rs` is the other direct runtime boundary. It adapts both synchronous and
   asynchronous inference to `ort`, including `run_async` and its `RunOptions`.
5. `rust/lib/src/error.rs` exposes `ort::Error` through Magika's error type.
6. `rust/lib/Cargo.toml` declares `ort` with `ndarray` and `std`; its `_test` feature downloads ONNX
   Runtime binaries. The final binary is otherwise responsible for linking ONNX Runtime.
7. `rust/cli/src/main.rs` initializes ONNX Runtime, maps experimental thread/optimization flags to
   the library builder, and performs batched asynchronous inference.
8. `rust/cli/Cargo.toml` depends on a default-featured `ort`, so the distributed CLI carries the
   deployable ONNX Runtime choice in addition to the Magika library.

The current model is `assets/models/standard_v3_3/model.onnx` (3,163,737 bytes). The Rust model
symlink chain ends at that asset, so the model bytes are already embedded in release binaries.
Feature extraction and score-to-label policy do not need to change for a runtime experiment.
The ONNX graph declares an `i32` input shaped `[dynamic batch, 2048]` and produces 214 `f32`
probabilities per input.

### Natural replacement seam

The narrowest seam is the session's batch inference operation:

```text
files/content -> Features ([i32; 2048]) -> batched tensor -> runtime -> score matrix -> FileType
```

A polished change should hide the runtime behind a small internal backend interface while keeping
the public `Session` methods stable. The spike should not introduce that abstraction prematurely;
its standalone harness can feed the exact same feature tensors to both engines first.

## tract map

Upstream inspected at commit `876a0296aaa9a103a577674d1f7fd7b180f4debe` (2026-08-06); the latest
published crate observed during the spike is `tract 0.23.4`.

- tract loads ONNX into an inference model, resolves it into a typed model, then lets a named
  runtime prepare the optimized runnable form.
- `tract::runtime_for_name("default")` selects the CPU runtime. On Apple targets,
  `tract::runtime_for_name("metal")` selects the Metal runtime when the Metal crate is linked.
- tract reads and writes NNEF. Its recommended small-runtime deployment is a one-time ONNX-to-NNEF
  translation followed by runtime loading with only the NNEF/core pieces required by the graph.
- tract-OPL extends standard NNEF when a typed tract operator has no portable NNEF equivalent.
  Serialized OPL compatibility is promised within a `0.x` line only from older patch versions to
  newer patch versions, so conversion must be pinned and reproducible.
- The high-level `tract` crate is the supported public API, but it currently pulls ONNX, NNEF,
  extra/transformer facilities and Apple Metal on Apple targets. That is convenient for the spike
  but may not deliver the smallest binary. A successful follow-up should compare the supported
  facade with the minimal `tract-core` + `tract-nnef` deployment described upstream before choosing
  dependencies for production.
- Runtime preparation is backend-specific. The serialized model remains platform-independent;
  CPU or Metal optimization happens after loading it.

The expected conversion is performed once, not at application startup:

```sh
tract assets/models/standard_v3_3/model.onnx dump --nnef model.nnef.tgz
```

The original graph does not resolve in tract 0.23.4 with a symbolic batch: its one-hot broadcast is
reported as variable-rank, several exporter-generated shape subgraphs cast the symbol through
`i32`, and two full-range slices use one billion as an end sentinel that tract propagates as a
literal dimension. The checked converter fixes these before typed-graph resolution by replacing
one-hot-plus-matmul with a gather, supplying equivalent symbolic `TDim` shapes, and removing the
identity slices. The resulting NNEF input is `[N, 2048]`; the same prepared tract state passes parity
for changing batches `1, 2, 3, 8, 16`. CLI output is not a stable scripting API, so the durable
conversion step pins tract and validates the generated artifact instead of parsing human-readable
dumps.

## Benchmark design

Use one executable and one fixed corpus so both backends receive identical `[batch, 2048]` `i32`
tensors. Keep feature extraction outside inference timings, but report an end-to-end CLI sample
separately later if the runtime comparison is favorable.

Measure:

- serialized ONNX and NNEF model bytes;
- clean release binary bytes and any required dynamic libraries/frameworks;
- cold load plus backend preparation time;
- first inference after preparation;
- warm single-item latency;
- warm batch latency for representative CLI batch sizes;
- throughput and peak resident memory when a reproducible measurement is available;
- prediction parity: identical top label and tightly bounded score differences on representative
  repository fixtures.

Report CPU and Metal as different rows. Alternate or randomize repeated backend runs when chasing
small differences, because sustained Apple Silicon benchmarks can be thermally biased. Do not mix
model preparation into warm inference statistics.

### Initial decision thresholds

Proceed to production-quality integration only if all of these hold:

- representative predictions have identical winning labels, with numerical differences explained
  and bounded;
- tract CPU warm inference is no more than 20% slower than ONNX Runtime, or a larger slowdown buys
  a compelling distribution-size reduction;
- the deployable CLI footprint is materially smaller (target: at least 30%);
- cold preparation remains acceptable for a command-line tool (target: no more than 50 ms slower);
- NNEF conversion is deterministic or its nondeterminism is understood and validation makes it
  safe to update;
- no unsupported graph operator requires keeping the ONNX parser/runtime in the shipped product.

Metal earns a production feature only if it improves a realistic Magika batch by at least 15%
after preparation and does not meaningfully regress startup or binary size. A GPU win on an
artificially large batch is not sufficient.

## Known risks and questions to resolve with evidence

- Whether the current graph converts without ONNX-specific tract-OPL extensions.
- Why tract CPU does not amortize work over this model's batch dimension as effectively as ORT.
- Whether tract's latest public facade can be trimmed enough without relying on unstable internal
  crates.
- How many dedicated tract compute owners maximize throughput without oversubscribing tract's own
  matrix-multiplication threads.
- Whether Metal supports the graph's important operators, and whether such a small model benefits
  after CPU/GPU synchronization.
- Cross-platform build and behavior on Linux, Windows, macOS, and the Rust library's supported
  architectures.

## Deferred if the spike is favorable

- internal backend abstraction and removal of `ort` from public implementation details;
- final builder API, thread controls, and backwards-compatibility policy;
- full fixture parity suite and score-tolerance policy;
- cross-platform CI and release packaging;
- minimal dependency audit, license/notice updates, and supply-chain review;
- model-update tooling, deterministic generation checks, and serialization compatibility checks;
- documentation, changelog, migration notes, and polished Metal configuration/fallback behavior.

## Initial result

The first Apple M5 Max result is a no-go for immediate integration. Numerical parity is excellent,
and the minimal tract CPU executable is 19.15% smaller, but its median warm inference is about 20%
slower and its mean is 23.28% slower than ONNX Runtime. Its original 7.76% model-size claim compared
raw ONNX with gzip NNEF and is superseded by the codec-equivalent measurements below.

Metal improves tract CPU warm mean by only 5.14% and p50 by 6.14% at batch 1 while adding 12.75%
to the tract CPU executable and making preparation roughly 3.7 times as expensive. It should not
be enabled by default. The dynamic model now permits the larger-batch measurement needed to decide
whether any opt-in Metal path is justified.

See `rust/tract-bench/results/2026-08-07-m5-max.md` for the complete environment, methodology,
measurements, and next recommendation.

## CPU optimization follow-up

The bounded CPU follow-up is favorable enough to continue the experiment. Reusing one tract state,
using a two-thread matrix-multiplication executor, and converting one-hot-plus-matmul into an
equivalent embedding gather moves tract from about 20% slower at warm p50 to 3.63% faster in ten
alternating-order 1,000-iteration trials. Warm mean is effectively tied (-0.74%), p95 is 3.32%
slower, and score parity remains `2.4883775e-9` with identical winning labels.

Rayon increases the optimized tract CPU-only executable to 16,900,704 bytes, still 14.54% smaller
than the 19,776,896-byte ORT executable. Equal-codec comparison shows that format size is a wash:
raw NNEF is 0.0619% larger, gzip-9 NNEF is 0.0738% smaller, and zstd-19 NNEF is 0.0698% smaller.

## Dynamic batching follow-up

Symbolic conversion is now solved and deterministic. One prepared state accepts changing and final
partial batches, with identical winning labels and maximum absolute score difference
`2.3841858e-7`. The compute-stage harness accumulates each input before timing and reports both
batch latency and per-file throughput.

The CPU evidence is unfavorable for production integration: across three alternating-order sweeps,
four-thread tract delivers roughly 450-470 files/s while ORT rises from 587 files/s at batch one to
737 files/s at batch 32. tract is 29-38% behind for batches 2-32. Eight threads improve the best
batch-32 tract sample only to 465 files/s, while 16 and 18 threads regress. The competitive tuned
batch-one result therefore does not predict batched CLI performance.

### Tokio accumulate/compute ownership

tract should not reuse the existing ORT execution architecture. In particular, it should not map
the CLI's `num_tasks` default onto replicated tract sessions and should not wrap synchronous tract
work in the existing async inference call. Build a separate accumulate/compute path:

1. An accumulator owns pending features, emits full batches, and explicitly flushes the final partial
   batch.
2. A bounded queue between accumulation and compute supplies backpressure.
3. Dedicated owner threads construct and retain their mutable, non-`Send` tract states.
4. Owners run synchronous inference outside Tokio and return ordered result messages to the async
   I/O/output boundary.
5. Owner count and tract-internal threads form one measured CPU budget (`1 x 8`, `2 x 4`, `4 x 2`,
   and so on), rather than copying the ORT session-per-task policy.

The benchmark harness implements this owner/queue model without editing production code. It exists
to determine the topology first; production integration remains gated on those measurements.

The first topology sweep validates the design. At batch 8, moving from `1 x 8` to `4 x 2` and
`8 x 1` raises tract from roughly 453 files/s to 1,391 and 2,110 files/s in the initial run.
Alternating-order medians put tract about 5.2% behind ORT at `4 x 2` and 7.2% behind at `8 x 1`, a
much smaller gap than the single-state batch sweep. Twelve and sixteen owners regress.

The runnable/state distinction matters: tract's prepared runnable is `Send + Sync`, so it is
prepared once and shared, while each owner spawns a private non-`Send` state. Sharing reduces the
eight-owner tract peak from about 449 MB to 406 MB. One owner uses about 75 MB; an equivalent
eight-owner ORT-only process reaches about 541 MB. Topology selection therefore needs both throughput
and a memory budget rather than a CPU-count default.

Dynamic Metal improves over ORT only once batches become fairly large: roughly 4% at batch 8, 9% at
batch 32, 14% at batch 64, and 17% at batch 128 in a directional sweep. Only batch 128 crosses the
15% threshold, while preparation costs around 71-84 ms and first use of a new shape can be expensive.
That does not justify Metal for the normal CLI path.
