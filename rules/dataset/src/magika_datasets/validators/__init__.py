# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Independent, scoped structural observations; never automatic ground truth."""

import hashlib
from pathlib import Path

from . import (
    application,
    archive,
    data,
    executable,
    font,
    geometry,
    image,
    media,
    office,
    system,
    text,
)
from .contract import Observation, describe

MAX_BYTES = (
    16 * 1024 * 1024
)  # largest baseline sample is 8.4 MB; per-validator budgets bound the work
NEUTRAL_HINTS = frozenset({"txt", "text", "unknown", "invalid"})
MODULES = (
    application.MODULES
    + archive.MODULES
    + data.MODULES
    + executable.MODULES
    + font.MODULES
    + geometry.MODULES
    + image.MODULES
    + media.MODULES
    + office.MODULES
    + system.MODULES
    + text.MODULES
)
VERSION = 4


def registry(modules=MODULES) -> dict[str, dict]:
    """Format id to module metadata; duplicate declarations are a startup error."""
    seen = {}
    for module in modules:
        meta = describe(module)
        for format_id in meta["format_ids"]:
            if format_id in seen:
                # Two modules may share a format only when both say so explicitly and
                # apply to disjoint byte prefixes (binary GLB versus JSON glTF).
                if format_id in meta["shared"] and format_id in seen[format_id]["shared"]:
                    continue
                raise ValueError(
                    f"{format_id} declared by both {seen[format_id]['name']} and {meta['name']}"
                )
            seen[format_id] = meta
    return seen


REGISTRY = registry()
DESCRIBED = [(module, describe(module)) for module in MODULES]


def auto_eligible(meta: dict, result: Observation, hints: frozenset[str]) -> bool:
    """Permissive grammars and generic containers only relabel unambiguous samples."""
    if meta["context_required"]:
        return result.format_id in hints and hints <= NEUTRAL_HINTS | {result.format_id}
    if result.generic:
        return hints <= NEUTRAL_HINTS | {result.format_id}
    return True


def record(meta: dict, result: Observation, hints: frozenset[str]) -> dict:
    if result.format_id not in meta["format_ids"]:
        raise ValueError(f"{meta['name']} named undeclared format {result.format_id}")
    return {
        "validator": meta["name"],
        "version": VERSION,
        "format_id": result.format_id,
        "category": meta["family"],
        "auto_eligible": auto_eligible(meta, result, hints),
        "status": result.status,
        "detail": result.detail,
        "scope": meta["scope"],
        "tags": list(result.tags) if result.status == "pass" else [],
    }


def observe(path: str | Path, expected_sha256: str, *, hints=()) -> dict:
    hints = frozenset(hints)
    try:
        with Path(path).open("rb") as stream:
            data = stream.read(MAX_BYTES + 1)
    except OSError:
        return {
            "status": "unavailable",
            "reason": "Cannot read acquired object",
            "observations": [],
        }
    if len(data) > MAX_BYTES:
        return {"status": "skipped", "reason": "16 MiB input limit", "observations": []}
    if hashlib.sha256(data).hexdigest() != expected_sha256:
        return {
            "status": "error",
            "reason": "SHA256 differs from sample identity",
            "observations": [],
        }
    observations = []
    for module, meta in DESCRIBED:
        if meta["requires_hint"] and not hints & set(meta["format_ids"]):
            continue
        try:
            result = module.validate(data, hints)
        except Exception as error:  # a validator bug must not abort a corpus run
            detail = f"Validator error: {type(error).__name__}: {error}"[:200]
            observations.append(
                {
                    **record(
                        meta, Observation("inconclusive", detail, meta["format_ids"][0]), hints
                    ),
                    "auto_eligible": False,
                }
            )
            continue
        if result is None:
            continue
        if meta["prefix_only"] and result.status != "pass" and result.format_id not in hints:
            continue  # a short prefix is not evidence that a failing file was ever this format
        observations.append(record(meta, result, hints))
    return {
        "status": "observed" if observations else "unsupported",
        "observations": observations,
        "policy": "Scoped structural evidence; no label assignment or consensus override",
    }
