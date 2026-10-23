# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Maya scenes: ASCII (.ma) MEL statements or binary (.mb) IFF containers."""

import re
import struct

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("maya",)
SCOPE = "ASCII: `//Maya ASCII` header, every statement a known scene command terminated by a semicolon with balanced quotes; binary: FOR4/FOR8 form with Maya type, chunks tiling the form with 4- or 8-byte alignment; attribute values not interpreted"
HEADER = re.compile(rb"//Maya ASCII [^\r\n]+ scene\r?\n")
COMMANDS = {
    b"requires",
    b"currentUnit",
    b"fileInfo",
    b"createNode",
    b"setAttr",
    b"connectAttr",
    b"disconnectAttr",
    b"addAttr",
    b"select",
    b"relationship",
    b"parent",
    b"lockNode",
    b"applyMetadata",
    b"dataStructure",
    b"rename",
    b"file",
    b"namespace",
    b"deleteAttr",
    b"instancer",
    b"animLayer",
    b"ogsRender",
    b"container",
    b"createNodes",
    b"connectionAttr",
    b"editAttr",
    b"setDynamic",
    b"shadingConnection",
    b"cacheFile",
}
STATEMENTS = 1 << 20


def ascii_scene(data: bytes) -> Observation:
    position, statements, unknown = HEADER.match(data).end(), 0, set()
    length = len(data)
    while position < length:
        byte = data[position]
        if byte in b" \t\r\n":
            position += 1
            continue
        if data.startswith(b"//", position):
            end = data.find(b"\n", position)
            position = length if end < 0 else end + 1
            continue
        match = re.compile(rb"[A-Za-z_][A-Za-z0-9_]*").match(data, position)
        if match is None:
            return Observation(
                "fail", f"Statement at byte {position} does not start with a command", "maya"
            )
        command = match.group()
        if command not in COMMANDS:
            unknown.add(command)
            if len(unknown) > 8:
                return Observation("fail", f"Unknown commands {sorted(unknown)[:4]}", "maya")
        position = match.end()
        quoted = False
        while position < length:  # to the terminating semicolon outside quotes
            byte = data[position]
            if byte == 0x5C and quoted:
                position += 2
                continue
            if byte == 0x22:
                quoted = not quoted
            elif byte == 0x3B and not quoted:
                break
            position += 1
        else:
            return Observation("fail", "Statement is not terminated by a semicolon", "maya")
        position += 1
        statements += 1
        if statements > STATEMENTS:
            return Observation("inconclusive", "Statement budget exceeded", "maya")
    if statements == 0:
        return Observation("fail", "Scene has no statements", "maya")
    return Observation("pass", f"Maya ASCII scene with {statements} statements", "maya", ("ascii",))


def binary_scene(data: bytes) -> Observation:
    wide = data[:4] == b"FOR8"
    if wide:
        size = struct.unpack_from(">Q", data, 8)[0]
        header, align = 16, 8
    else:
        size = struct.unpack_from(">I", data, 4)[0]
        header, align = 8, 4
    if header + size != len(data) and header + size + (-(header + size)) % align != len(data):
        return Observation("fail", "Form size differs from the file", "maya")
    offset, chunks = header + 4, 0
    while offset + header <= len(data):
        tag = data[offset : offset + 4]
        length = (
            struct.unpack_from(">Q", data, offset + 8)[0]
            if wide
            else struct.unpack_from(">I", data, offset + 4)[0]
        )
        if not all(0x20 <= byte < 0x7F for byte in tag):
            return Observation("fail", "Chunk id is not printable", "maya")
        if tag in (b"FOR4", b"FOR8", b"LIS4", b"LIS8"):
            offset += header + 4  # descend into the group
            continue
        if offset + header + length > len(data):
            return Observation("fail", f"Chunk {tag.decode()} outside file", "maya")
        offset += header + length + (-(header + length)) % align
        chunks += 1
    if offset != len(data):
        return Observation("fail", "Chunks do not tile the form", "maya")
    return Observation(
        "pass", f"Maya binary scene with {chunks} chunks tiling the form", "maya", ("binary",)
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if HEADER.match(data):
        return ascii_scene(data)
    if len(data) >= 16 and data[:4] == b"FOR4" and data[8:12] == b"Maya":
        return binary_scene(data)
    if len(data) >= 24 and data[:4] == b"FOR8" and data[16:20] == b"Maya":
        return binary_scene(data)
    return None
