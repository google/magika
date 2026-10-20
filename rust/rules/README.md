# magika-rules

Bounded format rules for Magika evaluated in pure Rust over the first 4 KiB of an input (plus
an archive's trailing central directory when needed). The crate has no native dependency, no
`unsafe`, no environment reads and no global state, and never names a Magika content type:
callers map the rule labels to their own label type.

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
    Outcome::Match(labels) => match &labels[..] {
        [i] => println!("{}", rules.labels()[*i]),
        _ => {}
    },
    Outcome::InsufficientInput => {}
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

`RuleSet::bundled` loads the bundled rule metadata table and evaluates inputs through a layered,
magic-keyed native dispatch cascade (`src/magic/`). A `RuleSet` is `Send + Sync` and scans take
`&self`, so one instance can be shared through an `Arc`.

`Input::prefix` must be exactly the first `min(size, PREFIX_LIMIT)` bytes; anything else is
`Outcome::InsufficientInput`.

## Architecture

Each scan passes through five dispatch tiers in order (`src/magic/mod.rs`), returning on the
first match:

1. **4-byte magic (`magic32`)**: fixed-offset dispatch on `be32(prefix, 0)`.
2. **2-byte magic (`magic16`)**: fixed-offset dispatch on `be16(prefix, 0)` for short prefixes
   and 2-byte signatures (`MZ` → PE, `7z`, `xz`, `bzip2`, `gzip`, `lzip`, `lz4`, `rar`, `txt*`,
   etc.).
3. **Non-zero-offset probes (`offset_probes`)**: formats whose magic sits at a fixed non-zero
   offset (`dicom` at `@128`, `ustar` at `@257`, ISO-BMFF `ftyp` at `@4`, `icc` `"acsp"` at
   `@36`, etc.).
4. **Bounded-window heuristics (`heuristics`)**: rules that scan a bounded prefix window (`ini`,
   `pem`, `pgp`, `gitattributes`, `vimswap`, `html`, and the floating-XML formats `gpx`, `kml`,
   `osm` gated on an XML-compatible leading byte).

The ZIP (`PK\x03\x04` / `PK\x05\x06`) and PE (`MZ`) entries run the bounded structural parsers
in `src/facts/{zip,pe}.rs` on demand.

## Directory

- `src/magic/mod.rs`: layered dispatch cascade and native verifiers for the 140 enforced rules.
- `src/magic/notworking.rs`: reference-only verifiers for the 52 disabled rules (`#[cfg(test)]`).
- `src/facts/`: bounded ZIP central-directory and PE header parsers.
- `src/label.rs` & `src/table.rs`: enforced label enum (`Label`) and rule metadata table.
- `rulesets/{full,partial,notworking}/`: original `.yar` specifications retained as reference.
- `LICENSES`: notices for the sources rules were adapted from; exact references stay with
  each rule's `source_refs`.

## Tests

`./test.sh` runs the full test, formatting, clippy, and doc suite.
`tests/corpus.rs` verifies that no bundled rule mislabels a sample under `tests_data/basic`.
`tests/regressions.rs` replays the 15,169 recorded signature regressions (reviewed headers,
corruptions, truncations and reported false positives in `tests/data`). The ignored
`tests/dataset.rs` measures the rules over the evaluation dataset; its documentation describes
how to export the manifest it reads.
