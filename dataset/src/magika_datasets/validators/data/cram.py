# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""CRAM alignments: file definition and container headers with CRC-32 tiling the file."""

import struct
import zlib

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("cram",)
SCOPE = "File definition (magic, version 2.x or 3.x, file id), every container header's ITF8/LTF8 fields, header CRC-32 for version 3, container payloads tiling the file, EOF container; compression blocks not decoded"
MAGIC = b"CRAM"
CONTAINERS = 1 << 20


class Malformed(Exception):
    pass


def itf8(data: bytes, offset: int) -> tuple[int, int]:
    first = data[offset]
    if first < 0x80:
        return first, offset + 1
    if first < 0xC0:
        return ((first & 0x3F) << 8) | data[offset + 1], offset + 2
    if first < 0xE0:
        return ((first & 0x1F) << 16) | (data[offset + 1] << 8) | data[offset + 2], offset + 3
    if first < 0xF0:
        return ((first & 0x0F) << 24) | (data[offset + 1] << 16) | (data[offset + 2] << 8) | data[
            offset + 3
        ], offset + 4
    value = (
        ((first & 0x0F) << 28)
        | (data[offset + 1] << 20)
        | (data[offset + 2] << 12)
        | (data[offset + 3] << 4)
        | (data[offset + 4] & 0x0F)
    )
    return value, offset + 5


def ltf8(data: bytes, offset: int) -> tuple[int, int]:
    first = data[offset]
    for bits in range(8):
        if not first & (0x80 >> bits):
            width = bits
            break
    else:
        width = 8
    if width == 0:
        return first, offset + 1
    value = (first & (0x7F >> width)) if width < 8 else 0
    for index in range(width):
        value = (value << 8) | data[offset + 1 + index]
    return value, offset + 1 + width


def container(data: bytes, offset: int, major: int) -> tuple[int, int]:
    """(payload start, payload length) for the container header at offset."""
    length = struct.unpack_from("<i", data, offset)[0]
    position = offset + 4
    for _ in range(3):  # reference sequence id, alignment start, alignment span
        _, position = itf8(data, position)
    _, position = itf8(data, position)  # number of records
    _, position = ltf8(data, position)  # record counter
    _, position = ltf8(data, position)  # bases
    _, position = itf8(data, position)  # number of blocks
    landmarks, position = itf8(data, position)
    if landmarks > 1 << 20:
        raise Malformed("Landmark count implausible")
    for _ in range(landmarks):
        _, position = itf8(data, position)
    if major >= 3:
        stored = struct.unpack_from("<I", data, position)[0]
        if zlib.crc32(data[offset:position]) != stored:
            raise Malformed("Container header CRC-32 mismatch")
        position += 4
    if length < 0:
        raise Malformed("Negative container length")
    return position, length


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC) or len(data) < 26:
        return None
    major, minor = data[4], data[5]
    if major not in (2, 3):
        return Observation("inconclusive", f"CRAM version {major}.{minor} not modelled", "cram")
    offset, containers = 26, 0
    try:
        while offset < len(data):
            start, length = container(data, offset, major)
            offset = start + length
            if offset > len(data):
                raise Malformed(f"Container {containers} extends past EOF")
            containers += 1
            if containers > CONTAINERS:
                return Observation("inconclusive", "Container budget exceeded", "cram")
    except (Malformed, IndexError, struct.error) as error:
        return Observation("fail", str(error) or "Container header truncated", "cram")
    if containers < 2:
        return Observation("fail", "Fewer than two containers (header and EOF)", "cram")
    return Observation(
        "pass",
        f"CRAM {major}.{minor} with {containers} containers tiling the file"
        + ("; header CRC-32 values verified" if major >= 3 else ""),
        "cram",
    )
