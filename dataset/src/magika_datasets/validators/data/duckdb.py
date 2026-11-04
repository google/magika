# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""DuckDB database files: main header and both database headers with their 64-bit checksums, block count against the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("duckdb",)
SCOPE = "Main header magic and checksum, both database headers' checksums, block allocation size, header block count against the file size; blocks not parsed"
MAGIC = b"DUCK"
HEADER = 4096
DEFAULT_BLOCK = 262144
MULTIPLIER = 0xBF58476D1CE4E5B9
MASK = (1 << 64) - 1


def checksum(buffer: bytes) -> int:
    """DuckDB's header checksum: XOR of multiplied 64-bit words seeded with 5381."""
    result = 5381
    words = len(buffer) // 8
    for (value,) in struct.iter_unpack("<Q", buffer[: words * 8]):
        result ^= (value * MULTIPLIER) & MASK
    return result if len(buffer) == words * 8 else None


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 12 or data[8:12] != MAGIC:
        return None
    if len(data) < 3 * HEADER:
        return Observation("fail", "File shorter than the three header blocks", "duckdb")
    if struct.unpack_from("<Q", data, 0)[0] != checksum(data[8:HEADER]):
        return Observation("fail", "Main header checksum mismatch", "duckdb")
    version = struct.unpack_from("<Q", data, 12)[0]
    sizes = set()
    for index in (1, 2):
        offset = index * HEADER
        if struct.unpack_from("<Q", data, offset)[0] != checksum(
            data[offset + 8 : offset + HEADER]
        ):
            return Observation("fail", f"Database header {index} checksum mismatch", "duckdb")
        iteration, _, _, block_count, block_size = struct.unpack_from("<QQQQQ", data, offset + 8)
        sizes.add(block_size)
    if len(sizes) != 1:
        return Observation("fail", "Database headers disagree on the block size", "duckdb")
    block_size = sizes.pop() or DEFAULT_BLOCK  # headers before v0.10 leave the field zero
    if block_size & (block_size - 1):
        return Observation("fail", "Block size is not a power of two", "duckdb")
    if (len(data) - 3 * HEADER) % block_size:
        return Observation("fail", "File is not headers plus whole blocks", "duckdb")
    return Observation(
        "pass",
        f"Storage version {version}, {(len(data) - 3 * HEADER) // block_size} blocks of {block_size} bytes; header checksums verified",
        "duckdb",
    )
