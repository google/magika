# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Windows security catalogs: a PKCS#7 SignedData DER structure carrying a certificate trust list."""

from .._shared import der
from ..contract import Observation

FAMILY = "application"
FORMAT_IDS = ("cat",)
SCOPE = "Outer DER SEQUENCE equal to the file, signedData content type, [0] EXPLICIT SignedData whose encapsulated content is the szOID_CTL certificate trust list, every nested DER element tiling its parent; signatures not verified"
SIGNED_DATA = b"\x06\x09\x2a\x86\x48\x86\xf7\x0d\x01\x07\x02"
CTL = b"\x06\x09\x2b\x06\x01\x04\x01\x82\x37\x0a\x01"  # 1.3.6.1.4.1.311.10.1


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 32 or data[0] != 0x30:
        return None
    try:
        tag, start, length, _ = der.element(data, 0)
    except der.Malformed:
        return None
    if data[start : start + len(SIGNED_DATA)] != SIGNED_DATA:
        return None
    if start + length != len(data):
        return Observation("fail", "Outer SEQUENCE does not span the whole file", "cat")
    try:
        der.walk(data, 0, len(data))
    except der.Malformed as error:
        return Observation("fail", f"DER structure invalid: {error}", "cat")
    try:  # ContentInfo { contentType, [0] { SignedData { version, digestAlgorithms, encapContentInfo { eContentType ... } } } }
        _, explicit_start, _, _ = der.element(data, start + len(SIGNED_DATA))
        _, signed_start, _, _ = der.element(data, explicit_start)
        _, version_start, version_length, _ = der.element(data, signed_start)
        _, digests_start, digests_length, _ = der.element(data, version_start + version_length)
        _, content_start, _, _ = der.element(data, digests_start + digests_length)
    except der.Malformed:
        return Observation("fail", "SignedData structure truncated", "cat")
    if data[content_start : content_start + len(CTL)] != CTL:
        return Observation(
            "inconclusive", "SignedData does not wrap a certificate trust list", "cat"
        )
    return Observation(
        "pass",
        "PKCS#7 SignedData wrapping a certificate trust list; DER elements tile the file",
        "cat",
    )
