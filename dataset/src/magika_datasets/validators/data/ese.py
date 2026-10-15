# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Extensible Storage Engine databases: header XOR checksum, shadow header and page lattice."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("ese",)
SCOPE = "Header signature, XOR-32 checksum over the header page, identical shadow header, page size, file size a whole number of pages; page contents and ECC checksums not verified"
SIGNATURE = 0x89ABCDEF


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 240 or struct.unpack_from("<I", data, 4)[0] != SIGNATURE:
        return None
    version, kind = struct.unpack_from("<II", data, 8)
    page_size = struct.unpack_from("<I", data, 236)[0]
    if page_size not in (2048, 4096, 8192, 16384, 32768) or len(data) < 2 * page_size:
        return Observation("fail", "Page size invalid or file shorter than two header pages", "ese")
    stored = struct.unpack_from("<I", data, 0)[0]
    computed = SIGNATURE
    for (word,) in struct.iter_unpack("<I", data[4:page_size]):
        computed ^= word
    if computed != stored:
        return Observation("fail", "Header XOR checksum mismatch", "ese")
    if data[:page_size] != data[page_size : 2 * page_size]:
        return Observation(
            "inconclusive", "Shadow header differs from the header (mid-update image)", "ese"
        )
    if len(data) % page_size:
        return Observation("fail", "File is not a whole number of pages", "ese")
    kinds = {0: "database", 1: "stream", 2: "log"}
    return Observation(
        "pass",
        f"ESE {kinds.get(kind, kind)} format 0x{version:x}, {len(data) // page_size} pages of {page_size} bytes; header checksum and shadow verified",
        "ese",
    )
