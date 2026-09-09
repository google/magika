# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
import json
import os
import sys
import venv
from copy import deepcopy

import pytest
from magika_rules_benchmark import comparison as c
from magika_rules_benchmark.identity import tool_record


def legacy_result():
    return {
        "config": {
            "tools": [
                {
                    "id": "trid",
                    "adapter": "trid",
                    "command": ["python", "trid.py"],
                    "settings": {"strings": True, "stringzilla": "5.1.2"},
                }
            ]
        },
        "tools": {
            "trid": {
                "version": "TrID - File Identifier v2.48\n  Using Stringzilla: False",
                "executable_sha256": "a" * 64,
            }
        },
    }


def test_historical_trid_label_uses_observed_backend_without_rewriting_evidence():
    result = legacy_result()
    original = deepcopy(result)
    identity = tool_record(result, "trid")
    assert identity["config"] == "strings=True; StringZilla=off"
    assert identity["configuration"]["settings"]["stringzilla"] == "off"
    assert identity["corrections"][0]["declared"] == "5.1.2"
    assert result == original


def test_trid_missing_backend_evidence_is_not_assumed_enabled():
    result = legacy_result()
    result["tools"]["trid"]["version"] = "TrID - File Identifier v2.48"
    assert "StringZilla=unverified" in tool_record(result, "trid")["config"]


@pytest.mark.parametrize("requested,observed", [("5.1.2", False), ("off", True)])
def test_runtime_rejects_trid_backend_mismatch(tmp_path, requested, observed):
    script = tmp_path / "trid.py"
    script.write_text(f"print('TrID - File Identifier v2.48\\n  Using Stringzilla: {observed}')")
    tool = {
        "id": "trid",
        "adapter": "trid",
        "command": [sys.executable, str(script)],
        "version_command": [sys.executable, str(script), "-v"],
        "artifacts": {"script": str(script)},
        "settings": {"stringzilla": requested},
    }
    with pytest.raises(ValueError, match="StringZilla"):
        c.identify_tool(tool, os.environ.copy(), 10)


def test_identification_preserves_virtual_environment(tmp_path):
    environment = tmp_path / "venv"
    venv.EnvBuilder(with_pip=False, symlinks=os.name != "nt").create(environment)
    executable = environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    tool = {
        "id": "probe",
        "adapter": "magika-jsonl",
        "command": [str(executable)],
        "version_command": [str(executable), "-c", "import sys; print(sys.prefix)"],
        "artifacts": {"python": str(executable)},
    }
    identity = c.identify_tool(tool, os.environ.copy(), 10)
    assert identity["command"][0] == str(executable)
    assert identity["version"] == str(environment)
    prefix = c.invoke(
        [tool["command"][0], "-c", "import sys,json; print(json.dumps(sys.prefix))"],
        os.environ.copy(),
        10,
    )
    assert json.loads(prefix) == str(environment)


@pytest.mark.parametrize(
    "version,correct_artifact", [("5.1.2", True), ("9.0", True), ("5.1.2", False)]
)
def test_enabled_trid_checks_loaded_module_version_and_hash(tmp_path, version, correct_artifact):
    module = tmp_path / "stringzilla.py"
    module.write_text("__version__ = '5.1.2'\n")
    script = tmp_path / "trid.py"
    script.write_text(
        "import stringzilla\nprint('TrID - File Identifier v2.48\\n  Using Stringzilla: True')"
    )
    tool = {
        "id": "trid-stringzilla",
        "adapter": "trid",
        "command": [sys.executable, str(script)],
        "version_command": [sys.executable, str(script), "-v"],
        "artifacts": {
            "script": str(script),
            "stringzilla": str(module if correct_artifact else script),
        },
        "settings": {"strings": True, "stringzilla": version},
    }
    if version != "5.1.2" or not correct_artifact:
        with pytest.raises(ValueError, match="StringZilla version/artifact"):
            c.identify_tool(tool, os.environ.copy(), 10)
    else:
        identity = c.identify_tool(tool, os.environ.copy(), 10)
        assert identity["stringzilla"]["version"] == "5.1.2"
        report = tool_record(
            {"config": {"tools": [tool]}, "tools": {tool["id"]: identity}}, tool["id"]
        )
        assert report["config"] == "strings=True; StringZilla=5.1.2"
        assert not report["corrections"]
