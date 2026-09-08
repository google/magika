# Cross-tool benchmark v1

`magika-compare` extends the existing `magika-rules-benchmark` package. Benchmark
protocol **1.1.0** uses structured tool configuration, hashed input snapshots,
raw tool output and Hyperfine JSON. Reports and revision deltas are generated
from saved JSON by ordinary code. No model or LLM participates in measurement,
label mapping, scoring, reporting or comparison.

## Measurement contract

- Compare Magika 1's released Rust CLI, Magika 2 with rules off, Magika 2 with
  rules enforced with ML fallback, Magika 2 rules-only, `file`/libmagic, and TrID. Additional instances of these adapters
  can be added in JSON. Every instance records its version, executable hash,
  artifact hashes and full command/settings. Missing tools must be explicitly
  listed as unavailable; an execution or parsing failure cannot become a fast
  successful result.
- Measure **input-file counts**, not internal inference batch sizes. The first
  configuration uses 1, 2, 5, 10, 25, 100 and 1,000 distinct files per invocation.
  Internal reader, inference-worker and batch settings stay fixed and are
  recorded separately. These are actual CLI workloads, including input opening,
  classification, output serialization and shutdown.
- One-file elapsed time includes startup. Larger workloads report whole-process
  files/second with startup amortized. Neither is isolated model initialization
  or a measurement of an infinitely long-running service.
- Hyperfine 1.20.0 runs commands with `--shell=none`, one warmup and three measured
  runs in the initial configuration. Each cell retains all raw times and exit
  codes. Median, minimum, maximum and throughput derive from those times.
  Tool/workload execution order is shuffled with a recorded seed. Runs are
  sequential; builds and other benchmarks must finish first. The initial run
  uses a shared host, not dedicated hardware.
- OS and tool caches are warm: input hashing and quality evaluation happen
  before timing. Model preparation still happens in each fresh process. This
  protocol does not drop system caches and does not claim cold-disk performance.
- All tools receive the same extensionless, SHA-256-named files. TrID sorts and
  deduplicates its inputs, so every tool receives distinct files in that same
  lexical order. Selection is seeded before sorting. A requested workload that
  would need duplicates or a fractional rule hit is omitted.
- Evaluate accuracy on the **entire supplied adjudicated snapshot**, separately
  from speed workloads. Speed uses both the natural sample mix and exact
  0/20/50/100% rule-hit selections where representable. Hits mean a deterministic
  result from the configured Magika rules instance when its ML-only control is
  not deterministic. Protocol 1.1 also supports `rules_reference_mode: "only"`: the
  reference runs `--rules=only`, and hits are native signature decisions, excluding
  unknowns/errors. It requires no ML control and never initializes a model.
  Both requested and observed ratios are retained. When
  reusing old workloads, observed ratios can change with the revision.
- Input size and SHA-256 are checked before measurement and hashes checked again
  at completion. Executables and declared artifacts are also checked again.
  Results directories cannot be reused or silently overwritten.

The initial CPU settings are explicit rather than described as equivalent
internal architectures: Magika 1 uses one task and two ORT intra-op threads;
Magika 2 uses two inference workers and two readers. Their internal batch sizes
are respectively the release default of one and eight. libmagic and TrID use
their serial CLIs. These settings bound the selected concurrency but do not pin
CPU affinity on macOS. GPU/default-Auto comparisons need separately named tool
instances and their own recorded configuration.

## Labels and denominators

The input snapshot supplies truth. The adapter never uses truth to choose a tool
prediction or to repair an output label:

- Magika output labels map through the corpus's Magika aliases.
- libmagic MIME output maps through the corpus's MIME metadata.
- TrID's highest-ranked extension maps through corpus extensions. A tie at the
  displayed top score abstains; lower-ranked candidates cannot rescue a mistake.
- An ambiguous alias has no exact canonical decision. Generic binary output and
  explicit unknowns abstain. Unmapped outputs and tool errors remain counted.

Accuracy is correct / all files; precision is correct / mapped decisions;
decision coverage is mapped decisions / all files. Macro recall averages each
truth class's correct / files, including classes with zero correct results.
Format coverage counts classes with at least one correct decision. Raw outputs,
candidate mappings, unmapped/ambiguous counts and per-class counts are retained.
This measures exact corpus labels: a coarse ZIP MIME cannot silently earn an APK
decision. Mapping limitations must not be presented as native tool failures.

A secondary table restricts scoring to the intersection of unambiguous label
vocabularies and supplied model support. Its class list and sample count are
saved before scoring. It is a vocabulary intersection, not an assertion that
each detector implements every class. The all-file table stays visible.

## Run and reproduce

Install the locked benchmark environment (the comparator extra supplies TrID's
StringZilla accelerator; it is not a production Magika dependency):

```sh
uv sync --directory rules --locked --extra comparators
```

