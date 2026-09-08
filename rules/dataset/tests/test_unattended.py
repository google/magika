import hashlib
import json
import time

import pytest

from magika_datasets.unattended import (
    Runner,
    Search,
    pause_provider,
    read,
    ready,
    retry_delay,
    status,
    upgrade_legacy_cooldown,
    write,
)
from magika_datasets.unattended_sources import cached_candidates, path_matcher


def fixture(kind="alpha", number=0, provider="virustotal", size=5):
    sha = hashlib.sha256(f"{kind}:{number}".encode()).hexdigest()
    return {
        "discovery_format": kind,
        "provider": provider,
        "sha256": sha,
        "size": size,
        "source": "virustotal",
        "revision": "a" * 64,
        "path": sha,
    }


def plan(kinds=("alpha", "beta", "gamma"), count=98):
    return {
        "selected_total": count * len(kinds),
        "known_sha256": [],
        "cached": {},
        "classes": {
            k: {"selected": count, "budget": 3 * (100 - count), "cached": [], "role": "format"}
            for k in kinds
        },
        "jobs": [
            {
                "id": k + ":" + p,
                "format_id": k,
                "provider": p,
                "preferred": p == "github",
                "fixtures": [],
                "queries": [],
            }
            for k in kinds
            for p in ("github", "virustotal")
        ],
    }


def report(batch):
    n = len(batch["fixtures"])
    return {
        "acquired_origins": n,
        "resumed_origins": 0,
        "new_objects": n,
        "new_bytes": sum(f["size"] for f in batch["fixtures"]),
        "failures": [],
        "deferred": [],
        "lfs_pointers": [],
    }


def test_retry_windows_and_provider_isolation():
    vt, gh = {}, {}
    now = 100
    for attempt in range(4):
        pause_provider(vt, {"http_status": 429, "error": "quota", "retry_after": "60"}, now)
        pause_provider(vt, {"http_status": 429, "error": "quota"}, now + 1)
        assert vt["failures"] == attempt + 1
        assert ready(gh, now)
        assert not ready(vt, now)
        now += 61
    assert vt["stopped"] and vt["reason"] == "retry_windows_exhausted"


@pytest.mark.parametrize("code", [401, 403])
def test_auth_is_terminal_without_retries(code):
    provider = {}
    pause_provider(provider, {"http_status": code, "error": "access"}, 100)
    assert provider["reason"] == "credentials_required"
    assert not ready(provider, 999999)


def test_retry_after_date_and_default():
    assert 2 <= retry_delay("bad", 100) <= 3
    assert retry_delay("Thu, 01 Jan 1970 00:03:20 GMT", 100) == 100


def test_tenacity_exponential_backoff_and_caps(monkeypatch):
    monkeypatch.setattr("tenacity.wait.random.uniform", lambda *_: 0)
    assert [retry_delay(None, 100, attempt=n) for n in range(1, 11)] == [
        2,
        4,
        8,
        16,
        32,
        64,
        128,
        256,
        300,
        300,
    ]
    assert [retry_delay(None, 100, attempt=n, http_status=429) for n in range(1, 6)] == [
        30,
        60,
        120,
        240,
        300,
    ]
    assert retry_delay("3600", 100, attempt=7, http_status=429) == 3600


def test_transport_reaches_five_minute_cap_before_stopping(monkeypatch):
    monkeypatch.setattr("tenacity.wait.random.uniform", lambda *_: 0)
    provider, now = {}, 100
    for attempt in range(1, 11):
        pause_provider(provider, {"error": "VirusTotal transport failed"}, now)
        assert provider.get("stopped", False) == (attempt == 10)
        if attempt == 9:
            assert provider["wait_seconds"] == 300
        now = provider["next_at"]


def test_backoff_survives_persisted_provider_state(tmp_path, monkeypatch):
    monkeypatch.setattr("tenacity.wait.random.uniform", lambda *_: 0)
    provider = {}
    error = {"error": "VirusTotal transport failed"}
    pause_provider(provider, error, 100)
    assert provider["next_at"] == 102
    write(tmp_path / "provider.json", provider)
    provider = read(tmp_path / "provider.json")
    pause_provider(provider, error, 101)
    assert provider["failures"] == 1
    pause_provider(provider, error, 102)
    assert provider["next_at"] == 106 and provider["failures"] == 2


def test_upgrade_only_shortens_legacy_transport_wait(monkeypatch):
    monkeypatch.setattr("tenacity.wait.random.uniform", lambda *_: 0)
    transport = {"error": "VirusTotal transport failed", "failures": 1, "next_at": 1900}
    upgrade_legacy_cooldown(transport, 200)
    assert transport["next_at"] == 200 and transport["failures"] == 1
    rate = {
        "error": "VirusTotal HTTP 429: quota/rate limit reached; retry later",
        "failures": 1,
        "next_at": 1900,
    }
    before = dict(rate)
    upgrade_legacy_cooldown(rate, 200)
    assert rate == before


