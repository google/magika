---
title: Creating New Bindings
---

These notes aim at helping bindings developers.

### Reference implementation

The reference implementation is the python's `Magika` module, at `python/src/magika.py`.

The input vs. expected output examples are stored in `tests_data/reference`. See below about information on the format.

The reference tests are generated with `cd python && uv run ./scripts/generate_reference.py`.


### Aspects to implement

There are three aspects that need to be implemented:
- Logic that handles "should we even use the model"? See `_get_result_or_features_from_path`.
- Features extraction. See `_extract_features_from_seekable`.
- How to obtain "Magika's output" from the model's prediction, the score (which depends on the prediction mode, thresholds, and overwrite_map). See `_get_output_ct_label_from_dl_result`.


### Testing

We have a number of test cases that one can use to check that a new implementation matches the reference implementation.

Testing that the output (e.g., model prediction, tool overall prediction, score) of the tool matches the expectations:
- We have a number of test cases that cover normal situations as well as corner cases related to small files, content types with custom thresholds and overwrite maps, and prediction modes. Note that these corner cases are model-specific (the actual weights). We use a fuzzing-like approach to generate them.
- These examples are stored in two formats, "examples by path" and "examples by content". They are stored at `tests_data/reference/<model-name>-inference_examples_by_content.json.gz` and `tests_data/reference/<model-name>-inference_examples_by_content.json.gz`. These store a list of `ExampleByPath` and `ExampleByContent` (defined in `python/tests/test_inference_vs_reference.py`), respectively.

Testing the features extraction:
- Input and expected output of the features extraction: `tests_data/reference/features_extraction_examples.json.gz`.
- The JSON contains a list of `FeaturesExtractionExample` (defined in `python/tests/test_features_extraction_vs_reference.py`).
- Suggestion: having a testable "extract features" function makes your life much easier.
- Note that end-to-end tests would not be enough to be confident the features extraction is correctly implemented, as small bugs may require VERY specific input to show differences.

What is *not* covered by the existing tests:
- How to deal with special files (e.g., symlinks, directory).
- How to deal with `permission error`.
- How to deal with `file_not_found_error`.
