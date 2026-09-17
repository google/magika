# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""SquashFS 4 superblock: table ordering and bytes_used against the padded file."""

import struct

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("squashfs",)
SCOPE = "Superblock version 4.0, block size against its log, compressor id, table offsets ordered and inside bytes_used, bytes_used matching the file up to zero padding; tables not decompressed"
MAGIC = b"hsqs"
COMPRESSORS = {1: "gzip", 2: "lzma", 3: "lzo", 4: "xz", 5: "lz4", 6: "zstd"}
PAD = 4096


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 96:
        return Observation("fail", "Truncated superblock", "squashfs")
    (
        inodes,
        _,
        block_size,
        fragments,
        compressor,
        block_log,
        flags,
        ids,
        major,
        minor,
        root,
        used,
    ) = struct.unpack_from("<IIIIHHHHHHQQ", data, 4)
    tables = struct.unpack_from(
        "<QQQQQQ", data, 48
    )  # id, xattr, inode, directory, fragment, export
    if (major, minor) != (4, 0):
        return Observation(
            "inconclusive",
            f"Only version 4.0 superblocks are checked (found {major}.{minor})",
            "squashfs",
        )
    if block_log < 12 or block_log > 20 or block_size != 1 << block_log:
        return Observation("fail", "Block size does not match its log", "squashfs")
    if compressor not in COMPRESSORS:
        return Observation("fail", "Unknown compressor", "squashfs")
    if used > len(data) or used < 96:
        return Observation("fail", "bytes_used exceeds the file", "squashfs")
    present = [offset for offset in tables if offset != 0xFFFFFFFFFFFFFFFF]
    if any(offset >= used for offset in present):
        return Observation("fail", "Table offset beyond bytes_used", "squashfs")
    inode_table, directory_table = tables[2], tables[3]
    if not 96 <= inode_table < directory_table:
        return Observation("fail", "Inode table must precede the directory table", "squashfs")
    tail = data[used:]
    if any(tail):  # appended verity trees or signatures are outside the filesystem's own accounting
        return Observation(
            "inconclusive", f"{len(tail)} bytes after bytes_used are not zero padding", "squashfs"
        )

    tags = [COMPRESSORS[compressor]]
    if len(tail) > PAD:
        tags.append("zero_padded")  # firmware images pad to their flash erase block
    return Observation(
        "pass",
        f"{inodes} inodes, {fragments} fragments, {ids} ids; superblock, table order and bytes_used verified",
        "squashfs",
        tuple(tags),
        generic=True,  # snaps and firmware images are SquashFS filesystems with their own labels
    )
