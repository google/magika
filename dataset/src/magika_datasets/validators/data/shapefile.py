# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ESRI shapefiles (.shp): file header, record headers and content lengths tiling the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("shapefile",)
SCOPE = ".shp or .shx: file code 9994, file length in 16-bit words equal to the file, known shape type, every record's number and content length tiling the file, record shape types consistent with the header; geometry not decoded"
SHAPES = {0, 1, 3, 5, 8, 11, 13, 15, 18, 21, 23, 25, 28, 31}
RECORDS = 10_000_000


def index(data: bytes, shape: int) -> Observation:
    """A .shx index: 8-byte (offset, content length) entries in ascending word offsets."""
    entries = (len(data) - 100) // 8
    if entries > RECORDS:
        return Observation("inconclusive", "Index entry budget exceeded", "shapefile")
    expected = 50
    for number in range(entries):
        offset, words = struct.unpack_from(">II", data, 100 + 8 * number)
        if offset != expected or words < 2:
            return Observation(
                "fail", f"Index entry {number + 1} offset or length invalid", "shapefile"
            )
        expected = offset + 4 + words
    return Observation(
        "pass",
        f"{entries} index entries with contiguous record offsets; shape type {shape}",
        "shapefile",
        ("index",),
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 100 or struct.unpack_from(">I", data)[0] != 9994:
        return None
    length = struct.unpack_from(">I", data, 24)[0] * 2
    version, shape = struct.unpack_from("<II", data, 28)
    if length != len(data):
        return Observation("fail", "File length differs from header", "shapefile")
    if version != 1000 or shape not in SHAPES:
        return Observation("fail", "Unknown version or shape type", "shapefile")
    if (
        (len(data) - 100) % 8 == 0
        and len(data) >= 108
        and struct.unpack_from(">I", data, 100)[0] == 50
    ):
        return index(data, shape)
    offset, count = 100, 0
    while offset < len(data):
        count += 1
        if count > RECORDS:
            return Observation("inconclusive", "Record budget exceeded", "shapefile")
        if offset + 12 > len(data):
            return Observation("fail", "Truncated record header", "shapefile")
        number, words = struct.unpack_from(">II", data, offset)
        record_shape = struct.unpack_from("<I", data, offset + 8)[0]
        if number != count or offset + 8 + 2 * words > len(data) or words < 2:
            return Observation("fail", f"Record {count} numbering or length invalid", "shapefile")
        if record_shape not in (0, shape):
            return Observation(
                "fail", f"Record {count} shape type differs from header", "shapefile"
            )
        offset += 8 + 2 * words
    return Observation(
        "pass", f"{count} shape records tiling the file; shape type {shape}", "shapefile"
    )
