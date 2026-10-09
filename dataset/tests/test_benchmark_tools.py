# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import json
import os
import venv

import pytest

from magika_datasets.benchmark import tools


def test_mime_mapping_does_not_choose_between_ambiguous_classes():
    classes = {
        "zip": {"mimes": ["application/zip"]},
        "apk": {"mimes": ["application/zip"]},
        "pdf": {"mimes": ["application/pdf"]},
    }
    mapping = tools.label_mapping(classes, tools.Adapter.FILE)
    raw = b"a\0application/zip\0b\0application/pdf\0c\0application/octet-stream\0"
    rows = tools.parse_output("file-mime", raw, ["a", "b", "c"], mapping)
    assert [r["prediction"] for r in rows] == [None, "pdf", None]
    assert [r["mapping"] for r in rows] == ["ambiguous", "mapped", "abstained"]


def test_magika_output_must_keep_paths_and_records_errors():
    raw = json.dumps({"path": "a", "result": {"status": "permission_denied"}}).encode()
    assert tools.parse_output("magika-jsonl", raw, ["a"], {})[0]["error"] == "permission_denied"
    with pytest.raises(ValueError, match="paths"):
        tools.parse_output("magika-jsonl", raw, ["b"], {})


def test_magika_rule_decisions_are_marked_deterministic():
    value = {"output": {"label": "png"}, "dl": {"label": "undefined"}}
    raw = json.dumps({"path": "a", "result": {"status": "ok", "value": value}}).encode()
    row = tools.parse_output("magika-jsonl", raw, ["a"], {"png": ["png"]})[0]
    assert row["prediction"] == "png" and row["deterministic"] is True


def test_trid_takes_only_the_top_rank_and_abstains_on_ties():
    raw = (
        b"File: a\n 50.0% (.DOC) Word (1/1)\n 50.0% (.ZIP) Zip (1/1)\n"
        b"File: b\n 70.0% (.PDF) PDF (1/1)\n 30.0% (.DOC) Word (1/1)\nFile: c\n Unknown!\n"
    )
    rows = tools.parse_output(
        "trid", raw, ["a", "b", "c"], {"doc": ["doc"], "zip": ["zip"], "pdf": ["pdf"]}
    )
    assert [r["prediction"] for r in rows] == [None, "pdf", None]


@pytest.mark.parametrize("flag", ["--threads=2", "--readers=2", "--batch-size=8", "--threads"])
def test_resource_overrides_are_refused(flag):
    with pytest.raises(ValueError, match="defaults"):
        tools.validate_default_policy({"tools": [{"id": "magika", "command": ["magika", flag]}]})


def test_thread_caps_in_the_environment_are_refused():
    with pytest.raises(ValueError, match="defaults"):
        tools.validate_default_policy({"tools": [], "environment": {"OMP_NUM_THREADS": "2"}})


def test_identification_keeps_a_virtual_environment_interpreter(tmp_path):
    environment = tmp_path / "venv"
    venv.EnvBuilder(with_pip=False, symlinks=os.name != "nt").create(environment)
    executable = environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    tool = {
        "command": [str(executable)],
        "artifacts": {"python": str(executable)},
        "version_command": [str(executable), "-c", "import sys; print(sys.prefix)"],
    }
    identity = tools.identify(tool, os.environ.copy(), 10)
    assert identity["command"][0] == str(executable) and identity["version"] == str(environment)
