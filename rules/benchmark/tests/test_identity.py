# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
from copy import deepcopy

import pytest
from magika_rules_benchmark.identity import dataset_record, tool_record, utc_timestamp


def test_dataset_requires_name_version_and_matching_snapshot():
    result = {"compatibility": {"corpus": "a" * 64}, "quality": {"tool": {"files": 10}}}
    good = {
        "id": "sembiance",
        "name": "Sembiance",
        "version": "v3",
        "corpus_sha256": "a" * 64,
        "source_files": 20,
    }
    assert dataset_record(result, good)["scored_files"] == 10
    for change in [{"version": ""}, {"name": ""}, {"corpus_sha256": "b" * 64}, {"source_files": 5}]:
        with pytest.raises(ValueError):
            dataset_record(result, good | change)


def test_timestamp_is_timezone_aware_and_normalized():
    assert utc_timestamp("2026-09-08T18:36:35-04:00") == "2026-09-08T22:36:35Z"
    with pytest.raises(ValueError):
        utc_timestamp("2026-09-08T18:36:35")


def test_tool_identity_distinguishes_same_version_revisions_and_builds():
    result = {
        "revision": "a" * 40,
        "config": {
            "environment": {},
            "tools": [
                {
                    "id": "magika2-ml",
                    "adapter": "magika-jsonl",
                    "command": ["/bin/magika", "--rules=off"],
                    "settings": {
                        "backend": "CPU",
                        "rules": "off",
                        "readers": 2,
                        "threads": 2,
                        "internal_batch": 8,
                    },
                }
            ],
        },
        "tools": {
            "magika2-ml": {
                "version": "magika 2.0.0-dev standard_v3_3",
                "executable_sha256": "b" * 64,
                "artifacts": {},
            }
        },
    }
    first = tool_record(result, "magika2-ml")
    other = deepcopy(result)
    other["revision"] = "c" * 40
    second = tool_record(other, "magika2-ml")
    assert first["version"] != second["version"]
    assert first["config_id"] == second["config_id"]
    other["tools"]["magika2-ml"]["executable_sha256"] = "d" * 64
    assert tool_record(other, "magika2-ml")["build_id"] != second["build_id"]
    other["config"]["tools"][0]["settings"]["threads"] = 4
    assert tool_record(other, "magika2-ml")["config_id"] != first["config_id"]
    other["config"]["tools"][0]["settings"]["startup_backend"] = "CPU"
    assert "CPU warmup" in tool_record(other, "magika2-ml")["config"]
