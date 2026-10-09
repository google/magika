# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""STEP physical files (ISO 10303-21): section skeleton and entity instance statements."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("step",)
SCOPE = "ISO-10303-21 opener, HEADER and DATA sections each closed by ENDSEC, every DATA statement a #n = ENTITY(...); instance, END-ISO-10303-21 terminator; entities not interpreted"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.lstrip().startswith(b"ISO-10303-21;"):
        return None
    text = data.decode("latin-1")
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    if "END-ISO-10303-21;" not in text:
        return Observation("fail", "Missing END-ISO-10303-21 terminator", "step")
    if text.split("END-ISO-10303-21;", 1)[1].strip():
        return Observation("fail", "Content after the terminator", "step")
    sections = re.findall(r"\b(HEADER|DATA)\s*;(.*?)ENDSEC\s*;", text, flags=re.S)
    names = [name for name, _ in sections]
    if "HEADER" not in names or "DATA" not in names:
        return Observation("fail", "HEADER or DATA section missing or unterminated", "step")
    entities = 0
    for name, body in sections:
        if name != "DATA":
            continue
        for statement in re.split(r";\s*", body.strip()):
            if not statement.strip():
                continue
            if not re.match(r"^\s*#\d+\s*=\s*(\(|[A-Z_0-9]+\s*\()", statement):
                return Observation(
                    "fail", f"DATA statement is not an entity instance: {statement[:40]!r}", "step"
                )
            entities += 1
    return Observation(
        "pass", f"{entities} entity instances in {names.count('DATA')} DATA section(s)", "step"
    )
