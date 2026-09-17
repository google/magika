import struct

from magika_datasets.validators.geometry import alembic


def archive(frozen: int = 0xFF, child_size: int = 5) -> bytes:
    body = bytearray(b"Ogawa" + bytes([frozen]) + b"\x00\x01" + b"\0" * 8)
    first = len(body)
    body += struct.pack("<Q", child_size) + b"hello"
    inner = len(body)
    body += struct.pack("<QQ", 1, alembic.DATA | first)
    root = len(body)
    body += struct.pack("<QQQQ", 3, alembic.DATA | first, inner, alembic.DATA)  # empty block
    body[8:16] = struct.pack("<Q", root)
    return bytes(body)


def test_the_tree_is_walked_inside_the_file():
    result = alembic.validate(archive(), frozenset())
    assert result.status == "pass" and result.detail == "2 groups and 1 data blocks inside the file"


def test_a_data_block_past_the_end_fails():
    assert alembic.validate(archive(child_size=10**6), frozenset()).status == "fail"


def test_truncation_fails():
    assert alembic.validate(archive()[:-4], frozenset()).status == "fail"


def test_an_unfinalized_archive_is_inconclusive():
    assert alembic.validate(archive(frozen=0), frozenset()).status == "inconclusive"
