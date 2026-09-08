# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Portable Executable images: headers, section table, directories, overlay and checksum."""

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("pe",)
SCOPE = "DOS and PE headers, optional header magic, section raw and virtual ranges inside file and image without overlap, data directories inside sections, Authenticode directory covering the overlay, optional header checksum when nonzero; code never executed"
MACHINES = {
    0x14C: "machine_x86",
    0x8664: "machine_x64",
    0xAA64: "machine_arm64",
    0x1C0: "machine_arm",
    0x1C4: "machine_arm",
}


def checksum(data: bytes) -> int:
    """PE checksum: 16-bit ones-complement sum over the file with the CheckSum field zeroed."""
    e_lfanew = struct.unpack_from("<I", data, 0x3C)[0]
    field = e_lfanew + 24 + 64
    total = 0
    padded = data + (b"\0" if len(data) % 2 else b"")
    words = struct.unpack(f"<{len(padded) // 2}H", padded)
    for index, word in enumerate(words):
        if index * 2 in (field, field + 2):
            continue
        total += word
        total = (total & 0xFFFF) + (total >> 16)
    total = (total & 0xFFFF) + (total >> 16)
    return (total & 0xFFFF) + len(data)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:2] != b"MZ" or len(data) < 0x40:
        return None
    e_lfanew = struct.unpack_from("<I", data, 0x3C)[0]
    if e_lfanew + 24 > len(data) or e_lfanew % 4 or data[e_lfanew : e_lfanew + 4] != b"PE\0\0":
        return None  # a DOS-only MZ program or unrelated bytes, not a broken PE
    machine, sections, _, _, _, optional_size, characteristics = struct.unpack_from(
        "<HHIIIHH", data, e_lfanew + 4
    )
    optional = e_lfanew + 24
    if sections > 96 or optional_size < 96 or optional + optional_size > len(data):
        return Observation("fail", "COFF header fields out of range", "pe")
    magic = struct.unpack_from("<H", data, optional)[0]
    if magic not in (0x10B, 0x20B):
        return Observation("fail", "Unknown optional header magic", "pe")
    plus = magic == 0x20B
    size_of_image, size_of_headers, stored_checksum, subsystem = struct.unpack_from(
        "<IIIH", data, optional + 56
    )
    directory_count = struct.unpack_from("<I", data, optional + (108 if plus else 92))[0]
    directories_at = optional + (112 if plus else 96)
    table = optional + optional_size
    if (
        directory_count > 16
        or directories_at + 8 * directory_count > table
        or table + 40 * sections > len(data)
    ):
        return Observation("fail", "Data directories or section table outside headers", "pe")
    if size_of_headers < table + 40 * sections or size_of_headers > len(data):
        return Observation("fail", "SizeOfHeaders inconsistent with section table", "pe")
    directories = [
        struct.unpack_from("<II", data, directories_at + 8 * index)
        for index in range(directory_count)
    ]
    raw_ranges, virtual_ranges = [], []
    for index in range(sections):
        _, virtual_size, virtual_address, raw_size, raw_pointer = struct.unpack_from(
            "<8sIIII", data, table + 40 * index
        )
        if raw_size:
            if raw_pointer < size_of_headers or raw_pointer + raw_size > len(data):
                return Observation("fail", f"Section {index} raw data outside file", "pe")
            raw_ranges.append((raw_pointer, raw_pointer + raw_size))
        if virtual_address + max(virtual_size, raw_size) > size_of_image:
            return Observation("fail", f"Section {index} virtual range outside image", "pe")
        virtual_ranges.append((virtual_address, virtual_address + max(virtual_size, raw_size, 1)))
    raw_ranges.sort()
    for (_, end), (start, _) in zip(raw_ranges, raw_ranges[1:]):
        if start < end:
            return Observation("fail", "Section raw data overlaps", "pe")
    for index, (rva, size) in enumerate(directories):
        if not size or index == 4:
            continue
        if not any(start <= rva < end for start, end in virtual_ranges) and rva >= size_of_headers:
            return Observation("fail", f"Data directory {index} outside every section", "pe")
    content_end = max([end for _, end in raw_ranges] + [size_of_headers])
    tags = set()
    if len(directories) > 4 and directories[4][1]:
        offset, size = directories[4]
        if offset < content_end or offset + size > len(data):
            return Observation("fail", "Security directory outside the overlay", "pe")
        tags.add("signed")
        if offset > content_end or offset + size < len(data):
            tags.add("overlay")  # installers commonly append payload after the signature
    elif len(data) > content_end:
        tags.add("overlay")
    if stored_checksum and checksum(data) != stored_checksum:
        tags.add("checksum_mismatch")
    tags.add("pe32plus" if plus else "pe32")
    if machine in MACHINES:
        tags.add(MACHINES[machine])
    if characteristics & 0x2000:
        tags.add("dll")
    elif subsystem == 1:
        tags.add("driver")
    else:
        tags.add("executable")
    if len(directories) > 14 and directories[14][1]:
        tags.add("dotnet")
    return Observation(
        "pass",
        f"{sections} sections, {sum(1 for _, s in directories if s)} directories bounded",
        "pe",
        tuple(sorted(tags)),
    )
