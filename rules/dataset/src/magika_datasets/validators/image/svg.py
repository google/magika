# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""SVG XML identity and viewport checks; never render scripts or external resources."""

import math
import re

from .._shared import xml
from ..contract import Observation

FAMILY = "image"
FORMAT_IDS = ("svg",)
PREFIX_ONLY = True  # applicability rests on a short prefix; failures need a hint
SCOPE = "Complete safe XML parse, SVG namespace/root and viewBox checks; no rendering, CSS/path evaluation or external resource validation"
NS = "http://www.w3.org/2000/svg"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.lstrip(b"\xef\xbb\xbf \t\r\n").startswith(b"<"):
        return None
    try:
        root = xml.parse(data)
    except (xml.Unsafe, xml.Unsupported) as error:
        return Observation("inconclusive", str(error), "svg")
    except xml.Malformed as error:
        return Observation("fail", str(error), "svg")
    if root.tag != "{" + NS + "}svg":
        return None
    if "viewBox" in root.attrib:
        try:
            values = [float(v) for v in re.split(r"[\s,]+", root.attrib["viewBox"].strip())]
        except ValueError:
            return Observation("fail", "Invalid viewBox numbers", "svg")
        if (
            len(values) != 4
            or not all(math.isfinite(v) for v in values)
            or values[2] < 0
            or values[3] < 0
        ):
            return Observation("fail", "Invalid viewBox dimensions", "svg")
    elements = 0
    scripts = False
    for element in root.iter():
        elements += 1
        if elements > 100000:
            return Observation("inconclusive", "Element budget exceeded", "svg")
        scripts |= element.tag == "{" + NS + "}script"
    return Observation(
        "pass",
        f"SVG XML and viewport checked; contains_scripts={scripts}; no resources executed",
        "svg",
        ("contains_scripts",) if scripts else (),
    )
