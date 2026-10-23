# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""OneNote revision stores (.one/.onetoc2): header GUIDs, expected file length and root chunk references."""

import struct
import uuid

from ..contract import Observation

FAMILY = "office"
FORMAT_IDS = ("one",)
SCOPE = "Header file-type and format GUIDs, cbExpectedFileLength against the file, transaction log, file node list root, free chunk list and hashed chunk list references inside the file; object spaces not decoded"
FILE_TYPES = {
    uuid.UUID("{7B5C52E4-D88C-4DA7-AEB1-5378D02996D3}"): "section",
    uuid.UUID("{43FF2FA1-EFD9-4C76-9EE2-10EA5722765F}"): "toc",
}
FORMAT = uuid.UUID("{109ADD3F-911B-49F5-A5D0-1791EDC8AED8}")
HEADER = 1024


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if len(data) < 16:
        return None
    kind = FILE_TYPES.get(uuid.UUID(bytes_le=data[:16]))
    if kind is None:
        return None
    if len(data) < HEADER:
        return Observation("fail", "Header shorter than 1024 bytes", "one")
    if uuid.UUID(bytes_le=data[48:64]) != FORMAT:
        return Observation("fail", "guidFileFormat is not the OneNote revision store GUID", "one")
    expected = struct.unpack_from("<Q", data, 196)[0]
    if expected != len(data):
        return Observation(
            "fail", f"cbExpectedFileLength {expected} differs from the file size", "one"
        )
    references = {
        "hashed chunk list": struct.unpack_from("<QI", data, 148),
        "transaction log": struct.unpack_from("<QI", data, 160),
        "file node list root": struct.unpack_from("<QI", data, 172),
        "free chunk list": struct.unpack_from("<QI", data, 184),
    }
    for name, (start, size) in references.items():
        if start == 0xFFFFFFFFFFFFFFFF or (start == 0 and size == 0):
            continue  # nil or zero reference
        if start < HEADER or start + size > len(data):
            return Observation("fail", f"{name} reference outside the file", "one")
    root = references["file node list root"]
    if root[0] in (0, 0xFFFFFFFFFFFFFFFF) or root[1] == 0:
        return Observation("fail", "File node list root reference missing", "one")
    return Observation(
        "pass",
        f"OneNote {kind}; expected length, GUIDs and chunk references verified",
        "one",
        (kind,),
    )
