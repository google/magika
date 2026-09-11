# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""7-Zip archives: signature header CRC and the next-header placement and CRC."""

import struct
import zlib

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("sevenzip",)
SCOPE = "Signature header (version, start-header CRC-32), packed streams and next header tiling the file, next-header CRC-32; streams not unpacked"
MAGIC = b"7z\xbc\xaf\x27\x1c"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 32:
        return Observation("fail", "Truncated signature header", "sevenzip")
    major, minor, start_crc = struct.unpack_from("<BBI", data, 6)
    offset, size, crc = struct.unpack_from("<QQI", data, 12)
    if major != 0 or minor > 4:
        return Observation("fail", f"Unsupported version {major}.{minor}", "sevenzip")
    if zlib.crc32(data[12:32]) != start_crc:
        return Observation("fail", "Start header CRC-32 mismatch", "sevenzip")
    if offset == size == crc == 0:
        if len(data) != 32:
            return Observation("fail", "Bytes after an empty archive", "sevenzip")
        return Observation("pass", "Empty archive; start header CRC-32 verified", "sevenzip")
    if size == 0 or 32 + offset + size != len(data):
        return Observation("fail", "Next header does not end at the end of the file", "sevenzip")
    header = data[32 + offset :]
    if zlib.crc32(header) != crc:
        return Observation("fail", "Next header CRC-32 mismatch", "sevenzip")
    tags = ()
    if header[0] == 0x17:
        tags = ("encoded_header",)  # header itself is packed (and possibly encrypted)
    elif header[0] != 0x01:
        return Observation("fail", "Next header is neither Header nor EncodedHeader", "sevenzip")
    return Observation(
        "pass",
        f"Packed streams of {offset} bytes and a {size}-byte next header tile the file; both CRC-32 values verified",
        "sevenzip",
        tags,
    )
