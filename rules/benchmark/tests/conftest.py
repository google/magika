# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import hashlib
import json
import os
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq
import pytest
from magika_rules_benchmark.corpus import file_hash


def pytest_addoption(parser):
    parser.addoption(
        "--run-native", action="store_true", help="Run actual Magika integration tests"
    )
    parser.addoption(
        "--run-packaging", action="store_true", help="Build staged Rust source packages"
    )


def pytest_collection_modifyitems(config, items):
    for marker, option in (("native", "--run-native"), ("packaging", "--run-packaging")):
        if not config.getoption(option):
            for item in items:
                if marker in item.keywords:
                    item.add_marker(pytest.mark.skip(reason=f"requires explicit {option}"))
    if config.getoption("--run-native"):
        for variable in ("MAGIKA_TEST_BINARY", "MAGIKA_VECTORSCAN_LIBRARY"):
            if not os.environ.get(variable) or not Path(os.environ[variable]).is_file():
                raise pytest.UsageError(f"Native suite requires an existing {variable}")


@pytest.fixture
def corpus_factory(tmp_path):
    def build(contents=(b"PNG sample", b"GIF sample"), labels=("png", "gif"), ambiguous=False):
        root = tmp_path / "dataset"
        (root / "shards").mkdir(parents=True, exist_ok=True)
        classes = [
            dict(
                format_id=name,
                name=name.upper(),
                categories=["image"],
                extensions=[name],
                metadata_json=json.dumps(dict(magika=dict(output_labels=[name], kb_labels=[name]))),
            )
            for name in ("png", "gif", "extra")
        ]
        pq.write_table(pa.Table.from_pylist(classes), root / "classes.parquet")
        rows = []
        for content, label in zip(contents, labels, strict=True):
            rows.append(
                dict(
                    sha256=hashlib.sha256(content).digest(),
                    size=len(content),
                    format_id=label,
                    origins=["generated test fixture"],
                    annotation_json=json.dumps(
                        dict(
                            format_ids=[label],
                            ambiguous=ambiguous,
                            label_status="accepted",
                            properties=dict(
                                artifact_layout=dict(state="known", value="single_file")
                            ),
                        )
                    ),
                    content=content,
                )
            )
        pq.write_table(pa.Table.from_pylist(rows), root / "shards/000.parquet")
        metadata = hashlib.sha256()
        for row in rows:
            encoded = {
                k: v.hex() if isinstance(v, bytes) else v for k, v in row.items() if k != "content"
            }
            metadata.update(
                (json.dumps(encoded, sort_keys=True, separators=(",", ":")) + "\n").encode()
            )
        manifest = dict(
            format="parquet-whole-file-v1",
            samples=len(rows),
            original_bytes=sum(len(c) for c in contents),
            metadata_rows_sha256=metadata.hexdigest(),
            source_metadata_receipt=dict(
                files={"classes.parquet": dict(sha256=file_hash(root / "classes.parquet"))}
            ),
            shards=[
                dict(
                    path="shards/000.parquet",
                    samples=len(rows),
                    sha256=file_hash(root / "shards/000.parquet"),
                )
            ],
        )
        (root / "manifest.json").write_text(json.dumps(manifest))
        return root

    return build


@pytest.fixture
def observations():
    def row(truth, prediction, hit):
        return dict(
            truth=truth,
            ambiguous=False,
            raw_matches=["png_rule"] if hit else [],
            rule_prediction="png" if hit else None,
            ml_prediction=prediction,
            hybrid_prediction="png" if hit else prediction,
            ml_score=0.9,
            hybrid_score=1.0 if hit else 0.9,
            reference_error=None,
            reference_mismatch=False,
            conflict=False,
        )

    return dict(
        samples=[
            row("png", "png", True),
            row("png", "gif", False),
            row("gif", "gif", False),
            row("extra", "extra", True),
        ],
        rules={"png_rule": dict(format_id="png", enforced=True)},
        classes={
            name: dict(categories=["image"], extensions=[name]) for name in ("png", "gif", "extra")
        },
        supported=["png", "gif"],
    )
