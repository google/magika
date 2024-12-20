# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Note that for version number starting with a `0`, i.e., `0.x.y`, a bump of `x`
should be considered as a major (and thus potentially breaking) change. See
semver guidelines for more details about this.

## [0.6.0-rc3] - 2024-11-20

- Fixed problems with installing Magika via `uv` on MacOS.


## [0.6.0-rc2] - 2024-11-19

- Fixed manylinux wheel and other small fixes.


## [0.6.0-rc1] - 2024-10-07

- The package ships a new command line client written in Rust. This replaces the old client written in Python.
- Magika's new CLI output JSON is slightly different: it is now produced by the Rust client, the `score` field is at the same level as `dl` and `output` (and it is not anymore at `dl.score` and `output.score`), the output includes `is_text`, which indicates whether the detected content type is of textual nature, and `extensions`, a list of extensions associated with the predicted content type. Check an example in the main [README.md](../README.md).
- The Magika's python API now returns a `MagikaResult`, which is a [`absl::StatusOr`](https://abseil.io/docs/cpp/guides/status)-like object that wraps `MagikaPrediction`. See the API documentation [here](https://github.com/google/magika/blob/main/python/README.md). The rationale is to have a clear separation between valid predictions and error situations.
- `dl.ct_label` and `output.ct_label` are deprecated and now accessible as `dl.label` and `output.label`. Such `label` field is of type `ContentTypeLabel` (which is a `StrEnum`). You can treat it as a string.
- the `magic` metadata is deprecated; use `description` instead.
- the content type metadata now includes `is_text` (a boolean determining whether the predicted content type is of textual form) and `extensions` (a list of file extensions usually associated with this content type).
- When the deep learning model is not used, `dl.label` is now `ContentTypeLabel.UNDEFINED`, rather than setting the full `dl` field to `None`.
- The python module is now fully typed
- Under the hood, the module uses a new model, which has support for more than 200 content types. You may want to check the [new list of supported content types](https://github.com/google/magika/blob/main/assets/models/standard_v2_1/README.md) and update any code depending on specific results. Note that the model uses more input bytes than the previous one, and it is slightly slower. If this is a problem for your use case, please reach out! We do have smaller models (although slightly less accurate); your feedback would is helpful to know what to prioritize.


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
