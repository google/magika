# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""vCard 2.1, 3.0 and 4.0: one or more BEGIN/END cards of well-formed content lines."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("vcard",)
SCOPE = "One or more BEGIN:VCARD..END:VCARD cards (2.1 nested AGENT cards allowed), VERSION 2.1, 3.0 or 4.0 in each card, every unfolded line a [group.]NAME[;params]:value content line, quoted-printable soft breaks and 2.1 base64 blocks joined, nothing outside the cards but trailing NUL or Ctrl-Z padding; property values not interpreted"
LINE = re.compile(r"^(?:[A-Za-z0-9-]+\.)?([A-Za-z0-9-]+)((?:;[^:]*)?):(.*)$", re.S)
VERSIONS = {"2.1", "3.0", "4.0"}
CARDS = 100_000


def logical_lines(text: str):
    """Unfold RFC 6350 folding, 2.1 quoted-printable soft breaks and 2.1 base64 blocks."""
    physical = re.split(r"\r*\n|\r", text)  # some exporters write \r\r\n
    index = 0
    while index < len(physical):
        start, line = index + 1, physical[index]
        index += 1
        while True:
            if index < len(physical) and physical[index][:1] in (" ", "\t"):
                line += physical[index][1:]
            elif line.endswith("=") and "QUOTED-PRINTABLE" in line.split(":", 1)[0].upper():
                if index >= len(physical):
                    break
                line = line[:-1] + physical[index]
            elif (
                re.search(r"ENCODING=(?:BASE64|B)\b", line.split(":", 1)[0].upper())
                and index < len(physical)
                and physical[index].strip()
                and ":" not in physical[index]
            ):
                line += physical[index].strip()  # 2.1 base64 continues until a blank line
            else:
                break
            index += 1
        yield start, line


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    head = data.lstrip(b"\xef\xbb\xbf \t\r\n")
    if head[:11].upper() != b"BEGIN:VCARD":
        return None
    body = head.rstrip(b"\0\x1a")
    tags = ("trailing_padding",) if len(body) < len(head) else ()
    head = body
    try:
        text = head.decode("utf-8")
    except UnicodeDecodeError:
        try:
            text = head.decode("cp1252")  # 2.1 exporters often write the platform charset
        except UnicodeDecodeError:
            return Observation("fail", "Neither UTF-8 nor Windows-1252 text", "vcard")
    cards, depth, versions = 0, 0, set()
    for number, line in logical_lines(text):
        if not line.strip():
            continue
        match = LINE.match(line.strip())
        if not match:
            return Observation("fail", f"Line {number} is not a content line", "vcard")
        name, value = match.group(1).upper(), match.group(3).strip()
        if name == "BEGIN":
            if value.upper() != "VCARD":
                return Observation("fail", f"Line {number}: BEGIN:{value} inside a vCard", "vcard")
            if depth and "2.1" not in versions:
                return Observation("fail", f"Line {number}: nested card outside vCard 2.1", "vcard")
            if not depth:
                versions = set()
            depth += 1
            continue
        if not depth:
            return Observation("fail", f"Line {number} lies outside any card", "vcard")
        if name == "END":
            if value.upper() != "VCARD":
                return Observation("fail", f"Line {number}: END:{value} closes no card", "vcard")
            depth -= 1
            if not depth:
                if not versions or not versions <= VERSIONS:
                    return Observation(
                        "fail", f"Card ending on line {number} has no valid VERSION", "vcard"
                    )
                cards += 1
                if cards > CARDS:
                    return Observation("inconclusive", "Card budget exceeded", "vcard")
            continue
        if name == "VERSION":
            versions.add(value)
    if depth:
        return Observation("fail", "Last card is not closed", "vcard")
    return Observation("pass", f"{cards} cards; content lines well formed", "vcard", tags)
