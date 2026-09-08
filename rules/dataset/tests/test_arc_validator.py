import struct

from magika_datasets.validators.archive import arc


def entry(name: bytes, body: bytes, method: int = 2) -> bytes:
    header = (
        bytes([0x1A, method])
        + name.ljust(13, b"\0")
        + struct.pack("<IHHH", len(body), 0, 0, arc.crc16(body))
    )
    if method != 1:
        header += struct.pack("<I", len(body))
    return header + body


def test_archive_passes_and_crc_is_verified():
    data = entry(b"A.TXT", b"hello") + entry(b"B.TXT", b"world", method=1) + b"\x1a\x00"
    result = arc.validate(data, frozenset())
    assert result.status == "pass" and "2 stored" in result.detail
    corrupt = bytearray(data)
    corrupt[30] ^= 1
    assert arc.validate(bytes(corrupt), frozenset()).status == "fail"
    assert arc.validate(data[:-2], frozenset()).status == "fail"
