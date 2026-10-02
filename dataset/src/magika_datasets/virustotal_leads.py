# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Leads from VirusTotal, found by its own labels and settled by the validators.

VirusTotal's type tag and Magika label find candidates, and pairing them with NOT fetches the
files where they disagree -- the hard cases. Every label here is a hint. See SAMPLING.md,
Standing decisions.
"""

import json
import threading
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from queue import Queue

import pyarrow.parquet as pq

from .refill import MAX_FILE_BYTES, PER_CLASS, acquire, held_counts, shortfall
from .validation import tool_consensus
from .validators import REGISTRY
from .virustotal import VTError

OVERSAMPLE = 10
"""Leads per unit of shortfall. Most candidates for a class will not prove, and a query stops
early once it stops paying, so the surplus costs search results rather than downloads."""
PAGE = 40
"""Search page size. Each result carries full file attributes, so a 300-result page is slow to
read and can pass the client's 8 MiB response ceiling, losing the query."""
IDLE_PAGES = 3
"""Consecutive pages that add nothing before a query is abandoned, so a productive query pages
as deep as it pays and an unproductive one stops instead of reading its whole result set."""


def vt_queries(kind: str, row: dict, saved: list[str]) -> list[str]:
    """Searches for one class, most specific first.

    VirusTotal's own type tag and Magika label find the class directly, and pairing them
    with NOT fetches the files where the two disagree -- the hard cases. VirusTotal only
    finds the files; the structural validators decide what they are. The saved filename
    queries come last because a name is the weakest signal.
    """
    labels = json.loads(row["metadata_json"] or "{}").get("magika", {}).get("output_labels", [])
    queries = [f"type:{kind}"]
    for label in labels:
        queries += [
            f"magika:{label}",
            f"magika:{label} NOT type:{kind}",
            f"type:{kind} NOT magika:{label}",
        ]
    return list(dict.fromkeys(queries + saved))


def fetchable(kind: str, fixture: dict, classes: dict) -> bool:
    """Whether a downloaded file could be labelled at all.

    A class with a structural validator can be proven from bytes, so every candidate is
    worth fetching, disagreement included. Without one, a VirusTotal file can only be
    labelled by two non-Magika detectors agreeing, which its markings already show before
    any download.
    """
    if kind in REGISTRY:
        return True
    return (
        tool_consensus(
            {"vt_markings_history": [fixture.get("claim") or {}]}, {"observations": []}, classes
        )
        == kind
    )


def context(metadata: Path, formats=()) -> dict:
    """Classes, short classes, digests already held and saved queries, read once."""
    metadata = Path(metadata)
    classes = {r["format_id"]: r for r in pq.read_table(metadata / "classes.parquet").to_pylist()}
    samples = pq.read_table(metadata / "samples.parquet")
    wanted = shortfall(classes, held_counts(samples.to_pylist(), classes))
    if formats:
        wanted = {k: v for k, v in wanted.items() if k in formats}
    saved: dict[str, list[str]] = defaultdict(list)
    for entry in json.loads((metadata / "config/vt-queries.json").read_text())["queries"]:
        saved[entry["name"]].append(entry["query"])
    known = {sha.hex() for sha in samples.column("sha256").to_pylist()}
    return {"classes": classes, "wanted": wanted, "saved": saved, "seen": known}


def candidates(client, kind, ctx, *, need, max_file_bytes, lock, full=None, held=None):
    """Worth-fetching leads for one class, paging each query while it keeps paying.

    Progress is the class count when refill reports it (admission happens elsewhere), and
    otherwise the number of new leads a page produced.
    """
    classes = ctx["classes"]
    budget = f" size:{max_file_bytes // (1024 * 1024)}MB-" if max_file_bytes >= 1024 * 1024 else ""
    found = 0

    def done():
        return found >= need or (full is not None and full(kind))

    for query in vt_queries(kind, classes[kind], ctx["saved"].get(kind, [])):
        cursor, idle = None, 0
        before = held(kind) if held else 0
        while not done() and idle < IDLE_PAGES:
            try:
                page = client.search(
                    query if "size:" in query else query + budget,
                    limit=min(PAGE, need - found),
                    cursor=cursor,
                )
            except VTError as error:
                if error.stop_acquisition:
                    raise
                break  # a query VirusTotal rejects is skipped, not fatal
            produced = 0
            for fixture in page["fixtures"]:
                if done():
                    break
                if not 0 < fixture["size"] <= max_file_bytes or not fetchable(
                    kind, fixture, classes
                ):
                    continue
                with lock:
                    if fixture["sha256"] in ctx["seen"]:
                        continue
                    ctx["seen"].add(fixture["sha256"])
                found += 1
                produced += 1
                yield {**fixture, "format_id": kind, "query": query}
            if held:
                now = held(kind)
                idle, before = (0 if now > before else idle + 1), now
            else:
                idle = 0 if produced else idle + 1
            cursor = page["next_cursor"]
            if not cursor:
                break
        if done():
            break


def plan_virustotal(
    metadata, client, *, per_class=PER_CLASS, max_file_bytes=MAX_FILE_BYTES, formats=(), **_
):
    """Every lead a run would try, without downloading anything."""
    ctx, lock = context(metadata, formats), threading.Lock()
    return [
        lead
        for kind in sorted(ctx["wanted"])
        for lead in candidates(
            client,
            kind,
            ctx,
            need=min(per_class, ctx["wanted"][kind]) * OVERSAMPLE,
            max_file_bytes=max_file_bytes,
            lock=lock,
        )
    ]


def stream_virustotal(
    metadata,
    store,
    client_factory,
    *,
    per_class=PER_CLASS,
    max_file_bytes=MAX_FILE_BYTES,
    formats=(),
    workers=8,
    **_,
):
    """Search and download class by class, in parallel, yielding files as they arrive.

    Returns a function of refill's full and held predicates, so both sides share one view
    of how far each class has got.
    """
    ctx, lock = context(metadata, formats), threading.Lock()

    def produce(full, held):
        results: Queue = Queue(maxsize=workers * 4)
        finished = object()

        def work(kind):
            client = client_factory()
            try:
                for lead in candidates(
                    client,
                    kind,
                    ctx,
                    need=min(per_class, ctx["wanted"][kind]) * OVERSAMPLE,
                    max_file_bytes=max_file_bytes,
                    lock=lock,
                    full=full,
                    held=held,
                ):
                    lead["_digest"] = acquire(lead, Path(store), client)
                    results.put(lead)
            finally:
                results.put(finished)

        with ThreadPoolExecutor(max_workers=workers) as pool:
            for kind in sorted(ctx["wanted"]):
                pool.submit(work, kind)
            remaining = len(ctx["wanted"])
            while remaining:
                item = results.get()
                if item is finished:
                    remaining -= 1
                else:
                    yield item

    return produce
