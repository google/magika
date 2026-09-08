# Deferred inference backend spike

The distribution ships one small CLI and two native backend libraries. CPU loads
`libmagika_runtime_cpu.dylib`; GPU loads `libmagika_runtime_metal.dylib` on macOS.
Rules-only loads neither. The existing `magika-load` thread performs `dlopen` and
model preparation while the main pipeline loads rules, reads files and matches
signatures. Inference waits for preparation only when an ML input arrives.

The CPU library compiles the original tract runtime without its Metal feature.
The Metal library retains the existing graph transforms, batch plans, memory
arena and GPU agreement probe. Both use the same model bytes and pinned dependency
versions as the eager baseline. Session creation and inference remain on the same
worker thread because tract execution state is not `Send`.

A versioned C function table keeps Rust allocations and destructors within their
own library. The facade validates ABI version, table size, backend kind and tensor
dimensions. Sessions retain their runtime, and successfully loaded libraries stay
resident for the process lifetime. CPU and GPU loading have independent locks.
Explicit GPU failure remains an error; automatic selection retains CPU fallback.

`build.py ROOT TARGET DESTINATION RESTORE_BINARY` builds both backends and the CLI
with release LTO and locked dependencies, then restores the shared PR executable.
Keep `DESTINATION/magika` and `DESTINATION/lib/` together when moving the package.
`MAGIKA_RUNTIME_DIR` explicitly selects a trusted native-code directory; ordinary
loading uses the executable's adjacent `lib/`, never the working directory.

`check_linkage.py` inspects linkage; `check_loaded.py` records actual dyld images,
executes CPU/GPU selection, and checks rejection/fallback with an incompatible
backend library. `rust/runtime/tests/backend.rs` exercises real CPU/Metal scores,
padded/split batches, state reuse, invalid shapes and session lifetime. The two
existing CLI preparation tests cover rule completion during blocked preparation
and delivery of preparation errors to ML work.

`measure.py` reuses saved benchmark workloads and runs both complete products.
It first compares their classification JSON, then saves direct-execution
Hyperfine JSON. Its `--render RESULTS` option regenerates Markdown solely from
those saved measurements. Build and diagnostic executions are outside timing.

This is an isolated spike, not an updated release package. The shared-library
layout needs release/installer integration before adoption. macOS CPU and Metal
are exercised here; the CUDA feature forwarding is not qualified by this run.

[Paired timing table](results/report.md) and [raw measurement summary](results/summary.json)
retain both improvements and regressions. The exact executed runner is archived
as `results/measure-recorded.py`; the current renderer additionally derives
workload coverage from the saved JSON outputs.

The dyld logs include shared-cache image records subsequently marked delayed.
`check_loaded.py` follows those transitions: Metal is inactive in CPU/rules-only
runs and active in GPU runs. A raw image count alone does not measure initialized
frameworks or their startup time.
