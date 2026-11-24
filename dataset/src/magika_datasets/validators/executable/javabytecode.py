# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Java class files: constant pool, members and attributes walked to the last byte."""

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("javabytecode",)
SCOPE = "CAFEBABE magic, class version 45..70, every constant pool entry sized by tag, interfaces, fields, methods and attributes walked exactly to EOF; nothing loaded"
SIZES = {
    3: 4,
    4: 4,
    5: 8,
    6: 8,
    7: 2,
    8: 2,
    9: 4,
    10: 4,
    11: 4,
    12: 4,
    15: 3,
    16: 2,
    17: 4,
    18: 4,
    19: 2,
    20: 2,
}


class Malformed(Exception):
    pass


def attributes(data: bytes, offset: int) -> int:
    count = struct.unpack_from(">H", data, offset)[0]
    offset += 2
    for _ in range(count):
        _, length = struct.unpack_from(">HI", data, offset)
        offset += 6 + length
        if offset > len(data):
            raise Malformed("Attribute exceeds file")
    return offset


def members(data: bytes, offset: int) -> int:
    count = struct.unpack_from(">H", data, offset)[0]
    offset += 2
    for _ in range(count):
        offset = attributes(data, offset + 6)
    return offset


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"\xca\xfe\xba\xbe" or len(data) < 10:
        return None
    minor, major = struct.unpack_from(">HH", data, 4)
    if major < 45 or major > 1000:
        return None  # Mach-O fat binaries share the magic; their arch count sits here
    if major > 70:
        return Observation("fail", "Unsupported class file version", "javabytecode")
    try:
        count = struct.unpack_from(">H", data, 8)[0]
        offset, index = 10, 1
        while index < count:
            tag = data[offset]
            if tag == 1:
                length = struct.unpack_from(">H", data, offset + 1)[0]
                offset += 3 + length
            elif tag in SIZES:
                offset += 1 + SIZES[tag]
            else:
                raise Malformed(f"Unknown constant pool tag {tag}")
            index += 2 if tag in (5, 6) else 1
            if offset > len(data):
                raise Malformed("Constant pool exceeds file")
        offset += 6  # access flags, this class, super class
        interfaces = struct.unpack_from(">H", data, offset)[0]
        offset += 2 + 2 * interfaces
        offset = members(data, offset)  # fields
        offset = members(data, offset)  # methods
        offset = attributes(data, offset)
    except (Malformed, struct.error, IndexError) as error:
        detail = str(error) if isinstance(error, Malformed) else "Truncated structure"
        return Observation("fail", detail, "javabytecode")
    if offset != len(data):
        return Observation("fail", "Trailing bytes after class attributes", "javabytecode")
    return Observation(
        "pass",
        f"Class file version {major}.{minor}: {count - 1} pool entries and members bounded",
        "javabytecode",
        (f"java_major_{major}",),
    )
