# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import json
import os
import subprocess

import pytest
from magika_rules_benchmark import main


def test_installed_entrypoint():
    result = subprocess.run(["magika-rules-benchmark", "--help"], capture_output=True, text=True)
    assert result.returncode == 0
    assert "--dataset" in result.stdout
    assert "--phase" in result.stdout


def test_render_only_produces_one_markdown(tmp_path, observations):
    (tmp_path / "results.json").write_text(json.dumps(observations))
    with pytest.raises(SystemExit, match="Enforced rule"):
        main(["--phase", "render", "--output", str(tmp_path)])
    assert [path.name for path in tmp_path.glob("*.md")] == ["report.md"]
    assert "metrics" in json.loads((tmp_path / "results.json").read_text())


@pytest.mark.native
def test_real_product_and_reference_end_to_end(corpus_factory, tmp_path):
    dataset = corpus_factory()
    config = tmp_path / "model.json"
    config.write_text(json.dumps(dict(target_labels_space=["png", "gif"])))
    pack = tmp_path / "fixture.yar"
    pack.write_text(
        'rule fixture { meta: label = "png" enforced = true class = "full" '
        'fp_rate = 0 fn_rate = 0 strings: $a = "PNG" condition: $a at 0 }'
    )
    output = tmp_path / "result"
    main(
        [
            "--dataset",
            str(dataset),
            "--model-config",
            str(config),
            "--binary",
            os.environ["MAGIKA_TEST_BINARY"],
            "--rules-file",
            str(pack),
            "--output",
            str(output),
            "--phase",
            "quality",
        ]
    )
    raw = json.loads((output / "results.json").read_text())
    assert raw["samples"][0]["raw_matches"] == ["fixture"]
    assert raw["samples"][0]["hybrid_prediction"] == "png"
    assert raw["metrics"]["reference_mismatches"] == 0
    assert "performance" not in raw
    assert len(list(output.glob("*.md"))) == 1
