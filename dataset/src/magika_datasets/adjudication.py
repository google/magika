# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Recorded human decisions about specific samples, as attributed manual reviews."""

import json
import re
from pathlib import Path

DIGEST = re.compile(r"^[0-9a-f]{64}$")


def load(path: Path, formats: dict) -> dict[bytes, dict]:
    """Map each adjudicated sha256 to an annotation fragment the label ladder accepts.

    Adjudications carry no evidence of their own beyond who made them and where that is
    written down, so both are required: an unattributed correction is indistinguishable
    from the legacy "accepted" labels this replaces.
    """
    document = json.loads(Path(path).read_text())
    reviewer, evidence = document.get("reviewer"), document.get("evidence")
    if not reviewer:
        raise ValueError("Adjudications need a reviewer")
    if not evidence:
        raise ValueError("Adjudications need evidence naming where the decision is recorded")
    reviews: dict[bytes, dict] = {}
    for change in document["changes"]:
        digest, truth = change.get("sha256", ""), change.get("truth")
        if not DIGEST.match(digest):
            raise ValueError(f"Adjudication sha256 is not a digest: {digest!r}")
        if truth not in formats:
            raise ValueError(f"Adjudicated truth {truth!r} is not a class in the taxonomy")
        key = bytes.fromhex(digest)
        if key in reviews:
            raise ValueError(f"{digest} is adjudicated twice")
        reviews[key] = {
            "manual_validation": {
                "reviewer": reviewer,
                "decision": "validated",
                "evidence": evidence,
                "format_ids": [truth],
                "previous_truth": change.get("previous_truth"),
            }
        }
    return reviews
