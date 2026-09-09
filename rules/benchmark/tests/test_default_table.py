# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import importlib
from pathlib import Path

import pytest


def test_gpu_crossover_excludes_workloads_without_inference(monkeypatch):
    monkeypatch.syspath_prepend(str(Path(__file__).resolve().parents[2] / "benchmarks"))
    table = importlib.import_module("full_table")
    rows = []
    for name in ["magika2-ml", "magika2-gpu-ml", "magika2-rules", "magika2-gpu-rules"]:
        gpu = "gpu" in name
        timings = {
            str(n): {
                "median_seconds": (0.005 if n == 1000 else 0.020) if gpu else 0.010,
                "inference_files": n,
            }
            for n in table.COUNTS
        }
        if name.endswith("rules"):
            timings["1"] = {"median_seconds": 0.001 if gpu else 0.002, "inference_files": 0}
        rows.append({"id": name, "timings": timings})
    result = table.gpu_crossover(rows)
    assert result["ML"]["first_measured_gpu_win"] == 1000
    assert result["rules + ML"]["first_measured_gpu_win"] == 1000
    assert result["rules + ML"]["measurements"][0]["winner"] == "no inference"


def test_revision_timing_comparison_rejects_changed_inputs_or_trials(monkeypatch):
    monkeypatch.syspath_prepend(str(Path(__file__).resolve().parents[2] / "benchmarks"))
    compare = importlib.import_module("startup_comparison")
    before = {
        "case": "files-10-hits-natural",
        "file_count": 10,
        "distinct_files": 10,
        "input_order_sha256": "a" * 64,
        "trials": 3,
        "median_seconds": 0.08,
    }
    after = before | {"median_seconds": 0.02}
    assert compare.timing_change(before, after)["change_percent"] == -75
    for change in [{"input_order_sha256": "b" * 64}, {"trials": 1}, {"distinct_files": 1}]:
        with pytest.raises(ValueError):
            compare.timing_change(before, after | change)
