# Python Package Documentation

## Installation

Magika is available as `magika` on [PyPI](https://pypi.org/project/magika):

```shell
$ pip install magika
```

## Command Line Documentation

Magika takes one or more files as arguments, and a number of CLI options.

```help
$ magika -h
Usage: magika [OPTIONS] [FILE]...

  Magika - Determine type of FILEs with deep-learning.

Options:
  -r, --recursive                 When passing this option, magika scans every
                                  file within directories, instead of
                                  outputting "directory"
  --json                          Output in JSON format.
  --jsonl                         Output in JSONL format.
  -i, --mime-type                 Output the MIME type instead of a verbose
                                  content type description.
  -l, --label                     Output a simple label instead of a verbose
                                  content type description. Use --list-output-
                                  content-types for the list of supported
                                  output.
  -c, --compatibility-mode        Compatibility mode: output is as close as
                                  possible to `file` and colors are disabled.
  -s, --output-score              Output the prediction score in addition to
                                  the content type.
  -m, --prediction-mode [best-guess|medium-confidence|high-confidence]
  --batch-size INTEGER            How many files to process in one batch.
  --no-dereference                This option causes symlinks not to be
                                  followed. By default, symlinks are
                                  dereferenced.
  --colors / --no-colors          Enable/disable use of colors.
  -v, --verbose                   Enable more verbose output.
  -vv, --debug                    Enable debug logging.
  --generate-report               Generate report useful when reporting
                                  feedback.
  --version                       Print the version and exit.
  --list-output-content-types     Show a list of supported content types.
  --model-dir DIRECTORY           Use a custom model.
  -h, --help                      Show this message and exit.

  Magika version: "0.5.0"

  Default model: "standard_v1"

  Send any feedback to magika-dev@google.com or via GitHub issues.
```

Here are more details for some of the non-trivial options:
- `-r, --recursive`: by default, if you pass a directory as argument, magika will just print that it is a directory. This is to be backward compatible with existing tools, e.g., `file`. If you pass `-r`, magika will recursively scan every file in that directory.
- `--json`: make Magika output one single JSON with all the predictions for all files. Note that this will not output anything until the last file has been scanned; if you scan thousands of files, it could be quite slow!
- `--jsonl`: this makes Magika output one JSON per line (i.e., JSONL format). You may want to use this option if you are scanning hundreds or thousands of files.
- `-l, --label`: just output a textual content type label for each file.
- `-c, --compatibility-mode`: output a string that resembles what `file` would output (best-effort).
- `-s, --output-score`: output a string that resembles what `file` would output (best-effort).
- `-m, --prediction-mode`: Magika uses the deep learning model to perform content type inference, and it then determines whether the prediction should be considered as valid. This option tweaks the required confidence to output a prediction: `high-confidence` (the default), uses per-content-type thresholds that aim for high precision; `medium-confidence` uses a much lower number as the required confidence; `best-guess` outputs whatever content type has the highest score. You may want to use one or another mode depending on your deployment scenario.
- `--generate-report`: if you have a misdetection and would like to report it (Thank you!), you can use this option to generate a report that you can include in the github issue. **Do NOT send reports that may contain PII, the report contains (a small) part of the file!**

### Output format documentation

When using `--json` or `--jsonl`, Magika returns the following information:
```bash
$ magika doc.html --json
[
    {
        "path": "doc.html",
        "dl": {
            "ct_label": "html",
            "score": 0.9667956233024597,
            "group": "code",
            "mime_type": "text/html",
            "magic": "HTML document",
            "description": "HTML document"
        },
        "output": {
            "ct_label": "html",
            "score": 0.9667956233024597,
            "group": "code",
            "mime_type": "text/html",
            "magic": "HTML document",
            "description": "HTML document"
        }
    }
]
```

The semantics of this output is as follows:
- the `path` is simply the file path this prediction is referring to (relevant when scanning multiple files at the same time)
- the `dl` block returns information about the prediction with the deep learning model. In this case, the model predicted `html` with a score of `0.96`.
- the `output` block returns information about the prediction of "Magika the tool", which considers a number of aspects such as the prediction of the deep learning model, its confidence, the indicated "prediction mode" (see `--prediction-mode`). In this example, the model's confidence was high enough to be trustworthy, and thus the output of the "Magika the tool" matches the content type inferred by the deep learning model.

In case the model's confidence is low, the output from "Magika the tool" would be a generic content type, such as "Generic text document" or "Uknown binary data".

Note that the deep learning model is not used in all cases, e.g., if a file is too small (currently less than 16 bytes), in which case the `ct_label` of the `dl` block is set to `None`.


## API documentation

The `magika` modules define a `Magika` class that you can use for inference.

Example:

```python
>>> from magika import Magika
>>> m = Magika()
>>> res = m.identify_bytes(b"# Example\nThis is an example of markdown!")
>>> print(res.output.ct_label)
markdown
```

The `Magika` object exposes three methods:
- `identify_bytes(b"test")`: takes as input a stream of bytes and predict its content type.
- `identify_path(Path("test.txt"))`: takes as input one `Path` object and predicts its content type.
- `identify_paths([Path("test.txt"), Path("test2.txt")])`: takes as input a list of `Path` objects and returns the predicted type for each of them.

If you are dealing with big files, the `identify_path` and `identify_paths` variants are generally better: their implementation `seek()` around the file to extracted the needed features, without loading the entire content in memory.

These API returns an object of type [`MagikaResult`](./magika/types.py), which exposes the same information discussed above in the "Detailed output format" section.
