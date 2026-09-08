# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""3D Studio .3ds files: chunk tree with lengths tiling every container."""

import struct

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("3dsm",)
PREFIX_ONLY = True
SCOPE = "Main chunk 0x4D4D with every chunk's id and length, known container chunks recursed and their children tiling them, chunks tiling the file; meshes not interpreted"
CONTAINERS = {
    0x4D4D: 0,
    0x3D3D: 0,
    0xAFFF: 0,
    0x4000: None,
    0x4100: 0,
    0xB000: 0,
    0xB002: 0,
    0xB001: 0,
    0xB003: 0,
    0xB004: 0,
    0xB005: 0,
    0xB006: 0,
    0xB007: 0,
    0xA010: 0,
    0xA020: 0,
    0xA030: 0,
    0xA200: 0,
    0xA210: 0,
    0xA230: 0,
    0xA33A: 0,
    0xA33C: 0,
    0xA33D: 0,
    0xA33E: 0,
}
CHUNKS = 1_000_000


class Malformed(Exception):
    pass


def walk(data: bytes, start: int, end: int, depth: int, state: dict) -> None:
    offset = start
    while offset < end:
        state["count"] += 1
        if state["count"] > CHUNKS or depth > 16:
            raise Malformed("Chunk budget exceeded")
        if offset + 6 > end:
            raise Malformed("Truncated chunk header")
        identifier, length = struct.unpack_from("<HI", data, offset)
        if length < 6 or offset + length > end:
            raise Malformed(f"Chunk {identifier:#06x} exceeds its parent")
        skip = CONTAINERS.get(identifier, -1)
        if skip is not None and skip >= 0:
            walk(data, offset + 6 + skip, offset + length, depth + 1, state)
        elif skip is None:  # named object: NUL-terminated name then children
            name_end = data.find(b"\0", offset + 6, offset + length)
            if name_end < 0:
                raise Malformed("Object name unterminated")
            walk(data, name_end + 1, offset + length, depth + 1, state)
        offset += length


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:2] != b"MM" or len(data) < 6:
        return None
    length = struct.unpack_from("<I", data, 2)[0]
    if length != len(data):
        if "3dsm" in hints:
            return Observation("fail", "Main chunk length differs from file", "3dsm")
        return (
            None  # "MM" alone also opens big-endian TIFF; only a self-consistent length is evidence
        )
    state = {"count": 0}
    try:
        walk(data, 0, len(data), 0, state)
    except Malformed as error:
        return Observation("fail", str(error), "3dsm")
    return Observation("pass", f"{state['count']} chunks tiling the file", "3dsm")
