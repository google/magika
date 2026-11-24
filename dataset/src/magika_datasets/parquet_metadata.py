# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Lossless public Parquet metadata exports for the whole-file corpus."""

import hashlib
import json
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

CLASS_SCHEMA = pa.schema(
    [
        ("ordinal", pa.uint16()),
        ("format_id", pa.string()),
        ("name", pa.string()),
        ("categories", pa.list_(pa.string())),
        ("extensions", pa.list_(pa.string())),
        ("metadata_json", pa.string()),
    ]
)
SAMPLE_SCHEMA = pa.schema(
    [
        ("class_ordinal", pa.uint16()),
        ("sample_ordinal", pa.uint16()),
        ("format_id", pa.string()),
        ("label_status", pa.string()),
        ("sha256", pa.binary(32)),
        ("size", pa.uint64()),
        ("origins", pa.list_(pa.string())),
        ("hard_case", pa.bool_()),
        ("annotation_json", pa.string()),
    ]
)
MAX_SAMPLE_BYTES = 8 * 1024 * 1024
"""Largest whole file the corpus publishes. Shards, verification and replay downloads all
hold one file in memory under this budget, so a larger sample could be admitted but never
hydrated."""
LABEL_STATUSES = frozenset(
    {
        "validated_manual",
        "llm-validated",
        "validated_auto",
        "validated_tools",
        "validated_origin",
        "conflicting",
        "need_review",
    }
)
"""Label provenance, highest confidence first. The first four are benchmark truth."""


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encode(rows):
    return "".join(json.dumps(r, sort_keys=True, ensure_ascii=True) + "\n" for r in rows).encode()


def restore(classes_path, samples_path):
    class_table, sample_table = pq.read_table(classes_path), pq.read_table(samples_path)
    if not class_table.schema.equals(
        CLASS_SCHEMA, check_metadata=False
    ) or not sample_table.schema.equals(SAMPLE_SCHEMA, check_metadata=False):
        raise ValueError("Unexpected corpus Parquet schema")
    rows = []
    for record in sorted(class_table.to_pylist(), key=lambda r: r["ordinal"]):
        if record["ordinal"] != len(rows):
            raise ValueError("Duplicate or missing class ordinal")
        metadata = json.loads(record["metadata_json"])
        keys = {"format_id", "name", "categories", "extensions"}
        if set(metadata) & (keys | {"samples"}):
            raise ValueError("Class metadata overrides typed columns")
        rows.append({**metadata, **{k: record[k] for k in keys}, "samples": []})
    for record in sorted(
        sample_table.to_pylist(), key=lambda r: (r["class_ordinal"], r["sample_ordinal"])
    ):
        if record["class_ordinal"] >= len(rows):
            raise ValueError("Unknown sample class ordinal")
        row = rows[record["class_ordinal"]]
        if record["format_id"] != row["format_id"] or record["sample_ordinal"] != len(
            row["samples"]
        ):
            raise ValueError("Sample class/ordinal mismatch")
        annotation = json.loads(record["annotation_json"])
        if set(annotation) & {"sha256", "size", "origins", "hard_case", "label_status"}:
            raise ValueError("Sample annotation overrides typed columns")
        if record["label_status"] not in LABEL_STATUSES:
            raise ValueError(f"Unknown label status {record['label_status']!r}")
        row["samples"].append(
            {
                **annotation,
                "sha256": record["sha256"].hex(),
                "size": record["size"],
                "origins": record["origins"],
                "hard_case": record["hard_case"],
                "label_status": record["label_status"],
            }
        )
    return rows


BATCH = 128


def write_samples(grouped: dict, formats: dict, output: Path) -> list[dict]:
    """Renumber every class contiguously and publish the table.

    restore() requires sample_ordinal to run 0..n-1 within a class in table order, so a
    sample that changes class has to renumber both the class it left and the one it joined.
    """
    rows = []
    for kind in sorted(grouped, key=lambda k: formats[k]["ordinal"]):
        members = grouped[kind]
        if len(members) > 65535:
            raise ValueError(f"Class {kind} holds {len(members)} samples; sample_ordinal is uint16")
        for ordinal, member in enumerate(members):
            rows.append({**member, "sample_ordinal": ordinal})
    temporary = Path(output).with_suffix(".next")
    with pq.ParquetWriter(temporary, SAMPLE_SCHEMA, compression="zstd") as writer:
        for start in range(0, len(rows), BATCH):
            writer.write_table(
                pa.Table.from_pylist(rows[start : start + BATCH], schema=SAMPLE_SCHEMA)
            )
    temporary.replace(output)
    return rows
