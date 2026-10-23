# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""UDF filesystems: volume recognition sequence and anchor volume descriptor pointer with tag checksums."""

import struct

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("udf",)
SCOPE = "Volume recognition sequence at sector 16 (BEA01, NSR02/NSR03, TEA01), anchor volume descriptor pointer at sector 256 or the last sector with tag identifier 2 and tag checksum verified, main and reserve volume descriptor sequence extents inside the file, descriptor tags along the main sequence verified; file entries not walked"
SECTOR = 2048
DESCRIPTORS = 64


def tag(data: bytes, offset: int) -> int | None:
    """Tag identifier at offset when the descriptor tag checksum holds, else None."""
    header = data[offset : offset + 16]
    if len(header) < 16:
        return None
    identifier, version = struct.unpack_from("<HH", header, 0)
    checksum = sum(header[:4]) + sum(header[5:16])
    if checksum & 0xFF != header[4] or version not in (2, 3):
        return None
    return identifier


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 257 * SECTOR:
        return None
    recognition = [
        data[16 * SECTOR + index * SECTOR + 1 : 16 * SECTOR + index * SECTOR + 6]
        for index in range(3)
    ]
    if recognition[0] != b"BEA01" or recognition[1] not in (b"NSR02", b"NSR03"):
        return None
    if b"TEA01" not in [
        data[16 * SECTOR + index * SECTOR + 1 : 16 * SECTOR + index * SECTOR + 6]
        for index in range(2, 8)
    ]:
        return Observation("fail", "Volume recognition sequence not terminated by TEA01", "udf")
    anchors = [256 * SECTOR, len(data) - SECTOR, len(data) - 257 * SECTOR]
    anchor = next((offset for offset in anchors if offset >= 0 and tag(data, offset) == 2), None)
    if anchor is None:
        return Observation("fail", "No anchor volume descriptor pointer with a valid tag", "udf")
    main_length, main_location, reserve_length, reserve_location = struct.unpack_from(
        "<IIII", data, anchor + 16
    )
    for name, location, length in (
        ("main", main_location, main_length),
        ("reserve", reserve_location, reserve_length),
    ):
        if length and (location + (length + SECTOR - 1) // SECTOR) * SECTOR > len(data):
            return Observation("fail", f"{name} volume descriptor sequence outside file", "udf")
    seen, offset = set(), main_location * SECTOR
    for _ in range(DESCRIPTORS):
        identifier = tag(data, offset)
        if identifier is None:
            return Observation(
                "fail", "Descriptor tag checksum mismatch in the main sequence", "udf"
            )
        if identifier == 8:  # terminating descriptor
            break
        seen.add(identifier)
        offset += SECTOR
        if offset + SECTOR > len(data):
            return Observation("fail", "Main volume descriptor sequence runs past EOF", "udf")
    if not {1, 5, 6} <= seen:  # primary volume, partition, logical volume descriptors
        return Observation(
            "fail", "Main sequence lacks primary, partition or logical volume descriptors", "udf"
        )
    return Observation(
        "pass",
        f"{recognition[1].decode()} volume; anchor and {len(seen)} main-sequence descriptor tags verified",
        "udf",
    )
