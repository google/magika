# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Compiled HTML Help (ITSS): header sections, directory listing chunks and uncompressed-section entries."""

import struct

from ..contract import Observation

FAMILY = "application"
FORMAT_IDS = ("chm",)
SCOPE = "ITSF header and section table, section 0 file size against the file, ITSP directory header, PMGL/PMGI chunks tiling the directory, entry names and ENCINT fields, uncompressed-section entries inside the content area; LZX content not decompressed"
MAGIC = b"ITSF"
ENTRIES = 1 << 20


class Malformed(Exception):
    pass


def encint(data: bytes, offset: int, end: int) -> tuple[int, int]:
    value = 0
    for _ in range(10):
        if offset >= end:
            raise Malformed("Truncated ENCINT")
        byte = data[offset]
        offset += 1
        value = value << 7 | byte & 0x7F
        if not byte & 0x80:
            return value, offset
    raise Malformed("ENCINT too long")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 0x60:
        return Observation("fail", "Truncated ITSF header", "chm")
    version, header_length = struct.unpack_from("<II", data, 4)
    if version not in (2, 3) or header_length < 0x58:
        return Observation("fail", "Unsupported ITSF version or header length", "chm")
    section0_offset, section0_length, section1_offset, section1_length = struct.unpack_from(
        "<QQQQ", data, 0x38
    )
    content_offset = (
        struct.unpack_from("<Q", data, 0x58)[0]
        if version == 3
        else section1_offset + section1_length
    )
    if section0_offset + section0_length > len(data) or section1_offset + section1_length > len(
        data
    ):
        return Observation("fail", "Header sections outside file", "chm")
    if section0_length < 0x18 or struct.unpack_from("<Q", data, section0_offset + 8)[0] != len(
        data
    ):
        return Observation("fail", "Section 0 file size disagrees with the file", "chm")
    directory = section1_offset
    if data[directory : directory + 4] != b"ITSP" or section1_length < 0x54:
        return Observation("fail", "ITSP directory header missing", "chm")
    chunk_size = struct.unpack_from("<I", data, directory + 0x10)[0]
    directory_length = struct.unpack_from("<I", data, directory + 0x08)[0]
    chunks = struct.unpack_from("<I", data, directory + 0x2C)[0]
    if (
        chunk_size == 0
        or directory + directory_length + chunks * chunk_size != section1_offset + section1_length
    ):
        return Observation("fail", "Directory chunks do not tile section 1", "chm")
    content_size = len(data) - content_offset
    entries, uncompressed = 0, 0
    try:
        for index in range(chunks):
            start = directory + directory_length + index * chunk_size
            kind = data[start : start + 4]
            if kind == b"PMGI":
                continue
            if kind != b"PMGL":
                raise Malformed("Directory chunk is neither PMGL nor PMGI")
            free = struct.unpack_from("<I", data, start + 4)[0]
            position, end = start + 0x14, start + chunk_size - free
            if free > chunk_size - 0x14:
                raise Malformed("PMGL free space exceeds the chunk")
            while position < end:
                length, position = encint(data, position, end)
                name = data[position : position + length]
                position += length
                if len(name) != length or b"\0" in name:
                    raise Malformed("Entry name malformed")
                section, position = encint(data, position, end)
                offset, position = encint(data, position, end)
                size, position = encint(data, position, end)
                if section == 0 and offset + size > content_size:
                    raise Malformed("Uncompressed entry outside the content area")
                entries += 1
                uncompressed += section == 0
                if entries > ENTRIES:
                    raise Malformed("Entry budget exceeded")
    except (Malformed, struct.error) as error:
        return Observation("fail", str(error) or "Directory truncated", "chm")
    if entries == 0:
        return Observation("fail", "Directory lists no entries", "chm")
    return Observation(
        "pass",
        f"{entries} directory entries ({uncompressed} uncompressed); header sizes, directory chunks and entry extents verified",
        "chm",
    )
