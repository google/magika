# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Quite OK images: magic, end marker and Pillow decoding."""

from .._shared.pillow import SCOPE as SCOPE
from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("qoi",)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"qoif":
        return None
    if data[-8:] != b"\0" * 7 + b"\x01":
        return Observation("fail", "QOI end marker missing", "qoi")
    return Observation(*decode(data, "QOI"), "qoi")
