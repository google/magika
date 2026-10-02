# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""WiredTiger data files: block descriptor and every allocated block's CRC-32C."""

import struct

from .._shared.crc32c import crc32c
from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("wiredtiger",)
SCOPE = "Block manager descriptor (magic 120897, version 1.x) with its CRC-32C over the first 4 KB allocation unit, then every block walked in allocation units: disk size a whole number of units inside the file and CRC-32C over the whole block when WT_BLOCK_DATA_CKSUM is set, otherwise over its first 64 bytes; unallocated units skipped; checkpoint lists, page contents and compression not interpreted"
MAGIC = 120897
UNIT = 4096  # the default allocation size; the descriptor always occupies the first unit
PAGE_HEADER = 28
DATA_CHECKSUM = 0x01
SKIP = 64
BLOCKS = 1_000_000


def zeroed(block: bytes, at: int) -> bytes:
    return block[:at] + b"\0\0\0\0" + block[at + 4 :]


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 12 or struct.unpack_from("<I", data)[0] != MAGIC:
        return None
    major, minor, checksum = struct.unpack_from("<HHI", data, 4)
    if major != 1 or len(data) < UNIT or len(data) % UNIT:
        return Observation(
            "fail", "Unsupported version or size not whole allocation units", "wiredtiger"
        )
    if crc32c(zeroed(data[:UNIT], 8)) != checksum:
        return Observation("fail", "Block descriptor CRC-32C mismatch", "wiredtiger")
    offset, blocks, unallocated = UNIT, 0, 0
    while offset < len(data):
        size, stored, flags = struct.unpack_from("<IIB", data, offset + PAGE_HEADER)
        if not size or size % UNIT or offset + size > len(data):
            unallocated += 1
            offset += UNIT
            continue
        block = zeroed(data[offset : offset + size], PAGE_HEADER + 4)
        if crc32c(block if flags & DATA_CHECKSUM else block[:SKIP]) != stored:
            return Observation("fail", f"Block at offset {offset} fails its CRC-32C", "wiredtiger")
        blocks += 1
        if blocks > BLOCKS:
            return Observation("inconclusive", "Block budget exceeded", "wiredtiger")
        offset += size
    return Observation(
        "pass",
        f"Descriptor {major}.{minor}; {blocks} blocks verified, {unallocated} unallocated units",
        "wiredtiger",
    )
