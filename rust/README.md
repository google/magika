This directory contains the Rust crates and their tools. It has the following structure:
- The `cli` directory contains the Magika Rust CLI. It is published on crates.io as `magika-cli`. It
  can be compiled with `cargo build --release` from the `cli` directory. The output binary will be
  `target/release/magika`.
- The `lib` directory contains the Magika Rust library. It is published on crates.io as `magika`.
- The `gen` directory is for maintainers when a new model is available.
- The `test.sh` script tests the crates listed above. It runs as part of the Github continuous
  integration.
- The `sync.sh` script updates the library when a new model is available using the `gen` crate.
- The `publish.sh` script prepares the library and the CLI for publishing to crates.io. It generates
  a commit that must be merged first.
- The `color.sh` is a shell library for the scripts above.
- The remaining files have the usual meaning associated to their name.
