# Magika rules: canonical plan for replacing PR #1447

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task, inline, in one session. Do not spawn subagents. Steps use checkbox (`- [ ]`) syntax. Every task ends with a **Gate**: run it, paste the output into your progress notes, and do not start the next task until it passes.

This is the single source of truth. There is no separate design document.

---

## 0. Ground rules for the executing session

1. **The PR branch is a quarry, never a base.** `PR=origin/worktree/rules-pr`. Take files with `git checkout $PR -- <path>` or `git show $PR:<path>`. Never `git merge`, `git rebase`, or `git cherry-pick` from it except where Part A names a specific hash.
2. **Start from `origin/main`** in a fresh worktree: `git worktree add ../magika-rules-crate -b split/rules-crate origin/main`.
3. **No native code, no `unsafe`, no environment reads, no globals** in `rust/rules`. The manifest forbids `unsafe`; Gate G8 greps for the rest.
4. **One logical change per commit.** Each task is one commit unless a step says otherwise.
5. **Gates are hard.** A failing gate stops the work; do not weaken a threshold to pass it. If a gate looks wrong, stop and report.
6. **Report outcomes exactly.** Paste gate output, not summaries of it.
7. **Do not touch** `rust/lib`, `rust/cli`, `python`, `js`, `.github` in Part B except the two lines Task 8 names.
8. **The rules benchmark copies are deleted, not moved.** `rules/benchmark/` and `rules/benchmarks/` are dropped from the split (the benchmark now lives in the dataset project). Only their rules-specific tests stay: `test_rule_regressions.py` is replayed by `rust/rules/tests/regressions.rs` with its fixtures in `rust/rules/tests/data`.
9. **The evaluation corpus is `dataset/`** (on `worktree/dataset`): 71,296 whole files, labels in `samples.parquet` (`format_id`, `label_status`), bytes in `dataset/local/corpus/objects/<2>/<64>`. Only `validated_*` labels count as truth; detector labels are hints.

---

## 1. Why this plan exists

PR #1447 is 145 commits and 278 files carrying eight independent workstreams: the rules engine, the zip/PE facts stream, 106 new content-type labels, deferred inference runtimes, CLI GPU scheduling, an 8k-line benchmark tool with 64 generated report files, nine unrelated fixes, and test samples. Reviewers asked for the rules system as a self-contained crate and for the rest to go elsewhere. Both are right.

The rules engine in the PR also has a shipping defect: it executes rules with Vectorscan, a C library loaded with `dlopen` at runtime from a path the user must provide (`MAGIKA_VECTORSCAN_LIBRARY`, Homebrew, or a hand-built bundle). `cargo install`, plain wheels and npm cannot deliver that. Everything downstream of that choice, the compiled-pack cache with lock files and repair paths, the mmap spike, the engine identity in cache keys, the pinned Vectorscan build with an ARM patch in CI, and all `unsafe`, exists only to serve it.

A survey of the shipped rulesets shows none of it is needed:

| What the 241 shipped patterns use | Count |
|---|---|
| `$x at N` (anchored compare) | 224 |
| `$x in (a..b)` (bounded window; widest is the 4 KiB prefix) | 13 |
| regex patterns (short, fixed-width byte classes) | 5 |
| hex alternations `( .. \| .. )`, wildcards `??`, jumps `[n]` | 55 / 37 / 8 |
| integer reads `uint8/16/32[be]`, `filesize` | 380 |
| `contains` / `startswith` on facts views (buffers of at most 16 KiB) | 31 |
| floating unanchored searches | 0 |

So the engine becomes pure Rust: fixed patterns lower to masked byte compares, the few alternations and regexes lower to `regex-automata` in bytes mode, `contains` lowers to `memchr::memmem`. All three crates are already in `magika`'s lock file through tract. The crate builds and runs wherever `magika` builds.

The history is too entangled to cherry-pick (most commits touch four or more workstreams), so every split PR is built from the PR's final tree by path.

---

## 2. Design

### 2.1 Crate layout

```
rust/rules/
  Cargo.toml         magika-rules 0.1.0-dev; feature bundled (default); unsafe_code = "forbid"
  Cargo.lock
  LICENSE            copy of /LICENSE
  LICENSES           third-party rule notices (from rules/LICENSES)
  README.md
  build.rs           concatenates rulesets/{full,partial,notworking}/*.yar + LICENSES -> OUT_DIR/bundled.yar
  test.sh
  rulesets/full/*.yar        zero observed FP and FN
  rulesets/partial/*.yar     zero observed FP, some FN
  rulesets/notworking/*.yar  disabled; includes facts-pending.yar until Part D
  src/
    lib.rs           public API: PREFIX_LIMIT, Input, Outcome, RuleSet; re-exports Error, Source, RuleInfo, Bucket, Class
    error.rs         Error
    source.rs        Source, RuleInfo: parse the YARA subset, validate metadata and buckets
    ir.rs            Program, Rule, Cond, Int, Read
    lower.rs         Source -> Program (AST walk; from the PR's compiler.rs minus Vectorscan strings)
    matcher.rs       Pattern::{Masked, Regex}; matches_at / matches_in
    eval.rs          Program x Input -> Outcome; agree/conflict across rules
    facts/           Part D: stream-B facts and views (zip, pe), pure functions
  tests/
    source.rs        parse/validate rejection paths
    bundled.rs       bundled source parses; every file matches its bucket; no facts rule enforced yet
    scan.rs          end-to-end on byte samples; ported engine tests
    corpus.rs        zero-false-positive gate over tests_data
    perf.rs          compile and scan budgets
```

### 2.2 Public API

```rust
pub const PREFIX_LIMIT: usize = 4096;

pub struct Source { .. }                       // validated YARA text
impl Source {
    pub fn parse(text: &str) -> Result<Source, Error>;
    pub fn parse_in_bucket(text: &str, bucket: Bucket) -> Result<Source, Error>;
    pub fn bundled() -> Source;                // feature `bundled`; validated by build.rs
    pub fn rules(&self) -> &[RuleInfo];        // id, label, class, enforced, bucket
    pub fn text(&self) -> &str;
}

pub struct RuleSet { .. }                      // Send + Sync; share via Arc
impl RuleSet {
    pub fn compile(source: &Source) -> Result<RuleSet, Error>;
    pub fn labels(&self) -> &[String];         // distinct labels of enforced rules; Outcome::Match indexes this
    pub fn rules(&self) -> &[RuleInfo];
    pub fn needs_facts(&self) -> bool;         // false until Part D
    pub fn scan(&self, input: Input<'_>) -> Outcome;   // &self, no allocation, never panics
}

pub struct Input<'a> { pub prefix: &'a [u8], pub size: u64, pub tail: Option<&'a [u8]> }

pub enum Outcome { Match(usize), NoMatch, Conflict, InsufficientInput }

pub enum Error {
    Parse(String),
    Metadata { rule: String, reason: String },
    Unsupported { rule: String, reason: String },
}
```

Design points:

- **`scan` takes `&self`.** No scratch, no per-thread state. Part D may add a reusable buffer if measurement shows the 20 KiB stream-B allocation matters.
- **Two pattern kinds, chosen at compile.** A pattern whose HIR is a fixed-width sequence of literals and mask-expressible byte classes becomes `Masked` (`??` is mask `0x00`, nibble wildcards are `0xf0`/`0x0f`). Anything else becomes one `regex_automata::meta::Regex` built with `unicode(false).utf8(false)`, searched anchored at the `at` offset or unanchored over the `in` window with the start checked against the bound.
- **`Outcome::Match(usize)` indexes `labels()`.** Two enforced rules with the same label agree; different labels conflict; same semantics as the PR's Vectorscan callback.
- **`InsufficientInput`** when `prefix` is empty or `prefix.len() != min(size, PREFIX_LIMIT)`, exactly as the PR.
- **Undefined integers.** A read past the prefix is `None`; any comparison involving `None` is `false`. Arithmetic wraps in `i64`.
- **Bundled rules** are validated at build time (parse, no imports, no globals, unique ids) and fully at first use by `Source::bundled()`.

### 2.3 How `magika` consumes it (Part C)

```rust
[features] rules = ["dep:magika-rules"]

pub enum RulesMode { Off, Enforce, Only }
impl Builder {
    pub fn with_rules_mode(self, mode: RulesMode) -> Self;
    pub fn with_ruleset(self, rules: Arc<magika_rules::RuleSet>) -> Self;   // feature `rules`
}
```

One internal module `magika::rules` holds every feature gate, the label mapping (`ContentType::from_label`, unknown labels rejected at load), and a process-wide `OnceLock` for the compiled bundled pack (process policy belongs to `magika`, not the crate). `Only` skips inference runtime construction. The CLI keeps `--rules {off,enforce,only}`, `--rules-file`, `--write-default-rules`; `--compile-rules` and every `MAGIKA_RULES_*` / `MAGIKA_VECTORSCAN_*` variable are gone.

### 2.4 What is deleted from the PR and never ported

