# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import hashlib
import json

import pyarrow as pa
import pyarrow.parquet as pq
import pytest

from magika_datasets.parquet_metadata import CLASS_SCHEMA, SAMPLE_SCHEMA


@pytest.fixture
def corpus(tmp_path):
    """A two-sample metadata directory and byte store, as validate_dataset expects them."""
    metadata, store = tmp_path / "metadata", tmp_path / "store"
    contents = [b"\x89PNG\r\n\x1a\n" + b"\0" * 16, b"plain text sample\n"]
    classes = [
        {
            "ordinal": ordinal,
            "format_id": name,
            "name": name.upper(),
            "categories": ["image"],
            "extensions": [name],
            "metadata_json": json.dumps({"counts": {"target": 100}}),
        }
        for ordinal, name in enumerate(("png", "txt", "unknown"))
    ]
    rows = []
    for ordinal, (content, name) in enumerate(zip(contents, ("png", "txt"), strict=True)):
        digest = hashlib.sha256(content).hexdigest()
        target = store / "objects" / digest[:2] / digest
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(content)
        rows.append(
            {
                "class_ordinal": ordinal,
                "sample_ordinal": 0,
                "format_id": name,
                "label_status": "need_review",
                "sha256": bytes.fromhex(digest),
                "size": len(content),
                "origins": [f"vt:{digest}"],
                "hard_case": False,
                "annotation_json": json.dumps({"format_ids": [name]}),
            }
        )
    metadata.mkdir(parents=True, exist_ok=True)
    pq.write_table(pa.Table.from_pylist(classes, schema=CLASS_SCHEMA), metadata / "classes.parquet")
    pq.write_table(pa.Table.from_pylist(rows, schema=SAMPLE_SCHEMA), metadata / "samples.parquet")
    return metadata, store, tmp_path / "validation.parquet"
