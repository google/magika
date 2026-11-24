# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""PNG framing and critical-chunk checks: https://www.w3.org/TR/png-3/."""

import struct
import zlib

from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("png",)

SIGNATURE = b"\x89PNG\r\n\x1a\n"
ADAM7 = (
    (0, 0, 8, 8),
    (4, 0, 8, 8),
    (0, 4, 4, 8),
    (2, 0, 4, 4),
    (0, 2, 2, 4),
    (1, 0, 2, 2),
    (0, 1, 1, 2),
)
SCOPE = "Signature, chunk bounds/CRC, IHDR fields, critical ordering and palette; bounded zlib/scanline validation; Adam7 interlaced sizes; no APNG validation"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(SIGNATURE):
        return None
    offset, seen, ended_idat = 8, [], False
    color = depth = None
    compressed = bytearray()
    while offset < len(data):
        if len(seen) >= 4096:
            return Observation("inconclusive", "Chunk count exceeds 4096", "png")
        if offset + 12 > len(data):
            return Observation("fail", "Truncated chunk header/trailer", "png")
        length = int.from_bytes(data[offset : offset + 4], "big")
        kind = data[offset + 4 : offset + 8]
        end = offset + 12 + length
        if length > 0x7FFFFFFF or end > len(data):
            return Observation("fail", "Chunk length exceeds file bounds", "png")
        body = data[offset + 8 : end - 4]
        if zlib.crc32(kind + body) != int.from_bytes(data[end - 4 : end], "big"):
            return Observation("fail", "Chunk CRC mismatch", "png")
        if not all(65 <= c <= 90 or 97 <= c <= 122 for c in kind) or kind[2] & 32:
            return Observation("fail", "Invalid chunk type", "png")
        if not seen and kind != b"IHDR":
            return Observation("fail", "IHDR must be first", "png")
        if kind == b"IHDR":
            if seen or length != 13:
                return Observation("fail", "Duplicate or malformed IHDR", "png")
            width, height, depth, color, compression, filtering, interlace = struct.unpack(
                ">IIBBBBB", body
            )
            depths = {0: {1, 2, 4, 8, 16}, 2: {8, 16}, 3: {1, 2, 4, 8}, 4: {8, 16}, 6: {8, 16}}
            if (
                not 0 < width <= 0x7FFFFFFF
                or not 0 < height <= 0x7FFFFFFF
                or depth not in depths.get(color, set())
                or compression
                or filtering
                or interlace not in (0, 1)
            ):
                return Observation("fail", "Invalid IHDR fields", "png")
        elif kind == b"PLTE":
            if (
                b"PLTE" in seen
                or b"IDAT" in seen
                or not length
                or length % 3
                or length > 768
                or color in (0, 4)
                or (color == 3 and length // 3 > 2**depth)
            ):
                return Observation("fail", "Invalid palette size or ordering", "png")
        elif kind == b"IDAT":
            compressed.extend(body)
            if ended_idat or (color == 3 and b"PLTE" not in seen):
                return Observation("fail", "Nonconsecutive IDAT or missing indexed palette", "png")
        elif kind == b"IEND":
            if length or b"IDAT" not in seen or end != len(data):
                return Observation(
                    "fail", "Invalid IEND, missing image data or trailing bytes", "png"
                )
            if b"acTL" in seen:
                return Observation(
                    "inconclusive", "Animated image validation is not implemented", "png"
                )
            channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[color]
            row_bytes = (width * depth * channels + 7) // 8
            if interlace:
                expected = 0
                for x0, y0, dx, dy in ADAM7:
                    pass_width = (width - x0 + dx - 1) // dx
                    pass_height = (height - y0 + dy - 1) // dy
                    if pass_width and pass_height:
                        expected += pass_height * ((pass_width * depth * channels + 7) // 8 + 1)
            else:
                expected = height * (row_bytes + 1)
            if expected > 16 * 1024 * 1024:
                return Observation("inconclusive", "Decoded image exceeds 16 MiB limit", "png")
            try:
                decoder = zlib.decompressobj()
                pixels = decoder.decompress(compressed, expected + 1)
            except zlib.error:
                return Observation("fail", "Invalid compressed image stream", "png")
            if (
                len(pixels) != expected
                or not decoder.eof
                or decoder.unused_data
                or decoder.unconsumed_tail
            ):
                return Observation(
                    "fail",
                    "Image stream length, termination or trailing compressed data mismatch",
                    "png",
                )
            if not interlace and any(pixels[i] > 4 for i in range(0, expected, row_bytes + 1)):
                return Observation("fail", "Invalid scanline filter", "png")
            return Observation(
                "pass",
                "Critical structure, CRCs, zlib stream and scanline lengths/filters checked",
                "png",
            )

        elif not kind[0] & 32:
            return Observation("inconclusive", "Unsupported critical chunk", "png")
        if b"IDAT" in seen and kind != b"IDAT":
            ended_idat = True
        seen.append(kind)
        offset = end
    return Observation("fail", "Missing IEND", "png")
