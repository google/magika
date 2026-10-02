# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Every GitHub repository the corpus draws from is published with its licence."""

import json

import pyarrow as pa
import pyarrow.parquet as pq
import pytest

from magika_datasets.licenses import RESIDUAL, markdown, readme_block, resolve

POLICY = {"allowed_spdx": ["MIT", "Apache-2.0"]}


def sources(tmp_path, *entries):
    from magika_datasets.sources import SCHEMA

    rows = [
        {
            "repository": name,
            "url": "https://github.com/" + name,
            "revisions": ["a" * 40],
            "samples": samples,
            "license_observations_json": json.dumps(observations),
        }
        for name, samples, observations in entries
    ]
    path = tmp_path / "github-used.parquet"
    pq.write_table(pa.Table.from_pylist(rows, SCHEMA), path)
    return path


def inventory(spdx, revision="b" * 40):
    return {
        "spdx_id": spdx,
        "license_permalink": f"https://github.com/x/y/blob/{revision}/LICENSE",
        "revision": revision,
        "basis": "frozen_repository_inventory; may differ from sample revision",
    }


def annotated(spdx):
    return {"spdx_id": spdx, "permalink": "https://github.com/x/y/blob/c/LICENSE", "source": "s"}


def test_the_pinned_inventory_resolves_a_licence(tmp_path):
    rows, receipt = resolve(sources(tmp_path, ("a/b", 5, [inventory("MIT")])), POLICY)
    assert rows[0]["repository"] == "a/b"
    assert rows[0]["spdx_id"] == "MIT"
    assert rows[0]["basis"] == "frozen_repository_inventory"
    assert rows[0]["allowed_by_policy"] is True
    assert rows[0]["samples"] == 5
    assert receipt["resolved"] == 1 and receipt["unresolved"] == 0


def test_a_repository_with_no_evidence_stays_unresolved(tmp_path):
    rows, receipt = resolve(sources(tmp_path, ("a/b", 1, [])), POLICY)
    assert rows[0]["spdx_id"] is None
    assert rows[0]["basis"] == "unresolved"
    assert rows[0]["allowed_by_policy"] is False
    assert receipt["unresolved"] == 1
    assert receipt["basis_counts"]["unresolved"] == 1


def test_noassertion_does_not_close_the_question(tmp_path):
    path = sources(tmp_path, ("a/b", 1, [inventory("NOASSERTION"), annotated("MIT")]))
    rows, receipt = resolve(path, POLICY)
    assert (rows[0]["spdx_id"], rows[0]["basis"]) == ("MIT", "sample_annotation")
    assert receipt["annotation_conflicts"] == []


def test_noassertion_alone_stays_unresolved(tmp_path):
    rows, _ = resolve(sources(tmp_path, ("a/b", 1, [inventory("NOASSERTION")])), POLICY)
    assert rows[0]["spdx_id"] is None and rows[0]["basis"] == "unresolved"


def test_a_licence_outside_the_policy_is_reported_not_hidden(tmp_path):
    rows, _ = resolve(sources(tmp_path, ("a/b", 1, [inventory("GPL-3.0")])), POLICY)
    assert rows[0]["spdx_id"] == "GPL-3.0"
    assert rows[0]["allowed_by_policy"] is False


def test_a_sample_annotation_agreeing_is_recorded_as_a_cross_check(tmp_path):
    path = sources(tmp_path, ("a/b", 1, [inventory("MIT"), annotated("MIT")]))
    _, receipt = resolve(path, POLICY)
    assert receipt["annotation_agreements"] == 1
    assert receipt["annotation_conflicts"] == []


def test_a_sample_annotation_disagreeing_is_surfaced(tmp_path):
    path = sources(tmp_path, ("a/b", 1, [inventory("MIT"), annotated("Apache-2.0")]))
    rows, receipt = resolve(path, POLICY)
    assert rows[0]["spdx_id"] == "MIT", "the pinned inventory wins"
    assert receipt["annotation_conflicts"] == [
        {"repository": "a/b", "inventory": "MIT", "annotation": "Apache-2.0"}
    ]


def test_an_annotation_alone_still_resolves(tmp_path):
    rows, _ = resolve(sources(tmp_path, ("a/b", 1, [annotated("ISC")])), POLICY)
    assert (rows[0]["spdx_id"], rows[0]["basis"]) == ("ISC", "sample_annotation")