def test_budget_size_and_duplicate_suppression_survive_resume(tmp_path):
    p = plan(("alpha",), 99)
    runner = Runner(tmp_path, {}, p)
    runner.enqueue([fixture(number=i) for i in range(10)] + [fixture()], "virustotal")
    assert runner.reserved["alpha"] == 3
    restored = Runner(tmp_path, {}, p)
    restored.enqueue([fixture(number=i) for i in range(10)], "virustotal")
    assert restored.reserved["alpha"] == 3
    other = Runner(tmp_path / "other", {}, plan(("alpha",)))
    other.enqueue([fixture(size=262145)], "github")
    other.enqueue([fixture(size=1048577)], "virustotal")
    assert not other.batches


def test_all_classes_and_fallback_then_resume_without_download(tmp_path, monkeypatch):
    real_sleep = time.sleep
    monkeypatch.setattr("magika_datasets.unattended.time.sleep", lambda _: real_sleep(0.005))
    downloads = []

    def search(job, state):
        return (
            [fixture(job["format_id"], i) for i in range(3)]
            if job["provider"] == "virustotal"
            else []
        ), {"done": True}

    def download(batch):
        downloads.append(batch["fixtures"][0]["discovery_format"])
        return report(batch)

    p = plan(count=99)
    config = {"store": str(tmp_path / "store")}
    Runner(tmp_path, config, p, search=search, download=download).run()
    assert sorted(downloads) == ["alpha", "beta", "gamma"]
    assert read(tmp_path / "status.json")["phase"] == "collected"
    assert read(tmp_path / "status.json")["new_objects"] == 9
    Runner(tmp_path, config, p, search=search, download=download).run()
    assert len(downloads) == 3


def test_partial_download_retry_counts_and_reuses_same_batch(tmp_path):
    runner = Runner(tmp_path, {}, plan(("alpha",)))
    runner.enqueue([fixture(number=i) for i in range(3)], "virustotal")
    key, batch = next(iter(runner.batches.items()))
    first = report(batch)
    first.update(
        acquired_origins=1,
        new_objects=1,
        new_bytes=5,
        failures=[{"http_status": 429, "error": "quota"}],
    )
    runner.finish_download(key, batch, first)
    assert len(runner.pending_batches()) == 1
    second = report(batch)
    second.update(acquired_origins=2, resumed_origins=1, new_objects=2, new_bytes=10)
    runner.finish_download(key, batch, second)
    receipt = read(tmp_path / "receipts" / (key + ".json"))
    assert receipt["acquired"] == 3 and receipt["new_objects"] == 3 and receipt["new_bytes"] == 15


def test_status_read_only_and_stale_pid(tmp_path):
    path = tmp_path / "status.json"
    write(path, {"phase": "collecting", "pid": 999999})
    before = {p.name: (p.read_bytes(), p.stat().st_mtime_ns) for p in tmp_path.iterdir()}
    assert status(tmp_path)["phase"] == "interrupted"
    assert before == {p.name: (p.read_bytes(), p.stat().st_mtime_ns) for p in tmp_path.iterdir()}


def test_cache_sha_and_explicit_negative_exclusion(tmp_path):
    data = b"cache"
    sha = hashlib.sha256(data).hexdigest()
    p = tmp_path / sha
    p.write_bytes(data)
    record = {"sha256": sha, "size": len(data), "path": str(p), "origins": [fixture()]}
    rows = {"alpha": {"samples": [], "class_role": "format"}}
    assert cached_candidates([record], rows, set()) == {"alpha": [sha]}
    assert (
        cached_candidates([record], rows, set(), [{"sha256": sha, "format_ids": ["invalid"]}]) == {}
    )
    p.write_bytes(b"wrong")
    assert cached_candidates([record], rows, set()) == {}


@pytest.mark.parametrize("license_value", [{"spdx_id": "GPL-3.0"}, None, {}])
def test_cache_rejects_unlicensed_github(tmp_path, license_value):
    data = b"abc"
    sha = hashlib.sha256(data).hexdigest()
    p = tmp_path / sha
    p.write_bytes(data)
    origin = {
        **fixture(provider="github", size=3),
        "source": "github:owner/repo",
        "claim": {"repository_license": license_value},
    }
    records = [{"sha256": sha, "size": 3, "path": str(p), "origins": [origin]}]
    assert (
        cached_candidates(records, {"alpha": {"samples": [], "class_role": "format"}}, {"MIT"})
        == {}
    )


def test_filename_hints_are_multilabel():
    match = path_matcher({"a": {"extensions": ["pdb"]}, "b": {"extensions": ["pdb"]}})
    assert match("tests/example.pdb") == {"a", "b"}


def test_vt_page_cache_filters_acquired_and_oversize(tmp_path):
    p = plan(("alpha",))
    p["known_sha256"] = [fixture()["sha256"]]
    query = "type:alpha"
    key = hashlib.sha256(json.dumps([query, None]).encode()).hexdigest()
    write(
        tmp_path / "search" / (key + ".json"),
        {
            "fixtures": [fixture(), fixture(number=1), fixture(number=2, size=1048577)],
            "next_cursor": None,
        },
    )
    result, update = Search(tmp_path, p)(
        {"provider": "virustotal", "format_id": "alpha", "queries": [query]}, {}
    )
    assert [f["sha256"] for f in result] == [fixture(number=1)["sha256"]]
    assert update["done"]
