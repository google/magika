# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Compare identical natural workloads across the CPU-warmup scheduling change."""

import argparse
from pathlib import Path

from full_table import COUNTS, TOOLS, result_file, write_receipt
from magika_rules_benchmark import comparison as c
from magika_rules_benchmark import corpus
from magika_rules_benchmark.identity import source_dataset, tool_record


def timing_change(before, after):
    for key in ("case", "file_count", "distinct_files", "input_order_sha256", "trials"):
        if before[key] != after[key]:
            raise ValueError(f"Changed measurement input or control: {key}")
    a, b = before["median_seconds"] * 1000, after["median_seconds"] * 1000
    return {"before_ms": a, "after_ms": b, "change_percent": 100 * (b / a - 1)}


def controls(tool):
    result = c.command_settings(tool)
    # These describe the implementation change being measured, not command overrides.
    result["settings"] = {
        k: v
        for k, v in (result["settings"] or {}).items()
        if k not in ("startup_backend", "gpu_admission")
    }
    return result


def compare(previous, current, destination):
    old, new = [c.load_json(result_file(p)) for p in (previous, current)]
    for result in (old, new):
        if result["status"] != "complete":
            raise ValueError("Cannot compare unfinished measurements")
        c.validate_default_policy(result["config"])
        if result["benchmark_version"] not in ("1.2.0", "1.2.1"):
            raise ValueError("This comparison covers the 1.2 timing protocol only")
    for key in ("host", "corpus", "environment", "hyperfine", "protocol", "mappings"):
        if old["compatibility"][key] != new["compatibility"][key]:
            raise ValueError(f"Changed comparison control: {key}")
    old_tools = {t["id"]: t for t in old["config"]["tools"]}
    for tool in new["config"]["tools"]:
        if controls(tool) != controls(old_tools[tool["id"]]):
            raise ValueError(f"Changed tool command or resource policy: {tool['id']}")
    dataset = source_dataset(current, new)
    if dataset != source_dataset(previous, old):
        raise ValueError("Changed dataset identity")
    measurements = [
        {
            (m["tool"], m["file_count"]): m
            for m in r["measurements"]
            if m["requested_rule_hit_percent"] is None
        }
        for r in (old, new)
    ]
    summary = {
        "schema": 1,
        "dataset": dataset,
        "before": {
            "revision": old["revision"],
            "measured_at_utc": old["created_at"],
            "results_sha256": corpus.file_hash(result_file(previous)),
        },
        "after": {
            "revision": new["revision"],
            "measured_at_utc": new["created_at"],
            "results_sha256": corpus.file_hash(result_file(current)),
        },
        "compatibility": "Same host, dataset, exact natural input lists, command flags, default resources, Hyperfine, trials and warmups. Protocol 1.2.1 adds workload prediction validation; the 1.2.0 timing recipe is unchanged. CPU warmup and GPU admission are the implementation changes under test. This scoped comparison does not relax the general history compatibility gate.",
        "rows": [],
    }
    for tool, _, _ in TOOLS:
        if tool not in old["quality"] or tool not in new["quality"]:
            continue
        summary["rows"].append(
            {
                "id": tool,
                "before_tool": tool_record(old, tool),
                "after_tool": tool_record(new, tool),
                "timings": {
                    str(n): timing_change(measurements[0][tool, n], measurements[1][tool, n])
                    for n in COUNTS
                },
                "quality": {
                    side: {
                        k: result["quality"][tool][k]
                        for k in ("accuracy", "precision", "decision_coverage", "errors")
                    }
                    for side, result in [("before", old), ("after", new)]
                },
            }
        )
    destination.mkdir(parents=True, exist_ok=True)
    corpus.atomic_json(destination / "comparison.json", summary)
    lines = [
        "# Revision timing comparison",
        "",
        f"Dataset: **{dataset['name']}**, version `{dataset['version']}`.",
        "",
        f"Before: `{old['revision']}` at {old['created_at']}. After: `{new['revision']}` at {new['created_at']}.",
        "",
        summary["compatibility"],
        "",
        "Each cell is **before → after** median milliseconds, followed by percentage change. Negative changes mean less time. These are recorded observations on a shared host.",
        "",
        "| Tool | Version before → after | Config after | "
        + " | ".join(f"{n:,} files" for n in COUNTS)
        + " |",
        "|---|---|---|" + "---:|" * len(COUNTS),
    ]
    for row in summary["rows"]:
        cells = [
            row["after_tool"]["tool"],
            row["before_tool"]["version"] + " → " + row["after_tool"]["version"],
            row["after_tool"]["config"],
        ]
        for count in COUNTS:
            t = row["timings"][str(count)]
            cells.append(
                f"{t['before_ms']:.2f} → {t['after_ms']:.2f} ({t['change_percent']:+.1f}%)"
            )
        lines.append("| " + " | ".join(cells) + " |")
    lines += [
        "",
        "[Full current accuracy, precision, coverage and rule-match table](overview.md). Both sets of quality scores and all arithmetic above are retained in `comparison.json`.",
        "",
    ]
    (destination / "comparison.md").write_text("\n".join(lines))
    write_receipt(destination, result_file(current), Path(__file__))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("previous", "current", "destination"):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    compare(args.previous, args.current, args.destination)
