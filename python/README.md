# Magika Python Package

[![image](https://img.shields.io/pypi/v/magika.svg)](https://pypi.python.org/pypi/magika)<!-- [![image](https://img.shields.io/pypi/l/magika.svg)](https://pypi.python.org/pypi/magika) -->
[![image](https://img.shields.io/pypi/pyversions/magika.svg)](https://pypi.python.org/pypi/magika)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/google/magika/badge)](https://securityscorecards.dev/viewer/?uri=github.com/google/magika)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/8706/badge)](https://www.bestpractices.dev/en/projects/8706)
![CodeQL](https://github.com/google/magika/workflows/CodeQL/badge.svg)
[![Actions status](https://github.com/google/magika/actions/workflows/python-build-package.yml/badge.svg)](https://github.com/google/magika/actions)
[![PyPI Monthly Downloads](https://img.shields.io/pypi/dm/magika)](https://pypi.org/project/magika/)


Magika is a novel AI powered file type detection tool that rely on the recent advance of deep learning to provide accurate detection. Under the hood, Magika employs a custom, highly optimized Keras model that only weighs about 1MB, and enables precise file identification within milliseconds, even when running on a single CPU.

Use Magika as a command line client or in your Python code!

Please check out Magika on GitHub for more information and documentation: [https://github.com/google/magika](https://github.com/google/magika).

> [!WARNING]
> This README is about the soon-to-be released `magika 0.6.0` (currently released as `0.6.0rc2` for testing). For older versions, browse the git repository at the latest stable release, [here](https://github.com/google/magika/blob/python-v0.5.1/python/README.md) and [here](https://github.com/google/magika/blob/python-v0.5.1/docs/python.md).
>
> See [`CHANGELOG.md`](https://github.com/google/magika/blob/main/python/CHANGELOG.md) for more details.


## Installing Magika

Magika is available as `magika` on [PyPI](https://pypi.org/project/magika):

To install the most recent stable version:
```shell
$ pip install magika
```

If you intend to use Magika only as a command line, you may want to use `$ pipx install magika` instead.


To install a specific, possibly unstable version published as a release candidate:

```shell
$ pip install magika==0.6.0rc1
```


## Using Magika as a command-line tool

Starting from magika `0.6.0`, the python package ships the new CLI, written in Rust (which replaces the old one written in python).

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


Check the [Rust CLI docs](https://github.com/google/magika/blob/main/rust/cli/README.md) for more information.

Check the [docs on Magika's output](https://github.com/google/magika/blob/main/docs/magika_output.md) for more details about the output format.


## Using Magika as a Python module

> [!WARNING] The new API is very similar to the old one, but it ships with a number of improvements and introduces a few breaking changes. Updating existing clients should be fairly straighforward, and, where we could, we kept support for the old API and added deprecation warnings. See the [CHANGELOG.md](https://github.com/google/magika/blob/main/python/CHANGELOG.md) for the full list of changes and suggestions on how to fix.

```python
>>> from magika import Magika
>>> m = Magika()
>>> res = m.identify_bytes(b"# Example\nThis is an example of markdown!")
>>> print(res.output.label)
markdown
```


### API documentation

First, create a `Magika` instance: `magika = Magika()`.

The `Magika` object exposes three methods:
- `magika.identify_bytes(b"test")`: takes as input a stream of bytes and predict its content type.
- `magika.identify_path(Path("test.txt"))`: takes as input one `Path` object and predicts its content type.
- `magika.identify_paths([Path("test.txt"), Path("test2.txt")])`: takes as input a list of `Path` objects and returns the predicted type for each of them.

If you are dealing with big files, the `identify_path` and `identify_paths` variants are generally better: their implementation `seek()`s around the file to extract the needed features, without loading the entire content in memory.

These API returns an object of type [`MagikaResult`](https://github.com/google/magika/blob/main/python/src/magika/types/magika_result.py), an [`absl::StatusOr`](https://abseil.io/docs/cpp/guides/status)-like wrapper around [`MagikaPrediction`](https://github.com/google/magika/blob/main/python/src/magika/types/magika_prediction.py), which exposes the same information discussed in the [Magika's output documentation](https://github.com/google/magika/blob/main/docs/magika_output.md).

Here is how the main types look like:

```python
class MagikaResult:
    path: Path
    status: Status
    prediction: MagikaPrediction
    [...]
```

```python
class MagikaPrediction:
    dl: ContentTypeInfo
    output: ContentTypeInfo
    score: float
```

```python
class ContentTypeInfo:
    label: ContentTypeLabel
    mime_type: str
    group: str
    description: str
    extensions: List[str]
    is_text: bool
```

```python
class ContentTypeLabel(StrEnum):
    APK = "apk"
    BMP = "bmp"
    [...]
```


### Development setup

- `magika` uses `uv` as a project and dependency managment tool. To install all the dependencies: `$ cd python; uv sync`.
- To run the tests suite: `$ cd python; uv run pytest tests -m "not slow"`. Check the github action workflows for more information.
- We use the `maturin` backend to combine the Rust CLI with the python codebase. To build: `$ cd python; uv run ./scripts/build_python_package.py`.


## Citation
If you use this software for your research, please cite it as:
```bibtex
@misc{magika,
      title={{Magika: AI-Powered Content-Type Detection}},
      author={{Fratantonio, Yanick and Invernizzi, Luca and Farah, Loua and Kurt, Thomas and Zhang, Marina and Albertini, Ange and Galilee, Francois and Metitieri, Giancarlo and Cretin, Julien and Petit-Bianco, Alexandre and Tao, David and Bursztein, Elie}},
      year={2024},
      eprint={2409.13768},
      archivePrefix={arXiv},
      primaryClass={cs.CR},
      url={https://arxiv.org/abs/2409.13768},
}
```

> [!NOTE]
> The Magika paper was accepted at IEEE/ACM International Conference on Software Engineering (ICSE) 2025!