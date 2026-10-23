import json
import zipfile

import pytest

from magika_datasets.acquisition import sha256_file
from magika_datasets.archive import RECEIPTS, TABLES, package


def metadata(tmp_path):
    for name in TABLES:
        (tmp_path / name).write_text(name)
    (tmp_path / "corpus-parquet-receipt.json").write_text(json.dumps({"samples": 1}))
    for table, receipt in RECEIPTS.items():
        (tmp_path / receipt).write_text(
            json.dumps({"parquet_sha256": sha256_file(tmp_path / table)})
        )
    (tmp_path / ".env").write_text("SECRET=do-not-package")
    (tmp_path / "corpus.bin").write_bytes(b"not metadata")


def test_archive_excludes_bytes_and_credentials_and_checks_integrity(tmp_path, monkeypatch):
    monkeypatch.setattr("magika_datasets.archive.check_sources", lambda *_: None)
    metadata(tmp_path)
    output = tmp_path / "out/metadata.zip"
    result = package(tmp_path, output)
    assert result["verified"] and result["samples"] == 1
    with zipfile.ZipFile(output) as archive:
        names = set(archive.namelist())
    assert names == {
        *TABLES,
        "corpus-parquet-receipt.json",
        *RECEIPTS.values(),
        "archive-manifest.json",
    }
    assert package(tmp_path, output)["sha256"] == result["sha256"]


def test_the_download_list_describes_what_was_packaged(tmp_path, monkeypatch):
    monkeypatch.setattr("magika_datasets.archive.check_sources", lambda *_: None)
    metadata(tmp_path)
    package(tmp_path, tmp_path / "out/metadata.zip")
    listed = json.loads((tmp_path / "metadata-downloads.json").read_text())["files"]
    assert {f["name"]: f["sha256"] for f in listed} == {
        name: sha256_file(tmp_path / name) for name in TABLES
    }


def test_a_table_its_receipt_does_not_describe_is_refused(tmp_path, monkeypatch):
    monkeypatch.setattr("magika_datasets.archive.check_sources", lambda *_: None)
    metadata(tmp_path)
    output = tmp_path / "out/metadata.zip"
    result = package(tmp_path, output)
    (tmp_path / "repository-licenses.parquet").write_bytes(b"stale")
    with pytest.raises(ValueError, match="repository-licenses.parquet"):
        package(tmp_path, output)
    assert sha256_file(output) == result["sha256"]


def test_a_further_collection_is_packaged_beside_the_corpus(tmp_path, monkeypatch):
    monkeypatch.setattr("magika_datasets.archive.check_sources", lambda *_: None)
    metadata(tmp_path)
    collection = tmp_path / "sembiance"
    collection.mkdir()
    for name in ("classes.parquet", "samples.parquet"):
        (collection / name).write_text(name)
    (collection / "corpus-parquet-receipt.json").write_text(json.dumps({"samples": 1}))
    output = tmp_path / "out/metadata.zip"
    package(tmp_path, output)
    with zipfile.ZipFile(output) as archive:
        assert "sembiance/samples.parquet" in archive.namelist()
    listed = json.loads((tmp_path / "metadata-downloads.json").read_text())["files"]
    assert "sembiance/samples.parquet" in {f["name"] for f in listed}


def test_the_hosting_location_is_recorded_beside_the_archive(tmp_path, monkeypatch):
    monkeypatch.setattr("magika_datasets.archive.check_sources", lambda *_: None)
    metadata(tmp_path)
    output = tmp_path / "out/metadata.zip"
    plain = package(tmp_path, output)["sha256"]
    url = "https://example.invalid/metadata.zip"
    result = package(tmp_path, output, download_url=url)
    # The link lives beside the archive, never inside it, so recording it leaves the bytes
    # already published untouched.
    assert result["sha256"] == plain
    assert result["download_url"] == url
    assert json.loads(output.with_suffix(".zip.json").read_text())["download_url"] == url
    listed = json.loads((tmp_path / "metadata-downloads.json").read_text())
    assert listed["download_url"] == url
    assert "Hosting location" not in listed["scope"]
