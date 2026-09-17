import json

import pyarrow as pa
import pyarrow.parquet as pq
import pytest

from magika_datasets.sources import export_sources


def test_used_repositories_unique_counts_and_revision_evidence(tmp_path):
    sha, revision = "a" * 64, "b" * 40
    origin = f"github:https://github.com/Owner/Repo/blob/{revision}/file.py:{sha}"
    row = {
        "sha256": bytes.fromhex(sha),
        "origins": [origin],
        "annotation_json": json.dumps(
            {"repository_licenses": [{"source": "github:Owner/Repo", "spdx_id": "MIT"}]}
        ),
    }
    samples = tmp_path / "samples.parquet"
    pq.write_table(pa.Table.from_pylist([row, row]), samples)
    out = tmp_path / "used.parquet"
    assert export_sources(samples, out) == 1
    result = pq.read_table(out).to_pylist()[0]
    assert result["samples"] == 1 and result["revisions"] == [revision]
    assert json.loads(result["license_observations_json"])[0]["spdx_id"] == "MIT"
    row["origins"] = [origin.replace(revision, "main")]
    pq.write_table(pa.Table.from_pylist([row]), samples)
    with pytest.raises(ValueError, match="pinned"):
        export_sources(samples, out)


def test_the_export_is_bound_to_its_inputs_by_a_receipt(tmp_path):
    from magika_datasets.acquisition import sha256_file
    from magika_datasets.sources import main

    sha, revision = "a" * 64, "b" * 40
    row = {
        "sha256": bytes.fromhex(sha),
        "origins": [f"github:https://github.com/o/r/blob/{revision}/f.py:{sha}"],
        "annotation_json": "{}",
    }
    samples, out = tmp_path / "samples.parquet", tmp_path / "used.parquet"
    pq.write_table(pa.Table.from_pylist([row]), samples)
    receipt = tmp_path / "used-receipt.json"
    main(
        [
            "--samples",
            str(samples),
            "--pool",
            str(tmp_path / "absent.parquet"),
            "--output",
            str(out),
            "--receipt",
            str(receipt),
        ]
    )
    assert json.loads(receipt.read_text()) == {
        "repositories": 1,
        "samples_sha256": sha256_file(samples),
        "inventory_sha256": None,
        "parquet_sha256": sha256_file(out),
    }
