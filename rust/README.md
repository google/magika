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
