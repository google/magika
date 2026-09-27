# Changelog

## 0.1.0-dev

This is the initial version of a bounded YARA subset evaluated in pure Rust over the first 4 KiB.
`RuleSet::bundled` loads the bundled rules compiled when the crate is built; regexes are built on first use.
