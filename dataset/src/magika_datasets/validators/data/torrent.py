# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""BitTorrent metainfo: complete bencode parse and required info dictionary keys."""

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("torrent",)
SCOPE = "Bencode parsed exactly to EOF with depth and item budgets, top-level dictionary with an info dictionary holding name, piece length and pieces (a multiple of 20 bytes); nothing fetched"
ITEMS = 1_000_000


class Malformed(Exception):
    pass


def decode(data: bytes, offset: int, depth: int, state: dict):
    state["items"] += 1
    if state["items"] > ITEMS or depth > 64:
        raise Malformed("Budget exceeded")
    if offset >= len(data):
        raise Malformed("Truncated value")
    byte = data[offset]
    if byte == ord("i"):
        end = data.find(b"e", offset)
        if end < 0:
            raise Malformed("Unterminated integer")
        text = data[offset + 1 : end]
        if (
            not text
            or (text.startswith(b"-") and not text[1:].isdigit())
            or (not text.startswith(b"-") and not text.isdigit())
        ):
            raise Malformed("Invalid integer")
        return int(text), end + 1
    if byte == ord("l"):
        items, offset = [], offset + 1
        while data[offset : offset + 1] != b"e":
            value, offset = decode(data, offset, depth + 1, state)
            items.append(value)
        return items, offset + 1
    if byte == ord("d"):
        items, offset = {}, offset + 1
        while data[offset : offset + 1] != b"e":
            key, offset = decode(data, offset, depth + 1, state)
            if not isinstance(key, bytes):
                raise Malformed("Dictionary key is not a string")
            value, offset = decode(data, offset, depth + 1, state)
            items[key] = value
        return items, offset + 1
    if 48 <= byte <= 57:
        colon = data.find(b":", offset)
        if colon < 0 or not data[offset:colon].isdigit():
            raise Malformed("Invalid string length")
        length = int(data[offset:colon])
        if colon + 1 + length > len(data):
            raise Malformed("String exceeds file")
        return data[colon + 1 : colon + 1 + length], colon + 1 + length
    raise Malformed(f"Unexpected byte {byte:#x}")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"d") or len(data) < 4 or not (48 <= data[1] <= 57):
        return None
    state = {"items": 0}
    try:
        value, end = decode(data, 0, 0, state)
    except Malformed as error:
        return Observation("fail", str(error), "torrent")
    except (IndexError, RecursionError):
        return Observation("fail", "Truncated or too deeply nested", "torrent")
    if end != len(data):
        return Observation("fail", "Bytes after the bencoded value", "torrent")
    info = value.get(b"info") if isinstance(value, dict) else None
    if not isinstance(info, dict):
        return Observation("fail", "No info dictionary", "torrent")
    pieces = info.get(b"pieces")
    if (
        not isinstance(pieces, bytes)
        or len(pieces) % 20
        or not isinstance(info.get(b"piece length"), int)
        or not isinstance(info.get(b"name"), bytes)
    ):
        if b"file tree" in info and b"meta version" in info:
            return Observation("pass", "BitTorrent v2 metainfo parsed exactly", "torrent", ("v2",))
        return Observation(
            "fail", "info lacks name, piece length or a valid pieces string", "torrent"
        )
    return Observation("pass", f"Metainfo parsed exactly; {len(pieces) // 20} pieces", "torrent")
