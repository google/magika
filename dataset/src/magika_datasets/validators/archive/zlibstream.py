# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Bare zlib (RFC 1950) streams: header arithmetic, full inflation and Adler-32."""

import zlib

from .._shared import decompress
from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("zlibstream",)
PREFIX_ONLY = True  # applicability rests on a short prefix; failures need a hint
SCOPE = "RFC 1950 header consistency, complete bounded inflation of a single stream and Adler-32 verification; contents not interpreted"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 2:
        return None
    cmf, flg = data[0], data[1]
    if cmf & 0x0F != 8 or cmf >> 4 > 7 or (cmf * 256 + flg) % 31:
        return None
    if flg & 0x20:
        return Observation(
            "inconclusive", "Preset dictionary required; stream not inflated", "zlibstream"
        )
    return decompress.inspect(
        "zlibstream", zlib.decompressobj, data, b"", zlib.error, concatenated=False
    )
