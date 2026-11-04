# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""UF2 firmware images: 512-byte blocks with three magics and consistent numbering."""

import struct

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("uf2",)
SCOPE = "Every 512-byte block's start magics and final magic, payload size at most 476, sequential block numbers and total equal to the block count; firmware not interpreted"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"UF2\n" or len(data) < 512 or data[4:8] != b"\x57\x51\x5d\x9e":
        return None
    if len(data) % 512:
        return Observation("fail", "File length is not a multiple of 512", "uf2")
    count = len(data) // 512
    for index in range(count):
        offset = index * 512
        magic0, magic1, _, _, size, number, total, _ = struct.unpack_from("<IIIIIIII", data, offset)
        final = struct.unpack_from("<I", data, offset + 508)[0]
        if magic0 != 0x0A324655 or magic1 != 0x9E5D5157 or final != 0x0AB16F30:
            return Observation("fail", f"Block {index} magic mismatch", "uf2")
        if size > 476 or number != index or total != count:
            return Observation("fail", f"Block {index} size or numbering invalid", "uf2")
    return Observation("pass", f"{count} UF2 blocks verified", "uf2")
