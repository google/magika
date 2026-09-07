# Changelog

## 1.2.0-dev

### Minor

- Remove the dependency on the ONNX Runtime

- Add `--rules=off|enforce`, `--rules-file`, `--write-default-rules`, and `--compile-rules`; optional Vectorscan execution supports full and partial enforced rules, disabled candidates, compiled packs and a persistent compilation cache. Rules remain off by default; requesting enforcement reports initialization failures.

### Patch

- Preserve completed JSON results and close arrays on classification errors; report invalid UTF-8 paths without panicking and preserve earlier failures when output closes.
- Reject inference batches with mismatched result counts before dispatching any rows.
- Reject named pipes and other unsupported special files before reader dispatch; JSON reports `unsupported_file_type` and regular files continue processing.
- Bound experimental batch sizes to 1–64 and reader/inference worker counts to 1–256 before allocating queues or starting threads.
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
