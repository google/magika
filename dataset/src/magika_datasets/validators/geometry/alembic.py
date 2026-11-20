# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Alembic Ogawa archives: header and the group/data tree walked within the file."""

import struct

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("alembic",)
SCOPE = "Ogawa magic, frozen flag and version 1, root group offset, then every group reachable from the root (child count and child offsets) and every data block (size and bytes) inside the file; the Alembic schema, samples and HDF5-backed archives not interpreted"
MAGIC = b"Ogawa"
DATA = 1 << 63
NODES = 2_000_000


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 16 or data[5] not in (0x00, 0xFF) or data[6:8] != b"\x00\x01":
        return Observation("fail", "Ogawa frozen flag or version unexpected", "alembic")
    if data[5] != 0xFF:
        return Observation("inconclusive", "Archive was never finalized (not frozen)", "alembic")
    (root,) = struct.unpack_from("<Q", data, 8)
    pending, groups, blocks, seen = [root], 0, 0, set()
    while pending:
        offset = pending.pop()
        if offset in seen:
            continue
        seen.add(offset)
        if len(seen) > NODES:
            return Observation("inconclusive", "Node budget exceeded", "alembic")
        if offset & DATA:
            start = offset & ~DATA
            if start == 0:
                continue  # an empty data block
            if start + 8 > len(data):
                return Observation("fail", f"Data block at {start} outside the file", "alembic")
            (size,) = struct.unpack_from("<Q", data, start)
            if start + 8 + size > len(data):
                return Observation("fail", f"Data block at {start} extends past the end", "alembic")
            blocks += 1
            continue
        if offset == 0:
            continue  # an empty group
        if offset + 8 > len(data):
            return Observation("fail", f"Group at {offset} outside the file", "alembic")
        (count,) = struct.unpack_from("<Q", data, offset)
        if offset + 8 + count * 8 > len(data):
            return Observation("fail", f"Group at {offset} lists children past the end", "alembic")
        pending.extend(struct.unpack_from(f"<{count}Q", data, offset + 8))
        groups += 1
    if groups == 0:
        return Observation("fail", "Root group is empty or missing", "alembic")
    return Observation(
        "pass", f"{groups} groups and {blocks} data blocks inside the file", "alembic"
    )
