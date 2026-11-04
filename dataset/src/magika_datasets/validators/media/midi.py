# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Standard MIDI files: header chunk, exact track count, event walk to End of Track."""

import struct

from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("midi",)
SCOPE = "MThd header fields, exactly the declared number of MTrk chunks tiling the file, every track's delta times, running status, meta and sysex lengths walked to a final End of Track; nothing is played"
EVENTS = 1_000_000


def varint(data: bytes, offset: int, end: int) -> tuple[int, int]:
    value = 0
    for _ in range(4):
        if offset >= end:
            raise ValueError("Truncated variable-length quantity")
        byte = data[offset]
        offset += 1
        value = (value << 7) | (byte & 0x7F)
        if not byte & 0x80:
            return value, offset
    raise ValueError("Variable-length quantity too long")


def track(data: bytes, offset: int, end: int) -> None:
    status_byte, count = None, 0
    while offset < end:
        count += 1
        if count > EVENTS:
            raise ValueError("Event budget exceeded")
        _, offset = varint(data, offset, end)
        if offset >= end:
            raise ValueError("Truncated event")
        byte = data[offset]
        if byte == 0xFF:
            kind = data[offset + 1] if offset + 1 < end else None
            length, offset = varint(data, offset + 2, end)
            offset += length
            if offset > end:
                raise ValueError("Meta event exceeds track")
            if kind == 0x2F:
                if offset != end:
                    raise ValueError("Data after End of Track")
                return
            continue
        if byte in (0xF0, 0xF7):
            length, offset = varint(data, offset + 1, end)
            offset += length
            if offset > end:
                raise ValueError("Sysex event exceeds track")
            continue
        if byte & 0x80:
            status_byte, offset = byte, offset + 1
        if status_byte is None or status_byte >= 0xF0:
            raise ValueError("Data byte without running status")
        offset += 1 if status_byte & 0xF0 in (0xC0, 0xD0) else 2
        if offset > end:
            raise ValueError("Channel event exceeds track")
    raise ValueError("Track lacks End of Track")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"MThd" or len(data) < 14:
        return None
    length, fmt, tracks, division = struct.unpack_from(">IHHH", data, 4)
    if length != 6 or fmt > 2 or not tracks or division == 0:
        return Observation("fail", "Invalid MThd fields", "midi")
    offset, seen = 8 + length, 0
    try:
        while offset < len(data):
            if data[offset : offset + 4] != b"MTrk" or offset + 8 > len(data):
                raise ValueError("Expected MTrk chunk")
            size = struct.unpack_from(">I", data, offset + 4)[0]
            end = offset + 8 + size
            if end > len(data):
                raise ValueError("Track exceeds file")
            track(data, offset + 8, end)
            seen += 1
            offset = end
        if seen != tracks:
            raise ValueError("Track count differs from header")
    except ValueError as error:
        status = "inconclusive" if "budget" in str(error) else "fail"
        return Observation(status, str(error), "midi")
    return Observation("pass", f"{seen} tracks walked to End of Track", "midi")
