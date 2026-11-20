import bz2
import lzma
import zlib

import pytest

from magika_datasets.validators._shared import decompress


def test_drain_reports_length_and_unused_bytes():
    payload = zlib.compress(b"abc" * 1000)
    assert decompress.drain(zlib.decompressobj(), payload + b"rest", 10_000) == (3000, b"rest")
    with pytest.raises(decompress.Truncated):
        decompress.drain(zlib.decompressobj(), payload[:-3], 10_000)
    with pytest.raises(decompress.Budget):
        decompress.drain(zlib.decompressobj(), payload, 2999)


def test_drain_handles_needs_input_style_decompressors():
    payload = bz2.compress(b"xyz" * 500)
    assert decompress.drain(bz2.BZ2Decompressor(), payload, 10_000) == (1500, b"")
    with pytest.raises(decompress.Truncated):
        decompress.drain(bz2.BZ2Decompressor(), payload[:-5], 10_000)


def test_streams_walks_concatenated_members_and_padding():
    one = lzma.compress(b"a" * 10, format=lzma.FORMAT_XZ)
    data = one + b"\x00" * 4 + one

    def factory():
        return lzma.LZMADecompressor(lzma.FORMAT_XZ)

    assert decompress.streams(factory, data, b"\xfd7zXZ\x00", padding=4) == (2, 20)
    with pytest.raises(decompress.Trailing):
        decompress.streams(factory, one + b"junk", b"\xfd7zXZ\x00", padding=4)


def test_inspect_maps_errors_to_observations():
    good = zlib.compress(b"q" * 100)

    def inspect(data):
        return decompress.inspect(
            "zlibstream", zlib.decompressobj, data, b"", zlib.error, concatenated=False
        ).status

    assert inspect(good) == "pass"
    assert inspect(good + b"!") == "fail"
    assert inspect(good[:-2] + b"\x00\x00") == "fail"
    assert inspect(zlib.compress(b"\x00" * (decompress.LIMIT + 1))) == "inconclusive"
