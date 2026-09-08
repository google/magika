# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Freeze a collection plan from verified inventory and existing acquired bytes."""

import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path
from urllib.parse import quote

from .acquisition import acquired_records, sha256_file
from .filenames import NAMES, extension_candidates, matches_name
from .format_discovery import preferred_provider
from .manifest import validate_github_fixture
from .parquet_inventory import digest, load
from .repository_diversity import github_repositories
from .repository_pool import charge, draw_repositories


def path_matcher(rows):
    suffixes, names = defaultdict(set), defaultdict(set)
    for kind, row in rows.items():
        for suffix in row.get("extensions", []):
            suffixes[suffix.lower()].add(kind)
        for name in NAMES.get(kind, []):
            names[name].add(kind)

    def match(path):
        found = set(names.get(Path(path).name, ()))
        for suffix in extension_candidates(path):
            found.update(suffixes.get(suffix, ()))
        if "dockerfile" in rows and matches_name("dockerfile", Path(path).name):
            found.add("dockerfile")
        return found

    return match


def cached_candidates(records, rows, allowed, annotations=(), *, target=100):
    """Count a SHA once per claimed class; retain claims, never manufacture labels."""
    selected = {s["sha256"] for row in rows.values() for s in row["samples"]}
    prior = {a["sha256"]: a for a in annotations}
    result = defaultdict(list)
    repo_counts = {
        k: Counter(r for s in row["samples"] for r in github_repositories(s))
        for k, row in rows.items()
    }
    for record in sorted(records, key=lambda r: r["sha256"]):
        sha = record["sha256"]
        if sha in selected or not 0 < record["size"] <= 1048576:
            continue
        kinds = set()
        for origin in record["origins"]:
            kind = origin.get("discovery_format")
            if kind not in rows or rows[kind]["class_role"] != "format":
                continue
            if origin.get("provider") == "github":
                license_id = ((origin.get("claim") or {}).get("repository_license") or {}).get(
                    "spdx_id"
                )
                if record["size"] > 262144 or license_id not in allowed:
                    continue
            elif origin.get("provider") != "virustotal":
                continue
            a = prior.get(sha, {})
            if a.get("association") == "reviewed_negative" or a.get("format_ids") == ["invalid"]:
                continue
            if a.get("label_status") == "accepted" and a.get("format_ids") != [kind]:
                continue
            if len(result[kind]) < 3 * max(0, target - len(rows[kind]["samples"])):
                kinds.add(kind)
        repos = github_repositories(record)
        kinds = {k for k in kinds if all(repo_counts[k][repo] < 5 for repo in repos)}
        if kinds:
            p = Path(record["path"])
            if not p.is_file() or p.stat().st_size != record["size"] or sha256_file(p) != sha:
                continue
            for kind in sorted(kinds):
                result[kind].append(sha)
                repo_counts[kind].update(repos)
    return {kind: shas for kind, shas in result.items() if shas}


