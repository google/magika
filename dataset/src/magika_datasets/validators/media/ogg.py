# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Ogg pages: capture pattern, segment tables, CRC-32 per page, BOS/EOS per logical stream."""

import struct

from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("ogg",)
SCOPE = "Every page's capture pattern, version, segment table and CRC-32 (polynomial 0x04C11DB7, no reflection), pages tiling the file, BOS on each stream's first page; missing EOS is inconclusive; codec identified from the first packet, not decoded"
PAGES = 1_000_000
CODECS = {
    b"\x01vorbis": "vorbis",
    b"OpusHead": "opus",
    b"\x80theora": "theora",
    b"\x7fFLAC": "flac",
    b"Speex   ": "speex",
}


def table() -> list[int]:
    entries = []
    for byte in range(256):
        register = byte << 24
        for _ in range(8):
            register = ((register << 1) ^ 0x04C11DB7) if register & 0x80000000 else register << 1
        entries.append(register & 0xFFFFFFFF)
    return entries


TABLE = table()


def crc(data: bytes) -> int:
    register = 0
    for byte in data:
        register = ((register << 8) & 0xFFFFFFFF) ^ TABLE[((register >> 24) ^ byte) & 0xFF]
    return register


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"OggS":
        return None
    offset, count, streams, tags = 0, 0, {}, set()
    while offset < len(data):
        count += 1
        if count > PAGES:
            return Observation("inconclusive", "Page budget exceeded", "ogg")
        if data[offset : offset + 4] != b"OggS" or offset + 27 > len(data):
            return Observation("fail", f"Page {count} capture pattern missing or truncated", "ogg")
        if data[offset + 4] != 0:
            return Observation("fail", "Unsupported Ogg page version", "ogg")
        flags = data[offset + 5]
        serial, sequence, expected, segments = struct.unpack_from("<IIIB", data, offset + 14)
        table_end = offset + 27 + segments
        if table_end > len(data):
            return Observation("fail", "Segment table truncated", "ogg")
        size = table_end + sum(data[offset + 27 : table_end])
        if size > len(data):
            return Observation("fail", "Page body exceeds file", "ogg")
        page = bytearray(data[offset:size])
        page[22:26] = b"\0\0\0\0"
        if crc(bytes(page)) != expected:
            return Observation("fail", f"Page {count} CRC mismatch", "ogg")
        state = streams.get(serial)
        if state is None:
            if not flags & 0x02:
                return Observation("fail", "Stream begins without a BOS page", "ogg")
            tags.add(
                next(
                    (name for magic, name in CODECS.items() if data[table_end:].startswith(magic)),
                    "unknown_codec",
                )
            )
            streams[serial] = {"next": sequence + 1, "eos": False}
        else:
            if state["eos"]:
                return Observation("fail", "Page after EOS on the same stream", "ogg")
            state["next"] = sequence + 1
        if flags & 0x04:
            streams[serial]["eos"] = True
        offset = size
    if not streams:
        return Observation("fail", "No pages", "ogg")
    if not all(state["eos"] for state in streams.values()):
        return Observation(
            "inconclusive",
            f"{count} pages verified; a logical stream is not terminated with EOS",
            "ogg",
        )
    return Observation(
        "pass",
        f"{count} pages with CRC-32 verified across {len(streams)} stream(s)",
        "ogg",
        tuple(sorted(tags)),
    )
