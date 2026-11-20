import struct

from magika_datasets.validators.data import mysql_storage


def myisam(length: int | None = None) -> bytes:
    data = bytearray(2048)
    data[0:4] = b"\xfe\xfe\x07\x01"
    struct.pack_into(">HHHHHH", data, 6, 176, 100, 100, 176, 1, 0)
    data[18] = 1
    struct.pack_into(">Q", data, 60, len(data) if length is None else length)
    return bytes(data)


def innodb(pages: int = 2, bad_page: bool = False) -> bytes:
    size = 16384
    data = bytearray(size * pages)
    for index in range(pages):
        struct.pack_into(">I", data, index * size + 4, index)
        struct.pack_into(">I", data, index * size + 34, 7)
    struct.pack_into(">H", data, 24, 8)  # FSP_HDR on page 0
    if bad_page:
        struct.pack_into(">I", data, size + 4, 9)
    return bytes(data)


def test_myisam_index_passes_and_length_mismatch_fails():
    assert mysql_storage.validate(myisam(), frozenset()).tags == ("myisam_index",)
    assert mysql_storage.validate(myisam(length=100), frozenset()).status == "fail"


def test_innodb_tablespace_pages_are_numbered():
    assert mysql_storage.validate(innodb(), frozenset()).tags == ("innodb_tablespace",)
    assert mysql_storage.validate(innodb(bad_page=True), frozenset()).status == "fail"
    assert mysql_storage.validate(innodb() + b"\0", frozenset()).status == "fail"
