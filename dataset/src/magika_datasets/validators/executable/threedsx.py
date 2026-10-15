# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Nintendo 3DS homebrew executables: segment sizes, relocation tables, SMDH and RomFS tiling the file."""

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("3dsx",)
SCOPE = "Header sizes, code/rodata/data segment sizes with bss subtracted, per-segment relocation table counts, SMDH offset and size, RomFS offset, all sections tiling the file in order; code not interpreted"
MAGIC = b"3DSX"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 32:
        return Observation("fail", "Truncated header", "3dsx")
    header_size, reloc_size, version, flags, code, rodata, data_size, bss = struct.unpack_from(
        "<HHIIIIII", data, 4
    )
    if header_size not in (32, 44) or reloc_size != 8 or version != 0 or bss > data_size:
        return Observation("fail", "Header sizes or version invalid", "3dsx")
    if header_size + 3 * reloc_size > len(data):
        return Observation("fail", "Relocation headers outside file", "3dsx")
    relocations = 0
    for segment in range(3):
        absolute, relative = struct.unpack_from("<II", data, header_size + segment * reloc_size)
        relocations += absolute + relative
    expected = header_size + 3 * reloc_size + code + rodata + (data_size - bss) + 4 * relocations
    if expected > len(data):
        return Observation("fail", "Segments and relocation tables exceed the file", "3dsx")
    tags = []
    end = expected
    if header_size == 44:
        smdh_offset, smdh_size, romfs_offset = struct.unpack_from("<III", data, 32)
        if smdh_offset:
            if smdh_offset != expected or smdh_offset + smdh_size > len(data):
                return Observation("fail", "SMDH does not follow the relocation tables", "3dsx")
            if data[smdh_offset : smdh_offset + 4] != b"SMDH":
                return Observation("fail", "SMDH magic missing", "3dsx")
            end = smdh_offset + smdh_size
            tags.append("smdh")
        if romfs_offset:
            if romfs_offset != end or struct.unpack_from("<I", data, romfs_offset)[0] != 0x28:
                return Observation(
                    "fail", "RomFS does not follow the SMDH or lacks its level-3 header", "3dsx"
                )
            end = len(data)
            tags.append("romfs")
    if end != len(data):
        return Observation("fail", f"{len(data) - end} bytes after the last section", "3dsx")
    return Observation(
        "pass",
        f"code {code}, rodata {rodata}, data {data_size - bss} bytes and {relocations} relocations tile the file",
        "3dsx",
        tuple(tags),
    )
