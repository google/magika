# Negative signature fixtures

These inputs exercise truncated magic, literal escape sequences and unrelated
text that previously triggered format rules. Enforced rules must abstain, except
that the valid TIFF fixture may be identified as TIFF. The current assertions
live in `rules/benchmark/tests/test_rule_regressions.py` and native parity tests.

The [original failures](https://github.com/google/magika/blob/5bc2f250ce67153589c00bb546d0ad9ebfc4fc77/tests_data/rules_negative/RESULTS.md)
are retained in Git history.

## Container and PE preprocessor probes

Binary: `rust/cli` built from the rules-preprocess worktree (on 96e16f91) with `--release --features yara-rules`, CPU runtime from `rust/runtime-plugin`.
Command: `magika --rules off --backend cpu -l *` then `magika --rules enforce --backend cpu -l *`. The probes carry the surface evidence of a lifted format without the structure its rule requires; every rule abstains on them (the enforced column is the ML label falling through). Generated deterministically by the `task4-make_negatives.py` script recorded with the change.

| file | what it is | rules off | rules enforce | rule probed |
|---|---|---|---|---|
| zip_plain_names.zip | one stored member `notes.txt` | zip | zip | every `zip_names` rule |
| jar_manifest_only.zip | `META-INF/MANIFEST.MF` + a properties file, no `.class` | jar | jar (ML) | taxonomy_jar |
| aar_manifest_classes_jar.zip | Android library: `AndroidManifest.xml` first, `classes.jar`, `R.txt` | jar | jar (ML) | taxonomy_apk (manifest-first branch), taxonomy_apk_names |
| docx_names_in_comment.zip | `readme.txt` member; the OOXML part names only in the archive comment | txt | txt (ML) | taxonomy_xlsx, taxonomy_pptx, taxonomy_docx |
| odf_deflated_mimetype.zip | deflated `mimetype` (ODF text) + `content.xml` | odt | odt (ML) | taxonomy_odt (no `mimetype=` line) |
| odf_text_template.ott | stored `mimetype` = `...opendocument.text-template` + `content.xml` | odt | odt (ML) | taxonomy_odt (exact media type) |
| zip64_docx_names.zip | zip64 end record and locator; members `[Content_Types].xml`, `word/document.xml` | docx | docx (ML) | every `zip_names` rule (zip64 ends the analysis) |
| mz_garbage.bin | `MZ`, `e_lfanew` = 0x80 pointing at `XX\0\0` | txt | txt (ML) | taxonomy_pebin |
| pe_lfanew_out_of_range.bin | `MZ`, `e_lfanew` = 0x10000 beyond the file | pebin | pebin (ML) | taxonomy_pebin |

9 of 9 probes abstain with rules enforced; the `rules enforce` label equals the `rules off` label for each. The first AAR draft exposed the previous `taxonomy_apk` manifest-first branch, which accepted any archive opening with `AndroidManifest.xml`; the branch now also needs a `classes.dex` or `resources.arsc` local entry within the prefix.
