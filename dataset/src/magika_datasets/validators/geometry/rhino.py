# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Rhino 3DM (openNURBS) files: top-level chunks tiling the file to the end-of-file chunk."""

import struct
import zlib

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("rhinoceros",)
SCOPE = "32-byte header naming the archive version, a leading comment block, top-level chunks (4-byte lengths before version 5, 8-byte from 5) tiling the file, short chunks carrying no data, CRC-32 of every chunk that declares one, and a final TCODE_ENDOFFILE whose value equals the file size; object and table contents not interpreted"
MAGIC = b"3D Geometry File Format "
COMMENT_BLOCK = 0x00000001
END_OF_FILE = 0x00007FFF
SHORT = 0x80000000
CRC = 0x00008000
CHUNKS = 1_000_000


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    field = data[24:32].strip()
    if len(data) < 32 or not field.isdigit():
        return Observation("fail", "Archive version field is not a number", "rhinoceros")
    version = int(field)
    width = 8 if version >= 50 else 4  # version 5 onward is written as 50, 60, ...
    length = struct.Struct("<Q" if width == 8 else "<I")
    offset, chunks = 32, 0
    try:
        while True:
            (code,) = struct.unpack_from("<I", data, offset)
            (size,) = length.unpack_from(data, offset + 4)
            offset += 4 + width
            if chunks == 0 and code != COMMENT_BLOCK:
                raise ValueError("First chunk is not the comment block")
            chunks += 1
            if chunks > CHUNKS:
                return Observation("inconclusive", "Chunk budget exceeded", "rhinoceros")
            if code & SHORT:
                continue
            if offset + size > len(data):
                raise ValueError(f"Chunk {code:#010x} extends past the end of the file")
            body = data[offset : offset + size]
            if code == END_OF_FILE:
                if size != width:
                    raise ValueError("End-of-file chunk has the wrong size")
                (declared,) = length.unpack(body)
                if declared != len(data) or offset + size != len(data):
                    raise ValueError("End-of-file chunk does not match the file size")
                break
            if code & CRC and size >= 4:
                if zlib.crc32(body[:-4]) != struct.unpack("<I", body[-4:])[0]:
                    raise ValueError(f"Chunk {code:#010x} fails its CRC-32")
            offset += size
    except struct.error:
        return Observation(
            "fail", "Ends inside a chunk header, before TCODE_ENDOFFILE", "rhinoceros"
        )
    except ValueError as error:
        return Observation("fail", str(error), "rhinoceros")
    release = version // 10 if version >= 50 else version
    return Observation(
        "pass",
        f"Version {release} archive; {chunks} top-level chunks tile the file",
        "rhinoceros",
    )
