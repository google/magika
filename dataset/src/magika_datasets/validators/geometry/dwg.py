# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""AutoCAD DWG: R13-R2000 section locators with CRC, R2004+ encrypted file header, page map and section map."""

import struct
import zlib

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("dwg",)
SCOPE = "R13-R2000 (AC1012-AC1015): section locator records inside the file, CRC-16 (seed 0xC0C1) over the header, sentinel, sections reaching EOF; R2004+ (AC1018-AC1032): file header decrypted with the magic sequence, its CRC-32, page map system page decompressed with the R2004 LZ77 variant and Adler-32 verified, section map page verified, every page start inside the file; R12 and older only recognised; drawing objects not decoded"
SYSTEM_PAGE = 0x41630E3B  # section page map
SECTION_MAP = 0x4163003B
SENTINEL = b"\x95\xa0\x4e\x28\x99\x82\x1a\xe5\x5e\x41\xe0\x5f\x9d\x3a\x4d\x00"
PAGES = 1 << 20

TABLE = []
for index in range(256):
    value = index
    for _ in range(8):
        value = (value >> 1) ^ 0xA001 if value & 1 else value >> 1
    TABLE.append(value)


class Malformed(Exception):
    pass


def crc16(data: bytes, seed: int) -> int:
    value = seed
    for byte in data:
        value = (value >> 8) ^ TABLE[(value ^ byte) & 0xFF]
    return value


def magic_sequence(count: int) -> bytes:
    seed, out = 1, bytearray()
    for _ in range(count):
        seed = (seed * 0x343FD + 0x269EC3) & 0xFFFFFFFF
        out.append((seed >> 16) & 0xFF)
    return bytes(out)


class Decompressor:
    """The R2004 LZ77 variant (ODA specification 4.5)."""

    def __init__(self, source: bytes):
        self.source, self.position = source, 0

    def byte(self) -> int:
        if self.position >= len(self.source):
            raise Malformed("Compressed page truncated")
        value = self.source[self.position]
        self.position += 1
        return value

    def literal_length(self) -> tuple[int, int]:
        value = self.byte()
        if 1 <= value <= 0x0F:
            return value + 3, 0
        if value == 0:
            total = 0x0F
            value = self.byte()
            while value == 0:
                total += 0xFF
                value = self.byte()
            return total + value + 3, 0
        return 0, value  # not a literal run: the byte is the next opcode

    def long_length(self) -> int:
        total, value = 0, self.byte()
        if value == 0:
            total, value = 0xFF, self.byte()
            while value == 0:
                total += 0xFF
                value = self.byte()
        return total + value

    def two_byte_offset(self) -> tuple[int, int]:
        first, second = self.byte(), self.byte()
        return (first >> 2) | (second << 6), first & 0x03

    def run(self, size: int) -> bytes:
        out = bytearray()
        literal, opcode = self.literal_length()
        out += self.take(literal)
        while self.position < len(self.source) and len(out) < size:
            if opcode == 0:
                opcode = self.byte()
            if opcode >= 0x40:
                length = ((opcode & 0xF0) >> 4) - 1
                offset = (self.byte() << 2) | ((opcode & 0x0C) >> 2)
                literal, opcode = (opcode & 0x03, 0) if opcode & 0x03 else self.literal_length()
            elif 0x21 <= opcode <= 0x3F:
                length = opcode - 0x1E
                offset, literal = self.two_byte_offset()
                literal, opcode = (literal, 0) if literal else self.literal_length()
            elif opcode == 0x20:
                length = self.long_length() + 0x21
                offset, literal = self.two_byte_offset()
                literal, opcode = (literal, 0) if literal else self.literal_length()
            elif 0x12 <= opcode <= 0x1F:
                length = (opcode & 0x0F) + 2
                offset, literal = self.two_byte_offset()
                offset += 0x3FFF
                literal, opcode = (literal, 0) if literal else self.literal_length()
            elif opcode == 0x10:
                length = self.long_length() + 9
                offset, literal = self.two_byte_offset()
                offset += 0x3FFF
                literal, opcode = (literal, 0) if literal else self.literal_length()
            elif opcode == 0x11:
                break
            else:
                raise Malformed(f"Invalid compression opcode {opcode:#x}")
            start = len(out) - offset - 1
            if start < 0:
                raise Malformed("Back-reference before the page start")
            for step in range(length):
                out.append(out[start + step])
            out += self.take(literal)
        if len(out) != size:
            raise Malformed("Decompressed page size differs from its header")
        return bytes(out)

    def take(self, count: int) -> bytes:
        chunk = self.source[self.position : self.position + count]
        if len(chunk) != count:
            raise Malformed("Literal run truncated")
        self.position += count
        return chunk


