# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Wavefront OBJ models and MTL material libraries: known keywords and numeric arguments."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("obj", "mtl")
SCOPE = "Every non-comment line starts with a known OBJ or MTL keyword, vertex data numeric, face indices integer with optional texture and normal slots; hint required"
REQUIRES_HINT = True
CONTEXT_REQUIRED = True
OBJ = {
    "v",
    "vt",
    "vn",
    "vp",
    "f",
    "l",
    "p",
    "o",
    "g",
    "s",
    "mtllib",
    "usemtl",
    "cstype",
    "deg",
    "bmat",
    "step",
    "curv",
    "curv2",
    "surf",
    "parm",
    "trim",
    "hole",
    "scrv",
    "sp",
    "end",
    "con",
    "mg",
    "bevel",
    "c_interp",
    "d_interp",
    "lod",
    "shadow_obj",
    "trace_obj",
    "ctech",
    "stech",
    "call",
    "csh",
}
MTL = {
    "newmtl",
    "Ka",
    "Kd",
    "Ks",
    "Ke",
    "Tf",
    "Ns",
    "Ni",
    "d",
    "Tr",
    "illum",
    "sharpness",
    "map_Ka",
    "map_Kd",
    "map_Ks",
    "map_Ke",
    "map_Ns",
    "map_d",
    "map_bump",
    "bump",
    "disp",
    "decal",
    "refl",
    "map_refl",
    "Pr",
    "Pm",
    "Ps",
    "Pc",
    "Pcr",
    "aniso",
    "anisor",
    "norm",
    "map_Pr",
    "map_Pm",
    "map_Ps",
    "map_Ke",
}
NUMBER = re.compile(r"^[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?$")
FACE = re.compile(r"^-?\d+(/-?\d*)?(/-?\d+)?$")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    kind = "mtl" if "mtl" in hints and "obj" not in hints else "obj"
    keywords = MTL if kind == "mtl" else OBJ
    try:
        text = data.decode("utf-8-sig")
    except UnicodeDecodeError:
        text = data.decode("latin-1")
    statements = 0
    for number, raw in enumerate(text.splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        while line.endswith("\\"):
            line = line[:-1]
        words = line.split()
        keyword, args = words[0], words[1:]
        if keyword not in keywords:
            return Observation("fail", f"Line {number}: unknown keyword {keyword!r}", kind)
        if (
            kind == "obj"
            and keyword in ("v", "vt", "vn", "vp")
            and not all(NUMBER.match(a) for a in args)
        ):
            return Observation("fail", f"Line {number}: non-numeric vertex data", kind)
        if (
            kind == "obj"
            and keyword in ("f", "l", "p")
            and (not args or not all(FACE.match(a) for a in args))
        ):
            return Observation("fail", f"Line {number}: malformed element indices", kind)
        if (
            kind == "mtl"
            and keyword in ("Ka", "Kd", "Ks", "Ke", "Ns", "Ni", "d", "Tr")
            and args
            and args[0] not in ("spectral", "xyz")
            and not all(NUMBER.match(a) for a in args)
        ):
            return Observation("fail", f"Line {number}: non-numeric material value", kind)
        statements += 1
    if not statements:
        return Observation("inconclusive", "No statements", kind)
    return Observation("pass", f"{statements} {kind} statements with known keywords", kind)
