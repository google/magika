# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""MySQL storage files: MyISAM index headers and InnoDB tablespace page lattices."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("mysql_storage",)
SCOPE = "MyISAM .MYI: file version, header/state/base lengths, key_file_length against the file; InnoDB .ibd: FSP header on page 0, every page's number and space id, file a whole number of pages; page checksums and key blocks not verified"
MYI = b"\xfe\xfe\x07"
FSP_HDR = 8
PAGES = 1 << 20


def myisam(data: bytes) -> Observation:
    if len(data) < 64:
        return Observation("fail", "Truncated MyISAM header", "mysql_storage")
    header_length, state_length, base_length, base_pos, key_parts, unique_parts = (
        struct.unpack_from(">HHHHHH", data, 6)
    )
    keys = data[18]
    if header_length < 24 + state_length or base_pos > header_length or keys > 64:
        return Observation("fail", "MyISAM header lengths inconsistent", "mysql_storage")
    key_file_length = struct.unpack_from(">Q", data, 24 + 4 + 32)[0]
    if key_file_length != len(data):
        return Observation(
            "fail", f"key_file_length {key_file_length} differs from the file size", "mysql_storage"
        )
    return Observation(
        "pass",
        f"MyISAM index with {keys} keys; header lengths and key_file_length verified",
        "mysql_storage",
        ("myisam_index",),
    )


def innodb(data: bytes) -> Observation | None:
    if len(data) < 16384 or struct.unpack_from(">H", data, 24)[0] != FSP_HDR:
        return None
    space = struct.unpack_from(">I", data, 34)[0]
    flags = struct.unpack_from(">I", data, 38 + 16)[0]
    shift = (flags >> 6) & 0xF
    page_size = 1 << (9 + shift) if shift else 16384
    if len(data) % page_size:
        return Observation(
            "fail", f"File is not a whole number of {page_size}-byte pages", "mysql_storage"
        )
    count = len(data) // page_size
    if count > PAGES:
        return Observation("inconclusive", "Page budget exceeded", "mysql_storage")
    for index in range(count):
        page = data[index * page_size : (index + 1) * page_size]
        if not any(page):
            continue  # unallocated pages are all zero
        number, page_space = (
            struct.unpack_from(">I", page, 4)[0],
            struct.unpack_from(">I", page, 34)[0],
        )
        if number != index or page_space != space:
            return Observation(
                "fail",
                f"Page {index} carries number {number} and space {page_space}",
                "mysql_storage",
            )
    return Observation(
        "pass",
        f"InnoDB tablespace {space} with {count} pages of {page_size} bytes; page numbering verified",
        "mysql_storage",
        ("innodb_tablespace",),
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data.startswith(MYI):
        return myisam(data)
    return innodb(data)
