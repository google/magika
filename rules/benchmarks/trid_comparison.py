# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Derive a paired TrID/StringZilla comparison from one saved benchmark run."""

import argparse
from pathlib import Path

from full_table import COUNTS, observations, result_file, write_receipt
from magika_rules_benchmark import comparison as c
from magika_rules_benchmark import corpus
from magika_rules_benchmark.identity import source_dataset, tool_record
from magika_rules_benchmark.trid import enabled
from startup_comparison import timing_change


def validate_pair(result):
    if result["status"] != "complete":
        raise ValueError("TrID comparison requires a complete run")
    c.validate_default_policy(result["config"])
    ids = ("trid", "trid-stringzilla")
    specs = {t["id"]: t for t in result["config"]["tools"]}
    a, b = [result["tools"][i] for i in ids]
    if enabled(a["version"]) is not False or enabled(b["version"]) is not True:
        raise ValueError("TrID comparison requires observed disabled and enabled acceleration")
    if not b.get("stringzilla"):
        raise ValueError("Missing loaded StringZilla identity")
    if a["executable_sha256"] != b["executable_sha256"]:
        raise ValueError("Changed Python interpreter build")
    for artifact in ("script", "definitions"):
        if a["artifacts"][artifact] != b["artifacts"][artifact]:
            raise ValueError(f"Changed TrID {artifact}")
    controls = []
    for i in ids:
        control = c.command_settings(specs[i])
        control.pop("id")
        control["settings"] = {k: v for k, v in control["settings"].items() if k != "stringzilla"}
        controls.append(control)
    if controls[0] != controls[1]:
        raise ValueError("Changed TrID command or settings beyond acceleration")
    return ids


def derive(source, destination):
    result = c.load_json(result_file(source))
    ids = validate_pair(result)
    rows = observations(source)
    samples = sorted(c.load_json(source / "inputs.json.gz")["samples"], key=lambda s: s["sha256"])
    differences = [
        {"sha256": sample["sha256"], "without": before, "with": after}
        for sample, before, after in zip(samples, rows[ids[0]], rows[ids[1]], strict=True)
        if before != after
    ]
    times = {
        (m["tool"], m["file_count"]): m
        for m in result["measurements"]
        if m["requested_rule_hit_percent"] is None
    }
    summary = {
        "schema": 1,
        "dataset": source_dataset(source, result),
        "measured_at_utc": result["created_at"],
        "results_sha256": corpus.file_hash(result_file(source)),
        "benchmark_version": result["benchmark_version"],
        "tools": {i: tool_record(result, i) for i in ids},
        "matched_observations": len(samples) - len(differences),
        "files": len(samples),
        "differences": differences,
        "timings": {},
    }
    for count in COUNTS:
        before, after = [times[i, count] for i in ids]
        delta = timing_change(before, after)
        summary["timings"][str(count)] = delta | {"speedup": delta["before_ms"] / delta["after_ms"]}
    destination.mkdir(parents=True, exist_ok=True)
    corpus.atomic_json(destination / "trid-comparison.json", summary)
    d = summary["dataset"]
    lines = [
        "# TrID StringZilla comparison",
        "",
        f"Dataset: **{d['name']}**, version `{d['version']}`. Measured at **{result['created_at']}**.",
        "",
        "Both configurations use the same Python executable bytes, TrID script, definitions, "
        "input files and default resource policy. The accelerated configuration uses a Python "
        "virtual environment containing the verified StringZilla library; its startup is included. "
        "One warmup and three measured runs per configuration/workload, with shuffled execution order.",
        "",
        f"Normalized observations agree on **{summary['matched_observations']:,} / {len(samples):,} files**. "
        "Quality scores and exact identities are in the [full table](overview.md); any differences are retained in JSON.",
        "",
        "Whole-process median milliseconds, with warm OS caches. Speedup is the unaccelerated median divided by the accelerated median.",
        "",
        "| Files | TrID without StringZilla | TrID with StringZilla | Speedup | Time change |",
        "|---:|---:|---:|---:|---:|",
    ]
    for count in COUNTS:
        t = summary["timings"][str(count)]
        lines.append(
            f"| {count:,} | {t['before_ms']:.2f} | {t['after_ms']:.2f} | {t['speedup']:.2f}× | {t['change_percent']:+.1f}% |"
        )
    (destination / "trid-comparison.md").write_text("\n".join(lines) + "\n")
    write_receipt(destination, result_file(source), Path(__file__))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    derive(args.source, args.destination)
