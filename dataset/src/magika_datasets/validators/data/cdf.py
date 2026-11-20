# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""NASA Common Data Format: version 2 and 3 record chains tiling the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("cdf",)
SCOPE = "CDF 2.x (32-bit) or 3.x (64-bit) magic, compression magic, CDF descriptor record first, every internal record's size and type, records tiling the file; variable data not decoded"
MAGICS = {b"\xcd\xf2\x60\x02": 32, b"\xcd\xf3\x00\x01": 64}
COMPRESSED = b"\xcc\xcc\x00\x01"
TYPES = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, -1}
RECORDS = 1 << 20


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    width = MAGICS.get(data[:4])
    if width is None or len(data) < 16:
        return None
    if data[4:8] == COMPRESSED:
        return Observation("inconclusive", "Whole-file compressed CDF not expanded", "cdf")
    if data[4:8] != b"\x00\x00\xff\xff":
        return Observation("fail", "Unknown compression magic", "cdf")
    size_format = ">Q" if width == 64 else ">I"
    size_width = width // 8
    offset, records, kinds = 8, 0, set()
    while offset < len(data):
        if offset + size_width + 4 > len(data):
            return Observation("fail", "Record header truncated", "cdf")
        size = struct.unpack_from(size_format, data, offset)[0]
        kind = struct.unpack_from(">i", data, offset + size_width)[0]
        if records == 0 and kind != 1:
            return Observation("fail", "First record is not the CDF descriptor record", "cdf")
        if kind not in TYPES or size < size_width + 4:
            return Observation("fail", f"Record {records} has type {kind} and size {size}", "cdf")
        offset += size
        if offset > len(data):
            return Observation("fail", f"Record {records} extends past EOF", "cdf")
        records += 1
        kinds.add(kind)
        if records > RECORDS:
            return Observation("inconclusive", "Record budget exceeded", "cdf")
    return Observation(
        "pass",
        f"CDF {'3' if width == 64 else '2'}: {records} records of {len(kinds)} types tile the file",
        "cdf",
    )
