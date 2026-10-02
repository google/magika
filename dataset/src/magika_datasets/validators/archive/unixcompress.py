# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Unix compress(1) LZW streams decoded completely with compress(1) bit-group alignment."""

from ..contract import Observation

FAMILY = "archive"
FORMAT_IDS = ("unixcompress",)
SCOPE = "Header bits (9..16 code width, block mode), complete LZW decode within an output budget with code-width and CLEAR group realignment, every code inside the current table; decoded bytes not interpreted"
MAGIC = b"\x1f\x9d"
LIMIT = 64 * 1024 * 1024
CODES = 4_000_000
CLEAR = 256


class Malformed(Exception):
    pass


class Budget(Exception):
    pass


def align(posbits: int, base: int, n_bits: int) -> int:
    """Skip to the next group of eight codes; groups restart at every width change or CLEAR."""
    group = n_bits << 3
    return base + -(-(posbits - base) // group) * group


def decode(data: bytes) -> int:
    """Decode the stream after the 3-byte header; returns the expanded length."""
    maxbits, block_mode = data[2] & 0x1F, bool(data[2] & 0x80)
    if data[2] & 0x60 or not 9 <= maxbits <= 16:
        raise Malformed("Invalid header bits")
    body = data[3:]
    inbits = len(body) * 8
    n_bits, maxcode = 9, (1 << 9) - 1
    maxmaxcode = 1 << maxbits
    free_ent = 257 if block_mode else 256
    prefix = [0] * maxmaxcode
    suffix = bytearray(range(256)) + bytearray(maxmaxcode - 256)
    oldcode, finchar, posbits, base, produced, codes = -1, 0, 0, 0, 0, 0
    while True:
        if free_ent > maxcode:
            posbits = base = align(posbits, base, n_bits)
            n_bits += 1

            maxcode = maxmaxcode if n_bits == maxbits else (1 << n_bits) - 1
        if inbits - posbits < n_bits:
            return produced
        byte = posbits >> 3
        code = (int.from_bytes(body[byte : byte + 3], "little") >> (posbits & 7)) & (
            (1 << n_bits) - 1
        )
        posbits += n_bits
        codes += 1
        if codes > CODES:
            raise Budget("Code budget exceeded")
        if oldcode == -1:
            if code > 255:
                raise Malformed("First code is not a literal")
            oldcode = finchar = code
            produced += 1
            continue
        if code == CLEAR and block_mode:
            posbits = base = align(posbits, base, n_bits)

            n_bits, maxcode, free_ent = 9, (1 << 9) - 1, 256
            continue
        incode = code
        pending = 0
        if code >= free_ent:
            if code > free_ent:
                raise Malformed("Code refers to an entry not yet defined")
            pending = 1  # KwKwK: the entry being defined ends with the previous first char
            code = oldcode
        while code >= 256:
            pending += 1
            code = prefix[code]
        finchar = suffix[code]
        produced += pending + 1
        if produced > LIMIT:
            raise Budget("Expanded output exceeds budget")
        if free_ent < maxmaxcode:
            prefix[free_ent] = oldcode
            suffix[free_ent] = finchar
            free_ent += 1
        oldcode = incode


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 4:
        return Observation("fail", "Stream has no codes", "unixcompress")
    try:
        expanded = decode(data)
    except Malformed as error:
        return Observation("fail", str(error), "unixcompress")
    except Budget as error:
        return Observation("inconclusive", str(error), "unixcompress")
    tags = (f"maxbits_{data[2] & 0x1F}",) + (("block_mode",) if data[2] & 0x80 else ())
    return Observation(
        "pass", f"LZW stream decoded to {expanded} bytes", "unixcompress", tags, generic=True
    )
