import struct
import zlib

from magika_datasets.validators.archive import rar, sevenzip


def seven(
    header: bytes = b"\x01\x04\x06\x00\x01\x09\x05\x00\x07\x0b\x01\x00\x01\x00\x0c\x05\x00\x08\x0a\x01\xd8\x1d\x1e\x8f\x00\x00\x05\x01\x11\x07\x00\x61\x00\x00\x00\x00\x00",
    packed: bytes = b"\x00" * 5,
) -> bytes:
    start = struct.pack("<QQI", len(packed), len(header), zlib.crc32(header))
    return (
        b"7z\xbc\xaf\x27\x1c\x00\x04"
        + struct.pack("<I", zlib.crc32(start))
        + start
        + packed
        + header
    )


def test_sevenzip_passes_and_fails_on_crc():
    assert sevenzip.validate(seven(), frozenset()).status == "pass"
    data = bytearray(seven())
    data[-1] ^= 1
    assert sevenzip.validate(bytes(data), frozenset()).status == "fail"
    assert sevenzip.validate(seven() + b"x", frozenset()).status == "fail"


def test_sevenzip_encoded_header_is_tagged():
    result = sevenzip.validate(seven(header=b"\x17\x06\x01\x00"), frozenset())
    assert result.status == "pass" and result.tags == ("encoded_header",)


def block5(kind: int, flags: int, extra: bytes = b"", payload: bytes = b"") -> bytes:
    body = bytes([kind, flags]) + (bytes([len(payload)]) if flags & 2 else b"") + extra
    header = bytes([len(body)]) + body
    return struct.pack("<I", zlib.crc32(header)) + header + payload


def archive5(*blocks: bytes) -> bytes:
    return b"Rar!\x1a\x07\x01\x00" + b"".join(blocks)


def test_rar5_chain_passes_and_trailing_bytes_fail():
    data = archive5(
        block5(1, 0, b"\x00"), block5(2, 2, b"\x00" * 6, b"payload"), block5(5, 0, b"\x00")
    )
    assert rar.validate(data, frozenset()).status == "pass"
    assert rar.validate(data + b"\0", frozenset()).status == "fail"
    assert rar.validate(data[:-3], frozenset()).status == "fail"


def test_rar5_header_crc_mismatch_fails():
    data = bytearray(archive5(block5(1, 0, b"\x00"), block5(5, 0, b"\x00")))
    data[8] ^= 1
    assert rar.validate(bytes(data), frozenset()).status == "fail"


def block4(kind: int, flags: int, body: bytes = b"", payload: bytes = b"") -> bytes:
    size = 7 + (4 if flags & 0x8000 else 0) + len(body)
    rest = bytes([kind]) + struct.pack("<HH", flags, size)
    if flags & 0x8000:
        rest += struct.pack("<I", len(payload))
    rest += body
    return struct.pack("<H", zlib.crc32(rest) & 0xFFFF) + rest + payload


def test_rar4_chain_passes_and_marks_encrypted_entries():
    data = (
        b"Rar!\x1a\x07\x00"
        + block4(0x73, 0x08, b"\x00" * 6)
        + block4(0x74, 0x8004, b"\x00" * 21, b"abc")
        + block4(0x7B, 0x4000)
    )
    result = rar.validate(data, frozenset())
    assert result.status == "pass" and result.tags == ("encrypted", "solid")
    assert rar.validate(data[:-1], frozenset()).status == "fail"


def test_encrypted_headers_are_inconclusive():
    data = b"Rar!\x1a\x07\x00" + block4(0x73, 0x80, b"\x00" * 6) + b"garbage" * 4
    assert rar.validate(data, frozenset()).status == "inconclusive"
    data = archive5(block5(4, 0, b"\x00\x0f\x00" + b"\x00" * 24), b"garbage" * 4)
    assert rar.validate(data, frozenset()).status == "inconclusive"
