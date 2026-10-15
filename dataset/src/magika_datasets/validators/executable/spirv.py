# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""SPIR-V modules: header words and instruction word counts tiling the file."""

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("spirv",)
SCOPE = "Magic in either endianness, version, nonzero bound, zero schema, every instruction's word count at least one and instructions tiling the file; opcodes not interpreted"
INSTRUCTIONS = 1_000_000


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 20:
        return None
    order = (
        "<" if data[:4] == b"\x03\x02\x23\x07" else ">" if data[:4] == b"\x07\x23\x02\x03" else None
    )
    if order is None:
        return None
    if len(data) % 4:
        return Observation("fail", "File length is not word-aligned", "spirv")
    _, version, _, bound, schema = struct.unpack_from(order + "IIIII", data)
    if version >> 24 or not bound or schema:
        return Observation("fail", "Invalid SPIR-V header fields", "spirv")
    offset, count = 20, 0
    while offset < len(data):
        count += 1
        if count > INSTRUCTIONS:
            return Observation("inconclusive", "Instruction budget exceeded", "spirv")
        words = struct.unpack_from(order + "I", data, offset)[0] >> 16
        if not words or offset + 4 * words > len(data):
            return Observation("fail", f"Instruction {count} word count invalid", "spirv")
        offset += 4 * words
    if not count:
        return Observation("fail", "No instructions", "spirv")
    return Observation(
        "pass",
        f"{count} instructions tiling the module; version {version >> 16}.{(version >> 8) & 0xFF}",
        "spirv",
    )
