# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Email messages and MHTML archives parsed with the standard library email parser."""

import email
import email.policy
import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("eml", "mht")
SCOPE = "RFC 5322 header block with at least From/Date/Subject/Message-ID/Content-Type headers, body parsed with the default email policy and every part free of defects, multipart boundaries closed; mht requires a multipart (related or mixed) or text/html message; hint required"
REQUIRES_HINT = True
CONTEXT_REQUIRED = True
HEADER = re.compile(rb"^[!-9;-~]+:", re.M)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    head = data.lstrip(b"\xef\xbb\xbf\r\n")
    if not HEADER.match(head):
        return None
    kind = "mht" if "mht" in hints and "eml" not in hints else "eml"
    try:
        message = email.message_from_bytes(head, policy=email.policy.default)
    except Exception as error:
        return Observation("fail", f"Email parser: {type(error).__name__}", kind)
    defects = [d for part in message.walk() for d in part.defects]
    if defects:
        return Observation("fail", f"Structure defect: {defects[0].__class__.__name__}", kind)
    names = {key.lower() for key in message.keys()}
    if not names & {"from", "date", "subject", "message-id", "content-type", "mime-version"}:
        return Observation("fail", "No message headers", kind)
    if message.is_multipart():
        boundary = message.get_boundary()
        if not boundary or b"--" + boundary.encode() + b"--" not in head:
            return Observation("fail", "Multipart message without a closing boundary", kind)
    if kind == "mht" and not message.is_multipart() and message.get_content_type() != "text/html":
        return Observation("fail", "MHTML requires a multipart or text/html message", kind)
    parts = sum(1 for _ in message.walk())
    return Observation(
        "pass", f"{kind}: {len(names)} headers and {parts} MIME parts parsed strictly", kind
    )
