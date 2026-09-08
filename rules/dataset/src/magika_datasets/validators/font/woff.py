# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""WOFF and WOFF2 font containers: directory bounds, table inflation and length accounting."""

import struct
import zlib

from .._shared import decompress
from ..contract import Observation
from .sfnt import checksum

FAMILY = "font"
FORMAT_IDS = ("woff", "woff2")
SCOPE = "WOFF: header length, table directory bounds, zlib inflation of every compressed table to its original length and sfnt checksum, metadata and private block coverage. WOFF2: header length, table directory with Base128 lengths, compressed block, metadata and private coverage; Brotli stream not decoded"
KNOWN_TAGS = (
    "cmap head hhea hmtx maxp name OS/2 post cvt  fpgm glyf loca prep CFF  VORG EBDT EBLC gasp "
    "hdmx kern LTSH PCLT VDMX vhea vmtx BASE GDEF GPOS GSUB EBSC JSTF MATH CBDT CBLC COLR CPAL "
    "SVG  sbix acnt avar bdat bloc bsln cvar fdsc feat fmtx fvar gvar hsty just lcar mort morx "
    "opbd prop trak Zapf Silf Glat Gloc Feat Sill"
)
TAGS = [KNOWN_TAGS[i : i + 4].encode() for i in range(0, len(KNOWN_TAGS), 5)]


class Malformed(Exception):
    pass


def padded(length: int) -> int:
    return length + (-length % 4)


def blocks(data: bytes, position: int, header) -> int:
    """Metadata and private blocks must follow in order and tile the file."""
    meta_offset, meta_length, _, priv_offset, priv_length = header
    for offset, length in ((meta_offset, meta_length), (priv_offset, priv_length)):
        if not length:
            continue
        if offset < position or offset > padded(position) or offset + length > len(data):
            raise Malformed("Metadata or private block outside its expected range")
        position = offset + length
    if padded(position) < len(data) or position > len(data):
        raise Malformed("Trailing bytes after last block")
    return position


def woff(data: bytes) -> Observation:
    length, count, _, _, _, _, *header = struct.unpack_from(">IHHIHHIIIII", data, 8)
    if length != len(data):
        raise Malformed("Declared length differs from file")
    if not count or count > 512 or 44 + 20 * count > len(data):
        raise Malformed("Table directory outside file")
    position = 44 + 20 * count
    ranges = []
    expanded_total = 0
    for index in range(count):
        tag, offset, comp, orig, expected = struct.unpack_from(">4sIIII", data, 44 + 20 * index)
        if offset + comp > len(data) or comp > orig:
            raise Malformed(f"Table {tag!r} outside file or longer than original")
        expanded_total += orig
        if expanded_total > decompress.LIMIT:
            raise decompress.Budget("Combined font tables exceed expansion budget")
        chunk = data[offset : offset + comp]
        if comp < orig:
            try:
                expanded, rest = decompress.drain(zlib.decompressobj(), chunk, decompress.LIMIT)
            except (decompress.Truncated, zlib.error) as error:
                raise Malformed(f"Table {tag!r} inflation failed") from error
            if expanded != orig or rest:
                raise Malformed(f"Table {tag!r} inflates to the wrong length")
            chunk = zlib.decompress(chunk)
        if tag == b"head":
            chunk = chunk[:8] + b"\0\0\0\0" + chunk[12:]
        if checksum(chunk) != expected:
            raise Malformed(f"Table checksum mismatch: {tag.decode('latin-1')}")
        ranges.append((offset, comp))
    for offset, comp in sorted(ranges):
        if offset < position or offset > padded(position):
            raise Malformed("Tables overlap or leave unaccounted bytes")
        position = offset + comp
    blocks(data, position, header)
    tags = ("has_metadata",) if header[1] else ()
    return Observation(
        "pass", f"{count} tables inflated and checksummed; blocks cover the file", "woff", tags
    )


def base128(data: bytes, offset: int) -> tuple[int, int]:
    value = 0
    for position in range(offset, min(offset + 5, len(data))):
        value = (value << 7) | (data[position] & 0x7F)
        if not data[position] & 0x80:
            return value, position + 1
    raise Malformed("Invalid Base128 length")


def woff2(data: bytes) -> Observation:
    fields = struct.unpack_from(">IHHIIHHIIIII", data, 8)
    length, count, _, _, compressed_size, _, _, *header = fields
    if length != len(data):
        raise Malformed("Declared length differs from file")
    if not count or count > 512:
        raise Malformed("Table count out of range")
    offset = 48
    for _ in range(count):
        if offset >= len(data):
            raise Malformed("Table directory outside file")
        flags = data[offset]
        offset += 1
        index, transform = flags & 0x3F, flags >> 6
        if index == 63:
            tag, offset = data[offset : offset + 4], offset + 4
        else:
            tag = TAGS[index]
        _, offset = base128(data, offset)
        transformed = (tag in (b"glyf", b"loca") and transform == 0) or (
            tag == b"hmtx" and transform == 1
        )
        if transformed:
            _, offset = base128(data, offset)
    if data[4:8] == b"ttcf":
        _, offset = struct.unpack_from(">I", data, offset)[0], offset + 4
        fonts, offset = base128(data, offset)
        for _ in range(min(fonts, 64)):
            tables, offset = base128(data, offset)
            offset += 4
            for _ in range(tables):
                _, offset = base128(data, offset)
    if offset + compressed_size > len(data):
        raise Malformed("Compressed block outside file")
    blocks(data, offset + compressed_size, header)
    tags = ("has_metadata",) if header[1] else ()
    return Observation(
        "pass",
        f"{count} table entries and compressed block cover the file; Brotli not decoded",
        "woff2",
        tags,
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 48 or data[:4] not in (b"wOFF", b"wOF2"):
        return None
    kind = "woff" if data[:4] == b"wOFF" else "woff2"
    try:
        return woff(data) if kind == "woff" else woff2(data)
    except decompress.Budget as error:
        return Observation("inconclusive", str(error), kind)
    except Malformed as error:
        return Observation("fail", str(error), kind)
    except struct.error:
        return Observation("fail", "Truncated structure", kind)
