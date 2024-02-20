# Python Package Documentation

## Installation

Magika is available as `magika` on [PyPI](https://pypi.org/project/magika):

```shell
$ pip install magika
```

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

If you are dealing with big files, the `identify_path` and `identify_paths` variants are generally better: their implementation `seek()` around the file to extract the needed features, without loading the entire content in memory.

These API returns an object of type [`MagikaResult`](./magika/types.py), which exposes the same information discussed above in the "Detailed output format" section.
