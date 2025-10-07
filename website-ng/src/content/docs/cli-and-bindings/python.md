---
title: "Python `Magika` Module"
---

This guide provides documentation on how to use the `magika` Python module to identify file types from your code.

:::tip
This section assumes you are familiar with the topics discussed in the [Core Concepts](/core-concepts/) section.
:::

## Quick Examples

The `magika` API is designed to be simple and intuitive. The following examples cover the most common use cases for identifying content from bytes, paths, and streams.

**From bytes:**
```python
>>> from magika import Magika
>>> m = Magika()
>>> res = m.identify_bytes(b'function log(msg) {console.log(msg);}')
>>> print(res.output.label)
javascript
```

**From a file path:**
```python
>>> from magika import Magika
>>> m = Magika()
>>> res = m.identify_path('./tests_data/basic/ini/doc.ini')
>>> print(res.output.label)
ini
```

**From an open file stream:**
```python
>>> from magika import Magika
>>> m = Magika()
>>> with open('./tests_data/basic/ini/doc.ini', 'rb') as f:
>>>     res = m.identify_stream(f)
>>> print(res.output.label)
ini
```

## API Reference

### Instantiating `Magika`

First, create an instance of the `Magika` class. The constructor accepts several optional arguments to customize its behavior.

```python
from magika import Magika, PredictionMode

# Default instantiation
magika = Magika()

# Custom instantiation
magika_custom = Magika(
    model_dir="/path/to/custom/model",
    prediction_mode=PredictionMode.BEST_GUESS,
    no_dereference=True,
)
```

**Constructor Arguments:**
- `model_dir` (`Path`, optional): Path to a directory containing a custom model. If not provided, defaults to the latest bundled model.
- `prediction_mode` (`PredictionMode`, optional): The prediction mode to use. Defaults to `PredictionMode.HIGH_CONFIDENCE`.
- `no_dereference` (`bool`, optional): If `True`, symbolic links will not be followed; their content type will be reported as `symlink`. Defaults to `False`.

**Identifying Content**

Once instantiated, the `Magika` object provides several methods for identifying content from different sources.
- `magika.identify_bytes(bytes)`: Identifies the content type of an in-memory bytes object.
- `magika.identify_path(path)`: Identifies the content type of a single file from its path (`str | os.PathLike`).
- `magika.identify_paths(paths)`: Identifies the content type for a list of file paths.
- `magika.identify_stream(stream)`: Identifies the content type from an already-open binary file-like object (e.g., the output of `open(file_path, 'rb')`). Note: 1) Magika will `seek()` around the stream; 2) the stream _is not closed_ (closing is the responsibility of the caller).

If you are dealing with large files, the `identify_path`, `identify_paths`, and `identify_stream` variants are generally better: their implementation `seek()`s around the file/stream to extract the needed features, without loading the entire content in memory.

:::tip[Performance with Large Files]
For large files, the `identify_path`, `identify_paths`, and `identify_stream` methods are highly recommended. They are optimized to read only the necessary portions of the file by seeking within the file/stream, which avoids loading the entire content into memory. If your content is already loaded into a bytes object, `identify_bytes` is the most direct and efficient option.
:::

**Understanding the Result**

All `identify_*` methods return a `MagikaResult` object. This object acts as a wrapper that contains the prediction details and the status of the operation. **You should always check if the operation was successful before accessing the prediction.**

```python
>>> result = m.identify_path("path/to/file")
>>> if result.ok:
...     print(f"File is a {result.output.description}")
...     print(f"MIME Type: {result.output.mime_type}")
... else:
...     print(f"Error: {result.status.message}")
```


## Data Models

The `MagikaResult` object and its nested data classes provide detailed information about the scan.

Consult the [Understanding the Output](/core-concepts/understanding-the-output) section for more context.

`MagikaResult`

```python
class MagikaResult:
    path: Path
    ok: bool
    status: Status
    prediction: MagikaPrediction
    # Shortcuts available only when result.ok is True
    dl: ContentTypeInfo
    output: ContentTypeInfo
    score: float
```

- `ok` (bool): `True` if the identification was successful, `False` otherwise.
- `status` (Status): Provides details on an error if `ok` is `False`.
- `prediction` (`MagikaPrediction`): The core prediction object, available only if `ok` is `True`.
- `dl`, `output`, `score`: For convenience, these are direct shortcuts to the corresponding fields within the `prediction` object.


`MagikaPrediction`

Contains the core deep learning model prediction and the final Magika output.

```python
class MagikaPrediction:
    dl: ContentTypeInfo
    output: ContentTypeInfo
    score: float
    overwrite_reason: OverwriteReason
```

- `dl` (`ContentTypeInfo`): The raw prediction from the deep learning model.
- `output` (`ContentTypeInfo`): The final prediction from "Magika the tool," which considers the model's prediction, its confidence score, and the selected prediction mode. **This is the result most users should rely on.**
- `score` (`float`): The model's confidence score (from 0.0 to 1.0).
- `overwrite_reason` (`OverwriteReason`): It indicates why the deep learning model's prediction was overwritten (e.g., low confidence).

`ContentTypeInfo`

Contains detailed metadata about a predicted content type.

```python
class ContentTypeInfo:
    label: ContentTypeLabel # e.g., "python"
    mime_type: str          # e.g., "text/x-python"
    group: str              # e.g., "code"
    description: str        # e.g., "Python source"
    extensions: List[str]   # e.g., ["py", "pyc"]
    is_text: bool           # e.g., True
```

`ContentTypeLabel`

A string enum (`StrEnum`) of all possible content type labels. Because it's a `StrEnum`, its members can be used and compared just like regular strings.

```python
class ContentTypeLabel(StrEnum):
    APK = "apk"
    BMP = "bmp"
    # ... and many more
```

:::caution
**`ContentTypeLabel` is a superset of supported types.**

This enum is generated from our internal Content Types Knowledge Base and includes many types that the default model may not be trained to detect.

The presence of a label in this enum **does not guarantee** it can be a prediction result. To get the definitive list of possible output labels that "Magika the tool" can return, use the `magika.get_output_content_types()` method, discussed next.
:::


## Additional APIs

The `Magika` class also exposes a few helper methods:
- `get_output_content_types()`: Returns a list of all possible content type labels that Magika can return in the `output.label` field. This is the recommended way to get a definitive list of Magika's possible outputs.
- `get_model_content_types()`: Returns a list of all possible content type labels the _deep learning model_ can return (i.e., the possible values for `dl.label`, in addition to `undefined`). This is useful for debugging.
- `get_module_version()`: Returns the `magika` Python package version as a string.
- `get_model_version()`: Returns the name of the model being used as a string.


## Development setup

This section is for contributors to the `magika` Python package.

- **Project Management:** `magika` uses `uv` for dependency management. To install all development dependencies, run: `cd python; uv sync`.

- **Testing:** To run the test suite, use `pytest`. You can exclude slow tests for faster runs: `cd python; uv run pytest tests -m "not slow"`. Refer to the GitHub Actions workflows for more testing examples.

- **Packaging:** We use `maturin` to build the Python package, which combines the Rust-based CLI with the Python source code. This process is automated in our [Build Python Package GitHub Action](https://github.com/google/magika/blob/main/.github/workflows/python-build-package.yml).
