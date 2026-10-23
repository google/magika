# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""JPEG XL: boxed container walk; raw codestreams stay inconclusive without a decoder."""

import struct

from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("jxl",)
SCOPE = "Container signature, ftyp brand, box bounds and jxlc/jxlp presence tiling the file; codestream pixels not decoded (no JPEG XL decoder available)"
SIGNATURE = b"\0\0\0\x0cJXL \r\n\x87\n"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:2] == b"\xff\x0a":
        return Observation("inconclusive", "Raw JPEG XL codestream; no decoder available", "jxl")
    if not data.startswith(SIGNATURE):
        return None
    offset, kinds = 12, []
    while offset < len(data):
        if offset + 8 > len(data):
            return Observation("fail", "Truncated box header", "jxl")
        size, kind = struct.unpack_from(">I4s", data, offset)
        header = 8
        if size == 1:
            if offset + 16 > len(data):
                return Observation("fail", "Truncated largesize header", "jxl")
            size, header = struct.unpack_from(">Q", data, offset + 8)[0], 16
        elif size == 0:
            size = len(data) - offset
        if size < header or offset + size > len(data):
            return Observation("fail", "Box size outside file", "jxl")
        kinds.append(kind)
        if len(kinds) > 4096:
            return Observation("inconclusive", "Box budget exceeded", "jxl")
        offset += size
    if not kinds or kinds[0] != b"ftyp" or data[20:24] != b"jxl ":
        return Observation("fail", "Missing ftyp box with jxl brand", "jxl")
    if b"jxlc" not in kinds and b"jxlp" not in kinds:
        return Observation("fail", "No codestream box", "jxl")
    return Observation("pass", f"{len(kinds)} JPEG XL boxes bounded; codestream not decoded", "jxl")
