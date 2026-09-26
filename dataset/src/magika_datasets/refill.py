# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Refill short classes and admit only what verifies into one.

Leads come from the GitHub index (github_leads) or VirusTotal (virustotal_leads). Every lead
is fetched, validated and labelled by the same ladder as the rest of the corpus; a file that
verifies as nothing, lands in a full class or exceeds a repository's cap is left out.
"""

import argparse
import json
import tempfile
from collections import Counter, defaultdict
from pathlib import Path

import ppdeep
import pyarrow.parquet as pq

from .acquisition import object_path, sha256_file
from .manifest import GitHubReader, github_blob_matches
from .origins import Provenance
from .parquet_metadata import MAX_SAMPLE_BYTES, write_samples
from .receipt import refresh
from .repository_diversity import github_repositories
from .selection import Selector
from .validation import PRECEDENCE, apply_label, sample_hints
from .validators import MAX_BYTES as VALIDATOR_BYTES
from .validators import observe

MAX_FILE_BYTES = min(VALIDATOR_BYTES, MAX_SAMPLE_BYTES)
"""A lead must be both provable and publishable: above the validators' ceiling it cannot be
proven, above the corpus's it cannot be hydrated."""
PER_CLASS = 100
MAX_PER_REPOSITORY = 10
UNKNOWN = "unknown"
CHECKPOINT = 200
"""Verified additions between table writes, so a long run does not lose hours to one crash."""


def shortfall(classes: dict, held: Counter) -> dict[str, int]:
    wanted = {}
    for name, row in classes.items():
        if name != UNKNOWN and _target(row) > held[name]:
            wanted[name] = _target(row) - held[name]
    return wanted


def _target(row: dict) -> int:
    return json.loads(row["metadata_json"] or "{}").get("counts", {}).get("target", 0)


def counting(rows, classes: dict, max_per_repository: int = MAX_PER_REPOSITORY) -> Selector:
    """What the held rows already count towards each class, decided as selection decides.

    A row counts only if it is verified, within its repository's cap and not a near
    duplicate of one already counted. Counting raw rows let 299 copies of one generated
    file fill a class that holds a single distinct sample.
    """
    selector = Selector(max_per_repository=max_per_repository)
    for row in rows:
        if row["format_id"] in classes:
            selector.offer(row, per_class=_target(classes[row["format_id"]]))
    return selector


def held_counts(rows, classes: dict) -> Counter:
    return Counter({kind: len(chosen) for kind, chosen in counting(rows, classes).chosen.items()})


def _provenance_of(lead: dict, digest: str) -> tuple[list[str], dict, str | None]:
    """Origins, evidence and capped repository for a lead, by where it came from."""
    if lead.get("provider") == "generated":
        # The origin is the evidence: it names the class only if the bytes regenerate.
        annotation = {"format_ids": [], "discovery_format_ids": [lead["format_id"]]}
        return [lead["origin"]], annotation, None
    if lead.get("provider") == "virustotal":
        # VirusTotal origins are permitted by the corpus policy and assert no repository
        # licence; the markings are detector evidence, never a label.
        annotation = {
            "format_ids": [],
            "discovery_format_ids": [lead["format_id"]],
            "discovery_queries": [lead["query"]],
            "vt_markings_history": [lead.get("claim") or {}],
        }
        return [f"vt:{digest}"], annotation, None
    annotation = {
        "format_ids": [],
        "discovery_format_ids": lead.get("hints") or [lead["format_id"]],
        "repository_licenses": [
            {"source": "github:" + lead["repository"].lower(), "spdx_id": lead["spdx_id"]}
        ],
    }
    return [f"github:{lead['permalink']}:{digest}"], annotation, lead["repository"].lower()


