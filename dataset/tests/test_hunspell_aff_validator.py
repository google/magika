from magika_datasets.validators.text import hunspell_aff as aff

AFF = b"""# English affixes
SET UTF-8
TRY esianrtolcdugmphbyfvkwz
REP 2
REP f ph
REP ph f

PFX A Y 1
PFX A   0     re         .

SFX N Y 3
SFX N   e     ion        e
SFX N   y     ication    y
SFX N   0     en         [^ey]
"""


def test_tables_whose_counts_hold_pass():
    result = aff.validate(AFF, frozenset())
    assert result.status == "pass" and result.detail == "2 affix tables with 6 counted entries"


def test_a_short_affix_table_fails():
    data = AFF.replace(b"SFX N Y 3", b"SFX N Y 4")
    assert "ends 1 entries early" in aff.validate(data, frozenset()).detail


def test_a_rule_for_another_flag_fails():
    data = AFF.replace(b"SFX N   y", b"SFX M   y")
    assert aff.validate(data, frozenset()).status == "fail"


def test_prose_lines_fail():
    assert aff.validate(AFF + b"these are words\n", frozenset()).status == "fail"


def test_files_without_affix_tables_are_not_claimed():
    assert aff.validate(b"SET UTF-8\nTRY abc\n", frozenset()) is None
    assert aff.validate(b"\0PFX A Y 1\n", frozenset()) is None
