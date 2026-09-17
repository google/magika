# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""MPEG transport streams: sync-byte lattice, adaptation field bounds and a PAT."""

from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("mpegts",)
SCOPE = "Sync byte on every 188-byte packet (or 192-byte M2TS packet) through the whole file, adaptation field lengths, at least one program association table; elementary streams not decoded"
PACKETS = 1_000_000


def lattice(data: bytes) -> tuple[int, int] | None:
    """(packet size, sync offset) when the first three packets align."""
    for size, skip in ((188, 0), (192, 4)):
        if len(data) >= 3 * size and all(data[skip + size * n] == 0x47 for n in range(3)):
            return size, skip
    return None


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    grid = lattice(data)
    if grid is None:
        return None
    size, skip = grid
    if len(data) % size:
        return Observation("fail", "File length is not a whole number of packets", "mpegts")
    count = len(data) // size
    if count > PACKETS:
        return Observation("inconclusive", "Packet budget exceeded", "mpegts")
    pat = False
    for number in range(count):
        offset = number * size + skip
        if data[offset] != 0x47:
            return Observation("fail", f"Sync byte missing at packet {number}", "mpegts")
        pid = ((data[offset + 1] & 0x1F) << 8) | data[offset + 2]
        control = data[offset + 3] >> 4 & 0x3
        if control & 0x2 and data[offset + 4] > 183:
            return Observation("fail", f"Adaptation field overflows packet {number}", "mpegts")
        pat |= pid == 0
    if not pat:
        return Observation("fail", "No program association table packet", "mpegts")
    tags = ("m2ts",) if size == 192 else ()
    return Observation(
        "pass", f"{count} packets on a {size}-byte lattice with a PAT", "mpegts", tags
    )
