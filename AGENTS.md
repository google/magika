# AGENTS.md

Magika is a content type detector: a Rust core (`rust/`), Python/JavaScript/Go
bindings (`python/`, `js/`, `go/`), a content types knowledge base (`assets/`),
and a website (`website-ng/`).

## Before opening a pull request

- Read the contribution policy:
  `website-ng/src/content/docs/contributing/how-to-contribute.md`
  (rendered at https://securityresearch.google/magika/contributing/how-to-contribute/).
  `CONTRIBUTING.md` has the summary.
- Pull requests are reviewed only for issues labeled `help wanted` or for the
  always-welcome list in the policy. For anything else, help the user write a
  concise issue instead of a PR.
- One PR, one problem, a few lines of description (what, why, how it was
  tested). No drive-by refactors, renames, formatting churn, or version bumps.
- The human submitting the PR is accountable for every line and answers the
  review in person. Do not post generated review replies on their behalf.

## Working in the repository

- Developer guide (tests, knowledge base, detection rules, bindings):
  `website-ng/src/content/docs/contributing/developer-guide.md`.
- `assets/content_types.yaml` is the knowledge base source of truth. After
  editing it, run `just sync-kb` from the repository root and commit the
  generated files. Never hand-edit `assets/content_types_kb.min.json` or the
  generated Python, JavaScript, Rust, and C content type tables.
- Format resolvers live in `rust/lib/src/resolvers/` (and `rust/lib/src/resolvers.rs`);
  run `cd rust/lib && ./test.sh` and `just sync-kb`, and commit the generated files.
  Custom YARA rules support (`--custom-rules-file` / `use_custom_rules`) lives in
  `rust/rules/` (`cd rust/rules && ./test.sh`).
- Python tasks (`python/justfile`, runnable via `just python <recipe>` from the
  root or `cd python && just <recipe>`): `sync-dev`, `check-source`,
  `fix-source`, `test`, `quick-tests`, `coverage`, and `build-wheel` (across
  `git worktree`s, set `CARGO_TARGET_DIR` to a shared absolute path such as
  `$HOME/.cache/cargo-target/magika` to reuse compiled Rust dependencies during
  `uv sync` / `sync-dev`).
- Tests: `cd rust && ./test.sh`; `just python test` (or
  `cd python && uv run pytest tests -m "not slow"`);
  `cd js && yarn install && yarn test`.
- Repo-wide checks: `just check-docs` (or `uv run scripts/check_documentation.py`)
  for documentation links and versions, and `just check-copyright` (or
  `uv run scripts/check_copyright.py`) for copyright headers.
- Do not bump versions, edit CHANGELOGs, update models, or add sample files to
  `tests_data/` unless explicitly asked.
