# Rule quality and development evidence

Rules identify a format from at most 4 KiB. They do not validate the complete file.
All active bundled rules require at least eight observed bytes, including the
complete eight-byte empty WebAssembly module. This floor prevents decisions on
truncated magic; it does not make arbitrary padded magic safe. Format checks below
require related header fields, sizes, versions or container structure as available.

The review changes the YARA definitions directly. Sources are the pinned libmagic,
Tika, PRONOM and puremagic revisions in [LICENSES](LICENSES), supplemented by format
specifications. A signature copied from another tool remains evidence to examine,
not a reason to enable a weak alternative. Rules retain their individual source
references. Literal escape examples must never substitute for binary bytes.

## Reviewed format batch

The following counts use the existing parquet-v56 corpus of 29,523 distinct files.
Every materialized file's size and SHA-256 was checked before evaluation. These are
development samples used during fixes, not an independent holdout. None of these
rules produced an observed false positive in this batch; that is not a universal
precision guarantee. Unreviewed rules remain subject to the ongoing format audit.

| Format | Review decision | Correct matches / positives |
| --- | --- | --- |
| 3DSM | Require little-endian root and plausible version/main child chunk; remove TIFF collision. | 100 / 100 |
| AVI | Correlate RIFF, AVI form, LIST and header-list type. | 100 / 100 |
| BMP | Decode binary reserved fields; distinguish OS/2 core and uncompressed Windows DIB headers. | 98 / 100 |
| CAB | Require binary signature, reserved fields, cabinet version, folder count and bounded flags. | 56 / 100 |
| DWG | Require modern version marker, following reserved zeros and a 128-byte observed header. | 100 / 100 |
| EPUB | Require the first stored ZIP member to be the exact mimetype; correlate sizes, flags and optional data descriptor. | 96 / 100 |
| FLV | Require version, legal stream flags, standard data offset and first previous-tag size. Extended headers remain unsupported. | 100 / 100 |
| Gzip | Require compression method 8, legal flags and minimum member size. | 100 / 100 |
| ICO | Require ICONDIR count and a plausible first resource entry. | 53 / 100 |
| JXL | Retain the container signature and file-type box; bare two-byte codestream magic cannot enforce identity. | 14 / 17 |
| MPEG-TS | Check 21 complete 188-byte or 192-byte packets with sync and legal transport-header bits. | 100 / 100 |
| Ogg | Require version, initial-page flags, sequence and segment-table evidence. | 43 / 100 |
| PSD/PSB | Require version, reserved zeros, channels, dimension limits, depth and color mode. | 100 / 100 |
| RPM | Require a binary lead and signature-header magic at the expected offset. | 17 / 100 |
| SAS | Require SAS7BDAT binary magic and the SAS FILE marker, with header length and byte-order evidence. | 100 / 100 |
| 7-Zip | Require the binary signature, major version and complete start header. | 100 / 100 |
| SWF | Correlate FWS/CWS/ZWS version and length with RECT, zlib or LZMA header fields. Real ZWS fixtures cover both observed variants. | 100 / 100 |
| Unix compress | Require binary magic and legal LZW flags; retain the eight-byte prefix floor. | 47 / 48 |
| WebAssembly | Require exact magic and standard version; test all single-byte header mutations. | 100 / 100 |
| WAV | Correlate RIFF/RIFX/RF64 with WAVE and minimum container/header lengths. | 100 / 100 |
| ARC | Disabled: the reviewed short signature does not establish safe identity. | — |
| ORC | Disabled: a short leading ORC string is insufficient; the format also relies on footer metadata. | — |
| XCOFF | Disabled until a stronger header/section check replaces its two-byte magic. | — |

Specification references include [gzip](https://www.rfc-editor.org/rfc/rfc1952),
[EPUB ZIP requirements](https://www.w3.org/TR/epub-33/#sec-zip-container-mime),
[WebAssembly modules](https://webassembly.github.io/spec/core/binary/modules.html),
[Photoshop formats](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/),
[ORC](https://orc.apache.org/specification/ORCv1/) and
[pandas SAS constants](https://github.com/pandas-dev/pandas/blob/main/pandas/io/sas/sas_constants.py).

## Regression and evaluation requirements

The maintained regression suite includes the twenty original false-positive probes,
literal escape text, padded short magic, truncated public fixtures, malformed field
mutations, ZIP sibling confusion and real positive files. Native integration checks
the same positives and negatives against the actual product and YARA-X reference,
and asserts that product rules really executed. Source-only success is insufficient.

Changes are evaluated in batches against the complete ready corpus. Record the exact
source, binary, model and corpus identities with raw observations. Update affected
full/partial metadata from the measured counts. A partial rule still enforces its
matches; missed files fall through to ML. Enabled false positives, unadjudicated
matches, conflicts and engine discrepancies prevent qualification. The reviewed
batch introduced zero hybrid errors and corrected eighteen ML errors among the
21,014 samples in supported model classes; 8,509 additional-class samples were also
checked. Independent qualification remains pending.
