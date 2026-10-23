# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""BSON: documents decoded element by element; one document is bson, several a dump."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("bson", "mongodb_bson")
SCOPE = "Every document's int32 size, NUL-terminated element list and trailing NUL, each element's type (double through decimal128, min and max key), NUL-terminated name and value size inside its document, strings with their length and terminator, booleans 0 or 1, nested documents, arrays and code-with-scope walked the same way; a file of exactly one document is bson, of two or more concatenated documents (as mongodump writes) mongodb_bson; string encodings and values not interpreted"
PREFIX_ONLY = True  # a leading int32 is no evidence against an unhinted file
FIXED = {
    0x01: 8,
    0x06: 0,
    0x07: 12,
    0x09: 8,
    0x0A: 0,
    0x10: 4,
    0x11: 8,
    0x12: 8,
    0x13: 16,
    0xFF: 0,
    0x7F: 0,
}
STRINGS = {0x02, 0x0D, 0x0E}
DOCUMENTS = {0x03, 0x04}
DEPTH, ELEMENTS, DOCUMENTS_LIMIT = 100, 5_000_000, 10_000_000


class Invalid(Exception):
    pass


class Walker:
    def __init__(self, data: bytes):
        self.data, self.elements = data, 0

    def cstring(self, at: int, end: int) -> int:
        stop = self.data.find(b"\0", at, end)
        if stop < 0:
            raise Invalid(f"Unterminated name at offset {at}")
        return stop + 1

    def string(self, at: int, end: int) -> int:
        if at + 4 > end:
            raise Invalid(f"String length truncated at offset {at}")
        (length,) = struct.unpack_from("<i", self.data, at)
        if length < 1 or at + 4 + length > end or self.data[at + 3 + length] != 0:
            raise Invalid(f"String at offset {at} has a bad length or terminator")
        return at + 4 + length

    def document(self, at: int, end: int, depth: int = 0) -> int:
        if depth > DEPTH:
            raise Invalid("Documents nested too deeply")
        if at + 5 > end:
            raise Invalid(f"Document truncated at offset {at}")
        (size,) = struct.unpack_from("<i", self.data, at)
        stop = at + size
        if size < 5 or stop > end or self.data[stop - 1] != 0:
            raise Invalid(f"Document at offset {at} has a bad size or terminator")
        cursor = at + 4
        while cursor < stop - 1:
            kind = self.data[cursor]
            cursor = self.cstring(cursor + 1, stop - 1)
            self.elements += 1
            if self.elements > ELEMENTS:
                raise OverflowError("Element budget exceeded")
            if kind in FIXED:
                cursor += FIXED[kind]
            elif kind == 0x08:
                if cursor >= stop - 1 or self.data[cursor] > 1:
                    raise Invalid(f"Boolean at offset {cursor} is not 0 or 1")
                cursor += 1
            elif kind in STRINGS:
                cursor = self.string(cursor, stop - 1)
            elif kind in DOCUMENTS:
                cursor = self.document(cursor, stop - 1, depth + 1)
            elif kind == 0x05:
                if cursor + 5 > stop - 1:
                    raise Invalid(f"Binary header truncated at offset {cursor}")
                (length,) = struct.unpack_from("<i", self.data, cursor)
                if length < 0:
                    raise Invalid(f"Negative binary length at offset {cursor}")
                cursor += 5 + length
            elif kind == 0x0B:
                cursor = self.cstring(self.cstring(cursor, stop - 1), stop - 1)
            elif kind == 0x0C:
                cursor = self.string(cursor, stop - 1) + 12
            elif kind == 0x0F:
                if cursor + 4 > stop - 1:
                    raise Invalid(f"Code with scope truncated at offset {cursor}")
                (total,) = struct.unpack_from("<i", self.data, cursor)
                scope = self.string(cursor + 4, stop - 1)
                if self.document(scope, stop - 1, depth + 1) != cursor + total:
                    raise Invalid(f"Code with scope at offset {cursor} has a bad total size")
                cursor += total
            else:
                raise Invalid(f"Unknown element type {kind:#04x} at offset {cursor}")
            if cursor > stop - 1:
                raise Invalid(f"Element value overruns its document at offset {cursor}")
        return stop


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 5:
        return None
    (size,) = struct.unpack_from("<i", data)
    if not 5 <= size <= len(data) or data[size - 1] != 0:
        return None
    hinted = "mongodb_bson" if "mongodb_bson" in hints and "bson" not in hints else "bson"
    walker, offset, documents = Walker(data), 0, 0
    try:
        while offset < len(data):
            offset = walker.document(offset, len(data))
            documents += 1
            if documents > DOCUMENTS_LIMIT:
                return Observation("inconclusive", "Document budget exceeded", hinted)
    except OverflowError as error:
        return Observation("inconclusive", str(error), hinted)
    except Invalid as error:
        return Observation("fail", str(error), "mongodb_bson" if documents else hinted)
    kind = "bson" if documents == 1 else "mongodb_bson"
    return Observation(
        "pass", f"{documents} documents with {walker.elements} elements decoded", kind
    )
