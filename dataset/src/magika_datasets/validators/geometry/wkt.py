# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Well-Known Text geometry: the whole file parsed as one or more WKT or EWKT geometries."""

import re

from ..contract import Observation

FAMILY = "geometry"
FORMAT_IDS = ("wkt",)
SCOPE = "Every non-blank token sequence parsed as OGC Simple Features WKT (optionally EWKT with SRID=n;): point, line and polygon types, their MULTI forms, GEOMETRYCOLLECTION, curve and surface types, Z, M and ZM dimensions, EMPTY, and every coordinate in a geometry carrying the same number of ordinates its dimension allows; ring closure and geometric validity not checked"
TOKEN = re.compile(
    r"\s*(?:(?P<srid>SRID=\d+;)|(?P<word>[A-Za-z]+)|(?P<number>[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?)|(?P<punct>[(),]))"
)
# Parenthesis depth of each type's coordinates: 1 is "(x y, x y)", 2 a list of those, ...
LEVELS = {
    "POINT": 1,
    "LINESTRING": 1,
    "CIRCULARSTRING": 1,
    "LINEARRING": 1,
    "POLYGON": 2,
    "TRIANGLE": 2,
    "MULTIPOINT": 2,
    "MULTILINESTRING": 2,
    "MULTIPOLYGON": 3,
    "TIN": 3,
    "POLYHEDRALSURFACE": 3,
}
COLLECTIONS = {"GEOMETRYCOLLECTION", "COMPOUNDCURVE", "CURVEPOLYGON", "MULTICURVE", "MULTISURFACE"}
DIMENSIONS = {"": (2, 3, 4), "Z": (3,), "M": (3,), "ZM": (4,)}
LIMIT = 16 * 1024 * 1024
DEPTH = 32


class Invalid(Exception):
    pass


class Parser:
    def __init__(self, text: str):
        self.tokens = []
        position = 0
        while position < len(text):
            if text[position:].strip() == "":
                break
            match = TOKEN.match(text, position)
            if not match:
                raise Invalid(f"Unexpected character at offset {position}")
            kind = match.lastgroup
            self.tokens.append((kind, match.group(kind).upper()))
            position = match.end()
        self.index = 0

    def peek(self):
        return self.tokens[self.index] if self.index < len(self.tokens) else (None, None)

    def take(self, kind=None, value=None):
        token = self.peek()
        if token[0] is None or (kind and token[0] != kind) or (value and token[1] != value):
            raise Invalid(f"Expected {value or kind} at token {self.index}")
        self.index += 1
        return token[1]

    def geometry(self, depth=0) -> int:
        if depth > DEPTH:
            raise Invalid("Geometry nesting too deep")
        if depth == 0 and self.peek()[0] == "srid":
            self.take("srid")
        name = self.take("word")
        dimension = ""
        if self.peek() == ("word", "ZM") or self.peek() in (("word", "Z"), ("word", "M")):
            dimension = self.take("word")
        widths = DIMENSIONS[dimension]
        if self.peek() == ("word", "EMPTY"):
            self.take("word")
            return 0
        if name in LEVELS:
            count = self.sequence(LEVELS[name], widths, name == "MULTIPOINT", set())
            if name == "POINT" and count != 1:
                raise Invalid("POINT holds more than one coordinate")
            return count
        if name in COLLECTIONS:
            self.take("punct", "(")
            count = self.member(depth, name)
            while self.peek() == ("punct", ","):
                self.take("punct")
                count += self.member(depth, name)
            self.take("punct", ")")
            return count
        raise Invalid(f"Unknown geometry type {name}")

    def member(self, depth, parent):
        # Curve collections may hold bare coordinate sequences standing for line strings.
        if parent != "GEOMETRYCOLLECTION" and self.peek() == ("punct", "("):
            return self.sequence(1, (2, 3, 4), False, set())
        return self.geometry(depth + 1)

    def coordinate(self, widths, seen):
        ordinates = 0
        while self.peek()[0] == "number":
            self.take("number")
            ordinates += 1
        if ordinates not in widths or (seen and ordinates not in seen):
            raise Invalid(f"Coordinate with {ordinates} ordinates at token {self.index}")
        seen.add(ordinates)

    def sequence(self, levels, widths, bare_points, seen) -> int:
        """'(' items ')' where items are coordinates at level 1 and sequences above it."""
        self.take("punct", "(")
        count = 0
        while True:
            if self.peek() == ("word", "EMPTY"):
                self.take("word")
            elif levels == 1 or (bare_points and self.peek()[0] == "number"):
                self.coordinate(widths, seen)
                count += 1
            else:
                count += self.sequence(levels - 1, widths, bare_points, seen)
            if self.peek() != ("punct", ","):
                break
            self.take("punct")
        self.take("punct", ")")
        return count


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    head = data.lstrip(b"\xef\xbb\xbf \t\r\n")
    if not re.match(rb"(?:SRID=\d+;\s*)?[A-Za-z]+\s*(?:Z|M|ZM)?\s*[(E]", head[:64], re.I):
        return None
    word = re.match(rb"(?:SRID=\d+;\s*)?([A-Za-z]+)", head).group(1).upper().decode()
    if word not in set(LEVELS) | COLLECTIONS:
        return None
    if len(data) > LIMIT:
        return Observation("inconclusive", "Text too large to parse", "wkt")
    try:
        parser = Parser(head.decode("ascii"))
        geometries = coordinates = 0
        while parser.peek()[0] is not None:
            coordinates += parser.geometry()
            geometries += 1
    except UnicodeDecodeError:
        return Observation("fail", "Not ASCII text", "wkt")
    except Invalid as error:
        return Observation("fail", str(error), "wkt")
    return Observation(
        "pass", f"{geometries} geometries with {coordinates} coordinates parsed", "wkt"
    )
