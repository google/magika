# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""SubRip subtitles: numbered cues with timing lines."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("srt",)
SCOPE = "Cues of an index line, a HH:MM:SS,mmm --> HH:MM:SS,mmm timing line and text separated by blank lines, indices strictly ascending"
TIMING = re.compile(
    r"^\d{1,2}:\d{2}:\d{2}[,.]\d{1,3}\s+-->\s+\d{1,2}:\d{2}:\d{2}[,.]\d{1,3}(\s.*)?$"
)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    head = data.lstrip(b"\xef\xbb\xbf \r\n")
    if not re.match(rb"\d+\r?\n\d{1,2}:\d{2}:\d{2}[,.]\d{1,3}\s+-->", head):
        return None
    try:
        text = head.decode("utf-8")
    except UnicodeDecodeError:
        text = head.decode("latin-1")
    cues = [block for block in re.split(r"\r?\n\s*\r?\n", text.strip()) if block.strip()]
    previous = -1
    for number, cue in enumerate(cues, 1):
        lines = cue.strip().splitlines()
        if len(lines) < 2 or not lines[0].strip().isdigit() or int(lines[0]) <= previous:
            return Observation("fail", f"Cue {number} index missing or out of order", "srt")
        previous = int(
            lines[0]
        )  # files may start at 0 or mid-sequence; indices only need to ascend
        if not TIMING.match(lines[1].strip()):
            return Observation("fail", f"Cue {number} timing line invalid", "srt")
    return Observation("pass", f"{len(cues)} SubRip cues", "srt")
