# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""What detectors claim a file is, and whether they disagree. Nothing here assigns a label."""

import json

from .detectors import description_kind, detector_evidence, local_trid_evidence


def file_kind(raw):
    return description_kind(raw)


def claims(markings, formats):
    result = {
        name: value["format_ids"][0]
        for name, value in detector_evidence(markings, formats).items()
        if value["status"] == "mapped" and value["format_ids"][0] != "unknown"
    }
    kind = file_kind(markings.get("magic") or "")
    if kind in formats and kind != "unknown":
        result["file"] = kind
    return result


def disagreement(markings, formats):
    return len(set(claims(markings, formats).values())) > 1


def observed_conflict(markings_list, observations, formats):
    mapped = {}
    for number, markings in enumerate(markings_list):
        mapped.update(
            {f"vt.{number}.{name}": kind for name, kind in claims(markings, formats).items()}
        )
    for name in ("file", "trid", "magika"):
        observation = observations.get(name, {})
        if (
            observation.get("status") != "ok"
            or observation.get("exit_code") != 0
            or observation.get("output_truncated") is not False
        ):
            continue
        raw = observation.get("stdout", "")
        kind = None
        if name == "file":
            kind = file_kind(raw)
        elif name == "trid":
            evidence = local_trid_evidence(raw, formats)
            if evidence["status"] == "mapped":
                kind = evidence["format_ids"][0]
        else:
            try:
                value = json.loads(raw)
                if value.get("status") == "ok":
                    label = value["prediction"]["output"]["label"]
                    evidence = detector_evidence({"magika": label}, formats)["magika"]
                    if evidence["status"] == "mapped":
                        kind = evidence["format_ids"][0]
            except (ValueError, KeyError, TypeError, AttributeError):
                pass
        if kind in formats and kind != "unknown":
            mapped["local." + name] = kind
    return {
        "conflicting": len(set(mapped.values())) > 1,
        "mapped_claims": mapped,
        "policy": "observed_disagreement_no_adjudication_v1",
        "scope": "Observable differences only, including container/subtype differences; human reconciliation required",
    }
