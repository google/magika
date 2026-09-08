import json

from magika_datasets.repository_pool import draw_repositories
from magika_datasets.unattended_sources import prepare


def test_rebalance_favors_unused_without_dropping_used_repositories():
    pool = [
        {
            "repository": name,
            "counts": {"classes": {"pdf": count}, "categories": {}, "total": count},
        }
        for name, count in [("a/used", 3), ("z/new", 0)]
    ]
    row = {"format_id": "pdf", "categories": []}
    for seed in range(10):
        result = draw_repositories(pool, row, seed=seed, limit=2, unused_first=True)
        assert [r["repository"] for r in result] == ["z/new", "a/used"]


def test_rebalance_includes_full_classes_and_reports_entire_pool(tmp_path, monkeypatch):
    import magika_datasets.unattended_sources as sources

    policy = tmp_path / "policy.json"
    policy.write_text(json.dumps({"allowed_spdx": ["MIT"]}))
    row = {
        "format_id": "pdf",
        "samples": [{"sha256": f"{i:064x}", "origins": []} for i in range(100)],
        "class_role": "format",
        "extensions": ["pdf"],
        "categories": ["document"],
    }
    entries = [
        {"repository": "a/one", "status": "inventoried", "eligible": True, "spdx_id": "MIT"},
        {"repository": "b/excluded", "status": "license_excluded", "eligible": False},
    ]
    pool = {
        "repositories": [
            {"repository": e["repository"], "counts": {"total": 0, "classes": {}, "categories": {}}}
            for e in entries
        ]
    }
    monkeypatch.setattr(sources, "load", lambda *_: (pool, entries, {"pdf": row}, iter([]), []))
    config = {
        "parquet_dir": str(tmp_path),
        "license_policy": str(policy),
        "store": str(tmp_path / "store"),
        "vt_recipes": [],
        "seed": 1,
        "rebalance": True,
        "rebalance_candidates_per_class": 20,
    }
    result = prepare(config, lambda _: None)
    assert result["classes"]["pdf"]["budget"] == 20
    assert result["classes"]["pdf"]["selected"] == 100
    assert [j["provider"] for j in result["jobs"]] == ["github"]
    assert result["repository_pool"]["total"] == 2
    assert result["repository_pool"]["eligible_indexed"] == 1
    assert result["cached"] == {}
    config.update(rebalance=False, target_per_class=120)
    result = prepare(config, lambda _: None)
    assert result["classes"]["pdf"]["budget"] == 60
