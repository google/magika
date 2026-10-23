import hashlib
import json

import pytest
from helpers import status

from magika_datasets.validators import observe
from magika_datasets.validators.text import json_family


def result(data, hints=frozenset()):
    return json_family.validate(data, hints)


@pytest.mark.parametrize("payload", [b"{}", b"[]", b'{"x":[true,false,null,1.5]}'])
def test_valid_containers(payload):
    observation = result(payload)
    assert (observation.status, observation.format_id, observation.generic) == (
        "pass",
        "json",
        True,
    )


@pytest.mark.parametrize(
    "payload", [b'{"x":NaN}', b'{"x":Infinity}', b"{} trailing", b"[1,]", b'{"x":"\xff"}']
)
def test_bad_json(payload):
    assert status(json_family, payload, frozenset({"json"})) == "fail"


def test_ambiguity_and_specific_types_are_not_auto_relabelled(tmp_path):
    assert status(json_family, b'{"a":1,"a":2}', frozenset({"json"})) == "inconclusive"
    assert status(json_family, b"42") == "not_applicable"
    path = tmp_path / "file"
    data = b'{"cells":[]}'
    path.write_bytes(data)
    sha = hashlib.sha256(data).hexdigest()
    assert not observe(path, sha, hints={"pytorch"})["observations"][0]["auto_eligible"]
    assert observe(path, sha, hints={"json"})["observations"][0]["auto_eligible"]


def test_geojson_ipynb_and_gltf_dispatch():
    geo = json.dumps(
        {
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": {"type": "Point", "coordinates": [1, 2]},
                    "properties": {},
                }
            ],
        }
    ).encode()
    observation = result(geo)
    assert (observation.status, observation.format_id, observation.generic) == (
        "pass",
        "geojson",
        False,
    )
    assert (
        result(json.dumps({"type": "Point", "coordinates": [1, 2]}).encode()).format_id == "geojson"
    )
    bad_geo = json.dumps({"type": "FeatureCollection", "features": "nope"}).encode()
    assert (result(bad_geo).status, result(bad_geo).format_id) == ("fail", "geojson")
    notebook = json.dumps(
        {"cells": [], "metadata": {}, "nbformat": 4, "nbformat_minor": 5}
    ).encode()
    assert result(notebook).format_id == "ipynb"
    assert result(json.dumps({"cells": "x", "nbformat": 4}).encode()).status == "fail"
    gltf = json.dumps({"asset": {"version": "2.0"}, "scenes": [], "nodes": []}).encode()
    assert result(gltf).format_id == "gltf"


def test_jsonl_requires_a_hint_and_complete_lines():
    lines = b'{"a": 1}\n{"a": 2}\n\n{"a": 3}\n'
    assert (
        status(json_family, lines) == "fail"
    )  # unhinted: not one JSON value; prefix-only drops it unless hinted json
    observation = result(lines, frozenset({"jsonl"}))
    assert (observation.status, observation.format_id) == ("pass", "jsonl")
    assert status(json_family, lines + b'{"a":', frozenset({"jsonl"})) == "fail"
    assert status(json_family, b'{"a": 1}\n', frozenset({"jsonl"})) == "pass"
