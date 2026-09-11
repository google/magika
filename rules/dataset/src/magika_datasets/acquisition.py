# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Resumable acquisition from pinned Git blobs and VirusTotal SHA-256 identities."""

import fcntl
import hashlib
import json
import os
import re
import sqlite3
import subprocess
import tempfile
import uuid
from contextlib import ExitStack, contextmanager
from datetime import datetime, timezone
from pathlib import Path

from .git_io import git


def now():
    return datetime.now(timezone.utc).isoformat()


def sha256_file(path):
    with path.open("rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def object_path(root, digest):
    if not re.fullmatch(r"[a-f0-9]{64}", digest):
        raise ValueError("invalid content hash")
    return root / "objects" / digest[:2] / digest


class GitReader:
    """One batch process per source; validate both Git identity and stored bytes."""

    def __init__(self, repo):
        self.repo = repo
        self.trees = {}
        self.closed = False
        self.process = subprocess.Popen(
            ["git", "-C", str(repo), "cat-file", "--batch"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )

    def read_into(self, fixture, output):
        oid, algorithm = fixture["git_oid"], fixture["git_hash_algorithm"]
        if algorithm not in {"sha1", "sha256"} or not re.fullmatch(
            r"[a-f0-9]{40}|[a-f0-9]{64}", oid
        ):
            raise ValueError("invalid Git object identifier")
        revision = fixture["revision"]
        if not re.fullmatch(r"[a-f0-9]{40}|[a-f0-9]{64}", revision):
            raise ValueError("a pinned Git commit is required")
        if revision not in self.trees:
            entries = {}
            for raw in git(self.repo, "ls-tree", "-rlz", revision).split(b"\0"):
                if raw:
                    metadata, path = raw.split(b"\t", 1)
                    entries[path.decode("utf-8", errors="surrogateescape")] = metadata.split()
            self.trees[revision] = entries
        entry = self.trees[revision].get(fixture["path"])
        if (
            not entry
            or entry[0] not in {b"100644", b"100755"}
            or entry[1:] != [b"blob", oid.encode(), str(fixture["size"]).encode()]
        ):
            raise ValueError("fixture path/object/size does not match its pinned Git tree")
        self.process.stdin.write(oid.encode() + b"\n")
        self.process.stdin.flush()
        header = self.process.stdout.readline().split()
        if header != [oid.encode(), b"blob", str(fixture["size"]).encode()]:
            raise ValueError("Git blob metadata differs from fixture inventory")
        remaining = fixture["size"]
        content_hash, git_hash = hashlib.sha256(), hashlib.new(algorithm)
        git_hash.update(f"blob {remaining}\0".encode())
        prefix = b""
        while remaining:
            chunk = self.process.stdout.read(min(1024 * 1024, remaining))
            if not chunk:
                raise ValueError("truncated Git object stream")
            if len(prefix) < 128:
                prefix += chunk[: 128 - len(prefix)]
            output.write(chunk)
            content_hash.update(chunk)
            git_hash.update(chunk)
            remaining -= len(chunk)
        if self.process.stdout.read(1) != b"\n" or git_hash.hexdigest() != oid:
            raise ValueError("Git blob hash/stream verification failed")
        return content_hash.hexdigest(), prefix.startswith(
            b"version https://git-lfs.github.com/spec/v1\n"
        )

    def close(self):
        if self.closed:
            return
        self.closed = True
        self.process.stdin.close()
        self.process.stdout.close()
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.terminate()
            self.process.wait(timeout=5)


@contextmanager
def ledger(root: Path):
    root.mkdir(parents=True, exist_ok=True, mode=0o700)
    with (root / "writer.lock").open("a") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise ValueError("another acquisition writer is using this store") from None
        db = sqlite3.connect(root / "acquisition.sqlite")
        db.row_factory = sqlite3.Row
        try:
            db.executescript("""
                PRAGMA foreign_keys=ON;
                CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
                INSERT OR IGNORE INTO settings VALUES ('schema_version', '1');
                CREATE TABLE IF NOT EXISTS objects(
                    sha256 TEXT PRIMARY KEY, size INTEGER NOT NULL, created_at TEXT NOT NULL);
                CREATE TABLE IF NOT EXISTS origins(
                    source TEXT NOT NULL, revision TEXT NOT NULL, path TEXT NOT NULL,
                    sha256 TEXT REFERENCES objects(sha256), status TEXT NOT NULL,
                    error TEXT, fixture_json TEXT NOT NULL, updated_at TEXT NOT NULL,
                    PRIMARY KEY(source, revision, path));
                CREATE TABLE IF NOT EXISTS runs(
                    id TEXT PRIMARY KEY, started_at TEXT NOT NULL, finished_at TEXT,
                    status TEXT NOT NULL, report_json TEXT);
                CREATE INDEX IF NOT EXISTS origins_by_hash ON origins(sha256, status);
            """)
            if (
                db.execute("SELECT value FROM settings WHERE key='schema_version'").fetchone()[0]
                != "1"
            ):
                raise ValueError("unsupported acquisition ledger version")
            yield db
        finally:
            db.close()


def record_origin(db, fixture, status, digest=None, error=None):
    db.execute(
        """INSERT INTO origins VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(source, revision, path) DO UPDATE SET sha256=excluded.sha256,
        status=excluded.status, error=excluded.error, fixture_json=excluded.fixture_json,
        updated_at=excluded.updated_at""",
        (
            fixture["source"],
            fixture["revision"],
            fixture["path"],
            digest,
            status,
            error,
            json.dumps(fixture, sort_keys=True),
            now(),
        ),
    )


def collect(
    inventory: dict,
    root: Path,
    *,
    max_file_bytes=1024**2,
    max_bytes=1024**3,
    max_files=10000,
    vt_client=None,
    workers=1,
):
    if type(workers) is not int or not 1 <= workers <= 16:
        raise ValueError("acquisition requires 1–16 workers")
    if workers > 1 or any(f.get("provider") == "https" for f in inventory.get("fixtures", [])):
        from .parallel_acquisition import collect_parallel

        return collect_parallel(
            inventory,
            root,
            max_file_bytes=max_file_bytes,
            max_bytes=max_bytes,
            max_files=max_files,
            vt_client=vt_client,
            workers=workers,
        )
    if inventory.get("schema_version") != 1:
        raise ValueError("unsupported fixture inventory")
    if min(max_file_bytes, max_bytes, max_files) < 1:
        raise ValueError("acquisition limits must be positive")
    for fixture in inventory["fixtures"]:
        if type(fixture.get("size")) is not int or fixture["size"] < 0:
            raise ValueError("fixture sizes must be nonnegative integers")
        if fixture.get("provider", "git") not in {"git", "virustotal", "github"}:
            raise ValueError("unsupported acquisition provider")
        if fixture.get("provider") == "github":
            from .manifest import github_blob_matches, validate_github_fixture

            validate_github_fixture(fixture)
        if fixture.get("provider", "git") != "git" and (
            fixture.get("provider") != "github" or "sha256" in fixture
        ):
            object_path(root, fixture.get("sha256", ""))
    report = {
        "run_id": uuid.uuid4().hex,
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
    with ledger(root) as db, ExitStack() as stack:
        # A previous process can leave a running row; recovery never trusts it
        # as proof of live work. The writer lock proves exclusive access now.
        db.execute(
            "UPDATE runs SET status='interrupted', finished_at=? WHERE status='running'", (now(),)
        )
        db.execute(
            "INSERT INTO runs VALUES (?, ?, NULL, 'running', NULL)", (report["run_id"], now())
        )
        db.commit()
        readers = {}
        temporary_dir = root / "temporary"
        temporary_dir.mkdir(exist_ok=True)
        for index, fixture in enumerate(inventory["fixtures"]):
            provider = fixture.get("provider", "git")
            identity = (fixture["source"], fixture["revision"], fixture["path"])
            previous = db.execute(
                "SELECT * FROM origins WHERE source=? AND revision=? AND path=?", identity
            ).fetchone()
            if previous and previous["status"] == "acquired":
                path = object_path(root, previous["sha256"])
                old_fixture = json.loads(previous["fixture_json"])
                content_key = (
                    "git_oid" if provider == "git" or "sha256" not in fixture else "sha256"
                )
                if (
                    old_fixture.get(content_key) == fixture[content_key]
                    and path.is_file()
                    and path.stat().st_size == fixture["size"]
                    and sha256_file(path) == previous["sha256"]
                    and (provider != "github" or github_blob_matches(path, fixture))
                ):
                    report["resumed_origins"] += 1
                    with db:
                        record_origin(db, fixture, "acquired", previous["sha256"])
                    continue
            if provider != "git" and "sha256" in fixture:
                # Avoid spending a download quota on bytes already acquired
                # through GitHub or another VT query; retain the new origin.
                path = object_path(root, fixture["sha256"])
                if (
                    path.is_file()
                    and path.stat().st_size == fixture["size"]
                    and sha256_file(path) == fixture["sha256"]
                    and (provider != "github" or github_blob_matches(path, fixture))
                ):
                    with db:
                        db.execute(
                            "INSERT OR IGNORE INTO objects VALUES (?, ?, ?)",
                            (fixture["sha256"], fixture["size"], now()),
                        )
                        record_origin(db, fixture, "acquired", fixture["sha256"])
                    report["resumed_origins"] += 1
                    continue
            reason = None
            if fixture["size"] > max_file_bytes:
                reason = "per-file byte limit"
            elif report["reserved_bytes"] + fixture["size"] > max_bytes:
                reason = "run byte limit"
            elif report["attempted_origins"] >= max_files:
                reason = "run file limit"
            if reason:
                with db:
                    record_origin(db, fixture, "deferred", error=reason)
                report["deferred"].append(
                    {"source": identity[0], "path": identity[2], "reason": reason}
                )
                continue
            temp = None
            report["reserved_bytes"] += fixture["size"]
            report["attempted_origins"] += 1
            reader_key = (provider, fixture.get("repo"))
            try:
                if reader_key not in readers:
                    if provider == "virustotal":
                        from .virustotal import Client

                        readers[reader_key] = vt_client if vt_client is not None else Client()
                    elif provider == "github":
                        from .manifest import GitHubReader

                        readers[reader_key] = GitHubReader()
                    else:
                        readers[reader_key] = GitReader(fixture["repo"])
                    stack.callback(readers[reader_key].close)
                with tempfile.NamedTemporaryFile(dir=temporary_dir, delete=False) as handle:
                    temp = Path(handle.name)
                    digest, is_lfs = readers[reader_key].read_into(fixture, handle)
                    handle.flush()
                    os.fsync(handle.fileno())
                report["read_bytes"] += fixture["size"]
                if is_lfs:
                    with db:
                        record_origin(
                            db, fixture, "lfs_pointer", error="actual LFS object not acquired"
                        )
                    report["lfs_pointers"].append({"source": identity[0], "path": identity[2]})
                    continue
                dest = object_path(root, digest)
                already_present = dest.is_file() and sha256_file(dest) == digest
                if not already_present:
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    os.replace(temp, dest)
                    report["new_objects"] += 1
                    report["new_bytes"] += fixture["size"]
                with db:
                    db.execute(
                        "INSERT OR IGNORE INTO objects VALUES (?, ?, ?)",
                        (digest, fixture["size"], now()),
                    )
                    record_origin(db, fixture, "acquired", digest)
                report["acquired_origins"] += 1
            except (OSError, ValueError) as exc:
                # Reset a failed stream before the next request so unread bytes
                # from one object cannot corrupt the following object's header.
                if reader_key in readers:
                    reader = readers.pop(reader_key)
                    reader.close()
                error = str(exc)[:400]
                with db:
                    record_origin(db, fixture, "error", error=error)
                report["failures"].append(
                    {"source": identity[0], "path": identity[2], "error": error}
                )
                if getattr(exc, "stop_acquisition", False):
                    report["halted"] = error
                    report["remaining_origins"] = len(inventory["fixtures"]) - index - 1
                    break
            finally:
                if temp is not None:
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
    return report


def acquired_records(root: Path):
    """Read a committed inventory snapshot without taking the writer lock."""
    database = (root / "acquisition.sqlite").resolve()
    if not database.exists():
        return
    db = sqlite3.connect(database.as_uri() + "?mode=ro", uri=True)
    try:
        version = db.execute("SELECT value FROM settings WHERE key='schema_version'").fetchone()
        if version is None or version[0] != "1":
            raise ValueError("unsupported acquisition ledger version")
        # A single query keeps objects and origins in one committed snapshot.
        # Materialize and close before yielding so a slow consumer cannot hold
        # a SQLite read lock while the downloader tries to commit another file.
        rows = db.execute("""
            SELECT objects.sha256, objects.size, origins.fixture_json
            FROM objects JOIN origins ON origins.sha256 = objects.sha256
            WHERE origins.status = 'acquired'
            ORDER BY objects.sha256, origins.source, origins.revision, origins.path
        """).fetchall()
    finally:
        db.close()
    record = None
    for sha, size, fixture in rows:
        if record is None or record["sha256"] != sha:
            if record is not None:
                yield record
            record = {
                "sha256": sha,
                "size": size,
                "path": str(object_path(root, sha)),
                "origins": [],
                "label_status": "unreviewed",
            }
        record["origins"].append(json.loads(fixture))
    if record is not None:
        yield record
