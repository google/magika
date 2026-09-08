# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Unattended acquisition. No detector, labeling, or snapshot code runs here."""

import argparse
import fcntl
import hashlib
import json
import os
import subprocess
import sys
import time
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from email.utils import parsedate_to_datetime
from pathlib import Path

from tenacity import RetryCallState, Retrying, stop_after_attempt, wait_exponential_jitter

from .acquisition import acquired_records, collect
from .runtime import implementation_hashes
from .virustotal import Client

TERMINAL = {"collected", "collected_with_shortfalls", "failed"}
RETRY_POLICY = "tenacity-exponential-v1"
TRANSPORT_RETRY = Retrying(
    wait=wait_exponential_jitter(initial=2, max=300, jitter=1), stop=stop_after_attempt(10)
)
RATE_RETRY = Retrying(
    wait=wait_exponential_jitter(initial=30, max=300, jitter=1), stop=stop_after_attempt(4)
)


def retry_state(attempt, http_status=None):
    policy = RATE_RETRY if http_status == 429 else TRANSPORT_RETRY
    state = RetryCallState(retry_object=policy, fn=None, args=(), kwargs={})
    state.attempt_number = max(1, attempt)
    return state


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    content = json.dumps(value, sort_keys=True) + "\n"
    if path.exists() and path.read_text() == content:
        return
    temporary = path.with_suffix(path.suffix + ".next")
    temporary.write_text(content)
    temporary.replace(path)


def retry_delay(value, now, attempt=1, http_status=None):
    try:
        return max(1, int(value))
    except (ValueError, TypeError):
        try:
            return max(1, parsedate_to_datetime(value).timestamp() - now)
        except (ValueError, TypeError, AttributeError, OverflowError):
            state = retry_state(attempt, http_status)
            return state.retry_object.wait(state)


def failure(error):
    if isinstance(error, dict):
        result = error
    else:
        result = {
            "error": str(error)[:400],
            "http_status": getattr(error, "http_status", None) or getattr(error, "code", None),
            "retry_after": getattr(error, "retry_after", None),
        }
    status, message = result.get("http_status"), result.get("error", "")
    auth = status in {401, 403} or "VT_API_KEY" in message
    transient = status == 429 or (isinstance(status, int) and status >= 500)
    transient = transient or any(
        s in message.lower() for s in ("transport", "timed out", "connection", "stream failed")
    )
    return {**result, "auth": auth, "transient": transient}


def pause_provider(provider, error, now, operation="download"):
    """One increment per failed retry window, not per in-flight request."""
    detail = failure(error)
    if detail["auth"]:
        provider.update(stopped=True, reason="credentials_required", error=detail["error"])
    elif detail["transient"] and not provider.get("stopped") and now >= provider.get("next_at", 0):
        provider["failures"] = provider.get("failures", 0) + 1
        delay = retry_delay(
            detail.get("retry_after"), now, provider["failures"], detail.get("http_status")
        )
        provider.update(
            error=detail["error"],
            operation=operation,
            next_at=now + delay,
            failed_at=now,
            wait_seconds=delay,
            http_status=detail.get("http_status"),
            retry_after=detail.get("retry_after"),
            retry_policy=RETRY_POLICY,
        )
        state = retry_state(provider["failures"], detail.get("http_status"))
        if state.retry_object.stop(state):
            provider.update(stopped=True, reason="retry_windows_exhausted")
    return detail


def upgrade_legacy_cooldown(provider, now):
    """Migrate only a known legacy transport wait; never shorten a server quota wait."""
    if provider.get("retry_policy") or provider.get("stopped"):
        return
    detail = failure({"error": provider.get("error", "")})
    if not detail["transient"] or "HTTP" in detail["error"]:
        return
    failed_at = provider.get("next_at", 0) - 1800
    delay = retry_delay(None, now, max(1, provider.get("failures", 1)))
    provider.update(
        next_at=max(now, failed_at + delay),
        failed_at=failed_at,
        wait_seconds=delay,
        retry_policy=RETRY_POLICY,
    )


def ready(provider, now):
    return not provider.get("stopped") and now >= provider.get("next_at", 0)


def fixture_key(fixture):
    return (
        fixture["discovery_format"],
        fixture.get("sha256") or (fixture["source"], fixture["revision"], fixture["path"]),
    )


