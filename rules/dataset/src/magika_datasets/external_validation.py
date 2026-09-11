# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Apply existing validators to an external corpus, retaining source evidence."""

import fcntl
import json
import shutil
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

from .acquisition import object_path, sha256_file
from .external import write
from .parquet_metadata import CLASS_SCHEMA, SAMPLE_SCHEMA
from .validation import apply_label
from .validators import VERSION, observe


def validate_row(row, store, formats, report=None):
    annotation = json.loads(row["annotation_json"])
    # Unmapped source labels remain hints: generic containers must not erase an
    # application subtype just because we have no canonical mapping for it yet.
    hints = set(annotation.get("format_ids", [])) | {row["format_id"]}
    if report is None:
        report = observe(object_path(store, row["sha256"].hex()), row["sha256"].hex(), hints=hints)
    if report["status"] in {"error", "unavailable", "skipped"}:
        raise ValueError(f"Object validation failed: {row['sha256'].hex()}: {report['status']}")
    result = apply_label(annotation, report, formats)
    result["validation"] = report
    result["source_format_ids"] = annotation.get("source_format_ids", annotation["format_ids"])
    validated = result["validation_status"] == "validated_auto"
    changed = validated and result["format_ids"] != annotation["format_ids"]
    result["source_identity_changed"] = bool(changed)
    disagreement = changed and not row["format_id"].startswith("sembiance/")
    result["source_label_disagreement"] = bool(disagreement)
    result["hard_case"] = bool(result.get("hard_case") or disagreement)
    result["conflicting"] = bool(
        result.get("conflicting") or result["validation_status"] == "conflicting"
    )
    result["evaluation_basis"] = "validated_auto" if validated else "reviewed_source_mapping"
    result["evaluation_eligible"] = bool(
        (validated or annotation.get("evaluation_eligible"))
        and result["validation_status"] != "conflicting"
        and not any(o["status"] == "fail" for o in report["observations"])
        and not annotation.get("overlap_existing_sha256")
    )
    kind = result["format_ids"][0] if validated else row["format_id"]
    return {
        **row,
        "format_id": kind,
        "hard_case": result["hard_case"],
        "annotation_json": json.dumps(result, sort_keys=True),
    }


def validate_external(metadata: Path, store: Path, taxonomy: Path, output: Path, workers=4):
    """Resume bounded batches; publish new metadata without modifying the input."""
    output.mkdir(parents=True, exist_ok=True)
    with (output / ".validation.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        return _validate_external(metadata, store, taxonomy, output, workers)


def _validate_external(metadata, store, taxonomy, output, workers):
    if not 1 <= workers <= 8 or metadata.resolve() == output.resolve():
        raise ValueError("Use 1..8 workers and a separate output directory")
    output.mkdir(parents=True, exist_ok=True)
    config = {
        "samples_sha256": sha256_file(metadata / "samples.parquet"),
        "classes_sha256": sha256_file(metadata / "classes.parquet"),
        "taxonomy_sha256": sha256_file(taxonomy),
        "validator_version": VERSION,
    }
    config_path = output / "validation-config.json"
    if config_path.exists() and json.loads(config_path.read_text()) != config:
        raise ValueError("Validation inputs changed; use a new output directory")
    write(config_path, config)
    formats = {r["format_id"]: r for r in pq.read_table(taxonomy).to_pylist()}
    formats.update(
        {r["format_id"]: r for r in pq.read_table(metadata / "classes.parquet").to_pylist()}
    )
    cache = output / ".validation-batches"
    cache.mkdir(exist_ok=True)
    rows = []
    with ThreadPoolExecutor(max_workers=workers) as pool:
        for index, batch in enumerate(
            pq.ParquetFile(metadata / "samples.parquet").iter_batches(batch_size=128)
        ):
            path = cache / f"{index:06d}.parquet"
            if path.exists():
                validated = pq.read_table(path).to_pylist()
                if [r["sha256"] for r in validated] != batch.column("sha256").to_pylist():
                    raise ValueError("Validation cache identity mismatch")
                validated = [
                    validate_row(
                        original,
                        store,
                        formats,
                        json.loads(cached["annotation_json"])["validation"],
                    )
                    for original, cached in zip(batch.to_pylist(), validated)
                ]
            else:
                validated = list(
                    pool.map(lambda r: validate_row(r, store, formats), batch.to_pylist())
                )
                temporary = path.with_suffix(".next")
                pq.write_table(
                    pa.Table.from_pylist(validated, SAMPLE_SCHEMA), temporary, compression="zstd"
                )
                temporary.replace(path)
            rows.extend(validated)
            write(
                output / "validation-progress.json", {"phase": "validating", "completed": len(rows)}
            )
    grouped = defaultdict(list)
    for row in rows:
        grouped[row["format_id"]].append(row)
    classes, samples = [], []
    for ordinal, (kind, members) in enumerate(sorted(grouped.items())):
        classes.append({**formats[kind], "ordinal": ordinal})
        samples.extend(
            {**r, "class_ordinal": ordinal, "sample_ordinal": i}
            for i, r in enumerate(sorted(members, key=lambda r: r["sha256"]))
        )
    for name, records, schema in [
        ("classes", classes, CLASS_SCHEMA),
        ("samples", samples, SAMPLE_SCHEMA),
    ]:
        pq.write_table(
            pa.Table.from_pylist(records, schema), output / f"{name}.parquet", compression="zstd"
        )
    for name in ("mapping.json", "source-inventory.parquet", "import-summary.json"):
        if (metadata / name).exists():
            shutil.copyfile(metadata / name, output / name)
    annotations = [json.loads(r["annotation_json"]) for r in samples]
    eligible = [r for r, a in zip(samples, annotations) if a["evaluation_eligible"]]
    summary = {
        "samples": len(samples),
        "counts": dict(Counter(a["validation_status"] for a in annotations)),
        "eligible_samples": len(eligible),
        "eligible_classes": len({r["format_id"] for r in eligible}),
        "source_label_disagreements": sum(a["source_label_disagreement"] for a in annotations),
        "per_class": {
            k: dict(Counter(json.loads(r["annotation_json"])["validation_status"] for r in v))
            for k, v in sorted(grouped.items())
        },
    }
    write(output / "validation-summary.json", summary)
    write(
        output / "corpus-parquet-receipt.json",
        {
            "schema_version": 1,
            "samples": len(samples),
            "classes": len(classes),
            "files": {
                name: {"sha256": sha256_file(output / name)}
                for name in ("samples.parquet", "classes.parquet")
            },
        },
    )
    write(output / "validation-progress.json", {"phase": "complete", "completed": len(samples)})
    return summary
