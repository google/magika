# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Apple UDIF disk images: koly trailer, forks and XML plist tiling the file, blkx chunk tables inside the data fork."""

import base64
import plistlib
import struct
import zlib

from .._shared import xml
from ..contract import Observation

FAMILY = "application"
FORMAT_IDS = ("dmg",)
SCOPE = "koly trailer version and size, data fork, resource fork, XML property list, code signature superblob and trailer tiling the file, plist parsed without external entities, every blkx mish chunk's compressed range inside the data fork, data fork CRC-32 when the trailer carries one; chunk payloads not decompressed"
MAGIC = b"koly"
CHUNKS = 1 << 20


def chunks(mish: bytes, fork_length: int) -> int:
    """Count chunk entries of one blkx resource; raises ValueError when any leaves the data fork."""
    if mish[:4] != b"mish" or len(mish) < 204:
        raise ValueError("blkx resource lacks the mish header")
    count = struct.unpack_from(">I", mish, 200)[0]
    if count > CHUNKS or 204 + 40 * count > len(mish):
        raise ValueError("Chunk table outside the blkx resource")
    for index in range(count):
        kind, _, _, _, offset, length = struct.unpack_from(">IIQQQQ", mish, 204 + 40 * index)
        if kind == 0xFFFFFFFF:
            continue
        if offset + length > fork_length:
            raise ValueError("Chunk lies outside the data fork")
    return count


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 512 or data[-512:-508] != MAGIC:
        return None
    trailer = data[-512:]
    version, header_size = struct.unpack_from(">II", trailer, 4)
    if version != 4 or header_size != 512:
        return Observation("fail", "Unsupported koly version or size", "dmg")
    _, data_offset, data_length, rsrc_offset, rsrc_length = struct.unpack_from(
        ">QQQQQ", trailer, 16
    )
    checksum_type, checksum_size = struct.unpack_from(">II", trailer, 80)
    stored = struct.unpack_from(">I", trailer, 88)[0]
    xml_offset, xml_length = struct.unpack_from(">QQ", trailer, 216)
    signature_offset, signature_length = struct.unpack_from(">QQ", trailer, 296)
    body = len(data) - 512
    ranges = [
        (data_offset, data_length),
        (rsrc_offset, rsrc_length),
        (xml_offset, xml_length),
        (signature_offset, signature_length),
    ]
    if any(offset + length > body for offset, length in ranges):
        return Observation("fail", "A fork or the XML plist lies outside the file", "dmg")
    covered = sum(length for _, length in ranges)
    if covered != body:
        return Observation(
            "fail", f"{body - covered} bytes are not part of any fork or the plist", "dmg"
        )
    if xml_length == 0:
        return Observation("inconclusive", "Image has no XML plist (resource fork only)", "dmg")
    try:
        root = xml.parse(data[xml_offset : xml_offset + xml_length])
    except xml.Unsafe:
        return Observation("inconclusive", "Plist uses entities or external references", "dmg")
    except (xml.Malformed, xml.Unsupported):
        return Observation("fail", "XML plist is not well-formed", "dmg")
    if root.tag != "plist":
        return Observation("fail", "XML section is not a property list", "dmg")
    try:
        plist = plistlib.loads(data[xml_offset : xml_offset + xml_length], fmt=plistlib.FMT_XML)
        blocks = plist["resource-fork"]["blkx"]
        total = sum(
            chunks(
                base64.b64decode(block["Data"])
                if isinstance(block["Data"], str)
                else block["Data"],
                data_length,
            )
            for block in blocks
        )
    except (KeyError, TypeError, ValueError, plistlib.InvalidFileException) as error:
        return Observation("fail", f"blkx table invalid: {error}", "dmg")
    tags = []
    if signature_length:
        if data[signature_offset : signature_offset + 4] != b"\xfa\xde\x0c\xc0":
            return Observation("fail", "Code signature range does not hold a superblob", "dmg")
        tags.append("code_signature")
    if checksum_type == 2 and checksum_size == 32:
        if zlib.crc32(data[data_offset : data_offset + data_length]) != stored:
            return Observation("fail", "Data fork CRC-32 mismatch", "dmg")
        tags.append("data_crc32")
    return Observation(
        "pass",
        f"{len(blocks)} blkx tables with {total} chunks inside the data fork; forks, plist and trailer tile the file",
        "dmg",
        tuple(tags),
    )
