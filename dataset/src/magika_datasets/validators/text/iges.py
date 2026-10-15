# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""IGES files: 80-column records with section letters in order and a terminate record."""

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("iges",)
SCOPE = "Every record 80 columns (plus newline) with section letter S, G, D, P then T in column 73 and ascending sequence numbers, terminate record counts equal to the section sizes; entities not interpreted"
ORDER = "SGDPT"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    lines = data.splitlines()
    if not lines or len(lines[0]) < 73 or lines[0][72:73] != b"S":
        return None
    counts = {letter: 0 for letter in ORDER}
    stage = 0
    for number, line in enumerate(lines, 1):
        if len(line) < 73 or len(line.rstrip(b"\r")) > 80:
            return Observation("fail", f"Record {number} is not 80 columns", "iges")
        letter = chr(line[72])
        if letter not in ORDER:
            return Observation("fail", f"Record {number} has section letter {letter!r}", "iges")
        if ORDER.index(letter) < stage:
            return Observation("fail", f"Record {number} section {letter} out of order", "iges")
        stage = ORDER.index(letter)
        try:
            sequence = int(line[73:80])
        except ValueError:
            return Observation("fail", f"Record {number} sequence number unreadable", "iges")
        counts[letter] += 1
        if sequence != counts[letter]:
            return Observation("fail", f"Record {number} sequence {sequence} out of order", "iges")
    if counts["T"] != 1 or counts["D"] % 2:
        return Observation("fail", "Terminate record missing or directory entries unpaired", "iges")
    terminate = lines[-1]
    declared = {}
    for letter, position in (("S", 0), ("G", 8), ("D", 16), ("P", 24)):
        try:
            declared[letter] = int(terminate[position + 1 : position + 8])
        except ValueError:
            return Observation("fail", "Terminate record counts unreadable", "iges")
    if any(declared[letter] != counts[letter] for letter in "SGDP"):
        return Observation("fail", "Terminate record counts differ from the sections", "iges")
    return Observation(
        "pass", f"{counts['D'] // 2} directory entries and {counts['P']} parameter records", "iges"
    )
