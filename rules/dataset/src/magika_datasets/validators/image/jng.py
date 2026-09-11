# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""JPEG Network Graphics: PNG-style chunk framing around a JPEG stream."""

import struct
import zlib

from .._shared.pillow import decode
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("jng",)
SCOPE = "Signature, chunk bounds and CRCs, JHDR fields, JDAT presence and IEND at EOF; the concatenated JPEG stream is decoded with Pillow; alpha chunks not decoded"
SIGNATURE = b"\x8bJNG\r\n\x1a\n"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(SIGNATURE):
        return None
    offset, count, header, jpeg = 8, 0, None, bytearray()
    while offset < len(data):
        count += 1
        if count > 4096:
            return Observation("inconclusive", "Chunk budget exceeded", "jng")
        if offset + 12 > len(data):
            return Observation("fail", "Truncated chunk header/trailer", "jng")
        length = struct.unpack_from(">I", data, offset)[0]
        kind = data[offset + 4 : offset + 8]
        end = offset + 12 + length
        if end > len(data):
            return Observation("fail", "Chunk length exceeds file bounds", "jng")
        body = data[offset + 8 : end - 4]
        if zlib.crc32(kind + body) != struct.unpack_from(">I", data, end - 4)[0]:
            return Observation("fail", "Chunk CRC mismatch", "jng")
        if header is None:
            if kind != b"JHDR" or length != 16:
                return Observation("fail", "JHDR must be the first chunk", "jng")
            header = struct.unpack(">IIBBBBBBBB", body)
            width, height, color, depth, compression = header[:5]
            if not width or not height or color not in (8, 10, 12, 14):
                return Observation("fail", "Invalid JHDR dimensions or color type", "jng")
            if depth not in (8, 12, 20) or compression != 8:
                return Observation("fail", "Invalid JHDR sample depth or compression", "jng")
        elif kind == b"JDAT":
            jpeg.extend(body)
        elif kind == b"IEND":
            if length or end != len(data):
                return Observation("fail", "Invalid IEND or trailing bytes", "jng")
            if not jpeg:
                return Observation("fail", "No JDAT image data", "jng")
            if not jpeg.startswith(b"\xff\xd8\xff"):
                return Observation("fail", "JDAT stream is not JPEG", "jng")
            status, detail = decode(bytes(jpeg), "JPEG")
            return Observation(status, f"JNG chunks checked; JPEG stream: {detail}", "jng")
        offset = end
    return Observation("fail", "Missing IEND", "jng")
