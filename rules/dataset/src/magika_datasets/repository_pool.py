# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Repository seed lists and reproducible, usage-weighted random draws."""

import math
import random


def draw_repositories(pool, row, *, seed, limit=20, languages=(), unused_first=False):
    """Weighted draw without replacement; counts lower probability, never exclude."""
    wanted = {s.casefold() for s in languages}
    rng = random.Random(f"{seed}:{row['format_id']}")
    ranked = []
    for repo in sorted(pool, key=lambda r: r["repository"]):
        if wanted and not wanted.intersection(s.casefold() for s in repo["languages"]):
            continue
        counts = repo["counts"]
        by_class = counts["classes"].get(row["format_id"], 0)
        by_category = sum(counts["categories"].get(c, 0) for c in set(row.get("categories", [])))
        # Exponential race: lower scores win, with probability proportional to
        # inverse usage. Stars establish pool membership, not sampling weight.
        penalty = (1 + by_class) * (1 + by_category) * (1 + counts["total"])
        score = -math.log1p(-rng.random()) * penalty
        ranked.append((int(by_class > 0) if unused_first else 0, score, repo["repository"], repo))
    return [repo for _, _, _, repo in sorted(ranked)[:limit]]


def charge(pool_row, row, amount):
    """Count provisional candidates separately in the caller's pass snapshot."""
    counts = pool_row["counts"]
    counts["total"] += amount
    key = row["format_id"]
    counts["classes"][key] = counts["classes"].get(key, 0) + amount
    for category in set(row.get("categories", [])):
        counts["categories"][category] = counts["categories"].get(category, 0) + amount
