import struct
import zlib

from magika_datasets.validators.geometry import rhino


def chunk(code: int, body: bytes, width: int) -> bytes:
    size = struct.pack("<Q" if width == 8 else "<I", len(body))
    return struct.pack("<I", code) + size + body


def archive(version: bytes = b"      70", corrupt: bool = False) -> bytes:
    width = 8 if int(version) >= 50 else 4
    payload = b"table data"
    crc = zlib.crc32(payload) ^ (1 if corrupt else 0)
    body = rhino.MAGIC + version
    body += chunk(rhino.COMMENT_BLOCK, b"written by a test", width)
    body += struct.pack("<I", 0x80000001) + (b"\0" * width)  # short chunk, value only
    body += chunk(0x10000000 | rhino.CRC, payload + struct.pack("<I", crc), width)
    total = len(body) + 4 + width + width
    return body + chunk(rhino.END_OF_FILE, struct.pack("<Q" if width == 8 else "<I", total), width)


def test_version_7_chunks_tile_the_file():
    result = rhino.validate(archive(), frozenset())
    assert result.status == "pass" and result.detail.startswith("Version 7 archive; 4 top-level")


def test_version_4_uses_32_bit_lengths():
    assert rhino.validate(archive(b"       4"), frozenset()).status == "pass"


def test_a_chunk_crc_mismatch_fails():
    assert "CRC-32" in rhino.validate(archive(corrupt=True), frozenset()).detail


def test_truncation_and_appended_bytes_fail():
    data = archive()
    assert rhino.validate(data[:-3], frozenset()).status == "fail"
    assert rhino.validate(data + b"\0" * 16, frozenset()).status == "fail"


def test_a_rhino_stl_export_is_not_claimed():
    assert rhino.validate(b"Rhinoceros Binary STL ( Sep 23 2019 )\r\n", frozenset()) is None
