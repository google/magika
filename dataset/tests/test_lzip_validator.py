import lzma
import struct
import zlib

from magika_datasets.validators.archive import lzip


def member(payload: bytes) -> bytes:
    filters = [{"id": lzma.FILTER_LZMA1, "dict_size": 1 << 20, "lc": 3, "lp": 0, "pb": 2}]
    stream = lzma.compress(payload, format=lzma.FORMAT_RAW, filters=filters)
    body = b"LZIP\x01\x14" + stream
    return body + struct.pack("<IQQ", zlib.crc32(payload), len(payload), len(body) + 20)


def test_members_decode_and_trailers_verify():
    data = member(b"hello lzip " * 50) + member(b"second")
    result = lzip.validate(data, frozenset())
    assert result.status == "pass" and "2 members" in result.detail
    corrupt = bytearray(data)
    corrupt[-30] ^= 1
    assert lzip.validate(bytes(corrupt), frozenset()).status == "fail"
    assert lzip.validate(data[:-5], frozenset()).status == "fail"
