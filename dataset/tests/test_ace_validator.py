import struct

from magika_datasets.validators.archive import ace


def block(kind: int, flags: int, body: bytes, payload: bytes = b"") -> bytes:
    header = bytes([kind]) + struct.pack("<H", flags) + body
    return struct.pack("<HH", ace.crc16(header), len(header)) + header + payload


def archive(payload: bytes = b"packed") -> bytes:
    main = block(0, 0x1000, b"**ACE**" + bytes(30))
    file = block(1, 0x8001 | 0x4000, struct.pack("<II", len(payload), 12) + bytes(20), payload)
    return main + file


def test_archive_passes_with_encrypted_tag():
    result = ace.validate(archive(), frozenset())
    assert result.status == "pass" and result.tags == ("encrypted",)


def test_header_crc_mismatch_fails():
    data = bytearray(archive())
    data[0] ^= 1  # stored CRC of the main header

    assert ace.validate(bytes(data), frozenset()).status == "fail"


def test_truncated_payload_fails():
    assert ace.validate(archive()[:-2], frozenset()).status == "fail"
