# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Scores and timings computed from saved observations, and the report rendered from them."""

import statistics
from collections import defaultdict

from tabulate import tabulate


def quality(samples: list[dict], rows: list[dict]) -> dict:
    """Accuracy over all files, so unknowns and errors count against a tool."""
    per_class = defaultdict(lambda: dict(files=0, correct=0, decisions=0, errors=0))
    counts = dict(files=len(samples), correct=0, wrong=0, decisions=0, errors=0)
    counts |= dict(ambiguous=0, unmapped=0, abstained=0)
    for sample, row in zip(samples, rows, strict=True):
        truth, prediction = sample["truth"], row["prediction"]
        counts["correct"] += prediction == truth
        counts["wrong"] += prediction is not None and prediction != truth
        counts["decisions"] += prediction is not None
        counts["errors"] += bool(row["error"])
        for state in ("ambiguous", "unmapped", "abstained"):
            counts[state] += row.get("mapping") == state
        entry = per_class[truth]
        entry["files"] += 1
        entry["correct"] += prediction == truth
        entry["decisions"] += prediction is not None
        entry["errors"] += bool(row["error"])
    n = counts["files"]
    return counts | dict(
        accuracy=counts["correct"] / n if n else None,
        decision_coverage=counts["decisions"] / n if n else None,
        precision=counts["correct"] / counts["decisions"] if counts["decisions"] else None,
        macro_recall=statistics.mean(v["correct"] / v["files"] for v in per_class.values())
        if per_class
        else None,
        formats_with_correct_decision=sum(v["correct"] > 0 for v in per_class.values()),
        formats=len(per_class),
        per_class=dict(sorted(per_class.items())),
    )


def timing(hyperfine: dict, count: int) -> dict:
    times, exits = hyperfine["times"], hyperfine["exit_codes"]
    if not times or len(exits) != len(times) or any(exits):
        raise ValueError("Hyperfine trials have missing data or nonzero exit codes")
    if any(t <= 0 for t in times):
        raise ValueError("Hyperfine timing must be positive")
    median = statistics.median(times)
    return dict(
        trials=len(times),
        median_seconds=median,
        min_seconds=min(times),
        max_seconds=max(times),
        files_per_second=count / median,
    )


def percent(value) -> str:
    return "—" if value is None else f"{100 * value:.2f}%"


def render(result: dict) -> str:
    text = f"# File-type detector benchmark\n\nRevision `{result['revision']}`, {result['status']}.\n\n"
    text += (
        "Whole-process wall time with warm caches and no shell. One file includes startup; "
        "larger workloads amortize it.\n\n"
    )
    text += tabulate(
        [
            [
                tool,
                q["files"],
                percent(q["accuracy"]),
                percent(q["precision"]),
                percent(q["decision_coverage"]),
                percent(q["macro_recall"]),
                q["wrong"],
                q["errors"],
                q["ambiguous"],
                q["unmapped"],
                f"{q['formats_with_correct_decision']}/{q['formats']}",
            ]
            for tool, q in result["quality"].items()
        ],
        headers=[
            "Tool",
            "Files",
            "Accuracy",
            "Precision",
            "Mapped coverage",
            "Macro recall",
            "Wrong",
            "Errors",
            "Ambiguous",
            "Unmapped",
            "Formats correct",
        ],
        tablefmt="github",
    )
    text += "\n\n" + tabulate(
        [
            [
                m["tool"],
                m["file_count"],
                round(1000 * m["median_seconds"], 3),
                round(m["files_per_second"], 1),
                m["trials"],
            ]
            for m in result["measurements"]
        ],
        headers=["Tool", "Files", "Median ms", "Files/s", "Trials"],
        tablefmt="github",
    )
    common = result.get("common_vocabulary")
    if common:
        text += (
            f"\n\nOn the {len(common['classes'])} classes every tool can name unambiguously "
            f"({common['files']} files), chosen from class metadata before scoring:\n\n"
        )
        text += tabulate(
            [
                [
                    tool,
                    percent(q["accuracy"]),
                    percent(q["decision_coverage"]),
                    percent(q["macro_recall"]),
                ]
                for tool, q in common["quality"].items()
            ],
            headers=["Tool", "Accuracy", "Mapped coverage", "Macro recall"],
            tablefmt="github",
        )
    if result.get("unavailable"):
        text += "\n\n" + tabulate(
            sorted(result["unavailable"].items()),
            headers=["Unavailable tool", "Reason"],
            tablefmt="github",
        )
    return text + "\n"
