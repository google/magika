import plistlib

from helpers import status

from magika_datasets.validators.data import applebplist
from magika_datasets.validators.text import csv_text, ini


def test_binary_plist():
    data = plistlib.dumps(
        {"a": [1, 2.5, "s", b"b", {"n": None if False else 0}]}, fmt=plistlib.FMT_BINARY
    )
    observation = applebplist.validate(data, frozenset())
    assert (observation.status, observation.format_id) == ("pass", "applebplist")
    assert status(applebplist, data[:-3]) == "fail"
    assert (
        status(applebplist, data[:-32] + b"\0" * 8 + data[-32:]) != "fail"
    )  # padding before the trailer is tolerated
    assert status(applebplist, b"bplist00" + b"\0" * 40) == "fail"
    assert status(applebplist, b"bplist99") == "not_applicable"


def test_ini_requires_hint_and_sections():
    text = b"; comment\n[main]\nkey = value\nother: 2\n\n[second]\nflag\n"
    assert ini.REQUIRES_HINT and ini.CONTEXT_REQUIRED
    observation = ini.validate(text, frozenset({"ini"}))
    assert (observation.status, observation.format_id) == ("pass", "ini")
    assert status(ini, b"key = value\n", frozenset({"ini"})) == "inconclusive"
    assert status(ini, b"[main]\n[main]\nk=1\n", frozenset({"ini"})) == "fail"
    assert status(ini, b"[main]\nk=1\n  \x00\n", frozenset({"ini"})) == "fail"
    assert status(ini, b"\xef\xbb\xbf[a]\nb=c\n", frozenset({"ini"})) == "pass"
    assert (
        status(ini, b"[a]\nflag\n  continued\n", frozenset({"ini"})) == "fail"
    )  # stdlib crash path


def test_csv_and_tsv_consistency():
    csv = b'a,b,c\n1,2,3\n4,5,"6,7"\n'
    observation = csv_text.validate(csv, frozenset({"csv"}))
    assert (observation.status, observation.format_id) == ("pass", "csv")
    tsv = b"a\tb\n1\t2\n3\t4\n"
    assert csv_text.validate(tsv, frozenset({"tsv"})).format_id == "tsv"
    assert (
        status(csv_text, csv, frozenset({"tsv"})) == "inconclusive"
    )  # no tabs: one column, no evidence
    assert status(csv_text, b"a,b\n1,2,3\n4,5\n", frozenset({"csv"})) == "fail"
    assert status(csv_text, b"single\n", frozenset({"csv"})) == "inconclusive"
    assert status(csv_text, b"a,b\n1,\x00\n2,3\n", frozenset({"csv"})) == "fail"
    assert csv_text.REQUIRES_HINT
