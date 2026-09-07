# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import json
from pathlib import Path

import pytest
from magika_rules_benchmark.corpus import class_mapping, prepare


def test_whole_files_and_class_mapping(corpus_factory, tmp_path):
    classes, rows, identity = prepare(corpus_factory(), tmp_path / "output")
    assert identity["samples"] == 2
    assert [r["truth"] for r in rows] == ["png", "gif"]
    assert Path(rows[0]["path"]).read_bytes() == b"PNG sample"
    mapping, supported = class_mapping(classes, dict(target_labels_space=["png", "gif"]))
    assert mapping["png"] == "png"
    assert supported == ["gif", "png"]


def test_duplicate_content_rejected(corpus_factory, tmp_path):
    with pytest.raises(ValueError, match="Duplicate content"):
        prepare(corpus_factory(contents=(b"same", b"same")), tmp_path / "output")


@pytest.mark.parametrize("field", ["samples", "original_bytes", "metadata_rows_sha256"])
def test_manifest_integrity(corpus_factory, tmp_path, field):
    root = corpus_factory()
    spec = json.loads((root / "manifest.json").read_text())
    spec[field] = "bad" if field.endswith("sha256") else 999
    (root / "manifest.json").write_text(json.dumps(spec))
    with pytest.raises(ValueError, match="Corpus"):
        prepare(root, tmp_path / "output")


def test_ambiguity_does_not_become_ground_truth(corpus_factory, tmp_path):
    _, rows, _ = prepare(corpus_factory(ambiguous=True), tmp_path / "output")
    assert all(row["truth"] is None and row["ambiguous"] for row in rows)


def test_modified_materialized_file_rejected(corpus_factory, tmp_path):
    root = corpus_factory()
    _, rows, _ = prepare(root, tmp_path / "output")
    Path(rows[0]["path"]).write_bytes(b"changed")
    with pytest.raises(ValueError, match="materialized"):
        prepare(root, tmp_path / "output")


def test_unknown_model_class_is_explicit_error():
    with pytest.raises(ValueError, match="unambiguously"):
        class_mapping({}, dict(target_labels_space=["png"]))
