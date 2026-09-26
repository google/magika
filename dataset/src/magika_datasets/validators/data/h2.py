# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""H2 PageStore databases: triple file header, page size and a whole number of pages."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("h2",)
SCOPE = "Three copies of the `-- H2 0.5/B --` header, page size a power of two between 64 and 32768, read/write format versions, file a whole number of pages with at least the three fixed pages; page contents not decoded"
HEADER = b"-- H2 0.5/B -- \n"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(HEADER):
        return None
    if data[:48] != HEADER * 3 or len(data) < 56:
        return Observation("fail", "Header is not repeated three times", "h2")
    page_size = struct.unpack_from(">I", data, 48)[0]
    write_version, read_version = data[52], data[53]
    if not 64 <= page_size <= 32768 or page_size & (page_size - 1):
        return Observation("fail", f"Page size {page_size} invalid", "h2")
    if write_version > 3 or read_version > 3:
        return Observation("fail", "Unknown format version", "h2")
    if len(data) % page_size or len(data) < 3 * page_size:
        return Observation("fail", "File is not a whole number of pages", "h2")
    return Observation(
        "pass",
        f"PageStore with {len(data) // page_size} pages of {page_size} bytes; header and format versions verified",
        "h2",
    )
