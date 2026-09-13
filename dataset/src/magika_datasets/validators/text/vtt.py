# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""WebVTT: header, cue timing lines and NOTE/STYLE/REGION blocks."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("vtt",)
SCOPE = "WEBVTT header line, blocks separated by blank lines that are NOTE, STYLE, REGION or cues whose first or second line is a --> timing line with dotted milliseconds"
TIMING = re.compile(r"^(\d{1,2}:)?\d{2}:\d{2}\.\d{3}\s+-->\s+(\d{1,2}:)?\d{2}:\d{2}\.\d{3}(\s.*)?$")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    head = data.lstrip(b"\xef\xbb\xbf")
    if not head.startswith(b"WEBVTT") or head[6:7] not in (b"", b"\n", b"\r", b" ", b"\t"):
        return None
    try:
        text = head.decode("utf-8")
    except UnicodeDecodeError:
        return Observation("fail", "Not UTF-8 text", "vtt")
    blocks = [block for block in re.split(r"\r?\n\s*\r?\n", text.strip()) if block.strip()]
    cues = 0
    for number, block in enumerate(blocks[1:], 2):
        lines = block.strip().splitlines()
        if lines[0].startswith(("NOTE", "STYLE", "REGION")):
            continue
        if TIMING.match(lines[0].strip()) or (len(lines) > 1 and TIMING.match(lines[1].strip())):
            cues += 1
            continue
        return Observation(
            "fail", f"Block {number} is neither a cue nor a NOTE/STYLE/REGION block", "vtt"
        )
    return Observation("pass", f"{cues} WebVTT cues", "vtt")
