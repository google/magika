# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Lay a hydrated snapshot's files out on disk with the labels detectors are scored on."""

import json
from pathlib import Path

import pyarrow.parquet as pq

from ..acquisition import sha256_file
from ..parquet_corpus import shard_rows
from ..runtime import write
from ..validation import PRECEDENCE

VERIFIED = frozenset(PRECEDENCE)
FORMAT = "parquet-whole-file-v1"


def truth(row: dict) -> str | None:
    """The label a detector is scored against, or None when the sample is not ground truth.

    Only a verified label counts. A sample known to be one member of a multi-file artifact
    is not scored on its own, since a detector sees only that member.
    """
    layout = json.loads(row["annotation_json"]).get("properties", {}).get("artifact_layout") or {}
    split = layout.get("state") == "known" and layout.get("value") != "single_file"
    return row["format_id"] if row["label_status"] in VERIFIED and not split else None


def prepare(dataset: Path, output: Path) -> tuple[dict, list[dict]]:
    """(classes, samples), every byte checked against the snapshot before it is written."""
    dataset, output = Path(dataset).resolve(), Path(output).resolve()
    manifest = json.loads((dataset / "manifest.json").read_text())
    if manifest.get("format") != FORMAT:
        raise ValueError("Expected a hydrated whole-file Parquet snapshot")
    expected = manifest["source_metadata_receipt"]["files"]["classes.parquet"]["sha256"]
    if sha256_file(dataset / "classes.parquet") != expected:
        raise ValueError("Class metadata hash mismatch")
    classes = {
        row["format_id"]: json.loads(row["metadata_json"])
        | {key: row[key] for key in ("format_id", "name", "categories", "extensions")}
        for row in pq.read_table(dataset / "classes.parquet").to_pylist()
    }
    files = output / "files"
    files.mkdir(parents=True, exist_ok=True)
    samples = []
    for row, content in shard_rows(dataset):
        sha = row["sha256"].hex()
        if row["format_id"] not in classes:
            raise ValueError("Sample names a class the snapshot does not define")
        target = files / sha
        if not target.exists():
            target.write_bytes(content)
        elif sha256_file(target) != sha:
            raise ValueError("Previously materialized input was modified")
        samples.append(dict(sha256=sha, size=len(content), path=str(target), truth=truth(row)))
    if len({s["sha256"] for s in samples}) != len(samples):
        raise ValueError("Snapshot repeats a sample")
    write(output / "inputs.json", dict(classes=classes, samples=samples))
    return classes, samples
