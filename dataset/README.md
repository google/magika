# Magika evaluation dataset

A whole-file evaluation corpus: **71,296 samples across 474 format classes**, 12.5 GiB of
original bytes, every sample carrying its full SHA-256, its permanent origin, the
observations of eight detectors, and a label status saying what verified it. **47,914
samples across 389 classes carry a verified label**; the rest are in the corpus and marked
as unverified rather than dropped or guessed.

What it is not: a holdout. Samples were admitted with help from Magika's own predictions,
so measuring Magika against this corpus does not establish unbiased accuracy. It is for
comparing detectors, finding disagreement, and checking that a change does not regress a
format. The samples that several sources read differently are the interesting ones, and
17,152 of them are flagged `hard_case` for exactly that reason — computed from the
evidence each sample actually carries, not inherited from an earlier pass.

Two things go downstream: hydrated Parquet shards, which the detector benchmark in this
project runs on, and [REPOSITORIES.md](REPOSITORIES.md), the licence of every GitHub
repository the corpus draws from.

## Structure

```
dataset/
  src/magika_datasets/     the builder: acquisition, validation, labelling, export
    validators/            194 structural validators in 11 families
    benchmark/             Magika against libmagic and TrID on a hydrated snapshot
  tests/                   660 tests, no network, no corpus bytes
  config/                  VT queries, licence policy, taxonomy and label corrections
  local/                   gitignored working state: corpus/objects/<2>/<64>, caches, runs
  *.parquet                metadata: gitignored, distributed separately (see below)
  README.md                this file
  SAMPLING.md              how samples are found and admitted
  REPOSITORIES.md          generated: every source repository and its licence
```

Nothing in Magika's build reads anything here.

### Modules

| Area | Modules |
| --- | --- |
| Acquisition | `acquisition`, `parallel_acquisition`, `manifest`, `virustotal`, `git_io`, `runtime` |
| Corpus policy and refill | `prune`, `refill`, `github_leads`, `virustotal_leads`, `generated`, `generated_leads`, `receipt` |
| Detector evidence | `detectors`, `detector_conflicts` |
| Structural validation | `validators/`, `validation` |
| Labelling | `validation`, `adjudication`, `origins`, `normalization`, `filenames` |
| Metadata and snapshots | `parquet_metadata`, `parquet_corpus`, `hydrate`, `archive`, `fingerprints`, `stats` |
| Evaluation set | `selection`, `duplicates`, `repository_diversity` |
| Provenance | `sources`, `licenses`, `github_api` |
| External collections | `sembiance` |
| Taxonomy | `taxonomy` |
| Benchmark | `benchmark/` |

## Metadata distribution

`*.parquet` is gitignored. 140 MB of derived tables belongs in neither git nor LFS, and
every one is reproducible from the corpus store and the tracked configuration. The metadata
ships as one archive:

[`magika-eval-dataset-v1.0-metadata.zip`][metadata] — 188 MB, SHA-256
`a03d46f80927f36beb8d3b5324ac97929a4a62722b262afcd3c1a99324341c27`. Download it into
this directory and unzip it there. It holds the corpus tables, the repository and licence
tables, their receipts and the `sembiance/` collection; no corpus bytes and no
credentials.

[metadata]: https://drive.google.com/file/d/1TdGZJpx6KIUeocf4WofrnEkCRRPj4fI4/view?usp=sharing

To rebuild it:

```bash
uv run --no-sync magika-datasets archive \
  --output local/magika-eval-dataset-v1.0-metadata.zip --download-url URL
```

`archive` refuses any table its receipt does not describe, then rewrites
`metadata-downloads.json` from what it packaged, so the download list names each file with
its SHA-256 and the hosting location. Members are stored at a fixed timestamp, so the same
tables rebuild the same bytes. Check the archive hash before unpacking. Receipts,
configuration and the generated Markdown are tracked; bytes and credentials never are.

## Quickstart

```bash
cd dataset
uv sync
shasum -a 256 magika-eval-dataset-v1.0-metadata.zip    # the hash above, before unpacking
unzip magika-eval-dataset-v1.0-metadata.zip
uv run --no-sync magika-datasets stats
uv run --no-sync magika-datasets hydrate --verify-only --output local/corpus-snapshot
```

