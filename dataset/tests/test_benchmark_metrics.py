# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import pytest

from magika_datasets.benchmark import metrics


def test_errors_and_abstentions_stay_in_the_accuracy_denominator():
    samples = [{"truth": "pdf"}, {"truth": "pdf"}, {"truth": "zip"}, {"truth": "zip"}]
    rows = [
        {"prediction": "pdf", "error": None},
        {"prediction": "zip", "error": None},
        {"prediction": None, "error": None},
        {"prediction": None, "error": "failed"},
    ]
    q = metrics.quality(samples, rows)
    assert (q["accuracy"], q["decision_coverage"], q["precision"], q["macro_recall"]) == (
        0.25,
        0.5,
        0.5,
        0.25,
    )


def test_hyperfine_json_is_the_source_of_timing():
    t = metrics.timing({"times": [0.2, 0.1, 0.3], "exit_codes": [0, 0, 0]}, 10)
    assert t["median_seconds"] == 0.2 and t["files_per_second"] == 50
    with pytest.raises(ValueError, match="nonzero"):
        metrics.timing({"times": [0.1], "exit_codes": [1]}, 1)
