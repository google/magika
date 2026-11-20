import struct

from magika_datasets.validators.data import hdf5


def superblock_v0(total: int = 2048, user_block: int = 0) -> bytes:
    head = bytearray(96)
    head[0:8] = hdf5.SIGNATURE
    head[8:16] = bytes([0, 0, 0, 0, 0, 8, 8, 0])
    struct.pack_into("<HH", head, 16, 4, 16)
    struct.pack_into("<QQQQ", head, 24, user_block, 0xFFFFFFFFFFFFFFFF, total, 0xFFFFFFFFFFFFFFFF)
    struct.pack_into(
        "<QQ", head, 56, 0, 96
    )  # root symbol table entry: link name offset, object header
    return bytes(user_block) + bytes(head) + bytes(total - len(head))


def superblock_v2(total: int = 1024) -> bytes:
    head = bytearray(48)
    head[0:8] = hdf5.SIGNATURE
    head[8:12] = bytes([2, 8, 8, 0])
    struct.pack_into("<QQQQ", head, 12, 0, 0xFFFFFFFFFFFFFFFF, total, 48)
    struct.pack_into("<I", head, 44, hdf5.lookup3(bytes(head[:44])))
    return bytes(head) + bytes(total - len(head))


def test_lookup3_reference_value():
    assert hdf5.lookup3(b"") == 0xDEADBEEF
    assert hdf5.lookup3(b"Four score and seven years ago") == 0x17770551


def test_superblocks_pass():
    assert hdf5.validate(superblock_v0(), frozenset()).status == "pass"
    assert hdf5.validate(superblock_v0(user_block=512), frozenset()).status == "pass"
    result = hdf5.validate(superblock_v2(), frozenset())
    assert result.status == "pass" and result.tags == ("superblock_checksum",) and result.generic


def test_eof_mismatch_and_bad_checksum_fail():
    assert hdf5.validate(superblock_v0() + b"\0", frozenset()).status == "fail"
    data = bytearray(superblock_v2())
    data[20] ^= 1
    assert hdf5.validate(bytes(data), frozenset()).status == "fail"
