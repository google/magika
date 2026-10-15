# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""GIF identification and bounded pixel decoding."""

from .._shared.pillow import SCOPE as SCOPE
from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("gif",)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:6] not in (b"GIF87a", b"GIF89a"):
        return None
    return Observation(*decode(data, "GIF"), "gif")
