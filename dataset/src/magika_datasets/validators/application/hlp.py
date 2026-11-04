# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""WinHelp files: header, internal directory B+tree and every internal file's extent."""

import struct

from ..contract import Observation

FAMILY = "application"
FORMAT_IDS = ("hlp",)
SCOPE = "Header magic, declared file size against the file, directory FILEHEADER and B+tree pages, every internal file's FILEHEADER inside the file with used space (plus header) within reserved space; internal file contents not decoded"
MAGIC = b"?_\x03\x00"
FILES = 65536


class Malformed(Exception):
    pass


def leaf_entries(data: bytes, page: int, page_size: int, base: int) -> list[int]:
    start = base + 38 + page * page_size
    if start + 8 > len(data):
        raise Malformed("B+tree page outside file")
    _, count, _, _ = struct.unpack_from("<HHhh", data, start)
    position, offsets = start + 8, []
    for _ in range(count):
        end = data.find(b"\0", position, start + page_size)
        if end < 0 or end + 5 > start + page_size:
            raise Malformed("B+tree leaf entry malformed")
        offsets.append(struct.unpack_from("<I", data, end + 1)[0])
        position = end + 5
    return offsets


def index_pages(data: bytes, page: int, page_size: int, base: int) -> list[int]:
    start = base + 38 + page * page_size
    if start + 6 > len(data):
        raise Malformed("B+tree page outside file")
    _, count, previous = struct.unpack_from("<HHh", data, start)
    pages, position = [previous], start + 6
    for _ in range(count):
        end = data.find(b"\0", position, start + page_size)
        if end < 0 or end + 3 > start + page_size:
            raise Malformed("B+tree index entry malformed")
        pages.append(struct.unpack_from("<h", data, end + 1)[0])
        position = end + 3
    return pages


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC) or len(data) < 16:
        return None
    directory, first_free, size = struct.unpack_from("<IiI", data, 4)
    if size != len(data):
        return Observation("fail", f"Header declares {size} bytes, file has {len(data)}", "hlp")
    if directory + 9 + 38 > len(data):
        return Observation("fail", "Directory outside file", "hlp")
    reserved, used, _ = struct.unpack_from("<IIB", data, directory)
    if used + 9 > reserved or directory + reserved > len(data):
        return Observation("fail", "Directory FILEHEADER sizes invalid", "hlp")
    base = directory + 9
    magic, _, page_size = struct.unpack_from("<HHH", data, base)
    root, _, total_pages, levels, entries = struct.unpack_from("<hhHHI", data, base + 26)
    if magic != 0x293B or page_size == 0 or levels == 0 or root < 0:
        return Observation("fail", "Directory B+tree header invalid", "hlp")
    if base + 38 + total_pages * page_size > len(data) or entries > FILES:
        return Observation("fail", "B+tree pages outside file", "hlp")
    try:
        pages = [root]
        for _ in range(levels - 1):
            pages = [child for page in pages for child in index_pages(data, page, page_size, base)]
        offsets = [offset for page in pages for offset in leaf_entries(data, page, page_size, base)]
    except (Malformed, struct.error) as error:
        return Observation("fail", str(error) or "B+tree truncated", "hlp")
    if len(offsets) != entries:
        return Observation("fail", "B+tree entry count disagrees with its leaves", "hlp")
    for offset in offsets:
        if offset + 9 > len(data):
            return Observation("fail", "Internal file header outside file", "hlp")
        reserved, used, _ = struct.unpack_from("<IIB", data, offset)
        if used + 9 > reserved or offset + reserved > len(data):
            return Observation("fail", "Internal file extent outside file", "hlp")
    return Observation(
        "pass",
        f"{entries} internal files; declared size, directory B+tree and file extents verified",
        "hlp",
    )
