# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Use the dataset lane's Sembiance eligibility flags with frozen comparator aliases."""

import argparse
import json
from collections import Counter
from pathlib import Path

import pyarrow.parquet as pq
from magika_rules_benchmark import comparison as c
from magika_rules_benchmark import corpus


def prepare(snapshot, verified, taxonomy, output):
    source = c.load_json(verified / "inputs.json")
    manifest = c.load_json(snapshot / "manifest.json")
    assert corpus.file_hash(snapshot / "manifest.json") == source["identity"]["manifest_sha256"]
    base = c.load_json(taxonomy)
    by_hash = {s["sha256"]: s for s in source["samples"]}
    eligible, annotations = [], []
    classes = dict(base["classes"])
    added_classes = []
    for shard in manifest["shards"]:
        path = snapshot / shard["path"]
        assert corpus.file_hash(path) == shard["sha256"]
        for batch in pq.ParquetFile(path).iter_batches(
            columns=["sha256", "format_id", "annotation_json"], batch_size=256, use_threads=False
        ):
            for row in batch.to_pylist():
                sha = row["sha256"].hex()
                a = json.loads(row["annotation_json"])
                accepted = bool(a["evaluation_eligible"])
                annotations.append(
                    {
                        "sha256": sha,
                        "source_format": row["format_id"],
                        "evaluation_eligible": accepted,
                        "evaluation_basis": a.get("evaluation_basis"),
                        "validation_status": a.get("validation_status"),
                        "overlap_existing_sha256": a.get("overlap_existing_sha256"),
                        "source_claims": a.get("source_claims", []),
                    }
                )
                if not accepted:
                    continue
                assert not a.get("conflicting") and not a.get("overlap_existing_sha256")
                assert a["format_ids"] == [row["format_id"]]
                if row["format_id"] not in classes:
                    classes[row["format_id"]] = source["classes"][row["format_id"]]
                    added_classes.append(row["format_id"])
                eligible.append(
                    by_hash[sha]
                    | {
                        "truth": row["format_id"],
                        "ambiguous": False,
                        "evaluation_basis": a["evaluation_basis"],
                    }
                )
    assert len(annotations) == manifest["samples"] == len(by_hash)
    assert len({r["sha256"] for r in annotations}) == len(by_hash)
    output.mkdir(parents=True, exist_ok=False)
    identity = source["identity"] | {
        "scope": "Sembiance evaluation_eligible subset",
        "eligible_samples": len(eligible),
        "eligible_classes": len({s["truth"] for s in eligible}),
        "label_bases": dict(Counter(s["evaluation_basis"] for s in eligible)),
        "excluded_samples": len(annotations) - len(eligible),
        "taxonomy_inputs_sha256": corpus.file_hash(taxonomy),
        "additional_source_classes": sorted(added_classes),
        "near_duplicates": "not certified",
        "baseline_dataset": "unchanged; external evaluation only",
    }
    c.save_gzip(
        output / "inputs.json.gz",
        {
            "identity": identity,
            "classes": classes,
            "supported": base.get("supported", []),
            "samples": eligible,
        },
    )
    c.save_gzip(output / "annotations.json.gz", annotations)
    corpus.atomic_json(output / "selection.json", identity)
    print(json.dumps(identity, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["snapshot", "verified", "taxonomy", "output"]:
        parser.add_argument(name, type=Path)
    a = parser.parse_args()
    prepare(a.snapshot.resolve(), a.verified.resolve(), a.taxonomy.resolve(), a.output.resolve())
