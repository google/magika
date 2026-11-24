# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""A hydrated snapshot built the way parquet_corpus writes one."""

import hashlib
import json

import pyarrow as pa
import pyarrow.parquet as pq

from magika_datasets.acquisition import sha256_file
from magika_datasets.parquet_corpus import metadata_line
from magika_datasets.parquet_metadata import SAMPLE_SCHEMA

SNAPSHOT_SCHEMA = SAMPLE_SCHEMA.append(pa.field("content", pa.binary()))


def build_snapshot(root, samples):
    """samples: (content, format_id, label_status) triples."""
    (root / "shards").mkdir(parents=True)
    names = sorted({name for _, name, _ in samples} | {"png", "gif"})
    classes = [
        dict(
            ordinal=i,
            format_id=name,
            name=name.upper(),
            categories=["image"],
            extensions=[name],
            metadata_json=json.dumps(
                dict(magika=dict(output_labels=[name], kb_labels=[name]), mimes=[f"image/{name}"])
            ),
        )
        for i, name in enumerate(names)
    ]
    pq.write_table(pa.Table.from_pylist(classes), root / "classes.parquet")
    rows = [
        dict(
            class_ordinal=names.index(name),
            sample_ordinal=i,
            format_id=name,
            label_status=status,
            sha256=hashlib.sha256(content).digest(),
            size=len(content),
            origins=["generated test fixture"],
            hard_case=False,
            annotation_json=json.dumps(dict(format_ids=[name])),
            content=content,
        )
        for i, (content, name, status) in enumerate(samples)
    ]
    pq.write_table(pa.Table.from_pylist(rows, SNAPSHOT_SCHEMA), root / "shards/000.parquet")
    digest = hashlib.sha256()
    for row in rows:
        digest.update(metadata_line({k: v for k, v in row.items() if k != "content"}))
    manifest = dict(
        format="parquet-whole-file-v1",
        samples=len(rows),
        original_bytes=sum(len(c) for c, _, _ in samples),
        metadata_rows_sha256=digest.hexdigest(),
        source_metadata_receipt=dict(
            files={"classes.parquet": dict(sha256=sha256_file(root / "classes.parquet"))}
        ),
        shards=[
            dict(
                path="shards/000.parquet",
                samples=len(rows),
                sha256=sha256_file(root / "shards/000.parquet"),
            )
        ],
    )
    (root / "manifest.json").write_text(json.dumps(manifest))
    return root
