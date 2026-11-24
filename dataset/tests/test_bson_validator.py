import struct

from magika_datasets.validators.data import bson


def element(kind: int, name: bytes, value: bytes) -> bytes:
    return bytes([kind]) + name + b"\0" + value


def string(text: bytes) -> bytes:
    return struct.pack("<i", len(text) + 1) + text + b"\0"


def document(*elements: bytes) -> bytes:
    body = b"".join(elements)
    return struct.pack("<i", len(body) + 5) + body + b"\0"


RECORD = document(
    element(0x07, b"_id", b"\x01" * 12),
    element(0x02, b"name", string(b"ada")),
    element(0x08, b"active", b"\x01"),
    element(0x04, b"tags", document(element(0x02, b"0", string(b"math")))),
    element(0x05, b"blob", struct.pack("<i", 3) + b"\x00abc"),
    element(0x12, b"count", struct.pack("<q", 7)),
    element(0x0A, b"nothing", b""),
)


def test_one_document_is_bson():
    result = bson.validate(RECORD, frozenset())
    assert (result.status, result.format_id) == ("pass", "bson")
    assert result.detail == "1 documents with 8 elements decoded"


def test_concatenated_documents_are_a_mongodb_dump():
    result = bson.validate(RECORD * 3, frozenset())
    assert (result.status, result.format_id) == ("pass", "mongodb_bson")


def test_a_bad_string_terminator_fails():
    data = RECORD.replace(b"ada\0", b"adax")
    assert bson.validate(data, frozenset()).status == "fail"


def test_a_boolean_outside_zero_and_one_fails():
    data = RECORD.replace(b"active\0\x01", b"active\0\x02")
    assert "Boolean" in bson.validate(data, frozenset()).detail


def test_a_truncated_dump_fails_as_a_dump():
    result = bson.validate((RECORD * 2)[:-3], frozenset())
    assert (result.status, result.format_id) == ("fail", "mongodb_bson")
