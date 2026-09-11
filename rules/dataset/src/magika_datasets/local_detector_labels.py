# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Explicit detector description mappings; these functions do not adjudicate labels."""

import re

from .detector_labels import description_kind, generic_continuation
from .detectors import TRID_NAMES

EXTRA_TRID_NAMES = {
    "avif": {"AV1 Image File Format bitmap", "AV1 Image File Format Sequence bitmap"},
    "zst": {"Zstandard compressed data", "Zstandard compressed data (old/generic)"},
}

EXTRA_FILE_NAMES = {
    "ISO Media, AVIF Image": "avif",
    "ISO Media, AVIF Image Sequence": "avif",
    "Zstandard compressed data (v0.8+), Dictionary ID: None": "zst",
}

SPIRV_DESCRIPTION = re.compile(
    r"Khronos SPIR-V binary, (?:little|big)-endian, version 0x[0-9a-f]{6,8}, "
    r"generator (?:0x[0-9a-f]{6,8}|00000000)"
)

TRID_LINE = re.compile(r"^\s*(\d+(?:\.\d+)?)%\s+\([^\n]*?\)\s+(.+?)\s+\(\d+/\d+/\d+\)\s*$")


def local_file_kind(stdout):
    lines = stdout.splitlines()
    if not lines or any(not generic_continuation(line) for line in lines[1:]):
        return None
    first = lines[0]
    return (
        "spirv"
        if SPIRV_DESCRIPTION.fullmatch(first)
        else EXTRA_FILE_NAMES.get(first) or description_kind(first)
    )


def local_trid_evidence(stdout, formats):
    ranked = []
    for line in stdout.splitlines():
        match = TRID_LINE.fullmatch(line)
        if match:
            probability = float(match[1])
            if not 0 <= probability <= 100:
                return {"status": "malformed", "format_ids": [], "raw": ranked}
            ranked.append({"probability": probability, "file_type": match[2]})
        elif re.match(r"^\s*[\d.+-]+%", line):
            return {"status": "malformed", "format_ids": [], "raw": ranked}
    if not ranked:
        return {"status": "missing", "format_ids": [], "raw": []}
    names = {k: set(v) | EXTRA_TRID_NAMES.get(k, set()) for k, v in TRID_NAMES.items()}
    for kind, values in EXTRA_TRID_NAMES.items():
        names.setdefault(kind, values)
    top = [r for r in ranked if r["probability"] == max(x["probability"] for x in ranked)]
    mapped = [
        {k for k, values in names.items() if k in formats and r["file_type"] in values} for r in top
    ]
    kinds = sorted(set().union(*mapped))
    status = "mapped" if all(mapped) and len(kinds) == 1 else "unmapped"
    return {"status": status, "format_ids": kinds, "raw": ranked}
