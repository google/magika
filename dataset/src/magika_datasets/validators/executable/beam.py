# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Erlang BEAM modules: IFF FOR1 container with 4-byte padded chunks."""

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("beam",)
SCOPE = "FOR1 size equal to the file, BEAM form, every chunk's size with 4-byte padding tiling the container, atom and code chunks present; bytecode not interpreted"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"FOR1" or data[8:12] != b"BEAM":
        return None
    size = struct.unpack_from(">I", data, 4)[0]
    if size + 8 != len(data):
        return Observation("fail", "FOR1 size differs from file", "beam")
    offset, kinds = 12, []
    while offset < len(data):
        if offset + 8 > len(data):
            return Observation("fail", "Truncated chunk header", "beam")
        kind, length = struct.unpack_from(">4sI", data, offset)
        end = offset + 8 + length
        if end > len(data):
            return Observation("fail", f"Chunk {kind!r} exceeds file", "beam")
        kinds.append(kind)
        if len(kinds) > 4096:
            return Observation("inconclusive", "Chunk budget exceeded", "beam")
        offset = end + (-length % 4)
    if offset != len(data):
        return Observation("fail", "Chunk padding exceeds file", "beam")
    if not {b"Atom", b"AtU8"} & set(kinds) or b"Code" not in kinds:
        return Observation("fail", "Missing atom or code chunk", "beam")
    return Observation("pass", f"{len(kinds)} BEAM chunks tiling the file", "beam")
