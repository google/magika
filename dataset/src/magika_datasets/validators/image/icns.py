# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Apple icon suites: declared length, icon table bounds and Pillow decoding."""

from .._shared.pillow import SCOPE as SCOPE
from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("icns",)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"icns":
        return None
    if int.from_bytes(data[4:8], "big") != len(data):
        return Observation("fail", "Declared length differs from file size", "icns")
    offset, entries = 8, 0
    while offset < len(data):
        entries += 1
        length = int.from_bytes(data[offset + 4 : offset + 8], "big")
        if entries > 256 or length < 8 or offset + length > len(data):
            return Observation("fail", "Icon table entry outside file bounds", "icns")
        offset += length
    return Observation(*decode(data, "ICNS"), "icns")
