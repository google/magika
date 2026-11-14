# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Sun/NeXT audio: header fields and data size against the file."""

import struct

from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("au",)
SCOPE = "Magic, data offset, data size (or unknown marker) equal to the remaining bytes, known encoding, nonzero rate and channels; samples not decoded"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b".snd" or len(data) < 24:
        return None
    offset, size, encoding, rate, channels = struct.unpack_from(">IIIII", data, 4)
    if offset < 24 or offset > len(data) or not 1 <= encoding <= 27 or not rate or not channels:
        return Observation("fail", "Invalid AU header fields", "au")
    if size != 0xFFFFFFFF and offset + size != len(data):
        return Observation("fail", "Data size differs from remaining bytes", "au")
    return Observation("pass", "AU header and data extent checked", "au")
