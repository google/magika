# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Precompiled Lua chunks (5.1 to 5.4): header sizes and every function prototype walked to EOF."""

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("luabytecode",)
SCOPE = "LuaJIT dumps (flags, chunk name and prototype sizes to the terminator) or Lua signature and version 5.1 to 5.4, header size fields, main function and nested prototypes with code, constants, upvalues and debug tables walked exactly to EOF; nothing executed"
PROTOTYPES = 100_000


class Malformed(Exception):
    pass


class Reader:
    def __init__(self, data: bytes, version: int):
        self.data, self.version, self.offset, self.count = data, version, 0, 0
        self.order = "little"
        self.int_size = self.size_t = 4
        self.instruction = 4
        self.number = 8
        self.integer = 8

    def take(self, size: int) -> bytes:
        if self.offset + size > len(self.data):
            raise Malformed("Structure exceeds file")
        chunk = self.data[self.offset : self.offset + size]
        self.offset += size
        return chunk

    def byte(self) -> int:
        return self.take(1)[0]

    def int(self) -> int:
        if self.version >= 0x54:
            return self.varint()
        return int.from_bytes(self.take(self.int_size), self.order, signed=True)

    def varint(self) -> int:
        value = 0
        for _ in range(10):
            byte = self.byte()
            value = (value << 7) | (byte & 0x7F)
            if byte & 0x80:
                return value
        raise Malformed("Varint too long")

    def size(self) -> int:
        if self.version >= 0x54:
            return self.varint()
        return int.from_bytes(self.take(self.size_t), self.order)

    def string(self) -> None:
        if self.version == 0x51 or self.version == 0x52:
            self.take(self.size())
        elif self.version == 0x53:
            length = self.byte()
            if length == 0xFF:
                length = int.from_bytes(self.take(self.size_t), self.order)
            self.take(max(length - 1, 0))
        else:
            length = self.varint()
            self.take(max(length - 1, 0))

    def constant(self) -> None:
        kind = self.byte()
        if kind in (0, 0x11):
            return
        if kind == 1:
            if self.version < 0x54:
                self.take(1)  # boolean payload byte before 5.4
            return
        if kind in (3, 0x13):
            self.take(self.number if kind == 3 else self.integer)
        elif kind in (4, 0x14):
            self.string()
        else:
            raise Malformed(f"Unknown constant type {kind}")

    def debug(self) -> None:
        v = self.version
        if v == 0x54:
            self.take(self.int())  # lineinfo bytes
            for _ in range(self.int()):  # abslineinfo pairs
                self.varint()
                self.varint()
        else:
            self.take(self.int() * self.int_size)  # lineinfo ints
        for _ in range(self.int()):  # locvars
            self.string()
            self.int()
            self.int()
        for _ in range(self.int()):  # upvalue names
            self.string()

    def function(self) -> None:
        self.count += 1
        if self.count > PROTOTYPES:
            raise Malformed("Prototype budget exceeded")
        v = self.version
        if v != 0x52:
            self.string()  # source; 5.2 keeps it in the debug block
        self.int()  # linedefined
        self.int()  # lastlinedefined
        self.take(4 if v == 0x51 else 3)  # (nups,) numparams, is_vararg, maxstacksize
        self.take(self.int() * self.instruction)  # code
        for _ in range(self.int()):
            self.constant()
        if v in (0x51, 0x52):  # nested prototypes follow the constants
            for _ in range(self.int()):
                self.function()
            if v == 0x52:
                for _ in range(self.int()):
                    self.take(2)  # upvalue descriptors
                self.string()  # source
        else:
            for _ in range(self.int()):
                self.take(2 if v == 0x53 else 3)  # upvalue descriptors
            for _ in range(self.int()):
                self.function()
        self.debug()


def uleb128(data: bytes, offset: int) -> tuple[int, int]:
    value, shift = 0, 0
    for _ in range(10):
        if offset >= len(data):
            raise Malformed("Truncated LuaJIT varint")
        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        shift += 7
        if not byte & 0x80:
            return value, offset
    raise Malformed("LuaJIT varint too long")


def luajit(data: bytes) -> Observation:
    version = data[3]
    if version not in (1, 2):
        return Observation("fail", f"Unsupported LuaJIT dump version {version}", "luabytecode")
    try:
        flags, offset = uleb128(data, 4)
        if not flags & 0x02:  # not stripped: chunk name follows
            length, offset = uleb128(data, offset)
            offset += length
        protos = 0
        while True:
            size, offset = uleb128(data, offset)
            if size == 0:
                break
            protos += 1
            if protos > PROTOTYPES:
                raise Malformed("Prototype budget exceeded")
            offset += size
            if offset > len(data):
                raise Malformed("LuaJIT prototype exceeds file")
    except Malformed as error:
        return Observation("fail", str(error), "luabytecode")
    if offset != len(data):
        return Observation("fail", "Trailing bytes after the LuaJIT terminator", "luabytecode")
    if not protos:
        return Observation("fail", "LuaJIT dump without prototypes", "luabytecode")
    return Observation(
        "pass", f"{protos} LuaJIT prototypes bounded", "luabytecode", (f"luajit_v{version}",)
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:3] == b"\x1bLJ" and len(data) >= 5:
        return luajit(data)
    if data[:4] != b"\x1bLua" or len(data) < 12:
        return None
    version = data[4]
    if version not in (0x51, 0x52, 0x53, 0x54):
        return Observation("fail", f"Unsupported Lua version {version:#x}", "luabytecode")
    if data[5] != 0:
        return Observation("fail", "Non-official chunk format", "luabytecode")
    reader = Reader(data, version)
    try:
        reader.take(6)
        if version in (0x51, 0x52):
            reader.order = "little" if reader.take(1)[0] else "big"
            reader.int_size, reader.size_t, reader.instruction, reader.number = reader.take(4)
            reader.take(1)  # integral flag
            if version == 0x52:
                reader.take(6)  # LUAC_TAIL
        else:
            if reader.take(6) != b"\x19\x93\r\n\x1a\n":
                raise Malformed("LUAC_DATA mismatch")
            if version == 0x53:
                (
                    reader.int_size,
                    reader.size_t,
                    reader.instruction,
                    reader.integer,
                    reader.number,
                ) = reader.take(5)
            else:
                reader.instruction, reader.integer, reader.number = reader.take(3)
            probe = reader.take(reader.integer)  # LUAC_INT 0x5678 reveals the byte order
            reader.order = "little" if probe[0] == 0x78 else "big"
            reader.take(reader.number)  # LUAC_NUM
            reader.take(1)  # upvalue count of the main closure
        if reader.int_size not in (4, 8) or reader.size_t not in (4, 8) or reader.instruction != 4:
            raise Malformed("Unsupported header sizes")
        reader.function()
    except Malformed as error:
        return Observation("fail", str(error), "luabytecode")
    except RecursionError:
        return Observation(
            "inconclusive", "Prototype nesting exceeds recursion limit", "luabytecode"
        )
    if reader.offset != len(data):
        return Observation("fail", "Trailing bytes after the main function", "luabytecode")
    tag = f"lua_5_{version - 0x50}"
    return Observation(
        "pass",
        f"{reader.count} prototypes walked for Lua 5.{version - 0x50}",
        "luabytecode",
        (tag,),
    )
