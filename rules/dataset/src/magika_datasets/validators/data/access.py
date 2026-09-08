# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Microsoft Access (Jet/ACE) databases: header page, page size and page type lattice."""

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("access",)
SCOPE = "Header page signature and engine string, page size by engine version, file size a whole number of pages, every page's type byte in the Jet page-type set; RC4-obfuscated header fields and page contents not decoded"
MAGIC = b"\x00\x01\x00\x00Standard "
PAGE_TYPES = {0, 1, 2, 3, 4, 5, 8, 9}  # 8 and 9 appear in every corpus engine version
PAGES = 1 << 20


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC) or len(data) < 24:
        return None
    engine = data[13:20]
    if engine == b"Jet DB\0":
        version = data[20]
        page_size, name = (2048, "Jet 3") if version == 0 else (4096, "Jet 4")
    elif engine == b"ACE DB\0":
        page_size, name = 4096, "ACE"
    else:
        return Observation("fail", "Unknown database engine", "access")
    if len(data) % page_size:
        return Observation(
            "fail", f"File is not a whole number of {page_size}-byte pages", "access"
        )
    count = len(data) // page_size
    if count > PAGES:
        return Observation("inconclusive", "Page budget exceeded", "access")
    for index in range(1, count):
        kind = data[index * page_size]
        if kind not in PAGE_TYPES:
            return Observation("fail", f"Page {index} has unknown type {kind}", "access")
    return Observation(
        "pass",
        f"{name} database, {count} pages; header, page size and page types verified",
        "access",
    )
