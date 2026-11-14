# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Fast, read-only corpus statistics from typed Parquet metadata; no payload reads."""

import argparse
import json
from collections import Counter, defaultdict
from pathlib import Path

import pyarrow.parquet as pq

from .parquet_metadata import LABEL_STATUSES
from .repository_diversity import github_repositories
from .validation import PRECEDENCE

VERIFIED = PRECEDENCE
"""Default counting basis: a per-class count that mixes verified and unreviewed samples
answers no question anyone asks of an evaluation corpus."""


def statuses(requested) -> tuple[str, ...]:
    if not requested:
        return VERIFIED
    if "all" in requested:
        return tuple(sorted(LABEL_STATUSES))
    unknown = set(requested) - LABEL_STATUSES
    if unknown:
        raise ValueError("Unknown label status: " + ", ".join(sorted(unknown)))
    return tuple(sorted(set(requested)))


def summarize(root, formats=(), label_statuses=()):
    classes = pq.read_table(
        root / "classes.parquet", columns=["format_id", "name", "categories", "metadata_json"]
    ).to_pylist()
    ids = [r["format_id"] for r in classes]
    if len(ids) != len(set(ids)):
        raise ValueError("Duplicate class IDs in metadata")
    unknown = set(formats) - set(ids)
    if unknown:
        raise ValueError("Unknown filetype: " + ", ".join(sorted(unknown)))
    selected = set(formats) if formats else set(ids)
    counted = set(statuses(label_statuses))
    counts, sizes, hard = Counter(), Counter(), Counter()
    per_status = defaultdict(Counter)
    providers = defaultdict(Counter)
    repositories = defaultdict(set)
    hashes, seen = set(), set()
    columns = ["format_id", "label_status", "sha256", "size", "origins", "hard_case"]
    for batch in pq.ParquetFile(root / "samples.parquet").iter_batches(columns=columns):
        for row in batch.to_pylist():
            kind = row["format_id"]
            if kind not in ids:
                raise ValueError("Sample refers to an unknown class: " + kind)
            if kind not in selected:
                continue
            # The full breakdown is recorded whatever the filter, so `missing` stays
            # interpretable: a class can be short of target and still hold 20,000 files.
            per_status[kind][row["label_status"]] += 1
            if row["label_status"] not in counted:
                continue
            key = (kind, row["sha256"])
            if key in seen:
                raise ValueError("Duplicate SHA within class: " + kind)
            seen.add(key)
            hashes.add(row["sha256"])
            counts[kind] += 1
            sizes[kind] += row["size"]
            hard[kind] += bool(row["hard_case"])
            origins = row["origins"]
            gh = any(o.startswith("github:") for o in origins)
            vt = any(o.startswith("vt:") for o in origins)
            providers[kind][
                "both" if gh and vt else "github_only" if gh else "vt_only" if vt else "other"
            ] += 1
            repositories[kind].update(github_repositories({"origins": origins}))
    rows = []
    categories = defaultdict(lambda: {"classes": 0, "samples": 0, "bytes": 0})
    for row in classes:
        kind = row["format_id"]
        if kind not in selected:
            continue
        target = json.loads(row["metadata_json"])["counts"]["target"]
        if type(target) is not int or target < 0:
            raise ValueError("Invalid class target: " + kind)
        entry = {
            "format_id": kind,
            "name": row["name"],
            "categories": row["categories"],
            "samples": counts[kind],
            "target": target,
            "missing": max(0, target - counts[kind]),
            "bytes": sizes[kind],
            "hard_cases": hard[kind],
            "label_status_counts": dict(sorted(per_status[kind].items())),
            "github_repositories": len(repositories[kind]),
            "sources": {
                key: providers[kind][key] for key in ("github_only", "vt_only", "both", "other")
            },
        }
        rows.append(entry)
        for category in set(row["categories"]):
            categories[category]["classes"] += 1
            categories[category]["samples"] += counts[kind]
            categories[category]["bytes"] += sizes[kind]
    source_counts = Counter()
    for row in rows:
        source_counts.update(row["sources"])
    summary = {
        "samples": sum(counts.values()),
        "unique_sha256": len(hashes),
        "bytes": sum(sizes.values()),
        "classes": len(rows),
        "full_classes": sum(r["samples"] >= r["target"] for r in rows),
        "partial_classes": sum(0 < r["samples"] < r["target"] for r in rows),
        "empty_classes": sum(r["samples"] == 0 for r in rows),
        "target": sum(r["target"] for r in rows),
        "missing": sum(r["missing"] for r in rows),
        "hard_cases": sum(hard.values()),
        "label_status_counts": dict(
            sorted(Counter(k for c in per_status.values() for k in c.elements()).items())
        ),
        "sources": dict(source_counts),
        "github_repositories": len(set().union(*repositories.values())) if repositories else 0,
    }
    return {
        "schema_version": 1,
        "counted_label_statuses": list(statuses(label_statuses)),
        "scope": (
            "Counts include only the label statuses listed in counted_label_statuses; "
            "label_status_counts gives the full breakdown"
        ),
        "metadata_dir": str(root.resolve()),
        "summary": summary,
        "categories": dict(sorted(categories.items())),
        "classes": sorted(rows, key=lambda r: r["format_id"]),
        "category_counts_overlap": True,
        "payload_bytes_read": 0,
    }


