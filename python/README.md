# Magika Python Package

[![image](https://img.shields.io/pypi/v/magika.svg)](https://pypi.python.org/pypi/magika)<!-- [![image](https://img.shields.io/pypi/l/magika.svg)](https://pypi.python.org/pypi/magika) -->
[![image](https://img.shields.io/pypi/pyversions/magika.svg)](https://pypi.python.org/pypi/magika)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/google/magika/badge)](https://securityscorecards.dev/viewer/?uri=github.com/google/magika)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/8706/badge)](https://www.bestpractices.dev/en/projects/8706)
![CodeQL](https://github.com/google/magika/workflows/CodeQL/badge.svg)
[![Actions status](https://github.com/google/magika/actions/workflows/python-build-package.yml/badge.svg)](https://github.com/google/magika/actions)
[![PyPI Monthly Downloads](https://img.shields.io/pypi/dm/magika)](https://pypi.org/project/magika/)


Magika is a novel AI-powered file type detection tool that relies on the recent advance of deep learning to provide accurate detection. Under the hood, Magika employs a custom, highly optimized model that only weighs about a few MBs, and enables precise file identification within milliseconds, even when running on a single CPU. Magika has been trained and evaluated on a dataset of ~100M samples across 200+ content types (covering both binary and textual file formats), and it achieves an average ~99% accuracy on our test set.

Use Magika as a command line client or in your Python code!

You can find more information on which content types are supported, extended documentation, and bindings for other languages on our GitHub project at [https://github.com/google/magika](https://github.com/google/magika).

> The `magika` Python package is suitable for production use. However, because it's currently in its zero major version (`0.x.y`), future `0.x+1.z` updates may include breaking changes (more in general, Magika adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)). For detailed information and migration guidance, please refer to the [`CHANGELOG.md`](https://github.com/google/magika/blob/main/python/CHANGELOG.md).

> **IMPORTANT**: This latest 0.6.1 version has a few breaking changes from the latest stable version, 0.5.1. Please consult the [CHANGELOG.md](https://github.com/google/magika/blob/main/python/CHANGELOG.md#061---2025-03-19) and the [migration guide](https://github.com/google/magika/blob/main/python/CHANGELOG.md#breaking-changes-and-migration-guide).


## Installing Magika

Magika is available as `magika` on [PyPI](https://pypi.org/project/magika):

To install the most recent stable version:
```shell
$ pip install magika
```

If you intend to use Magika only as a command line, you may want to use `$ pipx install magika` instead.

If you want to test out the latest release candidate, you can install it with `pip install --pre magika`.


## Using Magika as a command-line tool

> Beginning with version `0.6.0`, the magika Python package includes a pre-compiled Rust-based command-line tool, replacing the previous Python version. This binary is distributed as platform-specific wheels for most common architectures. For unsupported platforms, a pure-Python wheel is also available, providing the legacy Python client as a fallback.

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


## Using Magika as a Python module

> Note: The Python API introduced in version `0.6.0` closely resembles the previous version, but includes several enhancements and a few breaking changes. Migrating existing clients should be relatively straightforward. Where possible, we have maintained compatibility with the old API and added deprecation warnings. For a complete list of changes and migration guidance, consult the [CHANGELOG.md](https://github.com/google/magika/blob/main/python/CHANGELOG.md).

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


## Documentation on core concepts

To get the most out of Magika, it's worth learning about its core concepts. You can read about the models, prediction modes, output structure, and content type knowledge base in the documentation [here](https://github.com/google/magika/blob/main/docs/concepts.md).


### API documentation

First, create a `Magika` instance: `magika = Magika()`.

The constructor accepts the following optional arguments:
- `model_dir`: path to a model to use; defaults to the latest available model.
- `prediction_mode`: which prediction mode to use; defaults to `PredictionMode.HIGH_CONFIDENCE`.
- `no_dereference`: controls whether symlinks should be dereferenced; defaults to `False`.

Once instantiated, the `Magika` object exposes methods to identify the content type of a `bytes` object, of files identified by their paths, and of an already-open binary stream:
- `magika.identify_bytes(b"test")`: takes as input a stream of bytes and predict its content type.
- `magika.identify_path("test.txt")`: takes as input one `str | os.PathLike` object and predicts its content type.
- `magika.identify_paths(["test.txt", "test2.txt"])`: takes as input a list of `str | os.PathLike` objects and returns the predicted type for each of them.
- `magika.identify_stream(stream: typing.BinaryIO)`: takes as input an *already open* binary file-like object (e.g., the output of `open(file_path, 'rb')`) and returns its predicted content type. Keep in mind that Magika will `seek()` around the stream, and that the stream *is not closed* (closing is the responsibility of the caller).

If you are dealing with large files, the `identify_path`, `identify_paths`, and `identify_stream` variants are generally better: their implementation `seek()`s around the file/stream to extract the needed features, without loading the entire content in memory.

These API returns an object of type [`MagikaResult`](https://github.com/google/magika/blob/main/python/src/magika/types/magika_result.py), an [`absl::StatusOr`](https://abseil.io/docs/cpp/guides/status)-like wrapper around [`MagikaPrediction`](https://github.com/google/magika/blob/main/python/src/magika/types/magika_prediction.py), which exposes the same information discussed in the [Magika's output documentation](https://github.com/google/magika/blob/main/docs/concepts.md).

Here is how the main types look like:

```python
class MagikaResult:
    path: Path
    ok: bool
    status: Status
    prediction: MagikaPrediction
    dl: ContentTypeInfo  # Shortcut for `prediction.dl`, valid only for `status == Status.OK`
    output: ContentTypeInfo  # Same as above, shortcut to `prediction.output`
    score: float  # Same as above, shortcut to `prediction.float`
```

```python
class MagikaPrediction:
    dl: ContentTypeInfo
    output: ContentTypeInfo
    score: float
    # Specify why the model's output has been overwritten (if that's the case)
    overwrite_reason: OverwriteReason
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


### Additional APIs

- `get_output_content_types()`: Returns a list of all possible content type labels that Magika can output (i.e., the possible values of `MagikaResult.prediction.output.label`). This is the recommended method for most users that want to have a list of what is the output space of Magika.
- `get_model_content_types()`: Returns a list of all possible content type labels the *deep learning model* can output (i.e., `MagikaResult.prediction.dl.label`). Useful for debugging, most users should refer to `get_output_content_types()`.
- `get_module_version()` and `get_model_version()`: Returns the module version and the model's name being used, respectively.


## Development setup

- `magika` uses `uv` as a project and dependency managment tool. To install all the dependencies: `$ cd python; uv sync`.
- To run the tests suite: `$ cd python; uv run pytest tests -m "not slow"`. Check the github action workflows for more information.
- We use the `maturin` backend to combine the Rust CLI with the python codebase in the `magika` python package. This process is automated via the [build python package GitHub action](https://github.com/google/magika/blob/main/.github/workflows/python-build-package.yml).


## Research Paper and Citation
We describe how we developed Magika and the choices we made in our research paper, which was accepted at the International Conference on Software Engineering (ICSE) 2025. A pre-print of our paper is available on arxiv: [https://arxiv.org/abs/2409.13768](https://arxiv.org/abs/2409.13768).

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
