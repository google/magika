# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""GRIB edition 1 and 2 messages: section lengths and 7777 terminators tiling the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("grib",)
SCOPE = "GRIB indicator with edition and total length, edition 1 sections by flags with 24-bit lengths, edition 2 sections with 32-bit lengths, 7777 terminator, messages tiling the file; fields not decoded"
MESSAGES = 100_000


def edition1(data: bytes, offset: int) -> int:
    total = int.from_bytes(data[offset + 4 : offset + 7], "big")
    end = offset + total
    if total < 24 or end > len(data) or data[end - 4 : end] != b"7777":
        raise ValueError("Edition 1 length or terminator invalid")
    position = offset + 8
    length = int.from_bytes(data[position : position + 3], "big")
    if length < 28:
        raise ValueError("Product definition section too short")
    flags = data[position + 7]
    position += length
    for present in (flags & 0x80, flags & 0x40):
        if present:
            position += int.from_bytes(data[position : position + 3], "big")
    position += int.from_bytes(data[position : position + 3], "big")  # binary data section
    if position != end - 4:
        raise ValueError("Edition 1 sections do not tile the message")
    return end


def edition2(data: bytes, offset: int) -> int:
    total = struct.unpack_from(">Q", data, offset + 8)[0]
    end = offset + total
    if total < 21 or end > len(data) or data[end - 4 : end] != b"7777":
        raise ValueError("Edition 2 length or terminator invalid")
    position, count = offset + 16, 0
    while position < end - 4:
        count += 1
        if count > 4096 or position + 5 > end:
            raise ValueError("Edition 2 section table invalid")
        length, number = struct.unpack_from(">IB", data, position)
        if length < 5 or not 1 <= number <= 7:
            raise ValueError("Edition 2 section invalid")
        position += length
    if position != end - 4:
        raise ValueError("Edition 2 sections do not tile the message")
    return end


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"GRIB" or len(data) < 8:
        return None
    offset, messages, padded = 0, 0, False
    try:
        while offset < len(data):
            if data[offset] == 0:  # writers pad between and after messages with NUL bytes
                stripped = len(data) - len(data[offset:].lstrip(b"\0"))
                offset, padded = stripped, True
                continue
            messages += 1
            if messages > MESSAGES:
                return Observation("inconclusive", "Message budget exceeded", "grib")
            if data[offset : offset + 4] != b"GRIB" or offset + 16 > len(data):
                raise ValueError(f"Message {messages} indicator missing")
            edition = data[offset + 7]
            if edition == 1:
                offset = edition1(data, offset)
            elif edition == 2:
                offset = edition2(data, offset)
            else:
                raise ValueError(f"Unknown GRIB edition {edition}")
    except (ValueError, struct.error) as error:
        return Observation(
            "fail", str(error) if isinstance(error, ValueError) else "Truncated structure", "grib"
        )
    if not messages:
        return Observation("fail", "No messages", "grib")
    return Observation(
        "pass", f"{messages} GRIB messages tiling the file", "grib", ("padded",) if padded else ()
    )
