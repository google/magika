# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ICC colour profiles: declared size, signature and tag table bounds."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("icc",)
SCOPE = "Profile size equal to the file, acsp signature, tag count and every tag's offset and size inside the profile; colour transforms not evaluated"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 132 or data[36:40] != b"acsp":
        return None
    size = struct.unpack_from(">I", data)[0]
    if size != len(data):
        return Observation("fail", "Profile size differs from file", "icc")
    count = struct.unpack_from(">I", data, 128)[0]
    if count > 4096 or 132 + 12 * count > len(data):
        return Observation("fail", "Tag table outside profile", "icc")
    for index in range(count):
        _, offset, length = struct.unpack_from(">4sII", data, 132 + 12 * index)
        if offset < 132 + 12 * count or offset + length > len(data):
            return Observation("fail", "Tag data outside profile", "icc")
    return Observation("pass", f"{count} ICC tags bounded inside the declared profile", "icc")
