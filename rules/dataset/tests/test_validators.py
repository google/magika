import hashlib
import struct
import zlib

from helpers import status

from magika_datasets.validators import MAX_BYTES, observe
from magika_datasets.validators.image import png


def chunk(kind, body=b""):
    return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body))


def image():
    return (
        png.SIGNATURE
        + chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(b"\x00\xff\x00\x00"))
        + chunk(b"IEND")
    )


def test_png_structural_checks():
    data = image()
    assert status(png, data) == "pass"
    assert status(png, data[:-1]) == "fail"
    assert status(png, data + b"trailing") == "fail"
    broken = bytearray(data)
    broken[20] ^= 1
    assert status(png, bytes(broken)) == "fail"
    assert status(png, png.SIGNATURE + chunk(b"IDAT")) == "fail"
    assert status(png, b"not png") == "not_applicable"


def test_observations_bind_to_bytes_and_respect_limit(tmp_path):
    path = tmp_path / "image"
    path.write_bytes(image())
    assert (
        observe(path, hashlib.sha256(image()).hexdigest())["observations"][0]["format_id"] == "png"
    )
    assert observe(path, "0" * 64)["status"] == "error"
    path.write_bytes(b"x" * (MAX_BYTES + 1))
    assert observe(path, "0" * 64)["status"] == "skipped"
    assert MAX_BYTES == 16 * 1024 * 1024


def test_validation_groups_preserve_disagreement_and_require_manual_evidence():
    from magika_datasets.validation import classify

    report = {"observations": [{"format_id": "png", "status": "pass", "auto_eligible": True}]}
    formats = {"png": {}, "pdf": {}}
    assert classify({}, report, formats) == "validated_auto"
    # Structural proof settles identity even when detectors disagreed.
    assert classify({"conflicting": True}, report, formats) == "validated_auto"
    assert classify({"vt_markings": {"magika": "pdf"}}, report, formats) == "validated_auto"
    # Without proof, disagreement stays a conflict; two proofs are a conflict too.
    assert classify({"conflicting": True}, {"observations": []}, formats) == "conflicting"
    two = {
        "observations": [
            dict(report["observations"][0]),
            {"format_id": "pdf", "status": "pass", "auto_eligible": True},
        ]
    }
    assert classify({}, two, formats) == "conflicting"
    failed = {
        "observations": [
            dict(report["observations"][0]),
            {"format_id": "pdf", "status": "fail", "auto_eligible": True},
        ]
    }
    assert classify({"vt_markings": {"magika": "pdf"}}, failed, formats) == "conflicting"
    assert classify({"label_status": "accepted"}, {"observations": []}, formats) == "unknown"
    manual = {
        "manual_validation": {
            "reviewer": "reviewer",
            "format_ids": ["png"],
            "decision": "validated",
            "evidence": "review-1",
        }
    }
    assert classify(manual, {"observations": []}, formats) == "validated_manual"


def test_png_compressed_payload_is_checked():
    header = chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 2, 0, 0, 0))
    for body in [b"not-zlib", zlib.compress(b"\x05\xff\x00\x00"), zlib.compress(b"\x00")]:
        assert status(png, png.SIGNATURE + header + chunk(b"IDAT", body) + chunk(b"IEND")) == "fail"


def test_automatic_relabel_settles_label_and_keeps_hard_case():
    from magika_datasets.validation import apply_label

    annotation = {"format_ids": ["pdf"], "conflicting": True}
    report = {"observations": [{"format_id": "png", "status": "pass", "auto_eligible": True}]}
    result = apply_label(annotation, report, {"png": {}, "pdf": {}})
    assert result["format_ids"] == ["png"]
    assert result["previous_format_ids"] == ["pdf"]
    assert result["label_decision"]["basis"] == "validated_auto"
    assert result["validation_status"] == "validated_auto"
    assert not result["conflicting"] and result["hard_case"]
    assert "detectors_disagree" in result["tags"]
    assert annotation["format_ids"] == ["pdf"]
    agreed = apply_label({"format_ids": ["png"]}, report, {"png": {}, "pdf": {}})
    assert not agreed["hard_case"] and "detectors_disagree" not in agreed["tags"]


def test_llm_label_requires_attributed_evidence():
    from magika_datasets.validation import apply_label

    report = {"observations": []}
    annotation = {
        "llm_validation": {
            "model": "review-model",
            "decision": "validated",
            "format_ids": ["png"],
            "evidence": "review record 12",
        }
    }
    result = apply_label(annotation, report, {"png": {}})
    assert result["validation_status"] == "llm-validated"
    assert result["format_ids"] == ["png"]
    del annotation["llm_validation"]["evidence"]
    assert apply_label(annotation, report, {"png": {}})["validation_status"] == "unknown"


def test_pixel_decoders_validate_real_images_and_reject_truncation():
    import io

    from PIL import Image

    from magika_datasets.validators.image import bmp, gif, jpeg, tiff
    from magika_datasets.validators.media import riff

    for kind, validator in [
        ("JPEG", jpeg),
        ("GIF", gif),
        ("BMP", bmp),
        ("TIFF", tiff),
        ("WEBP", riff),
    ]:
        output = io.BytesIO()
        Image.new("RGB", (4, 4), "red").save(output, format=kind)
        data = output.getvalue()
        assert status(validator, data) == "pass", kind
        assert status(validator, data[:20]) != "pass", kind


def test_decoder_timeout_is_inconclusive(monkeypatch):
    import subprocess

    from magika_datasets.validators._shared import pillow

    def timeout(*args, **kwargs):
        raise subprocess.TimeoutExpired("decoder", 5)

    monkeypatch.setattr(pillow.subprocess, "run", timeout)
    assert pillow.decode(b"x", "JPEG")[0] == "inconclusive"


def test_base_decoder_does_not_relabel_or_certify_specialized_format():
    from magika_datasets.validation import apply_label

    annotation = {"format_ids": ["geotiff"]}
    report = {"observations": [{"format_id": "tiff", "status": "pass", "auto_eligible": True}]}
    result = apply_label(annotation, report, {"geotiff": {}, "tiff": {}})
    assert result["format_ids"] == ["tiff"]
    assert result["tags"] == ["geotiff"]
    assert result["validation_status"] == "validated_auto"
    assert result["label_decision"]["basis"] == "validated_auto"


def test_pillow_ico_decodes_every_size():
    import io

    from PIL import Image

    from magika_datasets.validators._shared import pillow

    output = io.BytesIO()
    Image.new("RGBA", (64, 64), "red").save(
        output, format="ICO", sizes=[(16, 16), (32, 32), (64, 64)]
    )
    data = output.getvalue()
    assert pillow.inspect(data, "ICO")[0] == "pass"
    assert pillow.inspect(data[:-40], "ICO")[0] == "fail"


def test_pillow_decodes_frame_zero_even_when_seeking_is_impossible():
    import struct

    from magika_datasets.validators._shared import pillow

    header = b"8BPS\0\1" + b"\0" * 6 + struct.pack(">HIIHH", 1, 4, 4, 8, 1)
    body = struct.pack(">I", 0) * 3 + struct.pack(">H", 0) + b"\x7f" * 16
    assert pillow.inspect(header + body, "PSD")[0] == "pass"
    assert pillow.inspect(header + body[:-8], "PSD")[0] == "fail"


def test_interlaced_png_uses_adam7_pass_sizes():
    import io

    from PIL import Image

    output = io.BytesIO()
    Image.new("RGB", (9, 7), "red").save(output, format="PNG", interlace=1)
    assert status(png, output.getvalue()) == "pass"
    assert status(png, output.getvalue()[:-20]) == "fail"
