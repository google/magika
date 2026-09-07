# Changelog

## 2.0.0-dev

### Major

- Remove the dependency on the ONNX Runtime
- Remove async support for `Session`
- Remove the `_async` methods of `Session`
- Remove the `_sync` suffix from the `Session` methods
- Rename `SyncInput` to `Input`
- Remove `Error` and `Result` using the `anyhow` crate instead
- Change all methods from `Builder` which now builds a `Runtime`
- Remove `Session::builder()` in favor of `Runtime::builder()`

### Minor

- Add `cuda` feature for GPU inference on CUDA (Metal is used automatically on macOS)
- Add `Runtime` to prepare the model once and create one `Session` per inference thread
- Add `Builder::with_backend()` to select automatic, CPU-only or GPU-required inference
- Add `Builder::with_max_batch()` to limit resident plans: CPU retains the largest applicable plan and pads smaller tails; GPU retains the applicable fixed classes
- Add `Backend` and `BackendInfo` for backend selection (CPU or GPU) and information
- Add `Session::backend_info()` to get information about the backend

- Add the opt-in `yara-rules` feature, `RulesMode`, `DEFAULT_RULES`, `Builder::with_rules_mode()` and `Builder::with_ruleset()`, and `FeaturesOrRuled::extract_with_rules()` / `extract_with_ruleset()` for bounded YARA format rules executed by Vectorscan. Full and partial rules can enforce matches; disabled candidates abstain. Rules remain off by default.
- Add `RuleSet` source/file loading, `RuleSet::bundled()`, compiled-pack export and a persistent compilation cache. `Builder::build()` reports requested rule initialization failures.
- Add `ContentType::from_label()` and `Runtime::backend_info()`.
- Block promotion of the bundled SQLite header variants because MBTiles subtypes can share their complete scan prefix and file length
- Add rule-only output types from the knowledge base and canonical dataset taxonomy, including AVIF, QOI, GGUF and DuckDB, without changing the model's classes or existing output metadata and mappings

### Patch

- Require Rust 1.93 and verify all features and targets on that minimum version.
- Expose rules APIs in stable Rust documentation and use maintained rule sources ahead of stale packaging copies in repository builds.
- Reject unsafe Unix cache files and directories, avoid blocking on special cache files, and fall back to uncached compilation after bounded lock contention.
- Remove deprecated `package.authors` field in `Cargo.toml`
- Update dependencies

## 1.1.0

### Minor

- Unseal `SyncInput` and `AsyncInput` for custom file-like objects
- Support files larger than 4GB on 32-bits architectures

### Patch

- Update dependencies
- Document how to handle ONNX Runtime in the final binary

## 1.0.2

This version is not used by the `magika` crate. It has been yanked from `crates.io` because it
contained the version 1.0.2 of the `magika-cli` crate instead.

## 1.0.1

### Patch

- Update dependencies

## 1.0.0

### Patch

- Fix rustdoc nightly build by replacing deprecated `doc_auto_cfg` feature with `doc_cfg` (RFC 3631)

## 0.2.2

### Patch

- Change description and documentation

## 0.2.1

### Patch

- Update dependencies

## 0.2.0

### Major

- Change `FileType::Ruled` to take `ContentType` directly and remove `RuledType`
- Change `InferredType::content_type` to describe the content type when overwritten
- Add `InferredType::inferred_type` for the (possibly overwritten) inferred content type

### Minor

- Remove features extraction logic of older models
- Use the `standard_v3_3` model instead of `standard_v3_2` (see [model changelog])
- Add `OverwriteReason` to document why the inferred content type is overwritten

### Patch

- Update dependencies
- Add inference tests with the new reference files
- Update features extraction test to the new reference file

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
- Add content types `ContentType::Random{bytes,txt}` (those are only returned in
  `InferredType::content_type` and not in `RuledType::content_type`)
- Add a `MODEL_MAJOR_VERSION` integer in addition to the `MODEL_NAME` string

### Patch

- Update dependencies

## 0.1.0-rc.2

### Patch

- Update dependencies

## 0.1.0-rc.1

### Minor

- Change model from `standard_v2_0` to `standard_v2_1`

## 0.1.0-rc.0

This version is the initial implementation and should be considered unstable. In particular, it
ships a new model in comparison to the Python binary and we would love feedback on
[GitHub](https://github.com/google/magika/issues).

## 0.0.0

This version is a placeholder and does not expose anything.

[model changelog]: https://github.com/google/magika/blob/main/assets/models/CHANGELOG.md
