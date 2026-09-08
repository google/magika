This directory contains the Rust crates and their tools. It has the following structure:
- The `cli` directory contains the Magika Rust CLI. It is published on crates.io as `magika-cli`. It
  can be compiled with `cargo build --release` from the `cli` directory. The output binary will be
  `../target/release/magika` relative to `cli/` (`rust/target/release/magika` from the repository root).
- The `lib` directory contains the Magika Rust library. It is published on crates.io as `magika`.
- The `tract-runtime` directory contains the shared inference runtime and checked NNEF model. It is
  published first as `magika-tract-runtime`; regenerate or verify its model with
  `tract-bench/scripts/convert-model.sh`.
- The `tract-bench` directory contains the runtime benchmark and model conversion/verification tools.
- The `gen` directory is for maintainers when a new model is available.
- The `test.sh` script tests the crates listed above. It runs as part of the Github continuous
  integration.
- The `sync.sh` script updates the library when a new model is available using the `gen` crate.
- The `publish.sh` script prepares the runtime, library, and CLI for publishing to crates.io in
  dependency order. It generates a commit that must be merged first.
- The `color.sh` is a shell library for the scripts above.
- The remaining files have the usual meaning associated to their name.

Optional signature rules require the `yara-rules` Cargo feature and a native engine.
See the [rules build and platform installation instructions](../rules/README.md#build-and-use)
for `MAGIKA_VECTORSCAN_LIBRARY` and the optional `MAGIKA_RULES_CACHE` override.

The crates' Apache-2.0 license describes Magika's own sources; dependencies retain
their individual licenses. The locked tract dependency graph includes MPL-2.0
`dyn-eq` 0.1.3 through tract's core/data crates and, with CUDA enabled, MPL-2.0
`option-ext` 0.2.0 through `tract-cuda` → `dirs` → `dirs-sys`. Their source packages
are used without local patches. Inspect the selected build's graph with
`cargo tree --locked --manifest-path cli/Cargo.toml -i dyn-eq` and the equivalent
command for `option-ext`; feature and target selection can change that graph.

## Migrating to Magika 2.0

The Rust CLI and library are both `2.0.0-dev`. The embedded tract runtime replaces
ONNX Runtime; applications must migrate the public Rust API before upgrading.
Python and JavaScript keep their independent versions and existing runtimes.

- Build a shared `Runtime` with `Runtime::builder()`; create one `Session` per
  inference thread. `Builder` now builds a runtime rather than a session.
- Replace `SyncInput` with `Input`, remove `_sync` suffixes, and replace removed
  async session calls with synchronous calls on each worker's session.
- Handle `anyhow::Result` rather than the removed Magika `Error`/`Result` types.
- Select `Backend::Cpu` for a reproducible CPU comparison. Automatic selection
  can use a GPU for bulk inference; explicit GPU requests require a working
  backend. See the CLI help for batch, reader and inference-worker controls.
- Rules are opt-in: build with `yara-rules` and request `--rules=enforce`. This
  also requires the documented Vectorscan library. The default remains ML with
  rules off; signature misses and conflicts fall through to ML.

See [the library changelog](lib/CHANGELOG.md) and
[the CLI changelog](cli/CHANGELOG.md) for the complete API and behavior changes.
Benchmark results distinguish Magika 2 with rules off from rules enforced.

### Rules-only classification

Build with `yara-rules`, then run `magika --rules=only --jsonl FILE...` (optionally
`--rules-file PACK.yar`). This scans the selected pack against at most the first
4,096 bytes with the original file size available to rule guards. Misses and
conflicts produce `unknown`, including empty and short text inputs without a
matching signature. Input errors still report errors. No ML features are extracted
and no inference model or GPU backend is initialized; `--backend-info` reports
`none (rules-only)`. Filesystem directory/symlink reporting remains available.
The library equivalent is `RuleSet::identify_input`, returning `Ok(None)` for an
abstention. The rules matcher executes on the CPU.