Obtain the exact tool versions and databases named in a run's `config.json` and
`results.json.gz`. The first run uses the official `cli/v1.1.0` ARM64 macOS Magika
release, the locally built Magika 2 release, macOS `file` 5.41, TrID 2.48 with
StringZilla 5.1.2, and Hyperfine 1.20.0. TrID and its definitions are external
downloads, not redistributed dependencies. Record changed versions as a new run.

- [Magika release](https://github.com/google/magika/releases/tag/cli%2Fv1.1.0)
- [Hyperfine 1.20.0](https://github.com/sharkdp/hyperfine/releases/tag/v1.20.0)
- [TrID and definition downloads](https://mark0.net/soft-trid-e.html)

Build Magika 2 from the recorded source revision:

```sh
cargo build --release --locked --manifest-path rust/cli/Cargo.toml --features yara-rules
```

Configure absolute tool/artifact paths in a copy of the saved `config.json`.
Preserve flags, environment, seed, warmups, repetitions and workload settings.
Hydrate the supplied corpus by SHA-256 into `CORPUS_FILES`; input data remains
owned by the separate dataset project. A portable `inputs.json.gz` records every
hash, size, adjudicated label, class metadata and the source receipt. The first
run uses the 25,421-file validated snapshot plus the seven explicitly recorded
corpus-lane patch 136 APK corrections; it is not a replay of all later sidecars.

```sh
rules/.venv/bin/magika-compare \
  --config /absolute/path/to/config.json \
  --inputs /absolute/path/to/inputs.json.gz \
  --files-root /absolute/path/to/CORPUS_FILES \
  --workloads /absolute/path/to/workloads.json \
  --hyperfine /absolute/path/to/hyperfine \
  --revision "$(git rev-parse HEAD)" \
  --output /absolute/path/to/a-new-run-directory
```

Omit `--workloads` only when generating a new seeded workload suite. Add
`--previous /path/to/previous/results.json.gz` to compute revision deltas. Additional tool instances retain comparisons for existing matching IDs. Tool
paths can move; their actual flags remain part of the comparison identity.
Changed corpus/labels, workloads, class mappings, benchmark code/protocol,
Hyperfine version, configured environment, settings or recorded host properties
block speedup claims. Executable/model/definition versions are recorded as the
objects being compared, not silently treated as fixed.

Render or compare existing measurements without executing a detector:

```sh
rules/.venv/bin/magika-compare \
  --render /path/to/results.json.gz \
  --previous /path/to/previous/results.json.gz \
  --output /path/to/report.md
```

`--previous` is optional. There is no historical speedup for the first run.
Same-run tools appear side by side; revision deltas use matching tool IDs and
workload identities and include accuracy, coverage and rule-hit changes as well
as elapsed-time changes. No narrative conclusion is generated.

## Stored evidence

Each committed run keeps `results.json.gz`, the generated `report.md`, portable
`inputs.json.gz`, `workloads.json`, `config.json`, `label-mappings.json`, and an
archive of normalized observations plus raw detector/Hyperfine output. Hashes
in `artifacts.json` cover the saved files. Corpus contents, installed tools,
model/database caches, and machine-local file lists are excluded.

The machine-local working directory also retains uncompressed `results.json`.
JSON compression changes storage only: render accepts compressed and plain JSON.
Future runs go into new directories; previous measurements are never rewritten
to make a chart look smoother or a speedup larger.

Store a finished run mechanically, including a generated report and an appended
JSON history index:

```sh
rules/.venv/bin/python rules/benchmarks/store.py \
  /absolute/path/to/completed-run \
  rules/benchmarks/results/v1/UNIQUE_RUN_ID
```

The store command refuses an existing destination or unfinished measurements.

An interrupted quality phase can be resumed into a **new** output directory with
`--reuse-quality /path/to/old-run`. Only raw detector stdout is reused, and only
when input bytes/labels/class metadata, tool identity, command, environment and
quality chunk size agree. Saved bytes are hashed in the new receipt and reparsed
with the current adapter. Timing is always measured afresh. This supports parser
fixes without repeatedly running successful classifiers.

## Rules-only measurements

[Magika 2 rules-only results](results/v1/2026-09-08-2d6c3992-rules-only/report.md)
use protocol 1.1.0 and source revision `2d6c3992`. The run classifies the same
25,421-file snapshot and measures the same 30 saved workloads as the initial CPU
and Metal runs, using `--rules=only`. The executable skips model initialization
and feature extraction, and emits `unknown` on signature misses. Vectorscan
executes on the CPU; there is no separate GPU rules-only mode.

The run directory retains its configuration, portable inputs, workloads, raw
predictions, Hyperfine JSON, generated report and SHA-256 receipt. Earlier CPU
and Metal results retain their original revisions and protocol versions. This
new mode does not claim a same-tool revision speedup against those older modes.
To reproduce, use this run's `config.json`, `inputs.json.gz` and `workloads.json`
with the command above. `rules_reference_mode: "only"` removes the need for an
ML-only control; deterministic unknown outputs do not count as rule matches.
