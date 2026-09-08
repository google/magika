import json
import tarfile

import pytest

from magika_datasets.acquisition import sha256_file
from magika_datasets.archive import BASE_FILES, package


def test_archive_excludes_bytes_and_credentials_and_checks_integrity(tmp_path, monkeypatch):
    monkeypatch.setattr("magika_datasets.archive.check_sources", lambda *_: None)
    for name in BASE_FILES:
        (tmp_path / name).write_text("{}")
    (tmp_path / "corpus-parquet-receipt.json").write_text(json.dumps({"samples": 1}))
    (tmp_path / "metadata-downloads.json").write_text(
        json.dumps(
            {
                "files": [
                    {"name": "samples.parquet", "sha256": sha256_file(tmp_path / "samples.parquet")}
                ]
            }
        )
    )
    (tmp_path / ".env").write_text("SECRET=do-not-package")
    (tmp_path / "corpus.bin").write_bytes(b"not metadata")
    output = tmp_path / "out/metadata.tar.gz"
    result = package(tmp_path, output)
    assert result["verified"] and result["accepted_samples"] == 1
    with tarfile.open(output) as archive:
        assert set(archive.getnames()) == {*BASE_FILES, "archive-manifest.json"}
    assert package(tmp_path, output)["sha256"] == result["sha256"]
    (tmp_path / "samples.parquet").write_bytes(b"corrupt")
    with pytest.raises(ValueError, match="checksum mismatch"):
        package(tmp_path, output)
    assert sha256_file(output) == result["sha256"]
