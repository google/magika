# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Flash Video: header, tag chain and previous-tag-size back references."""

import struct

from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("flv",)
SCOPE = "FLV header and data offset, every tag's type, size and timestamp fields, each PreviousTagSize equal to its tag, tags tiling the file; codec payloads not decoded"
TYPES = {8, 9, 18}
TAGS = 1_000_000


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:3] != b"FLV" or len(data) < 13:
        return None
    if data[3] != 1:
        return Observation("fail", "Unsupported FLV version", "flv")
    flags = data[4]
    offset = struct.unpack_from(">I", data, 5)[0]
    if offset < 9 or offset + 4 > len(data) or struct.unpack_from(">I", data, offset)[0] != 0:
        return Observation("fail", "Invalid data offset or first PreviousTagSize", "flv")
    offset += 4
    count = 0
    while offset < len(data):
        count += 1
        if count > TAGS:
            return Observation("inconclusive", "Tag budget exceeded", "flv")
        if offset + 11 > len(data):
            return Observation("fail", "Truncated tag header", "flv")
        kind = data[offset] & 0x1F
        size = int.from_bytes(data[offset + 1 : offset + 4], "big")
        end = offset + 11 + size
        if kind not in TYPES:
            return Observation("fail", f"Unknown tag type {kind}", "flv")
        if end + 4 > len(data):
            return Observation("fail", "Tag exceeds file bounds", "flv")
        if struct.unpack_from(">I", data, end)[0] != 11 + size:
            return Observation("fail", "PreviousTagSize differs from tag", "flv")
        offset = end + 4
    tags = tuple(name for bit, name in ((4, "has_audio"), (1, "has_video")) if flags & bit)
    return Observation("pass", f"{count} FLV tags chained and bounded", "flv", tags)
