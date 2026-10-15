# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Binary glTF (GLB) containers: header length, JSON and BIN chunks tiling the file."""

import json
import struct

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("gltf",)
SHARED_FORMAT_IDS = ("gltf",)  # JSON .gltf documents are named by the JSON family
SCOPE = "glTF magic, version 2, declared length equal to the file, a first JSON chunk parsing with an asset object, optional BIN chunk, 4-byte aligned chunks tiling the file; accessors not resolved (JSON .gltf files are handled by the JSON family)"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"glTF" or len(data) < 12:
        return None
    version, length = struct.unpack_from("<II", data, 4)
    if version != 2:
        return Observation("fail", f"Unsupported GLB version {version}", "gltf")
    if length != len(data):
        return Observation("fail", "Declared length differs from file", "gltf")
    offset, kinds = 12, []
    while offset < len(data):
        if offset + 8 > len(data):
            return Observation("fail", "Truncated chunk header", "gltf")
        size, kind = struct.unpack_from("<II", data, offset)
        if size % 4 or offset + 8 + size > len(data):
            return Observation("fail", "Chunk size misaligned or outside file", "gltf")
        payload = data[offset + 8 : offset + 8 + size]
        if not kinds:
            if kind != 0x4E4F534A:
                return Observation("fail", "First chunk is not JSON", "gltf")
            try:
                document = json.loads(payload.decode("utf-8"))
            except (UnicodeError, ValueError):
                return Observation("fail", "JSON chunk does not parse", "gltf")
            if not isinstance(document, dict) or "asset" not in document:
                return Observation("fail", "JSON chunk lacks an asset object", "gltf")
        elif kind not in (0x4E4F534A, 0x004E4942):
            return Observation("fail", f"Unknown chunk type {kind:#x}", "gltf")
        kinds.append(kind)
        offset += 8 + size
        if len(kinds) > 64:
            return Observation("fail", "Too many chunks", "gltf")
    return Observation(
        "pass", f"GLB with {len(kinds)} chunks tiling the file; JSON parsed", "gltf", ("binary",)
    )
