# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Avro object container files: header map, schema, sync markers and block sizes."""

import bz2
import json
import lzma
import zlib

from .._shared import decompress
from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("avro",)
SCOPE = "Obj1 magic, header metadata map with a parseable schema, 16-byte sync marker after the header and every block, block byte sizes tiling the file; null, deflate, bzip2 and xz blocks bounded-inflated; records not decoded"
BLOCKS = 1_000_000


class Malformed(Exception):
    pass


class Budget(Exception):
    pass


def varint(data: bytes, offset: int) -> tuple[int, int]:
    value, shift = 0, 0
    for _ in range(10):
        if offset >= len(data):
            raise Malformed("Truncated varint")
        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        shift += 7
        if not byte & 0x80:
            return (value >> 1) ^ -(value & 1), offset
    raise Malformed("Varint too long")


def blob(data: bytes, offset: int) -> tuple[bytes, int]:
    length, offset = varint(data, offset)
    if length < 0 or offset + length > len(data):
        raise Malformed("Length-prefixed value outside file")
    return data[offset : offset + length], offset + length


def inflate(codec: bytes, payload: bytes) -> None:
    factories = {
        b"deflate": lambda: zlib.decompressobj(-15),
        b"bzip2": bz2.BZ2Decompressor,
        b"xz": lambda: lzma.LZMADecompressor(lzma.FORMAT_XZ),
    }
    factory = factories.get(codec)
    if factory is None:
        return  # snappy and zstandard are not decodable with the standard library
    try:
        _, rest = decompress.drain(factory(), payload, decompress.LIMIT)
    except decompress.Budget as error:
        raise Budget(str(error)) from error
    except (decompress.Truncated, zlib.error, OSError, lzma.LZMAError) as error:
        raise Malformed("Block decompression failed") from error
    if rest:
        raise Malformed("Trailing bytes inside a compressed block")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"Obj\x01"):
        return None
    try:
        offset, metadata = 4, {}
        while True:
            count, offset = varint(data, offset)
            if count == 0:
                break
            if count < 0:
                count, (_, offset) = -count, varint(data, offset)
            for _ in range(count):
                key, offset = blob(data, offset)
                value, offset = blob(data, offset)
                metadata[key] = value
        schema = metadata.get(b"avro.schema")
        if schema is None:
            raise Malformed("Header lacks avro.schema")
        try:
            json.loads(schema.decode("utf-8"))
        except (UnicodeError, ValueError) as error:
            raise Malformed("avro.schema is not valid JSON") from error
        codec = metadata.get(b"avro.codec", b"null")
        if offset + 16 > len(data):
            raise Malformed("Truncated sync marker")
        sync, offset = data[offset : offset + 16], offset + 16
        blocks = 0
        while offset < len(data):
            blocks += 1
            if blocks > BLOCKS:
                return Observation("inconclusive", "Block budget exceeded", "avro")
            count, offset = varint(data, offset)
            payload, offset = blob(data, offset)
            if count < 0:
                raise Malformed("Negative block object count")
            inflate(codec, payload)
            if data[offset : offset + 16] != sync:
                raise Malformed(f"Block {blocks} sync marker mismatch")
            offset += 16
    except Malformed as error:
        return Observation("fail", str(error), "avro")
    except Budget as error:
        return Observation("inconclusive", str(error), "avro")
    return Observation(
        "pass",
        f"{blocks} blocks with sync markers; schema parsed",
        "avro",
        (f"codec_{codec.decode('ascii', 'replace')}",),
    )
