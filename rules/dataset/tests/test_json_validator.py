import hashlib

import pytest
from helpers import status

from magika_datasets.validators import observe
from magika_datasets.validators.text import json


@pytest.mark.parametrize("payload", [b"{}", b"[]", b'{"x":[true,false,null,1.5]}'])
def test_valid_containers(payload):
    assert status(json, payload) == "pass"


@pytest.mark.parametrize(
    "payload", [b'{"x":NaN}', b'{"x":Infinity}', b"{} trailing", b"[1,]", b'{"x":"\xff"}']
)
def test_bad_json(payload):
    assert status(json, payload) == "fail"


def test_ambiguity_and_specific_types_are_not_auto_relabelled(tmp_path):
    assert status(json, b'{"a":1,"a":2}') == "inconclusive"
    assert status(json, b"42") == "not_applicable"
    path = tmp_path / "file"
    data = b'{"cells":[]}'
    path.write_bytes(data)
    sha = hashlib.sha256(data).hexdigest()
    assert not observe(path, sha, hints={"ipynb"})["observations"][0]["auto_eligible"]
    assert observe(path, sha, hints={"json"})["observations"][0]["auto_eligible"]
