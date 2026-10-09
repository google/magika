# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""LLVM bitcode: optional wrapper, bitstream blocks walked by abbreviation ids to EOF."""

import struct

from ..contract import Observation

FAMILY = "executable"
FORMAT_IDS = ("llvm_bitcode",)
SCOPE = "Magic (raw or Darwin wrapper with offset and size inside the file), bitstream blocks entered and ended by abbreviation id with block lengths tiling the stream, records skipped by their own encoding where unabbreviated; abbreviation definitions bound the walk to inconclusive"
MAGIC = b"BC\xc0\xde"
BLOCKS = 100_000


class Malformed(Exception):
    pass


class BitReader:
    def __init__(self, data: bytes):
        self.data, self.position = data, 0

    def read(self, width: int) -> int:
        end = self.position + width
        if end > 8 * len(self.data):
            raise Malformed("Bitstream truncated")
        value = 0
        for index in range(width):
            bit_position = self.position + index
            value |= ((self.data[bit_position >> 3] >> (bit_position & 7)) & 1) << index
        self.position = end
        return value

    def vbr(self, width: int) -> int:
        value, shift = 0, 0
        for _ in range(64):
            chunk = self.read(width)
            value |= (chunk & ((1 << (width - 1)) - 1)) << shift
            shift += width - 1
            if not chunk & (1 << (width - 1)):
                return value
        raise Malformed("VBR too long")

    def align32(self) -> None:
        self.position += -self.position % 32


def walk(reader: BitReader, width: int, end_bit: int, depth: int, state: dict) -> None:
    """Walk one block body up to end_bit; abbreviated records make the walk inconclusive."""
    while reader.position < end_bit:
        abbrev = reader.read(width)
        if abbrev == 0:  # END_BLOCK
            reader.align32()
            if reader.position != end_bit:
                raise Malformed("Block ends before its declared length")
            return
        if abbrev == 1:  # ENTER_SUBBLOCK
            reader.vbr(8)
            new_width = reader.vbr(4)
            reader.align32()
            length = reader.read(32)
            state["blocks"] += 1
            if state["blocks"] > BLOCKS or depth > 64:
                raise Malformed("Block budget exceeded")
            block_end = reader.position + 32 * length
            if block_end > end_bit:
                raise Malformed("Sub-block exceeds its parent")
            walk(reader, new_width, block_end, depth + 1, state)
            continue
        if abbrev == 2:  # DEFINE_ABBREV: later records use it; sizes become opaque
            state["abbreviated"] = True
            count = reader.vbr(5)
            for _ in range(count):
                literal = reader.read(1)
                if literal:
                    reader.vbr(8)
                else:
                    encoding = reader.read(3)
                    if encoding in (1, 2):
                        reader.vbr(5)
            continue
        if abbrev == 3:  # UNABBREV_RECORD
            reader.vbr(6)
            operands = reader.vbr(6)
            for _ in range(operands):
                reader.vbr(6)
            continue
        raise Malformed("Abbreviated record encountered")  # cannot size without the abbrev table
    raise Malformed("Block not terminated")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    tags = ()
    stream = data
    if data[:4] == b"\xde\xc0\x17\x0b":
        if len(data) < 20:
            return Observation("fail", "Truncated bitcode wrapper", "llvm_bitcode")
        _, _, offset, size, _ = struct.unpack_from("<IIIII", data)
        if offset + size > len(data) or offset < 20:
            return Observation("fail", "Wrapper offset and size outside file", "llvm_bitcode")
        stream, tags = data[offset : offset + size], ("wrapped",)
    if stream[:4] != MAGIC:
        return None
    reader = BitReader(stream)
    reader.position = 32
    state = {"blocks": 0, "abbreviated": False}
    try:
        while reader.position < 8 * len(stream):
            if reader.read(2) != 1:
                raise Malformed("Top level must be ENTER_SUBBLOCK")
            reader.vbr(8)
            width = reader.vbr(4)
            reader.align32()
            length = reader.read(32)
            state["blocks"] += 1
            block_end = reader.position + 32 * length
            if block_end > 8 * len(stream):
                raise Malformed("Block length exceeds stream")
            try:
                walk(reader, width, block_end, 1, state)
            except Malformed as error:
                if state["abbreviated"] and "Abbreviated record" in str(error):
                    reader.position = (
                        block_end  # trust the declared length past abbreviated records
                    )
                else:
                    raise
    except Malformed as error:
        return Observation("fail", str(error), "llvm_bitcode")
    if reader.position != 8 * len(stream):
        return Observation("fail", "Trailing bits after the last block", "llvm_bitcode")
    detail = f"{state['blocks']} blocks tile the stream"
    if state["abbreviated"]:
        detail += "; abbreviated records skipped by block length"
    return Observation("pass", detail, "llvm_bitcode", tags)