class Search:
    def __init__(self, root, plan):
        self.root = root
        self.known = set(plan["known_sha256"])

    def __call__(self, job, state):
        if job["provider"] == "github":
            offset = state.get("offset", 0)
            fixtures = job["fixtures"][offset : offset + 16]
            return fixtures, {
                "offset": offset + len(fixtures),
                "done": offset + len(fixtures) >= len(job["fixtures"]),
            }
        index = state.get("query_index", 0)
        if index >= len(job["queries"]) or state.get("pages", 0) >= 50:
            return [], {"done": True}
        query, cursor = job["queries"][index], state.get("cursor")
        key = hashlib.sha256(json.dumps([query, cursor]).encode()).hexdigest()
        path = self.root / "search" / (key + ".json")
        if path.exists():
            page = read(path)
        else:
            client = Client()
            try:
                page = client.search(query, limit=100, cursor=cursor)
            finally:
                client.close()
            write(path, page)
        fixtures = [
            {**f, "discovery_format": job["format_id"]}
            for f in page["fixtures"]
            if 0 < f["size"] <= 1048576 and f["sha256"] not in self.known
        ]
        following = page["next_cursor"]
        pages = state.get("pages", 0) + 1
        return fixtures, {
            "pages": pages,
            "cursor": following,
            "query_index": index if following else index + 1,
            "done": pages >= 50 or (not following and index + 1 >= len(job["queries"])),
        }


