# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Plain text by encoding: ASCII, UTF-8 and UTF-16, decoded in full with no control bytes."""

import codecs

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("txtascii", "txtutf8", "txtutf16")
SCOPE = "Whole file decoded strictly: printable ASCII with whitespace; otherwise UTF-8 (optional BOM) containing non-ASCII text; or UTF-16 with a byte-order mark and even length. Any C0 control other than tab, newline, carriage return or form feed fails. Runs only when a text encoding is hinted, because nearly every text format is also valid text"
REQUIRES_HINT = True
# Generic: a LightGBM model or a CSV is also valid text, and must not be outvoted by its encoding.
ALLOWED = set("\t\n\r\f")


def clean(text: str) -> bool:
    return all(ch >= " " or ch in ALLOWED for ch in text) and "\x7f" not in text


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data:
        return Observation("fail", "Empty file is not a text document", "txtascii")
    if data.startswith((codecs.BOM_UTF16_LE, codecs.BOM_UTF16_BE)):
        if len(data) % 2:
            return Observation("fail", "UTF-16 with an odd byte count", "txtutf16")
        try:
            text = data.decode("utf-16")
        except UnicodeError:
            return Observation("fail", "Invalid UTF-16", "txtutf16")
        if not clean(text):
            return Observation("fail", "UTF-16 text contains control characters", "txtutf16")
        return Observation(
            "pass", f"{len(text)} UTF-16 characters decoded", "txtutf16", generic=True
        )
    body = data.removeprefix(codecs.BOM_UTF8)
    try:
        text = body.decode("utf-8")
    except UnicodeError:
        return Observation("fail", "Neither ASCII, UTF-8 nor BOM-marked UTF-16", "txtutf8")
    if not clean(text):
        kind = "txtascii" if text.isascii() else "txtutf8"
        return Observation("fail", "Text contains control characters", kind)
    if text.isascii() and not data.startswith(codecs.BOM_UTF8):
        return Observation(
            "pass", f"{len(text)} printable ASCII characters", "txtascii", generic=True
        )
    return Observation("pass", f"{len(text)} UTF-8 characters decoded", "txtutf8", generic=True)
