# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import json
import sys

import pytest
from benchmark_fixtures import build_snapshot

from magika_datasets.benchmark import run

FAKE_MAGIKA = """
import json, sys
paths = sys.argv[sys.argv.index("--") + 1:]
for path in paths:
    label = "png" if open(path, "rb").read().startswith(b"PNG") else "gif"
    value = {"output": {"label": label}, "dl": {"label": label}}
    print(json.dumps({"path": path, "result": {"status": "ok", "value": value}}))
"""
FAKE_HYPERFINE = """
import json, sys
if sys.argv[1] == "--version":
    print("hyperfine 1.0"); sys.exit()
out = sys.argv[sys.argv.index("--export-json") + 1]
json.dump({"results": [{"times": [0.01, 0.02], "exit_codes": [0, 0]}]}, open(out, "w"))
"""


def script(tmp_path, name, body):
    path = tmp_path / name
    path.write_text(body)
    return path


def config(tmp_path):
    magika = script(tmp_path, "magika.py", FAKE_MAGIKA)
    return {
        "schema": 1,
        "file_counts": [1, 2, 50],
        "runs": 2,
        "warmup": 1,
        "seed": 1,
        "tools": [
            {
                "id": "fake-magika",
                "adapter": "magika-jsonl",
                "command": [sys.executable, str(magika)],
                "version_command": [sys.executable, "--version"],
                "artifacts": {"script": str(magika)},
            },
            {
                "id": "absent",
                "adapter": "trid",
                "command": ["trid"],
                "unavailable": "not installed",
            },
        ],
    }


def test_a_run_scores_verified_files_and_times_each_workload(tmp_path):
    snapshot = build_snapshot(
        tmp_path / "snapshot",
        [
            (b"PNG one", "png", "validated_auto"),
            (b"GIF one", "png", "validated_origin"),
            (b"PNG two", "png", "need_review"),
        ],
    )
    hyperfine = script(tmp_path, "hyperfine", "#!" + sys.executable + "\n" + FAKE_HYPERFINE)
    hyperfine.chmod(0o755)
    result = run.run(config(tmp_path), snapshot, tmp_path / "out", "abc", str(hyperfine), 30)
    quality = result["quality"]["fake-magika"]
    assert result["snapshot"]["scored"] == 2  # the need_review sample is not ground truth
    assert quality["files"] == 2 and quality["accuracy"] == 0.5
    assert [m["file_count"] for m in result["measurements"]] == [1, 2]  # 50 would repeat files
    assert result["status"] == "partial" and result["unavailable"] == {"absent": "not installed"}
    assert "| fake-magika |" in (tmp_path / "out" / "report.md").read_text()
    assert json.loads((tmp_path / "out" / "results.json").read_text())["status"] == "partial"


def test_a_run_never_overwrites_an_earlier_one(tmp_path):
    snapshot = build_snapshot(tmp_path / "snapshot", [(b"PNG one", "png", "validated_auto")])
    (tmp_path / "out").mkdir()
    with pytest.raises(FileExistsError):
        run.run(config(tmp_path), snapshot, tmp_path / "out", "abc", "hyperfine", 30)


def test_timed_output_must_repeat_the_quality_observation():
    samples = [{"sha256": "a", "truth": "png"}]
    base = {"prediction": "png", "error": None, "deterministic": False, "mapping": "mapped"}
    changed = base | {"prediction": "gif"}
    with pytest.raises(ValueError, match="differ"):
        run.check_workload(samples, [changed], [base])
    quality, differences = run.check_workload(samples, [changed], [base], adaptive=True)
    assert quality["wrong"] == 1 and len(differences) == 1
