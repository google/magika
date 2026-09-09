# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Render an artifact comparison from stored measurements and identities."""

import argparse
from pathlib import Path

from full_table import observations
from magika_rules_benchmark import comparison as c
from magika_rules_benchmark import corpus
from magika_rules_benchmark.identity import (
    report_identity_lines,
    report_metadata,
    source_dataset,
    tool_cells,
    tool_record,
)


def derive(source, output):
    result = c.load_json(source / "results.json.gz")
    observed = observations(source)
    assert result["status"] == "complete" and observed["before"] == observed["after"]
    summary = {
        "schema": 2,
        "report_kind": "paired",
        "revision": result["revision"],
        **report_metadata(result, source_dataset(source, result)),
        "identical_decisions": len(observed["before"]),
        "runs": result["config"]["runs"],
        "warmup": result["config"]["warmup"],
        "rows": [],
    }
    for tool in ["before", "after", "rules-reference"]:
        quality = result["quality"][tool]
        assert not quality["errors"]
        summary["rows"].append(
            {
                "id": tool,
                "tool_identity": tool_record(result, tool),
                "quality": quality,
                "rule_matches": quality["decision_coverage"] if tool == "rules-reference" else None,
                "timings": {
                    str(m["file_count"]): m for m in result["measurements"] if m["tool"] == tool
                },
            }
        )
    output.mkdir(parents=True, exist_ok=False)
    corpus.atomic_json(output / "overview.json", summary)
    render(output / "overview.json", output / "overview.md")


def render(source, output):
    summary = c.load_json(source)
    counts = sorted(map(int, summary["rows"][0]["timings"]))
    lines = [
        "# CPU artifact comparison",
        "",
        *report_identity_lines(summary),
        f"{summary['identical_decisions']:,} identical normalized CPU decisions; zero tool errors. "
        f"{summary['runs']} measured runs after {summary['warmup']} warmup. "
        "Whole-process median milliseconds, including startup and shutdown; warm OS caches.",
        "",
        "The two ML rows use the same CLI with different CPU runtime artifacts. "
        "The measured source tree was dirty; exact artifact hashes and the unfused/fused model configuration "
        "are retained above and in JSON. The rules-only row uses the unchanged native reference library.",
        "",
        "| Tool | Version | Config | Accuracy | Precision | Coverage | Rule matches | "
        + " | ".join(f"{n:,} files" for n in counts)
        + " |",
        "|---|---|---|" + "---:|" * (4 + len(counts)),
    ]
    for row in summary["rows"]:
        q = row["quality"]
        cells = tool_cells(row)
        if row["tool_identity"]["configuration"]["settings"].get("source_dirty"):
            cells[1] += " + dirty"
        cells += [f"{100 * q[k]:.2f}%" for k in ["accuracy", "precision", "decision_coverage"]]
        cells.append("—" if row["rule_matches"] is None else f"{100 * row['rule_matches']:.2f}%")
        cells += [f"{row['timings'][str(n)]['median_seconds'] * 1000:.2f}" for n in counts]
        lines.append("| " + " | ".join(cells) + " |")
    lines += [
        "",
        "The initial timing run contained outliers. "
        "[Separate confirmation measurements and handoff dispositions](../2026-09-09-handoff-validation/dispositions.md) "
        "retain that limitation; no initial measurements were overwritten.",
        "",
    ]
    output.write_text("\n".join(lines))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    derive(args.source, args.output)
