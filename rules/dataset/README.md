# Dataset builder and hydrator

This directory is the complete public Python project. It builds reproducible
whole-file datasets from GitHub and VirusTotal, using Parquet for both public
metadata and local byte shards. It has no dependency on the research workspace
in `datasets/`, TensorFlow, or Sedpack. Python 3.12+, uv and Git are required;
the background runner currently supports POSIX systems (Linux and macOS).

## Install

From the repository root:

```sh
cd rules/dataset
uv sync --python 3.12
cp -n env-config .env
```

Add your credentials to `.env`; `env-config` explains how to obtain them. VT
Intelligence search and file download permissions are required for VT collection.
Credentials, downloaded bytes and Parquet metadata remain ignored. Metadata is
published separately in cloud storage, not Git LFS. The hosting destination has
not been selected yet; `metadata-downloads.json` records the required filenames,
full SHA-256 checksums and sizes, with a download URL to be filled before release.

The distribution is one `dataset-metadata.tar.gz` archive, with a companion JSON
receipt containing its SHA-256 and size. Download both from the published bucket
or Drive folder, check the archive hash, and unpack it in `rules/dataset/`:

```sh
shasum -a 256 dataset-metadata.tar.gz  # Compare with the distribution receipt.
tar -xzf dataset-metadata.tar.gz
```

`archive-manifest.json` inside the archive lists every member's SHA-256 and size.
The baseline files live at the archive root; completed candidate metadata lives
in `candidate-metadata/`. The versioned receipts bind the baseline to its expected
snapshot. No external research files are needed to hydrate it.

Maintainers can create the bundle with:

```sh
uv run --no-sync magika-datasets archive --output .local/dataset-metadata.tar.gz
# Include a completed candidate export:
uv run --no-sync magika-datasets archive --output .local/dataset-metadata.tar.gz \
  --candidates candidate-metadata
```

Only allowlisted metadata is included. Archive creation verifies checksums and
reads the archive back before atomically publishing it. The companion receipt is
written as `dataset-metadata.tar.gz.json`. This archive is excluded from Git.

## Hydrate the published dataset

```sh
uv run --no-sync --env-file .env magika-datasets hydrate \
  --download --workers 8 --output .local/corpus-snapshot
uv run --no-sync magika-datasets stats --by-class
```

Hydration downloads missing objects from their permanent origins, verifies full
SHA-256 and size, and writes verified Parquet shards. Interrupted downloads resume
from verified objects. Use `--verify-only` with the same output to check a snapshot.
The published baseline contains 29,523 accepted samples. Its labels are historical
human/review decisions, not decisions made by this builder.

## Build new candidates automatically

Configure the eight detector commands in `config/tools.json` for your installations.
The adapters are included; upstream detector executables, models and signatures
are not redistributed here. Paths are local to this directory. Missing tools are
recorded as `unavailable`, not as agreement or successful classification. Tool
versions, artifact hashes, errors and disagreements stay in the metadata.

```sh
uv run --no-sync --env-file .env magika-datasets build start \
  --config config/build.json --run-dir .local/build
uv run --no-sync magika-datasets build status --run-dir .local/build
# After interruption:
uv run --no-sync --env-file .env magika-datasets build resume --run-dir .local/build
```

One detached program runs the stages without an agent:

1. Search and download VT disagreements first, including already-full classes:
   up to 100 candidates per format, bounded at 20 search pages per class.
2. Fill ordinary coverage using the frozen GitHub inventory and VT queries.
   GitHub is preferred for text/code; VT for other formats, with provider fallback.
3. Reuse cached observations and run missing detector observations, conflicts first.
4. Export every candidate and hydrate verified whole-file Parquet shards.

There are eight finder workers, eight download workers and one database writer.
New GitHub files are capped at 256 KiB and VT files at 1 MiB. GitHub sampling
filters licenses and favors repository diversity. Provider failures back off with
Tenacity, capped at five minutes except for explicit server retry timing. Status
is read-only. No scheduler or model is required; an external supervisor can use
`--foreground`.

Outputs are `.local/build/snapshot/shards/` for bytes and `candidate-metadata/`
for public metadata to upload separately to cloud storage. The latter includes `github-used.parquet`, full origins,
all detector observations and conflict flags. Rehydrate that snapshot with:

```sh
uv run --no-sync --env-file .env magika-datasets hydrate --download \
  --samples candidate-metadata/samples.parquet \
  --classes candidate-metadata/classes.parquet \
  --receipt candidate-metadata/corpus-parquet-receipt.json \
  --output .local/rebuilt-candidates
```

