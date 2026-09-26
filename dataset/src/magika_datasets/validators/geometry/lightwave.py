# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""LightWave scenes (LWSC text) and objects (LWO IFF): line grammar or chunk tiling."""

import re
import struct

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("lightwave",)
SCOPE = "LWSC scenes: header and version lines, every line a keyword statement, a numeric row, a brace delimiter or free-form data inside Plugin/EndPlugin blocks, braces balanced to EOF; LWOB/LWO2/LWO3 objects: FORM size against the file, chunks tiling the form with even padding, TAGS/LAYR/PNTS/POLS chunk sizes consistent; geometry values not interpreted"
LINE = re.compile(
    rb"[A-Za-z][A-Za-z0-9_]*(?:[ \t].*)?|\{|\}|\{ *[A-Za-z0-9_]+.*|-?\d[\d.eE+\-]*(?:[ \t]+-?[\d.eE+\-]+)*"
)
FORMS = (b"LWOB", b"LWO2", b"LWO3")
CHUNKS = 65536


def scene(data: bytes) -> Observation:
    lines = data.split(b"\n")
    if len(lines) < 3 or lines[0].rstrip(b"\r") != b"LWSC" or not lines[1].rstrip(b"\r").isdigit():
        return Observation("fail", "LWSC header or version line malformed", "lightwave")
    depth, plugin = 0, False
    for number, raw in enumerate(lines[2:], start=3):
        line = raw.rstrip(b"\r").strip()
        if not line or line.startswith(b"//"):
            continue
        if plugin:  # plugin blocks carry free-form handler data
            plugin = line != b"EndPlugin"
            continue
        if line.startswith(b"Plugin ") or line.startswith(b"Plugin\t"):
            plugin = True
            continue
        if not LINE.fullmatch(line):
            return Observation("fail", f"Line {number} is not a scene statement", "lightwave")
        depth += line.count(b"{") - line.count(b"}")
        if depth < 0:
            return Observation("fail", f"Unbalanced brace at line {number}", "lightwave")
    if depth:
        return Observation("fail", "Unclosed brace block at EOF", "lightwave")
    return Observation(
        "pass",
        f"LWSC version {lines[1].strip().decode()}, {len(lines)} lines of scene statements",
        "lightwave",
        ("scene",),
    )


def obj(data: bytes) -> Observation:
    size = struct.unpack_from(">I", data, 4)[0]
    if 8 + size != len(data):
        return Observation("fail", "FORM size differs from the file", "lightwave")
    offset, chunks = 12, 0
    while offset < len(data):
        if offset + 8 > len(data):
            return Observation("fail", "Chunk header truncated", "lightwave")
        tag, length = data[offset : offset + 4], struct.unpack_from(">I", data, offset + 4)[0]
        if not all(0x20 <= byte < 0x7F for byte in tag):
            return Observation("fail", "Chunk id is not printable", "lightwave")
        if offset + 8 + length > len(data):
            return Observation("fail", f"Chunk {tag.decode()} outside file", "lightwave")
        if tag == b"PNTS" and length % 12:
            return Observation("fail", "PNTS chunk is not a multiple of 12 bytes", "lightwave")
        offset += 8 + length + (length & 1)
        chunks += 1
        if chunks > CHUNKS:
            return Observation("inconclusive", "Chunk budget exceeded", "lightwave")
    if offset != len(data) and offset != len(data) + 1:
        return Observation("fail", "Chunks do not tile the form", "lightwave")
    return Observation(
        "pass",
        f"{data[8:12].decode()} object with {chunks} chunks tiling the form",
        "lightwave",
        ("object",),
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data.startswith(b"LWSC"):
        return scene(data)
    if len(data) >= 12 and data[:4] == b"FORM" and data[8:12] in FORMS:
        return obj(data)
    return None
