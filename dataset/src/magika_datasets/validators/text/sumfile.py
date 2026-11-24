# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Go module checksum files (go.sum): module, version and h1: hash per line."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("sum",)
SCOPE = "Every line 'module version h1:base64' with a 44-character base64 hash; hint required"
REQUIRES_HINT = True
CONTEXT_REQUIRED = True
LINE = re.compile(rb"^\S+ \S+ h1:[A-Za-z0-9+/]{43}=$")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if b" h1:" not in data[:4096]:
        return None
    lines = [line for line in data.splitlines() if line.strip()]
    for number, line in enumerate(lines, 1):
        if not LINE.match(line.strip()):
            return Observation("fail", f"Line {number} is not a go.sum entry", "sum")
    return Observation("pass", f"{len(lines)} go.sum entries", "sum")
