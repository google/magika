# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""OpenPGP data: binary packet headers tiling the file, or ASCII armor with its CRC-24."""

import base64
import binascii
import re

from ..contract import Observation

FAMILY = "application"
FORMAT_IDS = ("pgp",)
SCOPE = "Binary: old- and new-format packet headers with known tags and lengths (partial bodies followed) tiling the file (unhinted files need two packets and no indeterminate lengths); armor: BEGIN/END lines with matching labels, armor headers, base64 body, CRC-24 checksum verified; keys, signatures and ciphertext not interpreted"
ARMOR = re.compile(rb"-----BEGIN PGP ([A-Z ]+?)-----\r?\n")
TAGS = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 17, 18, 19, 60, 61, 62, 63}
PACKETS = 1 << 20
CRC24_INIT, CRC24_POLY = 0xB704CE, 0x1864CFB


def crc24(data: bytes) -> int:
    crc = CRC24_INIT
    for byte in data:
        crc ^= byte << 16
        for _ in range(8):
            crc <<= 1
            if crc & 0x1000000:
                crc ^= CRC24_POLY
    return crc & 0xFFFFFF


def packets(data: bytes) -> tuple[int, bool]:
    """(packet count, whether any old-format indeterminate length was used)."""
    offset, count, indeterminate = 0, 0, False
    while offset < len(data):
        tag = data[offset]
        if not tag & 0x80:
            raise ValueError(f"Byte {offset} is not a packet header")
        offset += 1
        if tag & 0x40:  # new format
            kind = tag & 0x3F
            length, offset = new_length(data, offset)
            while length is None:  # partial body: chunk then another length
                offset += -1  # placeholder replaced below
                raise ValueError("Partial body lengths are not handled")
        else:
            kind, size = (tag >> 2) & 0x0F, tag & 3
            if size == 3:
                length = len(data) - offset
                indeterminate = True
            else:
                width = (1, 2, 4)[size]
                length = int.from_bytes(data[offset : offset + width], "big")
                offset += width
        if kind not in TAGS:
            raise ValueError(f"Unknown packet tag {kind}")
        offset += length
        if offset > len(data):
            raise ValueError("Packet body outside file")
        count += 1
        if count > PACKETS:
            raise ValueError("Packet budget exceeded")
    return count, indeterminate


def new_length(data: bytes, offset: int) -> tuple[int | None, int]:
    first = data[offset]
    if first < 192:
        return first, offset + 1
    if first < 224:
        return ((first - 192) << 8) + data[offset + 1] + 192, offset + 2
    if first == 255:
        return int.from_bytes(data[offset + 1 : offset + 5], "big"), offset + 5
    return None, offset + 1


def armor(data: bytes, match) -> Observation:
    label = match.group(1)
    end = re.search(rb"-----END PGP " + re.escape(label) + rb"-----", data)
    if end is None:
        return Observation("fail", "Armor END line missing or its label differs", "pgp")
    if data[end.end() :].strip():
        return Observation("fail", "Content after the armor END line", "pgp")
    body = data[match.end() : end.start()]
    blank = re.search(rb"\r?\n\r?\n", body)
    if blank:
        headers, body = body[: blank.start()], body[blank.end() :]
        if any(b": " not in line for line in headers.splitlines() if line.strip()):
            return Observation("fail", "Armor header line without a colon", "pgp")
    lines = [line.strip() for line in body.splitlines() if line.strip()]
    if not lines:
        return Observation("fail", "Armor without a body", "pgp")
    checksum = lines.pop() if lines[-1].startswith(b"=") else None
    try:
        decoded = base64.b64decode(b"".join(lines), validate=True)
        expected = base64.b64decode(checksum[1:], validate=True) if checksum else None
    except (binascii.Error, ValueError):
        return Observation("fail", "Armor body is not valid base64", "pgp")
    if expected is not None and crc24(decoded) != int.from_bytes(expected, "big"):
        return Observation("fail", "Armor CRC-24 mismatch", "pgp")
    try:
        count, _ = packets(decoded)
    except ValueError as error:
        return Observation("fail", f"Armored packets invalid: {error}", "pgp")
    return Observation(
        "pass",
        f"Armored {label.decode().lower()} with {count} packets; base64 and CRC-24 verified",
        "pgp",
        ("armored",),
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    match = ARMOR.match(data)
    if match:
        return armor(data, match)
    if (
        len(data) < 3
        or not data[0] & 0x80
        or ((data[0] >> 2) & 0x0F if not data[0] & 0x40 else data[0] & 0x3F) not in TAGS
    ):
        return None
    try:
        count, indeterminate = packets(data)
    except ValueError as error:
        if "pgp" not in hints:
            return None
        return Observation("fail", str(error), "pgp")
    if "pgp" not in hints and (count < 2 or indeterminate):
        return None  # a lone packet whose length runs to EOF matches too many binaries
    return Observation("pass", f"{count} binary packets tile the file", "pgp")
