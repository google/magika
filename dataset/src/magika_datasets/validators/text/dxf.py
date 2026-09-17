# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ASCII DXF: group code and value pairs, SECTION/ENDSEC balance and a final EOF."""

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("dxf",)
SCOPE = "Alternating integer group code and value lines, 0/SECTION opened by 2/NAME and closed by 0/ENDSEC without nesting, 0/EOF as the final pair; entities not interpreted"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    head = data.lstrip(b"\xef\xbb\xbf \r\n")
    if not (head.startswith(b"0\n") or head.startswith(b"0\r\n")) or b"SECTION" not in head[:64]:
        if not head.startswith(b"999"):
            return None
    try:
        text = head.decode("utf-8")
    except UnicodeDecodeError:
        text = head.decode("latin-1")
    lines = text.splitlines()
    if len(lines) % 2:
        return Observation("fail", "Odd number of lines: a group code lacks its value", "dxf")
    in_section, sections, ended = False, 0, False
    for index in range(0, len(lines), 2):
        if ended:
            return Observation("fail", "Pairs after 0/EOF", "dxf")
        code, value = lines[index].strip(), lines[index + 1].strip()
        if not code.lstrip("-").isdigit():
            return Observation("fail", f"Line {index + 1} is not a group code", "dxf")
        if code == "0":
            if value == "SECTION":
                if in_section:
                    return Observation("fail", "Nested SECTION", "dxf")
                in_section, sections = True, sections + 1
            elif value == "ENDSEC":
                if not in_section:
                    return Observation("fail", "ENDSEC without SECTION", "dxf")
                in_section = False
            elif value == "EOF":
                if in_section:
                    return Observation("fail", "EOF inside a section", "dxf")
                ended = True
    if not ended:
        return Observation("fail", "Missing 0/EOF", "dxf")
    return Observation("pass", f"{sections} sections balanced; EOF present", "dxf")
