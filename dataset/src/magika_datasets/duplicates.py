# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Auditable ssdeep exclusion against the representatives actually selected."""

import re

import ppdeep


def signature(sample):
    value = sample.get("content_fingerprint", {}).get("ssdeep")
    if value is None:
        value = sample.get("vt_markings", {}).get("ssdeep")
    if not isinstance(value, str) or not re.fullmatch(
        r"[0-9]{1,12}:[A-Za-z0-9+/]{0,64}:[A-Za-z0-9+/]{0,64}", value
    ):
        return None
    block = int(value.split(":", 1)[0])
    factor = block // 3
    if block % 3 or factor < 1 or factor & (factor - 1):
        return None
    return value


def informative(sample):
    value = signature(sample)
    return value is not None and max(map(len, value.split(":")[1:])) >= 7


def near_duplicate(sample, representatives, *, threshold=90):
    """Score is ssdeep's similarity score, not a percentage of identical bytes.

    Compare within the class against at most 100 selected representatives.
    Missing/degenerate signatures are explicitly unmeasured, never matches.
    """
    value = signature(sample)
    if not informative(sample):
        return None
    for other in representatives:
        previous = signature(other)
        if previous is None:
            continue
        largest = max(sample["size"], other["size"])
        if not largest or min(sample["size"], other["size"]) / largest < 0.8:
            continue
        score = ppdeep.compare(value, previous)
        if score >= threshold:
            return {
                "sha256": sample["sha256"],
                "representative": other["sha256"],
                "format_id": sample["format_ids"][0],
                "method": "ssdeep",
                "score": score,
                "threshold": threshold,
            }
    return None
