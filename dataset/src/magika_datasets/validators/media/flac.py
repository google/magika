# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""FLAC: metadata blocks, frame headers with CRC-8, frame CRC-16, frames tiling the file."""

import struct

from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("flac",)
SCOPE = "fLaC marker, STREAMINFO first with sane fields, every metadata block bounded, every frame header sync and CRC-8, every frame's CRC-16 up to the next verified header or EOF, sample total when declared; trailing zero bytes after the last frame are invisible to CRC-16; audio not decoded"
FRAMES = 200_000
BLOCK_SIZES = {
    1: 192,
    2: 576,
    3: 1152,
    4: 2304,
    5: 4608,
    8: 256,
    9: 512,
    10: 1024,
    11: 2048,
    12: 4096,
    13: 8192,
    14: 16384,
    15: 32768,
}


def table(polynomial: int, width: int) -> list[int]:
    top, mask = 1 << (width - 1), (1 << width) - 1
    entries = []
    for byte in range(256):
        register = byte << (width - 8)
        for _ in range(8):
            register = ((register << 1) ^ polynomial) if register & top else register << 1
        entries.append(register & mask)
    return entries


CRC8 = table(0x07, 8)
CRC16 = table(0x8005, 16)


def crc8(data: bytes) -> int:
    register = 0
    for byte in data:
        register = CRC8[register ^ byte]
    return register


def crc16(data: bytes) -> int:
    register = 0
    for byte in data:
        register = ((register << 8) & 0xFFFF) ^ CRC16[(register >> 8) ^ byte]
    return register


def header_length(data: bytes, offset: int) -> tuple[int, int] | None:
    """Length of the frame header at offset (including CRC-8) and its block size, or None."""
    if offset + 5 > len(data) or data[offset] != 0xFF or data[offset + 1] & 0xFE != 0xF8:
        return None
    block_code, rate_code = data[offset + 2] >> 4, data[offset + 2] & 0x0F
    if block_code == 0 or rate_code == 15 or (data[offset + 3] >> 4) > 10:
        return None
    position = offset + 4
    first = data[position]
    width = (
        1
        if first < 0x80
        else (
            2
            if first >> 5 == 0b110
            else 3
            if first >> 4 == 0b1110
            else 4
            if first >> 3 == 0b11110
            else 5
            if first >> 2 == 0b111110
            else 6
            if first >> 1 == 0b1111110
            else 7
            if first == 0xFE
            else 0
        )
    )
    if not width:
        return None
    position += width
    if block_code == 6:
        block, position = data[position] + 1, position + 1
    elif block_code == 7:
        block, position = struct.unpack_from(">H", data, position)[0] + 1, position + 2
    else:
        block = BLOCK_SIZES[block_code]
    if rate_code == 12:
        position += 1
    elif rate_code in (13, 14):
        position += 2
    if position + 1 > len(data):
        return None
    if crc8(data[offset:position]) != data[position]:
        return None
    return position + 1 - offset, block


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"fLaC":
        return None
    offset, last, first = 4, False, True
    total_samples = 0
    while not last:
        if offset + 4 > len(data):
            return Observation("fail", "Truncated metadata block header", "flac")
        last, kind = bool(data[offset] & 0x80), data[offset] & 0x7F
        length = int.from_bytes(data[offset + 1 : offset + 4], "big")
        if first and (kind != 0 or length != 34):
            return Observation("fail", "STREAMINFO must be the first block", "flac")
        if offset + 4 + length > len(data):
            return Observation("fail", "Metadata block exceeds file", "flac")
        if first:
            packed = int.from_bytes(data[offset + 14 : offset + 22], "big")
            rate, channels, bits = (
                packed >> 44,
                ((packed >> 41) & 0x7) + 1,
                ((packed >> 36) & 0x1F) + 1,
            )
            total_samples = packed & ((1 << 36) - 1)
            if not rate or rate > 655350 or channels > 8 or bits < 4 or bits > 32:
                return Observation("fail", "STREAMINFO fields out of range", "flac")
            first = False
        offset += 4 + length
    frames, samples = 0, 0
    while offset < len(data):
        frames += 1
        if frames > FRAMES:
            return Observation("inconclusive", "Frame budget exceeded", "flac")
        header = header_length(data, offset)
        if header is None:
            return Observation("fail", f"Frame {frames} header sync or CRC-8 invalid", "flac")
        length, block = header
        end = offset + length
        while True:  # the frame ends where the next verified header starts, or at EOF
            end = data.find(b"\xff", end)
            if end < 0 or end + 2 > len(data):
                end = len(data)
                break
            if data[end + 1] & 0xFE == 0xF8 and header_length(data, end) is not None:
                break
            end += 1
        if (
            end - offset < length + 2
            or crc16(data[offset : end - 2]) != struct.unpack_from(">H", data, end - 2)[0]
        ):
            return Observation("fail", f"Frame {frames} CRC-16 mismatch", "flac")
        samples += block
        offset = end
    if not frames:
        return Observation("fail", "No audio frames", "flac")
    if total_samples and samples < total_samples:
        return Observation("fail", "Frames carry fewer samples than STREAMINFO declares", "flac")
    return Observation("pass", f"{frames} frames with CRC-8 and CRC-16 verified", "flac")
