# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""TIFF identification and bounded pixel decoding."""

from .._shared.pillow import SCOPE as SCOPE
from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("tiff",)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] not in (b"II*\x00", b"MM\x00*", b"II+\x00", b"MM\x00+"):
        return None
    return Observation(*decode(data, "TIFF"), "tiff")
