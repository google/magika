# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Render handoff measurements from saved JSON. Never invoke a measured tool."""

import argparse
import json
import statistics
from pathlib import Path


def render(directory):
    def read(name):
        return json.loads((directory / name).read_text())

    median = statistics.median
    profile = read("fusion-profile.json")["measurements"]
    scratch = read("crc-scratch.json")["runs"]
    prep = read("preparation.json")["rows"]
    teardown = read("teardown.json")["traces"]
    lines = [
        "# Performance handoff validation",
        "",
        f"Recorded: {read('artifacts.json')['recorded_at_utc']}. Apple M5 Max, release builds.",
        "",
        "The original handoff is preserved unchanged. This table is generated from the linked raw JSON; "
        "whole-process, inference-loop, and component timings remain separate.",
        "",
        "| Measurement | Before | Candidate |",
        "|---|---:|---:|",
    ]
    for batch in [1, 4]:
        rates = [
            median(
                r["batch"] * r["iterations"] * 1e9 / r["inference_ns"]
                for r in profile
                if r["batch"] == batch and r["fused"] == fused
            )
            for fused in [False, True]
        ]
        lines.append(
            f"| CPU inference loop, batch {batch}, files/s | {rates[0]:,.0f} | {rates[1]:,.0f} |"
        )
    spans = [
        median(
            e["duration_ns"]
            for r in scratch
            if r["mode"] == mode
            for e in r["trace"]["events"]
            if e["name"] == "native_scratch_allocate"
        )
        / 1000
        for mode in ["software", "hardware"]
    ]
    lines.append(f"| Native scratch allocation, µs | {spans[0]:.2f} | {spans[1]:.2f} |")
    times = [
        median(r["shared_prepare_ns"] for r in prep if r["variant"] == mode) / 1000
        for mode in ["before", "after"]
    ]
    lines.append(f"| Fresh-process CPU prepare, µs | {times[0]:.2f} | {times[1]:.2f} |")
    lines += ["", "| Follow-up whole-process timing | Before | Candidate |", "|---|---:|---:|"]
    for count in [5, 1000]:
        values = [median(r["times"]) * 1000 for r in read(f"confirmation-{count}.json")["results"]]
        lines.append(f"| {count:,} files, ms | {values[0]:.2f} | {values[1]:.2f} |")
    threads = read("thread-default.json")["results"]
    lines += [
        "",
        "| Default policy check, 1,000 files | 17 threads | 4 threads |",
        "|---|---:|---:|",
    ]
    lines.append(
        f"| Median wall time, ms | {median(threads[0]['times']) * 1000:.2f} | {median(threads[1]['times']) * 1000:.2f} |"
    )
    lines.append(
        f"| Mean user CPU, seconds | {threads[0]['user']:.3f} | {threads[1]['user']:.3f} |"
    )
    drops = {
        name: median(e["duration_ns"] for r in teardown for e in r["events"] if e["name"] == name)
        / 1000
        for name in ["inference_session_drop", "inference_runtime_drop"]
    }
    lines += [
        "",
        f"Median session drop: {drops['inference_session_drop']:.2f} µs; "
        f"runtime drop: {drops['inference_runtime_drop']:.2f} µs.",
        "",
        f"CRC oracle: {read('crc-oracle.json')['crc32c_oracle_cases']:,} passing cases under ASan/UBSan. "
        f"Maximum differing probe-score bits: {max(r['score_bit_differences'] for r in profile)}.",
        "",
        "The full 25,421-file comparison retained every normalized decision and recorded zero tool errors. "
        "[Original accuracy and timing table](https://github.com/google/magika/blob/9214192eda842d02c8d50d9af75f7d5a043901f3/rules/benchmarks/reports/2026-09-09-754c063e-fusion/overview.md). "
        "The follow-up timings above retain the confirmation run after outliers in the initial run; neither run was discarded.",
        "",
    ]
    (directory / "measurements.md").write_text("\n".join(lines))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    render(parser.parse_args().directory)
