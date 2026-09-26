# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""xz streams decoded completely; block and index checks verified by liblzma."""

import lzma

from .._shared import decompress
from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("xz",)
SCOPE = "Stream header/footer, complete bounded block decoding, index and integrity checks of every concatenated stream with four-byte padding; contents not interpreted"
MAGIC = b"\xfd7zXZ\x00"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    return decompress.inspect(
        "xz",
        lambda: lzma.LZMADecompressor(lzma.FORMAT_XZ),
        data,
        MAGIC,
        lzma.LZMAError,
        padding=4,
    )
