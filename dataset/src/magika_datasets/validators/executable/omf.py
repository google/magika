# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Intel/Microsoft OMF object modules: typed, length-prefixed, checksummed records."""

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("omf",)
SCOPE = "One or more object modules, each opening with THEADR or LHEADR whose name length fits the record and closing with MODEND; every record of a known OMF type with its length inside the file and a checksum byte that is zero or makes the record sum to zero; trailing zero padding allowed; OMF libraries (LIBHDR) and record contents not interpreted"
PREFIX_ONLY = True  # one type byte and a length also open many binaries; failures need a hint
HEADERS = {0x80, 0x82}
MODEND = {0x8A, 0x8B}
KNOWN = (
    HEADERS
    | MODEND
    | {
        0x88,
        0x8C,
        0x90,
        0x91,
        0x94,
        0x95,
        0x96,
        0x98,
        0x99,
        0x9A,
        0x9C,
        0x9D,
        0xA0,
        0xA1,
        0xA2,
        0xA3,
        0xB0,
        0xB2,
        0xB3,
        0xB4,
        0xB5,
        0xB6,
        0xB7,
        0xB8,
        0xBC,
        0xC2,
        0xC3,
        0xC4,
        0xC5,
        0xC6,
        0xC8,
        0xC9,
        0xCA,
        0xCC,
        0xCE,
    }
)
RECORDS = 2_000_000


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 5 or data[0] not in HEADERS:
        return None
    (length,) = struct.unpack_from("<H", data, 1)
    if not 2 <= length <= len(data) - 3 or data[3] + 2 > length:
        return None  # the header record's own name length must fit before this is OMF
    offset, records, modules, open_module = 0, 0, 0, False
    while offset < len(data):
        kind = data[offset]
        if not open_module and not data[offset:].strip(b"\0"):
            break  # zero padding after the last module
        if offset + 3 > len(data):
            return Observation("fail", f"Record header truncated at offset {offset}", "omf")
        (length,) = struct.unpack_from("<H", data, offset + 1)
        end = offset + 3 + length
        if length < 1 or end > len(data):
            return Observation("fail", f"Record at offset {offset} extends past the end", "omf")
        if kind not in KNOWN:
            return Observation("fail", f"Unknown record type {kind:#04x} at offset {offset}", "omf")
        if data[end - 1] and sum(data[offset:end]) & 0xFF:
            return Observation("fail", f"Record at offset {offset} fails its checksum", "omf")
        if kind in HEADERS:
            if open_module:
                return Observation("fail", f"Module header at offset {offset} before MODEND", "omf")
            if data[offset + 3] + 2 > length:
                return Observation("fail", "Module name longer than its record", "omf")
            open_module = True
        elif not open_module:
            return Observation("fail", f"Record at offset {offset} outside any module", "omf")
        elif kind in MODEND:
            open_module = False
            modules += 1
        offset = end
        records += 1
        if records > RECORDS:
            return Observation("inconclusive", "Record budget exceeded", "omf")
    if open_module:
        return Observation("fail", "Last module has no MODEND record", "omf")
    return Observation("pass", f"{modules} modules of {records} checksummed records", "omf")