def human_bytes(size):
    return f"{size / 1024**3:.2f} GiB" if size >= 1024**3 else f"{size / 1024**2:.2f} MiB"


def render(report, *, by_class=False):
    s = report["summary"]
    basis = ", ".join(report["counted_label_statuses"])
    lines = [
        f"Counting {basis}: {s['samples']:,} samples / {s['target']:,} target; {s['missing']:,} missing",
        f"Classes: {s['full_classes']} full, {s['partial_classes']} partial, {s['empty_classes']} empty ({s['classes']} total)",
        f"Whole-file bytes: {s['bytes']:,} ({human_bytes(s['bytes'])}); hard cases: {s['hard_cases']:,}",
        f"Unique SHA-256: {s['unique_sha256']:,}; GitHub repositories: {s['github_repositories']:,}",
        "Sources (exclusive): " + ", ".join(f"{k}={v:,}" for k, v in s["sources"].items()),
    ]
    lines.append(
        "All statuses: " + ", ".join(f"{k}={v:,}" for k, v in s["label_status_counts"].items())
    )
    if by_class:
        lines += [
            "",
            f"{'FILETYPE':30} {'COUNT/TARGET':>13} {'MISSING':>8} {'HELD':>8} {'BYTES':>12} {'HARD':>6} {'REPOS':>6}",
        ]
        for r in report["classes"]:
            count = f"{r['samples']}/{r['target']}"
            held = sum(r["label_status_counts"].values())
            lines.append(
                f"{r['format_id']:30} {count:>13} {r['missing']:8} {held:8} {r['bytes']:12,} {r['hard_cases']:6} {r['github_repositories']:6}"
            )
    else:
        lines += ["", "Categories (may overlap):"]
        for name, row in report["categories"].items():
            lines.append(f"  {name:28} {row['samples']:7,} samples  {row['classes']:3} classes")
    return "\n".join(lines)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, prog="magika-datasets stats")
    parser.add_argument("--metadata-dir", type=Path, default=Path("."))
    parser.add_argument(
        "--by-class", action="store_true", help="List every filetype, including empty classes"
    )
    parser.add_argument(
        "--format", action="append", default=[], help="Filter by exact filetype ID; repeatable"
    )
    parser.add_argument(
        "--json", action="store_true", help="Emit summary and all per-class statistics as JSON"
    )
    parser.add_argument(
        "--label-status",
        action="append",
        default=[],
        help="Count only these label statuses; 'all' for every one. Repeatable. "
        "Defaults to the verified statuses.",
    )
    args = parser.parse_args(argv)
    try:
        result = summarize(args.metadata_dir, args.format, args.label_status)
    except (OSError, ValueError, KeyError) as error:
        parser.error(str(error))
    print(
        json.dumps(result, sort_keys=True)
        if args.json
        else render(result, by_class=args.by_class or bool(args.format))
    )


if __name__ == "__main__":
    main()
