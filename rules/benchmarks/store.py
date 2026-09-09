# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Store a completed run's JSON evidence, excluding corpus bytes and local caches."""

import argparse
import gzip
import io
import json
import tarfile
from pathlib import Path

from magika_rules_benchmark import comparison, corpus
from magika_rules_benchmark.identity import index_entry


def store(source, destination, dataset=None):
    source, destination = Path(source), Path(destination)
    result = comparison.load_json(source / "results.json")
    if result["status"] not in ("complete", "partial"):
        raise ValueError("Cannot publish unfinished measurements")
    if len(result["revision"]) < 40:
        revisions = {
            t["source_revision"]
            for t in result.get("config", {}).get("tools", [])
            if len(t.get("source_revision", "")) == 40
            and t["source_revision"].startswith(result["revision"])
        }
        if len(revisions) == 1:
            result["revision_argument"] = result["revision"]
            result["revision"] = revisions.pop()
    index_path = destination.parent / "index.json"
    index = json.loads(index_path.read_text()) if index_path.exists() else {"schema": 2, "runs": []}
    if destination.exists():
        raise FileExistsError(destination)
    dataset = dataset or result.get("config", {}).get("dataset")
    if dataset is None:
        dataset = next(
            (
                r["dataset"]
                for r in index["runs"]
                if r.get("dataset", {}).get("corpus_sha256") == result["compatibility"]["corpus"]
            ),
            None,
        )
    entry = index_entry(destination.name, result, dataset, None)
    for run in index["runs"]:
        old = run.get("dataset", {})
        if (old.get("id"), old.get("version")) == (
            entry["dataset"]["id"],
            entry["dataset"]["version"],
        ):
            if old != entry["dataset"]:
                raise ValueError(
                    "Dataset version already identifies a different snapshot or selection"
                )
    destination.mkdir(parents=True, exist_ok=False)
    comparison.save_gzip(destination / "results.json.gz", result)
    for name in ("config.json", "inputs.json.gz", "workloads.json", "label-mappings.json"):
        (destination / name).write_bytes((source / name).read_bytes())
    (destination / "report.md").write_text(comparison.render(result))
    files = [source / "observations.json.gz", *sorted((source / "raw").glob("*"))]
    with (destination / "raw-output.tar.gz").open("wb") as output:
        with gzip.GzipFile(fileobj=output, mode="wb", filename="", mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w|") as archive:
                for path in files:
                    data = path.read_bytes()
                    info = tarfile.TarInfo(str(path.relative_to(source)))
                    info.size = len(data)
                    info.mode = 0o644
                    archive.addfile(info, io.BytesIO(data))
    receipt = dict(
        benchmark_version=result["benchmark_version"],
        revision=result["revision"],
        status=result["status"],
        files={p.name: corpus.file_hash(p) for p in sorted(destination.iterdir())},
    )
    corpus.atomic_json(destination / "artifacts.json", receipt)
    entry["artifacts_sha256"] = corpus.file_hash(destination / "artifacts.json")
    index["schema"] = 2
    index["runs"].append(entry)
    corpus.atomic_json(index_path, index)
    return receipt


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    parser.add_argument(
        "--dataset",
        type=Path,
        help="JSON descriptor with dataset id, name, version and snapshot identity",
    )
    args = parser.parse_args()
    store(
        args.source, args.destination, comparison.load_json(args.dataset) if args.dataset else None
    )
