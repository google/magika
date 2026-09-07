# Optional format rules

Magika's Rust library and CLI can identify formats from the first 4 KiB before
constructing ML features. YARA is the editable format; Vectorscan executes the
supported bounded conditions. Conflicting labels, insufficient evidence and scan
failures abstain and use the existing pipeline. Format identity does not certify
full-file validity. The default remains `--rules=off`.

Requesting enforcement reports an initialization error if the rule pack or native
library cannot load. Per-file scan failures still abstain. Both full and partial
rules can decide a label: partial means some files of that format are missed.

Every active bundled rule requires at least eight observed prefix bytes. Most
reviewed formats require a larger header and correlated structural fields; padding
a short magic string does not establish identity. See [rule quality](QUALITY.md)
for the reviewed formats, source comparisons, measured coverage and limitations.

## Directory

- `rulesets/full/`: rules with zero observed false positives and false negatives.
- `rulesets/partial/`: zero observed false positives and some false negatives.
- `rulesets/notworking/`: disabled rules with insufficient evidence or known failures.
- `benchmark/`: one Python tool and its own pytest suite for correctness, coverage,
  disk throughput and memory. Its `src/` contains input handling, execution and reporting.
- `package.py`: source staging and binary distribution helper, using Python's standard library.
- `content-types.json`: metadata needed to regenerate binary rule-only output labels.
- `LICENSES`: rule-source notices; exact source references stay with the YARA definitions.

The directories contain the sole maintained rule sources. Cargo assembles the
embedded source into its build output; source distributions stage these same files
inside the crate. No generated pack or evaluation report belongs in git.

## Build and use

```sh
cargo build --locked --release --manifest-path rust/cli/Cargo.toml --features yara-rules
rust/target/release/magika --rules=enforce example.png
rust/target/release/magika --write-default-rules custom.yar
rust/target/release/magika --compile-rules custom.yar
rust/target/release/magika --rules=enforce --rules-file custom.yar example.png
```

Ordinary Cargo builds require no Python or dataset. Execution and native compilation
need a compatible Vectorscan `libhs`; set `MAGIKA_VECTORSCAN_LIBRARY` to its location,
or place it in `lib/` beside the executable. Cargo installation does not install libhs.
Native databases are compiled for the execution target, not by the Cargo build script.

An installed `rules/promoted.yar` beside the executable takes precedence over embedded
defaults. A paired `.hsdb` is reused when compatible. Modified or incompatible source
compiles into the user cache. `MAGIKA_RULES_CACHE` selects the cache directory; an empty
value disables writable caching. Sources and compiled packs are trusted configuration.

Terminals require a canonical Magika `label`, `enforced`, `class`, `fp_rate` and
`fn_rate`. Active full rules require both rates zero; partial rules require FP rate
zero and FN rate strictly between zero and one. Rates are fractions. The build checks
directory/metadata agreement. Metadata records the author's evidence; it is not proof
of precision. Generic YARA engines do not apply Magika enforcement metadata.

Supported conditions include bounded text/hex/byte-regex matches, fixed-offset unsigned
integer comparisons, original/prefix length comparisons, power-of-two modulo equality,
and positive AND/OR coordination. Unsupported enabled conditions fail loading; they
are never silently dropped. Use `original_size` for actual length; YARA `filesize`
would describe only the scanned prefix and is rejected. No imports, decompression,
full-file scanning or runtime condition interpreter is used.

The Rust interfaces are `RulesMode`, `Builder::with_rules_mode`, and `RuleSet` source/file
loading and compiled-pack export. Existing extraction entry points retain rules-off
behavior. A ruled result's score of 1 denotes a deterministic decision, not probability.

## Run the benchmark and report

From the repository root:

```sh
uv sync --project rules --locked
uv run --project rules magika-rules-benchmark \
  --dataset /path/to/parquet-corpus \
  --model-config /path/to/model-config.json \
  --binary /path/to/magika \
  --output tmp/rules-release
```

Supply a hydrated whole-file Parquet directory with `manifest.json`, `classes.parquet`
and `shards/`. The tool verifies hashes, labels and counts and materializes ordinary
disk files beneath the output directory. It neither acquires data nor imports dataset
project code. Supplied annotations are ground truth; ML or signature-tool agreement
does not establish labels.

The command writes `report.md` and raw JSON. By default it exports the binary's exact
embedded source; `--rules-file` selects a custom pack. `--phase quality` runs correctness
only, `--phase performance` uses saved observations for the same binary/rules/model,
and `--phase render --output tmp/rules-release` regenerates Markdown without inference.
An enforced-rule false positive or unadjudicated match, required-abstention violation, reference scan error or
engine mismatch returns a failing exit status after writing the report. Such failures
also prevent the `all` phase from starting performance work. Disabled candidates remain
in the report without failing this gate. Saved input bytes are verified before timing.

Performance defaults are 10/100/1000 files, four workers, CPU, exact rule-hit percentages
from 0 through 100 by five, and three shuffled trials per case. Configure `--counts`,
`--workers`, `--hit-rates`, `--backends cpu gpu`, `--repeats`, and `--seed` as needed.
Impossible whole-file mixtures are skipped. Available inputs repeat in shuffled cycles
when necessary; repeated samples do not count as independent evidence.

Times are median complete CLI invocation times, including startup, reads, classification,
output and shutdown. File and rule caches are warmed; Parquet decoding is outside timed
work. Memory is maximum measured process peak RSS, not GPU allocation. Worker counts
on a shared machine do not establish isolated physical-core scaling.

The report leads with supported-class accuracy, rule coverage and speed; additional
classes are separate. Family tables use full/partial/not_working/none plus FP/FN and
conflicting types/extensions. FP means assigning another class's file; FN includes
abstaining on this class's file. Any observed FP blocks the affected rule. Missing
denominators are unmeasured. ROC AUC uses the reported label confidence, zero for other
labels and one for rule matches; the full model probability vector is unavailable.

## Tests

```sh
uv run --directory rules pytest
uv run --directory rules ruff check .
uv run --directory rules ruff format --check .
```

The tool has its own generated Parquet fixtures and does not need a downloaded corpus
for unit tests. Set `MAGIKA_TEST_BINARY` and `MAGIKA_VECTORSCAN_LIBRARY`, then run
`uv run --directory rules pytest --run-native -m native` for actual-product integration.
An explicitly requested native suite fails if its dependencies are absent.

## Maintainer packaging

```sh
python3 rules/package.py source --output tmp/rules-source
python3 rules/package.py bundle --binary /path/to/magika \
  --library /path/to/libhs.dylib --license /path/to/vectorscan/LICENSE \
  --output tmp/magika-rules.tar.gz
```

Source staging preserves the local tract-runtime dependency for verification. Binary
bundles export the executable's embedded source, compile a target-specific `.hsdb`,
and include notices. Existing outputs are not overwritten. These commands do not publish.
