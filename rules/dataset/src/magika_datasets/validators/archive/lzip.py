# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""lzip members: LZMA stream decoded completely, trailer CRC-32 and sizes verified."""

import lzma
import struct
import zlib

from .._shared import decompress
from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("lz",)
SCOPE = "Member header (version 1, coded dictionary size), complete bounded LZMA decode (lc=3, lp=0, pb=2), trailer CRC-32 of the decoded data, data size and member size, concatenated members tiling the file"
MAGIC = b"LZIP"
MEMBERS = 4096


def dictionary_size(coded: int) -> int:
    base = 1 << (coded & 0x1F)
    return base - (base // 16) * ((coded >> 5) & 7)


def member(data: bytes, offset: int) -> tuple[int, int]:
    """(next offset, decoded size) for the member at offset; raises ValueError."""
    if data[offset : offset + 4] != MAGIC or offset + 6 > len(data):
        raise ValueError("Member magic missing")
    version, coded = data[offset + 4], data[offset + 5]
    if version != 1 or not 12 <= (coded & 0x1F) <= 29:
        raise ValueError("Unsupported lzip version or dictionary size")
    filters = [
        {"id": lzma.FILTER_LZMA1, "dict_size": dictionary_size(coded), "lc": 3, "lp": 0, "pb": 2}
    ]
    decoder = lzma.LZMADecompressor(lzma.FORMAT_RAW, filters=filters)
    crc, total = 0, 0
    pending = data[offset + 6 :]
    while True:
        chunk = decoder.decompress(pending, max_length=1 << 20)
        pending = b""
        crc = zlib.crc32(chunk, crc)
        total += len(chunk)
        if total > decompress.LIMIT:
            raise decompress.Budget("Expanded output exceeds budget")
        if decoder.eof:
            break
        if decoder.needs_input:
            raise ValueError("LZMA stream ended before its end marker")
    unused = decoder.unused_data
    consumed = len(data) - offset - 6 - len(unused)
    trailer = data[offset + 6 + consumed : offset + 6 + consumed + 20]
    if len(trailer) != 20:
        raise ValueError("Trailer truncated")
    stored_crc, data_size, member_size = struct.unpack("<IQQ", trailer)
    if stored_crc != crc or data_size != total:
        raise ValueError("Trailer CRC-32 or data size mismatch")
    if member_size != 6 + consumed + 20:
        raise ValueError("Trailer member size mismatch")
    return offset + member_size, total


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    offset, members, expanded = 0, 0, 0
    try:
        while offset < len(data):
            offset, size = member(data, offset)
            expanded += size
            members += 1
            if members > MEMBERS:
                return Observation("inconclusive", "Member budget exceeded", "lz")
    except decompress.Budget as error:
        return Observation("inconclusive", str(error), "lz")
    except (ValueError, lzma.LZMAError) as error:
        return Observation("fail", str(error) or "Corrupt LZMA data", "lz")
    return Observation(
        "pass",
        f"{members} members decoded to {expanded} bytes; trailer CRC-32 and sizes verified",
        "lz",
        generic=True,
    )
