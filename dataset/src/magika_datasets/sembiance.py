# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Import Sembiance's file-format samples as a second collection on the same pipeline.

`crawl` walks the public directory listings and downloads every file into the shared byte
store, recording each URL, SHA-256 and size in a resumable SQLite ledger. `export` writes a
metadata directory with the corpus taxonomy and one row per distinct file: its origins are
plain HTTPS URLs, so `hydrate` fetches them like any other origin, and the reviewed folder
mapping is only a discovery hint, so `validate` labels the samples exactly as it labels the
main corpus. Credit: Sembiance / dexvert; original creators retain their rights.
"""

import argparse
import hashlib
import json
import shutil
import sqlite3
import tempfile
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urljoin, urlsplit
from urllib.request import Request, build_opener

from .acquisition import object_path
from .parquet_metadata import MAX_SAMPLE_BYTES, write_samples
from .receipt import refresh
from .runtime import private_output
from .virustotal import NoRedirect

BASE = "https://sembiance.com/fileFormatSamples/"
CREDIT = "Sembiance / dexvert; original creators retain their rights"
LISTING_BYTES = 4 * 1024 * 1024
BATCH = 256


class Links(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []

    def handle_starttag(self, tag, attrs):
        if tag == "a":
            self.links.extend(value for key, value in attrs if key == "href" and value)


def children(url: str, body: bytes) -> list[tuple[str, str]]:
    """(url, kind) for the entries directly under a listing, never outside BASE."""
    parser = Links()
    parser.feed(body.decode("utf-8", "replace"))
    found = set()
    for href in parser.links:
        target = urljoin(url, href)
        parts = urlsplit(target)
        rest = target[len(url) :]
        if (
            not target.startswith(url)
            or target == url
            or parts.query
            or parts.fragment
            or ".." in unquote(parts.path).split("/")
            or "/" in rest.rstrip("/")
        ):
            continue
        found.add((target, "directory" if target.endswith("/") else "file"))
    return sorted(found)


def fetch(url: str, limit: int) -> bytes:
    request = Request(url, headers={"Accept-Encoding": "identity"})
    with build_opener(NoRedirect()).open(request, timeout=60) as response:
        body = response.read(limit + 1)
    if len(body) > limit:
        raise OverflowError(f"larger than {limit} bytes")
    return body


def ledger(run: Path) -> sqlite3.Connection:
    run.mkdir(parents=True, exist_ok=True)
    db = sqlite3.connect(run / "inventory.sqlite")
    db.execute(
        "CREATE TABLE IF NOT EXISTS entries (url TEXT PRIMARY KEY, kind TEXT, path TEXT,"
        " status TEXT DEFAULT 'pending', sha256 TEXT, size INTEGER, detail TEXT)"
    )
    db.execute(
        "INSERT OR IGNORE INTO entries(url, kind, path) VALUES (?, 'directory', '')", (BASE,)
    )
    return db


def visit(url: str, kind: str, store: Path, limit: int):
    """(status, sha256, size, children) for one ledger entry."""
    try:
        if kind == "directory":
            return "done", None, None, children(url, fetch(url, LISTING_BYTES))
        body = fetch(url, limit)
        sha = hashlib.sha256(body).hexdigest()
        target = object_path(store, sha)
        if not target.exists():
            target.parent.mkdir(parents=True, exist_ok=True)
            # Two URLs can carry the same bytes and finish together; each writes its own file.
            with tempfile.NamedTemporaryFile(dir=target.parent, delete=False) as handle:
                handle.write(body)
            Path(handle.name).replace(target)
        return "done", sha, len(body), []
    except OverflowError:
        return "excluded_size", None, None, []
    except OSError as error:
        return f"error: {str(error)[:120]}", None, None, []


def crawl(run: Path, store: Path, *, limit: int = MAX_SAMPLE_BYTES, workers: int = 4) -> dict:
    """Walk listings, then download files, resuming from whatever the ledger already holds."""
    db = ledger(run)
    with ThreadPoolExecutor(workers) as pool:
        for kind in ("directory", "file"):
            while rows := db.execute(
                "SELECT url FROM entries WHERE kind = ? AND status = 'pending' LIMIT ?",
                (kind, BATCH),
            ).fetchall():
                urls = [url for (url,) in rows]
                for url, (status, sha, size, found) in zip(
                    urls, pool.map(lambda u: visit(u, kind, store, limit), urls)
                ):
                    db.execute(
                        "UPDATE entries SET status = ?, sha256 = ?, size = ? WHERE url = ?",
                        (status, sha, size, url),
                    )
                    db.executemany(
                        "INSERT OR IGNORE INTO entries(url, kind, path) VALUES (?, ?, ?)",
                        [(u, k, u[len(BASE) :]) for u, k in found],
                    )
                db.commit()
    counts = dict(db.execute("SELECT status, count(*) FROM entries GROUP BY status").fetchall())
    db.close()
    return counts


def export(run: Path, store: Path, metadata: Path, classes: Path, mapping: dict) -> dict:
    """One need_review sample per distinct file, hinted with the classes its folders map to."""
    import pyarrow.parquet as pq

    metadata.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(classes, metadata / "classes.parquet")
    formats = {r["format_id"]: r for r in pq.read_table(classes).to_pylist()}
    db = sqlite3.connect(run / "inventory.sqlite")
    urls = defaultdict(list)
    sizes = {}
    for url, sha, size in db.execute(
        "SELECT url, sha256, size FROM entries WHERE kind = 'file' AND status = 'done'"
    ):
        if size <= MAX_SAMPLE_BYTES and object_path(store, sha).exists():
            urls[sha].append(url)
            sizes[sha] = size
    db.close()
    rows = []
    for sha in sorted(urls):
        folders = sorted({"/".join(url[len(BASE) :].split("/")[:2]) for url in urls[sha]})
        hints = sorted({mapping[f] for f in folders if f in mapping and mapping[f] in formats})
        annotation = {
            "format_ids": ["unknown"],
            "discovery_format_ids": hints,
            "source_claims": folders,
            "attribution": CREDIT,
        }
        rows.append(
            {
                "class_ordinal": formats["unknown"]["ordinal"],
                "sample_ordinal": 0,
                "format_id": "unknown",
                "label_status": "need_review",
                "sha256": bytes.fromhex(sha),
                "size": sizes[sha],
                "origins": sorted(f"{url}:{sha}" for url in urls[sha]),
                "hard_case": False,
                "annotation_json": json.dumps(annotation, sort_keys=True),
            }
        )
    write_samples({"unknown": rows}, formats, metadata / "samples.parquet")
    refresh(metadata)
    return {
        "samples": len(rows),
        "hinted": sum(bool(json.loads(r["annotation_json"])["discovery_format_ids"]) for r in rows),
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, prog="magika-datasets sembiance")
    parser.add_argument("action", choices=["crawl", "export"])
    parser.add_argument("--run-dir", type=Path, default=Path("local/sembiance"))
    parser.add_argument("--store", type=Path, default=Path("local/corpus"))
    parser.add_argument("--metadata", type=Path, default=Path("sembiance"))
    parser.add_argument("--classes", type=Path, default=Path("classes.parquet"))
    parser.add_argument("--mapping", type=Path, default=Path("config/sembiance-mapping.json"))
    parser.add_argument("--workers", type=int, default=4)
    args = parser.parse_args(argv)
    if args.action == "crawl":
        result = crawl(private_output(args.run_dir), args.store, workers=args.workers)
    else:
        mapping = json.loads(args.mapping.read_text())["classes"]
        result = export(args.run_dir, args.store, args.metadata, args.classes, mapping)
    print(json.dumps(result, sort_keys=True))
