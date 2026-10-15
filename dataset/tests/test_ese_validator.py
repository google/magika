import struct

from magika_datasets.validators.data import ese


def build(page_size: int = 4096, pages: int = 4) -> bytes:
    head = bytearray(page_size)
    struct.pack_into("<III", head, 4, ese.SIGNATURE, 0x620, 0)
    struct.pack_into("<I", head, 236, page_size)
    checksum = ese.SIGNATURE
    for (word,) in struct.iter_unpack("<I", bytes(head[4:])):
        checksum ^= word
    struct.pack_into("<I", head, 0, checksum)
    return bytes(head) * 2 + bytes(page_size * (pages - 2))


def test_database_passes():
    result = ese.validate(build(), frozenset())
    assert result.status == "pass" and "4 pages" in result.detail


def test_checksum_shadow_and_size_checks():
    data = bytearray(build())
    data[300] ^= 1
    assert ese.validate(bytes(data), frozenset()).status == "fail"
    data = bytearray(build())
    data[4096 + 300] ^= 1
    assert ese.validate(bytes(data), frozenset()).status == "inconclusive"
    assert ese.validate(build() + b"\0", frozenset()).status == "fail"
