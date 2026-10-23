# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Polygon File Format: header grammar and element data walked for binary and ASCII."""

import struct

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("ply",)
SCOPE = "ply header with format, elements and properties to end_header, binary element data walked property by property including list counts to exactly EOF, ASCII element instances counted line by line; geometry not interpreted"
SIZES = {
    "char": "b",
    "int8": "b",
    "uchar": "B",
    "uint8": "B",
    "short": "h",
    "int16": "h",
    "ushort": "H",
    "uint16": "H",
    "int": "i",
    "int32": "i",
    "uint": "I",
    "uint32": "I",
    "float": "f",
    "float32": "f",
    "double": "d",
    "float64": "d",
}
INSTANCES = 10_000_000


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"ply\n") and not data.startswith(b"ply\r\n"):
        return None
    end = data.find(b"end_header")
    if end < 0:
        return Observation("fail", "Missing end_header", "ply")
    newline = data.find(b"\n", end)
    if newline < 0:
        return Observation("fail", "end_header not terminated", "ply")
    body_start = newline + 1
    order, elements = None, []
    for line in data[:end].decode("ascii", "replace").splitlines()[1:]:
        words = line.split()
        if not words or words[0] in ("comment", "obj_info"):
            continue
        if words[0] == "format":
            if len(words) < 2 or words[1] not in (
                "ascii",
                "binary_little_endian",
                "binary_big_endian",
            ):
                return Observation("fail", "Unknown format line", "ply")
            order = {"ascii": None, "binary_little_endian": "<", "binary_big_endian": ">"}[words[1]]
        elif words[0] == "element":
            if len(words) != 3 or not words[2].isdigit():
                return Observation("fail", "Invalid element line", "ply")
            elements.append((words[1], int(words[2]), []))
        elif words[0] == "property":
            if not elements:
                return Observation("fail", "Property before any element", "ply")
            if len(words) == 5 and words[1] == "list":
                if words[2] not in SIZES or words[3] not in SIZES:
                    return Observation("fail", "Unknown list property types", "ply")
                elements[-1][2].append(("list", SIZES[words[2]], SIZES[words[3]]))
            elif len(words) == 3 and words[1] in SIZES:
                elements[-1][2].append(("scalar", SIZES[words[1]], None))
            else:
                return Observation("fail", "Invalid property line", "ply")
        else:
            return Observation("fail", f"Unknown header keyword {words[0]}", "ply")
    if (
        order is None
        and not any(True for _ in [0])
        or "format" not in data[:end].decode("ascii", "replace")
    ):
        return Observation("fail", "Missing format line", "ply")
    total = sum(count for _, count, _ in elements)
    if total > INSTANCES:
        return Observation("inconclusive", "Element budget exceeded", "ply")
    if order is None:
        lines = [line for line in data[body_start:].splitlines() if line.strip()]
        if len(lines) != total:
            return Observation(
                "fail", f"{len(lines)} data lines for {total} element instances", "ply"
            )
        return Observation("pass", f"ASCII ply: {total} element instances", "ply", ("ascii",))
    offset = body_start
    try:
        for _, count, properties in elements:
            for _ in range(count):
                for kind, fmt, item in properties:
                    if kind == "scalar":
                        offset += struct.calcsize(fmt)
                    else:
                        n = struct.unpack_from(order + fmt, data, offset)[0]
                        offset += struct.calcsize(fmt) + n * struct.calcsize(item)
                if offset > len(data):
                    raise struct.error
    except struct.error:
        return Observation("fail", "Element data exceeds file", "ply")
    if offset != len(data):
        return Observation("fail", "Bytes after the last element", "ply")
    return Observation(
        "pass", f"Binary ply: {total} element instances sized exactly", "ply", ("binary",)
    )
