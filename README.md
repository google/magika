# Magika

Magika is a new content type detection tool based on deep learning. Under the hood, Magika employs a custom, highly optimized deep-learning model that only weighs about 1MB, and enables precise file identification within milliseconds, even when running on a CPU. In an evaluation with more than 1M files and over 100 content types (covering both binary and textual file formats), Magika achieves 99%+ precision and recall. Internally, Magika is used at scale to help improve Google users’ safety by routing Gmail, Drive, and Safe Browsing files to the proper security and content policy scanners.

We release as open source a Python client (a command-line tool and an API), and an experimental TensorflowJS model.

Try Magika today with our [web demo](https://google.github.io/magika/), which runs locally in your browser!

<p align="center">
    <img src="./assets/magika-screenshot.png" width="600">
</p>


## Highlights

- Available as a python command line, a python API, and an experimental TFJS version (which powers our web demo).
- Trained on a dataset of over 25M files across more than 100 content types.
- On our evaluation, Magika achieves 99%+ average precision and recall, outperforming existing approaches.
- More than 100 content types (see full list here). TODO: add link.
- After the model is loaded (this is a one-off overhead), the inference time is about 5ms per file.
- Batching: You can pass to the command line and API multiple files at the same time, and Magika will use batching to speed up the inference time. You can invoke Magika with even thousands of files at the same time. You can also use `-r` for recursively scanning a directory.
- Magika uses a per-content-type threshold system that determines whether to "trust" the prediction for the model, or whether to return a generic label, such as "Generic text document" or "Unknown binary data".
- Support three different prediction modes, which controls the tolerance to errors: `high-confidence`, `medium-confidence`, and `best-guess`.
- It's open source (and we will release more components later this year)!

For more details, see the documentation for the [python package](./python/DOCS.md) and for the [js package](./js/DOCS.md).


## Getting Started

### Installation

Magika is available as `magika` on PyPI:

```shell
$ pip install magika
```


### Usage

#### Python command line

Examples:

```shell
$ magika -r tests_data/
tests_data/README.md: Markdown document (text)
tests_data/basic/code.asm: Assembly (code)
tests_data/basic/code.c: C source (code)
tests_data/basic/code.css: CSS source (code)
tests_data/basic/code.js: JavaScript source (code)
tests_data/basic/code.py: Python source (code)
tests_data/basic/code.rs: Rust source (code)
...
tests_data/mitra/7-zip.7z: 7-zip archive data (archive)
tests_data/mitra/bmp.bmp: BMP image data (image)
tests_data/mitra/bzip2.bz2: bzip2 compressed data (archive)
tests_data/mitra/cab.cab: Microsoft Cabinet archive data (archive)
tests_data/mitra/elf.elf: ELF executable (executable)
tests_data/mitra/flac.flac: FLAC audio bitstream data (audio)
...
```

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

```shell
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

See [`./python/DOCS.md`](./python/DOCS.md) for detailed documentation.


#### Python API

Examples:

```python
>>> from magika import Magika
>>> m = Magika()
>>> res = m.identify_bytes(b"# Example\nThis is an example of markdown!")
>>> print(res.output.ct_label)
markdown
```


See [`./python/DOCS.md`](./python/DOCS.md) for detailed documentation.


## Development Setup

We use poetry for development and packaging:

```shell
$ git clone https://github.com/google/magika
$ cd magika/python
$ poetry shell && poetry install
$ magika ../tests_data
```

To run the tests:

```shell
$ cd magika/python
$ poetry shell
$ pytest tests/
```



## Known Limitations & Contributing

Magika improves over the state of the art, but it can be improved on a number of aspects: detection accuracy, support for additional content types, bindings for more languages, etc.

Moreover, this first release does not support polyglots, and it was not specifically trained or evaluated against adversarial samples.

We would love to hear from the community and get help. We are particularly interested to hear about encountered problems, misdetections, features requests, need for support for additional content types, etc.

Check our open GitHub issues to see what is on our roadmap and please report misdetections or feature requests by either opening GitHub issues (preferred) or by emailing us at magika-dev@google.com.

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for details.


## Additional Resources

- [Google's OSS blog post](https://opensource.googleblog.com/2024/02/magika-ai-powered-fast-and-efficient-file-type-identification.html) about Magika announcement.
- Web demo: [web demo](https://google.github.io/magika/).


## Citation
If you use this software for your research, please cite it as:
```bibtex
@software{magika,
author = {Fratantonio, Yanick and Bursztein, Elie and Invernizzi, Luca and Zhang, Marina and Metitieri, Giancarlo and Kurt, Thomas and Galilee, Francois and Petit-Bianco, Alexandre and Farah, Loua and Albertini, Ange},
title = {{Magika content-type scanner}},
url = {https://github.com/google/magika}
}
```

## License

Apache 2.0; see [`LICENSE`](LICENSE) for details.

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
