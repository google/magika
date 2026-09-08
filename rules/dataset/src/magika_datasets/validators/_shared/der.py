# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Minimal DER (ASN.1) framing: element sizes and a bounded structural walk."""

ELEMENTS = 1 << 20


class Malformed(Exception):
    pass


def element(data: bytes, offset: int) -> tuple[int, int, int, bool]:
    """(tag byte, content start, content length, constructed) for the DER element at offset."""
    if offset + 2 > len(data):
        raise Malformed("Truncated element header")
    tag = data[offset]
    if tag & 0x1F == 0x1F:
        raise Malformed("Multi-byte tags are not used by DER structures checked here")
    first = data[offset + 1]
    if first < 0x80:
        return tag, offset + 2, first, bool(tag & 0x20)
    count = first & 0x7F
    if count == 0 or count > 4 or offset + 2 + count > len(data):
        raise Malformed("Indefinite or oversized length")
    length = int.from_bytes(data[offset + 2 : offset + 2 + count], "big")
    if length < 0x80 or (count > 1 and length < 1 << (8 * (count - 1))):
        raise Malformed("Length is not minimally encoded")
    return tag, offset + 2 + count, length, bool(tag & 0x20)


def total(data: bytes, offset: int) -> int:
    """Whole size of the element at offset, header included."""
    _, start, length, _ = element(data, offset)
    return start - offset + length


def walk(data: bytes, offset: int, end: int, depth: int = 0, budget: list | None = None) -> None:
    """Check that DER elements tile data[offset:end], recursing into constructed ones."""
    budget = budget if budget is not None else [ELEMENTS]
    if depth > 64:
        raise Malformed("Nesting too deep")
    while offset < end:
        tag, start, length, constructed = element(data, offset)
        if start + length > end:
            raise Malformed("Element extends past its parent")
        budget[0] -= 1
        if budget[0] < 0:
            raise Malformed("Element budget exceeded")
        if constructed:
            walk(data, start, start + length, depth + 1, budget)
        offset = start + length
    if offset != end:
        raise Malformed("Elements do not tile their parent")
