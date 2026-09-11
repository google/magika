# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Windows icon and cursor resources: directory bounds and decoding of every image."""

import struct

from .._shared.pillow import SCOPE as SCOPE
from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("ico",)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] not in (b"\0\0\1\0", b"\0\0\2\0") or data[4:6] == b"\0\0":
        return None
    kind = data[2]
    count = struct.unpack_from("<H", data, 4)[0]
    if 6 + 16 * count > len(data):
        return Observation("fail", "Icon directory exceeds file bounds", "ico")
    for index in range(count):
        size, offset = struct.unpack_from("<II", data, 6 + 16 * index + 8)
        if offset < 6 + 16 * count or offset + size > len(data) or not size:
            return Observation("fail", "Icon image outside file bounds", "ico")
    status, detail = decode(data, "CUR" if kind == 2 else "ICO")
    return Observation(status, detail, "ico", ("cursor",) if kind == 2 else ())
