# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""BMP identification and bounded pixel decoding."""

from .._shared.pillow import SCOPE as SCOPE
from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("bmp",)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"BM"):
        return None
    return Observation(*decode(data, "BMP"), "bmp")