Reconciliation is manual. Conflicting and unresolved samples remain present;
collection does not assign accepted labels or replace the baseline. The target is
100 accepted samples for each of 472 formats plus an explicit `invalid` class;
source exhaustion is reported as a shortfall. Invalid labels require human defect
evidence. Candidate pool bucket IDs are storage groups, not taxonomy labels.

## Public inputs and provenance

| File | Purpose |
| --- | --- |
| `classes.parquet`, `samples.parquet` | Baseline taxonomy, accepted samples, full SHA-256, permanent origins and observations |
| `github-used.parquet` | Repositories actually represented in the baseline, distinct sample counts and pinned revisions |
| `repositories.parquet` | Frozen source pool from GitHub Ranking, GitHub Leaderboard and Awesome leaf lists, with provenance and license/crawl outcomes |
| `repository-files.parquet` | Pinned repository inventory: path, size, Git object identity and license evidence |
| `*-receipt.json` | Hashes binding metadata to the frozen snapshots |
| `config/collection.json`, `config/vt-queries.json` | Reproducible selection settings, seed and saved VT queries |

Repository license observations are not file-level license adjudication. Empty
license observations mean missing evidence. Inventory observations retain their
own revision; they do not imply that another sample revision has the same license.
Full per-file GitHub URLs and VT SHA-256 origins are in `samples.parquet`.
Acquisition remains subject to upstream availability and your provider permissions.

## Python reader

```python
import pyarrow.dataset as ds

corpus = ds.dataset('.local/corpus-snapshot/shards', format='parquet')
for batch in corpus.scanner(columns=['sha256', 'format_id', 'content'], batch_size=16).to_batches():
    for sample in batch.to_pylist():
        content = sample['content']  # Original bytes; never execute them.
```

## Development and PR boundary

```sh
uv run --no-sync pytest -q
uv run --no-sync ruff check src tests
uv build
```

Only `rules/dataset/` belongs in this PR. Research scripts, historical experiments,
local runs, credentials, Parquet metadata and corpus bytes stay outside the change.
Only their checksum manifest and small receipts are versioned; cloud-hosted metadata
files are supplied separately. The source tree
separates orchestration (`pipeline`), discovery (`unattended_sources`), durable
acquisition (`unattended`, `parallel_acquisition`), observations (`verdicts`),
hydration (`hydrate`, `parquet_corpus`), candidate export (`candidate_export`), and
observation cache reuse (`observation_cache`). No automated reconciliation is shipped.
Configuration is checked before a background run starts, and saved runs pin all
runtime modules to prevent silently resuming with changed acquisition policies.

## Repository pool and future rebalancing

All **12,162** discovered repositories are listed in `repositories.parquet`,
including **7,115 indexed**, **4,122 license exclusions**, **812 failed crawls**
and **113 ineligible repositories**. `github-used.parquet` is only the contributing
subset, never the sampling allowlist. Every collection plan reports the full pool
and eligible counts. Failed/excluded entries stay visible; using them requires a
successful inventory refresh and license eligibility, not simply a larger target.

For a future training corpus, `config/rebalance.json` enables a bounded diversity
pass, including classes already at 100. It proposes up to 20 new GitHub candidates
per class from the entire eligible inventory. Repositories unused for that class
come first; class/category/corpus usage weights diversify the remainder. Existing
objects are not downloaded again. This configuration does not start another VT
search pass. Candidates still receive detector observations and conflict flags.

No rebalance is launched automatically. To opt in later, use `build start` with
`--config config/rebalance.json` and a fresh `--run-dir .local/rebalance`.
Set `target_per_class` in a normal collection config to grow beyond 100 instead.
Use a new run/config for each draw; do not edit a frozen running configuration.
Both paths preserve existing accepted samples and produce candidates for manual
review. The present validation corpus remains usable while disagreement collection
continues. No automatic relabeling, replacement, or training/validation split is
performed by rebalance.

## Structural validation and review groups

Validators live under `src/magika_datasets/validators/<family>/<module>.py`. Each
module declares `FAMILY`, `FORMAT_IDS` and `SCOPE`, and returns a typed
`Observation` (pass, fail or inconclusive) or `None` when the bytes are not its
format. A pass means the checks declared in that module's `SCOPE` succeeded. Coverage
varies: some validators decode payloads and verify checksums, while others check
container structure, headers and referenced extents. It is not a guarantee that
all payload semantics or checksums were checked. Budget exhaustion and unavailable
decoders are `inconclusive`.
Containers are walked once and name the most specific provable
format, so a JAR is reported as `jar`, not also as `zip`.

