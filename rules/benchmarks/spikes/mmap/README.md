# Mapped Vectorscan image spike

## Current loader: no outer payload checksum

Source `55c3209ddf7b24447c51f1b2a1302fd7647a205f` removes the outer cryptographic
checksum from both serialized and mapped pack loading/writing. The checksum field,
BLAKE3/Rayon CLI/library variants and their optional dependencies are removed.
Serialized format is now `MAGIKAHS` version 4; mapped format is `MAGIKAMM` version 3.
Earlier packs are rebuilt once. Source/compiler cache identity hashing remains to
invalidate stale compiled rules; build `_sha2-accel-spike` for its accelerated SHA.

Bounds, alignment, trusted-file rules, source/compiler identity, engine/CPU identity
and recognized labels remain checked. Vectorscan checks its bytecode CRC during
serialized deserialization or mapped scratch allocation, before scanning. Metadata
is trusted configuration and no longer has an outer cryptographic corruption check.

The no-checksum pack test failed before implementation and passes now. Three mapped
tests and the serialized cache/edit/corruption test pass. The native corruption
regressions change actual bytecode bytes, not unused allocation padding. Mapped
scratch allocation rejects those bytes; the public identification API abstains on
native failure as documented.

[Generated before/after startup comparison](checksum-removal/comparison.md) retains
raw observations and exact artifacts. The sections below document earlier spike
checkpoints and the hash experiments that led to the current loader.

Source revision: `24b86573616b9fe30a02920e8171064ae08b8dbc`.
Vectorscan source inspected: `acd7363aadea43da9c5246542d9969db843dd132`.
This experiment is on `worktree/rules-mmap-spike`; the PR's default loader is unchanged.

## Implementation

Build with `--features _mmap-spike`. Set `MAGIKA_RULES_MMAP_SPIKE=1` to select
mapped images; unset it or use `0` for the original serialized path. During
`--compile-rules`, the native compiler produces the same database, then
`hs_deserialize_database_at` constructs it once at 64-byte alignment. The persisted
image is placed at a 4 KiB boundary after the bounded manifest. Subsequent loads
mmap the complete file read-only and pass its native image directly to Vectorscan.
The database owns the mapping; workers retain the database. Mapped memory is
unmapped, never passed to `hs_free_database`.

The prototype retains source/compiler identity, source hashing, engine/CPU checks,
manifest parsing, label validation, full payload checksum, trusted-file/cache
checks and per-worker Vectorscan scratch allocation/CRC checks. It removes the
payload read into a heap buffer, native allocation, reconstruction and copying
on a cache hit. A separate cache identity prevents mixing the two representations.
Default shipped images and changed-rule cache images use the same writer/reader.

## Evidence

- [Complete paired replay and 60 timing cells](paired/report.md): identical normalized
  decisions across all 25,421 files, with 9,624 correct decisions, no wrong decisions
  or errors, and 15,797 abstentions in both modes. This first matrix uses the same
  `/usr/bin/env` wrapper for both modes; its absolute timings include that extra exec.
- [Direct-execution diagnostic](direct-exec/report.md): 20 trials per mode/workload,
  one warmup, same binary, rules and saved inputs. Mode is set in the child environment,
  without an extra timed executable. Full raw Hyperfine JSON and summary are retained.
- Three focused native tests passed: relocation at two simultaneous addresses and
  lifetime after dropping the owner/removing the file; header bounds/version rejection;
  manifest/checksum/source identity validation and changed-source cache rebuilding.
- [Combined model/rules package direction](package-format.md).

The source comment in `src/scratch.c:250` explicitly discusses mmap of deserialized
databases. `src/database.h` uses a relative bytecode offset; `db_copy_bytecode` chooses
alignment during reconstruction. The source compiler already performs component-tree
optimization and receives CPU tuning/features. This spike changes database storage,
not rule predicates, compiler optimization passes or generated scan bytecode semantics.

## Reproduction

Use the recorded native library and its SHA-256 from the saved tool artifacts.
Build the spike CLI from its recorded revision:

```sh
cargo build --release --locked --manifest-path rust/cli/Cargo.toml --features _mmap-spike
```

Copy the same YARA source into separate serialized and mapped directories. Create
both distributable packs; compilation refuses to overwrite existing outputs:

```sh
MAGIKA_RULES_MMAP_SPIKE=0 magika --compile-rules serialized/rules.yar
MAGIKA_RULES_MMAP_SPIKE=1 magika --compile-rules mapped/rules.yar
```

