# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Canonical type/tag normalization, independent of validation evidence."""


def canonical_type(kind: str) -> str:
    return "tiff" if kind == "geotiff" else kind


def normalize_annotation(annotation: dict) -> dict:
    result = dict(annotation)
    kinds = annotation.get("format_ids", [])
    known = [*kinds, *annotation.get("existing_accepted_format_ids", [])]
    if "geotiff" in known:
        result["tags"] = sorted(set(annotation.get("tags", [])) | {"geotiff"})
        result.setdefault("previous_format_ids", list(kinds))
        result["format_ids"] = sorted({canonical_type(k) for k in kinds})
        result["type_normalization"] = (
            "GeoTIFF represented as TIFF with geotiff tag; tag inherited from prior metadata"
        )
    return result
