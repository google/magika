# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Sembiance's samples import onto the shared store and taxonomy, hinted but unlabelled."""

import hashlib
import json

import pyarrow.parquet as pq

from magika_datasets import sembiance
from magika_datasets.acquisition import object_path

BASE = sembiance.BASE
SITE = {
    BASE: b'<a href="../">up</a><a href="archive/">archive</a><a href="https://elsewhere/x">x</a>',
    BASE + "archive/": b'<a href="arj/">arj</a><a href="?C=N;O=D">sort</a>',
    BASE + "archive/arj/": b'<a href="a.arj">a</a><a href="copy.arj">b</a><a href="big.arj">c</a>',
    BASE + "archive/arj/a.arj": b"\x60\xea arj bytes",
    BASE + "archive/arj/copy.arj": b"\x60\xea arj bytes",
    BASE + "archive/arj/big.arj": b"x" * 64,
}


def fake_fetch(url, limit):
    body = SITE[url]
    if len(body) > limit:
        raise OverflowError("too big")
    return body


def test_listings_only_yield_entries_under_the_listing():
    assert sembiance.children(BASE, SITE[BASE]) == [(BASE + "archive/", "directory")]


def test_crawl_downloads_into_the_store_and_resumes(tmp_path, monkeypatch):
    monkeypatch.setattr(sembiance, "fetch", fake_fetch)
    run, store = tmp_path / "run", tmp_path / "store"
    counts = sembiance.crawl(run, store, limit=32, workers=2)
    assert counts == {"done": 5, "excluded_size": 1}
    sha = hashlib.sha256(SITE[BASE + "archive/arj/a.arj"]).hexdigest()
    assert object_path(store, sha).exists()
    monkeypatch.setattr(sembiance, "fetch", lambda *a: (_ for _ in ()).throw(AssertionError))
    assert sembiance.crawl(run, store, limit=32) == counts  # nothing pending, nothing refetched


def test_export_writes_one_hinted_unlabelled_sample_per_file(tmp_path, monkeypatch, corpus):
    metadata, _, _ = corpus
    monkeypatch.setattr(sembiance, "fetch", fake_fetch)
    run, store, out = tmp_path / "run", tmp_path / "store", tmp_path / "sembiance"
    sembiance.crawl(run, store, limit=32)
    result = sembiance.export(run, store, out, metadata / "classes.parquet", {"archive/arj": "png"})
    assert result == {"samples": 1, "hinted": 1}
    (row,) = pq.read_table(out / "samples.parquet").to_pylist()
    assert (row["format_id"], row["label_status"]) == ("unknown", "need_review")
    assert len(row["origins"]) == 2 and all(o.startswith(BASE) for o in row["origins"])
    annotation = json.loads(row["annotation_json"])
    assert annotation["discovery_format_ids"] == ["png"] and annotation["source_claims"] == [
        "archive/arj"
    ]
    assert (out / "corpus-parquet-receipt.json").exists()
