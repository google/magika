# Negative signature fixtures

These inputs exercise truncated magic, literal escape sequences and unrelated
text that previously triggered format rules. Enforced rules must abstain, except
that the valid TIFF fixture may be identified as TIFF. The current assertions
live in `rules/benchmark/tests/test_rule_regressions.py` and native parity tests.

The [original failures](https://github.com/google/magika/blob/5bc2f250ce67153589c00bb546d0ad9ebfc4fc77/tests_data/rules_negative/RESULTS.md)
are retained in Git history.
