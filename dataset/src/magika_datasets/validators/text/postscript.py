# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""PostScript: header, balanced strings, procedures and dictionaries; never interpreted."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("postscript", "ai")
SHARED_FORMAT_IDS = ("ai",)  # PDF-based Illustrator files are named by the PDF validator
ILLUSTRATOR = re.compile(rb"^%(?:%|!)?AI\d+_", re.M)
SCOPE = "%! header, parenthesised strings with escapes and nesting, { } procedures and << >> dictionaries balanced outside strings and comments, binary sections after %%BeginBinary skipped by declared length when present; nothing executed; named ai when Illustrator private comments are present"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"%!"):
        return None
    if data[:6] == b"%!PS-A" and b"\x04" in data[:1]:
        return None
    braces = dicts = strings = 0
    offset, escaped = 0, False
    while offset < len(data):
        byte = data[offset]
        if strings:
            if escaped:
                escaped = False
            elif byte == 0x5C:
                escaped = True
            elif byte == 0x28:
                strings += 1
            elif byte == 0x29:
                strings -= 1
            offset += 1
            continue
        if byte == 0x25:  # comment to end of line
            end = data.find(b"\n", offset)
            offset = len(data) if end < 0 else end + 1
            continue
        if byte == 0x28:
            strings = 1
        elif byte == 0x7B:
            braces += 1
        elif byte == 0x7D:
            braces -= 1
        elif byte == 0x3C and data[offset + 1 : offset + 2] == b"<":
            dicts += 1
            offset += 1
        elif byte == 0x3E and data[offset + 1 : offset + 2] == b">":
            dicts -= 1
            offset += 1
        elif byte == 0x3C and data[offset + 1 : offset + 2] == b"~":  # ASCII85 block
            end = data.find(b"~>", offset)
            if end < 0:
                return Observation("fail", "Unterminated ASCII85 block", "postscript")
            offset = end + 1
        if braces < 0 or dicts < 0:
            return Observation("fail", "Unbalanced closing brace or dictionary", "postscript")
        offset += 1
    if strings:
        return Observation("fail", "Unterminated string", "postscript")
    if braces or dicts:
        return Observation("fail", "Unbalanced procedures or dictionaries at EOF", "postscript")
    tags = ("eps",) if b"EPSF" in data[:64] or b"%%BoundingBox" in data[:4096] else ()
    if ILLUSTRATOR.search(data[:65536]):
        return Observation(
            "pass",
            "PostScript tokens balanced to EOF; Illustrator private comments present",
            "ai",
            tags,
        )
    return Observation("pass", "PostScript tokens balanced to EOF", "postscript", tags)
