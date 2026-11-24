# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Leads for classes whose samples are generated rather than collected.

A generated lead is admitted like any other: its bytes land in the store, the validators
observe them, and the ladder labels them. What names the class is the reproducible origin,
so nothing here asserts a label.
"""

import hashlib
from pathlib import Path

import pyarrow.parquet as pq

from . import generated
from .refill import held_counts, shortfall


class Reader:
    """Regenerates a lead's bytes instead of downloading them."""

    def read_into(self, lead, output):
        data = generated.generate(lead["format_id"], lead["index"])
        output.write(data)
        return hashlib.sha256(data).hexdigest(), False


def plan(metadata: Path, formats=(), **_) -> list[dict]:
    """Enough indices to fill each short generated class, skipping samples already held."""
    metadata = Path(metadata)
    classes = {r["format_id"]: r for r in pq.read_table(metadata / "classes.parquet").to_pylist()}
    samples = pq.read_table(metadata / "samples.parquet")
    wanted = shortfall(classes, held_counts(samples.to_pylist(), classes))
    held = {sha.hex() for sha in samples.column("sha256").to_pylist()}
    leads = []
    for kind in generated.GENERATORS:
        if kind not in wanted or (formats and kind not in formats):
            continue
        need, index = wanted[kind], 0
        limit = generated.DISTINCT.get(kind)
        while need and (limit is None or index < limit):
            value, data = generated.origin(kind, index)
            digest = value.rsplit(":", 1)[1]
            if digest not in held:
                leads.append(
                    {
                        "provider": "generated",
                        "format_id": kind,
                        "index": index,
                        "origin": value,
                        "sha256": digest,
                        "size": len(data),
                    }
                )
                need -= 1
            index += 1
    return leads
