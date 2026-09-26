# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""PDF: header, objects, cross-reference table or stream, trailer and %%EOF."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("pdf", "ai")
SHARED_FORMAT_IDS = ("ai",)  # EPS-based Illustrator files are named by the PostScript validator
SCOPE = "%PDF-1.x header, %%EOF within the trailing kilobyte, startxref offset pointing at an xref table or an xref stream object, trailer or stream dictionary with /Root, every N G obj balanced by endobj with declared stream lengths respected; content streams not decoded; encryption and JavaScript recorded as tags; named ai when AIPrivateData is embedded"
OBJECT = re.compile(rb"(\d+)\s+(\d+)\s+obj\b")
OBJECTS = 1_000_000


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(b"%PDF-"):
        return None
    if not re.match(rb"%PDF-\d\.\d", data):
        return Observation("fail", "Malformed version header", "pdf")
    tail = data[-1024:]
    eof = tail.rfind(b"%%EOF")
    if eof < 0:
        return Observation("fail", "%%EOF not found in the last kilobyte", "pdf")
    if tail[eof + 5 :].strip():
        return Observation("fail", "Content after the final %%EOF", "pdf")
    start = tail.rfind(b"startxref", 0, eof)
    if start < 0:
        return Observation("fail", "startxref missing before %%EOF", "pdf")
    try:
        offset = int(tail[start + 9 : eof].split()[0])
    except (IndexError, ValueError):
        return Observation("fail", "startxref offset unreadable", "pdf")
    if offset >= len(data):
        return Observation("fail", "startxref offset outside file", "pdf")
    at = data[offset : offset + 64].lstrip()
    if at.startswith(b"xref"):
        trailer = data.rfind(b"trailer", offset)
        if trailer < 0 or b"/Root" not in data[trailer : trailer + 4096]:
            return Observation("fail", "xref table without a trailer /Root", "pdf")
        tags = []
        if b"/Encrypt" in data[trailer : trailer + 4096]:
            tags.append("encrypted")
    elif OBJECT.match(at):
        stream_end = data.find(b"stream", offset)
        header = data[offset : stream_end if stream_end > 0 else offset + 4096]
        if b"/XRef" not in header or b"/Root" not in header:
            return Observation(
                "fail", "startxref does not point at an xref stream with /Root", "pdf"
            )
        tags = ["xref_stream"]
        if b"/Encrypt" in header:
            tags.append("encrypted")
    else:
        return Observation("fail", "startxref does not point at xref or an object", "pdf")
    count, position = 0, 0
    while True:
        match = OBJECT.search(data, position)
        if not match:
            break
        count += 1
        if count > OBJECTS:
            return Observation("inconclusive", "Object budget exceeded", "pdf")
        end = data.find(b"endobj", match.end())
        following = OBJECT.search(data, match.end())
        if end < 0 or (following and following.start() < end):
            return Observation(
                "fail",
                f"Object {match.group(1).decode()} lacks endobj before the next object",
                "pdf",
            )
        stream = data.find(b"stream", match.end(), end)
        if stream > 0:
            length = re.search(rb"/Length\s+(\d+)(?!\s+\d+\s+R)", data[match.end() : stream])
            if length:
                body = stream + 6
                body += 2 if data[body : body + 2] == b"\r\n" else 1
                declared = int(length.group(1))
                if body + declared > end:
                    return Observation(
                        "fail",
                        f"Object {match.group(1).decode()} stream length exceeds endobj",
                        "pdf",
                    )
        position = end + 6
    if not count:
        return Observation("fail", "No objects", "pdf")
    if re.search(rb"/JavaScript|/JS\b", data):
        tags.append("has_javascript")
    if b"/AA" in data or b"/OpenAction" in data:
        tags.append("has_actions")
    if b"/AIPrivateData" in data:  # native Illustrator content embedded in the PDF container
        return Observation(
            "pass",
            f"{count} objects bounded; cross-reference and trailer located; AIPrivateData present",
            "ai",
            tuple(tags),
        )
    return Observation(
        "pass", f"{count} objects bounded; cross-reference and trailer located", "pdf", tuple(tags)
    )
