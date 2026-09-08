import struct
import zlib

from magika_datasets.validators.data import cram


def itf8(value: int) -> bytes:
    assert value < 0x80
    return bytes([value])


def container(payload: bytes) -> bytes:
    header = (
        struct.pack("<i", len(payload))
        + itf8(0) * 3
        + itf8(1)
        + b"\x00"
        + b"\x00"
        + itf8(1)
        + itf8(1)
        + itf8(0)
    )
    return header + struct.pack("<I", zlib.crc32(header)) + payload


def build() -> bytes:
    return b"CRAM\x03\x00" + b"id".ljust(20, b"\0") + container(b"header-block") + container(b"eof")


def test_containers_pass_and_crc_and_truncation_are_checked():
    assert cram.validate(build(), frozenset()).status == "pass"
    data = bytearray(build())
    data[30] ^= 1
    assert cram.validate(bytes(data), frozenset()).status == "fail"
    assert cram.validate(build()[:-1], frozenset()).status == "fail"
