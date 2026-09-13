# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Give every sample an ssdeep fingerprint of its own bytes.

Selection excludes near-duplicates by content_fingerprint.ssdeep. A sample without one is
never compared, so a class can look full while holding many copies of one file.
"""

import argparse
import json
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import ppdeep
import pyarrow as pa
import pyarrow.parquet as pq

from .acquisition import object_path
from .parquet_metadata import BATCH, SAMPLE_SCHEMA
from .receipt import refresh


def _hash(path: str) -> str | None:
    try:
        return ppdeep.hash_from_file(path)
    except OSError:
        return None


def fill(metadata: Path, store: Path, *, workers: int = 8) -> dict:
    """Fingerprint the samples that lack one; rows, order and ordinals are unchanged."""
    metadata, store = Path(metadata), Path(store)
    samples = metadata / "samples.parquet"
    rows = pq.read_table(samples).to_pylist()
    missing = [
        index
        for index, row in enumerate(rows)
        if "content_fingerprint" not in json.loads(row["annotation_json"])
    ]
    paths = [str(object_path(store, rows[index]["sha256"].hex())) for index in missing]
    with ProcessPoolExecutor(workers) as pool:
        hashes = list(pool.map(_hash, paths, chunksize=16))
    unavailable = 0
    for index, value in zip(missing, hashes):
        if value is None:
            unavailable += 1
            continue
        annotation = json.loads(rows[index]["annotation_json"])
        annotation["content_fingerprint"] = {"ssdeep": value}
        rows[index]["annotation_json"] = json.dumps(annotation, sort_keys=True)
    if missing and unavailable < len(missing):
        temporary = samples.with_suffix(".next")
        with pq.ParquetWriter(temporary, SAMPLE_SCHEMA, compression="zstd") as writer:
            for start in range(0, len(rows), BATCH):
                writer.write_table(
                    pa.Table.from_pylist(rows[start : start + BATCH], schema=SAMPLE_SCHEMA)
                )
        temporary.replace(samples)
        refresh(metadata)
    return {
        "samples": len(rows),
        "fingerprinted": len(missing) - unavailable,
        "unavailable": unavailable,
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, prog="magika-datasets fingerprint")
    parser.add_argument("--metadata", type=Path, default=Path("."))
    parser.add_argument("--store", type=Path, default=Path("local/corpus"))
    parser.add_argument("--workers", type=int, default=8)
    args = parser.parse_args(argv)
    print(json.dumps(fill(args.metadata, args.store, workers=args.workers), sort_keys=True))
