# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import hashlib
import json
import os
import struct
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq
import pytest
from magika_rules_benchmark.corpus import file_hash


@pytest.fixture(scope="module")
def reviewed_binary_headers():
    """Header structures from the format readers, plus independently invalid fields."""
    dex = bytearray(112)
    dex[:8] = b"dex\n035\0"
    struct.pack_into("<III", dex, 32, 112, 112, 0x12345678)
    cases = [
        ("cram", b"CRAM\x03\0" + bytes(20), 26, [(4, b"\0"), (5, b"\xff")]),
        ("dex", bytes(dex), 112, [(7, b"!"), (6, b"x"), (36, bytes(4)), (40, bytes(4))]),
        ("redis_rdb", b"REDIS0009\xff" + bytes(8), 9, [(5, b"x009"), (5, b"0000")]),
        ("lz", b"LZIP\x01\xce" + bytes(30), 8, [(4, b"\x02"), (5, b"\0"), (5, b"\xfe")]),
        ("rzip", b"RZIP\x02\x01" + bytes(18), 24, [(4, b"\0"), (14, b"\x01")]),
        (
            "xar",
            struct.pack(">4sHHQQI", b"xar!", 28, 1, 8, 8, 0) + bytes(8),
            28,
            [(8, bytes(8)), (16, bytes(8))],
        ),
        (
            "spirv",
            struct.pack("<5I", 0x07230203, 0x00010000, 0, 1, 0),
            20,
            [(4, bytes(4)), (12, bytes(4)), (16, b"\x01")],
        ),
        (
            "icns",
            struct.pack(">4sI4sI", b"icns", 16, b"TOC ", 8),
            16,
            [(4, bytes(4)), (12, (7).to_bytes(4, "big"))],
        ),
    ]
    result = []
    for label, header, minimum, mutations in cases:
        invalid = []
        for offset, value in mutations:
            changed = bytearray(header)
            changed[offset : offset + len(value)] = value
            invalid.append(bytes(changed))
        result.append((label, header, minimum, invalid))
    return result


@pytest.fixture(scope="module")
def reviewed_binary_header_variants():
    variants = []
    for endian in ("<", ">"):
        for version, size in ((b"035", 112), (b"041", 120)):
            dex = bytearray(size)
            dex[:8] = b"dex\n" + version + b"\0"
            struct.pack_into(endian + "III", dex, 32, size, size, 0x12345678)
            variants.append(("dex", bytes(dex)))
        variants.append(("spirv", struct.pack(endian + "5I", 0x07230203, 0x00010600, 0, 1, 0)))
    variants.extend(
        [
            ("xar", struct.pack(">4sHHQQI", b"xar!", 1, 28, 8, 8, 0) + bytes(8)),
            ("xar", struct.pack(">4sHHQQI", b"xar!", 32, 1, 8, 8, 1) + bytes(12)),
            ("icns", struct.pack(">4sI", b"icns", 8)),
            ("lz", b"LZIP\x00\x0c" + bytes(30)),
        ]
    )
    return variants


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
