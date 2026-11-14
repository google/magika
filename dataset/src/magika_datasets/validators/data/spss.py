# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""SPSS system files (.sav): dictionary records, then uncompressed or bytecode-compressed cases."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("spss",)
SCOPE = "$FL2 header, variable, value label, document and extension records to the 999 terminator, case data sized from the case and variable counts (uncompressed) or walked through the bytecode command stream (compression 1) to exactly EOF; zsav (compression 2) inconclusive"
RECORDS = 1_000_000


class Malformed(Exception):
    pass


class Reader:
    def __init__(self, data: bytes, order: str):
        self.data, self.order, self.offset = data, order, 0

    def take(self, size: int) -> bytes:
        if self.offset + size > len(self.data):
            raise Malformed("Structure exceeds file")
        chunk = self.data[self.offset : self.offset + size]
        self.offset += size
        return chunk

    def i32(self) -> int:
        return struct.unpack(self.order + "i", self.take(4))[0]


def dictionary(reader: Reader) -> int:
    """Walk dictionary records; returns the number of variable width slots."""
    slots, count = 0, 0
    while True:
        count += 1
        if count > RECORDS:
            raise Malformed("Record budget exceeded")
        kind = reader.i32()
        if kind == 2:
            _, has_label, missing, _ = struct.unpack(
                reader.order + "iiii", reader.take(16)
            )  # type, label flag, missing count, print format
            reader.take(12)  # write format and name
            if has_label:
                length = reader.i32()
                reader.take(length + (-length % 4))
            reader.take(8 * abs(missing))
            slots += 1
        elif kind == 3:
            labels = reader.i32()
            for _ in range(labels):
                reader.take(8)
                length = reader.take(1)[0]
                reader.take(length + (-(length + 1) % 8))
        elif kind == 4:
            reader.take(4 * reader.i32())
        elif kind == 6:
            reader.take(80 * reader.i32())
        elif kind == 7:
            reader.i32()
            size, items = reader.i32(), reader.i32()
            reader.take(size * items)
        elif kind == 999:
            reader.take(4)
            return slots
        else:
            raise Malformed(f"Unknown record type {kind}")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] not in (b"$FL2", b"$FL3"):
        return None
    if len(data) < 176:
        return Observation("fail", "Truncated header", "spss")
    order = "<" if 0 < struct.unpack_from("<i", data, 64)[0] < 100000 else ">"
    layout, _, compression, _, cases, _ = struct.unpack_from(order + "iiiiii", data, 64)
    if layout not in (2, 3):
        return Observation("fail", "Unknown layout code", "spss")
    reader = Reader(data, order)
    reader.offset = 176
    try:
        slots = dictionary(reader)
        if compression == 2:
            return Observation("inconclusive", "zsav zlib-compressed cases not walked", "spss")
        if compression == 0:
            if cases < 0:
                return Observation("inconclusive", "Unknown case count", "spss")
            expected = reader.offset + 8 * slots * cases
            if expected != len(data):
                raise Malformed(f"Case data would end at {expected}, file is {len(data)} bytes")
        else:
            offset, ended = reader.offset, False
            while offset < len(data) and not ended:
                if offset + 8 > len(data):
                    raise Malformed("Truncated command block")
                commands = data[offset : offset + 8]
                offset += 8
                for command in commands:
                    if command == 252:
                        ended = True
                        break
                    if command == 253:
                        if offset + 8 > len(data):
                            raise Malformed("Truncated inline value")
                        offset += 8
            if offset != len(data):
                raise Malformed("Bytes after the end-of-file command")
    except Malformed as error:
        return Observation("fail", str(error), "spss")
    return Observation(
        "pass",
        f"{slots} variable slots and {cases if cases >= 0 else 'unknown'} cases; compression {compression}",
        "spss",
    )
