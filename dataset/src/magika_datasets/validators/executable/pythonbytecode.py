# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Python .pyc files: magic-derived version and header, marshal stream walked natively."""

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("pythonbytecode",)
PREFIX_ONLY = True  # a two-byte magic plus CRLF also matches pcapng and text; failures need a hint
SCOPE = "Magic number mapped to an interpreter version, header size by version, marshal stream walked with version-specific code object layouts to exactly EOF, top-level object a code object; nothing is imported or executed"
VERSIONS = [
    (60000, 62999, (2, 7)),
    (3000, 3131, (3, 0)), (3132, 3151, (3, 1)), (3152, 3180, (3, 2)), (3181, 3230, (3, 3)),
    (3231, 3310, (3, 4)), (3311, 3351, (3, 5)), (3352, 3379, (3, 6)), (3380, 3394, (3, 7)),
    (3395, 3413, (3, 8)), (3414, 3425, (3, 9)), (3426, 3439, (3, 10)), (3440, 3495, (3, 11)),
    (3496, 3531, (3, 12)), (3532, 3571, (3, 13)), (3572, 3650, (3, 14)),
]  # fmt: skip
OBJECTS = 1_000_000


class Malformed(Exception):
    pass


def version_for(magic: int):
    for low, high, version in VERSIONS:
        if low <= magic <= high:
            return version
    return None


class Walker:
    def __init__(self, data: bytes, version: tuple[int, int]):
        self.data, self.version, self.count = data, version, 0

    def need(self, offset: int, size: int) -> int:
        if offset + size > len(self.data):
            raise Malformed("Marshal object exceeds file")
        return offset + size

    def i32(self, offset: int) -> tuple[int, int]:
        end = self.need(offset, 4)
        return struct.unpack_from("<i", self.data, offset)[0], end

    def sized(self, offset: int, width: int) -> int:
        length_end = self.need(offset, width)
        length = int.from_bytes(self.data[offset:length_end], "little", signed=width == 4)
        if length < 0:
            raise Malformed("Negative marshal length")
        return self.need(length_end, length)

    def code(self, offset: int) -> int:
        major, minor = self.version
        if major == 2:
            leading = 4  # argcount, nlocals, stacksize, flags
        elif minor <= 7:
            leading = 5  # + kwonlyargcount
        elif minor <= 10:
            leading = 6  # + posonlyargcount
        else:
            leading = 5  # nlocals moved into localsplus
        offset = self.need(offset, 4 * leading)
        if major == 3 and minor >= 11:
            for _ in range(
                8
            ):  # code, consts, names, localsplusnames, localspluskinds, filename, name, qualname
                offset = self.walk(offset)
            offset = self.need(offset, 4)
            for _ in range(2):  # linetable, exceptiontable
                offset = self.walk(offset)
            return offset
        for _ in range(8):  # code, consts, names, varnames, freevars, cellvars, filename, name
            offset = self.walk(offset)
        offset = self.need(offset, 4)  # firstlineno
        return self.walk(offset)  # lnotab

    def walk(self, offset: int) -> int:
        self.count += 1
        if self.count > OBJECTS:
            raise Malformed("Object budget exceeded")
        offset = self.need(offset, 1)
        kind = chr(self.data[offset - 1] & 0x7F)
        if kind in "0NFTS.":
            return offset
        if kind in "iR" or kind == "r":
            return self.need(offset, 4)
        if kind == "I":
            return self.need(offset, 8)
        if kind == "l":
            digits, offset = self.i32(offset)
            return self.need(offset, 2 * abs(digits))
        if kind == "g":
            return self.need(offset, 8)
        if kind == "y":
            return self.need(offset, 16)
        if kind == "f":
            return self.sized(offset, 1)
        if kind == "x":
            return self.sized(self.sized(offset, 1), 1)
        if kind in "stuaA":
            return self.sized(offset, 4)
        if kind in "zZ":
            return self.sized(offset, 1)
        if kind in "([<>":
            count, offset = self.i32(offset)
            if count < 0:
                raise Malformed("Negative container length")
            for _ in range(count):
                offset = self.walk(offset)
            return offset
        if kind == ")":
            count = self.data[offset]
            offset += 1
            for _ in range(count):
                offset = self.walk(offset)
            return offset
        if kind == "{":
            while True:
                offset = self.need(offset, 1)
                if self.data[offset - 1] & 0x7F == ord("0"):
                    return offset
                offset = self.walk(self.walk(offset - 1))
        if kind == "c":
            return self.code(offset)
        if kind == ":":  # slice objects, Python 3.13+
            return self.walk(self.walk(self.walk(offset)))
        raise Malformed(f"Unknown marshal type {kind!r}")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 8 or data[2:4] != b"\r\n":
        return None
    magic = struct.unpack_from("<H", data)[0]
    version = version_for(magic)
    if version is None:
        return None  # two bytes plus CRLF is not enough to call arbitrary text a pyc
    header = 8 if version < (3, 3) else 12 if version < (3, 7) else 16
    if len(data) < header + 1:
        return Observation("fail", "Truncated pyc header", "pythonbytecode")
    if data[header] & 0x7F != ord("c"):
        return Observation(
            "fail", "Top-level marshal object is not a code object", "pythonbytecode"
        )
    walker = Walker(data, version)
    try:
        end = walker.walk(header)
    except Malformed as error:
        return Observation("fail", str(error), "pythonbytecode")
    except RecursionError:
        return Observation(
            "inconclusive", "Marshal nesting exceeds recursion limit", "pythonbytecode"
        )
    if end != len(data):
        return Observation("fail", "Trailing bytes after the marshal stream", "pythonbytecode")
    tag = f"python_{version[0]}_{version[1]}"
    return Observation(
        "pass",
        f"{walker.count} marshal objects walked for Python {version[0]}.{version[1]}",
        "pythonbytecode",
        (tag,),
    )
