# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Blender files: header pointer size and endianness, file blocks to ENDB."""

import struct

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("blend",)
SCOPE = "BLENDER header with pointer size, endianness and version, every file block's code, size and count with the header width from the pointer size, blocks tiling the file to the ENDB block; DNA not decoded"
BLOCKS = 1_000_000


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"BLENDER") or len(data) < 12:
        return None
    pointer, endian, version = data[7:8], data[8:9], data[9:12]
    if pointer not in (b"-", b"_") or endian not in (b"v", b"V") or not version.isdigit():
        return Observation("fail", "Invalid header fields", "blend")
    order = "<" if endian == b"v" else ">"
    wide = pointer == b"-"
    header = order + ("IQII" if wide else "IIII")
    size = struct.calcsize(header) + 4
    offset, count = 12, 0
    while offset < len(data):
        count += 1
        if count > BLOCKS:
            return Observation("inconclusive", "Block budget exceeded", "blend")
        if offset + size > len(data):
            return Observation("fail", "Truncated block header", "blend")
        code = data[offset : offset + 4]
        length, _, _, _ = struct.unpack_from(header, data, offset + 4)
        offset += size + length
        if offset > len(data):
            return Observation("fail", f"Block {code!r} exceeds file", "blend")
        if code == b"ENDB":
            if offset != len(data):
                return Observation("fail", "Bytes after the ENDB block", "blend")
            return Observation(
                "pass",
                f"{count} file blocks tiling the file; version {version.decode()}",
                "blend",
                ("pointer64" if wide else "pointer32",),
            )
    return Observation("fail", "Missing ENDB block", "blend")
