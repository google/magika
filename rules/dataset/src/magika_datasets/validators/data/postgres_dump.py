# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""PostgreSQL custom-format dumps: archive header, table of contents and data blocks tiling the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("postgres_dump",)
SCOPE = "PGDMP header (format version 1.12 to 1.16, custom format), integer and offset sizes, table of contents entries with their string and dependency fields, every data block chain (BLK_DATA and BLK_BLOBS) tiling the file to EOF, TOC data offsets pointing at their blocks; SQL and data not interpreted"
MAGIC = b"PGDMP"
ENTRIES = 1 << 20


class Malformed(Exception):
    pass


class Reader:
    def __init__(self, data: bytes, int_size: int, offset_size: int):
        self.data, self.position, self.int_size, self.offset_size = data, 11, int_size, offset_size

    def byte(self) -> int:
        if self.position >= len(self.data):
            raise Malformed("Unexpected end of dump")
        value = self.data[self.position]
        self.position += 1
        return value

    def integer(self) -> int:
        sign = self.byte()
        value = 0
        for index in range(self.int_size):
            value |= self.byte() << (8 * index)
        return -value if sign else value

    def string(self) -> bytes | None:
        length = self.integer()
        if length < 0:
            return None
        if self.position + length > len(self.data):
            raise Malformed("String outside file")
        value = self.data[self.position : self.position + length]
        self.position += length
        return value

    def offset(self) -> tuple[int, int]:
        flag = self.byte()
        value = 0
        for index in range(self.offset_size):
            value |= self.byte() << (8 * index)
        return flag, value


def toc(reader: Reader, version: tuple[int, int]) -> list[tuple[int, int, int]]:
    """(dumpId, hadDumper, data offset) for every entry."""
    count = reader.integer()
    if count < 0 or count > ENTRIES:
        raise Malformed("TOC entry count invalid")
    entries = []
    for _ in range(count):
        dump_id = reader.integer()
        had_dumper = reader.integer()
        reader.string()  # tableoid
        reader.string()  # oid
        reader.string()  # tag
        reader.string()  # desc
        reader.integer()  # section
        reader.string()  # defn
        reader.string()  # dropStmt
        reader.string()  # copyStmt
        reader.string()  # namespace
        reader.string()  # tablespace
        if version >= (1, 14):
            reader.string()  # tableam
        if version >= (1, 16):
            reader.integer()  # relkind
        reader.string()  # owner
        if version < (1, 14):
            reader.string()  # withOids
        for _ in range(ENTRIES):
            if reader.string() is None:
                break
        flag, position = reader.offset()
        entries.append((dump_id, had_dumper, position if flag == 2 else -1))
    return entries


def blocks(reader: Reader) -> dict[int, int]:
    """Walk data blocks to EOF; returns {dumpId: block offset}."""
    found = {}
    while reader.position < len(reader.data):
        start = reader.position
        kind = reader.byte()
        dump_id = reader.integer()
        if kind == 1:  # BLK_DATA: length-prefixed chunks until a zero length
            while True:
                length = reader.integer()
                if length == 0:
                    break
                if length < 0 or reader.position + length > len(reader.data):
                    raise Malformed("Data chunk outside file")
                reader.position += length
        elif kind == 3:  # BLK_BLOBS: (oid, chunks...) until oid 0
            while True:
                oid = reader.integer()
                if oid == 0:
                    break
                while True:
                    length = reader.integer()
                    if length == 0:
                        break
                    if length < 0 or reader.position + length > len(reader.data):
                        raise Malformed("Blob chunk outside file")
                    reader.position += length
        else:
            raise Malformed(f"Unknown block type {kind}")
        found[dump_id] = start
        if len(found) > ENTRIES:
            raise Malformed("Block budget exceeded")
    return found


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 12:
        return Observation("fail", "Truncated header", "postgres_dump")
    major, minor, _, int_size, offset_size, fmt = struct.unpack_from("<BBBBBB", data, 5)
    version = (major, minor)
    if fmt != 1:
        return Observation(
            "inconclusive", f"Archive format {fmt} is not the custom format", "postgres_dump"
        )
    if not (1, 12) <= version <= (1, 16) or int_size not in (4, 8) or offset_size not in (4, 8):
        return Observation(
            "inconclusive", f"Dump version {major}.{minor} layout not modelled", "postgres_dump"
        )
    reader = Reader(data, int_size, offset_size)
    try:
        if version >= (1, 15):
            reader.byte()  # compression algorithm
        else:
            reader.integer()  # compression level
        for _ in range(7):
            reader.integer()  # timestamp fields
        reader.string()  # database name
        reader.string()  # remote version
        reader.string()  # pg_dump version
        entries = toc(reader, version)
        found = blocks(reader)
    except Malformed as error:
        return Observation("fail", str(error), "postgres_dump")
    dumped = [(dump_id, position) for dump_id, had_dumper, position in entries if had_dumper]
    for dump_id, position in dumped:
        if position >= 0 and found.get(dump_id) != position:
            return Observation(
                "fail",
                f"TOC entry {dump_id} points at offset {position}, block found at {found.get(dump_id)}",
                "postgres_dump",
            )
    if len(found) != len(dumped):
        return Observation(
            "fail", f"{len(found)} data blocks for {len(dumped)} dumped entries", "postgres_dump"
        )
    return Observation(
        "pass",
        f"pg_dump {major}.{minor} custom format: {len(entries)} TOC entries, {len(found)} data blocks tile the file",
        "postgres_dump",
    )
