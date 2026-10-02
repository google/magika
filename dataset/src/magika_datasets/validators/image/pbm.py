# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Netpbm (PBM/PGM/PPM) images decoded with Pillow; the taxonomy id is pbm for the family."""

from .._shared.pillow import SCOPE as SCOPE
from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("pbm",)
PREFIX_ONLY = True


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:1] != b"P" or data[1:2] not in b"123456" or data[2:3] not in b" \t\r\n":
        return None
    return Observation(*decode(data, "PPM"), "pbm")
