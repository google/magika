# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Observe disagreements without resolving them or assigning accepted labels."""

import json
import re
from collections import defaultdict

from .detectors import detector_evidence
from .local_detector_labels import local_file_kind, local_trid_evidence
from .local_detector_labels_v2 import description_kind as v2_kind
from .local_detector_labels_v3 import description_kind as v3_kind
from .unattended import Search


def file_kind(raw):
    return v3_kind(raw) or v2_kind(raw) or local_file_kind(raw)


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


def conflict_plan(rows, recipes, records, *, per_class=100):
    """Every ordinary class gets a job, even full classes and missing-query cases."""
    formats = {r["format_id"]: r for r in rows}
    queries = defaultdict(list)
    for recipe in recipes:
        for item in recipe.get("queries", []):
            query = item["query"]
            if re.search(r"\b(?:fs|ls|first_submission|last_submission):", query):
                continue
            query = re.sub(r"\bsize:\d+(?:KB|MB|GB)-", "size:1MB-", query, flags=re.I)
            if query not in queries[item["name"]]:
                queries[item["name"]].append(query)
    existing = {s["sha256"]: r["format_id"] for r in rows for s in r["samples"]}
    cached = defaultdict(set)
    for record in records:
        if not 0 < record["size"] <= 1048576:
            continue
        for origin in record["origins"]:
            if origin.get("provider") != "virustotal" or not disagreement(
                origin.get("claim") or {}, formats
            ):
                continue
            for kind in {origin.get("discovery_format"), existing.get(record["sha256"])}:
                if kind in formats and kind != "invalid" and len(cached[kind]) < per_class:
                    cached[kind].add(record["sha256"])
    classes, jobs = {}, []
    for kind, row in sorted(formats.items()):
        if kind == "invalid" or row.get("class_role") != "format":
            continue
        if not queries[kind]:
            extensions = [
                s for s in row.get("extensions", []) if re.fullmatch(r"[A-Za-z0-9_+-]+", s)
            ]
            if extensions:
                queries[kind] = [
                    "(" + " OR ".join("name:*." + s for s in extensions[:5]) + ") size:1MB-"
                ]
        classes[kind] = {
            "selected": len(row["samples"]),
            "budget": per_class,
            "cached": sorted(cached[kind]),
            "role": "format",
        }
        jobs.append(
            {
                "id": kind + ":virustotal",
                "format_id": kind,
                "provider": "virustotal",
                "preferred": True,
                "fixtures": [],
                "queries": queries[kind],
            }
        )
    return {
        "schema_version": 1,
        "classes": classes,
        "jobs": jobs,
        "cached": {k: sorted(v) for k, v in cached.items()},
        "known_sha256": sorted(r["sha256"] for r in records),
        "selected_total": len(existing),
        "scope": "Dedicated VT disagreement candidates; not accepted labels",
    }


class ConflictSearch(Search):
    def __init__(self, root, plan, formats, max_pages=20):
        super().__init__(root, plan)
        self.formats, self.max_pages = formats, max_pages

    def __call__(self, job, state):
        if state.get("pages", 0) >= self.max_pages:
            return [], {"done": True, "reason": "page_budget_reached"}
        fixtures, update = super().__call__(job, state)
        retained = [f for f in fixtures if disagreement(f.get("claim") or {}, self.formats)]
        update["examined_new_candidates"] = state.get("examined_new_candidates", 0) + len(fixtures)
        update["disagreement_candidates"] = state.get("disagreement_candidates", 0) + len(retained)
        if update.get("pages", 0) >= self.max_pages:
            update.update(done=True, reason="page_budget_reached")
        if not job["queries"]:
            update["reason"] = "no_query_or_extension"
        return retained, update
