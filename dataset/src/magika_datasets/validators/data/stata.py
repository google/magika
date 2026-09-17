# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Stata .dta files: tagged sections with map offsets for releases 117 and later."""

import re
import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("stata",)
SCOPE = "Release 117+: <stata_dta> envelope, header tags, map of 14 offsets ascending and matching the section tags, </stata_dta> at EOF; releases 102 to 115 header sanity only, inconclusive"
SECTIONS = (
    "stata_dta",
    "map",
    "variable_types",
    "varnames",
    "sortlist",
    "formats",
    "value_label_names",
    "variable_labels",
    "characteristics",
    "data",
    "strls",
    "value_labels",
)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data.startswith(b"<stata_dta>"):
        if not data.endswith(b"</stata_dta>"):
            return Observation("fail", "Missing </stata_dta> at EOF", "stata")
        release = re.search(rb"<release>(\d{3})</release>", data[:200])
        order = re.search(rb"<byteorder>(MSF|LSF)</byteorder>", data[:200])
        if not release or not order:
            return Observation("fail", "Header lacks release or byteorder", "stata")
        endian = "<" if order.group(1) == b"LSF" else ">"
        map_at = data.find(b"<map>")
        if (
            map_at < 0
            or map_at + 5 + 112 + 6 > len(data)
            or data[map_at + 5 + 112 : map_at + 5 + 118] != b"</map>"
        ):
            return Observation("fail", "Map section missing or malformed", "stata")
        offsets = struct.unpack_from(endian + "14Q", data, map_at + 5)
        present = [o for o in offsets if o]
        if offsets[0] != 0 or present != sorted(present) or offsets[-1] != len(data):
            return Observation("fail", "Map offsets are not ascending to EOF", "stata")
        for index, name in enumerate(SECTIONS[1:], 1):
            if not offsets[index]:
                continue  # some writers leave unused sections at zero
            tag = f"<{name}>".encode()
            if data[offsets[index] : offsets[index] + len(tag)] != tag:
                return Observation(
                    "fail", f"Map offset {index} does not point at <{name}>", "stata"
                )
        if data[offsets[12] : offsets[12] + 12] != b"</stata_dta>":
            return Observation("fail", "Map offset 12 does not point at the closing tag", "stata")
        return Observation(
            "pass",
            f"Stata release {release.group(1).decode()}: 13 sections mapped to EOF",
            "stata",
            (f"release_{release.group(1).decode()}",),
        )
    if (
        len(data) >= 110
        and 102 <= data[0] <= 115
        and data[1] in (1, 2)
        and data[2] == 1
        and data[3] == 0
    ):
        return Observation(
            "inconclusive", f"Stata release {data[0]} legacy layout not walked", "stata"
        )
    return None
