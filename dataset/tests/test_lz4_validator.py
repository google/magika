import struct

from magika_datasets.validators.archive import lz4


def test_xxh32_reference_vectors():
    assert lz4.xxh32(b"") == 0x02CC5D05
    assert lz4.xxh32(b"a") == 0x550D7456
    assert lz4.xxh32(b"abc") == 0x32D153FF
    assert lz4.xxh32(b"Nobody inspects the spammish repetition") == 0xE2293B2F


def frame(
    blocks: list[bytes], block_checksums: bool = False, content_checksum: bool = False
) -> bytes:
    flags = 0x40 | 0x20 | (0x10 if block_checksums else 0) | (0x04 if content_checksum else 0)
    descriptor = bytes([flags, 0x40])
    header = b"\x04\x22\x4d\x18" + descriptor + bytes([(lz4.xxh32(descriptor) >> 8) & 0xFF])
    body = b""
    for payload in blocks:
        body += struct.pack("<I", len(payload) | 0x80000000) + payload
        if block_checksums:
            body += struct.pack("<I", lz4.xxh32(payload))
    return header + body + b"\0\0\0\0" + (b"\0\0\0\0" if content_checksum else b"")


def test_frame_with_checksums_passes():
    result = lz4.validate(frame([b"hello", b"world"], True, True), frozenset())
    assert result.status == "pass" and result.tags == ("block_checksums", "content_checksum")


def test_bad_header_checksum_fails():
    data = bytearray(frame([b"x"]))
    data[6] ^= 0xFF
    assert lz4.validate(bytes(data), frozenset()).status == "fail"


def test_bad_block_checksum_fails():
    data = bytearray(frame([b"payload"], block_checksums=True))
    data[7 + 4 + 7] ^= 1
    assert lz4.validate(bytes(data), frozenset()).status == "fail"


def test_truncated_and_legacy_frames():
    assert lz4.validate(frame([b"abc"])[:-3], frozenset()).status == "fail"
    legacy = b"\x02\x21\x4c\x18" + struct.pack("<I", 3) + b"abc"
    result = lz4.validate(legacy, frozenset())
    assert result.status == "pass" and result.tags == ("legacy_frame",)
