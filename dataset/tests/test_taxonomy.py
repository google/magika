# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Reviewed classes append to the taxonomy; existing ordinals never move."""

import json

import pyarrow.parquet as pq
import pytest

from magika_datasets.taxonomy import extend


def additions(tmp_path, *entries):
    path = tmp_path / "additions.json"
    path.write_text(json.dumps({"classes": list(entries)}))
    return path


VIB = {
    "format_id": "vib",
    "name": "VMware Installation Bundle",
    "categories": ["archive"],
    "extensions": ["vib"],
    "metadata": {"class_role": "format", "reason": "handover adjudication names it"},
}


def test_a_reviewed_class_appends_at_the_next_ordinal(corpus, tmp_path):
    metadata, _, _ = corpus
    receipt = extend(metadata / "classes.parquet", additions(tmp_path, VIB))
    rows = pq.read_table(metadata / "classes.parquet").to_pylist()
    assert [r["format_id"] for r in rows] == ["png", "txt", "unknown", "vib"]
    assert [r["ordinal"] for r in rows] == [0, 1, 2, 3]
    added = rows[-1]
    assert added["name"] == "VMware Installation Bundle"
    assert added["categories"] == ["archive"]
    metadata_json = json.loads(added["metadata_json"])
    assert metadata_json["class_role"] == "format"
    assert metadata_json["id"] == "format:vib"
    assert metadata_json["counts"] == {"samples": 0, "target": 0}
    assert receipt == {"added": ["vib"], "classes": 4}


def test_redefining_a_published_class_is_refused(corpus, tmp_path):
    metadata, _, _ = corpus
    redefinition = {**VIB, "format_id": "png"}
    with pytest.raises(ValueError, match="different definition"):
        extend(metadata / "classes.parquet", additions(tmp_path, redefinition))


def test_applying_twice_adds_nothing(corpus, tmp_path):
    metadata, _, _ = corpus
    path = additions(tmp_path, VIB)
    extend(metadata / "classes.parquet", path)
    before = (metadata / "classes.parquet").read_bytes()
    assert extend(metadata / "classes.parquet", path) == {"added": [], "classes": 4}
    assert (metadata / "classes.parquet").read_bytes() == before


def corrections(tmp_path, *entries):
    path = tmp_path / "corrections.json"
    path.write_text(json.dumps({"classes": list(entries)}))
    return path


def test_a_reviewed_correction_redefines_extensions_in_place(corpus, tmp_path):
    from magika_datasets.taxonomy import correct

    metadata, _, _ = corpus
    fix = {"format_id": "png", "extensions": ["png", "apng"], "reason": "reviewed"}
    receipt = correct(metadata / "classes.parquet", corrections(tmp_path, fix))
    rows = pq.read_table(metadata / "classes.parquet").to_pylist()
    assert rows[0]["extensions"] == ["png", "apng"] and rows[0]["ordinal"] == 0
    assert json.loads(rows[0]["metadata_json"])["corrections"] == [
        {"extensions": ["png"], "reason": "reviewed"}
    ]
    assert receipt == {"corrected": ["png"], "classes": 3}
    before = (metadata / "classes.parquet").read_bytes()
    assert correct(metadata / "classes.parquet", corrections(tmp_path, fix))["corrected"] == []
    assert (metadata / "classes.parquet").read_bytes() == before


def test_a_correction_needs_a_reason_and_a_known_class(corpus, tmp_path):
    from magika_datasets.taxonomy import correct

    metadata, _, _ = corpus
    with pytest.raises(ValueError, match="reason"):
        correct(
            metadata / "classes.parquet",
            corrections(tmp_path, {"format_id": "png", "extensions": ["png"]}),
        )
    with pytest.raises(ValueError, match="nope"):
        correct(
            metadata / "classes.parquet",
            corrections(tmp_path, {"format_id": "nope", "extensions": ["x"], "reason": "r"}),
        )


def test_a_correction_can_set_a_target_and_merge_a_class(corpus, tmp_path):
    from magika_datasets.taxonomy import correct

    metadata, _, _ = corpus
    fixes = corrections(
        tmp_path,
        {
            "format_id": "txt",
            "target": 0,
            "merged_into": "png",
            "extensions": [],
            "reason": "same bytes",
        },
    )
    assert correct(metadata / "classes.parquet", fixes)["corrected"] == ["txt"]
    row = pq.read_table(metadata / "classes.parquet").to_pylist()[1]
    recorded = json.loads(row["metadata_json"])
    assert recorded["counts"]["target"] == 0 and recorded["merged_into"] == "png"
    assert recorded["corrections"][0] == {
        "extensions": ["txt"],
        "merged_into": None,
        "target": 100,
        "reason": "same bytes",
    }
    with pytest.raises(ValueError, match="unknown"):
        correct(
            metadata / "classes.parquet",
            corrections(tmp_path, {"format_id": "png", "merged_into": "nope", "reason": "r"}),
        )
