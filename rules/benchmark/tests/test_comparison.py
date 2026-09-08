# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import json

import pytest
from magika_rules_benchmark import comparison as c


def test_mime_mapping_does_not_choose_truth_from_ambiguous_alias():
    classes = {
        "zip": {"mimes": ["application/zip"]},
        "apk": {"mimes": ["application/zip"]},
        "pdf": {"mimes": ["application/pdf"]},
    }
    mapping = c.label_mapping(classes, c.Adapter.FILE)
    rows = c.parse_output(
        c.Adapter.FILE,
        b"application/zip\napplication/pdf\napplication/octet-stream\n",
        ["a", "b", "c"],
        mapping,
    )
    assert [r["prediction"] for r in rows] == [None, "pdf", None]
    assert rows[0]["mapping"] == "ambiguous"
    assert rows[2]["mapping"] == "abstained"


def test_magika_requires_same_paths_and_retains_classification_errors():
    raw = json.dumps({"path": "a", "result": {"status": "permission_denied"}}).encode()
    row = c.parse_output(c.Adapter.MAGIKA, raw, ["a"], {})[0]
    assert row["error"] == "permission_denied" and row["prediction"] is None
    with pytest.raises(ValueError, match="paths"):
        c.parse_output(c.Adapter.MAGIKA, raw, ["b"], {})


def test_trid_uses_only_top_rank_and_abstains_on_ties():
    raw = b"File: a\n 50.0% (.DOC) Word (1/1)\n 50.0% (.ZIP) Zip (1/1)\nFile: b\n 70.0% (.PDF) PDF (1/1)\n 30.0% (.DOC) Word (1/1)\nFile: c\n Unknown!\n"
    rows = c.parse_output(
        c.Adapter.TRID, raw, ["a", "b", "c"], {"doc": ["doc"], "zip": ["zip"], "pdf": ["pdf"]}
    )
    assert [r["prediction"] for r in rows] == [None, "pdf", None]
    assert rows[0]["mapping"] == "ambiguous"


def test_metrics_include_errors_and_abstentions_in_accuracy_denominator():
    samples = [{"truth": "pdf"}, {"truth": "pdf"}, {"truth": "zip"}, {"truth": "zip"}]
    rows = [
        {"prediction": "pdf", "error": None},
        {"prediction": "zip", "error": None},
        {"prediction": None, "error": None},
        {"prediction": None, "error": "failed"},
    ]
    m = c.quality_metrics(samples, rows)
    assert m["accuracy"] == 0.25 and m["decision_coverage"] == 0.5
    assert m["precision"] == 0.5 and m["errors"] == 1
    assert m["macro_recall"] == 0.25


def test_workloads_mean_file_counts_and_exact_rule_ratios():
    samples = [{"sha256": str(i), "size": 1} for i in range(10)]
    cases = c.make_workloads(samples, {str(i) for i in range(5)}, [1, 5, 10], [0, 20, 100], 7)
    assert not any(x["file_count"] == 1 and x["requested_rule_hit_percent"] == 20 for x in cases)
    for case in cases:
        assert len(case["samples"]) == case["file_count"]
        rate = case["requested_rule_hit_percent"]
        if rate is not None:
            assert sum(int(h) < 5 for h in case["samples"]) * 100 == rate * case["file_count"]
    assert cases == c.make_workloads(
        samples, {str(i) for i in range(5)}, [1, 5, 10], [0, 20, 100], 7
    )


def test_history_refuses_different_workloads_or_machine():
    base = {
        "benchmark_version": "1.0.0",
        "compatibility": {"host": "a", "workloads": "b"},
        "measurements": [],
    }
    assert c.compare_previous(base, base)["comparable"]
    changed = base | {"compatibility": {"host": "other", "workloads": "b"}}
    assert not c.compare_previous(base, changed)["comparable"]


def test_hyperfine_json_is_source_of_timing_statistics():
    row = c.timing_summary({"times": [0.01, 0.02, 0.03], "exit_codes": [0, 0, 0]}, 10)
    assert row["median_seconds"] == 0.02
    assert row["files_per_second"] == 500
    with pytest.raises(ValueError, match="exit"):
        c.timing_summary({"times": [0.001], "exit_codes": [1]}, 1)


def test_unknown_adapter_is_rejected():
    with pytest.raises(ValueError):
        c.Adapter("magkia")


def test_file_fat_macho_architecture_lines_belong_to_one_input():
    raw = b"application/x-mach-binary\nfiles/abc (for architecture x86_64):\tapplication/x-mach-binary\nfiles/abc (for architecture arm64):\tapplication/x-mach-binary\napplication/pdf\n"
    rows = c.parse_output(
        c.Adapter.FILE,
        raw,
        ["files/abc", "files/def"],
        {"application/x-mach-binary": ["macho"], "application/pdf": ["pdf"]},
    )
    assert [r["prediction"] for r in rows] == ["macho", "pdf"]


