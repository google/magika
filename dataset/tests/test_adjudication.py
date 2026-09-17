# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Human adjudications enter the label ladder as attributed manual reviews."""

import json

import pytest

from magika_datasets.adjudication import load
from magika_datasets.validation import apply_label

FORMATS = {"png": {}, "vib": {}, "pem": {}}
SHA = "ab" * 32


def records(tmp_path, *entries, reviewer="handover", evidence="DB_PR_HANDOVER.md"):
    path = tmp_path / "adjudications.json"
    path.write_text(
        json.dumps({"reviewer": reviewer, "evidence": evidence, "changes": list(entries)})
    )
    return path


def test_an_adjudication_becomes_an_attributed_manual_review(tmp_path):
    path = records(tmp_path, {"sha256": SHA, "previous_truth": "pem", "truth": "vib"})
    reviews = load(path, FORMATS)
    review = reviews[bytes.fromhex(SHA)]
    assert review["manual_validation"]["format_ids"] == ["vib"]
    assert review["manual_validation"]["reviewer"] == "handover"
    assert review["manual_validation"]["decision"] == "validated"
    assert review["manual_validation"]["evidence"]
    assert review["manual_validation"]["previous_truth"] == "pem"


def test_the_review_outranks_a_structural_proof(tmp_path):
    path = records(tmp_path, {"sha256": SHA, "previous_truth": "pem", "truth": "vib"})
    review = load(path, FORMATS)[bytes.fromhex(SHA)]
    proof = {"observations": [{"format_id": "png", "status": "pass", "auto_eligible": True}]}
    result = apply_label(review, proof, FORMATS)
    assert (result["label_status"], result["format_ids"]) == ("validated_manual", ["vib"])
    assert result["hard_case"] is True


def test_an_undefined_truth_is_refused(tmp_path):
    path = records(tmp_path, {"sha256": SHA, "previous_truth": "pem", "truth": "nosuchformat"})
    with pytest.raises(ValueError, match="not a class"):
        load(path, FORMATS)


def test_one_sample_may_not_be_adjudicated_twice(tmp_path):
    path = records(
        tmp_path,
        {"sha256": SHA, "previous_truth": "pem", "truth": "vib"},
        {"sha256": SHA, "previous_truth": "pem", "truth": "png"},
    )
    with pytest.raises(ValueError, match="adjudicated twice"):
        load(path, FORMATS)


def test_attribution_is_required(tmp_path):
    path = records(tmp_path, {"sha256": SHA, "truth": "vib"}, reviewer="")
    with pytest.raises(ValueError, match="reviewer"):
        load(path, FORMATS)
