# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Render tool-default comparisons and measured GPU crossover from saved JSON."""

import argparse
import gzip
import json
import tarfile
from pathlib import Path

from magika_rules_benchmark import comparison as c
from magika_rules_benchmark import corpus
from magika_rules_benchmark.identity import (
    report_identity_lines,
    report_metadata,
    source_dataset,
    tool_cells,
    tool_record,
)

TOOLS = [
    ("magika1", "Magika 1 CPU", "cpu"),
    ("magika2-ml", "Magika 2 CPU ML", "cpu"),
    ("magika2-rules", "Magika 2 CPU rules + ML", "cpu"),
    ("magika2-gpu-ml", "Magika 2 GPU ML", "gpu"),
    ("magika2-gpu-rules", "Magika 2 GPU rules + ML", "gpu"),
    ("magika2-auto-ml", "Magika 2 Auto ML", "auto"),
    ("magika2-auto-rules", "Magika 2 Auto rules + ML", "auto"),
    ("magika2-rules-only", "Magika 2 rules-only CPU", "rules-only"),
    ("libmagic", "libmagic", "cpu"),
    ("trid", "TrID", "cpu"),
]
COUNTS = [1, 2, 5, 10, 25, 100, 1000]


def observations(source):
    if (source / "observations.json.gz").exists():
        return c.load_json(source / "observations.json.gz")
    with tarfile.open(source / "raw-output.tar.gz") as archive:
        return json.loads(gzip.decompress(archive.extractfile("observations.json.gz").read()))


def result_file(source):
    path = source / "results.json"
    return path if path.exists() else source / "results.json.gz"


def gpu_crossover(rows):
    by_id = {row["id"]: row for row in rows}
    result = {}
    for mode, cpu, gpu in [
        ("ML", "magika2-ml", "magika2-gpu-ml"),
        ("rules + ML", "magika2-rules", "magika2-gpu-rules"),
    ]:
        comparisons = []
        for count in COUNTS:
            a, b = by_id[cpu]["timings"][str(count)], by_id[gpu]["timings"][str(count)]
            cpu_ms, gpu_ms = a["median_seconds"] * 1000, b["median_seconds"] * 1000
            inference_files = a["inference_files"]
            comparisons.append(
                {
                    "files": count,
                    "cpu_ms": cpu_ms,
                    "gpu_ms": gpu_ms,
                    "inference_files": inference_files,
                    "winner": "no inference"
                    if not inference_files
                    else "GPU"
                    if gpu_ms < cpu_ms
                    else "CPU",
                }
            )
        result[mode] = {
            "measurements": comparisons,
            "first_measured_gpu_win": next(
                (r["files"] for r in comparisons if r["winner"] == "GPU"), None
            ),
        }
    return result