def test_file_nul_frames_keep_nested_architecture_details_with_the_parent():
    raw = b"files/a\0application/x-java-applet\nfiles/a (for architecture cputype (79431682) cpusubtype (278464)):\tapplication/octet-stream\0files/b\0application/pdf\0"
    rows = c.parse_output(
        c.Adapter.FILE,
        raw,
        ["files/a", "files/b"],
        {"application/x-java-applet": ["java"], "application/pdf": ["pdf"]},
    )
    assert [r["prediction"] for r in rows] == ["java", "pdf"]
    with pytest.raises(ValueError, match="paths"):
        c.parse_output(c.Adapter.FILE, raw, ["files/b", "files/a"], {})


def test_quality_reuse_rejects_changed_labels():
    original = {"samples": [{"sha256": "a", "size": 1, "truth": "pdf"}], "classes": {"pdf": {}}}
    changed = original | {"samples": [{"sha256": "a", "size": 1, "truth": "zip"}]}
    assert c.input_identity(original) != c.input_identity(changed)


def test_changed_flags_block_comparison_but_relocated_artifacts_do_not():
    tool = {
        "id": "m2",
        "adapter": "magika-jsonl",
        "command": ["/old/magika", "--threads=2", "--rules-file", "/old/rules"],
        "artifacts": {"rules": "/old/rules"},
    }
    moved = tool | {
        "command": ["/new/magika", "--threads=2", "--rules-file", "/new/rules"],
        "artifacts": {"rules": "/new/rules"},
    }
    assert c.command_settings(tool) == c.command_settings(moved)
    moved["command"][1] = "--threads=8"
    assert c.command_settings(tool) != c.command_settings(moved)


def test_workloads_never_repeat_files_and_share_trid_sort_order():
    samples = [{"sha256": str(i)} for i in range(10)]
    cases = c.make_workloads(samples, {"1", "2"}, [1, 5, 100], [100], 9)
    assert all(case["samples"] == sorted(set(case["samples"])) for case in cases)
    assert all(case["file_count"] <= 10 for case in cases)


def test_render_is_only_a_function_of_saved_json(tmp_path, monkeypatch):
    result = {
        "benchmark_version": "1.0.0",
        "revision": "abc",
        "status": "complete",
        "quality": {},
        "measurements": [],
    }
    saved = tmp_path / "results.json"
    saved.write_text(json.dumps(result))
    monkeypatch.setattr(c, "invoke", lambda *a: pytest.fail("render must not execute tools"))
    output = tmp_path / "report.md"
    c.main(["--render", str(saved), "--output", str(output)])
    assert output.read_text() == c.render(result)


def test_history_derives_quality_deltas_from_json():
    base = {
        "benchmark_version": "1.0.0",
        "compatibility": {},
        "measurements": [],
        "quality": {
            "m2": {
                "accuracy": 0.8,
                "decision_coverage": 0.9,
                "macro_recall": 0.7,
                "rule_hit_percent": 20,
            }
        },
    }
    after = base | {
        "quality": {"m2": base["quality"]["m2"] | {"accuracy": 0.9, "rule_hit_percent": 25}}
    }
    delta = c.compare_previous(after, base)["quality_deltas"][0]
    assert delta["accuracy"] == pytest.approx(0.1)
    assert delta["rule_hit_percent"] == 5


def test_adding_a_tool_preserves_history_for_existing_tools():
    old = {
        "benchmark_version": "1.0.0",
        "compatibility": {"settings": {"m1": "a"}, "mappings": {"m1": "b"}},
        "measurements": [],
    }
    new = old | {
        "compatibility": {"settings": {"m1": "a", "m2": "c"}, "mappings": {"m1": "b", "m2": "d"}}
    }
    assert c.compare_previous(new, old)["comparable"]


def test_rules_only_hits_exclude_abstentions_and_errors_without_ml_control():
    samples = [{"sha256": str(i)} for i in range(4)]
    rows = [
        c.mapped(["pdf"], {"pdf": ["pdf"]}, deterministic=True),
        c.mapped(["unknown"], {}, abstain=True, deterministic=True),
        c.mapped([], {}, error="permission_denied"),
        c.mapped(["future-label"], {}, deterministic=True),
    ]
    assert c.rule_hit_hashes(samples, rows) == {"0", "3"}
    # Rules matched even when the corpus label vocabulary cannot map the output.
    rows[0]["deterministic"] = False
    with pytest.raises(ValueError, match="inference"):
        c.rule_hit_hashes(samples, rows)


def test_hybrid_rule_hits_still_require_the_ml_control():
    samples = [{"sha256": str(i)} for i in range(3)]
    rows = [c.mapped(["pdf"], {"pdf": ["pdf"]}, deterministic=True) for _ in samples]
    control = [dict(deterministic=x) for x in [False, True, False]]
    rows[2] = c.mapped(["unknown"], {}, abstain=True, deterministic=True)
    assert c.rule_hit_hashes(samples, rows, control) == {"0"}
