# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Intel HEX: every record's length, type and checksum, ending with one EOF record."""

import re

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("intelhex",)
SCOPE = "Every line a colon record with matching byte count, known record type and valid checksum, exactly one EOF record as the last line; data semantics not interpreted"
RECORD = re.compile(rb":([0-9A-Fa-f]{2})([0-9A-Fa-f]{4})([0-9A-Fa-f]{2})([0-9A-Fa-f]*)")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b":") or not RECORD.match(data):
        return None
    lines = [line.strip() for line in data.splitlines()]
    lines = [line for line in lines if line]
    ended = False
    for number, line in enumerate(lines, 1):
        if ended:
            return Observation("fail", f"Record {number} follows the EOF record", "intelhex")
        match = RECORD.fullmatch(line)
        if not match:
            return Observation("fail", f"Line {number} is not a record", "intelhex")
        try:
            raw = bytes.fromhex(line[1:].decode("ascii"))
        except ValueError:
            return Observation("fail", f"Line {number} has odd hex digits", "intelhex")
        count, kind = raw[0], raw[3]
        if len(raw) != 5 + count:
            return Observation("fail", f"Line {number} byte count differs from data", "intelhex")
        if kind > 5:
            return Observation("fail", f"Line {number} unknown record type", "intelhex")
        if sum(raw) & 0xFF:
            return Observation("fail", f"Line {number} checksum mismatch", "intelhex")
        ended = kind == 1
    if not ended:
        return Observation("fail", "Missing EOF record", "intelhex")
    return Observation(
        "pass", f"{len(lines)} Intel HEX records with checksums verified", "intelhex"
    )
