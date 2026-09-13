# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""bzip2 streams decoded completely; block and stream CRCs verified by the decoder."""

import bz2

from .._shared import decompress
from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("bzip",)
SCOPE = "Stream header, complete bounded block decoding and CRC verification of every concatenated stream; contents not interpreted"
MAGIC = b"BZh"
BLOCK = b"\x31\x41\x59\x26\x53\x59"
EMPTY = b"\x17\x72\x45\x38\x50\x90"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not (data.startswith(MAGIC) and data[3:4] in b"123456789" and data[4:10] in (BLOCK, EMPTY)):
        return None
    return decompress.inspect("bzip", bz2.BZ2Decompressor, data, MAGIC, (OSError, ValueError))
