# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Stereolithography files: binary facet count arithmetic or ASCII facet grammar."""

import re
import struct

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("stl",)
SCOPE = "Binary: 80-byte header plus 50 bytes per declared facet equal to the file. ASCII: solid/endsolid with facet normal, outer loop, three vertices, endloop, endfacet grammar to EOF. Binary has no magic, so failures need a hint"
PREFIX_ONLY = True
NUMBER = r"[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?"
FACET = re.compile(
    rf"\s*facet\s+normal\s+{NUMBER}\s+{NUMBER}\s+{NUMBER}\s+outer\s+loop\s+(?:vertex\s+{NUMBER}\s+{NUMBER}\s+{NUMBER}\s+){{3}}endloop\s+endfacet\s+"
)


def ascii(data: bytes) -> Observation:
    text = data.decode("ascii", "replace")
    match = re.match(r"\s*solid\b[^\n]*\n", text)
    if not match:
        return Observation("fail", "ASCII STL without a solid line", "stl")
    position, facets = match.end(), 0
    while True:
        facet = FACET.match(text, position)
        if not facet:
            break
        facets += 1
        position = facet.end()
    tail = text[position:].strip()
    if not tail.startswith("endsolid"):
        return Observation("fail", f"Expected endsolid after {facets} facets", "stl")
    if "\n" in tail.strip() and not re.fullmatch(r"endsolid[^\n]*", tail.strip()):
        return Observation("fail", "Text after endsolid", "stl")
    return Observation("pass", f"ASCII STL with {facets} facets", "stl", ("ascii",))


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) >= 84:
        count = struct.unpack_from("<I", data, 80)[0]
        if 84 + 50 * count == len(data) and count:
            return Observation(
                "pass", f"Binary STL with {count} facets sized exactly", "stl", ("binary",)
            )
    if data.lstrip().startswith(b"solid"):
        return ascii(data)
    if "stl" in hints and len(data) >= 84:
        return Observation(
            "fail", "Neither binary facet arithmetic nor ASCII grammar matches", "stl"
        )
    return None
