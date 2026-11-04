# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""FITS files: 2880-byte header and data units sized from the header cards."""

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("fits",)
SCOPE = "SIMPLE = T first card, 80-column cards to END padded to 2880 bytes, data unit sized from BITPIX, NAXISn, PCOUNT and GCOUNT for every HDU, units tiling the file; values not interpreted"
BLOCK = 2880
UNITS = 4096


class Malformed(Exception):
    pass


def cards(data: bytes, offset: int) -> tuple[dict[str, str], int]:
    values, position, seen = {}, offset, 0
    while True:
        if position + 80 > len(data):
            raise Malformed("Header unit truncated before END")
        card = data[position : position + 80]
        position += 80
        seen += 1
        if seen > 100_000:
            raise Malformed("Header card budget exceeded")
        if card.startswith(b"END") and not card[3:].strip():
            break
        if card[8:10] == b"= " and card[:8].strip():
            key = card[:8].decode("ascii", "replace").strip()
            value = card[10:].split(b"/", 1)[0].strip().decode("ascii", "replace")
            values.setdefault(key, value)
    position += -position % BLOCK
    return values, position


def integer(values: dict[str, str], key: str, default: int | None = None) -> int:
    raw = values.get(key)
    if raw is None:
        if default is None:
            raise Malformed(f"Missing {key} card")
        return default
    try:
        return int(raw)
    except ValueError as error:
        raise Malformed(f"{key} is not an integer") from error


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"SIMPLE  ="):
        return None
    offset, units = 0, 0
    try:
        while offset < len(data):
            units += 1
            if units > UNITS:
                return Observation("inconclusive", "HDU budget exceeded", "fits")
            values, offset = cards(data, offset)
            if units == 1 and values.get("SIMPLE") != "T":
                raise Malformed("SIMPLE is not T")
            if units > 1 and "XTENSION" not in values:
                raise Malformed("Extension unit without XTENSION")
            bitpix, naxis = integer(values, "BITPIX"), integer(values, "NAXIS")
            if bitpix not in (8, 16, 32, 64, -32, -64) or not 0 <= naxis <= 999:
                raise Malformed("Invalid BITPIX or NAXIS")
            size = 1 if naxis else 0
            for axis in range(1, naxis + 1):
                size *= integer(values, f"NAXIS{axis}")
            size = (
                abs(bitpix)
                // 8
                * integer(values, "GCOUNT", 1)
                * (integer(values, "PCOUNT", 0) + size)
            )
            offset += size + (-size % BLOCK)
            if offset > len(data):
                raise Malformed(f"Data unit {units} exceeds file")
    except Malformed as error:
        return Observation("fail", str(error), "fits")
    if len(data) % BLOCK:
        return Observation("fail", "File is not a whole number of 2880-byte blocks", "fits")
    return Observation("pass", f"{units} header/data units tiling the file", "fits")
