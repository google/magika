import struct

from magika_datasets.validators.archive import squashfs


def superblock(used: int = 200, padding: int = 0, block_log: int = 17) -> bytes:
    head = struct.pack(
        "<IIIIHHHHHHQQ", 5, 0, 1 << block_log, 0, 4, block_log, 0xC0, 1, 4, 0, 0, used
    )
    tables = struct.pack(
        "<QQQQQQ", 190, 0xFFFFFFFFFFFFFFFF, 96, 150, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF
    )
    data = b"hsqs" + head + tables
    return data + b"\x01" * (used - len(data)) + bytes(padding)


def test_squashfs_passes_with_padding_and_is_generic():
    result = squashfs.validate(superblock(padding=100), frozenset())
    assert result.status == "pass" and result.generic and result.tags == ("xz",)
    assert squashfs.validate(superblock(padding=9000), frozenset()).tags == ("xz", "zero_padded")


def test_squashfs_nonzero_tail_is_inconclusive():
    assert squashfs.validate(superblock() + b"\x01", frozenset()).status == "inconclusive"


def test_squashfs_bad_block_log_fails():
    assert squashfs.validate(superblock(block_log=5), frozenset()).status == "fail"


def test_squashfs_truncated_fails():
    assert squashfs.validate(superblock()[:150], frozenset()).status == "fail"
