import json

import pyarrow as pa
import pyarrow.parquet as pq
import pytest

from magika_datasets.stats import main, render, summarize


@pytest.fixture
def corpus(tmp_path):
    classes = [
        {
            "format_id": k,
            "name": k.upper(),
            "categories": cats,
            "metadata_json": json.dumps({"counts": {"target": target}}),
        }
        for k, cats, target in [
            ("a", ["text", "code"], 2),
            ("b", ["text"], 2),
            ("c", ["binary"], 1),
        ]
    ]
    gh = "github:https://github.com/owner/repo/blob/" + "a" * 40 + "/file.txt:" + "1" * 64
    samples = [
        {
            "format_id": "a",
            "sha256": bytes.fromhex("1" * 64),
            "size": 10,
            "origins": [gh, "vt:" + "1" * 64],
            "hard_case": True,
            "label_status": "validated_auto",
        },
        {
            "format_id": "a",
            "sha256": bytes.fromhex("2" * 64),
            "size": 20,
            "origins": ["vt:" + "2" * 64],
            "hard_case": False,
            "label_status": "validated_origin",
        },
        {
            "format_id": "b",
            "sha256": bytes.fromhex("3" * 64),
            "size": 30,
            "origins": [gh],
            "hard_case": False,
            "label_status": "need_review",
        },
    ]
    pq.write_table(pa.Table.from_pylist(classes), tmp_path / "classes.parquet")
    pq.write_table(pa.Table.from_pylist(samples), tmp_path / "samples.parquet")
    return tmp_path


def test_summary_counts_sources_and_overlapping_categories(corpus):
    result = summarize(corpus, label_statuses=["all"])
    s = result["summary"]
    assert (s["samples"], s["target"], s["missing"], s["bytes"]) == (3, 5, 2, 60)
    assert (s["full_classes"], s["partial_classes"], s["empty_classes"]) == (1, 1, 1)
    assert s["hard_cases"] == 1 and s["github_repositories"] == 1
    assert s["sources"] == {"both": 1, "github_only": 1, "vt_only": 1, "other": 0}
    assert result["categories"]["text"]["samples"] == 3
    assert result["categories"]["code"]["samples"] == 2
    assert result["payload_bytes_read"] == 0


def test_empty_class_and_exact_filter(corpus):
    result = summarize(corpus, ["c"])
    assert result["summary"]["samples"] == 0 and result["summary"]["missing"] == 1
    assert [r["format_id"] for r in result["classes"]] == ["c"]
    with pytest.raises(ValueError, match="Unknown filetype"):
        summarize(corpus, ["typo"])


def test_json_cli_is_read_only_and_includes_all_classes(corpus, capsys):
    before = {p.name: (p.read_bytes(), p.stat().st_mtime_ns) for p in corpus.iterdir()}
    main(["--metadata-dir", str(corpus), "--json"])
    result = json.loads(capsys.readouterr().out)
    assert len(result["classes"]) == 3
    assert before == {p.name: (p.read_bytes(), p.stat().st_mtime_ns) for p in corpus.iterdir()}


def test_duplicate_sample_is_reported(corpus):
    p = corpus / "samples.parquet"
    table = pq.read_table(p)
    pq.write_table(pa.concat_tables([table, table.slice(0, 1)]), p)
    with pytest.raises(ValueError, match="Duplicate SHA"):
        summarize(corpus)


def test_unverified_samples_do_not_inflate_counts(corpus):
    report = summarize(corpus)
    assert report["summary"]["samples"] == 2, "the need_review sample is held, not counted"
    assert report["summary"]["label_status_counts"] == {
        "need_review": 1,
        "validated_auto": 1,
        "validated_origin": 1,
    }
    held = {r["format_id"]: sum(r["label_status_counts"].values()) for r in report["classes"]}
    assert held == {"a": 2, "b": 1, "c": 0}
    text = render(report)
    assert "Counting validated_manual" in text and "All statuses:" in text


def test_counting_every_status_includes_the_unverified(corpus):
    report = summarize(corpus, label_statuses=["all"])
    assert report["summary"]["samples"] == 3
    assert report["counted_label_statuses"][0] == "conflicting"


def test_an_unknown_label_status_is_refused(corpus):
    with pytest.raises(ValueError, match="Unknown label status"):
        summarize(corpus, label_statuses=["accepted"])
