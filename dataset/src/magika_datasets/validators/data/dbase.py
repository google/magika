# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""dBase tables: header arithmetic and field descriptors; a version byte is a weak prefix."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("dbase",)
SCOPE = "Version byte, header length covering the field descriptors and terminator, record count times record length plus header (and optional EOF byte) equal to the file; field descriptors with printable names and known types; a one-byte version prefix is weak, so failures need a hint"
PREFIX_ONLY = True
VERSIONS = {
    0x02,
    0x03,
    0x04,
    0x05,
    0x30,
    0x31,
    0x32,
    0x43,
    0x63,
    0x83,
    0x8B,
    0x8E,
    0xCB,
    0xE5,
    0xF5,
    0xFB,
}
FIELD_TYPES = set(b"CNLDMFBGIPTY+O@V0W")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 33 or data[0] not in VERSIONS:
        return None
    _, year, month, day, records, header_len, record_len = struct.unpack_from("<BBBBIHH", data)
    if not 1 <= month <= 12 or not 1 <= day <= 31 or header_len < 33 or record_len < 1:
        return None
    if header_len > len(data) or data[header_len - 1] != 0x0D:
        return Observation("fail", "Header length or terminator invalid", "dbase")
    fields = (header_len - 33) // 32
    total = 0
    for index in range(fields):
        base = 32 + 32 * index
        name, kind, size = data[base : base + 11], data[base + 11], data[base + 16]
        if kind not in FIELD_TYPES or not name.split(b"\0", 1)[0]:
            return Observation("fail", f"Field {index} descriptor invalid", "dbase")
        total += size
    if fields and total + 1 != record_len and data[0] not in (0x30, 0x31, 0x32):
        return Observation("fail", "Field widths do not sum to the record length", "dbase")
    expected = header_len + records * record_len
    if len(data) not in (expected, expected + 1):
        return Observation(
            "fail", f"{records} records of {record_len} bytes do not fill the file", "dbase"
        )
    if len(data) == expected + 1 and data[-1] != 0x1A:
        return Observation("fail", "Trailing byte is not the EOF marker", "dbase")
    return Observation(
        "pass", f"{records} records of {record_len} bytes with {fields} fields", "dbase"
    )
