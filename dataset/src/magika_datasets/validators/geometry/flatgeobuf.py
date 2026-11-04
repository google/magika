# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""FlatGeobuf: header table, packed Hilbert R-tree index and size-prefixed features."""

import struct

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("flatgeobuf",)
SCOPE = "Magic with major version 3, size-prefixed header FlatBuffer (root and vtable inside the buffer), geometry type in range, index size derived from features_count and index_node_size, then size-prefixed Feature FlatBuffers tiling the file with their count matching the header when it declares one; coordinates and properties not decoded"
MAGIC = b"fgb\x03fgb"
NODE = 40  # NodeItem: four doubles and a uint64 offset
FEATURES = 10_000_000
GEOMETRY_TYPES = 18
# Header table field slots (header.fbs)
GEOMETRY_TYPE, FEATURES_COUNT, INDEX_NODE_SIZE = 2, 8, 9


def table(buffer: bytes) -> tuple[int, dict[int, int]]:
    """(table position, field slot to absolute offset) for a FlatBuffer's root table."""
    if len(buffer) < 8:
        raise ValueError("FlatBuffer too short for a root table")
    (root,) = struct.unpack_from("<I", buffer)
    if root + 4 > len(buffer):
        raise ValueError("Root table offset outside the buffer")
    (back,) = struct.unpack_from("<i", buffer, root)
    vtable = root - back
    if vtable < 0 or vtable + 4 > len(buffer):
        raise ValueError("Vtable outside the buffer")
    vsize, tsize = struct.unpack_from("<HH", buffer, vtable)
    if vsize < 4 or vsize % 2 or vtable + vsize > len(buffer) or root + tsize > len(buffer):
        raise ValueError("Vtable or table size outside the buffer")
    fields = {}
    for slot in range((vsize - 4) // 2):
        (offset,) = struct.unpack_from("<H", buffer, vtable + 4 + 2 * slot)
        if offset:
            if offset >= tsize:
                raise ValueError("Field offset outside its table")
            fields[slot] = root + offset
    return root, fields


def index_size(features: int, node_size: int) -> int:
    if node_size == 0 or features == 0:
        return 0
    if node_size < 2:
        raise ValueError("Index node size below 2")
    level, nodes = features, features
    while True:  # the reference calcTreeSize adds a root level even above a single item
        level = -(-level // node_size)
        nodes += level
        if level == 1:
            return nodes * NODE


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    try:
        (size,) = struct.unpack_from("<I", data, 8)
        header = data[12 : 12 + size]
        if len(header) != size:
            raise ValueError("Header extends past the end of the file")
        _, fields = table(header)
        geometry = header[fields[GEOMETRY_TYPE]] if GEOMETRY_TYPE in fields else 0
        if geometry >= GEOMETRY_TYPES:
            raise ValueError(f"Unknown geometry type {geometry}")
        declared = (
            struct.unpack_from("<Q", header, fields[FEATURES_COUNT])[0]
            if FEATURES_COUNT in fields
            else 0
        )
        node_size = (
            struct.unpack_from("<H", header, fields[INDEX_NODE_SIZE])[0]
            if INDEX_NODE_SIZE in fields
            else 16
        )
        offset = 12 + size + index_size(declared, node_size)
        if offset > len(data):
            raise ValueError("Spatial index extends past the end of the file")
        features = 0
        while offset < len(data):
            (length,) = struct.unpack_from("<I", data, offset)
            body = data[offset + 4 : offset + 4 + length]
            if len(body) != length:
                raise ValueError(f"Feature {features} extends past the end of the file")
            table(body)
            features += 1
            if features > FEATURES:
                return Observation("inconclusive", "Feature budget exceeded", "flatgeobuf")
            offset += 4 + length
        if declared and features != declared:
            raise ValueError(f"Header declares {declared} features, file holds {features}")
    except struct.error:
        return Observation("fail", "Truncated size prefix", "flatgeobuf")
    except ValueError as error:
        return Observation("fail", str(error), "flatgeobuf")
    return Observation(
        "pass",
        f"Header, index for node size {node_size} and {features} features tile the file",
        "flatgeobuf",
    )
