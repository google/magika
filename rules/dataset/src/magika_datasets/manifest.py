# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Public hash/origin lists; sample bytes remain in the local acquisition store."""

import hashlib
import re
from urllib.parse import quote, unquote, urlsplit
from urllib.request import Request, build_opener

from .http_source import validate_url
from .virustotal import NoRedirect


def github_parts(url):
    parsed = urlsplit(url)
    match = re.fullmatch(
        r"/([A-Za-z0-9_.-]+)/([A-Za-z0-9_.-]+)/blob/([a-f0-9]{40}|[a-f0-9]{64})/(.+)", parsed.path
    )
    if (
        parsed.scheme != "https"
        or parsed.netloc != "github.com"
        or parsed.query
        or parsed.fragment
        or not match
    ):
        raise ValueError("GitHub origins require a permanent HTTPS blob URL with a full commit")
    owner, repo, revision, path = match.groups()
    if any(p in {"", ".", ".."} for p in unquote(path).split("/")):
        raise ValueError("invalid GitHub file path")
    return owner, repo, revision, path


def validate_record(record):
    sha = record.get("sha256")
    if not isinstance(sha, str) or not re.fullmatch(r"[a-f0-9]{64}", sha):
        raise ValueError("manifest requires a SHA-256")
    if type(record.get("size")) is not int or record["size"] < 0:
        raise ValueError("manifest requires a nonnegative size")
    if not isinstance(record.get("origins"), list) or not record["origins"]:
        raise ValueError("manifest requires at least one origin")
    for origin in record["origins"]:
        if origin == "vt:" + sha:
            continue
        if isinstance(origin, str) and origin.startswith("https:"):
            url, digest = origin.rsplit(":", 1)
            validate_url(url)
            if digest != sha:
                raise ValueError("origin hash does not match record")
            continue
        if not isinstance(origin, str) or not origin.startswith("github:"):
            raise ValueError("unsupported manifest origin")
        url, digest = origin[7:].rsplit(":", 1)
        if digest != sha:
            raise ValueError("origin hash does not match record")
        github_parts(url)
    return record


def export_records(samples, selection=None):
    chosen = set(selection["selected"]) if selection is not None else None
    rows = []
    for sample in samples:
        sha = sample["sha256"]
        if chosen is not None and sha not in chosen:
            continue
        origins = set()
        for origin in sample["origins"]:
            provider = origin.get("provider", "git")
            if provider == "virustotal":
                origins.add("vt:" + sha)
            elif provider == "github":
                origins.add("github:" + origin["permalink"] + ":" + sha)
            elif provider == "git":
                repo = origin["repo_url"].removesuffix(".git").rstrip("/")
                url = repo + "/blob/" + origin["revision"] + "/" + quote(origin["path"], safe="/")
                origins.add("github:" + url + ":" + sha)
        rows.append(
            validate_record(
                {
                    "sha256": sha,
                    "size": sample["size"],
                    "origins": sorted(origins),
                    "format_ids": sample.get("format_ids", []),
                    "label_status": sample.get("label_status", "unreviewed"),
                }
            )
        )
    if len({r["sha256"] for r in rows}) != len(rows):
        raise ValueError("duplicate manifest sample identity")
    if chosen is not None and {r["sha256"] for r in rows} != chosen:
        raise ValueError("selection contains missing samples")
    return sorted(rows, key=lambda x: x["sha256"])


