# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Redis RDB snapshots: opcode walk with length-prefixed values to the EOF opcode."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("redis_rdb",)
SCOPE = "REDIS magic and 4-digit version, AUX/SELECTDB/RESIZEDB/EXPIRETIME opcodes, key-value pairs with every string, list, set, hash, zset and ziplist encoding sized by RDB length encoding, EOF opcode followed by the 8-byte checksum for version 5 and later, ending exactly at EOF; checksum not recomputed"
ENTRIES = 10_000_000


class Malformed(Exception):
    pass


class Reader:
    def __init__(self, data: bytes):
        self.data, self.offset, self.entries = data, 0, 0

    def take(self, size: int) -> bytes:
        if self.offset + size > len(self.data):
            raise Malformed("Structure exceeds file")
        chunk = self.data[self.offset : self.offset + size]
        self.offset += size
        return chunk

    def length(self) -> tuple[int, int | None]:
        """RDB length encoding: (length, special encoding or None)."""
        first = self.take(1)[0]
        kind = first >> 6
        if kind == 0:
            return first & 0x3F, None
        if kind == 1:
            return ((first & 0x3F) << 8) | self.take(1)[0], None
        if kind == 2:
            if first == 0x80:
                return struct.unpack(">I", self.take(4))[0], None
            if first == 0x81:
                return struct.unpack(">Q", self.take(8))[0], None
            raise Malformed("Unknown length prefix")
        return 0, first & 0x3F

    def string(self) -> None:
        length, special = self.length()
        if special is None:
            self.take(length)
        elif special in (0, 1, 2):
            self.take(1 << special)
        elif special == 3:  # LZF: compressed length, uncompressed length, bytes
            compressed, _ = self.length()
            self.length()
            self.take(compressed)
        else:
            raise Malformed("Unknown string encoding")

    def value(self, kind: int) -> None:
        if kind == 0:
            self.string()
        elif kind in (
            1,
            2,
            3,
            4,
        ):  # list, set, zset v1, hash: count then elements (pairs for 3 and 4)
            count, _ = self.length()
            for _ in range(count * (2 if kind in (3, 4) else 1)):
                self.string()
        elif kind == 5:  # zset v2: member then 8-byte score
            count, _ = self.length()
            for _ in range(count):
                self.string()
                self.take(8)
        elif kind in (
            9,
            10,
            11,
            12,
            13,
            16,
            17,
            20,
        ):  # single-blob ziplist, intset and listpack encodings
            self.string()
        elif kind == 14:  # quicklist: count of ziplist blobs
            count, _ = self.length()
            for _ in range(count):
                self.string()
        elif kind == 18:  # quicklist v2: count of (container type, blob)
            count, _ = self.length()
            for _ in range(count):
                self.length()
                self.string()
        elif kind in (6, 7, 15, 19, 21):
            raise Unwalked(f"Value type {kind} (module or stream) is not walked")
        else:
            raise Malformed(f"Unknown value type {kind}")


class Unwalked(Exception):
    pass


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"REDIS") or len(data) < 9:
        return None
    if not data[5:9].isdigit():
        return Observation("fail", "Version is not four digits", "redis_rdb")
    version = int(data[5:9])
    reader = Reader(data)
    reader.offset = 9
    keys = 0
    try:
        while True:
            reader.entries += 1
            if reader.entries > ENTRIES:
                return Observation("inconclusive", "Entry budget exceeded", "redis_rdb")
            opcode = reader.take(1)[0]
            if opcode == 0xFF:
                break
            if opcode == 0xFA:
                reader.string()
                reader.string()
            elif opcode == 0xFE:
                reader.length()
            elif opcode == 0xFB:
                reader.length()
                reader.length()
            elif opcode == 0xFD:
                reader.take(4)
            elif opcode == 0xFC:
                reader.take(8)
            elif opcode in (0xF5, 0xF6):  # function payloads
                reader.string()
            elif opcode == 0xF8:  # idle time
                reader.length()
            elif opcode == 0xF9:  # LFU frequency
                reader.take(1)
            elif opcode == 0xF7:  # module aux data: opaque module opcodes follow
                raise Unwalked("Module auxiliary data is not walked")
            else:
                reader.string()  # key
                reader.value(opcode)
                keys += 1
        if version >= 5:
            reader.take(8)
    except Malformed as error:
        return Observation("fail", str(error), "redis_rdb")
    except Unwalked as error:
        return Observation("inconclusive", str(error), "redis_rdb")
    if reader.offset != len(data):
        return Observation("fail", "Bytes after the EOF opcode and checksum", "redis_rdb")
    return Observation("pass", f"RDB v{version}: {keys} keys walked to EOF", "redis_rdb")
