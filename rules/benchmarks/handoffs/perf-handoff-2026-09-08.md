# Magika speed handoff (2026-09-08)

Measured on the deferred-backend spike build, branch `worktree/rules-mmap-spike` at `cb0d5970`,
binary `tmp/comparison-tools/magika2-deferred-spike-v3/magika`, idle M5 Max, hyperfine `-N`.
Workload lists come from `tmp/comparison-mmap-spike-24b86573/workloads.json`.
Check `pgrep rustc` is empty before timing anything; a background build inflated early runs 30-50%.

## Baseline

| Whole process | ms |
|---|---:|
| `/usr/bin/true` | 1.0 |
| `file -b`, 1 file | 1.8 |
| `magika --version` | 1.8 |
| rules-only, 1 file | 3.4 |
| CPU ML, 1 file | 7.4 |
| CPU ML, 2 files | 13.3 |
| `file`, 1000 files | 236 |
| rules-only, 1000 files | 70 |
| enforce + ML, 1000 files | 174 |
| ML only, 1000 files | 270 |

Rules-only already beats libmagic. The work below is on the ML path and the 2-10 file case.

## Items, in priority order

### 1. Cap the batch class at the file count (CLI)
`rust/cli/src/main.rs:226` `inference_configuration` only special-cases exactly one path. With 2+
paths the CPU runtime keeps only the batch-8 plan, so 2 files run an 8-row inference:
7.4 ms per batch-8 run vs 1.3 ms per batch-1 run (runtime bench). Cap `max_batch` at
`path.len()` when not recursive; consider adding a class 2 or 4 to `BATCH_CLASSES`.
Accept: 2-file CPU ML run under 9 ms (today 13.3; `--batch-size=1` already gives 8.2). Output identical.

### 2. Hardware CRC32C in our Vectorscan build (ARM)
`hs_alloc_scratch` calls `dbIsValid`, which CRC32Cs the full 1.6 MB bytecode every process.
Upstream `src/crc32.c` (5.4.13, latest) only has an SSE4.2 path; ARM uses software slice-by-8.
Cost: 0.58 ms of the 1.45 ms rules-only `main`. Add an `#elif` using `__crc32cd/__crc32cb` from
`arm_acle.h` under `__ARM_FEATURE_CRC32`, plus the CMake flag; carry it in our libhs build, upstream after.
Accept: `native_scratch_allocate` span under 0.1 ms via `MAGIKA_STARTUP_TRACE=1`. x86 unaffected.

### 3. Fused conv for batch 1
`rust/tract-runtime/src/lib.rs:63` `DIRECT_FUSED_MIN_BATCH = 8`. Batch-1 rows use the unfused
conv: 1.3 ms vs 0.92 ms fused. Extend `direct_conv` to small batches.
Accept: bench `--batch 1 --threads 1` above 1000 files/s (today 780); probe scores bit-identical.

### 4. Exit right after output
Single-file ML spends ~1 ms more outside `main` than `--version`; rules-only only ~0.2 ms.
Presumed runtime/state destructors. Flush stdout, then `process::exit`. Verify the cause first
with the startup trace; if teardown is not the cause, drop this item.
Accept: single-file ML whole-process under 6.5 ms.

### 5. Pre-packed model artifact
Preparation is 2.0-2.3 ms per process: parse 76 KB `model.graph.json`, build tensors, pack the
conv kernel (`rust/tract-runtime/src/artifact.rs`). Ship a binary graph with pre-packed weights.
Accept: bench `shared_prepare_us` under 800.

### 6. macOS thread default (optional)
17 threads: 270 ms wall, 3.8 s CPU on 1000 files. 4 threads: 355 ms, 1.4 s CPU. AMX is shared
per cluster. Decide whether the default should trade 30% wall for 60% less CPU.

## Not fixable in code
The model is ~0.67 GFLOP per file and the fused conv already runs near AMX peak. Beating libmagic
by a wide margin on ML-only needs fp16/int8 or a lighter conv (model decision). Rule coverage is
the cheaper lever: each rule hit saves ~0.9 ms CPU.

## Repro
```
H=tmp/comparison-tools/hyperfine/hyperfine-v1.20.0-aarch64-apple-darwin/hyperfine
M=tmp/comparison-tools/magika2-deferred-spike-v3/magika
export MAGIKA_VECTORSCAN_LIBRARY=tmp/rules-research/vectorscan-build/lib/libhs.5.4.13.dylib MAGIKA_RULES_MMAP_SPIKE=1
$H -N -w 3 -r 25 "$M --jsonl --backend=cpu --rules=off -- FILE1" "$M --jsonl --backend=cpu --rules=off -- FILE1 FILE2"
cargo build --release --manifest-path rust/tract-bench/Cargo.toml --no-default-features
rust/target/release/magika-runtime-bench --backend cpu --batch 1 --threads 1 --iterations 300
```
