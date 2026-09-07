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
| Bzip2 | Require a legal block-size digit and the block marker or empty-stream end marker; compare all nine stdlib compression levels. | 100 / 100 |
| QOI | Require dimensions, channel count, colorspace and the minimum complete encoded image size. | 13 / 13 |
| GLB | Require version 2, aligned lengths and a first JSON chunk. Version 1 is outside this reviewed variant. | 100 / 100 |
| WOFF | Require a complete header and first directory entry, legal reserved fields and a common sfnt flavor. | 100 / 100 |
| WOFF2 | Require header, directory/payload space, table and compressed-size evidence. Reserved-field violations abstain even though tolerant decoders may accept them. | 100 / 100 |
| NumPy | Distinguish v1's 16-bit header length from v2/v3's 32-bit length; require aligned dictionary-header evidence without fixing key order. | 100 / 100 |
| MIDI | Require header format, track count, valid timing division and first track header. Extended header chunks are outside this reviewed variant. | 100 / 100 |
| FBX | Require the full little-endian binary header and ufbx-supported versions 3000–7700. Corpus evaluation caught and restored the legacy version through a dedicated fixture. | 100 / 100 |
| Blender | Require pointer width, byte order and version fields in the pre-v5 header. Blender 5's extended header is outside this reviewed variant. | 100 / 100 |
| GIF | Require version, nonzero dimensions and a block marker after the complete color table. | 97 / 100 |
| PCAP | Require the 24-byte version-2.4 header, matching byte order and nonzero snapshot length. One formerly matched corpus file claims unsupported version 1.0 and now abstains. | 81 / 100 |
| AU | Require the complete header, data offset, sample rate, channels and known encoding; support both byte orders. This corpus has only one AU positive, so coverage evidence is limited. | 1 / 1 |
| 3DSX | Require version-zero standard/extended header, relocation header space and aligned segments. | 100 / 100 |

