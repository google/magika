import struct
import zlib

from magika_datasets.validators.data import bam


def raw_bam(records: int = 2) -> bytes:
    text = b"@HD\tVN:1.5\n"
    out = b"BAM\x01" + struct.pack("<I", len(text)) + text + struct.pack("<I", 1)
    out += struct.pack("<I", 5) + b"chr1\0" + struct.pack("<I", 1000)
    for _ in range(records):
        body = bytes(36)
        out += struct.pack("<I", len(body)) + body
    return out


def bgzf(payload: bytes) -> bytes:
    compressor = zlib.compressobj(6, zlib.DEFLATED, -15)
    deflated = compressor.compress(payload) + compressor.flush()
    size = 18 + len(deflated) + 8
    header = (
        b"\x1f\x8b\x08\x04\0\0\0\0\0\xff"
        + struct.pack("<H", 6)
        + b"BC"
        + struct.pack("<HH", 2, size - 1)
    )
    return header + deflated + struct.pack("<II", zlib.crc32(payload), len(payload))


def test_uncompressed_bam():
    assert bam.validate(raw_bam(), frozenset()).tags == ("uncompressed",)
    assert bam.validate(raw_bam() + b"\0\0", frozenset()).status == "fail"


def test_bgzf_blocks():
    data = bgzf(raw_bam()[:20]) + bgzf(raw_bam()[20:]) + bam.EOF_BLOCK
    result = bam.validate(data, frozenset())
    assert result.status == "pass" and result.tags == ("eof_marker",)
    corrupt = bytearray(data)
    corrupt[25] ^= 1
    assert bam.validate(bytes(corrupt), frozenset()).status == "fail"
    assert bam.validate(data[:-3], frozenset()).status == "fail"


def test_bgzf_limits_and_cross_block_truncation():
    assert bam.validate(bgzf(b"x" * 65537), frozenset()).status == "fail"
    assert bam.validate(bgzf(raw_bam()[:-1]) + bam.EOF_BLOCK, frozenset()).status == "fail"
    assert bam.validate(bgzf(b"VCF data") + bam.EOF_BLOCK, frozenset()) is None
