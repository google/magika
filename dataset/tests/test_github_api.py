# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""The licence lookup needs a token, caches every answer, and never raises on a 404."""

import json

import pytest

from magika_datasets import github_api


def client(tmp_path, monkeypatch, token="t"):
    monkeypatch.setenv("GITHUB_TOKEN", token)
    return github_api.MetadataClient(cache=tmp_path / "cache")


def test_a_token_is_required(tmp_path, monkeypatch):
    monkeypatch.delenv("GITHUB_TOKEN", raising=False)
    with pytest.raises(ValueError, match="GITHUB_TOKEN"):
        github_api.MetadataClient(cache=tmp_path / "cache")


def test_a_licence_is_read_and_cached(tmp_path, monkeypatch):
    calls = []
    payload = {
        "license": {"spdx_id": "MIT", "name": "MIT License"},
        "html_url": "https://github.com/a/b/blob/main/LICENSE",
    }
    api = client(tmp_path, monkeypatch)
    monkeypatch.setattr(api, "_fetch", lambda endpoint: calls.append(endpoint) or payload)
    assert api.license("a/b") == {
        "spdx_id": "MIT",
        "name": "MIT License",
        "permalink": "https://github.com/a/b/blob/main/LICENSE",
    }
    assert calls == ["repos/a/b/license"]
    assert api.license("a/b")["spdx_id"] == "MIT"
    assert calls == ["repos/a/b/license"], "the second read comes from the cache"


def test_a_cold_cache_on_disk_is_reused(tmp_path, monkeypatch):
    api = client(tmp_path, monkeypatch)
    monkeypatch.setattr(api, "_fetch", lambda endpoint: {"license": {"spdx_id": "ISC"}})
    api.license("a/b")
    fresh = client(tmp_path, monkeypatch)
    monkeypatch.setattr(fresh, "_fetch", lambda endpoint: pytest.fail("should not refetch"))
    assert fresh.license("a/b")["spdx_id"] == "ISC"


def test_a_missing_repository_is_recorded_not_raised(tmp_path, monkeypatch):
    api = client(tmp_path, monkeypatch)

    def missing(endpoint):
        raise github_api.GitHubStatus(404)

    monkeypatch.setattr(api, "_fetch", missing)
    assert api.license("a/gone") is None
    assert api.failures["a/gone"] == 404


def test_a_repository_with_no_licence_file_resolves_to_nothing(tmp_path, monkeypatch):
    api = client(tmp_path, monkeypatch)
    monkeypatch.setattr(api, "_fetch", lambda endpoint: {"license": None})
    assert api.license("a/b") is None


def test_noassertion_is_not_a_licence(tmp_path, monkeypatch):
    api = client(tmp_path, monkeypatch)
    monkeypatch.setattr(api, "_fetch", lambda endpoint: {"license": {"spdx_id": "NOASSERTION"}})
    assert api.license("a/b") is None


def test_a_failure_records_only_the_status(tmp_path, monkeypatch):
    api = client(tmp_path, monkeypatch, token="secret-token-value")

    def forbidden(endpoint):
        raise github_api.GitHubStatus(403)

    monkeypatch.setattr(api, "_fetch", forbidden)
    assert api.license("a/private") is None
    assert api.failures == {"a/private": 403}
    assert "secret-token-value" not in json.dumps(api.failures)


def test_an_offline_client_reads_the_cache_without_a_token(tmp_path, monkeypatch):
    """Cached answers are local evidence, so the default resolution keeps them."""
    warm = client(tmp_path, monkeypatch)
    monkeypatch.setattr(warm, "_fetch", lambda endpoint: {"license": {"spdx_id": "MIT"}})
    warm.license("a/b")
    monkeypatch.delenv("GITHUB_TOKEN", raising=False)
    offline = github_api.MetadataClient(cache=tmp_path / "cache", offline=True)
    assert offline.license("a/b")["spdx_id"] == "MIT"


def test_an_offline_client_never_reaches_the_network(tmp_path, monkeypatch):
    monkeypatch.delenv("GITHUB_TOKEN", raising=False)
    offline = github_api.MetadataClient(cache=tmp_path / "cache", offline=True)
    monkeypatch.setattr(offline, "_fetch", lambda endpoint: pytest.fail("made a request"))
    assert offline.license("never/seen") is None
