# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""PMTiles v3 archives: header sections inside the file and the root directory decoded."""

import gzip
import json
import struct
import zlib

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("pmtiles",)
SCOPE = "127-byte version 3 header: root directory, metadata, leaf directories and tile data sections inside the file, known compression and tile type codes, zoom range and bounds in range; the root directory decompressed (none or gzip) and its varint entries decoded with run lengths, lengths and offsets inside tile data; metadata decompressed to a JSON object; tiles and brotli or zstd internals not decoded"
MAGIC = b"PMTiles\x03"
HEADER = struct.Struct("<8s11QBBBBBBiiiiBii")
NONE, GZIP = 1, 2
COMPRESSIONS = {0, 1, 2, 3, 4}  # unknown, none, gzip, brotli, zstd
TILE_TYPES = {0, 1, 2, 3, 4, 5, 6}  # unknown, mvt, png, jpeg, webp, avif, mlt
ENTRIES = 10_000_000


def inflate(data: bytes, compression: int) -> bytes | None:
    if compression == NONE:
        return data
    if compression == GZIP:
        return gzip.decompress(data)
    return None  # brotli and zstd need libraries the validators do not carry


def varints(data: bytes):
    value, shift = 0, 0
    for byte in data:
        value |= (byte & 0x7F) << shift
        if byte & 0x80:
            shift += 7
            if shift > 63:
                raise ValueError("Varint longer than 64 bits")
            continue
        yield value
        value, shift = 0, 0
    if shift:
        raise ValueError("Directory ends inside a varint")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < HEADER.size:
        return Observation("fail", "Header truncated", "pmtiles")
    (
        _,
        root_at,
        root_len,
        meta_at,
        meta_len,
        leaf_at,
        leaf_len,
        tiles_at,
        tiles_len,
        _,
        _,
        _,
        _,
        internal,
        tile_compression,
        tile_type,
        min_zoom,
        max_zoom,
        min_lon,
        min_lat,
        max_lon,
        max_lat,
        _,
        _,
        _,
    ) = HEADER.unpack_from(data)
    for name, start, length in (
        ("root directory", root_at, root_len),
        ("metadata", meta_at, meta_len),
        ("leaf directories", leaf_at, leaf_len),
        ("tile data", tiles_at, tiles_len),
    ):
        if start < HEADER.size and length or start + length > len(data):
            return Observation("fail", f"{name.capitalize()} outside the file", "pmtiles")
    if (
        internal not in COMPRESSIONS
        or tile_compression not in COMPRESSIONS
        or tile_type not in TILE_TYPES
    ):
        return Observation("fail", "Unknown compression or tile type code", "pmtiles")
    if min_zoom > max_zoom or max_zoom > 32:
        return Observation("fail", "Zoom range invalid", "pmtiles")
    if not (
        -1800000000 <= min_lon <= max_lon <= 1800000000
        and -900000000 <= min_lat <= max_lat <= 900000000
    ):
        return Observation("fail", "Bounds outside longitude and latitude ranges", "pmtiles")
    try:
        root = inflate(data[root_at : root_at + root_len], internal)
        metadata = inflate(data[meta_at : meta_at + meta_len], internal)
    except (OSError, EOFError, zlib.error):
        return Observation("fail", "Directory or metadata does not decompress", "pmtiles")
    if root is None:
        return Observation(
            "inconclusive", "Brotli or zstd internal compression not decoded", "pmtiles"
        )
    try:
        if not isinstance(json.loads(metadata or b"{}"), dict):
            raise ValueError
    except ValueError:
        return Observation("fail", "Metadata is not a JSON object", "pmtiles")
    try:
        values = list(varints(root))
        if not values or values[0] > ENTRIES:
            raise ValueError("Root directory entry count missing or too large")
        count = values[0]
        if len(values) != 1 + 4 * count:
            raise ValueError("Root directory entry fields do not match its count")
        lengths = values[1 + 2 * count : 1 + 3 * count]
        offsets = values[1 + 3 * count :]
        position = 0
        for index, (length, raw) in enumerate(zip(lengths, offsets)):
            if not length:
                raise ValueError(f"Entry {index} has zero length")
            if raw == 0 and not index:
                raise ValueError("First entry has no offset")
            position = position + lengths[index - 1] if raw == 0 else raw - 1
            limit = leaf_len if values[1 + count + index] == 0 else tiles_len
            if position + length > limit:
                raise ValueError(f"Entry {index} points past its section")
    except ValueError as error:
        return Observation("fail", str(error), "pmtiles")
    return Observation(
        "pass", f"Sections bounded; root directory of {count} entries decoded", "pmtiles"
    )
