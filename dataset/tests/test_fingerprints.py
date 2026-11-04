import json

import ppdeep
import pyarrow.parquet as pq

from magika_datasets.fingerprints import fill


def fingerprints(metadata):
    return [
        json.loads(row["annotation_json"]).get("content_fingerprint")
        for row in pq.read_table(metadata / "samples.parquet").to_pylist()
    ]


def test_missing_fingerprints_are_computed_from_the_stored_bytes(corpus):
    metadata, store, _ = corpus
    before = pq.read_table(metadata / "samples.parquet").to_pylist()
    receipt = fill(metadata, store, workers=1)
    assert receipt == {"samples": 2, "fingerprinted": 2, "unavailable": 0}
    after = pq.read_table(metadata / "samples.parquet").to_pylist()
    for old, new in zip(before, after, strict=True):
        digest = new["sha256"].hex()
        path = store / "objects" / digest[:2] / digest
        stored = json.loads(new["annotation_json"])["content_fingerprint"]["ssdeep"]
        assert stored == ppdeep.hash_from_file(str(path))
        assert {k: v for k, v in old.items() if k != "annotation_json"} == {
            k: v for k, v in new.items() if k != "annotation_json"
        }


def test_a_second_run_changes_nothing(corpus):
    metadata, store, _ = corpus
    fill(metadata, store, workers=1)
    first = (metadata / "samples.parquet").read_bytes()
    assert fill(metadata, store, workers=1)["fingerprinted"] == 0
    assert (metadata / "samples.parquet").read_bytes() == first
