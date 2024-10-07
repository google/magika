# Magika Output Documentation

The command line tool and the various APIs (in Python, JS, Rust, etc.) all return the same information.

Consider the following example:

```shell
$ magika tests_data/basic/javascript/code.js --json
[
  {
    "path": "tests_data/basic/javascript/code.js",
    "result": {
      "status": "ok",
      "value": {
        "dl": {
          "description": "JavaScript source",
          "extensions": [
            "js",
            "mjs",
            "cjs"
          ],
          "group": "code",
          "is_text": true,
          "label": "javascript",
          "mime_type": "application/javascript"
        },
        "output": {
          "description": "JavaScript source",
          "extensions": [
            "js",
            "mjs",
            "cjs"
          ],
          "group": "code",
          "is_text": true,
          "label": "javascript",
          "mime_type": "application/javascript"
        },
        "score": 0.9710000157356262
      }
    }
  }
]
```

This is how to interpret the output:
- `path` is simply the file path this prediction is referring to (relevant when scanning multiple files at the same time).
- `result.status` indicates whether magika was able to scan the sample. `ok` means all was good, in which case a `value` field is present with the details about the output.
- `score` indicates the confidence of the prediction.
- the `dl` block returns information about the prediction with the deep learning model. In this case, the model predicted `javascript`.
- the `output` block returns information about the prediction of "Magika the tool", which considers a number of aspects such as the prediction of the deep learning model, its confidence, and the indicated "prediction mode" (see `--prediction-mode`). In this example, the model's confidence was high enough to be trustworthy, and thus the output of the "Magika the tool" matches the content type inferred by the deep learning model.
- the `dl` and `output` blocks contain a number of metadata about the predicted content type, such as a `label` suitable for automated processing, a human-readable description, MIME Type, a list of extensions usually associated with the predicted content type, a high-level group, and a `is_text` boolean that indicates whether the type is textual or not.

In case the model's confidence is low, the output from "Magika the tool" would be a generic content type, such as "Generic text document" or "Unknown binary data".

Note that the deep learning model is not used in all cases, e.g., if a file is too small (currently less than 16 bytes), in which case the `label` of the `dl` block is set to `ContentTypeLabel.UNDEFINED`.

We expect most clients to just use `output.label`; we return the raw prediction of the deep learning model mostly for debugging.

See also the [FAQs](./faq.md) about why it may be appropriate to integrate Magika's results by focusing on `label`, rather than `description`, `mime_type`, or other output formats.
