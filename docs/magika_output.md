# Magika Output Documentation

When using `--json` or `--jsonl`, Magika returns the following information:
```shell
$ magika code.py --json
[
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
]
```

The Python API returns similar information.

Here we explain what this output means and how to interpret it:
- the `path` is simply the file path this prediction is referring to (relevant when scanning multiple files at the same time)
- the `dl` block returns information about the prediction with the deep learning model. In this case, the model predicted `python` with a score of `0.99`.
- the `output` block returns information about the prediction of "Magika the tool", which considers a number of aspects such as the prediction of the deep learning model, its confidence, the indicated "prediction mode" (see `--prediction-mode`). In this example, the model's confidence was high enough to be trustworthy, and thus the output of the "Magika the tool" matches the content type inferred by the deep learning model.

In case the model's confidence is low, the output from "Magika the tool" would be a generic content type, such as "Generic text document" or "Unknown binary data".

Note that the deep learning model is not used in all cases, e.g., if a file is too small (currently less than 16 bytes), in which case the `ct_label` of the `dl` block is set to `None`.

We expect most clients to just need `output.ct_label`; we return the raw prediction of the deep learning model mostly for debugging.

See also the [FAQs](./faq.md) about why it may be appropriate to integrate Magika's results by focusing on `ct_label`, rather than `description`, `mime_type`, or other output formats.
