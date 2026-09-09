# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Backfill catalogue identities and refresh overview metadata from saved JSON only."""

import argparse
from pathlib import Path

import full_table
from magika_rules_benchmark import comparison as c
from magika_rules_benchmark import corpus
from magika_rules_benchmark.identity import index_entry, report_metadata, tool_record


def refresh(root, descriptors):
    index_path = root / "results/v1/index.json"
    index = c.load_json(index_path)
    entries = []
    for old in index["runs"]:
        source = index_path.parent / old["id"]
        result = c.load_json(source / "results.json.gz")
        descriptor = descriptors[result["compatibility"]["corpus"]]
        assert corpus.file_hash(source / "artifacts.json") == old["artifacts_sha256"]
        for name, digest in c.load_json(source / "artifacts.json")["files"].items():
            assert corpus.file_hash(source / name) == digest, source / name
        entry = index_entry(old["id"], result, descriptor, old["artifacts_sha256"])
        entries.append(entry)
        report = root / "reports" / old["id"]
        if not (report / "overview.json").exists():
            continue
        summary = c.load_json(report / "overview.json")
        summary.update(report_metadata(result, descriptor))
        summary["schema"] = 2
        for row in summary["rows"]:
            row["tool_identity"] = tool_record(result, row["id"])
        corpus.atomic_json(report / "overview.json", summary)
        generator = full_table
        generator.render(report / "overview.json", report / "overview.md")
        receipt = c.load_json(report / "artifacts.json")
        receipt["schema"] = 2
        receipt.pop("generator_sha256", None)
        receipt["code_sha256"] = receipt.get("code_sha256", {}) | {
            name: corpus.file_hash(root / name) for name in ["catalog.py", "full_table.py"]
        }
        receipt["code_sha256"]["../benchmark/src/magika_rules_benchmark/identity.py"] = (
            corpus.file_hash(root.parent / "benchmark/src/magika_rules_benchmark/identity.py")
        )
        receipt["files"] = {name: corpus.file_hash(report / name) for name in receipt["files"]}
        corpus.atomic_json(report / "artifacts.json", receipt)
    corpus.atomic_json(index_path, {"schema": 2, "runs": entries})
    render_catalog(root, entries)


def render_catalog(root, entries):
    lines = [
        "# Benchmark catalogue",
        "",
        "UTC measurement timestamps and dataset versions come from saved evidence. "
        "The adjudicated combined snapshot has a content-addressed version; it is not the full V56 release. "
        "Sembiance v3 records the eligible scoring subset and the complete source size.",
        "",
        "| Dataset | Version | Scored / source files | Measured at (UTC) | Protocol | Revision | Run |",
        "|---|---|---:|---|---|---|---|",
    ]
    for entry in entries:
        d = entry["dataset"]
        report = f"reports/{entry['id']}/overview.md"
        if not (root / report).exists():
            report = f"results/v1/{entry['id']}/report.md"
        lines.append(
            "| "
            + " | ".join(
                [
                    d["name"],
                    f"`{d['version']}`",
                    f"{d['scored_files']:,} / {d['source_files']:,}",
                    entry["measured_at_utc"],
                    entry["benchmark_version"],
                    f"`{entry['revision'][:8]}`",
                    f"[{entry['id']}]({report})",
                ]
            )
            + " |"
        )
    lines += [
        "",
        "The [JSON index](results/v1/index.json) retains full revisions, corpus fingerprints and artifact receipts. "
        "Each overview separates Tool, Version and Config; its JSON retains exact commands, settings, environment and binary/database hashes.",
        "",
    ]
    (root / "CATALOG.md").write_text("\n".join(lines))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).parent)
    parser.add_argument("--datasets", type=Path, default=Path(__file__).parent / "datasets.json")
    args = parser.parse_args()
    refresh(args.root, c.load_json(args.datasets))
