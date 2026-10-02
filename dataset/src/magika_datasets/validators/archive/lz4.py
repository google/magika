# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""LZ4 frames: descriptor with xxHash-32 header checksum, block chain and block checksums."""

import struct

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("lz4",)
SCOPE = "Frame descriptor (version, flags, block max size, header checksum via xxHash-32), block sizes tiling to the end mark, block checksums when present, content checksum size, legacy frames and skippable frames; blocks not decoded"
MAGIC = b"\x04\x22\x4d\x18"
LEGACY = b"\x02\x21\x4c\x18"
SKIPPABLE = 0x184D2A50
PRIME1, PRIME2, PRIME3, PRIME4, PRIME5 = 2654435761, 2246822519, 3266489917, 668265263, 374761393
MASK = 0xFFFFFFFF
BLOCKS = 1 << 20


class Malformed(Exception):
    pass


def rotl(value: int, bits: int) -> int:
    return ((value << bits) | (value >> (32 - bits))) & MASK


def xxh32(data: bytes, seed: int = 0) -> int:
    length = len(data)
    index = 0
    if length >= 16:
        v1, v2, v3, v4 = (
            (seed + PRIME1 + PRIME2) & MASK,
            (seed + PRIME2) & MASK,
            seed,
            (seed - PRIME1) & MASK,
        )
        while index + 16 <= length:
            lanes = struct.unpack_from("<IIII", data, index)
            v1 = (rotl((v1 + lanes[0] * PRIME2) & MASK, 13) * PRIME1) & MASK
            v2 = (rotl((v2 + lanes[1] * PRIME2) & MASK, 13) * PRIME1) & MASK
            v3 = (rotl((v3 + lanes[2] * PRIME2) & MASK, 13) * PRIME1) & MASK
            v4 = (rotl((v4 + lanes[3] * PRIME2) & MASK, 13) * PRIME1) & MASK
            index += 16
        total = (rotl(v1, 1) + rotl(v2, 7) + rotl(v3, 12) + rotl(v4, 18)) & MASK
    else:
        total = (seed + PRIME5) & MASK
    total = (total + length) & MASK
    while index + 4 <= length:
        total = (
            rotl((total + struct.unpack_from("<I", data, index)[0] * PRIME3) & MASK, 17) * PRIME4
        ) & MASK
        index += 4
    while index < length:
        total = (rotl((total + data[index] * PRIME5) & MASK, 11) * PRIME1) & MASK
        index += 1
    total ^= total >> 15
    total = (total * PRIME2) & MASK
    total ^= total >> 13
    total = (total * PRIME3) & MASK
    return total ^ (total >> 16)


def frame(data: bytes, offset: int, tags: set) -> int:
    flags, block_descriptor = data[offset + 4], data[offset + 5]
    if flags >> 6 != 1 or flags & 0x02:
        raise Malformed("Unsupported frame version or reserved flag bit")
    if block_descriptor & 0x8F or not 4 <= block_descriptor >> 4 <= 7:
        raise Malformed("Invalid block maximum size descriptor")
    position = offset + 6
    if flags & 0x08:
        position += 8
    if flags & 0x01:
        position += 4
    if position + 1 > len(data):
        raise Malformed("Truncated frame descriptor")
    if (xxh32(data[offset + 4 : position]) >> 8) & 0xFF != data[position]:
        raise Malformed("Header checksum mismatch")
    position += 1
    maximum = 1 << (8 + 2 * (block_descriptor >> 4))
    blocks = 0
    while True:
        if position + 4 > len(data):
            raise Malformed("Truncated block size")
        size = struct.unpack_from("<I", data, position)[0]
        position += 4
        if size == 0:
            break
        length = size & 0x7FFFFFFF
        if length > maximum:
            raise Malformed("Block larger than the declared maximum")
        block = data[position : position + length]
        if len(block) != length:
            raise Malformed("Block data outside file")
        position += length
        if flags & 0x10:
            if xxh32(block) != struct.unpack_from("<I", data, position)[0]:
                raise Malformed("Block checksum mismatch")
            position += 4
            tags.add("block_checksums")
        blocks += 1
        if blocks > BLOCKS:
            raise Malformed("Block budget exceeded")
    if flags & 0x04:
        position += 4
        tags.add("content_checksum")
    if position > len(data):
        raise Malformed("Truncated content checksum")
    return position


def legacy(data: bytes, offset: int) -> int:
    position = offset + 4
    while position + 4 <= len(data):
        size = struct.unpack_from("<I", data, position)[0]
        if size == 0x184C2102:  # next legacy frame
            return position
        if size & 0x80000000 or size > 8 * 1024 * 1024 + 15:
            raise Malformed("Legacy block size invalid")
        position += 4 + size
    if position != len(data):
        raise Malformed("Legacy block outside file")
    return position


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not (data.startswith(MAGIC) or data.startswith(LEGACY)):
        return None
    offset, frames, tags = 0, 0, set()
    try:
        while offset < len(data):
            if offset + 4 > len(data):
                raise Malformed("Truncated frame magic")
            magic = struct.unpack_from("<I", data, offset)[0]
            if data[offset : offset + 4] == MAGIC:
                if offset + 7 > len(data):
                    raise Malformed("Truncated frame descriptor")
                offset = frame(data, offset, tags)
            elif data[offset : offset + 4] == LEGACY:
                offset = legacy(data, offset)
                tags.add("legacy_frame")
            elif magic & 0xFFFFFFF0 == SKIPPABLE:
                offset += 8 + struct.unpack_from("<I", data, offset + 4)[0]
                tags.add("skippable_frame")
            else:
                raise Malformed("Bytes between frames are not a frame")
            if offset > len(data):
                raise Malformed("Frame outside file")
            frames += 1
    except (Malformed, struct.error) as error:
        return Observation("fail", str(error) or "Truncated frame", "lz4")
    return Observation(
        "pass",
        f"{frames} frames; descriptors, header checksums and block chains tile the file",
        "lz4",
        tuple(sorted(tags)),
        generic=True,  # an lz4 stream wraps arbitrary content
    )
