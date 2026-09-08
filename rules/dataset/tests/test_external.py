import hashlib
import io
import json

import pyarrow as pa
import pyarrow.parquet as pq
import pytest

from magika_datasets import external, http_source
from magika_datasets.manifest import inventory, validate_record
from magika_datasets.parallel_acquisition import collect_parallel
from magika_datasets.parquet_corpus import pack, verify


class Response(io.BytesIO):
    def __init__(self, data, headers=None):
        super().__init__(data)
        self.headers = headers or {}


def mock_http(monkeypatch, body, headers=None):
    monkeypatch.setattr(http_source, "open_response", lambda *a, **k: Response(body, headers))


def test_http_hash_size_and_url(monkeypatch):
    data = b"payload"
    sha = hashlib.sha256(data).hexdigest()
    mock_http(monkeypatch, data)
    assert http_source.download("https://example.org/a", 7, 7, sha)[0] == data
    for size, digest in [(6, sha), (8, sha), (7, "0" * 64)]:
        with pytest.raises(ValueError):
            http_source.download("https://example.org/a", size, size, digest)
    for url in [
        "http://example.org/a",
        "https://user:secret@example.org/a",
        "https://example.org/a#f",
    ]:
        with pytest.raises(ValueError):
            http_source.validate_url(url)
    mock_http(monkeypatch, data, {"Content-Length": "10"})
    with pytest.raises(ValueError):
        http_source.download("https://example.org/a", 20)


def test_manifest_https_and_durable_hydration(tmp_path, monkeypatch):
    data = b"payload"
    sha = hashlib.sha256(data).hexdigest()
    row = {"sha256": sha, "size": len(data), "origins": ["https://example.org/a:" + sha]}
    validate_record(row)
    fixtures = inventory([row])
    mock_http(monkeypatch, data)
    kwargs = dict(max_file_bytes=10, max_bytes=10, max_files=1, vt_client=None, workers=1)
    first = collect_parallel(fixtures, tmp_path, **kwargs)
    assert first["new_objects"] == 1 and not first["failures"]
    monkeypatch.setattr(
        http_source, "download", lambda *a: pytest.fail("re-downloaded completed bytes")
    )
    assert collect_parallel(fixtures, tmp_path, **kwargs)["resumed_origins"] == 1


def test_listing_stays_in_collection():
    page = b'<a href="../">up</a><a href="%23a.bin">ok</a><a href="https://evil.org/a">bad</a><a href="%2e%2e/b">bad</a><a href="sub/">dir</a>'
    rows = external.entries(external.BASE + "image/png/", page)
    assert len(rows) == 2
    assert any("%23a.bin" in r[0] for r in rows)


def test_import_export_rehydrate_and_score(tmp_path, monkeypatch):
    run, store = tmp_path / "run", tmp_path / "store"
    pages = {
        external.BASE: b'<a href="image/">image</a>',
        external.BASE + "image/": b'<a href="png/">png</a>',
        external.BASE + "image/png/": b'<a href="a.png">a</a><a href="b.png">duplicate</a>',
    }

    def fetch(url, limit):
        body = pages.get(url, b"fixture")
        return body, {"sha256": hashlib.sha256(body).hexdigest()}

    monkeypatch.setattr(external, "fetch", fetch)
    external.run_import(run, store, 2, 1024)
    monkeypatch.setattr(external, "fetch", lambda *a: pytest.fail("resume issued a request"))
    external.run_import(run, store, 2, 1024)
    out = tmp_path / "export"
    receipt = external.export(run, out, {"image/png": {"format_id": "png"}})
    assert receipt["samples"] == 1
    row = pq.read_table(out / "samples.parquet").to_pylist()[0]
    assert len(row["origins"]) == 2
    assert json.loads(row["annotation_json"])["evaluation_eligible"]
    assert json.loads(row["annotation_json"])["validation_status"] == "unknown"
    pack(out / "samples.parquet", out / "classes.parquet", receipt, store, out / "snapshot")
    assert (
        verify(out / "snapshot", out / "samples.parquet", out / "classes.parquet")["samples"] == 1
    )
    predictions = tmp_path / "predictions.parquet"
    pq.write_table(
        pa.Table.from_pylist([{"sha256": row["sha256"], "format_id": "png"}]), predictions
    )
    assert external.score(out / "samples.parquet", predictions)["macro_accuracy"] == 1
    excluded = tmp_path / "overlap-export"
    external.export(run, excluded, {"image/png": {"format_id": "png"}}, [out / "samples.parquet"])
    assert external.score(excluded / "samples.parquet", predictions)["eligible_samples"] == 0


def test_connection_reuse_and_redirect_refusal(monkeypatch):
    from urllib.error import HTTPError

    connections = []
    codes = [200, 200, 302, 200]

    class Connection:
        def __init__(self, host, timeout):
            self.host = host
            connections.append(self)

        def request(self, method, path, headers):
            assert method == "GET"

        def getresponse(self):
            response = Response(b"x")
            response.status = codes.pop(0)
            response.reason = "test"
            response.isclosed = lambda: True
            return response

        def close(self):
            pass

    monkeypatch.setattr(http_source.http.client, "HTTPSConnection", Connection)
    monkeypatch.setattr(http_source._connections, "connection", None, raising=False)
    for _ in range(2):
        assert http_source.download("https://example.org/a", 1)[0] == b"x"
    assert len(connections) == 1
    with pytest.raises(HTTPError):
        http_source.download("https://example.org/a", 1)
    assert http_source.download("https://example.org/a", 1)[0] == b"x"
    assert len(connections) == 2
    http_source._connections.connection = None


def test_server_retry_timing_and_auth_failures():
    from types import SimpleNamespace
    from urllib.error import HTTPError

    error = HTTPError("https://example.org", 429, "rate limited", {"Retry-After": "120"}, None)
    state = SimpleNamespace(outcome=SimpleNamespace(exception=lambda: error))
    assert external.retry_wait(state) == 120
    assert external.retryable(error)
    assert not external.retryable(HTTPError("https://example.org", 403, "forbidden", {}, None))


def test_status_does_not_create_state(tmp_path, capsys):
    (tmp_path / "status.json").write_text('{"phase":"complete"}')
    external.main(["status", "--run-dir", str(tmp_path)])
    assert "complete" in capsys.readouterr().out
    assert not (tmp_path / "inventory.sqlite").exists()


def test_reviewed_crosswalk_preserves_known_name_collisions():
    from pathlib import Path

    mapping = json.loads((Path(__file__).parents[1] / "config/sembiance-mapping.json").read_text())
    assert "image/pcd" not in mapping  # Kodak Photo CD is not point-cloud PCD.
    assert "image/srt" not in mapping  # Synthetic Arts is not SubRip.
    assert "text/c" not in mapping  # Upstream explicitly groups C and C++.
    assert mapping["text/pointCloudData"]["format_id"] == "pcd"
    assert mapping["image/bigTIFF"]["format_id"] == "tiff"
    assert mapping["archive/jarARJ"]["format_id"] != mapping["archive/jar"]["format_id"]
