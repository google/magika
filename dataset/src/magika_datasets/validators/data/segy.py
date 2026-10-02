# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""SEG-Y seismic data: file headers, then 240-byte trace headers and samples tiling the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("segy",)
SCOPE = "3200-byte textual header (EBCDIC or ASCII card images) and 400-byte binary header in big- or little-endian order: a known data sample format code, extended textual header count, then every trace's 240-byte header and its sample count (the trace's own, or the binary header's when fixed-length or unset) tiling the file exactly; sample values not decoded"
PREFIX_ONLY = True  # a leading "C" also opens ordinary text; failures need a hint
TEXT, BINARY, TRACE = 3200, 400, 240
BYTES = {1: 4, 2: 4, 3: 2, 5: 4, 6: 8, 8: 1, 9: 8, 10: 4, 11: 2, 12: 8, 15: 3, 16: 1}
TRACES = 5_000_000
EBCDIC_C, ASCII_C = 0xC3, ord("C")


def order(binary: bytes) -> str | None:
    """Byte order whose format code is known, preferring big-endian as the standard says."""
    for prefix in (">", "<"):
        if struct.unpack_from(prefix + "H", binary, 24)[0] in BYTES:
            return prefix
    return None


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < TEXT + BINARY or data[0] not in (EBCDIC_C, ASCII_C) and "segy" not in hints:
        return None
    binary = data[TEXT : TEXT + BINARY]
    prefix = order(binary)
    if prefix is None:
        return Observation("fail", "Binary header has no known data sample format code", "segy")
    samples, code = (
        struct.unpack_from(prefix + "HH", binary, 20)[0],
        struct.unpack_from(prefix + "H", binary, 24)[0],
    )
    revision, fixed, extended = struct.unpack_from(prefix + "HHh", binary, 300)
    if extended < 0:
        return Observation(
            "inconclusive", "Variable extended textual headers end at a stanza", "segy"
        )
    width = BYTES[code]
    offset, traces = TEXT + BINARY + extended * TEXT, 0
    while offset < len(data):
        if offset + TRACE > len(data):
            return Observation("fail", f"Trace {traces + 1} header truncated", "segy")
        own = struct.unpack_from(prefix + "H", data, offset + 114)[0]
        count = samples if (fixed and revision >= 0x0100) or not own else own
        if not count:
            return Observation("fail", f"Trace {traces + 1} has no sample count", "segy")
        offset += TRACE + count * width
        traces += 1
        if traces > TRACES:
            return Observation("inconclusive", "Trace budget exceeded", "segy")
    if offset != len(data) or not traces:
        return Observation("fail", "Traces do not tile the file", "segy")
    return Observation(
        "pass",
        f"{traces} traces of format code {code} tile the file after the headers",
        "segy",
    )
