# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Keep only samples the corpus policy allows: redistributable, and not one project's style.

A sample drawn from a GitHub repository carries that repository's terms, so it stays only
when the licence is established and permitted by config/repository-license-policy.json. A
repository whose licence could not be established is dropped as well: the policy says to
report it and keep it out until reviewed, and an unestablished licence is not a permissive
one.

VirusTotal origins are permitted by the corpus policy and assert no repository licence, so
a sample acquired there is unaffected. What decides the terms is where the bytes came from,
not which mirror served them, so a file also present on VirusTotal does not escape the
licence of the repository it was taken from.

A class also holds at most `max_per_repository` samples from any one repository, so a single
project's conventions cannot stand in for a format. The earliest members in table order are
kept, which keeps the established ones.
"""

import argparse
import json
from collections import Counter, defaultdict
from pathlib import Path

import pyarrow.parquet as pq

from .parquet_metadata import MAX_SAMPLE_BYTES, write_samples
from .receipt import refresh
from .repository_diversity import github_repositories
from .runtime import private_output

BATCH = 128
MAX_PER_REPOSITORY = 10
UNKNOWN = "unknown"


def standing(licences: dict) -> tuple[set[str], set[str]]:
    """Repositories the corpus may redistribute from, and those it may not."""
    allowed = {name for name, row in licences.items() if row["allowed_by_policy"]}
    return allowed, set(licences) - allowed


def prune(metadata: Path, max_per_repository: int = MAX_PER_REPOSITORY) -> dict:
    metadata = Path(metadata)
    licences = {
        r["repository"]: r
        for r in pq.read_table(metadata / "repository-licenses.parquet").to_pylist()
    }
    allowed, refused = standing(licences)
    formats = {r["format_id"]: r for r in pq.read_table(metadata / "classes.parquet").to_pylist()}
    grouped: dict[str, list[dict]] = defaultdict(list)
    removed, removed_spdx = Counter(), Counter()
    before, after = Counter(), Counter()
    per_repo: dict[str, Counter] = defaultdict(Counter)
    samples = metadata / "samples.parquet"
    for batch in pq.ParquetFile(samples).iter_batches(batch_size=BATCH):
        for row in batch.to_pylist():
            kind = row["format_id"]
            before[kind] += 1
            if row["size"] > MAX_SAMPLE_BYTES:
                # Admitted but unpublishable: hydration refuses a file over the corpus ceiling.
                removed["too_large"] += 1
                continue
            blocked = github_repositories(row) & refused
            if blocked:
                for name in sorted(blocked):
                    spdx = licences[name]["spdx_id"]
                    removed["outside_policy" if spdx else "unresolved"] += 1
                    removed_spdx[spdx or "unresolved"] += 1
                continue
            repositories = github_repositories(row)
            # unknown holds samples nothing verified, not a format, so there is no house style
            # to keep from standing in for one.
            capped = kind != UNKNOWN and any(
                per_repo[kind][name] >= max_per_repository for name in repositories
            )
            if capped:
                removed["repository_cap"] += 1
                continue
            for name in repositories:
                per_repo[kind][name] += 1
            after[kind] += 1
            grouped[kind].append(row)
    write_samples(grouped, formats, samples)
    refresh(metadata)
    return {
        "samples_before": sum(before.values()),
        "samples_after": sum(after.values()),
        "removed": dict(sorted(removed.items())),
        "removed_spdx": dict(sorted(removed_spdx.items())),
        "classes_emptied": sorted(k for k in before if not after[k]),
        "by_class": {
            k: {"before": before[k], "after": after[k]}
            for k in sorted(before)
            if before[k] != after[k]
        },
        "allowed_repositories": len(allowed),
        "refused_repositories": len(refused),
        "max_per_repository": max_per_repository,
        "policy": (
            "A GitHub sample keeps its repository's terms; VirusTotal origins are permitted "
            "by the corpus policy and assert no repository licence."
        ),
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, prog="magika-datasets prune")
    parser.add_argument("--metadata", type=Path, default=Path("."))
    parser.add_argument("--max-per-repository", type=int, default=MAX_PER_REPOSITORY)
    parser.add_argument("--receipt", type=Path, default=Path("prune-receipt.json"))
    args = parser.parse_args(argv)
    result = prune(args.metadata, args.max_per_repository)
    private_output(args.metadata / "samples.parquet")
    Path(args.receipt).write_text(json.dumps(result, indent=1, sort_keys=True) + "\n")
    print(json.dumps({k: v for k, v in result.items() if k != "by_class"}, sort_keys=True))
