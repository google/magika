from magika_datasets.validators.text import bibtex

BIB = b'%% comment\n@string{jgr = "J. Geophys. Res."}\n@article{key1,\n  author = {A. Author and B. {Author}},\n  title = "Title with {Braces}",\n  journal = jgr # " Letters",\n  year = 2020,\n}\n@Book(key2, title = {Book})\n@comment{ anything { nested } }\n'


def test_entries_parse():
    result = bibtex.validate(BIB, frozenset())
    assert result.status == "pass" and "4 entries" in result.detail


def test_malformed_entries_fail_and_non_bib_is_not_applicable():
    assert bibtex.validate(BIB + b"@article{key3, title = {open}\n", frozenset()).status == "fail"
    assert bibtex.validate(BIB + b"stray text\n", frozenset()).status == "pass"
    assert bibtex.validate(b"@misc{, note = {no key}}", frozenset()).status == "pass"
    assert (
        bibtex.validate(b"@Timeout(Duration(seconds: 60))\nvoid main() {}", frozenset()).status
        == "fail"
    )
    assert bibtex.validate(b"@TestOn('vm')\nvoid main() {}", frozenset()).status == "fail"
    only_macros = b'@preamble{"x"}\n@string{a = "b"}\n'
    assert bibtex.validate(only_macros, frozenset()).status == "fail"
    assert bibtex.validate(only_macros, frozenset({"bib"})).status == "pass"

    assert bibtex.validate(b"plain text", frozenset()) is None
