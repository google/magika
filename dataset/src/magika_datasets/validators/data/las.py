# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""LAS point clouds: header, offset to point data and point records against the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("las",)
SCOPE = "LASF signature, version 1.0 to 1.4, header size, offset to point data at or after the header and VLRs, point record length times count equal to the point data (extended VLRs after it in 1.4); LAZ files checked for the LASzip VLR and chunk table offset; points not interpreted"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"LASF" or len(data) < 227:
        return None
    major, minor = data[24], data[25]
    if major != 1 or minor > 4:
        return Observation("fail", f"Unsupported LAS version {major}.{minor}", "las")
    header_size = struct.unpack_from("<H", data, 94)[0]
    offset, vlrs = struct.unpack_from("<II", data, 96)
    fmt, length, count = struct.unpack_from("<BHI", data, 104)
    if header_size < 227 or offset < header_size or offset > len(data):
        return Observation("fail", "Header size or point data offset invalid", "las")
    if minor >= 4:
        count = struct.unpack_from("<Q", data, 247)[0] or count
        evlr_offset, evlr_count = struct.unpack_from("<QI", data, 235)
    else:
        evlr_offset, evlr_count = 0, 0
    if (
        fmt & 0x80
    ):  # LASzip: compressed points; the first 8 bytes at the offset locate the chunk table
        if offset + 8 > len(data):
            return Observation("fail", "LAZ point data offset outside file", "las")
        table = struct.unpack_from("<q", data, offset)[0]
        if table != -1 and not offset + 8 <= table <= len(data) - 8:
            return Observation("fail", "LAZ chunk table offset outside file", "las")
        if b"laszip encoded" not in data[header_size:offset]:
            return Observation("fail", "LAZ points without a LASzip VLR", "las")
        return Observation(
            "pass",
            f"LAZ 1.{minor}: {count} compressed points of format {fmt & 0x7F}; chunk table located",
            "las",
            ("laz",),
        )
    end = offset + count * length
    if end > len(data):
        return Observation("fail", "Point records exceed file", "las")
    if evlr_count:
        if evlr_offset < end or evlr_offset > len(data):
            return Observation("fail", "Extended VLRs outside file", "las")
    elif end != len(data):
        return Observation("fail", f"{len(data) - end} bytes after the point records", "las")
    return Observation("pass", f"LAS 1.{minor}: {count} points of format {fmt}, {vlrs} VLRs", "las")
