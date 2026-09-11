# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Windows Enhanced Metafiles: header record, record walk and EMR_EOF; nothing is rendered."""

import struct

from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("emf",)
SCOPE = "EMR_HEADER signature and declared size, every record's size and alignment, record count and terminating EMR_EOF; drawing semantics not rendered"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 88 or data[:4] != b"\1\0\0\0" or data[40:44] != b" EMF":
        return None
    header_size, declared, records = (
        struct.unpack_from("<I", data, 4)[0],
        *struct.unpack_from("<II", data, 48),
    )
    if header_size < 88 or header_size % 4 or declared != len(data):
        return Observation("fail", "Header size or declared byte count differs from file", "emf")
    offset, count, last = 0, 0, None
    while offset < len(data):
        count += 1
        if count > 65536:
            return Observation("inconclusive", "Record budget exceeded", "emf")
        if offset + 8 > len(data):
            return Observation("fail", "Truncated record header", "emf")
        kind, size = struct.unpack_from("<II", data, offset)
        if size < 8 or size % 4 or offset + size > len(data):
            return Observation("fail", "Record size invalid or outside file", "emf")
        last = kind
        offset += size
    if last != 14:
        return Observation("fail", "Last record is not EMR_EOF", "emf")
    if count != records:
        return Observation("fail", "Record count differs from header", "emf")
    return Observation("pass", f"{count} EMF records bounded and terminated", "emf")
