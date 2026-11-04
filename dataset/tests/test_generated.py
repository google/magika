# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Generated samples are reproducible from their origin, and nothing else names them."""

import hashlib
import string

import pytest

from magika_datasets import generated


def test_generation_is_deterministic_and_varied():
    for kind in generated.GENERATORS:
        assert generated.generate(kind, 3) == generated.generate(kind, 3)
    samples = {generated.generate("randombytes", i) for i in range(20)}
    assert len(samples) == 20
    sizes = [len(s) for s in samples]
    assert min(sizes) >= generated.MIN_SIZE and max(sizes) <= generated.MAX_SIZE


def test_each_generator_makes_what_its_class_names():
    assert generated.generate("empty", 0) == b""
    ascii_sample = generated.generate("randomascii", 1)
    assert ascii_sample and all(0x20 <= b <= 0x7E for b in ascii_sample)
    text = generated.generate("randomtxt", 1).decode("ascii")
    assert set(text) <= set(string.ascii_letters + " .,\n") and " " in text
    link = generated.generate("symlinktext", 1)
    assert link and b"\n" not in link and b"\0" not in link


def test_an_origin_names_its_class_only_when_the_bytes_regenerate():
    value, data = generated.origin("randomtxt", 7)
    assert generated.identity(value) == "randomtxt"
    assert value.endswith(hashlib.sha256(data).hexdigest())
    forged = value[:-64] + "0" * 64
    assert generated.identity(forged) is None
    assert generated.identity(value.replace(":v1:", ":v2:")) is None
    assert generated.identity("generated:png:v1:0:" + "0" * 64) is None


def test_unknown_generators_are_refused():
    with pytest.raises(ValueError):
        generated.generate("directory", 0)


def test_refill_admits_generated_samples_labelled_by_their_origin(corpus):
    import json

    import pyarrow as pa
    import pyarrow.parquet as pq

    from magika_datasets.generated_leads import Reader, plan
    from magika_datasets.parquet_metadata import CLASS_SCHEMA
    from magika_datasets.refill import refill

    metadata, store, _ = corpus
    rows = pq.read_table(metadata / "classes.parquet").to_pylist()
    for name, target in (("randomtxt", 3), ("empty", 5)):
        rows.append(
            {
                "ordinal": len(rows),
                "format_id": name,
                "name": name,
                "categories": ["special"],
                "extensions": [],
                "metadata_json": json.dumps({"counts": {"target": target}}),
            }
        )
    pq.write_table(pa.Table.from_pylist(rows, CLASS_SCHEMA), metadata / "classes.parquet")
    leads = plan(metadata)
    assert [(lead["format_id"], lead["index"]) for lead in leads] == [
        ("empty", 0),
        ("randomtxt", 0),
        ("randomtxt", 1),
        ("randomtxt", 2),
    ]  # every empty file is the same file, so one lead however large the target
    receipt = refill(metadata, store, reader=Reader(), leads=leads)
    assert receipt["added_by_class"] == {"empty": 1, "randomtxt": 3}
    added = [
        r
        for r in pq.read_table(metadata / "samples.parquet").to_pylist()
        if r["format_id"] in ("empty", "randomtxt")
    ]
    assert {r["label_status"] for r in added} == {"validated_origin"}
    assert all(r["origins"][0].startswith("generated:") for r in added)
    assert plan(metadata) == []  # held now; the empty class can take no second sample
