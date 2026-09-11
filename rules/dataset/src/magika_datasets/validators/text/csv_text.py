# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""CSV and TSV tables: consistent field counts across rows for the hinted delimiter."""

import csv
import io

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("csv", "tsv")
SCOPE = "UTF-8 text without NUL bytes, csv module parse with the hinted delimiter, at least two rows with a consistent field count and more than one field; hint required"
REQUIRES_HINT = True
CONTEXT_REQUIRED = True
ROWS = 1_000_000


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    kind = "tsv" if "tsv" in hints and "csv" not in hints else "csv"
    if b"\0" in data:
        return Observation("fail", "NUL bytes in text", kind)
    try:
        text = data.decode("utf-8-sig")
    except UnicodeDecodeError:
        return Observation("fail", "Not UTF-8 text", kind)
    delimiter = "\t" if kind == "tsv" else ","
    counts = set()
    rows = 0
    try:
        for row in csv.reader(io.StringIO(text), delimiter=delimiter):
            if not row:
                continue
            rows += 1
            if rows > ROWS:
                return Observation("inconclusive", "Row budget exceeded", kind)
            counts.add(len(row))
    except csv.Error as error:
        return Observation("fail", f"csv module: {error}", kind)
    if rows < 2 or counts == {1}:
        return Observation("inconclusive", "Fewer than two rows or a single column", kind)
    if len(counts) != 1:
        return Observation("fail", f"Inconsistent field counts {sorted(counts)}", kind)
    return Observation(
        "pass", f"{rows} rows of {counts.pop()} fields with delimiter {delimiter!r}", kind
    )
