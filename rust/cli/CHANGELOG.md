# Changelog

## 0.1.0-rc.3-dev

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
