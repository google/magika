# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Dalvik executables: header, Adler-32, SHA-1 signature, map list and table offsets."""

import hashlib
import struct
import zlib

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("dex",)
SCOPE = "dex magic and version, file size equal to the file, Adler-32 checksum and SHA-1 signature verified, header size, map list and every id/data table inside the file; nothing executed"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    return check(data)


def check(data: bytes, *, signature: bool = True) -> Observation | None:
    """Validate a DEX; `signature=False` skips the SHA-1, which dexopt leaves stale in odex files."""
    if data[:4] != b"dex\n" or len(data) < 0x70:
        return None
    version = data[4:8]
    if not (version[:3].isdigit() and version[3] == 0):
        return Observation("fail", "Invalid dex version", "dex")
    checksum, stored_signature = struct.unpack_from("<I", data, 8)[0], data[12:32]
    file_size, header_size, endian = struct.unpack_from("<III", data, 32)
    if file_size != len(data) or header_size != 0x70 or endian not in (0x12345678, 0x78563412):
        return Observation("fail", "Header size, file size or endian tag invalid", "dex")
    if zlib.adler32(data[12:]) != checksum:
        return Observation("fail", "Adler-32 checksum mismatch", "dex")
    if signature and hashlib.sha1(data[32:]).digest() != stored_signature:
        return Observation("fail", "SHA-1 signature mismatch", "dex")
    link_size, link_off, map_off = struct.unpack_from("<III", data, 44)
    if map_off + 4 > len(data) or (link_size and link_off + link_size > len(data)):
        return Observation("fail", "Map list or link section outside file", "dex")
    items = struct.unpack_from("<I", data, map_off)[0]
    if items > 4096 or map_off + 4 + 12 * items > len(data):
        return Observation("fail", "Map list outside file", "dex")
    for index in range(items):
        _, _, _, offset = struct.unpack_from("<HHII", data, map_off + 4 + 12 * index)
        if offset > len(data):
            return Observation("fail", "Map item outside file", "dex")
    tables = struct.unpack_from(
        "<" + "II" * 6, data, 56
    )  # string, type, proto, field, method, class (size, offset)
    widths = (4, 4, 12, 8, 8, 32)
    for index, width in enumerate(widths):
        size, offset = tables[2 * index], tables[2 * index + 1]
        if size and offset + size * width > len(data):
            return Observation("fail", f"Table {index} outside file", "dex")
    return Observation(
        "pass",
        f"dex {version[:3].decode()}: checksum, signature and {items} map items bounded",
        "dex",
        (f"dex_version_{version[:3].decode()}",),
    )
