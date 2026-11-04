# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""HDF4 files: data descriptor block chain and every data element inside the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("hdf4",)
SCOPE = "Magic, data descriptor block chain (counts and next pointers inside the file), every descriptor's element offset and length inside the file, descriptors and elements reaching EOF (one trailing zero pad byte allowed); element contents not decoded"
MAGIC = b"\x0e\x03\x13\x01"
BLOCKS = 65536
NONE = 0xFFFFFFFF


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 10:
        return Observation("fail", "Truncated first descriptor block", "hdf4")
    offset, seen, descriptors, end = 4, set(), 0, 4
    while offset:
        if offset in seen or offset + 6 > len(data):
            return Observation("fail", "Descriptor block chain loops or leaves the file", "hdf4")
        seen.add(offset)
        if len(seen) > BLOCKS:
            return Observation("inconclusive", "Descriptor block budget exceeded", "hdf4")
        count, following = struct.unpack_from(">HI", data, offset)
        block_end = offset + 6 + 12 * count
        if block_end > len(data):
            return Observation("fail", "Descriptor block outside file", "hdf4")
        end = max(end, block_end)
        for index in range(count):
            tag, _, position, length = struct.unpack_from(">HHII", data, offset + 6 + 12 * index)
            if tag == 0 or tag == 1:
                continue  # free or null descriptors
            if position == NONE or length == NONE:
                continue  # special elements defined elsewhere
            if position + length > len(data):
                return Observation("fail", f"Element of tag {tag} outside file", "hdf4")
            end = max(end, position + length)
            descriptors += 1
        offset = following
    tags = ()
    if end + 1 == len(data) and data[-1] == 0:
        tags = ("pad_byte",)  # the library leaves one zero byte after the last element
    elif end != len(data):
        return Observation(
            "fail", f"{len(data) - end} bytes are not referenced by any descriptor", "hdf4"
        )
    return Observation(
        "pass",
        f"{len(seen)} descriptor blocks, {descriptors} elements; descriptor chain and elements reach EOF",
        "hdf4",
        tags,
    )
