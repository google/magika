from magika_datasets.validators.text import snapshot

JEST = b'// Jest Snapshot v1, https://goo.gl/fbAQLP\n\nexports[`renders 1`] = `\n<div>\n  hi \\` there ${x}\n</div>\n`;\n\nexports[`renders 2`] = `"quoted"`;\n'
INSTA = b"---\nsource: src/tests/render.rs\nexpression: rendered\n---\n<html></html>\n"


def test_jest_and_insta_snapshots_pass():
    result = snapshot.validate(JEST, frozenset())
    assert result.status == "pass" and result.tags == ("jest",) and "2 exports" in result.detail
    assert snapshot.validate(INSTA, frozenset()).tags == ("insta",)


def test_broken_statement_and_front_matter_fail():
    assert snapshot.validate(JEST + b"module.exports = 1;\n", frozenset()).status == "fail"
    assert snapshot.validate(JEST[:-3], frozenset()).status == "fail"
    assert snapshot.validate(b"---\nsource: a\nnot a key\n---\n", frozenset()).status == "fail"
    nested = b"---\nsource: a\ninfo:\n  args:\n    - x\n---\nbody\n"
    assert snapshot.validate(nested, frozenset()).status == "pass"
    assert snapshot.validate(b"---\ntitle: a\n---\n", frozenset()) is None
