import hashlib

import pytest
from helpers import status

from magika_datasets.validators import observe
from magika_datasets.validators.text import toml


@pytest.mark.parametrize(
    "payload",
    [b'name="example"', b'[project]\nrequires-python=">=3.12"', b"x = 1979-05-27T07:32:00Z"],
)
def test_toml_documents(payload):
    assert status(toml, payload) == "pass"


@pytest.mark.parametrize("payload", [b"x=1\nx=2", b"[table", b'x = "\xff"'])
def test_invalid_toml(payload):
    assert status(toml, payload) == "fail"


def test_empty_toml_and_python_overlap(tmp_path):
    assert status(toml, b"# comment") == "inconclusive"
    path = tmp_path / "data"
    data = b"x=1"
    path.write_bytes(data)
    sha = hashlib.sha256(data).hexdigest()
    assert observe(path, sha, hints={"python"})["observations"] == []
    obs = observe(path, sha, hints={"toml"})["observations"]
    assert obs[0]["status"] == "pass" and obs[0]["auto_eligible"]
