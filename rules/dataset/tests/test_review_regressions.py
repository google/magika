import struct

import pytest
from test_fonts import woff_file
from test_isobmff import box, ftyp
from test_onnx_validator import build, field

from magika_datasets.validation import apply_label, classify
from magika_datasets.validators import auto_eligible
from magika_datasets.validators.contract import describe
from magika_datasets.validators.data import onnx, tflite
from magika_datasets.validators.media import isobmff


@pytest.mark.parametrize(
    "field_name,actor,status",
    [
        ("manual_validation", "reviewer", "validated_manual"),
        ("llm_validation", "model", "llm-validated"),
    ],
)
def test_explicit_review_settles_conflict_consistently(field_name, actor, status):
    annotation = {
        "conflicting": True,
        field_name: {
            actor: "reviewer",
            "decision": "validated",
            "format_ids": ["png"],
            "evidence": "record 1",
        },
    }
    result = apply_label(annotation, {"observations": []}, {"png": {}})
    assert result["validation_status"] == result["label_status"] == status
    assert result["hard_case"]
    annotation[field_name]["format_ids"] = ["not-a-format"]
    assert classify(annotation, {"observations": []}, {"png": {}}) == "conflicting"


def test_manual_review_requires_labels_and_matches_llm_precedence():
    review = {"reviewer": "x", "decision": "validated", "evidence": "record"}
    assert classify({"manual_validation": review}, {"observations": []}, {"png": {}}) == "unknown"
    annotation = {
        "manual_validation": {**review, "format_ids": ["png"]},
        "llm_validation": {**review, "model": "model", "format_ids": ["jpeg"]},
    }
    result = apply_label(annotation, {"observations": []}, {"png": {}, "jpeg": {}})
    assert result["validation_status"] == result["label_status"] == "llm-validated"
    assert result["format_ids"] == ["jpeg"]


def test_success_and_failure_without_detector_claims_are_conflicting():
    report = {
        "observations": [
            {"format_id": "png", "status": "pass", "auto_eligible": True},
            {"format_id": "png", "status": "fail", "auto_eligible": True},
        ]
    }
    assert classify({}, report, {"png": {}}) == "conflicting"


def test_media_depth_budget_does_not_certify_uninspected_bytes():
    payload = b"uninspected garbage"
    for _ in range(isobmff.DEPTH + 2):
        payload = box(b"moov", payload)
    assert isobmff.validate(ftyp(b"isom") + payload, frozenset()).status == "inconclusive"


def test_large_ftyp_and_malformed_brand_list():
    large = struct.pack(">I4sQ", 1, b"ftyp", 24) + b"isom" + bytes(4)
    assert isobmff.validate(large + box(b"moov", box(b"mvhd")), frozenset()).format_id == "mp4"
    assert isobmff.validate(box(b"ftyp", b"isom") + box(b"moov"), frozenset()).status == "fail"


def test_flatbuffer_string_requires_terminator_inside_file():
    buffer = tflite.Buffer(struct.pack("<II", 4, 1) + b"x")
    with pytest.raises(tflite.Malformed, match="NUL"):
        buffer.string(0)


def test_woff_expansion_budget_is_inconclusive(monkeypatch):
    data, woff = woff_file()
    monkeypatch.setattr(woff.decompress, "LIMIT", 1)
    assert woff.validate(data, frozenset()).status == "inconclusive"


def test_onnx_weak_prefix_requires_identity_context():
    result = onnx.validate(build(), frozenset())
    assert not auto_eligible(describe(onnx), result, frozenset({"unknown"}))
    assert auto_eligible(describe(onnx), result, frozenset({"onnx"}))
    assert onnx.validate(field(1, 0, 8) + field(7, 2, b""), frozenset()).status == "inconclusive"
    reordered = build()[2:] + build()[:2]
    assert onnx.validate(reordered, frozenset({"onnx"})).status == "pass"


def test_duplicate_icon_dimensions_do_not_hide_corrupt_entry():
    import io

    from PIL import Image

    from magika_datasets.validators._shared import pillow

    output = io.BytesIO()
    Image.new("RGBA", (16, 16), "red").save(output, format="PNG")
    png = output.getvalue()
    good = bytes([16, 16, 0, 0]) + struct.pack("<HHII", 1, 32, len(png), 38)
    bad = bytes([16, 16, 0, 0]) + struct.pack("<HHII", 1, 8, 20, 38 + len(png))
    icon = b"\0\0\1\0\2\0" + good + bad + png + bytes(20)
    assert pillow.inspect(icon, "ICO")[0] == "fail"
