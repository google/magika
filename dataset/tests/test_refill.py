# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""A refill may only draw from repositories the licence policy permits."""

import hashlib
import json

import ppdeep
import pyarrow as pa
import pyarrow.parquet as pq
import pytest

from magika_datasets.github_leads import plan
from magika_datasets.licenses import SCHEMA as LICENCE_SCHEMA
from magika_datasets.refill import refill

INVENTORY = pa.schema(
    [
        ("repository", pa.string()),
        ("revision", pa.string()),
        ("spdx_id", pa.string()),
        ("path", pa.string()),
        ("size", pa.uint64()),
        ("extension", pa.string()),
        ("git_oid", pa.string()),
        ("git_hash_algorithm", pa.string()),
        ("regular_file", pa.bool_()),
    ]
)
REV = "a" * 40


def blob_oid(content):
    return hashlib.sha1(b"blob %d\0" % len(content) + content).hexdigest()


def inventory(path, *entries):
    rows = [
        {
            "repository": repo,
            "revision": REV,
            "spdx_id": spdx,
            "path": name,
            "size": len(content),
            # The inventory records an empty extension for a file with no suffix.
            "extension": name.rsplit(".", 1)[-1] if "." in name.lstrip(".") else "",
            "git_oid": blob_oid(content),
            "git_hash_algorithm": "sha1",
            "regular_file": regular,
        }
        for repo, spdx, name, content, regular in entries
    ]
    pq.write_table(pa.Table.from_pylist(rows, INVENTORY), path)


def licences(path, *entries):
    rows = [
        {
            "repository": name,
            "url": "https://github.com/" + name,
            "spdx_id": spdx,
            "license_name": None,
            "license_permalink": None,
            "revision": REV,
            "basis": "frozen_repository_inventory",
            "samples": 0,
            "allowed_by_policy": allowed,
        }
        for name, spdx, allowed in entries
    ]
    pq.write_table(pa.Table.from_pylist(rows, LICENCE_SCHEMA), path)


def _png(seed=0):
    """A real 1x1 PNG, so the png validator proves it rather than refuting it."""
    import struct
    import zlib

    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

    header = struct.pack(">IIBBBBB", 1, 1, 8, 2, 0, 0, 0)
    pixels = zlib.compress(b"\x00" + bytes([seed % 256, 0, 0]))
    return (
        b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", pixels) + chunk(b"IEND", b"")
    )


PNG = _png(0)


class Reader:
    """Serves the bytes the inventory describes, verifying nothing beyond identity."""

    def __init__(self, contents):
        self.contents = contents
        self.asked = []

    def read_into(self, fixture, output):
        self.asked.append(fixture["permalink"])
        content = self.contents[fixture["path"]]
        output.write(content)
        return hashlib.sha256(content).hexdigest(), False


@pytest.fixture
def tree(corpus, tmp_path):
    metadata, store, _ = corpus
    (metadata / "config").mkdir(exist_ok=True)
    (metadata / "config/repository-license-policy.json").write_text(
        json.dumps({"allowed_spdx": ["MIT", "Apache-2.0"]})
    )
    inventory(
        metadata / "repository-files.parquet",
        ("ok/permissive", "MIT", "a.png", PNG, True),
        ("ok/permissive", "MIT", "b.png", _png(1), True),
        ("bad/copyleft", "GPL-3.0", "c.png", _png(2), True),
        ("gone/unresolved", None, "d.png", _png(3), True),
        ("ok/permissive", "MIT", "dir", b"", False),
    )
    licences(
        metadata / "repository-licenses.parquet",
        ("ok/permissive", "MIT", True),
        ("bad/copyleft", "GPL-3.0", False),
        ("gone/unresolved", None, False),
    )
    return metadata, store


def test_only_permitted_repositories_supply_leads(tree):
    metadata, _ = tree
    leads = plan(metadata, per_class=10)
    assert {lead["repository"] for lead in leads} == {"ok/permissive"}


