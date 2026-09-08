# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ISO base media boxes walked once; ftyp brands name MP4, 3GPP, QuickTime, HEIF or AVIF."""

import struct

from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("mp4", "3gp", "qt", "heif", "avif")
SCOPE = "ftyp brand mapping, every box and container box bound including largesize and open-ended boxes, required moov/meta/mdat presence; AVIF pixels decoded with Pillow, other codec payloads not decoded"
CONTAINERS = {
    b"moov", b"trak", b"mdia", b"minf", b"stbl", b"edts", b"dinf", b"udta", b"mvex",
    b"moof", b"traf", b"iprp", b"ipco", b"meta",
}  # fmt: skip
BRANDS = [
    ("avif", {b"avif", b"avis"}),
    ("heif", {b"heic", b"heix", b"hevc", b"hevx", b"mif1", b"msf1", b"heim", b"heis"}),
    ("qt", {b"qt  "}),
    (
        "mp4",
        {
            b"isom",
            b"iso2",
            b"iso4",
            b"iso5",
            b"iso6",
            b"mp41",
            b"mp42",
            b"mp4v",
            b"M4V ",
            b"M4A ",
            b"M4B ",
            b"dash",
            b"avc1",
            b"mmp4",
            b"f4v ",
        },  # fmt: skip
    ),
]
BOXES = 4096
DEPTH = 8


class Malformed(Exception):
    pass


class Budget(Exception):
    pass


def boxes(data: bytes, start: int, end: int, depth: int, counter: list[int]):
    offset = start
    while offset < end:
        if offset + 8 > end:
            raise Malformed("Truncated box header")
        size, kind = struct.unpack_from(">I4s", data, offset)
        header = 8
        if size == 1:
            if offset + 16 > end:
                raise Malformed("Truncated largesize box header")
            size, header = struct.unpack_from(">Q", data, offset + 8)[0], 16
        elif size == 0:
            size = end - offset
        if size < header or offset + size > end:
            raise Malformed("Box size outside its parent")
        counter[0] += 1
        if counter[0] > BOXES:
            raise Budget("Box budget exceeded")
        yield kind, offset, size, depth
        if kind in CONTAINERS:
            if depth >= DEPTH:
                raise Budget("Box nesting budget exceeded")
            skip = header
            if kind == b"meta" and not data[offset + header + 4 : offset + header + 8].isalnum():
                skip += 4  # ISO full box; QuickTime meta has no version/flags
            yield from boxes(data, offset + skip, offset + size, depth + 1, counter)
        offset += size


def brand(kind: bytes, compatible: list[bytes]) -> str | None:
    for format_id, brands in BRANDS:
        if kind in brands:
            return format_id
    if kind.startswith((b"3gp", b"3g2")):
        return "3gp"
    for candidate in compatible:
        for format_id, brands in BRANDS:
            if candidate in brands:
                return format_id
        if candidate.startswith((b"3gp", b"3g2")):
            return "3gp"
    return None


CLASSIC = {b"moov", b"mdat", b"free", b"skip", b"wide", b"pnot", b"junk"}


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 16 or (data[4:8] != b"ftyp" and data[4:8] not in CLASSIC):
        return None
    try:
        walked = list(boxes(data, 0, len(data), 0, [0]))
    except Malformed as error:
        return Observation("fail", str(error), "mp4")
    except Budget as error:
        return Observation("inconclusive", str(error), "mp4")
    top = {k for k, _, _, depth in walked if depth == 0}
    tags = ("fragmented",) if b"moof" in top else ()
    ftyp = [(offset, size) for k, offset, size, depth in walked if k == b"ftyp" and depth == 0]
    if not ftyp:
        if b"moov" not in top:
            return None  # a bare mdat/free prefix is not evidence of a movie
        kind, tags = "qt", tags + ("no_ftyp",)  # classic QuickTime predates ftyp
    else:
        offset, ftyp_size = ftyp[0]
        header = 16 if data[offset : offset + 4] == b"\0\0\0\1" else 8
        if len(ftyp) != 1 or ftyp_size < header + 8 or (ftyp_size - header) % 4:
            return Observation("fail", "Invalid ftyp brand list", "mp4")
        major = data[offset + header : offset + header + 4]
        compatible = [data[i : i + 4] for i in range(offset + header + 8, offset + ftyp_size, 4)]
        kind = brand(major, compatible)
        if kind is None:
            return Observation("inconclusive", f"Unmapped ftyp brand {major!r}", "mp4")
    if kind in ("mp4", "3gp", "qt"):
        if b"moov" not in top:
            return Observation("fail", "Missing moov box", kind)
        return Observation("pass", f"{len(walked)} boxes bounded; moov present", kind, tags)
    if b"meta" not in top:
        return Observation("fail", "Missing meta box", kind)
    if kind == "avif":
        return Observation(*decode(data, "AVIF"), kind)
    return Observation(
        "pass", f"{len(walked)} boxes bounded; meta present; pixels not decoded", kind
    )
