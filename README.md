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
> - The documentation on GitHub refers to the latest, potentially unreleased and unstable version of Magika. The latest stable release of the `magika` Python package is `0.6.2`, and you can consult the associated documentation [here](https://github.com/google/magika/blob/python-v0.6.2/python/README.md). You can install the latest stable version with: `pip install magika`.
> - A detailed changelog and migration guidelines from older versions are discussed [here](https://github.com/google/magika/blob/python-v0.6.2/python/CHANGELOG.md).
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

If you intend to use Magika only as a command line, you may want to use one of the alternatives
listed in the [CLI readme](https://github.com/google/magika/tree/main/rust/cli#installation).

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

Magika is available as a command line tool, written in Rust (the `magika` python package ships this client). The output of `magika --help` and the [README](./rust/cli/README.md) document the list of options.

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
| [Python `Magika` module](./python/README.md)                               | Stable         | `0.6.2`        | [`standard_v3_3`](./assets/models/standard_v3_3/README.md) |
| [Rust `magika` CLI](https://crates.io/crates/magika-cli)                   | Stable         | `1.0.0`        | [`standard_v3_3`](./assets/models/standard_v3_3/README.md) |
| [Rust `magika` library](https://docs.rs/magika)                            | Stable         | `0.2.2`        | [`standard_v3_3`](./assets/models/standard_v3_3/README.md) |
| TypeScript / NPM package ([README](./js/README.md) & [docs](./docs/js.md)) | Stable         | `0.4.0`        | [`standard_v3_3`](./assets/models/standard_v3_3/README.md) |
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

We describe how we developed Magika and the choices we made in our research paper, which was accepted at the International Conference on Software Engineering (ICSE) 2025. You can find a copy of the paper [here](./assets/2025_icse_magika.pdf).

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
