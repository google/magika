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
- Documentation: `just check-docs` (or `uv run scripts/check_documentation.py`) validates links and version references across all Markdown files.
- Copyright headers: `just check-copyright` (or `uv run scripts/check_copyright.py`) checks that source files carry a copyright notice.

By default, Cargo builds into `rust/target/` inside each checkout ([`.cargo/config.toml`](https://github.com/google/magika/blob/main/.cargo/config.toml)). When working across multiple `git worktree`s on Python bindings, set `CARGO_TARGET_DIR` to a shared **absolute** path (for example, `export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/magika"`) so external Rust crates compile once instead of rebuilding from scratch on every new worktree's `uv sync` or `just python sync-dev`.

## Updating the content types knowledge base

Magika maintains a content types knowledge base (KB) with metadata (`mime_type`, `group`, `description`, `extensions`, and `is_text`) for all known content types. The KB is a superset of the content types detected by any given model or resolver set: adding or editing an entry updates its metadata, whereas detecting a new content type requires a format resolver and/or model training.

### 1. Edit the YAML source of truth

[`assets/content_types.yaml`](https://github.com/google/magika/blob/main/assets/content_types.yaml) is the human-editable source of truth. **Do not** hand-edit [`assets/content_types_kb.min.json`](https://github.com/google/magika/blob/main/assets/content_types_kb.min.json), [`assets/exposed_content_types.txt`](https://github.com/google/magika/blob/main/assets/exposed_content_types.txt), generated language-binding content type tables, model documentation (`assets/models/<active_model>/README.md` and `website-ng/src/content/docs/models/<active_model>.md`), or [`website-ng/src/content/docs/content-types/main.md`](https://github.com/google/magika/blob/main/website-ng/src/content/docs/content-types/main.md).

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
- **Do not add derived fields**: `in_heuristics`, `in_resolvers`, `in_ml_model`, and `vetoed_by_resolvers` in the JSON KB are computed automatically from the built-in heuristics, Rust format resolvers, and active model configuration.

### 2. Regenerate derived files

After editing the YAML, Rust format resolvers, or the active model, run `just sync-kb` (defined in the root [`justfile`](https://github.com/google/magika/blob/main/justfile)) from the repository root and commit all resulting changes in your pull request:

```bash
just sync-kb
```

Without [`just`](https://github.com/casey/just), run `uv run scripts/sync_kb.py` directly.

What `just sync-kb` does (verified in CI via `uv run scripts/sync_kb.py --check`):

1. Validates `assets/content_types.yaml`, derives the four stage-coverage booleans (`in_heuristics`, `in_resolvers`, `in_ml_model`, `vetoed_by_resolvers`), updates the monotonic [`assets/exposed_content_types.txt`](https://github.com/google/magika/blob/main/assets/exposed_content_types.txt) allowlist, and writes [`assets/content_types_kb.min.json`](https://github.com/google/magika/blob/main/assets/content_types_kb.min.json).
2. Regenerates the Rust, C FFI, Python, and TypeScript content type definitions ([`rust/lib/src/content.rs`](https://github.com/google/magika/blob/main/rust/lib/src/content.rs), [`rust/lib/src/model.rs`](https://github.com/google/magika/blob/main/rust/lib/src/model.rs), [`rust/ffi/src/content.rs`](https://github.com/google/magika/blob/main/rust/ffi/src/content.rs), [`python/src/magika/types/content_type_label.py`](https://github.com/google/magika/blob/main/python/src/magika/types/content_type_label.py), [`js/src/content-type-label.ts`](https://github.com/google/magika/blob/main/js/src/content-type-label.ts), and [`js/src/content-types-infos.ts`](https://github.com/google/magika/blob/main/js/src/content-types-infos.ts)).
3. Regenerates the decoupled `main` supported-content-types reference ([`website-ng/src/content/docs/content-types/main.md`](https://github.com/google/magika/blob/main/website-ng/src/content/docs/content-types/main.md)), the per-version index (`content-types/index.md`), and the model-only documentation for the active ML model (`assets/models/<active_model>/README.md` and `website-ng/src/content/docs/models/<active_model>.md`). Note that resolver-only changes update `content-types/main.md` without touching the active model's README.

If a change also modifies C FFI signatures, the active ONNX model weights, or CLI output snapshots, run `just rust sync` (or `(cd rust && ./sync.sh)`), which CI verifies via `(cd rust && ./sync.sh --check)`.

### 3. Releases: supported-content-types snapshots

Each released version of the CLI, Rust library, Python module, and JavaScript package gets a frozen snapshot of its supported content types in `website-ng/src/content/docs/content-types/<artifact>-v<version>.md`, listed in the generated [per-version index](/magika/content-types). Snapshots follow the version each artifact declares in its own manifest ([`rust/cli/Cargo.toml`](https://github.com/google/magika/blob/main/rust/cli/Cargo.toml), [`rust/lib/Cargo.toml`](https://github.com/google/magika/blob/main/rust/lib/Cargo.toml), [`python/pyproject.toml`](https://github.com/google/magika/blob/main/python/pyproject.toml), [`js/package.json`](https://github.com/google/magika/blob/main/js/package.json)), so there is nothing to remember:

- **Between releases**, `main` declares pre-release versions (e.g. `2.0.0-dev`, `2.0.0.dev0`, `1.1.0-dev`), which never get snapshots. Just run `just sync-kb` as usual.
- **In the release PR**, drop the pre-release suffix and run `just sync-kb` (for Rust, `rust/publish.sh` does both): it freezes the snapshot of the new stable version and pins its sha256 in [`assets/content_types_snapshots.sha256`](https://github.com/google/magika/blob/main/assets/content_types_snapshots.sha256). CI (`just sync-kb --check`) fails the release PR until it does.
- **When publishing** (the `python-v*` tag workflow, `rust/publish.sh`, and the JS publish workflow), `sync_kb.py --verify-release <artifact>` checks that the released tree reproduces its snapshot byte for byte, and aborts the release otherwise.

Snapshots are immutable once pinned: `just sync-kb --check` fails if one is modified or deleted, or if an unpinned one is added. After publishing, `just check-docs` requires the "Latest version" row in [`cli-and-bindings/overview.md`](https://github.com/google/magika/blob/main/website-ng/src/content/docs/cli-and-bindings/overview.md) to point to the new version and its snapshot.

## Improving detection with resolvers

Magika combines a deep learning model with native format resolvers in [`rust/lib/src/resolvers/`](https://github.com/google/magika/tree/main/rust/lib/src/resolvers) (wired through [`rust/lib/src/resolvers.rs`](https://github.com/google/magika/blob/main/rust/lib/src/resolvers.rs)) evaluated over the first 4 KiB of a file (plus container tails when needed). For caller-supplied YARA rules at runtime (`--custom-rules-file` in the CLI, `Options::use_custom_rules` in Rust, `Magika(rules=...)` in Python, `magika_rules_new` in C), see the [`magika-rules`](https://github.com/google/magika/tree/main/rust/rules) crate and its [README](https://github.com/google/magika/blob/main/rust/rules/README.md).

Format resolvers are the part of built-in detection the community can improve directly. A resolver PR is warranted when you can show what it fixes: misdetections you can reproduce, or a content type in the knowledge base that neither the model nor the current resolvers handle. Resolvers for content types **already in the knowledge base** are welcome without an issue, both new resolvers and fixes to existing ones. If the content type is not in the knowledge base, [request it](/magika/contributing/reporting-issues/#new-content-types) first; a resolver can only return an existing `ContentType`.

What a resolver PR contains:

- The resolver logic in [`rust/lib/src/resolvers/`](https://github.com/google/magika/tree/main/rust/lib/src/resolvers) with zero observed false positives. If the resolver also has zero observed false negatives (every valid file of that format matches), list its `ContentType` in `VETOS` in [`rust/lib/src/resolvers.rs`](https://github.com/google/magika/blob/main/rust/lib/src/resolvers.rs) (and update the `VETOS.len()` assertion in its tests) so model false positives for that type are vetoed.
- Unit tests covering positive and negative headers, plus your evidence in the PR description: which samples you tested against, how many, where they come from, and what the signature was adapted from if applicable. Keep it factual and short.
- The regenerated files: run `cd rust/lib && ./test.sh`, then `just sync-kb` from the repository root, and commit the changes.

Do not add sample files to the repository as part of a resolver PR; if a sample is needed to make your case, attach it to an issue and link it.

## New bindings

Magika's core is the Rust [`magika`](https://github.com/google/magika/tree/main/rust/lib) library. The Python package wraps it directly through [`rust/pyo3`](https://github.com/google/magika/tree/main/rust/pyo3); for other languages, the C API in [`rust/ffi`](https://github.com/google/magika/tree/main/rust/ffi) is the intended entry point. New bindings should wrap the core rather than reimplement feature extraction and inference, and should be validated against the reference outputs in [`tests_data/reference`](https://github.com/google/magika/tree/main/tests_data/reference).

A new binding is a long-term maintenance commitment. Open an issue first so that we can agree on scope and ownership before any code is written.
