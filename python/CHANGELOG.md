# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Note that for version number starting with a `0`, i.e., `0.x.y`, a bump of `x`
should be considered as a major (and thus potentially breaking) change. See
semver guidelines for more details about this.


## [0.6.1] - 2025-03-19

### Overview

Magika v0.6.1 is a significant update featuring a new model with 2x supported content types, a new command line client in Rust, performance improvements, API enhancements, and a few breaking changes. This changelog entry rolls up all changes from v0.5.1, the last stable release.

> [!IMPORTANT]
> There are a few breaking changes! After reading about the new key features and improvements, we suggest to consult the migration guide below and the [updated documention](./README.md).


### Key Features and Improvements

- **New deep learning model:** We introduce a new model, `standard_v3_2`, which supports 2x content types (200+ in total, see full list [here](../assets/models/standard_v3_2/README.md)), has a similar ~99% average accuracy, and is ~20% faster, with an inference speed of about ~2ms on CPUs (YMMV depending on your testing setup). See [models' CHANGELOG](../assets/models/CHANGELOG.md) for more information.
- **New command line client, written in Rust:** We developed a new command line client, written in Rust, which is not affected by the one-time boostrap overhead caused by the python's interpreter itself. This new client is packaged, pre-compiled, into the `magika` python package. This new client replaces the old client written in Python (but the old Python one is still available as a fallback for those platforms for which we don't have precompiled rust binaries).
- **New stream-based identification:** Added `identify_stream(stream: typing.BinaryIO)` API to infer content types from open binary streams. ([#970](https://github.com/google/magika/issues/970))
- **Improved path handling:** `identify_path` and `identify_paths` now accept `Union[str, os.PathLike]` objects. You no longer need to explicitly use `pathlib.Path`. ([#935](https://github.com/google/magika/issues/935))
- **Improved python API:** The new Python APIs offer a number of improvements, including: the inference APIs now return a `MagikaResult`, which is a [`absl::StatusOr`](https://abseil.io/docs/cpp/guides/status)-like object that wraps `MagikaPrediction`, with a clear separation between valid predictions and error situations; the output content types (`label`) are not just `str` anymore, but of type `ContentTypeLabel`, making integrations more robust (`ContentTypeLabel` extends `StrEnum`: thus, they are not just `str`, but you can treat them as such). The `MagikaPrediction` object now has additional `is_text` and `extensions` fields (in addition to the existing `label`, `mime_type`, `group`, and `description`).
- **New debugging APIs**: Added new APIs to ease debugging and introspection, such as `get_output_content_types()`, `get_model_content_types()`, `get_module_version()`, and `get_model_name()`.


### Breaking Changes and Migration Guide

This release introduces several breaking changes. Please review this guide carefully to update your code:

1. **New `identify_*` API output format:** The inference Python APIs now return a `MagikaResult` object, which is similar to `absl::StatusOr`; This provides a cleaner way to handle errors. `dl.ct_label` and `output.ct_label` are renamed to `dl.label` and `output.label`. `label`s are now of type `ContentTypeLabel`, which extends `StrEnum` (thus, they are not just `str`, but you can treat them as such). The `score` field is now at the top level, alongside `dl` and `output`. The `magic` field has been removed as it was often either incorrect or reduntant; use `description` instead.

  - **Before (v0.5.x and earlier):**
    ```python
    import magika
    m = magika.Magika()
    result = m.identify_path("my_file.py")
    print(result.output.ct_label)  # Assumed success
    ```

  - **After (v0.6.1):**
    ```python
    import magika
    m = magika.Magika()
    result = m.identify_path("my_file.py")
    if result.ok():
        print(result.output.label)
    else:
        print(f"Error: {result.status}")
    ```


1. **CLI Output Format Change (v0.6.0):**  The JSON output format of the CLI has changed. Those changes are analogous to the changes to the Python APIs. The `score` field is now at the top level, alongside `dl` and `output`, and is no longer nested within `dl` or `output`. The output also includes `is_text` and `extensions` fields. The `magic` metadata has been removed as it was often either incorrect or reduntant; use `description` instead. Moreover, similarly to what happens under the hood with the `StatusOr` pattern, `result.status` indicates whether the prediction was successful, and the prediction results are available under the `result.value` key.

  - **Before (v0.5.x and earlier):**  (Illustrative example - adapt to your specific output)
    ```json
    {
      "path": "code.py",
      "dl": {
        "ct_label": "python",
        "score": 0.9940916895866394,
        "group": "code",
        "mime_type": "text/x-python",
        "magic": "Python script",
        "description": "Python source"
      },
      "output": {
        "ct_label": "python",
        "score": 0.9940916895866394,
        "group": "code",
        "mime_type": "text/x-python",
        "magic": "Python script",
        "description": "Python source"
      }
    }
    ```

  - **After (v0.6.1):**
    ```json
    {
      "path": "code.py",
      "result": {
        "status": "ok",
        "value": {
          "dl": {
            "description": "Python source",
            "extensions": [
              "py",
              "pyi"
            ],
            "group": "code",
            "is_text": true,
            "label": "python",
            "mime_type": "text/x-python"
          },
          "output": {
            "description": "Python source",
            "extensions": [
              "py",
              "pyi"
            ],
            "group": "code",
            "is_text": true,
            "label": "python",
            "mime_type": "text/x-python"
          },
          "score": 0.9890000224113464
        }
      }
    }
    ```


1. **`dl.label == ContentTypeLabel.UNDEFINED` when the model is not used:**  There are situations in which the deep learning model is not used, for example when the file is too small or empty. In these cases, `dl.label` is now set to `ContentTypeLabel.UNDEFINED` instead of having the full `dl` block being set to `None`.

  - **Before (v0.5.x and earlier):**
      ```python
      # ... (assuming successful result)
      if prediction.dl is not None:
          print(prediction.dl.ct_label)
      ```

  - **After (v0.6.1):**
      ```python
      # ... (assuming successful result)
      if prediction.dl.label != magika.ContentTypeLabel.UNDEFINED:
          print(prediction.dl.label)
      ```


1. **Expanded List of Content Types:** The model now supports over 200 content types.

  - **Migration:** Review the [updated list of supported content types](../assets/models/standard_v3_2/README.md) and adjust any code that relies on specific content type labels returned by previous versions. Labels have *not* changed, but a file previously detected as `javascript` may not be detected as `typescript`.  Consider using `get_output_content_types()` to dynamically retrieve the supported labels.


1. **Pure Python Wheel and Rust Client Fallback:** If you are installing Magika on a platform *without* pre-built wheels (e.g., Windows on ARM), you will automatically get the pure-python wheel. In this case, the package does *not* include the Rust binary client, but it does include the old python client as fallback; you can use such old python client with `$ magika-python-client`.


### Full Changelog

For a detailed list of all changes, including those from the -rc releases, please refer to the individual changelog entries for each release candidate:
- [0.6.1-rc3](https://github.com/google/magika/blob/python-v0.6.1-rc3/python/CHANGELOG.md#061-rc3---2025-03-17)
- [0.6.1-rc2](https://github.com/google/magika/blob/python-v0.6.1-rc2/python/CHANGELOG.md#061-rc2---2025-03-11)
- [0.6.1-rc1](https://github.com/google/magika/blob/python-v0.6.1-rc1/python/CHANGELOG.md#061-rc1---2025-02-04)
- [0.6.1-rc0](https://github.com/google/magika/blob/python-v0.6.1-rc0/python/CHANGELOG.md#061-rc0---2025-01-23)
- [0.6.0-rc3](https://github.com/google/magika/blob/python-v0.6.0-rc3/python/CHANGELOG.md#060-rc3---2024-11-20)
- [0.6.0-rc2](https://github.com/google/magika/blob/python-v0.6.0-rc2/python/CHANGELOG.md#060-rc2---2024-11-19)
- [0.6.0-rc1](https://github.com/google/magika/blob/python-v0.6.0-rc2/python/CHANGELOG.md#060-rc1---2024-10-07)


## [0.5.1] - 2024-03-06

- Add support for python 3.12. Magika now supports python >=3.8 and <3.13.
- Fix bugs for features extraction to cover more corner cases.
- Remove MIME types from table of supported content types (Relevant for `--list-output-content-types`; see FAQs for context).
- Refactor features extraction around a Seekable abstraction; we now have only one reference implementation.
- Start groundwork for v2 of features extraction.
- Various clean ups and internal refactors.


## [0.5.0] - 2024-02-15

- New public python APIs: `identify_paths`, `identify_path`, `identify_bytes`.
- The APIs now return a `MagikaResult` object.
- When the model's prediction has low confidence and we return a generic content
  type, print anyways (with a disclaimer) the model's best guess.
- Updated description for "unknown" to "Unknown binary data".
- Magika CLI now defaults to "high-confidence" mode. "default" mode is now called "medium-confidence".
- Magika CLI `-p/--output-probability` has been renamed to `-s/--output-score` for consistency.
- Default model is now called `standard_v1`.
- Major refactoring and clean up.

## [0.4.1] - 2024-02-07

- Various improvements and clean ups.


## [0.4.0] - 2023-12-22

### Changed

- Update model to dense_v4_top_20230910.
- Package now contains the model itself.
- Support reading from stdin:
  - `$ cat <path> | magika -`
  - `$ curl <url> | magika -`
- Change how we deal with padding, using 256 instead of 0. This boosts precision.
- "symlink" output label has been renamed to "symlinktext" to better reflect its nature.
- New `--prediction-mode` CLI option to indicate which confidence is required
  for the predictions. We support three modes: `best-guess`, `default`,
  `high-confidence`.
- Support for directories and symlinks similarly to `file`.
- Adapt `-r` / `--recursive` CLI option to be compatible with the new way magika
  handles directories.
- Add special handling for small files.
- Magika does not crash anymore when scanning files with permission
  issues. It now returns "permission_error".
- Do not resolve file paths (i.e., relative paths remain relative).
- Add --no-dereference CLI option: by default symlinks are dereferenced.
  this option makes magika not dereferencing symlinks. This is what `file` does.
- Clean up and many bug fixes.


## [0.3.1] - 2023-08-23

### Changed

- Removed warnings when using MIME type and compatibility mode.


## [0.3.0] - 2023-08-23

### Changed

- By default, magika now outputs a human-readable output.
- Add `-l` / `--label` CLI option to output a stable, content type label.
- JSON/JSONL output now shows all metadata about a given content type.
- Add metadata about magic and description for each relevant content type.
- Logs are now printed to stderr, not stdout.
- Add `--generate-report` CLI option to output a JSON report that can be useful for debugging and reporting feedback.
- Be more flexible with the required python version (now we require "^3.8" instead of "^3.8,<3.11")
- Show a descriptive error in case magika can't find any file to scan (instead of silently exiting).


## [0.2.2] - 2023-08-11

### Changed

- If the prediction score is higher than a given threshold (0.95), consider it
  regardless of the per-content-type threshold.
- Output format is back being just `<content type>`; group is displayed only
  when showing metadata.
- Update metadata of some content types.


## [0.2.1] - 2023-08-10

### Changed

- Several small bug fixes.


## [0.2.0] - 2023-08-09

### Added

- Input files are now processed in multiple small batches, instead of one big batch.
- Per-content-type threshold based on the 0.005 quantile for recall.
- MIME type and "group" metadata for all content types.
- Introduce basic support for compatibility mode.
- `-c` / `--compatibility-mode` CLI option to enable compatibility mode.
- `--no-colors` CLI option to disable colors.
- `-b` / `--batch-size` CLI option to specify the batch size.
- `--guess` / `--output-highest-probability` CLI option to output the content type with the highest probability regardless of its probability score.
- `--version` CLI option to print Magika's version.

### Changed

- Output follows the `<group>::<content type>` format.
- Probability score is not shown by default; enable with `-p`.
- Output is colored according to the file content type's group.
- Remove dependency from richlogger, add a much simpler logger.


## [0.1.0] - 2023-07-28

- First release.