def test_a_directory_entry_is_not_a_lead(tree):
    metadata, _ = tree
    assert all(lead["path"] != "dir" for lead in plan(metadata, per_class=10))


def test_the_per_repository_cap_holds(tree):
    metadata, _ = tree
    assert len(plan(metadata, per_class=10, max_per_repository=1)) == 1


def test_a_lead_over_the_size_budget_is_skipped(tree):
    metadata, _ = tree
    assert plan(metadata, per_class=10, max_file_bytes=1) == []


def test_planning_is_deterministic(tree):
    metadata, _ = tree
    assert plan(metadata, per_class=10) == plan(metadata, per_class=10)


def test_refill_adds_samples_with_their_licence_recorded(tree):
    metadata, store = tree
    contents = {"a.png": PNG, "b.png": _png(1)}
    receipt = refill(metadata, store, reader=Reader(contents), per_class=10)
    assert receipt["added"] == 2
    rows = pq.read_table(metadata / "samples.parquet").to_pylist()
    fresh = [
        r
        for r in rows
        if r["sha256"] in {hashlib.sha256(PNG).digest(), hashlib.sha256(_png(1)).digest()}
    ]
    assert len(fresh) == 2
    for row in fresh:
        annotation = json.loads(row["annotation_json"])
        assert annotation["repository_licenses"][0]["spdx_id"] == "MIT"
        assert row["origins"][0].startswith("github:https://github.com/ok/permissive/blob/")
        # Near-duplicate exclusion needs a fingerprint of the bytes actually held.
        stored = store / "objects" / row["sha256"].hex()[:2] / row["sha256"].hex()
        assert annotation["content_fingerprint"]["ssdeep"] == ppdeep.hash_from_file(str(stored))


def test_a_refilled_sample_is_labelled_by_evidence(tree):
    metadata, store = tree
    receipt = refill(metadata, store, reader=Reader({"a.png": PNG, "b.png": _png(1)}), per_class=10)
    assert set(receipt["label_status_counts"]) <= {
        "validated_auto",
        "validated_origin",
        "validated_tools",
        "conflicting",
        "need_review",
    }


def test_a_lead_already_held_is_not_added_twice(tree):
    metadata, store = tree
    reader = Reader({"a.png": PNG, "b.png": _png(1)})
    refill(metadata, store, reader=reader, per_class=10)
    again = refill(metadata, store, reader=Reader({"a.png": PNG, "b.png": _png(1)}), per_class=10)
    assert again["added"] == 0


def test_a_conventional_filename_supplies_a_lead(corpus, tmp_path):
    """Makefile and friends carry no suffix, so an extension-only search never finds them."""
    metadata, _, _ = corpus
    (metadata / "config").mkdir(exist_ok=True)
    (metadata / "config/repository-license-policy.json").write_text(
        json.dumps({"allowed_spdx": ["MIT"]})
    )
    classes = pq.read_table(metadata / "classes.parquet").to_pylist()
    classes.append(
        {
            "ordinal": 3,
            "format_id": "makefile",
            "name": "Makefile",
            "categories": [],
            "extensions": [],
            "metadata_json": '{"counts": {"target": 5}, "class_role": "format"}',
        }
    )
    from magika_datasets.parquet_metadata import CLASS_SCHEMA

    pq.write_table(pa.Table.from_pylist(classes, CLASS_SCHEMA), metadata / "classes.parquet")
    inventory(
        metadata / "repository-files.parquet",
        ("ok/permissive", "MIT", "Makefile", b"all:\n\techo hi\n", True),
    )
    licences(metadata / "repository-licenses.parquet", ("ok/permissive", "MIT", True))
    leads = plan(metadata, per_class=5)
    assert [lead["format_id"] for lead in leads] == ["makefile"]