No credentials are needed for any of that, or for resolving licences, or for the tests.
Credentials are needed only to acquire new bytes.

## Keys

```bash
cp -n env-config .env
```

`.env` is gitignored; `env-config` is the tracked reference and keeps empty values. Existing
environment variables win.

**`VT_API_KEY`** — a **premium** VirusTotal account. Intelligence search and file download
are premium endpoints; a free key cannot run this workflow. Every request spends quota, so
start any new query with a small `--limit`.

**`GITHUB_TOKEN`** — a fine-grained token with **public repository read and no other
permission**. Never a token with write or admin scope. Downloading public Git blobs works
without it; it is required only for `licenses --resolve-residual`.

```bash
uv run --no-sync --env-file .env magika-datasets refill --source virustotal --format png
```

## Hydration

Hydration turns the metadata plus a local byte store into verified whole-file Parquet
shards.

```bash
# Verify an existing snapshot against the metadata.
uv run --no-sync magika-datasets hydrate --verify-only --output local/corpus-snapshot

# Fetch missing bytes, then build shards.
uv run --no-sync --env-file .env magika-datasets hydrate \
  --download --workers 8 --output local/corpus-snapshot
```

The byte store is content-addressed at `local/corpus/objects/<first two hex>/<sha256>`, so
a sample present for one class is not fetched again for another. `--verify-only` re-reads
every object and compares size and digest; `--download` refetches only what is missing or
wrong. Nothing is trusted transitively: `corpus-parquet-receipt.json` binds the metadata
files, the shard `manifest.json` binds each shard and the row digest, and every sample's
SHA-256 is checked against its bytes.

Read the result with any Parquet reader:

```python
import pyarrow.dataset as ds

corpus = ds.dataset("local/corpus-snapshot/shards", format="parquet")
columns = ["sha256", "format_id", "label_status", "content"]
for batch in corpus.scanner(columns=columns, batch_size=16).to_batches():
    for sample in batch.to_pylist():
        content = sample["content"]  # Original bytes. Never execute them.
```

## Sampling

The corpus is built by a frozen pass, not an open-ended crawl. Snapshots, per-class counts,
the class ordering and the seed are all fixed before a pass starts, and no pass reruns with
a different seed — there is nothing to shop for. Failures, empty results and rejection
reasons are preserved rather than retried away, because a format with no findable samples
is a fact about the format. Code and text are sought on GitHub first and everything else on
VirusTotal first. A discovery hint is never a label.

The frozen pool is `repositories.parquet`: **12,162 repositories** — 7,115 inventoried,
4,122 excluded on licence, 812 failed crawls, 113 ineligible. `github-used.parquet` is the
contributing subset, never the allowlist. `refill` adds samples only to classes short of
their target and never relabels or replaces an existing sample.

Full contract in [SAMPLING.md](SAMPLING.md).

An evaluation subset is drawn separately:

```bash
uv run --no-sync magika-datasets select --per-class 100 --max-per-repository 10 \
  --output local/selection.parquet --receipt local/selection-receipt.json
```

Only verified samples are eligible. One repository may contribute at most ten samples to a
class, ssdeep near-duplicates are excluded against the representatives already chosen, and
the draw follows the table's own order so it is deterministic and takes no seed. Every
exclusion is counted by reason. The current draw is **28,097 samples across 324 classes**.

## Labels

Eight detectors observe every sample, 173 structural validators try to account for its
bytes, and a recorded human or model decision can override both. `label_status` says which
of those settled the label.

Validators live in `src/magika_datasets/validators/<family>/<module>.py`. Each declares
`FAMILY`, `FORMAT_IDS` and a `SCOPE` stating exactly what it checked, and returns a typed
`Observation` — pass, fail or inconclusive — or `None` when the bytes are not its format. A
**pass means the whole byte range was accounted for**: every chunk, member or block lies
inside the file, trailing bytes are rejected, and checksums are verified where the format
defines them. Header-only recognition is `inconclusive`. Containers are walked once and name
the most specific format they can prove, so a JAR is `jar` and not also `zip`. Input is
capped at 16 MiB, decoding runs in a subprocess, and a validator that raises produces an
`inconclusive` observation rather than aborting a corpus run.

