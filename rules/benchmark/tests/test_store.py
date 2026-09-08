# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import json
import runpy
import tarfile
from pathlib import Path

import pytest
from magika_rules_benchmark.comparison import load_json, save_gzip

store = runpy.run_path(str(Path(__file__).parents[2] / "benchmarks/store.py"))["store"]


def test_store_keeps_json_evidence_and_excludes_corpus_and_caches(tmp_path):
    source = tmp_path / "source"
    source.mkdir()
    result = dict(
        status="complete", benchmark_version="1.0.0", revision="abc", quality={}, measurements=[]
    )
    (source / "results.json").write_text(json.dumps(result))
    for name in ("config.json", "workloads.json", "label-mappings.json"):
        (source / name).write_text("{}")
    save_gzip(source / "inputs.json.gz", {})
    save_gzip(source / "observations.json.gz", {})
    (source / "raw").mkdir()
    (source / "raw/trial.json").write_text("{}")
    (source / "files").mkdir()
    (source / "files/private-corpus").write_bytes(b"not publication data")
    destination = tmp_path / "history/run-1"
    receipt = store(source, destination)
    assert receipt["revision"] == "abc"
    assert load_json(destination / "results.json.gz") == result
    with tarfile.open(destination / "raw-output.tar.gz") as archive:
        assert archive.getnames() == ["observations.json.gz", "raw/trial.json"]
    assert json.loads((destination.parent / "index.json").read_text())["runs"][0]["id"] == "run-1"
    with pytest.raises(FileExistsError):
        store(source, destination)


def test_store_rejects_unfinished_measurements(tmp_path):
    (tmp_path / "results.json").write_text('{"status":"in_progress"}')
    with pytest.raises(ValueError, match="unfinished"):
        store(tmp_path, tmp_path / "published")
    assert not (tmp_path / "published").exists()
