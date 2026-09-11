# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Resumable Sembiance source benchmark import, export and hash-joined evaluation."""

import argparse
import fcntl
import hashlib
import http.client
import json
import sqlite3
import tempfile
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import unquote, urljoin, urlsplit

import pyarrow as pa
import pyarrow.parquet as pq
from tenacity import retry, retry_if_exception, stop_after_attempt, wait_exponential

from .acquisition import object_path, sha256_file
from .http_source import download
from .parquet_corpus import pack
from .parquet_metadata import CLASS_SCHEMA, SAMPLE_SCHEMA
from .runtime import private_output

BASE = "https://sembiance.com/fileFormatSamples/"
CREDIT = "Sembiance / dexvert; original creators retain their rights"


def write(path, value):
    temporary = path.with_suffix(path.suffix + ".next")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    temporary.replace(path)


class Listing(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []

    def handle_starttag(self, tag, attrs):
        if tag == "a":
            self.links.extend(value for key, value in attrs if key == "href")


def entries(url, body):
    parser = Listing()
    parser.feed(body.decode("utf-8"))
    result = []
    for href in parser.links:
        resolved = urljoin(url, href)
        parsed = urlsplit(resolved)
        parts = unquote(parsed.path).split("/")
        if (
            not resolved.startswith(BASE)
            or parsed.query
            or parsed.fragment
            or any(p in (".", "..") for p in parts)
            or "\\" in unquote(parsed.path)
            or resolved == url
            or not resolved.startswith(url)
        ):
            continue
        relative = resolved[len(BASE) :]
        # Only immediate children; this excludes parent links and cross-directory links.
        if "/" in resolved[len(url) :].rstrip("/"):
            continue
        result.append((resolved, "directory" if resolved.endswith("/") else "file", relative))
    return sorted(set(result))


def retryable(error):
    return isinstance(error, (OSError, http.client.HTTPException, URLError)) and (
        not isinstance(error, HTTPError) or error.code in (408, 429, 500, 502, 503, 504)
    )


def retry_wait(state):
    error = state.outcome.exception()
    if isinstance(error, HTTPError):
        value = error.headers.get("Retry-After")
        if value:
            try:
                return max(0, float(value))
            except ValueError:
                from email.utils import parsedate_to_datetime

                try:
                    return max(
                        0,
                        (parsedate_to_datetime(value) - datetime.now(timezone.utc)).total_seconds(),
                    )
                except (ValueError, TypeError):
                    pass
    return wait_exponential(multiplier=2, max=300)(state)


@retry(
    retry=retry_if_exception(retryable), wait=retry_wait, stop=stop_after_attempt(3), reraise=True
)
def fetch(url, limit):
    return download(url, limit)


def connect(run):
    db = sqlite3.connect(run / "inventory.sqlite")
    db.row_factory = sqlite3.Row
    db.execute("""CREATE TABLE IF NOT EXISTS entries (
        url TEXT PRIMARY KEY, kind TEXT, path TEXT, status TEXT DEFAULT 'pending',
        sha256 TEXT, size INTEGER, detail TEXT)""")
    return db


def progress(db, run, phase):
    counts = {
        f"{r[0]}:{r[1]}": r[2]
        for r in db.execute("SELECT kind,status,count(*) FROM entries GROUP BY kind,status")
    }
    result = {
        "phase": phase,
        "counts": counts,
        "updated_at": datetime.now(timezone.utc).isoformat(),
    }
    write(run / "status.json", result)
    return result


def acquire(row, run, store, cap):
    try:
        if row["kind"] == "directory":
            body, info = fetch(row["url"], 4 * 1024**2)
            listing = run / "listings" / (hashlib.sha256(row["url"].encode()).hexdigest() + ".html")
            listing.write_bytes(body)
            return "done", info["sha256"], len(body), info, entries(row["url"], body)
        body, info = fetch(row["url"], cap)
        sha = info["sha256"]
        target = object_path(store, sha)
        target.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(dir=target.parent, delete=False) as stream:
            stream.write(body)
            temporary = Path(stream.name)
        temporary.replace(target)
        return "done", sha, len(body), info, []
    except Exception as error:
        status = (
            "excluded_size" if isinstance(error, ValueError) and "budget" in str(error) else "error"
        )
        return status, None, None, {"error": str(error)[:300]}, []


def run_import(run, store, workers, cap):
    run.mkdir(parents=True, exist_ok=True)
    (run / "listings").mkdir(exist_ok=True)
    store.mkdir(parents=True, exist_ok=True)
    with (run / "writer.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        db = connect(run)
        with db:
            db.execute(
                "INSERT OR IGNORE INTO entries(url,kind,path) VALUES (?,'directory','')", (BASE,)
            )
            # Recheck cached objects once on resume; completed downloads are never trusted by path alone.
            for row in db.execute(
                "SELECT * FROM entries WHERE kind='file' AND status='done'"
            ).fetchall():
                path = object_path(store, row["sha256"])
                if (
                    not path.exists()
                    or path.stat().st_size != row["size"]
                    or sha256_file(path) != row["sha256"]
                ):
                    db.execute("UPDATE entries SET status='pending' WHERE url=?", (row["url"],))
        try:
            with ThreadPoolExecutor(max_workers=workers) as pool:
                for kind in ("directory", "file"):
                    while True:
                        batch = db.execute(
                            "SELECT * FROM entries WHERE kind=? AND status='pending' ORDER BY path LIMIT 128",
                            (kind,),
                        ).fetchall()
                        if not batch:
                            break
                        progress(db, run, "inventory" if kind == "directory" else "downloading")
                        for row, result in zip(
                            batch, pool.map(lambda r: acquire(r, run, store, cap), batch)
                        ):
                            status, sha, size, detail, children = result
                            with db:
                                db.execute(
                                    "UPDATE entries SET status=?,sha256=?,size=?,detail=? WHERE url=?",
                                    (status, sha, size, json.dumps(detail), row["url"]),
                                )
                                db.executemany(
                                    "INSERT OR IGNORE INTO entries(url,kind,path) VALUES (?,?,?)",
                                    children,
                                )
                            progress(db, run, "inventory" if kind == "directory" else "downloading")
            return progress(db, run, "collected")
        finally:
            db.close()


def export(run, output, mapping, overlaps=()):
    """Preserve source labels; canonical mapping is an explicit separate evaluation view."""
    output.mkdir(parents=True, exist_ok=False)
    known = set()
    for path in overlaps:
        for batch in pq.ParquetFile(path).iter_batches(columns=["sha256"]):
            known.update(batch.column(0).to_pylist())
    with connect(run) as db:
        all_rows = [dict(row) for row in db.execute("SELECT * FROM entries ORDER BY path")]
    files = [r for r in all_rows if r["kind"] == "file" and r["status"] == "done"]
    by_hash = defaultdict(list)
    for row in files:
        by_hash[row["sha256"]].append(row)
    grouped = defaultdict(list)
    for sha, origins in sorted(by_hash.items()):
        claims = sorted({"/".join(r["path"].split("/")[:2]) for r in origins})
        assignments = [mapping.get(claim) for claim in claims]
        targets = {a["format_id"] for a in assignments if a is not None}
        mapped = len(targets) == 1 and all(a is not None for a in assignments)
        target = next(iter(targets)) if mapped else "sembiance/" + claims[0]
        conflict = len(targets) > 1 or (len(claims) > 1 and not mapped)
        annotation = {
            "dataset_id": "sembiance-eval-v1",
            "source_claims": claims,
            "format_ids": [target],
            "label_status": "source_claim",
            "validation_status": "unknown",
            "canonical_mapping_status": "reviewed_source_mapping" if mapped else "pending",
            "canonical_format_ids": sorted(targets) if mapped else [],
            "tags": sorted({tag for a in assignments if a for tag in a.get("tags", [])}),
            "hard_case": conflict,
            "conflicting": conflict,
            "attribution": CREDIT,
            "license_status": "unknown_per_sample",
            "source_references": [
                {"url": r["url"], "path": r["path"], "http": json.loads(r["detail"])}
                for r in origins
            ],
            "overlap_existing_sha256": bytes.fromhex(sha) in known,
            "near_duplicate_check": "pending",
            "evaluation_eligible": mapped and not conflict and bytes.fromhex(sha) not in known,
        }
        grouped[target].append(
            {
                "sha256": bytes.fromhex(sha),
                "size": origins[0]["size"],
                "origins": sorted(r["url"] + ":" + sha for r in origins),
                "hard_case": conflict,
                "annotation_json": json.dumps(annotation, sort_keys=True),
            }
        )
    classes, samples = [], []
    for ordinal, (kind, rows) in enumerate(sorted(grouped.items())):
        classes.append(
            {
                "ordinal": ordinal,
                "format_id": kind,
                "name": kind,
                "categories": ["external_evaluation"],
                "extensions": [],
                "metadata_json": json.dumps(
                    {"dataset_id": "sembiance-eval-v1", "attribution": CREDIT}
                ),
            }
        )
        for index, row in enumerate(rows):
            samples.append(
                {**row, "class_ordinal": ordinal, "sample_ordinal": index, "format_id": kind}
            )
    pq.write_table(
        pa.Table.from_pylist(classes, CLASS_SCHEMA), output / "classes.parquet", compression="zstd"
    )
    pq.write_table(
        pa.Table.from_pylist(samples, SAMPLE_SCHEMA), output / "samples.parquet", compression="zstd"
    )
    pq.write_table(
        pa.Table.from_pylist(all_rows), output / "source-inventory.parquet", compression="zstd"
    )
    write(output / "mapping.json", mapping)
    receipt = {
        "schema_version": 1,
        "samples": len(samples),
        "classes": len(classes),
        "files": {
            name: {"sha256": sha256_file(output / name)}
            for name in ["samples.parquet", "classes.parquet"]
        },
    }
    write(output / "corpus-parquet-receipt.json", receipt)
    write(
        output / "import-summary.json",
        {
            "source_file_urls": len(files),
            "unique_files": len(samples),
            "populated_classes": len(classes),
            "eligible_mapped_files": sum(
                json.loads(r["annotation_json"])["evaluation_eligible"] for r in samples
            ),
            "excluded_or_failed": Counter(r["status"] for r in all_rows if r["status"] != "done"),
            "label_basis": "upstream source claims, not automatic validation",
            "attribution": CREDIT,
            "overlap_inputs": [{"name": p.name, "sha256": sha256_file(p)} for p in overlaps],
            "near_duplicate_check": "pending; do not claim certified independent holdout",
        },
    )
    return receipt


def score(samples, predictions):
    expected = {}
    label_bases = Counter()
    for row in pq.read_table(samples).to_pylist():
        annotation = json.loads(row["annotation_json"])
        if annotation["evaluation_eligible"]:
            expected[row["sha256"]] = row["format_id"]
            label_bases[annotation.get("evaluation_basis", "reviewed_source_mapping")] += 1
    seen, counts = set(), defaultdict(Counter)
    for row in pq.read_table(predictions, columns=["sha256", "format_id"]).to_pylist():
        sha = bytes.fromhex(row["sha256"]) if isinstance(row["sha256"], str) else row["sha256"]
        if sha in seen:
            raise ValueError("Duplicate prediction SHA-256")
        seen.add(sha)
        if sha in expected:
            counts[expected[sha]][
                "correct" if row["format_id"] == expected[sha] else "incorrect"
            ] += 1
    for sha, kind in expected.items():
        if sha not in seen:
            counts[kind]["missing"] += 1
    accuracies = [v["correct"] / sum(v.values()) for v in counts.values()]
    return {
        "eligible_samples": len(expected),
        "label_bases": dict(label_bases),
        "per_class": counts,
        "macro_accuracy": sum(accuracies) / len(accuracies) if accuracies else None,
        "basis": "per-sample evaluation_basis: structural validation or reviewed upstream mapping; exact overlaps excluded, near-duplicates not certified",
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "action", choices=["start", "resume", "status", "export", "score", "validate"]
    )
    parser.add_argument("--run-dir", type=Path, default=Path(".local/sembiance"))
    parser.add_argument("--store", type=Path, default=Path(".local/sembiance-store"))
    parser.add_argument("--output", type=Path)
    parser.add_argument("--mapping", type=Path, default=Path("config/sembiance-mapping.json"))
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--max-file-bytes", type=int, default=1024**2)
    parser.add_argument("--overlap", type=Path, action="append", default=[])
    parser.add_argument("--predictions", type=Path)
    parser.add_argument("--samples", type=Path)
    parser.add_argument("--metadata", type=Path)
    parser.add_argument("--taxonomy", type=Path)
    args = parser.parse_args(argv)
    if args.action == "validate":
        from .external_validation import validate_external

        if not all((args.metadata, args.taxonomy, args.output)):
            parser.error("validate requires --metadata, --taxonomy and --output")
        print(
            json.dumps(
                validate_external(
                    args.metadata,
                    args.store,
                    args.taxonomy,
                    private_output(args.output),
                    args.workers,
                ),
                sort_keys=True,
            )
        )
        return
    if args.action == "status":
        print((args.run_dir / "status.json").read_text())
        return
    if args.action == "score":
        print(json.dumps(score(args.samples, args.predictions), sort_keys=True))
        return
    if not 1 <= args.workers <= 8 or not 1 <= args.max_file_bytes <= 1024**2:
        parser.error("Use 1..8 workers and at most 1 MiB per source file")
    run, store = private_output(args.run_dir), private_output(args.store)
    mapping = json.loads(args.mapping.read_text())
    if args.action in ("start", "resume"):
        config = {"source": BASE, "max_file_bytes": args.max_file_bytes, "mapping": mapping}
        if (run / "config.json").exists() and json.loads(
            (run / "config.json").read_text()
        ) != config:
            raise ValueError("Resume configuration differs; choose a new run")
        run.mkdir(parents=True, exist_ok=True)
        write(run / "config.json", config)
        run_import(run, store, args.workers, args.max_file_bytes)
    if args.output:
        output = private_output(args.output)
        receipt = export(run, output, mapping, args.overlap)
        result = pack(
            output / "samples.parquet",
            output / "classes.parquet",
            receipt,
            store,
            output / "snapshot",
        )
        if args.action in ("start", "resume"):
            write(run / "delivery.json", {"output": str(output), "verification": result})
            with connect(run) as db:
                print(json.dumps(progress(db, run, "complete"), sort_keys=True))
        else:
            print(json.dumps({"output": str(output), "verification": result}, sort_keys=True))
