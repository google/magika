# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""JSON documents parsed once; the root shape names GeoJSON, notebooks, glTF or JSON Lines."""

import json

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("json", "geojson", "ipynb", "gltf", "jsonl")
SHARED_FORMAT_IDS = ("gltf",)  # binary GLB containers are named by geometry/gltf.py
SCOPE = "Strict UTF-8 JSON parse (duplicate keys inconclusive, no NaN/Infinity), root shape dispatch to GeoJSON (type plus features, geometry or coordinates), Jupyter notebooks (cells and nbformat), glTF JSON (asset.version) or JSON Lines when hinted; plain objects and arrays are generic json"
PREFIX_ONLY = True
CONTEXT_REQUIRED = True
GEOJSON = {
    "FeatureCollection",
    "Feature",
    "Point",
    "MultiPoint",
    "LineString",
    "MultiLineString",
    "Polygon",
    "MultiPolygon",
    "GeometryCollection",
}


class DuplicateKey(ValueError):
    pass


def pairs(items):
    result = {}
    for key, value in items:
        if key in result:
            raise DuplicateKey(key)
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError("Non-JSON constant")


def parse(data: bytes):
    return json.loads(data.decode("utf-8"), object_pairs_hook=pairs, parse_constant=reject_constant)


def lines(data: bytes, hints: frozenset[str]) -> Observation:
    count = 0
    for number, line in enumerate(data.splitlines(), 1):
        if not line.strip():
            continue
        try:
            parse(line)
        except DuplicateKey:
            return Observation("inconclusive", f"Line {number} has duplicate keys", "jsonl")
        except (UnicodeError, ValueError, RecursionError):
            return Observation("fail", f"Line {number} is not a JSON value", "jsonl")
        count += 1
    if not count:
        return Observation("fail", "No JSON lines", "jsonl")
    return Observation("pass", f"{count} JSON lines parsed", "jsonl")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    stripped = data.lstrip(b"\xef\xbb\xbf \t\r\n")
    if not stripped.startswith((b"{", b"[")):
        return None
    if "jsonl" in hints and data.count(b"\n") > 1 and not stripped.startswith(b"["):
        return lines(data, hints)
    try:
        document = parse(data.removeprefix(b"\xef\xbb\xbf"))
    except DuplicateKey:
        return Observation(
            "inconclusive", "Duplicate object keys have ambiguous interpretation", "json"
        )
    except RecursionError:
        return Observation("inconclusive", "Parser recursion limit reached", "json")
    except (UnicodeError, ValueError):
        if "jsonl" in hints:
            return lines(data, hints)
        return Observation("fail", "Invalid UTF-8 or JSON syntax", "json")
    if isinstance(document, dict):
        kind = document.get("type")
        if kind in GEOJSON:
            if kind == "FeatureCollection" and not isinstance(document.get("features"), list):
                return Observation("fail", "FeatureCollection without a features list", "geojson")
            if kind == "Feature" and not isinstance(document.get("geometry"), (dict, type(None))):
                return Observation("fail", "Feature without a geometry object", "geojson")
            if kind not in (
                "FeatureCollection",
                "Feature",
                "GeometryCollection",
            ) and not isinstance(document.get("coordinates"), list):
                return Observation("fail", "Geometry without coordinates", "geojson")
            if kind == "GeometryCollection" and not isinstance(document.get("geometries"), list):
                return Observation("fail", "GeometryCollection without geometries", "geojson")
            return Observation("pass", f"GeoJSON {kind}", "geojson")
        if "cells" in document and "nbformat" in document:
            if not isinstance(document["cells"], list) or not isinstance(document["nbformat"], int):
                return Observation("fail", "Notebook cells or nbformat malformed", "ipynb")
            return Observation(
                "pass",
                f"Jupyter notebook nbformat {document['nbformat']} with {len(document['cells'])} cells",
                "ipynb",
            )
        asset = document.get("asset")
        if (
            isinstance(asset, dict)
            and "version" in asset
            and any(k in document for k in ("scenes", "nodes", "meshes", "buffers", "accessors"))
        ):
            return Observation("pass", f"glTF JSON asset version {asset['version']}", "gltf")
    return Observation(
        "pass",
        "Complete JSON object/array parsed; application type not established",
        "json",
        generic=True,
    )
