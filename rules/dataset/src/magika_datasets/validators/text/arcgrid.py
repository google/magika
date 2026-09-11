# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ESRI ASCII grids: header keys and exactly rows times columns numeric values."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("arcgrid",)
SCOPE = "ncols, nrows, corner/centre and cellsize header lines, optional NODATA_value, then exactly nrows times ncols numeric tokens"
NUMBER = re.compile(rb"^[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?$")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.lstrip().lower().startswith(b"ncols"):
        return None
    lines = data.splitlines()
    header, index = {}, 0
    while index < len(lines):
        words = lines[index].split()
        if len(words) == 2 and words[0].lower() in (
            b"ncols",
            b"nrows",
            b"xllcorner",
            b"yllcorner",
            b"xllcenter",
            b"yllcenter",
            b"cellsize",
            b"nodata_value",
            b"dx",
            b"dy",
        ):
            header[words[0].lower()] = words[1]
            index += 1
        else:
            break
    try:
        cols, rows = int(header[b"ncols"]), int(header[b"nrows"])
    except (KeyError, ValueError):
        return Observation("fail", "ncols or nrows missing", "arcgrid")
    if not {b"xllcorner", b"xllcenter"} & header.keys() or not (
        b"cellsize" in header or b"dx" in header
    ):
        return Observation("fail", "Corner or cell size header missing", "arcgrid")
    tokens = 0
    for line in lines[index:]:
        for token in line.split():
            if not NUMBER.match(token):
                return Observation("fail", f"Non-numeric grid value {token[:20]!r}", "arcgrid")
            tokens += 1
    if tokens != rows * cols:
        return Observation("fail", f"{tokens} values for a {rows}x{cols} grid", "arcgrid")
    return Observation("pass", f"{rows}x{cols} grid values counted", "arcgrid")
