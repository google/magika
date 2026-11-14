# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""MATLAB Level 5 MAT-files: header and data elements tiling the file."""

import struct
import zlib

from .._shared import decompress
from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("mat",)
SCOPE = "128-byte header with version 0x0100 and endian indicator, every data element's type and size (small element format included) padded to 8 bytes tiling the file, compressed elements bounded-inflated; v7.3 HDF5 files inconclusive"
TYPES = set(range(1, 19))
ELEMENTS = 1_000_000


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"MATLAB ") or len(data) < 128:
        return None
    if data.startswith(b"MATLAB 7.3"):
        return Observation("inconclusive", "MAT v7.3 is HDF5; not decoded", "mat")
    endian = data[126:128]
    if endian == b"IM":
        order = "<"
    elif endian == b"MI":
        order = ">"
    else:
        return Observation("fail", "Invalid endian indicator", "mat")
    if struct.unpack_from(order + "H", data, 124)[0] != 0x0100:
        return Observation("fail", "Unknown MAT version", "mat")
    offset, count = 128, 0
    while offset < len(data):
        count += 1
        if count > ELEMENTS:
            return Observation("inconclusive", "Element budget exceeded", "mat")
        if offset + 8 > len(data):
            return Observation("fail", "Truncated element tag", "mat")
        kind, size = struct.unpack_from(order + "II", data, offset)
        if kind >> 16:  # small data element: 2-byte size and type, 4-byte data
            size, kind = (
                (kind >> 16, kind & 0xFFFF) if order == "<" else (kind & 0xFFFF, kind >> 16)
            )
            offset += 8
            if size > 4 or kind not in TYPES:
                return Observation("fail", "Invalid small data element", "mat")
            continue
        if kind not in TYPES:
            return Observation("fail", f"Unknown element type {kind}", "mat")
        end = offset + 8 + size
        if end > len(data):
            return Observation("fail", "Element exceeds file", "mat")
        if kind == 15:
            try:
                decompress.drain(zlib.decompressobj(), data[offset + 8 : end], decompress.LIMIT)
            except decompress.Budget as error:
                return Observation("inconclusive", str(error), "mat")
            except (decompress.Truncated, zlib.error):
                return Observation("fail", "Compressed element does not inflate", "mat")
        offset = (
            end if kind == 15 else end + (-size % 8)
        )  # MATLAB writes compressed elements unpadded
        if offset > len(data) and end == len(data):
            offset = end  # the final element may omit its padding
    if offset != len(data):
        return Observation("fail", "Element padding exceeds file", "mat")
    return Observation("pass", f"{count} MAT elements tiling the file", "mat")
