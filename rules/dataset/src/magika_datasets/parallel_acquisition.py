# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Concurrent byte readers with one budget/accounting and SQLite writer thread."""

import json
import os
import tempfile
import threading
import uuid
from concurrent.futures import FIRST_COMPLETED, ThreadPoolExecutor, wait
from pathlib import Path

from .acquisition import GitReader, ledger, now, object_path, record_origin, sha256_file
from .manifest import GitHubReader, github_blob_matches, validate_github_fixture
from .virustotal import Client


def collect_parallel(inventory, root, *, max_file_bytes, max_bytes, max_files, vt_client, workers):
    if inventory.get("schema_version") != 1:
        raise ValueError("unsupported fixture inventory")
    if min(max_file_bytes, max_bytes, max_files) < 1:
        raise ValueError("acquisition limits must be positive")
    groups, by_hash = [], {}
    for fixture in inventory["fixtures"]:
        provider = fixture.get("provider", "git")
        if type(fixture.get("size")) is not int or fixture["size"] < 0:
            raise ValueError("fixture sizes must be nonnegative integers")
        if provider not in {"git", "github", "virustotal"}:
            raise ValueError("unsupported acquisition provider")
        if provider == "github":
            validate_github_fixture(fixture)
        if provider == "git" or provider == "github" and "sha256" not in fixture:
            groups.append([fixture])
        else:
            sha = fixture.get("sha256", "")
            object_path(root, sha)
            if sha not in by_hash:
                by_hash[sha] = []
                groups.append(by_hash[sha])
            if by_hash[sha] and by_hash[sha][0]["size"] != fixture["size"]:
                raise ValueError("conflicting sizes for one SHA-256")
            by_hash[sha].append(fixture)
    report = {
        "run_id": uuid.uuid4().hex,
        "workers": workers,
        "acquired_origins": 0,
        "resumed_origins": 0,
        "new_objects": 0,
        "read_bytes": 0,
        "reserved_bytes": 0,
        "attempted_origins": 0,
        "new_bytes": 0,
        "failures": [],
        "deferred": [],
        "lfs_pointers": [],
    }
    readers, reader_lock = {}, threading.Lock()
    temporary_dir = root / "temporary"

    def fetch(fixture):
        provider = fixture.get("provider", "git")
        key = (threading.get_ident(), provider, fixture.get("repo"))
        temp = None
        try:
            with reader_lock:
                if key not in readers:
                    readers[key] = (
                        (vt_client if vt_client is not None else Client())
                        if provider == "virustotal"
                        else (
                            GitHubReader() if provider == "github" else GitReader(fixture["repo"])
                        )
                    )
                reader = readers[key]
            with tempfile.NamedTemporaryFile(dir=temporary_dir, delete=False) as handle:
                temp = Path(handle.name)
                digest, is_lfs = reader.read_into(fixture, handle)
                handle.flush()
                os.fsync(handle.fileno())
            if (
                temp.stat().st_size != fixture["size"]
                or sha256_file(temp) != digest
                or provider != "git"
                and "sha256" in fixture
                and digest != fixture["sha256"]
            ):
                raise ValueError("downloaded bytes do not match fixture identity")
            return {"temp": temp, "sha256": digest, "is_lfs": is_lfs}
        except (OSError, ValueError) as exc:
            if temp:
                temp.unlink(missing_ok=True)
            with reader_lock:
                failed = readers.pop(key, None)
            if failed:
                failed.close()
            status = getattr(exc, "http_status", None) or getattr(exc, "code", None)
            headers = getattr(exc, "headers", None)
            retry = getattr(exc, "retry_after", None) or (
                headers.get("Retry-After") if headers else None
            )
            if hasattr(exc, "close"):
                exc.close()
            return {
                "error": str(exc)[:400],
                "http_status": status,
                "retry_after": retry,
                "halt": bool(getattr(exc, "stop_acquisition", False)) or status in {401, 403, 429},
            }

    with ledger(root) as db:
        db.execute(
            "UPDATE runs SET status='interrupted', finished_at=? WHERE status='running'", (now(),)
        )
        db.execute(
            "INSERT INTO runs VALUES (?, ?, NULL, 'running', NULL)", (report["run_id"], now())
        )
        db.commit()
        temporary_dir.mkdir(exist_ok=True)
        pending, next_group, halted_providers = {}, 0, {}
        pool = ThreadPoolExecutor(max_workers=workers)
        try:
            while pending or next_group < len(groups):
                while len(pending) < workers and next_group < len(groups):
                    group = groups[next_group]
                    next_group += 1
                    fixture = group[0]
                    provider = fixture.get("provider", "git")
                    digest = fixture.get("sha256")
                    if provider == "git" or provider == "github" and "sha256" not in fixture:
                        previous = db.execute(
                            "SELECT * FROM origins WHERE source=? AND revision=? AND path=?",
                            (fixture["source"], fixture["revision"], fixture["path"]),
                        ).fetchone()
                        if (
                            previous
                            and previous["status"] == "acquired"
                            and json.loads(previous["fixture_json"]).get("git_oid")
                            == fixture["git_oid"]
                        ):
                            digest = previous["sha256"]
                    if digest:
                        path = object_path(root, digest)
                        if (
                            path.is_file()
                            and path.stat().st_size == fixture["size"]
                            and sha256_file(path) == digest
                            and (provider != "github" or github_blob_matches(path, fixture))
                        ):
                            with db:
                                db.execute(
                                    "INSERT OR IGNORE INTO objects VALUES (?, ?, ?)",
                                    (digest, fixture["size"], now()),
                                )
                                for origin in group:
                                    record_origin(db, origin, "acquired", digest)
                            report["resumed_origins"] += len(group)
                            continue
                    reason = (
                        halted_providers[provider]
                        if provider in halted_providers
                        else "per-file byte limit"
                        if fixture["size"] > max_file_bytes
                        else "run byte limit"
                        if report["reserved_bytes"] + fixture["size"] > max_bytes
                        else "run file limit"
                        if report["attempted_origins"] >= max_files
                        else None
                    )
                    if reason:
                        with db:
                            for origin in group:
                                record_origin(db, origin, "deferred", error=reason)
                                report["deferred"].append(
                                    {
                                        "source": origin["source"],
                                        "path": origin["path"],
                                        "reason": reason,
                                    }
                                )
                        continue
                    report["reserved_bytes"] += fixture["size"]
                    report["attempted_origins"] += 1
                    pending[pool.submit(fetch, fixture)] = group
                if not pending:
                    break
                finished, _ = wait(pending, return_when=FIRST_COMPLETED)
                for future in finished:
                    group = pending.pop(future)
                    result = future.result()
                    if "error" in result:
                        with db:
                            for origin in group:
                                record_origin(db, origin, "error", error=result["error"])
                                report["failures"].append(
                                    {
                                        "source": origin["source"],
                                        "path": origin["path"],
                                        "error": result["error"],
                                        "http_status": result.get("http_status"),
                                        "retry_after": result.get("retry_after"),
                                    }
                                )
                        if result["halt"]:
                            report["halted"] = result["error"]
                            halted_providers[group[0].get("provider", "git")] = result["error"]
                            report["halted_providers"] = dict(halted_providers)
                        continue
                    temp, digest = result["temp"], result["sha256"]
                    size = group[0]["size"]
                    report["read_bytes"] += size
                    try:
                        if result["is_lfs"]:
                            with db:
                                for origin in group:
                                    record_origin(
                                        db,
                                        origin,
                                        "lfs_pointer",
                                        error="actual LFS object not acquired",
                                    )
                                    report["lfs_pointers"].append(
                                        {"source": origin["source"], "path": origin["path"]}
                                    )
                            continue
                        dest = object_path(root, digest)
                        if not dest.is_file() or sha256_file(dest) != digest:
                            dest.parent.mkdir(parents=True, exist_ok=True)
                            os.replace(temp, dest)
                            report["new_objects"] += 1
                            report["new_bytes"] += size
                        with db:
                            db.execute(
                                "INSERT OR IGNORE INTO objects VALUES (?, ?, ?)",
                                (digest, size, now()),
                            )
                            for origin in group:
                                record_origin(db, origin, "acquired", digest)
                        report["acquired_origins"] += len(group)
                    finally:
                        temp.unlink(missing_ok=True)
            report["store"] = dict(
                db.execute(
                    "SELECT COUNT(*) AS objects, COALESCE(SUM(size),0) AS bytes FROM objects"
                ).fetchone()
            )
            with db:
                db.execute(
                    "UPDATE runs SET finished_at=?, status='finished', report_json=? WHERE id=?",
                    (now(), json.dumps(report), report["run_id"]),
                )
        finally:
            pool.shutdown(wait=True, cancel_futures=True)
            for future in pending:
                if not future.cancelled() and future.exception() is None:
                    temp = future.result().get("temp")
                    if temp:
                        temp.unlink(missing_ok=True)
            for reader in readers.values():
                reader.close()
    return report
