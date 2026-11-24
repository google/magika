# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Finder .DS_Store files: buddy allocator header, root block and block addresses."""

import struct

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("dsstore",)
SCOPE = "Bud1 header with matching root block offsets, root block inside the file, every allocated block address and size inside the file, table of contents entries; records not decoded"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:8] != b"\0\0\0\x01Bud1" or len(data) < 36:
        return None
    root_offset, root_size, root_again = struct.unpack_from(">III", data, 8)
    if root_offset != root_again or root_size < 12 or 4 + root_offset + root_size > len(data):
        return Observation("fail", "Root block offsets or size invalid", "dsstore")
    position = 4 + root_offset
    count = struct.unpack_from(">I", data, position)[0]
    position += 8
    if count > 4096 or position + 4 * count > len(data):
        return Observation("fail", "Block address table outside file", "dsstore")
    for index in range(count):
        address = struct.unpack_from(">I", data, position + 4 * index)[0]
        if not address:
            continue
        offset, size = address & ~0x1F, 1 << (address & 0x1F)
        if 4 + offset + size > len(data):
            return Observation("fail", f"Block {index} outside file", "dsstore")
    position += 4 * count
    for _ in range(32):  # free lists, one per power-of-two size
        if position + 4 > len(data):
            return Observation("fail", "Free lists outside file", "dsstore")
        entries = struct.unpack_from(">I", data, position)[0]
        position += 4 + 4 * entries
    if position + 4 > len(data):
        return Observation("fail", "Table of contents outside file", "dsstore")
    toc = struct.unpack_from(">I", data, position)[0]
    position += 4
    for _ in range(min(toc, 4096)):
        if position >= len(data):
            return Observation("fail", "Table of contents entry outside file", "dsstore")
        length = data[position]
        position += 1 + length + 4
    if position > 4 + root_offset + root_size:
        return Observation("fail", "Root block contents exceed its size", "dsstore")
    return Observation("pass", f"{count} blocks and {toc} directory entries bounded", "dsstore")
