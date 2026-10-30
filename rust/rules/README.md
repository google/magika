# magika-rules

Bounded custom format rules for Magika: a YARA subset evaluated in pure Rust over the first 4 KiB
of an input. The crate has no native dependency, no `unsafe`, no environment reads and no
global state, and never names a Magika content type: callers map the rule labels to their
own label type.

A scan either names one label, reports that enforced rules disagree, or abstains. Format
identity from a bounded prefix does not certify that the whole file is valid. Both full and
partial rules can decide a label: partial means some files of that format are missed.

## Usage

```rust,no_run
use magika_rules::{Input, Outcome, RuleSet, Source};

let yara = std::fs::read_to_string("custom.yar")?;
let rules = RuleSet::compile(&Source::parse(&yara)?)?;
let bytes = std::fs::read("example.png")?;
let prefix = &bytes[..bytes.len().min(magika_rules::PREFIX_LIMIT)];
match rules.scan(Input { prefix, size: bytes.len() as u64, tail: None }) {
    Outcome::Match(labels) => match &labels[..] {
        [i] => println!("{}", rules.labels()[*i]),
        _ => {}
    },
    Outcome::InsufficientInput => {}
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Source::parse` and `RuleSet::compile` validate and compile custom YARA rules, building each regex
the first time a scan needs it. A `RuleSet` is `Send + Sync` and scans take `&self`, so one
compiled pack can be shared through an `Arc`.

`Input::prefix` must be exactly the first `min(size, PREFIX_LIMIT)` bytes; anything else is
`Outcome::InsufficientInput`.

## Rule metadata

Every non-private rule is checked when a source is parsed:

- `enforced` (or its alias `enabled`) is a boolean and defaults to `false`.
- An enforced rule needs a string `label`, `fp_rate = 0`, and either `class = "full"` with
  `fn_rate = 0` or `class = "partial"` with `0 < fn_rate < 1`. Rates are fractions in
  `[0, 1]`, or `"unmeasured"`.
- Rule ids are unique; imports, includes and global rules are rejected.

Metadata records the author's evidence; the crate cannot verify it.

## Custom rules in Magika

Magika checks custom rules before its built-in format resolvers (`rust/lib/src/resolvers/`) and
deep learning model: `magika::Rules::compile` (or `Rules::from_files`, passed via
`Options::use_custom_rules`) in the Rust library, `--custom-rules-file` (and `--custom-rules-check`
to validate a file) in the CLI, `Magika(rules=..., rules_files=...)` in Python, and
`magika_rules_new` in C. Custom rules use this YARA subset and metadata, and their `label` values
must be Magika content types. Custom rules identify a file when the ones that match agree;
otherwise the built-in resolvers, then the model, decide as usual. When every enforced custom rule
of a content type has `class = "full"`, a model prediction of it is vetoed if none of them matched.
`tests_data/rules/custom.yar` is a small example.

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
| Facts | zip (`zip_valid`, `zip_entries`, `zip_names_entries`, ...) and PE (`pe_valid`, `pe_machine`, `pe_subsystem`, ...) integers, zero for other inputs; see `src/facts` |
| Views | `zip_names` and `zip_first_entry`, searched with `contains` or `startswith` and a literal string |
| Logic | `and`, `or`, `not`, `true`, `false`, references to other rules (private helpers are active unless disabled) |

A read past the prefix is undefined and makes its comparison false. Regexes are byte
oriented: `\xff` is one byte, never a UTF-8 sequence.

## Tests

`./test.sh` runs the full suite: unit and integration tests, documentation tests, formatting, and
clippy checks.
