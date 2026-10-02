# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Bind the published metadata files to their digests.

Hydration refuses metadata that does not match this receipt, which is what stops a corpus
being built from tables that drifted apart. Anything that rewrites a published table has to
rewrite the receipt too, or the next hydration fails with no obvious cause.
"""

import argparse
import json
from pathlib import Path

import pyarrow.parquet as pq

from .acquisition import sha256_file
from .runtime import write

FILES = ("samples.parquet", "classes.parquet")
NAME = "corpus-parquet-receipt.json"


def refresh(metadata: Path) -> dict:
    """Recompute the derived fields, keeping the prose that describes the export."""
    metadata = Path(metadata)
    path = metadata / NAME
    receipt = json.loads(path.read_text()) if path.exists() else {}
    receipt["files"] = {
        name: {
            "sha256": sha256_file(metadata / name),
            "bytes": (metadata / name).stat().st_size,
        }
        for name in FILES
    }
    receipt["samples"] = pq.ParquetFile(metadata / "samples.parquet").metadata.num_rows
    receipt["classes"] = pq.ParquetFile(metadata / "classes.parquet").metadata.num_rows
    write(path, receipt)
    return receipt


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, prog="magika-datasets receipt")
    parser.add_argument("--metadata", type=Path, default=Path("."))
    args = parser.parse_args(argv)
    result = refresh(args.metadata)
    print(json.dumps({k: result[k] for k in ("samples", "classes", "files")}, sort_keys=True))
