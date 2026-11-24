# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Append reviewed classes to the published taxonomy, and apply reviewed corrections.

Existing ordinals never move: samples reference a class by ordinal."""

import argparse
import json
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

from .parquet_metadata import CLASS_SCHEMA
from .receipt import refresh as refresh_receipt
from .runtime import private_output

FIELDS = ("format_id", "name", "categories", "extensions")


def additions(path: Path) -> list[dict]:
    entries = json.loads(Path(path).read_text())["classes"]
    for entry in entries:
        missing = [name for name in FIELDS if not entry.get(name)]
        if missing:
            raise ValueError(f"Class addition is missing {', '.join(missing)}")
    return entries


def extend(classes_path: Path, additions_path: Path) -> dict:
    """Append every class not already present, in file order, and rewrite the table.

    Samples reference a class by ordinal, so an addition may only take the next free one.
    Reapplying the same file is a no-op, which keeps the step safe to rerun.
    """
    rows = pq.read_table(classes_path).to_pylist()
    known = {row["format_id"]: row for row in rows}
    if len(known) != len(rows):
        raise ValueError("Taxonomy already contains a duplicate format id")
    added = []
    for entry in additions(additions_path):
        existing = known.get(entry["format_id"])
        if existing is not None:
            # Rerunning the same file must be a no-op, but the additions file is not a
            # place to redefine a published class: samples already carry its ordinal.
            if any(existing[name] != entry[name] for name in FIELDS):
                raise ValueError(
                    f"{entry['format_id']} is already in the taxonomy with a different definition"
                )
            continue
        metadata = dict(entry.get("metadata", {}))
        metadata.setdefault("class_role", "format")
        metadata["id"] = f"{metadata['class_role']}:{entry['format_id']}"
        metadata["counts"] = {"samples": 0, "target": 0}
        rows.append(
            {
                "ordinal": len(rows),
                **{name: entry[name] for name in FIELDS},
                "metadata_json": json.dumps(metadata, sort_keys=True),
            }
        )
        known[entry["format_id"]] = rows[-1]
        added.append(entry["format_id"])
    if added:
        _write(classes_path, rows)
    return {"added": added, "classes": len(rows)}


CORRECTABLE = ("name", "categories", "extensions")
METADATA = ("target", "merged_into")
"""Corrections kept in metadata_json: the count target, and the class a merged one joined."""


def _write(classes_path: Path, rows: list[dict]) -> None:
    temporary = Path(classes_path).with_suffix(".next")
    pq.write_table(pa.Table.from_pylist(rows, schema=CLASS_SCHEMA), temporary, compression="zstd")
    temporary.replace(classes_path)
    if (Path(classes_path).parent / "samples.parquet").exists():
        refresh_receipt(Path(classes_path).parent)


def correct(classes_path: Path, corrections_path: Path) -> dict:
    """Redefine descriptive fields of published classes, keeping what each replaced.

    The name, categories, extensions, count target and merge target may change; the format
    id and ordinal stay, so
    every sample keeps its class. The replaced values are recorded in the class metadata
    with the reason, and reapplying the same file is a no-op.
    """
    rows = pq.read_table(classes_path).to_pylist()
    known = {row["format_id"]: row for row in rows}
    corrected = []
    for entry in json.loads(Path(corrections_path).read_text())["classes"]:
        row = known.get(entry["format_id"])
        if row is None:
            raise ValueError(f"No class {entry['format_id']} to correct")
        if not entry.get("reason"):
            raise ValueError(f"Correction to {entry['format_id']} gives no reason")
        metadata = json.loads(row["metadata_json"] or "{}")
        current = {
            **{k: row[k] for k in CORRECTABLE},
            "target": metadata.get("counts", {}).get("target"),
            "merged_into": metadata.get("merged_into"),
        }
        changes = {
            k: entry[k] for k in CORRECTABLE + METADATA if k in entry and entry[k] != current[k]
        }
        if not changes:
            continue
        if "merged_into" in changes and changes["merged_into"] not in known:
            raise ValueError(
                f"{entry['format_id']} cannot merge into unknown {changes['merged_into']}"
            )
        replaced = {k: current[k] for k in changes}
        metadata.setdefault("corrections", []).append({**replaced, "reason": entry["reason"]})
        if "target" in changes:
            metadata.setdefault("counts", {})["target"] = changes["target"]
        if "merged_into" in changes:
            metadata["merged_into"] = changes["merged_into"]
        row.update({k: v for k, v in changes.items() if k in CORRECTABLE})
        row["metadata_json"] = json.dumps(metadata, sort_keys=True)
        corrected.append(entry["format_id"])
    if corrected:
        _write(classes_path, rows)
    return {"corrected": corrected, "classes": len(rows)}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--classes", type=Path, default=Path("classes.parquet"))
    parser.add_argument("--additions", type=Path, default=Path("config/taxonomy-additions.json"))
    parser.add_argument(
        "--corrections", type=Path, default=Path("config/taxonomy-corrections.json")
    )
    args = parser.parse_args(argv)
    classes = private_output(args.classes)
    result = extend(classes, args.additions)
    if args.corrections.exists():
        result.update(correct(classes, args.corrections))
    print(json.dumps(result, sort_keys=True))
