# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""PalmOS PDB and PRC databases: header and record list inside the file."""

import struct

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("palmos",)
PREFIX_ONLY = True  # no magic: only a hinted sample reports a failure
CONTEXT_REQUIRED = True  # and only a hinted sample is relabeled by a pass
SCOPE = "78-byte header with printable type and creator, PDB record or PRC resource entries by the resource attribute, offsets ascending and inside the file; contents not interpreted"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 80:
        return None
    kind, creator = data[60:64], data[64:68]
    if not all(32 <= byte < 127 for byte in kind + creator):
        return None
    name = data[:32].split(b"\0", 1)[0]
    if not name or not all(32 <= byte < 127 for byte in name) or len(name) == 32:
        return None
    if data[len(name) : 32].strip(b"\0"):
        return None  # the name field is NUL padded
    attributes, version = struct.unpack_from(">HH", data, 32)
    app_info, sort_info = struct.unpack_from(">II", data, 52)
    next_list = struct.unpack_from(">I", data, 72)[0]
    if next_list or (app_info and app_info >= len(data)) or (sort_info and sort_info >= len(data)):
        return None
    if attributes & 0x8000 or version > 0x00FF:
        return None
    resource = bool(attributes & 0x0001)  # PRC resource database: 10-byte entries
    width = 10 if resource else 8
    records = struct.unpack_from(">H", data, 76)[0]
    if 78 + width * records > len(data):
        return Observation("fail", "Record list outside file", "palmos")
    if not records:
        return Observation("fail", "Database without records", "palmos")
    list_end = 78 + width * records
    first = struct.unpack_from(">I", data, 78 + (6 if resource else 0))[0]
    if first not in (list_end, list_end + 2) and not (app_info and first > app_info):
        return Observation("fail", "First record does not follow the record list", "palmos")
    previous = list_end
    for index in range(records):
        entry = 78 + width * index
        offset = struct.unpack_from(">I", data, entry + 6 if resource else entry)[0]
        if offset < previous or offset > len(data):
            return Observation(
                "fail", f"Record {index} offset out of order or outside file", "palmos"
            )
        previous = offset
    tags = (f"palm_{kind.decode().lower()}", "prc" if resource else "pdb")
    return Observation(
        "pass",
        f"{records} {'resources' if resource else 'records'} bounded; type {kind.decode()} creator {creator.decode()}",
        "palmos",
        tags,
    )
