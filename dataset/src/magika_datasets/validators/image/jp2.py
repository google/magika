# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""JPEG 2000 boxed files and raw codestreams, decoded with OpenJPEG via Pillow."""

from .._shared.pillow import SCOPE as SCOPE
from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("jp2",)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not (
        data.startswith(b"\x00\x00\x00\x0cjP  \r\n\x87\n") or data.startswith(b"\xff\x4f\xff\x51")
    ):
        return None
    return Observation(*decode(data, "JPEG2000"), "jp2")
