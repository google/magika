# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Unix ar archives and Debian packages: 60-byte member headers tiling the file."""

import re

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("ar", "deb")
SCOPE = "Global magic, member headers (decimal size, `\\x60\\n` terminator), even padding and members tiling the file; deb requires a leading debian-binary member holding 2.0 and control/data tarballs; member contents not interpreted"
MAGIC = b"!<arch>\n"
HEADER = 60
MEMBERS = 65536
NAME = re.compile(rb"[\x20-\x7e]{16}")


def members(data: bytes) -> list[tuple[bytes, int, int]]:
    """(name, offset, size) for every member; raises ValueError on malformed headers."""
    found, offset = [], len(MAGIC)
    while offset < len(data):
        if offset + HEADER > len(data):
            raise ValueError("Truncated member header")
        header = data[offset : offset + HEADER]
        if header[58:] != b"`\n" or not NAME.fullmatch(header[:16]):
            raise ValueError("Malformed member header")
        try:
            size = int(header[48:58].strip() or b"x")
        except ValueError:
            raise ValueError("Non-decimal member size") from None
        start = offset + HEADER
        name = header[:16].rstrip()
        if name.startswith(b"#1/"):  # BSD long name stored ahead of the data
            length = int(name[3:])
            if length > size:
                raise ValueError("BSD long name exceeds member size")
            name, start, size = (
                data[start : start + length].rstrip(b"\0"),
                start + length,
                size - length,
            )
        if start + size > len(data):
            raise ValueError("Member data outside file")
        found.append((name, start, size))
        if len(found) > MEMBERS:
            raise ValueError("Member budget exceeded")
        offset = start + size + (size & 1)
        if offset == len(data) + 1:  # odd final member without its pad byte
            offset -= 1
    return found


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    try:
        found = members(data)
    except ValueError as error:
        return Observation("fail", str(error), "deb" if "deb" in hints else "ar")
    names = [name for name, _, _ in found]
    tags = []
    if b"//" in names:
        tags.append("gnu_long_names")
    if any(name in (b"/", b"__.SYMDEF", b"__.SYMDEF SORTED") for name in names):
        tags.append("symbol_index")
    if names and names[0].rstrip(b"/") == b"debian-binary":
        _, start, size = found[0]
        body = data[start : start + size]
        controls = [n for n in names if n.rstrip(b"/").startswith(b"control.tar")]
        payloads = [n for n in names if n.rstrip(b"/").startswith(b"data.tar")]
        if not body.strip().startswith(b"2.0"):
            return Observation("fail", "debian-binary member does not declare version 2.0", "deb")
        if not controls or not payloads:
            return Observation("fail", "Missing control.tar or data.tar member", "deb")
        return Observation(
            "pass",
            f"{len(found)} members; debian-binary 2.0, {controls[0].decode('latin-1')} and {payloads[0].decode('latin-1')} present",
            "deb",
            tuple(tags),
        )
    return Observation(
        "pass",
        f"{len(found)} members; headers and data bounds tile the file",
        "ar",
        tuple(tags),
        generic=True,  # deb, .a and .lib all share the container
    )
