# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""HDF5 files: superblock versions 0-3 with the end-of-file address against the file and the lookup3 checksum."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("h5",)
SCOPE = "Superblock signature at a power-of-two offset, version 0/1 fields (sizes, addresses, root symbol table entry inside the file) or version 2/3 fields with the Jenkins lookup3 checksum verified, end-of-file address equal to the file size; object headers not decoded"
SIGNATURE = b"\x89HDF\r\n\x1a\n"
MASK = 0xFFFFFFFF


def rotate(value: int, bits: int) -> int:
    return ((value << bits) | (value >> (32 - bits))) & MASK


def lookup3(data: bytes, initial: int = 0) -> int:
    """Bob Jenkins' hashlittle, as used by the HDF5 library for metadata checksums."""
    a = b = c = (0xDEADBEEF + len(data) + initial) & MASK
    position, remaining = 0, len(data)
    while remaining > 12:
        a = (a + struct.unpack_from("<I", data, position)[0]) & MASK
        b = (b + struct.unpack_from("<I", data, position + 4)[0]) & MASK
        c = (c + struct.unpack_from("<I", data, position + 8)[0]) & MASK
        a = (a - c) & MASK
        a ^= rotate(c, 4)
        c = (c + b) & MASK
        b = (b - a) & MASK
        b ^= rotate(a, 6)
        a = (a + c) & MASK
        c = (c - b) & MASK
        c ^= rotate(b, 8)
        b = (b + a) & MASK
        a = (a - c) & MASK
        a ^= rotate(c, 16)
        c = (c + b) & MASK
        b = (b - a) & MASK
        b ^= rotate(a, 19)
        a = (a + c) & MASK
        c = (c - b) & MASK
        c ^= rotate(b, 4)
        b = (b + a) & MASK
        position += 12
        remaining -= 12
    if remaining == 0:
        return c
    tail = data[position:] + bytes(12 - remaining)
    a = (a + struct.unpack_from("<I", tail, 0)[0]) & MASK
    b = (b + struct.unpack_from("<I", tail, 4)[0]) & MASK
    c = (c + struct.unpack_from("<I", tail, 8)[0]) & MASK
    c ^= b
    c = (c - rotate(b, 14)) & MASK
    a ^= c
    a = (a - rotate(c, 11)) & MASK
    b ^= a
    b = (b - rotate(a, 25)) & MASK
    c ^= b
    c = (c - rotate(b, 16)) & MASK
    a ^= c
    a = (a - rotate(c, 4)) & MASK
    b ^= a
    b = (b - rotate(a, 14)) & MASK
    c ^= b
    c = (c - rotate(b, 24)) & MASK
    return c


def address(data: bytes, offset: int, size: int) -> int:
    return int.from_bytes(data[offset : offset + size], "little")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    base = 0
    while base + 8 <= len(data):
        if data[base : base + 8] == SIGNATURE:
            break
        base = 512 if base == 0 else base * 2
    else:
        return None
    if base + 48 > len(data):
        return Observation("fail", "Superblock truncated", "h5")
    version = data[base + 8]
    remaining = len(data) - base
    if version in (0, 1):
        offsets, lengths = data[base + 13], data[base + 14]
        if offsets not in (2, 4, 8) or lengths not in (2, 4, 8):
            return Observation("fail", "Size of offsets or lengths invalid", "h5")
        position = base + 24 if version == 0 else base + 28
        base_address = address(data, position, offsets)
        eof = address(data, position + 2 * offsets, offsets)
        root = position + 4 * offsets
        if root + 2 * offsets + 8 > len(data):
            return Observation("fail", "Root group symbol table entry outside file", "h5")
        object_header = address(data, root + offsets, offsets)
        if object_header >= remaining:
            return Observation("fail", "Root object header outside file", "h5")
        tags = ()
    elif version in (2, 3):
        offsets, lengths = data[base + 9], data[base + 10]
        if offsets not in (2, 4, 8) or lengths not in (2, 4, 8):
            return Observation("fail", "Size of offsets or lengths invalid", "h5")
        position = base + 12
        base_address = address(data, position, offsets)
        eof = address(data, position + 2 * offsets, offsets)
        root = address(data, position + 3 * offsets, offsets)
        checksum_offset = position + 4 * offsets
        if checksum_offset + 4 > len(data):
            return Observation("fail", "Superblock truncated", "h5")
        if (
            lookup3(data[base:checksum_offset])
            != struct.unpack_from("<I", data, checksum_offset)[0]
        ):
            return Observation("fail", "Superblock checksum mismatch", "h5")
        if root >= remaining:
            return Observation("fail", "Root object header outside file", "h5")
        tags = ("superblock_checksum",)
    else:
        return Observation("fail", f"Unknown superblock version {version}", "h5")
    if base_address != base and base_address != 0:
        return Observation("fail", "Base address disagrees with the superblock position", "h5")
    if eof != remaining:
        return Observation(
            "fail",
            f"End-of-file address {eof} differs from the {remaining} bytes after the base",
            "h5",
        )
    return Observation(
        "pass",
        f"Superblock version {version} at offset {base}; end-of-file address matches the file",
        "h5",
        tags,
        generic=True,  # NetCDF-4, Keras and MATLAB v7.3 files are HDF5 containers
    )
