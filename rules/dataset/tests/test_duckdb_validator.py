import struct

from magika_datasets.validators.data import duckdb


def header(fields: bytes) -> bytes:
    body = fields.ljust(4088, b"\0")
    return struct.pack("<Q", duckdb.checksum(body)) + body


def build(block_size: int = 262144, blocks: int = 1) -> bytes:
    main = header(b"DUCK" + struct.pack("<Q", 64) + bytes(32) + b"v1.0.0".ljust(32, b"\0"))
    db = header(struct.pack("<QQQQQ", 1, 0, 0, blocks, block_size))
    return main + db + db + bytes(block_size * blocks)


def test_database_passes():
    result = duckdb.validate(build(), frozenset())
    assert result.status == "pass" and "1 blocks" in result.detail


def test_checksum_and_size_mismatches_fail():
    data = bytearray(build())
    data[100] ^= 1
    assert duckdb.validate(bytes(data), frozenset()).status == "fail"
    assert duckdb.validate(build() + b"\0", frozenset()).status == "fail"
