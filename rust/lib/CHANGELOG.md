# Changelog

## 2.0.0-rc.0

### Major

- Identify files with built-in format resolvers before model inference by default
- Follow symbolic links by default when identifying files (`Options::follow_symlink`)
- Replace `FileType::{Inferred,Ruled}` and `InferredType` with `FileType::Regular(RegularFileType)`
  and remove `FileType::score()`
- Mark `FileType` as `#[non_exhaustive]`
- Return `RegularFileType` instead of `FileType` from
  `Session::identify_{content,features,features_batch}`
- Replace `FeaturesOrRuled` and `FeaturesOrRuled::extract_{sync,async}()` with `FeaturesOr<T>` and
  `FeaturesOr::extract_{file,content}()`
- Add `{in_heuristics,in_resolvers,in_ml_model,vetoed_by_resolvers}` fields to `TypeInfo`
- Remove the dependency on the ONNX Runtime
- Remove async support for `Session`
- Remove the `_async` methods of `Session`
- Remove the `_sync` suffix from the `Session` methods
- Replace the `SyncInput` and `AsyncInput` traits with the `Input` struct
- Remove `Error` and `Result` using the `anyhow` crate instead
- Change all methods from `Builder` which now builds a `Runtime`
- Remove `Session::builder()` in favor of `Runtime::builder()`
- Bump Rust edition to 2024 and set MSRV to 1.93

### Minor

- Add 184 content types identified by the built-in format resolvers
- Report files that are not regular instead of trying to read them
- Add `Options` (`use_custom_rules`, `use_resolvers`, `use_model`, `prediction_mode`,
  `follow_symlink`, `debug_mode`) and `PredictionMode`
- Add `RegularFileType::content_types` and `{FileType,RegularFileType}::content_types()`
- Add `RegularFileType::metadata` for format-specific metadata and `DebugInfo` for debugging
- Add `OverwriteReason::{ResolversVeto,RulesVeto}` and `MODEL_{,TRAINING_}LABELS`
- Add `pipeline` feature to identify many paths in parallel
- Add `Rules` and `Options::use_custom_rules` for custom rules
- Add `TypeInfo::{possible,model}_output()` to list possible outputs
- Add `ContentType::from_label()` to get a content type from its label
- Add `Input::from_{slice,box,vec,file}()` and `From`/`TryFrom` conversions
- Generalize `Session::identify_features_batch()` to take `&[impl Borrow<Features>]`
- Add `cuda` feature for GPU inference on CUDA (Metal is used automatically on macOS)
- Add `Runtime` to prepare the model once and create one `Session` per inference thread
- Add `Builder::with_backend()` to select automatic, CPU-only or GPU-required inference
- Add `Builder::with_max_batch()` to prepare only the batch classes a caller can reach
- Add `Builder::with_options()` and `{Runtime,Session}::options{,_mut}()` to configure options
- Add `Backend` and `BackendInfo` for backend selection (CPU or GPU) and information
- Add `{Runtime,Session}::backend_info()` to get information about the backend
- Implement `Hash` for `ContentType`, `Copy`/`PartialEq`/`Eq` for `OverwriteReason`, and `Clone` for
  `Builder`

### Patch

- Update metadata (descriptions, MIME types, groups, and extensions) for 29 existing content types
- Read small files once when extracting features
- Use positional file reads on Unix and Windows
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
