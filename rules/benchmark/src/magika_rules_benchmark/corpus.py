# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Verify supplied whole-file Parquet data and expose ordinary disk inputs."""

import hashlib
import json
from collections import defaultdict
from pathlib import Path

import pyarrow.parquet as pq


def file_hash(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def atomic_json(path, value):
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")
    temporary.replace(path)


def class_mapping(classes, model_config):
    aliases = defaultdict(set)
    for name, row in classes.items():
        metadata = row.get("magika", {})
        for label in {name, *metadata.get("output_labels", []), *metadata.get("kb_labels", [])}:
            aliases[label].add(name)
    outputs = {
        model_config.get("overwrite_map", {}).get(label, label)
        for label in model_config["target_labels_space"]
    }
    if any(len(aliases[label]) != 1 for label in outputs):
        raise ValueError("Model output labels must map unambiguously to supplied class metadata")
    canonical = {alias: next(iter(names)) for alias, names in aliases.items() if len(names) == 1}
    return canonical, sorted({canonical[label] for label in outputs})


def prepare(dataset, output):
    dataset, output = Path(dataset).resolve(), Path(output).resolve()
    spec_bytes = (dataset / "manifest.json").read_bytes()
    spec = json.loads(spec_bytes)
    if spec.get("format") != "parquet-whole-file-v1":
        raise ValueError("Expected a hydrated whole-file Parquet corpus")
    class_path = dataset / "classes.parquet"
    expected = spec["source_metadata_receipt"]["files"]["classes.parquet"]["sha256"]
    if file_hash(class_path) != expected:
        raise ValueError("Class metadata hash mismatch")
    classes = {}
    for batch in pq.ParquetFile(class_path).iter_batches(batch_size=64, use_threads=False):
        for row in batch.to_pylist():
            name = row["format_id"]
            if name in classes:
                raise ValueError("Duplicate class identifier")
            classes[name] = json.loads(row["metadata_json"]) | {
                key: row[key] for key in ("format_id", "name", "categories", "extensions")
            }
    listed = [shard["path"] for shard in spec["shards"]]
    actual = {str(path.relative_to(dataset)) for path in (dataset / "shards").glob("*.parquet")}
    if len(listed) != len(set(listed)) or set(listed) != actual:
        raise ValueError("Shard inventory mismatch")
    files = output / "files"
    files.mkdir(parents=True, exist_ok=True)
    records, seen, total = [], set(), 0
    metadata_hash = hashlib.sha256()
    for shard in spec["shards"]:
        path = (dataset / shard["path"]).resolve()
        if not path.is_relative_to(dataset / "shards") or file_hash(path) != shard["sha256"]:
            raise ValueError("Shard path or hash mismatch")
        count = 0
        for batch in pq.ParquetFile(path).iter_batches(batch_size=16, use_threads=False):
            for row in batch.to_pylist():
                content = row.pop("content")
                digest = row["sha256"].hex()
                if digest in seen or hashlib.sha256(content).hexdigest() != digest:
                    raise ValueError("Duplicate content or whole-file hash mismatch")
                if len(content) != row["size"]:
                    raise ValueError("Whole-file size mismatch")
                seen.add(digest)
                line = {k: v.hex() if isinstance(v, bytes) else v for k, v in row.items()}
                metadata_hash.update(
                    (json.dumps(line, sort_keys=True, separators=(",", ":")) + "\n").encode()
                )
                annotation = json.loads(row["annotation_json"])
                formats = annotation["format_ids"]
                if row["format_id"] not in formats or any(name not in classes for name in formats):
                    raise ValueError("Sample label disagrees with annotation or class metadata")
                ambiguous = bool(annotation.get("ambiguous") or len(formats) != 1)
                accepted = annotation.get("label_status") == "accepted" and annotation.get(
                    "properties", {}
                ).get("artifact_layout") == {"state": "known", "value": "single_file"}
                target = files / digest
                if target.exists():
                    if file_hash(target) != digest:
                        raise ValueError("Previously materialized input was modified")
                else:
                    target.write_bytes(content)
                records.append(
                    dict(
                        sha256=digest,
                        size=len(content),
                        path=str(target),
                        truth=formats[0] if accepted and not ambiguous else None,
                        ambiguous=ambiguous,
                        split=annotation.get("split", "development"),
                        group=annotation.get("group", row["format_id"]),
                        provenance=row["origins"],
                    )
                )
                count += 1
                total += len(content)
        if count != shard["samples"]:
            raise ValueError("Shard sample count mismatch")
    if (
        len(records) != spec["samples"]
        or total != spec["original_bytes"]
        or metadata_hash.hexdigest() != spec["metadata_rows_sha256"]
    ):
        raise ValueError("Corpus count, byte size or metadata digest mismatch")
    identity = dict(
        manifest_sha256=hashlib.sha256(spec_bytes).hexdigest(),
        classes_sha256=expected,
        samples=len(records),
        original_bytes=total,
        shards=len(listed),
    )
    atomic_json(output / "inputs.json", dict(identity=identity, classes=classes, samples=records))
    return classes, records, identity