Specification references include [gzip](https://www.rfc-editor.org/rfc/rfc1952),
[EPUB ZIP requirements](https://www.w3.org/TR/epub-33/#sec-zip-container-mime),
[WebAssembly modules](https://webassembly.github.io/spec/core/binary/modules.html),
[Photoshop formats](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/),
[ORC](https://orc.apache.org/specification/ORCv1/) and
[pandas SAS constants](https://github.com/pandas-dev/pandas/blob/main/pandas/io/sas/sas_constants.py).
The next batch compares pinned libmagic/Tika entries with the
[QOI specification](https://qoiformat.org/qoi-specification.pdf),
[Khronos GLB specification](https://github.com/KhronosGroup/glTF/blob/main/specification/2.0/Specification.adoc#glb-file-format-specification),
[WOFF](https://www.w3.org/TR/WOFF/), [WOFF2](https://www.w3.org/TR/WOFF2/),
[NumPy format documentation](https://numpy.org/doc/stable/reference/generated/numpy.lib.format.html)
and [ufbx's reader](https://github.com/ufbx/ufbx/blob/master/ufbx.c).

The following review batch compares libmagic's permissive magic matches with the
[GIF89a specification](https://www.w3.org/Graphics/GIF/spec-gif89a.txt), the
[PCAP format draft](https://www.ietf.org/archive/id/draft-ietf-opsawg-pcap-05.html),
[libsndfile's AU reader and writer](https://github.com/libsndfile/libsndfile/blob/master/src/au.c)
and [devkitPro's 3DSX writer](https://github.com/devkitPro/3dstools/blob/master/src/3dsxtool.cpp).
GIF requires version, nonzero dimensions and a block marker after the complete
global color table, covering all eight table sizes and the no-global-table case.
PCAP requires the complete 24-byte header, version 2.4 and a nonzero snapshot
length; both byte orders and microsecond/nanosecond timestamps are covered.
The obsolete timezone/accuracy fields remain unrestricted. Older and extended
PCAP versions remain outside this rule's coverage.
AU requires its 24-byte header, a data offset beyond that header, nonzero sample
rate and channels, and a documented encoding; both byte orders are supported.
3DSX requires the standard or extended version-zero header, zero flags, relocation
header space and aligned segment sizes. These are header checks; complete audio,
image, packet or executable payload validation remains outside prefix matching.

## Disabled container candidates

The following candidates were reviewed individually against their referenced pinned
sources and the existing 29,523-file observations. Their definitions are unchanged
from that evaluation. Counts below are raw candidate matches against known labels,
not enforced decisions or claims of independent accuracy. All remain disabled;
none is referenced by another rule, and the native compiler omits their terminal
expressions. Changing or promoting them is unnecessary for the current bug fixes.

| Format | Source comparison and disposition | Raw correct / other-label matches |
| --- | --- | --- |
| DOCX | puremagic's ZIP header and ZIP flags do not identify the Word document content type. | 100 / 1,924 |
| DOTX | The same ZIP magic cannot distinguish a Word template from a document. | 100 / 1,924 |
| PPTX | The ZIP header lacks presentation-part or content-type evidence. | 100 / 1,924 |
| XLSB | ZIP magic does not establish a binary Excel workbook. | 100 / 1,924 |
| XLSX | ZIP magic does not establish an XML Excel workbook. | 100 / 1,924 |
| JAR | puremagic includes generic ZIP alternatives; Tika's JAR detector examines archive entries, including the manifest. | 100 / 1,924 |
| ODP | Tika correlates ZIP and an exact mimetype, but the combined candidate also accepts generic ZIP and literal escape text. | 100 / 1,924 |
| ODS | The candidate retains weak fixed-offset text and escaped-text alternatives alongside Tika's mimetype check; one other-label match remains. | 98 / 1 |
| ODT | Generic ZIP and uncorrelated `text` alternatives override the more specific Tika mimetype check. | 100 / 1,930 |
| DOC | Legacy Word signatures are mixed with generic compound-file magic and escaped strings. Tika's compound detector checks document streams. | 100 / 565 |
| PPT | Four-byte patterns at offset 512 do not replace compound-directory and PowerPoint stream evidence. | 83 / 336 |
| XLS | puremagic's fixed-offset `Microsoft Excel 5.0 Worksheet` string is a narrow historical case; Tika examines workbook streams. | 0 / 0 (100 positives) |
| MSI | No candidate was implemented (`condition: false`). libmagic's compound-file detection uses directory/CLSID context and also identifies related validation modules. | 0 / 0 (100 positives) |
| Publisher | PRONOM signature 1882 describes Publisher 1, with four bytes in the first 12 bytes; it is not evidence for modern compound Publisher files. | 0 / 0 (no positives) |
| WMA | puremagic's GUID is the shared ASF header, not an audio-stream discriminator. | 0 / 2 (no positives) |
| WMV | Both the full and shortened ASF GUID alternatives lack video-stream evidence. | 0 / 2 (no positives) |
| ASF | PRONOM signature 80 correlates the full GUID with reserved bytes at offset 28, unlike WMA/WMV's shorter checks. Neither available ASF sample matches; leave unqualified. | 0 / 0 (2 positives) |
| WebM | puremagic uses only the EBML magic. Tika declares WebM a Matroska subtype; subtype evidence is absent. | 100 / 100 |
| MP4 | The merged alternatives include generic ISO brands and even `ftyp3gp5`; container-family identity is not the narrower MP4 label. | 99 / 104 |
| QuickTime | libmagic correlates some atoms with following fields and comments out bare `free`/`skip`; merged alternatives include those words and generic `ftyp`. | 100 / 311 |

The source review includes puremagic's referenced `magic_data.json` entries,
Tika's MIME definitions, `JarDetector`, `OpenDocumentDetector` and
`POIFSContainerDetector`, libmagic's `animation`, `msdos` and
`ole2compounddocs` files, and PRONOM V125 signatures 80 and 1882. Revisions and
notices are in [LICENSES](LICENSES). No new inference or regression runs were
needed: the candidates and their non-enforcement are unchanged. This disposition
does not qualify any candidate for future promotion.

## Regression and evaluation requirements

### Targeted FTYP review

The 3GP, AVIF and HEIF rules now require a complete 16-byte fixed `ftyp` header,
a normal 32-bit box size of at least 16 bytes, four-byte alignment, and a complete
major brand. Redundant upstream alternatives are consolidated. 3GP uses complete
registered brands from the previously supported families, retaining the legacy
`3gp1`–`3gp3` and `3gs7` spellings recognized by the pinned Tika data. AVIF retains
`avif`/`avis`; HEIF retains its eight HEVC/L-HEVC brands. Generic `mif1` and `isom`
do not establish these narrower identities, and 3GPP2 remains separate.

The comparison uses the pinned libmagic/Tika entries, the
[MP4 registration authority's brands](https://github.com/mp4ra/mp4ra.github.io/blob/main/data/brands.csv),
and [GPAC's FileTypeBox reader](https://github.com/gpac/gpac/blob/master/src/isomedia/box_code_base.c).
The fixed header includes both major brand and minor version; compatible brands
occupy four-byte entries. These rules do not validate the entire declared box or
media payload. Extended-size boxes and brands found only in a compatibility list
remain outside the reviewed variants.

The targeted check retained 100/100 3GP, 100/100 AVIF and 12/12 HEIF matches from
the same development corpus, with no native/reference disagreement. Twenty-nine
focused checks cover fixed-header truncations, illegal sizes/alignment, incomplete
and sibling brands, retained variants, and a brand table beyond the scan window.
This is targeted evidence; a new full-corpus evaluation is deferred until a larger
batch of rule changes is ready.

### Batch qualification

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
