# Changelog

## 1.2.0-dev

### Minor

- Replace the ONNX Runtime backend with an embedded tract runtime, so the binary no longer needs
  an external inference runtime
- Add `--backend` to select automatic, CPU-only or GPU-required inference
- Add `--verbose` to report the selected inference device and implementation
- Add `--readers` to size the pool of threads reading files and extracting features
- Scale `--threads` with the available parallelism on a CPU instead of always using four, which
  left most of a multi-core host idle. Use every logical CPU on x86_64 Linux and leave one free on
  other CPU targets, and default `--readers` to one per inference thread
- Add `--trace-utilization` to report busy and waiting time per pipeline stage
- Rename `--num-tasks` to `--threads` and document it, keeping `--num-tasks` as an alias
- Document `--batch-size` and raise its default from one to eight
- Remove `--intra-threads`, `--inter-threads`, `--optimization-level` and `--parallel-execution`,
  which configured the ONNX Runtime

### Patch

- Accumulate features from every reader into one global inference batch and bound ordered results
- Remove deprecated `package.authors` field in `Cargo.toml`
- Update dependencies

## 1.1.0

### Minor

- Exit successfully when the standard output is closed by the user

### Patch

- Join all threads before shutdown to avoid segmentation faults in ONNX Runtime
- Update dependencies
- Fix new clippy lints

## 1.0.2

### Patch

- Update dependencies
- Enable full LTO for the release profile

## 1.0.1

### Patch

- Update dependencies

## 1.0.0

### Patch

- Change description
- Update dependencies

## 0.1.4

### Minor

- Use true colors when available

### Patch

- Dissociate repository from published content (see `publish.sh` script)
- Remove `package.metadata.deb`

## 0.1.3

### Minor

- Change performance tuning configuration

### Patch

- Add `package.metadata.deb` for `cargo-deb` customization
- Update dependencies

## 0.1.2

### Minor

- Use the `standard_v3_3` model instead of `standard_v3_2` (see [model changelog])
- Do not print the low-confidence warning if the content type was simply overwritten

### Patch

- Update dependencies

## 0.1.1

### Minor

- Use the `standard_v3_2` model instead of `standard_v3_1` (see [model changelog])

## 0.1.0

No changes.

## 0.1.0-rc.5

### Minor

- Use the `standard_v3_1` model instead of `standard_v3_0` (see [model changelog])

## 0.1.0-rc.4

### Minor

- Update the model thresholds

## 0.1.0-rc.3

### Minor

- Use the `standard_v3_0` model instead of `standard_v2_1` (see [model changelog])

### Patch

- Update dependencies

## 0.1.0-rc.2

### Minor

- Exit with non-zero code if at least one error was encountered (fixes #780)

### Patch

- Update dependencies

## 0.1.0-rc.1

### Minor

- Print model version with `--version`
- Change model from `standard_v2_0` to `standard_v2_1`

### Patch

- Fix running on MacOS by defaulting `--intra-threads` to 4
- Fix the `--version` binary name from `magika-cli` to `magika`
- Make sure ONNX Runtime telemetry is disabled
- Change the default of the hidden flag `--num-tasks` from 1 to the number of CPUs

## 0.1.0-rc.0

This version is the initial implementation and should be considered unstable. In particular, it
ships a new model in comparison to the Python binary and we would love feedback on
[GitHub](https://github.com/google/magika/issues).

## 0.0.0

This version is a placeholder and does not expose anything.

[model changelog]: https://github.com/google/magika/blob/main/assets/models/CHANGELOG.md
