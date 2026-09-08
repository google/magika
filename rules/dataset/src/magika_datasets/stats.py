# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Fast, read-only corpus statistics from typed Parquet metadata; no payload reads."""

import argparse
import json
from collections import Counter, defaultdict
from pathlib import Path

import pyarrow.parquet as pq

from .repository_diversity import github_repositories


def summarize(root, formats=()):
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
    counts, sizes, hard = Counter(), Counter(), Counter()
    providers = defaultdict(Counter)
    repositories = defaultdict(set)
    hashes, seen = set(), set()
    columns = ["format_id", "sha256", "size", "origins", "hard_case"]
    for batch in pq.ParquetFile(root / "samples.parquet").iter_batches(columns=columns):
        for row in batch.to_pylist():
            kind = row["format_id"]
            if kind not in ids:
                raise ValueError("Sample refers to an unknown class: " + kind)
            if kind not in selected:
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
        "sources": dict(source_counts),
        "github_repositories": len(set().union(*repositories.values())) if repositories else 0,
    }
    candidate_pool = any(
        json.loads(r["metadata_json"]).get("class_role") == "candidate_pool" for r in classes
    )
    return {
        "schema_version": 1,
        "scope": "unreconciled candidate pool" if candidate_pool else "accepted dataset metadata",
        "metadata_dir": str(root.resolve()),
        "summary": summary,
        "categories": dict(sorted(categories.items())),
        "classes": sorted(rows, key=lambda r: r["format_id"]),
        "category_counts_overlap": True,
        "payload_bytes_read": 0,
    }


def collection_stats(root):
    from .unattended import status

    result = status(root)
    receipt = root / "collection-report.json"
    if receipt.exists():
        report = json.loads(receipt.read_text())
        result["unique_candidate_files"] = report["candidate_rows"]
    result["scope"] = "unreviewed candidate pool; never added to accepted counts"
    return result


def human_bytes(size):
    return f"{size / 1024**3:.2f} GiB" if size >= 1024**3 else f"{size / 1024**2:.2f} MiB"


def render(report, *, by_class=False):
    s = report["summary"]
    lines = [
        f"Accepted dataset: {s['samples']:,} samples / {s['target']:,} target; {s['missing']:,} missing",
        f"Classes: {s['full_classes']} full, {s['partial_classes']} partial, {s['empty_classes']} empty ({s['classes']} total)",
        f"Whole-file bytes: {s['bytes']:,} ({human_bytes(s['bytes'])}); hard cases: {s['hard_cases']:,}",
        f"Unique SHA-256: {s['unique_sha256']:,}; GitHub repositories: {s['github_repositories']:,}",
        "Sources (exclusive): " + ", ".join(f"{k}={v:,}" for k, v in s["sources"].items()),
    ]
    if report["scope"] == "unreconciled candidate pool":
        lines[:2] = [
            f"Unreconciled candidate pool: {s['samples']:,} files; no new accepted labels",
            f"Physical storage groups: {s['classes']}; filetype claims are in candidate-index.parquet",
        ]
    if by_class:
        lines += [
            "",
            f"{'FILETYPE':30} {'COUNT/TARGET':>13} {'MISSING':>8} {'BYTES':>12} {'HARD':>6} {'REPOS':>6}",
        ]
        for r in report["classes"]:
            count = f"{r['samples']}/{r['target']}"
            lines.append(
                f"{r['format_id']:30} {count:>13} {r['missing']:8} {r['bytes']:12,} {r['hard_cases']:6} {r['github_repositories']:6}"
            )
    else:
        lines += ["", "Categories (may overlap):"]
        for name, row in report["categories"].items():
            lines.append(f"  {name:28} {row['samples']:7,} samples  {row['classes']:3} classes")
    if "collection" in report:
        c = report["collection"]
        lines += [
            "",
            f"Unreviewed collection: {c['phase']} (process alive: {c['process_alive']})",
            f"  New downloads: {c.get('new_objects', 0):,}; failed candidates: {c.get('failed_candidates', 0):,}; queued batches: {c.get('pending_batches', 0):,}",
        ]
        if "unique_candidate_files" in c:
            lines.append(
                f"  Candidate pool: {c['unique_candidate_files']:,} unique files; reconciliation pending"
            )
        lines.append("  Candidate counts are separate from the accepted dataset above.")
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
        "--collection", type=Path, help="Include separate status of an unattended run"
    )
    args = parser.parse_args(argv)
    try:
        result = summarize(args.metadata_dir, args.format)
        if args.collection:
            result["collection"] = collection_stats(args.collection)
    except (OSError, ValueError, KeyError) as error:
        parser.error(str(error))
    print(
        json.dumps(result, sort_keys=True)
        if args.json
        else render(result, by_class=args.by_class or bool(args.format))
    )


if __name__ == "__main__":
    main()
