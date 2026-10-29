---
title: Developer Guide
---

Practical notes for the contributions we accept; see [How to Contribute](/magika/contributing/how-to-contribute/) for what those are.

## Repository map

| Directory | Content |
| --- | --- |
| [`assets/`](https://github.com/google/magika/tree/main/assets) | Content types knowledge base and models |
| [`rust/`](https://github.com/google/magika/tree/main/rust) | Core library (`lib`), detection rules (`rules`), CLI (`cli`), C API (`ffi`), Python extension (`pyo3`), generated code (`gen`) |
| [`python/`](https://github.com/google/magika/tree/main/python), [`js/`](https://github.com/google/magika/tree/main/js), [`go/`](https://github.com/google/magika/tree/main/go) | Language bindings |
| [`tests_data/`](https://github.com/google/magika/tree/main/tests_data) | Test samples and reference outputs |
| [`website-ng/`](https://github.com/google/magika/tree/main/website-ng) | This website; the contributing pages live in `src/content/docs/contributing/` |
| [`scripts/`](https://github.com/google/magika/tree/main/scripts) | Repository-level maintenance scripts |

## Running tests

- Rust: `cd rust && ./test.sh` runs every crate; each crate also has its own `test.sh` (for example `rust/rules/test.sh`).
- Python: [`python/justfile`](https://github.com/google/magika/blob/main/python/justfile) defines common development recipes (runnable from the repository root via `just python <recipe>`, or from `python/` via `just <recipe>`), including `test`, `check-source`, `fix-source`, `quick-tests`, `coverage`, `sync-dev`, and `build-wheel`. Without [`just`](https://github.com/casey/just), run `cd python && uv run pytest tests -m "not slow"`.
- JavaScript: `cd js && yarn install && yarn test`.
- Documentation: `just python check-docs` (or `cd python && uv run ./scripts/check_documentation.py`) validates links and version references across all Markdown files.

## Updating the content types knowledge base

Magika maintains a content types knowledge base (KB) with metadata (`mime_type`, `group`, `description`, `extensions`, and `is_text`) for all known content types. The KB is a superset of the content types detected by any given model or ruleset: adding or editing an entry updates its metadata, whereas detecting a new content type requires detection rules and/or model training.

### 1. Edit the YAML source of truth

[`assets/content_types.yaml`](https://github.com/google/magika/blob/main/assets/content_types.yaml) is the human-editable source of truth. **Do not** edit [`assets/content_types_kb.min.json`](https://github.com/google/magika/blob/main/assets/content_types_kb.min.json) or any language-specific content type definitions by hand, as they are generated.

Each entry looks like:

```yaml
"tsv":
  mime_type: "text/tab-separated-values"
  group: "code"
  description: "TSV document"
  extensions: ["tsv"]
  is_text: true
```

Follow these formatting rules (enforced by [`scripts/sync_kb.py`](https://github.com/google/magika/blob/main/scripts/sync_kb.py)):

- **Quote all strings**: content type names, `mime_type`, `group`, `description`, and every item in `extensions` must be quoted (e.g., `"null"`, `"3gp"`). Only field names and the literals `null`, `true`, and `false` are unquoted. This avoids YAML implicit-typing pitfalls.
- **Keep entries sorted**: content types must be sorted alphabetically by name, and names must match `^[a-z0-9_]+$`.
- **Keep fields in order**: every entry lists `mime_type`, `group`, `description`, `extensions`, and `is_text` in that exact order. Use `null` when `mime_type`, `group`, or `description` is unknown, and `[]` when there are no known extensions.
- **Format extensions without a leading dot**: extensions are case-sensitive and must not start with `.` (e.g., `"tsv"`, `"R"`).
- **Do not add derived fields**: `rule_coverage` and `in_ml_model` in the JSON KB are computed automatically from the rulesets and the model configuration.

### 2. Regenerate derived files

After editing the YAML, run `just sync-kb` (defined in the root [`justfile`](https://github.com/google/magika/blob/main/justfile)) from the repository root and commit all resulting changes in your pull request:

```bash
just sync-kb
```

If you do not have [`just`](https://github.com/casey/just) installed, run the underlying commands directly:

```bash
uv run scripts/sync_kb.py
uv run python/scripts/sync.py python
uv run python/scripts/sync.py js
(cd rust && ./sync.sh)
```

What each command does:

1. `uv run scripts/sync_kb.py` validates the YAML, derives `rule_coverage` and `in_ml_model`, and regenerates `assets/content_types_kb.min.json`. CI verifies this with `uv run scripts/sync_kb.py --check`.
2. `uv run python/scripts/sync.py python` regenerates the Python `ContentTypeLabel` enum in [`python/src/magika/types/content_type_label.py`](https://github.com/google/magika/blob/main/python/src/magika/types/content_type_label.py) (only changes when labels are added, removed, or renamed).
3. `uv run python/scripts/sync.py js` regenerates the TypeScript definitions in [`js/src/content-type-label.ts`](https://github.com/google/magika/blob/main/js/src/content-type-label.ts) and [`js/src/content-types-infos.ts`](https://github.com/google/magika/blob/main/js/src/content-types-infos.ts) (only changes when labels or `is_text` values change).
4. `(cd rust && ./sync.sh)` regenerates the bundled rules, the Rust and C content type tables ([`rust/lib/src/content.rs`](https://github.com/google/magika/blob/main/rust/lib/src/content.rs), [`rust/ffi/src/content.rs`](https://github.com/google/magika/blob/main/rust/ffi/src/content.rs), [`rust/ffi/include/magika.h`](https://github.com/google/magika/blob/main/rust/ffi/include/magika.h)), and the CLI snapshot outputs. CI verifies this with `(cd rust && ./sync.sh --check)`. To regenerate only the Rust content type tables, run `(cd rust/gen && cargo run)`.

## Improving detection with rules

Magika combines a deep learning model with detection rules: a YARA subset evaluated in pure Rust over the first 4 KiB of a file. The rules live in the [`magika-rules`](https://github.com/google/magika/tree/main/rust/rules) crate, and its [README](https://github.com/google/magika/blob/main/rust/rules/README.md) documents the supported YARA subset, the rule metadata, and the test suite. Read it first.

Rules are the part of detection the community can improve directly. A rule PR is warranted when you can show what it fixes: misdetections you can reproduce, or a content type in the knowledge base that neither the model nor the current rules handle. Rules for content types **already in the knowledge base** are welcome without an issue, both new rules and fixes to existing ones. If the content type is not in the knowledge base, [request it](/magika/contributing/reporting-issues/#new-content-types) first; a rule can only name an existing label.

What a rule PR contains:

- The rule in the right ruleset: `rust/rules/rulesets/full/` for rules with zero observed false positives and false negatives, `rust/rules/rulesets/partial/` for zero false positives and some false negatives. The rule metadata (`label`, `class`, `fp_rate`, `fn_rate`, `source_refs`) must agree with the directory.
- Your evidence, in the PR description: which samples you tested against, how many, where they come from, and what the rule was adapted from if applicable. The metadata records this evidence; the crate cannot verify it, so we rely on what you tell us. Keep it factual and short.
- The regenerated files: run `cd rust/rules && ./test.sh`, then `cd rust && ./sync.sh` and `just sync-kb` from the repository root, and commit the changes.

Do not add sample files to the repository as part of a rule PR; if a sample is needed to make your case, attach it to an issue and link it.

## New bindings

Magika's core is the Rust [`magika`](https://github.com/google/magika/tree/main/rust/lib) library. The Python package wraps it directly through [`rust/pyo3`](https://github.com/google/magika/tree/main/rust/pyo3); for other languages, the C API in [`rust/ffi`](https://github.com/google/magika/tree/main/rust/ffi) is the intended entry point. New bindings should wrap the core rather than reimplement feature extraction and inference, and should be validated against the reference outputs in [`tests_data/reference`](https://github.com/google/magika/tree/main/tests_data/reference).

A new binding is a long-term maintenance commitment. Open an issue first so that we can agree on scope and ownership before any code is written.
