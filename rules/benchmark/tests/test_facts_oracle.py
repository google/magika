# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""YARA-X evaluates fact and view conditions when the oracle supplies the globals."""

import hashlib
import json
import os
import re
import subprocess
from collections import Counter
from pathlib import Path

import pytest
import yara_x
from magika_rules_benchmark import preprocess, runner
from magika_rules_benchmark.preprocess import (
    FACT_NAMES,
    VIEW_NAMES,
    ZIP_FLAG_NAMES_TRUNCATED,
    ZIP_NAMES_BYTES,
    facts_for,
    is_active,
)
from preprocess_fixtures import (
    FACTS_CASES,
    LONG_NAMES,
    LONG_NAMES_HELD,
    archive,
    docx_like,
    overflowing_archive,
    pe32,
    pe32plus,
    stored,
)

ROOT = Path(__file__).resolve().parents[3]
RULESETS = sorted((ROOT / "rules/rulesets").rglob("*.yar"))


def rule(condition, name="candidate"):
    return (
        f'rule {name} {{ meta: label = "png" enforced = true class = "full" fp_rate = 0 '
        f"fn_rate = 0 condition: {condition} }}"
    )


def matches(condition, payloads):
    compiler = yara_x.Compiler(includes_enabled=False)
    preprocess.define_globals(compiler)
    compiler.add_source(rule(condition))
    scanner = yara_x.Scanner(compiler.build())
    decisions = []
    for payload in payloads:
        facts, view = facts_for(payload)
        preprocess.set_globals(scanner, facts, view)
        decisions.append(bool(scanner.scan(payload[:4096]).matching_rules))
    return decisions


@pytest.mark.parametrize("condition,payloads,expected", FACTS_CASES.values(), ids=FACTS_CASES)
def test_fact_and_view_conditions_decide_the_shared_cases(condition, payloads, expected):
    assert matches(condition, payloads) == expected


def test_a_name_beyond_the_truncation_boundary_is_absent():
    payload = overflowing_archive()
    facts, view = facts_for(payload)
    assert facts["zip_flags"] == ZIP_FLAG_NAMES_TRUNCATED
    assert facts["zip_names_entries"] == LONG_NAMES_HELD
    assert len(view) == ZIP_NAMES_BYTES + preprocess.ZIP_FIRST_ENTRY_BYTES
    last_held, first_dropped = LONG_NAMES[LONG_NAMES_HELD - 1], LONG_NAMES[LONG_NAMES_HELD]
    present = 'zip_names contains "\\n' + last_held.decode() + '\\n"'
    absent = 'zip_names contains "\\n' + first_dropped.decode() + '\\n"'
    assert matches(present, [payload]) == [True]
    assert matches(absent, [payload]) == [False]


def test_size_facts_are_unchanged():
    condition = "original_size >= 4096 and prefix_size == 4096 and original_size % 4 == 0"
    payloads = [b"A" * n for n in (4095, 4096, 4097, 4100)]
    assert matches(condition, payloads) == [False, True, False, True]


def test_view_bytes_reach_yara_x_verbatim_including_non_ascii_names():
    # A `str` global would be re-encoded to UTF-8 by YARA-X; the oracle passes bytes so
    # that a non-ASCII name matches the same byte literal the native engine sees.
    payload = archive([stored(b"caf\xe9.txt", b"")])
    assert matches('zip_names contains "\\ncaf\\xe9.txt\\n"', [payload]) == [True]
    assert matches('zip_names contains "\\ncaf\\xc3\\xa9.txt\\n"', [payload]) == [False]


# --- the inactive synthetic ---------------------------------------------------------------

# A rule the all-zero facts header satisfies: YARA-X, given zeros for the facts of a family
# that did not fire, matches it on inputs the native engine never scans stream B for.
ZERO_ANCHORED = "pe_is_dll == 0"
REJECTION = (
    "rule `loose` would match inputs no preprocessor touched; "
    "anchor it on a nonzero fact or a view membership"
)


