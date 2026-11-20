# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""AppleSingle and AppleDouble: version 2 header and non-overlapping entry table."""

import struct

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("applesingle", "appledouble")
SCOPE = "Magic, version, entry count and every entry's offset and length inside the file without overlap; forks not interpreted"
MAGICS = {0x00051600: "applesingle", 0x00051607: "appledouble"}


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 26:
        return None
    magic, version = struct.unpack_from(">II", data)
    kind = MAGICS.get(magic)
    if kind is None:
        return None
    if version not in (0x00010000, 0x00020000):
        return Observation("fail", "Unknown AppleSingle version", kind)
    count = struct.unpack_from(">H", data, 24)[0]
    if count > 64 or 26 + 12 * count > len(data):
        return Observation("fail", "Entry table outside file", kind)
    ranges = []
    for index in range(count):
        _, offset, length = struct.unpack_from(">III", data, 26 + 12 * index)
        if offset + length > len(data):
            return Observation("fail", f"Entry {index} outside file", kind)
        ranges.append((offset, offset + length))
    ranges.sort()
    for (_, end), (start, _) in zip(ranges, ranges[1:]):
        if start < end:
            return Observation("fail", "Entries overlap", kind)
    return Observation("pass", f"{count} entries bounded", kind)
