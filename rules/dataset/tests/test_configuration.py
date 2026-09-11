import json
from pathlib import Path

import pytest

from magika_datasets.configuration import validate_build_config
from magika_datasets.runtime import implementation_hashes


def config(tmp_path):
    from magika_datasets.verdicts import TOOLS

    for name in (
        "classes.parquet",
        "samples.parquet",
        "corpus-parquet-receipt.json",
        "repositories.parquet",
        "repositories-receipt.json",
        "repository-files.parquet",
        "repository-files-receipt.json",
        "policy.json",
    ):
        (tmp_path / name).touch()
    collection = {
        "store": str(tmp_path / "store"),
        "parquet_dir": str(tmp_path),
        "license_policy": str(tmp_path / "policy.json"),
        "vt_recipes": [],
        "seed": 1,
    }
    (tmp_path / "collection.json").write_text(json.dumps(collection))
    tools = [{"id": name, "argv": ["not-installed", "{path}"]} for name in sorted(TOOLS)]
    (tmp_path / "tools.json").write_text(json.dumps(tools))
    return {
        "collection_config": str(tmp_path / "collection.json"),
        "metadata_dir": str(tmp_path),
        "collection_run": str(tmp_path / "collection"),
        "tools": str(tmp_path / "tools.json"),
    }


def test_validation_allows_unavailable_tools_but_rejects_bad_workers(tmp_path):
    settings = config(tmp_path)
    validate_build_config(settings)
    settings["verdict_workers"] = 0
    with pytest.raises(ValueError, match="verdict_workers"):
        validate_build_config(settings)
    assert not (tmp_path / "collection").exists()


def test_duplicate_tool_configuration_rejected(tmp_path):
    settings = config(tmp_path)
    path = Path(settings["tools"])
    tools = json.loads(path.read_text())
    tools[-1] = tools[0]
    path.write_text(json.dumps(tools))
    with pytest.raises(ValueError, match="exactly once"):
        validate_build_config(settings)


def test_missing_inventory_fails_before_detaching(tmp_path):
    settings = config(tmp_path)
    (tmp_path / "repository-files.parquet").unlink()
    with pytest.raises(ValueError, match="repository-files.parquet"):
        validate_build_config(settings)


def test_runtime_identity_covers_discovery_and_exports():
    hashes = implementation_hashes()
    assert {
        "unattended_sources.py",
        "candidate_export.py",
        "observation_cache.py",
        "configuration.py",
        "sources.py",
    } <= hashes.keys()
    assert all(len(value) == 64 for value in hashes.values())
