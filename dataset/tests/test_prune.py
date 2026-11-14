# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""A sample stays only if the corpus may redistribute it."""

import json

import pyarrow as pa
import pyarrow.parquet as pq
import pytest

from magika_datasets.licenses import SCHEMA as LICENCE_SCHEMA
from magika_datasets.parquet_metadata import SAMPLE_SCHEMA
from magika_datasets.prune import prune


def licences(path, *entries):
    rows = [
        {
            "repository": name,
            "url": "https://github.com/" + name,
            "spdx_id": spdx,
            "license_name": None,
            "license_permalink": None,
            "revision": None,
            "basis": "frozen_repository_inventory" if spdx else "unresolved",
            "samples": 1,
            "allowed_by_policy": allowed,
        }
        for name, spdx, allowed in entries
    ]
    pq.write_table(pa.Table.from_pylist(rows, LICENCE_SCHEMA), path)
    return path


def sample(name, *, repo=None, vt=False, format_id="png", klass=0):
    digest = bytes([name]) + b"\0" * 31
    origins = []
    if repo:
        origins.append(f"github:https://github.com/{repo}/blob/{'a' * 40}/f.png:{digest.hex()}")
    if vt:
        origins.append("vt:" + digest.hex())
    return {
        "class_ordinal": klass,
        "sample_ordinal": name,
        "format_id": format_id,
        "label_status": "validated_auto",
        "sha256": digest,
        "size": 4,
        "origins": origins,
        "hard_case": False,
        "annotation_json": json.dumps({"format_ids": [format_id]}),
    }


@pytest.fixture
def tree(tmp_path, corpus):
    metadata, _, _ = corpus
    rows = [
        sample(1, repo="ok/permissive"),
        sample(2, repo="bad/copyleft"),
        sample(3, repo="who/knows"),
        sample(4, vt=True),
        sample(5, repo="bad/copyleft", vt=True),
    ]
    pq.write_table(pa.Table.from_pylist(rows, SAMPLE_SCHEMA), metadata / "samples.parquet")
    licences(
        metadata / "repository-licenses.parquet",
        ("ok/permissive", "MIT", True),
        ("bad/copyleft", "GPL-3.0", False),
        ("who/knows", None, False),
    )
    return metadata


def kept(metadata):
    return {r["sha256"][0] for r in pq.read_table(metadata / "samples.parquet").to_pylist()}


def test_a_permissive_repository_is_kept(tree):
    prune(tree)
    assert 1 in kept(tree)


def test_a_copyleft_repository_is_dropped(tree):
    receipt = prune(tree)
    assert 2 not in kept(tree)
    assert receipt["removed"]["outside_policy"] == 2
    assert receipt["removed_spdx"]["GPL-3.0"] == 2


def test_a_repository_with_no_established_licence_is_dropped(tree):
    receipt = prune(tree)
    assert 3 not in kept(tree)
    assert receipt["removed"]["unresolved"] == 1


def test_a_virustotal_sample_is_kept(tree):
    """The corpus policy permits VirusTotal origins; they assert no repository licence."""
    prune(tree)
    assert 4 in kept(tree)


def test_a_virustotal_copy_does_not_launder_a_copyleft_source(tree):
    """Where the bytes came from decides the terms, not which mirror served them."""
    prune(tree)
    assert 5 not in kept(tree)


def test_the_receipt_reports_what_each_class_lost(tree):
    receipt = prune(tree)
    assert receipt["samples_before"] == 5
    assert receipt["samples_after"] == 2
    assert receipt["by_class"]["png"] == {"before": 5, "after": 2}


def test_pruning_twice_removes_nothing_more(tree):
    prune(tree)
    again = prune(tree)
    assert again["samples_before"] == again["samples_after"] == 2
    assert again["removed"] == {}


def test_one_repository_cannot_fill_a_class(tree):
    """The earliest members are kept; later ones from the same repository go."""
    receipt = prune(tree, max_per_repository=0)
    assert 1 not in kept(tree), "even a permissive repository is capped"
    assert 4 in kept(tree), "a VirusTotal sample has no repository to cap"
    assert receipt["removed"]["repository_cap"] == 1
