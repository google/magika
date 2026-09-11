# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Revalidate samples into review groups without changing labels or moving bytes."""

import argparse
import json
from collections import Counter
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

from .acquisition import object_path
from .candidate_conflicts import observed_conflict
from .normalization import canonical_type, normalize_annotation
from .runtime import private_output
from .unattended import write
from .validators import observe

STATUSES = {"conflicting", "validated_auto", "unknown", "validated_manual", "llm-validated"}


def detectors_disagree(annotation: dict, formats: dict) -> tuple[bool, set[str]]:
    """Whether detector claims conflict with each other or the label, and what they claim."""
    markings = annotation.get("vt_markings_history", [annotation.get("vt_markings", {})])
    evidence = observed_conflict(markings, annotation.get("verdicts", {}), formats)
    claims = {canonical_type(k) for k in evidence["mapped_claims"].values()}
    disagreement = bool(
        annotation.get("conflicting")
        or annotation.get("label_status") == "conflicting"
        or evidence["conflicting"]
    )
    return disagreement, claims


def proven(validation: dict) -> str | None:
    """The single format a structural validator proved, when nothing else failed."""
    successful = {
        o["format_id"]
        for o in validation["observations"]
        if o["status"] == "pass" and o.get("auto_eligible")
    }
    if len(successful) == 1 and not any(o["status"] == "fail" for o in validation["observations"]):
        return next(iter(successful))
    return None


def review_decision(annotation: dict, formats: dict) -> dict | None:
    """Only attributed reviews with known, explicit labels can settle identity."""
    for field, actor, basis in [
        ("llm_validation", "model", "llm-validated"),
        ("manual_validation", "reviewer", "validated_manual"),
    ]:
        review = annotation.get(field, {})
        labels = review.get("format_ids", [])
        if (
            review.get(actor)
            and review.get("decision") == "validated"
            and review.get("evidence")
            and isinstance(labels, list)
            and labels
            and all(isinstance(label, str) and label in formats for label in labels)
        ):
            return {"basis": basis, "format_ids": sorted(set(labels)), "evidence": review}
    return None


def classify(annotation: dict, validation: dict, formats: dict) -> str:
    """Review group; attributed decisions settle labels and retain hard-case evidence."""
    validated = proven(validation)
    if validated is not None and validated in formats:
        return "validated_auto"
    review = review_decision(annotation, formats)
    if review:
        return review["basis"]
    disagreement, claims = detectors_disagree(annotation, formats)
    successful = {
        o["format_id"]
        for o in validation["observations"]
        if o["status"] == "pass" and o.get("auto_eligible")
    }
    failed = any(o["status"] == "fail" for o in validation["observations"])
    if disagreement or len(claims | successful) > 1 or (successful and failed):
        return "conflicting"
    return "unknown"


def apply_label(annotation: dict, validation: dict, formats: dict) -> dict:
    """Assign supported labels while retaining earlier labels and independent conflicts."""
    annotation = normalize_annotation(annotation)
    result = dict(annotation)
    successful = [
        o
        for o in validation["observations"]
        if o["status"] == "pass" and o.get("auto_eligible") and o["format_id"] in formats
    ]
    kinds = sorted({o["format_id"] for o in successful})
    decision = None
    if len(kinds) == 1 and not any(o["status"] == "fail" for o in validation["observations"]):
        decision = {"basis": "validated_auto", "format_ids": kinds, "evidence": successful}
    if decision is None:
        decision = review_decision(annotation, formats)
    result["validation_status"] = classify(annotation, validation, formats)
    if decision:
        disagreement, claims = detectors_disagree(annotation, formats)
        contested = disagreement or bool(claims - set(decision["format_ids"]))
        result["tags"] = sorted(
            set(result.get("tags", []))
            | {tag for o in successful for tag in o.get("tags", [])}
            | ({"detectors_disagree"} if contested else set())
        )
        result.setdefault("previous_format_ids", annotation.get("format_ids", []))
        result["format_ids"] = decision["format_ids"]
        result["label_decision"] = decision
        result["label_status"] = decision["basis"]
        result["conflicting"] = result["validation_status"] == "conflicting"
        # Detector disagreement on a proven file is exactly the discriminative case a
        # benchmark wants, so it stays a hard case even though the label is settled.
        result["hard_case"] = bool(annotation.get("hard_case") or contested)
    return result


def validate_dataset(metadata: Path, store: Path, output: Path) -> dict:
    taxonomy = metadata / "taxonomy.parquet"
    if not taxonomy.exists():
        taxonomy = metadata / "classes.parquet"
    formats = {r["format_id"]: r for r in pq.read_table(taxonomy).to_pylist()}
    schema = pa.schema(
        [
            ("sha256", pa.binary(32)),
            ("format_id", pa.string()),
            ("validation_status", pa.string()),
            ("validation_json", pa.string()),
            ("assigned_format_ids", pa.list_(pa.string())),
            ("tags", pa.list_(pa.string())),
            ("label_decision_json", pa.string()),
        ]
    )
    counts = Counter()
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_suffix(".next")
    with pq.ParquetWriter(temporary, schema, compression="zstd") as writer:
        for batch in pq.ParquetFile(metadata / "samples.parquet").iter_batches(batch_size=128):
            rows = []
            for row in batch.to_pylist():
                sha = row["sha256"].hex()
                annotation = json.loads(row["annotation_json"])
                hints = set(annotation.get("format_ids", [])) | set(
                    annotation.get("discovery_format_ids", [])
                )
                if not row["format_id"].startswith("__candidate_pool_"):
                    hints.add(row["format_id"])
                report = observe(object_path(store, sha), sha, hints=hints)
                decision = apply_label(annotation, report, formats)
                status = decision["validation_status"]
                counts[status] += 1
                rows.append(
                    {
                        "sha256": row["sha256"],
                        "format_id": row["format_id"],
                        "validation_status": status,
                        "assigned_format_ids": decision.get("format_ids", []),
                        "tags": decision.get("tags", []),
                        "label_decision_json": json.dumps(
                            decision.get("label_decision"), sort_keys=True
                        ),
                        "validation_json": json.dumps(report, sort_keys=True),
                    }
                )
            writer.write_table(pa.Table.from_pylist(rows, schema))
    temporary.replace(output)
    result = {
        "counts": {s: counts[s] for s in sorted(STATUSES)},
        "samples": sum(counts.values()),
        "scope": "Logical review groups; original labels and bytes preserved",
    }
    write(output.with_suffix(".json"), result)
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", type=Path, default=Path("."))
    parser.add_argument("--store", type=Path, default=Path(".local/corpus"))
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    print(
        json.dumps(
            validate_dataset(args.metadata, args.store, private_output(args.output)), sort_keys=True
        )
    )
