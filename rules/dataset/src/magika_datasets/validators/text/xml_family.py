# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""XML documents parsed once; the root namespace names the specific format."""

import math
import plistlib
import re

from .._shared import xml
from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = (
    "xml",
    "svg",
    "gpx",
    "kml",
    "gml",
    "collada",
    "rdf",
    "xsd",
    "appleplist",
    "mum",
    "osm",
)
SCOPE = "Complete safe XML parse (entities refused), root namespace or element dispatch to svg, gpx, kml, gml, collada, rdf, xsd, appleplist (values parsed with plistlib), mum and osm; other roots are generic xml that never relabels hinted specific formats; nothing rendered or fetched"
PREFIX_ONLY = True
SHARED_FORMAT_IDS = ("osm",)  # OSM PBF blobs are named by data/osm_pbf.py
SVG = "http://www.w3.org/2000/svg"
KML = {
    "http://www.opengis.net/kml/2.2",
    "http://earth.google.com/kml/2.0",
    "http://earth.google.com/kml/2.1",
    "http://earth.google.com/kml/2.2",
}
ROOTS = {
    "{http://www.topografix.com/GPX/1/1}gpx": "gpx",
    "{http://www.topografix.com/GPX/1/0}gpx": "gpx",
    "{http://www.collada.org/2005/11/COLLADASchema}COLLADA": "collada",
    "{http://www.collada.org/2008/03/COLLADASchema}COLLADA": "collada",
    "{http://www.w3.org/1999/02/22-rdf-syntax-ns#}RDF": "rdf",
    "{http://www.w3.org/2001/XMLSchema}schema": "xsd",
    "{urn:schemas-microsoft-com:asm.v3}assembly": "mum",
    "{urn:schemas-microsoft-com:asm.v1}assembly": "mum",
    "plist": "appleplist",
    "osm": "osm",
}
ELEMENTS = 1_000_000


def svg(root, data: bytes) -> Observation:
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
    scripts = any(element.tag == "{" + SVG + "}script" for element in root.iter())
    return Observation(
        "pass",
        f"SVG XML and viewport checked; contains_scripts={scripts}; no resources executed",
        "svg",
        ("contains_scripts",) if scripts else (),
    )


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.lstrip(b"\xef\xbb\xbf \t\r\n").startswith(b"<"):
        return None
    hinted = next(
        (kind for kind in FORMAT_IDS[1:] if kind in hints), "xml"
    )  # a broken hinted svg fails as svg
    try:
        root = xml.parse(data)
    except (xml.Unsafe, xml.Unsupported) as error:
        return Observation("inconclusive", str(error), hinted)
    except xml.Malformed as error:
        return Observation("fail", str(error), hinted)
    count = 0
    for _ in root.iter():
        count += 1
        if count > ELEMENTS:
            return Observation("inconclusive", "Element budget exceeded", "xml")
    tag = root.tag
    if tag == "{" + SVG + "}svg":
        return svg(root, data)
    if tag in {"{" + ns + "}kml" for ns in KML}:
        return Observation("pass", "KML root in the KML namespace", "kml")
    kind = ROOTS.get(tag)
    if kind == "appleplist":
        try:
            plistlib.loads(data, fmt=plistlib.FMT_XML)
        except Exception as error:  # plistlib raises several exception types for bad values
            return Observation(
                "fail", f"plist values do not parse: {str(error)[:60]}", "appleplist"
            )
        return Observation("pass", "XML property list parsed by plistlib", "appleplist")
    if kind == "osm" and "version" not in root.attrib:
        kind = None
    if kind:
        return Observation("pass", f"{kind} root element {tag}", kind)
    namespace = tag[1:].split("}", 1)[0] if tag.startswith("{") else ""
    if namespace.startswith("http://www.opengis.net/gml") or any(
        element.tag.startswith("{http://www.opengis.net/gml")
        for _, element in zip(range(5000), root.iter())
    ):
        return Observation("pass", "GML feature collection", "gml")
    return Observation(
        "pass", f"Well-formed XML with {count} elements; root {tag}", "xml", generic=True
    )
