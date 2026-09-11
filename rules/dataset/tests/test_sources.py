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
