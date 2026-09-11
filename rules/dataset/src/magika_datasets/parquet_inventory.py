# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Verified public Parquet inputs for offline repository sampling."""

import copy
import hashlib
import json
from collections import Counter

import pyarrow.parquet as pq

from . import parquet_metadata
from .parquet_corpus import check_sources
from .repository_diversity import github_repositories
from .repository_pool import charge


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def load(root, policy_path):
    """Return current pool/counts, outcomes, matrix and a streaming file iterator."""
    paths = [
        root / name
        for name in (
            "repositories.parquet",
            "repositories-receipt.json",
            "repository-files.parquet",
            "repository-files-receipt.json",
            "classes.parquet",
            "samples.parquet",
            "corpus-parquet-receipt.json",
        )
    ]
    (
        repositories,
        repo_receipt_path,
        files_path,
        files_receipt_path,
        classes,
        samples,
        corpus_receipt_path,
    ) = paths
    receipt = json.loads(repo_receipt_path.read_text())
    if digest(repositories) != receipt["parquet_sha256"]:
        raise ValueError("Repository Parquet hash differs from receipt")
    if receipt["sweep_complete"] is not True or receipt["unvisited_repositories"] != 0:
        raise ValueError("Inventory crawl is incomplete; finish it before sampling")
    if digest(policy_path) != receipt["policy_sha256"]:
        raise ValueError("License policy differs from inventory policy")
    files_receipt = json.loads(files_receipt_path.read_text())
    if (
        digest(files_receipt_path) != receipt["files_receipt_sha256"]
        or files_receipt["coverage_sha256"] != receipt["coverage_sha256"]
        or files_receipt["journal_prefix"] != receipt["journal_prefix"]
        or files_receipt["parquet_sha256"] != receipt["repository_files_sha256"]
        or digest(files_path) != files_receipt["parquet_sha256"]
    ):
        raise ValueError("File inventory hash/snapshot differs from repository receipt")
    if (
        receipt["round_trip_verified"] is not True
        or files_receipt["round_trip_verified"] is not True
    ):
        raise ValueError("Inventory exports have not been verified")
    table = pq.read_table(repositories)
    if table.schema.metadata[b"coverage_sha256"].decode() != receipt["coverage_sha256"]:
        raise ValueError("Repository schema describes another snapshot")
    sources, outcomes, seen = [], [], set()
    for ordinal, row in enumerate(table.to_pylist()):
        source, outcome = (
            json.loads(row["source_json"]),
            json.loads(row["outcome_json"]) if row["outcome_json"] else None,
        )
        name = row["repository"]
        if (
            row["pool_ordinal"] != ordinal
            or name.lower() in seen
            or outcome is None
            or source["repository"] != name
            or outcome["repository"].lower() != name.lower()
            or row["status"] != outcome["status"]
            or row["eligible"] != outcome.get("eligible")
            or row["status"] == "unvisited"
        ):
            raise ValueError("Incomplete or inconsistent repository rows")
        seen.add(name.lower())
        sources.append(source)
        outcomes.append({**outcome, "repository": name})
    counts = dict(Counter(r["status"] for r in outcomes))
    if (
        len(seen) != receipt["repositories"]
        or len(seen) != receipt["recorded_repositories"]
        or counts != receipt["status_counts"]
    ):
        raise ValueError("Repository coverage/count mismatch")
    corpus_receipt = json.loads(corpus_receipt_path.read_text())
    check_sources(samples, classes, corpus_receipt)
    matrix = parquet_metadata.restore(classes, samples)
    if parquet_metadata.digest(parquet_metadata.encode(matrix)) != corpus_receipt["matrix_sha256"]:
        raise ValueError("Reconstructed matrix differs from receipt")
    pool = copy.deepcopy(sources)
    by_name = {row["repository"].lower(): row for row in pool}
    for source in pool:
        source["counts"] = {"total": 0, "classes": {}, "categories": {}}
    for row in matrix:
        for sample in row["samples"]:
            for name in github_repositories(sample):
                if name in by_name:
                    charge(by_name[name], row, 1)
    metadata = {row["repository"]: row for row in outcomes if row["status"] == "inventoried"}
    if len(metadata) != files_receipt["inventoried_repositories"]:
        raise ValueError("Inventoried repository count differs from file receipt")

    def files():
        counts, total = Counter(), 0
        for batch in pq.ParquetFile(files_path).iter_batches(batch_size=16384):
            for row in batch.to_pylist():
                name = row["repository"]
                meta = metadata.get(name)
                if meta is None:
                    raise ValueError("File entry has no inventoried repository")
                for key in [
                    "revision",
                    "spdx_id",
                    "license_permalink",
                    "license_path",
                    "license_git_oid",
                ]:
                    value = row[key].hex() if isinstance(row[key], bytes) else row[key]
                    if value != meta[key]:
                        raise ValueError("File entry repository/license identity mismatch")
                counts[name] += 1
                total += 1
                yield (
                    name,
                    {
                        key: value.hex() if isinstance(value, bytes) else value
                        for key, value in row.items()
                        if key
                        in {
                            "path",
                            "size",
                            "extension",
                            "git_oid",
                            "git_hash_algorithm",
                            "mode",
                            "type",
                            "regular_file",
                        }
                    },
                )
        if total != files_receipt["rows"] or any(
            counts[name] != row["file_entries"] for name, row in metadata.items()
        ):
            raise ValueError("File inventory is missing repository entries")

    return (
        {"repositories": pool, "source_matrix_sha256": corpus_receipt["matrix_sha256"]},
        outcomes,
        {row["format_id"]: row for row in matrix},
        files(),
        paths,
    )
