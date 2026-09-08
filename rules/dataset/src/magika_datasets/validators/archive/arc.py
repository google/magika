# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ARC archives: entry headers with method bytes, sizes tiling the file, CRC-16 of stored entries."""

import struct

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("arc",)
SCOPE = "Every entry's 0x1A marker, method byte (1-11), NUL-terminated name, compressed and original sizes, entries tiling the file to the 0x1A 0x00 end marker, CRC-16 verified for stored (method 1 and 2) entries; compressed entries not decoded"
MARK = 0x1A
ENTRIES = 65536

TABLE = []
for index in range(256):
    value = index
    for _ in range(8):
        value = (value >> 1) ^ 0xA001 if value & 1 else value >> 1
    TABLE.append(value)


def crc16(data: bytes) -> int:
    value = 0
    for byte in data:
        value = (value >> 8) ^ TABLE[(value ^ byte) & 0xFF]
    return value


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 30 or data[0] != MARK or not 1 <= data[1] <= 11 or not 0x20 < data[2] < 0x7F:
        return None  # the first entry name must start with a printable character
    offset, entries, verified = 0, 0, 0
    while True:
        if offset + 2 > len(data):
            return Observation("fail", "Archive ends without its end marker", "arc")
        if data[offset] != MARK:
            return Observation("fail", f"Entry {entries} lacks the 0x1A marker", "arc")
        method = data[offset + 1]
        if method == 0:
            if offset + 2 != len(data):
                return Observation("fail", "Bytes after the end marker", "arc")
            break
        if method > 11:
            return Observation("fail", f"Unknown compression method {method}", "arc")
        header = 25 if method == 1 else 29
        if offset + header > len(data):
            return Observation("fail", "Entry header truncated", "arc")
        name = data[offset + 2 : offset + 15]
        if b"\0" not in name:
            return Observation("fail", "Entry name is not NUL-terminated", "arc")
        compressed = struct.unpack_from("<I", data, offset + 15)[0]
        crc = struct.unpack_from("<H", data, offset + 23)[0]
        original = compressed if method == 1 else struct.unpack_from("<I", data, offset + 25)[0]
        body = data[offset + header : offset + header + compressed]
        if len(body) != compressed:
            return Observation("fail", "Entry data outside file", "arc")
        if method in (1, 2):
            if original != compressed or crc16(body) != crc:
                return Observation("fail", "Stored entry CRC-16 or size mismatch", "arc")
            verified += 1
        offset += header + compressed
        entries += 1
        if entries > ENTRIES:
            return Observation("inconclusive", "Entry budget exceeded", "arc")
    return Observation(
        "pass", f"{entries} entries tile the file; {verified} stored entries CRC-checked", "arc"
    )
