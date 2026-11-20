# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""FileMaker Pro 12+ (.fmp12) files: HBAM7 header and the page chain it anchors."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("filemaker",)
SCOPE = "15-byte file signature and HBAM7 marker, size a whole number of 4096-byte pages, page 1 recording the last page number, and pages 2 onward forming one doubly-linked chain that visits every page exactly once; page payloads, encryption and the pre-12 .fp7 layout not interpreted"
SIGNATURE = b"\x00\x01\x00\x00\x00\x02\x00\x01\x00\x05\x00\x02\x00\x02\xc0"
MARKER = b"HBAM7"
PAGE = 4096
LINKS = struct.Struct(">4xII")  # previous page, next page


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(SIGNATURE):
        return None
    if data[15:20] != MARKER:
        return Observation("fail", "HBAM7 marker missing after the signature", "filemaker")
    pages = len(data) // PAGE
    if len(data) % PAGE or pages < 3:
        return Observation("fail", "Size is not a whole number of pages", "filemaker")
    links = {page: LINKS.unpack_from(data, page * PAGE) for page in range(1, pages)}
    last = pages - 1
    if links[1] != (0, last):
        return Observation("fail", "Page 1 does not record the last page number", "filemaker")
    heads = [page for page in range(2, pages) if links[page][0] == 0]
    if len(heads) != 1:
        return Observation("fail", f"{len(heads)} page chains instead of one", "filemaker")
    page, previous, visited = heads[0], 0, 0
    while page:
        if not 2 <= page < pages or links[page][0] != previous:
            return Observation("fail", f"Page {page} breaks the chain", "filemaker")
        visited += 1
        if visited > pages:
            return Observation("fail", "Page chain loops", "filemaker")
        previous, page = page, links[page][1]
    if visited != pages - 2:
        return Observation("fail", f"Chain visits {visited} of {pages - 2} pages", "filemaker")
    return Observation("pass", f"{pages} pages chained head to tail", "filemaker")