def acquire(lead: dict, store: Path, reader) -> str | None:
    """Fetch one lead into the byte store and return its SHA-256, or None if unusable."""
    store.mkdir(parents=True, exist_ok=True)
    known = lead.get("sha256")
    if known and object_path(store, known).exists():
        # Already downloaded, perhaps by an interrupted run: verify and reuse the bytes.
        if sha256_file(object_path(store, known)) == known:
            return known
    with tempfile.NamedTemporaryFile(dir=store, delete=False) as handle:
        temporary = Path(handle.name)
        try:
            digest, lfs = reader.read_into(lead, handle)
        except (OSError, ValueError):
            temporary.unlink(missing_ok=True)
            return None
    if lfs:
        temporary.unlink(missing_ok=True)
        return None
    target = object_path(store, digest)
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        temporary.unlink(missing_ok=True)
    else:
        temporary.replace(target)
    if not github_blob_matches(target, {**lead, "size": lead["size"]}):
        return None
    return digest


def refill(metadata: Path, store: Path, *, reader=None, leads=None, formats=(), **options) -> dict:
    metadata, store = Path(metadata), Path(store)
    reader = reader if reader is not None else GitHubReader()
    if leads is None:
        from .github_leads import plan

        leads = plan(metadata, formats=formats, store=store, **options)
    classes = {r["format_id"]: r for r in pq.read_table(metadata / "classes.parquet").to_pylist()}
    provenance = Provenance(classes)
    table = pq.read_table(metadata / "samples.parquet")
    grouped: dict[str, list[dict]] = defaultdict(list)
    per_repo: dict[str, Counter] = defaultdict(Counter)
    known = set()
    for row in table.to_pylist():
        grouped[row["format_id"]].append(row)
        known.add(row["sha256"])
        for repository in github_repositories(row):
            per_repo[row["format_id"]][repository] += 1
    cap = options.get("max_per_repository", MAX_PER_REPOSITORY)
    selector = counting(table.to_pylist(), classes, cap)
    added, statuses, failed = Counter(), Counter(), Counter()
    since_checkpoint = 0

    def held(kind):
        return len(selector.chosen[kind])

    def full(kind):
        return held(kind) >= _target(classes[kind])

    if callable(leads):
        leads = leads(full, held)
    planned = 0
    for lead in leads:
        planned += 1
        if full(lead["format_id"]):
            # Filled by an earlier lead in this run: skip the download entirely.
            failed["class_full"] += 1
            continue
        digest = lead["_digest"] if "_digest" in lead else acquire(lead, store, reader)
        if digest is None:
            failed["unavailable"] += 1
            continue
        if bytes.fromhex(digest) in known:
            failed["already_held"] += 1
            continue
        origins, annotation, repository = _provenance_of(lead, digest)
        row = {"format_id": UNKNOWN, "origins": origins, "sha256": bytes.fromhex(digest)}
        hints = sample_hints(row, annotation, classes, provenance)
        report = observe(object_path(store, digest), digest, hints=hints)
        decision = apply_label(annotation, report, classes, origins, None, provenance)
        status = decision["validation_status"]
        labels = decision.get("format_ids") or []
        kind = labels[0] if status in PRECEDENCE and labels else UNKNOWN
        if kind == UNKNOWN:
            # A refill exists to fill format classes. A file that verifies as nothing fills
            # none of them, so it is left out rather than added to the unverified pool.
            failed["unverified"] += 1
            continue
        if full(kind):
            # Validation placed it in a class that is already at target.
            failed["class_full"] += 1
            continue
        # The plan checked the cap for the class it expected; validation can place the file
        # in another, so the cap is checked again where the sample actually lands.
        if repository is not None and per_repo[kind][repository] >= cap:
            failed["repository_cap"] += 1
            continue
        blob = {k: v for k, v in decision.items() if k not in ("label_status", "hard_case")}
        blob.pop("validation_status", None)
        blob["format_ids"] = [kind]
        # Selection excludes near-duplicates by this fingerprint, so every admitted sample
        # carries one computed from its own bytes rather than a detector's copy.
        blob["content_fingerprint"] = {
            "ssdeep": ppdeep.hash_from_file(str(object_path(store, digest)))
        }
        row = {
            "class_ordinal": classes[kind]["ordinal"],
            "sample_ordinal": 0,
            "format_id": kind,
            "label_status": status,
            "sha256": bytes.fromhex(digest),
            "size": lead["size"],
            "origins": origins,
            "hard_case": bool(decision.get("hard_case")),
            "annotation_json": json.dumps(blob, sort_keys=True),
        }
        # A near duplicate of a sample already counted fills nothing, like an unverified one.
        reason = selector.offer(row, per_class=_target(classes[kind]))
        if reason is not None:
            failed[reason] += 1
            continue
        if repository is not None:
            per_repo[kind][repository] += 1
        grouped[kind].append(row)
        known.add(bytes.fromhex(digest))
        added[kind] += 1
        statuses[status] += 1
        since_checkpoint += 1
        if since_checkpoint >= CHECKPOINT:
            # A long VirusTotal run must not lose hours of verified additions to one crash.
            write_samples(grouped, classes, metadata / "samples.parquet")
            refresh(metadata)
            since_checkpoint = 0
    rows = write_samples(grouped, classes, metadata / "samples.parquet")
    refresh(metadata)
    return {
        "leads": planned,
        "added": sum(added.values()),
        "added_by_class": dict(sorted(added.items())),
        "label_status_counts": dict(sorted(statuses.items())),
        "not_added": dict(sorted(failed.items())),
        "samples": len(rows),
        "policy": (
            "GitHub leads come only from repositories the licence policy permits; "
            "VirusTotal origins are permitted by the corpus policy."
        ),
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, prog="magika-datasets refill")
    parser.add_argument("--metadata", type=Path, default=Path("."))
    parser.add_argument("--store", type=Path, default=Path("local/corpus"))
    parser.add_argument("--per-class", type=int, default=PER_CLASS)
    parser.add_argument("--max-per-repository", type=int, default=MAX_PER_REPOSITORY)
    parser.add_argument("--max-file-bytes", type=int, default=MAX_FILE_BYTES)
    parser.add_argument("--format", action="append", default=[], help="Limit to these classes")
    parser.add_argument("--source", choices=["github", "virustotal", "generated"], default="github")
    parser.add_argument("--workers", type=int, default=8)
    parser.add_argument("--plan-only", action="store_true")
    parser.add_argument("--receipt", type=Path, default=Path("refill-receipt.json"))
    args = parser.parse_args(argv)
    options = dict(
        per_class=args.per_class,
        max_per_repository=args.max_per_repository,
        max_file_bytes=args.max_file_bytes,
    )
    reader = leads = None
    if args.source == "virustotal":
        from .virustotal import Client
        from .virustotal_leads import plan_virustotal, stream_virustotal

        reader = Client()
        if args.plan_only:
            leads = plan_virustotal(args.metadata, reader, formats=args.format, **options)
        else:
            leads = stream_virustotal(
                args.metadata,
                args.store,
                Client,
                formats=args.format,
                workers=args.workers,
                **options,
            )
    if args.source == "generated":
        from .generated_leads import Reader
        from .generated_leads import plan as plan_generated

        reader = Reader()
        leads = plan_generated(args.metadata, formats=args.format)
    if args.plan_only:
        if leads is None:
            from .github_leads import plan

            leads = plan(args.metadata, formats=args.format, store=args.store, **options)
        counts = Counter(lead["format_id"] for lead in leads)
        print(
            json.dumps(
                {"leads": len(leads), "by_class": dict(sorted(counts.items()))}, sort_keys=True
            )
        )
        return
    result = refill(
        args.metadata, args.store, reader=reader, leads=leads, formats=args.format, **options
    )
    Path(args.receipt).write_text(json.dumps(result, indent=1, sort_keys=True) + "\n")
    print(json.dumps({k: v for k, v in result.items() if k != "added_by_class"}, sort_keys=True))
