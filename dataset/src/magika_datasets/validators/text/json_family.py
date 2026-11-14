# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""JSON documents parsed once; the root shape names GeoJSON, notebooks, glTF, TensorFlow.js and
XGBoost models, or JSON Lines."""

import json

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("json", "geojson", "ipynb", "gltf", "jsonl", "tensorflowjs", "xgboost")
SHARED_FORMAT_IDS = ("gltf",)  # binary GLB containers are named by geometry/gltf.py
SCOPE = "Strict UTF-8 JSON parse (duplicate keys inconclusive, no NaN/Infinity), root shape dispatch to GeoJSON (type plus features, geometry or coordinates), Jupyter notebooks (cells and nbformat), glTF JSON (asset.version), TensorFlow.js model.json (modelTopology plus a weightsManifest whose groups list string paths and named weights with shape and dtype), XGBoost JSON models (version triple plus learner with gradient_booster and objective), or JSON Lines when hinted; plain objects and arrays are generic json"
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


def tensorflowjs(document: dict) -> Observation | None:
    """A TF.js model.json: the topology and a manifest naming every weight file and tensor."""
    if "modelTopology" not in document or "weightsManifest" not in document:
        return None
    manifest = document["weightsManifest"]
    if not isinstance(manifest, list) or not manifest:
        return Observation("fail", "weightsManifest is not a non-empty list", "tensorflowjs")
    tensors = 0
    for number, group in enumerate(manifest):
        paths, weights = (
            (group.get("paths"), group.get("weights")) if isinstance(group, dict) else (None, None)
        )
        if not isinstance(paths, list) or not all(isinstance(p, str) and p for p in paths):
            return Observation(
                "fail", f"Manifest group {number} has no string paths", "tensorflowjs"
            )
        if not isinstance(weights, list):
            return Observation(
                "fail", f"Manifest group {number} has no weights list", "tensorflowjs"
            )
        for weight in weights:
            if not (
                isinstance(weight, dict)
                and isinstance(weight.get("name"), str)
                and isinstance(weight.get("shape"), list)
                and all(type(d) is int and d >= 0 for d in weight["shape"])
                and isinstance(weight.get("dtype"), str)
            ):
                return Observation(
                    "fail", f"Manifest group {number} has a malformed weight", "tensorflowjs"
                )
            tensors += 1
    if not isinstance(document["modelTopology"], (dict, type(None))):
        return Observation("fail", "modelTopology is not an object", "tensorflowjs")
    kind = document.get("format", "unspecified")
    return Observation(
        "pass",
        f"TensorFlow.js {kind} manifest: {len(manifest)} weight groups, {tensors} tensors",
        "tensorflowjs",
    )


def xgboost(document: dict) -> Observation | None:
    """An XGBoost JSON model: a version triple and a learner naming its booster and objective."""
    if "learner" not in document or "version" not in document:
        return None
    version, learner = document["version"], document["learner"]
    if not (
        isinstance(version, list) and len(version) == 3 and all(type(v) is int for v in version)
    ):
        return Observation("fail", "version is not three integers", "xgboost")
    if not isinstance(learner, dict):
        return Observation("fail", "learner is not an object", "xgboost")
    booster = learner.get("gradient_booster")
    if not isinstance(booster, dict) or not isinstance(booster.get("name"), str):
        return Observation("fail", "learner has no named gradient_booster", "xgboost")
    if not isinstance(learner.get("objective"), dict):
        return Observation("fail", "learner has no objective", "xgboost")
    return Observation(
        "pass",
        f"XGBoost {'.'.join(map(str, version))} model with {booster['name']} booster",
        "xgboost",
    )


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
        model = tensorflowjs(document)
        if model:
            return model
        booster = xgboost(document)
        if booster:
            return booster
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
