# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""GIMP XCF: header, property lists and every layer, channel, hierarchy and tile offset."""

import re
import struct

from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("xcf",)
SCOPE = "Magic and version, image properties, layer and channel offsets, each layer's properties, hierarchy, levels and tile offsets resolving inside the file; tile bytes not decoded"
MAGIC = b"gimp xcf "


class Bounds(Exception):
    pass


def u32(data: bytes, offset: int) -> tuple[int, int]:
    if offset + 4 > len(data):
        raise Bounds("Field outside file")
    return struct.unpack_from(">I", data, offset)[0], offset + 4


def pointer(data: bytes, offset: int, wide: bool) -> tuple[int, int]:
    if wide:
        if offset + 8 > len(data):
            raise Bounds("Field outside file")
        return struct.unpack_from(">Q", data, offset)[0], offset + 8
    return u32(data, offset)


def properties(data: bytes, offset: int) -> int:
    for _ in range(4096):
        kind, offset = u32(data, offset)
        length, offset = u32(data, offset)
        if kind == 0:
            return offset
        offset += length
        if offset > len(data):
            raise Bounds("Property payload outside file")
    raise Bounds("Property budget exceeded")


def pointers(data: bytes, offset: int, wide: bool) -> tuple[list[int], int]:
    found = []
    for _ in range(4096):
        value, offset = pointer(data, offset, wide)
        if value == 0:
            return found, offset
        if value >= len(data):
            raise Bounds("Offset outside file")
        found.append(value)
    raise Bounds("Offset budget exceeded")


def hierarchy(data: bytes, offset: int, wide: bool):
    _, offset = u32(data, offset)
    _, offset = u32(data, offset)
    _, offset = u32(data, offset)
    levels, _ = pointers(data, offset, wide)
    if not levels:
        raise Bounds("Hierarchy without levels")
    # Only the first level carries tiles; GIMP terminates the unused levels with a
    # 32-bit zero even in the 64-bit-pointer versions, so they are bounds-checked only.
    _, position = u32(data, levels[0])
    _, position = u32(data, position)
    pointers(data, position, wide)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    tag = data[9:14]
    match = re.fullmatch(rb"(file|v(\d{3}))\0", tag)
    if not match:
        return Observation("fail", "Unknown XCF version tag", "xcf")
    version = int(match.group(2) or 0)
    wide = version >= 11
    try:
        offset = 14
        _, offset = u32(data, offset)
        _, offset = u32(data, offset)
        base, offset = u32(data, offset)
        if base > 2:
            return Observation("fail", "Unknown base type", "xcf")
        if version >= 4:
            _, offset = u32(data, offset)
        offset = properties(data, offset)
        layers, offset = pointers(data, offset, wide)
        channels, offset = pointers(data, offset, wide)
        for index, position in enumerate(layers + channels):
            _, position = u32(data, position)
            _, position = u32(data, position)
            if index < len(layers):
                _, position = u32(data, position)  # layer type; channels have none
            length, position = u32(data, position)
            position += length
            position = properties(data, position)
            tree, position = pointer(data, position, wide)
            if tree >= len(data):
                raise Bounds("Hierarchy offset outside file")
            hierarchy(data, tree, wide)
    except Bounds as error:
        return Observation("fail", str(error), "xcf")
    except struct.error:
        return Observation("fail", "Truncated structure", "xcf")
    return Observation(
        "pass", f"XCF v{version}: {len(layers)} layers, {len(channels)} channels resolved", "xcf"
    )
