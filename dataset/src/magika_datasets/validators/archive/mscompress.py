# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Microsoft SZDD and KWAJ compressed files: bounded LZSS/MSZIP expansion checked against the declared length."""

import struct
import zlib

from .._shared import decompress
from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("mscompress",)
SCOPE = "SZDD and KWAJ headers, optional KWAJ fields, LZSS (SZDD/KWAJ method 2) decoded completely against the declared length, XOR and stored methods, MSZIP blocks inflated within budget; LZ+Huffman (method 3) not decoded"
SZDD = b"SZDD\x88\xf0\x27\x33"
KWAJ = b"KWAJ\x88\xf0\x27\xd1"
LIMIT = 64 * 1024 * 1024


class Malformed(Exception):
    pass


def lzss(data: bytes, offset: int, initial: int) -> int:
    """Expand an LZSS stream (4 KiB ring buffer); returns the expanded length."""
    window = bytearray(b" " * 4096)
    position = initial
    produced = 0
    index = offset
    while index < len(data):
        control = data[index]
        index += 1
        for bit in range(8):
            if index >= len(data):
                return produced
            if control >> bit & 1:
                window[position] = data[index]
                position = (position + 1) & 0xFFF
                index += 1
                produced += 1
            else:
                if index + 2 > len(data):
                    raise Malformed("Truncated match")
                low, high = data[index], data[index + 1]
                index += 2
                start, length = low | (high & 0xF0) << 4, (high & 0x0F) + 3
                for step in range(length):
                    window[position] = window[(start + step) & 0xFFF]
                    position = (position + 1) & 0xFFF
                produced += length
            if produced > LIMIT:
                raise Malformed("Expanded output exceeds budget")
    return produced


def mszip(data: bytes, offset: int) -> int:
    produced = 0
    while offset < len(data):
        size = struct.unpack_from("<H", data, offset)[0]
        block = data[offset + 2 : offset + 2 + size]
        if len(block) != size or block[:2] != b"CK":
            raise Malformed("MSZIP block framing invalid")
        expanded, rest = decompress.drain(zlib.decompressobj(-15), block[2:], LIMIT - produced)
        produced += expanded
        offset += 2 + size
    return produced


def kwaj(data: bytes) -> Observation:
    method, offset, flags = struct.unpack_from("<HHH", data, 8)
    position, declared = 14, None
    if flags & 1:
        declared = struct.unpack_from("<I", data, position)[0]
        position += 4
    if flags & 2:
        position += 2
    if flags & 4:
        position += 2 + struct.unpack_from("<H", data, position)[0]
    for _ in range(2 if flags & 8 else 0, 2 if flags & 0x10 else 0):
        pass
    if flags & 8:
        position = data.index(b"\0", position) + 1
    if flags & 0x10:
        position = data.index(b"\0", position) + 1
    if flags & 0x20:
        position += 2 + struct.unpack_from("<H", data, position)[0]
    if position > offset:
        return Observation("fail", "Optional fields overrun the data offset", "mscompress")
    if method == 0:
        expanded = len(data) - offset
    elif method == 1:
        expanded = len(data) - offset
    elif method == 2:
        expanded = lzss(data, offset, 4096 - 16)
    elif method == 4:
        expanded = mszip(data, offset)
    elif method == 3:
        return Observation(
            "inconclusive", "LZ+Huffman payload not decoded", "mscompress", ("kwaj",)
        )
    else:
        return Observation("fail", f"Unknown KWAJ method {method}", "mscompress")
    if declared is not None and declared != expanded:
        return Observation(
            "fail", f"Expanded {expanded} bytes, header declares {declared}", "mscompress"
        )
    return Observation(
        "pass", f"KWAJ method {method} expanded to {expanded} bytes", "mscompress", ("kwaj",)
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data.startswith(SZDD):
        if len(data) < 14 or data[8] != 0x41:
            return Observation("fail", "Bad SZDD mode byte or truncated header", "mscompress")
        declared = struct.unpack_from("<I", data, 10)[0]
        try:
            expanded = lzss(data, 14, 4096 - 16)
        except Malformed as error:
            return Observation("fail", str(error), "mscompress")
        if expanded != declared:
            return Observation(
                "fail", f"Expanded {expanded} bytes, header declares {declared}", "mscompress"
            )
        return Observation(
            "pass", f"SZDD stream expanded to {declared} bytes", "mscompress", ("szdd",)
        )
    if data.startswith(KWAJ):
        if len(data) < 14:
            return Observation("fail", "Truncated KWAJ header", "mscompress")
        try:
            return kwaj(data)
        except (Malformed, decompress.Truncated, decompress.Trailing, zlib.error) as error:
            return Observation("fail", str(error) or "Corrupt compressed data", "mscompress")
        except decompress.Budget as error:
            return Observation("inconclusive", str(error), "mscompress")
        except (ValueError, struct.error):
            return Observation("fail", "Optional fields outside file", "mscompress")
    return None
