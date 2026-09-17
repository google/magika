# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""CRC-32C (Castagnoli), as storage engines checksum their pages."""

_TABLE = []
for _index in range(256):
    _value = _index
    for _ in range(8):
        _value = (_value >> 1) ^ 0x82F63B78 if _value & 1 else _value >> 1
    _TABLE.append(_value)


def crc32c(data: bytes, value: int = 0) -> int:
    value ^= 0xFFFFFFFF
    table = _TABLE
    for byte in data:
        value = (value >> 8) ^ table[(value ^ byte) & 0xFF]
    return value ^ 0xFFFFFFFF
