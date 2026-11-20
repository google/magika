import plistlib
import struct
import zlib

from magika_datasets.validators.application import dmg


def build(with_crc: bool = True, chunk_end: int | None = None, signature: bytes = b"") -> bytes:
    fork = b"D" * 1000
    mish = bytearray(204)
    mish[0:4] = b"mish"
    struct.pack_into(">I", mish, 200, 2)
    mish += struct.pack(">IIQQQQ", 0x80000005, 0, 0, 1, 0, 512)
    mish += struct.pack(">IIQQQQ", 0x80000005, 0, 1, 1, 512, (chunk_end or 1000) - 512)
    plist = plistlib.dumps(
        {"resource-fork": {"blkx": [{"Data": bytes(mish), "Name": "whole", "ID": "0"}]}},
        fmt=plistlib.FMT_XML,
    )
    trailer = bytearray(512)
    trailer[0:4] = b"koly"
    struct.pack_into(">II", trailer, 4, 4, 512)
    xml_offset = len(fork) + len(signature)
    struct.pack_into(">QQQQQ", trailer, 16, 0, 0, len(fork), 0, 0)
    if with_crc:
        struct.pack_into(">III", trailer, 80, 2, 32, zlib.crc32(fork))
    struct.pack_into(">QQ", trailer, 216, xml_offset, len(plist))
    if signature:
        struct.pack_into(">QQ", trailer, 296, len(fork), len(signature))
    return fork + signature + plist + bytes(trailer)


def test_image_passes_with_crc():
    result = dmg.validate(build(), frozenset())
    assert result.status == "pass" and result.tags == ("data_crc32",)


def test_code_signature_is_accounted():
    result = dmg.validate(build(signature=b"\xfa\xde\x0c\xc0" + bytes(28)), frozenset())
    assert result.status == "pass" and "code_signature" in result.tags


def test_crc_mismatch_chunk_overrun_and_gap_fail():
    data = bytearray(build())
    data[10] ^= 1
    assert dmg.validate(bytes(data), frozenset()).status == "fail"
    assert dmg.validate(build(chunk_end=2000), frozenset()).status == "fail"
    plain = build(with_crc=False)
    assert dmg.validate(plain[:1000] + b"gap" + plain[1000:], frozenset()).status == "fail"
