# Magika

[![image](https://img.shields.io/pypi/v/magika.svg)](https://pypi.python.org/pypi/magika)
[![NPM Version](https://img.shields.io/npm/v/magika)](https://npmjs.com/package/magika)
[![image](https://img.shields.io/pypi/l/magika.svg)](https://pypi.python.org/pypi/magika)
[![image](https://img.shields.io/pypi/pyversions/magika.svg)](https://pypi.python.org/pypi/magika)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/8706/badge)](https://www.bestpractices.dev/en/projects/8706)
![CodeQL](https://github.com/google/magika/workflows/CodeQL/badge.svg)
[![Actions status](https://github.com/google/magika/actions/workflows/python-build-package.yml/badge.svg)](https://github.com/google/magika/actions)
[![PyPI Monthly Downloads](https://static.pepy.tech/badge/magika/month)](https://pepy.tech/projects/magika)
[![PyPI Downloads](https://static.pepy.tech/badge/magika)](https://pepy.tech/projects/magika)

<!-- [![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/google/magika/badge)](https://securityscorecards.dev/viewer/?uri=github.com/google/magika) -->

Magika is a novel AI-powered file type detection tool that relies on the recent advance of deep learning to provide accurate detection. Under the hood, Magika employs a custom, highly optimized model that only weighs about a few MBs, and enables precise file identification within milliseconds, even when running on a single CPU. Magika has been trained and evaluated on a dataset of ~100M samples across 200+ content types (covering both binary and textual file formats), and it achieves an average ~99% accuracy on our test set.

Here is an example of what Magika command line output looks like:

<p align="center">
    <img src="./assets/magika-screenshot.png" width="600">
</p>

Magika is used at scale to help improve Google users' safety by routing Gmail, Drive, and Safe Browsing files to the proper security and content policy scanners, processing hundreds billions samples on a weekly basis. Magika has also been integrated with [VirusTotal](https://www.virustotal.com/) ([example](./assets/magika-vt.png)) and [abuse.ch](https://bazaar.abuse.ch/) ([example](./assets/magika-abusech.png)).

For more context you can read our initial [announcement post on Google's OSS blog](https://opensource.googleblog.com/2024/02/magika-ai-powered-fast-and-efficient-file-type-identification.html), and you can read more in our [research paper](#research-paper-and-citation), published at the IEEE/ACM International Conference on Software Engineering (ICSE) 2025.

You can try Magika without installing anything by using our [web demo](https://google.github.io/magika/), which runs locally in your browser! (Note: the website runs an older version of the model; but while results may differ, it is still useful to get an idea of Magika's capabilities.)

> [!IMPORTANT]
>
> - The documentation on GitHub refers to the latest, potentially unreleased and unstable version of Magika. The latest stable release of the `magika` Python package is `0.6.1`, and you can consult the associated documentation [here](https://github.com/google/magika/blob/python-v0.6.1/python/README.md). You can install the latest stable version with: `pip install magika`.
> - A detailed changelog and migration guidelines from older versions are discussed [here](https://github.com/google/magika/blob/python-v0.6.1/python/CHANGELOG.md).
> - Help testing the latest release candidate is very appreciated! See the available candidates [here](https://pypi.org/project/magika/#history) and the recent changes in the [CHANGELOG.md](./python/CHANGELOG.md). You can install the latest release candidate with `pip install --pre magika`.

# Highlights

- Available as a command line tool written in Rust, a Python API, and additional bindings for Rust, JavaScript/TypeScript (with an experimental npm package, which powers our [web demo](https://google.github.io/magika/)), and GoLang (WIP).
- Trained and evaluated on a dataset of ~100M files across [200+ content types](./assets/models/standard_v3_3/README.md).
- On our test set, Magika achieves ~99% average precision and recall, outperforming existing approaches.
- After the model is loaded (which is a one-off overhead), the inference time is about 5ms per file, even when run on a single CPU.
- You can invoke Magika with even thousands of files at the same time. You can also use `-r` for recursively scanning a directory.
- Near-constant inference time, independently from the file size; Magika only uses a limited subset of the file's content.
- Magika uses a per-content-type threshold system that determines whether to "trust" the prediction for the model, or whether to return a generic label, such as "Generic text document" or "Unknown binary data".
- The tolerance to errors can be controlled via different prediction modes, such as `high-confidence`, `medium-confidence`, and `best-guess`.
- The client and the bindings are already open source, and more is coming soon!

# Table of Contents

1. [Getting Started](#getting-started)
   1. [Installation](#installation)
   1. [Usage](#usage)
      1. [Command line client](#command-line-client)
      1. [Python module and other bindings](#python-module-and-other-bindings)
1. [Documentation](#documentation)
1. [Bindings](#bindings)
1. [Development Setup](#development-setup)
1. [Known Limitations & Contributing](#known-limitations--contributing)
1. [Additional Resources](#additional-resources)
1. [Research Paper and Citation](#research-paper-and-citation)
1. [Security vulnerabilities](#security-vulnerabilities)
1. [License](#license)
1. [Disclaimer](#disclaimer)

# Getting Started

## Installation

Magika is available as `magika` on PyPI:

```shell
$ pip install magika
```

If you intend to use Magika only as a command line, you may want to use `$ pipx install magika` instead.

If you want to test out the latest release candidate, you can install with `pip install --pre magika`.

If you want to test Magika within a Docker container, you can run:

```shell
git clone https://github.com/google/magika
cd magika/
docker build -t magika .
docker run -it --rm -v $(pwd):/magika magika -r /magika/tests_data/basic
```

## Usage

### Command line client

Magika is available as a command line tool, written in Rust (the `magika` python package ships this client). Here below are a few examples on how to use it, and the output of `magika --help`, which documents the list of options. See [here](./rust/cli/README.md) for developer notes and documentation on how to install the Rust client via `cargo`.

Examples:

```shell
$ cd tests_data/basic && magika -r *
asm/code.asm: Assembly (code)
batch/simple.bat: DOS batch file (code)
c/code.c: C source (code)
css/code.css: CSS source (code)
csv/magika_test.csv: CSV document (code)
dockerfile/Dockerfile: Dockerfile (code)
docx/doc.docx: Microsoft Word 2007+ document (document)
epub/doc.epub: EPUB document (document)
epub/magika_test.epub: EPUB document (document)
flac/test.flac: FLAC audio bitstream data (audio)
handlebars/example.handlebars: Handlebars source (code)
html/doc.html: HTML document (code)
ini/doc.ini: INI configuration file (text)
javascript/code.js: JavaScript source (code)
jinja/example.j2: Jinja template (code)
jpeg/magika_test.jpg: JPEG image data (image)
json/doc.json: JSON document (code)
latex/sample.tex: LaTeX document (text)
makefile/simple.Makefile: Makefile source (code)
markdown/README.md: Markdown document (text)
[...]
```

```shell
$ magika ./tests_data/basic/python/code.py --json
[
  {
    "path": "./tests_data/basic/python/code.py",
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
        "score": 0.753000020980835
      }
    }
  }
]
```

```shell
$ cat doc.ini | magika -
-: INI configuration file (text)
```

```help
$ magika --help
Determines the content type of files with deep-learning

Usage: magika [OPTIONS] [PATH]...

Arguments:
  [PATH]...
          List of paths to the files to analyze.

          Use a dash (-) to read from standard input (can only be used once).

Options:
  -r, --recursive
          Identifies files within directories instead of identifying the directory itself

      --no-dereference
          Identifies symbolic links as is instead of identifying their content by following them

      --colors
          Prints with colors regardless of terminal support

      --no-colors
          Prints without colors regardless of terminal support

  -s, --output-score
          Prints the prediction score in addition to the content type

  -i, --mime-type
          Prints the MIME type instead of the content type description

  -l, --label
          Prints a simple label instead of the content type description

      --json
          Prints in JSON format

      --jsonl
          Prints in JSONL format

      --format <CUSTOM>
          Prints using a custom format (use --help for details).

          The following placeholders are supported:

            %p  The file path
            %l  The unique label identifying the content type
            %d  The description of the content type
            %g  The group of the content type
            %m  The MIME type of the content type
            %e  Possible file extensions for the content type
            %s  The score of the content type for the file
            %S  The score of the content type for the file in percent
            %b  The model output if overruled (empty otherwise)
            %%  A literal %

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Python module and other bindings

While the command line client is handy for many use cases, it may not be the most suitable for automated workflows. Thus, Magika comes with bindings for Python and other languages (see the [Bindings section](#bindings) below for more details).

Here is a few examples on how to use the `Magika` Python module:

```python
>>> from magika import Magika
>>> m = Magika()
>>> res = m.identify_bytes(b'function log(msg) {console.log(msg);}')
>>> print(res.output.label)
javascript
```

```python
>>> from magika import Magika
>>> m = Magika()
>>> res = m.identify_path('./tests_data/basic/ini/doc.ini')
>>> print(res.output.label)
ini
```

```python
>>> from magika import Magika
>>> m = Magika()
>>> with open('./tests_data/basic/ini/doc.ini', 'rb') as f:
>>>     res = m.identify_stream(f)
>>> print(res.output.label)
ini
```

Please consult the [python documentation](./python/README.md) for details on the Python `Magika` API, and the [additional documentation listed below](#documentation) for more information about the output format and other aspects.

# Documentation

- [Core concepts: models, prediction mode, magika's output, and content types knowledge base](./docs/concepts.md)
- [List of supported content types by the latest model, `standard_v3_3`](./assets/models/standard_v3_3/README.md)
- [Documentation about the CLI](./rust/cli/README.md)
- [Documentation about the python `magika` package and module](./python/README.md)
- [Frequently Asked Questions](./docs/faq.md)

# Bindings

| Artifact                                                                   | Status         | Latest version | Default model                                              |
| -------------------------------------------------------------------------- | -------------- | -------------- | ---------------------------------------------------------- |
| [Python `Magika` module](./python/README.md)                               | Stable         | `0.6.1`        | [`standard_v3_2`](./assets/models/standard_v3_2/README.md) |
| [Rust `magika` CLI](https://crates.io/crates/magika-cli)                   | Stable         | `0.1.2`        | [`standard_v3_3`](./assets/models/standard_v3_2/README.md) |
| [Rust `magika` library](https://docs.rs/magika)                            | Stable         | `0.2.0`        | [`standard_v3_3`](./assets/models/standard_v3_2/README.md) |
| TypeScript / NPM package ([README](./js/README.md) & [docs](./docs/js.md)) | Stable         | `0.3.2`        | [`standard_v3_3`](./assets/models/standard_v3_3/README.md) |
| [Demo Website](https://google.github.io/magika/)                           | Stable         | -              | [`standard_v3_3`](./assets/models/standard_v3_3/README.md) |
| [GoLang](./go/README.md)                                                   | In development | -              | -                                                          |

# Development Setup

We have Magika bindings in multiple languages; each of them has its own development setup. Consult the documentation associated to each binding for more information. For example, for the python `magika` package and module, consult [python/README.md](./python/README.md).

# Known Limitations & Contributing

Magika significantly improves over the state of the art, but there's always room for improvement! More work can be done to increase detection accuracy, support for additional content types, bindings for more languages, etc.

This initial release is not targeting polyglot detection, and we're looking forward to seeing adversarial examples from the community.
We would also love to hear from the community about encountered problems, misdetections, features requests, need for support for additional content types, etc.

Check our open GitHub issues to see what is on our roadmap and please report misdetections or feature requests by either opening GitHub issues (preferred) or by emailing us at magika-dev@google.com.

**NOTE: Do NOT send reports about files that may contain PII!**

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for details.

# Additional Resources

- [Google's OSS blog post](https://opensource.googleblog.com/2024/02/magika-ai-powered-fast-and-efficient-file-type-identification.html) about Magika announcement.
- Web demo: [web demo](https://google.github.io/magika/).

# Research Paper and Citation

We describe how we developed Magika and the choices we made in our research paper, which was accepted at the International Conference on Software Engineering (ICSE) 2025. You can find a copy of the paper [here](./assets/2025_icse_magika.pdf). (A previous version of our paper is available on arxiv: [https://arxiv.org/abs/2409.13768](https://arxiv.org/abs/2409.13768).)

If you use this software for your research, please cite it as:

```bibtex
@InProceedings{fratantonio25:magika,
  author = {Yanick Fratantonio and Luca Invernizzi and Loua Farah and Kurt Thomas and Marina Zhang and Ange Albertini and Francois Galilee and Giancarlo Metitieri and Julien Cretin and Alexandre Petit-Bianco and David Tao and Elie Bursztein},
  title = {{Magika: AI-Powered Content-Type Detection}},
  booktitle = {Proceedings of the International Conference on Software Engineering (ICSE)},
  month = {April},
  year = {2025}
}
```

# Security vulnerabilities

Please contact us directly at magika-dev@google.com

# License

Apache 2.0; see [`LICENSE`](LICENSE) for details.

# Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
