# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Targa images: header sanity and bounded decoding; no magic, so a hint is required."""

from .._shared.pillow import SCOPE as SCOPE
from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("tga",)
REQUIRES_HINT = True
FOOTER = b"TRUEVISION-XFILE.\0"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 18:
        return None
    if (
        data[1] not in (0, 1)
        or data[2] not in (1, 2, 3, 9, 10, 11)
        or data[16] not in (8, 15, 16, 24, 32)
        or not int.from_bytes(data[12:14], "little")
        or not int.from_bytes(data[14:16], "little")
    ):
        return Observation("fail", "Invalid TGA header fields", "tga")
    status, detail = decode(data, "TGA")
    return Observation(status, detail, "tga", ("tga_footer",) if data.endswith(FOOTER) else ())
