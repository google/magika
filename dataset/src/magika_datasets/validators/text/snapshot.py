# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Test snapshot files: Jest/Vitest/Bun export statements or insta YAML-fronted snapshots."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("snap",)
SCOPE = "Jest, Vitest and Bun snapshots: version header comment, every statement `exports[`name`] = `body`;` with template literals parsed including escapes; insta snapshots: `---` front matter (key: value lines, nested YAML allowed) with a source key then `---`; snapshot bodies not interpreted"
JS_HEADER = re.compile(rb"// (Jest|Vitest|Bun) Snapshot v1, https?://\S+\r?\n")
INSTA_KEY = re.compile(rb"[a-z_]+: ?.*")
STATEMENTS = 1 << 20


class Malformed(Exception):
    pass


def template(data: bytes, position: int) -> int:
    """Return the offset after the template literal starting at position (a backtick)."""
    if data[position : position + 1] != b"`":
        raise Malformed(f"Expected a template literal at byte {position}")
    position += 1
    while position < len(data):
        byte = data[position]
        if byte == 0x5C:
            position += 2
            continue
        if byte == 0x60:
            return position + 1
        position += 1
    raise Malformed("Unterminated template literal")


def javascript(data: bytes, flavour: bytes) -> Observation:
    position, statements = JS_HEADER.match(data).end(), 0
    length = len(data)
    while position < length:
        if data[position] in b" \t\r\n":
            position += 1
            continue
        if not data.startswith(b"exports[", position):
            raise Malformed(f"Statement at byte {position} is not an exports assignment")
        position = template(data, position + 8)
        if not data.startswith(b"] = ", position):
            raise Malformed("Snapshot name is not followed by `] = `")
        position = template(data, position + 4)
        if data[position : position + 1] != b";":
            raise Malformed("Snapshot body is not followed by a semicolon")
        position += 1
        statements += 1
        if statements > STATEMENTS:
            return Observation("inconclusive", "Statement budget exceeded", "snap")
    if statements == 0:
        raise Malformed("Snapshot file has no exports")
    return Observation(
        "pass",
        f"{flavour.decode()} snapshot with {statements} exports",
        "snap",
        (flavour.decode().lower(),),
    )


def insta(data: bytes) -> Observation:
    lines = data.split(b"\n")
    keys = {}
    for number, raw in enumerate(lines[1:], start=2):
        line = raw.rstrip(b"\r")
        if line == b"---":
            break
        if line[:1] in (b" ", b"\t", b"-"):
            continue  # nested YAML value or list item
        if not INSTA_KEY.fullmatch(line):
            raise Malformed(f"Front matter line {number} is not a key: value pair")
        keys[line.split(b":", 1)[0]] = True
    else:
        raise Malformed("Front matter is not closed by ---")
    if b"source" not in keys:
        raise Malformed("Front matter lacks the source key")
    return Observation(
        "pass", f"insta snapshot with {len(keys)} front matter keys", "snap", ("insta",)
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    header = JS_HEADER.match(data)
    try:
        if header:
            return javascript(data, header.group(1))
        if data.startswith(b"---\n") or data.startswith(b"---\r\n"):
            if b"\nsource:" not in data[:4096]:
                return None
            return insta(data)
    except Malformed as error:
        return Observation("fail", str(error), "snap")
    return None