def prepare(config, progress):
    base, policy = Path(config["parquet_dir"]), Path(config["license_policy"])
    progress("Verifying fixed Parquet inventory and current dataset metadata")
    pool, entries, rows, files, inputs = load(base, policy)
    allowed = set(json.loads(policy.read_text())["allowed_spdx"])
    target = config.get("target_per_class", 100)
    rebalance = config.get("rebalance", False)
    budget = config.get("rebalance_candidates_per_class", 20)
    if type(target) is not int or target < 1 or type(budget) is not int or budget < 1:
        raise ValueError("Targets and candidate budgets must be positive integers")
    wanted = {k: r for k, r in rows.items() if rebalance or len(r["samples"]) < target}
    records = list(acquired_records(Path(config["store"])))
    annotations_path = config.get("annotations")
    annotations = (
        [json.loads(s) for s in Path(annotations_path).read_text().splitlines()]
        if annotations_path and Path(annotations_path).exists()
        else []
    )
    progress("Reusing size/hash-checked cached candidates; no format validation")
    selected = {s["sha256"] for row in rows.values() for s in row["samples"]}
    cached = (
        {}
        if rebalance
        else cached_candidates(
            [r for r in records if r["sha256"] not in selected],
            wanted,
            allowed,
            annotations,
            target=target,
        )
    )
    known_sha = {r["sha256"] for r in records}
    known_oid = {o.get("git_oid") for r in records for o in r["origins"]}
    counts = {
        k: {
            "selected": len(r["samples"]),
            "budget": budget if rebalance else 3 * max(0, target - len(r["samples"])),
            "cached": cached.get(k, []),
            "role": r["class_role"],
        }
        for k, r in wanted.items()
    }
    ordinary = {
        k: dict(r)
        for k, r in wanted.items()
        if r["class_role"] == "format" and len(cached.get(k, [])) < counts[k]["budget"]
    }
    if config.get("github_hints"):
        from .format_discovery import extension_hints

        hint_path = Path(config["github_hints"])
        hints = extension_hints(
            json.loads(hint_path.read_text()).get("github_extensions", {}), set(rows)
        )
        for kind, row in ordinary.items():
            row["extensions"] = sorted(set(row.get("extensions", [])) | set(hints.get(kind, [])))
        inputs.append(hint_path)
    queries = defaultdict(list)
    for name in config["vt_recipes"]:
        path = Path(name)
        recipe = json.loads(path.read_text())
        for query in recipe.get("queries", []):
            kind, text = query["name"], query["query"]
            # Existing recipes only. Do not add a date cutoff to discovery.
            import re

            if re.search(r"\b(?:fs|ls|first_submission|last_submission):", text):
                raise ValueError(f"Date-filtered query is not allowed: {path}:{kind}")
            text = re.sub(r"\bsize:\d+(?:KB|MB|GB)-", "size:1MB-", text, flags=re.I)
            if kind in ordinary and text not in queries[kind]:
                queries[kind].append(text)
        inputs.append(path)
    metadata = {
        r["repository"]: r
        for r in entries
        if r.get("eligible") is True
        and r["status"] == "inventoried"
        and r.get("spdx_id") in allowed
    }
    by_repo = {r["repository"]: r for r in pool["repositories"]}
    by_lower = {name.lower(): name for name in by_repo}
    by_sha = {r["sha256"]: r for r in records}
    for kind, shas in cached.items():
        for sha in shas:
            for repo in github_repositories(by_sha[sha]):
                name = by_lower.get(repo)
                if name:
                    charge(by_repo[name], rows[kind], 1)
    if rebalance:
        for record in records:
            if record["sha256"] in selected:
                continue
            for repo in github_repositories(record):
                name = by_lower.get(repo)
                if name:
                    by_repo[name]["counts"]["total"] += 1
    match = path_matcher(ordinary)
    options = defaultdict(lambda: defaultdict(list))
    matches = Counter()
    progress("Sampling existing repository file inventory offline")
    for name, file in files:
        if (
            name not in metadata
            or file["regular_file"] is not True
            or file["type"] != "blob"
            or file["mode"] not in {"100644", "100755"}
            or not 0 < file["size"] <= 262144
            or file["git_oid"] in known_oid
        ):
            continue
        for kind in match(file["path"]):
            matches[kind] += 1
            capacity = max(0, 5 - by_repo[name]["counts"]["classes"].get(kind, 0))
            if not capacity:
                continue
            # Keep only the deterministic random reservoir needed from each repo.
            rank = hashlib.sha256(
                f"{config['seed']}:{kind}:{name}:{file['path']}".encode()
            ).hexdigest()
            bucket = options[kind][name]
            bucket.append((rank, file))
            bucket.sort(key=lambda x: (x[0], x[1]["path"]))
            del bucket[capacity:]
    jobs = []
    for kind, row in ordinary.items():
        fixtures, used = [], set()
        eligible = [by_repo[n] for n in options[kind]]
        ordered = draw_repositories(
            eligible, row, seed=config["seed"], limit=len(eligible), unused_first=rebalance
        )
        remaining = counts[kind]["budget"] - len(cached.get(kind, []))
        while len(fixtures) < remaining and ordered:
            next_round = []
            for repo in ordered:
                name = repo["repository"]
                bucket = options[kind][name]
                while bucket and bucket[0][1]["git_oid"] in used:
                    bucket.pop(0)
                if not bucket:
                    continue
                _, file = bucket.pop(0)
                meta = metadata[name]
                fixture = {
                    "provider": "github",
                    "source": "github:" + name,
                    "revision": meta["revision"],
                    "path": file["path"],
                    "git_oid": file["git_oid"],
                    "git_hash_algorithm": file.get("git_hash_algorithm", "sha1"),
                    "permalink": f"https://github.com/{name}/blob/{meta['revision']}/{quote(file['path'], safe='/')}",
                    "size": file["size"],
                    "source_group": f"repository:https://github.com/{name}.git",
                    "cohort": "unattended_inventory",
                    "label_status": "unreviewed",
                    "producer": None,
                    "discovery_format": kind,
                    "claim": {
                        "seed": config["seed"],
                        "basis": "Inventory filename match; not a label",
                        "repository_license": {
                            "spdx_id": meta["spdx_id"],
                            "permalink": meta["license_permalink"],
                            "git_oid": meta["license_git_oid"],
                        },
                    },
                }
                validate_github_fixture(fixture)
                fixtures.append(fixture)
                used.add(file["git_oid"])
                charge(repo, row, 1)
                if bucket:
                    next_round.append(repo)
                if len(fixtures) == remaining:
                    break
            ordered = next_round
        preferred = "github" if rebalance else preferred_provider(row)
        providers = (
            ("github",)
            if rebalance
            else (preferred, "github" if preferred == "virustotal" else "virustotal")
        )
        for provider in providers:
            jobs.append(
                {
                    "id": kind + ":" + provider,
                    "format_id": kind,
                    "provider": provider,
                    "preferred": provider == preferred,
                    "fixtures": fixtures if provider == "github" else [],
                    "queries": queries[kind] if provider == "virustotal" else [],
                    "matching_inventory_files": matches[kind] if provider == "github" else None,
                }
            )
    return {
        "schema_version": 1,
        "classes": counts,
        "jobs": jobs,
        "cached": cached,
        "known_sha256": sorted(known_sha),
        "selected_total": sum(len(r["samples"]) for r in rows.values()),
        "input_sha256": {str(p): digest(p) for p in [*inputs, policy]},
        "target_per_class": target,
        "rebalance": rebalance,
        "repository_pool": {
            "total": len(entries),
            "eligible_indexed": len(metadata),
            "statuses": dict(Counter(r["status"] for r in entries)),
        },
        "seed": config["seed"],
        "scope": "Collection candidates only; final labels/counts unchanged",
    }
