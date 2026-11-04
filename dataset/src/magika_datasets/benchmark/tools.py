# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""The detectors under test: their commands, identities and how their output maps to classes."""

import hashlib
import json
import re
import shutil
from collections import defaultdict
from enum import StrEnum
from pathlib import Path

from ..acquisition import sha256_file
from .process import run_cli

RESOURCE_FLAGS = {
    "--threads",
    "--readers",
    "--batch-size",
    "--intra-threads",
    "--inter-threads",
    "--num-tasks",
}
THREAD_CAPS = {
    "OMP_NUM_THREADS",
    "OMP_THREAD_LIMIT",
    "OMP_DYNAMIC",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "MKL_DYNAMIC",
    "RAYON_NUM_THREADS",
    "VECLIB_MAXIMUM_THREADS",
    "NUMEXPR_NUM_THREADS",
}


class Adapter(StrEnum):
    MAGIKA = "magika-jsonl"
    FILE = "file-mime"
    TRID = "trid"


def validate_default_policy(spec: dict) -> None:
    """Every tool runs with its shipping resource defaults, so nothing is tuned for the test."""
    if THREAD_CAPS.intersection(spec.get("environment", {})):
        raise ValueError(
            "Benchmark requires tool defaults; remove thread-cap environment variables"
        )
    for tool in spec["tools"]:
        if any(argument.split("=", 1)[0] in RESOURCE_FLAGS for argument in tool["command"]):
            raise ValueError(
                f"{tool['id']}: benchmark requires tool defaults; remove resource overrides"
            )


def fingerprint(value) -> str:
    text = json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)
    return hashlib.sha256(text.encode()).hexdigest()


def label_mapping(classes: dict, adapter: Adapter) -> dict[str, list[str]]:
    """Tool label to the classes it can mean, from class metadata, never from the answer."""
    aliases = defaultdict(set)
    for name, row in classes.items():
        if adapter == Adapter.MAGIKA:
            meta = row.get("magika", {})
            labels = [name, *meta.get("output_labels", []), *meta.get("kb_labels", [])]
        elif adapter == Adapter.FILE:
            labels = row.get("mimes", [])
        else:
            labels = [x.lower().lstrip(".") for x in row.get("extensions", [])]
        for label in labels:
            aliases[label].add(name)
    return {key: sorted(value) for key, value in sorted(aliases.items())}


def mapped(raw, aliases, *, abstain=False, tie=False, error=None, deterministic=None) -> dict:
    candidates = sorted({name for label in raw for name in aliases.get(label, [])})
    if error:
        state = "error"
    elif abstain:
        state = "abstained"
    elif tie or len(candidates) > 1:
        state = "ambiguous"
    else:
        state = "mapped" if candidates else "unmapped"
    return dict(
        raw_labels=raw,
        candidates=candidates,
        mapping=state,
        prediction=candidates[0] if state == "mapped" else None,
        error=error,
        deterministic=deterministic,
    )


def parse_magika(raw: bytes, paths, aliases) -> list[dict]:
    values = [json.loads(line) for line in raw.splitlines()]
    if [v["path"] for v in values] != list(paths):
        raise ValueError("Magika output paths are missing or reordered")
    rows = []
    for value in values:
        result = value["result"]
        if result["status"] != "ok":
            rows.append(mapped([], aliases, error=result["status"]))
            continue
        label = result["value"]["output"]["label"]
        rows.append(
            mapped(
                [label],
                aliases,
                abstain=label in ("unknown", "undefined"),
                # A rule decision leaves the model's label undefined.
                deterministic=result["value"]["dl"]["label"] == "undefined",
            )
        )
    return rows


def parse_file(raw: bytes, paths, aliases) -> list[dict]:
    """`file -0 -0 --mime-type` output: NUL-framed names and complete descriptions."""
    fields = raw.decode().split("\0")
    if fields[-1] != "" or fields[:-1:2] != list(paths):
        raise ValueError("file output paths are missing or reordered")
    # Apple's multiline fat-binary output keeps the top-level MIME first; score that.
    labels = [field.splitlines()[0] for field in fields[1:-1:2]]
    return [
        mapped(
            [label],
            aliases,
            abstain=label == "application/octet-stream",
            error=label if label.startswith("ERROR:") else None,
        )
        for label in labels
    ]


def parse_trid(raw: bytes, paths, aliases) -> list[dict]:
    groups = re.split(r"(?m)^File: (.+)\r?$", raw.decode())
    if groups[1::2] != list(paths):
        raise ValueError("TrID output paths are missing or reordered")
    rows = []
    for group in groups[2::2]:
        matches = re.findall(r"(?m)^\s*([\d.]+)% \(\.([^)]*)\)", group)
        if not matches:
            if "Unknown!" not in group:
                raise ValueError("TrID output has neither predictions nor an abstention")
            rows.append(mapped([], aliases, abstain=True))
            continue
        # Only the top rank counts: never a lower candidate that happens to match truth.
        score, extensions = matches[0]
        tie = len(matches) > 1 and matches[1][0] == score
        rows.append(mapped(extensions.lower().split("/"), aliases, tie=tie, abstain=not extensions))
    return rows


PARSERS = {Adapter.MAGIKA: parse_magika, Adapter.FILE: parse_file, Adapter.TRID: parse_trid}


def parse_output(adapter, raw: bytes, paths, aliases) -> list[dict]:
    return PARSERS[Adapter(adapter)](raw, paths, aliases)


def command(tool: dict, paths, directory: Path) -> list[str]:
    if tool["adapter"] == Adapter.TRID:  # TrID reads long file lists from a file
        listing = directory / (fingerprint(paths) + ".txt")
        listing.write_text("\n".join(paths) + "\n")
        return [*tool["command"], "-f", str(listing)]
    return [*tool["command"], "--", *paths]


def identify(tool: dict, env: dict, timeout: float) -> dict:
    """Version, executable and artifact hashes, so a result names exactly what ran."""
    # Resolving a venv's Python symlink would run the base interpreter without its packages.
    executable = str(Path(shutil.which(tool["command"][0]) or tool["command"][0]).absolute())
    tool["command"][0] = executable
    return dict(
        version=run_cli(tool["version_command"], env, timeout)[1].decode().strip(),
        executable_sha256=sha256_file(Path(executable)),
        artifacts={role: sha256_file(Path(path)) for role, path in tool["artifacts"].items()},
        command=tool["command"],
    )
