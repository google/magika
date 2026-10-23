# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Dalvik optimized DEX (odex) containers: section layout, embedded DEX and Adler-32 over the tail."""

import struct
import zlib

from ..contract import Observation
from . import dex

FAMILY = "executable"
FORMAT_IDS = ("odex",)
SCOPE = "dey magic and version, dex/deps/opt sections in order and tiling the file, embedded DEX validated (Adler-32 and tables; its SHA-1 is stale after dexopt), Adler-32 over the deps and opt sections; bytecode not interpreted"
MAGIC = b"dey\n"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC):
        return None
    if len(data) < 40:
        return Observation("fail", "Truncated header", "odex")
    version = data[4:8]
    if not (version[:3].isdigit() and version[3] == 0):
        return Observation("fail", "Invalid odex version", "odex")
    dex_offset, dex_length, deps_offset, deps_length, opt_offset, opt_length, _, checksum = (
        struct.unpack_from("<8I", data, 8)
    )
    if (
        dex_offset != 40
        or dex_offset + dex_length > deps_offset
        or deps_offset + deps_length > opt_offset
        or opt_offset + opt_length != len(data)
    ):
        return Observation("fail", "Sections do not tile the file in dex, deps, opt order", "odex")
    if zlib.adler32(data[deps_offset:]) != checksum:
        return Observation("fail", "Adler-32 over deps and opt sections mismatch", "odex")
    inner = dex.check(data[dex_offset : dex_offset + dex_length], signature=False)
    if inner is None or inner.status != "pass":
        return Observation(
            "fail", f"Embedded DEX: {inner.detail if inner else 'magic missing'}", "odex"
        )
    return Observation(
        "pass",
        f"odex {version[:3].decode()} wrapping a {dex_length}-byte DEX; embedded DEX verified, tail Adler-32 verified",
        "odex",
        tuple(tag for tag in inner.tags if tag.startswith("dex_")),
    )
