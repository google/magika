# Cross-tool benchmark

The [catalogue](CATALOG.md) records named/versioned datasets, UTC timestamps and
results for Magika 1, Magika 2 CPU/GPU/Auto, rules-only, libmagic and TrID with and
without StringZilla. The harness records observations and Hyperfine JSON; report
code calculates scores and timing comparisons from those records.

## Measurement contract

- Use each tool's shipping worker, reader and internal batch defaults. The harness
  rejects resource overrides and removes inherited thread-cap variables.
- Measure 1, 2, 5, 10, 25, 100 and 1,000 distinct files per invocation. Whole-process
  timing includes startup, I/O, output and shutdown. These are warm-cache runs.
- Use Hyperfine with no shell, one warmup and three measured runs. Shuffle
  tool/workload order with a saved seed; run without concurrent builds or tests.
- Verify file hashes, sizes and output order. Retain raw outputs, exit codes,
  executable/model/database hashes, commands and environment. Changed deterministic
  decisions fail validation. CPU-warmup configurations record per-workload
  inference differences because scheduling can select a different backend.
- Reuse exact saved workloads for comparisons. Record natural input mixes and,
  when requested, representable rule-hit ratios without duplicating files.

Accuracy is correct / all files; precision is correct / mapped decisions;
coverage is mapped decisions / all files. Unknowns and errors stay in the
accuracy denominator. Rule matches count deterministic, non-abstaining decisions.
Labels map through recorded metadata, never through the expected answer.

The combined snapshot contains 25,421 adjudicated files and is not the complete
V56 release. Sembiance v3 contains 33,421 source files, with 2,400 eligible for
this evaluation. Dataset construction and publication are maintained separately.

## Run

Install the harness and prepare the CLI/runtime distribution:

```sh
uv sync --directory rules --locked --extra comparators
python3 rust/build-runtime.py --gpu metal --output /absolute/path/to/magika2
```

Create two environments from the same Python build for the TrID comparison:

```sh
uv venv --python /path/to/base/python /path/to/trid-plain
uv venv --python /path/to/base/python /path/to/trid-stringzilla
uv pip install --python /path/to/trid-stringzilla/bin/python stringzilla==5.1.2
```

Keep the virtual-environment executable paths intact. The builder discovers
StringZilla's version and path; the harness checks TrID's loaded backend and
module hash before measurement.

```sh
PYTHONPATH=rules/benchmark/src rules/.venv/bin/python rules/benchmarks/default_config.py \
  --magika1 /path/to/released/magika --magika2 /path/to/magika2 \
  --native /path/to/libhs.dylib \
  --file /usr/bin/file --magic /usr/share/file/magic.mgc \
  --python /path/to/trid-plain/bin/python \
  --python-stringzilla /path/to/trid-stringzilla/bin/python \
  --trid /path/to/trid.py --trid-definitions /path/to/triddefs.trd \
  --dataset /path/to/dataset-descriptor.json --output /path/to/config.json

rules/.venv/bin/magika-compare \
  --config /path/to/config.json --inputs /path/to/inputs.json.gz \
  --files-root /path/to/corpus-files --workloads /path/to/workloads.json.gz \
  --hyperfine /path/to/hyperfine --revision "$(git rev-parse HEAD)" \
  --output /path/to/new-run
```

A dataset descriptor supplies `id`, `name`, `version` and optionally `source_files`;
its identity must agree with the supplied input snapshot. Corpus files are named
by SHA-256. Omit `--workloads` to generate seeded workloads. Existing measurement
directories cannot be overwritten. A focused TrID run keeps `trid`,
`trid-stringzilla` and the rules-only control in the config.

## Store and regenerate

```sh
PYTHONPATH=rules/benchmark/src rules/.venv/bin/python rules/benchmarks/store.py \
  /path/to/new-run rules/benchmarks/results/v1/RUN_ID
PYTHONPATH=rules/benchmark/src rules/.venv/bin/python rules/benchmarks/catalog.py
```

Each stored run contains compressed results (including configuration), inputs,
workloads, label mappings, raw outputs and a checksum receipt. Corpus bytes and
local caches are excluded. Markdown tables are committed for browsing; expanded
report JSON is generated locally, not checked in as a duplicate of the evidence.
The catalogue command rebuilds tables and registered comparisons from this data.

For the recorded CPU-warmup comparison, `comparison_base` in the index names the
baseline run. Its generator checks the same host, dataset, command flags, input
lists and timing controls. The general history API remains strict about changed
benchmark code or protocol. TrID comparisons additionally check identical Python
executable bytes, script, definitions and non-acceleration settings.

The earlier TrID rows were mislabeled as accelerated despite a recorded
`Using Stringzilla: False`. Their tables now use the observed configuration;
raw measurements remain unchanged. The new paired runs retain identical outputs:

- [Combined corpus TrID comparison](reports/2026-09-09-7599ca96-trid-combined/trid-comparison.md)
- [Sembiance v3 TrID comparison](reports/2026-09-09-7599ca96-trid-sembiance/trid-comparison.md)

Historical profiling experiments and the original review are linked from the
[preserved handoff](handoffs/README.md).
