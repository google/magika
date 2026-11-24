# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""WebAssembly binaries: version, section ids in order, sizes tiling the file."""

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("wasm",)
SCOPE = "Magic and version 1, every section's id and LEB128 size, known sections in canonical order, sections tiling the file; instructions not decoded"
ORDER = {1: 1, 2: 2, 3: 3, 4: 4, 5: 5, 6: 6, 7: 7, 8: 8, 9: 9, 12: 10, 10: 11, 11: 12, 13: 13}
SECTIONS = 4096


def leb128(data: bytes, offset: int) -> tuple[int, int]:
    value, shift = 0, 0
    for _ in range(5):
        if offset >= len(data):
            raise ValueError("Truncated LEB128")
        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        shift += 7
        if not byte & 0x80:
            return value, offset
    raise ValueError("LEB128 too long")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"\0asm" or len(data) < 8:
        return None
    if data[4:8] != b"\x01\0\0\0":
        return Observation("fail", "Unsupported WebAssembly version", "wasm")
    offset, count, last = 8, 0, 0
    custom = []
    try:
        while offset < len(data):
            count += 1
            if count > SECTIONS:
                return Observation("inconclusive", "Section budget exceeded", "wasm")
            identifier = data[offset]
            size, offset = leb128(data, offset + 1)
            if offset + size > len(data):
                return Observation("fail", f"Section {identifier} exceeds file", "wasm")
            if identifier == 0:
                length, name_offset = leb128(data, offset)
                custom.append(
                    data[name_offset : name_offset + min(length, 64)].decode("utf-8", "replace")
                )
            elif identifier in ORDER:
                if ORDER[identifier] <= last:
                    return Observation("fail", f"Section {identifier} out of order", "wasm")
                last = ORDER[identifier]
            else:
                return Observation("fail", f"Unknown section id {identifier}", "wasm")
            offset += size
    except ValueError as error:
        return Observation("fail", str(error), "wasm")
    tags = tuple(
        sorted(
            {
                "custom_" + name
                for name in custom
                if name in ("name", "producers", "sourceMappingURL")
            }
        )
    )
    return Observation("pass", f"{count} sections bounded and ordered", "wasm", tags)
