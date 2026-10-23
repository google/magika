# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""NRRD images: header fields and raw or gzip data sized from type and sizes."""

import zlib

from .._shared import decompress
from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("nrrd",)
SCOPE = "NRRD magic and version, header fields to the blank line, type and sizes, raw data equal to the product or gzip data bounded-inflated to it; detached data files and text encodings inconclusive"
TYPES = {
    "signed char": 1,
    "int8": 1,
    "int8_t": 1,
    "uchar": 1,
    "unsigned char": 1,
    "uint8": 1,
    "uint8_t": 1,
    "short": 2,
    "short int": 2,
    "signed short": 2,
    "int16": 2,
    "int16_t": 2,
    "ushort": 2,
    "unsigned short": 2,
    "uint16": 2,
    "uint16_t": 2,
    "int": 4,
    "signed int": 4,
    "int32": 4,
    "int32_t": 4,
    "uint": 4,
    "unsigned int": 4,
    "uint32": 4,
    "uint32_t": 4,
    "longlong": 8,
    "long long": 8,
    "int64": 8,
    "int64_t": 8,
    "ulonglong": 8,
    "uint64": 8,
    "uint64_t": 8,
    "float": 4,
    "double": 8,
}


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"NRRD000"):
        return None
    if data[7:8] not in b"12345":
        return Observation("fail", "Unknown NRRD version", "nrrd")
    end = data.find(b"\n\n")
    if end < 0:
        end = data.find(b"\r\n\r\n")
        if end < 0:
            return Observation("fail", "Header not terminated by a blank line", "nrrd")
        body_start = end + 4
    else:
        body_start = end + 2
    fields = {}
    for line in data[:end].decode("utf-8", "replace").splitlines()[1:]:
        if line.startswith("#") or ":" not in line:
            continue
        key, value = line.split(":", 1)
        fields[key.strip().lower()] = value.strip().lstrip("=").strip()
    try:
        sizes = [int(v) for v in fields["sizes"].split()]
        count = 1
        for size in sizes:
            count *= size
        itemsize = TYPES[fields["type"].lower()]
        encoding = fields.get("encoding", "raw").lower()
    except (KeyError, ValueError):
        return Observation("fail", "Missing or invalid type, sizes or encoding", "nrrd")
    if "data file" in fields or "datafile" in fields:
        return Observation("inconclusive", "Detached data file not available", "nrrd")
    expected = count * itemsize
    payload = data[body_start:]
    if encoding == "raw":
        if len(payload) != expected:
            return Observation(
                "fail", f"Raw data is {len(payload)} bytes; header needs {expected}", "nrrd"
            )
    elif encoding in ("gzip", "gz"):
        try:
            expanded, rest = decompress.drain(zlib.decompressobj(31), payload, decompress.LIMIT)
        except decompress.Budget as error:
            return Observation("inconclusive", str(error), "nrrd")
        except (decompress.Truncated, zlib.error):
            return Observation("fail", "gzip data does not inflate", "nrrd")
        if rest or expanded != expected:
            return Observation("fail", "gzip data inflates to the wrong length", "nrrd")
    else:
        return Observation("inconclusive", f"Encoding {encoding} not sized", "nrrd")
    return Observation(
        "pass", f"{len(sizes)}-D {fields['type']} data sized exactly ({encoding})", "nrrd"
    )
