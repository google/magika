# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""iCalendar: balanced BEGIN/END components and NAME:VALUE content lines with folding."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("ics",)
SCOPE = "BEGIN:VCALENDAR root, nested components balanced by name, every unfolded line a NAME[;params]:value content line, END:VCALENDAR last; property semantics not checked"
LINE = re.compile(r"^[A-Za-z0-9-]+(;[^:]*)?:")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    head = data.lstrip(b"\xef\xbb\xbf \r\n")
    if not head.startswith(b"BEGIN:VCALENDAR"):
        return None
    try:
        text = head.decode("utf-8")
    except UnicodeDecodeError:
        return Observation("fail", "Not UTF-8 text", "ics")
    unfolded = re.sub(r"\r?\n[ \t]", "", text)
    stack = []
    for number, line in enumerate(unfolded.splitlines(), 1):
        if not line.strip():
            continue
        if line.startswith("BEGIN:"):
            stack.append(line[6:].strip().upper())
            continue
        if line.startswith("END:"):
            name = line[4:].strip().upper()
            if not stack or stack.pop() != name:
                return Observation(
                    "fail", f"Line {number}: END:{name} does not close the open component", "ics"
                )
            if (
                not stack
                and unfolded.splitlines()[number:]
                and any(rest.strip() for rest in unfolded.splitlines()[number:])
            ):
                return Observation("fail", "Content after END:VCALENDAR", "ics")
            continue
        if not stack or not LINE.match(line):
            return Observation("fail", f"Line {number} is not a content line", "ics")
    if stack:
        return Observation("fail", f"Unclosed component {stack[-1]}", "ics")
    return Observation("pass", "iCalendar components balanced and content lines well formed", "ics")
