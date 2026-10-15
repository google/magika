# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Hunspell affix files: directives, and affix and replacement tables whose counts hold."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("aff",)
SCOPE = "Text lines that are blank, # comments or directives named by an upper-case keyword; at least one PFX or SFX table; every PFX/SFX header (flag, Y/N cross product, count) followed by exactly that many rules for the same flag, and every counted table (REP, MAP, PHONE, BREAK, ICONV, OCONV, COMPOUNDRULE, AF, AM, CHECKCOMPOUNDPATTERN) followed by exactly its count of entries; affix conditions and morphology not interpreted"
KEYWORD = re.compile(r"^[A-Z][A-Z0-9_]*(?:\s|$)")
COUNTED = {
    "REP",
    "MAP",
    "PHONE",
    "BREAK",
    "ICONV",
    "OCONV",
    "COMPOUNDRULE",
    "AF",
    "AM",
    "CHECKCOMPOUNDPATTERN",
}
AFFIXES = {"PFX", "SFX"}


def decode(data: bytes) -> str | None:
    for codec in ("utf-8-sig", "iso-8859-1"):  # SET names the charset; Latin-1 decodes any byte
        try:
            return data.decode(codec)
        except UnicodeDecodeError:
            continue
    return None


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if b"\0" in data:
        return None  # binary content that happens to hold a PFX line is not an affix file
    text = decode(data)
    lines = [line.strip() for line in re.split(r"\r\n|\n|\r", text)]
    content = [(n, line) for n, line in enumerate(lines, 1) if line and not line.startswith("#")]
    if not any(line.split()[0] in AFFIXES for _, line in content if line.split()):
        return None
    tables, pending, rules = 0, None, 0
    for number, line in content:
        fields = line.split()
        keyword = fields[0]
        if not KEYWORD.match(line):
            return Observation("fail", f"Line {number} is not a directive", "aff")
        if pending:
            expected_keyword, flag, remaining = pending
            if keyword != expected_keyword or (
                flag is not None and (len(fields) < 2 or fields[1] != flag)
            ):
                return Observation(
                    "fail",
                    f"Line {number}: {expected_keyword} table ends {remaining} entries early",
                    "aff",
                )
            if keyword in AFFIXES and len(fields) < 4:
                return Observation(
                    "fail", f"Line {number}: affix rule lacks strip and add fields", "aff"
                )
            rules += 1
            pending = (expected_keyword, flag, remaining - 1) if remaining > 1 else None
            continue
        if keyword in AFFIXES:
            if len(fields) < 4 or fields[2] not in ("Y", "N") or not fields[3].isdigit():
                return Observation(
                    "fail", f"Line {number}: affix header is not flag, Y/N and count", "aff"
                )
            tables += 1
            count = int(fields[3])
            pending = (keyword, fields[1], count) if count else None
        elif keyword in COUNTED and len(fields) == 2 and fields[1].isdigit():
            count = int(fields[1])
            pending = (keyword, None, count) if count else None
    if pending:
        return Observation(
            "fail", f"{pending[0]} table ends {pending[2]} entries early at end of file", "aff"
        )
    return Observation("pass", f"{tables} affix tables with {rules} counted entries", "aff")
