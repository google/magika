# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Advanced Systems Format: header object, its sub-objects, the data object and indexes."""

import struct
import uuid

from ..contract import Observation

FAMILY = "media"
FORMAT_IDS = ("asf",)
SCOPE = "Header object GUID and size, sub-object count and bounds, file properties present (stream properties tagged when absent), data object and trailing index objects tiling the file; packets not decoded"
HEADER = "75B22630-668E-11CF-A6D9-00AA0062CE6C"
DATA = "75B22636-668E-11CF-A6D9-00AA0062CE6C"
FILE_PROPERTIES = "8CABDCA1-A947-11CF-8EE4-00C00C205365"
STREAM_PROPERTIES = "B7DC0791-A9B7-11CF-8EE6-00C00C205365"
OBJECTS = 4096


def objects(data: bytes, start: int, end: int):
    offset, count = start, 0
    while offset < end:
        count += 1
        if count > OBJECTS:
            raise ValueError("Object budget exceeded")
        if offset + 24 > end:
            raise ValueError("Truncated object header")
        kind = uuid.UUID(bytes_le=data[offset : offset + 16])
        size = struct.unpack_from("<Q", data, offset + 16)[0]
        if size < 24 or offset + size > end:
            raise ValueError("Object size outside its container")
        yield kind, offset, size
        offset += size


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 30 or data[:16] != uuid.UUID(HEADER).bytes_le:
        return None
    try:
        walked = list(objects(data, 0, len(data)))
        _, offset, size = walked[0]
        count = struct.unpack_from("<I", data, offset + 24)[0]
        subs = list(objects(data, offset + 30, offset + size))
        if len(subs) != count:
            raise ValueError("Header sub-object count differs from declared")
        kinds = {str(k).upper() for k, _, _ in subs}
        if FILE_PROPERTIES not in kinds:
            raise ValueError("Missing file properties object")
        if len(walked) < 2 or str(walked[1][0]).upper() != DATA:
            raise ValueError("Data object does not follow the header")
    except ValueError as error:
        status = "inconclusive" if "budget" in str(error) else "fail"
        return Observation(status, str(error), "asf")
    tags = () if STREAM_PROPERTIES in kinds else ("no_stream_properties",)
    return Observation(
        "pass",
        f"{len(walked)} top-level ASF objects and {count} header objects bounded",
        "asf",
        tags,
    )
