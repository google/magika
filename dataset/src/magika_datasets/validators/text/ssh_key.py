# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""OpenSSH public keys: each line's base64 blob must encode the key type it is labelled with."""

import base64
import binascii
import struct

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("pub",)
SCOPE = "Every non-blank, non-comment line is a key type, a base64 blob and an optional comment; the blob decodes and begins with a length-prefixed type string equal to the line's key type"
TYPES = {
    b"ssh-rsa",
    b"ssh-dss",
    b"ssh-ed25519",
    b"ecdsa-sha2-nistp256",
    b"ecdsa-sha2-nistp384",
    b"ecdsa-sha2-nistp521",
    b"sk-ssh-ed25519@openssh.com",
    b"sk-ecdsa-sha2-nistp256@openssh.com",
}


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    lines = [
        line.strip() for line in data.splitlines() if line.strip() and not line.startswith(b"#")
    ]
    if not lines or lines[0].split(b" ", 1)[0] not in TYPES:
        return None
    for number, line in enumerate(lines, 1):
        fields = line.split()
        if len(fields) < 2 or fields[0] not in TYPES:
            return Observation("fail", f"Line {number} is not a public key", "pub")
        try:
            blob = base64.b64decode(fields[1], validate=True)
            (length,) = struct.unpack(">I", blob[:4])
        except (binascii.Error, struct.error):
            return Observation("fail", f"Line {number} blob is not base64", "pub")
        if blob[4 : 4 + length] != fields[0]:
            return Observation("fail", f"Line {number} blob encodes a different key type", "pub")
    return Observation("pass", f"{len(lines)} OpenSSH public keys, blobs match their types", "pub")
