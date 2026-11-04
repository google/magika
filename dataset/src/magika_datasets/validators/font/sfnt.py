# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""sfnt fonts (TrueType, OpenType, collections): table directory, checksums and coverage."""

import struct

from ..contract import Observation

FAMILY = "font"
FORMAT_IDS = ("ttf", "otf")
SCOPE = "sfnt version, table directory bounds and alignment, per-table checksums, required and outline tables, whole-file coverage without overlap; collections share tables; glyph programs not executed"
VERSIONS = {b"\0\1\0\0": "ttf", b"true": "ttf", b"OTTO": "otf"}
REQUIRED = (b"head", b"hhea", b"maxp", b"hmtx")
OUTLINES = (
    {b"glyf", b"loca"},
    {b"CFF "},
    {b"CFF2"},
    {b"EBDT", b"EBLC"},
    {b"CBDT", b"CBLC"},
    {b"sbix"},
)


class Malformed(Exception):
    pass


def checksum(chunk: bytes) -> int:
    chunk += b"\0" * (-len(chunk) % 4)
    return sum(struct.unpack(f">{len(chunk) // 4}I", chunk)) & 0xFFFFFFFF


def face(data: bytes, start: int) -> tuple[str, dict[bytes, tuple[int, int]], bool]:
    """Validate one sfnt header at start; returns (format, tables, adjustment_ok)."""
    version = data[start : start + 4]
    kind = VERSIONS.get(version)
    if kind is None:
        raise Malformed("Unknown sfnt version in collection")
    if start + 12 > len(data):
        raise Malformed("Truncated sfnt header")
    count = struct.unpack_from(">H", data, start + 4)[0]
    if not count or count > 512 or start + 12 + 16 * count > len(data):
        raise Malformed("Table directory outside file")
    tables = {}
    for index in range(count):
        tag, expected, offset, length = struct.unpack_from(">4sIII", data, start + 12 + 16 * index)
        if offset % 4 or offset + length > len(data) or tag in tables:
            raise Malformed(f"Table {tag!r} outside file, misaligned or duplicated")
        chunk = data[offset : offset + length]
        if tag == b"head":
            if length < 54 or struct.unpack_from(">I", chunk, 12)[0] != 0x5F0F3CF5:
                raise Malformed("head table magic mismatch")
            chunk = chunk[:8] + b"\0\0\0\0" + chunk[12:]
        if checksum(chunk) != expected:
            raise Malformed(f"Table checksum mismatch: {tag.decode('latin-1')}")
        tables[tag] = (offset, length)
    for tag in REQUIRED:
        if tag not in tables:
            raise Malformed(f"Missing required table {tag.decode('latin-1')}")
    if not any(group <= tables.keys() for group in OUTLINES):
        raise Malformed("No outline or bitmap glyph tables")
    if kind == "otf" and not {b"CFF ", b"CFF2"} & tables.keys():
        raise Malformed("OTTO font without CFF table")
    head_offset = tables[b"head"][0]
    adjustment = struct.unpack_from(">I", data, head_offset + 8)[0]
    whole = data[: head_offset + 8] + b"\0\0\0\0" + data[head_offset + 12 :]
    adjustment_ok = (0xB1B0AFBA - checksum(whole)) & 0xFFFFFFFF == adjustment
    return kind, tables, adjustment_ok


def coverage(data: bytes, ranges: list[tuple[int, int]]) -> None:
    """Every byte belongs to a header, directory or (4-padded) table; no overlaps."""
    position = 0
    for offset, length in sorted(set(ranges)):
        if offset < position:
            raise Malformed("Overlapping structures")
        if offset > position:
            raise Malformed("Unaccounted bytes between structures")
        position = offset + length + (-length % 4)
    if position < len(data):
        raise Malformed("Trailing bytes after last table")


def plausible(data: bytes, start: int) -> bool:
    """A directory whose tags are printable ASCII; \0\1\0\0 alone is a weak prefix."""
    count = struct.unpack_from(">H", data, start + 4)[0] if start + 6 <= len(data) else 0
    if not count or count > 512 or start + 12 + 16 * count > len(data):
        return False
    return all(
        32 <= byte < 127
        for index in range(count)
        for byte in data[start + 12 + 16 * index : start + 16 + 16 * index]
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 12 or (data[:4] not in VERSIONS and data[:4] != b"ttcf"):
        return None
    if data[:4] == b"\0\1\0\0" and not plausible(data, 0):
        return None
    tags = []
    try:
        if data[:4] == b"ttcf":
            major, count = (
                struct.unpack_from(">HHI", data, 4)[0],
                struct.unpack_from(">I", data, 8)[0],
            )
            if not count or count > 64:
                raise Malformed("Collection font count out of range")
            header = 12 + 4 * count + (12 if major >= 2 else 0)
            if header > len(data):
                raise Malformed("Collection header outside file")
            ranges = [(0, header)]
            kinds, adjustments = set(), []
            for index in range(count):
                start = struct.unpack_from(">I", data, 12 + 4 * index)[0]
                kind, tables, adjustment_ok = face(data, start)
                kinds.add(kind)
                adjustments.append(adjustment_ok)
                ranges.append((start, 12 + 16 * len(tables)))
                ranges.extend(tables.values())
            if major >= 2:
                _, dsig_length, dsig_offset = struct.unpack_from(">III", data, 12 + 4 * count)
                if dsig_length:
                    ranges.append((dsig_offset, dsig_length))
            kind = "otf" if kinds == {"otf"} else "ttf"
            tags.append("collection")
            adjustment_ok = all(adjustments)
            faces = count
        else:
            kind, tables, adjustment_ok = face(data, 0)
            ranges = [(0, 12 + 16 * len(tables)), *tables.values()]
            faces = 1
            if b"fvar" in tables:
                tags.append("variable")
        coverage(data, ranges)
    except Malformed as error:
        return Observation("fail", str(error), VERSIONS.get(data[:4], "ttf"))
    except struct.error:
        return Observation("fail", "Truncated structure", VERSIONS.get(data[:4], "ttf"))
    if not adjustment_ok:
        tags.append("checksum_adjustment_mismatch")
    return Observation(
        "pass",
        f"{faces} face(s); table checksums and whole-file coverage verified; adjustment_ok={adjustment_ok}",
        kind,
        tuple(tags),
    )
