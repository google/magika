# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Windows registry hives: base block checksum and hive bins tiling the declared size."""

import re
import struct

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("winregistry", "hve")
SCOPE = "Registry export text (REGEDIT4 or Windows Registry Editor 5.00 header, key, value, continuation and comment lines to EOF) or regf base block with XOR-32 checksum over the first 508 bytes, format version, hive bins size, every hbin signature, offset and size tiling the bins region; cells not walked"
BINS = 100_000
HEADERS = (b"Windows Registry Editor Version 5.00", b"REGEDIT4")
VALUE = re.compile(
    rb'^("(?:[^"\\]|\\.)*"|@)\s*=\s*(-|"(?:[^"\\]|\\.)*"|dword:[0-9A-Fa-f]{1,8}|hex(?:\([0-9A-Fa-f]+\))?:.*)$'
)
KEY = re.compile(rb"^\[-?[^\]\r\n]+\]$")
COMMENT_TAIL = re.compile(rb"(?<=\])\s*;.*$")


def export(data: bytes) -> Observation | None:
    """Text registry exports as written by regedit."""
    if data.startswith(b"\xff\xfe"):
        try:
            text = data[2:].decode("utf-16-le").encode("utf-8")
        except UnicodeDecodeError:
            return None  # some other UTF-16 text, or not text at all
        tags = ("utf16",)
    else:
        text, tags = data.removeprefix(b"\xef\xbb\xbf"), ()
    if not text.startswith(HEADERS):
        return None
    lines = text.splitlines()
    continued, keys, values = False, 0, 0
    for number, line in enumerate(lines[1:], 2):
        stripped = line.strip()
        if continued:
            continued = stripped.endswith(b"\\")
            if not re.fullmatch(rb"[0-9A-Fa-f]{2}(,[0-9A-Fa-f]{2})*,?\\?", stripped):
                return Observation(
                    "fail", f"Line {number} is not a hex continuation", "winregistry"
                )
            continue
        if not stripped or stripped.startswith(b";"):
            continue
        stripped = COMMENT_TAIL.sub(b"", stripped).rstrip()  # regedit ignores trailing comments
        if KEY.match(stripped):
            keys += 1
            continue
        if VALUE.match(stripped):
            values += 1
            continued = stripped.endswith(b"\\")
            continue
        return Observation(
            "fail", f"Line {number} is neither key, value nor comment", "winregistry"
        )
    if continued:
        return Observation("fail", "Export ends inside a continued value", "winregistry")
    return Observation(
        "pass",
        f"Registry export: {keys} keys and {values} values parsed",
        "winregistry",
        tags + ("export",),
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"regf":
        return export(data)  # text exports are winregistry; binary hives are hve
    if len(data) < 4096:
        return Observation("fail", "Truncated base block", "hve")
    primary, secondary = struct.unpack_from("<II", data, 4)
    major, minor = struct.unpack_from("<II", data, 20)
    bins_size = struct.unpack_from("<I", data, 40)[0]
    checksum = 0
    for offset in range(0, 508, 4):
        checksum ^= struct.unpack_from("<I", data, offset)[0]
    if checksum != struct.unpack_from("<I", data, 508)[0]:
        return Observation("fail", "Base block checksum mismatch", "hve")
    if major != 1 or minor > 6:
        return Observation("fail", "Unknown hive format version", "hve")
    if 4096 + bins_size > len(data):
        return Observation("fail", "Hive bins size exceeds file", "hve")
    offset, count = 4096, 0
    while offset < 4096 + bins_size:
        count += 1
        if count > BINS:
            return Observation("inconclusive", "Hive bin budget exceeded", "hve")
        if data[offset : offset + 4] != b"hbin":
            return Observation("fail", f"Hive bin {count} signature missing", "hve")
        relative, size = struct.unpack_from("<II", data, offset + 4)
        if relative != offset - 4096 or not size or size % 4096 or offset + size > 4096 + bins_size:
            return Observation("fail", f"Hive bin {count} offset or size invalid", "hve")
        offset += size
    tags = ("dirty",) if primary != secondary else ()
    if len(data) > offset:
        tags += ("trailing_bytes",)
    return Observation(
        "pass", f"{count} hive bins tiling {bins_size} bytes; checksum verified", "hve", tags
    )
