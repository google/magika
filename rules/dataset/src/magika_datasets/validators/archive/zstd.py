# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Zstandard frames: frame headers, block headers and skippable frames tiling the file."""

import struct

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("zst",)
SCOPE = "Frame header descriptor fields, window and dictionary fields, every block header with its declared size, last-block flag, content checksum presence and skippable frames tiling the file; blocks not decoded, xxHash not verified"
MAGIC = b"\x28\xb5\x2f\xfd"
SKIPPABLE = 0x184D2A50
FRAMES = 65536
BLOCKS = 1 << 20


class Malformed(Exception):
    pass


def frame(data: bytes, offset: int) -> tuple[int, int, bool]:
    """Walk one compressed frame; returns (end offset, block count, has checksum)."""
    descriptor = data[offset + 4]
    single_segment = descriptor & 0x20
    fcs_flag, dict_flag = descriptor >> 6, descriptor & 3
    if descriptor & 0x08:
        raise Malformed("Reserved frame header bit set")
    position = offset + 5
    if not single_segment:
        position += 1  # window descriptor
    position += (0, 1, 2, 4)[dict_flag]
    position += (1 if single_segment else 0, 2, 4, 8)[fcs_flag]
    if position > len(data):
        raise Malformed("Truncated frame header")
    blocks = 0
    while True:
        if position + 3 > len(data):
            raise Malformed("Truncated block header")
        header = int.from_bytes(data[position : position + 3], "little")
        last, kind, size = header & 1, (header >> 1) & 3, header >> 3
        if kind == 3:
            raise Malformed("Reserved block type")
        length = 1 if kind == 1 else size
        if size > 128 * 1024 and kind != 1:
            raise Malformed("Block larger than the format maximum")
        position += 3 + length
        if position > len(data):
            raise Malformed("Block data outside file")
        blocks += 1
        if blocks > BLOCKS:
            raise Malformed("Block budget exceeded")
        if last:
            break
    if descriptor & 0x04:
        position += 4
        if position > len(data):
            raise Malformed("Truncated content checksum")
    return position, blocks, bool(descriptor & 0x04)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    offset, frames, blocks, tags = 0, 0, 0, set()
    try:
        while offset < len(data):
            if offset + 4 > len(data):
                raise Malformed("Truncated frame magic")
            magic = struct.unpack_from("<I", data, offset)[0]
            if magic & 0xFFFFFFF0 == SKIPPABLE:
                size = struct.unpack_from("<I", data, offset + 4)[0]
                offset += 8 + size
                tags.add("skippable_frame")
            elif data[offset : offset + 4] == MAGIC:
                if offset + 5 > len(data):
                    raise Malformed("Truncated frame header")
                offset, count, checksum = frame(data, offset)
                blocks += count
                if checksum:
                    tags.add("content_checksum")
            else:
                raise Malformed("Bytes between frames are not a frame")
            if offset > len(data):
                raise Malformed("Frame outside file")
            frames += 1
            if frames > FRAMES:
                return Observation("inconclusive", "Frame budget exceeded", "zst")
    except (Malformed, struct.error) as error:
        return Observation("fail", str(error) or "Truncated frame", "zst")
    return Observation(
        "pass",
        f"{frames} frames, {blocks} blocks; frame and block headers tile the file",
        "zst",
        tuple(sorted(tags)),
        generic=True,  # a zst stream wraps arbitrary content (tar, sql, models)
    )
