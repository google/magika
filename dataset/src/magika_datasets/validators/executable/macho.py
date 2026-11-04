# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Mach-O images and fat binaries: load commands, segments and code signature in bounds."""

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("macho",)
SCOPE = "Thin and fat headers, every fat slice inside the file, load commands walked within sizeofcmds, segment file ranges and the code signature blob inside the slice; nothing executed"
MAGICS = {
    0xFEEDFACE: (32, "<"),
    0xFEEDFACF: (64, "<"),
    0xCEFAEDFE: (32, ">"),
    0xCFFAEDFE: (64, ">"),
}
CPUS = {7: "x86", 0x01000007: "x86_64", 12: "arm", 0x0100000C: "arm64"}
FILETYPES = {1: "object", 2: "executable", 6: "dylib", 8: "bundle", 11: "dsym"}
COMMANDS = 4096


class Malformed(Exception):
    pass


def slice_tags(data: bytes, start: int, end: int) -> set[str]:
    magic = struct.unpack_from("<I", data, start)[0]
    if magic not in MAGICS:
        raise Malformed("Unknown Mach-O magic in slice")
    bits, order = MAGICS[magic]
    header_size = 32 if bits == 64 else 28
    if start + header_size > end:
        raise Malformed("Truncated Mach-O header")
    cputype, _, filetype, ncmds, sizeofcmds = struct.unpack_from(order + "iiIII", data, start + 4)
    if not ncmds:
        raise Malformed("No load commands")
    if ncmds > COMMANDS:
        raise Malformed("Load command budget exceeded")
    if start + header_size + sizeofcmds > end:
        raise Malformed("Load commands outside slice")
    tags = {f"macho{bits}", FILETYPES.get(filetype, "other")}
    if cputype in CPUS:
        tags.add(CPUS[cputype])
    offset = start + header_size
    for index in range(ncmds):
        if offset + 8 > start + header_size + sizeofcmds:
            raise Malformed(f"Load command {index} outside sizeofcmds")
        cmd, cmdsize = struct.unpack_from(order + "II", data, offset)
        if cmdsize < 8 or offset + cmdsize > start + header_size + sizeofcmds:
            raise Malformed(f"Load command {index} size invalid")
        if cmd in (0x1, 0x19):  # LC_SEGMENT, LC_SEGMENT_64
            fmt = order + ("QQQQ" if cmd == 0x19 else "IIII")
            _, _, fileoff, filesize = struct.unpack_from(fmt, data, offset + 24)
            if start + fileoff + filesize > end:
                raise Malformed("Segment file range outside slice")
        elif cmd == 0x1D:  # LC_CODE_SIGNATURE
            dataoff, datasize = struct.unpack_from(order + "II", data, offset + 8)
            if start + dataoff + datasize > end:
                raise Malformed("Code signature outside slice")
            tags.add("signed")
        offset += cmdsize
    return tags


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 28:
        return None
    magic = struct.unpack_from("<I", data)[0]
    if magic == 0xBEBAFECA:  # CAFEBABE big-endian, shared with Java class files
        count = struct.unpack_from(">I", data, 4)[0]
        if not 1 <= count <= 64 or 8 + 20 * count > len(data):
            return None
        slices = [struct.unpack_from(">iiIII", data, 8 + 20 * index) for index in range(count)]
        if any(cputype not in CPUS for cputype, *_ in slices):
            return None  # a Java class carries its version where cputype would be
        tags = {"fat"}
        try:
            for _, _, offset, size, _ in slices:
                if offset + size > len(data):
                    raise Malformed("Fat slice outside file")
                tags |= slice_tags(data, offset, offset + size)
        except (Malformed, struct.error) as error:
            return Observation(
                "fail",
                str(error) if isinstance(error, Malformed) else "Truncated structure",
                "macho",
            )
        return Observation(
            "pass", f"{count} fat slices with load commands bounded", "macho", tuple(sorted(tags))
        )
    if magic not in MAGICS:
        return None
    try:
        tags = slice_tags(data, 0, len(data))
    except (Malformed, struct.error) as error:
        return Observation(
            "fail", str(error) if isinstance(error, Malformed) else "Truncated structure", "macho"
        )
    return Observation(
        "pass", "Load commands, segments and signature bounded", "macho", tuple(sorted(tags))
    )
