# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Shockwave Flash: signature, declared length, bounded decompression and tag stream."""

import lzma
import struct
import zlib

from .._shared import decompress
from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("swf",)
SCOPE = "FWS/CWS/ZWS signature, declared uncompressed length equal to the (inflated) body, frame RECT and header, every tag's code and length, End tag closing the file; ActionScript never executed"
TAGS = 100_000


def inflate(kind: bytes, data: bytes) -> bytes:
    if kind == b"FWS":
        return data[8:]
    if kind == b"CWS":
        expanded, rest = decompress.drain(zlib.decompressobj(), data[8:], decompress.LIMIT)
        if rest:
            raise decompress.Trailing("Trailing bytes after zlib body")
        return zlib.decompress(data[8:])
    compressed_size = struct.unpack_from("<I", data, 8)[0]
    properties, payload = data[12:17], data[17:]
    if compressed_size != len(payload):
        raise decompress.Truncated("LZMA body length differs from header")
    alone = properties + struct.pack("<Q", struct.unpack_from("<I", data, 4)[0] - 8) + payload
    expanded, rest = decompress.drain(
        lzma.LZMADecompressor(lzma.FORMAT_ALONE), alone, decompress.LIMIT
    )
    if rest:
        raise decompress.Trailing("Trailing bytes after LZMA body")
    return lzma.decompress(alone, format=lzma.FORMAT_ALONE)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    kind = data[:3]
    if kind not in (b"FWS", b"CWS", b"ZWS") or len(data) < 8:
        return None
    declared = struct.unpack_from("<I", data, 4)[0]
    try:
        body = inflate(kind, data)
    except decompress.Budget as error:
        return Observation("inconclusive", str(error), "swf")
    except (
        decompress.Truncated,
        decompress.Trailing,
        zlib.error,
        lzma.LZMAError,
        struct.error,
    ) as error:
        return Observation("fail", f"Body decompression failed: {error}", "swf")
    if 8 + len(body) != declared:
        return Observation("fail", "Declared length differs from body", "swf")
    if not body:
        return Observation("fail", "Empty body", "swf")
    nbits = body[0] >> 3
    offset = (5 + 4 * nbits + 7) // 8 + 4  # RECT, frame rate, frame count
    if offset > len(body):
        return Observation("fail", "Truncated frame header", "swf")
    count, ended = 0, False
    while offset < len(body):
        count += 1
        if count > TAGS:
            return Observation("inconclusive", "Tag budget exceeded", "swf")
        if offset + 2 > len(body):
            return Observation("fail", "Truncated tag header", "swf")
        code_and_length = struct.unpack_from("<H", body, offset)[0]
        code, length = code_and_length >> 6, code_and_length & 0x3F
        offset += 2
        if length == 63:
            if offset + 4 > len(body):
                return Observation("fail", "Truncated long tag length", "swf")
            length = struct.unpack_from("<I", body, offset)[0]
            offset += 4
        if offset + length > len(body):
            return Observation("fail", "Tag exceeds body", "swf")
        offset += length
        if code == 0:
            ended = offset == len(body)
            break
    if not ended:
        return Observation("fail", "Tag stream does not end with an End tag at EOF", "swf")
    tags = {b"CWS": ("compressed_zlib",), b"ZWS": ("compressed_lzma",)}.get(kind, ())
    return Observation("pass", f"{count} SWF tags bounded; declared length verified", "swf", tags)
