# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""DOS master boot records: exactly one sector, boot signature and a consistent partition table."""

import struct

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("dosmbr",)
SCOPE = "512-byte sector, 0x55AA signature, four partition entries with valid boot indicators, non-overlapping LBA extents for used entries, at most one active partition; boot code not interpreted"
CONTEXT_REQUIRED = True  # any bootable sector image shares the layout


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) != 512 or data[510:512] != b"\x55\xaa":
        return None
    extents, active, used = [], 0, 0
    for index in range(4):
        entry = data[446 + 16 * index : 462 + 16 * index]
        indicator, kind = entry[0], entry[4]
        start, size = struct.unpack_from("<II", entry, 8)
        if indicator not in (0x00, 0x80):
            return Observation(
                "fail", f"Partition {index} has boot indicator {indicator:#x}", "dosmbr"
            )
        if kind == 0:
            continue
        if size == 0:
            return Observation(
                "fail", f"Partition {index} has type {kind:#x} but no sectors", "dosmbr"
            )
        extents.append((start, start + size))
        active += indicator == 0x80
        used += 1
    if active > 1:
        return Observation("fail", "More than one active partition", "dosmbr")
    extents.sort()
    for (_, end), (next_start, _) in zip(extents, extents[1:]):
        if next_start < end:
            return Observation("fail", "Partition extents overlap", "dosmbr")
    if used == 0 and not any(data[:446]):
        return Observation("inconclusive", "Empty partition table and no boot code", "dosmbr")
    return Observation(
        "pass", f"{used} partitions with consistent extents; boot signature present", "dosmbr"
    )
