This crate is for maintenance purposes only. It is used to update the Rust library to a new model.
There are 3 files in the Rust library that depend on the model:

- The model itself, `rust/lib/src/model.onnx`, which is a symbolic link to some model under
  `assets/models`, controlled by the `rust/gen/model` symbolic link. Publishing the crate will
  dereference this symbolic link.
- The labels describing the model output, `rust/lib/src/model.rs`, which is generated from the model
  configuration, `rust/gen/model/config.min.json`.
- The list of possible file types, `rust/lib/src/content.rs`, which is generated from the knowledge
  base of content types, `assets/content_types_kb.min.json`.

The purpose of this crate is to generate the last two files. There is a test to make sure that they
are up-to-date. If the test fails, one simply needs to run `./sync.sh` from the `rust` directory to
regenerate them.

An alternative design to generating the files before publishing the crate, would be to publish the
model and Magika configurations and use a build script to generate the files during compilation.
This has a few disadvantages:

- We need to publish the model and Magika configurations which contain more information than needed
  to use the library (and the CLI).
- We need to use a build script, which is frown upon for security reasons, as the entity compiling
  the library or CLI now needs to trust the build script, which can run arbitrary code. This only
  matters when the entity compiling the library or CLI is not the same as the one running the
  library or CLI (e.g. Debian maintainers), since the library and CLI too can run arbitrary code.
- Using a build script also increases compilation time (and compilation complexity) instead of
  having it factored before publishing.
