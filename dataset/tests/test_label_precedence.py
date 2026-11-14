# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Every sample's label comes from the highest-precedence evidence that exists."""

import pytest

from magika_datasets.validation import apply_label, classify

FORMATS = {"3dsm": {}, "png": {}, "pdf": {}, "gif": {}}
# Real detector output: libmagic's description and a TrID name, both mapping to 3dsm.
MAGIC = "3D Studio model"
TRID = [{"file_type": "3D Studio mesh", "probability": 100.0}]


def observations(*pairs, auto=True):
    return {
        "observations": [
            {"status": status, "format_id": kind, "auto_eligible": auto, "tags": []}
            for kind, status in pairs
        ]
    }


def agreeing_tools():
    """libmagic and TrID both naming 3dsm, in the shape VirusTotal reports."""
    return {"vt_markings_history": [{"magic": MAGIC, "trid": TRID}]}


def split_tools():
    """libmagic naming 3dsm while TrID names something else."""
    return {
        "vt_markings_history": [
            {
                "magic": MAGIC,
                "trid": [{"file_type": "Portable Network Graphics", "probability": 100.0}],
            }
        ]
    }


def only_magika(kind="3dsm"):
    return {"vt_markings_history": [{"magika": kind}, {"magika": kind}]}


def only_trid():
    return {"vt_markings_history": [{"trid": TRID}]}


def manual(kind="gif"):
    return {
        "manual_validation": {
            "reviewer": "handover",
            "decision": "validated",
            "evidence": "adjudication receipt",
            "format_ids": [kind],
        }
    }


def llm(kind="pdf"):
    return {
        "llm_validation": {
            "model": "claude-opus-5",
            "decision": "validated",
            "evidence": "structural reading",
            "format_ids": [kind],
        }
    }


def test_no_evidence_needs_review():
    assert classify({}, observations(), FORMATS) == "need_review"


def test_legacy_accepted_is_not_evidence():
    annotation = {"label_status": "accepted", "format_ids": ["png"]}
    assert classify(annotation, observations(), FORMATS) == "need_review"
    result = apply_label(annotation, observations(), FORMATS)
    assert result["legacy_label_status"] == "accepted"


def test_structural_proof_beats_tool_consensus():
    result = apply_label(agreeing_tools(), observations(("png", "pass")), FORMATS)
    assert result["label_status"] == "validated_auto"
    assert result["format_ids"] == ["png"]
    assert "detectors_disagree" in result["tags"]
    assert result["hard_case"] is True


def test_tool_consensus_labels_what_no_validator_proved():
    result = apply_label(agreeing_tools(), observations(), FORMATS)
    assert result["label_status"] == "validated_tools"
    assert result["format_ids"] == ["3dsm"]
    assert result["hard_case"] is False


def test_magika_is_never_its_own_evidence():
    assert classify(only_magika(), observations(), FORMATS) == "need_review"


def test_one_detector_is_not_a_consensus():
    assert classify(only_trid(), observations(), FORMATS) == "need_review"


def test_disagreeing_detectors_are_conflicting():
    assert classify(split_tools(), observations(), FORMATS) == "conflicting"


def test_a_failing_validator_contradicts_its_own_consensus():
    assert classify(agreeing_tools(), observations(("3dsm", "fail")), FORMATS) == "conflicting"


def test_a_failure_elsewhere_leaves_consensus_intact():
    assert classify(agreeing_tools(), observations(("gif", "fail")), FORMATS) == "validated_tools"


@pytest.mark.parametrize("lower", [{}, agreeing_tools()])
def test_human_adjudication_wins(lower):
    result = apply_label({**manual("gif"), **lower}, observations(("png", "pass")), FORMATS)
    assert result["label_status"] == "validated_manual"
    assert result["format_ids"] == ["gif"]
    assert "detectors_disagree" in result["tags"]
    assert result["hard_case"] is True


