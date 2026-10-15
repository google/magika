# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""JPEG identification and bounded pixel decoding."""

from .._shared.pillow import SCOPE as SCOPE
from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("jpeg",)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"\xff\xd8\xff"):
        return None
    if not data.endswith(b"\xff\xd9"):
        return Observation("fail", "JPEG end marker missing or trailing bytes present", "jpeg")
    return Observation(*decode(data, "JPEG"), "jpeg")
