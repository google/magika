# Crafted false-positive probes

Binary: `rust/cli` built at 5607dda with `--release --features yara-rules`.
Vectorscan: `tmp/rules-research/vectorscan-build/lib/libhs.5.4.13.dylib` via `MAGIKA_VECTORSCAN_LIBRARY`.
Command: `magika --rules off -l *` then `magika --rules enforce -l *` in this directory.

| file | what it is | rules off | rules enforce | rule (bucket) |
|---|---|---|---|---|
| ac10_text.txt | text "AC10 part number list…" | txt | **dwg** | taxonomy_dwg (full) |
| arc_1a02.bin | `1a 02 41 42` + random | unknown | **arc** | taxonomy_arc (full) |
| avi_only_at8.bin | `XXXXXXXXAVI ` + random, no RIFF | unknown | **avi** | taxonomy_avi (full) |
| cws_text.txt | text "CWS hello world…" | txt | **swf** | taxonomy_swf (full) |
| dna_mpegts.txt | ACGT×250 with G at 0,188,376,564,752 | unknown | **mpegts** | taxonomy_mpegts (full) |
| empty_ish_sas.txt | 3 bytes "SAS" | txt | **sas** | taxonomy_sas (full) |
| flv_text.txt | text "FLV Studio release notes" | txt | **flv** | taxonomy_flv (full) |
| gzip_2bytes.bin | 2 bytes `1f 8b` | unknown | **gzip** | taxonomy_gzip (full), truncated |
| gzip_escape_literal.txt | literal text `\037\213 some text` | txt | **gzip** | taxonomy_gzip un-decoded escape |
| ico_4bytes.bin | `00 00 01 00` + random | unknown | **ico** | taxonomy_ico (partial) |
| jxl_2bytes.bin | `ff 0a` + random | unknown | **jxl** | taxonomy_jxl (full) |
| msa_wasm.bin | `msa\0` + random (byte-reversed magic) | unknown | **wasm** | taxonomy_wasm (full) |
| ogg_4bytes.bin | 4 bytes "OggS" | txt | **ogg** | taxonomy_ogg (partial), truncated |
| orc_text.txt | text "ORCID: 0000-…" | yaml | **orc** | taxonomy_orc (full) |
| pk_escape_literal.txt | literal text `PK\003\004 text` | txt | **epub** | taxonomy_epub un-decoded escape |
| psd_escape_literal.txt | literal text `8BPS  \000\000\000\000 text` | txt | **psd** | taxonomy_psd un-decoded escape |
| sas_text.txt | text "SAS Viya deployment notes…" | txt | **sas** | taxonomy_sas (full) |
| tiff_be_3d3d.tif | big-endian TIFF, IFD offset 0x3d3d | tiff | **3dsm** | taxonomy_3dsm (full) |
| wave_only_at8.bin | `XXXXXXXXWAVE` + random, no RIFF | unknown | **wav** | taxonomy_wav (full) |
| xcoff_2bytes.bin | `01 f7` + random | unknown | **xcoff** | taxonomy_xcoff (partial) |

20 of 20 probes are mislabeled with rules enforced; all are handled correctly (txt/unknown/tiff) with rules off.

These files are the seed of a negative-corpus regression test: every one must abstain (or the rule must be demoted) before the corresponding rule ships enabled.
