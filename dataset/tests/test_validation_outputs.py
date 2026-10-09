# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""The counts summary must be attributable to the table it was computed from."""

import hashlib
import json

from magika_datasets.validation import validate_dataset


def test_summary_identifies_its_table(corpus):
    metadata, store, output = corpus
    result = validate_dataset(metadata, store, output)
    summary = json.loads(output.with_suffix(".json").read_text())
    assert summary["validation_sha256"] == hashlib.sha256(output.read_bytes()).hexdigest()
    assert summary["counts"] == result["counts"]


def test_provenance_labels_survive_a_revalidation(corpus):
    """A folded corpus keeps the labels only a pinned source path supports."""
    import pyarrow as pa
    import pyarrow.parquet as pq

    from magika_datasets.parquet_metadata import SAMPLE_SCHEMA

    metadata, store, output = corpus
    table = pq.read_table(metadata / "samples.parquet").to_pylist()
    blob = "github:https://github.com/o/r/blob/" + "a" * 40 + "/src/thing.png:"
    for row in table:
        row["format_id"] = "unknown" if row["format_id"] == "txt" else row["format_id"]
        row["class_ordinal"] = 2 if row["format_id"] == "unknown" else row["class_ordinal"]
        row["origins"] = [blob + row["sha256"].hex()]
    pq.write_table(pa.Table.from_pylist(table, SAMPLE_SCHEMA), metadata / "samples.parquet")
    result = validate_dataset(metadata, store, output)
    assert result["counts"]["validated_origin"] >= 1


def test_apply_writes_the_decision_back(corpus):
    """Without this, validate keeps reporting labels the published table does not carry."""
    import pyarrow.parquet as pq

    metadata, store, output = corpus
    before = pq.read_table(metadata / "samples.parquet", columns=["label_status"])
    assert set(before.column("label_status").to_pylist()) == {"need_review"}
    result = validate_dataset(metadata, store, output, apply=True)
    after = pq.read_table(metadata / "samples.parquet", columns=["label_status", "format_id"])
    statuses = after.column("label_status").to_pylist()
    assert result["applied"] is True
    assert statuses == [s for s in statuses if s in result["counts"]]
    assert sum(result["counts"].values()) == len(statuses)
    # Running it again is a no-op: the table is now a fixed point of its own labelling.
    second = validate_dataset(metadata, store, output, apply=True)
    assert second["counts"] == result["counts"]


def test_a_new_adjudication_is_applied_by_the_next_validation(corpus):
    """Corrections take effect on revalidation; nothing needs a one-off migration."""
    import pyarrow.parquet as pq

    metadata, store, output = corpus
    target = pq.read_table(metadata / "samples.parquet").to_pylist()[1]["sha256"].hex()
    (metadata / "config").mkdir(exist_ok=True)
    (metadata / "config/label-adjudications.json").write_text(
        json.dumps(
            {
                "reviewer": "maintainer",
                "evidence": "review notes",
                "changes": [{"sha256": target, "previous_truth": "txt", "truth": "png"}],
            }
        )
    )
    validate_dataset(metadata, store, output, apply=True)
    row = next(
        r
        for r in pq.read_table(metadata / "samples.parquet").to_pylist()
        if r["sha256"].hex() == target
    )
    assert (row["format_id"], row["label_status"]) == ("png", "validated_manual")
