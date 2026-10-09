# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""RIFF containers walked once; WAVE, AVI, ACON and WEBP forms are checked in place."""

import struct
from dataclasses import dataclass

from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("wav", "avi", "ani", "webp")
SCOPE = "RIFF size, every chunk and LIST bound with padding, form-specific header chunks (PCM, float and extensible fmt/data, avih/movi/idx1, anih/fram, VP8 chunk order plus Pillow decode); codec payloads not decoded except WebP"
FORMS = {b"WAVE": "wav", b"AVI ": "avi", b"ACON": "ani", b"WEBP": "webp"}
CHUNKS = 4096
DEPTH = 8


class Malformed(Exception):
    pass


class Budget(Exception):
    pass


@dataclass(frozen=True)
class Chunk:
    kind: bytes
    body: int  # offset of the payload (after the LIST form code for lists)
    length: int  # payload length including the form code for lists
    form: bytes | None
    depth: int

    @property
    def end(self):
        return self.body + self.length


def chunks(data: bytes, start: int, end: int, depth: int, counter: list[int]):
    offset = start
    while offset < end:
        if offset + 8 > end:
            raise Malformed("Truncated chunk header")
        kind = data[offset : offset + 4]
        length = struct.unpack_from("<I", data, offset + 4)[0]
        body, stop = offset + 8, offset + 8 + length
        padded = stop + length % 2
        if stop > end:
            raise Malformed("Chunk exceeds container bounds")
        if padded > end:
            padded = end  # writers often leave the last odd chunk's pad byte to the parent
        elif padded > stop and data[stop] != 0:
            raise Malformed("Nonzero padding byte")
        counter[0] += 1
        if counter[0] > CHUNKS:
            raise Budget("Chunk budget exceeded")
        form = data[body : body + 4] if kind in (b"LIST", b"RIFF") and length >= 4 else None
        yield Chunk(kind, body, length, form, depth)
        if form is not None and depth < DEPTH:
            yield from chunks(data, body + 4, stop, depth + 1, counter)
        offset = padded


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"RIFF" or len(data) < 12:
        return None
    kind = FORMS.get(data[8:12])
    if kind is None:
        return None
    if struct.unpack_from("<I", data, 4)[0] + 8 != len(data):
        return Observation("fail", "RIFF size differs from complete file", kind)
    try:
        walked = list(chunks(data, 12, len(data), 0, [0]))
    except Malformed as error:
        return Observation("fail", str(error), kind)
    except Budget as error:
        return Observation("inconclusive", str(error), kind)
    top = [c for c in walked if c.depth == 0]
    return {"wav": wave, "avi": avi, "ani": ani, "webp": webp}[kind](data, walked, top)


def wave(data: bytes, walked: list[Chunk], top: list[Chunk]) -> Observation:
    fmt = payload = None
    for chunk in top:
        if chunk.kind == b"fmt ":
            if fmt is not None or payload is not None or chunk.length < 16:
                return Observation("fail", "Missing, duplicate or misplaced format fields", "wav")
            fmt = struct.unpack_from("<HHIIHH", data, chunk.body)
            fmt_chunk = chunk
        elif chunk.kind == b"data":
            if fmt is None or payload is not None:
                return Observation("fail", "Data before format or multiple data chunks", "wav")
            payload = chunk.length
    if fmt is None or payload is None:
        return Observation("fail", "Missing format or data chunk", "wav")
    codec, channels, rate, byte_rate, alignment, bits = fmt
    tags = ()
    if codec == 0xFFFE:  # WAVE_FORMAT_EXTENSIBLE: the real codec leads the SubFormat GUID
        if fmt_chunk.length < 40:
            return Observation("fail", "Extensible format chunk too short", "wav")
        codec = struct.unpack_from("<H", data, fmt_chunk.body + 24)[0]
        tags = ("extensible",)
    if codec not in (1, 3):
        return Observation("inconclusive", "Only PCM and IEEE float samples are validated", "wav")
    widths = (8, 16, 24, 32) if codec == 1 else (32, 64)
    if (
        not channels
        or not rate
        or bits not in widths
        or alignment != channels * bits // 8
        or byte_rate != rate * alignment
    ):
        return Observation("fail", "Inconsistent sample format fields", "wav")
    if payload % alignment:
        return Observation("fail", "Incomplete sample frame", "wav")
    kind = "PCM" if codec == 1 else "IEEE float"
    return Observation("pass", f"{kind} container and all sample-frame bounds checked", "wav", tags)


def avi(data: bytes, walked: list[Chunk], top: list[Chunk]) -> Observation:
    lists = {c.form: c for c in top if c.kind == b"LIST"}
    header, movie = lists.get(b"hdrl"), lists.get(b"movi")
    if header is None or movie is None:
        return Observation("fail", "Missing hdrl or movi list", "avi")
    avih = [c for c in walked if c.kind == b"avih" and header.body < c.body < header.end]
    if len(avih) != 1 or avih[0].length < 56:
        return Observation("fail", "Missing or short avih header", "avi")
    index = [c for c in top if c.kind == b"idx1"]
    if index:
        entries = index[0].length // 16
        for number in range(entries):
            offset, size = struct.unpack_from("<II", data, index[0].body + 16 * number + 8)
            if movie.body - 4 + offset + size > len(data) and offset + size > len(data):
                return Observation("fail", "Index entry outside file bounds", "avi")
    tags = ("has_index",) if index else ()
    return Observation("pass", "AVI header, movie list and index bounds checked", "avi", tags)


def ani(data: bytes, walked: list[Chunk], top: list[Chunk]) -> Observation:
    anih = [c for c in top if c.kind == b"anih"]
    if (
        len(anih) != 1
        or anih[0].length != 36
        or struct.unpack_from("<I", data, anih[0].body)[0] != 36
    ):
        return Observation("fail", "Missing or malformed anih header", "ani")
    frames = struct.unpack_from("<I", data, anih[0].body + 4)[0]
    fram = [c for c in top if c.kind == b"LIST" and c.form == b"fram"]
    if not frames or len(fram) != 1:
        return Observation("fail", "Missing frame list or zero frames", "ani")
    icons = [c for c in walked if c.kind == b"icon" and fram[0].body < c.body < fram[0].end]
    if len(icons) != frames:
        return Observation("fail", "Icon chunk count differs from anih frame count", "ani")
    for icon in icons:
        if data[icon.body : icon.body + 4] not in (b"\0\0\1\0", b"\0\0\2\0"):
            return Observation("fail", "Frame is not an icon or cursor resource", "ani")
    return Observation("pass", f"{frames} cursor frames and animation header checked", "ani")


def webp(data: bytes, walked: list[Chunk], top: list[Chunk]) -> Observation:
    if not top or top[0].kind not in (b"VP8 ", b"VP8L", b"VP8X"):
        return Observation("inconclusive", "No supported initial image chunk", "webp")
    return Observation(*decode(data, "WEBP"), "webp")
