from pathlib import Path

import pytest

from magika_datasets.validators.archive import unixcompress

FIXTURES = Path(__file__).parent / "fixtures"
HELLO = bytes.fromhex("1f9d9068cab061f30644c081050f1234289020")  # compress(1) of 23 bytes


@pytest.mark.parametrize("name", ["lzw-16bit.Z", "lzw-9bit.Z"])
def test_streams_from_compress_decode_to_the_original_length(name):
    data = (FIXTURES / name).read_bytes()  # both encode the same 5337-byte text
    assert unixcompress.decode(data) == 5337
    result = unixcompress.validate(data, frozenset())
    assert result.status == "pass" and result.generic and "block_mode" in result.tags


def test_small_stream_and_tampering():
    assert unixcompress.decode(HELLO) == 23
    corrupt = bytearray(HELLO)
    corrupt[10] = 0xFF
    result = unixcompress.validate(bytes(corrupt), frozenset())
    assert result.status == "fail" or unixcompress.decode(bytes(corrupt)) != 23


def test_header_bits_are_checked():
    assert unixcompress.validate(b"\x1f\x9d\x88\x00\x00", frozenset()).status == "fail"
    assert unixcompress.validate(b"\x1f\x9d", frozenset()).status == "fail"
