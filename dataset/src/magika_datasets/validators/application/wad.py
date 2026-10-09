# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Doom-engine WAD files: directory placement and lump extents covering the file."""

import struct

from ..contract import Observation

FAMILY = "application"
FORMAT_IDS = ("wad",)
SCOPE = "IWAD/PWAD magic, directory inside the file, every lump extent inside the file, lump names printable, lumps and directory together reaching EOF; lump contents not interpreted"
LUMPS = 1 << 20


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 12 or data[:4] not in (b"IWAD", b"PWAD"):
        return None
    count, directory = struct.unpack_from("<II", data, 4)
    if count > LUMPS:
        return Observation("inconclusive", "Lump budget exceeded", "wad")
    if directory < 12 or directory + 16 * count > len(data):
        return Observation("fail", "Directory outside file", "wad")
    end = directory + 16 * count
    for index in range(count):
        position, size = struct.unpack_from("<II", data, directory + 16 * index)
        name = data[directory + 16 * index + 8 : directory + 16 * index + 16]
        if size and position + size > len(data):
            return Observation("fail", "Lump extent outside file", "wad")
        if any(byte and (byte < 0x20 or byte > 0x7E) for byte in name):
            return Observation("fail", "Lump name is not printable ASCII", "wad")
        if size:
            end = max(end, position + size)
    if end != len(data):
        return Observation(
            "fail", f"{len(data) - end} bytes are neither lump data nor directory", "wad"
        )
    return Observation(
        "pass",
        f"{data[:4].decode()} with {count} lumps; directory and lump extents reach EOF",
        "wad",
        ("iwad",) if data[0] == ord("I") else ("pwad",),
    )
