# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Motorola S-record: every record's type, byte count, address width and checksum."""

import re

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("srecord",)
SCOPE = "Every non-blank line an S0-S9 record (S4 reserved) whose byte count matches its hex payload and whose one's-complement checksum holds, data records' address width matching their type, count records agreeing with the data records before them, at most one termination record and only as the last record; data semantics not interpreted"
PREFIX_ONLY = True  # "S", a digit and two hex digits also open ordinary text
RECORD = re.compile(rb"S([0-9])([0-9A-Fa-f]{2})((?:[0-9A-Fa-f]{2})*)")
ADDRESS = {0: 2, 1: 2, 2: 3, 3: 4, 5: 2, 6: 3, 7: 4, 8: 3, 9: 2}
DATA, COUNTS, ENDS = {1, 2, 3}, {5, 6}, {7, 8, 9}


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not RECORD.match(data):
        return None
    lines = [line.strip() for line in data.splitlines()]
    lines = [line for line in lines if line]
    records, ended = 0, False
    for number, line in enumerate(lines, 1):
        if ended:
            return Observation("fail", f"Record {number} follows the termination record", "srecord")
        match = RECORD.fullmatch(line)
        if not match:
            return Observation("fail", f"Line {number} is not an S-record", "srecord")
        kind = int(match.group(1))
        raw = bytes.fromhex((match.group(2) + match.group(3)).decode("ascii"))
        if kind == 4:
            return Observation("fail", f"Line {number} uses reserved type S4", "srecord")
        if raw[0] != len(raw) - 1 or raw[0] < ADDRESS[kind] + 1:
            return Observation(
                "fail", f"Line {number} byte count differs from its payload", "srecord"
            )
        if (sum(raw[:-1]) + raw[-1]) & 0xFF != 0xFF:
            return Observation("fail", f"Line {number} checksum mismatch", "srecord")
        if kind in COUNTS:
            declared = int.from_bytes(raw[1 : 1 + ADDRESS[kind]], "big")
            if declared != records:
                return Observation(
                    "fail",
                    f"Line {number} counts {declared} data records, found {records}",
                    "srecord",
                )
        records += kind in DATA
        ended = kind in ENDS
    if not records and not ended:
        return Observation("fail", "No data or termination records", "srecord")
    return Observation("pass", f"{len(lines)} S-records with checksums verified", "srecord")
