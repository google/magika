# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import json
from pathlib import Path

import pytest
from benchmark_fixtures import build_snapshot

from magika_datasets.benchmark.snapshot import prepare

SAMPLES = [(b"PNG sample", "png", "validated_auto"), (b"GIF sample", "gif", "need_review")]


def test_files_are_laid_out_and_only_verified_labels_are_truth(tmp_path):
    root = build_snapshot(tmp_path / "snapshot", SAMPLES)
    classes, samples = prepare(root, tmp_path / "out")
    assert set(classes) == {"png", "gif"}
    assert [s["truth"] for s in samples] == ["png", None]
    assert Path(samples[0]["path"]).read_bytes() == b"PNG sample"


def test_a_tampered_manifest_is_refused(tmp_path):
    root = build_snapshot(tmp_path / "snapshot", SAMPLES)
    manifest = json.loads((root / "manifest.json").read_text())
    manifest["metadata_rows_sha256"] = "0" * 64
    (root / "manifest.json").write_text(json.dumps(manifest))
    with pytest.raises(ValueError, match="digest"):
        prepare(root, tmp_path / "out")


def test_a_modified_materialized_file_is_refused(tmp_path):
    root = build_snapshot(tmp_path / "snapshot", SAMPLES)
    _, samples = prepare(root, tmp_path / "out")
    Path(samples[0]["path"]).write_bytes(b"changed")
    with pytest.raises(ValueError, match="modified"):
        prepare(root, tmp_path / "out")