class Runner:
    def __init__(self, root, config, plan, *, search=None, download=None):
        self.root, self.config, self.plan = root, config, plan
        self.search = search or Search(root, plan)
        self.download = download or self.acquire
        self.state = (
            read(root / "state.json")
            if (root / "state.json").exists()
            else {
                "jobs": {},
                "providers": {p: {} for p in ("github", "virustotal")},
                "phase": "collecting",
            }
        )
        for provider in self.state["providers"].values():
            upgrade_legacy_cooldown(provider, time.time())
        self.batches, self.seen, self.reserved = {}, set(), Counter()
        for path in sorted((root / "queue").glob("*.json")):
            batch = read(path)
            self.batches[path.stem] = batch
            for f in batch["fixtures"]:
                self.seen.add(fixture_key(f))
                self.reserved[f["discovery_format"]] += 1
        self.searching = {}
        self.downloading = None
        self.download_turns = self.state.setdefault("download_turns", {})

    def acquire(self, batch):
        cap = 262144 if batch["provider"] == "github" else 1048576
        return collect(
            {"schema_version": 1, "fixtures": batch["fixtures"]},
            Path(self.config["store"]),
            workers=8,
            max_files=16,
            max_file_bytes=cap,
            max_bytes=16 * cap,
        )

    def remaining(self, kind):
        row = self.plan["classes"][kind]
        return max(0, row["budget"] - len(row["cached"]) - self.reserved[kind])

    def enqueue(self, fixtures, provider):
        chosen = []
        for f in fixtures:
            kind, key = f["discovery_format"], fixture_key(f)
            cap = 262144 if provider == "github" else 1048576
            if not 0 < f["size"] <= cap or key in self.seen or not self.remaining(kind):
                continue
            self.seen.add(key)
            self.reserved[kind] += 1
            chosen.append(f)
        for offset in range(0, len(chosen), 16):
            batch = {"provider": provider, "fixtures": chosen[offset : offset + 16]}
            key = hashlib.sha256(json.dumps(batch, sort_keys=True).encode()).hexdigest()
            write(self.root / "queue" / (key + ".json"), batch)
            self.batches[key] = batch

    def pending_batches(self):
        return [
            (key, b)
            for key, b in self.batches.items()
            if not (self.root / "receipts" / (key + ".json")).exists()
        ]

    def save(self):
        write(self.root / "state.json", self.state)
        summaries = {
            k: {
                "selected": r["selected"],
                "candidate_budget": r["budget"],
                "cached": len(r["cached"]),
                "queued_candidates": self.reserved[k],
                "downloaded": 0,
                "failed": 0,
                "role": r["role"],
            }
            for k, r in self.plan["classes"].items()
        }
        total = Counter()
        for path in (self.root / "receipts").glob("*.json"):
            receipt = read(path)
            total.update(
                {k: receipt.get(k, 0) for k in ("new_objects", "new_bytes", "resumed_origins")}
            )
            row = summaries[receipt["format_id"]]
            row["downloaded"] += receipt["acquired"]
            row["failed"] += receipt["failed"]
        for kind, row in summaries.items():
            row["candidates_ready"] = row["cached"] + row["downloaded"]
            row["candidate_shortfall"] = max(0, row["candidate_budget"] - row["candidates_ready"])
        write(self.root / "classes-status.json", summaries)
        status = {
            "schema_version": 1,
            "phase": self.state["phase"],
            "pid": os.getpid(),
            "updated_at": time.time(),
            "baseline_selected": self.plan["selected_total"],
            "unfinished_classes": len(summaries),
            "jobs": len(self.plan["jobs"]),
            "jobs_finished": sum(bool(s.get("done")) for s in self.state["jobs"].values()),
            "cached_candidates": sum(r["cached"] for r in summaries.values()),
            "candidates_ready": sum(r["candidates_ready"] for r in summaries.values()),
            "candidate_budget": sum(r["candidate_budget"] for r in summaries.values()),
            "buffered_classes": sum(r["candidate_shortfall"] == 0 for r in summaries.values()),
            "new_objects": total["new_objects"],
            "new_bytes": total["new_bytes"],
            "failed_candidates": sum(r["failed"] for r in summaries.values()),
            "pending_batches": len(self.pending_batches()),
            "finder_workers": 8,
            "download_workers": 8,
            "providers": self.state["providers"],
            "reconciliation": "pending; accepted dataset unchanged",
        }
        write(self.root / "status.json", status)

    def finish_download(self, key, batch, report):
        provider = self.state["providers"][batch["provider"]]
        details = [failure(f) for f in report["failures"]]
        retry = any(d["transient"] or d["auth"] for d in details)
        if retry:
            for d in details:
                if d["transient"] or d["auth"]:
                    pause_provider(provider, d, time.time())
            # Durable attempt evidence; replay of this same batch reuses completed origins.
            directory = self.root / "attempts" / key
            number = len(list(directory.glob("*.json")))
            write(directory / f"{number:03d}.json", report)
            if not provider.get("stopped"):
                return
        elif ready(provider, time.time()) and provider.get("operation", "download") == "download":
            provider.update(failures=0, next_at=0)
        acquired = report["acquired_origins"] + report["resumed_origins"]
        # Include bytes downloaded in interrupted/retried attempts in aggregate reporting.
        attempts = [read(p) for p in sorted((self.root / "attempts" / key).glob("*.json"))]
        if retry:
            attempts = attempts[:-1] if attempts else []
        receipt = {
            "format_id": batch["fixtures"][0]["discovery_format"],
            "acquired": acquired,
            "failed": len(batch["fixtures"]) - acquired,
            "new_objects": report["new_objects"] + sum(r["new_objects"] for r in attempts),
            "new_bytes": report["new_bytes"] + sum(r["new_bytes"] for r in attempts),
            "resumed_origins": report["resumed_origins"],
            "report": report,
        }
        write(self.root / "receipts" / (key + ".json"), receipt)

    def run(self):
        self.state["phase"] = "collecting"
        last_save = 0
        with (
            ThreadPoolExecutor(max_workers=8) as finders,
            ThreadPoolExecutor(max_workers=1) as writer,
        ):
            while True:
                now = time.time()
                for future, job in list(self.searching.items()):
                    if not future.done():
                        continue
                    del self.searching[future]
                    state = self.state["jobs"].setdefault(job["id"], {})
                    provider = self.state["providers"][job["provider"]]
                    try:
                        fixtures, update = future.result()
                        self.enqueue(fixtures, job["provider"])
                        state.update(update)
                        if ready(provider, now) and provider.get("operation", "search") == "search":
                            provider.update(failures=0, next_at=0)
                    except (ValueError, OSError) as error:
                        detail = pause_provider(provider, error, now, "search")
                        if not detail["auth"] and not detail["transient"]:
                            state.update(done=True, error=detail["error"])
                if self.downloading and self.downloading[0].done():
                    future, key, batch = self.downloading
                    self.finish_download(key, batch, future.result())
                    self.downloading = None
                active = {j["id"] for j in self.searching.values()}
                candidates = []
                for job in self.plan["jobs"]:
                    state = self.state["jobs"].setdefault(job["id"], {})
                    provider = self.state["providers"][job["provider"]]
                    if job["id"] in active or state.get("done"):
                        continue
                    if not self.remaining(job["format_id"]) or provider.get("stopped"):
                        state.update(
                            done=True, reason=provider.get("reason", "candidate_budget_reached")
                        )
                        continue
                    if not job["preferred"]:
                        preferred = next(
                            j
                            for j in self.plan["jobs"]
                            if j["format_id"] == job["format_id"] and j["preferred"]
                        )
                        preferred_state = self.state["jobs"].get(preferred["id"], {})
                        if not preferred_state.get("done") and ready(
                            self.state["providers"][preferred["provider"]], now
                        ):
                            continue
                    if ready(provider, now):
                        candidates.append(job)
                # Page count/offset rotates all classes through the finder window.
                candidates.sort(
                    key=lambda j: (self.state["jobs"][j["id"]].get("turns", 0), j["id"])
                )
                for job in candidates[: 8 - len(self.searching)]:
                    state = self.state["jobs"][job["id"]]
                    state["turns"] = state.get("turns", 0) + 1
                    self.searching[finders.submit(self.search, job, dict(state))] = job
                if not self.downloading:
                    batches = self.pending_batches()
                    eligible = [
                        (k, b)
                        for k, b in batches
                        if ready(self.state["providers"][b["provider"]], now)
                    ]
                    eligible.sort(
                        key=lambda pair: (
                            self.download_turns.get(pair[1]["fixtures"][0]["discovery_format"], 0),
                            pair[0],
                        )
                    )
                    if eligible:
                        key, batch = eligible[0]
                        self.download_turns[batch["fixtures"][0]["discovery_format"]] = now
                        self.downloading = (writer.submit(self.download, batch), key, batch)
                    for key, batch in batches:
                        p = self.state["providers"][batch["provider"]]
                        if p.get("stopped"):
                            # Never treat a queued object as successfully downloaded.
                            attempts = [
                                read(path)
                                for path in sorted((self.root / "attempts" / key).glob("*.json"))
                            ]
                            last = attempts[-1] if attempts else {}
                            acquired = last.get("acquired_origins", 0) + last.get(
                                "resumed_origins", 0
                            )
                            write(
                                self.root / "receipts" / (key + ".json"),
                                {
                                    "format_id": batch["fixtures"][0]["discovery_format"],
                                    "acquired": acquired,
                                    "failed": len(batch["fixtures"]) - acquired,
                                    "reason": p["reason"],
                                    "new_objects": sum(r["new_objects"] for r in attempts),
                                    "new_bytes": sum(r["new_bytes"] for r in attempts),
                                },
                            )
                done = all(
                    self.state["jobs"].get(j["id"], {}).get("done") for j in self.plan["jobs"]
                )
                if (
                    done
                    and not self.searching
                    and not self.downloading
                    and not self.pending_batches()
                ):
                    self.state["phase"] = "collected_with_shortfalls"
                    self.save()
                    counts = read(self.root / "classes-status.json")
                    if all(r["candidate_shortfall"] == 0 for r in counts.values()):
                        self.state["phase"] = "collected"
                        self.save()
                    self.export_handoff()
                    break
                # State is cheap; receipts and public status are aggregated every 30s.
                write(self.root / "state.json", self.state)
                if now - last_save >= 30:
                    self.save()
                    last_save = now
                time.sleep(1)

    def export_handoff(self):
        """One candidate table for the later reconciliation; no identities accepted."""
        import pyarrow as pa
        import pyarrow.parquet as pq

        from .manifest import export_records

        claims = {}
        for kind, shas in self.plan["cached"].items():
            for sha in shas:
                claims.setdefault(sha, set()).add(kind)
        origin_classes = {}
        for batch in self.batches.values():
            for f in batch["fixtures"]:
                key = (f["source"], f["revision"], f["path"])
                origin_classes.setdefault(key, set()).add(f["discovery_format"])
        rows = []
        for record in acquired_records(Path(self.config["store"])):
            kinds = set(claims.get(record["sha256"], ()))
            for origin in record["origins"]:
                kinds.update(
                    origin_classes.get((origin["source"], origin["revision"], origin["path"]), ())
                )
            if kinds:
                rows.append(
                    {
                        "sha256": bytes.fromhex(record["sha256"]),
                        "size": record["size"],
                        "origins": export_records([record])[0]["origins"],
                        "discovery_classes": sorted(kinds),
                        "label_status": "unreviewed",
                    }
                )
        schema = pa.schema(
            [
                ("sha256", pa.binary(32)),
                ("size", pa.int64()),
                ("origins", pa.list_(pa.string())),
                ("discovery_classes", pa.list_(pa.string())),
                ("label_status", pa.string()),
            ]
        )
        path = self.root / "candidates.parquet"
        temporary = path.with_suffix(".next")
        pq.write_table(pa.Table.from_pylist(rows, schema=schema), temporary, compression="zstd")
        assert pq.read_table(temporary).to_pylist() == rows
        temporary.replace(path)
        report = {
            "status": read(self.root / "status.json"),
            "classes": read(self.root / "classes-status.json"),
            "jobs": self.state["jobs"],
            "candidate_rows": len(rows),
            "candidate_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "round_trip_verified": True,
            "reconciliation": "pending",
            "accepted_samples_changed": False,
            "scope": "Discovery claims only; reuse existing observations/policies in one subsequent reconciliation",
        }
        write(self.root / "collection-report.json", report)