Run `magika-compare` using `paired/config.json`, its portable input snapshot and
saved workloads after adjusting local executable/artifact paths. Run `diagnose.py`
against that completed working run to reproduce the direct-execution diagnostic.
The script verifies each timed workload against the complete saved observations.

```sh
python rules/benchmarks/spikes/mmap/diagnose.py WORKING_RUN NEW_OUTPUT HYPERFINE
```

For the focused tests:

```sh
MAGIKA_RULES_MMAP_SPIKE=1 cargo test --locked --manifest-path rust/lib/Cargo.toml \
  --features _mmap-spike mmap_spike -- --ignored
```

All commands requiring Vectorscan need `MAGIKA_VECTORSCAN_LIBRARY` set to the
recorded trusted library. Compilation and benchmarking ran sequentially.

## Limits before production

Validated on this ARM64 macOS host only. The native header bounds checks describe
the inspected Vectorscan ABI (104-byte header, native-endian fields, 64-byte image
alignment). Production needs an explicit ABI/build fingerprint and target matrix,
not an assumption that an equal engine version implies identical internal layout.
Map only trusted, immutable artifacts; atomic replacement must preserve mapped
inodes. No full model mmap implementation or new compilation optimization was
introduced. The unified package design identifies those follow-up constraints.

## Startup attribution and ARM SHA configuration

[Generated stage comparison](attribution.md), [software-SHA traces](startup-traces/report.md)
and [accelerated-SHA traces](startup-traces-accelerated/report.md) retain ten
fresh-process captures for each loader on a one-file signature hit and miss.
`trace_startup.py` verifies every result against the saved corpus observations;
`attribution.py` derives the table directly from the captured JSON.

The measured dominant in-process cost was our full-image SHA-256 checksum.
Inspection of pinned `sha2` 0.10.9 `src/sha256.rs` shows that ARM hardware dispatch
is gated on `feature = "asm"`. Our default dependency configuration did not enable
it. Experimental `_sha2-accel-spike` enables that dependency feature, retaining
runtime CPU detection/software fallback and byte-identical hash semantics.
Source revision `29685e568a13733a299b82ca8fafd8b539b04fcf` includes that feature;
traced software-SHA source is `46e45ac81ba592158adce12f0c8f93596b777fd1`.

The accelerated build loaded the exact same precompiled images without rebuilding.
All three mapped-image validation tests passed again and all 40 captured
classifications agreed with the saved observations. The full 25,421-file parity
run predates the SHA acceleration experiment; the later change only selects the
hash implementation. Timed 1-, 10- and 1,000-file outputs were also checked against
those observations before measurement.

Build accelerated tracing and capture a new output directory:

```sh
cargo build --release --locked --manifest-path rust/cli/Cargo.toml --features _sha2-accel-spike
python rules/benchmarks/spikes/mmap/trace_startup.py WORKING_RUN NEW_TRACE_OUTPUT BINARY PACKS_ROOT
```

`PACKS_ROOT` contains `serialized/rules.yar` and `mapped/rules.yar`, each beside its
precompiled `rules.hsdb`. Software and accelerated trace binaries reuse these same
packs. `MAGIKA_STARTUP_TRACE=1` enables structured stderr traces; normal runs leave
it unset. The trace records nested/concurrent spans separately. Parent launch-to-main
includes Python process launch and scheduling, so it is not isolated dyld time.
The main timer ends before emitting the trace, and the parent tail includes that
emission and process/pipe scheduling. The trace-disabled direct-execution timing
script also accepts `--binary BINARY --packs PACKS_ROOT` for another build; it
records the new artifact hashes and treats full-corpus observations as a reference.

The initial zero-file diagnostic, which exits before rules loading, took about
2.3 ms. It includes CLI parsing/output/shutdown as well as pre-main costs; we do
not subtract it to manufacture an isolated loader measurement. The actual
Vectorscan `dlopen` span was roughly 0.24 ms and thread launch roughly 0.03 ms.
Our wrapper currently requires compiler-only `hs_populate_platform`, which is
absent from the available `libhs_runtime`; splitting compiler/runtime loading is a
separate compatibility change. The local runtime-only dylib also still links
libc++, so its dependency footprint must be measured rather than inferred from
upstream's generic documentation.

## BLAKE3 comparison

