# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Rich Text Format: rtf1 header and balanced groups to the last byte."""

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("rtf",)
SCOPE = "{\\rtf1 header, braces balanced with escaped braces and \\'hh hex escapes honoured, the outer group closing at the last non-whitespace byte; control words not interpreted"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"{\\rtf"):
        return None
    if not data.startswith(b"{\\rtf1"):
        return Observation("fail", "Unsupported RTF version", "rtf")
    depth, offset, closed_at = 0, 0, None
    while offset < len(data):
        byte = data[offset]
        if byte == 0x5C:  # backslash: escaped brace, hex escape or control word
            offset += 2 if data[offset + 1 : offset + 2] != b"'" else 4
            continue
        if byte == 0x7B:
            depth += 1
        elif byte == 0x7D:
            depth -= 1
            if depth < 0:
                return Observation("fail", "Unbalanced closing brace", "rtf")
            if depth == 0:
                closed_at = offset
                break
        offset += 1
    if depth != 0 or closed_at is None:
        return Observation("fail", "Outer group never closes", "rtf")
    if data[closed_at + 1 :].strip(b" \t\r\n\0\x1a"):  # Word appends a NUL after the group
        return Observation("fail", "Content after the outer group", "rtf")
    return Observation("pass", "RTF groups balanced to EOF", "rtf")
