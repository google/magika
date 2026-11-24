# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""GGUF model files: metadata key/values, tensor infos and aligned tensor data ranges."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("gguf",)
SCOPE = "GGUF magic and version 2 or 3, every metadata value sized by type including arrays, tensor infos with dims and quantization types, data section alignment, every tensor's block-sized range inside the file and the last ending at EOF within alignment; weights not interpreted"
SCALARS = {0: 1, 1: 1, 2: 2, 3: 2, 4: 4, 5: 4, 6: 4, 7: 1, 10: 8, 11: 8, 12: 8}
BLOCKS = {
    0: (1, 4),
    1: (1, 2),
    2: (32, 18),
    3: (32, 20),
    6: (32, 22),
    7: (32, 24),
    8: (32, 34),
    9: (32, 36),
    10: (256, 84),
    11: (256, 110),
    12: (256, 144),
    13: (256, 176),
    14: (256, 210),
    15: (256, 292),
    16: (256, 66),
    17: (256, 74),
    18: (256, 98),
    19: (256, 50),
    20: (32, 18),
    21: (256, 110),
    22: (256, 82),
    23: (256, 136),
    24: (1, 1),
    25: (1, 2),
    26: (1, 4),
    27: (1, 8),
    28: (1, 8),
    29: (256, 56),
    30: (1, 2),
}
ITEMS = 10_000_000


class Malformed(Exception):
    pass


class Reader:
    def __init__(self, data: bytes):
        self.data, self.offset, self.items = data, 0, 0

    def take(self, size: int) -> bytes:
        if self.offset + size > len(self.data):
            raise Malformed("Structure exceeds file")
        chunk = self.data[self.offset : self.offset + size]
        self.offset += size
        return chunk

    def u32(self) -> int:
        return struct.unpack("<I", self.take(4))[0]

    def u64(self) -> int:
        return struct.unpack("<Q", self.take(8))[0]

    def string(self) -> bytes:
        return self.take(self.u64())

    def value(self, kind: int):
        self.items += 1
        if self.items > ITEMS:
            raise Malformed("Metadata budget exceeded")
        if kind in SCALARS:
            return self.take(SCALARS[kind])
        if kind == 8:
            return self.string()
        if kind == 9:
            element, count = self.u32(), self.u64()
            if element in SCALARS:
                self.items += count
                if self.items > ITEMS:
                    raise Malformed("Metadata budget exceeded")
                return self.take(SCALARS[element] * count)
            for _ in range(count):
                self.value(element)
            return None
        raise Malformed(f"Unknown metadata type {kind}")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"GGUF" or len(data) < 24:
        return None
    reader = Reader(data)
    try:
        reader.take(4)
        version = reader.u32()
        if version not in (2, 3):
            raise Malformed(f"Unsupported GGUF version {version}")
        tensors, kvs = reader.u64(), reader.u64()
        if tensors > 65536 or kvs > 1_000_000:
            raise Malformed("Tensor or metadata count out of range")
        alignment, architecture = 32, None
        for _ in range(kvs):
            key = reader.string()
            kind = reader.u32()
            value = reader.value(kind)
            if key == b"general.alignment" and kind == 4:
                alignment = struct.unpack("<I", value)[0]
            if key == b"general.architecture" and kind == 8:
                architecture = value.decode("ascii", "replace")
        if not alignment or alignment & (alignment - 1):
            raise Malformed("Alignment is not a power of two")
        infos = []
        for _ in range(tensors):
            name = reader.string()
            dims = reader.u32()
            if dims > 8:
                raise Malformed(f"Tensor {name!r} has too many dimensions")
            shape = [reader.u64() for _ in range(dims)]
            kind = reader.u32()
            offset = reader.u64()
            if kind not in BLOCKS:
                return Observation("inconclusive", f"Unknown tensor type {kind}", "gguf")
            block, size = BLOCKS[kind]
            count = 1
            for dim in shape:
                count *= dim
            if count % block:
                raise Malformed(f"Tensor {name!r} element count is not block aligned")
            infos.append((offset, count // block * size))
        start = reader.offset + (-reader.offset % alignment)
        end = start
        for offset, size in sorted(infos):
            if start + offset + size > len(data):
                raise Malformed("Tensor data outside file")
            end = max(end, start + offset + size)
        if len(data) - end >= alignment:
            raise Malformed("Bytes after the last tensor exceed alignment padding")
    except Malformed as error:
        return Observation("fail", str(error), "gguf")
    tags = (f"arch_{architecture}",) if architecture else ()
    return Observation(
        "pass", f"GGUF v{version}: {kvs} metadata items and {tensors} tensors bounded", "gguf", tags
    )
