# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""SAS7BDAT datasets: header alignment, page size and page count against the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("sas",)
SCOPE = "32-byte magic, alignment markers, endianness, header length, page size and page count with header plus pages equal to the file; page contents not decoded"
MAGIC = (
    b"\0" * 12 + b"\xc2\xea\x81\x60\xb3\x14\x11\xcf\xbd\x92\x08\x00\x09\xc7\x31\x8c\x18\x1f\x10\x11"
)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 288:
        return Observation("fail", "Truncated header", "sas")
    align1 = 4 if data[32] == 0x33 else 0
    align2 = 4 if data[35] == 0x33 else 0
    order = ">" if data[37] == 0 else "<"
    header_length, page_size = struct.unpack_from(order + "II", data, 196 + align1)
    if align2:
        pages = struct.unpack_from(order + "Q", data, 204 + align1)[0]
    else:
        pages = struct.unpack_from(order + "I", data, 204 + align1)[0]
    if header_length < 1024 or page_size < 512 or page_size > 16 * 1024 * 1024:
        return Observation("fail", "Header length or page size out of range", "sas")
    expected = header_length + page_size * pages
    if expected != len(data):
        return Observation(
            "fail",
            f"{pages} pages of {page_size} bytes plus the header differ from the file",
            "sas",
        )
    return Observation(
        "pass",
        f"{pages} pages of {page_size} bytes; {'64-bit' if align2 else '32-bit'} layout",
        "sas",
    )
