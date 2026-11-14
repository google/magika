# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""BibTeX databases: every @entry parsed with balanced braces, keys and field assignments."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("bib",)
PREFIX_ONLY = True  # a leading @ also opens annotations in Dart, Java and decorators
SCOPE = "Leading comments, every @type{key, field = value, ...} entry (braces or parentheses) with balanced brace values, quoted strings, numbers and macro identifiers joined by #, @string/@preamble/@comment forms, text between entries ignored as BibTeX does; citation contents not interpreted"
NAME = re.compile(rb"[A-Za-z][A-Za-z0-9_\-:.+/]*")
KEY = re.compile(rb"[^\s,{}()\"]+")
ENTRIES = 1 << 20


class Malformed(Exception):
    pass


class Parser:
    def __init__(self, data: bytes):
        self.data, self.position = data, 0

    def skip(self):
        while self.position < len(self.data):
            byte = self.data[self.position]
            if byte in b" \t\r\n":
                self.position += 1
            elif byte == 0x25:  # % comment
                end = self.data.find(b"\n", self.position)
                self.position = len(self.data) if end < 0 else end + 1
            else:
                return

    def balanced(self, opener: int, closer: int):
        depth = 0
        while self.position < len(self.data):
            byte = self.data[self.position]
            self.position += 1
            if byte == opener:
                depth += 1
            elif byte == closer:
                depth -= 1
                if depth == 0:
                    return
        raise Malformed("Unbalanced braces in value")

    def value(self):
        while True:
            self.skip()
            if self.position >= len(self.data):
                raise Malformed("Value truncated")
            byte = self.data[self.position]
            if byte == 0x7B:
                self.balanced(0x7B, 0x7D)
            elif byte == 0x22:
                self.position += 1
                while True:
                    if self.position >= len(self.data):
                        raise Malformed("Unterminated quoted value")
                    if self.data[self.position] == 0x7B:
                        self.balanced(0x7B, 0x7D)
                        continue
                    if self.data[self.position] == 0x22:
                        self.position += 1
                        break
                    self.position += 1
            else:
                match = NAME.match(self.data, self.position) or re.compile(rb"\d+").match(
                    self.data, self.position
                )
                if not match:
                    raise Malformed(f"Bad value at byte {self.position}")
                self.position = match.end()
            self.skip()
            if self.data[self.position : self.position + 1] == b"#":
                self.position += 1
                continue
            return

    def entry(self) -> tuple[bytes, int]:
        self.position += 1  # @
        match = NAME.match(self.data, self.position)
        if not match:
            raise Malformed("Entry type missing after @")
        kind = match.group().lower()
        self.position = match.end()
        self.skip()
        opener = self.data[self.position : self.position + 1]
        if opener not in (b"{", b"("):
            raise Malformed(f"Entry {kind.decode()} not opened by a brace or parenthesis")
        closer = b"}" if opener == b"{" else b")"
        self.position += 1
        if kind == b"comment":
            self.position -= 1
            self.balanced(opener[0], closer[0])
            return kind, 0
        if kind == b"preamble":
            self.value()
        elif kind == b"string":
            self.skip()
            match = NAME.match(self.data, self.position)
            if not match:
                raise Malformed("@string without a macro name")
            self.position = match.end()
            self.skip()
            if self.data[self.position : self.position + 1] != b"=":
                raise Malformed("@string without =")
            self.position += 1
            self.value()
        fields = 0
        if kind in (b"preamble", b"string"):
            pass
        else:
            self.skip()
            match = KEY.match(self.data, self.position)
            if match:  # BibTeX accepts an empty citation key
                self.position = match.end()
            while True:
                self.skip()
                token = self.data[self.position : self.position + 1]
                if token == closer:
                    break
                if token != b",":
                    raise Malformed(f"Expected , or {closer.decode()} at byte {self.position}")
                self.position += 1
                self.skip()
                if self.data[self.position : self.position + 1] == closer:
                    break  # trailing comma
                match = NAME.match(self.data, self.position)
                if not match:
                    raise Malformed(f"Field name expected at byte {self.position}")
                self.position = match.end()
                self.skip()
                if self.data[self.position : self.position + 1] != b"=":
                    raise Malformed("Field without =")
                self.position += 1
                self.value()
                fields += 1
        self.skip()
        if self.data[self.position : self.position + 1] != closer:
            raise Malformed(f"Entry {kind.decode()} not closed")
        self.position += 1
        return kind, fields


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    parser = Parser(data)
    parser.skip()
    if parser.data[parser.position : parser.position + 1] != b"@":
        return None
    counts, assignments = {}, 0
    try:
        while True:
            parser.skip()
            if parser.position >= len(data):
                break
            if data[parser.position] != 0x40:  # BibTeX ignores text between entries
                following = data.find(b"@", parser.position)
                if following < 0:
                    break
                parser.position = following
                continue
            kind, fields = parser.entry()
            counts[kind] = counts.get(kind, 0) + 1
            assignments += fields
            if sum(counts.values()) > ENTRIES:
                return Observation("inconclusive", "Entry budget exceeded", "bib")
    except Malformed as error:
        return Observation("fail", str(error), "bib")
    if not counts or (not assignments and "bib" not in hints):
        return Observation("fail", "No entries with field assignments", "bib")
    return Observation(
        "pass", f"{sum(counts.values())} entries parsed ({len(counts)} types)", "bib"
    )