`rust/lib/src/rules/{native,cache,mapped,engine}.rs`, `rust/lib/src/startup_trace.rs`, `rules/native/**`, `rules/package.py`, `rules/benchmark/**` and `rules/benchmarks/**` (except the regressions carried into `rust/rules/tests`), commit `233708ae` (Windows DLL search flags), `--compile-rules`, `.hsdb` packs, the CI Vectorscan build.

### 2.5 Migration map

| PR (`rust/lib/src/rules/`) | Crate (`rust/rules/src/`) | Notes |
|---|---|---|
| `rules.rs` | `lib.rs` + `magika::rules` | `RulesMode` moves to `magika` |
| `metadata.rs` + validation half of `compiler.rs` | `source.rs` | typed errors |
| lowering half of `compiler.rs` | `ir.rs`, `lower.rs`, `matcher.rs` | HIR pass kept; Vectorscan strings replaced by `Pattern` values |
| `native.rs` `matched` callback | `eval.rs` | agree/conflict loop |
| `engine.rs`, `cache.rs`, `mapped.rs`, `startup_trace.rs` | deleted | |
| `preprocess/**` | `facts/**` | Part D, verbatim |
| `build.rs` in `magika` | `build.rs` in `magika-rules` | |

---

## 3. Gates

Every gate is a command with an expected result. Gates G1 to G8 belong to Part B tasks; GC and GD to Parts C and D. "Release" means `--release`.

| Gate | Command | Passes when |
|---|---|---|
| **G1 build** | `cd rust/rules && cargo check --locked --all-targets` | exit 0 |
| **G2 source** | `cargo test --locked --test source` | 8 passed |
| **G3 bundled** | `cargo test --locked --test bundled` | 3 passed; commit message lists parked rule ids |
| **G5 matcher** | `cargo test --locked --lib matcher` | 7 new + ported hex tests pass |
| **G6 scan** | `cargo test --locked --test scan` | all pass, including 5 ported engine tests |
| **G6b corpus (zero FP)** | `cargo test --locked --release --test corpus -- --nocapture` | 0 false positives over `tests_data`; hit count printed and recorded |
| **G6c dataset (zero FP)** | `MAGIKA_RULES_DATASET=<manifest> cargo test --locked --release --test dataset -- --ignored --nocapture` | 0 false positives over `validated_*` samples of `dataset/`; hits, disagreements on unverified samples, and scan throughput printed and recorded; not run in CI |
| **G7 perf** | `cargo test --locked --release --test perf -- --nocapture` | compile < 200 ms and scan < 500 us (CI ceilings); printed line recorded in the PR; expected order: compile single-digit ms, scan < 20 us |
| **G8 hermetic** | the five greps and `cargo tree` in Task 8 | no env reads, no `unsafe`, no `magika` coupling, no globals; direct deps exactly `memchr regex-automata regex-syntax yara-x-parser` |
| **G-all** | `cd rust/rules && ./test.sh` | exit 0 (check, test, fmt, clippy `-D warnings`, doc) |
| **GC1 lib** | `cd rust/lib && cargo test --locked --features rules` | all pass, including `Only` mode without an inference runtime |
| **GC2 cli** | `cd rust/cli && ./test.sh` with `rules` feature | all pass |
| **GC3 e2e perf** | `rust/cli/bench-rules.sh` (Task 9 Step 7) | `--rules=enforce` mean wall time over `tests_data` within 5% of `--rules=off`; `--rules=only` under 25% of `--rules=off`; numbers quoted in the PR |
| **GC4 parity** | `rust/cli/tests/rules.rs` `bundled_rules_never_contradict_sample_labels` | for every `tests_data/basic/<label>/*` file, `--rules=enforce` outputs either the sample's label or the `--rules=off` output |
| **GD** | G3, G6, G6b, G7 re-run with facts rules un-parked | same thresholds; corpus hit count increases and is recorded |

Definition of done for a PR: every gate in its part green, outputs pasted in the PR body, `rust/rules/test.sh` green on CI.

---

## 4. Part A: the PR campaign

All branches start from `origin/main` (`e6a4c8ef` at the time of writing).

| Order | Branch | Contents | How to build it | Reviewer story |
|---|---|---|---|---|
| 1 | `split/fix-dir-cycles` | `f9383b1d` | `git cherry-pick f9383b1d` | Recursive directory cycles reported once |
| 2 | `split/fix-cli-limits` | `c4762f68`, `e8b99770` | cherry-pick both; keep the `rust/lib` hunk of `c4762f68` | CLI rejects unsafe limits, keeps output on pipeline errors |
| 3 | `split/fix-prefix-double-read` | `1124388c` | cherry-pick | One read for small files |
| 4 | `split/fix-tract-runtime` | `b20a5797`, `6630956a`, `d618d0e2`, `bc1a6426`, `26978c45`, `78e326e3`, `ba3b776b`, `dd14b1be` | cherry-pick in order; resolve `rust/lib` hunks of `bc1a6426`/`dd14b1be` by hand | GPU batch plan and conv padding correctness |
| 5 | `split/fix-release-scripts` | `543bf0eb` (drop its `rules/` hunk), `1bba16ab`, `e49587d2` | cherry-pick, then `git checkout HEAD -- rules/` | Portable release scripts, CI credential hygiene |
| 6 | `split/test-samples` | `tests_data/**` not on main, `rust/cli/README.md` table | `git checkout $PR -- tests_data`, then `rust/sync.sh` | Test data only |
| 7 | `split/content-types` | `rust/gen/content_types`, `rust/gen/src/main.rs`, `rust/lib/src/content.rs`, `python/src/magika/types/content_type_label.py`, `assets/content_types_kb.min.json`, `python/src/magika/config/content_types_kb.min.json`, `js/src/content-type-label.ts`, `js/src/content-types-infos.ts`, `rules/content-types.json` -> `rust/gen/rule-content-types.json` | `git checkout $PR -- <paths>`; regenerate with `cargo run` in `rust/gen` to prove generator output | 106 new labels, generated |
| 8 | `split/rules-crate` | new `rust/rules/` | **Part B** | Self-contained pure-Rust engine |
| 9 | `split/rules-integration` | `rust/lib` `rules` feature, `Builder`, `Session`, CLI flags, tests, bench script | **Part C** | Wiring only |
| 10 | `split/rules-facts` | `rust/rules/src/facts/**`, facts rules | **Part D** | One more input stream |
| 11 | `split/deferred-runtimes` | `rust/runtime`, `rust/runtime-abi`, `rust/runtime-plugin`, `rust/build-runtime.py`, `rust/distribution/*`, `python/scripts/prepare_runtime_wheel.py`, workflow packaging hunks, `Dockerfile`, `dist-workspace.toml` | `git checkout $PR -- <paths>`; rebase `rust/lib/Cargo.toml` swap and `rust/ffi` link by hand; drop every Vectorscan line | Loader + ABI + plugin |
| 12 | `split/cli-scheduling` | `rust/cli/src/main.rs` scheduling regions, `progress.rs` | manual port against main after #1461; **blocked until the #1461 freeze is understood** | Async GPU prep and handoff |
| 13 | `split/version-2.0` | version bumps, changelogs | last | |

Dependencies: 9 needs 7 and 8 merged. 10 needs 9. Everything else is independent. Start with 8; open 7 in parallel. Close #1447 with a link to the series once 8 and 9 are open.

---

## 5. Part B: `split/rules-crate`

### Task 1: Scaffold the crate

**Files:** create `rust/rules/Cargo.toml`, `rust/rules/src/lib.rs`, `rust/rules/src/error.rs`, `rust/rules/test.sh`, `rust/rules/LICENSE`, `rust/rules/README.md`

- [ ] **Step 1: Branch**

```bash
cd /Users/elie/git/magika && git fetch origin && git worktree add ../magika-rules-crate -b split/rules-crate origin/main
cd ../magika-rules-crate
```

- [ ] **Step 2: `rust/rules/Cargo.toml`**

```toml
[package]
name = "magika-rules"
version = "0.1.0-dev"
license = "Apache-2.0"
edition = "2021"
rust-version = "1.93"
description = "Bounded format rules for Magika: a YARA subset evaluated in pure Rust"
repository = "https://github.com/google/magika"
include = ["/LICENSE", "/LICENSES", "/README.md", "/build.rs", "/src", "/rulesets"]

[features]
default = ["bundled"]
bundled = []

[dependencies]
memchr = "2.7"
regex-automata = { version = "0.4", default-features = false, features = ["std", "syntax", "meta", "nfa", "dfa", "hybrid"] }
regex-syntax = "=0.8.11"
yara-x-parser = "=1.20.0"

[build-dependencies]
yara-x-parser = "=1.20.0"

[lints.rust]
missing_docs = "warn"
unreachable_pub = "warn"
unused = { level = "warn", priority = -1 }
unsafe_code = "forbid"
```

- [ ] **Step 3: `rust/rules/src/lib.rs`**

```rust
// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Bounded format rules: a YARA subset evaluated in pure Rust over the first [`PREFIX_LIMIT`]
//! bytes of an input. The crate has no native dependency, never reads environment variables,
//! holds no global state and never names a Magika content type; callers map
//! [`RuleSet::labels`] to their own label type.

#![forbid(unsafe_code)]

mod error;
mod source;

pub use error::Error;
pub use source::{Bucket, Class, RuleInfo, Source};

/// Bytes of input a rule may inspect.
pub const PREFIX_LIMIT: usize = 4096;
```

