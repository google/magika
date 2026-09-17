# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""SQLite databases: header arithmetic, integrity_check on a read-only copy, schema dispatch."""

import os
import sqlite3
import struct
import tempfile
import threading

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("sqlite", "geopackage", "mbtiles")
SCOPE = "Header string, page size and page count equal to the file, PRAGMA integrity_check on an immutable read-only copy with a five-second interrupt, table names naming GeoPackage or MBTiles; generic sqlite passes never relabel hinted derivatives (qgis is claimed by the .qgz ZIP probe)"
MAGIC = b"SQLite format 3\0"
SECONDS = 5


def inspect(data: bytes) -> tuple[str, set[str], tuple[str, ...]]:
    """(integrity result, table names, tags) using a temporary immutable copy."""
    handle, path = tempfile.mkstemp(suffix=".sqlite")
    try:
        with os.fdopen(handle, "wb") as stream:
            stream.write(data)
        connection = sqlite3.connect(f"file:{path}?mode=ro&immutable=1", uri=True)
        timer = threading.Timer(SECONDS, connection.interrupt)
        timer.start()
        try:
            connection.execute("PRAGMA query_only=1")
            result = connection.execute("PRAGMA integrity_check").fetchone()[0]
            tables = {
                row[0]
                for row in connection.execute("SELECT name FROM sqlite_master WHERE type='table'")
            }
            journal = connection.execute("PRAGMA journal_mode").fetchone()[0]
        finally:
            timer.cancel()
            connection.close()
    finally:
        os.unlink(path)
    tags = ("wal_mode",) if journal == "wal" or data[18] == 2 else ()
    if any(name.endswith("_fts") or name.endswith("_content") for name in tables):
        tags += ("has_fts",)
    return result, tables, tags


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(MAGIC) or len(data) < 100:
        return None
    page_size = struct.unpack_from(">H", data, 16)[0]
    page_size = 65536 if page_size == 1 else page_size
    pages = struct.unpack_from(">I", data, 28)[0]
    if page_size < 512 or page_size > 65536 or page_size & (page_size - 1):
        return Observation("fail", "Invalid page size", "sqlite")
    if data[18] not in (1, 2) or data[19] not in (1, 2):
        return Observation("fail", "Invalid file format version", "sqlite")
    if pages and pages * page_size != len(data):
        return Observation("fail", "Page count differs from file size", "sqlite")
    if not pages and len(data) % page_size:
        return Observation("fail", "File is not a whole number of pages", "sqlite")
    try:
        result, tables, tags = inspect(data)
    except sqlite3.OperationalError as error:
        if "interrupted" in str(error):
            return Observation("inconclusive", "integrity_check exceeded the time limit", "sqlite")
        return Observation("fail", f"SQLite refused the database: {error}", "sqlite")
    except sqlite3.DatabaseError as error:
        return Observation("fail", f"SQLite refused the database: {error}", "sqlite")
    if result != "ok":
        return Observation("fail", f"integrity_check: {result[:80]}", "sqlite")
    if {"gpkg_contents", "gpkg_spatial_ref_sys"} <= tables:
        return Observation(
            "pass", "GeoPackage: integrity_check ok and gpkg tables present", "geopackage", tags
        )
    if {"tiles", "metadata"} <= tables or {"map", "images", "metadata"} <= tables:
        return Observation(
            "pass", "MBTiles: integrity_check ok and tile tables present", "mbtiles", tags
        )
    return Observation(
        "pass", f"integrity_check ok; {len(tables)} tables", "sqlite", tags, generic=True
    )