def system_page(data: bytes, offset: int) -> bytes:
    """Verify and decompress a system section page (page map or section map)."""
    if offset + 20 > len(data):
        raise Malformed("System page header outside file")
    kind, expanded, compressed, method, stored = struct.unpack_from("<IIIII", data, offset)
    if kind not in (SYSTEM_PAGE, SECTION_MAP) or method != 2:
        raise Malformed("System page type or compression method invalid")
    page = data[offset : offset + 20 + compressed]
    if len(page) != 20 + compressed:
        raise Malformed("System page outside file")
    if zlib.adler32(page[:16] + b"\0\0\0\0" + page[20:], 0) != stored:
        raise Malformed("System page checksum mismatch")
    return Decompressor(page[20:]).run(expanded)


def r2004(data: bytes) -> Observation:
    if len(data) < 0x100 + 0x6C:
        raise Malformed("File shorter than the R2004 header")
    header = bytes(a ^ b for a, b in zip(data[0x80 : 0x80 + 0x6C], magic_sequence(0x6C)))
    if header[:12] != b"AcFssFcAJMB\0":
        raise Malformed("Decrypted file header id mismatch")
    if zlib.crc32(header[:0x68] + b"\0\0\0\0") != struct.unpack_from("<I", header, 0x68)[0]:
        raise Malformed("File header CRC-32 mismatch")
    map_id = struct.unpack_from("<I", header, 0x50)[0]
    map_address = struct.unpack_from("<Q", header, 0x54)[0] + 0x100
    section_map_id = struct.unpack_from("<I", header, 0x5C)[0]
    page_map = system_page(data, map_address)
    pages, position, address = {}, 0, 0x100
    while position + 8 <= len(page_map):
        number, size = struct.unpack_from("<iI", page_map, position)
        position += 8
        if number < 0:
            position += 16  # gap: parent, left, right, 0x00
        if address >= len(data):
            raise Malformed("Page starts beyond the end of the file")
        if number > 0:
            pages[number] = (address, size)
        address += size
        if len(pages) > PAGES:
            raise Malformed("Page budget exceeded")
    if map_id not in pages or pages[map_id][0] != map_address:
        raise Malformed("Page map does not list itself at its own address")
    if section_map_id not in pages:
        raise Malformed("Section map page missing from the page map")
    section_map = system_page(data, pages[section_map_id][0])
    count = struct.unpack_from("<I", section_map, 0)[0]
    position, sections = 20, 0
    for _ in range(count):
        if position + 96 > len(section_map):
            raise Malformed("Section map truncated")
        page_count = struct.unpack_from("<I", section_map, position + 8)[0]
        position += 96  # size, page count, max size, unknown, compressed, id, encrypted, name[64]
        for _ in range(page_count):
            if position + 16 > len(section_map):
                raise Malformed("Section page list truncated")
            page = struct.unpack_from("<I", section_map, position)[0]
            if page not in pages:
                raise Malformed("Section refers to a page missing from the page map")
            position += 16
        sections += 1
    return Observation(
        "pass",
        f"{data[:6].decode()}: header CRC, page map ({len(pages)} pages) and section map ({sections} sections) checksums verified",
        "dwg",
        ("r2004_layout",),
    )


def r2000(data: bytes) -> Observation:
    count = struct.unpack_from("<I", data, 0x15)[0]
    end = 0x19 + 9 * count
    if count > 32 or end + 18 > len(data):
        raise Malformed("Section locator table outside file")
    if crc16(data[:end], 0xC0C1) != struct.unpack_from("<H", data, end)[0]:
        raise Malformed("Header CRC-16 mismatch")
    if data[end + 2 : end + 18] != SENTINEL:
        raise Malformed("Header sentinel missing")
    furthest = end + 18
    for index in range(count):
        _, start, size = struct.unpack_from("<BII", data, 0x19 + 9 * index)
        if start + size > len(data):
            raise Malformed("Section outside file")
        furthest = max(furthest, start + size)
    if furthest != len(data):
        raise Malformed(f"{len(data) - furthest} bytes after the last section")
    return Observation(
        "pass",
        f"{data[:6].decode()}: header CRC-16, sentinel and {count} sections reaching EOF verified",
        "dwg",
        ("r2000_layout",),
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 0x20 or data[:2] != b"AC" or not data[2:6].isdigit():
        return None
    version = data[:6]
    try:
        if version >= b"AC1018":
            return r2004(data)
        if version >= b"AC1012":
            return r2000(data)
    except (Malformed, struct.error, UnicodeDecodeError) as error:
        return Observation("fail", str(error) or "Header truncated", "dwg")
    return Observation("inconclusive", f"{version.decode()} layout not modelled", "dwg")
