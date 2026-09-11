# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""MPEG audio layer III: ID3v2 header, frame chain from the standard tables, ID3v1/APE tails."""

import struct

from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("mp3",)
SCOPE = "Optional ID3v2 header (syncsafe size, footer), every MPEG layer III frame header's version, bitrate, sample rate and padding with the resulting frame length chaining to the end, optional ID3v1 and APEv2 tails; free-format bitrate inconclusive; audio not decoded"
FRAMES = 1_000_000
# (version key, layer) -> bitrate table in kbit/s; version key 1 = MPEG1, 2 = MPEG2/2.5
BITRATES = {
    (1, 3): [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0],
    (2, 3): [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0],
}
RATES = {3: [44100, 48000, 32000], 2: [22050, 24000, 16000], 0: [11025, 12000, 8000]}


def frame_length(data: bytes, offset: int) -> int | None:
    """Byte length of the layer III frame at offset, 0 for free format, None if not a frame."""
    if offset + 4 > len(data) or data[offset] != 0xFF or data[offset + 1] & 0xE0 != 0xE0:
        return None
    version_bits = (data[offset + 1] >> 3) & 0x3
    layer_bits = (data[offset + 1] >> 1) & 0x3
    if version_bits == 1 or layer_bits != 1:  # reserved version, or not layer III
        return None
    bitrate_index = data[offset + 2] >> 4
    rate_index = (data[offset + 2] >> 2) & 0x3
    padding = (data[offset + 2] >> 1) & 0x1
    if bitrate_index == 15 or rate_index == 3:
        return None
    version = 1 if version_bits == 3 else 2
    bitrate = BITRATES[(version, 3)][bitrate_index]
    rate = RATES[version_bits][rate_index]
    if not bitrate:
        return 0
    samples = 1152 if version == 1 else 576
    return samples * bitrate * 1000 // 8 // rate + padding


def id3v2(data: bytes) -> int:
    if data[:3] != b"ID3" or len(data) < 10 or any(byte & 0x80 for byte in data[6:10]):
        return 0
    size = (data[6] << 21) | (data[7] << 14) | (data[8] << 7) | data[9]
    return 10 + size + (10 if data[5] & 0x10 else 0)


def tails(data: bytes, end: int) -> int:
    """Strip ID3v1 and APEv2 tags from the end; returns the end of audio data."""
    if end >= 128 and data[end - 128 : end - 125] == b"TAG":
        end -= 128
    if end >= 32 and data[end - 32 : end - 24] == b"APETAGEX":
        size, _, flags = struct.unpack_from("<III", data, end - 20)
        header = 32 if flags & 0x80000000 else 0
        end -= size + header
    return end


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    start = id3v2(data)
    if not start and frame_length(data, 0) is None:
        return None
    for _ in range(16):  # taggers sometimes chain several ID3v2 tags
        if start > len(data):
            return Observation("fail", "ID3v2 header exceeds file", "mp3")
        more = id3v2(data[start:])
        if not more:
            break
        start += more
    end = tails(data, len(data))
    offset, frames = start, 0
    while offset < end:
        frames += 1
        if frames > FRAMES:
            return Observation("inconclusive", "Frame budget exceeded", "mp3")
        length = frame_length(data, offset)
        if length is None:
            return Observation("fail", f"Frame {frames} header invalid at offset {offset}", "mp3")
        if length == 0:
            return Observation("inconclusive", "Free-format bitrate frames not sized", "mp3")
        if offset + length > end:
            return Observation("fail", f"Frame {frames} exceeds audio data", "mp3")
        offset += length
    if not frames:
        return Observation("fail", "No audio frames", "mp3")
    return Observation("pass", f"{frames} layer III frames chained; tags bounded", "mp3")