- [ ] **Step 4: `rust/rules/src/error.rs`**

```rust
// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

/// Everything that can go wrong before a scan. Scans never fail; see `Outcome`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The YARA text is syntactically invalid.
    Parse(String),
    /// A rule's metadata is missing, malformed, or inconsistent with its bucket.
    Metadata { rule: String, reason: String },
    /// A construct outside the supported YARA subset.
    Unsupported { rule: String, reason: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "invalid YARA: {e}"),
            Error::Metadata { rule, reason } => write!(f, "rule {rule}: {reason}"),
            Error::Unsupported { rule, reason } => write!(f, "rule {rule}: unsupported: {reason}"),
        }
    }
}

impl std::error::Error for Error {}
```

- [ ] **Step 5: `rust/rules/test.sh`**

```bash
#!/bin/sh
# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
set -e
. ../color.sh
x cargo check --locked
x cargo check --locked --all-features --all-targets
x cargo test --locked
x cargo test --locked --no-default-features
x cargo test --locked --release --test corpus --test perf
x cargo fmt -- --check
x cargo clippy --locked --all-features --all-targets -- --deny=warnings
x env RUSTDOCFLAGS=--deny=warnings cargo doc --locked --all-features --no-deps
```

- [ ] **Step 6: License and README stub**

```bash
cp LICENSE rust/rules/LICENSE
printf '# magika-rules\n\nBounded format rules for Magika, evaluated in pure Rust. See src/lib.rs docs.\n' > rust/rules/README.md
chmod +x rust/rules/test.sh
```

- [ ] **Step 7: Gate G1.** Temporarily comment `mod source;` and its `pub use`, run `cd rust/rules && cargo check --locked`. Expected exit 0. Restore the lines.

- [ ] **Step 8: Commit**: `git add rust/rules && git commit -m "Scaffold the magika-rules crate"`

---

### Task 2: `source.rs`: parsing and metadata validation

**Files:** create `rust/rules/src/source.rs`, `rust/rules/tests/source.rs`. Source material: `$PR:rust/lib/src/rules/metadata.rs` (97 lines), `$PR:rust/lib/src/rules/compiler.rs` lines 119-160.

- [ ] **Step 1: Failing tests** `rust/rules/tests/source.rs`

```rust
use magika_rules::{Bucket, Class, Error, Source};

const OK: &str = r#"
rule png { meta: label = "png" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = { 89 50 4E 47 0D 0A 1A 0A } condition: $a at 0 and filesize >= 8 }
"#;

#[test]
fn parses_an_enforced_full_rule() {
    let source = Source::parse(OK).unwrap();
    let rule = &source.rules()[0];
    assert_eq!(rule.id, "png");
    assert_eq!(rule.label.as_deref(), Some("png"));
    assert_eq!(rule.class, Class::Full);
    assert!(rule.enforced);
    assert_eq!(rule.bucket, None);
}

#[test]
fn rejects_syntax_errors() {
    assert!(matches!(Source::parse("rule { }"), Err(Error::Parse(_))));
}

#[test]
fn rejects_imports_includes_and_global_rules() {
    assert!(matches!(Source::parse("import \"pe\"\n"), Err(Error::Unsupported { .. })));
    assert!(matches!(Source::parse("global rule g { condition: true }"), Err(Error::Unsupported { .. })));
}

#[test]
fn rejects_duplicate_rule_ids() {
    let text = format!("{OK}\n{}", OK.replace("$a", "$b"));
    assert!(matches!(Source::parse(&text), Err(Error::Metadata { .. })));
}

#[test]
fn enforced_rules_need_zero_fp_and_class_consistent_fn() {
    let bad = OK.replace("fp_rate = 0", "fp_rate = 0.01");
    assert!(matches!(Source::parse(&bad), Err(Error::Metadata { .. })));
    let partial_with_zero_fn = OK.replace("class = \"full\"", "class = \"partial\"");
    assert!(matches!(Source::parse(&partial_with_zero_fn), Err(Error::Metadata { .. })));
}

#[test]
fn enforced_rules_need_a_label() {
    let bad = OK.replace("label = \"png\" ", "");
    assert!(matches!(Source::parse(&bad), Err(Error::Metadata { .. })));
}

#[test]
fn unenforced_rules_are_kept_but_flagged() {
    let off = OK.replace("enforced = true", "enforced = false");
    assert!(!Source::parse(&off).unwrap().rules()[0].enforced);
}

#[test]
fn bucket_check_rejects_a_full_rule_in_partial() {
    assert!(matches!(Source::parse_in_bucket(OK, Bucket::Partial), Err(Error::Metadata { .. })));
    assert!(Source::parse_in_bucket(OK, Bucket::Full).is_ok());
}
```

- [ ] **Step 2: Run** `cargo test --test source`. Expected: compile error, `Source` not found.

- [ ] **Step 3: Write `rust/rules/src/source.rs`**

