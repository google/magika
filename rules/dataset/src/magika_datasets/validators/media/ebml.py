# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""EBML documents walked once; the header DocType names Matroska or WebM."""

from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("mkv", "webm")
SCOPE = "EBML header and DocType, variable-length IDs and sizes, every element bounded inside its parent including unknown-size Segment and Cluster, required Info and Tracks; codec payloads not decoded"
HEADER = b"\x1a\x45\xdf\xa3"
DOCTYPE = b"\x42\x82"
SEGMENT = b"\x18\x53\x80\x67"
INFO = b"\x15\x49\xa9\x66"
TRACKS = b"\x16\x54\xae\x6b"
CLUSTER = b"\x1f\x43\xb6\x75"
CONTAINERS = {
    SEGMENT,
    CLUSTER,
    INFO,
    TRACKS,
    b"\xae",
    b"\x11\x4d\x9b\x74",
    b"\x4d\xbb",
    b"\xa0",
    b"\x1c\x53\xbb\x6b",
    b"\xbb",
    b"\xb7",
}
UNKNOWN_ALLOWED = {SEGMENT, CLUSTER}
DOCTYPES = {b"matroska": "mkv", b"webm": "webm"}
ELEMENTS = 1_000_000  # clusters hold one element per block; real files run to hundreds of thousands
DEPTH = 6


class Malformed(Exception):
    pass


class Budget(Exception):
    pass


def vint(data: bytes, offset: int, end: int, keep_marker: bool) -> tuple[int, int, bool]:
    """Read an EBML variable-length integer. Returns (value, next offset, is_unknown)."""
    if offset >= end:
        raise Malformed("Truncated element")
    first = data[offset]
    if first == 0:
        raise Malformed("Invalid variable-length integer")
    width = 1
    while not first & (0x80 >> (width - 1)):
        width += 1
    if width > 8 or offset + width > end:
        raise Malformed("Variable-length integer outside its parent")
    raw = int.from_bytes(data[offset : offset + width], "big")
    if keep_marker:
        return raw, offset + width, False
    value = raw & ((1 << (7 * width)) - 1)
    return value, offset + width, value == (1 << (7 * width)) - 1


def elements(data: bytes, start: int, end: int, depth: int, state: dict):
    offset = start
    while offset < end:
        identifier, offset, _ = vint(data, offset, end, keep_marker=True)
        identifier = identifier.to_bytes((identifier.bit_length() + 7) // 8, "big")
        size, offset, unknown = vint(data, offset, end, keep_marker=False)
        if unknown:
            if identifier not in UNKNOWN_ALLOWED:
                raise Malformed("Unknown size on a sized element")
            state["unknown"] = True
            size = end - offset
        if offset + size > end:
            raise Malformed("Element exceeds its parent")
        state["count"] += 1
        if state["count"] > ELEMENTS:
            raise Budget("Element budget exceeded")
        yield identifier, offset, size, depth
        if identifier in CONTAINERS and depth < DEPTH:
            yield from elements(data, offset, offset + size, depth + 1, state)
        offset += size


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(HEADER):
        return None
    state = {"count": 0, "unknown": False}
    kind = "mkv"
    try:
        size, body, unknown = vint(data, 4, len(data), keep_marker=False)
        if unknown or body + size > len(data):
            raise Malformed("EBML header outside file")
        doctype = None
        for identifier, offset, length, _ in elements(data, body, body + size, 1, state):
            if identifier == DOCTYPE:
                doctype = data[offset : offset + length]
        kind = DOCTYPES.get(doctype)
        if kind is None:
            return None  # a foreign or unreadable DocType is not our format
        walked = list(elements(data, body + size, len(data), 0, state))
    except Malformed as error:
        return Observation("fail", str(error), kind)
    except Budget as error:
        return Observation("inconclusive", str(error), kind)
    top = [w for w in walked if w[3] == 0]
    if len(top) != 1 or top[0][0] != SEGMENT:
        return Observation("fail", "Expected exactly one Segment after the header", kind)
    children = {w[0] for w in walked if w[3] == 1}
    if INFO not in children or TRACKS not in children:
        return Observation("fail", "Segment lacks Info or Tracks", kind)
    tags = ("unknown_size",) if state["unknown"] else ()
    return Observation(
        "pass", f"{state['count']} EBML elements bounded; Info and Tracks present", kind, tags
    )