def inventory(rows, *, prefer="github"):
    fixtures, seen = [], set()
    for row in rows:
        validate_record(row)
        sha = row["sha256"]
        if sha in seen:
            raise ValueError("duplicate manifest sample identity")
        seen.add(sha)
        origin = min(row["origins"], key=lambda x: (not x.startswith(prefer + ":"), x))
        common = {
            "sha256": sha,
            "size": row["size"],
            "producer": None,
            "label_status": "unreviewed",
            "cohort": "public_manifest",
            "claim": {
                "basis": "public manifest; revalidate downloaded bytes",
                "format_ids": row.get("format_ids", []),
            },
        }
        # Identification claims survive replay, but never confer acceptance.
        if isinstance(row.get("vt_markings"), dict):
            common["claim"]["vt_markings"] = row["vt_markings"]
        if isinstance(row.get("discovery_queries"), list):
            common["claim"]["discovery_queries"] = row["discovery_queries"]
        if origin.startswith("https:"):
            url = origin.rsplit(":", 1)[0]
            fixtures.append(
                {
                    **common,
                    "provider": "https",
                    "url": url,
                    "source": "https:" + urlsplit(url).netloc,
                    "revision": sha,
                    "path": url,
                    "source_group": "external:" + urlsplit(url).netloc,
                }
            )
        elif origin.startswith("vt:"):
            fixtures.append(
                {
                    **common,
                    "provider": "virustotal",
                    "source": "virustotal",
                    "revision": "manifest-v1",
                    "path": sha,
                    "source_group": "collection:virustotal",
                }
            )
        else:
            url = origin[7:].rsplit(":", 1)[0]
            owner, repo, revision, path = github_parts(url)
            fixtures.append(
                {
                    **common,
                    "provider": "github",
                    "source": f"github:{owner}/{repo}",
                    "revision": revision,
                    "path": unquote(path),
                    "permalink": url,
                    "source_group": f"repository:https://github.com/{owner}/{repo}.git",
                }
            )
    return {"schema_version": 1, "fixtures": fixtures}


def validate_github_fixture(fixture):
    """Require an immutable URL and either a known SHA-256 or a Git blob identity."""
    _, _, revision, path = github_parts(fixture["permalink"])
    if fixture.get("revision") != revision or fixture.get("path") != unquote(path):
        raise ValueError("GitHub fixture path/revision differs from its permalink")
    sha = fixture.get("sha256")
    if "sha256" in fixture and (not isinstance(sha, str) or not re.fullmatch(r"[a-f0-9]{64}", sha)):
        raise ValueError("invalid GitHub SHA-256")
    oid = fixture.get("git_oid")
    if "git_oid" in fixture:
        length = {"sha1": 40, "sha256": 64}.get(fixture.get("git_hash_algorithm"))
        if (
            length is None
            or not isinstance(oid, str)
            or not re.fullmatch(r"[a-f0-9]{%d}" % length, oid)
        ):
            raise ValueError("invalid GitHub Git blob identity")
    if sha is None and oid is None:
        raise ValueError("GitHub fixture requires a SHA-256 or Git blob identity")


def github_blob_matches(path, fixture):
    """A cached SHA-256 object must also satisfy any supplied Git identity."""
    if "git_oid" not in fixture:
        return True
    digest = hashlib.new(fixture["git_hash_algorithm"])
    digest.update(f"blob {fixture['size']}\0".encode())
    with path.open("rb") as handle:
        while chunk := handle.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest() == fixture["git_oid"]


class GitHubReader:
    def read_into(self, fixture, output):
        validate_github_fixture(fixture)
        owner, repo, revision, path = github_parts(fixture["permalink"])
        url = f"https://raw.githubusercontent.com/{owner}/{repo}/{revision}/{quote(unquote(path), safe='/')}"
        remaining, digest, prefix = fixture["size"], hashlib.sha256(), b""
        git_hash = None
        if "git_oid" in fixture:
            git_hash = hashlib.new(fixture["git_hash_algorithm"])
            git_hash.update(f"blob {remaining}\0".encode())
        with build_opener(NoRedirect()).open(Request(url), timeout=30) as response:
            while remaining:
                chunk = response.read(min(1024 * 1024, remaining))
                if not chunk:
                    raise ValueError("GitHub download truncated")
                if len(prefix) < 128:
                    prefix += chunk[: 128 - len(prefix)]
                remaining -= len(chunk)
                digest.update(chunk)
                if git_hash is not None:
                    git_hash.update(chunk)
                output.write(chunk)
            if response.read(1):
                raise ValueError("GitHub download exceeds manifest size")
        if "sha256" in fixture and digest.hexdigest() != fixture["sha256"]:
            raise ValueError("GitHub download SHA-256 mismatch")
        if git_hash is not None and git_hash.hexdigest() != fixture["git_oid"]:
            raise ValueError("GitHub download Git blob hash mismatch")
        return digest.hexdigest(), prefix.startswith(
            b"version https://git-lfs.github.com/spec/v1\n"
        )

    def close(self):
        pass
