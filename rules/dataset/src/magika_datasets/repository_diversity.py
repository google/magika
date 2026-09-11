# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Repository attribution, independent of local paths and source recipe aliases."""

from urllib.parse import urlsplit


def github_repositories(sample):
    repositories = set()
    for origin in sample.get("origins", []):
        if isinstance(origin, dict):
            url = origin.get("repo_url") or origin.get("permalink", "")
        elif isinstance(origin, str) and origin.startswith("github:"):
            url = origin.removeprefix("github:")
        else:
            continue
        parsed = urlsplit(url)
        parts = parsed.path.strip("/").split("/")
        if parsed.scheme != "https" or parsed.hostname != "github.com" or len(parts) < 2:
            continue
        owner, repo = parts[:2]
        if owner and repo:
            repositories.add(f"{owner.lower()}/{repo.lower().removesuffix('.git')}")
    return repositories
