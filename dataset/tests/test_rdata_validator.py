import struct

from magika_datasets.validators.data import rdata


def i(*values: int) -> bytes:
    return struct.pack(f">{len(values)}i", *values)


def char(text: bytes) -> bytes:
    return i(9 | 0x40000) + i(len(text)) + text  # CHARSXP, UTF-8 flag in the level bits


def header(version: int = 3) -> bytes:
    out = i(version, 0x040301, 0x030500)
    return out + (i(5) + b"UTF-8" if version == 3 else b"")


# pairlist (x = 1.5, x = <reference to symbol 1>)
BODY = (
    i(2 | 1 << 10)
    + i(1)
    + char(b"x")
    + i(14, 1)
    + struct.pack(">d", 1.5)
    + i(2 | 1 << 10)
    + i(255 | 1 << 8)
    + i(16, 1)
    + char(b"y")
    + i(254)
)


def test_a_saved_pairlist_is_walked_to_the_end():
    result = rdata.validate(b"RDX3\nX\n" + header() + BODY, frozenset())
    assert result.status == "pass" and "R 4.3.1 (UTF-8)" in result.detail


def test_a_version_2_rds_stream_passes():
    assert rdata.validate(b"X\n" + header(2) + BODY, frozenset()).status == "pass"


def test_truncation_and_trailing_bytes_fail():
    data = b"RDX3\nX\n" + header() + BODY
    assert rdata.validate(data[:-1], frozenset()).status == "fail"
    assert "follow" in rdata.validate(data + b"\0", frozenset()).detail


def test_a_reference_before_its_definition_fails():
    body = i(2 | 1 << 10) + i(255 | 2 << 8) + i(254) + i(254)
    result = rdata.validate(b"RDX3\nX\n" + header() + body, frozenset())
    assert result.status == "fail" and "Reference 2" in result.detail


def test_a_header_version_disagreeing_with_the_stream_fails():
    assert rdata.validate(b"RDX2\nX\n" + header() + BODY, frozenset()).status == "fail"


def test_compressed_or_ascii_files_are_not_claimed():
    assert rdata.validate(b"\x1f\x8b\x08\x00", frozenset()) is None
    assert rdata.validate(b"RDA3\nA\n3\n", frozenset()) is None
