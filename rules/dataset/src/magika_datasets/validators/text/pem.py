# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""PEM blocks and DER certificates: matching labels, valid base64, ASN.1 outer length."""

import base64
import binascii
import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("pem", "crt")
SCOPE = "PEM: one or more BEGIN/END blocks with matching labels and valid base64, text outside blocks tagged rather than failed, encrypted-key header lines allowed. DER (crt): outer SEQUENCE whose length equals the file and three nested elements. PEM certificates are named crt only when hinted; PGP armor is left to the pgp validator"
BEGIN = re.compile(rb"-----BEGIN ([A-Z0-9 ]+)-----\r?\n")
END = re.compile(rb"-----END ([A-Z0-9 ]+)-----\r?\n?")


def der_length(data: bytes, offset: int) -> tuple[int, int] | None:
    if offset >= len(data):
        return None
    first = data[offset]
    if first < 0x80:
        return first, offset + 1
    count = first & 0x7F
    if not 1 <= count <= 4 or offset + 1 + count > len(data):
        return None
    return int.from_bytes(data[offset + 1 : offset + 1 + count], "big"), offset + 1 + count


def der(data: bytes) -> Observation:
    parsed = der_length(data, 1)
    if parsed is None or parsed[0] + parsed[1] != len(data):
        return Observation("fail", "Outer SEQUENCE length differs from file", "crt")
    length, offset = parsed
    elements = 0
    while offset < len(data):
        elements += 1
        inner = der_length(data, offset + 1)
        if data[offset] not in (0x30, 0x03) or inner is None or inner[0] + inner[1] > len(data):
            return Observation(
                "fail", f"Element {elements} is not a bounded SEQUENCE or BIT STRING", "crt"
            )
        offset = inner[0] + inner[1]
    if elements != 3:
        return Observation("fail", f"Certificate has {elements} top-level elements, not 3", "crt")
    return Observation(
        "pass",
        "DER certificate: tbsCertificate, signatureAlgorithm and signature bounded",
        "crt",
        ("der",),
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if (
        data.startswith(b"\x30\x82")
        or data.startswith(b"\x30\x81")
        or (data.startswith(b"\x30") and "crt" in hints)
    ):
        if "crt" in hints or len(data) > 64:
            return der(data) if "crt" in hints else None
    head = data.lstrip(b"\xef\xbb\xbf \r\n")
    if b"-----BEGIN " not in head[:4096]:
        return None
    kind = "crt" if "crt" in hints else "pem"
    offset, blocks, labels, outside = 0, 0, set(), False
    while offset < len(data):
        begin = BEGIN.search(data, offset)
        if begin and begin.group(1).startswith(b"PGP"):
            return None  # OpenPGP armor belongs to the pgp validator
        gap = data[offset : begin.start()] if begin else data[offset:]
        if any(line.strip() and not line.lstrip().startswith(b"#") for line in gap.splitlines()):
            outside = True  # openssl -text dumps and comments commonly surround blocks
        if not begin:
            break
        end = END.search(data, begin.end())
        if not end:
            return Observation("fail", "Unterminated PEM block", kind)
        if end.group(1) != begin.group(1):
            return Observation("fail", "BEGIN and END labels differ", kind)
        inner = data[begin.end() : end.start()]
        if re.match(rb"^[A-Za-z-]+:", inner):  # encrypted keys carry Proc-Type/DEK-Info headers
            split = re.search(rb"\r?\n\r?\n", inner)
            if not split:
                return Observation("fail", "PEM headers without a blank line", kind)
            inner = inner[split.end() :]
        body = b"".join(inner.split())
        try:
            base64.b64decode(body, validate=True)
        except binascii.Error:
            return Observation("fail", "PEM body is not valid base64", kind)
        if not body:
            return Observation("fail", "Empty PEM block", kind)
        blocks += 1
        labels.add(begin.group(1).decode("ascii"))
        offset = end.end()
    if not blocks:
        return Observation("fail", "No PEM blocks", kind)
    tags = sorted("pem_" + label.lower().replace(" ", "_") for label in labels)
    if outside:
        tags.append("text_outside_blocks")
    return Observation(
        "pass",
        f"{blocks} PEM blocks with valid base64: {', '.join(sorted(labels))}",
        kind,
        tuple(tags),
    )
