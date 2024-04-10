This crate is for maintenance purposes only. It is used to update the Rust library to a new model.
It relies on 2 symbolic links to be coherent:
- `rust/lib/src/model.onnx` should point to the model.
- `rust/gen/data/config.json` should point to the definition of the target labels for that model.

Ideally if those 2 symbolic links are incoherent some test should fail, such that a PR updating the
model without updating the config would not pass the continuous integration.

There is another test to make sure the library code is in sync with the config. If this test fails,
one simply needs to run `./sync.sh` from the `rust` directory to update the library.
