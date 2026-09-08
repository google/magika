# Rule quality and development evidence

Rules identify a format from at most 4 KiB. They do not validate the complete file.
All active bundled rules require at least eight observed bytes, including the
complete eight-byte empty WebAssembly module. This floor prevents decisions on
truncated magic; it does not make arbitrary padded magic safe. Format checks below
require related header fields, sizes, versions or container structure as available.

## APK, dBase, EMF and Python bytecode review

The last four format audits used the existing regression suite and `runner.observe`.
All 859 previous matches were retained across 1,044 distinct supplied positives:
400 baseline files plus 661 automatically validated candidates, with 17 duplicates
removed. Selected sizes and SHA-256 values were verified, and candidate truth came
from a single assigned label in an immutable validation-sidecar read. The 185
misses were unchanged. No wrong labels, conflicts or native/YARA-X disagreements
were observed in this positive batch; accumulated cross-class FP and performance
qualification are still separate gates.

| Rule | Source comparison and maintained scope | Matches / positives |
| --- | --- | --- |
| APK | Keep libmagic's first-member names `classes.dex` and `AndroidManifest.xml`, with 41/49-byte floors. Android's ZIP reader supports stored/deflated entries. Reject encrypted entries, unsupported compression and absent sizes unless deferred through the data-descriptor flag. Keep UTF-8/deflate flags, ZIP extra fields beyond 4 KiB and version-needed `0`, which occurs in real supplied APKs. This does not inspect compressed member contents or verify an APK signature. | 566 / 607 |
| dBase | Replace the long PRONOM letter alternatives with equivalent character classes while retaining each version/type family. For III/IV layouts require the complete first 32-byte field descriptor (64-byte prefix), a header length of at least 64 and nonzero record size. Shapelib tolerates a missing terminator, so do not require 65. For II require a complete first descriptor (24 bytes), the fixed 521-byte file-header floor and nonzero record/field widths. Correct the inherited month ceiling from 28 to 12; keep the zero-date variant. Empty-field tables, other first-field types and the different Level-7 layout are outside the inherited coverage. | 118 / 121 |
| EMF | Retain PRONOM's eight header/description/pixel-format layouts with readable field checks. Require the applicable 88/100/108-byte fixed prefix, aligned record/file sizes, and at least the header and EOF records. Preserve variable-size descriptions, fields beyond the scan prefix, zero created handles and ignored reserved values. The checks do not traverse every EMF record or compare dynamic offsets with the total file length. | 171 / 206 |
| Python bytecode | Keep exactly the existing 89 magic values, grouped by the CPython header layout. Require the marshalled code-object marker after the timestamp or source-size word: minimum 9 or 13 observed bytes. Keep Python 1.0–1.2's uppercase marker, later lowercase markers, PyPy 2.7's existing magic and marshal reference flags in the 3.4+ group. Magic 3210 introduced the size word; earlier 3.3 development magics still use the shorter header. This neither parses the full code object nor adds modern PEP-552 magics. | 4 / 110 |

