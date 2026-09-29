# magika-rules

Bounded format rules for Magika: a YARA subset evaluated in pure Rust over the first 4 KiB
of an input. The crate has no native dependency, no `unsafe`, no environment reads and no
global state, and never names a Magika content type: callers map the rule labels to their
own label type.

A scan either names one label, reports that enforced rules disagree, or abstains. Format
identity from a bounded prefix does not certify that the whole file is valid. Both full and
partial rules can decide a label: partial means some files of that format are missed.

## Usage

```rust,no_run
use magika_rules::{Input, Outcome, RuleSet};

let rules = RuleSet::bundled();
let bytes = std::fs::read("example.png")?;
let prefix = &bytes[..bytes.len().min(magika_rules::PREFIX_LIMIT)];
match rules.scan(Input { prefix, size: bytes.len() as u64, tail: None }) {
    Outcome::Match(i) => println!("{}", rules.labels()[i]),
    Outcome::NoMatch | Outcome::Conflict | Outcome::InsufficientInput => {}
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

`RuleSet::bundled` loads the bundled rules as they were compiled when the crate was built, in
well under a millisecond: it parses no YARA and builds each regex the first time a scan needs
it. `Source::parse` and `RuleSet::compile` accept custom rules with the same validation. A
`RuleSet` is `Send + Sync` and scans take `&self`, so one compiled pack can be shared through an
`Arc`.

`Input::prefix` must be exactly the first `min(size, PREFIX_LIMIT)` bytes; anything else is
`Outcome::InsufficientInput`.

## Directory

- `rulesets/full/`: rules with zero observed false positives and false negatives.
- `rulesets/partial/`: zero observed false positives and some false negatives.
- `rulesets/notworking/`: rules kept for reference and never enforced, including rules
  refuted by evaluation and, in `facts-pending.yar`, rules that read archive and
  executable facts, which this crate does not evaluate yet.
- `LICENSES`: notices for the sources rules were adapted from; exact references stay with
  each rule's `source_refs`.
- `build.rs`: bundles the rulesets and notices into the crate with the `bundled` feature
  (on by default), and compiles them with the crate's own `source`, `lower` and `codegen`
  modules into the Rust code `RuleSet::bundled` runs.

## Rule metadata

Every non-private rule is checked when a source is parsed:

- `enforced` (or its alias `enabled`) is a boolean and defaults to `false`.
- An enforced rule needs a string `label`, `fp_rate = 0`, and either `class = "full"` with
  `fn_rate = 0` or `class = "partial"` with `0 < fn_rate < 1`. Rates are fractions in
  `[0, 1]`, or `"unmeasured"`.
- Rule ids are unique; imports, includes and global rules are rejected.
- A rule's directory must agree with its metadata: `full` holds enforced full rules,
  `partial` enforced partial rules, `notworking` unenforced `class = "not-working"` rules.

Metadata records the author's evidence; the crate cannot verify it.

## Supported YARA

Anything outside this subset is rejected at compile time rather than ignored.

| Construct | Supported form |
| --- | --- |
| Strings | text, hex (wildcards `??` and nibbles, alternations, bounded jumps) and regex, without modifiers |
| Placement | `$a at N`, `$a in (N..M)` with literal bounds inside the prefix |
| Integers | `uint8`, `uint16`, `uint16be`, `uint32`, `uint32be` at a literal offset inside the prefix |
| Sizes | `filesize` or `original_size` (the whole input), `prefix_size` (bytes in the prefix) |
| Comparisons | `==`, `!=`, `<`, `<=`, `>`, `>=` between literals, sizes and reads |
| Modulo | `x % N == M` with `N` a power of two |
| Logic | `and`, `or`, `not`, `true`, `false`, references to other rules (private helpers are active unless disabled) |

A read past the prefix is undefined and makes its comparison false. Regexes are byte
oriented: `\xff` is one byte, never a UTF-8 sequence.

## Tests

`./test.sh` runs the full suite, including a release-mode gate that no bundled rule
mislabels a sample under `tests_data/basic` and a compile and scan budget.
`tests/regressions.rs` replays the signature regressions recorded from #1447 (reviewed
headers, corruptions, truncations and reported false positives in `tests/data`). The ignored
`tests/dataset.rs` measures the rules over the evaluation dataset; its documentation
describes how to export the manifest it reads.
