# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Portable Executable images: headers, section table, directories, overlay and checksum."""
# MUI resource files are PE images too; they are named by their MUI resource below.

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("pe", "mui")
SCOPE = "DOS and PE headers, optional header magic, section raw and virtual ranges inside file and image without overlap, data directories inside sections, Authenticode directory covering the overlay, optional header checksum when nonzero; named mui when the resource directory holds an MUI resource whose configuration (signature 0xFECDFECD) marks a language resource file (type 0x12 or 0x22); code never executed"
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


MUI_SIGNATURE = 0xFECDFECD
MUI_FILE_TYPES = {0x12, 0x22}  # language-specific resource files; 0x11 is the neutral binary
RESOURCE_ENTRIES = 4096


def mui_file_type(data: bytes, sections, directory) -> int | None:
    """dwFileType from the image's MUI resource configuration, or None when it has none."""
    rva, size = directory
    if not size:
        return None

    def offset(address):
        for virtual, raw_pointer, raw_size in sections:
            if virtual <= address < virtual + raw_size:
                return raw_pointer + address - virtual
        return None

    base = offset(rva)
    if base is None:
        return None

    def entries(at):
        if at + 16 > len(data):
            return []
        named, ids = struct.unpack_from("<HH", data, at + 12)
        if named + ids > RESOURCE_ENTRIES or at + 16 + 8 * (named + ids) > len(data):
            return []
        return [struct.unpack_from("<II", data, at + 16 + 8 * i) for i in range(named + ids)]

    for name, target in entries(base):
        if not name & 0x80000000 or not target & 0x80000000:
            continue
        label = base + (name & 0x7FFFFFFF)
        if label + 2 > len(data):
            continue
        (length,) = struct.unpack_from("<H", data, label)
        if data[label + 2 : label + 2 + 2 * length] != "MUI".encode("utf-16-le"):
            continue
        level = base + (target & 0x7FFFFFFF)
        for _ in range(2):  # resource name, then language
            found = entries(level)
            if not found or not found[0][1] & 0x80000000:
                break
            level = base + (found[0][1] & 0x7FFFFFFF)
        leaf = entries(level)
        if not leaf or leaf[0][1] & 0x80000000:
            return None
        entry = base + leaf[0][1]
        if entry + 8 > len(data):
            return None
        data_rva, data_size = struct.unpack_from("<II", data, entry)
        start = offset(data_rva)
        if start is None or data_size < 20 or start + 20 > len(data):
            return None
        signature, _, _, _, file_type = struct.unpack_from("<5I", data, start)
        return file_type if signature == MUI_SIGNATURE else None
    return None


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
    raw_ranges, virtual_ranges, mapped = [], [], []
    for index in range(sections):
        _, virtual_size, virtual_address, raw_size, raw_pointer = struct.unpack_from(
            "<8sIIII", data, table + 40 * index
        )
        if raw_size:
            if raw_pointer < size_of_headers or raw_pointer + raw_size > len(data):
                return Observation("fail", f"Section {index} raw data outside file", "pe")
            raw_ranges.append((raw_pointer, raw_pointer + raw_size))
            mapped.append((virtual_address, raw_pointer, raw_size))
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
    detail = f"{sections} sections, {sum(1 for _, s in directories if s)} directories bounded"
    file_type = mui_file_type(data, mapped, directories[2]) if len(directories) > 2 else None
    if file_type in MUI_FILE_TYPES:
        return Observation(
            "pass", f"{detail}; MUI resource file type {file_type:#x}", "mui", tuple(sorted(tags))
        )
    return Observation("pass", detail, "pe", tuple(sorted(tags)))
