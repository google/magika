import hashlib
import struct
import zlib

from helpers import status

from magika_datasets.validators.executable import dex, javabytecode


def dex_file(corrupt=None):
    header = bytearray(b"dex\n035\0" + b"\0" * (0x70 - 8))
    map_offset = 0x70
    struct.pack_into("<I", header, 36, 0x70)  # header size
    struct.pack_into("<I", header, 40, 0x12345678)  # endian tag
    struct.pack_into("<I", header, 52, map_offset)
    struct.pack_into("<II", header, 56, 1, map_offset + 16)  # one string id at offset
    body = struct.pack("<I", 1) + struct.pack(
        "<HHII", 0x2000, 0, 1, map_offset + 16 + 4
    )  # map list: 1 item
    body += struct.pack("<I", map_offset + 16 + 4)  # string_id -> data offset
    body += b"\x03abc\0" + b"\0" * 6
    data = bytearray(bytes(header) + body)
    struct.pack_into("<I", data, 32, len(data))
    data[12:32] = hashlib.sha1(data[32:]).digest()
    struct.pack_into("<I", data, 8, zlib.adler32(bytes(data[12:])))
    if corrupt == "adler":
        data[8] ^= 1
    if corrupt == "sha":
        data[12] ^= 1
    if corrupt == "map":
        struct.pack_into("<I", data, 52, 0xFFFFFF)
    return bytes(data)


def test_dex_checksums_and_offsets():
    observation = dex.validate(dex_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "dex")
    assert status(dex, dex_file(corrupt="adler")) == "fail"
    assert status(dex, dex_file(corrupt="sha")) == "fail"
    assert status(dex, dex_file(corrupt="map")) == "fail"
    assert status(dex, dex_file()[:-4]) == "fail"
    assert status(dex, dex_file() + b"\0") == "fail"
    assert status(dex, b"dex\n999\0" + b"\0" * 200) == "fail"
    assert status(dex, b"dey\n035\0" + b"\0" * 200) == "not_applicable"


def class_file(trailing=b"", pool=None):
    pool = (
        pool
        if pool is not None
        else [
            b"\x07" + struct.pack(">H", 2),  # Class -> #2
            b"\x01" + struct.pack(">H", 5) + b"Hello",  # Utf8
            b"\x05" + struct.pack(">Q", 42),  # Long (two slots)
            None,
            b"\x01" + struct.pack(">H", 4) + b"Code",
        ]
    )
    entries = b"".join(e for e in pool if e is not None)
    data = (
        b"\xca\xfe\xba\xbe" + struct.pack(">HH", 0, 52) + struct.pack(">H", len(pool) + 1) + entries
    )
    data += struct.pack(">HHH", 0x21, 1, 0) + struct.pack(
        ">H", 0
    )  # access, this, super, interfaces
    data += struct.pack(">H", 0)  # fields
    data += (
        struct.pack(">H", 1)
        + struct.pack(">HHHH", 1, 5, 5, 1)
        + struct.pack(">HI", 5, 4)
        + b"\0\0\0\0"
    )  # one method with a 4-byte attribute
    data += struct.pack(">H", 1) + struct.pack(">HI", 5, 2) + b"\0\0"  # class attribute
    return data + trailing


def test_java_class_constant_pool_and_members():
    observation = javabytecode.validate(class_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "javabytecode")
    assert "java_major_52" in observation.tags
    assert status(javabytecode, class_file()[:-3]) == "fail"
    assert status(javabytecode, class_file(trailing=b"\0")) == "fail"
    assert status(javabytecode, class_file(pool=[b"\x63" + b"\0" * 4])) == "fail"
    assert (
        status(javabytecode, b"\xca\xfe\xba\xbe" + struct.pack(">HH", 0, 200) + b"\0" * 10)
        == "fail"
    )
    assert (
        status(javabytecode, b"\xca\xfe\xba\xbe" + struct.pack(">II", 2, 0x0100000C) + b"\0" * 40)
        == "not_applicable"
    )
