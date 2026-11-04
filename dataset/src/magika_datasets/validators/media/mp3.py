# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""MPEG audio layers III and II: ID3v2 header, frame chain from the standard tables, tails."""

import struct

from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("mp3", "mp2")
SCOPE = "Optional ID3v2 header (syncsafe size, footer), every MPEG layer III (mp3) or layer II (mp2) frame header's version, one layer throughout,, bitrate, sample rate and padding with the resulting frame length chaining to the end, optional ID3v1 and APEv2 tails; free-format bitrate inconclusive; audio not decoded"
FRAMES = 1_000_000
# (version key, layer) -> bitrate table in kbit/s; version key 1 = MPEG1, 2 = MPEG2/2.5
BITRATES = {
    (1, 3): [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0],
    (2, 3): [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0],
    (1, 2): [0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 0],
    (2, 2): [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0],
}
LAYERS = {1: 3, 2: 2}  # header layer bits to layer number
NAMES = {3: "mp3", 2: "mp2"}
RATES = {3: [44100, 48000, 32000], 2: [22050, 24000, 16000], 0: [11025, 12000, 8000]}


def frame_layer(data: bytes, offset: int) -> int | None:
    """Layer (3 or 2) of the frame header at offset, or None if there is none."""
    if offset + 4 > len(data) or data[offset] != 0xFF or data[offset + 1] & 0xE0 != 0xE0:
        return None
    if (data[offset + 1] >> 3) & 0x3 == 1:  # reserved version
        return None
    return LAYERS.get((data[offset + 1] >> 1) & 0x3)


def frame_length(data: bytes, offset: int, layer: int = 3) -> int | None:
    """Byte length of the frame at offset, 0 for free format, None if not a frame of layer."""
    if frame_layer(data, offset) != layer:
        return None
    version_bits = (data[offset + 1] >> 3) & 0x3
    bitrate_index = data[offset + 2] >> 4
    rate_index = (data[offset + 2] >> 2) & 0x3
    padding = (data[offset + 2] >> 1) & 0x1
    if bitrate_index == 15 or rate_index == 3:
        return None
    version = 1 if version_bits == 3 else 2
    bitrate = BITRATES[(version, layer)][bitrate_index]
    rate = RATES[version_bits][rate_index]
    if not bitrate:
        return 0
    samples = 576 if layer == 3 and version == 2 else 1152
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
    if not start and frame_layer(data, 0) is None:
        return None
    for _ in range(16):  # taggers sometimes chain several ID3v2 tags
        if start > len(data):
            return Observation(
                "fail", "ID3v2 header exceeds file", "mp2" if "mp2" in hints else "mp3"
            )
        more = id3v2(data[start:])
        if not more:
            break
        start += more
    # The first frame fixes the layer; a file mixing layers is not a stream of either.
    layer = frame_layer(data, start) or (2 if "mp2" in hints and "mp3" not in hints else 3)
    name = NAMES[layer]
    end = tails(data, len(data))
    offset, frames = start, 0
    while offset < end:
        frames += 1
        if frames > FRAMES:
            return Observation("inconclusive", "Frame budget exceeded", name)
        length = frame_length(data, offset, layer)
        if length is None:
            return Observation("fail", f"Frame {frames} header invalid at offset {offset}", name)
        if length == 0:
            return Observation("inconclusive", "Free-format bitrate frames not sized", name)
        if offset + length > end:
            return Observation("fail", f"Frame {frames} exceeds audio data", name)
        offset += length
    if not frames:
        return Observation("fail", "No audio frames", name)
    return Observation(
        "pass", f"{frames} layer {'III' if layer == 3 else 'II'} frames chained; tags bounded", name
    )
