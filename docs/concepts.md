# Magika Concepts

Here we discuss concepts about Magika that are relevant regardless of which binding/language you are using.

## Models and supported content types

In essence, Magika clients are wrappers around deep learning models. We started with the `standard_v1` model, but with time we developed more advanced models that support the detection of 2x content types (100+ => 200+), with pretty much the same overall average accuracy and inference time.

Each model supports the detection of a specific set of content types, which are documented in the model's README. For example, here is the [documentation for the `standard_v3_1` model](../assets/models/standard_v3_1/README.md). More recent models usually support more content types than less recent onces, but that may not necessarily be the case.

Notes on the improvements and tradeoffs for each model can be found in the [models/CHANGELOG.md](../assets/models/CHANGELOG.md) file.

Supported clients and bindings usually integrate the latest available model, but that is not always the case. For more information, check the [bindings section of the main README.md](../README.md#bindings).

Note that you may find references in this repository (in the documentation, assets, source code, etc.) about a "content types knowledge base" (in short, content types KB). The KB has information about all the content types we are aware of, which is a super set of the content types supported but a given model. In other words, a content type being present in the KB does *not* imply that one of the model supports its detection. As mentioned above, the source of truth to know which content type can be detected by a given model is its `README.md` file.


## Magika prediction mode

Magika's underlying model returns a prediction with an associated score. This score can be interpreted as a confidence score: the higher the score, the more confident the model is. At times, the prediction is with a very high confidence, and thus it can be easily trusted. At other times, that's not the case, and the model's prediction has a low confidence.

One common question when using these classification models is: for which confidence score one can trust the results? The answer is not so simple; and, in the case of Magika, it actually depends on the specific content type being predicted.

Magika deals with this aspects in two ways:
- Each model ships with per-content-type thresholds, which are tuned by looking at the confidence score distribution when evaluating the model with our validation dataset.
- A misdetection can have more or less negative repercussions depending on the deployment scenario. In some cases, one may prefer to get an output content type only for a high-enough confidence; in other cases, one may be interested to know what is the model's best guess. To this end, Magika has a configurable prediction mode, `high-confidence`, `default`, `best-guess`, which lets the user selects the most appropriate setting. This config can be selected via a command line option or via an optional arguments in the constructors of various bindings' objects.


# Magika output

Regardless of whether you use the command-line interface or the language bindings (Python, Rust, JavaScript, etc.), Magika delivers the same core prediction data. The CLI allows for flexible output formatting, including detailed JSON, while the APIs encapsulate the information within dedicated objects, which vary depending on the bindings (for example, the python `Magika` module returns a `MagikaResult` object, see the docs [here](../python/README.md)).

The semantics of the various fields is best described with an example. Consider the following:

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
- the `output` block returns information about the prediction of "Magika the tool", which considers a number of aspects such as the prediction of the deep learning model, its confidence score, and the selected "prediction mode" (see discussion above). In the example above, the model's confidence was high enough to be trustworthy, and thus the output of the "Magika the tool" matches the content type inferred by the deep learning model; but that is not necessarily the case: In case the model's confidence is too low (where "too low" depends from the predicted content type and "prediction mode"), the output from "Magika the tool" would be a generic content type, such as "Generic text document" (`TXT`) or "Unknown binary data" (`UNKNOWN`).
- the `dl` and `output` blocks contain a number of metadata about the predicted content type, such as a simple textual label suitable for automated processing (`label`), a human-readable description (`description`), MIME Type (`mime_type`), a list of extensions usually associated with the predicted content type (`extensions`), a high-level group (`group`), and a boolean that indicates whether the type is textual or not (`is_text`).

Note that the deep learning model is not used for every prediction. This could happen for a number of reasons, e.g., if a file is empty, if it is too small for a meaningful prediction, or if it is not a regular file (like a directory or a symlink).
In these cases, the `dl` block metadata is set to `undefined` (e.g., `dl.label = ContentTypeLabel.UNDEFINED`), while the `output` block is always set appropriately.

We expect most clients to just use `output.label`; we return the raw prediction of the deep learning model mostly for debugging.

See also the [FAQs](./faq.md) about why it may be appropriate to integrate Magika's results by focusing on `label`, rather than `description`, `mime_type`, or other output formats.
