# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Apple binary property lists: trailer arithmetic and a full plistlib parse."""

import plistlib
import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("applebplist",)
SCOPE = "bplist00 magic, 32-byte trailer with offset table and top object inside the file, object graph parsed by plistlib; values not interpreted"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"bplist0"):
        return None
    if len(data) < 40:
        return Observation("fail", "Truncated binary plist", "applebplist")
    offset_size, ref_size, objects, top, table = struct.unpack_from(
        ">6xBBQQQ", data, len(data) - 32
    )
    if not 1 <= offset_size <= 8 or not 1 <= ref_size <= 8 or not objects:
        return Observation("fail", "Invalid trailer sizes", "applebplist")
    if top >= objects or table < 8 or table + objects * offset_size > len(data) - 32:
        return Observation(
            "fail", "Offset table outside the file", "applebplist"
        )  # writers may pad before the trailer
    try:
        plistlib.loads(data, fmt=plistlib.FMT_BINARY)
    except Exception as error:  # plistlib raises several exception types for bad graphs
        return Observation(
            "fail", f"plistlib rejected the object graph: {str(error)[:60]}", "applebplist"
        )
    return Observation("pass", f"Binary plist with {objects} objects parsed", "applebplist")
