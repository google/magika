# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""HTTPS origins hydrate like any other: bounded, redirect-free and bound to their hash."""

import hashlib
import io

import pytest

from magika_datasets import manifest
from magika_datasets.acquisition import object_path
from magika_datasets.parallel_acquisition import collect_parallel

CONTENT = b"sample bytes from an external collection"
SHA = hashlib.sha256(CONTENT).hexdigest()
URL = "https://example.org/samples/archive/arj/file.arj"


class Response(io.BytesIO):
    def __enter__(self):
        return self

    def __exit__(self, *exc):
        return False


def serve(monkeypatch, body=CONTENT):
    class Opener:
        def open(self, request, timeout):
            assert request.full_url == URL
            return Response(body)

    monkeypatch.setattr(manifest, "build_opener", lambda *handlers: Opener())


def test_an_https_origin_becomes_an_https_fixture():
    record = {"sha256": SHA, "size": len(CONTENT), "origins": [f"{URL}:{SHA}"]}
    (fixture,) = manifest.inventory([record])["fixtures"]
    assert (fixture["provider"], fixture["url"]) == ("https", URL)


@pytest.mark.parametrize(
    "origin",
    [
        f"http://example.org/a:{SHA}",
        f"https://user@example.org/a:{SHA}",
        f"https://example.org/a?x=1:{SHA}",
        f"{URL}:{'0' * 64}",
    ],
)
def test_unsafe_or_mismatched_https_origins_are_refused(origin):
    with pytest.raises(ValueError):
        manifest.validate_record({"sha256": SHA, "size": 1, "origins": [origin]})


def test_hydration_downloads_and_verifies_an_https_origin(tmp_path, monkeypatch):
    serve(monkeypatch)
    record = {"sha256": SHA, "size": len(CONTENT), "origins": [f"{URL}:{SHA}"]}
    report = collect_parallel(
        manifest.inventory([record]),
        tmp_path,
        max_file_bytes=1024,
        max_bytes=1024,
        max_files=10,
        vt_client=None,
        workers=2,
    )
    assert object_path(tmp_path, SHA).read_bytes() == CONTENT
    assert not report["failures"]


def test_changed_bytes_at_the_origin_are_rejected(tmp_path, monkeypatch):
    serve(monkeypatch, b"x" * len(CONTENT))
    with pytest.raises(ValueError, match="SHA-256"):
        manifest.HTTPSReader().read_into(
            {"url": URL, "size": len(CONTENT), "sha256": SHA}, io.BytesIO()
        )
