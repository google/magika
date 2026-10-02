# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Draw a balanced evaluation subset from the corpus, and record what it left out.

The corpus holds everything collected, including samples nothing has verified and classes
far past their target. An evaluation set wants neither: it wants a stated number of
verified samples per class, drawn from enough distinct repositories that a single project's
house style cannot stand in for a format.
"""

import argparse
import json
from collections import Counter, defaultdict
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

from .duplicates import near_duplicate
from .parquet_metadata import SAMPLE_SCHEMA
from .repository_diversity import github_repositories
from .runtime import private_output, write
from .validation import PRECEDENCE

PER_CLASS = 100
MAX_PER_REPOSITORY = 10
THRESHOLD = 90
VERIFIED = frozenset(PRECEDENCE)


class Selector:
    """The per-row decision behind a selection, usable while rows are still arriving.

    Refill asks the same question as selection -- would this sample count towards its
    class? -- so both share this instead of each counting rows its own way.
    """

    def __init__(
        self, *, per_class=PER_CLASS, max_per_repository=MAX_PER_REPOSITORY, threshold=THRESHOLD
    ):
        if per_class < 1:
            raise ValueError("per_class must be at least one sample")
        self.per_class, self.max_per_repository, self.threshold = (
            per_class,
            max_per_repository,
            threshold,
        )
        self.chosen: dict[str, list[dict]] = defaultdict(list)
        self.repositories: dict[str, Counter] = defaultdict(Counter)
        self.excluded, self.duplicates = Counter(), []

    def offer(self, row: dict, per_class: int | None = None) -> str | None:
        """Take the row, or return why not: unverified, class_full, repository_cap or near_duplicate."""
        kind = row["format_id"]
        reason = None
        repos = github_repositories(row)
        if row["label_status"] not in VERIFIED:
            reason = "unverified"
        elif len(self.chosen[kind]) >= (self.per_class if per_class is None else per_class):
            reason = "class_full"
        elif repos and min(self.repositories[kind][n] for n in repos) >= self.max_per_repository:
            reason = "repository_cap"
        else:
            annotation = json.loads(row["annotation_json"])
            candidate = {
                "sha256": row["sha256"].hex(),
                "size": row["size"],
                "format_ids": annotation.get("format_ids") or [kind],
                **{k: annotation[k] for k in ("content_fingerprint",) if k in annotation},
            }
            match = near_duplicate(candidate, self.chosen[kind], threshold=self.threshold)
            if match:
                reason = "near_duplicate"
                self.duplicates.append(match)
            else:
                self.chosen[kind].append(candidate)
                for name in repos:
                    self.repositories[kind][name] += 1
        if reason:
            self.excluded[reason] += 1
        return reason


def select(
    samples, *, per_class=PER_CLASS, max_per_repository=MAX_PER_REPOSITORY, threshold=THRESHOLD
):
    """Rows to evaluate on, plus a receipt naming every exclusion and its reason.

    Reading in the table's own (class_ordinal, sample_ordinal) order makes the draw
    deterministic without a seed, so the same metadata always yields the same set.
    """
    selector = Selector(
        per_class=per_class, max_per_repository=max_per_repository, threshold=threshold
    )
    rows = []
    for batch in pq.ParquetFile(samples).iter_batches():
        for row in batch.to_pylist():
            if selector.offer(row) is None:
                rows.append(row)
    chosen, excluded, duplicates = selector.chosen, selector.excluded, selector.duplicates
    receipt = {
        "selected": len(rows),
        "classes": len(chosen),
        "per_class": per_class,
        "max_per_repository": max_per_repository,
        "selected_by_class": {k: len(v) for k, v in sorted(chosen.items())},
        "excluded": dict(sorted(excluded.items())),
        "near_duplicates": duplicates,
        "policy": {
            "eligible_label_statuses": list(PRECEDENCE),
            "near_duplicates": {
                "method": "ssdeep",
                "threshold": threshold,
                "minimum_size_ratio": 0.8,
                "scope": "within class, against the representatives already selected",
            },
            "order": "table order; the draw is deterministic and takes no seed",
        },
    }
    return rows, receipt


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, prog="magika-datasets select")
    parser.add_argument("--samples", type=Path, default=Path("samples.parquet"))
    parser.add_argument("--per-class", type=int, default=PER_CLASS)
    parser.add_argument("--max-per-repository", type=int, default=MAX_PER_REPOSITORY)
    parser.add_argument("--output", type=Path, default=Path("selection.parquet"))
    parser.add_argument("--receipt", type=Path, default=Path("selection-receipt.json"))
    args = parser.parse_args(argv)
    rows, receipt = select(
        args.samples, per_class=args.per_class, max_per_repository=args.max_per_repository
    )
    pq.write_table(
        pa.Table.from_pylist(rows, schema=SAMPLE_SCHEMA),
        private_output(args.output),
        compression="zstd",
    )
    write(args.receipt, receipt)
    print(json.dumps({k: v for k, v in receipt.items() if k != "near_duplicates"}, sort_keys=True))
