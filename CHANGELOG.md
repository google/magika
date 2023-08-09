# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
