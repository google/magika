# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""USD scene descriptions: ASCII usda brace balance; binary usdc left inconclusive."""

import struct

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("usd",)
SCOPE = "#usda 1.0 header with braces and parentheses balanced outside strings and comments to EOF; PXR-USDC crate files: version, table of contents at its offset with every section inside the file and the TOC ending at EOF"


def crate(data: bytes) -> Observation:
    """USD crate: header, TOC offset, sections (16-byte name, start, size) inside the file."""
    if len(data) < 24:
        return Observation("fail", "Crate header truncated", "usd")
    major, minor = data[8], data[9]
    toc = struct.unpack_from("<Q", data, 16)[0]
    if major != 0 or toc + 8 > len(data):
        return Observation("fail", "Crate version unsupported or TOC offset outside file", "usd")
    count = struct.unpack_from("<Q", data, toc)[0]
    if count > 64 or toc + 8 + 32 * count > len(data):
        return Observation("fail", "Section table outside file", "usd")
    names = []
    for index in range(count):
        entry = toc + 8 + 32 * index
        name = data[entry : entry + 16].split(b"\0", 1)[0]
        start, size = struct.unpack_from("<qq", data, entry + 16)
        if start < 24 or size < 0 or start + size > toc:
            return Observation("fail", f"Section {name!r} outside the file body", "usd")
        names.append(name.decode("latin-1"))
    if toc + 8 + 32 * count != len(data):
        return Observation("fail", "Bytes after the table of contents", "usd")
    if not {"TOKENS", "PATHS", "SPECS"} <= set(names):
        return Observation("fail", "Crate lacks the TOKENS, PATHS or SPECS sections", "usd")
    return Observation(
        "pass",
        f"USD crate 0.{minor}.{data[10]} with {count} sections inside the body and TOC at EOF",
        "usd",
        ("crate",),
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data.startswith(b"PXR-USDC"):
        return crate(data)
    if not data.startswith(b"#usda "):
        return None
    depth, parens, offset, quote = 0, 0, 0, None
    while offset < len(data):
        byte = data[offset : offset + 1]
        if quote:
            if byte == b"\\":
                offset += 1
            elif byte == quote:
                quote = None
        elif byte == b"#":
            end = data.find(b"\n", offset)
            offset = len(data) if end < 0 else end
        elif byte in (b'"', b"'"):
            quote = byte
        elif byte == b"{":
            depth += 1
        elif byte == b"}":
            depth -= 1
        elif byte == b"(":
            parens += 1
        elif byte == b")":
            parens -= 1
        if depth < 0 or parens < 0:
            return Observation("fail", "Unbalanced closing bracket", "usd")
        offset += 1
    if quote or depth or parens:
        return Observation("fail", "Unbalanced strings or brackets at EOF", "usd")
    return Observation("pass", "usda brackets balanced to EOF", "usd", ("ascii",))
