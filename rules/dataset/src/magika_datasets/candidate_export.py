# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Export unreconciled candidates, observations and provenance to Parquet."""

import json
from collections import Counter
from pathlib import Path

import ppdeep
import pyarrow as pa
import pyarrow.parquet as pq

from .acquisition import sha256_file
from .candidate_conflicts import observed_conflict
from .manifest import export_records
from .parquet_metadata import CLASS_SCHEMA, SAMPLE_SCHEMA
from .sources import export_sources
from .unattended import write
from .validation import apply_label
from .validators import observe
from .verdicts import TOOLS


def public_observations(index, record, tools):
    observations = index.observations(record)
    replacements = {
        str(Path(record["path"]).resolve()): "sha256:" + record["sha256"],
        record["path"]: "sha256:" + record["sha256"],
    }
    for tool in tools:
        for artifact in tool.get("artifacts", []):
            replacements[artifact] = "artifact:" + Path(artifact).name
            replacements[str(Path(artifact).resolve())] = "artifact:" + Path(artifact).name
    for observation in observations.values():
        for key in ("stdout", "stderr", "error"):
            if key in observation:
                for path, replacement in sorted(replacements.items(), key=lambda p: -len(p[0])):
                    observation[key] = observation[key].replace(path, replacement)
        observation["display_normalization"] = (
            "Local input/artifact paths replaced; original raw_record_sha256 retained"
        )
    return observations


