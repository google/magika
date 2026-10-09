# How to Contribute

We would love to accept your patches and contributions to this project!

Check [open issues labeled as "help wanted"](https://github.com/google/magika/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22) as a starting point.

## Before you begin

### Sign our Contributor License Agreement

Contributions to this project must be accompanied by a
[Contributor License Agreement](https://cla.developers.google.com/about) (CLA).
You (or your employer) retain the copyright to your contribution; this simply
gives us permission to use and redistribute your contributions as part of the
project.

If you or your current employer have already signed the Google CLA (even if it
was for a different project), you probably don't need to do it again.

Visit <https://cla.developers.google.com/> to see your current agreements or to
sign a new one.

### Review our Community Guidelines

This project follows [Google's Open Source Community
Guidelines](https://opensource.google/conduct/).

## Contribution process

### Code Reviews

All submissions, including submissions by project members, require review. We
use [GitHub pull requests](https://docs.github.com/articles/about-pull-requests)
for this purpose.

## Updating the Content Types Knowledge Base

Magika maintains a content types knowledge base (KB) with metadata (`mime_type`,
`group`, `description`, `extensions`, and `is_text`) for all known content
types. Note that the KB is a superset of the content types detected by any
given model or ruleset; adding or editing an entry in the KB updates its
metadata, whereas detecting a new content type requires YARA rules and/or model
training.

### 1. Edit the YAML source of truth

[`assets/content_types.yaml`](assets/content_types.yaml) is the human-editable
source of truth for the knowledge base. **Do not** edit
[`assets/content_types_kb.min.json`](assets/content_types_kb.min.json) or any
language-specific content type definitions by hand, as they are generated.

Each entry in [`assets/content_types.yaml`](assets/content_types.yaml) looks
like:

```yaml
"tsv":
  mime_type: "text/tab-separated-values"
  group: "code"
  description: "TSV document"
  extensions: ["tsv"]
  is_text: true
```

When editing [`assets/content_types.yaml`](assets/content_types.yaml), follow
these formatting rules (enforced by [`scripts/sync_kb.py`](scripts/sync_kb.py)):

- **Quote all strings**: Content type names, `mime_type`, `group`,
  `description`, and every item in `extensions` must be quoted (e.g., `"null"`,
  `"3gp"`). Only field names and the literals `null`, `true`, and `false` are
  unquoted. This avoids YAML implicit-typing pitfalls.
- **Keep entries sorted**: Content types must be sorted alphabetically by name,
  and names must match `^[a-z0-9_]+$`.
- **Keep fields in order**: Every entry must list `mime_type`, `group`,
  `description`, `extensions`, and `is_text` in that exact order. Use `null`
  when `mime_type`, `group`, or `description` is unknown, and `[]` when there
  are no known extensions.
- **Format extensions without a leading dot**: Extensions are case-sensitive and
  must not start with `.` (e.g., `"tsv"`, `"R"`).
- **Do not add derived fields**: `rule_coverage` and `in_ml_model` in the JSON
  KB are computed automatically from the YARA rulesets and model configuration.

### 2. Regenerate derived files

After editing [`assets/content_types.yaml`](assets/content_types.yaml), run
`just sync-kb` (defined in the root [`justfile`](justfile)) from the repository
root and commit all resulting changes in your pull request:

```bash
just sync-kb
```

If you do not have [`just`](https://github.com/casey/just) installed, you can
run the underlying sync commands directly:

```bash
uv run scripts/sync_kb.py
uv run python/scripts/sync.py python
uv run python/scripts/sync.py js
(cd rust && ./sync.sh)
```

What each command does:

1. `uv run scripts/sync_kb.py` validates
   [`assets/content_types.yaml`](assets/content_types.yaml), derives
   `rule_coverage` and `in_ml_model`, and regenerates
   [`assets/content_types_kb.min.json`](assets/content_types_kb.min.json). CI
   verifies this with `uv run scripts/sync_kb.py --check`.
2. `uv run python/scripts/sync.py python` regenerates the Python
   `ContentTypeLabel` enum in
   [`python/src/magika/types/content_type_label.py`](python/src/magika/types/content_type_label.py)
   (only changes when content type labels are added, removed, or renamed).
3. `uv run python/scripts/sync.py js` regenerates the TypeScript definitions in
   [`js/src/content-type-label.ts`](js/src/content-type-label.ts) and
   [`js/src/content-types-infos.ts`](js/src/content-types-infos.ts) (only
   changes when content type labels or `is_text` values change).
4. `(cd rust && ./sync.sh)` regenerates the Rust and C FFI content type
   tables ([`rust/lib/src/content.rs`](rust/lib/src/content.rs),
   [`rust/ffi/src/content.rs`](rust/ffi/src/content.rs), and
   [`rust/ffi/include/magika.h`](rust/ffi/include/magika.h)) along with CLI
   snapshot outputs for content types exposed by the Rust crate. CI verifies
   this with `(cd rust && ./sync.sh --check)`. If you only need to regenerate
   the Rust content type tables without running the full model/CLI sync, you can
   run `(cd rust/gen && cargo run)`.
