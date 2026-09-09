# Cross-tool benchmark

Protocol **1.2.2** measures each tool with its shipping resource defaults.
`magika-compare` records tool output and Hyperfine JSON; ordinary code generates
all tables, scores and revision comparisons from those observations. No LLM
participates in measurement, label mapping, scoring or report generation.

## Measurement contract

- Run the released Magika 1 CLI, Magika 2 CPU/GPU/Auto with ML and rules + ML,
  Magika 2 rules-only, libmagic and TrID with and without StringZilla. Record Tool, Version and Config,
  executable/model/database hashes, full commands and the machine environment.
- Use default worker counts, readers and internal batch sizes. The harness
  rejects overrides and thread-cap environment variables. Its isolated process
  environment does not inherit such variables from the invoking shell.
- Measure invocations containing 1, 2, 5, 10, 25, 100 and 1,000 distinct files.
  These are numbers of input files, not model batch sizes. Whole-process time
  includes startup, file I/O, classification, output serialization and shutdown.
- Evaluate accuracy over the entire supplied snapshot. Speed workloads use its
  natural mix and representable 0/20/50/100% rule-hit mixes. Exact saved file
  lists let revisions replay the same workload. A requested mix that requires
  duplicate files or fractional hits is omitted.
- Hyperfine uses `--shell=none`, one warmup and three measured runs. Every raw
  time and exit code is retained. Median, range and throughput are arithmetic
  over those observations. Tool/workload order is shuffled with a recorded seed.
- Runs are sequential, with builds and other benchmarks finished first. OS and
  tool caches are warm: quality evaluation and input hashing precede timing.
  These are not cold-disk measurements or isolated model-preparation timings.
- Every tool receives the same extensionless, SHA-256-named files in lexical
  order. Input sizes and hashes are verified before measurement and hashes again
  at completion. Tool artifacts are rechecked at completion too. Output errors,
  early exits and changed deterministic decisions fail the run. Static modes
  must also retain their inference decisions. For CPU warmup modes, CPU/GPU
  scheduling can change a borderline inference decision: each workload's raw
  validation output, quality metrics and differences from the full quality pass
  are recorded explicitly. Protocol 1.2.1 adds this validation evidence without
  changing the timing method used by the 1.2.0 baseline. Protocol 1.2.2 preserves
  Python virtual-environment executable paths and rejects TrID configurations
  whose acceleration flag, loaded StringZilla version or module hash disagrees
  with the requested configuration.
- CPU/GPU crossover compares measured medians at the tested file counts. No
  crossover is interpolated between workloads. A workload resolved entirely by
  rules does not demonstrate a GPU inference advantage.

Default settings are the benchmark policy, not a claim that every tool has been
exhaustively tuned. Backend and rule flags select a configuration; they do not
set artificial resource limits. GPU results on this host use Metal.

## Dataset and scoring identities

The [catalogue](CATALOG.md) and [JSON index](results/v1/index.json) record each
snapshot's name, version, UTC measurement timestamp, eligible/scored count and
source size. Dataset versions cannot silently identify different inputs.
The adjudicated combined snapshot contains 25,421 files, including the seven
recorded APK patch-136 corrections; it is not the full V56 release. Sembiance v3
contains 33,421 source files, of which 2,400 were already eligible for this
benchmark. Dataset construction and publication are separate work.

The supplied snapshot determines truth. Magika labels map through its recorded
aliases, libmagic through MIME metadata, and TrID through its highest-ranked
extension. Ties and ambiguous aliases abstain. The adapter never uses the true
label to choose a prediction. Unknown/unmapped outputs and errors remain in the
denominator; coarse ZIP output does not silently receive APK credit.

Accuracy is correct / all files. Precision is correct / mapped decisions.
Coverage is mapped decisions / all files. Rule matches count deterministic
signature decisions from the rules-only control, excluding unknowns/errors.
Per-class counts, macro recall and the shared unambiguous vocabulary comparison
remain in the raw result. CPU/GPU score or decision differences remain visible.

## Reproduce

Install the harness and prepare the current CLI with its CPU/GPU libraries:

```sh
uv sync --directory rules --locked --extra comparators
python3 rust/build-runtime.py --gpu metal --output /absolute/path/to/magika2
```

The default configuration builder accepts paths to the actual tools and a named
dataset descriptor. For the Apple Silicon comparison:

```sh
PYTHONPATH=rules/benchmark/src rules/.venv/bin/python rules/benchmarks/default_config.py \
  --magika1 /path/to/released/magika \
  --magika2 /path/to/magika2 \
  --native /path/to/libhs.dylib \
  --file /usr/bin/file --magic /usr/share/file/magic.mgc \
  --python /path/to/trid-plain/bin/python \
  --python-stringzilla /path/to/trid-stringzilla/bin/python \
  --trid /path/to/trid.py --trid-definitions /path/to/triddefs.trd \
  --dataset /path/to/dataset-descriptor.json --output /path/to/config.json
```