[Generated BLAKE3 versus ARM SHA-256 table](hash-comparison/comparison.md) records
fresh measurements for both variants on the same saved inputs. BLAKE3 source is
`fd3f13570876c904a94d245a9ab17d157048508d`; build with `_blake3-spike`. The pinned
`blake3` 1.8.5 build selects NEON automatically on this little-endian ARM64 target.
This variant uses BLAKE3 for both cache identities and manifest/payload checksums,
with separate serialized/mapped pack versions and explicit digest prefixes.
SHA-256 benchmark provenance hashes retain their existing encoding.

Both mapped packs contain the same 1,633,536 native payload bytes, verified by
identical payload digests in [receipt.json](hash-comparison/receipt.json). Two new
tests failed before implementation and passed after it; all three native mapping,
corruption and source-change tests passed. All 80 traced classifications and each
1-, 10- and 1,000-file timed workload matched the saved observations. This focused
experiment does not claim a new full-corpus replay or non-Mac performance results.

BLAKE3's measured checksum span was 0.7593 ms versus accelerated SHA-256's 0.5460 ms.
The corresponding one-file whole-process medians were 5.507 and 5.526 ms, with
overlapping ranges. These observations do not establish an end-to-end startup win
for BLAKE3 on this host. The default hash selection remains unchanged in the spike.

Reproduce using `trace_startup.py` and `diagnose.py` with each receipt's binary and
separately compiled pack directories. Place observations under
`hash-comparison/{sha256,blake3}/{startup,direct}` in a fresh result tree, then run
`hash_comparison.py` to derive JSON and Markdown. Existing observations are immutable.
The scripts, commands, binary/pack hashes, raw timings, traces and TDD logs are retained.

## Cargo options and Rayon follow-up

The [compiler-option audit](hash-options/README.md) retains actual Cargo backend
selection, verbose optimized build logs and matched generic/native CPU measurements.
The initial BLAKE3 variant already used opt-level=3, LTO, one codegen unit and ARM
NEON. Neither Rust-only `target-cpu=native` nor additional C native tuning improved
sequential BLAKE3 warm hash time materially on this host. Rayon was missing from
that first variant and does reduce the checksum span here.

[Actual four-thread Rayon CLI comparison](hash-comparison-rayon/comparison.md)
uses source `90fffe8f2cdabcb04b05c3e7df4cf5b3f5111600` with feature
`_blake3-rayon-spike` and `RAYON_NUM_THREADS=4`. The CLI uses the same generic release
profile as the SHA build, isolating Rayon from CPU tuning. Both mapping modes and
all checks remain active; native images are byte-identical. Both hash tests and
all three native tests pass; all 80 traced classifications and timed workloads
match the saved observations. JSON explicitly records the thread setting.

The measured one-file checksum medians were 0.4737 ms for BLAKE3/Rayon and 0.5516 ms
for accelerated SHA-256. In-process medians were effectively equal (2.1193 and
2.1191 ms). Whole-process one-file medians were 5.228 and 5.449 ms, with overlapping
ranges. This is a modest observed process difference, not a portable speed claim.
The full tables retain 10- and 1,000-file timings and all raw ranges.

To reproduce, add `--rayon-threads 4` to both measurement scripts. Render their
saved JSON with `hash_comparison.py --root NEW_RESULTS --blake3-label
'BLAKE3 + Rayon (4 threads)'`. Each result directory retains its exact script copy.
The experimental feature leaves thread policy to Rayon/environment; production
thread budgeting still needs a deliberate policy before enabling it by default.

## Complete process accounting

[Same-run parent/main/exit accounting](process-accounting/results/report.md) fixes
the earlier incomplete attribution: internal spans alone excluded most startup.
The native Hyperfine timer now records boundaries for the very same child traces.
Before-main averages 3.09 ms, main 1.51 ms and trace/exit/wakeup 0.65 ms, totaling
5.24 ms with tracing. The untraced median is 4.55 ms. The major remaining bucket is
before-main startup; its framework/loader/scheduler split is not yet isolated.

## Rust debugger inspection

[Rust LLDB startup stacks and dependency findings](rust-debug/README.md) show 347
images present before the first Rust main statement, including ML/GPU frameworks.
The debugger observes Objective-C initialization called from dyld before Rust main.
Cargo confirms unconditional `magika-tract-runtime -> tract-metal` linkage on macOS.
This identifies an eager-loading target; the marginal timing saving needs a build
that removes that dependency chain. Debugger pauses are not used as benchmark data.