def export_candidates(records, discovery, rows, index, tools, output):
    """Keep every SHA once. Physical pool buckets explicitly are not format labels."""
    output.mkdir(parents=True, exist_ok=True)
    if len(records) != len({r["sha256"] for r in records}):
        raise ValueError("Duplicate candidate SHA256")
    formats = {r["format_id"]: r for r in rows}
    prior = {s["sha256"]: r["format_id"] for r in rows for s in r["samples"]}
    classes, counts, statuses, tool_statuses = [], Counter(), Counter(), Counter()
    samples_path = output / "samples.parquet"
    review_path = output / "candidate-index.parquet"
    review_schema = pa.schema(
        [
            ("sha256", pa.binary(32)),
            ("size", pa.uint64()),
            ("origins", pa.list_(pa.string())),
            ("discovery_format_ids", pa.list_(pa.string())),
            ("existing_accepted_format_ids", pa.list_(pa.string())),
            ("label_status", pa.string()),
            ("assigned_format_ids", pa.list_(pa.string())),
            ("validation_status", pa.string()),
            ("validation_json", pa.string()),
            ("conflicting", pa.bool_()),
            ("hard_case", pa.bool_()),
            ("ssdeep", pa.string()),
            ("verdicts_json", pa.string()),
            ("vt_markings_json", pa.string()),
            ("conflict_json", pa.string()),
            ("repository_licenses_json", pa.string()),
        ]
    )
    sample_buffer, review_buffer = [], []
    with (
        pq.ParquetWriter(
            samples_path.with_suffix(".next"), SAMPLE_SCHEMA, compression="zstd"
        ) as sample_writer,
        pq.ParquetWriter(
            review_path.with_suffix(".next"), review_schema, compression="zstd"
        ) as review_writer,
    ):
        for number, record in enumerate(sorted(records, key=lambda r: r["sha256"])):
            sha = record["sha256"]
            observations = public_observations(index, record, tools)
            if set(observations) != TOOLS or any(
                v["status"] == "pending" for v in observations.values()
            ):
                raise ValueError("Every candidate needs eight completed tool observations")
            tool_statuses.update(name + ":" + v["status"] for name, v in observations.items())
            markings = []
            licenses = []
            for origin in record["origins"]:
                claim = origin.get("claim") or {}
                if origin.get("provider") == "virustotal" and claim not in markings:
                    markings.append(claim)
                license_record = claim.get("repository_license")
                if license_record:
                    licenses.append({"source": origin["source"], **license_record})
            conflict = observed_conflict(markings, observations, formats)
            label = "conflicting" if conflict["conflicting"] else "unresolved"
            fingerprint = ppdeep.hash_from_file(record["path"])
            origins = export_records([record])[0]["origins"]
            annotation = {
                "validation": observe(
                    record["path"],
                    sha,
                    hints=set(discovery.get(sha, [])) | ({prior[sha]} if sha in prior else set()),
                ),
                "format_ids": [],
                "label_status": label,
                "conflicting": conflict["conflicting"],
                "hard_case": conflict["conflicting"],
                "conflict_evidence": conflict,
                "discovery_format_ids": sorted(discovery.get(sha, set())),
                "existing_accepted_format_ids": [prior[sha]] if sha in prior else [],
                "verdicts": observations,
                "vt_markings_history": markings,
                "repository_licenses": licenses,
                "content_fingerprint": {"ssdeep": fingerprint},
                "scope": "Unreconciled candidate; bucket format_id is storage grouping, never a label",
            }
            annotation = apply_label(annotation, annotation["validation"], formats)
            label = annotation["label_status"]
            bucket = number // 65535
            kind = f"__candidate_pool_{bucket:03d}__"
            sample_buffer.append(
                {
                    "class_ordinal": bucket,
                    "sample_ordinal": counts[bucket],
                    "format_id": kind,
                    "sha256": bytes.fromhex(sha),
                    "size": record["size"],
                    "origins": origins,
                    "hard_case": conflict["conflicting"],
                    "annotation_json": json.dumps(annotation, sort_keys=True),
                }
            )
            counts[bucket] += 1
            statuses.update([label])
            review_buffer.append(
                {
                    "sha256": bytes.fromhex(sha),
                    "size": record["size"],
                    "origins": origins,
                    "discovery_format_ids": annotation["discovery_format_ids"],
                    "existing_accepted_format_ids": annotation["existing_accepted_format_ids"],
                    "label_status": label,
                    "assigned_format_ids": annotation["format_ids"],
                    "validation_status": annotation["validation_status"],
                    "validation_json": json.dumps(annotation["validation"], sort_keys=True),
                    "conflicting": conflict["conflicting"],
                    "hard_case": conflict["conflicting"],
                    "ssdeep": fingerprint,
                    "verdicts_json": json.dumps(observations, sort_keys=True),
                    "vt_markings_json": json.dumps(markings, sort_keys=True),
                    "conflict_json": json.dumps(conflict, sort_keys=True),
                    "repository_licenses_json": json.dumps(licenses, sort_keys=True),
                }
            )
            if len(sample_buffer) >= 128:
                sample_writer.write_table(pa.Table.from_pylist(sample_buffer, schema=SAMPLE_SCHEMA))
                review_writer.write_table(pa.Table.from_pylist(review_buffer, schema=review_schema))
                sample_buffer.clear()
                review_buffer.clear()
        if sample_buffer:
            sample_writer.write_table(pa.Table.from_pylist(sample_buffer, schema=SAMPLE_SCHEMA))
            review_writer.write_table(pa.Table.from_pylist(review_buffer, schema=review_schema))
    for bucket, count in sorted(counts.items()):
        classes.append(
            {
                "ordinal": bucket,
                "format_id": f"__candidate_pool_{bucket:03d}__",
                "name": "Unreconciled candidate pool",
                "categories": ["unreviewed"],
                "extensions": [],
                "metadata_json": json.dumps(
                    {
                        "class_role": "candidate_pool",
                        "counts": {"samples": count, "target": count},
                        "scope": "Physical partition only; canonical taxonomy is taxonomy.parquet",
                    }
                ),
            }
        )
    pq.write_table(
        pa.Table.from_pylist(classes, CLASS_SCHEMA), output / "classes.parquet", compression="zstd"
    )
    samples_path.with_suffix(".next").replace(samples_path)
    review_path.with_suffix(".next").replace(review_path)
    assert (
        pq.ParquetFile(samples_path).metadata.num_rows
        == pq.ParquetFile(review_path).metadata.num_rows
        == len(records)
    )
    export_sources(samples_path, output / "github-used.parquet")
    receipt = {
        "schema_version": 1,
        "samples": len(records),
        "classes": len(classes),
        "label_status_counts": dict(statuses),
        "tool_status_counts": dict(tool_statuses),
        "completed_observations": sum(tool_statuses.values()),
        "metadata_scope": "Unreconciled candidates with eight tool observations; no accepted labels assigned",
        "files": {
            name: {"sha256": sha256_file(output / name), "bytes": (output / name).stat().st_size}
            for name in (
                "samples.parquet",
                "classes.parquet",
                "candidate-index.parquet",
                "github-used.parquet",
            )
        },
    }
    write(output / "corpus-parquet-receipt.json", receipt)
    return receipt
