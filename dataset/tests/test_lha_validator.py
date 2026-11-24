import struct

from magika_datasets.validators.archive import lha


def level0(
    name: bytes = b"A.TXT",
    body: bytes = b"stored bytes",
    method: bytes = b"-lh0-",
    crc: int | None = None,
) -> bytes:
    crc = lha.crc16(body) if crc is None else crc
    fields = (
        method
        + struct.pack("<II", len(body), len(body))
        + bytes(4)
        + b"\x20\x00"
        + bytes([len(name)])
        + name
        + struct.pack("<H", crc)
    )
    return bytes([len(fields), sum(fields) & 0xFF]) + fields + body


def level1(body: bytes = b"stored bytes") -> bytes:
    # the base header ends with the first extension size; extensions count toward the packed size
    ext = b"\x01A" + struct.pack("<H", 0)  # size 4: type 1, name, next size 0
    fields = (
        b"-lh0-"
        + struct.pack("<II", len(body) + len(ext), len(body))
        + bytes(4)
        + b"\x20\x01"
        + b"\x00"
        + struct.pack("<H", lha.crc16(body))
        + b"M"
        + struct.pack("<H", len(ext))
    )
    return bytes([len(fields), sum(fields) & 0xFF]) + fields + ext + body


def level2(body: bytes = b"stored bytes", method: bytes = b"-lh0-") -> bytes:
    # extension chain: size 5 -> [type 0, crc(2), next 4] -> [type 1, "A", next 0]
    ext = (
        struct.pack("<H", 5)
        + b"\0"
        + b"\0\0"
        + struct.pack("<H", 4)
        + b"\x01A"
        + struct.pack("<H", 0)
    )
    fields = (
        method
        + struct.pack("<II", len(body), len(body))
        + bytes(4)
        + b"\x20\x02"
        + struct.pack("<H", lha.crc16(body))
        + b"M"
        + ext
    )
    header = bytearray(struct.pack("<H", len(fields) + 2) + fields)
    struct.pack_into("<H", header, 27, lha.crc16(bytes(header)))
    return bytes(header) + body


def test_all_header_levels_pass_with_end_marker():
    result = lha.validate(level0() + level1() + level2() + b"\0", frozenset())
    assert result.status == "pass" and "3 entries" in result.detail and result.tags == ()


def test_missing_end_marker_is_tagged_and_trailing_bytes_fail():
    assert lha.validate(level0(), frozenset()).tags == ("no_end_marker",)
    assert lha.validate(level0() + b"\0\0", frozenset()).status == "fail"


def test_header_checksum_and_stored_crc_are_verified():
    data = bytearray(level0() + b"\0")
    data[1] ^= 1
    assert lha.validate(bytes(data), frozenset()).status == "fail"
    assert lha.validate(level0(crc=0) + b"\0", frozenset()).status == "fail"


def test_level_2_header_crc_is_verified():
    data = bytearray(level2() + b"\0")
    data[33] ^= 1  # the name inside the second extension
    assert lha.validate(bytes(data), frozenset()).status == "fail"


def test_unknown_method_is_not_applicable():
    assert lha.validate(level0(method=b"-xyz-"), frozenset()) is None
