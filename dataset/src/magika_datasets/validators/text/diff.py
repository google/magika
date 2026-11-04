# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Unified and git diffs: hunk headers whose line counts match the hunk bodies."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("diff",)
SCOPE = "Unified diff file headers and @@ -a,b +c,d @@ hunks whose context, removed and added line counts match the header; git diff and index lines allowed; hint required"
REQUIRES_HINT = True
CONTEXT_REQUIRED = True
HUNK = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not re.search(rb"^(diff --git |--- |\+\+\+ |@@ -)", data[:65536], re.M):
        return None
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError:
        text = data.decode("latin-1")
    lines = text.splitlines()
    hunks, index = 0, 0
    while index < len(lines):
        match = HUNK.match(lines[index])
        if not match:
            index += 1
            continue
        hunks += 1
        old = int(match.group(2)) if match.group(2) is not None else 1
        new = int(match.group(4)) if match.group(4) is not None else 1
        index += 1
        while (old > 0 or new > 0) and index < len(lines):
            line = lines[index]
            if line.startswith("\\ No newline"):
                index += 1
                continue
            if line.startswith("+"):
                new -= 1
            elif line.startswith("-"):
                old -= 1
            elif line.startswith(" ") or line == "":
                old -= 1
                new -= 1
            else:
                return Observation("fail", f"Line {index + 1} is not a hunk line", "diff")
            index += 1
        if old or new:
            return Observation("fail", f"Hunk {hunks} line counts do not match its header", "diff")
    if not hunks:
        return Observation("inconclusive", "No hunks", "diff")
    return Observation(
        "pass",
        f"{hunks} hunks with matching line counts",
        "diff",
        ("git",) if text.startswith("diff --git") else (),
    )
