# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""pcapng captures: section header first, every block's leading and trailing lengths tiling the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("pcapng",)
SCOPE = "Section Header Block first with byte-order magic, every block's type, length at both ends and 4-byte alignment, blocks tiling the file; packet contents not decoded"
BLOCKS = 10_000_000


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"\x0a\x0d\x0d\x0a":
        return None
    if len(data) < 28:
        return Observation("fail", "Truncated section header", "pcapng")
    order = (
        "<"
        if data[8:12] == b"\x4d\x3c\x2b\x1a"
        else ">"
        if data[8:12] == b"\x1a\x2b\x3c\x4d"
        else None
    )
    if order is None:
        return Observation("fail", "Byte-order magic missing", "pcapng")
    offset, count, sections = 0, 0, 0
    while offset < len(data):
        count += 1
        if count > BLOCKS:
            return Observation("inconclusive", "Block budget exceeded", "pcapng")
        if offset + 12 > len(data):
            return Observation("fail", "Truncated block header", "pcapng")
        kind, length = struct.unpack_from(order + "II", data, offset)
        if kind == 0x0A0D0D0A:
            sections += 1
            order = "<" if data[offset + 8 : offset + 12] == b"\x4d\x3c\x2b\x1a" else ">"
            length = struct.unpack_from(order + "I", data, offset + 4)[0]
        if length < 12 or length % 4 or offset + length > len(data):
            return Observation("fail", f"Block {count} length invalid", "pcapng")
        if struct.unpack_from(order + "I", data, offset + length - 4)[0] != length:
            return Observation("fail", f"Block {count} trailing length differs", "pcapng")
        offset += length
    return Observation(
        "pass", f"{count} blocks in {sections} section(s) tiling the capture", "pcapng"
    )
