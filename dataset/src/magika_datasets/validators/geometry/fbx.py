# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Autodesk FBX: binary node records walked by end offsets, or the ASCII brace grammar."""

import re
import struct

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("fbx",)
SCOPE = "Binary: Kaydara magic and version, every node's end offset, property list length and name, nested nodes tiling their parent to the null record, top-level nodes to the null record and a footer (up to 4 KiB in FBX 6.x) ending with the footer magic. ASCII: FBX comment header, balanced braces and key: value lines; ASCII passes relabel only hinted samples"
CONTEXT_REQUIRED = False
MAGIC = b"Kaydara FBX Binary  \0\x1a\0"
NODES = 1_000_000
FOOTER_MAGIC = b"\xf8\x5a\x8c\x6a\xde\xf5\xd9\x7e\xec\xe9\x0c\xe3\x75\x8f\x29\x0b"


class Malformed(Exception):
    pass


def node(data: bytes, offset: int, end: int, wide: bool, state: dict) -> int | None:
    """Walk one node; returns the next offset, or None for a null record."""
    header = "<QQQ" if wide else "<III"
    size = struct.calcsize(header)
    if offset + size + 1 > end:
        raise Malformed("Truncated node header")
    node_end, _, props_length = struct.unpack_from(header, data, offset)
    name_length = data[offset + size]
    if node_end == 0 and props_length == 0 and name_length == 0:
        return None
    state["count"] += 1
    if state["count"] > NODES:
        raise Malformed("Node budget exceeded")
    position = offset + size + 1 + name_length
    if node_end > end or node_end < position + props_length:
        raise Malformed("Node end offset outside its parent")
    position += props_length
    if position < node_end:  # nested nodes until a null record
        while position < node_end:
            following = node(data, position, node_end, wide, state)
            if following is None:
                position += size + 1
                break
            position = following
    if position != node_end:
        raise Malformed("Nested nodes do not tile the node")
    return node_end


def binary(data: bytes) -> Observation:
    if len(data) < 27:
        return Observation("fail", "Truncated FBX header", "fbx")
    version = struct.unpack_from("<I", data, 23)[0]
    wide = version >= 7500
    size = (struct.calcsize("<QQQ") if wide else 12) + 1
    state = {"count": 0}
    offset = 27
    try:
        while True:
            following = node(data, offset, len(data), wide, state)
            if following is None:
                offset += size
                break
            offset = following
    except Malformed as error:
        return Observation("fail", str(error), "fbx")
    remaining = len(data) - offset
    if remaining < 16 or remaining > 4096 or not data.endswith(FOOTER_MAGIC):
        return Observation("fail", "Footer missing, oversized or without the footer magic", "fbx")
    return Observation(
        "pass", f"Binary FBX {version}: {state['count']} nodes tiling the file", "fbx", ("binary",)
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data.startswith(MAGIC):
        return binary(data)
    if not data.startswith(b"; FBX "):
        return None
    text = data.decode("utf-8", "replace")
    depth = 0
    for number, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if not stripped or stripped.startswith(";"):
            continue
        if stripped == "}":
            depth -= 1
            if depth < 0:
                return Observation("fail", f"Line {number}: unbalanced closing brace", "fbx")
            continue
        if not re.match(r"^[A-Za-z0-9_]+:\s*.*$", stripped):
            return Observation("fail", f"Line {number} is not a key: value entry", "fbx")
        if stripped.endswith("{"):
            depth += 1
    if depth:
        return Observation("fail", "Unbalanced braces at EOF", "fbx")
    return Observation("pass", "ASCII FBX grammar parsed to EOF", "fbx", ("ascii",))
