# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""HTML Applications: a markup document declaring an HTA:APPLICATION element."""

import codecs
from html.parser import HTMLParser

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("hta",)
SCOPE = "Text (UTF-8, UTF-16 with BOM or Windows-1252) whose first content is markup, tokenized by the standard HTML parser, with an hta:application start tag outside script, style and comments; a document that only mentions the tag in script or text is not claimed; nothing rendered or executed"
TAG = b"hta:application"
RAW = {"script", "style", "textarea", "xmp"}


class Scan(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.depth, self.found, self.elements = 0, None, 0

    def handle_starttag(self, tag, attrs):
        self.elements += 1
        if tag in RAW:
            self.depth += 1
        elif tag == "hta:application" and not self.depth and self.found is None:
            self.found = dict(attrs)

    def handle_endtag(self, tag):
        if tag in RAW and self.depth:
            self.depth -= 1


def decode(data: bytes) -> str | None:
    for bom, codec in ((codecs.BOM_UTF16_LE, "utf-16"), (codecs.BOM_UTF16_BE, "utf-16")):
        if data.startswith(bom):
            return data.decode(codec, errors="strict")
    for codec in ("utf-8-sig", "cp1252"):
        try:
            return data.decode(codec)
        except UnicodeDecodeError:
            continue
    return None


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    lowered = data.lower()
    if TAG not in lowered and TAG.decode().encode("utf-16-le") not in lowered:
        return None
    try:
        text = decode(data)
    except UnicodeDecodeError:
        text = None
    if text is None or not text.lstrip().startswith("<"):
        return None  # a script or text that mentions the tag is not an HTML Application
    scan = Scan()
    try:
        scan.feed(text)
        scan.close()
    except Exception as error:  # the tolerant parser still raises on pathological input
        return Observation("inconclusive", f"HTML tokenizer error: {str(error)[:60]}", "hta")
    if scan.found is None:
        return Observation(
            "fail", "hta:application appears only inside script, style or text", "hta"
        )
    name = scan.found.get("applicationname") or scan.found.get("id") or "unnamed"
    return Observation(
        "pass",
        f"HTA:APPLICATION element ({name[:40]}) in a {scan.elements}-element document",
        "hta",
    )
