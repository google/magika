# Magika

This library crate provides file content type detection using AI. A command-line interface (CLI) for
this library is provided by the [magika-cli](https://crates.io/crates/magika-cli) binary crate.

Rust 1.93 or later is required. This checkout develops version 2.0; see the repository
changelog for API and model changes.

Prepare a shared `Runtime` and create a `Session` for each inference worker. Call
`Runtime::builder().with_max_batch(1).build()` for single-file callers. Larger batch
limits retain the plans needed for batched inference; sessions reuse those plans.

The optional `yara-rules` feature exposes `RulesMode`, `RuleSet` and rule-aware feature
extraction. Rules are off by default. `Builder::with_rules_mode(RulesMode::Enforce)`
loads bundled rules and reports initialization errors, including a missing native
Vectorscan library. `Builder::with_ruleset()` selects an explicitly loaded pack.
Full and partial enabled rules may decide a label; missed or conflicting matches
fall through to the existing pipeline. A deterministic score of 1 is not a
probability or proof that the complete file is valid.

Bundled rules include third-party adaptations. Source distributions include their
attribution in `RULES-LICENSES`; the repository also maintains notices and format
review guidance in its `rules/` directory.

## Disclaimer

This project is not an official Google project. It is not supported by Google and Google
specifically disclaims all warranties as to its quality, merchantability, or fitness for a
particular purpose.

Development versions may contain breaking changes. Release compatibility follows
[Cargo semver compatibility](https://doc.rust-lang.org/cargo/reference/semver.html).
Report issues on [GitHub](https://github.com/google/magika/issues).
