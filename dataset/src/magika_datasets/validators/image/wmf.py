# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Windows Metafiles: placeable header checksum, standard header, record walk and META_EOF."""

import struct

from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("wmf",)
SCOPE = "Placeable header checksum when present, standard header fields and size in words, every record's size and the terminating META_EOF; drawing semantics not rendered"
PLACEABLE = 0x9AC6CDD7


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    offset, tags = 0, ()
    if len(data) >= 22 and struct.unpack_from("<I", data)[0] == PLACEABLE:
        checksum = 0
        for word in struct.unpack_from("<10H", data):
            checksum ^= word
        if checksum != struct.unpack_from("<H", data, 20)[0]:
            return Observation("fail", "Placeable header checksum mismatch", "wmf")
        offset, tags = 22, ("placeable",)
    if len(data) < offset + 18:
        return None
    kind, header_words, version, words = struct.unpack_from("<HHHI", data, offset)
    if kind not in (1, 2) or header_words != 9 or version not in (0x0100, 0x0300):
        return None
    if words * 2 != len(data) - offset:
        return Observation("fail", "Size in words differs from file", "wmf")
    position, count, last = offset + 18, 0, None
    while position < len(data):
        count += 1
        if count > 65536:
            return Observation("inconclusive", "Record budget exceeded", "wmf")
        if position + 6 > len(data):
            return Observation("fail", "Truncated record header", "wmf")
        size, function = struct.unpack_from("<IH", data, position)
        if size < 3 or position + size * 2 > len(data):
            return Observation("fail", "Record size invalid or outside file", "wmf")
        last = (size, function)
        position += size * 2
    if last != (3, 0):
        return Observation("fail", "Last record is not META_EOF", "wmf")
    return Observation("pass", f"{count} WMF records bounded and terminated", "wmf", tags)
