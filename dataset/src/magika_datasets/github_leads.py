# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Leads from the frozen GitHub repository index, restricted to permitted repositories.

A suffix owned by one class names the file; a suffix several classes share is only a hint,
used when a validator will decide from content. Leads are ordered by the index itself, so
planning takes no seed and repeats exactly.
"""

import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path
from urllib.parse import quote

import pyarrow.parquet as pq

from .acquisition import object_path
from .filenames import extension_candidates, matches_name
from .origins import extension_owners as suffix_owners
from .refill import (
    MAX_FILE_BYTES,
    MAX_PER_REPOSITORY,
    PER_CLASS,
    UNKNOWN,
    held_counts,
    shortfall,
)
from .repository_diversity import github_repositories
from .validators import REGISTRY

OVERSAMPLE = 5
"""Most files behind a shared suffix will not prove, so a validator-backed class may draw this
many times its shortfall; refill stops fetching once the class is full."""


def _hex(value) -> str:
    return value.hex() if isinstance(value, bytes) else value


def held_blob_oids(samples, store: Path, max_file_bytes: int) -> set[str]:
    """Git blob ids of content already in the corpus.

    Only files within the size budget can match a lead, so only those are hashed. Without
    this a refill re-fetched boilerplate an earlier pass had already added, and discarded
    it on arrival.
    """
    oids = set()
    for sha, size in zip(samples.column("sha256").to_pylist(), samples.column("size").to_pylist()):
        if not 0 < size <= max_file_bytes:
            continue
        path = object_path(store, sha.hex())
        if not path.exists():
            continue
        digest = hashlib.sha1(b"blob %d\0" % size)
        digest.update(path.read_bytes())
        oids.add(digest.hexdigest())
    return oids


def extension_owners(classes: dict) -> dict[str, str]:
    """Suffixes exactly one ordinary class claims, so a lead's class is unambiguous.

    Same rule the provenance tier uses, minus the sink class: `unknown` owns no suffix and
    is not something to go looking for.
    """
    return {k: v for k, v in suffix_owners(classes).items() if v != UNKNOWN}


def plan(
    metadata: Path,
    *,
    per_class: int = PER_CLASS,
    max_per_repository: int = MAX_PER_REPOSITORY,
    max_file_bytes: int = MAX_FILE_BYTES,
    formats=(),
    store: Path | None = None,
) -> list[dict]:
    """Leads for every short class, from permitted repositories only."""
    metadata = Path(metadata)
    classes = {r["format_id"]: r for r in pq.read_table(metadata / "classes.parquet").to_pylist()}
    permitted = {
        r["repository"]
        for r in pq.read_table(
            metadata / "repository-licenses.parquet", columns=["repository", "allowed_by_policy"]
        ).to_pylist()
        if r["allowed_by_policy"]
    }
    policy = set(
        json.loads((metadata / "config/repository-license-policy.json").read_text())["allowed_spdx"]
    )
    samples = pq.read_table(metadata / "samples.parquet")
    held = held_counts(samples.to_pylist(), classes)
    seen_paths = set()
    for origins in samples.column("origins").to_pylist():
        for origin in origins:
            if origin.startswith("github:"):
                seen_paths.add(origin[7:].rsplit(":", 1)[0])
    wanted = shortfall(classes, held)
    if formats:
        wanted = {k: v for k, v in wanted.items() if k in formats}
    owners = extension_owners(classes)
    # A suffix several classes share cannot name a file, but it is still a hint when a
    # validator will decide from content. Every short, validator-backed class that lists the
    # suffix claims the file, and all of them are hinted so each relevant validator runs.
    claimants: dict[str, list[str]] = defaultdict(list)
    for kind in sorted(wanted):
        if kind in REGISTRY:
            for suffix in classes[kind]["extensions"] or []:
                claimants[suffix.lower()].append(kind)
    chosen: dict[str, list[dict]] = defaultdict(list)

    def quota(kind):
        # Most files behind a shared suffix will not prove, so a validator-backed class may
        # draw several times its shortfall; refill stops fetching once the class is full.
        return min(per_class, wanted[kind]) * (OVERSAMPLE if kind in REGISTRY else 1)

    # The cap is per class across the whole corpus, not per pass. Counting only this plan's
    # leads let each refill add ten more from the same repository, and repeated passes put a
    # hundred files from one project into a class.
    per_repo: dict[str, Counter] = defaultdict(Counter)
    for kind, origins in zip(
        samples.column("format_id").to_pylist(), samples.column("origins").to_pylist()
    ):
        for repository in github_repositories({"origins": origins}):
            per_repo[kind][repository] += 1
    # A Git object id names content, so boilerplate copied across repositories -- the same
    # .gitattributes in a hundred projects -- is one lead, not a hundred wasted fetches.
    oids: set[str] = held_blob_oids(samples, Path(store), max_file_bytes) if store else set()
    columns = [
        "repository", "revision", "spdx_id", "path", "size", "extension",
        "git_oid", "git_hash_algorithm", "regular_file",
    ]  # fmt: skip
    for batch in pq.ParquetFile(metadata / "repository-files.parquet").iter_batches(
        columns=columns
    ):
        for row in batch.to_pylist():
            if not row["regular_file"] or not row["git_oid"]:
                continue
            if not 0 < (row["size"] or 0) <= max_file_bytes:
                continue
            # Both gates, because the licence table is the published authority and the
            # inventory's own reading is what was pinned; a lead needs to satisfy each.
            name = row["repository"].lower()
            if name not in permitted or row["spdx_id"] not in policy:
                continue
            kind = owners.get((row["extension"] or "").lower())
            if kind is None and not row["extension"]:
                # Makefile, Dockerfile, .gitmodules: classes whose members are recognised
                # by a conventional name rather than a suffix, and which an extension-only
                # search can never refill.
                base = row["path"].rsplit("/", 1)[-1]
                kind = next((name for name in wanted if matches_name(name, base)), None)
            hints = [kind] if kind in wanted else []
            if not hints:
                shared = {
                    c for s in extension_candidates(row["path"]) for c in claimants.get(s, ())
                }
                kind = next(
                    (
                        c
                        for c in sorted(shared)
                        if len(chosen[c]) < quota(c) and per_repo[c][name] < max_per_repository
                    ),
                    None,
                )
                # Hint only the planned class. Hinting every claimant put lightgbm into a plain
                # .txt file's hints, and a generic encoding pass cannot settle a sample hinted
                # as something more specific; a real model is still proven by its own validator.
                hints = [kind] if kind else []
            if kind is None or kind not in wanted:
                continue
            if len(chosen[kind]) >= quota(kind):
                continue
            if per_repo[kind][name] >= max_per_repository:
                continue
            # The inventory stores Git identities as raw bytes; a permalink needs hex.
            revision = _hex(row["revision"])
            permalink = f"https://github.com/{row['repository']}/blob/{revision}/" + quote(
                row["path"], safe="/"
            )
            if (
                f"https://github.com/{row['repository']}/blob/{revision}/{row['path']}"
                in seen_paths
            ):
                continue
            oid = _hex(row["git_oid"])
            if oid in oids:
                continue
            oids.add(oid)
            per_repo[kind][name] += 1
            chosen[kind].append(
                {
                    "format_id": kind,
                    "repository": row["repository"],
                    "revision": revision,
                    "path": row["path"],
                    "size": row["size"],
                    "git_oid": _hex(row["git_oid"]),
                    "git_hash_algorithm": row["git_hash_algorithm"],
                    "spdx_id": row["spdx_id"],
                    "permalink": permalink,
                    "hints": hints,
                }
            )
    return [lead for kind in sorted(chosen) for lead in chosen[kind]]
