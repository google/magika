# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Export the GitHub repositories actually represented in sample metadata."""

import json
import re
from collections import defaultdict
from pathlib import Path
from urllib.parse import urlsplit

import pyarrow as pa
import pyarrow.parquet as pq

SCHEMA = pa.schema(
    [
        ("repository", pa.string()),
        ("url", pa.string()),
        ("revisions", pa.list_(pa.string())),
        ("samples", pa.uint64()),
        ("license_observations_json", pa.string()),
    ]
)


def export_sources(samples, output, pool=None):
    """Counts unique SHA256 per repository; missing license evidence stays explicit."""
    repos = defaultdict(lambda: {"hashes": set(), "revisions": set(), "licenses": set()})
    for batch in pq.ParquetFile(samples).iter_batches(
        columns=["sha256", "origins", "annotation_json"]
    ):
        for row in batch.to_pylist():
            annotation = json.loads(row["annotation_json"])
            for origin in row["origins"]:
                if not origin.startswith("github:"):
                    continue
                url, sha = origin[7:].rsplit(":", 1)
                parts = urlsplit(url)
                match = re.fullmatch(
                    r"/([^/]+/[^/]+)/blob/([0-9a-f]{40}|[0-9a-f]{64})/(.+)", parts.path
                )
                if (
                    parts.scheme != "https"
                    or parts.netloc != "github.com"
                    or not match
                    or sha != row["sha256"].hex()
                ):
                    raise ValueError("Expected a pinned GitHub origin with matching SHA256")
                name, revision, _ = match.groups()
                repo = repos[name.lower()]
                repo["hashes"].add(sha)
                repo["revisions"].add(revision)
                for license_record in annotation.get("repository_licenses", []):
                    if license_record.get("source", "").lower() == "github:" + name.lower():
                        repo["licenses"].add(json.dumps(license_record, sort_keys=True))
    if pool is not None:
        for batch in pq.ParquetFile(pool).iter_batches(
            columns=["repository", "spdx_id", "license_permalink", "revision"]
        ):
            for row in batch.to_pylist():
                if row["repository"].lower() in repos and row["spdx_id"]:
                    revision = row["revision"]
                    record = {
                        "spdx_id": row["spdx_id"],
                        "license_permalink": row["license_permalink"],
                        "revision": revision.hex() if isinstance(revision, bytes) else revision,
                        "basis": "frozen_repository_inventory; may differ from sample revision",
                    }
                    repos[row["repository"].lower()]["licenses"].add(
                        json.dumps(record, sort_keys=True)
                    )
    rows = [
        {
            "repository": name,
            "url": "https://github.com/" + name,
            "revisions": sorted(repo["revisions"]),
            "samples": len(repo["hashes"]),
            "license_observations_json": "[" + ",".join(sorted(repo["licenses"])) + "]",
        }
        for name, repo in sorted(repos.items())
    ]
    output = Path(output)
    temporary = output.with_suffix(".next")
    pq.write_table(pa.Table.from_pylist(rows, SCHEMA), temporary, compression="zstd")
    temporary.replace(output)
    return len(rows)
