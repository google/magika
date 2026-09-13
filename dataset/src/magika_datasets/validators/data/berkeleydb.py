# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Berkeley DB files bounded by their metadata page: magic, page size and last page number."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("berkeleydb",)
SCOPE = "Metadata page 0: B-tree, hash or heap magic in either byte order, matching page type, power-of-two page size from 512 to 65536, and a last page number that accounts for the whole file; queue databases, whose extents live outside this file, are inconclusive"
MAGICS = {0x00053162: ("btree", 9), 0x00061561: ("hash", 8), 0x00074582: ("heap", 13)}
QUEUE = 0x00042253


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 512:
        return None
    for order in "<>":
        (magic,) = struct.unpack(order + "I", data[12:16])
        if magic in MAGICS or magic == QUEUE:
            break
    else:
        return None
    if magic == QUEUE:
        return Observation(
            "inconclusive", "Queue database extents are separate files", "berkeleydb"
        )
    kind, page_type = MAGICS[magic]
    (pgno,) = struct.unpack(order + "I", data[8:12])
    (pagesize,) = struct.unpack(order + "I", data[20:24])
    if pgno != 0:
        return Observation("fail", "Metadata page is not page 0", "berkeleydb")
    if pagesize < 512 or pagesize > 65536 or pagesize & (pagesize - 1):
        return Observation("fail", f"Invalid page size {pagesize}", "berkeleydb")
    if data[25] != page_type:
        return Observation("fail", f"{kind} magic with page type {data[25]}", "berkeleydb")
    (last,) = struct.unpack(order + "I", data[32:36])
    if (last + 1) * pagesize != len(data):
        return Observation(
            "fail", f"{last + 1} pages of {pagesize} bytes do not equal the file size", "berkeleydb"
        )
    return Observation(
        "pass", f"Berkeley DB {kind}: {last + 1} pages of {pagesize} bytes", "berkeleydb", (kind,)
    )
