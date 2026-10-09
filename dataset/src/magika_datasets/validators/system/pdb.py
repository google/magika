# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Program databases: MSF 7.0 superblock and directory map, or MSF 2.0 page header."""

import struct

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("pdb",)
SCOPE = "Portable PDB metadata root (version string, stream headers and ranges inside the file, #Pdb stream present), MSF 7.0 superblock (block size, block count equal to the file, block map address and directory block indices inside the file) or MSF 2.0 header (page size and page count equal to the file); streams not decoded"
MSF7 = b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0"
MSF2 = b"Microsoft C/C++ program database 2.00\r\n\x1aJG\0\0"


def portable(data: bytes) -> Observation:
    """ECMA-335 metadata root as used by .NET portable PDBs."""
    if len(data) < 20:
        return Observation("fail", "Truncated metadata root", "pdb")
    length = struct.unpack_from("<I", data, 12)[0]
    offset = 16 + length
    if length > 256 or offset + 4 > len(data):
        return Observation("fail", "Version string outside file", "pdb")
    streams = struct.unpack_from("<H", data, offset + 2)[0]
    offset += 4
    names = []
    for index in range(streams):
        if offset + 8 > len(data):
            return Observation("fail", f"Stream header {index} outside file", "pdb")
        start, size = struct.unpack_from("<II", data, offset)
        end = data.find(b"\0", offset + 8)
        if end < 0 or end - offset - 8 > 32:
            return Observation("fail", f"Stream {index} name unterminated", "pdb")
        if start + size > len(data):
            return Observation("fail", f"Stream {index} outside file", "pdb")
        names.append(data[offset + 8 : end].decode("ascii", "replace"))
        offset = end + 1 + (-(end + 1 - 0) % 4 if (end + 1) % 4 else 0)
    if "#Pdb" not in names:
        return Observation("fail", "Metadata root without a #Pdb stream", "pdb")
    return Observation(
        "pass", f"Portable PDB: {streams} metadata streams bounded", "pdb", ("portable_pdb",)
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data.startswith(b"BSJB"):
        return portable(data)
    if data.startswith(MSF2):
        if len(data) < 60:
            return Observation("fail", "Truncated MSF 2.0 header", "pdb")
        page_size, _, pages, _, _ = struct.unpack_from("<IHHII", data, 44)
        if page_size not in (1024, 2048, 4096) or pages * page_size != len(data):
            return Observation("fail", "MSF 2.0 page count differs from file", "pdb")
        return Observation(
            "pass", f"MSF 2.0: {pages} pages of {page_size} bytes", "pdb", ("msf_2",)
        )
    if not data.startswith(MSF7):
        return None
    if len(data) < 56:
        return Observation("fail", "Truncated superblock", "pdb")
    block_size, _, blocks, directory_bytes, _, map_block = struct.unpack_from("<IIIIII", data, 32)
    if block_size not in (512, 1024, 2048, 4096):
        return Observation("fail", "Unsupported block size", "pdb")
    if blocks * block_size != len(data):
        return Observation("fail", "Block count differs from file size", "pdb")
    if map_block >= blocks:
        return Observation("fail", "Block map address outside file", "pdb")
    entries = (directory_bytes + block_size - 1) // block_size
    if entries * 4 > block_size:
        return Observation("inconclusive", "Directory map spans several blocks; not walked", "pdb")
    for index in range(entries):
        block = struct.unpack_from("<I", data, map_block * block_size + 4 * index)[0]
        if block >= blocks:
            return Observation("fail", "Directory block index outside file", "pdb")
    return Observation(
        "pass", f"MSF 7.0: {blocks} blocks of {block_size} bytes; directory map bounded", "pdb"
    )
