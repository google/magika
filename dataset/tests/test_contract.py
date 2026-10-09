import types

import pytest

from magika_datasets.validators.contract import Observation, describe


def test_observation_rejects_unknown_status():
    assert Observation("pass", "ok", "png").tags == ()
    with pytest.raises(ValueError):
        Observation("not_applicable", "x", "png")


def test_describe_requires_family_format_ids_and_scope():
    module = types.ModuleType("magika_datasets.validators.image.png")
    module.FAMILY, module.FORMAT_IDS, module.SCOPE = "image", ("png",), "scope"
    meta = describe(module)
    assert meta["name"] == "image/png"
    assert meta["context_required"] is False and meta["requires_hint"] is False
    assert meta["prefix_only"] is False
    del module.SCOPE
    with pytest.raises(TypeError):
        describe(module)
    module.SCOPE, module.FORMAT_IDS = "scope", "png"
    with pytest.raises(TypeError):
        describe(module)
