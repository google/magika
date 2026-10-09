# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Selection draws a balanced evaluation subset from verified samples only."""

import json

import pyarrow as pa
import pyarrow.parquet as pq
import pytest

from magika_datasets.parquet_metadata import SAMPLE_SCHEMA
from magika_datasets.selection import select


def sample(index, format_id, *, label_status="validated_auto", repo="owner/repo", ssdeep=None):
    digest = bytes([index]) + b"\0" * 31
    annotation = {"format_ids": [format_id]}
    if ssdeep:
        annotation["content_fingerprint"] = {"ssdeep": ssdeep}
    return {
        "class_ordinal": 0 if format_id == "png" else 1,
        "sample_ordinal": index,
        "format_id": format_id,
        "label_status": label_status,
        "sha256": digest,
        "size": 4096,
        "origins": [
            f"github:https://github.com/{repo}/blob/{'a' * 40}/f{index}.png:{digest.hex()}"
        ],
        "hard_case": False,
        "annotation_json": json.dumps(annotation),
    }


def table(tmp_path, rows):
    path = tmp_path / "samples.parquet"
    pq.write_table(pa.Table.from_pylist(rows, schema=SAMPLE_SCHEMA), path)
    return path


def test_only_verified_samples_are_eligible(tmp_path):
    rows = [sample(1, "png"), sample(2, "png", label_status="need_review")]
    result, receipt = select(table(tmp_path, rows), per_class=10)
    assert [r["sha256"] for r in result] == [rows[0]["sha256"]]
    assert receipt["excluded"]["unverified"] == 1


def test_the_per_class_cap_holds(tmp_path):
    rows = [sample(i, "png", repo=f"owner/r{i}") for i in range(5)]
    result, receipt = select(table(tmp_path, rows), per_class=2)
    assert len(result) == 2
    assert receipt["selected_by_class"] == {"png": 2}
    assert receipt["excluded"]["class_full"] == 3


def test_one_repository_cannot_dominate_a_class(tmp_path):
    rows = [sample(i, "png", repo="one/repo") for i in range(5)]
    result, receipt = select(table(tmp_path, rows), per_class=10, max_per_repository=2)
    assert len(result) == 2
    assert receipt["excluded"]["repository_cap"] == 3


def test_the_repository_cap_is_per_class(tmp_path):
    rows = [sample(1, "png", repo="one/repo"), sample(2, "gif", repo="one/repo")]
    result, _ = select(table(tmp_path, rows), per_class=10, max_per_repository=1)
    assert {r["format_id"] for r in result} == {"png", "gif"}


def test_a_near_duplicate_is_excluded_against_what_was_selected(tmp_path):
    signature = "3072:" + "A" * 40 + ":" + "B" * 40
    rows = [
        sample(1, "png", repo="a/one", ssdeep=signature),
        sample(2, "png", repo="a/two", ssdeep=signature),
    ]
    result, receipt = select(table(tmp_path, rows), per_class=10)
    assert len(result) == 1
    assert receipt["excluded"]["near_duplicate"] == 1
    assert receipt["near_duplicates"][0]["representative"] == rows[0]["sha256"].hex()


def test_selection_is_deterministic(tmp_path):
    rows = [sample(i, "png", repo=f"owner/r{i}") for i in range(6)]
    path = table(tmp_path, rows)
    first, _ = select(path, per_class=3)
    second, _ = select(path, per_class=3)
    assert [r["sha256"] for r in first] == [r["sha256"] for r in second]


def test_a_negative_per_class_is_refused(tmp_path):
    with pytest.raises(ValueError, match="per_class"):
        select(table(tmp_path, [sample(1, "png")]), per_class=0)