def test_identical_content_is_one_lead(tree):
    """Boilerplate copied across repositories must not consume several fetches."""
    metadata, _ = tree
    inventory(
        metadata / "repository-files.parquet",
        ("ok/permissive", "MIT", "a.png", PNG, True),
        ("ok/permissive", "MIT", "copy/a.png", PNG, True),
    )
    assert len(plan(metadata, per_class=10)) == 1


def test_the_repository_cap_counts_samples_already_held(tree):
    """Across passes, one repository must not keep adding ten more to the same class."""
    metadata, _ = tree
    held = pq.read_table(metadata / "samples.parquet").to_pylist()
    from magika_datasets.parquet_metadata import SAMPLE_SCHEMA

    template = held[0]
    for i in range(3):
        digest = bytes([200 + i]) + b"\0" * 31
        held.append(
            {
                **template,
                "sample_ordinal": len(held),
                "sha256": digest,
                "origins": [
                    f"github:https://github.com/ok/permissive/blob/{REV}/held{i}.png:{digest.hex()}"
                ],
            }
        )
    held.sort(key=lambda r: (r["class_ordinal"], r["sha256"]))
    counters = {}
    for row in held:
        row["sample_ordinal"] = counters.setdefault(row["class_ordinal"], 0)
        counters[row["class_ordinal"]] += 1
    pq.write_table(pa.Table.from_pylist(held, SAMPLE_SCHEMA), metadata / "samples.parquet")
    assert plan(metadata, per_class=10, max_per_repository=3) == []


class FakeVirusTotal:
    """Search pages and downloads, served from a dict of content."""

    def __init__(self, contents, pages):
        self.contents, self.pages, self.searched, self.fetched = contents, pages, [], []

    def search(self, query, *, limit, cursor=None):
        self.searched.append((query, cursor))
        fixtures, next_cursor = self.pages.get((query, cursor), ([], None))
        return {"fixtures": fixtures[:limit], "next_cursor": next_cursor}

    def read_into(self, fixture, output):
        self.fetched.append(fixture["sha256"])
        output.write(self.contents[fixture["sha256"]])
        return fixture["sha256"], False


def vt_fixture(content, query):
    digest = hashlib.sha256(content).hexdigest()
    return {
        "provider": "virustotal",
        "sha256": digest,
        "size": len(content),
        "query": query,
        "claim": {"magika": "png"},
    }


@pytest.fixture
def vt_tree(corpus):
    metadata, store, _ = corpus
    (metadata / "config").mkdir(exist_ok=True)
    (metadata / "config/vt-queries.json").write_text(
        json.dumps({"queries": [{"name": "png", "query": "name:*.png"}]})
    )
    return metadata, store


def test_virustotal_is_searched_by_label_then_by_saved_query(vt_tree):
    """Type tag first, saved filename query after, following continuation pages."""
    from magika_datasets.virustotal_leads import plan_virustotal

    metadata, _ = vt_tree
    one, two = _png(10), _png(11)
    client = FakeVirusTotal(
        {},
        {
            ("type:png size:8MB-", None): ([vt_fixture(one, "type:png")], "page2"),
            ("type:png size:8MB-", "page2"): ([], None),
            ("name:*.png size:8MB-", None): ([vt_fixture(two, "name:*.png")], None),
        },
    )
    leads = plan_virustotal(metadata, client, per_class=2, oversample=1)
    assert [lead["query"] for lead in leads] == ["type:png", "name:*.png"]
    assert client.searched[0] == ("type:png size:8MB-", None)


def test_a_conflicting_vt_label_is_still_fetched_as_a_hard_case(vt_tree):
    """VirusTotal's label is a hint; the validator decides, disagreement included."""
    from magika_datasets.virustotal_leads import plan_virustotal

    metadata, _ = vt_tree
    odd = {**vt_fixture(_png(14), "type:png"), "claim": {"magika": "pdf"}}
    client = FakeVirusTotal({}, {("type:png size:8MB-", None): ([odd], None)})
    leads = plan_virustotal(metadata, client, per_class=1, oversample=1)
    assert len(leads) == 1


