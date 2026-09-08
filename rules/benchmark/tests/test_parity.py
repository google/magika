# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Compare actual Vectorscan decisions with YARA-X on the same source and bytes."""

import hashlib
import os
import subprocess
from pathlib import Path

import pytest
from magika_rules_benchmark.runner import observe

pytestmark = pytest.mark.native


def rule(name, condition, patterns="", label="png", enabled=True):
    return (
        f'rule {name} {{ meta: label = "{label}" enforced = {str(enabled).lower()} '
        f'class = "full" fp_rate = 0 fn_rate = 0 {patterns} condition: {condition} }}'
    )


def scan(tmp_path, source, payloads):
    pack = tmp_path / "parity.yar"
    pack.write_text(source)
    records = []
    for index, content in enumerate(payloads):
        path = tmp_path / f"input-{index}"
        path.write_bytes(content)
        records.append(
            dict(path=str(path), size=len(content), sha256=hashlib.sha256(content).hexdigest())
        )
    env = dict(os.environ, MAGIKA_RULES_CACHE=str(tmp_path / "cache"))
    rows, _ = observe(
        Path(env["MAGIKA_TEST_BINARY"]), pack, records, {"png": "png", "gif": "gif"}, env
    )
    assert all(row["reference_error"] is None for row in rows)
    assert not any(row["reference_mismatch"] for row in rows), rows
    return rows


def test_reviewed_binary_headers_native(
    tmp_path, reviewed_binary_headers, reviewed_binary_header_variants
):
    root = Path(__file__).resolve().parents[3]
    source = (root / "rules/rulesets/full/formats.yar").read_text()
    payloads, expected = [], []
    for label, header, minimum, invalid in reviewed_binary_headers:
        payloads.extend([header, header[: minimum - 1], *invalid])
        expected.extend([label, None, *([None] * len(invalid))])
    for label, header in reviewed_binary_header_variants:
        payloads.append(header)
        expected.append(label)
    assert [row["rule_prediction"] for row in scan(tmp_path, source, payloads)] == expected


@pytest.mark.parametrize("brand,label", [(b"3gp6", "3gp"), (b"avif", "avif"), (b"heic", "heif")])
def test_ftyp_native_bounds(tmp_path, brand, label):
    root = Path(__file__).resolve().parents[3]
    source = (root / "rules/rulesets/full/formats.yar").read_text()
    header = (20).to_bytes(4, "big") + b"ftyp" + brand + bytes(4) + brand
    payloads = [header, header[:11], header[:15]]
    payloads += [size.to_bytes(4, "big") + header[4:] for size in (1, 12, 17, 0xFFFFFFFF)]
    rows = scan(tmp_path, source, payloads)
    assert [row["rule_prediction"] for row in rows] == [label] + [None] * 6


@pytest.mark.parametrize(
    "condition,patterns,payloads,expected",
    [
        (
            "$a at 2",
            'strings: $a = "ABCD"',
            [b"xxABCD", b"xABCD", b"xxxABCD", b"xxABC"],
            [True, False, False, False],
        ),
        (
            "$a in (2..4)",
            "strings: $a = { 41 (42 | 43) 44 }",
            [b"xxABD", b"xxxxACD", b"xABD", b"xxxxxABD"],
            [True, True, False, False],
        ),
        (
            "$a at 0",
            r"strings: $a = /[\x80-\xff](A|BC){1,2}/",
            [b"\x80A", b"\xffBCBC", b"\x7fA", b"\xffBD"],
            [True, True, False, False],
        ),
        (
            "uint16(0) == 258 and uint16be(2) == 772",
            "",
            [b"\x02\x01\x03\x04", b"\x01\x02\x03\x04", b"\x02\x01\x04\x03", b"\x02\x01\x03"],
            [True, False, False, False],
        ),
        (
            "uint8(0) >= 127 and uint8(0) <= 129",
            "",
            [bytes([n]) + b"x" * 32 for n in (126, 127, 128, 129, 130)],
            [False, True, True, True, False],
        ),
        (
            "original_size >= 4096 and prefix_size == 4096 and original_size % 4 == 0",
            "",
            [b"A" * n for n in (4095, 4096, 4097, 4100)],
            [False, True, False, True],
        ),
        (
            "uint32be(4092) == 1094861636",
            "",
            [b"x" * 4092 + b"ABCD", b"x" * 4092 + b"ABC", b"x" * 4093 + b"ABCD"],
            [True, False, False],
        ),
        (
            "($a at 0 and $b at 2) or ($c at 0 and $d at 2)",
            'strings: $a = "AB" $b = "CD" $c = "EF" $d = "GH"',
            [b"ABCD", b"EFGH", b"ABGH", b"EFCD"],
            [True, True, False, False],
        ),
    ],
    ids=[
        "fixed-offset",
        "bounded-range",
        "binary-regex",
        "endianness",
        "integer-range",
        "original-length",
        "prefix-boundary",
        "correlated-alternatives",
    ],
)
def test_supported_predicates_match_yara_x(tmp_path, condition, patterns, payloads, expected):
    rows = scan(tmp_path, rule("candidate", condition, patterns), payloads)
    assert [row["rule_prediction"] == "png" for row in rows] == expected


@pytest.mark.parametrize(
    "other_label,enabled,expected",
    [
        ("png", True, "png"),
        ("gif", True, None),
        ("gif", False, "png"),
    ],
)
def test_all_matches_agree_or_abstain(tmp_path, other_label, enabled, expected):
    source = rule("first", "uint8(0) == 65") + rule(
        "second", "uint8(0) == 65", label=other_label, enabled=enabled
    )
    rows = scan(tmp_path, source, [b"ABCD", b"BCDE"])
    assert rows[0]["rule_prediction"] == expected
    assert rows[0]["conflict"] == (other_label == "gif" and enabled)
    assert rows[1]["rule_prediction"] is None


def test_private_helper_coordination_and_cached_reload(tmp_path):
    source = "private rule helper { condition: uint8(0) == 65 }" + rule(
        "candidate", "helper and uint8(1) == 66"
    )
    expected = None
    for _ in range(2):
        rows = scan(tmp_path, source, [b"ABCD", b"ACBD", b"BCDE"])
        decisions = [row["rule_prediction"] for row in rows]
        assert decisions == ["png", None, None]
        if expected is not None:
            assert decisions == expected
        expected = decisions


def test_embedded_pack_against_public_fixture_bytes(tmp_path):
    binary = Path(os.environ["MAGIKA_TEST_BINARY"])
    pack = tmp_path / "embedded.yar"
    subprocess.run(
        [str(binary), "--write-default-rules", str(pack)], check=True, capture_output=True
    )
    root = Path(__file__).resolve().parents[3] / "tests_data/basic"
    files = [root / "png/magika_test.png", root / "pdf/magika_test.pdf"]
    assert all(path.is_file() for path in files)
    scan(tmp_path, pack.read_text(), [path.read_bytes() for path in files])
