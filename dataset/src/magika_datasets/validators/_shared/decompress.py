# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Bounded decompression shared by stream and container validators."""

from ..contract import Observation

LIMIT = 64 * 1024 * 1024
MEMBERS = 4096


class Budget(Exception):
    """Expanded output or stream count would exceed the budget."""


class Truncated(Exception):
    """Input ended before the stream's end marker."""


class Trailing(Exception):
    """Bytes remain after the final stream that do not start another stream."""


def drain(decompressor, data: bytes, limit: int) -> tuple[int, bytes]:
    """Inflate one stream completely. Returns (expanded length, unused trailing bytes)."""
    total, pending = 0, data
    while True:
        output = decompressor.decompress(pending, limit + 1 - total)
        total += len(output)
        if total > limit:
            raise Budget("Expanded output exceeds budget")
        if decompressor.eof:
            return total, decompressor.unused_data
        tail = getattr(decompressor, "unconsumed_tail", None)
        if tail is None:  # bz2/lzma style: buffered input, needs_input flag
            if decompressor.needs_input:
                raise Truncated("Stream ended before its end marker")
            pending = b""
        elif tail:  # zlib style: unconsumed input is handed back
            pending = tail
        else:
            raise Truncated("Stream ended before its end marker")


def streams(factory, data: bytes, magic: bytes, *, limit: int = LIMIT, padding: int = 0):
    """Inflate concatenated streams. Returns (stream count, expanded length)."""
    count, total, rest = 0, 0, data
    while rest:
        if not rest.startswith(magic):
            raise Trailing("Trailing bytes are not another stream")
        expanded, rest = drain(factory(), rest, limit - total)
        total += expanded
        count += 1
        if count > MEMBERS:
            raise Budget("Stream count exceeds budget")
        while padding and rest.startswith(b"\x00" * padding):
            rest = rest[padding:]
    return count, total


def inspect(
    format_id: str, factory, data: bytes, magic: bytes, corrupt, *, padding=0, concatenated=True
) -> Observation:
    """Whole-file observation for a compressed stream format."""
    try:
        if concatenated:
            count, expanded = streams(factory, data, magic, padding=padding)
        else:
            expanded, rest = drain(factory(), data, LIMIT)
            if rest:
                raise Trailing("Trailing bytes after stream")
            count = 1
    except Budget as error:
        return Observation("inconclusive", str(error), format_id)
    except (Truncated, Trailing) as error:
        return Observation("fail", str(error), format_id)
    except corrupt:
        return Observation("fail", "Corrupt compressed data or checksum", format_id)
    return Observation(
        "pass",
        f"{count} stream(s) inflated to {expanded} bytes; framing and checksums verified",
        format_id,
        generic=True,  # a joblib pickle is a zlib stream; a gzipped graph is gzip: never relabel hinted inner formats
    )
