# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""safetensors files: header JSON, dtype sizes and contiguous data offsets covering the payload."""

import json
import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("safetensors",)
SCOPE = "Header length, JSON header with optional __metadata__, every tensor's dtype, shape product times dtype size equal to its offsets span, offsets sorted and contiguous from zero to the payload length"
SIZES = {
    "F64": 8,
    "F32": 4,
    "F16": 2,
    "BF16": 2,
    "I64": 8,
    "I32": 4,
    "I16": 2,
    "I8": 1,
    "U8": 1,
    "BOOL": 1,
    "U16": 2,
    "U32": 4,
    "U64": 8,
    "F8_E4M3": 1,
    "F8_E5M2": 1,
}


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 16:
        return None
    length = struct.unpack_from("<Q", data)[0]
    if length < 2 or length > 100 * 1024 * 1024 or 8 + length > len(data):
        return None
    header = data[8 : 8 + length]
    if not header.lstrip().startswith(b"{"):
        return None
    try:
        parsed = json.loads(header.decode("utf-8"))
    except (UnicodeError, ValueError):
        return None  # a little-endian length followed by bytes is too weak to call this a failure
    if not isinstance(parsed, dict) or not parsed:
        return Observation("fail", "Header is not a non-empty object", "safetensors")
    payload = len(data) - 8 - length
    spans = []
    for name, entry in parsed.items():
        if name == "__metadata__":
            if not isinstance(entry, dict):
                return Observation("fail", "__metadata__ is not an object", "safetensors")
            continue
        try:
            dtype, shape, (start, end) = entry["dtype"], entry["shape"], entry["data_offsets"]
        except (KeyError, TypeError, ValueError):
            return Observation(
                "fail", f"Tensor {name!r} lacks dtype, shape or data_offsets", "safetensors"
            )
        if dtype not in SIZES:
            return Observation(
                "fail", f"Tensor {name!r} has unknown dtype {dtype!r}", "safetensors"
            )
        count = 1
        for dim in shape:
            if not isinstance(dim, int) or dim < 0:
                return Observation("fail", f"Tensor {name!r} has an invalid shape", "safetensors")
            count *= dim
        if (
            not isinstance(start, int)
            or not isinstance(end, int)
            or end - start != count * SIZES[dtype]
        ):
            return Observation(
                "fail", f"Tensor {name!r} offsets do not match its shape", "safetensors"
            )
        spans.append((start, end))
    spans.sort()
    position = 0
    for start, end in spans:
        if start != position:
            return Observation("fail", "Tensor offsets are not contiguous from zero", "safetensors")
        position = end
    if position != payload:
        return Observation(
            "fail", f"Tensors cover {position} bytes; payload is {payload}", "safetensors"
        )
    return Observation("pass", f"{len(spans)} tensors covering the payload exactly", "safetensors")
