# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Adobe Photoshop documents: version 1 decoded with Pillow; PSB left inconclusive."""

from .._shared.pillow import SCOPE as SCOPE
from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("psd",)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"8BPS":
        return None
    version = int.from_bytes(data[4:6], "big")
    if version == 2:
        return Observation("inconclusive", "PSB large document format not decoded", "psd")
    if version != 1:
        return Observation("fail", "Unknown PSD version", "psd")
    return Observation(*decode(data, "PSD"), "psd")
