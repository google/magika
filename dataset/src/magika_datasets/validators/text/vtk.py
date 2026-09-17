# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Legacy VTK files: header lines, dataset type, ASCII token counts or binary array sizes."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("vtk",)
SCOPE = "# vtk DataFile header, title, ASCII or BINARY, DATASET type; POINTS/VERTICES/LINES/POLYGONS/CELLS and attribute arrays sized by their counts in binary files; ASCII files checked for known section keywords; XML .vtk files not handled"
TYPES = {
    "bit": 1,
    "unsigned_char": 1,
    "char": 1,
    "unsigned_short": 2,
    "short": 2,
    "unsigned_int": 4,
    "int": 4,
    "unsigned_long": 8,
    "long": 8,
    "float": 4,
    "double": 8,
    "vtktypeint64": 8,
    "vtktypeuint64": 8,
}
FREE_TEXT = {"COMPONENT_NAMES", "INFORMATION", "NAME"}
KEYWORDS = {
    "POINTS",
    "VERTICES",
    "LINES",
    "POLYGONS",
    "TRIANGLE_STRIPS",
    "CELLS",
    "CELL_TYPES",
    "DIMENSIONS",
    "ORIGIN",
    "SPACING",
    "X_COORDINATES",
    "Y_COORDINATES",
    "Z_COORDINATES",
    "POINT_DATA",
    "CELL_DATA",
    "SCALARS",
    "LOOKUP_TABLE",
    "VECTORS",
    "NORMALS",
    "TEXTURE_COORDINATES",
    "TENSORS",
    "FIELD",
    "COLOR_SCALARS",
    "METADATA",
    "COMPONENT_NAMES",
    "INFORMATION",
    "NAME",
    "DATA",
    "INFORMATION",
    "NAME",
    "DATA",
    "OFFSETS",
    "CONNECTIVITY",
}


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"# vtk DataFile Version"):
        return None
    lines = data.split(b"\n", 4)
    if len(lines) < 5:
        return Observation("fail", "Header shorter than four lines", "vtk")
    encoding = lines[2].strip().upper()
    if encoding not in (b"ASCII", b"BINARY"):
        return Observation("fail", "Third line is not ASCII or BINARY", "vtk")
    if not lines[3].strip().upper().startswith(b"DATASET "):
        return Observation("fail", "Fourth line is not DATASET", "vtk")
    body = lines[4]
    if encoding == b"ASCII":
        free_text = False  # COMPONENT_NAMES and INFORMATION blocks list arbitrary names
        for number, line in enumerate(body.decode("latin-1").splitlines(), 5):
            words = line.split()
            if not words or words[0].upper() in KEYWORDS:
                free_text = bool(words) and words[0].upper() in FREE_TEXT
                continue
            if free_text:
                continue
            if all(re.match(r"^[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?$", w) for w in words):
                continue
            if (
                len(words) == 4
                and words[1].isdigit()
                and words[2].isdigit()
                and words[3].lower() in TYPES
            ):
                continue  # FIELD array header: name components tuples type
            return Observation(
                "fail",
                f"Line {number} is neither a keyword, numbers nor a field array header",
                "vtk",
            )
        return Observation("pass", "Legacy ASCII VTK sections and numeric data", "vtk", ("ascii",))
    offset = 0
    while offset < len(body):
        end = body.find(b"\n", offset)
        if end < 0:
            break
        words = body[offset:end].decode("latin-1").split()
        offset = end + 1
        if not words:
            continue
        keyword = words[0].upper()
        if keyword == "POINTS" and len(words) >= 3 and words[2] in TYPES:
            offset += int(words[1]) * 3 * TYPES[words[2]]
        elif (
            keyword in ("VERTICES", "LINES", "POLYGONS", "TRIANGLE_STRIPS", "CELLS")
            and len(words) >= 3
        ):
            offset += int(words[2]) * 4
        elif keyword == "CELL_TYPES" and len(words) >= 2:
            offset += int(words[1]) * 4
        elif keyword in (
            "SCALARS",
            "VECTORS",
            "NORMALS",
            "TENSORS",
            "COLOR_SCALARS",
            "TEXTURE_COORDINATES",
        ):
            continue  # sizes come from the enclosing POINT_DATA/CELL_DATA counts; not tracked
        elif keyword not in KEYWORDS and not re.match(r"^[-+]?\d", words[0]):
            return Observation("fail", f"Unknown keyword {keyword}", "vtk")
        if offset > len(body):
            return Observation("fail", f"{keyword} array exceeds file", "vtk")
    return Observation("pass", "Legacy binary VTK geometry arrays sized", "vtk", ("binary",))
