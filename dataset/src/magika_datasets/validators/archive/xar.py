# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""xar archives: header, bounded TOC inflation, TOC checksum and heap references."""

import hashlib
import struct
import zlib

from .._shared import decompress, xml
from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("xar",)
SCOPE = "Header sizes, TOC inflated within budget to its declared length, TOC XML parsed safely, TOC checksum digest verified against the heap, every file data and signature range inside the heap with archived-checksum digests verified; file payloads not expanded"
MAGIC = b"xar!"
DIGESTS = {0: None, 1: "sha1", 2: "md5", 3: "sha256", 4: "sha512"}
FILES = 65536


def ranges(toc, heap: bytes) -> tuple[int, int, int]:
    """(range count, verified archived digests, furthest heap byte reached); raises on a bad range."""
    end, count, verified = 0, 0, 0
    for holder in toc.iter():
        if holder.tag not in ("data", "signature", "x-signature", "checksum", "ea"):
            continue
        offset, length = (
            holder.findtext("offset"),
            holder.findtext("length") or holder.findtext("size"),
        )
        if offset is None or length is None:
            continue
        start, size = int(offset), int(length)
        if start < 0 or size < 0 or start + size > len(heap):
            raise ValueError("Heap range outside file")
        end = max(end, start + size)
        count += 1
        if count > FILES:
            raise ValueError("Range budget exceeded")
        # xar writes extended-attribute (ea) digests over the wrong bytes; only data digests are reliable
        archived = holder.find("archived-checksum") if holder.tag == "data" else None
        style = (archived.get("style") or "").lower() if archived is not None else ""
        if style in ("sha1", "md5", "sha256", "sha512"):
            digest = hashlib.new(style, heap[start : start + size]).hexdigest()
            if (archived.text or "").strip().lower() != digest:
                raise ValueError("Archived checksum mismatch for a heap range")
            verified += 1
    return count, verified, end


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 28:
        return Observation("fail", "Truncated header", "xar")
    header_size, version, compressed, uncompressed, algorithm = struct.unpack_from(
        ">HHQQI", data, 4
    )
    if header_size < 28 or version != 1 or header_size + compressed > len(data):
        return Observation("fail", "Header sizes disagree with the file", "xar")
    heap_start = header_size + compressed
    try:
        expanded, rest = decompress.drain(
            zlib.decompressobj(), data[header_size:heap_start], decompress.LIMIT
        )
    except decompress.Budget:
        return Observation("inconclusive", "TOC exceeds the expansion budget", "xar")
    except (decompress.Truncated, zlib.error):
        return Observation("fail", "TOC is not a complete zlib stream", "xar")
    if rest or expanded != uncompressed:
        return Observation("fail", "TOC inflated length differs from the header", "xar")
    stream = zlib.decompressobj()
    toc_bytes = stream.decompress(data[header_size:heap_start])
    try:
        root = xml.parse(toc_bytes)
    except xml.Unsafe:
        return Observation("inconclusive", "TOC XML uses entities or external references", "xar")
    except (xml.Malformed, xml.Unsupported):
        return Observation("fail", "TOC is not well-formed XML", "xar")
    toc = root.find("toc")
    if root.tag != "xar" or toc is None:
        return Observation("fail", "TOC XML lacks the xar/toc root", "xar")
    heap = data[heap_start:]
    try:
        count, verified, end = ranges(toc, heap)
    except ValueError as error:
        return Observation("fail", str(error), "xar")
    tags = []
    checksum = toc.find("checksum")
    digest = DIGESTS.get(algorithm) if algorithm in DIGESTS else None
    if checksum is not None:
        style = (checksum.get("style") or "").lower()
        digest = style if style in ("sha1", "md5", "sha256", "sha512") else digest
        if digest is None:
            return Observation("inconclusive", "Unknown TOC checksum algorithm", "xar")
        offset, size = int(checksum.findtext("offset") or 0), int(checksum.findtext("size") or 0)
        expected = hashlib.new(digest, data[header_size:heap_start]).digest()
        if heap[offset : offset + size] != expected:
            return Observation("fail", "TOC checksum does not match the heap digest", "xar")
        tags.append(f"toc_{digest}")
    if toc.find("signature") is not None or toc.find("x-signature") is not None:
        tags.append("signed")
    if end != len(heap):  # xar leaves dead space behind when entries are rewritten
        tags.append("unreferenced_heap_bytes")
    return Observation(
        "pass",
        f"TOC inflated to {uncompressed} bytes and digest verified; {count} heap ranges in bounds, {verified} archived checksums verified",
        "xar",
        tuple(tags),
    )
