# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""A pinned public source can name a sample no validator or detector can."""

from magika_datasets.origins import extension_owners, identity
from magika_datasets.validation import apply_label, classify

FORMATS = {
    "c": {"extensions": ["c"], "metadata_json": '{"class_role": "format"}'},
    "cpp": {"extensions": ["cpp", "cc"], "metadata_json": '{"class_role": "format"}'},
    "h": {"extensions": ["h"], "metadata_json": '{"class_role": "format"}'},
    "png": {"extensions": ["png"], "metadata_json": '{"class_role": "format"}'},
    "dockerfile": {"extensions": [], "metadata_json": '{"class_role": "format"}'},
    "invalid": {"extensions": [], "metadata_json": '{"class_role": "negative"}'},
    "unknown": {"extensions": [], "metadata_json": '{"class_role": "format"}'},
}
REVISION = "5e57b94cdd3f2bc2f6db8581dcce76a89fc9c9f7"


def blob(path):
    return [f"github:https://github.com/file/file/blob/{REVISION}/{path}:004a8c8e9fd3d664"]


def no_observations():
    return {"observations": []}


def test_a_pinned_blob_extension_names_one_class():
    assert identity(blob("src/funcs.c"), FORMATS) == "c"
    assert identity(blob("include/deep/dir/thing.cc"), FORMATS) == "cpp"


def test_a_conventional_filename_counts():
    assert identity(blob("build/Dockerfile"), FORMATS) == "dockerfile"


def test_an_unpinned_or_foreign_origin_names_nothing():
    assert identity([f"vt:{'ab' * 32}"], FORMATS) is None
    assert identity(["github:https://github.com/a/b/blob/main/src/funcs.c:00"], FORMATS) is None
    assert identity(blob("src/funcs"), FORMATS) is None


def test_a_negative_class_is_never_named_by_provenance():
    owners = extension_owners(FORMATS)
    assert "invalid" not in owners.values()


def test_provenance_labels_what_nothing_else_can():
    result = apply_label({}, no_observations(), FORMATS, origins=blob("src/funcs.c"))
    assert result["label_status"] == "validated_origin"
    assert result["format_ids"] == ["c"]


def test_provenance_ranks_below_every_other_tier():
    proof = {"observations": [{"format_id": "png", "status": "pass", "auto_eligible": True}]}
    result = apply_label({}, proof, FORMATS, origins=blob("src/funcs.c"))
    assert result["label_status"] == "validated_auto"
    assert result["format_ids"] == ["png"]
    assert "detectors_disagree" in result["tags"]


def test_a_validator_failure_refutes_provenance():
    failed = {"observations": [{"format_id": "c", "status": "fail", "auto_eligible": True}]}
    assert classify({}, failed, FORMATS, origins=blob("src/funcs.c")) == "need_review"


def test_provenance_never_overrides_a_curated_negative_label():
    assert (
        classify({}, no_observations(), FORMATS, origins=blob("broken.png"), prior="invalid")
        == "need_review"
    )


def test_a_detector_claim_elsewhere_refutes_provenance():
    annotation = {"vt_markings_history": [{"magika": "png"}]}
    assert (
        classify(annotation, no_observations(), FORMATS, origins=blob("src/funcs.c"))
        == "need_review"
    )


def test_without_origins_nothing_changes():
    assert classify({}, no_observations(), FORMATS) == "need_review"


def test_a_generated_origin_names_its_class_when_the_bytes_regenerate():
    from magika_datasets import generated

    formats = {
        **FORMATS,
        "randomtxt": {"extensions": [], "metadata_json": '{"class_role": "format"}'},
    }
    value, _ = generated.origin("randomtxt", 4)
    assert identity([value], formats) == "randomtxt"
    assert identity([value[:-64] + "f" * 64], formats) is None
    result = apply_label({}, no_observations(), formats, origins=[value])
    assert result["label_status"] == "validated_origin"
    assert result["label_decision"]["evidence"] == "reproducible_generator"


def test_a_generated_origin_for_a_class_outside_the_taxonomy_names_nothing():
    from magika_datasets import generated

    value, _ = generated.origin("randomtxt", 4)
    assert identity([value], FORMATS) is None


def test_a_refutation_of_another_format_does_not_block_provenance():
    # A validator saying "this is not png" says nothing against the path naming it c.
    failed = {"observations": [{"format_id": "png", "status": "fail", "auto_eligible": True}]}
    result = apply_label({}, failed, FORMATS, origins=blob("src/funcs.c"))
    assert result["label_status"] == "validated_origin" and result["format_ids"] == ["c"]


def test_a_refutation_of_the_named_format_blocks_provenance():
    failed = {"observations": [{"format_id": "c", "status": "fail", "auto_eligible": True}]}
    result = apply_label({}, failed, FORMATS, origins=blob("src/funcs.c"))
    assert result["label_status"] != "validated_origin"
