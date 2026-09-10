This directory contains the Rust crates and their tools. It has the following structure:
- The `cli` directory contains the Magika Rust CLI. It is published on crates.io as `magika-cli`. It
  can be compiled with `cargo build --release` from the `cli` directory. The output binary will be
  `../target/release/magika` relative to `cli/` (`rust/target/release/magika` from the repository root).
- The `lib` directory contains the Magika Rust library. It is published on crates.io as `magika`.
- The `tract-runtime` directory contains the shared inference runtime and portable model artifacts. It is
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
  can use a GPU for bulk inference; library GPU requests require a working
  backend. See the CLI help for batch, reader and inference-worker controls.
- Rules are opt-in: build with `yara-rules` and request `--rules=enforce`. This
  also requires the documented Vectorscan library. The default remains ML with
  rules off; signature misses and conflicts fall through to ML.

See [the library changelog](lib/CHANGELOG.md) and
[the CLI changelog](cli/CHANGELOG.md) for the complete API and behavior changes.
Benchmark results distinguish Magika 2 with rules off from rules enforced.

### Rules-only classification

Build with `yara-rules` and run `magika --rules=only --jsonl FILE...`. Rules scan
at most 4 KiB and abstain on misses/conflicts without initializing ML. The library
equivalent is `RuleSet::identify_input`. See the [rules documentation](../rules/README.md)
for native-engine installation and custom packs.
