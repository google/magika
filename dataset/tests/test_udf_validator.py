import struct

from magika_datasets.validators.archive import udf


def tagged(identifier: int, body: bytes) -> bytes:
    head = bytearray(struct.pack("<HH", identifier, 2)) + bytes(12)
    head += body
    head[4] = (sum(head[:4]) + sum(head[5:16])) & 0xFF
    return bytes(head).ljust(2048, b"\0")


def build() -> bytes:
    data = bytearray(300 * 2048)
    for index, ident in enumerate((b"BEA01", b"NSR02", b"TEA01")):
        data[(16 + index) * 2048 : (16 + index) * 2048 + 6] = b"\0" + ident
    anchor = tagged(2, struct.pack("<IIII", 3 * 2048, 32, 0, 0))
    data[256 * 2048 : 257 * 2048] = anchor
    for index, ident in enumerate((1, 5, 6, 8)):
        data[(32 + index) * 2048 : (33 + index) * 2048] = tagged(ident, b"")
    return bytes(data)


def test_filesystem_passes_and_bad_tag_fails():
    assert udf.validate(build(), frozenset()).status == "pass"
    data = bytearray(build())
    data[33 * 2048 + 6] ^= 1  # inside a descriptor tag
    assert udf.validate(bytes(data), frozenset()).status == "fail"
