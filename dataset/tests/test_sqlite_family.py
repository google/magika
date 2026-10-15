import sqlite3
import tempfile
from pathlib import Path

from helpers import status

from magika_datasets.validators.data import sqlite


def database(statements=("CREATE TABLE t(x INTEGER)", "INSERT INTO t VALUES (1)"), journal=None):
    with tempfile.TemporaryDirectory() as folder:
        path = Path(folder) / "db.sqlite"
        connection = sqlite3.connect(path)
        if journal:
            connection.execute(f"PRAGMA journal_mode={journal}")
        for statement in statements:
            connection.execute(statement)
        connection.commit()
        if journal == "wal":
            connection.execute("PRAGMA wal_checkpoint(TRUNCATE)")
        connection.close()
        return path.read_bytes()


def test_sqlite_integrity_and_dispatch():
    observation = sqlite.validate(database(), frozenset())
    assert (observation.status, observation.format_id, observation.generic) == (
        "pass",
        "sqlite",
        True,
    )
    corrupt = bytearray(database())
    corrupt[4096] = 0x00  # invalid b-tree page type on page 2
    assert status(sqlite, bytes(corrupt)) != "pass"
    assert status(sqlite, database()[:-100]) == "fail"
    assert status(sqlite, database() + b"\0") == "fail"
    assert status(sqlite, b"SQLite format 4\0" + b"\0" * 200) == "not_applicable"
    gpkg = database(
        (
            "CREATE TABLE gpkg_spatial_ref_sys(srs_id INTEGER)",
            "CREATE TABLE gpkg_contents(table_name TEXT)",
        )
    )
    assert sqlite.validate(gpkg, frozenset()).format_id == "geopackage"
    mbtiles = database(
        (
            "CREATE TABLE tiles(zoom_level INTEGER, tile_column INTEGER, tile_row INTEGER, tile_data BLOB)",
            "CREATE TABLE metadata(name TEXT, value TEXT)",
            "INSERT INTO metadata VALUES ('format', 'png')",
        )
    )
    assert sqlite.validate(mbtiles, frozenset()).format_id == "mbtiles"
    qgd = database(("CREATE TABLE qgis_projects(name TEXT)",))
    assert (
        sqlite.validate(qgd, frozenset({"qgis"})).format_id == "sqlite"
    )  # qgz owns the qgis class


def test_sqlite_wal_mode_is_tagged():
    observation = sqlite.validate(database(journal="wal"), frozenset())
    assert observation.status == "pass" and "wal_mode" in observation.tags
