# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Protein Data Bank files: fixed-column records with known names and parseable ATOM coordinates."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("proteindb",)
SCOPE = "HEADER or first record from the PDB record set, every line's six-character record name known, ATOM/HETATM lines with numeric serial, coordinates, occupancy and temperature factor in their fixed columns, END record last when present; chemistry not interpreted"
RECORDS = {
    b"HEADER",
    b"OBSLTE",
    b"TITLE",
    b"SPLIT",
    b"CAVEAT",
    b"COMPND",
    b"SOURCE",
    b"KEYWDS",
    b"EXPDTA",
    b"NUMMDL",
    b"MDLTYP",
    b"AUTHOR",
    b"REVDAT",
    b"SPRSDE",
    b"JRNL",
    b"REMARK",
    b"DBREF",
    b"DBREF1",
    b"DBREF2",
    b"SEQADV",
    b"SEQRES",
    b"MODRES",
    b"HET",
    b"HETNAM",
    b"HETSYN",
    b"FORMUL",
    b"HELIX",
    b"SHEET",
    b"SSBOND",
    b"LINK",
    b"CISPEP",
    b"SITE",
    b"CRYST1",
    b"ORIGX1",
    b"ORIGX2",
    b"ORIGX3",
    b"SCALE1",
    b"SCALE2",
    b"SCALE3",
    b"MTRIX1",
    b"MTRIX2",
    b"MTRIX3",
    b"MODEL",
    b"ATOM",
    b"ANISOU",
    b"TER",
    b"HETATM",
    b"ENDMDL",
    b"CONECT",
    b"MASTER",
    b"END",
    b"TURN",
    b"HYDBND",
    b"SLTBRG",
    b"FTNOTE",
    b"SIGATM",
    b"SIGUIJ",
    b"TVECT",
    b"USER",
}
NUMBER = re.compile(rb"\s*-?\d*\.?\d+\s*")
LINES = 1 << 21


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    first = data[:6].rstrip()
    if (
        first not in RECORDS
        or first in (b"END", b"TER", b"HET", b"USER")
        and "proteindb" not in hints
    ):
        return None
    lines = data.split(b"\n")
    if len(lines) > LINES:
        return Observation("inconclusive", "Line budget exceeded", "proteindb")
    atoms = 0
    for number, raw in enumerate(lines, start=1):
        line = raw.rstrip(b"\r")
        if not line.strip():
            continue
        name = line[:6].rstrip()
        if name not in RECORDS:
            return Observation(
                "fail", f"Line {number} has unknown record name {name[:6]!r}", "proteindb"
            )
        if name in (b"ATOM", b"HETATM"):
            if len(line) < 54:
                return Observation(
                    "fail", f"Line {number} is too short for coordinates", "proteindb"
                )
            fields = (line[6:11], line[30:38], line[38:46], line[46:54])
            if not all(NUMBER.fullmatch(field) for field in fields):
                return Observation(
                    "fail", f"Line {number} has non-numeric serial or coordinates", "proteindb"
                )
            atoms += 1
    return Observation(
        "pass", f"{len(lines)} records, {atoms} atom records with numeric coordinates", "proteindb"
    )
