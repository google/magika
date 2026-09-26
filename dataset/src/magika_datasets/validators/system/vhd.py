# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Virtual PC / Hyper-V VHD images: footer checksum, and for sparse disks the header and BAT."""

import struct

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("vhd",)
SCOPE = "conectix footer (512 bytes, or the legacy 511) at the end with version 1.0, disk type 2, 3 or 4 and its one's-complement checksum; fixed disks sized current size plus footer; dynamic and differencing disks with the footer copy at offset 0, the cxsparse header at its data offset with its checksum, and every allocated BAT entry's sector bitmap and block inside the file; VHDX and sector contents not interpreted"
COOKIE = b"conectix"
SPARSE = b"cxsparse"
FIXED, DYNAMIC, DIFFERENCING = 2, 3, 4
UNALLOCATED = 0xFFFFFFFF
SECTOR = 512


def checksum(record: bytes, at: int) -> bool:
    stored = struct.unpack_from(">I", record, at)[0]
    total = sum(record[:at]) + sum(record[at + 4 :])
    return (~total & 0xFFFFFFFF) == stored


def footer_at_end(data: bytes) -> tuple[bytes, int] | None:
    """(footer padded to 512 bytes, bytes it occupies) when the image ends with one."""
    for size in (512, 511):  # Virtual PC before 2004 wrote a 511-byte footer
        if len(data) >= size and data[-size:].startswith(COOKIE):
            return data[-size:].ljust(512, b"\0"), size
    return None


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    head = data.startswith(COOKIE)
    found = footer_at_end(data)
    if not head and found is None:
        return None
    if found is None:
        return Observation("fail", "No conectix footer at the end of the image", "vhd")
    footer, trailer = found
    version, offset = struct.unpack_from(">IQ", footer, 12)
    current, kind = struct.unpack_from(">Q", footer, 48)[0], struct.unpack_from(">I", footer, 60)[0]
    if version != 0x00010000 or kind not in (FIXED, DYNAMIC, DIFFERENCING):
        return Observation("fail", "Footer version or disk type unexpected", "vhd")
    if not checksum(footer, 64):
        return Observation("fail", "Footer checksum mismatch", "vhd")
    if kind == FIXED:
        if len(data) - trailer != current:
            return Observation(
                "fail", "Fixed disk size differs from the footer's current size", "vhd"
            )
        return Observation(
            "pass", f"Fixed disk of {current} bytes; footer checksum verified", "vhd"
        )
    if data[:trailer] != data[-trailer:]:
        return Observation("fail", "Footer copy at offset 0 differs from the footer", "vhd")
    if offset + 1024 > len(data) or data[offset : offset + 8] != SPARSE:
        return Observation("fail", "Sparse header missing at the footer's data offset", "vhd")
    header = data[offset : offset + 1024]
    if not checksum(header, 36):
        return Observation("fail", "Sparse header checksum mismatch", "vhd")
    table, header_version, entries, block = struct.unpack_from(">QIII", header, 16)
    if header_version != 0x00010000 or block == 0 or block % SECTOR:
        return Observation("fail", "Sparse header version or block size invalid", "vhd")
    if table + entries * 4 > len(data):
        return Observation("fail", "Block allocation table outside the image", "vhd")
    bitmap = -(-(block // SECTOR) // (8 * SECTOR)) * SECTOR  # one bit per sector, padded
    allocated = 0
    for index in range(entries):
        (sector,) = struct.unpack_from(">I", data, table + 4 * index)
        if sector == UNALLOCATED:
            continue
        if sector * SECTOR + bitmap + block > len(data) - trailer:
            return Observation("fail", f"Block {index} lies outside the image", "vhd")
        allocated += 1
    name = "Dynamic" if kind == DYNAMIC else "Differencing"
    return Observation(
        "pass",
        f"{name} disk; footer and sparse header checksums verified, {allocated} of {entries} blocks allocated inside the image",
        "vhd",
    )
