# Changelog

## 1.0.2-dev

### Patch

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