Sources: [Android ZIP reader](https://android.googlesource.com/platform/system/core/+/1ee4892e66ba314131b7ecf17e98bb1762c4b84c/libziparchive/zip_archive.cc),
[Borland dBase format documentation](https://blogs.embarcadero.com/dbase-dbf-file-structure/),
[original dBase II format description](https://www.fileformat.info/format/dbf/corion-dbase-ii.htm),
[Shapelib reader](https://github.com/OSGeo/shapelib/blob/master/dbfopen.c),
[Microsoft EMF header variants](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-emf/de081cd7-351f-4cc2-830b-d03fb55e89ab),
[Wine EMF loader](https://github.com/wine-mirror/wine/blob/master/dlls/gdi32/enhmetafile.c),
[CPython header history](https://github.com/python/cpython/blob/v3.6.15/Lib/importlib/_bootstrap_external.py),
[Python 1.0 marshal](https://github.com/python/cpython/blob/v1.0.1/Python/marshal.c) and
[Python 3.4 marshal](https://github.com/python/cpython/blob/v3.4.0/Python/marshal.c).

## Lua, Mach-O, Matroska and torrent review

These four rules were checked against their format sources and 814 distinct supplied
positives: 400 baseline files plus 434 automatically validated candidates, with 20
duplicates removed. Candidate labels came from the validation sidecar's single
assigned format, and every selected file's size and SHA-256 was verified. All 650
previous matches were retained; 164 misses were unchanged. There were no wrong
labels, conflicts or native/YARA-X disagreements in this positive batch. The final
accumulated cross-class false-positive and performance gates remain separate.

| Rule | Source comparison and maintained scope | Matches / positives |
| --- | --- | --- |
| Lua bytecode | Libmagic accepts the four-byte magic alone. Lua's own 2.4–5.5 loaders establish distinct version layouts, byte order, size fields and conversion markers. Floors are 8 bytes for 3.1/3.2, 11 for 2.4, 14 for 2.5/3.0 and 4.0, 15 for 5.0/5.4, 12 for 5.1, 18 for 5.2, 17 for 5.3 and 13 for 5.5. These cover structural prefixes; numeric representation payloads and function bodies are not fully validated. LuaJIT remains outside this rule's existing coverage. | 79 / 180 |
| Mach-O | Tika's four thin-file magic values are retained. Apple's `mach_header` and `mach_header_64` require 28 and 32 observed bytes. Require nonzero file type and consistent empty/nonempty command-table fields, with at least eight declared command bytes for nonempty tables. Preserve both byte orders, CPU identifiers, flags, future nonzero file types and command tables beyond the prefix. Fat binaries and complete load-command validation remain outside this rule. | 369 / 430 |
| Matroska | Puremagic's bare name at offsets 24/31 can identify unrelated text. Like libmagic, require EBML magic and a DocType element; additionally check its encoded length. Retain the inherited name offsets 8/24/31, with a 16-byte minimum and all eight legal VINT widths for the eight-byte name. This is an early DocType check, not full EBML traversal; padded DocType strings and other offsets remain unqualified. | 102 / 103 |
| Torrent | Libmagic's leading comment/info keys are also valid generic dictionaries. Require a typed HTTP(S), UDP or WS(S) tracker URL, or typed info/name/piece-length evidence within the prefix, with a 20-byte minimum. Preserve announce lists extending beyond 4 KiB, trackerless info-first files and v2 metadata. Prefix matching does not fully parse bencoding or verify tracker-string lengths or piece hashes. | 100 / 101 |

The initial Lua check rejected one quality-proven candidate with numeric-mode byte
`4`. Stock Lua uses `0/1`, but OpenWrt's LNUM patch defines integer-width modes
`2/4/8` and their complex-number variants. Those modes are retained in the 5.1/5.2
layouts, with a dedicated regression. The candidate's recorded validator walked
25 Lua 5.2 prototypes to EOF; its exact producer is not established. The legacy
and 5.5 header fixtures establish layout handling, not measured coverage of every
Lua version.

Primary sources: [Lua release sources](https://www.lua.org/ftp/),
[Lua 5.1 header](https://www.lua.org/source/5.1/lundump.c.html),
[Lua 5.2 header](https://www.lua.org/source/5.2/lundump.c.html),
[Lua 5.5 header](https://www.lua.org/source/5.5/lundump.c.html),
[OpenWrt LNUM patch](https://github.com/openwrt/openwrt/blob/main/package/utils/lua/patches/010-lua-5.1.3-lnum-full-260308.patch),
[Apple Mach-O structures](https://github.com/apple-oss-distributions/xnu/blob/main/EXTERNAL_HEADERS/mach-o/loader.h),
[EBML](https://www.rfc-editor.org/rfc/rfc8794.html),
[BEP 3](https://www.bittorrent.org/beps/bep_0003.html),
[BEP 12](https://www.bittorrent.org/beps/bep_0012.html) and
[BEP 52](https://www.bittorrent.org/beps/bep_0052.html).

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
precision guarantee. Final accumulated qualification is tracked separately.

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

### Further disabled formats

These candidates also remain unchanged and unreferenced. The same saved corpus
observations apply; zero matches with no positive samples provides no evidence for
promotion. The review retains the existing fallback rather than adding coverage.

| Format | Source comparison and disposition | Raw correct / other-label matches |
| --- | --- | --- |
| Access | PRONOM signatures 270 and 2143–2147 cover Access 1.x/2.0, including encrypted variants; their long correlated markers do not cover the available newer files. | 0 / 0 (100 positives) |
| ANI | libmagic requires RIFF's `ACON` form at offset 8; the puremagic-derived candidate only checks `RIFF`, matching AVI, WAV and WebP. | 100 / 298 |
| a.out | libmagic distinguishes three 32-bit magic values and byte orders, with additional header fields. The candidate only retains four magic bytes and has no positive qualification. | 0 / 0 (no positives) |
| ARJ | puremagic's two-byte `60 EA` marker is below the active prefix floor and has no surrounding header evidence. | 0 / 0 (no positives) |
| Arrow | libmagic's six-byte `ARROW1` file marker is retained without more structure; it is not evidence for Arrow stream variants or qualification without samples. | 0 / 0 (no positives) |
| Berkeley DB | libmagic correlates magic at offset 0 or 12 with version fields. The candidate mixes byte-order/format magics without those fields or positive samples. | 0 / 0 (no positives) |
| Bzip3 | No candidate is implemented. libmagic recognizes `BZ3v1` and reads a following block size, but implementing and qualifying that rule would add coverage. | 0 / 0 (no positives) |
| Cinema 4D | PRONOM 846/847/1562 describes distinct 4.x, 5.x and 6+ signatures, including correlated container tags. No sample qualifies any retained variant. | 0 / 0 (no positives) |
| COFF | No candidate is implemented. libmagic's COFF handling checks section count, processor and flags; a short machine value alone would not replace that context. | 0 / 0 (94 positives) |
| CRT | No candidate is implemented. libmagic's textual certificate patterns do not supply a qualified rule for the full certificate class. | 0 / 0 (100 positives) |
| DEB | libmagic correlates ar with the `debian-binary` or `debian-split` member. The generic `!<arch>` alternative also matches every sampled ar archive. | 100 / 100 |
| DMG | The source signature is an Apple Driver Map with a block-size mask, not universal DMG identity; it matches two ISO-labeled files and none of the DMG positives. | 0 / 2 (100 positives) |
| ELF | The four-byte ELF magic is shared by the 100 sampled CUDA binaries. libmagic additionally examines class, byte order and other header fields. | 99 / 100 |
| FileMaker | PRONOM's 3/5/7+/12 variants use long version-specific header strings and offsets. They have no positive coverage in this corpus. | 0 / 0 (no positives) |
| FlatGeobuf | The eight-byte magic agrees with libmagic's signature, but no sample establishes the retained variant's coverage. | 0 / 0 (no positives) |
| HDF5 | libmagic supports user-block offsets and PRONOM distinguishes superblock versions. Generic HDF5 magic also matches 35 Keras and 22 NetCDF files, plus one invalid-labeled sample. | 100 / 58 |
| HWP | puremagic/Tika's textual header targets older HWP files; Tika treats v5 as a compound-file subtype. None of the three available positives matches this candidate. | 0 / 0 (3 positives) |
| ISO | No candidate is implemented. libmagic's CD-ROM filesystem checks read well beyond the 4 KiB prefix, so copying them would violate the scan bound. | 0 / 0 (100 positives) |
| Java bytecode | PRONOM's `CA FE BA BE` signature also matches nine Mach-O files; additional class-file structure would be needed before promotion. | 100 / 9 |
| JPEG | The merged sources contain a JP2 signature mislabeled `image/jpeg` by puremagic, plus bare SOI and escaped text. It matches 97 JP2 and two invalid-labeled files. | 100 / 99 |

Comparisons use the per-rule source references, plus libmagic's `compress`, `coff`,
`securitycerts` and `filesystems` entries for unimplemented candidates and Tika's
HWP subtype definitions. No tests were added for these inactive alternatives;
future promotion would require focused regressions and corpus qualification.

### Remaining disabled candidates

This final disabled-candidate group follows the same unchanged-source comparison.
XZ and SQLite use the rule names `xz_v1` and `sqlite_v1`; their observations are
counted under those names, not nonexistent `taxonomy_` aliases.

| Format | Source comparison and disposition | Raw correct / other-label matches |
| --- | --- | --- |
| LHA | puremagic's complete method markers are weakened by an independent three-byte `-lh` alternative. A malformed sample matches. | 100 / 1 |
| LightWave | PRONOM signature 1583 correlates `FORM` with `LWOB`, a specific object variant. None of the available LightWave positives matches it. | 0 / 0 (100 positives) |
| LMDB | libmagic reads magic at offset 16 and a following version. The candidate only retains the magic and has no positive samples. | 0 / 0 (no positives) |
| LZX | libmagic's three-byte marker identifies the Amiga archive family; it is below the active prefix floor and lacks qualification. | 0 / 0 (no positives) |
| MP3 | libmagic examines frame header fields; the merged candidate accepts short sync patterns and bare ID3. One FLAC and three ICO files match. | 100 / 4 |
| OTF | No candidate is implemented. libmagic's `OTTO`/sfnt recognition does not by itself qualify new coverage. | 0 / 0 (100 positives) |
| Outlook | puremagic's compound-file header is shared by Office and other containers. Tika inspects message-specific storage names such as `__substg1.0_`. | 100 / 565 |
| Paradox | PRONOM 516–519 encodes version-specific header fields at offset 2 and later positions. No sample qualifies these variants. | 0 / 0 (no positives) |
| PCAPNG | libmagic also checks the byte-order magic at offset 8; the candidate only checks the initial block type and matches 18 PCAP-labeled samples. | 100 / 18 |
| PGP | Armor signatures and the two-byte binary-key alternative are mixed; only four positives match, plus two PEM-labeled samples. The broader PGP class is not qualified. | 4 / 2 (19 positives) |
| PNG | libmagic correlates PNG with IHDR/CgBI structure. The candidate also accepts literal escape text and does not reject 14 invalid-labeled samples. | 100 / 14 |
| PostScript | libmagic distinguishes leading separators and Adobe headers. Bare `%!` and escaped text in the combined candidate match two LaTeX files. | 82 / 2 |
| RData | libmagic's `RDX2`/`RDX3` lines continue into RDS interpretation; the candidate only retains five leading bytes and has no positive samples. | 0 / 0 (no positives) |
| Rhinoceros | PRONOM supplies distinct 3DM version 1–8 header strings and padding. No positive samples qualify these retained variants. | 0 / 0 (no positives) |
| SquashFS | libmagic follows the endian-dependent magic with superblock interpretation. Generic SquashFS identity also matches seven Snap packages. | 100 / 7 |
| TAR | puremagic/Tika provide `ustar` markers at offset 257; escaped-text variants and missing header correlation leave two invalid-labeled matches. | 98 / 2 |
| TGA | The puremagic source itself encodes literal backslash text at offset 1. This is not a binary TGA header and matches none of the positives. | 0 / 0 (100 positives) |
| TIFF | TIFF/BigTIFF magic does not distinguish GeoTIFF; the merged candidate also contains an `I I` text alternative. It matches 100 GeoTIFF and five invalid-labeled samples. | 100 / 105 |
| WebP | libmagic correlates RIFF with `WEBP` and then walks chunks. The standalone RIFF alternative also matches ANI, AVI and WAV. | 98 / 300 |
| WIM | puremagic's source contains literal `MSWIM` plus backslash escapes. libmagic uses binary zero bytes and a WIM header reader; the candidate matches no positives. | 0 / 0 (100 positives) |
| WMF | Tika's longer header alternatives are weakened by puremagic's independent two-byte `01 00` signature, which also matches 67 EMF files. | 100 / 67 |
| XZ | The reviewed 12-byte header checks magic, flags and header CRC for four check types. Existing indistinguishable-prefix ambiguity evidence keeps the prior promotion hold; the corpus alone does not resolve it. | 100 / 0 |
| zlib stream | Tika's four short CMF/FLG pairs are insufficient for the narrower stream label; 85 DMG files also match. libmagic additionally interprets compression-method and header check bits. | 100 / 85 |
| SQLite | The 100-byte header and page-size checks identify SQLite storage, not its applications. The same predicate matches 100 GeoPackage and three MBTiles files. Existing constructed MBTiles counterexamples also preserve the promotion hold. | 87 / 103 |

The review checked the referenced pinned entries and the existing XZ/SQLite
structural-verification evidence, including its reported counterexamples rather
than treating it as a passing qualification run. No inactive predicate was enabled,
rewritten or benchmarked again. All 64 candidates reviewed in these three tables
remain outside enforced classification.

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

### Fixed binary header review

Eight further active signatures now require their identifying header structure.
The comparison starts from the pinned libmagic/Puremagic signatures and checks the
format's writer or reader directly. These checks identify formats; they do not
validate complete payloads or checksums.

| Rule | Source comparison and maintained guard | Corpus positives retained |
| --- | --- | ---: |
| CRAM | The [CRAM specification](https://github.com/samtools/hts-specs/blob/master/CRAMv3.tex) defines a 26-byte file definition. Require all 26 bytes and the specified 1.0, 2.0/2.1 or 3.0/3.1 version pair. | 17/17 |
| DEX | The [DEX header specification](https://source.android.com/docs/core/runtime/dex-format) adds three decimal version bytes, a NUL terminator, a header-size field and an endian tag to the old four-byte signature. Require 112 bytes, or 120 for version 041, with matching header size and endian tag. Preserve both byte orders and numeric legacy versions. | 100/100 |
| Redis RDB | The [Redis writer and reader](https://github.com/redis/redis/blob/unstable/src/rdb.c) use `REDIS` plus four decimal version digits. Require all nine bytes and a nonzero version. | 77/77 |
| Lzip | The [libarchive reader](https://github.com/libarchive/libarchive/blob/master/libarchive/archive_read_support_filter_xz.c) recognizes versions 0/1 and dictionary exponents 12 through 29. Check those fields, the pack's eight-byte minimum prefix, and total size sufficient for the six-byte header plus the respective 12/20-byte trailer. | 1/1 |
| Rzip | The [rzip 2.1 source](https://rzip.samba.org/ftp/rzip/rzip-2.1.tar.gz) writes a 24-byte header with version 2.1, two size words and ten zero reserved bytes. Require the full header, retain 2.0/2.1, and check the reserved bytes. Older or future revisions are outside the reviewed variants. | 1/1 |
| XAR | The [Apple XAR reader](https://github.com/apple-oss-distributions/xar/blob/main/xar/lib/archive.c) reads the fixed 28-byte structure while tolerating unusual size/version fields. Require 28 bytes and nonzero TOC lengths. Preserve that tolerance and extended headers; do not restrict checksum algorithms. | 100/100 |
| SPIR-V | The [SPIRV-Tools reader](https://github.com/KhronosGroup/SPIRV-Tools/blob/main/source/binary.cpp) recognizes five-word headers and version encoding. Require 20 bytes, word-aligned total size, version 1.0 through 1.6, a nonzero ID bound and reserved schema zero, in either byte order. | 89/89 |
| ICNS | The [Pillow ICNS reader](https://github.com/python-pillow/Pillow/blob/main/src/PIL/IcnsImagePlugin.py) reads an eight-byte container header and eight-byte element headers. Preserve an empty eight-byte container; otherwise require the first full element header and minimum declared lengths. Do not restrict element type codes. | 100/100 |

Four XAR corpus samples have reversed size/version values (1/28), but their
compressed tables expand to XAR XML of the declared length. Rejecting them on
those fields would overrestrict identification relative to the reader. The rule
and regression fixtures preserve them.

All eight original rules failed the focused malformed-header/truncation checks
before the changes. Ten focused tests now pass, including one batched native
comparison covering the same mutations and retained endian/version/container
variants. A single targeted native/reference corpus check retained all 485
positives without conflicts, scan errors or disagreement. This is development
corpus evidence for the affected types, not fresh full-corpus qualification.
The existing full-corpus evaluation item remains open for the accumulated batch;
no runtime or benchmark code changed in this review.

### Version and container header review

Nine more active rules had reproducible short-header or malformed-field matches.
All nine failed their focused regression before edits. The following checks now
retain all 762 affected-format positives in one native/reference corpus run, with
no conflicts, errors or engine disagreement. This remains targeted development
corpus evidence, not a fresh full-corpus qualification.

| Rule | Direct source comparison and guard | Retained positives |
| --- | --- | ---: |
| Apple binary plist | [CoreFoundation's reader](https://github.com/apple-oss-distributions/CF/blob/main/CFBinaryPList.c) requires at least 41 bytes for classic `bplist0?`, and deliberately accepts any second version byte. Preserve that tolerance and the distinct numeric `bplist1x` / binary version spellings from the pinned signatures; reject an arbitrary unknown version pair. Other serialization variants keep their existing eight-byte minimum. | 100/100 |
| AppleDouble | [RFC 1740](https://www.rfc-editor.org/rfc/rfc1740) specifies the 26-byte fixed header and the distinct AppleDouble magic. Require the complete header and retain version 1/2. Preserve historical nonzero filler instead of applying the version-2 zero-filler recommendation to all files. | 59/59 |
| AppleSingle | The same fixed-header layout and version check apply with AppleSingle's different magic. Preserve zero-entry headers; no entry-table or payload traversal is added. | 100/100 |
| UF2 | The [UF2 specification](https://github.com/microsoft/uf2) identifies a block using two starting magic words and a third at byte 508. Require a complete 512-byte first block, all three words and payload length at most 476. Do not constrain board IDs, flag combinations, block order or the rest of the file. | 100/100 |
| XCF | The [GIMP reader](https://github.com/GNOME/gimp/blob/master/app/xcf/xcf-load.c) reads the 14-byte version string followed by dimensions and image type. Require all 26 bytes, `file` or a three-digit `vNNN` version with NUL, and base type 0/1/2. Preserve GIMP's tolerance for damaged dimensions instead of rejecting otherwise identifiable XCF files. | 3/3 |
| RAR | Pinned libmagic `archive` entries distinguish complete RAR4/RAR5 markers and pre-1.5 `RE~^`. Remove the broad `Rar!` alternative that bypassed the remaining signature bytes. Retain all three generations and the global eight-byte floor. | 100/100 |
| MAT | The [SciPy MAT-v5 writer/reader](https://github.com/scipy/scipy/blob/main/scipy/io/matlab/_mio5.py) uses a 128-byte header with version/endian fields at its end. Require the version-5 text prefix and matching version/endian pair, preserving both byte orders. Other MAT families remain outside this existing rule. | 100/100 |
| GGUF | The [GGUF specification and version history](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md) describe version 1's 32-bit counts and version 2/3's 64-bit counts. Require 16 or 24 bytes accordingly and preserve both byte orders. Do not validate tensor contents or model architecture. | 100/100 |
| WAD | The [Doom header definition](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/w_wad.h) contains `IWAD`/`PWAD`, a lump count and directory offset. Require all 12 bytes, preserving empty containers. | 100/100 |

The shared focused fixture covers each malformed field and every truncation from
the eight-byte floor to the required header length. The native check batches these
cases with retained legacy/endian variants. Nineteen focused checks passed after
the nine fixes; the later SketchUp cleanup passed its focused case and the batched
native check without rerunning the broader suite.

Four other identifying signatures were reviewed individually against their pinned
sources. Existing whole-corpus observations for each show 100/100 positives and no
other-label raw hits; these counts predate this review and are reused as supporting
evidence, not presented as a new evaluation.

| Rule | Source comparison and disposition |
| --- | --- |
| DICOM | Libmagic `images`, Puremagic and Tika agree on `DICM` at offset 128. The existing rule requires all 132 bytes, preserves arbitrary preamble data, and identifies the Part 10 wrapper. Keep it; raw datasets without this wrapper remain outside the signature. |
| BEAM | Libmagic `erlang` distinguishes OTP R3/R4's seven-byte marker from OTP R5+'s `FOR1` at 0 plus `BEAM` at 8. Both variants are preserved, with minimum observed lengths of 8 and 12 respectively. No change warranted. |
| OneNote | PRONOM internal signature 968 supplies the full 16-byte OneNote GUID at offset zero. The existing bounded pattern requires all 16 bytes and does not conflate it with another compound container. Keep the rule. |
| SketchUp | PRONOM internal signatures 241/243 give the 16-byte legacy and 32-byte Unicode headers. Every other existing version-specific alternative extends the same Unicode header and was already subsumed. Keep just those two complete headers, preserving the accepted byte language while removing redundant patterns. |

### LZ4 and Zstandard frame headers

Pinned libmagic `compress`, Puremagic and Tika identify these families primarily
by four-byte magic. The [LZ4 frame specification](https://github.com/lz4/lz4/blob/dev/doc/lz4_Frame_format.md)
and [Zstandard frame specification](https://github.com/facebook/zstd/blob/dev/doc/zstd_compression_format.md)
provide additional fixed-header constraints that distinguish impossible headers
without decompressing input.

| Rule | Review decision and limits |
| --- | --- |
| LZ4 | Modern frames require version 1, clear reserved bits, and one of the four defined block-size codes. Four alternatives distinguish optional content-size and dictionary fields and observe the complete header through its checksum byte. Retain the eight-byte prefix floor and require room for at least the end marker in the original file. Preserve the two existing legacy magics under their previous eight-byte floor; modern flags do not apply to those formats. Checksums and compressed blocks are not validated. |
| Zstandard | Modern frames require nine observed bytes, the shortest complete frame, and a clear reserved descriptor bit. Preserve the unused bit because conforming decoders ignore it. Retain the six existing legacy magics under their eight-byte floor. This checks minimum framing and the fixed descriptor, without parsing variable dictionary/content-size fields or compressed blocks. |

Neither rule adds skippable-frame identification: that marker is shared by both
formats and cannot establish which family follows. Two focused regressions failed
before the changes. Four checks passed afterward in 3.3 seconds, including shared
native/reference comparison, malformed descriptors, truncated optional LZ4 fields,
all four LZ4 block-size codes, and retained legacy variants. The affected corpus
evaluation and metadata reconciliation remain part of the accumulated batch gate;
these synthetic checks do not establish corpus precision or recall. No engine or
inference code changed, and no performance benchmark was repeated for this batch.

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

## ACE, BPG, DS_Store and DuckDB header review

The inherited libmagic rules primarily matched magic bytes. These four active
rules now require related header evidence using the existing bounded matcher:

| Format | Source comparison and decision | Retained matches / supplied positives |
| --- | --- | --- |
| ACE | acefile reads a main header with type zero, no add-size flag and at least 27 bytes after the CRC/size fields. Require 31 observed bytes and these fields; remove redundant PRONOM alternatives. Keep creator/extractor versions and host identifiers unrestricted. | 100 / 100 |
| BPG | Bellard's specification defines pixel formats 0–5, depth-minus-eight 0–6, color spaces 0–4 (zero for grayscale), and nonzero dimensions in shortest ue7(32) encoding. Require those fields and the following data-length byte, with a nine-byte floor. Alpha, range, extension and animation flags remain supported. | 10 / 10 |
| DS_Store | The ds_store reader consumes a 36-byte buddy header and follows an allocator root. Require the complete header, an offset beyond the header allocation and room for allocator counts. Preserve arbitrary root locations and the inherited nine-byte magic. | 100 / 100 |
| DuckDB | DuckDB stores its main header in a 4 KiB block. Require that block, DUCK at offset eight and a nonzero version with zero high 32 bits. Do not cap versions at a current release; retain historical versions and the newer 999 sentinel. | 53 / 53 |

Sources: [acefile's main-header reader](https://github.com/droe/acefile/blob/master/acefile.py),
[BPG specification](https://bellard.org/bpg/bpg_spec.txt),
[ds_store buddy allocator](https://github.com/dmgbuild/ds_store/blob/master/src/ds_store/buddy.py),
[DuckDB storage header](https://duckdb.org/docs/stable/internals/storage) and
[DuckDB main-header reader](https://github.com/duckdb/duckdb/blob/main/src/storage/single_file_block_manager.cpp).
Pinned libmagic entries are in `archive`, `images`, `apple` and `sql` respectively.
These checks establish header evidence, not whole-file validity: they do not verify
ACE/DuckDB checksums, decode BPG image data, or compare DS_Store allocator addresses.

Four regressions failed before the edits. Afterward, 32 shared header, variant and
native/YARA-X parity tests passed. They cover truncations, malformed fields, all
defined BPG pixel formats/depths, long dimensions and version/location variants.
The existing benchmark observer then retained all 263 distinct supplied positives,
with no conflicts, reference errors or native/YARA-X mismatches. This combines 259
baseline samples with four newly validated BPG candidates, selected using the
other lane's single assigned label and verified against each file's size/SHA-256.
This focused compatibility check does not replace the accumulated cross-class
zero-observed-FP gate. No runtime code, new I/O, corpus source or dataset commit
changed in this batch.

## ESE, FITS, LLVM, LRZIP, PostgreSQL, shapefile, SPSS and VHD review

Each rule was compared directly with its inherited signatures and a format reader
or specification. The changes use the existing bounded rule language.

| Format | Review decision | Retained matches / supplied positives |
| --- | --- | --- |
| ESE | Require the complete 668-byte documented header, nonzero format version and database/stream subtype. Keep the inherited zero field at offset 132, and leave revision/page-size values unrestricted. | 16 / 16 |
| FITS | Require one 2880-byte header block, a logical SIMPLE value and a legal BITPIX value on the second or third card. Preserve the inherited second-card spacing check, SIMPLE=F, flexible value positioning and all six pixel widths. | 100 / 100 |
| LLVM bitcode | Require word-aligned file size. Raw bitstreams reject an initial END_BLOCK but permit subblocks, abbreviations and records. Wrappers require the offset/size fields and room for embedded magic; do not constrain ignored wrapper versions or assume a 20-byte payload offset. | 465 / 465 |
| LRZIP | Require the 24-byte header, major zero and a nonzero minor version. Keep minor versions open and do not impose old reserved-field values on newer streaming/encryption flags. | 4 / 4 |
| PostgreSQL dump | Check major version, integer widths and custom/tar/directory format. Distinguish 1.0's missing revision byte and the offset-width byte introduced in 1.7; keep newer minor versions. | 49 / 49 |
| Shapefile | Preserve PRONOM's main/index first-entry distinction while requiring 108 bytes, legal header shape type and a minimum file length. Empty-header-only files remain outside the existing rule's coverage. | 100 / 100 |
| SPSS | Require the 176-byte SAV/ZSAV header, layout 2/3, legal compression and consistent byte order. Raise the inherited portable alternative's floor to its 464-byte logical header; no translated portable-header parser was added. | 163 / 163 |
| VHD | Require a complete 512-byte leading footer copy, version 1.0, defined feature/type values and a legal saved-state flag. Trailing-footer-only fixed disks remain outside the prefix rule's coverage. | 7 / 7 |

Sources: libyal's [ESE format documentation](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20(ESE)%20Database%20File%20(EDB)%20format.asciidoc),
[CFITSIO](https://github.com/HEASARC/cfitsio/blob/develop/fitscore.c),
[LLVM bitcode format](https://llvm.org/docs/BitCodeFormat.html) and
[wrapper reader](https://github.com/llvm/llvm-project/blob/main/llvm/include/llvm/Bitcode/BitcodeReader.h),
[LRZIP header layouts](https://github.com/ckolivas/lrzip/blob/master/doc/magic.header.txt),
[PostgreSQL ReadHead](https://github.com/postgres/postgres/blob/master/src/bin/pg_dump/pg_backup_archiver.c),
[ESRI shapefile specification](https://www.esri.com/library/whitepapers/pdfs/shapefile.pdf),
[ReadStat SAV header](https://github.com/WizardMac/ReadStat/blob/master/src/spss/readstat_sav.h),
[ReadStat portable reader](https://github.com/WizardMac/ReadStat/blob/master/src/spss/readstat_por_read.c)
and [QEMU's VHD reader](https://github.com/qemu/qemu/blob/master/block/vpc.c).
Pinned libmagic/PRONOM source identifiers remain attached to each rule.

Eight regressions failed before the changes. Forty shared header, variant and
native/YARA-X tests then passed in 5.96 seconds. These exercise truncated headers,
invalid fields, historical layouts, byte orders and newer flags. Existing corpus
observation retained all 904 distinct positives with no conflicts, reference errors
or native/YARA-X mismatches: 476 baseline files plus 452 validated candidates minus
24 duplicate hashes. Candidate labels came from the other lane's single assigned
label; all sample bytes were checked against size and SHA-256, without changing the
corpus. Cross-class false-positive evaluation remains the final accumulated gate.
These are prefix identifiers, not checksum, offset-target, image, archive or
statistical-data validators. No supplied positive currently qualifies the inherited
SPSS portable alternative beyond its signature and header-length regression.

## Partial CRX, FLAC, WinHelp, JP2, SZDD, NetCDF, PDB and Stata review

The inherited signatures identify only some variants of these formats. The fixes
preserve that coverage while requiring related header fields:

| Format | Review decision | Matches before / after / positives |
| --- | --- | --- |
| CRX | Distinguish CRX2's complete 16-byte header and nonzero key/signature lengths (at most 65536 each) from CRX3's 12-byte header and nonempty protobuf header. | 97 / 97 / 100 |
| FLAC | Require complete STREAMINFO, its 34-byte size, defined first-block type, block sizes at least 16 and nonzero sample rate. Preserve the last-metadata flag and the entire 20-bit sample-rate range. | 92 / 92 / 100 |
| WinHelp | Require the complete 16-byte header, plausible directory/file-size fields and the inherited no-free-chain signature. | 98 / 98 / 100 |
| JP2 | Require the full binary signature box and exact `jp2 ` brand, with a complete, aligned FTYP fixed header. OpenJPEG accepts an empty compatibility list, so the minimum is 28 bytes rather than 32. | 97 / 97 / 100 |
| SZDD | Require the complete 14-byte normal header and all eight signature bytes. Preserve mode B documented by libmagic for early Windows releases, as well as normal mode A. | 36 / 36 / 100 |
| NetCDF | Require the 32-byte minimum CDF1/CDF2 header. A nonempty dimension list requires its dimension tag; preserve the reader's tolerance for empty lists. CDF5 and HDF5-backed variants remain outside the rule. | 77 / 77 / 100 |
| PDB | Require complete MSF7/JG binary magic, 56/60-byte fixed headers and supported page sizes. MSF7 also checks its free-page-map selector. Portable PDB remains outside this rule. | 38 / 38 / 100 |
| Stata | Correlate a three-digit release with the complete byte-order field, requiring 63 bytes. Preserve both byte orders and do not cap future releases at 119. Pre-117 binary layouts remain outside the rule. | 59 / 59 / 100 |

Sources: [Chromium's CRX2 parser](https://github.com/chromium/chromium/blob/45.0.2454.85/components/crx_file/crx_file.cc)
and [CRX3 verifier](https://github.com/chromium/chromium/blob/main/components/crx_file/crx_verifier.cc),
[FLAC format](https://www.rfc-editor.org/rfc/rfc9639.html),
[Wine's WinHelp reader](https://github.com/wine-mirror/wine/blob/master/programs/winhlp32/hlpfile.c),
[OpenJPEG FTYP reader](https://github.com/uclouvain/openjpeg/blob/master/src/lib/openjp2/jp2.c),
[libmspack SZDD reader](https://github.com/kyz/libmspack/blob/master/libmspack/mspack/szddd.c),
[NetCDF format](https://docs.unidata.ucar.edu/nug/current/file_format_specifications.html)
and [reader](https://github.com/Unidata/netcdf-c/blob/main/libsrc/v1hpg.c),
[LLVM MSF header](https://github.com/llvm/llvm-project/blob/main/llvm/include/llvm/DebugInfo/MSF/MSFCommon.h),
[Wine's legacy PDB header](https://github.com/wine-mirror/wine/blob/master/include/wine/mscvpdb.h)
and [ReadStat's Stata reader](https://github.com/WizardMac/ReadStat/blob/master/src/stata/readstat_dta_read.c).
The existing libmagic, puremagic, Tika and PRONOM references remain on each rule.

Eight before-fix regressions exposed invalid or truncated header matches. The
existing shared/native regression test now consumes both rule packs, so partial
rules receive the same parity check as full rules. Forty-eight checks passed,
including header variants and the native matcher, in 10.80 seconds. All 594 previous
matches were retained across the 800 supplied baseline positives, with no wrong
rule labels, conflicts, reference errors or native/YARA-X mismatches. The six newly
validated Stata candidates were identical to already included samples; their bytes
and assigned labels were verified before deduplication. The 206 unmatched samples
remain coverage limitations, not newly introduced false negatives. These are
header checks; they do not verify CRX cryptographic signatures or parse complete
archives, audio, databases or statistical data. Final cross-class FP and runtime
performance gates remain separate.

## Raw BAM scope (R-17)

The BAM rule identifies the uncompressed BAM stream. Pinned libmagic explicitly
notes that ordinary BAM files wrap this stream in BGZF; HTSlib's `bam_hdr_read`
reads the raw magic, text length/text and reference count after that layer. The rule
is therefore limited, but is not dead: both supplied raw BAM positives matched in
the header review, and the shared/native regressions cover no-text and long-text
headers. Its zero FN metadata records those two supplied samples; it does not
claim complete coverage of BGZF-wrapped BAM files. The existing gzip rule identifies
the outer frame. Identifying its inner BAM payload would require decompression or
new container handling, outside the requested bug-fix scope. The first-version
corpus gate remains zero observed false positives, with coverage improvements left
for later work.