def test_resolving_the_residual_is_opt_in(tmp_path):
    path = sources(tmp_path, ("a/b", 1, []), ("c/d", 2, [inventory("MIT")]))
    asked = []

    class Client:
        def license(self, repository):
            asked.append(repository)
            return {"spdx_id": "Zlib", "name": "zlib", "permalink": "https://x/LICENSE"}

    rows, _ = resolve(path, POLICY)
    assert asked == [], "no client means no network"
    rows, receipt = resolve(path, POLICY, client=Client())
    assert asked == ["a/b"], "only the unresolved repository is queried"
    assert {r["repository"]: r["basis"] for r in rows} == {
        "a/b": RESIDUAL,
        "c/d": "frozen_repository_inventory",
    }
    assert receipt["unresolved"] == 0


def test_a_repository_the_api_cannot_answer_for_stays_unresolved(tmp_path):
    class Missing:
        def license(self, repository):
            return None

    rows, receipt = resolve(sources(tmp_path, ("a/b", 1, [])), POLICY, client=Missing())
    assert rows[0]["basis"] == "unresolved" and rows[0]["spdx_id"] is None
    assert receipt["unresolved"] == 1


def test_the_markdown_groups_by_licence(tmp_path):
    rows, _ = resolve(
        sources(tmp_path, ("z/z", 1, [inventory("MIT")]), ("a/a", 2, [inventory("Apache-2.0")])),
        POLICY,
    )
    text = markdown(rows)
    assert "## Apache-2.0" in text and "## MIT" in text
    assert text.index("## Apache-2.0") < text.index("## MIT"), "grouped, sorted"
    assert "| [a/a](https://github.com/a/a) | 2 |" in text


def test_the_readme_block_summarizes_by_licence(tmp_path):
    rows, _ = resolve(
        sources(tmp_path, ("z/z", 1, [inventory("MIT")]), ("a/a", 2, [inventory("MIT")])),
        POLICY,
    )
    block = readme_block(rows)
    assert "| MIT | 2 | 3 | yes |" in block


def test_the_readme_block_is_replaced_in_place(tmp_path):
    from magika_datasets.licenses import update_readme

    readme = tmp_path / "README.md"
    readme.write_text("before\n<!-- BEGIN licenses -->\nstale\n<!-- END licenses -->\nafter\n")
    rows, _ = resolve(sources(tmp_path, ("a/a", 2, [inventory("MIT")])), POLICY)
    assert update_readme(readme, rows) is True
    text = readme.read_text()
    assert text.startswith("before\n") and text.endswith("after\n")
    assert "stale" not in text and "| MIT |" in text
    assert update_readme(readme, rows) is False, "a current block is left alone"


def test_a_readme_without_markers_is_refused(tmp_path):
    from magika_datasets.licenses import update_readme

    readme = tmp_path / "README.md"
    readme.write_text("no markers here\n")
    rows, _ = resolve(sources(tmp_path, ("a/a", 1, [inventory("MIT")])), POLICY)
    with pytest.raises(ValueError, match="BEGIN licenses"):
        update_readme(readme, rows)


def test_the_published_table_is_bound_to_its_inputs_by_the_receipt(tmp_path):
    from magika_datasets.acquisition import sha256_file
    from magika_datasets.licenses import main

    path = sources(tmp_path, ("a/b", 1, [inventory("MIT")]))
    policy = tmp_path / "policy.json"
    policy.write_text(json.dumps(POLICY))
    output, receipt = tmp_path / "licenses.parquet", tmp_path / "receipt.json"
    main(
        [
            "--sources",
            str(path),
            "--policy",
            str(policy),
            "--output",
            str(output),
            "--receipt",
            str(receipt),
            "--markdown",
            str(tmp_path / "REPOSITORIES.md"),
            "--readme",
            str(tmp_path / "absent.md"),
        ]
    )
    recorded = json.loads(receipt.read_text())
    assert recorded["sources_sha256"] == sha256_file(path)
    assert recorded["policy_sha256"] == sha256_file(policy)
    assert recorded["parquet_sha256"] == sha256_file(output)
