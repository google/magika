# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""The receipt binding the published metadata is rewritten whenever a table changes."""

import json

import pytest

from magika_datasets.parquet_corpus import check_sources
from magika_datasets.receipt import refresh


def test_a_refreshed_receipt_accepts_its_own_metadata(corpus):
    metadata, _, _ = corpus
    written = refresh(metadata)
    receipt = json.loads((metadata / "corpus-parquet-receipt.json").read_text())
    assert receipt == written
    assert receipt["samples"] == 2 and receipt["classes"] == 3
    check_sources(metadata / "samples.parquet", metadata / "classes.parquet", receipt)


def test_a_stale_receipt_is_refused(corpus):
    metadata, _, _ = corpus
    receipt = refresh(metadata)
    (metadata / "samples.parquet").write_bytes((metadata / "samples.parquet").read_bytes() + b"\0")
    with pytest.raises(ValueError, match="differs from its receipt"):
        check_sources(metadata / "samples.parquet", metadata / "classes.parquet", receipt)


def test_prior_fields_survive_a_refresh(corpus):
    metadata, _, _ = corpus
    path = metadata / "corpus-parquet-receipt.json"
    path.write_text(json.dumps({"metadata_scope": "kept", "samples": 1, "files": {}}))
    receipt = refresh(metadata)
    assert receipt["metadata_scope"] == "kept", "prose describing the export is not derived"
    assert receipt["samples"] == 2, "counts are"
