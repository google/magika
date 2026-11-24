# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""gettext .po catalogs: msgctxt/msgid/msgid_plural/msgstr entries with quoted strings."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("po",)
SCOPE = "Entries of optional comments, optional msgctxt, msgid, optional msgid_plural and msgstr or msgstr[n] lines, every string a double-quoted literal with continuation lines; hint required"
REQUIRES_HINT = True
CONTEXT_REQUIRED = True
KEYWORD = re.compile(r'^(msgctxt|msgid|msgid_plural|msgstr(\[\d+\])?)\s+("(?:[^"\\]|\\.)*")\s*$')
STRING = re.compile(r'^"(?:[^"\\]|\\.)*"\s*$')


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if b"msgid" not in data[:65536]:
        return None
    try:
        text = data.decode("utf-8-sig")
    except UnicodeDecodeError:
        return Observation("fail", "Not UTF-8 text", "po")
    entries, state = 0, None  # state: None, "ctxt", "id", "plural", "str"
    for number, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if not stripped:
            if state == "str":
                entries += 1
            elif state is not None:
                return Observation("fail", f"Line {number}: entry ended before msgstr", "po")
            state = None
            continue
        if stripped.startswith("#"):
            if state is not None and state != "str":
                return Observation("fail", f"Line {number}: comment inside an entry", "po")
            if state == "str":
                entries, state = entries + 1, None
            continue
        if STRING.match(stripped):
            if state is None:
                return Observation(
                    "fail", f"Line {number}: string continuation without a keyword", "po"
                )
            continue
        match = KEYWORD.match(stripped)
        if not match:
            return Observation("fail", f"Line {number} is not a catalog line", "po")
        keyword = match.group(1)
        if keyword == "msgctxt" and state not in (None,):
            return Observation("fail", f"Line {number}: msgctxt out of order", "po")
        if keyword == "msgid" and state not in (None, "ctxt"):
            if state == "str":
                entries, state = entries + 1, None
            else:
                return Observation("fail", f"Line {number}: msgid out of order", "po")
        if keyword == "msgid_plural" and state != "id":
            return Observation("fail", f"Line {number}: msgid_plural without msgid", "po")
        if keyword.startswith("msgstr") and state not in ("id", "plural", "str"):
            return Observation("fail", f"Line {number}: msgstr without msgid", "po")
        state = {"msgctxt": "ctxt", "msgid": "id", "msgid_plural": "plural"}.get(keyword, "str")
    if state == "str":
        entries += 1
    elif state is not None:
        return Observation("fail", "Catalog ends inside an entry", "po")
    if not entries:
        return Observation("inconclusive", "No entries", "po")
    return Observation("pass", f"{entries} gettext entries", "po")
