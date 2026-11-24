# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Windows Imaging Format: header resources, lookup table entries and XML data."""

import struct

from .._shared import der, xml
from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("wim",)
SCOPE = "Header size and version, offset table, XML data and integrity resources inside the file, every lookup-table resource inside the file, XML data parsed safely with a WIM root, referenced ranges reaching EOF or a trailing Authenticode DER blob; resources not decompressed, signatures not verified"

MAGIC = b"MSWIM\0\0\0"
ENTRIES = 1 << 20
SOLID = 1 << 32  # original-size marker of the entry describing a solid (LZMS) resource


def resource(data: bytes, offset: int) -> tuple[int, int, int]:
    """(offset, size in file, flags) of a RESHDR_DISK_SHORT."""
    packed, position = struct.unpack_from("<QQ", data, offset)
    return position, packed & ((1 << 56) - 1), packed >> 56


def original(data: bytes, offset: int) -> int:
    return struct.unpack_from("<Q", data, offset + 16)[0]


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 208:
        return Observation("fail", "Truncated header", "wim")
    header_size, version, flags, chunk = struct.unpack_from("<IIII", data, 8)
    part, parts, images = struct.unpack_from("<HHI", data, 40)
    if header_size != 208 or version not in (0x10D00, 0xE00) or part == 0 or part > parts:
        return Observation("fail", "Header size, version or part numbers invalid", "wim")
    tags = []
    if flags & 2:
        tags.append("compressed")
    if parts > 1:
        tags.append("split")
    table = resource(data, 48)
    metadata = resource(data, 72)
    boot = resource(data, 96)
    integrity = resource(data, 124)
    end = header_size
    for name, (offset, size, _) in (
        ("offset table", table),
        ("XML data", metadata),
        ("boot metadata", boot),
        ("integrity table", integrity),
    ):
        if offset + size > len(data):
            return Observation("fail", f"{name} outside file", "wim")
        end = max(end, offset + size)
    if not table[1] or table[1] % 50:
        return Observation("fail", "Lookup table missing or not a multiple of 50 bytes", "wim")
    count = table[1] // 50
    if count > ENTRIES:
        return Observation("inconclusive", "Lookup table budget exceeded", "wim")
    for index in range(count):
        offset, size, entry_flags = resource(data, table[0] + index * 50)
        entry_part = struct.unpack_from("<H", data, table[0] + index * 50 + 24)[0]
        if entry_part != part:
            continue  # lives in another part of a split image
        if entry_flags & 0x10 and original(data, table[0] + index * 50) != SOLID:
            tags.append("solid") if "solid" not in tags else None
            continue  # solid entries are offsets inside a packed resource, not file offsets
        if offset + size > len(data):
            return Observation("fail", "Lookup table resource outside file", "wim")
        end = max(end, offset + size)
    if metadata[2] & 0x04 or not metadata[1]:
        return Observation("inconclusive", "XML data is compressed or missing", "wim")
    try:
        root = xml.parse(data[metadata[0] : metadata[0] + metadata[1]].decode("utf-16-le").encode())
    except (UnicodeDecodeError, xml.Malformed, xml.Unsupported):
        return Observation("fail", "XML data is not well-formed UTF-16 XML", "wim")
    except xml.Unsafe:
        return Observation("inconclusive", "XML data uses entities or external references", "wim")
    if root.tag != "WIM":
        return Observation("fail", "XML data root is not WIM", "wim")
    if end != len(data) and data[end] == 0x30:  # Authenticode PKCS#7 blob after the integrity table
        try:
            signed = der.total(data, end)
        except der.Malformed:
            signed = 0
        if end + signed == len(data):
            tags.append("signed")
            end = len(data)
    if end != len(data):
        return Observation(
            "fail", f"{len(data) - end} bytes after the last referenced resource", "wim"
        )
    return Observation(
        "pass",
        f"{images} images, {count} lookup entries; header resources, lookup table and XML data verified, ranges reach EOF",
        "wim",
        tuple(tags),
    )
