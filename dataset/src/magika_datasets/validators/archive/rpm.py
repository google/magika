# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""RPM packages: lead, signature and main header structures, MD5/SHA digests over header and payload."""

import hashlib
import struct

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("rpm",)
SCOPE = "Lead magic and version, signature and main header index tables with entries inside their stores, signature SIZE against header+payload, MD5 and SHA-1/SHA-256 header digests verified, payload magic against the declared compressor; payload not decompressed"
MAGIC = b"\xed\xab\xee\xdb"
HEADER = b"\x8e\xad\xe8\x01"
PAYLOADS = {
    b"\x1f\x8b": "gzip",
    b"\xfd7zXZ": "xz",
    b"BZh": "bzip2",
    b"\x28\xb5\x2f\xfd": "zstd",
    b"\x5d\x00\x00": "lzma",
}


class Malformed(Exception):
    pass


def header(data: bytes, offset: int, pad: bool) -> tuple[int, dict]:
    """Parse an rpm header section; returns (end offset, {tag: (type, bytes)})."""
    if data[offset : offset + 4] != HEADER:
        raise Malformed("Header magic missing")
    count, size = struct.unpack_from(">II", data, offset + 8)
    if count > 65536 or size > 1 << 26:
        raise Malformed("Header index too large")
    index, store = offset + 16, offset + 16 + count * 16
    end = store + size
    if end > len(data):
        raise Malformed("Header outside file")
    entries = {}
    for _ in range(count):
        tag, kind, position, number = struct.unpack_from(">IIII", data, index)
        index += 16
        if kind == 6 or kind == 8 or kind == 9:  # strings: NUL-terminated within the store
            stop = data.find(b"\0", store + position, end)
            if stop < 0:
                raise Malformed("String entry unterminated")
            entries[tag] = (kind, data[store + position : stop])
            continue
        width = {0: 0, 1: 1, 2: 1, 3: 2, 4: 4, 5: 8, 7: 1}.get(kind)
        if width is None:
            raise Malformed("Unknown entry type")
        if store + position + width * number > end:
            raise Malformed("Entry outside header store")
        entries[tag] = (kind, data[store + position : store + position + width * number])
    if pad:
        end += (-end) % 8
    return end, entries


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 96 + 16:
        return Observation("fail", "Truncated lead", "rpm")
    major, minor, kind, _ = struct.unpack_from(">BBHH", data, 4)
    signature_type = struct.unpack_from(">H", data, 78)[0]
    if major != 3 or kind > 1 or signature_type != 5:
        return Observation("fail", "Lead version, type or signature type unsupported", "rpm")
    tags = ["source" if kind == 1 else "binary"]
    try:
        header_start, signature = header(data, 96, pad=True)
        payload_start, main = header(data, header_start, pad=False)
    except (Malformed, struct.error) as error:
        return Observation("fail", str(error) or "Truncated header", "rpm")
    body = data[header_start:]
    checks = 0
    if 1000 in signature:
        if int.from_bytes(signature[1000][1], "big") != len(body):
            return Observation("fail", "Signature SIZE differs from header plus payload", "rpm")
        checks += 1
    if 1004 in signature:
        if hashlib.md5(body).digest() != signature[1004][1]:
            return Observation("fail", "MD5 of header plus payload mismatch", "rpm")
        checks += 1
    header_bytes = data[header_start:payload_start]
    for tag, algorithm in ((269, "sha1"), (273, "sha256")):
        if tag in signature:
            if hashlib.new(algorithm, header_bytes).hexdigest().encode() != signature[tag][1]:
                return Observation("fail", f"{algorithm} header digest mismatch", "rpm")
            checks += 1
    if 1002 in signature or 1005 in signature or 268 in signature:
        tags.append("signed")
    if 1000 not in signature and 1004 not in signature:
        return Observation("inconclusive", "Signature header carries neither SIZE nor MD5", "rpm")
    payload = data[payload_start:]
    compressor = main.get(1125, (0, b"gzip"))[1].decode("latin-1")
    detected = next((name for magic, name in PAYLOADS.items() if payload.startswith(magic)), None)
    if payload and detected and detected != compressor:
        return Observation(
            "fail", f"Payload is {detected} but the header declares {compressor}", "rpm"
        )
    tags.append(f"payload_{detected or compressor}")
    return Observation(
        "pass",
        f"{len(main)} header tags; {checks} signature checks verified over header and payload",
        "rpm",
        tuple(tags),
    )
