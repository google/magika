# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""BAM alignments: BGZF blocks (gzip members with BC extra field) inflated and the BAM header verified."""

import struct
import zlib

from .._shared import decompress
from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("bam",)
SCOPE = "Every BGZF block's gzip header with the BC extra subfield, block sizes tiling the file, complete bounded inflation with CRC-32 and ISIZE per block, BAM header and alignment block extents across concatenated blocks, optional EOF marker; uncompressed BAM walked by header, reference dictionary and alignment block sizes; alignment records not parsed"
MAGIC = b"\x1f\x8b\x08\x04"
EOF_BLOCK = bytes.fromhex("1f8b08040000000000ff0600424302001b0003000000000000000000")
BLOCKS = 1 << 20


def raw(data: bytes) -> Observation:
    """Uncompressed BAM: header, reference dictionary and alignment blocks tiling the file."""
    if len(data) < 12:
        return Observation("fail", "Truncated BAM header", "bam")
    text_length = struct.unpack_from("<I", data, 4)[0]
    position = 8 + text_length
    if position + 4 > len(data):
        return Observation("fail", "Header text outside file", "bam")
    references = struct.unpack_from("<I", data, position)[0]
    position += 4
    if references > 1 << 20:
        return Observation("fail", "Reference count implausible", "bam")
    for _ in range(references):
        if position + 4 > len(data):
            return Observation("fail", "Reference dictionary truncated", "bam")
        name_length = struct.unpack_from("<I", data, position)[0]
        position += 4 + name_length + 4
        if position > len(data):
            return Observation("fail", "Reference entry outside file", "bam")
    records = 0
    while position < len(data):
        if position + 4 > len(data):
            return Observation("fail", "Alignment block size truncated", "bam")
        block = struct.unpack_from("<I", data, position)[0]
        position += 4 + block
        if block < 32 or position > len(data):
            return Observation("fail", f"Alignment record {records} outside file", "bam")
        records += 1
        if records > BLOCKS:
            return Observation("inconclusive", "Record budget exceeded", "bam")
    return Observation(
        "pass",
        f"Uncompressed BAM with {references} references and {records} alignment records tiling the file",
        "bam",
        ("uncompressed",),
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data.startswith(b"BAM\x01"):
        return raw(data)
    if not data.startswith(MAGIC) or len(data) < 28 or data[12:14] != b"BC":
        return None
    offset, blocks, expanded = 0, 0, 0
    payloads = []
    while offset < len(data):
        if data[offset : offset + 4] != MAGIC:
            return Observation("fail", f"Block {blocks} lacks the BGZF gzip header", "bam")
        if offset + 12 > len(data):
            return Observation("fail", "Truncated BGZF header", "bam")
        extra_length = struct.unpack_from("<H", data, offset + 10)[0]
        if offset + 12 + extra_length > len(data):
            return Observation("fail", "Truncated BGZF extra fields", "bam")
        extra = data[offset + 12 : offset + 12 + extra_length]
        position, block_size = 0, None
        while position + 4 <= len(extra):
            sub_id, sub_length = (
                extra[position : position + 2],
                struct.unpack_from("<H", extra, position + 2)[0],
            )
            if position + 4 + sub_length > len(extra):
                return Observation("fail", "Truncated BGZF subfield", "bam")
            if sub_id == b"BC" and sub_length == 2:
                block_size = struct.unpack_from("<H", extra, position + 4)[0] + 1
            position += 4 + sub_length
        if position != len(extra) or block_size is None or block_size < 12 + extra_length + 8:
            return Observation("fail", f"Block {blocks} has no BC subfield", "bam")
        block = data[offset : offset + block_size]
        if len(block) != block_size:
            return Observation("fail", f"Block {blocks} extends past EOF", "bam")
        try:
            size, rest = decompress.drain(zlib.decompressobj(31), block, 1 << 16)
        except decompress.Budget:
            return Observation("fail", "BGZF block inflates beyond 64 KiB", "bam")
        except (decompress.Truncated, zlib.error):
            return Observation("fail", f"Block {blocks} does not inflate cleanly", "bam")
        if rest:
            return Observation("fail", f"Block {blocks} size disagrees with its BC field", "bam")
        expanded += size
        if expanded > decompress.LIMIT:
            return Observation("inconclusive", "Combined BGZF expansion budget exceeded", "bam")
        payloads.append(zlib.decompress(block, 31))
        offset += block_size
        blocks += 1
        if blocks > BLOCKS:
            return Observation("inconclusive", "Block budget exceeded", "bam")
    payload = b"".join(payloads)
    if not payload.startswith(b"BAM\x01"):
        return None  # BGZF is also used by tabix/VCF and other formats.
    result = raw(payload)
    if result.status != "pass":
        return result
    tags = ("eof_marker",) if data.endswith(EOF_BLOCK) else ()
    return Observation(
        "pass",
        f"{blocks} BGZF blocks inflated to {expanded} bytes with CRCs verified; BAM header present",
        "bam",
        tags,
    )
