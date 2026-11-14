# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""HFS and HFS+ volume images: volume header, its alternate copy and the catalog B-tree."""

import struct

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("hfs",)
SCOPE = "HFS+ (H+ v4) and HFSX (HX v5) volume header at 1024 with a power-of-two block size, free blocks within total blocks, the volume sized to the image, the alternate header 1024 bytes before the end identical in signature and geometry, and the allocation, extents and catalog special files' extents inside the volume with the catalog's first node a B-tree header node; classic HFS master directory block (BD) with allocation block geometry inside the image; file contents not read"
PREFIX_ONLY = True  # two bytes at 1024 also occur in code and text; failures need a hint
VOLUME = 1024
FORK = struct.Struct(">QII8Q")  # logical size, clump, total blocks, 8 extents (start, count)
BTREE_HEADER = 1


def forks(header: bytes):
    """(name, extents) for the allocation, extents and catalog files."""
    for name, at in (("allocation", 112), ("extents", 192), ("catalog", 272)):
        fields = FORK.unpack_from(header, at)
        yield name, [(fields[3 + i] >> 32, fields[3 + i] & 0xFFFFFFFF) for i in range(8)]


def plus(data: bytes, header: bytes) -> Observation:
    signature, version = header[:2], struct.unpack_from(">H", header, 2)[0]
    if (signature, version) not in ((b"H+", 4), (b"HX", 5)):
        return Observation("fail", "Volume signature and version disagree", "hfs")
    block, total, free = struct.unpack_from(">III", header, 40)
    if block < 512 or block & (block - 1) or free > total:
        return Observation("fail", "Block size or block counts invalid", "hfs")
    if total * block > len(data):
        return Observation("fail", "Volume is larger than the image", "hfs")
    alternate = data[total * block - VOLUME : total * block - VOLUME + 512]
    if alternate[:4] != header[:4] or alternate[40:48] != header[40:48]:
        return Observation("fail", "Alternate volume header missing or different", "hfs")
    catalog_start = None
    for name, extents in forks(header):
        for start, count in extents:
            if count and start + count > total:
                return Observation(
                    "fail", f"{name.capitalize()} file extent outside the volume", "hfs"
                )
        if name == "catalog":
            catalog_start = extents[0][0] if extents[0][1] else None
    if catalog_start is None:
        return Observation("fail", "Catalog file has no extents", "hfs")
    node = data[catalog_start * block : catalog_start * block + 14]
    if len(node) < 14 or node[8] != BTREE_HEADER:
        return Observation("fail", "Catalog does not begin with a B-tree header node", "hfs")
    kind = "HFS+" if signature == b"H+" else "HFSX"
    return Observation(
        "pass",
        f"{kind} volume of {total} blocks of {block} bytes; catalog header node found",
        "hfs",
    )


def classic(data: bytes, mdb: bytes) -> Observation:
    blocks, size, _, first = struct.unpack_from(">HIIH", mdb, 18)  # drNmAlBlks..drAlBlSt
    if size < 512 or size % 512 or first < 3:
        return Observation("fail", "Allocation block size or first block invalid", "hfs")
    if first * 512 + blocks * size > len(data):
        return Observation("fail", "Allocation blocks extend past the image", "hfs")
    return Observation("pass", f"HFS volume of {blocks} allocation blocks of {size} bytes", "hfs")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    header = data[VOLUME : VOLUME + 512]
    if len(header) < 512 or header[:2] not in (b"H+", b"HX", b"BD"):
        return None
    if header[:2] == b"BD":
        if "hfs" not in hints:
            return None  # two bytes at 1024 are too weak to speak for an unhinted file
        return classic(data, header)
    return plus(data, header)
