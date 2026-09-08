import hashlib
import json

import pyarrow.parquet as pq
import pytest

from magika_datasets.acquisition import object_path
from magika_datasets.candidate_conflicts import ConflictSearch, conflict_plan, disagreement
from magika_datasets.candidate_export import TOOLS, export_candidates
from magika_datasets.hydrate import main as hydrate
from magika_datasets.observation_cache import seed_journal
from magika_datasets.parquet_corpus import pack
from magika_datasets.pipeline import pipeline_status
from magika_datasets.unattended import write
from magika_datasets.verdicts import collect, tool_identity


def rows():
    return [
        {
            "format_id": k,
            "name": k,
            "categories": ["data"],
            "class_role": "format",
            "aliases": [],
            "samples": [],
            "extensions": [k],
        }
        for k in ("pdf", "lz4")
    ]


def markings():
    return {"magika": "pdf", "magic": "LZ4 compressed data (v1.4+)"}


class FakeIndex:
    def observations(self, record):
        return {
            name: {
                "basis": "local_bytes",
                "status": "ok",
                "exit_code": 0,
                "output_truncated": False,
                "stdout": "{}",
                "raw_record_sha256": "a" * 64,
            }
            for name in TOOLS
        }


def test_conflict_plan_includes_full_classes_and_missing_queries():
    classes = rows()
    classes[0]["samples"] = [{"sha256": f"{i:064x}"} for i in range(100)]
    classes[1]["extensions"] = []
    plan = conflict_plan(classes, [], [], per_class=20)
    assert set(plan["classes"]) == {"pdf", "lz4"}
    assert plan["classes"]["pdf"]["budget"] == 20
    assert next(j for j in plan["jobs"] if j["format_id"] == "lz4")["queries"] == []


def test_disagreement_requires_two_mapped_fields():
    formats = {r["format_id"]: r for r in rows()}
    assert disagreement(markings(), formats)
    assert not disagreement({"magika": "pdf"}, formats)
    assert not disagreement({"magika": "unknown", "magic": "LZ4 compressed data (v1.4+)"}, formats)


def test_disagreement_search_retains_claims_and_page_limit(tmp_path, monkeypatch):
    fixtures = [{"claim": markings()}, {"claim": {"magika": "pdf"}}]
    monkeypatch.setattr(
        "magika_datasets.unattended.Search.__call__",
        lambda *_: (fixtures, {"pages": 1, "done": False}),
    )
    search = ConflictSearch(
        tmp_path, {"known_sha256": []}, {r["format_id"]: r for r in rows()}, max_pages=1
    )
    found, result = search({"queries": ["name:*.pdf"]}, {})
    assert found == fixtures[:1] and result["done"] and result["disagreement_candidates"] == 1


def test_all_candidates_preserved_with_manual_labels_and_fresh_hydration(tmp_path, monkeypatch):
    store = tmp_path / "store"
    payloads = [b"candidate-one", b"candidate-two"]
    records, by_sha = [], {}
    for data in payloads:
        sha = hashlib.sha256(data).hexdigest()
        path = object_path(store, sha)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        by_sha[sha] = data
        records.append(
            {
                "sha256": sha,
                "size": len(data),
                "path": str(path),
                "origins": [
                    {
                        "provider": "virustotal",
                        "source": "virustotal",
                        "sha256": sha,
                        "revision": "b" * 64,
                        "path": sha,
                        "claim": markings() if len(records) == 0 else {},
                    }
                ],
            }
        )
    metadata = tmp_path / "metadata"
    receipt = export_candidates(
        records, {r["sha256"]: {"pdf"} for r in records}, rows(), FakeIndex(), [], metadata
    )
    assert receipt["samples"] == 2 and receipt["completed_observations"] == 16
    assert receipt["label_status_counts"] == {"conflicting": 1, "unresolved": 1}
    review = pq.read_table(metadata / "candidate-index.parquet").to_pylist()
    assert {r["sha256"].hex() for r in review} == set(by_sha)
    assert all(len(json.loads(r["verdicts_json"])) == 8 for r in review)
    for r in pq.read_table(metadata / "samples.parquet").to_pylist():
        assert json.loads(r["annotation_json"])["format_ids"] == []
    result = pack(
        metadata / "samples.parquet",
        metadata / "classes.parquet",
        receipt,
        store,
        tmp_path / "snapshot",
    )
    assert result["verified"] and result["whole_file_sha256_checked"] == 2

    class Download:
        def read_into(self, fixture, output):
            data = by_sha[fixture["sha256"]]
            output.write(data)
            return fixture["sha256"], False

        def close(self):
            pass

    monkeypatch.setattr("magika_datasets.parallel_acquisition.Client", Download)
    hydrate(
        [
            "--samples",
            str(metadata / "samples.parquet"),
            "--classes",
            str(metadata / "classes.parquet"),
            "--receipt",
            str(metadata / "corpus-parquet-receipt.json"),
            "--store",
            str(tmp_path / "fresh-store"),
            "--output",
            str(tmp_path / "fresh-snapshot"),
            "--download",
        ]
    )
    assert json.loads((tmp_path / "fresh-snapshot/verification.json").read_text())["verified"]
    for sha, data in by_sha.items():
        assert object_path(tmp_path / "fresh-store", sha).read_bytes() == data


