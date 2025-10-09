---
title: Understanding the Output
---

Whether you use the CLI or one of the language bindings (Python, Rust, JavaScript), Magika provides the same core prediction data. While many users only need the final content type label, detailed information is always available. The CLI offers flexible output formats like JSON, and the APIs provide dedicated result objects (e.g., the Python `MagikaResult` object).

The meaning of each field is best understood through an example.

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
- the `output` block returns information about the prediction of "Magika the tool", which, as discuss in previous sections, considers a number of aspects such as the prediction of the deep learning model, its confidence score, and the selected prediction mode. In the example above, the model's confidence was high enough to be trustworthy, and thus the output of the "Magika the tool" matches the content type inferred by the deep learning model.
- the `dl` and `output` blocks contain a number of metadata about the predicted content type, such as a simple textual label suitable for automated processing (`label`), a human-readable description (`description`), MIME Type (`mime_type`), a list of extensions usually associated with the predicted content type (`extensions`), a high-level group (`group`), and a boolean that indicates whether the type is textual or not (`is_text`).

Here is how to interpret the output:
- `path`: The file path corresponding to this prediction.
- `result.status`: `ok` indicates a successful scan. If the status is not `ok`, the `value` field will be absent.

The `value` field is present on successful scans and contains the following details:
- `score`: The model's confidence in this prediction.
- `dl`: Contains the raw prediction from the deep learning model.
- `output`: Contains the final prediction from "Magika the tool." This result considers the model's prediction, its confidence score, and the selected prediction mode. In this example, the model's confidence was high, so the final output matches the model's prediction.

Within both `dl` and `output`, you will find:
- `label`: A simple, machine-readable content type label (e.g., `javascript`). The possible values for `dl.label` and `output.label` are documented in each model's README.
- `description`: A human-readable description.
- `mime_type`: The corresponding MIME type.
- `group`: A high-level category (e.g., code, document, media).
- `is_text`: A boolean indicating if the content is textual.
- `extensions`: A list of common file extensions for this content type.

As mentioned previously, when the model is not used (e.g., for empty files), `dl.label` is set to `undefined`, and the output block will contain a generic content type like `txt` or `unknown`.

For most applications, you should use the `output.label` field, which is the default output of the CLI. The raw `dl` block is provided primarily for debugging and advanced use cases.

See also the [FAQ](/magika/additional-resources/faq) for why it is best to integrate Magika's results by focusing on label rather than other fields like `mime_type`.
