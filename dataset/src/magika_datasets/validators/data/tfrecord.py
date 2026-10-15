# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""TFRecord files: length-prefixed records with masked CRC-32C checks, tiling the file."""

import struct

from .._shared.crc32c import crc32c
from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("tfrecord",)
SCOPE = "Every record's 64-bit length with its masked CRC-32C, then the payload with its masked CRC-32C, records tiling the file exactly; compressed (GZIP/ZLIB) TFRecord files and payload protobufs not interpreted"
RECORDS = 1_000_000


def masked(data: bytes) -> int:
    value = crc32c(data)
    return (((value >> 15) | (value << 17)) + 0xA282EAD8) & 0xFFFFFFFF


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 16 or struct.unpack_from("<I", data, 8)[0] != masked(data[:8]):
        return None  # the first length's CRC is a 32-bit check: without it this is not a TFRecord
    offset, records, payload = 0, 0, 0
    while offset < len(data):
        if offset + 12 > len(data):
            return Observation("fail", f"Record {records + 1} header truncated", "tfrecord")
        (length,) = struct.unpack_from("<Q", data, offset)
        if struct.unpack_from("<I", data, offset + 8)[0] != masked(data[offset : offset + 8]):
            return Observation("fail", f"Record {records + 1} length CRC-32C mismatch", "tfrecord")
        start, end = offset + 12, offset + 12 + length
        if end + 4 > len(data):
            return Observation("fail", f"Record {records + 1} extends past the end", "tfrecord")
        if struct.unpack_from("<I", data, end)[0] != masked(data[start:end]):
            return Observation("fail", f"Record {records + 1} data CRC-32C mismatch", "tfrecord")
        offset = end + 4
        payload += length
        records += 1
        if records > RECORDS:
            return Observation("inconclusive", "Record budget exceeded", "tfrecord")
    return Observation(
        "pass", f"{records} records ({payload} payload bytes) with CRC-32C checks", "tfrecord"
    )
