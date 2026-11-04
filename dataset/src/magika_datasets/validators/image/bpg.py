# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Better Portable Graphics: header varints, extension block and picture data length."""

from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("bpg",)
SCOPE = "Magic, header flags, ue7 dimensions, extension block bounds and picture data length equal to the remaining bytes; HEVC payload not decoded"


def ue7(data: bytes, offset: int) -> tuple[int, int] | None:
    value = 0
    for position in range(offset, min(offset + 5, len(data))):
        value = (value << 7) | (data[position] & 0x7F)
        if not data[position] & 0x80:
            return value, position + 1
    return None


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"BPG\xfb":
        return None
    if len(data) < 8:
        return Observation("fail", "Truncated header", "bpg")
    extension = bool(data[5] & 0x08)
    offset = 6
    fields = []
    for _ in range(3):
        parsed = ue7(data, offset)
        if parsed is None:
            return Observation("fail", "Truncated header varint", "bpg")
        value, offset = parsed
        fields.append(value)
    width, height, length = fields
    if not width or not height:
        return Observation("fail", "Zero picture dimensions", "bpg")
    if extension:
        parsed = ue7(data, offset)
        if parsed is None:
            return Observation("fail", "Truncated extension length", "bpg")
        size, offset = parsed
        offset += size
        if offset > len(data):
            return Observation("fail", "Extension data exceeds file", "bpg")
    remaining = len(data) - offset
    if remaining <= 0 or (length and length != remaining):
        return Observation("fail", "Picture data length differs from remaining bytes", "bpg")
    return Observation(
        "pass", "BPG header and picture data bounds checked; HEVC not decoded", "bpg"
    )
