# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ELF objects and executables: identification, program and section tables in bounds."""

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("elf", "cubin")
SCOPE = "e_ident class, data and version, header sizes for the class, every program header's file range and every non-NOBITS section's file range inside the file, section name table terminated; unreferenced trailing bytes tagged; nothing executed"
TYPES = {1: "relocatable", 2: "executable", 3: "shared_object", 4: "core"}
CUDA_OSABI = 0x33


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"\x7fELF" or len(data) < 52:
        return None
    bits, endian, version, osabi = data[4], data[5], data[6], data[7]
    if bits not in (1, 2) or endian not in (1, 2) or version != 1:
        return Observation("fail", "Invalid e_ident class, data or version", "elf")
    order = "<" if endian == 1 else ">"
    if bits == 2:
        if len(data) < 64:
            return Observation("fail", "Truncated ELF64 header", "elf")
        (
            kind,
            machine,
            _,
            _,
            ph_offset,
            sh_offset,
            _,
            ehsize,
            phentsize,
            phnum,
            shentsize,
            shnum,
            shstrndx,
        ) = struct.unpack_from(order + "HHIQQQIHHHHHH", data, 16)
        expected = (64, 56, 64)
    else:
        (
            kind,
            machine,
            _,
            _,
            ph_offset,
            sh_offset,
            _,
            ehsize,
            phentsize,
            phnum,
            shentsize,
            shnum,
            shstrndx,
        ) = struct.unpack_from(order + "HHIIIIIHHHHHH", data, 16)
        expected = (52, 32, 40)
    if kind not in TYPES:
        return Observation("fail", "Unknown e_type", "elf")
    if (
        ehsize != expected[0]
        or (phnum and phentsize != expected[1])
        or (shnum and shentsize != expected[2])
    ):
        return Observation("fail", "Header sizes differ from the ELF class", "elf")
    if phnum > 4096 or shnum > 65535:
        return Observation("inconclusive", "Table budget exceeded", "elf")
    referenced = ehsize
    for index in range(phnum):
        base = ph_offset + index * phentsize
        if base + phentsize > len(data):
            return Observation("fail", "Program header table outside file", "elf")
        if bits == 2:
            _, _, offset, _, _, filesz = struct.unpack_from(order + "IIQQQQ", data, base)
        else:
            _, offset, _, _, filesz = struct.unpack_from(order + "IIIII", data, base)
        if offset + filesz > len(data):
            return Observation("fail", f"Segment {index} file range outside file", "elf")
        referenced = max(referenced, offset + filesz, base + phentsize)
    names = {}
    if shnum:
        if sh_offset + shnum * shentsize > len(data) or shstrndx >= shnum:
            return Observation("fail", "Section header table or name index outside file", "elf")
        referenced = max(referenced, sh_offset + shnum * shentsize)
        headers = []
        for index in range(shnum):
            base = sh_offset + index * shentsize
            if bits == 2:
                name, typ, _, _, offset, size = struct.unpack_from(order + "IIQQQQ", data, base)
            else:
                name, typ, _, _, offset, size = struct.unpack_from(order + "IIIIII", data, base)
            if (
                typ != 8 and index and offset + size > len(data)
            ):  # SHT_NOBITS occupies no file bytes
                return Observation("fail", f"Section {index} file range outside file", "elf")
            if typ != 8:
                referenced = max(referenced, offset + size)
            headers.append((name, typ, offset, size))
        _, _, table_offset, table_size = headers[shstrndx]
        table = data[table_offset : table_offset + table_size]
        if not table.endswith(b"\0"):
            return Observation("fail", "Section name table is not NUL-terminated", "elf")
        for name, typ, _, _ in headers:
            if name >= len(table):
                return Observation("fail", "Section name outside the name table", "elf")
            names[table[name:].split(b"\0", 1)[0]] = typ
    tags = {TYPES[kind], "elf64" if bits == 2 else "elf32"}
    if endian == 2:
        tags.add("big_endian")
    if shnum and b".symtab" not in names:
        tags.add("stripped")
    if referenced < len(data):
        tags.add("trailing_bytes")
    kind_id = "cubin" if osabi == CUDA_OSABI or b".nv.info" in names else "elf"
    return Observation(
        "pass",
        f"{phnum} segments and {shnum} sections bounded; machine {machine}",
        kind_id,
        tuple(sorted(tags)),
    )
