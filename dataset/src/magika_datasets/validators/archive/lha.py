# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""LHA/LZH archives: level 0, 1 and 2 headers, checksums and packed sizes tiling the file."""

import re
import struct

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("lha",)
SCOPE = "Header level 0/1/2 layout, header checksum (levels 0 and 1) or CRC-16 (level 2), method ids, extension header chains, packed sizes chaining to the end marker at EOF, CRC-16 of stored entries; compressed entries not decoded"
METHOD = re.compile(rb"-(lh[0-9a-z]|lz[s45]|pm[0-2s])-")
ENTRIES = 65536
STORED = (b"-lh0-", b"-lz4-", b"-pm0-")

TABLE = []
for index in range(256):
    value = index
    for _ in range(8):
        value = (value >> 1) ^ 0xA001 if value & 1 else value >> 1
    TABLE.append(value)


def crc16(data: bytes) -> int:
    value = 0
    for byte in data:
        value = (value >> 8) ^ TABLE[(value ^ byte) & 0xFF]
    return value


class Malformed(Exception):
    pass


def extensions(data: bytes, offset: int, limit: int) -> tuple[int, int | None, int | None]:
    """Walk an extension chain whose first size field sits at offset.

    Each extension is `size` bytes: a type byte, data, and the next size field as its last
    two bytes. Returns (end of chain, header-CRC field offset, stored header CRC).
    """
    crc_offset = crc = None
    size = struct.unpack_from("<H", data, offset)[0]
    offset += 2
    for _ in range(256):
        if size == 0:
            return offset, crc_offset, crc
        if size < 3 or offset + size > limit:
            raise Malformed("Extension header size invalid")
        if data[offset] == 0 and size >= 5 and crc is None:
            crc_offset, crc = offset + 1, struct.unpack_from("<H", data, offset + 1)[0]
        offset += size - 2
        size = struct.unpack_from("<H", data, offset)[0]
        offset += 2
    raise Malformed("Extension header budget exceeded")


def entry(data: bytes, offset: int) -> tuple[int, bytes, int, int, int]:
    """(next offset, method, packed size, data start, stored CRC) for the entry at offset."""
    if offset + 22 > len(data):
        raise Malformed("Truncated header")
    method = data[offset + 2 : offset + 7]
    if not METHOD.fullmatch(method):
        raise Malformed("Unknown compression method")
    packed = struct.unpack_from("<I", data, offset + 7)[0]
    level = data[offset + 20]
    if level in (0, 1):
        size = data[offset]
        header_end = offset + 2 + size
        if header_end > len(data):
            raise Malformed("Header outside file")
        if sum(data[offset + 2 : header_end]) & 0xFF != data[offset + 1]:
            raise Malformed("Header checksum mismatch")
        name_length = data[offset + 21]
        crc_offset = offset + 22 + name_length
        if crc_offset + 2 > header_end:
            raise Malformed("Name exceeds header")
        crc = struct.unpack_from("<H", data, crc_offset)[0]
        if (
            level == 1
        ):  # the base header ends with the first extension size; extensions count as packed bytes
            chain_end, _, _ = extensions(data, header_end - 2, len(data))
            extra = chain_end - header_end
            if extra > packed:
                raise Malformed("Extension headers exceed the packed size")
            return header_end + packed, method, packed - extra, chain_end, crc
        return header_end + packed, method, packed, header_end, crc
    if level == 2:
        size = struct.unpack_from("<H", data, offset)[0]
        header_end = offset + size
        if size < 26 or header_end > len(data):
            raise Malformed("Header outside file")
        crc = struct.unpack_from("<H", data, offset + 21)[0]
        chain_end, crc_offset, stored = extensions(data, offset + 24, header_end)
        if chain_end != header_end:
            raise Malformed("Header size disagrees with the extension chain")
        if stored is not None:
            header = bytearray(data[offset:header_end])
            header[crc_offset - offset : crc_offset - offset + 2] = b"\0\0"
            if crc16(bytes(header)) != stored:
                raise Malformed("Header CRC-16 mismatch")
        return header_end + packed, method, packed, header_end, crc
    raise Malformed(f"Unsupported header level {level}")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 22 or not METHOD.fullmatch(data[2:7]) or data[20] > 3:
        return None
    if data[20] == 3:
        return Observation("inconclusive", "Level 3 headers are not checked", "lha")
    offset, entries, verified, levels = 0, 0, 0, set()
    try:
        while offset < len(data) and data[offset] != 0:
            levels.add(data[offset + 20] if offset + 21 <= len(data) else -1)
            offset, method, packed, start, crc = entry(data, offset)
            if offset > len(data):
                raise Malformed("Packed data outside file")
            if method in STORED and crc16(data[start : start + packed]) != crc:
                raise Malformed("CRC-16 mismatch on a stored entry")
            verified += method in STORED
            entries += 1
            if entries > ENTRIES:
                return Observation("inconclusive", "Entry budget exceeded", "lha")
    except (Malformed, struct.error) as error:
        return Observation("fail", str(error) or "Truncated header", "lha")
    tags = ()
    if offset == len(data):  # some producers omit the end marker; every entry still tiled the file
        tags = ("no_end_marker",)
    elif offset + 1 != len(data):
        return Observation("fail", "Bytes after the end marker", "lha")
    return Observation(
        "pass",
        f"{entries} entries (header levels {sorted(levels)}); headers verified, {verified} stored entries CRC-checked, entries tile the file",
        "lha",
        tags,
    )
