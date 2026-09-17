# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""netCDF classic and 64-bit offset files: header lists and variable extents."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("netcdf",)
SCOPE = "CDF1/CDF2/CDF5 header: dimension, attribute and variable lists parsed with padded names and typed values, every variable's begin offset plus size (times record count for record variables) inside the file; netCDF-4/HDF5 not decoded"
TYPES = {1: 1, 2: 1, 3: 2, 4: 4, 5: 4, 6: 8, 7: 1, 8: 2, 9: 4, 10: 8, 11: 8}


class Malformed(Exception):
    pass


class Reader:
    def __init__(self, data: bytes, offsets: int):
        self.data, self.offset, self.offsets = data, 0, offsets

    def take(self, size: int) -> bytes:
        if self.offset + size > len(self.data):
            raise Malformed("Header exceeds file")
        chunk = self.data[self.offset : self.offset + size]
        self.offset += size
        return chunk

    def u32(self) -> int:
        return struct.unpack(">I", self.take(4))[0]

    def name(self) -> bytes:
        length = self.u32()
        text = self.take(length)
        self.take(-length % 4)
        return text

    def attributes(self) -> None:
        tag, count = self.u32(), self.u32()
        if tag not in (0, 0x0C) or (tag == 0 and count):
            raise Malformed("Invalid attribute list")
        for _ in range(count):
            self.name()
            kind, elements = self.u32(), self.u32()
            if kind not in TYPES:
                raise Malformed("Unknown attribute type")
            size = TYPES[kind] * elements
            self.take(size + (-size % 4))


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:3] != b"CDF" or len(data) < 8:
        return None
    version = data[3]
    if version not in (1, 2, 5):
        return Observation("fail", f"Unknown netCDF classic version {version}", "netcdf")
    reader = Reader(data, 8 if version in (2, 5) else 4)
    try:
        reader.take(4)
        records = reader.u32()
        tag, count = reader.u32(), reader.u32()
        if tag not in (0, 0x0A):
            raise Malformed("Invalid dimension list")
        dims = []
        for _ in range(count):
            reader.name()
            dims.append(reader.u32())
        reader.attributes()
        tag, count = reader.u32(), reader.u32()
        if tag not in (0, 0x0B):
            raise Malformed("Invalid variable list")
        variables = 0
        for _ in range(count):
            variables += 1
            reader.name()
            rank = reader.u32()
            if rank > 1024:
                raise Malformed("Variable rank out of range")
            ids = [reader.u32() for _ in range(rank)]
            if any(i >= len(dims) for i in ids):
                raise Malformed("Dimension id out of range")
            reader.attributes()
            kind, size = reader.u32(), reader.u32()
            if kind not in TYPES:
                raise Malformed("Unknown variable type")
            begin = struct.unpack(
                ">Q" if reader.offsets == 8 else ">I", reader.take(reader.offsets)
            )[0]
            record = bool(ids) and dims[ids[0]] == 0
            extent = size * (records if record and records != 0xFFFFFFFF else 1)
            if begin + extent > len(data) and not (record and records == 0xFFFFFFFF):
                raise Malformed("Variable data outside file")
    except Malformed as error:
        return Observation("fail", str(error), "netcdf")
    return Observation(
        "pass",
        f"netCDF classic v{version}: {len(dims)} dimensions and {variables} variables bounded",
        "netcdf",
    )
