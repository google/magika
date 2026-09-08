import struct

from magika_datasets.validators.data import cdf


def record(kind: int, body: bytes, width: int = 32) -> bytes:
    fmt = ">Q" if width == 64 else ">I"
    size = struct.calcsize(fmt) + 4 + len(body)
    return struct.pack(fmt, size) + struct.pack(">i", kind) + body


def test_v2_and_v3_records_tile():
    v2 = b"\xcd\xf2\x60\x02\x00\x00\xff\xff" + record(1, bytes(20)) + record(2, bytes(8))
    assert cdf.validate(v2, frozenset()).status == "pass"
    v3 = b"\xcd\xf3\x00\x01\x00\x00\xff\xff" + record(1, bytes(20), 64) + record(2, bytes(8), 64)
    assert cdf.validate(v3, frozenset()).status == "pass"
    assert cdf.validate(v2 + b"\0", frozenset()).status == "fail"
    assert cdf.validate(v2[:-3], frozenset()).status == "fail"
