# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Regenerate the catalogue and reports from stored measurements."""

import argparse
from pathlib import Path

import full_table
from magika_rules_benchmark import comparison as c
from magika_rules_benchmark import corpus
from magika_rules_benchmark.identity import index_entry
from startup_comparison import compare
from trid_comparison import derive as trid_compare


def refresh(root):
    index_path = root / "results/v1/index.json"
    index = c.load_json(index_path)
    entries = []
    for old in index["runs"]:
        source = index_path.parent / old["id"]
        result = c.load_json(source / "results.json.gz")
        assert corpus.file_hash(source / "artifacts.json") == old["artifacts_sha256"]
        for name, digest in c.load_json(source / "artifacts.json")["files"].items():
            assert corpus.file_hash(source / name) == digest, source / name
        entry = index_entry(old["id"], result, old["dataset"], old["artifacts_sha256"])
        report = root / "reports" / old["id"]
        full_table.derive(source, report)
        if old.get("comparison_base"):
            entry["comparison_base"] = old["comparison_base"]
            compare(index_path.parent / old["comparison_base"], source, report)
        if "trid-stringzilla" in result["quality"]:
            trid_compare(source, report)
        entries.append(entry)
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
        "Each overview separates Tool, Version and Config. Compressed run JSON retains commands, settings, environment and binary/database hashes; derived JSON can be regenerated locally.",
        "",
    ]
    (root / "CATALOG.md").write_text("\n".join(lines))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).parent)
    args = parser.parse_args()
    refresh(args.root)
