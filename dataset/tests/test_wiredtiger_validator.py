import struct

from magika_datasets.validators._shared.crc32c import crc32c
from magika_datasets.validators.data import wiredtiger

UNIT = wiredtiger.UNIT


def descriptor() -> bytes:
    unit = struct.pack("<IHHI", wiredtiger.MAGIC, 1, 0, 0) + b"\0" * (UNIT - 12)
    return unit[:8] + struct.pack("<I", crc32c(unit)) + unit[12:]


def block(units: int = 1, flags: int = wiredtiger.DATA_CHECKSUM) -> bytes:
    size = units * UNIT
    body = b"\x07" * 28 + struct.pack("<IIB", size, 0, flags) + b"\0\0\0" + b"payload" * 10
    body = body.ljust(size, b"\0")
    covered = body if flags & wiredtiger.DATA_CHECKSUM else body[:64]
    return body[:32] + struct.pack("<I", crc32c(covered)) + body[36:]


def test_the_crc32c_check_value():
    assert crc32c(b"123456789") == 0xE3069283


def test_descriptor_and_blocks_verify():
    data = descriptor() + block(2) + b"\0" * UNIT + block(1, flags=0)
    result = wiredtiger.validate(data, frozenset())
    assert result.status == "pass" and "2 blocks verified, 1 unallocated" in result.detail


def test_a_corrupt_block_fails():
    data = bytearray(descriptor() + block(1))
    data[UNIT + 100] ^= 1
    assert "CRC-32C" in wiredtiger.validate(bytes(data), frozenset()).detail


def test_a_corrupt_descriptor_fails():
    data = bytearray(descriptor())
    data[200] = 1
    assert wiredtiger.validate(bytes(data), frozenset()).status == "fail"


def test_a_partial_allocation_unit_fails():
    assert wiredtiger.validate(descriptor() + b"\0" * 10, frozenset()).status == "fail"
