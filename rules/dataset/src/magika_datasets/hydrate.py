# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Hydrate public Parquet metadata into locally verified whole-file Parquet shards."""

import argparse
import json
from pathlib import Path

from magika_datasets.acquisition import object_path, sha256_file
from magika_datasets.manifest import inventory
from magika_datasets.parallel_acquisition import collect_parallel
from magika_datasets.parquet_corpus import check_sources, metadata_rows, pack, verify

from .runtime import private_output


def main(argv=None, *, prog=None):
    parser = argparse.ArgumentParser(prog=prog, description=__doc__)
    parser.add_argument("--samples", type=Path, default=Path("samples.parquet"))
    parser.add_argument("--classes", type=Path, default=Path("classes.parquet"))
    parser.add_argument("--receipt", type=Path, default=Path("corpus-parquet-receipt.json"))
    parser.add_argument("--store", type=Path, default=Path(".local/corpus"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--download", action="store_true", help="Fetch missing bytes from pinned public origins"
    )
    parser.add_argument("--workers", type=int, default=8)
    parser.add_argument("--verify-only", action="store_true")
    parser.add_argument(
        "--validate", action="store_true", help="Write structural validation review groups"
    )
    args = parser.parse_args(argv)
    store, output = private_output(args.store), private_output(args.output)
    receipt = json.loads(args.receipt.read_text())
    check_sources(args.samples, args.classes, receipt)
    if not 1 <= args.workers <= 8:
        parser.error("Use 1..8 download workers")
    if args.verify_only:
        result = verify(output, args.samples, args.classes)
    else:
        if args.download:
            pending = []
            for row in metadata_rows(args.samples):
                sha = row["sha256"].hex()
                path = object_path(store, sha)
                if (
                    not path.is_file()
                    or path.stat().st_size != row["size"]
                    or sha256_file(path) != sha
                ):
                    pending.append(
                        {
                            "sha256": sha,
                            "size": row["size"],
                            "origins": row["origins"],
                            "format_ids": [row["format_id"]],
                        }
                    )
            if pending:
                fetched = collect_parallel(
                    inventory(pending),
                    store,
                    max_file_bytes=8 * 1024**2,
                    max_bytes=sum(r["size"] for r in pending),
                    max_files=len(pending),
                    vt_client=None,
                    workers=args.workers,
                )
                if fetched["failures"] or fetched["deferred"] or fetched["lfs_pointers"]:
                    raise ValueError("Hydration is incomplete; rerun to resume verified objects")
        result = pack(
            args.samples,
            args.classes,
            receipt,
            store,
            output,
            progress=lambda r: print(json.dumps(r), flush=True),
        )
    if args.validate:
        from .validation import validate_dataset

        result["validation"] = validate_dataset(
            args.samples.parent, store, output / "validation.parquet"
        )
    print(json.dumps(result, sort_keys=True), flush=True)