def test_a_virustotal_sample_is_recorded_as_such(vt_tree):
    metadata, store = vt_tree
    content = _png(12)
    lead = {**vt_fixture(content, "name:*.png"), "format_id": "png"}
    client = FakeVirusTotal({lead["sha256"]: content}, {})
    receipt = refill(metadata, store, reader=client, leads=[lead])
    assert receipt["added"] == 1
    row = next(
        r
        for r in pq.read_table(metadata / "samples.parquet").to_pylist()
        if r["sha256"].hex() == lead["sha256"]
    )
    assert row["origins"] == ["vt:" + lead["sha256"]]
    annotation = json.loads(row["annotation_json"])
    assert annotation["discovery_queries"] == ["name:*.png"]
    assert "repository_licenses" not in annotation


def test_a_full_class_is_not_downloaded_again(vt_tree):
    metadata, store = vt_tree
    rows = pq.read_table(metadata / "classes.parquet").to_pylist()
    for row in rows:
        if row["format_id"] == "png":
            row["metadata_json"] = json.dumps({"counts": {"target": 1}})
    from magika_datasets.parquet_metadata import CLASS_SCHEMA

    pq.write_table(pa.Table.from_pylist(rows, CLASS_SCHEMA), metadata / "classes.parquet")
    # Only a verified sample counts towards a target, so the held png must be one.
    samples = pq.read_table(metadata / "samples.parquet").to_pylist()
    for row in samples:
        if row["format_id"] == "png":
            row["label_status"] = "validated_auto"
    from magika_datasets.parquet_metadata import SAMPLE_SCHEMA

    pq.write_table(pa.Table.from_pylist(samples, SAMPLE_SCHEMA), metadata / "samples.parquet")
    content = _png(13)
    lead = {**vt_fixture(content, "name:*.png"), "format_id": "png"}
    client = FakeVirusTotal({lead["sha256"]: content}, {})
    receipt = refill(metadata, store, reader=client, leads=[lead])
    assert client.fetched == [], "png already holds its one-sample target"
    assert receipt["not_added"] == {"class_full": 1}


def test_streaming_fills_a_class_without_planning_everything_first(vt_tree):
    from magika_datasets.virustotal_leads import stream_virustotal

    metadata, store = vt_tree
    contents = {hashlib.sha256(_png(20 + i)).hexdigest(): _png(20 + i) for i in range(3)}
    fixtures = [vt_fixture(c, "type:png") for c in contents.values()]

    def factory():
        return FakeVirusTotal(contents, {("type:png size:8MB-", None): (fixtures, None)})

    receipt = refill(
        metadata,
        store,
        leads=stream_virustotal(metadata, store, factory, formats=["png"], workers=2),
    )
    assert receipt["added"] == 3


def _textured_png(variant=0):
    """A PNG with enough varied content for an informative ssdeep signature."""
    import random
    import struct
    import zlib

    rng = random.Random(7)
    text = bytes(rng.randrange(32, 127) for _ in range(16000))
    text = text[:8000] + bytes([65 + variant]) + text[8001:]

    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

    header = struct.pack(">IIBBBBB", 1, 1, 8, 2, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"tEXt", b"Comment\0" + text)
        + chunk(b"IDAT", zlib.compress(b"\x00\x00\x00\x00"))
        + chunk(b"IEND", b"")
    )


def test_a_near_duplicate_does_not_count_towards_its_class(tree):
    metadata, store = tree
    first, second = _textured_png(0), _textured_png(1)
    inventory(
        metadata / "repository-files.parquet",
        ("ok/permissive", "MIT", "a.png", first, True),
        ("ok/permissive", "MIT", "b.png", second, True),
    )
    receipt = refill(
        metadata, store, reader=Reader({"a.png": first, "b.png": second}), per_class=10
    )
    assert receipt["added"] == 1
    assert receipt["not_added"] == {"near_duplicate": 1}
