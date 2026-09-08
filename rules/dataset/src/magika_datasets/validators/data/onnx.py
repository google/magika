# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ONNX models: protobuf wire walk of ModelProto with field-type checks down to graph nodes."""

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("onnx",)
SCOPE = "Protobuf wire framing of the whole file as a ModelProto (ir_version first), wire types of known ModelProto, GraphProto, NodeProto and TensorProto fields, nested messages tiling their length prefixes, graph present with at least one node or initializer; tensor payloads not decoded"
PREFIX_ONLY = True
CONTEXT_REQUIRED = True
VARINT, FIXED64, LENGTH, FIXED32 = 0, 1, 2, 5
PACKED = (VARINT, LENGTH)  # repeated scalars may be packed or not
MODEL = {
    1: (VARINT,),
    2: (LENGTH,),
    3: (LENGTH,),
    4: (LENGTH,),
    5: (VARINT,),
    6: (LENGTH,),
    7: (LENGTH,),
    8: (LENGTH,),
    14: (LENGTH,),
    20: (LENGTH,),
    25: (LENGTH,),
}
GRAPH = {
    1: (LENGTH,),
    2: (LENGTH,),
    5: (LENGTH,),
    10: (LENGTH,),
    11: (LENGTH,),
    12: (LENGTH,),
    13: (LENGTH,),
    14: (LENGTH,),
    15: (LENGTH,),
    16: (LENGTH,),
}
NODE = {
    1: (LENGTH,),
    2: (LENGTH,),
    3: (LENGTH,),
    4: (LENGTH,),
    5: (LENGTH,),
    6: (LENGTH,),
    7: (LENGTH,),
    8: (LENGTH,),
    9: (LENGTH,),
}
TENSOR = {
    1: PACKED,
    2: (VARINT,),
    3: (LENGTH,),
    4: (FIXED32, LENGTH),
    5: PACKED,
    6: (LENGTH,),
    7: PACKED,
    8: (LENGTH,),
    9: (LENGTH,),
    10: (FIXED64, LENGTH),
    11: PACKED,
    12: (LENGTH,),
    13: (LENGTH,),
    14: (VARINT,),
    16: (LENGTH,),
}
ELEMENTS = 1 << 22


class Malformed(Exception):
    pass


class Walker:
    def __init__(self, data: bytes):
        self.data, self.budget = data, ELEMENTS

    def varint(self, position: int, end: int) -> tuple[int, int]:
        value, shift = 0, 0
        while True:
            if position >= end or shift > 63:
                raise Malformed("Varint truncated or too long")
            byte = self.data[position]
            position += 1
            value |= (byte & 0x7F) << shift
            shift += 7
            if not byte & 0x80:
                return value, position

    def fields(self, start: int, end: int):
        """Yield (field, wire type, value or (start, end)) for the message in data[start:end]."""
        position = start
        while position < end:
            self.budget -= 1
            if self.budget < 0:
                raise Malformed("Element budget exceeded")
            key, position = self.varint(position, end)
            field, wire = key >> 3, key & 7
            if field == 0:
                raise Malformed("Field number zero")
            if wire == VARINT:
                value, position = self.varint(position, end)
                yield field, wire, value
            elif wire == FIXED64:
                position += 8
                yield field, wire, None
            elif wire == FIXED32:
                position += 4
                yield field, wire, None
            elif wire == LENGTH:
                length, position = self.varint(position, end)
                if position + length > end:
                    raise Malformed("Length-delimited field outside its message")
                yield field, wire, (position, position + length)
                position += length
            else:
                raise Malformed(f"Unsupported wire type {wire}")
            if position > end:
                raise Malformed("Field outside its message")

    def check(self, start: int, end: int, schema: dict, nested: dict) -> dict:
        counts = {}
        for field, wire, value in self.fields(start, end):
            expected = schema.get(field)
            if expected is not None and wire not in expected:
                raise Malformed(f"Field {field} has wire type {wire}, expected {expected}")
            counts[field] = counts.get(field, 0) + 1
            if field in nested and wire == LENGTH:
                inner_schema, inner_nested = nested[field]
                self.check(value[0], value[1], inner_schema, inner_nested)
        return counts


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 4 or (data[0] != 0x08 and "onnx" not in hints):
        return None
    walker = Walker(data)
    node = (NODE, {})
    tensor = (TENSOR, {})
    graph = (GRAPH, {1: node, 5: tensor})
    try:
        counts = walker.check(0, len(data), MODEL, {7: graph})
    except Malformed as error:
        return Observation("fail", str(error), "onnx")
    if 1 not in counts or 7 not in counts:
        return Observation("fail", "ModelProto lacks ir_version or graph", "onnx")
    graphs = [
        value for field, wire, value in walker.fields(0, len(data)) if field == 7 and wire == LENGTH
    ]
    if not any(
        any(field in (1, 5, 11, 12) for field, _, _ in walker.fields(*graph)) for graph in graphs
    ):
        return Observation(
            "inconclusive", "Graph has no nodes, initializers or graph interface", "onnx"
        )
    return Observation(
        "pass",
        f"ModelProto with {sum(counts.values())} top-level fields; wire framing and field types verified through graph nodes and initializers",
        "onnx",
    )
