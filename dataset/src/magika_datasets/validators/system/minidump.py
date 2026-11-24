# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Windows minidumps: header, stream directory and every stream range inside the file."""

import struct

from ..contract import Observation

FAMILY = "system"
FORMAT_IDS = ("minidump",)
SCOPE = "MDMP signature and version, stream directory inside the file, every stream's RVA and size inside the file; memory contents not interpreted"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data[:4] != b"MDMP" or len(data) < 32:
        return None
    version, streams, directory = struct.unpack_from("<III", data, 4)
    if version & 0xFFFF != 0xA793:
        return Observation("fail", "Unknown minidump version", "minidump")
    if streams > 4096 or directory + 12 * streams > len(data):
        return Observation("fail", "Stream directory outside file", "minidump")
    kinds = set()
    for index in range(streams):
        kind, size, rva = struct.unpack_from("<III", data, directory + 12 * index)
        if size and rva + size > len(data):
            return Observation("fail", f"Stream {index} outside file", "minidump")
        kinds.add(kind)
    tags = tuple(
        name
        for code, name in (
            (3, "has_threads"),
            (4, "has_modules"),
            (9, "has_memory64"),
            (15, "has_misc_info"),
        )
        if code in kinds
    )
    return Observation("pass", f"{streams} streams bounded", "minidump", tags)