173 modules across 11 families: data 41, text 34, archive 24, image 18, executable 15,
media 12, geometry 9, system 9, application 7, font 2, office 2. The registry refuses to
load if two modules claim the same format without both declaring it shared.

### Precedence

Stronger evidence settles the label. Weaker evidence that names a different format never
vetoes it — it is recorded as a `detectors_disagree` tag and keeps the sample a hard case,
because a file several sources read differently is what a benchmark is for.

| `label_status` | What established it | Samples | Ground truth |
| --- | --- | --- | --- |
| `validated_manual` | A recorded human decision: reviewer, decision and evidence | 250 | yes |
| `llm-validated` | A recorded model decision: model, decision, evidence and known formats | 0 | yes |
| `validated_auto` | Exactly one auto-eligible validator passed and none failed | 38,826 | yes |
| `validated_tools` | Two independent detectors agreed and no validator refuted them | 256 | yes |
| `validated_origin` | A pinned public source path names exactly one class | 8,582 | yes |
| `conflicting` | Sources disagree and nothing stronger settled it | 8,132 | no |
| `need_review` | No usable evidence | 15,250 | no |

**Magika is excluded from `validated_tools`.** A corpus that admitted the model's own
predictions as truth would measure agreement with itself. That costs coverage — roughly
21,900 samples where Magika is the only corroborating detector stay unverified — and the
cost is the point.

**`validated_origin` is evidence about provenance, not about bytes.** A GitHub blob pinned
to a full commit id, whose path suffix exactly one class owns, is the only evidence that
exists for source code: no magic number says "this is C", so every structural validator
declines and the detectors that map to a class read text poorly. Measured against the
curated labels it agrees on 6,867 of 6,893 samples. It ranks last, and it is refused by a
curated negative label, a validator refuting the class it names, or a detector naming
something else.

**Some classes are generated, not collected.** Magika names classes no file in the wild
belongs to: `empty`, `randombytes`, `randomascii`, `randomtxt`, and `symlinktext` (a
symbolic link stored as the path it points to, as Git stores one). `refill --source
generated` makes them with SHA-256 in counter mode, seeded only by generator, version and
index, and records all of those with the SHA-256 in a `generated:` origin. That origin
labels the sample only when regenerating it reproduces the hash, so anyone can check it.
`empty` can hold one distinct sample. `directory`, `symlink`, `null`, `undefined` and
`invalid` name filesystem objects or Magika results rather than bytes, so their target is
zero, and `mun` is merged into `mui`: the files are the same MUI resource images.

Unverified samples carry `format_id = "unknown"`, not a guess. Their discovery hints,
VirusTotal markings and detector output stay in `annotation_json` as evidence. There is no
separate candidate corpus, no `__candidate_pool_NNN__`, no `candidate-index.parquet`. A
legacy `label_status` string survives as `annotation_json.legacy_label_status` and is not
proof of anything: an unattributed "accepted" records that some earlier pass admitted the
file, not that anyone verified it.

**Every per-class count must filter on `label_status`.** `unknown` alone holds 26,799 files
against a target of 100. `stats` counts the five verified statuses by default, takes
`--label-status` to widen or narrow that, names the basis in its output, and always reports
the full breakdown per class.

```bash
uv run --no-sync magika-datasets stats --by-class
uv run --no-sync magika-datasets stats --label-status all
uv run --no-sync magika-datasets validate --metadata . --store local/corpus \
  --output local/validation.parquet
```

`validate --apply` writes the decisions back, and the result is a fixed point: run it again
and nothing changes. That is not free — hints gate which validators run, so deriving hints
from the label made labels oscillate. Hints come only from evidence fixed for the life of
the sample, and a validator's refusal of a format is remembered in `refuted_format_ids`,
because it is a statement about bytes and the bytes do not change.

A validation summary records the digest of the table it counted, so a run killed between
writing the two files leaves a detectable mismatch rather than a believable stale number.

### Reviewed corrections

