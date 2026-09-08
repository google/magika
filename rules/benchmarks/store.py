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


def store(source, destination):
    source, destination = Path(source), Path(destination)
    result = comparison.load_json(source / "results.json")
    if result["status"] not in ("complete", "partial"):
        raise ValueError("Cannot publish unfinished measurements")
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
    index_path = destination.parent / "index.json"
    index = json.loads(index_path.read_text()) if index_path.exists() else {"schema": 1, "runs": []}
    index["runs"].append(
        dict(
            id=destination.name,
            benchmark_version=result["benchmark_version"],
            revision=result["revision"],
            status=result["status"],
            artifacts_sha256=corpus.file_hash(destination / "artifacts.json"),
        )
    )
    corpus.atomic_json(index_path, index)
    return receipt


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    store(args.source, args.destination)
