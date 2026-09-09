# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Render a paired small-file experiment from saved benchmark JSON."""

import argparse
from pathlib import Path

from full_table import observations
from magika_rules_benchmark import comparison as c
from magika_rules_benchmark import corpus
from magika_rules_benchmark.identity import (
    report_identity_lines,
    report_metadata,
    tool_cells,
    tool_record,
)


def derive(source, descriptor, output):
    result = c.load_json(source / "results.json.gz")
    assert result["status"] == "complete"
    observed = observations(source)
    assert observed["before"] == observed["after"]
    summary = {
        "schema": 1,
        "report_kind": "small-files",
        "revision": result["revision"],
        **report_metadata(result, descriptor),
        "rows": [],
        "identical_decisions": len(observed["before"]),
        "runs": result["config"]["runs"],
        "warmup": result["config"]["warmup"],
    }
    for tool in ["before", "after"]:
        times = {str(m["file_count"]): m for m in result["measurements"] if m["tool"] == tool}
        assert not result["quality"][tool]["errors"]
        summary["rows"].append(
            {
                "id": tool,
                "tool_identity": tool_record(result, tool),
                "quality": result["quality"][tool],
                "timings": times,
            }
        )
    summary["reductions_percent"] = {}
    for count, before in summary["rows"][0]["timings"].items():
        after = summary["rows"][1]["timings"][count]
        assert before["input_order_sha256"] == after["input_order_sha256"]
        summary["reductions_percent"][count] = 100 * (
            1 - after["median_seconds"] / before["median_seconds"]
        )
    output.mkdir(parents=True, exist_ok=False)
    corpus.atomic_json(output / "overview.json", summary)
    render(output / "overview.json", output / "overview.md")


def render(source, output):
    summary = c.load_json(source)
    counts = sorted(map(int, summary["rows"][0]["timings"]))
    lines = [
        "# Small-file CPU batching",
        "",
        *report_identity_lines(summary),
        f"{summary['identical_decisions']:,} identical normalized decisions, with zero tool errors. "
        f"{summary['runs']} measured runs after {summary['warmup']} warmup. "
        "Whole-process median milliseconds, warm OS caches. "
        "The subset is the union of saved natural workloads; it is not a new full-corpus accuracy evaluation.",
        "",
        "| Tool | Version | Config | " + " | ".join(f"{n:,} files" for n in counts) + " |",
        "|---|---|---|" + "---:|" * len(counts),
    ]
    for row in summary["rows"]:
        cells = [
            *tool_cells(row),
            *[f"{row['timings'][str(n)]['median_seconds'] * 1000:,.2f}" for n in counts],
        ]
        lines.append("| " + " | ".join(cells) + " |")
    lines += ["", "| Files | Elapsed-time reduction |", "|---:|---:|"]
    for n in counts:
        lines.append(f"| {n:,} | {summary['reductions_percent'][str(n)]:+.2f}% |")
    lines += [
        "",
        "Both binaries use identical commands and the same deferred CPU runtime library. "
        "Only explicit CPU invocations with two or three non-recursive paths receive the new cap. "
        "Single-file, GPU, recursive and larger-workload policies remain unchanged. "
        "Small movements in unchanged workloads measure run-to-run host variation.",
        "",
    ]
    Path(output).write_text("\n".join(lines))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["source", "dataset", "output", "render"]:
        parser.add_argument("--" + name, type=Path)
    args = parser.parse_args()
    if args.render:
        render(args.render, args.output)
    else:
        derive(args.source, c.load_json(args.dataset), args.output)