def derive(source, destination):
    path = result_file(source)
    current = c.load_json(path)
    c.validate_default_policy(current["config"])
    assert current["status"] == "complete"
    inputs = c.load_json(source / "inputs.json.gz")
    samples = sorted(inputs["samples"], key=lambda s: s["sha256"])
    observed = observations(source)
    lookup = {s["sha256"]: i for i, s in enumerate(samples)}
    cases = {w["id"]: w for w in c.load_json(source / "workloads.json")}
    summary = {
        "schema": 2,
        "report_kind": "defaults",
        "revision": current["revision"],
        "files": len(samples),
        "runs": current["config"]["runs"],
        "warmup": current["config"]["warmup"],
        "workloads": len(cases),
        "results_sha256": corpus.file_hash(path),
        "rows": [],
        **report_metadata(current, source_dataset(source, current)),
    }
    controls = {
        "magika2-rules": "magika2-ml",
        "magika2-gpu-rules": "magika2-gpu-ml",
        "magika2-rules-only": None,
        "magika2-auto-rules": "magika2-auto-ml",
    }
    for tool, label, _ in TOOLS:
        quality = current["quality"][tool]
        assert quality["files"] == len(samples) and quality["errors"] == 0
        rule_matches = None
        if tool in controls:
            control = observed[controls[tool]] if controls[tool] else None
            rule_matches = len(c.rule_hit_hashes(samples, observed[tool], control)) / len(samples)
        times = {}
        for m in current["measurements"]:
            if m["tool"] != tool or m["requested_rule_hit_percent"] is not None:
                continue
            case = cases[m["case"]]
            inference = (
                sum(not observed[tool][lookup[h]]["deterministic"] for h in case["samples"])
                if tool.startswith("magika2")
                else None
            )
            times[str(m["file_count"])] = m | {"inference_files": inference}
        assert set(times) == set(map(str, COUNTS))
        summary["rows"].append(
            {
                "id": tool,
                "label": label,
                "tool_identity": tool_record(current, tool),
                "quality": quality,
                "rule_matches": rule_matches,
                "timings": times,
            }
        )
    summary["gpu_crossover"] = gpu_crossover(summary["rows"])
    destination.mkdir(parents=True, exist_ok=False)
    corpus.atomic_json(destination / "overview.json", summary)
    render(destination / "overview.json", destination / "overview.md")
    corpus.atomic_json(
        destination / "artifacts.json",
        {
            "schema": 2,
            "code_sha256": {"full_table.py": corpus.file_hash(Path(__file__))},
            "files": {p.name: corpus.file_hash(p) for p in sorted(destination.iterdir())},
        },
    )


def render(path, output):
    summary = c.load_json(path)
    lines = [
        "# Tool-default benchmark",
        "",
        *report_identity_lines(summary),
        f"Accuracy on {summary['files']:,} files; {summary['workloads']} saved workloads, "
        f"{summary['runs']} measured runs after {summary['warmup']} warmup. "
        "Every tool uses its default worker, reader and internal batch policy. No thread-cap environment variables are set.",
        "",
        "Whole-process median milliseconds, including startup, I/O, output and shutdown; warm OS caches. "
        "Default settings are measured here; this is not a claim that every tool has been exhaustively tuned.",
        "",
        "| Tool | Version | Config | Accuracy | Precision | Coverage | Rule matches | "
        + " | ".join(f"{n:,} file" + ("s" if n != 1 else "") for n in COUNTS)
        + " |",
        "|---|---|---|" + "---:|" * (4 + len(COUNTS)),
    ]
    for row in summary["rows"]:
        q = row["quality"]
        cells = tool_cells(row)
        cells[2] += "; defaults"
        cells += [
            *[f"{100 * q[k]:.2f}%" for k in ["accuracy", "precision", "decision_coverage"]],
            "—" if row["rule_matches"] is None else f"{100 * row['rule_matches']:.2f}%",
            *[f"{row['timings'][str(n)]['median_seconds'] * 1000:,.2f}" for n in COUNTS],
        ]
        lines.append("| " + " | ".join(cells) + " |")
    lines += [
        "",
        "## CPU/GPU crossover",
        "",
        "Winners below compare measured medians on the same natural workloads. "
        "No crossover is interpolated between file counts. A rules-only hit with no inference is excluded from CPU/GPU crossover claims.",
        "",
        "| Mode | " + " | ".join(f"{n:,} files" for n in COUNTS) + " | First measured GPU win |",
        "|---|" + "---|" * (len(COUNTS) + 1),
    ]
    for mode, result in summary["gpu_crossover"].items():
        first = result["first_measured_gpu_win"]
        lines.append(
            "| "
            + " | ".join(
                [
                    mode,
                    *[r["winner"] for r in result["measurements"]],
                    "none observed" if first is None else str(first),
                ]
            )
            + " |"
        )
    lines += [
        "",
        "Raw observations, exact commands, executable/model/database hashes and per-workload timings are retained in the run JSON. GPU means Metal on this host.",
        "",
    ]
    output.write_text("\n".join(lines))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    derive(args.source, args.destination)
