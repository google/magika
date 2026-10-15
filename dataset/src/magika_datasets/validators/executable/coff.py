# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""COFF object files (including bigobj): section, relocation and symbol tables in bounds."""

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("coff",)
SCOPE = "Known machine, section headers with raw data, relocations and line numbers inside the file, symbol and string tables inside the file; a two-byte machine word is a weak prefix, so passes relabel only hinted samples and failures are reported only for hinted samples"
PREFIX_ONLY = True
CONTEXT_REQUIRED = True
MACHINES = {
    0x14C,
    0x8664,
    0x1C0,
    0x1C4,
    0xAA64,
    0x162,
    0x166,
    0x168,
    0x184,
    0x1A2,
    0x1C2,
    0x1F0,
    0x200,
    0x5032,
    0x5064,
    0xEBC,
    0x14D,
    0x9041,
    0x266,
}
BIGOBJ = b"\0\0\xff\xff"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 20:
        return None
    if data.startswith(BIGOBJ) and data[4:6] in (b"\x01\0", b"\x02\0"):
        machine, sections, symtab, symbols = (
            struct.unpack_from("<H", data, 6)[0],
            *struct.unpack_from("<III", data, 44),
        )
        header, symbol_size, tags = 56, 20, ("bigobj",)
    else:
        machine, sections, _, symtab, symbols, optional, _ = struct.unpack_from("<HHIIIHH", data)
        header, symbol_size, tags = 20 + optional, 18, ()
        if machine not in MACHINES or not sections or optional not in (0, 0xE0, 0xF0, 0x1C, 0x2C):
            return None  # object files may carry thousands of COMDAT sections
    if machine not in MACHINES or not 1 <= sections <= 65279:
        return None
    if header + 40 * sections > len(data):
        return Observation("fail", "Section table outside file", "coff")
    for index in range(sections):
        _, _, _, size, pointer, relocations, lines, nrelocs, nlines, flags = struct.unpack_from(
            "<8sIIIIIIHHI", data, header + 40 * index
        )
        if (
            size and not flags & 0x80 and pointer + size > len(data)
        ):  # uninitialized data has no bytes
            return Observation("fail", f"Section {index} raw data outside file", "coff")
        if nrelocs and relocations + 10 * nrelocs > len(data):
            return Observation("fail", f"Section {index} relocations outside file", "coff")
        if nlines and lines + 6 * nlines > len(data):
            return Observation("fail", f"Section {index} line numbers outside file", "coff")
    if symbols:
        if symtab + symbol_size * symbols + 4 > len(data):
            return Observation("fail", "Symbol table outside file", "coff")
        strings = (
            struct.unpack_from("<I", data, symtab + symbol_size * symbols)[0] or 4
        )  # some tools write 0 for empty
        if strings < 4 or symtab + symbol_size * symbols + strings > len(data):
            return Observation("fail", "String table outside file", "coff")
    return Observation(
        "pass",
        f"{sections} sections and {symbols} symbols bounded; machine {machine:#x}",
        "coff",
        tags,
    )
