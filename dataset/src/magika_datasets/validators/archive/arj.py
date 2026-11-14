# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ARJ archives: CRC-32 protected headers and packed sizes chaining to the end marker."""

import struct
import zlib

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("arj",)
SCOPE = "Header id 0x60 0xEA, main header (file type 2) and every local header CRC-32, extended header chains with their CRC-32s, local file types, packed sizes chaining to the zero-size end marker, CRC-32 of stored (method 0) entries; compressed entries not decoded"
MAGIC = b"\x60\xea"
PREFIX_ONLY = True  # a two-byte id; a failure is evidence only when ARJ was hinted
MAX_HEADER = 2600  # ARJ's own limit on a basic header
ENTRIES = 65536
LOCAL_TYPES = {0, 1, 3, 4}  # binary, 7-bit text, directory, volume label


def header(data: bytes, offset: int) -> tuple[bytes, int] | None:
    """(basic header, offset after its extended headers); None at the end marker."""
    if data[offset : offset + 2] != MAGIC or offset + 4 > len(data):
        raise ValueError(f"Header id missing at offset {offset}")
    size = struct.unpack_from("<H", data, offset + 2)[0]
    if size == 0:
        return None
    if size < 30 or size > MAX_HEADER or offset + 8 + size > len(data):
        raise ValueError("Basic header size invalid")
    basic = data[offset + 4 : offset + 4 + size]
    if zlib.crc32(basic) != struct.unpack_from("<I", data, offset + 4 + size)[0]:
        raise ValueError("Basic header CRC-32 mismatch")
    offset += 8 + size
    for _ in range(256):
        if offset + 2 > len(data):
            raise ValueError("Extended header size truncated")
        extended = struct.unpack_from("<H", data, offset)[0]
        offset += 2
        if extended == 0:
            return basic, offset
        if offset + extended + 4 > len(data):
            raise ValueError("Extended header truncated")
        stored = struct.unpack_from("<I", data, offset + extended)[0]
        if zlib.crc32(data[offset : offset + extended]) != stored:
            raise ValueError("Extended header CRC-32 mismatch")
        offset += extended + 4
    raise ValueError("Extended header budget exceeded")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    try:
        main = header(data, 0)
        if main is None or main[0][6] != 2:
            raise ValueError("First header is not an ARJ main header")
        offset, entries, stored = main[1], 0, 0
        while True:
            local = header(data, offset)
            if local is None:
                offset += 4
                break
            basic, offset = local
            if basic[6] not in LOCAL_TYPES:
                raise ValueError(f"Unknown local file type {basic[6]}")
            method = basic[5]
            packed, _, crc = struct.unpack_from("<III", basic, 12)
            if offset + packed > len(data):
                raise ValueError("Packed data outside file")
            if method == 0 and basic[6] in (0, 1):
                if zlib.crc32(data[offset : offset + packed]) != crc:
                    raise ValueError("Stored entry CRC-32 mismatch")
                stored += 1
            offset += packed
            entries += 1
            if entries > ENTRIES:
                return Observation("inconclusive", "Entry budget exceeded", "arj")
    except (ValueError, IndexError, struct.error) as error:
        return Observation("fail", str(error) or "Truncated header", "arj")
    tags = ("trailing_data",) if offset < len(data) else ()
    return Observation(
        "pass",
        f"{entries} entries; header CRC-32s verified, {stored} stored entries checked",
        "arj",
        tags,
    )
