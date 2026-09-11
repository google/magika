# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Validate build inputs before allocating a detached run."""

import json
from pathlib import Path

from .verdicts import TOOLS


def bounded_integer(config: dict, key: str, default: int, minimum: int, maximum: int) -> None:
    value = config.get(key, default)
    if type(value) is not int or not minimum <= value <= maximum:
        raise ValueError(f"{key} must be an integer in {minimum}..{maximum}")


def require_file(value: str | Path) -> Path:
    path = Path(value)
    if not path.is_file():
        raise ValueError(f"Required input does not exist: {path}")
    return path


def validate_build_config(config: dict) -> None:
    """Paths are relative to the documented project working directory."""
    for key in ("collection_run", "collection_config", "metadata_dir", "tools"):
        if not isinstance(config.get(key), str) or not config[key]:
            raise ValueError(f"{key} must be a nonempty path")
    bounded_integer(config, "verdict_workers", 4, 1, 8)
    bounded_integer(config, "conflicts_per_class", 100, 0, 10000)
    bounded_integer(config, "conflict_pages_per_class", 20, 1, 50)
    for name in ("classes.parquet", "samples.parquet", "corpus-parquet-receipt.json"):
        require_file(Path(config["metadata_dir"]) / name)
    collection = json.loads(require_file(config["collection_config"]).read_text())
    for key in ("store", "parquet_dir", "license_policy"):
        if not isinstance(collection.get(key), str) or not collection[key]:
            raise ValueError(f"collection.{key} must be a nonempty path")
    require_file(collection["license_policy"])
    for name in (
        "repositories.parquet",
        "repositories-receipt.json",
        "repository-files.parquet",
        "repository-files-receipt.json",
        "classes.parquet",
        "samples.parquet",
        "corpus-parquet-receipt.json",
    ):
        require_file(Path(collection["parquet_dir"]) / name)
    if collection.get("github_hints"):
        require_file(collection["github_hints"])
    if not isinstance(collection.get("vt_recipes"), list):
        raise ValueError("collection.vt_recipes must be a list of recipe paths")
    for recipe in collection["vt_recipes"]:
        require_file(recipe)
    bounded_integer(collection, "target_per_class", 100, 1, 65535)
    bounded_integer(collection, "rebalance_candidates_per_class", 20, 1, 65535)
    if type(collection.get("seed")) is not int:
        raise ValueError("collection.seed must be an integer")
    if type(collection.get("rebalance", False)) is not bool:
        raise ValueError("collection.rebalance must be boolean")
    tools = json.loads(require_file(config["tools"]).read_text())
    if (
        not isinstance(tools, list)
        or not all(isinstance(t, dict) for t in tools)
        or len(tools) != len(TOOLS)
        or {t.get("id") for t in tools} != TOOLS
    ):
        raise ValueError("Configure each of the eight detector IDs exactly once")
    for tool in tools:
        argv = tool.get("argv")
        if not isinstance(argv, list) or not argv or not all(isinstance(arg, str) for arg in argv):
            raise ValueError(f"{tool['id']}: argv must be a nonempty list of strings")
        if not any("{path}" in arg for arg in argv):
            raise ValueError(f"{tool['id']}: argv must include the sample placeholder {{path}}")