`config/label-adjudications.json` holds per-sample human decisions, each with the reviewer
and where the decision is written down. Attribution is mandatory: an unattributed
correction is indistinguishable from the legacy labels this replaces. A correction may only
name a class the taxonomy defines, and `config/taxonomy-additions.json` is how a class is
added — appended at the next free ordinal, never renumbering, since samples reference a
class by ordinal. `config/taxonomy-corrections.json` redefines a published class's name,
categories, extensions, count target or the class it merged into, in place; each correction states its reason, and the values it
replaced stay in the class metadata.

## The Sembiance collection

A second collection comes from Sembiance's
[file-format samples](https://sembiance.com/fileFormatSamples/), the test files of
[dexvert](https://github.com/Sembiance/dexvert). Credit goes to Sembiance and dexvert; the
original creators keep their rights. It lives in `sembiance/` and uses the same taxonomy,
byte store, labelling, hydration and benchmark as the corpus; it is kept separate so each
can be measured on its own.

```bash
uv run --no-sync magika-datasets sembiance crawl     # resumable; downloads into local/corpus
uv run --no-sync magika-datasets sembiance export    # sembiance/{classes,samples}.parquet
uv run --no-sync magika-datasets fingerprint --metadata sembiance
uv run --no-sync magika-datasets validate --metadata sembiance --store local/corpus \
  --output local/sembiance-validation.parquet --apply
uv run --no-sync magika-datasets hydrate --samples sembiance/samples.parquet \
  --classes sembiance/classes.parquet --receipt sembiance/corpus-parquet-receipt.json \
  --output local/sembiance-snapshot
```

Each distinct file is one sample whose origins are the HTTPS URLs it was found at, so
`hydrate --download` refetches it and refuses bytes that no longer match. Sembiance's folder
names are a claim about the files, not a label: `config/sembiance-mapping.json` maps 131
reviewed folders to taxonomy classes, and that mapping only hints which validators to run.
The current import holds **33,421 files**; validators prove **2,118** of them, and the rest
stay `need_review`. 3,587 files over the earlier 1 MiB download cap are not yet fetched.

## Benchmarking

Hydration is the handoff. `magika-datasets hydrate --output DIR` writes:

- `DIR/manifest.json` — `format: "parquet-whole-file-v1"`, the source metadata receipt, and
  one entry per shard with its SHA-256 and sample count
- `DIR/classes.parquet` — the taxonomy
- `DIR/shards/*.parquet` — sample metadata plus a `content` column of original bytes

```bash
uv run --no-sync magika-datasets hydrate --output local/corpus-snapshot
uv run --no-sync magika-datasets benchmark-config --magika1 PATH --magika2 DIR --native LIBHS \
  --file /usr/bin/file --magic MAGIC.mgc --python PYTHON --trid trid.py \
  --trid-definitions triddefs.trd --output local/benchmark-config.json
uv run --no-sync magika-datasets benchmark --config local/benchmark-config.json \
  --snapshot local/corpus-snapshot --revision "$(git rev-parse HEAD)" --output local/benchmark-run
```

The benchmark compares Magika 1, Magika 2 on CPU, GPU and Auto with and without its rules,
libmagic and TrID. It reads the snapshot through the same shard checks `hydrate
--verify-only` uses, so no file is scored before its bytes and the metadata digest match,
and it scores only samples whose `label_status` is one of the five verified statuses.

- Every tool runs at its shipping defaults: resource flags and thread-cap environment
  variables are refused.
- A tool label maps to a class only through class metadata (Magika labels, MIME types for
  libmagic, extensions for TrID), never through the expected answer; a label that could
  mean two classes is ambiguous, not a guess.
- Accuracy is correct over all scored files, so unknowns and errors count against a tool;
  precision is correct over decisions; coverage is decisions over files.
- Timings come from Hyperfine, three runs after a warmup, for each workload of 1 to 1,000
  distinct files, whole-process and warm-cache. Each workload is first run once and must
  repeat the quality observation before it is timed.
- `results.json` records the snapshot, host, tool versions and executable and artifact
  hashes; `report.md` is rendered from it, and `--render` rebuilds a report without running
  anything.

## Licences

Every GitHub repository the corpus draws from is published with the licence reported for it
at the revision its samples were pinned to.

```bash
uv run --no-sync magika-datasets sources                      # repositories actually used
uv run --no-sync magika-datasets licenses                     # resolve, offline, no token
uv run --no-sync magika-datasets licenses --check             # is the published list current?
uv run --no-sync --env-file .env magika-datasets licenses --resolve-residual
```

Resolution is offline by default so anyone holding the metadata can reproduce the published
table. Evidence comes from `repositories.parquet`, whose licence was read at a pinned
revision, and from licence records carried on the samples; the pinned reading wins and the
sample's claim is kept as a cross-check and reported when it disagrees. `NOASSERTION` means
GitHub found a licence file it could not identify — a question asked, not answered — so it
does not close the question. `--resolve-residual` is the only path that touches the network;
it caches every answer, so a second run is offline and produces the same table.

**Every GitHub repository the corpus draws from has a resolved, permitted licence.** That
is enforced, not observed: `prune` drops any sample whose source repository is outside
`config/repository-license-policy.json` or whose licence could not be established, and
`refill` may only draw replacements from repositories the policy permits, so a refill
cannot reintroduce what a prune removed.

```bash
uv run --no-sync magika-datasets prune     # drop what may not be redistributed
uv run --no-sync magika-datasets refill    # replace it from permitted repositories only
```

Getting there removed 5,663 samples — 1,070 under AGPL-3.0, GPL-3.0, GPL-2.0, LGPL, MPL,
EPL and others outside the policy, and 4,593 whose repository licence GitHub could not
establish. The frozen inventory had excluded copyleft sources, so these had entered by
paths it never covered and only surfaced once the API was asked.

`prune` also enforces the sampling contract's cap of ten samples per repository per class,
so a single project's conventions cannot stand in for a format; before it did, one class
held 100 samples from a single repository. `refill` then drained every permitted lead the
frozen inventory offers — including classes recognised by a conventional name such as
`Makefile` or `.gitmodules`, deduplicated by Git object id so copied boilerplate is fetched
once, up to the corpus's 8 MiB sample ceiling — admitting only files that verify into a real class. Run in turn, the two reach a
state where refill adds nothing and prune removes nothing.

VirusTotal then filled the classes a validator can prove, searching by VirusTotal's own type
tag and Magika label and letting the validators decide (see SAMPLING.md, Standing decisions).
A class counts towards its target the way `select` would draw it: verified samples, within
the repository cap, and not near duplicates of one another by ssdeep. Counting raw rows had
let 299 copies of one generated ERDAS file fill a class. The remaining shortfall of 14,507
samples across 238 classes is mostly in classes with no structural validator, where a
VirusTotal file can only be labelled by two non-Magika tools agreeing; more validators, not
more searching, close it.

A sample acquired from VirusTotal is unaffected: those origins are permitted by the corpus
policy and assert no repository licence. What decides the terms is where the bytes came
from, not which mirror served them, so a file also present on VirusTotal does not escape
the licence of the repository it was taken from.

<!-- BEGIN licenses -->

| licence | repositories | samples | allowed by policy |
| --- | --- | --- | --- |
| 0BSD | 2 | 6 | yes |
| Apache-2.0 | 842 | 6376 | yes |
| BSD-2-Clause | 59 | 426 | yes |
| BSD-3-Clause | 158 | 1291 | yes |
| BSL-1.0 | 2 | 37 | yes |
| CC0-1.0 | 27 | 75 | yes |
| ISC | 24 | 198 | yes |
| MIT | 2085 | 9855 | yes |
| Unlicense | 33 | 168 | yes |
| Zlib | 17 | 109 | yes |

Full list: [REPOSITORIES.md](REPOSITORIES.md) (3,249 repositories).

<!-- END licenses -->

This is the licence of a repository, not of a file. File-level and dependency licences are
not resolved, a sample may carry its own terms, and VirusTotal origins carry no repository
licence at all. 1,424 repositories are unresolved and published as unresolved.

## Development

```bash
uv run --no-sync pytest -q
uv run --no-sync ruff check src tests
uv run --no-sync ruff format --check src tests
uv build
```

One logical change per signed commit. Generated output must be gitignored —
`runtime.private_output()` refuses to write anywhere tracked. A saved run pins every
runtime module's digest, so it cannot silently resume under changed acquisition policy.
Configuration is validated before a background run allocates anything.
