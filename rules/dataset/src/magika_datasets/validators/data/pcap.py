# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Classic pcap captures: global header and packet records tiling the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("pcap",)
SCOPE = "Magic in four variants, version 2.4, snaplen, every packet record's captured length at most the original and the snaplen, records tiling the file; packet contents not decoded"
MAGICS = {
    b"\xd4\xc3\xb2\xa1": ("<", ()),
    b"\xa1\xb2\xc3\xd4": (">", ()),
    b"\x4d\x3c\xb2\xa1": ("<", ("nanoseconds",)),
    b"\xa1\xb2\x3c\x4d": (">", ("nanoseconds",)),
}
PACKETS = 10_000_000


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    magic = MAGICS.get(data[:4])
    if magic is None:
        return None
    order, tags = magic
    if len(data) < 24:
        return Observation("fail", "Truncated global header", "pcap")
    major, minor, _, _, snaplen, _ = struct.unpack_from(order + "HHiIII", data, 4)
    if (major, minor) != (2, 4):
        return Observation("fail", f"Unsupported pcap version {major}.{minor}", "pcap")
    offset, count = 24, 0
    while offset < len(data):
        count += 1
        if count > PACKETS:
            return Observation("inconclusive", "Packet budget exceeded", "pcap")
        if offset + 16 > len(data):
            return Observation("fail", "Truncated packet header", "pcap")
        _, _, captured, original = struct.unpack_from(order + "IIII", data, offset)
        if (
            captured > original
            or (snaplen and captured > snaplen)
            or offset + 16 + captured > len(data)
        ):
            return Observation("fail", f"Packet {count} length invalid", "pcap")
        offset += 16 + captured
    return Observation("pass", f"{count} packets tiling the capture", "pcap", tags)
