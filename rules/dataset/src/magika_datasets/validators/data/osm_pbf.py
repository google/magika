# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""OpenStreetMap PBF: BlobHeader/Blob pairs walked with a minimal protobuf wire reader."""

import struct
import zlib

from .._shared import decompress
from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("osm",)
SHARED_FORMAT_IDS = ("osm",)  # XML .osm documents are named by the XML family
SCOPE = "Big-endian BlobHeader lengths, BlobHeader protobuf fields (type, datasize), Blob raw_size and zlib data bounded-inflated to raw_size, first block OSMHeader, blocks tiling the file; primitive groups not decoded"
BLOBS = 1_000_000


class Malformed(Exception):
    pass


def varint(data: bytes, offset: int, end: int) -> tuple[int, int]:
    value, shift = 0, 0
    for _ in range(10):
        if offset >= end:
            raise Malformed("Truncated varint")
        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        shift += 7
        if not byte & 0x80:
            return value, offset
    raise Malformed("Varint too long")


def fields(data: bytes, start: int, end: int) -> dict:
    """Protobuf wire fields in [start, end): number -> value (int or bytes)."""
    result, offset = {}, start
    while offset < end:
        key, offset = varint(data, offset, end)
        number, wire = key >> 3, key & 7
        if wire == 0:
            value, offset = varint(data, offset, end)
        elif wire == 2:
            length, offset = varint(data, offset, end)
            if offset + length > end:
                raise Malformed("Length-delimited field exceeds message")
            value = data[offset : offset + length]
            offset += length
        elif wire == 1:
            value, offset = data[offset : offset + 8], offset + 8
        elif wire == 5:
            value, offset = data[offset : offset + 4], offset + 4
        else:
            raise Malformed(f"Unsupported wire type {wire}")
        if offset > end:
            raise Malformed("Field exceeds message")
        result[number] = value
    return result


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 16:
        return None
    length = struct.unpack_from(">I", data)[0]
    if length > 64 * 1024 or 4 + length > len(data):
        return None
    try:
        header = fields(data, 4, 4 + length)
    except Malformed:
        return None
    if header.get(1) not in (b"OSMHeader", b"OSMData"):
        return None
    if header.get(1) != b"OSMHeader":
        return Observation("fail", "First block is not OSMHeader", "osm")
    offset, blobs = 0, 0
    try:
        while offset < len(data):
            blobs += 1
            if blobs > BLOBS:
                return Observation("inconclusive", "Blob budget exceeded", "osm")
            if offset + 4 > len(data):
                raise Malformed("Truncated BlobHeader length")
            length = struct.unpack_from(">I", data, offset)[0]
            header = fields(data, offset + 4, offset + 4 + length)
            kind, size = header.get(1), header.get(3)
            if kind not in (b"OSMHeader", b"OSMData") or not isinstance(size, int):
                raise Malformed(f"Blob {blobs} header invalid")
            start = offset + 4 + length
            if start + size > len(data):
                raise Malformed(f"Blob {blobs} exceeds file")
            body = fields(data, start, start + size)
            raw_size = body.get(2)
            if 3 in body:
                expanded, rest = decompress.drain(zlib.decompressobj(), body[3], decompress.LIMIT)
                if rest or (isinstance(raw_size, int) and expanded != raw_size):
                    raise Malformed(f"Blob {blobs} zlib data does not match raw_size")
            elif 1 not in body:
                return Observation(
                    "inconclusive", f"Blob {blobs} uses a compression not decodable here", "osm"
                )
            offset = start + size
    except Malformed as error:
        return Observation("fail", str(error), "osm")
    except decompress.Budget as error:
        return Observation("inconclusive", str(error), "osm")
    except (decompress.Truncated, zlib.error):
        return Observation("fail", "Blob zlib data does not inflate", "osm")
    return Observation("pass", f"{blobs} OSM PBF blobs tiling the file", "osm", ("pbf",))
