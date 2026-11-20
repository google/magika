# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""RAR 4 and RAR 5 archives: header CRCs and block chains tiling the file."""

import struct
import zlib

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("rar",)
SCOPE = "Marker, every block header's CRC (CRC-16 of CRC-32 for RAR 4, CRC-32 for RAR 5), header and data sizes chaining to the end-of-archive block at EOF; packed data not decoded"
MAGIC4 = b"Rar!\x1a\x07\x00"
MAGIC5 = b"Rar!\x1a\x07\x01\x00"
BLOCKS = 65536


class Malformed(Exception):
    pass


def vint(data: bytes, offset: int) -> tuple[int, int]:
    value, shift = 0, 0
    for _ in range(10):
        if offset >= len(data):
            raise Malformed("Truncated variable-length integer")
        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        shift += 7
        if not byte & 0x80:
            return value, offset
    raise Malformed("Variable-length integer too long")


def rar5(data: bytes) -> Observation:
    offset, blocks, tags = len(MAGIC5), 0, set()
    while True:
        if offset + 7 > len(data):
            raise Malformed("Archive ends before an end-of-archive block")
        stored = struct.unpack_from("<I", data, offset)[0]
        size, position = vint(data, offset + 4)
        header_end = position + size
        if header_end > len(data):
            raise Malformed("Block header outside file")
        if zlib.crc32(data[offset + 4 : header_end]) != stored:
            raise Malformed("Block header CRC-32 mismatch")
        kind, position = vint(data, position)
        flags, position = vint(data, position)
        if flags & 1:
            _, position = vint(data, position)
        payload = 0
        if flags & 2:
            payload, position = vint(data, position)
        if kind == 4:
            return Observation(
                "inconclusive",
                "Headers are encrypted; block chain cannot be verified",
                "rar",
                ("encrypted",),
            )
        if kind == 2 and flags & 0x40:
            tags.add("solid")
        offset = header_end + payload
        blocks += 1
        if blocks > BLOCKS:
            raise Malformed("Block budget exceeded")
        if kind == 5:
            if offset != len(data):
                raise Malformed("Bytes after the end-of-archive block")
            return Observation(
                "pass",
                f"RAR 5, {blocks} blocks; header CRC-32 values and block chain verified to EOF",
                "rar",
                tuple(sorted(tags)),
            )
        if offset > len(data):
            raise Malformed("Block data outside file")


def rar4(data: bytes) -> Observation:
    offset, blocks, tags = len(MAGIC4), 0, set()
    while True:
        if offset + 7 > len(data):
            raise Malformed("Archive ends before an end-of-archive block")
        stored, kind, flags, size = struct.unpack_from("<HBHH", data, offset)
        if size < 7 or offset + size > len(data):
            raise Malformed("Block header outside file")
        if zlib.crc32(data[offset + 2 : offset + size]) & 0xFFFF != stored:
            raise Malformed("Block header CRC-16 mismatch")
        payload = 0
        if flags & 0x8000:
            if offset + 11 > len(data):
                raise Malformed("Truncated block size")
            payload = struct.unpack_from("<I", data, offset + 7)[0]
            if (
                kind == 0x74 and flags & 0x100
            ):  # 64-bit sizes: high 32 bits follow the name-size fields
                payload |= struct.unpack_from("<I", data, offset + 7 + 25)[0] << 32
        if kind == 0x73:
            if flags & 0x80:
                return Observation(
                    "inconclusive",
                    "Headers are encrypted; block chain cannot be verified",
                    "rar",
                    ("encrypted",),
                )
            if flags & 0x08:
                tags.add("solid")
        if kind == 0x74 and flags & 0x04:
            tags.add("encrypted")
        offset += size + payload
        blocks += 1
        if blocks > BLOCKS:
            raise Malformed("Block budget exceeded")
        if kind == 0x7B:
            if offset != len(data):
                raise Malformed("Bytes after the end-of-archive block")
            return Observation(
                "pass",
                f"RAR 4, {blocks} blocks; header CRC-16 values and block chain verified to EOF",
                "rar",
                tuple(sorted(tags)),
            )
        if offset > len(data):
            raise Malformed("Block data outside file")
        if offset == len(data):
            return Observation(
                "inconclusive", "Archive ends without an end-of-archive block", "rar"
            )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data.startswith(MAGIC5):
        walker = rar5
    elif data.startswith(MAGIC4):
        walker = rar4
    else:
        return None
    try:
        return walker(data)
    except (Malformed, struct.error) as error:
        return Observation("fail", str(error) or "Truncated block", "rar")
