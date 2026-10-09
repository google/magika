# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""POSIX ustar, GNU and v7 tar archives: header checksums and block accounting."""

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("tar",)
SCOPE = "Header checksums, size fields, entry data blocks and zero-block termination for ustar/GNU/v7 headers; entry contents not interpreted"
BLOCK = 512
ENTRIES = 4096
DATALESS = b"123456"  # link, symlink, char, block, directory, fifo


def number(field: bytes) -> int | None:
    if field[0] & 0x80:  # GNU base-256 extension
        return int.from_bytes(bytes([field[0] & 0x7F]) + field[1:], "big")
    text = field.split(b"\x00", 1)[0].strip(b" ")
    if not text:
        return 0
    try:
        return int(text, 8)
    except ValueError:
        return None


def checksum_matches(block: bytes) -> bool:
    stored = number(block[148:156])
    if stored is None:
        return False
    body = block[:148] + b" " * 8 + block[156:]
    return stored in (sum(body), sum(b - 256 if b > 127 else b for b in body))


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < BLOCK or not any(data[:BLOCK]):
        return None
    header = data[:BLOCK]
    if header[257:262] != b"ustar" and not ("tar" in hints and checksum_matches(header)):
        return None
    offset, entries = 0, 0
    while offset + BLOCK <= len(data):
        block = data[offset : offset + BLOCK]
        if not any(block):
            remainder = data[offset:]
            if any(remainder) or len(remainder) % BLOCK:
                return Observation(
                    "fail", "Nonzero or partial bytes after end-of-archive block", "tar"
                )
            if len(remainder) < 2 * BLOCK:
                return Observation("inconclusive", "Single zero block terminator", "tar")
            return Observation(
                "pass",
                f"{entries} entries; header checksums, block bounds and zero-block termination checked",
                "tar",
            )
        if not checksum_matches(block):
            return Observation("fail", "Header checksum mismatch", "tar")
        size = number(block[124:136])
        if size is None:
            return Observation("fail", "Invalid size field", "tar")
        entries += 1
        if entries > ENTRIES:
            return Observation("inconclusive", "Entry budget exceeded", "tar")
        blocks = 0 if block[156] in DATALESS else (size + BLOCK - 1) // BLOCK
        offset += BLOCK * (1 + blocks)
        if offset > len(data):
            return Observation("fail", "Entry data exceeds file bounds", "tar")
    if offset == len(data):
        return Observation("inconclusive", "Archive ends without zero-block terminator", "tar")
    return Observation("fail", "Trailing partial block", "tar")