def test_llm_review_beats_a_validator_but_loses_to_a_human():
    assert classify(llm("pdf"), observations(("png", "pass")), FORMATS) == "llm-validated"
    both = {**manual("gif"), **llm("pdf")}
    result = apply_label(both, observations(), FORMATS)
    assert (result["label_status"], result["format_ids"]) == ("validated_manual", ["gif"])


def test_disagreement_never_vetoes_a_review():
    annotation = {**manual("gif"), "conflicting": True}
    assert classify(annotation, observations(), FORMATS) == "validated_manual"


def test_a_review_naming_an_unknown_format_is_not_a_decision():
    assert classify(manual("nosuchformat"), observations(), FORMATS) == "need_review"


def test_hints_do_not_move_when_the_label_does():
    """Hints gate which validators run, so deriving them from the label oscillates.

    A provenance-based label unlocked a hint-gated validator that refuted it, which
    withdrew the label, which removed the hint, which restored the label.
    """
    from magika_datasets.origins import Provenance
    from magika_datasets.validation import sample_hints

    formats = {
        "c": {"extensions": ["c"], "metadata_json": '{"class_role": "format"}'},
        "unknown": {"extensions": [], "metadata_json": '{"class_role": "format"}'},
    }
    provenance = Provenance(formats)
    origins = [f"github:https://github.com/o/r/blob/{'a' * 40}/src/f.c:{'0' * 64}"]
    labelled = {"format_id": "c", "origins": origins}
    unlabelled = {"format_id": "unknown", "origins": origins}
    with_label = sample_hints(labelled, {"format_ids": ["c"]}, formats, provenance)
    without_label = sample_hints(unlabelled, {"format_ids": []}, formats, provenance)
    assert "c" in with_label and "c" in without_label, "provenance names it either way"
    assert with_label == without_label - {"unknown"}


def test_a_refutation_is_remembered():
    """Losing the hint that let a validator run must not restore the label it refuted."""
    tools = agreeing_tools()
    refuting = observations(("3dsm", "fail"))
    first = apply_label(tools, refuting, FORMATS)
    assert first["label_status"] == "conflicting"
    assert first["refuted_format_ids"] == ["3dsm"]
    # The validator no longer runs, but its finding about the bytes still stands.
    second = apply_label(first, observations(), FORMATS)
    assert second["label_status"] == "conflicting"
    assert second["refuted_format_ids"] == ["3dsm"]


def test_a_refutation_does_not_bleed_into_other_formats():
    annotation = {"refuted_format_ids": ["gif"], **agreeing_tools()}
    assert classify(annotation, observations(), FORMATS) == "validated_tools"


def test_a_specific_proof_outranks_a_generic_one():
    """A LightGBM model is also ASCII text; the encoding must not turn the proof into a tie."""
    both = {
        "observations": [
            {
                "format_id": "png",
                "status": "pass",
                "auto_eligible": True,
                "generic": False,
                "tags": [],
            },
            {
                "format_id": "gif",
                "status": "pass",
                "auto_eligible": True,
                "generic": True,
                "tags": [],
            },
        ]
    }
    result = apply_label({}, both, FORMATS)
    assert (result["label_status"], result["format_ids"]) == ("validated_auto", ["png"])


def test_a_generic_proof_relabels_a_file_whose_hinted_format_the_bytes_refute(tmp_path):
    """A .zip path over a gzip stream: zip is refuted, so the gzip proof may name the file."""
    import gzip
    import hashlib

    from magika_datasets.validators import observe

    data = gzip.compress(b"payload " * 64)
    path = tmp_path / "issue.zip"
    path.write_bytes(data)
    report = observe(path, hashlib.sha256(data).hexdigest(), hints={"zip"})
    gzip_pass = next(o for o in report["observations"] if o["format_id"] == "gzip")
    assert gzip_pass["status"] == "pass" and gzip_pass["auto_eligible"]
    assert any(o["format_id"] == "zip" and o["status"] == "fail" for o in report["observations"])
