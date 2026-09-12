# Rule quality

Rules identify a format from at most 4 KiB; they do not validate complete files.
Every enforced bundled rule requires at least eight observed bytes and the
format-specific structural checks below. Misses and conflicts abstain.

On the adjudicated combined snapshot, 9,624 of 25,421 files matched with zero
observed false positives. These are development-corpus results, not a universal
precision guarantee. Current scores, dataset identities and reproduction details
are in the [benchmark catalogue](benchmarks/CATALOG.md). Unqualified formats
remain disabled in `rulesets/notworking`.

The [full format-audit history](https://github.com/google/magika/blob/5bc2f250ce67153589c00bb546d0ad9ebfc4fc77/rules/QUALITY.md)
retains earlier sample counts, APK adjudications and test outcomes. The maintained
reference below describes rule scope and its sources.

## APK, dBase, EMF and Python bytecode

| Rule | Source comparison and maintained scope |
| --- | --- |
| APK | Require a ZIP local entry for `AndroidManifest.xml` within the prefix, including when `classes.dex` is first. A DEX-only ZIP/JAR or a manifest name in a ZIP comment is insufficient. Preserve stored/deflated and manifest-first native-only splits; compressed contents and signing are not validated. |
| dBase | Replace the long PRONOM letter alternatives with equivalent character classes while retaining each version/type family. For III/IV layouts require the complete first 32-byte field descriptor (64-byte prefix), a header length of at least 64 and nonzero record size. Shapelib tolerates a missing terminator, so do not require 65. For II require a complete first descriptor (24 bytes), the fixed 521-byte file-header floor and nonzero record/field widths. Correct the inherited month ceiling from 28 to 12; keep the zero-date variant. Empty-field tables, other first-field types and the different Level-7 layout are outside the inherited coverage. |
| EMF | Retain PRONOM's eight header/description/pixel-format layouts with readable field checks. Require the applicable 88/100/108-byte fixed prefix, aligned record/file sizes, and at least the header and EOF records. Preserve variable-size descriptions, fields beyond the scan prefix, zero created handles and ignored reserved values. The checks do not traverse every EMF record or compare dynamic offsets with the total file length. |
| Python bytecode | Keep exactly the existing 89 magic values, grouped by the CPython header layout. Require the marshalled code-object marker after the timestamp or source-size word: minimum 9 or 13 observed bytes. Keep Python 1.0–1.2's uppercase marker, later lowercase markers, PyPy 2.7's existing magic and marshal reference flags in the 3.4+ group. Magic 3210 introduced the size word; earlier 3.3 development magics still use the shorter header. This neither parses the full code object nor adds modern PEP-552 magics. |

Sources: [Android ZIP reader](https://android.googlesource.com/platform/system/core/+/1ee4892e66ba314131b7ecf17e98bb1762c4b84c/libziparchive/zip_archive.cc),
[Borland dBase format documentation](https://blogs.embarcadero.com/dbase-dbf-file-structure/),
[original dBase II format description](https://www.fileformat.info/format/dbf/corion-dbase-ii.htm),
[Shapelib reader](https://github.com/OSGeo/shapelib/blob/master/dbfopen.c),
[Microsoft EMF header variants](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-emf/de081cd7-351f-4cc2-830b-d03fb55e89ab),
[Wine EMF loader](https://github.com/wine-mirror/wine/blob/master/dlls/gdi32/enhmetafile.c),
[CPython header history](https://github.com/python/cpython/blob/v3.6.15/Lib/importlib/_bootstrap_external.py),
[Python 1.0 marshal](https://github.com/python/cpython/blob/v1.0.1/Python/marshal.c) and
[Python 3.4 marshal](https://github.com/python/cpython/blob/v3.4.0/Python/marshal.c).

## Lua, Mach-O, Matroska and torrent

| Rule | Source comparison and maintained scope |
| --- | --- |
| Lua bytecode | Libmagic accepts the four-byte magic alone. Lua's own 2.4–5.5 loaders establish distinct version layouts, byte order, size fields and conversion markers. Floors are 8 bytes for 3.1/3.2, 11 for 2.4, 14 for 2.5/3.0 and 4.0, 15 for 5.0/5.4, 12 for 5.1, 18 for 5.2, 17 for 5.3 and 13 for 5.5. These cover structural prefixes; numeric representation payloads and function bodies are not fully validated. LuaJIT remains outside this rule's existing coverage. |
| Mach-O | Tika's four thin-file magic values are retained. Apple's `mach_header` and `mach_header_64` require 28 and 32 observed bytes. Require nonzero file type and consistent empty/nonempty command-table fields, with at least eight declared command bytes for nonempty tables. Preserve both byte orders, CPU identifiers, flags, future nonzero file types and command tables beyond the prefix. Fat binaries and complete load-command validation remain outside this rule. |
| Matroska | Puremagic's bare name at offsets 24/31 can identify unrelated text. Like libmagic, require EBML magic and a DocType element; additionally check its encoded length. Retain the inherited name offsets 8/24/31, with a 16-byte minimum and all eight legal VINT widths for the eight-byte name. This is an early DocType check, not full EBML traversal; padded DocType strings and other offsets remain unqualified. |
| Torrent | Libmagic's leading comment/info keys are also valid generic dictionaries. Require a typed HTTP(S), UDP or WS(S) tracker URL, or typed info/name/piece-length evidence within the prefix, with a 20-byte minimum. Preserve announce lists extending beyond 4 KiB, trackerless info-first files and v2 metadata. Prefix matching does not fully parse bencoding or verify tracker-string lengths or piece hashes. |

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

## Binary formats

| Format | Review decision |
| --- | --- |
| 3DSM | Require little-endian root and plausible version/main child chunk; remove TIFF collision. |
| AVI | Correlate RIFF, AVI form, LIST and header-list type. |
| BMP | Decode binary reserved fields; distinguish OS/2 core and uncompressed Windows DIB headers. |
| CAB | Require binary signature, reserved fields, cabinet version, folder count and bounded flags. |
| DWG | Require modern version marker, following reserved zeros and a 128-byte observed header. |
| EPUB | Require the first stored ZIP member to be the exact mimetype; correlate sizes, flags and optional data descriptor. |
| FLV | Require version, legal stream flags, standard data offset and first previous-tag size. Extended headers remain unsupported. |
| Gzip | Require compression method 8, legal flags and minimum member size. |
| ICO | Require ICONDIR count and a plausible first resource entry. |
| JXL | Retain the container signature and file-type box; bare two-byte codestream magic cannot enforce identity. |
| MPEG-TS | Check 21 complete 188-byte or 192-byte packets with sync and legal transport-header bits. |
| Ogg | Require version, initial-page flags, sequence and segment-table evidence. |
| PSD/PSB | Require version, reserved zeros, channels, dimension limits, depth and color mode. |
| RPM | Require a binary lead and signature-header magic at the expected offset. |
| SAS | Require SAS7BDAT binary magic and the SAS FILE marker, with header length and byte-order evidence. |
| 7-Zip | Require the binary signature, major version and complete start header. |
| SWF | Correlate FWS/CWS/ZWS version and length with RECT, zlib or LZMA header fields. Real ZWS fixtures cover both observed variants. |
| Unix compress | Require binary magic and legal LZW flags; retain the eight-byte prefix floor. |
| WebAssembly | Require exact magic and standard version; test all single-byte header mutations. |
| WAV | Correlate RIFF/RIFX/RF64 with WAVE and minimum container/header lengths. |
| ARC | Disabled: the reviewed short signature does not establish safe identity. |
| ORC | Disabled: a short leading ORC string is insufficient; the format also relies on footer metadata. |
| XCOFF | Disabled until a stronger header/section check replaces its two-byte magic. |
| Bzip2 | Require a legal block-size digit and the block marker or empty-stream end marker; compare all nine stdlib compression levels. |
| QOI | Require dimensions, channel count, colorspace and the minimum complete encoded image size. |
| GLB | Require version 2, aligned lengths and a first JSON chunk. Version 1 is outside this reviewed variant. |
| WOFF | Require a complete header and first directory entry, legal reserved fields and a common sfnt flavor. |
| WOFF2 | Require header, directory/payload space, table and compressed-size evidence. Reserved-field violations abstain even though tolerant decoders may accept them. |
| NumPy | Distinguish v1's 16-bit header length from v2/v3's 32-bit length; require aligned dictionary-header evidence without fixing key order. |
| MIDI | Require header format, track count, valid timing division and first track header. Extended header chunks are outside this reviewed variant. |
| FBX | Require the full little-endian binary header and ufbx-supported versions 3000–7700. Corpus evaluation caught and restored the legacy version through a dedicated fixture. |
| Blender | Require pointer width, byte order and version fields in the pre-v5 header. Blender 5's extended header is outside this reviewed variant. |
| GIF | Require version, nonzero dimensions and a block marker after the complete color table. |
| PCAP | Require the 24-byte version-2.4 header, matching byte order and nonzero snapshot length. One formerly matched corpus file claims unsupported version 1.0 and now abstains. |
| AU | Require the complete header, data offset, sample rate, channels and known encoding; support both byte orders. This corpus has only one AU positive, so coverage evidence is limited. |
| 3DSX | Require version-zero standard/extended header, relocation header space and aligned segments. |

## Disabled container candidates

| Format | Source comparison and disposition |
| --- | --- |
| DOCX | puremagic's ZIP header and ZIP flags do not identify the Word document content type. |
| DOTX | The same ZIP magic cannot distinguish a Word template from a document. |
| PPTX | The ZIP header lacks presentation-part or content-type evidence. |
| XLSB | ZIP magic does not establish a binary Excel workbook. |
| XLSX | ZIP magic does not establish an XML Excel workbook. |
| JAR | puremagic includes generic ZIP alternatives; Tika's JAR detector examines archive entries, including the manifest. |
| ODP | Tika correlates ZIP and an exact mimetype, but the combined candidate also accepts generic ZIP and literal escape text. |
| ODS | The candidate retains weak fixed-offset text and escaped-text alternatives alongside Tika's mimetype check; one other-label match remains. |
| ODT | Generic ZIP and uncorrelated `text` alternatives override the more specific Tika mimetype check. |
| DOC | Legacy Word signatures are mixed with generic compound-file magic and escaped strings. Tika's compound detector checks document streams. |
| PPT | Four-byte patterns at offset 512 do not replace compound-directory and PowerPoint stream evidence. |
| XLS | puremagic's fixed-offset `Microsoft Excel 5.0 Worksheet` string is a narrow historical case; Tika examines workbook streams. |
| MSI | No candidate was implemented (`condition: false`). libmagic's compound-file detection uses directory/CLSID context and also identifies related validation modules. |
| Publisher | PRONOM signature 1882 describes Publisher 1, with four bytes in the first 12 bytes; it is not evidence for modern compound Publisher files. |
| WMA | puremagic's GUID is the shared ASF header, not an audio-stream discriminator. |
| WMV | Both the full and shortened ASF GUID alternatives lack video-stream evidence. |
| ASF | PRONOM signature 80 correlates the full GUID with reserved bytes at offset 28, unlike WMA/WMV's shorter checks. Neither available ASF sample matches; leave unqualified. |
| WebM | puremagic uses only the EBML magic. Tika declares WebM a Matroska subtype; subtype evidence is absent. |
| MP4 | The merged alternatives include generic ISO brands and even `ftyp3gp5`; container-family identity is not the narrower MP4 label. |
| QuickTime | libmagic correlates some atoms with following fields and comments out bare `free`/`skip`; merged alternatives include those words and generic `ftyp`. |

## Further disabled formats

| Format | Source comparison and disposition |
| --- | --- |
| Access | PRONOM signatures 270 and 2143–2147 cover Access 1.x/2.0, including encrypted variants; their long correlated markers do not cover the available newer files. |
| ANI | libmagic requires RIFF's `ACON` form at offset 8; the puremagic-derived candidate only checks `RIFF`, matching AVI, WAV and WebP. |
| a.out | libmagic distinguishes three 32-bit magic values and byte orders, with additional header fields. The candidate only retains four magic bytes and has no positive qualification. |
| ARJ | puremagic's two-byte `60 EA` marker is below the active prefix floor and has no surrounding header evidence. |
| Arrow | libmagic's six-byte `ARROW1` file marker is retained without more structure; it is not evidence for Arrow stream variants or qualification without samples. |
| Berkeley DB | libmagic correlates magic at offset 0 or 12 with version fields. The candidate mixes byte-order/format magics without those fields or positive samples. |
| Bzip3 | No candidate is implemented. libmagic recognizes `BZ3v1` and reads a following block size, but implementing and qualifying that rule would add coverage. |
| Cinema 4D | PRONOM 846/847/1562 describes distinct 4.x, 5.x and 6+ signatures, including correlated container tags. No sample qualifies any retained variant. |
| COFF | No candidate is implemented. libmagic's COFF handling checks section count, processor and flags; a short machine value alone would not replace that context. |
| CRT | No candidate is implemented. libmagic's textual certificate patterns do not supply a qualified rule for the full certificate class. |
| DEB | libmagic correlates ar with the `debian-binary` or `debian-split` member. The generic `!<arch>` alternative also matches every sampled ar archive. |
| DMG | The source signature is an Apple Driver Map with a block-size mask, not universal DMG identity; it matches two ISO-labeled files and none of the DMG positives. |
| ELF | The four-byte ELF magic is shared by the 100 sampled CUDA binaries. libmagic additionally examines class, byte order and other header fields. |
| FileMaker | PRONOM's 3/5/7+/12 variants use long version-specific header strings and offsets. They have no positive coverage in this corpus. |
| FlatGeobuf | The eight-byte magic agrees with libmagic's signature, but no sample establishes the retained variant's coverage. |
| HDF5 | libmagic supports user-block offsets and PRONOM distinguishes superblock versions. Generic HDF5 magic also matches 35 Keras and 22 NetCDF files, plus one invalid-labeled sample. |
| HWP | puremagic/Tika's textual header targets older HWP files; Tika treats v5 as a compound-file subtype. None of the three available positives matches this candidate. |
| ISO | No candidate is implemented. libmagic's CD-ROM filesystem checks read well beyond the 4 KiB prefix, so copying them would violate the scan bound. |
| Java bytecode | PRONOM's `CA FE BA BE` signature also matches nine Mach-O files; additional class-file structure would be needed before promotion. |
| JPEG | The merged sources contain a JP2 signature mislabeled `image/jpeg` by puremagic, plus bare SOI and escaped text. It matches 97 JP2 and two invalid-labeled files. |

## Remaining disabled candidates

| Format | Source comparison and disposition |
| --- | --- |
| LHA | puremagic's complete method markers are weakened by an independent three-byte `-lh` alternative. A malformed sample matches. |
| LightWave | PRONOM signature 1583 correlates `FORM` with `LWOB`, a specific object variant. None of the available LightWave positives matches it. |
| LMDB | libmagic reads magic at offset 16 and a following version. The candidate only retains the magic and has no positive samples. |
| LZX | libmagic's three-byte marker identifies the Amiga archive family; it is below the active prefix floor and lacks qualification. |
| MP3 | libmagic examines frame header fields; the merged candidate accepts short sync patterns and bare ID3. One FLAC and three ICO files match. |
| OTF | No candidate is implemented. libmagic's `OTTO`/sfnt recognition does not by itself qualify new coverage. |
| Outlook | puremagic's compound-file header is shared by Office and other containers. Tika inspects message-specific storage names such as `__substg1.0_`. |
| Paradox | PRONOM 516–519 encodes version-specific header fields at offset 2 and later positions. No sample qualifies these variants. |
| PCAPNG | libmagic also checks the byte-order magic at offset 8; the candidate only checks the initial block type and matches 18 PCAP-labeled samples. |
| PGP | Armor signatures and the two-byte binary-key alternative are mixed; only four positives match, plus two PEM-labeled samples. The broader PGP class is not qualified. |
| PNG | libmagic correlates PNG with IHDR/CgBI structure. The candidate also accepts literal escape text and does not reject 14 invalid-labeled samples. |
| PostScript | libmagic distinguishes leading separators and Adobe headers. Bare `%!` and escaped text in the combined candidate match two LaTeX files. |
| RData | libmagic's `RDX2`/`RDX3` lines continue into RDS interpretation; the candidate only retains five leading bytes and has no positive samples. |
| Rhinoceros | PRONOM supplies distinct 3DM version 1–8 header strings and padding. No positive samples qualify these retained variants. |
| SquashFS | libmagic follows the endian-dependent magic with superblock interpretation. Generic SquashFS identity also matches seven Snap packages. |
| TAR | puremagic/Tika provide `ustar` markers at offset 257; escaped-text variants and missing header correlation leave two invalid-labeled matches. |
| TGA | The puremagic source itself encodes literal backslash text at offset 1. This is not a binary TGA header and matches none of the positives. |
| TIFF | TIFF/BigTIFF magic does not distinguish GeoTIFF; the merged candidate also contains an `I I` text alternative. It matches 100 GeoTIFF and five invalid-labeled samples. |
| WebP | libmagic correlates RIFF with `WEBP` and then walks chunks. The standalone RIFF alternative also matches ANI, AVI and WAV. |
| WIM | puremagic's source contains literal `MSWIM` plus backslash escapes. libmagic uses binary zero bytes and a WIM header reader; the candidate matches no positives. |
| WMF | Tika's longer header alternatives are weakened by puremagic's independent two-byte `01 00` signature, which also matches 67 EMF files. |
| XZ | The reviewed 12-byte header checks magic, flags and header CRC for four check types. Existing indistinguishable-prefix ambiguity evidence keeps the prior promotion hold; the corpus alone does not resolve it. |
| zlib stream | Tika's four short CMF/FLG pairs are insufficient for the narrower stream label; 85 DMG files also match. libmagic additionally interprets compression-method and header check bits. |
| SQLite | The 100-byte header and page-size checks identify SQLite storage, not its applications. The same predicate matches 100 GeoPackage and three MBTiles files. Existing constructed MBTiles counterexamples also preserve the promotion hold. |

## Fixed binary header

| Rule | Source comparison and maintained guard |
| --- | --- |
| CRAM | The [CRAM specification](https://github.com/samtools/hts-specs/blob/master/CRAMv3.tex) defines a 26-byte file definition. Require all 26 bytes and the specified 1.0, 2.0/2.1 or 3.0/3.1 version pair. |
| DEX | The [DEX header specification](https://source.android.com/docs/core/runtime/dex-format) adds three decimal version bytes, a NUL terminator, a header-size field and an endian tag to the old four-byte signature. Require 112 bytes, or 120 for version 041, with matching header size and endian tag. Preserve both byte orders and numeric legacy versions. |
| Redis RDB | The [Redis writer and reader](https://github.com/redis/redis/blob/unstable/src/rdb.c) use `REDIS` plus four decimal version digits. Require all nine bytes and a nonzero version. |
| Lzip | The [libarchive reader](https://github.com/libarchive/libarchive/blob/master/libarchive/archive_read_support_filter_xz.c) recognizes versions 0/1 and dictionary exponents 12 through 29. Check those fields, the pack's eight-byte minimum prefix, and total size sufficient for the six-byte header plus the respective 12/20-byte trailer. |
| Rzip | The [rzip 2.1 source](https://rzip.samba.org/ftp/rzip/rzip-2.1.tar.gz) writes a 24-byte header with version 2.1, two size words and ten zero reserved bytes. Require the full header, retain 2.0/2.1, and check the reserved bytes. Older or future revisions are outside the reviewed variants. |
| XAR | The [Apple XAR reader](https://github.com/apple-oss-distributions/xar/blob/main/xar/lib/archive.c) reads the fixed 28-byte structure while tolerating unusual size/version fields. Require 28 bytes and nonzero TOC lengths. Preserve that tolerance and extended headers; do not restrict checksum algorithms. |
| SPIR-V | The [SPIRV-Tools reader](https://github.com/KhronosGroup/SPIRV-Tools/blob/main/source/binary.cpp) recognizes five-word headers and version encoding. Require 20 bytes, word-aligned total size, version 1.0 through 1.6, a nonzero ID bound and reserved schema zero, in either byte order. |
| ICNS | The [Pillow ICNS reader](https://github.com/python-pillow/Pillow/blob/main/src/PIL/IcnsImagePlugin.py) reads an eight-byte container header and eight-byte element headers. Preserve an empty eight-byte container; otherwise require the first full element header and minimum declared lengths. Do not restrict element type codes. |

## Version and container header

| Rule | Direct source comparison and guard |
| --- | --- |
| Apple binary plist | [CoreFoundation's reader](https://github.com/apple-oss-distributions/CF/blob/main/CFBinaryPList.c) requires at least 41 bytes for classic `bplist0?`, and deliberately accepts any second version byte. Preserve that tolerance and the distinct numeric `bplist1x` / binary version spellings from the pinned signatures; reject an arbitrary unknown version pair. Other serialization variants keep their existing eight-byte minimum. |
| AppleDouble | [RFC 1740](https://www.rfc-editor.org/rfc/rfc1740) specifies the 26-byte fixed header and the distinct AppleDouble magic. Require the complete header and retain version 1/2. Preserve historical nonzero filler instead of applying the version-2 zero-filler recommendation to all files. |
| AppleSingle | The same fixed-header layout and version check apply with AppleSingle's different magic. Preserve zero-entry headers; no entry-table or payload traversal is added. |
| UF2 | The [UF2 specification](https://github.com/microsoft/uf2) identifies a block using two starting magic words and a third at byte 508. Require a complete 512-byte first block, all three words and payload length at most 476. Do not constrain board IDs, flag combinations, block order or the rest of the file. |
| XCF | The [GIMP reader](https://github.com/GNOME/gimp/blob/master/app/xcf/xcf-load.c) reads the 14-byte version string followed by dimensions and image type. Require all 26 bytes, `file` or a three-digit `vNNN` version with NUL, and base type 0/1/2. Preserve GIMP's tolerance for damaged dimensions instead of rejecting otherwise identifiable XCF files. |
| RAR | Pinned libmagic `archive` entries distinguish complete RAR4/RAR5 markers and pre-1.5 `RE~^`. Remove the broad `Rar!` alternative that bypassed the remaining signature bytes. Retain all three generations and the global eight-byte floor. |
| MAT | The [SciPy MAT-v5 writer/reader](https://github.com/scipy/scipy/blob/main/scipy/io/matlab/_mio5.py) uses a 128-byte header with version/endian fields at its end. Require the version-5 text prefix and matching version/endian pair, preserving both byte orders. Other MAT families remain outside this existing rule. |
| GGUF | The [GGUF specification and version history](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md) describe version 1's 32-bit counts and version 2/3's 64-bit counts. Require 16 or 24 bytes accordingly and preserve both byte orders. Do not validate tensor contents or model architecture. |
| WAD | The [Doom header definition](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/w_wad.h) contains `IWAD`/`PWAD`, a lump count and directory offset. Require all 12 bytes, preserving empty containers. |

| Rule | Source comparison and disposition |
| --- | --- |
| DICOM | Libmagic `images`, Puremagic and Tika agree on `DICM` at offset 128. The existing rule requires all 132 bytes, preserves arbitrary preamble data, and identifies the Part 10 wrapper. Keep it; raw datasets without this wrapper remain outside the signature. |
| BEAM | Libmagic `erlang` distinguishes OTP R3/R4's seven-byte marker from OTP R5+'s `FOR1` at 0 plus `BEAM` at 8. Both variants are preserved, with minimum observed lengths of 8 and 12 respectively. No change warranted. |
| OneNote | PRONOM internal signature 968 supplies the full 16-byte OneNote GUID at offset zero. The existing bounded pattern requires all 16 bytes and does not conflate it with another compound container. Keep the rule. |
| SketchUp | PRONOM internal signatures 241/243 give the 16-byte legacy and 32-byte Unicode headers. Every other existing version-specific alternative extends the same Unicode header and was already subsumed. Keep just those two complete headers, preserving the accepted byte language while removing redundant patterns. |

## LZ4 and Zstandard frame headers

| Rule | Review decision and limits |
| --- | --- |
| LZ4 | Modern frames require version 1, clear reserved bits, and one of the four defined block-size codes. Four alternatives distinguish optional content-size and dictionary fields and observe the complete header through its checksum byte. Retain the eight-byte prefix floor and require room for at least the end marker in the original file. Preserve the two existing legacy magics under their previous eight-byte floor; modern flags do not apply to those formats. Checksums and compressed blocks are not validated. |
| Zstandard | Modern frames require nine observed bytes, the shortest complete frame, and a clear reserved descriptor bit. Preserve the unused bit because conforming decoders ignore it. Retain the six existing legacy magics under their eight-byte floor. This checks minimum framing and the fixed descriptor, without parsing variable dictionary/content-size fields or compressed blocks. |

## ACE, BPG, DS_Store and DuckDB header

| Format | Source comparison and decision |
| --- | --- |
| ACE | acefile reads a main header with type zero, no add-size flag and at least 27 bytes after the CRC/size fields. Require 31 observed bytes and these fields; remove redundant PRONOM alternatives. Keep creator/extractor versions and host identifiers unrestricted. |
| BPG | Bellard's specification defines pixel formats 0–5, depth-minus-eight 0–6, color spaces 0–4 (zero for grayscale), and nonzero dimensions in shortest ue7(32) encoding. Require those fields and the following data-length byte, with a nine-byte floor. Alpha, range, extension and animation flags remain supported. |
| DS_Store | The ds_store reader consumes a 36-byte buddy header and follows an allocator root. Require the complete header, an offset beyond the header allocation and room for allocator counts. Preserve arbitrary root locations and the inherited nine-byte magic. |
| DuckDB | DuckDB stores its main header in a 4 KiB block. Require that block, DUCK at offset eight and a nonzero version with zero high 32 bits. Do not cap versions at a current release; retain historical versions and the newer 999 sentinel. |

Sources: [acefile's main-header reader](https://github.com/droe/acefile/blob/master/acefile.py),
[BPG specification](https://bellard.org/bpg/bpg_spec.txt),
[ds_store buddy allocator](https://github.com/dmgbuild/ds_store/blob/master/src/ds_store/buddy.py),
[DuckDB storage header](https://duckdb.org/docs/stable/internals/storage) and
[DuckDB main-header reader](https://github.com/duckdb/duckdb/blob/main/src/storage/single_file_block_manager.cpp).
Pinned libmagic entries are in `archive`, `images`, `apple` and `sql` respectively.
These checks establish header evidence, not whole-file validity: they do not verify
ACE/DuckDB checksums, decode BPG image data, or compare DS_Store allocator addresses.

## ESE, FITS, LLVM, LRZIP, PostgreSQL, shapefile, SPSS and VHD

| Format | Review decision |
| --- | --- |
| ESE | Require the complete 668-byte documented header, nonzero format version and database/stream subtype. Keep the inherited zero field at offset 132, and leave revision/page-size values unrestricted. |
| FITS | Require one 2880-byte header block, a logical SIMPLE value and a legal BITPIX value on the second or third card. Preserve the inherited second-card spacing check, SIMPLE=F, flexible value positioning and all six pixel widths. |
| LLVM bitcode | Require word-aligned file size. Raw bitstreams reject an initial END_BLOCK but permit subblocks, abbreviations and records. Wrappers require the offset/size fields and room for embedded magic; do not constrain ignored wrapper versions or assume a 20-byte payload offset. |
| LRZIP | Require the 24-byte header, major zero and a nonzero minor version. Keep minor versions open and do not impose old reserved-field values on newer streaming/encryption flags. |
| PostgreSQL dump | Check major version, integer widths and custom/tar/directory format. Distinguish 1.0's missing revision byte and the offset-width byte introduced in 1.7; keep newer minor versions. |
| Shapefile | Preserve PRONOM's main/index first-entry distinction while requiring 108 bytes, legal header shape type and a minimum file length. Empty-header-only files remain outside the existing rule's coverage. |
| SPSS | Require the 176-byte SAV/ZSAV header, layout 2/3, legal compression and consistent byte order. Raise the inherited portable alternative's floor to its 464-byte logical header; no translated portable-header parser was added. |
| VHD | Require a complete 512-byte leading footer copy, version 1.0, defined feature/type values and a legal saved-state flag. Trailing-footer-only fixed disks remain outside the prefix rule's coverage. |

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

## Partial CRX, FLAC, WinHelp, JP2, SZDD, NetCDF, PDB and Stata

| Format | Review decision |
| --- | --- |
| CRX | Distinguish CRX2's complete 16-byte header and nonzero key/signature lengths (at most 65536 each) from CRX3's 12-byte header and nonempty protobuf header. |
| FLAC | Require complete STREAMINFO, its 34-byte size, defined first-block type, block sizes at least 16 and nonzero sample rate. Preserve the last-metadata flag and the entire 20-bit sample-rate range. |
| WinHelp | Require the complete 16-byte header, plausible directory/file-size fields and the inherited no-free-chain signature. |
| JP2 | Require the full binary signature box and exact `jp2 ` brand, with a complete, aligned FTYP fixed header. OpenJPEG accepts an empty compatibility list, so the minimum is 28 bytes rather than 32. |
| SZDD | Require the complete 14-byte normal header and all eight signature bytes. Preserve mode B documented by libmagic for early Windows releases, as well as normal mode A. |
| NetCDF | Require the 32-byte minimum CDF1/CDF2 header. A nonempty dimension list requires its dimension tag; preserve the reader's tolerance for empty lists. CDF5 and HDF5-backed variants remain outside the rule. |
| PDB | Require complete MSF7/JG binary magic, 56/60-byte fixed headers and supported page sizes. MSF7 also checks its free-page-map selector. Portable PDB remains outside this rule. |
| Stata | Correlate a three-digit release with the complete byte-order field, requiring 63 bytes. Preserve both byte orders and do not cap future releases at 119. Pre-117 binary layouts remain outside the rule. |

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

## Coverage limits

Lua 5.1/5.2 retain OpenWrt LNUM numeric modes alongside stock layouts. Mach-O
checks thin headers, not fat containers. Raw BAM identifies the uncompressed
stream; ordinary BGZF-wrapped BAM needs decompression and falls to the outer
gzip signature. These limitations are exercised by the regression suite.

Additional format sources: [gzip](https://www.rfc-editor.org/rfc/rfc1952),
[EPUB ZIP requirements](https://www.w3.org/TR/epub-33/#sec-zip-container-mime),
[WebAssembly modules](https://webassembly.github.io/spec/core/binary/modules.html),
[Photoshop formats](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/),
[ORC](https://orc.apache.org/specification/ORCv1/),
[pandas SAS constants](https://github.com/pandas-dev/pandas/blob/main/pandas/io/sas/sas_constants.py),
[QOI specification](https://qoiformat.org/qoi-specification.pdf),
[Khronos GLB specification](https://github.com/KhronosGroup/glTF/blob/main/specification/2.0/Specification.adoc#glb-file-format-specification),
[WOFF](https://www.w3.org/TR/WOFF/),
[WOFF2](https://www.w3.org/TR/WOFF2/),
[NumPy format documentation](https://numpy.org/doc/stable/reference/generated/numpy.lib.format.html),
[ufbx's reader](https://github.com/ufbx/ufbx/blob/master/ufbx.c),
[GIF89a specification](https://www.w3.org/Graphics/GIF/spec-gif89a.txt),
[PCAP format draft](https://www.ietf.org/archive/id/draft-ietf-opsawg-pcap-05.html),
[libsndfile's AU reader and writer](https://github.com/libsndfile/libsndfile/blob/master/src/au.c),
[devkitPro's 3DSX writer](https://github.com/devkitPro/3dstools/blob/master/src/3dsxtool.cpp),
[MP4 registration authority's brands](https://github.com/mp4ra/mp4ra.github.io/blob/main/data/brands.csv),
[GPAC's FileTypeBox reader](https://github.com/gpac/gpac/blob/master/src/isomedia/box_code_base.c),
[LZ4 frame specification](https://github.com/lz4/lz4/blob/dev/doc/lz4_Frame_format.md),
[Zstandard frame specification](https://github.com/facebook/zstd/blob/dev/doc/zstd_compression_format.md).

## Container and PE preprocessor rules

The zip and PE preprocessors (`rust/lib/src/rules/preprocess/`) derive integer facts
and two views from the first block, the input size and, for archives, the last 16 KiB,
extended back to a central directory of at most 256 KiB that starts before it. Rules on
that stream decide Office Open XML, OpenDocument, JAR, APK and EPUB packages from whole
central directory names, the stored `mimetype` entry and the content type declared in
an inflated `[Content_Types].xml` first entry, and PE images from the COFF and optional
headers. Every prior `PK\x03\x04` candidate for these formats matched 6.5% of
other-label files; the lifted rules match none.

Evidence: the adjudicated combined corpus (`snapshot-b2f86528d705`, the
`snapshot-1411a5c0fd4a` snapshot with the 250 byte-verified truth corrections folded
into its `identity.label_adjudication.changes`, 25,421 whole files, hydrated at
`tmp/comparison-mmap-spike-24b86573/files` of the code lane's worktree), scanned
by the release CLI built from this change
(`cargo build --release --manifest-path rust/cli/Cargo.toml --features yara-rules`,
CPU runtime from `rust/runtime-plugin`, Vectorscan 5.4.13) through
`magika_rules_benchmark.runner.observe`: the product in `--rules=off` and
`--rules=enforce` per 128-file batch, and YARA-X with the Python facts oracle as the
reference. Native and reference decisions agree on every file: 13,630 rule decisions
(53.62% of the corpus, up from 9,624 = 37.86% before the preprocessors), 0 enforced
false positives, 0 conflicts, 0 reference errors, 0 engine mismatches. The run script,
summary and raw rows (`task5-parity.py`, `parity-summary.json`, `parity-rows.json.gz`)
are recorded with the change: binary SHA-256 `66d0292e…`, pack `02b4d30c…`, raw rows
`3d60a816…`.

| Label | Files | Correct | Wrong | Precision | Recall | Rules |
|---|---:|---:|---:|---:|---:|---|
| docx | 522 | 498 | 0 | 1.000 | 0.954 | taxonomy_docx |
| dotx | 184 | 177 | 0 | 1.000 | 0.962 | taxonomy_dotx |
| xlsb | 170 | 158 | 0 | 1.000 | 0.929 | taxonomy_xlsb |
| xlsx | 125 | 125 | 0 | 1.000 | 1.000 | taxonomy_xlsx |
| pptx | 90 | 86 | 0 | 1.000 | 0.956 | taxonomy_pptx |
| jar | 117 | 80 | 0 | 1.000 | 0.684 | taxonomy_jar |
| odt | 101 | 77 | 0 | 1.000 | 0.762 | taxonomy_odt |
| ods | 113 | 98 | 0 | 1.000 | 0.867 | taxonomy_ods |
| odp | 101 | 98 | 0 | 1.000 | 0.970 | taxonomy_odp |
| epub | 123 | 95 | 0 | 1.000 | 0.772 | taxonomy_epub (95), taxonomy_epub_names (88) |
| apk | 613 | 606 | 0 | 1.000 | 0.989 | taxonomy_apk (468), taxonomy_apk_names (606) |
| pe (`pebin`) | 2,578 | 2,574 | 0 | 1.000 | 0.998 | taxonomy_pebin |

Recall is the union of the rules per label; each rule's own `fn_rate` metadata is its
single-rule miss rate on this corpus, and xlsx, with no miss, moves to the `full`
bucket. The names walk covers the whole directory, top-level names first, and a
directory that starts before the tail window is read on its own up to 256 KiB (the
largest in the corpus spans 171 KB); together they took APK names recall from 541 to
606, pptx from 80 to 86, jar from 62 to 80 and xlsx from 117 to 125. The remaining
misses are structural: resource-only JARs with a manifest but no `.class`, one zip64
epub, OpenDocument or EPUB packages whose `mimetype` entry is deflated, not first or
carries an extra field so no `mimetype=` line is written (24 odt, 15 ods, 3 odp, 35
epub for the names rule), and four PE images, three with the undocumented machine
0xEC20 and one with subsystem 0.

Docx, dotx and xlsb are decided from the first entry. A Word document and a template
list the same parts and differ only in the main content type declared inside
`[Content_Types].xml`, which Office writes as the archive's first entry, deflated and
small enough to lie whole in the first 4 KiB. The rules inflate it into
`zip_first_entry` and test the declared type (document or macro-enabled document,
template or macro-enabled template, binary workbook). The misses are packages whose
first entry is not a held content types stream: 18 docx and 1 dotx streamed with a
data descriptor, 5 docx, 6 dotx and 12 xlsb whose first entry is another part, and one
docx whose content types stream does not fit the first block.

The AAR probe in `tests_data/rules_negative` found the previous `taxonomy_apk`
manifest-first branch accepting any archive that opens with `AndroidManifest.xml`, an
Android library included. That branch now also needs a `classes.dex` or `resources.arsc`
local entry within the prefix, which 467 of the 570 manifest-first corpus APKs carry,
and `taxonomy_apk_names` decides the rest from the directory. Union recall is 606/613
(0.989), above the 605/613 of the looser prefix rule it replaces, and an Android
library no longer receives a deterministic wrong label.

Regressions: `rules/benchmark/tests/test_rule_regressions.py` exercises each lifted
rule with hand-built archives and PE images (positive layouts, lookalike names, names
in comments or member data, names beyond the view, split and zip64 end records,
OpenDocument subtypes and deflated or misplaced `mimetype` entries, PE machine,
subsystem, characteristics and header-holding boundaries), the nine crafted probes in
`tests_data/rules_negative` (`RESULTS.md`), and the real fixtures under `tests_data`
through the native parity test. `rules/benchmark/tests/golden/preprocess.json` pins
the native facts and both view digests of 22 fixtures for the Python oracle; regenerate it
with `MAGIKA_WRITE_GOLDEN=<path> cargo test --manifest-path rust/lib/Cargo.toml
--features yara-rules golden_facts`.

### Timing

Whole-process rules-only timings on the shared Apple Silicon host, warm caches,
Hyperfine 1.20.0 with `--shell=none`. "Before" is the code lane's distributed CLI
(`tmp/rules-pr-worktree/target/distrib/magika-cli-aarch64-apple-darwin/magika`,
SHA-256 `3775430d…`, bundled rules without container or PE rules); "after" is the
release CLI of this change (SHA-256 `9c7fab4a…`). The 1,000-file rows replay the
corpus's saved workloads (`workloads.json`, hit mixes defined by the earlier rule set)
with tool defaults, 1 warmup and 10 runs; the single-file rows use 3 warmups and 40
runs. Speed is unchanged: the preprocessors run only for `PK`/`MZ` prefixes, the
archive tail read costs one bounded `pread`, and the facts stream is scanned only when
a preprocessor produced something.

| Workload (`--rules=only`) | before, mean ± σ | after, mean ± σ |
|---|---:|---:|
| one PNG (`basic/png/magika_test.png`) | 2.8 ± 0.1 ms | 3.0 ± 0.1 ms |
| one docx (`basic/docx/doc.docx`, 292 KB) | 2.9 ± 0.1 ms | 3.1 ± 0.1 ms |
| one PE (`mitra/pebin/pe64.exe`) | 2.8 ± 0.0 ms | 3.0 ± 0.1 ms |
| 1,000 files, natural mix | 60.5 ± 1.7 ms | 60.4 ± 0.4 ms |
| 1,000 files, 0% hits | 59.1 ± 0.6 ms | 59.5 ± 0.3 ms |
| 1,000 files, 100% hits | 57.1 ± 0.9 ms | 57.2 ± 0.4 ms |

The 0.2 ms single-file difference is the binary build, not the rules: on the same
after binary, the PNG takes 3.3 ± 0.1 ms with the before pack (`--rules-file`) and
3.3 ± 0.0 ms with the after pack (60 runs, `isolate-png.json`), and the startup traces
differ only in the source hashing of the larger pack (`rules_cache_identity` 0.48 ms
against 0.60 ms; `native_scan` 51 µs against 33 µs). The distributed before binary was
built by `rust/build-runtime.py`; the after binary by a plain release build.

The directory read, the top-level-first walk and the first-entry view were re-timed
against the same before binary with 5 warmups and 100 runs, two alternating rounds per
binary, on the same host under load (load average 2.4 to 4.1), so these rows are noisier
than the table above:

| Input (`--rules=only`) | decision before → after | before, two rounds | after, two rounds |
|---|---|---:|---:|
| `basic/png/magika_test.png` | unknown → unknown | 3.6, 3.6 ms | 3.8, 3.7 ms |
| corpus docx, `[Content_Types].xml` first (`001ea7af…`) | unknown → docx | 3.8, 3.7 ms | 3.9, 3.9 ms |
| corpus apk, directory before the 16 KiB window (`0b725900…`) | unknown → apk | 4.3, 4.1 ms | 4.7, 3.8 ms |
| `mitra/pebin/pe64.exe` | unknown → pebin | 4.4, 3.9 ms | 4.1, 5.5 ms |

The spread between rounds of one binary (up to 1.6 ms) exceeds every before/after
difference, so no regression is measurable at this resolution; a quiet-host rerun is
owed before quoting sub-millisecond deltas.

The same corpus run through `magika-compare` (config, results and observations under
`tmp/task4/compare-run`, `results.json` SHA-256 `971fda7b…`, revision
`96e16f91-dirty-task4`, 3 runs after 1 warmup, tool defaults) measures rules-only
medians of 3.08 ms (before) and 3.40 ms (after) for one file and 60.16 ms and 59.20 ms
for the natural 1,000-file workload, rules + ML on CPU at 3.40 ms and 141.98 ms, and
records the quality above for every tool: before rules-only 9,624 decisions, after
12,700, both at precision 1.0 with zero errors; rules + ML accuracy 78.95% on the
corpus. The run is not a protocol 1.2.x catalogue entry: it measures four Magika
configurations for this change, not the cross-tool default set.

## Prefix signatures for labels the model lacks or misses

Ani, arrow, pcapng and xcoff are Magika content types the model cannot output. On the
adjudicated combined corpus it labels animated cursors mostly as ico, pcapng captures
mostly as pcap or unknown, XCOFF objects mostly as unknown, coff or elf, and Arrow IPC
files as unknown or txt, so every such file is wrong without a rule. Each rule reads its
format's fixed header and decides every file of its label with no wrong decision. The
five disabled `notworking` candidates they replace matched two to six bytes of magic.

The model does output pem and postscript. Their rules decide only files the model
already labels correctly, so they skip inference without changing accuracy. The model's
pem errors are 57 VMware ESX VIB packages (ar archives opening with a `descriptor.xml`
member) that carry the pem truth label, and two text files, an installer script and a
log, that do not open with a PEM block. Its PostScript errors are files that open with
`%!` but carry no `%!PS-Adobe` header. The PostScript rule requires that ten-byte header,
because a bare `%!PS` opening is shorter than the eight observed bytes every active rule
needs.

Evidence: the corpus, runner and oracle of the preprocessor section above, with the
release CLI and bundled pack of this change (binary SHA-256 `0d929936…`, pack
`13b7408c…`, raw rows `96c3e046…`). Native and reference decisions agree on every file:
14,128 rule decisions, up from 13,630 before these rules, with 0 enforced false
positives, 0 conflicts, 0 reference errors and 0 engine mismatches. The per-label rows of
the preprocessor table are unchanged.

| Label | Files | Rule correct | Wrong | Model correct | Rule (bucket) | Signature |
|---|---:|---:|---:|---:|---|---|
| ani | 100 | 100 | 0 | 0 | taxonomy_ani (full) | `RIFF` with form type `ACON` |
| arrow | 7 | 7 | 0 | 0 | taxonomy_arrow (full) | `ARROW1` padded to eight bytes |
| pcapng | 117 | 117 | 0 | 0 | taxonomy_pcapng (full) | Section Header Block, byte-order magic, version 1 |
| xcoff | 105 | 105 | 0 | 0 | taxonomy_xcoff (full) | XCOFF32 or XCOFF64 magic, auxiliary header size, 1 to 1,024 sections |
| pem | 171 | 109 | 0 | 112 | taxonomy_pem (partial) | RFC 7468 certificate and key labels at offset 0 |
| postscript | 161 | 60 | 0 | 79 | taxonomy_postscript (partial) | `%!PS-Adobe` at offset 0 |

The pem rule misses 62 files. Fifty-nine are the files above that do not open with a PEM
block; the other three hold a PEM block after a comment line, after blank lines, or after
PKCS#12 bag attributes. The PostScript rule misses 101 files. Eighty-two are files the
model also gets wrong, 80 of them with binary bytes within 64 bytes of the `%!`. The
other 19 are labeled correctly by the model: 9 Adobe font lists (`%!Adobe-FontList`), 2
bare `%!PS` openings, 2 binary openings and 6 bare `%!` openings followed by comments or
code.

Rule shape matters for the compiled pack more than rule count. Long hex strings with
wildcards and multi-byte range comparisons multiply anchored automaton states: the first
drafts of ani (`RIFF ?? ?? ?? ?? ACON`) and pcapng (with a four-byte block-length range)
compiled to 1,804,208 bytes, and a hex alternation for the XCOFF auxiliary header size
added 75 KB more. The kept forms read the ani form type as one exact integer and put the
pcapng version inside its literal. Compiled with `magika --compile-rules`, two runs each:

| Bundled pack | Compiled size | Compile time |
|---|---:|---:|
| without the six rules | 1,660,464 bytes | 9.47 s, 9.36 s |
| first rule drafts | 1,804,208 bytes | 11.5 s, 11.5 s |
| this change | 1,782,896 bytes | 11.17 s, 11.00 s |

Whole-process rules-only timings use the release binary of this change with each pack
through `--rules-file`, Hyperfine 1.20.0 with `--shell=none`, on the shared host under
load (load average 6.1 to 6.5). The single-file row uses 3 warmups and 40 runs; the
1,000-file rows replay the saved workloads above, whose hit mixes predate these rules,
with 1 warmup and 10 runs. Every difference is inside the spread, so the six rules add no
measurable scan time; the extra compile time is paid once per rule cache.

| Workload (`--rules=only`) | without the six rules | with the six rules |
|---|---:|---:|
| one PNG (`basic/png/magika_test.png`) | 3.6 ± 0.2 ms | 3.4 ± 0.2 ms |
| 1,000 files, natural mix | 66.9 ± 1.7 ms | 65.6 ± 0.6 ms |
| 1,000 files, 0% hits | 70.4 ± 7.2 ms | 67.4 ± 4.6 ms |

Thirteen more signatures measured on the same corpus decide labels the model does not
predict: minidump, hve, intelhex, grib, safetensors, pbm, ply, geopackage, cubin, jng,
palmos, nrrd and OSM PBF. The taxonomy has since gained these labels (OSM PBF under
`osm`), so each ships as an enforced rule in `rulesets/` with its measured miss rate,
rather than waiting as a non-canonical candidate.

Regressions: `test_prefix_signatures_for_labels_the_model_lacks_or_misses` in
`rules/benchmark/tests/test_rule_regressions.py` checks a positive and a near miss for
each label, including a bare `%!PS` opening, and the crafted `xcoff_2bytes.bin` probe in
`tests_data/rules_negative` abstains.

### ASF, ASCII FBX, LuaJIT and JSON glTF

After the prefix signatures, a whole-corpus ranking of the files rules plus the model
still label wrong put four canonical labels near the top that the model cannot output
and rules only partly covered: ASF (0 of 861 decided), FBX (137 of 165), Lua bytecode
(77 of 178) and glTF (101 of 167). Each gap had one structural cause and a header that
separates it:

- All 861 ASF files open with the header object GUID, 4 to 13 header objects and the
  reserved bytes 1 and 2. `taxonomy_asf` moves from `notworking` to `full` with those
  checks. It labels every ASF container asf, as the adjudicated corpus does; the
  taxonomy's wma and wmv labels have no corpus samples, and separating them needs stream
  properties beyond the header.
- The 28 FBX misses are ASCII exports opening with the SDK header comment
  `; FBX 7.3.0 project file`, which `taxonomy_fbx` now accepts beside the binary header.
- The 101 Lua bytecode misses are LuaJIT 2 dumps. Their flags use only the strip and FR2
  bits; 91 stripped dumps continue with the first prototype's length, flags, parameter
  count and frame size, and 10 carry a chunk name starting with `@`. `taxonomy_luabytecode`
  accepts both shapes and moves from `partial` to `full`.
- The 66 JSON glTF files open with the accessor array (35) or a flat asset object (31),
  and no other corpus file opens with a glTF top-level key. Accessor-first files name
  `"componentType"` within 82 bytes; asset-first files follow the asset object with
  `scene`, `scenes`, `accessors` or a `KHR_`/`EXT_` extension list. `taxonomy_gltf`
  anchors both shapes at offset 0, so ordinary JSON fails at its first key, and a 3D
  Tiles tileset, which also opens with an asset object, abstains at the key after it.

| Label | Files | Decided before | Decided after | Wrong | Rule (bucket) |
|---|---:|---:|---:|---:|---|
| asf | 861 | 0 | 861 | 0 | taxonomy_asf (full, from notworking) |
| fbx | 165 | 137 | 165 | 0 | taxonomy_fbx (full) |
| luabytecode | 178 | 77 | 178 | 0 | taxonomy_luabytecode (full, from partial) |
| gltf | 167 | 101 | 167 | 0 | taxonomy_gltf (full) |

Evidence: the same corpus, runner and oracle, with the release CLI and bundled pack of
this change (binary SHA-256 `fc1fc49d…`, pack `2913a6c5…`, raw rows `2d5e0591…`).
Native and reference decisions agree on every file: 15,184 rule decisions, up from
14,128, with 0 enforced false positives, 0 conflicts, 0 reference errors and 0 engine
mismatches. None of the four labels is a model output, so the model decides none of
these files correctly.

The compiled pack grows from 1,782,896 to 1,900,464 bytes and compiles in 11.37 s
and 11.88 s, against 10.67 s and 10.84 s before. The LuaJIT branches are the largest
share (60,672 bytes when swapped in alone); one merged LuaJIT pattern was both weaker
and larger (1,852,784 bytes). A first JSON glTF draft searched for `"version"`, `"nodes"`
and `"accessors"` anywhere in the prefix, which ordinary JSON files match; the anchored
form costs 6,336 more pack bytes and produces no match on them.

Rules-only timings on the same release binary with each pack through `--rules-file`,
Hyperfine 1.20.0 with `--shell=none`, 2 warmups and 15 runs per round, packs alternated
over three rounds on the shared host under load (one-minute load average 3.2 to 3.6):

| Workload (`--rules=only`) | committed pack, three rounds | this change, three rounds |
|---|---:|---:|
| 1,000 files, natural mix | 67.8, 67.3, 67.2 ms | 67.7, 69.8, 67.8 ms |
| 1,000 files, 0% hits | 66.4, 66.3, 66.2 ms | 66.9, 67.0, 66.5 ms |

The medians differ by 0.5 ms on the natural mix and 0.6 ms on the 0% hits mix per 1,000
files, under 1% and at the edge of the round-to-round spread, so any scan-time cost of the
four rules is below about 0.6 µs per file at this resolution.

PDF stays without a rule. All 310 corpus PDFs open with `%PDF-` and no other label does,
but the model labels 208 of them ai, and Illustrator files are PDF-compatible and open
with `%PDF-` too; the model outputs both labels, so a prefix rule would override a
distinction it cannot see.

Regressions: the prefix signature table in `rules/benchmark/tests/test_rule_regressions.py`
gains a positive and a near miss for asf (reserved bytes 1 and 1), fbx (no version),
luabytecode (an unknown LuaJIT flag bit) and gltf (a 3D Tiles tileset).

### VMware bundles, Illustrator artwork and the corpus corrections

Three sets of files carried a truth label that their own bytes disprove. The corpus
labels are auto-derived (`label_basis` "trid_and_local_file_model_gap_v1; not independent
re-adjudication"), so byte-exact evidence overrides them, the way the model's own
prediction did for the seven APK splits of patch 136.

- 57 files labelled pem are VMware Installation Bundles: a Unix ar archive whose first
  member is `descriptor.xml` holding a `<vib version="...">` root. The signature sits at
  fixed offsets (ar magic at 0, `descriptor.xml` at 8, `<vib version` at 68) and appears
  in exactly those 57 files of the 25,421. `taxonomy_vib` decides all 57; the new `vib`
  output label carries them. The model labelled them deb, the other ar-archive package.
- 191 files labelled pdf are Adobe Illustrator artwork: they embed Illustrator private
  and editing data (`/PieceInfo << /Illustrator`, `AIPrivateData`) or declare
  `illustrator:Type>Document`. That evidence sits 43 KB to 990 KB into the file, past the
  4 KiB a rule sees, so no rule decides them, but Magika's model already predicts ai for
  all 191. The correction credits a model that was right; it changes no code.
- 2 files labelled pem are a PowerShell installer and a tab-separated log, each with one
  PEM block embedded 1 to 3 KB in. Their primary content is the script and the log, which
  the model already predicts (powershell, tsv).

The corrections are made in the corpus's own `identity.label_adjudication.changes` — the
mechanism it already uses for the APK adjudications of patch 136 — and applied to
`samples[].truth` (250 changes: 191 pdf to ai, 57 pem to vib, one each pem to powershell
and pem to tsv), each carrying the per-file byte evidence above. The corrected corpus's
truth digest is `b2f86528…` and `27e48c7c…` (with size); folding these into the published
dataset digests is separate catalogue work.

Evidence, against the corrected snapshot (bytes unchanged, truth from the adjudication
changes): 16,754 rule decisions, 0 enforced false positives, 0 conflicts, 0 engine
mismatches.

| Label | Files | Model correct | Rule correct | Rule (bucket) |
|---|---:|---:|---:|---|
| vib | 57 | 0 | 57 | taxonomy_vib (full) |
| ai | 191 | 191 | 0 | model only (evidence past 4 KiB) |
| pdf | 119 | 102 | 0 | model only |
| pem | 112 | 112 | 109 | taxonomy_pem (partial) |

Before the correction the corpus scored the model wrong on all 191 Illustrator files and
counted a correct pem rule against 57 bundles it never claimed. PDF keeps no rule: every
corpus PDF opens with `%PDF-`, but so does Illustrator artwork, and the distinguishing
bytes are too deep for the prefix; the model already separates them.

Regressions: `test_prefix_signatures_for_labels_the_model_lacks_or_misses` gains a vib
positive (ar header built to land the descriptor at byte 68) and a near miss (a Debian ar
archive whose first member is `debian-binary`, which abstains).

### XML dialects and more zip packages

The 28 output labels added for the corpus formats let the rules those labels enable
finally ship. Each targets files the model has no class for and answers with a generic
label: XML dialects it calls xml, and zip packages it calls zip or an Office type.

Four XML-root rules read the bounded prefix and match the document's root element, with a
zip guard (`uint16(0) != 0x4B50`) so a stored copy of the format inside a zip does not
match. Five zip rules read the `zip_names` central-directory view. On the corrected corpus
none makes a wrong decision:

| Label | Files | Decided | Rule (bucket) | Signature |
|---|---:|---:|---|---|
| collada | 182 | 182 | taxonomy_collada (full) | `<COLLADA` root |
| gpx | 166 | 166 | taxonomy_gpx (full) | `<gpx` root |
| kml | 151 | 151 | taxonomy_kml (full) | `<kml` root |
| osm | 249 | 249 | taxonomy_osm (partial) + taxonomy_osm_pbf | `<osm` root, or the PBF blob |
| qgis | 40 | 40 | taxonomy_qgis (full) | a `.qgs` project in the directory |
| visio | 53 | 53 | taxonomy_visio (full) | `visio/document.xml` in the directory |
| keras | 55 | 55 | taxonomy_keras (full) | the weights and metadata parts |
| kmz | 126 | 98 | taxonomy_kmz (partial) | `doc.kml` in the directory |
| msix | 181 | 178 | taxonomy_msix (partial) | AppxManifest.xml and AppxBlockMap.xml |
| 3mf | 181 | 179 | taxonomy_3mf (partial) | `3D/3dmodel.model` in the directory |

Together with the osm PBF rule, osm reaches 249/249. The remaining kmz, msix and 3mf misses
are packages whose central directory begins beyond the 256 KiB window. The model labels
every one of these files wrong (xml, zip, or an Office type), so each rule is a net gain
with no accuracy cost, lifting corpus rule decisions to 18,004 with 0 enforced false
positives, 0 conflicts and 0 engine mismatches.

MSIX and the zip64 3MF packages are decided because the zip preprocessor now walks zip64
archives: when the end-of-central-directory record carries a zip64 locator whose zip64 end
record is held, its 64-bit entry count, directory size and offset are read and the central
directory is walked exactly as a 32-bit one, so every `zip_names` rule fires on zip64
packages too (msix 0 to 178, 3mf 116 to 179). A zip64 archive whose end record is not held,
or a sentinel without a locator, still reports a plain invalid zip. The XML-root rules match
the root element by a bounded search of the first 256 bytes rather than a true parse, so a
document embedding one of these root tokens in its first quarter-kilobyte could match; none
does on the corpus.

### Sembiance validation and its four new rules

The rules are tuned on the adjudicated combined corpus, so the test that matters is an
independent one. Run over the Sembiance v3 corpus (2,400 files, 167 truth labels, a
retro-computing distribution the adjudicated corpus does not overlap), the bundled rules
make 804 decisions with 0 enforced false positives, 0 conflicts and 0 engine mismatches.
The zero-false-positive gate holds off-distribution.

Auditing Sembiance the same way as the adjudicated corpus (one-directional model
disagreements, and a magic-vs-truth signature scan) found no byte-provable label errors:
the gif files that fail the magic check are MacBinary-wrapped (a 128-byte header before
the GIF payload, whose label is still gif), and the pem outliers carry a text preamble
before the armor. It surfaced four formats the model has no class for, each with a clean
signature that matches nothing else across both corpora (147,000+ files):

| Label | Sembiance files | Decided | Rule (bucket) | Signature |
|---|---:|---:|---|---|
| pgp | 10 | 10 | taxonomy_pgp (full) | `-----BEGIN PGP ` armor |
| step | 11 | 11 | taxonomy_step (full) | the `ISO-10303-21;` header |
| ilbm | 124 | 110 | taxonomy_ilbm (partial) | a FORM whose type is ILBM, ACBM or PBM |
| koala | 35 | 29 | taxonomy_koala (partial) | the compressed Koala header |
| degas | 121 | 49 | taxonomy_degas (partial) | resolution word 0-2 and the 32034/32066-byte screen dump |

pgp was a canonical label already; its disabled `notworking` rule (a two-byte binary
alternative with a measured false positive) is removed in favour of the armor rule. The
ilbm, step and koala output labels are added. The adjudicated-corpus gate is unchanged:
none of these labels has a file there, and none of the four rules fires on one.

Other Sembiance formats are left without a rule. printfox, pcpaint, acorn_sprite and iges resolve on structure a bounded prefix
cannot fix (only uncompressed Degas, at its exact screen size, is decidable), and iso's `CD001` marker sits at offset 32769, far beyond the 4 KiB the rules
see. Adding those would need either a real parse or a new preprocessor.