```rust
// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Parsed, validated YARA text.

use yara_x_parser::ast::{Item, MetaValue, Rule, RuleFlags, AST};

use crate::Error;

/// Which ruleset directory a rule ships in; checked against its metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bucket { Full, Partial, NotWorking }

/// Declared coverage class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class { Full, Partial, Untested }

/// One rule's metadata, in source order (private rules excluded).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleInfo {
    pub id: String,
    pub label: Option<String>,
    pub class: Class,
    pub enforced: bool,
    pub bucket: Option<Bucket>,
}

/// Validated YARA text.
#[derive(Clone, Debug)]
pub struct Source { text: String, rules: Vec<RuleInfo> }

impl Source {
    /// Largest accepted source.
    pub const MAX_BYTES: usize = 4 * 1024 * 1024;

    pub fn parse(text: &str) -> Result<Self, Error> { Self::parse_inner(text, None) }

    /// Like `parse`, also checking every rule's metadata agrees with `bucket`.
    pub fn parse_in_bucket(text: &str, bucket: Bucket) -> Result<Self, Error> {
        Self::parse_inner(text, Some(bucket))
    }

    /// The rules shipped with this crate. `build.rs` validated them; a failure is a build bug.
    #[cfg(feature = "bundled")]
    pub fn bundled() -> Self {
        Self::parse(include_str!(concat!(env!("OUT_DIR"), "/bundled.yar")))
            .expect("bundled rules validated at build time")
    }

    pub fn text(&self) -> &str { &self.text }
    pub fn rules(&self) -> &[RuleInfo] { &self.rules }

    fn parse_inner(text: &str, bucket: Option<Bucket>) -> Result<Self, Error> {
        if text.len() > Self::MAX_BYTES {
            return Err(Error::Parse("source exceeds 4 MiB".into()));
        }
        let ast = AST::from(text);
        if !ast.errors().is_empty() {
            return Err(Error::Parse(format!("{:?}", ast.errors())));
        }
        let mut seen = std::collections::HashSet::new();
        let mut rules = Vec::new();
        for item in ast.items() {
            let Item::Rule(rule) = item else {
                return Err(Error::Unsupported { rule: String::new(), reason: "imports and includes".into() });
            };
            let id = rule.identifier.name.to_string();
            if rule.flags.contains(RuleFlags::Global) {
                return Err(Error::Unsupported { rule: id, reason: "global rules".into() });
            }
            if !seen.insert(id.clone()) {
                return Err(Error::Metadata { rule: id, reason: "duplicate rule ID".into() });
            }
            if rule.flags.contains(RuleFlags::Private) { continue; }
            rules.push(Self::info(rule, bucket)?);
        }
        Ok(Source { text: text.to_string(), rules })
    }

    fn info(rule: &Rule<'_>, bucket: Option<Bucket>) -> Result<RuleInfo, Error> {
        let id = rule.identifier.name.to_string();
        let enforced = metadata::enforced(rule, bucket)?;
        let mut label = None;
        for meta in rule.meta.iter().flatten() {
            match (meta.identifier.name, &meta.value) {
                ("label", MetaValue::String((value, _))) => {
                    if label.replace(value.to_string()).is_some() {
                        return Err(Error::Metadata { rule: id, reason: "duplicate label".into() });
                    }
                }
                ("label", _) => return Err(Error::Metadata { rule: id, reason: "label must be a string".into() }),
                _ => {}
            }
        }
        if enforced && label.is_none() {
            return Err(Error::Metadata { rule: id, reason: "enforced rule needs a label".into() });
        }
        Ok(RuleInfo { class: metadata::class(rule)?, id, label, enforced, bucket })
    }
}

mod metadata {
    //! Ported from $PR:rust/lib/src/rules/metadata.rs. Same checks, typed errors:
    //! - `enforced`/`enabled` boolean, default false
    //! - `fp_rate` must be 0 when enforced
    //! - `class = "full"` requires `fn_rate = 0`; `class = "partial"` requires `0 < fn_rate < 1`
    //! - an untested rule (no class or no rates) must not be enforced
    //! - bucket Full requires class full; Partial requires class partial; NotWorking requires !enforced
    //! Replace every `bail!`/`ensure!` with `return Err(Error::Metadata { rule, reason })`,
    //! change `bucket: Option<&str>` to `Option<super::Bucket>`, and add
    //! `pub(super) fn class(rule: &Rule<'_>) -> Result<super::Class, Error>` (Untested when absent).
}
```

- [ ] **Step 4: Gate G2**: `cargo test --locked --test source`. Expected: 8 passed.

- [ ] **Step 5: Commit**: `git add rust/rules && git commit -m "Parse and validate rule sources"`

---

### Task 3: Move the rulesets and the bundled build script

**Files:** move `rules/rulesets/**` -> `rust/rules/rulesets/**`, `rules/LICENSES` -> `rust/rules/LICENSES`; create `rust/rules/build.rs`, `rust/rules/tests/bundled.rs`

- [ ] **Step 1: Bring the files over**

```bash
git checkout $PR -- rules/rulesets rules/LICENSES
git mv rules/rulesets rust/rules/rulesets && git mv rules/LICENSES rust/rules/LICENSES
```

- [ ] **Step 2: Park the facts rules.**

```bash
grep -lE 'zip_[a-z_]+|pe_[a-z_]+' rust/rules/rulesets/full/*.yar rust/rules/rulesets/partial/*.yar
```

Cut each such rule (its whole `rule ... { }` block) into `rust/rules/rulesets/notworking/facts-pending.yar`, set `enforced = false` on it, and list the ids in the commit message. They return in Part D.

- [ ] **Step 3: Failing tests** `rust/rules/tests/bundled.rs`

```rust
#![cfg(feature = "bundled")]
use magika_rules::{Bucket, Source};

#[test]
fn bundled_source_parses_and_has_enforced_rules() {
    assert!(Source::bundled().rules().iter().any(|r| r.enforced));
}

#[test]
fn every_ruleset_file_matches_its_bucket() {
    for (dir, bucket) in [("full", Bucket::Full), ("partial", Bucket::Partial), ("notworking", Bucket::NotWorking)] {
        for entry in std::fs::read_dir(format!("{}/rulesets/{dir}", env!("CARGO_MANIFEST_DIR"))).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "yar") {
                Source::parse_in_bucket(&std::fs::read_to_string(&path).unwrap(), bucket)
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            }
        }
    }
}

#[test]
fn no_enforced_rule_uses_facts_yet() {
    let text = Source::bundled().text().to_string();
    for block in text.split("\nrule ").skip(1) {
        let uses_facts = block.contains("zip_") || block.contains("pe_");
        let enforced = block.contains("enforced = true") || block.contains("enabled = true");
        assert!(!(uses_facts && enforced), "facts rule enforced before Part D:\nrule {block}");
    }
}
```

- [ ] **Step 4: `rust/rules/build.rs`.** Port `$PR:rust/lib/build.rs` with: root always `CARGO_MANIFEST_DIR/rulesets`; licenses `CARGO_MANIFEST_DIR/LICENSES`; output `OUT_DIR/bundled.yar`; gate on `std::env::var_os("CARGO_FEATURE_BUNDLED").is_some()`, writing an empty file otherwise; validation limited to parse errors, imports, globals, duplicate ids. Keep `rerun-if-changed` per file.

- [ ] **Step 5: Gate G3**: `cargo test --locked --test bundled`. Expected: 3 passed.

- [ ] **Step 6: Commit**: `git add -A rust/rules rules && git commit -m "Move the rulesets into the magika-rules crate"` (body lists parked ids).

---

### Task 4: `ir.rs`

**Files:** create `rust/rules/src/ir.rs`

- [ ] **Step 1: Write it**

```rust
// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! The compiled form of a pack: one condition tree per enforced rule over shared patterns.

use crate::matcher::Pattern;

pub(crate) struct Program {
    pub(crate) patterns: Vec<Pattern>,
    /// Enforced rules only, in source order.
    pub(crate) rules: Vec<Rule>,
}

pub(crate) struct Rule {
    /// Index into `RuleSet::labels`.
    pub(crate) label: usize,
    pub(crate) cond: Cond,
}

/// Integer read from the prefix; out-of-range reads are `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Read { U8, U16Le, U16Be, U32Le, U32Be }

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Int {
    Const(i64),
    FileSize,
    Read(Read, Box<Int>),
    Add(Box<Int>, Box<Int>), Sub(Box<Int>, Box<Int>), Mul(Box<Int>, Box<Int>),
    And(Box<Int>, Box<Int>), Or(Box<Int>, Box<Int>), Shl(Box<Int>, Box<Int>), Shr(Box<Int>, Box<Int>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cmp { Eq, Ne, Lt, Le, Gt, Ge }

/// Boolean condition. `At`/`In` index `Program::patterns`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Cond {
    True,
    Not(Box<Cond>),
    And(Vec<Cond>),
    Or(Vec<Cond>),
    Cmp(Cmp, Int, Int),
    /// `$p at off`
    At { pattern: usize, offset: Int },
    /// `$p in (lo..hi)`: a match whose start lies in `lo..=hi`.
    In { pattern: usize, lo: Int, hi: Int },
}
```

Offsets and bounds are `Int`, not constants, because several shipped rules use `uint32(x)` as an offset.

- [ ] **Step 2: Register** `mod ir; mod matcher;` in `lib.rs` with a stub `pub(crate) struct Pattern;` in `matcher.rs`. Gate G1. Commit: `git commit -am "Define the rule program IR"`.

---

### Task 5: `matcher.rs`

**Files:** create `rust/rules/src/matcher.rs`. Source material: `$PR:rust/lib/src/rules/compiler.rs` lines 380-420 and 700-800 (hex/text/regex to HIR).

- [ ] **Step 1: Failing unit tests** at the bottom of `matcher.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use yara_x_parser::ast::{Item, AST};

    fn pattern(yara: &str) -> Pattern {
        let text = format!("rule r {{ strings: $a = {yara} condition: $a }}");
        let ast = AST::from(text.as_str());
        let Item::Rule(rule) = &ast.items().next().unwrap() else { unreachable!() };
        Pattern::from_ast(rule.patterns.as_ref().unwrap().iter().next().unwrap()).unwrap()
    }

    #[test]
    fn hex_literal_becomes_masked_compare() {
        let p = pattern("{ 89 50 4E 47 }");
        assert!(matches!(p, Pattern::Masked { .. }));
        assert!(p.matches_at(b"\x89PNG....", 0));
        assert!(!p.matches_at(b".\x89PNG...", 0));
        assert!(p.matches_at(b".\x89PNG...", 1));
        assert!(!p.matches_at(b"\x89PN", 0), "truncated input never matches");
    }

    #[test]
    fn hex_wildcards_mask_nibbles_and_bytes() {
        let p = pattern("{ 4D 5A ?? 00 1? }");
        assert!(p.matches_at(b"MZ\x99\x00\x1f", 0));
        assert!(!p.matches_at(b"MZ\x99\x00\x2f", 0));
        assert!(!p.matches_at(b"MZ\x99\x01\x1f", 0));
    }

    #[test]
    fn text_literal_is_masked_too() {
        let p = pattern("\"GIF8\"");
        assert!(matches!(p, Pattern::Masked { .. }));
        assert!(p.matches_at(b"GIF89a", 0));
    }

    #[test]
    fn alternation_and_jump_become_regex() {
        let p = pattern("{ 47 49 46 38 ( 37 | 39 ) 61 }");
        assert!(matches!(p, Pattern::Regex(_)));
        assert!(p.matches_at(b"GIF87a", 0) && p.matches_at(b"GIF89a", 0) && !p.matches_at(b"GIF88a", 0));
        let j = pattern("{ 41 [2] 42 }");
        assert!(j.matches_at(b"AxxB", 0) && !j.matches_at(b"AxB", 0));
    }

    #[test]
    fn regex_patterns_are_byte_oriented() {
        let p = pattern("/BLENDER[_-][vV][1-4][0-9]{2}/");
        assert!(p.matches_at(b"BLENDER-v302", 0));
        assert!(!p.matches_at("BLENDER-v3é2".as_bytes(), 0));
        let c = pattern("/[\\x00-\\xff]/");
        assert!(c.matches_at(b"\xff", 0), "\\xff is one byte, not a UTF-8 sequence");
    }

    #[test]
    fn in_window_bounds_the_match_start() {
        let p = pattern("\"moov\"");
        assert!(p.matches_in(b"....moov", 0, 8));
        assert!(p.matches_in(b"....moov", 4, 4));
        assert!(!p.matches_in(b"....moov", 0, 3));
        assert!(!p.matches_in(b"....moov", 5, 100));
    }

    #[test]
    fn nocase_wide_and_xor_modifiers_are_rejected() {
        let text = "rule r { strings: $a = \"x\" nocase condition: $a }";
        let ast = AST::from(text);
        let Item::Rule(rule) = &ast.items().next().unwrap() else { unreachable!() };
        assert!(Pattern::from_ast(rule.patterns.as_ref().unwrap().iter().next().unwrap()).is_err());
    }
}
```

- [ ] **Step 2: Run** `cargo test --lib matcher`. Expected: compile failure.

- [ ] **Step 3: Implement**

```rust
// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! One YARA string, compiled for anchored or window-bounded matching over a byte buffer.

use regex_automata::meta::Regex;
use regex_automata::{Anchored, Input as ReInput};
use regex_syntax::hir::{Class, Hir, HirKind};
use yara_x_parser::ast::Pattern as AstPattern;

use crate::Error;

pub(crate) enum Pattern {
    /// Fixed width: byte `i` matches when `(input[i] & mask[i]) == bytes[i]`.
    Masked { bytes: Vec<u8>, mask: Vec<u8> },
    /// Alternation, jumps or repetition. Bytes mode, no Unicode.
    Regex(Regex),
}

impl Pattern {
    /// Builds the pattern; modifiers other than `ascii` are rejected.
    pub(crate) fn from_ast(pattern: &AstPattern<'_>) -> Result<Self, Error> {
        // 1. Reject modifiers (nocase, wide, xor, base64, fullword, private): Error::Unsupported.
        // 2. Build an HIR: text -> literal bytes; hex -> port `hex_to_hir` from $PR compiler.rs
        //    (nibble wildcards -> byte classes, `??` -> [\x00-\xff], `[n]` -> {n}, `[n-m]` -> {n,m},
        //    alternation -> Hir::alternation); regex ->
        //    regex_syntax::ParserBuilder::new().unicode(false).utf8(false).build().parse(..).
        // 3. If fixed_masked(&hir) is Some, Masked; else
        //    Regex::builder()
        //        .syntax(regex_automata::util::syntax::Config::new().unicode(false).utf8(false))
        //        .build_from_hir(&hir)
        //        .map_err(|e| Error::Unsupported { rule: String::new(), reason: e.to_string() })
        unimplemented!()
    }

    /// Does the pattern match starting exactly at `at`?
    pub(crate) fn matches_at(&self, buf: &[u8], at: usize) -> bool {
        match self {
            Pattern::Masked { bytes, mask } => buf.get(at..at.saturating_add(bytes.len()))
                .is_some_and(|w| w.iter().zip(bytes).zip(mask).all(|((b, e), m)| b & m == *e)),
            Pattern::Regex(re) => at <= buf.len()
                && re.is_match(ReInput::new(buf).range(at..).anchored(Anchored::Yes)),
        }
    }

    /// Does the pattern match starting at some offset in `lo..=hi`?
    pub(crate) fn matches_in(&self, buf: &[u8], lo: usize, hi: usize) -> bool {
        let hi = hi.min(buf.len().saturating_sub(1));
        if lo > hi { return false; }
        match self {
            Pattern::Masked { bytes, mask } if mask.iter().all(|&m| m == 0xff) => {
                memchr::memmem::find_iter(&buf[lo..], bytes).next().is_some_and(|i| lo + i <= hi)
            }
            Pattern::Masked { .. } => (lo..=hi).any(|at| self.matches_at(buf, at)),
            Pattern::Regex(re) => re.find(ReInput::new(buf).range(lo..)).is_some_and(|m| m.start() <= hi),
        }
    }
}

/// `Some((bytes, mask))` when every HIR node is a literal or a mask-expressible byte class.
fn fixed_masked(hir: &Hir) -> Option<(Vec<u8>, Vec<u8>)> {
    fn walk(hir: &Hir, bytes: &mut Vec<u8>, mask: &mut Vec<u8>) -> Option<()> {
        match hir.kind() {
            HirKind::Literal(l) => {
                bytes.extend_from_slice(&l.0);
                mask.extend(std::iter::repeat(0xff).take(l.0.len()));
                Some(())
            }
            HirKind::Class(Class::Bytes(c)) => {
                let ranges: Vec<_> = c.ranges().iter().map(|r| (r.start(), r.end())).collect();
                let (v, m) = maskable(&ranges)?;
                bytes.push(v);
                mask.push(m);
                Some(())
            }
            HirKind::Concat(parts) => parts.iter().try_for_each(|p| walk(p, bytes, mask)),
            HirKind::Empty => Some(()),
            _ => None,
        }
    }
    let (mut bytes, mut mask) = (Vec::new(), Vec::new());
    walk(hir, &mut bytes, &mut mask)?;
    (!bytes.is_empty()).then_some((bytes, mask))
}

/// Nibble wildcards produce {b | b & 0xf0 == v}, {b | b & 0x0f == v} and all bytes.
fn maskable(ranges: &[(u8, u8)]) -> Option<(u8, u8)> {
    let members: Vec<u8> = (0..=255u8).filter(|b| ranges.iter().any(|&(s, e)| s <= *b && *b <= e)).collect();
    let first = *members.first()?;
    for m in [0xffu8, 0xf0, 0x0f, 0x00] {
        let v = first & m;
        let expect: Vec<u8> = (0..=255u8).filter(|b| b & m == v).collect();
        if expect == members { return Some((v, m)); }
    }
    None
}
```

Port `hex_to_hir` from the PR's `compiler.rs` into this file as a private function; the PR's compiler unit tests that exercise it move into this module's `tests`.

- [ ] **Step 4: Gate G5**: `cargo test --locked --lib matcher`. Expected: 7 new tests plus the ported hex tests pass.

- [ ] **Step 5: Commit**: `git commit -am "Match patterns with masked compares and bounded regexes"`

---

### Task 6: `lower.rs`, `eval.rs`, and the public `RuleSet`

**Files:** create `rust/rules/src/lower.rs`, `rust/rules/src/eval.rs`, `rust/rules/tests/scan.rs`; modify `rust/rules/src/lib.rs`

- [ ] **Step 1: Failing end-to-end tests** `rust/rules/tests/scan.rs`

```rust
use magika_rules::{Input, Outcome, RuleSet, Source};

const PACK: &str = r#"
rule png { meta: label = "png" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = { 89 50 4E 47 0D 0A 1A 0A } condition: $a at 0 and filesize >= 8 }
rule png_twin { meta: label = "png" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = "PNG" condition: $a at 1 }
rule gif { meta: label = "gif" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = "GIF8" condition: $a at 0 and uint8(4) == 0x37 }
rule clash { meta: label = "clash" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = "GIF89a" condition: $a at 0 }
rule gif_any { meta: label = "gif" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = "GIF8" condition: $a at 0 and uint8(4) == 0x39 }
rule off { meta: label = "elf" enforced = false class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = { 7F 45 4C 46 } condition: $a at 0 }
rule mp4 { meta: label = "mp4" enforced = true class = "full" fp_rate = 0 fn_rate = 0
  strings: $a = "ftyp" condition: $a in (4..4) and uint32be(0) >= 8 and uint32be(0) <= filesize }
"#;

fn scan(rules: &RuleSet, bytes: &[u8]) -> Outcome {
    rules.scan(Input { prefix: bytes, size: bytes.len() as u64, tail: None })
}

#[test]
fn matches_agreeing_rules_and_reports_conflicts() {
    let rules = RuleSet::compile(&Source::parse(PACK).unwrap()).unwrap();
    assert_eq!(rules.labels(), ["png", "gif", "clash", "mp4"]);
    assert_eq!(scan(&rules, b"\x89PNG\r\n\x1a\nrest"), Outcome::Match(0));
    assert_eq!(scan(&rules, b"GIF87a...."), Outcome::Match(1));
    assert_eq!(scan(&rules, b"GIF89a...."), Outcome::Conflict, "gif_any and clash disagree");
    assert_eq!(scan(&rules, b"nothing"), Outcome::NoMatch);
    assert_eq!(scan(&rules, b"\x7fELF...."), Outcome::NoMatch, "unenforced rules never fire");
}

#[test]
fn integer_reads_and_filesize_guards() {
    let rules = RuleSet::compile(&Source::parse(PACK).unwrap()).unwrap();
    assert_eq!(scan(&rules, b"\x00\x00\x00\x14ftypisom\x00\x00\x00\x00\x00\x00\x00\x00"), Outcome::Match(3));
    assert_eq!(scan(&rules, b"\x00\x00\x00\x04ftyp"), Outcome::NoMatch, "box size below 8");
    assert_eq!(scan(&rules, b"\x00\x00\xff\xffftyp"), Outcome::NoMatch, "box size above filesize");
}

#[test]
fn prefix_must_be_exactly_the_bounded_head() {
    let rules = RuleSet::compile(&Source::parse(PACK).unwrap()).unwrap();
    assert_eq!(rules.scan(Input { prefix: b"", size: 0, tail: None }), Outcome::InsufficientInput);
    assert_eq!(rules.scan(Input { prefix: b"GIF87a", size: 100, tail: None }), Outcome::InsufficientInput);
    let big = vec![0u8; magika_rules::PREFIX_LIMIT];
    assert_eq!(rules.scan(Input { prefix: &big, size: 1 << 40, tail: None }), Outcome::NoMatch);
}

#[test]
fn rules_are_send_sync_and_shareable() {
    let rules = std::sync::Arc::new(RuleSet::compile(&Source::parse(PACK).unwrap()).unwrap());
    let handles: Vec<_> = (0..4).map(|_| { let r = rules.clone(); std::thread::spawn(move || {
        (0..1000).all(|_| scan(&r, b"GIF87a") == Outcome::Match(1))
    })}).collect();
    assert!(handles.into_iter().all(|h| h.join().unwrap()));
}

#[test]
fn unsupported_conditions_are_rejected_at_compile() {
    for cond in ["#a > 1", "@a[1] == 0", "all of them", "for any i in (1..2): (true)", "$a", "$a in (0..filesize)"] {
        let text = format!(r#"rule r {{ meta: label = "png" enforced = true class = "full" fp_rate = 0 fn_rate = 0 strings: $a = "x" condition: {cond} }}"#);
        assert!(RuleSet::compile(&Source::parse(&text).unwrap()).is_err(), "{cond}");
    }
}
```

- [ ] **Step 2: `lower.rs`.** Port the `Expr` walk from `$PR` `compiler.rs` (`condition`, `integer`, `uintN`, `filesize`, `at`, `in`, `and`/`or`/`not`, comparisons). Each Vectorscan emission becomes an `ir::Cond`/`ir::Int` constructor; every `Error::Unsupported` path is kept. Only enforced rules are lowered; private rules referenced by an enforced rule are inlined with the existing cycle check. Signature:

```rust
pub(crate) fn lower(source: &Source) -> Result<(ir::Program, Vec<String>), Error>
```

- [ ] **Step 3: `eval.rs`**

```rust
// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Evaluate a program over one input. Integer arithmetic wraps in `i64`; a read past the
//! prefix is `None` and makes its comparison `false`, matching YARA's undefined value.

use crate::ir::{Cmp, Cond, Int, Program, Read};
use crate::{Input, Outcome, PREFIX_LIMIT};

pub(crate) fn scan(program: &Program, input: Input<'_>) -> Outcome {
    let Input { prefix, size, .. } = input;
    if prefix.is_empty() || size > i64::MAX as u64 || prefix.len() != size.min(PREFIX_LIMIT as u64) as usize {
        return Outcome::InsufficientInput;
    }
    let ctx = Ctx { program, prefix, size: size as i64 };
    let mut outcome = Outcome::NoMatch;
    for rule in &program.rules {
        if ctx.cond(&rule.cond) {
            outcome = match outcome {
                Outcome::NoMatch => Outcome::Match(rule.label),
                Outcome::Match(l) if l == rule.label => outcome,
                _ => return Outcome::Conflict,
            };
        }
    }
    outcome
}

struct Ctx<'a> { program: &'a Program, prefix: &'a [u8], size: i64 }

impl Ctx<'_> {
    fn cond(&self, c: &Cond) -> bool {
        match c {
            Cond::True => true,
            Cond::Not(x) => !self.cond(x),
            Cond::And(xs) => xs.iter().all(|x| self.cond(x)),
            Cond::Or(xs) => xs.iter().any(|x| self.cond(x)),
            Cond::Cmp(op, a, b) => match (self.int(a), self.int(b)) {
                (Some(a), Some(b)) => match op {
                    Cmp::Eq => a == b, Cmp::Ne => a != b, Cmp::Lt => a < b,
                    Cmp::Le => a <= b, Cmp::Gt => a > b, Cmp::Ge => a >= b,
                },
                _ => false,
            },
            Cond::At { pattern, offset } => self.int(offset).and_then(|o| usize::try_from(o).ok())
                .is_some_and(|o| self.program.patterns[*pattern].matches_at(self.prefix, o)),
            Cond::In { pattern, lo, hi } => match (self.int(lo), self.int(hi)) {
                (Some(lo), Some(hi)) if lo >= 0 && hi >= lo =>
                    self.program.patterns[*pattern].matches_in(self.prefix, lo as usize, hi.min(PREFIX_LIMIT as i64) as usize),
                _ => false,
            },
        }
    }

    fn int(&self, i: &Int) -> Option<i64> {
        Some(match i {
            Int::Const(c) => *c,
            Int::FileSize => self.size,
            Int::Read(kind, at) => {
                let at = usize::try_from(self.int(at)?).ok()?;
                let get = |n: usize| self.prefix.get(at..at.checked_add(n)?);
                match kind {
                    Read::U8 => get(1)?[0] as i64,
                    Read::U16Le => u16::from_le_bytes(get(2)?.try_into().ok()?) as i64,
                    Read::U16Be => u16::from_be_bytes(get(2)?.try_into().ok()?) as i64,
                    Read::U32Le => u32::from_le_bytes(get(4)?.try_into().ok()?) as i64,
                    Read::U32Be => u32::from_be_bytes(get(4)?.try_into().ok()?) as i64,
                }
            }
            Int::Add(a, b) => self.int(a)?.wrapping_add(self.int(b)?),
            Int::Sub(a, b) => self.int(a)?.wrapping_sub(self.int(b)?),
            Int::Mul(a, b) => self.int(a)?.wrapping_mul(self.int(b)?),
            Int::And(a, b) => self.int(a)? & self.int(b)?,
            Int::Or(a, b) => self.int(a)? | self.int(b)?,
            Int::Shl(a, b) => self.int(a)?.checked_shl(u32::try_from(self.int(b)?).ok()?)?,
            Int::Shr(a, b) => self.int(a)?.checked_shr(u32::try_from(self.int(b)?).ok()?)?,
        })
    }
}
```

- [ ] **Step 4: Finish `lib.rs`**

```rust
mod eval;
mod ir;
mod lower;
mod matcher;

/// What one scan looks at. `tail` is only read by facts rules; pass `None` until then.
#[derive(Clone, Copy, Debug)]
pub struct Input<'a> { pub prefix: &'a [u8], pub size: u64, pub tail: Option<&'a [u8]> }

/// The pack's verdict. `Match` indexes [`RuleSet::labels`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { Match(usize), NoMatch, Conflict, InsufficientInput }

/// A compiled pack. `Send + Sync`; share through `Arc`.
pub struct RuleSet { program: ir::Program, labels: Vec<String>, rules: Vec<RuleInfo> }

impl RuleSet {
    pub fn compile(source: &Source) -> Result<Self, Error> {
        let (program, labels) = lower::lower(source)?;
        Ok(RuleSet { program, labels, rules: source.rules().to_vec() })
    }
    pub fn labels(&self) -> &[String] { &self.labels }
    pub fn rules(&self) -> &[RuleInfo] { &self.rules }
    pub fn needs_facts(&self) -> bool { false }
    pub fn scan(&self, input: Input<'_>) -> Outcome { eval::scan(&self.program, input) }
}
```

- [ ] **Step 5: Port the engine tests** from `$PR:rust/lib/src/rules.rs` `native_tests` that do not involve facts, streams or the cache: `native_bundled_structural_rules_preserve_identifying_fields`, `native_alignment_guards_preserve_unsigned_reads`, `native_ascii_alternatives_preserve_byte_semantics`, `native_integer_predicates_and_actual_length`, `native_pattern_placement_respects_at_and_in_bounds`. Append to `tests/scan.rs`. Replace `RuleSet::from_source(x).unwrap()` with `RuleSet::compile(&Source::parse(x).unwrap()).unwrap()`, `.identify(p, s, None)` with `scan(&rules, p)` (explicit `Input` when `s != p.len()`), `Some(ContentType::X)` with `Outcome::Match(i)` where `i` is the label's position in `rules.labels()`, and drop the `native_` prefix.

- [ ] **Step 6: Gate G6**: `cargo test --locked --test scan`. Expected: 5 new + 5 ported pass.

- [ ] **Step 7: Commit**: `git commit -am "Lower conditions to an IR and evaluate them in pure Rust"`

---

### Task 6b: Corpus gate: zero false positives over `tests_data`

**Files:** create `rust/rules/tests/corpus.rs`

The PR's headline claim is zero rules-only false positives. This test makes that a permanent gate over the repository's own samples, with no external corpus.

- [ ] **Step 1: Write the test**

```rust
#![cfg(feature = "bundled")]
//! Every sample under tests_data/basic/<label>/ must scan to NoMatch or to its own label.
//! Labels in tests_data use Magika's canonical names, which are also the rule labels.

use magika_rules::{Input, Outcome, RuleSet, Source, PREFIX_LIMIT};
use std::path::Path;

#[test]
fn bundled_rules_never_mislabel_a_sample() {
    let rules = RuleSet::compile(&Source::bundled()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests_data/basic");
    let (mut files, mut hits, mut errors) = (0usize, 0usize, Vec::new());
    for dir in std::fs::read_dir(&root).unwrap() {
        let dir = dir.unwrap().path();
        if !dir.is_dir() { continue; }
        let label = dir.file_name().unwrap().to_str().unwrap().to_string();
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if !path.is_file() { continue; }
            let bytes = std::fs::read(&path).unwrap();
            let prefix = &bytes[..bytes.len().min(PREFIX_LIMIT)];
            files += 1;
            match rules.scan(Input { prefix, size: bytes.len() as u64, tail: None }) {
                Outcome::Match(i) if rules.labels()[i] == label => hits += 1,
                Outcome::Match(i) => errors.push(format!("{}: rules say {} ", path.display(), rules.labels()[i])),
                Outcome::Conflict => errors.push(format!("{}: conflict", path.display())),
                Outcome::NoMatch | Outcome::InsufficientInput => {}
            }
        }
    }
    eprintln!("corpus: {files} files, {hits} rule hits, {} false positives", errors.len());
    assert!(errors.is_empty(), "false positives:\n{}", errors.join("\n"));
    assert!(hits > 0, "no rule matched any sample; the pack is not wired");
}
```

- [ ] **Step 2: Gate G6b**: `cargo test --locked --release --test corpus -- --nocapture`. Expected: `0 false positives`, and record the `hits` number in your notes; it is quoted in the PR and must not decrease in Part D.

- [ ] **Step 3: If a false positive appears**, do not edit the rule. Compare with the PR: `git show $PR:rules/QUALITY.md | grep -n <label>`. If the PR also reported that sample as a hit under that label, the sample directory is mislabeled and belongs to PR 6; note it and exclude that one path with a comment. If the PR did not, the lowering has a bug: fix `lower.rs`/`matcher.rs` and add a unit test reproducing it.

- [ ] **Step 4: Commit**: `git commit -am "Gate the bundled rules on zero false positives over tests_data"`

- [ ] **Step 5: Dataset gate G6c.** Create `rust/rules/tests/dataset.rs`, an `#[ignore]` test that reads a TSV manifest (`sha256<TAB>format_id<TAB>label_status<TAB>path`) named by `MAGIKA_RULES_DATASET` (tests may read the environment; `src` may not), scans each file's prefix, and fails on any `Match`/`Conflict` that disagrees with a `validated_*` label. Mismatches on unverified samples are printed as disagreements, not failures: they are hard cases to adjudicate in `dataset/`, never label fixes made here. It also prints total prefix bytes scanned and mean scan time. The manifest is exported from `dataset/samples.parquet` by a script in the test's doc comment. A false positive follows Step 3. Commit: `"Measure the bundled rules over the evaluation dataset"`.

---

### Task 7: Performance gate

**Files:** create `rust/rules/tests/perf.rs`

- [ ] **Step 1: Write the test**

```rust
#![cfg(feature = "bundled")]
use std::hint::black_box;
use std::time::Instant;
use magika_rules::{Input, Outcome, RuleSet, Source};

#[test]
fn compile_and_scan_stay_within_budget() {
    let t = Instant::now();
    let rules = RuleSet::compile(&Source::bundled()).unwrap();
    let compile = t.elapsed();

    // Worst case for anchored rules: a prefix that starts like nothing, so every rule is tried.
    let junk: Vec<u8> = (0..magika_rules::PREFIX_LIMIT).map(|i| (i * 7919 % 251) as u8).collect();
    let input = Input { prefix: &junk, size: 1 << 20, tail: None };
    for _ in 0..1000 { black_box(rules.scan(black_box(input))); }
    let t = Instant::now();
    let n = 20_000;
    for _ in 0..n { assert_eq!(black_box(rules.scan(black_box(input))), Outcome::NoMatch); }
    let per_scan = t.elapsed() / n;

    eprintln!("perf: compile {compile:?}, scan {per_scan:?}, {} rules, {} labels", rules.rules().len(), rules.labels().len());
    assert!(compile.as_millis() < 200, "compile {compile:?}");
    assert!(per_scan.as_micros() < 500, "scan {per_scan:?}");
}
```

- [ ] **Step 2: Gate G7**: `cargo test --locked --release --test perf -- --nocapture`. Record the `perf:` line. Expected order of magnitude: compile single-digit ms, scan under 20 us. The assertions are CI ceilings, not targets.

- [ ] **Step 3: If scan exceeds 50 us in release**, profile before changing anything (`cargo flamegraph --release --test perf` or `perf record`). Likely causes: `Cond::In` on non-`0xff`-mask patterns, or regex patterns over wide windows. Fix inside `matcher.rs` only, add a unit test for the case, re-run G6, G6b, G7.

- [ ] **Step 4: Commit**: `git commit -am "Add a compile and scan performance budget"`

---

### Task 8: Docs, repo wiring, hermeticity gate

**Files:** modify `rust/rules/README.md`, `rust/rules/src/lib.rs` docs, `rust/test.sh` (add `rules` beside `lib`), `.github/workflows/rust-test.yml` (add `rust/rules` wherever `rust/lib` is listed, plus `rust/rules/**` in path filters; no new jobs)

- [ ] **Step 1: README.** Port the "Optional format rules" and "Directory" sections of `$PR:rules/README.md`, dropping every mention of the CLI, `package.py`, Vectorscan, `MAGIKA_*` variables, caches and mmap. Add a usage example matching `tests/scan.rs` and a table of supported YARA constructs (text/hex/regex strings with `ascii` only; `at`; `in` with bounded constants; `uint8/16/32[be]`; `filesize`; `and/or/not`; comparisons; private rule references; the metadata schema).

- [ ] **Step 2: Repo wiring** as listed above.

- [ ] **Step 3: Gate G8**

```bash
cd rust/rules
grep -rn "std::env\|env::var" src || echo "no env reads"
grep -rn "unsafe" src || echo "no unsafe"
grep -rn "ContentType\|magika::" . --include=*.rs || echo "no magika coupling"
grep -rn "thread_local\|OnceLock\|static mut\|lazy_static" src || echo "no globals"
cargo package --list | grep -c "rulesets/"
cargo tree -e normal --depth 1
```

Expected: the four `no ...` lines, a ruleset file count above 0, and direct dependencies exactly `memchr`, `regex-automata`, `regex-syntax`, `yara-x-parser`.

- [ ] **Step 4: Gate G-all**: `./test.sh`. Expected exit 0.

- [ ] **Step 5: Commit and open PR 8.** Title: "Add the magika-rules crate". Body template:

```
Pure-Rust format rules crate. No native dependency, no unsafe, no env reads, no globals.

API: Source::parse -> RuleSet::compile -> RuleSet::scan(Input) -> Outcome. Match indexes labels().

Gates:
- corpus: <paste G6b line>
- perf (release): <paste G7 line>
- hermetic: <paste G8 output>
- ./test.sh: green

Parked until the facts PR (notworking/facts-pending.yar): <ids from Task 3>.
Not ported from #1447: Vectorscan FFI, compiled-pack cache, mmap, startup tracing, --compile-rules.
```

---

## 6. Part C: `split/rules-integration`

Branch from `main` after PRs 7 and 8 merge. Files: `rust/lib/Cargo.toml`, `rust/lib/src/{lib,builder,runtime,session,input}.rs`, new `rust/lib/src/rules.rs`, `rust/cli/src/main.rs`, `rust/cli/tests/rules.rs`, new `rust/cli/bench-rules.sh`, CI.

### Task 9: Bridge, builder, session, CLI

- [ ] **Step 1: Failing test** in `rust/lib/src/rules.rs`

```rust
#[cfg(all(test, feature = "rules"))]
mod tests {
    use super::*;
    #[test]
    fn labels_resolve_to_content_types_or_load_fails() {
        let bad = magika_rules::Source::parse(r#"rule x { meta: label = "not-a-magika-label" enforced = true class = "full" fp_rate = 0 fn_rate = 0 strings: $a = "x" condition: $a at 0 }"#).unwrap();
        assert!(Rules::from_source(&bad).is_err());
        assert!(Rules::bundled().is_ok());
    }
}
```

- [ ] **Step 2: `rust/lib/src/rules.rs`**

```rust
//! Bridge between `magika-rules` and Magika's content types. Every `rules` feature gate lives here.

use crate::ContentType;

/// Whether format rules may decide a file before, or instead of, inference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RulesMode {
    /// Inference only (the default).
    #[default]
    Off,
    /// Rules first; a miss or conflict falls through to inference.
    Enforce,
    /// Rules only; a miss is `Unknown`. No inference runtime is loaded.
    Only,
}

#[cfg(feature = "rules")]
pub(crate) struct Rules { set: std::sync::Arc<magika_rules::RuleSet>, labels: Vec<ContentType> }

#[cfg(feature = "rules")]
impl Rules {
    pub(crate) fn bundled() -> anyhow::Result<std::sync::Arc<Self>> {
        static BUNDLED: std::sync::OnceLock<Result<std::sync::Arc<Rules>, String>> = std::sync::OnceLock::new();
        BUNDLED.get_or_init(|| Rules::from_source(&magika_rules::Source::bundled()).map_err(|e| format!("{e:#}")))
            .clone().map_err(|e| anyhow::anyhow!("bundled rules: {e}"))
    }
    pub(crate) fn from_source(source: &magika_rules::Source) -> anyhow::Result<std::sync::Arc<Self>> {
        Self::with_set(std::sync::Arc::new(magika_rules::RuleSet::compile(source)?))
    }
    pub(crate) fn with_set(set: std::sync::Arc<magika_rules::RuleSet>) -> anyhow::Result<std::sync::Arc<Self>> {
        let labels = set.labels().iter().map(|l| ContentType::from_label(l)
            .ok_or_else(|| anyhow::anyhow!("rule label {l:?} is not a Magika content type"))).collect::<anyhow::Result<_>>()?;
        Ok(std::sync::Arc::new(Rules { set, labels }))
    }
    /// Whether identification should read an archive's tail before scanning (Part D).
    pub(crate) fn needs_tail(&self, _prefix: &[u8]) -> bool { false }
    pub(crate) fn identify(&self, prefix: &[u8], size: u64, tail: Option<&[u8]>) -> Option<ContentType> {
        match self.set.scan(magika_rules::Input { prefix, size, tail }) {
            magika_rules::Outcome::Match(i) => Some(self.labels[i]),
            _ => None,
        }
    }
}

#[cfg(not(feature = "rules"))]
pub(crate) struct Rules;
#[cfg(not(feature = "rules"))]
impl Rules {
    pub(crate) fn bundled() -> anyhow::Result<std::sync::Arc<Self>> {
        anyhow::bail!("format rules require the `rules` Cargo feature")
    }
    pub(crate) fn needs_tail(&self, _: &[u8]) -> bool { false }
    pub(crate) fn identify(&self, _: &[u8], _: u64, _: Option<&[u8]>) -> Option<ContentType> { None }
}
```

- [ ] **Step 3: `rust/lib/Cargo.toml`**: feature `rules = ["dep:magika-rules"]`; `magika-rules = { version = "0.1.0-dev", path = "../rules", optional = true }`; `_doc = ["serde", "rules"]`.

- [ ] **Step 4: Builder / Runtime / Session.** `Builder::with_rules_mode(RulesMode)`; `#[cfg(feature = "rules")] Builder::with_ruleset(Arc<magika_rules::RuleSet>)`. `Builder::build`: `Only` skips inference runtime construction (`Runtime.inner: Option<..>`; `Session::identify_features` errors "rules-only runtime has no inference backend"); any mode but `Off` sets `rules = Some(explicit via Rules::with_set, else Rules::bundled()?)`. `Runtime.rules: Option<Arc<Rules>>`, cloned into `Session`.

```rust
pub fn identify_content(&mut self, file: impl Input) -> Result<FileType> {
    match FeaturesOrRuled::extract_with(file, self.rules.as_deref())? {
        FeaturesOrRuled::Ruled(ct) => Ok(FileType::Ruled(ct)),
        FeaturesOrRuled::Features(features) => match self.rules_mode {
            RulesMode::Only => Ok(FileType::Ruled(ContentType::Unknown)),
            _ => self.identify_features(&features),
        },
    }
}
```

`input.rs`: the PR's three `extract_with_*` variants collapse to `pub fn extract_with(file: impl Input, rules: Option<&Rules>) -> Result<Self>`, which reads the first block of `max(block_size, PREFIX_LIMIT)`, calls `rules.identify(prefix, size, tail)` when `Some`, and otherwise behaves exactly like today's `extract`. `extract(file)` = `extract_with(file, None)`. `PREFIX_LIMIT` comes from `magika_rules` behind the feature, with a local `4096` otherwise, and a test asserts equality.

- [ ] **Step 5: CLI.** Delete `RuleInput` and the `Rules -> RulesMode` remap; `--rules {off,enforce,only}` maps 1:1. `--rules-file <path>` reads, `Source::parse`, `RuleSet::compile`, `with_ruleset`. `--write-default-rules <path>` writes `Source::bundled().text()`. Delete `--compile-rules` and every `MAGIKA_RULES_*` / `MAGIKA_VECTORSCAN_*` mention from `main.rs` and the README.

- [ ] **Step 6: Tests.** Bring `$PR:rust/cli/tests/rules.rs` over. Delete: `compiled_empty_export_is_feature_gated_and_needs_no_native_library`, `installed_source_pack_and_library_are_discovered`, `requested_rules_fail_visibly_when_native_library_is_missing`, `bundled_rules_decide_containers_and_executables_from_facts` (Part D). Rewrite `rules_only_empty_pack_abstains_and_reports_no_inference_backend` as: `--rules=only --rules-file empty.yar` prints `unknown` for every input and the JSON output reports no backend. Add the parity gate GC4:

```rust
#[test]
fn bundled_rules_never_contradict_sample_labels() {
    // For every tests_data/basic/<label>/<file>: run `--rules=off --format '%l'` and
    // `--rules=enforce --format '%l'` over the whole directory in one invocation each
    // (batch mode; keep order). For each file, the enforce output must equal either <label>
    // or the off output. Print the count of files where enforce != off (rule hits).
}
```

Add in `rust/lib`: `Builder::default().with_rules_mode(RulesMode::Only).build()` succeeds and `identify_content` of an unknown blob is `Ruled(Unknown)`.

- [ ] **Step 7: End-to-end perf script** `rust/cli/bench-rules.sh` (GC3)

```bash
#!/bin/sh
# Measures the wall-time cost of rules in the CLI over tests_data. Requires hyperfine.
set -e
cd "$(dirname "$0")"
cargo build --release --features rules
BIN=target/release/magika
hyperfine --warmup 3 --runs 15 --export-json "${OUT:-bench-rules.json}" \
  "$BIN --rules=off -r ../../tests_data/basic >/dev/null" \
  "$BIN --rules=enforce -r ../../tests_data/basic >/dev/null" \
  "$BIN --rules=only -r ../../tests_data/basic >/dev/null"
```

Gate GC3: `enforce` mean within 5% of `off`; `only` under 25% of `off` (no model load, no inference). Quote the three means in the PR. Not run in CI.

- [ ] **Step 8: Gates GC1, GC2, GC3, GC4.** Then six commits (one per step 2 to 7) and open PR 9 with the four gate outputs in the body.

---

## 7. Part D: `split/rules-facts`

Branch from `main` after PR 9 merges. Scope: `rust/rules/src/facts/{mod,zip,pe}.rs` from `$PR:rust/lib/src/rules/preprocess/**` verbatim (pure functions with contract docs and tests; drop the `startup_trace` line; `read_tail` moves into `magika`'s `input.rs` because it needs the `Input` trait); `ir.rs` gains `Cond::ViewContains { view, pattern }`, `Cond::ViewStartsWith { view, pattern }`, `Int::Fact(FactId)`; `lower.rs` accepts the facts identifiers; `eval.rs` builds facts and views from `Input.tail` when the program uses facts and evaluates `contains` with `memchr::memmem`; `Cargo.toml` adds `miniz_oxide`; rulesets un-park `facts-pending.yar`; `RuleSet::needs_facts()` returns the real value; `magika::rules::needs_tail` calls `magika_rules::wants_tail`; the parked engine tests (`facts_rules_scan_only_an_active_facts_stream`, `zip_names_rules_identify_archives_end_to_end`, `pe_facts_rules_identify_executables_end_to_end`, `prefix_only_packs_skip_preprocessing`, `view_membership_respects_view_boundaries`, `streams_report_into_one_decision`) and the CLI test dropped in Task 9 come back; the facts tests of `test_rule_regressions.py` that `rust/rules/tests/data/export_regressions.py` leaves unrecorded (`FACTS_OR_NATIVE` minus the native parity test, plus the facts positives in `FACTS_POSITIVES`) are recorded and replayed; `tests/corpus.rs` passes `tail` (last 16 KiB) so facts rules are exercised. No public signature changes.

Gate GD: G3, G6, G6b, G7, GC3, GC4 re-run; corpus hit count must be at least the Part B number plus the number of un-parked labels present in `tests_data`; scan budget unchanged for non-zip inputs. Write the step list for this PR when PR 9 is open, in this format.

---

## 8. Handoff prompt

```
cd /Users/elie/git/magika-split-plan
Read docs/superpowers/plans/2026-09-13-rules-crate-split-plan.md in full, sections 0 to 5.
Execute Part B (Tasks 1-8) with the superpowers:executing-plans skill, inline, no subagents.
Obey section 0. PR=origin/worktree/rules-pr is a quarry: checkout by path only, never merge or rebase.
After each task paste its Gate output. Stop after Task 8 and report G6b, G7 and G8 outputs verbatim.
Do not start Part C.
```

---

## 9. Self-review

- **Coverage.** Pure Rust, no unsafe, no env, no globals: Task 1 manifest, G8. Three types, one verb: Task 6. Label-agnostic: Task 6 `lower` returns the label table, Task 9 maps it. Rulesets in crate: Task 3. Facts API stable: `Input.tail`, `needs_facts`, Part D. Zero-FP claim preserved: G6b, GC4. Performance: G7 micro, GC3 end-to-end. `Only` mode in library: Task 9.
- **Placeholders.** Task 2's `metadata` module, Task 5's `from_ast`, Task 6's `lower.rs` and Task 9's parity test name their exact source and transformations instead of pasting hundreds of lines; Part D defers its step list until PR 9 is open. Everything else has code or exact commands.
- **Type consistency.** `Outcome::Match(usize)` indexes `RuleSet::labels()` in Tasks 6, 6b, 7, 9. `RuleSet::compile(&Source)` in Tasks 6, 6b, 7, 9. `Pattern::matches_at(buf, at)` / `matches_in(buf, lo, hi)` in Tasks 5 and 6. `lower::lower(&Source) -> (Program, Vec<String>)` in Task 6. `Rules::identify(prefix, size, tail)` in Task 9 and Part D. `test.sh` runs `corpus` and `perf` in release, matching G6b and G7.
