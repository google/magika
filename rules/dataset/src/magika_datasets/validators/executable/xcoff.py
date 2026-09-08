# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""XCOFF (AIX) objects: 32- and 64-bit headers, sections and symbol tables in bounds."""

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("xcoff",)
SCOPE = "XCOFF32 or XCOFF64 magic, section headers with raw data and relocations inside the file, symbol and string tables inside the file; a two-byte magic is a weak prefix, so failures are reported only for hinted samples"
PREFIX_ONLY = True


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:2] == b"\x01\xdf":
        bits = 32
    elif data[:2] == b"\x01\xf7":
        bits = 64
    else:
        return None
    if len(data) < 24:
        return Observation("fail", "Truncated header", "xcoff")
    if bits == 32:
        _, sections, _, symtab, symbols, optional, _ = struct.unpack_from(">HHIIIHH", data)
        header, section_size = 20 + optional, 40
    else:
        _, sections, _, symtab, optional, _, symbols = struct.unpack_from(">HHIQHHI", data)
        header, section_size = 24 + optional, 72
    if not sections or sections > 65535 or header + section_size * sections > len(data):
        return Observation("fail", "Section table outside file", "xcoff")
    for index in range(sections):
        base = header + section_size * index
        if bits == 32:
            _, _, _, size, pointer, relocations, _, nrelocs, _, flags = struct.unpack_from(
                ">8sIIIIIIHHI", data, base
            )
        else:
            _, _, _, size, pointer, relocations, _, nrelocs, _, flags = struct.unpack_from(
                ">8sQQQQQQIII", data, base
            )
        if size and pointer and flags & 0x80 == 0 and pointer + size > len(data):
            return Observation("fail", f"Section {index} raw data outside file", "xcoff")
        if nrelocs and relocations + (10 if bits == 32 else 14) * nrelocs > len(data):
            return Observation("fail", f"Section {index} relocations outside file", "xcoff")
    if symbols:
        if symtab + 18 * symbols + 4 > len(data):
            return Observation("fail", "Symbol table outside file", "xcoff")
        strings = struct.unpack_from(">I", data, symtab + 18 * symbols)[0]
        if strings < 4 or symtab + 18 * symbols + strings > len(data):
            return Observation("fail", "String table outside file", "xcoff")
    return Observation(
        "pass",
        f"XCOFF{bits}: {sections} sections and {symbols} symbols bounded",
        "xcoff",
        (f"xcoff{bits}",),
    )
