# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ACE archives: header CRC-16 of every block and block sizes tiling the file."""

import struct
import zlib

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("ace",)
SCOPE = "Main header magic and version, every header's CRC-16 (low half of an un-finalised CRC-32), ADDSIZE payloads chaining to EOF, encryption and solid flags; payloads not decoded"
MAGIC = b"**ACE**"
BLOCKS = 65536


def crc16(data: bytes) -> int:
    return (~zlib.crc32(data)) & 0xFFFF


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 14 or data[7:14] != MAGIC:
        return None
    offset, blocks, tags = 0, 0, set()
    while offset < len(data):
        if offset + 4 > len(data):
            return Observation("fail", "Truncated block header", "ace")
        stored, size = struct.unpack_from("<HH", data, offset)
        header = data[offset + 4 : offset + 4 + size]
        if len(header) != size or size < 3:
            return Observation("fail", "Block header outside file", "ace")
        if crc16(header) != stored:
            return Observation("fail", "Header CRC-16 mismatch", "ace")
        kind, flags = header[0], struct.unpack_from("<H", header, 1)[0]
        if blocks == 0 and (kind != 0 or header[3:10] != MAGIC):
            return Observation("fail", "First block is not the main header", "ace")
        payload = 0
        if flags & 1:
            if size < 7:
                return Observation("fail", "ADDSIZE flag without a size field", "ace")
            payload = struct.unpack_from("<I", header, 3)[0]
        if kind == 0 and flags & 0x8000:
            tags.add("solid")
        if kind == 1 and flags & 0x4000:
            tags.add("encrypted")
        offset += 4 + size + payload
        blocks += 1
        if blocks > BLOCKS:
            return Observation("inconclusive", "Block budget exceeded", "ace")
    if offset != len(data):
        return Observation("fail", "Last block extends past EOF", "ace")
    return Observation(
        "pass",
        f"{blocks} blocks; header CRCs and payload sizes tile the file",
        "ace",
        tuple(sorted(tags)),
    )
