import struct
import zlib

from magika_datasets.validators.archive import arj


def block(file_type: int, name: bytes, packed: int = 0, crc: int = 0, method: int = 0) -> bytes:
    basic = bytes([30, 11, 1, 2, 0, method, file_type, 0]) + struct.pack(
        "<IIIIHHH", 0, packed, packed, crc, 0, 0, 0
    )
    basic += name + b"\0" + b"\0"  # name, empty comment
    return (
        arj.MAGIC
        + struct.pack("<H", len(basic))
        + basic
        + struct.pack("<I", zlib.crc32(basic))
        + b"\0\0"
    )


def archive(*entries: bytes) -> bytes:
    body = block(2, b"test.arj")
    for payload in entries:
        body += block(0, b"a.txt", len(payload), zlib.crc32(payload)) + payload
    return body + arj.MAGIC + b"\0\0"


def test_headers_and_stored_entries_verify():
    result = arj.validate(archive(b"hello", b"world!"), frozenset())
    assert result.status == "pass" and "2 entries" in result.detail and not result.tags


def test_a_corrupt_stored_entry_fails():
    data = bytearray(archive(b"hello"))
    data[-6] ^= 1
    assert arj.validate(bytes(data), frozenset()).status == "fail"


def test_a_corrupt_header_fails():
    data = bytearray(archive(b"hello"))
    data[10] ^= 1
    assert "CRC-32" in arj.validate(bytes(data), frozenset()).detail


def test_a_missing_end_marker_fails():
    assert arj.validate(archive(b"hello")[:-4], frozenset()).status == "fail"


def test_bytes_after_the_end_marker_are_tagged():
    assert arj.validate(archive(b"x") + b"pad", frozenset()).tags == ("trailing_data",)


def test_a_bare_header_id_is_not_an_archive():
    assert arj.validate(b"\x60\xea\0\0\1\0\0\0" + b"\0" * 64, frozenset()).status == "fail"
