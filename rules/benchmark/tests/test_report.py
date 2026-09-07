# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import pytest
from magika_rules_benchmark.report import auc, confusion, metrics, performance_rows, render


def test_confusion_arithmetic():
    result = confusion(3, 1, 2, 4)
    assert result["precision"] == 0.75
    assert result["recall"] == 0.6
    assert result["f1"] == pytest.approx(2 / 3)
    assert result["fp_rate"] == 0.2
    assert confusion(0, 0, 0)["precision"] is None


def test_auc_ties_and_missing_denominator():
    assert auc({1.0: [1, 0], 0.0: [0, 1]}) == 1
    assert auc({0.5: [1, 1]}) == 0.5
    assert auc({1.0: [0, 1], 0.0: [1, 0]}) == 0
    assert auc({1.0: [2, 0]}) is None


def test_supported_headline_keeps_outside_class_fp(observations):
    values = metrics(observations)
    assert values["supported_samples"] == 3
    assert values["classification"]["rules"]["fp"] == 0
    assert values["classification"]["rules"]["fn"] == 2
    rule = values["per_rule"]["png_rule"]
    assert (rule["tp"], rule["fp"], rule["fn"]) == (1, 1, 1)
    assert rule["status"] == "not_working"
    assert rule["conflicts"] == {"extra": 1}


def test_partial_then_full_and_disabled(observations):
    observations["samples"].pop()
    assert metrics(observations)["per_rule"]["png_rule"]["status"] == "partial"
    observations["samples"].pop(1)
    assert metrics(observations)["per_rule"]["png_rule"]["status"] == "full"
    observations["rules"]["png_rule"]["enforced"] = False
    assert metrics(observations)["per_rule"]["png_rule"]["status"] == "not_working"


def test_single_deterministic_family_table_and_no_private_paths(observations):
    observations["samples"][0]["path"] = "/Users/private/laptop/file"
    text = render(observations)
    assert text == render(observations)
    assert "/Users/" not in text
    assert "extra (extra): 1" in text
    assert "### image" in text
    assert "none" in text
    assert "Per-rule evaluation" not in text


def test_median_time_and_peak_memory():
    data = dict(
        measurements=[
            dict(
                backend="cpu",
                files=100,
                hit_percent=50,
                workers=4,
                mode="hybrid",
                trials=[
                    dict(seconds=t, peak_rss_bytes=r * 2**20)
                    for t, r in [(1, 10), (9, 30), (2, 20)]
                ],
            )
        ]
    )
    result = performance_rows(data)["cpu", 100, 50, 4, "hybrid"]
    assert result == dict(ms=2000, fps=50, rss=30)


def test_enforced_failures_block_but_disabled_candidates_do_not(observations):
    from magika_rules_benchmark.report import failures

    assert failures(observations)
    observations["rules"]["png_rule"]["enforced"] = False
    assert failures(observations) == []
    observations["samples"][0]["reference_mismatch"] = True
    assert failures(observations) == ["reference_mismatches: 1"]


def test_multiclass_false_positive_rate_has_negative_denominator(observations):
    values = metrics(observations)["classification"]["ml"]
    assert values["fp"] == 1
    assert values["tn"] == 2
    assert values["fp_rate"] == pytest.approx(1 / 3)
