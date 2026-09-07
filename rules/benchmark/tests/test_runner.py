# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import os
import sys

import pytest
from magika_rules_benchmark.runner import exact_mix, predictions, run_cli, sample_order


@pytest.mark.parametrize("count", [1, 2, 5, 10, 20, 50, 100, 500, 1000])
def test_exact_mixtures(count):
    for rate in range(0, 101, 5):
        selected = exact_mix(["h1", "h2"], ["m1", "m2"], count, rate, 11)
        if count * rate % 100:
            assert selected is None
        else:
            assert len(selected) == count
            assert sum(p.startswith("h") for p in selected) == count * rate // 100
            assert selected == exact_mix(["h1", "h2"], ["m1", "m2"], count, rate, 11)


def test_nested_seeded_order_and_distinct_cycles():
    values = list(range(10))
    assert sample_order(values, 20, 7)[:5] == sample_order(values, 5, 7)
    assert set(sample_order(values, 20, 7)[:10]) == set(values)
    assert sample_order(values, 20, 7) != sample_order(values, 20, 8)


def test_invalid_output_fails():
    with pytest.raises(RuntimeError, match="reordered"):
        predictions(b'{"path":"other","result":{"status":"ok"}}', ["file"])
    with pytest.raises(RuntimeError, match="failed"):
        predictions(b'{"path":"file","result":{"status":"error"}}', ["file"])


def test_subprocess_errors_and_timeouts():
    with pytest.raises(RuntimeError, match="exited"):
        run_cli([sys.executable, "-c", "raise SystemExit(3)"], os.environ.copy(), 5)
    with pytest.raises(RuntimeError, match="exceeded"):
        run_cli([sys.executable, "-c", "import time;time.sleep(60)"], os.environ.copy(), 0.05)


def test_changed_materialized_input_rejected_before_execution(tmp_path, monkeypatch):
    from magika_rules_benchmark import runner

    sample = tmp_path / "sample"
    sample.write_bytes(b"changed")
    monkeypatch.setattr(runner, "run_cli", lambda *a, **k: pytest.fail("CLI must not run"))
    with pytest.raises(ValueError, match="Materialized input changed"):
        runner.matrix(
            None,
            None,
            [dict(path=str(sample), sha256="0" * 64)],
            tmp_path / "performance.json",
            {},
            counts=[10],
            rates=[50],
            workers=[4],
            backends=["cpu"],
        )


def test_matrix_pairing_resume_and_route_checks(tmp_path, monkeypatch):
    import json

    from magika_rules_benchmark import runner

    binary, pack = tmp_path / "magika", tmp_path / "rules.yar"
    binary.write_bytes(b"fake executable")
    pack.write_bytes(b"fake pack")
    rows = []
    for hit in (True, False):
        path = tmp_path / ("hit" if hit else "miss")
        path.write_text(path.name)
        rows.append(
            dict(
                path=str(path),
                sha256=runner.file_hash(path),
                product_rule=hit,
                ml_deterministic=False,
            )
        )
    calls = []

    def fake_cli(args, env, timeout):
        calls.append(args)
        hybrid = "--rules=enforce" in args
        output = [
            dict(
                path=path,
                result=dict(
                    status="ok",
                    value=dict(
                        dl=dict(label="undefined" if hybrid and path.endswith("/hit") else "txt"),
                        output=dict(label="txt"),
                        score=1.0,
                    ),
                ),
            )
            for path in args[args.index("--") + 1 :]
        ]
        return dict(seconds=0.01, cpu_seconds=0.01, peak_rss_bytes=1024), json.dumps(
            output[0]
        ).encode() if len(output) == 1 else b"\n".join(json.dumps(r).encode() for r in output)

    monkeypatch.setattr(runner, "run_cli", fake_cli)
    monkeypatch.setattr(runner, "host_info", lambda root: dict(architecture="test"))
    monkeypatch.setattr(runner, "read_baseline", lambda paths: {})
    args = dict(counts=[1, 20], rates=[0, 5, 50, 100], workers=[2, 4], backends=["cpu"], repeats=3)
    output = tmp_path / "matrix.json"
    result = runner.matrix(binary, pack, rows, output, {}, **args)
    assert len(result["skipped"]) == 2
    assert len(result["measurements"]) == 24
    orders = {}
    for cell in result["measurements"]:
        assert len(cell["trials"]) == 3
        for trial in cell["trials"]:
            key = cell["files"], cell["hit_percent"], trial["trial"]
            assert orders.setdefault(key, trial["order_sha256"]) == trial["order_sha256"]
    calls.clear()
    resumed = runner.matrix(binary, pack, rows, output, {}, resume=True, **args)
    assert resumed["measurements"] == result["measurements"]
    assert len(calls) == 2  # Only the two warmups, no repeated matrix cells.
    with pytest.raises(ValueError, match="changed"):
        runner.matrix(binary, pack, rows, output, {}, resume=True, **(args | dict(repeats=2)))