Current coverage: PNG (chunks, CRCs, bounded zlib, scanlines); JPEG, GIF, BMP,
TIFF, WebP and JPEG 2000 (Pillow 12.3.0 verification and all-frame decoding in a
ten-second subprocess: 256 frames, 32 million pixels/frame, 256 million total); SVG
(safe XML, namespace, viewBox); WAV (PCM frames); JSON and TOML (complete parse,
hint-gated); gzip, bzip2, xz and bare zlib (complete bounded inflation with
CRC/Adler/index verification, concatenated members); tar (header checksums, block
accounting, zero-block terminator); and the ZIP family (central directory, member
CRCs, safe names) dispatching to docx, dotx, xlsx, xlsb, pptx, visio, nupkg, msix,
3mf, epub, odt, ods, odp, apk, xpi, jar, kmz, crx and qgis from their manifests.

Images: ico and cur (directory bounds, every size decoded), tga (hint-gated,
header sanity), Netpbm, psd (composite decoded, layers not), icns, qoi, jng
(chunk CRCs plus JPEG stream decode), wmf and emf (record walks to EOF), bpg,
boxed jxl (container only), xcf (layer, channel and tile offsets). Media
containers walked once and dispatched: RIFF (wav, avi, ani, webp), ISO base
media (mp4, 3gp, qt including ftyp-less QuickTime, heif with container checks, avif with pixel
decode), EBML (mkv, webm). Streams: flv (tag chain), asf, mpegts (packet
lattice), ogg (page CRC-32), swf (declared length after bounded inflate, tag
stream), flac (frame CRC-8 and CRC-16), mp3 (frame chain with ID3 and APE
tails), midi (every track to End of Track), au. Fonts: ttf and otf including
collections (table checksums and whole-file coverage), woff (tables inflated and
checksummed), woff2 (directory and compressed block accounting). Also lnk
(every structure to the terminal block) and icc profiles.
Budgets: 16 MiB input (the largest baseline sample is 8.4 MB; 818 baseline
media files exceeded the former 1 MiB cap), 64 MiB expanded, 4096 members or
chunks, 16 MiB per ZIP member, 256 frames and 32 million pixels per frame for
Pillow with a ten-second subprocess limit. Unsupported PNG animation checks remain inconclusive. Unsupported formats
are explicitly unknown. See the [PNG specification](https://www.w3.org/TR/png-3/)
and [WebP container specification](https://developers.google.com/speed/webp/docs/riff_container).

```sh
uv run --no-sync magika-datasets validate --metadata . --store .local/corpus \
  --output .local/validation.parquet
# Or add --validate to a hydrate command.
```

Validation metadata is joined to samples by full SHA-256 and assigns one review group:

- `validated_auto`: exactly one eligible scoped validator proves the format and
  nothing fails. This settles the label even when detectors disagreed; the
  disagreement is kept as evidence, the sample is tagged `detectors_disagree`
  and stays a hard case, because a file that fools detectors but is structurally
  proven is exactly what gives the benchmark discriminative power. A pass does
  not declare every possible property valid.
- `conflicting`: detectors disagree and no validator settles it, two validators
  prove different formats, or a validator fails while another passes.
- `llm-validated`: an attributed model review with a validated decision, format IDs
  and supporting evidence. No status is granted merely because an LLM ran.
- `unknown`: unsupported, skipped, inconclusive, unavailable or insufficient evidence.
- `validated_manual`: explicit `manual_validation` with a reviewer, `validated`
  decision, known format IDs and evidence; historical accepted labels alone do not imply this.

Groups are logical, not physical moves or deletion. Existing accepted labels and
bytes stay intact. New candidate exports include `validation_status` in annotations;
revalidation of existing snapshots writes a separate `validation.parquet` table.
The metadata archive includes these tables when present. Validation tables are observations from the run that produced them, not a claim
that the latest code has revalidated the corpus. Run `validate` to refresh them;
new observations carry validator version 4. Counts are reported in the companion
JSON summary. Corpus bytes and historical labels remain available during review.

Executables and system files: pe (headers, sections, directories, Authenticode
overlay, checksum, with dll/driver/dotnet/machine/signed tags), elf and cubin,
macho including fat binaries, coff and xcoff (hint-gated), dex (Adler-32 and
SHA-1), Java class files, Python bytecode (native marshal walk for 2.7 to 3.14),
Lua 5.1 to 5.4 and LuaJIT chunks, wasm, SPIR-V, LLVM bitcode, BEAM, PDB (MSF 2,
MSF 7 and portable), minidump, registry hives (`hve`) and exports
(`winregistry`), DS_Store, AppleSingle/AppleDouble, PalmOS PDB/PRC
(hint-gated), UF2 and Intel HEX.

Archives and packages: ar and Debian packages, Microsoft Cabinet (CFDATA
checksums, Authenticode tails), 7-Zip (both header CRCs), RAR 4 and 5 (every
header CRC; encrypted headers stay inconclusive), xar (TOC digest, archived
checksums), WIM (lookup tables, XML data, solid resources, Authenticode blobs),
SquashFS 4 superblocks, Zstandard and LZ4 framing (xxHash-32 header and block
checksums), Unix compress (complete LZW decode), LHA levels 0-2, ACE, SZDD/KWAJ,
RPM (MD5 and SHA header digests), ISO 9660 (2048-byte and raw 2352-byte
sectors) and Apple disk images (koly trailer, plist, blkx chunk tables, data
fork CRC, code-signature blob). Compound files (CFB) are walked once with FAT,
mini-FAT and directory chains verified and dispatched to doc, xls, ppt, msi,
visio, hwp, outlook, thumbsdb or generic `ole`; OneNote sections and
notebooks by expected file length and chunk references. Applications: WAD,
WinHelp (B+tree directory, internal file extents), CHM (section sizes,
directory listing), Windows catalogs (full DER walk to the certificate trust
list), 3DSX and odex (embedded DEX verified; dexopt leaves the SHA-1 stale).

Data and science: SQLite with `PRAGMA integrity_check` on an immutable copy
(dispatching GeoPackage and MBTiles), Parquet, ORC and Arrow through pyarrow
metadata in a subprocess, Avro, npy and npz, safetensors, GGUF, Keras archives,
pickle and PyTorch checkpoints walked with `pickletools` (never loaded), FITS,
netCDF classic, MATLAB v5, DICOM, NIfTI, NRRD, GRIB, shapefile (.shp and .shx),
dBase, pcap and pcapng, torrent, Redis RDB, SPSS, SAS and Stata 117+, DuckDB
(64-bit header checksums), ESE (header XOR and shadow), Access page lattices,
MyISAM indexes and InnoDB tablespaces, PostgreSQL custom dumps (TOC and data
blocks), HDF4 descriptor chains, HDF5 superblocks (lookup3 checksum, EOF
address), ONNX (protobuf wire walk with field types) and TensorFlow Lite
(flatbuffer tables and vectors). Geometry: Blender, 3DS, PLY, STL, FBX, binary
glTF, DWG (R2000 locator CRC; R2004+ decrypted header, LZ77 page map and
section map with checksums), LightWave scenes and objects, Maya ASCII and
binary. Text families: XML parsed once and dispatched to svg, gpx, kml, gml,
collada, rdf, xsd, plist, mum and osm (plus OSM PBF); JSON dispatched to
GeoJSON, notebooks, glTF and JSON Lines; binary plists; YAML; hint-gated ONNX identity, INI,
CSV/TSV, diff, obj/mtl, go.sum, LaTeX (mismatches inconclusive) and Maya-style
grammars; iCalendar, PEM and DER certificates, SubRip, WebVTT, M3U, gettext
catalogs, RTF, PostScript and PDF (both naming Illustrator files), eml and
MHTML, STEP, IGES, DXF, VTK, PCD, USDA, Arc/Info grid, LAS/LAZ, Internet
shortcuts, Protein Data Bank records, BibTeX and Jest/Vitest/Bun/insta
snapshots. A generic container pass (plain `zip`, gzip, bzip2, xz, bare zlib,
zstd, lz4, compress, `ar`, `ole`, HDF5, SquashFS) never relabels a sample
hinted as a more specific format the validator could not prove, and validators
whose applicability rests on a short prefix (JSON, SVG, bare zlib) only report
failures for samples hinted as that format.

Strong validator results assign `format_ids` in new candidate exports and preserve
`previous_format_ids` plus `label_decision`. Conflicting detector reports stay
visible independently. Revalidation tables include `assigned_format_ids` and
`label_decision_json` for existing snapshots. Workflow order is automatic validators,
then evidence-backed LLM review, then human review where still necessary.

GeoTIFF is represented as `format_id: tiff` with a `geotiff` tag. The tag is
inherited from existing metadata; generic TIFF decoding validates TIFF, not the
georeferencing semantics. All 100 former GeoTIFF samples are retained, giving TIFF
200 samples and 473 canonical classes including invalid. Historical discovery
labels remain provenance, not separate canonical types.
