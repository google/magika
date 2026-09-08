# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""TensorFlow Lite flatbuffers: root Model table, vtables, subgraph/operator/tensor tables and buffer data inside the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("tflite",)
SCOPE = "TFL3 identifier, root offset, every visited table's vtable and object bounds, Model fields (version, operator codes, subgraphs, buffers, description, metadata), subgraph tensors and operators, buffer byte vectors inside the file; tensor data not interpreted"
IDENTIFIER = b"TFL3"
TABLES = 1 << 20


class Malformed(Exception):
    pass


class Buffer:
    def __init__(self, data: bytes):
        self.data, self.budget = data, TABLES

    def u32(self, offset: int) -> int:
        if offset < 0 or offset + 4 > len(self.data):
            raise Malformed("Read outside file")
        return struct.unpack_from("<I", self.data, offset)[0]

    def table(self, offset: int) -> tuple[int, list[int]]:
        """(object size, field offsets) for the table at offset."""
        self.budget -= 1
        if self.budget < 0:
            raise Malformed("Table budget exceeded")
        soffset = (
            struct.unpack_from("<i", self.data, offset)[0]
            if 0 <= offset <= len(self.data) - 4
            else None
        )
        if soffset is None:
            raise Malformed("Table outside file")
        vtable = offset - soffset
        if vtable < 0 or vtable + 4 > len(self.data):
            raise Malformed("vtable outside file")
        vsize, osize = struct.unpack_from("<HH", self.data, vtable)
        if (
            osize < 4
            or vsize < 4
            or vsize & 1
            or vtable + vsize > len(self.data)
            or offset + osize > len(self.data)
        ):
            raise Malformed("vtable or object bounds invalid")
        fields = list(struct.unpack_from(f"<{(vsize - 4) // 2}H", self.data, vtable + 4))
        if any(field >= osize for field in fields):
            raise Malformed("Field offset outside its object")
        return osize, fields

    def field(self, offset: int, fields: list[int], index: int) -> int | None:
        if index >= len(fields) or fields[index] == 0:
            return None
        return offset + fields[index]

    def reference(self, position: int) -> int:
        return position + self.u32(position)

    def vector(self, position: int, element: int) -> tuple[int, int]:
        start = self.reference(position)
        length = self.u32(start)
        if start + 4 + length * element > len(self.data):
            raise Malformed("Vector outside file")
        return start + 4, length

    def tables(self, position: int) -> list[int]:
        start, length = self.vector(position, 4)
        return [self.reference(start + 4 * index) for index in range(length)]

    def string(self, position: int) -> None:
        start, length = self.vector(position, 1)
        if start + length >= len(self.data) or self.data[start + length] != 0:
            raise Malformed("String is not NUL-terminated")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 8 or data[4:8] != IDENTIFIER:
        return None
    buffer = Buffer(data)
    try:
        root = buffer.u32(0)
        size, fields = buffer.table(root)
        version_position = buffer.field(root, fields, 0)
        version = buffer.u32(version_position) if version_position else 0
        if version != 3:
            return Observation("fail", f"Model version {version} is not 3", "tflite")
        codes = buffer.field(root, fields, 1)
        operator_codes = buffer.tables(codes) if codes else []
        for code in operator_codes:
            buffer.table(code)
        subgraphs_position = buffer.field(root, fields, 2)
        subgraphs = buffer.tables(subgraphs_position) if subgraphs_position else []
        tensors = operators = 0
        for subgraph in subgraphs:
            _, sfields = buffer.table(subgraph)
            tensors_position = buffer.field(subgraph, sfields, 0)
            for tensor in buffer.tables(tensors_position) if tensors_position else []:
                _, tfields = buffer.table(tensor)
                shape = buffer.field(tensor, tfields, 0)
                if shape:
                    buffer.vector(shape, 4)
                name = buffer.field(tensor, tfields, 3)
                if name:
                    buffer.string(name)
                tensors += 1
            operators_position = buffer.field(subgraph, sfields, 3)
            for operator in buffer.tables(operators_position) if operators_position else []:
                _, ofields = buffer.table(operator)
                for index in (1, 2):
                    io = buffer.field(operator, ofields, index)
                    if io:
                        buffer.vector(io, 4)
                operators += 1
        description = buffer.field(root, fields, 3)
        if description:
            buffer.string(description)
        buffers_position = buffer.field(root, fields, 4)
        buffers = buffer.tables(buffers_position) if buffers_position else []
        payload = 0
        for entry in buffers:
            _, bfields = buffer.table(entry)
            data_position = buffer.field(entry, bfields, 0)
            if data_position:
                payload += buffer.vector(data_position, 1)[1]
        metadata = buffer.field(root, fields, 6)
        for item in buffer.tables(metadata) if metadata else []:
            buffer.table(item)
    except (Malformed, struct.error) as error:
        return Observation("fail", str(error) or "Flatbuffer truncated", "tflite")
    if not subgraphs:
        return Observation("fail", "Model has no subgraphs", "tflite")
    return Observation(
        "pass",
        f"{len(subgraphs)} subgraphs, {tensors} tensors, {operators} operators, {len(buffers)} buffers ({payload} payload bytes); tables and vectors inside the file",
        "tflite",
    )