def test_missing_observation_does_not_publish(tmp_path):
    class Incomplete(FakeIndex):
        def observations(self, record):
            result = super().observations(record)
            result["file"]["status"] = "pending"
            return result

    with pytest.raises(ValueError, match="eight completed"):
        export_candidates(
            [{"sha256": "0" * 64, "path": "no-file", "origins": []}],
            {},
            rows(),
            Incomplete(),
            [],
            tmp_path,
        )
    assert not (tmp_path / "corpus-parquet-receipt.json").exists()


def test_completed_pipeline_status_is_terminal_and_read_only(tmp_path):
    write(tmp_path / "status.json", {"phase": "complete", "candidates": 2})
    before = (tmp_path / "status.json").stat().st_mtime_ns
    assert pipeline_status(tmp_path)["phase"] == "complete"
    assert not pipeline_status(tmp_path)["process_alive"]
    assert (tmp_path / "status.json").stat().st_mtime_ns == before


def test_cached_observations_skip_execution_and_use_latest_exact_identity(tmp_path):
    tool = {"id": "file", "version": "test", "argv": ["must-not-run", "{path}"]}
    identity, _ = tool_identity(tool)
    old = tmp_path / "old.jsonl"
    sha = "a" * 64
    old.write_text(
        "".join(
            json.dumps(r) + "\n"
            for r in [
                {
                    "sha256": sha,
                    "tool_identity": identity,
                    "observed_at": 1,
                    "status": "ok",
                    "tool": "file",
                    "stdout": "older",
                },
                {
                    "sha256": sha,
                    "tool_identity": "obsolete",
                    "observed_at": 9,
                    "status": "ok",
                    "tool": "file",
                },
                {
                    "sha256": sha,
                    "tool_identity": identity,
                    "observed_at": 2,
                    "status": "ok",
                    "tool": "file",
                    "stdout": "latest",
                },
            ]
        )
    )
    journal = tmp_path / "verdicts.jsonl"
    records = [{"sha256": sha, "path": "must-not-read"}]
    seed_journal([old], journal, records, [tool])
    assert json.loads(journal.read_text())["stdout"] == "latest"
    result = collect(records, [tool], journal, workers=1)
    assert result["new"] == 0 and result["resumed"] == 1


def test_known_disagreements_observed_first_without_dropping_others():
    from magika_datasets.pipeline import observation_priority

    formats = {r["format_id"]: r for r in rows()}
    ordinary = {"sha256": "a" * 64, "origins": []}
    conflict = {"sha256": "z" * 64, "origins": [{"provider": "virustotal", "claim": markings()}]}
    assert sorted([ordinary, conflict], key=lambda r: observation_priority(r, formats)) == [
        conflict,
        ordinary,
    ]
    assert all(c["budget"] == 100 for c in conflict_plan(rows(), [], [])["classes"].values())


def test_disagreement_collection_precedes_missing_ordinary_collection(tmp_path, monkeypatch):
    from magika_datasets import pipeline

    root = tmp_path / "run"
    root.mkdir()
    store = tmp_path / "store"
    config = tmp_path / "collection.json"
    write(config, {"store": str(store), "vt_recipes": []})
    write(
        root / "config.json",
        {
            "cwd": str(tmp_path),
            "collection_run": str(tmp_path / "ordinary"),
            "collection_config": str(config),
            "metadata_dir": str(tmp_path),
        },
    )
    (root / "baseline").mkdir()
    write(root / "baseline/complete.json", {})
    write(root / "baseline/corpus-parquet-receipt.json", {})
    monkeypatch.setattr(pipeline.os, "chdir", lambda _: None)
    monkeypatch.setattr(pipeline.os, "nice", lambda _: None)
    monkeypatch.setattr(pipeline, "check_sources", lambda *_: None)
    monkeypatch.setattr(pipeline, "taxonomy", lambda _: rows())
    monkeypatch.setattr(pipeline, "acquired_records", lambda _: [])
    monkeypatch.setattr(
        pipeline.subprocess, "run", lambda *a, **kw: pytest.fail("ordinary collection ran first")
    )

    class FirstStage:
        def __init__(self, root, config, plan, search):
            assert all(c["budget"] == 100 for c in plan["classes"].values())
            assert search.max_pages == 20

        def run(self):
            raise RuntimeError("disagreement stage first")

    monkeypatch.setattr(pipeline, "Runner", FirstStage)
    with pytest.raises(RuntimeError, match="disagreement stage first"):
        pipeline.run(root)
