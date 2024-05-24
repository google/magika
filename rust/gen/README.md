This crate is for maintenance purposes only. It is used to update the Rust library to a new model.
There are 2 files in the Rust library that depend on the model:

- The model itself, `rust/lib/src/model.onnx`, which can either be a symbolic link to the source of
  truth in this repository (publishing the crate will dereference the symbolic link), or a copy of
  that source of truth (in which case there should be a test to make sure the file is an up-to-date
  copy).
- The labels describing the model output, `rust/lib/src/label.rs`, which is generated from the model
  configuration, `rust/gen/data/config.json`, which can either be a symbolic link or a copy to the
  source of truth in this repository, and should match the model above.

The purpose of this crate is to generate this second file. There is a test to make sure that this
file is up-to-date. If the test fails, one simply needs to run `./sync.sh` from the `rust` directory
to regenerate the file.

An alternative design to generating the file before publishing the crate, would be to publish the
model configuration and use a build script to generate the file during compilation. This has a few
disadvantages:

- We need to publish the model configuration which contains more information than needed to use the
  library.
- We need to use a build script, which is frown upon for security reasons, as the entity compiling
  the library or CLI now needs to trust the build script, which can run arbitrary code. This only
  matters when the entity compiling the library or CLI is not the same as the one running the
  library or CLI (e.g. Debian maintainers), since the library and CLI too can run arbitrary code.
- Using a build script also increases compilation time (and compilation complexity) instead of
  having it factored before publishing.