def status(root):
    result = (
        read(root / "status.json") if (root / "status.json").exists() else {"phase": "not_started"}
    )
    # Check the actual held lock, not a stale PID or timestamp; this is read-only.
    path = root / "run.lock"
    alive = False
    if path.exists():
        with path.open("r") as lock:
            try:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                fcntl.flock(lock, fcntl.LOCK_UN)
            except BlockingIOError:
                alive = True
    result["process_alive"] = alive
    if not alive and result["phase"] not in TERMINAL | {"not_started"}:
        result["phase"] = "interrupted"
    return result


def execute(root):
    from .unattended_sources import prepare

    root.mkdir(parents=True, exist_ok=True)
    os.nice(10)
    with (root / "run.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        config = read(root / "config.json")
        os.chdir(config["working_directory"])
        for name, expected in config["implementation_sha256"].items():
            if hashlib.sha256((Path(__file__).parent / name).read_bytes()).hexdigest() != expected:
                raise ValueError(
                    "Collection implementation changed; preserve the frozen run before upgrading"
                )
        store = Path(config["store"])
        store.mkdir(parents=True, exist_ok=True)
        with (store / "hydrator.lock").open("a") as store_lock:
            fcntl.flock(store_lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            try:
                if not (root / "plan.json").exists():

                    def progress(message):
                        write(
                            root / "status.json",
                            {
                                "phase": "preparing",
                                "pid": os.getpid(),
                                "updated_at": time.time(),
                                "message": message,
                            },
                        )

                    write(root / "plan.json", prepare(config, progress))
                Runner(root, config, read(root / "plan.json")).run()
            except BaseException as error:
                write(
                    root / "status.json",
                    {
                        "phase": "failed",
                        "pid": os.getpid(),
                        "updated_at": time.time(),
                        "error": str(error)[:400],
                    },
                )
                raise


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["start", "resume", "status", "_run"])
    parser.add_argument("--run-dir", type=Path, required=True)
    parser.add_argument("--config", type=Path)
    parser.add_argument("--foreground", action="store_true")
    args = parser.parse_args(argv)
    root = args.run_dir.resolve()
    if args.action == "status":
        print(json.dumps(status(root), sort_keys=True))
        return
    if args.action == "_run":
        execute(root)
        return
    from .runtime import private_output

    private_output(root)
    if args.action == "start":
        if root.exists() or not args.config:
            parser.error(
                "start needs a new run directory and --config; use resume for existing runs"
            )
        config = read(args.config)
        config["working_directory"] = str(Path.cwd())
        config["implementation_sha256"] = implementation_hashes()
        write(root / "config.json", config)
    elif not (root / "config.json").exists():
        parser.error("No saved configuration to resume")
    if status(root)["process_alive"]:
        print(json.dumps({"already_running": True, "run_dir": str(root)}))
        return
    if args.foreground:
        execute(root)
    else:
        with (root / "process.log").open("a") as log:
            child = subprocess.Popen(
                [
                    sys.executable,
                    "-m",
                    "magika_datasets.unattended",
                    "_run",
                    "--run-dir",
                    str(root),
                ],
                cwd=read(root / "config.json")["working_directory"],
                stdin=subprocess.DEVNULL,
                stdout=log,
                stderr=subprocess.STDOUT,
                start_new_session=True,
            )
        print(json.dumps({"started_pid": child.pid, "run_dir": str(root)}))


if __name__ == "__main__":
    main()
