# Sampling methodology

How samples are found, admitted and counted today. Every sample in the corpus, whatever its
source, goes through the same admission: its bytes are fetched into the content-addressed
store, the validators observe them, and the label ladder in the [README](README.md#labels)
decides what they are.

## The contract

- A discovery hint is never a label. A filename, a VirusTotal marking or a query match only
  decides what to fetch and which validators to run.
- The target is 100 samples per class, counted the way `select` draws them: verified,
  within ten per repository, and not ssdeep near-duplicates of each other. The target is
  never a reason to weaken a label or a duplicate check.
- Failures are kept as facts. A class with no findable or provable samples stays short; its
  shortfall is reported, not papered over with a weaker label.
- Refill only adds. It never relabels or replaces a sample already held; relabelling is
  `validate --apply`, and it reaches a fixed point.
- Every sample records its SHA-256 and a permanent origin: a GitHub blob pinned to a full
  commit, a VirusTotal hash, or a reproducible `generated:` origin.

## Standing decisions

These were settled with the maintainer. They are the defaults; do not re-ask them.

**Labels are hints; validators decide.** Every detector label — VirusTotal's type tag,
Magika, TrID, libmagic, a filename, a discovery query — is a hint. It chooses what to
fetch and which validators to run. Identity comes from the label ladder in the README:
human decision, model review, structural proof, then agreement of two non-Magika tools,
then a pinned source path. Magika is never one of the agreeing tools.

**Conflicting labels are wanted.** A file where Magika, TrID and VirusTotal's type tag
disagree is a hard case, not noise. Fetch it on purpose, let the validator settle it, and
keep the disagreement as `detectors_disagree` and `hard_case`. Disagreement never vetoes a
proof.

**VirusTotal samples may be used.** VirusTotal origins are permitted by the corpus policy
and carry no repository licence. Only GitHub samples are licence-gated.

**GitHub samples must be licence-clean.** A GitHub sample stays only when its repository's
licence is resolved and in `config/repository-license-policy.json`; an unresolved licence
is not a permissive one. At most ten samples per repository per class. `prune` enforces
both; `refill` can only draw from permitted repositories.

**Unverified samples stay, as unknown.** A sample nothing verified has `format_id =
"unknown"` and `label_status` `need_review` or `conflicting`. Never a guessed label, never
a separate staging corpus.

### Finding samples on VirusTotal

VirusTotal returns its own type tag, Magika label, TrID ranking and libmagic string with
every search result, so a file's probable class is known before it is downloaded.

```bash
uv run --no-sync --env-file .env magika-datasets refill --source virustotal
```

For each short class, searches run most-specific first:

| Query | Finds |
| --- | --- |
| `type:<format>` | files VirusTotal types as the class |
| `magika:<label>` | files Magika labels as the class |
| `magika:<label> NOT type:<format>` | Magika says the class, VirusTotal disagrees — hard cases |
| `type:<format> NOT magika:<label>` | VirusTotal says the class, Magika disagrees — hard cases |
| saved queries in `config/vt-queries.json` | filename and marking queries, weakest signal |

`magika:` only matches Magika's own output labels, and `type:` only formats VirusTotal
knows; a class outside both relies on the saved queries. Negation is `NOT`, not `-`.
Every download is validated and admitted only when it lands in a real class. For a class
with no structural validator, a file can only be labelled by two non-Magika tools, so its
markings are checked before download and the rest are not fetched.

`VT_API_KEY` lives in `dataset/.env`, which is gitignored. Never delete a `.env` during a
clean-up: it holds credentials that are not in git and cannot be recovered from it.

## Where samples come from

**GitHub.** `refill` (the default source) draws leads from `repository-files.parquet`, the
frozen inventory of 7,115 repositories, and only from repositories whose licence the policy
permits. A suffix exactly one class owns makes a lead for that class; a suffix several
classes share makes a lead only for classes a validator can prove, which is what then
decides. Leads are deduplicated by Git blob id, including against bytes already held, and
the per-repository cap is checked both when planning and again for the class a file
actually lands in. Files over 8 MiB are never planned.

**VirusTotal.** `refill --source virustotal` searches as described above, class by class
with eight workers, and stops a query after three pages that add nothing.

**Generated.** `refill --source generated` makes the classes no file in the wild belongs to,
as the README describes.

```bash
uv run --no-sync magika-datasets refill
uv run --no-sync --env-file .env magika-datasets refill --source virustotal --format png
uv run --no-sync magika-datasets refill --source generated
uv run --no-sync magika-datasets prune
uv run --no-sync magika-datasets validate --metadata . --store local/corpus \
  --output local/validation.parquet --apply
```

Every admitted sample gets an ssdeep fingerprint of its own bytes, so near-duplicate
exclusion never depends on a detector's copy of one.

## Label corrections

Reproducible sampling does not require keeping a wrong label. Source filenames,
VT search markings and detector outputs are evidence to review. Correct a label
when format-specific evidence supports the change; do not relabel solely to match
the detector being evaluated or to reach the class quota. Preserve original tool
verdicts alongside the resolved annotation. Unresolved identity conflicts require
review; an unsupported detector or generic container description alone does not
contradict a more specific supported identity.

Each correction must identify the sample SHA-256, old and new labels, supporting
evidence, reason and policy/reviewer version. Record any resulting class-count or
selection change. A changed admission rule needs a new version and review of its
affected samples, with earlier checkpoint evidence retained. Parser failure alone
does not justify an `invalid` label, and accepting an identity does not establish
that the whole file is structurally valid.

## Hard cases

`hard_case` marks a sample several sources read differently, including one whose label a
validator settled over a disagreeing detector. It is computed from the evidence the sample
carries on every validation, never inherited. It is a slice to study, not an exclusion
filter: a benchmark that drops its hard cases measures the easy files.

## What can be reproduced

**Bytes.** Every sample's SHA-256 and origin let anyone fetch and verify the same file,
subject to the source staying available and, for VirusTotal, the reader's access. Generated
samples regenerate from their origin alone.

**Draws.** A GitHub refill plan is a function of the frozen inventory, the licence table, the
samples already held and the classes' targets, so the same inputs plan the same leads.
VirusTotal searches are not a uniform draw from VirusTotal: results arrive in the provider's
order and a query's total is its estimate.

Nothing here makes the corpus a holdout. Samples were found with Magika's labels among the
hints, so measuring Magika on this corpus does not establish unbiased accuracy.
