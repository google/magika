# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""gzip members inflated with zlib; CRC-32 and ISIZE verified for every member."""

import zlib

from .._shared import decompress
from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("gzip",)
SCOPE = "Member headers, complete bounded DEFLATE inflation, CRC-32 and ISIZE of every concatenated member; member contents not interpreted"
MAGIC = b"\x1f\x8b"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    return decompress.inspect("gzip", lambda: zlib.decompressobj(31), data, MAGIC, zlib.error)
