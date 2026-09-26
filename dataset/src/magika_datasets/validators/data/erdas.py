# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ERDAS IMAGINE HFA files: header, data dictionary and the node tree it types."""

import re
import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("erdas",)
SCOPE = "EHFA_HEADER_TAG, Ehfa_File record (version, entry header length 128), data dictionary parsed into type definitions up to its terminating period, then every Ehfa_Entry reachable from the root: bounds, parent and previous links consistent, no cycles, printable names, node types defined by the dictionary, data blocks inside the file; raster values and external .ige spill files not interpreted"
MAGIC = b"EHFA_HEADER_TAG\0"
ENTRY = struct.Struct("<6I64s32sI")  # the 128-byte header also carries 4 reserved bytes
HEADER = 128
NODES = 1_000_000
NAME = re.compile(rb"[\x20-\x7e]+")
# Types HFA writers use without restating them in every dictionary.
BUILTIN = {b"root", b"Ehfa_File", b"Ehfa_Entry", b"Ehfa_HeaderTag"}


def dictionary(data: bytes, offset: int) -> set[bytes]:
    """Names of the types the dictionary defines: `{fields}Name,` repeated, then `.`."""
    names, depth, start = set(), 0, offset
    while True:
        if offset >= len(data):
            raise ValueError("Data dictionary is not terminated")
        byte = data[offset]
        if byte == ord("{"):
            depth += 1
        elif byte == ord("}"):
            depth -= 1
            if depth < 0:
                raise ValueError("Unbalanced braces in the data dictionary")
            if depth == 0:
                start = offset + 1
        elif byte == ord(",") and depth == 0:
            if not NAME.fullmatch(data[start:offset]):
                raise ValueError("Unnamed type in the data dictionary")
            names.add(data[start:offset])
            start = offset + 1
        elif byte == ord(".") and depth == 0 and offset == start:
            break
        elif not 0x20 <= byte <= 0x7E and byte not in (0x0A, 0x0D, 0x09):
            raise ValueError("Data dictionary holds non-text bytes")
        offset += 1
    if not names:
        raise ValueError("Data dictionary defines no types")
    return names


def text(field: bytes) -> bytes:
    value = field.split(b"\0", 1)[0]
    if not NAME.fullmatch(value):
        raise ValueError("Entry name or type is not printable text")
    return value


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    try:
        (pointer,) = struct.unpack_from("<I", data, 16)
        version, _, root, length, types_at = struct.unpack_from("<IIIHI", data, pointer)
        if length != HEADER or version != 1:
            raise ValueError("Ehfa_File version or entry header length unexpected")
        types = dictionary(data, types_at) | BUILTIN
        seen, pending, used = set(), [(root, 0, 0)], set()
        while pending:
            offset, parent, previous = pending.pop()
            while offset:
                if offset in seen:
                    raise ValueError("Node tree contains a cycle")
                if offset + HEADER > len(data):
                    raise ValueError("Entry outside file")
                seen.add(offset)
                if len(seen) > NODES:
                    return Observation("inconclusive", "Node budget exceeded", "erdas")
                following, back, up, child, block, size, name, kind, _ = ENTRY.unpack_from(
                    data, offset
                )
                if up != parent or back != previous:
                    raise ValueError("Entry parent or previous link inconsistent")
                text(name)
                kind = text(kind)
                if kind not in types:
                    raise ValueError(f"Node type {kind.decode()} is not in the dictionary")
                used.add(kind)
                if size and block + size > len(data):
                    raise ValueError("Entry data outside file")
                if child:
                    pending.append((child, offset, 0))
                offset, previous = following, offset
    except struct.error:
        return Observation("fail", "Record truncated", "erdas")
    except ValueError as error:
        return Observation("fail", str(error), "erdas")
    return Observation(
        "pass",
        f"{len(seen)} nodes of {len(used)} dictionary types; links and data bounds consistent",
        "erdas",
    )
