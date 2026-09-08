import types
from pathlib import Path

import pyarrow.parquet as pq
import pytest

from magika_datasets import validators
from magika_datasets.validators import MODULES, REGISTRY, registry

CLASSES = Path(__file__).resolve().parents[1] / "classes.parquet"


@pytest.mark.skipif(not CLASSES.exists(), reason="taxonomy parquet is distributed separately")
def test_every_declared_format_exists_in_taxonomy():
    rows = pq.read_table(CLASSES, columns=["format_id"]).to_pylist()
    classes = {row["format_id"] for row in rows}
    missing = {format_id for format_id in REGISTRY if format_id not in classes}
    assert not missing, missing


def test_duplicate_format_declarations_are_rejected():
    clone = types.ModuleType("magika_datasets.validators.image.png2")
    clone.FAMILY, clone.FORMAT_IDS, clone.SCOPE = "image", ("png",), "dup"
    with pytest.raises(ValueError):
        registry(MODULES + (clone,))
    clone.FORMAT_IDS, clone.SHARED_FORMAT_IDS = ("gltf",), ("gltf",)
    assert "gltf" in registry(MODULES + (clone,))  # explicitly shared formats are allowed
    clone.SHARED_FORMAT_IDS = ()
    with pytest.raises(ValueError):
        registry(MODULES + (clone,))


def test_modules_return_none_for_unrelated_bytes():
    for module in MODULES:
        if getattr(module, "REQUIRES_HINT", False):
            continue  # hint-gated grammars have no signature and are never run unhinted
        assert module.validate(b"\x00" * 64, frozenset()) is None, module.__name__
        assert module.validate(b"", frozenset()) is None, module.__name__
    assert validators.VERSION == 4


def test_prefix_only_validators_report_failures_only_when_hinted(tmp_path):
    import hashlib

    from magika_datasets.validators import observe

    cases = {
        "svg": b"<html><body>not xml at all",
        "json": b'{"unterminated": ',
        "zlibstream": b"\x78\x9c" + b"\x00" * 20,
    }
    for format_id, data in cases.items():
        path = tmp_path / format_id
        path.write_bytes(data)
        sha = hashlib.sha256(data).hexdigest()
        assert observe(path, sha)["observations"] == [], format_id
        hinted = observe(path, sha, hints={format_id})["observations"]
        assert [o["status"] for o in hinted] == ["fail"], format_id


def test_validator_exception_is_recorded_not_raised(tmp_path, monkeypatch):
    import hashlib

    from magika_datasets import validators
    from magika_datasets.validators.image import png

    def explode(data, hints):
        raise RuntimeError("boom")

    monkeypatch.setattr(png, "validate", explode)
    data = png.SIGNATURE + b"rest"
    path = tmp_path / "sample"
    path.write_bytes(data)
    report = validators.observe(path, hashlib.sha256(data).hexdigest())
    errors = [o for o in report["observations"] if o["format_id"] == "png"]
    assert [o["status"] for o in errors] == ["inconclusive"]
    assert "RuntimeError" in errors[0]["detail"] and errors[0]["auto_eligible"] is False