Use the versions and artifacts recorded in the selected run. The comparison
uses the official `cli/v1.1.0` Magika 1 release, current Magika 2, macOS `file`
5.41, TrID 2.48 without acceleration and with StringZilla 5.1.2, and Hyperfine 1.20.0. TrID definitions
and the native rules engine are external dependencies.

Use the same Python version/build for both TrID environments. The plain
environment must have no StringZilla installed; install the pinned version in
the accelerated environment:

```sh
uv venv --python /path/to/base/python /path/to/trid-plain
uv venv --python /path/to/base/python /path/to/trid-stringzilla
uv pip install --python /path/to/trid-stringzilla/bin/python stringzilla==5.1.2
```

Keep the `bin/python` paths intact: resolving their symlinks bypasses the virtual
environment. The builder discovers StringZilla's version and module path, and
the measurement harness verifies what TrID actually loads.

**Historical TrID correction:** the five 1.2.0/1.2.1 runs originally declared
StringZilla 5.1.2, but their saved runtime probes reported `Using Stringzilla:
False`. Their derived tables now say `StringZilla=off`, with explicit correction
metadata. Raw records and timings remain unchanged. The separate 1.2.2 paired
runs measure both configurations again on the same seven saved natural
workloads; unchanged Magika ML and libmagic measurements are not rerun.

- [Combined corpus: paired TrID measurements](reports/2026-09-09-7599ca96-trid-combined/trid-comparison.md)
- [Sembiance v3: paired TrID measurements](reports/2026-09-09-7599ca96-trid-sembiance/trid-comparison.md)

Hydrate the snapshot by SHA-256 into `CORPUS_FILES`, then run:

```sh
rules/.venv/bin/magika-compare \
  --config /path/to/config.json --inputs /path/to/inputs.json.gz \
  --files-root /path/to/CORPUS_FILES --workloads /path/to/workloads.json \
  --hyperfine /path/to/hyperfine --revision "$(git rev-parse HEAD)" \
  --output /path/to/new-run-directory
```

Omit `--workloads` only to generate a new seeded suite. Existing outputs cannot
be overwritten. The inputs contain every sample hash, size and adjudicated
label; corpus bytes and local caches are not published in benchmark results.

Store the completed evidence, then render the table without executing tools:

```sh
PYTHONPATH=rules/benchmark/src rules/.venv/bin/python rules/benchmarks/store.py \
  /path/to/new-run-directory rules/benchmarks/results/v1/RUN_ID
PYTHONPATH=rules/benchmark/src rules/.venv/bin/python rules/benchmarks/full_table.py \
  rules/benchmarks/results/v1/RUN_ID rules/benchmarks/reports/RUN_ID
PYTHONPATH=rules/benchmark/src rules/.venv/bin/python rules/benchmarks/catalog.py
```

For a paired TrID run, keep `trid`, `trid-stringzilla` and the rules-only control
in the config, and supply the saved natural workloads. Generate the acceleration
comparison from the stored evidence:

```sh
PYTHONPATH=rules/benchmark/src rules/.venv/bin/python rules/benchmarks/trid_comparison.py \
  rules/benchmarks/results/v1/RUN_ID rules/benchmarks/reports/RUN_ID
```

This checks matching interpreter bytes, TrID script, definitions, non-acceleration
settings and exact workload inputs/trials, and records normalized output
differences alongside the arithmetic speedups. Each run retains its own UTC
timestamp; the paired comparison does not substitute new timings into older runs.

`inputs.json.gz`, `workloads.json`, `config.json`, `label-mappings.json`, normalized
observations, compressed raw tool output and Hyperfine JSON are retained with
SHA-256 receipts. Reports are regenerable from this evidence. The history API
refuses speedup claims across changed datasets, workload lists, machines,
execution settings, protocol or benchmark code.

The CPU-warmup follow-up replays the seven natural file-count workloads on both
datasets and reevaluates all eligible files. Unchanged external-tool quality
output is reused only when its input, command, environment and artifact identity
match; every timing is measured again. The baseline retains the additional
controlled rule-hit mixtures. The superseded `a8429b03` candidate remains in the
catalogue so its bulk regression is visible.

Generate the scoped startup-change comparison from two stored runs:

```sh
PYTHONPATH=rules/benchmark/src rules/.venv/bin/python rules/benchmarks/startup_comparison.py \
  rules/benchmarks/results/v1/BASELINE rules/benchmarks/results/v1/CURRENT \
  rules/benchmarks/reports/CURRENT
```

This checks identical natural input lists, host, tool flags, default resource
policy and timing controls. It covers the documented 1.2.0-to-1.2.1 validation
update and CPU-warmup configuration change; the general history compatibility
gate remains strict. Percent changes are calculated from saved medians.

The former fixed-thread comparison tables have been removed. The
[original performance handoff](handoffs/perf-handoff-2026-09-08.md) and
[measured implementation decisions](reports/2026-09-09-handoff-validation/dispositions.md)
remain preserved as engineering evidence.
