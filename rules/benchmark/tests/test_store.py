# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import json
import runpy
import tarfile
from pathlib import Path

import pytest
from magika_rules_benchmark.comparison import load_json, quality_metrics, save_gzip

store = runpy.run_path(str(Path(__file__).parents[2] / "benchmarks/store.py"))["store"]


@pytest.mark.parametrize(
    ("source_revisions", "expected_revision"),
    [
        (None, "abc"),
        (["abc" + "d" * 37], "abc" + "d" * 37),
        (["abc" + "d" * 37] * 2, "abc" + "d" * 37),
        (["abc" + "d" * 37, "abc" + "e" * 37], "abc"),
        (["f" * 40], "abc"),
    ],
)
def test_store_keeps_json_evidence_and_excludes_corpus_and_caches(
    tmp_path, source_revisions, expected_revision
):
    source = tmp_path / "source"
    source.mkdir()
    result = dict(
        status="complete",
        benchmark_version="1.0.0",
        revision="abc",
        quality={"tool": quality_metrics([], [])},
        measurements=[],
        created_at="2026-09-08T22:36:35+00:00",
        compatibility={"corpus": "a" * 64},
    )
    if source_revisions is not None:
        result["config"] = {"tools": [{"source_revision": r} for r in source_revisions]}
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
    dataset = {"id": "sample", "name": "Sample", "version": "v1"}
    with pytest.raises(ValueError, match="Dataset id"):
        store(source, destination)
    assert not destination.exists()
    receipt = store(source, destination, dataset)
    assert receipt["revision"] == expected_revision
    expected = result.copy()
    if expected_revision != "abc":
        expected.update(revision=expected_revision, revision_argument="abc")
    assert load_json(destination / "results.json.gz") == expected
    with tarfile.open(destination / "raw-output.tar.gz") as archive:
        assert archive.getnames() == ["observations.json.gz", "raw/trial.json"]
    assert json.loads((destination.parent / "index.json").read_text())["runs"][0]["id"] == "run-1"
    entry = load_json(destination.parent / "index.json")["runs"][0]
    assert entry["dataset"]["version"] == "v1"
    assert entry["measured_at_utc"] == "2026-09-08T22:36:35Z"
    # A matching snapshot can reuse its established identity.
    store(source, destination.parent / "run-2")
    result["compatibility"]["corpus"] = "b" * 64
    (source / "results.json").write_text(json.dumps(result))
    with pytest.raises(ValueError, match="different snapshot"):
        store(source, destination.parent / "run-3", dataset)
    assert not (destination.parent / "run-3").exists()
    with pytest.raises(FileExistsError):
        store(source, destination)


def test_store_rejects_unfinished_measurements(tmp_path):
    (tmp_path / "results.json").write_text('{"status":"in_progress"}')
    with pytest.raises(ValueError, match="unfinished"):
        store(tmp_path, tmp_path / "published")
    assert not (tmp_path / "published").exists()
