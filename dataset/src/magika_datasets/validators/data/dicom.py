# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""DICOM part 10 files: meta group, transfer syntax and data elements walked to EOF."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("dicom",)
SCOPE = "128-byte preamble and DICM, group 2 meta elements in explicit VR little endian, transfer syntax UID, dataset elements (explicit or implicit VR, little or big endian) with defined and undefined lengths, sequences and items walked to exactly EOF; pixel data not decoded"
LONG_VR = {
    b"OB",
    b"OW",
    b"OF",
    b"OL",
    b"OV",
    b"OD",
    b"SQ",
    b"UN",
    b"UT",
    b"UC",
    b"UR",
    b"SV",
    b"UV",
}
ELEMENTS = 1_000_000


class Malformed(Exception):
    pass


def header(
    data: bytes, offset: int, explicit: bool, order: str
) -> tuple[int, int, bytes | None, int, int]:
    """(group, element, VR, length, next offset)."""
    if offset + 8 > len(data):
        raise Malformed("Truncated element header")
    group, element = struct.unpack_from(order + "HH", data, offset)
    if group == 0xFFFE or not explicit:
        length = struct.unpack_from(order + "I", data, offset + 4)[0]
        return group, element, None, length, offset + 8
    vr = data[offset + 4 : offset + 6]
    if vr in LONG_VR:
        if offset + 12 > len(data):
            raise Malformed("Truncated long VR header")
        return group, element, vr, struct.unpack_from(order + "I", data, offset + 8)[0], offset + 12
    if not vr.isalpha() or not vr.isupper():
        raise Malformed(f"Invalid VR {vr!r}")
    return group, element, vr, struct.unpack_from(order + "H", data, offset + 6)[0], offset + 8


def walk(
    data: bytes, offset: int, end: int, explicit: bool, order: str, depth: int, state: dict
) -> int:
    """Walk elements until end (or a delimiter when end is None); returns the next offset."""
    while offset < end:
        state["count"] += 1
        if state["count"] > ELEMENTS:
            raise Malformed("Element budget exceeded")
        group, element, vr, length, offset = header(data, offset, explicit, order)
        if group == 0xFFFE and element in (0xE00D, 0xE0DD):  # item or sequence delimiter
            return offset
        if length == 0xFFFFFFFF:
            if depth > 32:
                raise Malformed("Sequence nesting too deep")
            if group == 0xFFFE and element == 0xE000:  # undefined-length item
                offset = walk(data, offset, len(data), explicit, order, depth + 1, state)
                continue
            offset = walk(
                data, offset, len(data), explicit, order, depth + 1, state
            )  # undefined-length sequence
            continue
        if offset + length > end:
            raise Malformed(f"Element ({group:04X},{element:04X}) exceeds its container")
        if vr == b"SQ" or (group == 0xFFFE and element == 0xE000):
            walk(data, offset, offset + length, explicit, order, depth + 1, state)
        elif (
            not explicit
            and length
            and depth < 32
            and data[offset : offset + 2] == b"\xfe\xff"
            and length >= 8
        ):
            walk(
                data, offset, offset + length, explicit, order, depth + 1, state
            )  # implicit VR sequence
        offset += length
    if offset != end:
        raise Malformed("Element overruns its container")
    return offset


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 132 or data[128:132] != b"DICM":
        return None
    state = {"count": 0}
    try:
        offset, syntax = 132, b"1.2.840.10008.1.2.1"
        while offset + 2 <= len(data):
            if struct.unpack_from("<H", data, offset)[0] != 0x0002:
                break  # the dataset may use implicit VR; do not parse it as meta
            group, element, vr, length, next_offset = header(data, offset, True, "<")
            if next_offset + length > len(data):
                raise Malformed("Meta element exceeds file")
            if element == 0x0010:
                syntax = data[next_offset : next_offset + length].rstrip(b"\0 ")
            offset = next_offset + length
        explicit = syntax != b"1.2.840.10008.1.2"
        order = ">" if syntax == b"1.2.840.10008.1.2.2" else "<"
        walk(data, offset, len(data), explicit, order, 0, state)
    except Malformed as error:
        return Observation("fail", str(error), "dicom")
    except struct.error:
        return Observation("fail", "Truncated structure", "dicom")
    tags = ("implicit_vr",) if not explicit else ()
    return Observation(
        "pass",
        f"{state['count']} elements walked; transfer syntax {syntax.decode('ascii', 'replace')}",
        "dicom",
        tags,
    )
