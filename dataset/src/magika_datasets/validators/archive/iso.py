# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ISO 9660 images: volume descriptors, path tables and root directory extents against the file."""

import struct

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("iso",)
SCOPE = "Primary volume descriptor and terminator, logical block size, volume space size against the file (2048-byte or raw 2352-byte sectors), path table and root directory extents inside the volume, root directory records walked; file contents not read"
DESCRIPTORS = 32
RECORDS = 4096


def descriptor_offset(data: bytes) -> tuple[int, int] | None:
    """(sector size, data offset within a raw sector) for the layout whose descriptor is at sector 16."""
    for sector, skip in ((2048, 0), (2352, 16), (2352, 24)):
        position = 16 * sector + skip
        if data[position + 1 : position + 6] == b"CD001" and data[position] in (0, 1, 2):
            return sector, skip
    return None


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 17 * 2048:
        return None
    layout = descriptor_offset(data)
    if layout is None:
        return None
    sector, skip = layout

    def block(number: int) -> bytes:
        start = number * sector + skip
        return data[start : start + 2048]

    primary = None
    tags = ["raw_sectors"] if sector == 2352 else []
    for index in range(DESCRIPTORS):
        current = block(16 + index)
        if len(current) < 2048 or current[1:6] != b"CD001":
            return Observation("fail", "Volume descriptor set is not terminated", "iso")
        if current[0] == 255:
            break
        if current[0] == 1 and primary is None:
            primary = current
        if current[0] == 2:
            tags.append("joliet")
        if current[0] == 0:
            tags.append("bootable")
    else:
        return Observation("fail", "No terminator within the first 32 descriptors", "iso")
    if primary is None:
        return Observation("fail", "No primary volume descriptor", "iso")
    space = struct.unpack_from("<I", primary, 80)[0]
    logical = struct.unpack_from("<H", primary, 128)[0]
    if logical != 2048 or space == 0:
        return Observation("inconclusive", f"Logical block size {logical} not supported", "iso")
    expected = space * sector
    if len(data) < expected:
        return Observation(
            "fail", f"Volume space needs {expected} bytes, file has {len(data)}", "iso"
        )
    path_size = struct.unpack_from("<I", primary, 132)[0]
    path_l = struct.unpack_from("<I", primary, 140)[0]
    if path_l == 0 or path_l * sector + path_size > expected:
        return Observation("fail", "L path table outside the volume", "iso")
    root = primary[156:190]
    root_extent, root_size = (
        struct.unpack_from("<I", root, 2)[0],
        struct.unpack_from("<I", root, 10)[0],
    )
    if root_extent * sector + root_size > expected or root_size == 0:
        return Observation("fail", "Root directory outside the volume", "iso")
    records, position = 0, 0
    directory = b"".join(block(root_extent + index) for index in range((root_size + 2047) // 2048))
    while position < root_size and records < RECORDS:
        length = directory[position]
        if length == 0:  # records never straddle a sector; the rest of this sector is padding
            position = (position // 2048 + 1) * 2048
            continue
        if length < 34 or position + length > root_size:
            return Observation("fail", "Root directory record malformed", "iso")
        extent, size = (
            struct.unpack_from("<I", directory, position + 2)[0],
            struct.unpack_from("<I", directory, position + 10)[0],
        )
        if extent * sector + size > expected:
            return Observation("fail", "Root entry extent outside the volume", "iso")
        position += length
        records += 1
    if len(data) > expected:
        return Observation(
            "inconclusive",
            f"{len(data) - expected} bytes beyond the volume space",
            "iso",
            tuple(tags),
        )
    return Observation(
        "pass",
        f"{records} root records, {space} logical blocks tile the file; descriptors, path table and extents verified",
        "iso",
        tuple(tags),
    )