def test_is_active_mirrors_the_native_stream_b_gate():
    # The size facts alone never activate stream B: stream A already carries them.
    for payload in (b"", b"plain text", b"A" * 5000, b"MZ" + bytes(0x80), b"PK\x03\x04 no"):
        facts, view = facts_for(payload)
        assert not is_active(facts, view), payload
    for payload in (docx_like(), pe32().build(), pe32(magic=0x107).build()):
        assert is_active(*facts_for(payload)), payload
    # A stored `mimetype` line alone is a nonempty view, whatever the directory yields.
    prefix = archive([stored(b"mimetype", b"application/epub+zip")])
    facts, view = preprocess.prepare(prefix, len(prefix) + 1, None)
    assert facts["zip_valid"] == 0 and view.startswith(b"\nmimetype=")
    assert is_active(facts, view)


def test_a_zero_anchored_rule_is_what_yara_x_and_native_would_disagree_on():
    payload = b"plain text"
    assert not is_active(*facts_for(payload))
    assert matches(ZERO_ANCHORED, [payload]) == [True]


@pytest.mark.native
def test_a_zero_anchored_rule_is_rejected_natively(tmp_path):
    # Pinned through `--compile-rules`: the rejection is the compiler's, not the runtime's,
    # so the loose pack never becomes a database.
    source = tmp_path / "loose.yar"
    source.write_text(rule(ZERO_ANCHORED, name="loose"))
    result = subprocess.run(
        [os.environ["MAGIKA_TEST_BINARY"], "--compile-rules", str(source)],
        capture_output=True,
        text=True,
    )
    assert result.returncode != 0
    assert REJECTION in result.stderr, result.stderr
    assert not source.with_suffix(".hsdb").exists()
    anchored = tmp_path / "anchored.yar"
    anchored.write_text(rule(f"pe_valid == 1 and {ZERO_ANCHORED}", name="anchored"))
    result = subprocess.run(
        [os.environ["MAGIKA_TEST_BINARY"], "--compile-rules", str(anchored)],
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr
    assert anchored.with_suffix(".hsdb").is_file()


# --- the maintained rulesets --------------------------------------------------------------

FACT_IDENTIFIERS = set(FACT_NAMES[2:]) | set(VIEW_NAMES)
RULE_HEADER = re.compile(r"^[ \t]*(?:private[ \t]+)?rule[ \t]+(\w+)", re.MULTILINE)


def rule_chunks(source):
    """`(name, text)` per rule of `source`, each text running up to the next rule header.

    Comments between rules land in the preceding chunk, which is harmless: chunks are
    scanned for identifiers with comments and string literals stripped.
    """
    headers = list(RULE_HEADER.finditer(source))
    ends = [match.start() for match in headers[1:]] + [len(source)]
    return [
        (match[1], source[match.start() : end]) for match, end in zip(headers, ends, strict=True)
    ]


def identifiers(text):
    stripped = re.sub(r"/\*.*?\*/", " ", text, flags=re.DOTALL)
    stripped = re.sub(r"//[^\n]*", " ", stripped)
    stripped = re.sub(r'"(?:\\.|[^"\\])*"', '""', stripped)
    return set(re.findall(r"\b\w+\b", stripped))


def split_rulesets(sources):
    """The rule chunks of `sources` and the names of those naming a fact, transitively."""
    chunks = [chunk for source in sources for chunk in rule_chunks(source)]
    names = {name: identifiers(text) for name, text in chunks}
    naming = {name for name, found in names.items() if found & FACT_IDENTIFIERS}
    while True:
        referring = {name for name, found in names.items() if found & naming} - naming
        if not referring:
            return chunks, naming
        naming |= referring


def matching(scanner, content):
    return {r.identifier for r in scanner.scan(content[:4096]).matching_rules}


def test_rules_not_naming_a_fact_decide_identically_with_and_without_the_globals():
    sources = [path.read_text() for path in RULESETS]
    chunks, naming = split_rulesets(sources)
    supplied = yara_x.Compiler(includes_enabled=False)
    preprocess.define_globals(supplied)
    for source in sources:
        supplied.add_source(source)
    compiled = supplied.build()
    # The text split saw exactly the rules the compiler did.
    assert Counter(name for name, _ in chunks) == Counter(r.identifier for r in compiled)
    assert naming and naming < {name for name, _ in chunks}
    plain = yara_x.Compiler(includes_enabled=False)
    plain.define_global("original_size", 0)
    plain.define_global("prefix_size", 0)
    plain.add_source("\n".join(text for name, text in chunks if name not in naming))
    plain, supplied = yara_x.Scanner(plain.build()), yara_x.Scanner(compiled)
    fixtures = sorted(p for p in (ROOT / "tests_data/basic").rglob("*") if p.is_file())
    assert len(fixtures) > 50
    decided, facts_decided = 0, 0
    for path in fixtures:
        content = path.read_bytes()
        plain.set_global("original_size", len(content))
        plain.set_global("prefix_size", len(content[:4096]))
        preprocess.set_globals(supplied, *facts_for(path))
        expected = matching(plain, content)
        found = matching(supplied, content)
        assert found - naming == expected, path
        decided += bool(expected)
        facts_decided += bool(found & naming)
    assert decided >= 5 and facts_decided >= 5


def test_set_globals_rejects_a_scanner_built_without_the_globals():
    scanner = yara_x.Scanner(yara_x.compile(rule("true")))
    with pytest.raises(ValueError):
        preprocess.set_globals(scanner, *facts_for(b""))


# --- the runner's reference path ------------------------------------------------------------


def fake_cli(command, env, timeout):
    """Stands in for the product: every input is undecided by the model, decided by rules."""
    paths = command[command.index("--") + 1 :]
    hybrid = "--rules=enforce" in command
    rows = [
        {
            "path": path,
            "result": {
                "status": "ok",
                "value": {
                    "output": {"label": "png" if hybrid else "txt"},
                    "score": 1.0,
                    "dl": {"label": "undefined" if hybrid else "txt"},
                },
            },
        }
        for path in paths
    ]
    return {}, "\n".join(json.dumps(row) for row in rows).encode()


def records_of(tmp_path, payloads):
    records = []
    for index, content in enumerate(payloads):
        path = tmp_path / f"input-{index}"
        path.write_bytes(content)
        records.append(
            dict(path=str(path), size=len(content), sha256=hashlib.sha256(content).hexdigest())
        )
    return records


def test_observe_opens_each_input_once_and_supplies_the_facts(tmp_path, monkeypatch):
    payloads = [docx_like(), pe32plus().build(), b"plain text", b"PK\x03\x04" + bytes(20_000)]
    records = records_of(tmp_path, payloads)
    pack = tmp_path / "pack.yar"
    pack.write_text(
        rule('zip_valid == 1 and zip_names contains "\\n[Content_Types].xml\\n"', "docx")
        + rule("pe_valid == 1 and pe_machine == 0x8664", "pe")
        + rule("original_size > 10000", "large")
    )
    monkeypatch.setattr(runner, "run_cli", fake_cli)
    opens = Counter()
    original = Path.open

    def counting(self, *args, **kwargs):
        opens[str(self)] += 1
        return original(self, *args, **kwargs)

    monkeypatch.setattr(Path, "open", counting)
    rows, rules = runner.observe(Path("magika"), pack, records, {"png": "png"}, {})
    assert [row["raw_matches"] for row in rows] == [["docx"], ["pe"], [], ["large"]]
    assert [opens[row["path"]] for row in rows] == [1, 1, 1, 1]
    assert [row["rule_prediction"] for row in rows] == ["png", "png", None, "png"]
    assert set(rules) == {"docx", "pe", "large"}


def test_observe_rejects_a_manifest_size_that_disagrees_with_the_file(tmp_path, monkeypatch):
    records = records_of(tmp_path, [b"PK\x03\x04" + bytes(20_000)])
    records[0]["size"] += 1
    pack = tmp_path / "pack.yar"
    pack.write_text(rule("original_size > 10000", "large"))
    monkeypatch.setattr(runner, "run_cli", fake_cli)
    with pytest.raises(ValueError, match="size"):
        runner.observe(Path("magika"), pack, records, {"png": "png"}, {})
