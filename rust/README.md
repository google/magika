This directory contains the Rust crates and their tools. It has the following structure:
- The `cli` directory contains the Magika Rust CLI. It is published on crates.io as `magika-cli`. It
  can be compiled with `cargo build --release` from the `cli` directory. The output binary will be
  `target/release/magika`.
- The `lib` directory contains the Magika Rust library. It is published on crates.io as `magika`.
- The `tract-runtime` directory contains the shared inference runtime and checked NNEF model. It is
  published first as `magika-tract-runtime`; regenerate or verify its model with
  `tract-bench/scripts/convert-model.sh`.
- The `gen` directory is for maintainers when a new model is available.
- The `test.sh` script tests the crates listed above. It runs as part of the Github continuous
  integration.
- The `sync.sh` script updates the library when a new model is available using the `gen` crate.
- The `publish.sh` script prepares the runtime, library, and CLI for publishing to crates.io in
  dependency order. It generates a commit that must be merged first.
- The `color.sh` is a shell library for the scripts above.
- The remaining files have the usual meaning associated to their name.
