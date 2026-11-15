// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tier 4 (`offset_probes`): length-gated signatures at non-zero offsets, trailers, or
//! structural headers that do not have an exclusive magic prefix at offset 0.
//!
//! # Why This Tier Runs After `magic32` / `magic16` / `magic8`
//! Tiers 1–3 (`magic32`, `magic16`, `magic8`) claim all formats with distinctive magic bytes
//! starting at offset 0. Anything that reaches Tier 4 has already failed those fast-path tables
//! (either because its magic lives at a non-zero offset / file trailer, or because its leading
//! bytes are generic values shared with higher-priority Tier 1–3 formats).
//!
//! # Internal Stage Order (and Why It Matters)
//! Probes inside [`probe`] are arranged in four deliberate stages:
//!
//! 1. **Core containers and trailers** — High-prevalence formats (`ISOBMFF`, `TFLite`, `DICOM`,
//!    `DuckDB`, `ESE`, `ICC`, `TAR`, `ISO 9660`, `DMG`, `HFS`, `LevelDB`, `RocksDB`, etc.).
//!    - *Latency*: common formats short-circuit before scanning long-tail tables.
//!    - *Priority*: core containers with leading zero/numeric preambles (e.g., ISO 9660's 32 KiB
//!      zero system area, DICOM's 128-byte preamble, ISOBMFF's 4-byte box size, FileMaker's
//!      `00 01 00 00`) must match before Stage 2's generic `@0..4` buckets or Stage 4's numeric
//!      header probes.
//! 2. **O(1) multi-byte fixed-offset `match` tables** — Standalone binary formats grouped by
//!    shared signature offset and slice width, ordered by **ascending offset** (`@0..4` through
//!    `@1080..1082`), with **4-byte tables before 2-byte tables** at the same offset.
//!    - *Locality*: lower offsets require shorter prefixes and stay in the first cache line.
//!    - *Secondary `@0..4` / `@0..2` tables*: these appear here (rather than Tier 1/2) because
//!      their leading bytes are generic values (`00 00 00 00`, `01 00 00 00`, `4C 00 00 00`,
//!      `00 00 02 00`, `00 00 00 28`, `2C 00 00 00`) or lower-priority prefixes that must yield
//!      to Tier 1–3 (`magic32`/`magic16`/`magic8`) and Stage 1 core containers first.
//! 3. **Unique-offset inline-guarded probes** — Formats whose fixed magic sits at an offset no
//!    other format shares (where a `match` table would have only one arm), ordered from exact
//!    disk-image size checks and lowest offset (`@1..5`) to highest (`@1464..1468`).
//! 4. **Structural, multi-offset, and numeric-header probes** — Formats without a single fixed
//!    constant magic (multiple variant offsets, numeric field ranges, block-size multiples, or
//!    `memchr` scans). Placed last so every fixed-magic format wins first, and bucketed by
//!    minimum prefix length (`>= 512 B`, then `>= 64 B`, then smaller/variable-size probes) so
//!    short files skip entire blocks in a single branch.
//!
//! # Why Inline Pre-Guards Wrap `super::probe`
//! `super::probe` opens a transactional read-tracking scope (used by fixture QA/hollowing) and
//! runs the full structural verifier. Checking cheap raw-slice / size / alignment invariants
//! first rejects non-matching files in a few cycles without entering the verifier, while every
//! match still executes the full tracked verifier.

use super::*;
use crate::ContentType as Label;

/// Tier 4 (`offset_probes`): probes fixed non-zero offsets, container trailers, and
/// structural binary headers in strict priority order (Stages 1–4; see module docs).
#[inline]
pub(crate) fn probe(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    let size = ctx.size;

    // =========================================================================
    // Stage 1: Core containers and trailers (highest priority & prevalence).
    // Must run before Stages 2–4 so mainstream containers with leading zeros or
    // numeric box sizes (ISOBMFF, DICOM, ISO 9660, FileMaker, TAR, DMG, etc.)
    // are claimed before generic `@0..4` buckets and numeric-header probes.
    // =========================================================================

    // ISO-BMFF boxes and TFLite FlatBuffers identifier at @4..8.
    // Within ISO-BMFF, `ftyp` (3GP/AVIF/HEIF/MP4/QT) runs before headerless
    // QuickTime/MP4 top-level atoms (`moov`/`wide`/`mdat`/`free`/`skip`/`pnot`).
    if let Some(w4) = prefix.get(4..8) {
        let hit = match w4 {
            b"ftyp" => super::probe(|| ftyp(prefix)),
            b"moov" | b"wide" | b"mdat" | b"free" | b"skip" | b"pnot" => {
                super::probe(|| isobmff_mp4_or_qt(prefix, size))
            }
            b"TFL3" => super::probe(|| tflite(prefix, size)),
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }
    // WinACE archive: "**ACE**" signature at @7..14.
    if prefix.get(7..14) == Some(b"**ACE**".as_slice())
        && let Some(lbl) = super::probe(|| ace(prefix))
    {
        return Some(lbl);
    }
    // DICOM medical image: 128-byte preamble followed by "DICM" at @128..132.
    if prefix.get(128..132) == Some(b"DICM".as_slice())
        && let Some(lbl) = super::probe(|| dicom(prefix))
    {
        return Some(lbl);
    }
    // DuckDB database: "DUCK" magic at @8..12.
    if prefix.get(8..12) == Some(b"DUCK".as_slice())
        && let Some(lbl) = super::probe(|| duckdb(prefix))
    {
        return Some(lbl);
    }
    // Microsoft ESE (Jet Blue) database: 0x89ABCDEF LE signature at @4..8.
    if prefix.get(4..8) == Some([0xEF, 0xCD, 0xAB, 0x89].as_slice())
        && let Some(lbl) = super::probe(|| ese(prefix))
    {
        return Some(lbl);
    }
    // ICC color profile: "acsp" profile signature at @36..40.
    if prefix.get(36..40) == Some(b"acsp".as_slice())
        && let Some(lbl) = super::probe(|| icc(prefix))
    {
        return Some(lbl);
    }
    // Palm OS PRC/PDB application database: "appl" type code at @60..64.
    if prefix.get(60..64) == Some(b"appl".as_slice())
        && let Some(lbl) = super::probe(|| palmos(prefix))
    {
        return Some(lbl);
    }
    // OpenStreetMap PBF: BE32 BlobHeader length < 65536 (@0..2 == 0) + protobuf field 1 "OSMHeader" at @4..15.
    if prefix.len() >= 16
        && prefix[0..2] == [0x00, 0x00]
        && &prefix[4..15] == b"\x0a\x09OSMHeader"
        && let Some(lbl) = super::probe(|| osm_pbf(prefix))
    {
        return Some(lbl);
    }
    // LHA / LHarc archive: hyphen-delimited 5-byte method tag (e.g. "-lh5-") at @2..7.
    if prefix.len() >= 24
        && prefix[2] == b'-'
        && prefix[6] == b'-'
        && let Some(lbl) = super::probe(|| lha(prefix, size))
    {
        return Some(lbl);
    }
    // POSIX ustar / GNU / V7 TAR: 512-byte header starting with a printable ASCII filename byte.
    // Checked before ISO 9660 so TAR archives >= 32 KiB match at @0 without scanning to 0x8000.
    if size >= 512
        && prefix.len() >= 512
        && prefix[0].is_ascii_graphic()
        && let Some(lbl) = super::probe(|| tar(prefix, size))
    {
        return Some(lbl);
    }
    // ISO 9660 / High Sierra / UDF optical disc image: volume descriptor at sector 16 (0x8000).
    if prefix.len() >= 0x8006
        && let Some(lbl) = super::probe(|| iso(prefix, size))
    {
        return Some(lbl);
    }
    // Apple UDIF Disk Image (.dmg): 512-byte "koly" trailer at `size - 512` (in `ctx.tail`).
    if size >= 512
        && ctx.tail.is_some_and(|t| {
            t.len() >= 512 && t.get(t.len() - 512..t.len() - 508) == Some(b"koly".as_slice())
        })
        && let Some(lbl) = super::probe(|| dmg_koly(size, ctx.tail))
    {
        return Some(lbl);
    }
    // Nintendo 3DS card/content container: "NCSD" or "NCCH" at @0x100 after the 256-byte RSA signature.
    if size >= 0x200
        && prefix.get(0x100..0x104).is_some_and(|s| s == b"NCSD" || s == b"NCCH")
        && let Some(lbl) = super::probe(|| nintendo_3ds(prefix, size))
    {
        return Some(lbl);
    }
    // Berkeley DB: Btree (0x00053162), Hash (0x00061561), or Queue (0x00042553) magic at @12..16 (LE/BE).
    if size >= 64
        && prefix.get(12..16).is_some_and(|s| {
            matches!(
                s,
                [0x62, 0x31, 0x05, 0x00]
                    | [0x00, 0x05, 0x31, 0x62]
                    | [0x61, 0x15, 0x06, 0x00]
                    | [0x00, 0x06, 0x15, 0x61]
                    | [0x53, 0x25, 0x04, 0x00]
                    | [0x00, 0x04, 0x25, 0x53]
            )
        })
        && let Some(lbl) = super::probe(|| berkeleydb(prefix, size))
    {
        return Some(lbl);
    }
    // Claris / FileMaker Pro (.fp7 / .fmp12): `00 01 00 00` + "HBAM2"/"HBAM7" at @525 (or @14).
    if size >= 544
        && prefix.len() >= 540
        && let Some(lbl) = super::probe(|| filemaker(prefix, size))
    {
        return Some(lbl);
    }
    // Apple HFS ("BD"), HFS+ ("H+"), or HFSX ("HX"): volume header magic at @1024..1026.
    if size >= 1026
        && prefix.get(1024..1026).is_some_and(|s| matches!(s, b"H+" | b"HX" | b"BD"))
        && let Some(lbl) = super::probe(|| hfs(prefix, size))
    {
        return Some(lbl);
    }
    // IGES CAD exchange: 80-column card Start section sequence marker at cols 73..80.
    if size >= 80
        && prefix.get(72..80).is_some_and(|s| s == b"S      1" || s == b"S0000001")
        && let Some(lbl) = super::probe(|| iges(prefix, size))
    {
        return Some(lbl);
    }
    // Google LevelDB SSTable: 48-byte footer magic at `size - 8` (in `ctx.tail`).
    if size >= 48
        && let Some(lbl) = super::probe(|| leveldb(prefix, size, ctx.tail))
    {
        return Some(lbl);
    }
    // Symas LMDB: 0xBEEFC0DE LE magic at @0 (lockfile) or @16 (64-bit meta page).
    if size >= 24
        && (prefix.get(0..4) == Some([0xDE, 0xC0, 0xEF, 0xBE].as_slice())
            || prefix.get(16..20) == Some([0xDE, 0xC0, 0xEF, 0xBE].as_slice()))
        && let Some(lbl) = super::probe(|| lmdb(prefix, size))
    {
        return Some(lbl);
    }
    // MySQL / MariaDB table definition (.frm) or MyISAM index (.MYI): `FE FE` header prefix.
    if size >= 64
        && prefix.len() >= 24
        && prefix[0] == 0xFE
        && prefix[1] == 0xFE
        && let Some(lbl) = super::probe(|| mysql_storage(prefix, size))
    {
        return Some(lbl);
    }
    // NIfTI-1 neuroimaging volume: `sizeof_hdr == 348` at @0..4 + `"n+1\0"`/`"ni1\0"` at @344..348.
    if size >= 348
        && prefix.len() >= 348
        && let Some(lbl) = super::probe(|| nifti(prefix, size))
    {
        return Some(lbl);
    }
    // Oracle Database datafile/controlfile: block header magic 0x7AC2F8F0 (LE/BE) at @20..24.
    if size >= 512
        && prefix.len() >= 32
        && let Some(lbl) = super::probe(|| oracle_dbf(prefix, size))
    {
        return Some(lbl);
    }
    // Borland Paradox table (.db): header/block geometry + version ID at @0..92.
    if size >= 96
        && prefix.len() >= 92
        && let Some(lbl) = super::probe(|| paradox(prefix, size))
    {
        return Some(lbl);
    }
    // Kodak Photo CD (.pcd): 2048-byte preamble followed by "PCD_IPI" or "PCD_OPA".
    if size >= 2055
        && prefix.get(2048..2055).is_some_and(|s| s == b"PCD_IPI" || s == b"PCD_OPA")
        && let Some(lbl) = super::probe(|| pcd_kodak(prefix, size))
    {
        return Some(lbl);
    }
    // Self-executing Python Archive (.par / .pyz): `#!` shebang + embedded PKZIP/EOCD in tail.
    if size >= 64
        && prefix.starts_with(b"#!")
        && let Some(lbl) = super::probe(|| pythonpar(prefix, size, ctx.tail))
    {
        return Some(lbl);
    }
    // Realm Mobile Database: "T-DB" magic at @16..20 after the 16-byte `m_top_ref[2]` header.
    if size >= 24
        && prefix.get(16..20) == Some(b"T-DB".as_slice())
        && let Some(lbl) = super::probe(|| realm_offset(prefix, size))
    {
        return Some(lbl);
    }
    // Meta RocksDB SSTable: 53-byte footer magic at `size - 8` (in `ctx.tail`).
    if size >= 53
        && let Some(lbl) = super::probe(|| rocksdb(prefix, size, ctx.tail))
    {
        return Some(lbl);
    }
    // UEFI Firmware Volume: 16-byte zero vector + "_FVH" signature at @40..44.
    if size >= 64
        && prefix.get(40..44) == Some(b"_FVH".as_slice())
        && let Some(lbl) = super::probe(|| uefi_fv(prefix, size))
    {
        return Some(lbl);
    }

    // =========================================================================
    // Stage 2: O(1) fixed-offset `match` tables for standalone binary formats.
    // Ordered by ascending offset (shortest required prefix / best cache locality
    // first), with 4-byte slices checked before 2-byte slices at the same offset.
    // =========================================================================

    // @0..4: Secondary/fallback 4-byte prefixes that yield to Tier 1 `magic32`
    // (e.g., `00 00 00 00` null_prefixed, `00 00 00 28` ftyp, `00 00 02 00` CUR/TGA,
    // `01 00 00 00` EMF/WMF/CouchDB, `2C 00 00 00` M3P, `4C 00 00 00` LNK) and
    // Stage 1 core containers above, as well as Tier 3 `magic8` (`0xFF` streams).
    if let Some(w) = prefix.get(0..4) {
        let hit = match w {
            // Leading zero u32: fallback formats verified by deeper header signatures/layouts.
            [0x00, 0x00, 0x00, 0x00] => super::probe(|| verify_bolt_db(prefix, size))
                .or_else(|| super::probe(|| verify_burnpipe_ba(prefix, size)))
                .or_else(|| super::probe(|| verify_d3d_index_buffer_ib(prefix, size)))
                .or_else(|| super::probe(|| verify_fanuc_pmc_patch(prefix, size)))
                .or_else(|| super::probe(|| verify_fm_towns_euphony_pmb(prefix, size)))
                .or_else(|| super::probe(|| verify_huawei_update_app(prefix, size)))
                .or_else(|| super::probe(|| verify_kens_digital_kdm(prefix, size)))
                .or_else(|| super::probe(|| verify_ms_cert_store_sst(prefix, size)))
                .or_else(|| super::probe(|| verify_nol_2900_layer(prefix, size)))
                .or_else(|| super::probe(|| verify_topspeed_0205_tps(prefix, size)))
                .or_else(|| super::probe(|| verify_wolfrpg_map_mps(prefix, size))),
            [0x00, 0x00, 0x00, 0x28] => super::probe(|| verify_lionheart_lion(prefix, size)), // Thalion Lionheart Amiga music (after 40B `ftyp` in Tier 1)
            [0x00, 0x00, 0x02, 0x00] => super::probe(|| verify_pcms_ccol_wk(prefix, size)), // Caustic PCMSynth preset (after CUR/TGA in Tier 1)
            [0x00, 0x00, 0x04, 0x00] => super::probe(|| verify_adlib_rol(prefix, size)), // AdLib Visual Composer v0.4 (.rol)
            [0x00, 0x01, 0x00, 0x01] => super::probe(|| verify_mssscard_ax_log(prefix, size)), // Microsoft SmartCard trace (.ax)
            // LE u32 `1` (version/type header): fallback formats after EMF/WMF/CouchDB in Tier 1.
            [0x01, 0x00, 0x00, 0x00] => super::probe(|| verify_boyscout_bsf(prefix, size))
                .or_else(|| super::probe(|| verify_flightsim_name_air(prefix, size)))
                .or_else(|| super::probe(|| verify_igo_poi_kns(prefix, size)))
                .or_else(|| super::probe(|| verify_locale_strndx(prefix, size)))
                .or_else(|| super::probe(|| verify_scumm_cutscene_cut(prefix, size)))
                .or_else(|| super::probe(|| verify_security_cfg_cch(prefix, size)))
                .or_else(|| super::probe(|| verify_stalker_xray_ogf(prefix, size))),
            [0x04, 0x00, 0x00, 0x00] => super::probe(|| verify_scs_pmd(prefix, size)), // SCS Prism3D model v4 (.pmd)
            // LE u32 `6` (version/header size): NcStudio CNC, RE4 message table, Valve VCS v6 shader.
            [0x06, 0x00, 0x00, 0x00] => super::probe(|| verify_ncstudio_dyn(prefix, size))
                .or_else(|| super::probe(|| verify_re4_mdt(prefix, size)))
                .or_else(|| super::probe(|| verify_valve_shader_vcs(prefix, size))),
            [0x2C, 0x00, 0x00, 0x00] => super::probe(|| verify_tomofon_tomodel(prefix, size)), // Klevgrand Tomofon model (after M3P in Tier 1)
            [0x4C, 0x00, 0x00, 0x00] => super::probe(|| verify_lenovo_logdat(prefix, size)), // Lenovo XOR log (after Windows .lnk in Tier 1)
            [0x64, 0x00, 0x00, 0x00] => super::probe(|| verify_picasa_ytf(prefix, size)), // Google Picasa font metrics (.ytf)
            b"\xFF\x00La" => super::probe(|| verify_lattice_bitstream(prefix, size)), // Lattice FPGA bitstream (after 0xFF JPEG/MP3 in Tier 3)
            [0xFF, 0xFF, 0x00, 0x00] => super::probe(|| verify_cryengine_dds_mip6(prefix, size)), // CryEngine split DDS mipmap (.dds.6)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @0..2: Secondary 2-byte prefixes (checked after Tier 2 `magic16` and `@0..4` above).
    if let Some(w) = prefix.get(0..2) {
        let hit = match w {
            [0x12, 0x34] => super::probe(|| verify_sierra_agi_vol(prefix, size)), // Sierra AGI resource volume (VOL.0)
            [0x1B, 0x5B] => super::probe(|| verify_ansi_csi_art(prefix, size)), // ANSI X3.64 / ECMA-48 CSI escape art (.ans)
            [0xA0, 0x00] => super::probe(|| verify_dec_xxdp(prefix, size)), // DEC PDP-11 XXDP diagnostic disk image
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @0..4: Google BinExport v2 protobuf (runs after Tier 3 `magic8` `0x0A` FileDescriptorSet).
    // Pre-gate: field 1 `Meta` submessage tag `0x0A`, 1- or 2-byte varint length >= 38,
    // immediately followed by a valid first `Meta` field tag (`0x0A | 0x12 | 0x1A | 0x20`).
    if size >= 48
        && prefix.len() >= 48
        && prefix[0] == 0x0A
        && prefix[1] >= 38
        && ((prefix[1] < 0x80 && matches!(prefix[2], 0x0A | 0x12 | 0x1A | 0x20))
            || (prefix[1] >= 0x80
                && prefix[2] <= 0x08
                && matches!(prefix[3], 0x0A | 0x12 | 0x1A | 0x20)))
        && let Some(lbl) = super::probe(|| verify_binexport2_with_stage(prefix, size))
    {
        return Some(lbl);
    }

    // @1..3: 2-byte signatures starting after a 1-byte version/header prefix.
    if let Some(w) = prefix.get(1..3) {
        let hit = match w {
            [0x1A, 0x00] => super::probe(|| verify_pmd_m_module(prefix, size)), // NEC PC-98 PMD FM module (Part A offset == 26)
            b"GT" => super::probe(|| verify_zx_global_tracker(prefix, size)), // ZX Spectrum Polaris Global Tracker (.gtr)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @2..6: 4-byte signatures starting after a 2-byte version/length prefix.
    if let Some(w) = prefix.get(2..6) {
        let hit = match w {
            [0x00, 0x00, 0x00, 0x07] => super::probe(|| verify_ultima6_m(prefix, size)), // Ultima VI AdLib music (.m)
            [0x03, 0x00, 0x00, 0x03] => super::probe(|| verify_sinclair_ql_f3(prefix, size)), // Sinclair QL bitmap font (.f3)
            [0x07, 0x00, 0x00, 0x1E] => super::probe(|| verify_soundplayer_sjs(prefix, size)), // Amiga SoundPlayer module (sjs.*)
            [0x08, 0x00, 0x12, 0x4D] => super::probe(|| verify_whatsapp_crypt14(prefix, size)), // WhatsApp .crypt14 KeyData protobuf framing
            b"GGAT" => super::probe(|| verify_bohemia_paa(prefix, size)), // Bohemia Interactive PAA/PAC texture
            b"LAYA" => super::probe(|| verify_layamodel_0501_lm(prefix, size)), // LayaAir 3D binary mesh ("LAYAMODEL:")
            b"MOR2" => super::probe(|| verify_more_outline_doc(prefix, size)), // Symantec MORE 2.0 Mac outline
            b"MOR3" => super::probe(|| verify_more_outline_doc(prefix, size)), // Symantec MORE 3.0 Mac outline
            b"MORE" => super::probe(|| verify_more_outline_doc(prefix, size)), // Living Videotext MORE Mac outline
            b"SV00" => super::probe(|| verify_roland_svq(prefix, size)), // Roland MRC-Pro sequencer song (.svq)
            b"SV01" => super::probe(|| verify_roland_svq(prefix, size)), // Roland MRC-Pro sequencer song (.svq)
            [0xA0, 0x9D, 0x8B, 0x8E] => super::probe(|| verify_advsys_game(prefix, size)), // AdvSys game (encoded "ADVS", `~b - 30`)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @2..4: 2-byte signatures at offset 2 (checked after 4-byte `@2..6` above).
    if let Some(w) = prefix.get(2..4) {
        let hit = match w {
            [0x00, 0x00] => super::probe(|| verify_vms_lib(prefix, size)), // DEC VAX/VMS Librarian reserved u16 == 0 (magic @4)
            [0x00, 0x01] => super::probe(|| verify_borland_turbo_debug_help_tdh(prefix, size)), // Borland Turbo Debugger Help (.tdh)
            [0xAC, 0x1D] => super::probe(|| verify_ac1d_packer(prefix, size)), // Amiga AC1D Packer module (0xAC1D BE)
            // V8 cached bytecode: `0xC0DE0xxx` LE (`@2..4 == [DE, C0]`, version nibble `@1` in 1..=15;
            // common version hashes are also fast-pathed in Tier 1 `magic32`).
            [0xDE, 0xC0] if (0x01..=0x0F).contains(&prefix[1]) => {
                super::probe(|| verify_v8_bytecode(prefix, size))
            }
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @3..7: 4-byte signatures starting at offset 3.
    if let Some(w) = prefix.get(3..7) {
        let hit = match w {
            [0x00, 0x00, 0x00, 0x64] => super::probe(|| verify_bns_dialog_cbf(prefix, size)), // Blade & Soul dialog bundle (.cbf)
            b"\x001Pb" => super::probe(|| verify_eastwest_tchk_key(prefix, size)), // EastWest TChk license key (.key)
            b"_COD" => super::probe(|| verify_award_bios_stf(prefix, size)), // Award BIOS language module (`_XX_CODE.BIN`)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @4..8: 4-byte signatures starting after a 4-byte length/version/checksum word.
    // (Stage 1 already handled `ftyp`/`moov`..`pnot`/`TFL3` and ESE `EF CD AB 89` at `@4..8`.)
    if let Some(w) = prefix.get(4..8) {
        let hit = match w {
            // Zero u32 at @4..8 (e.g., InnoDB page 0 `FIL_PAGE_OFFSET == 0`, Ableton `.asd`, GT `.dux`).
            [0x00, 0x00, 0x00, 0x00] => super::probe(|| verify_ableton_0649_asd(prefix, size))
                .or_else(|| super::probe(|| verify_gt_game_systems_dux(prefix, size)))
                .or_else(|| super::probe(|| verify_innodb_tablespace(prefix, size))),
            [0x00, 0x00, 0x00, 0x01] => super::probe(|| verify_foxpro_form_sct(prefix, size)), // Visual FoxPro 3.0 form/class memo (.sct/.vct)
            [0x00, 0x00, 0x00, 0x1C] => super::probe(|| verify_synth_dream_sdr(prefix, size)), // Synth Dream Amiga module (sdr.*)
            [0x02, 0x01, 0x01, 0x02] => super::probe(|| verify_tls_session_net443(prefix, size)), // OpenSSL DER SSL_SESSION cache (.net_443)
            [0x03, 0x00, 0x00, 0x00] => super::probe(|| verify_qualcomm_mbn40(prefix, size)), // Qualcomm MBN v3 bootloader header
            [0x06, 0x0A, 0x2A, 0x86] => super::probe(|| verify_pkcs15_ky(prefix, size)), // PKCS #15 TokenInfo DER OID prefix
            b"\rHYP" => super::probe(|| verify_hyperplanning_hyp(prefix, size)), // Index Education HYPERPLANNING (.hyp)
            b"\x0FLeg" => super::probe(|| verify_mir_map(prefix, size)), // Legend of Mir 2 tile map (.map)
            [0x10, 0x00, 0x00, 0x00] => super::probe(|| verify_procmon_pmc(prefix, size)), // Sysinternals Process Monitor config (`hdr_len == 16`)
            [0x20, 0x0D, 0x0A, 0x85] => super::probe(|| verify_plato_lesson(prefix, size)), // PLATO / NovaNET TUTOR lesson block
            b"7.0." => super::probe(|| verify_xna_course_crs(prefix, size)), // Windows Phone XNA 7.0 course package
            b"Broa" => super::probe(|| verify_broadcom_cfe(prefix, size)), // Broadcom CFE ImageTag router firmware
            [0x44, 0x00, 0x00, 0x00] => super::probe(|| verify_tunefish2_tf2m(prefix, size)), // Tunefish 2 synth module (`hdr_size == 0x44`)
            b"GEVS" => super::probe(|| verify_unity_enlighten_vis(prefix, size)), // Unity Enlighten visibility data (.vis)
            [0x48, 0xE7, 0x7F, 0xFE] => super::probe(|| verify_scumm_amiga(prefix, size)), // Lucasfilm SCUMM Amiga music (M68k `MOVEM.L`)
            b"RED1" => super::probe(|| verify_redcode_r3d_offset(prefix, size)), // RED Digital Cinema REDCODE RAW v1 (.r3d)
            b"RED2" => super::probe(|| verify_redcode_r3d_offset(prefix, size)), // RED Digital Cinema REDCODE RAW v2 (.r3d)
            b"RO\x0C\x00" => super::probe(|| verify_scumm_room_lfl(prefix, size)), // LucasArts SCUMM v3 room resource (.lfl)
            b"TP-L" => super::probe(|| verify_tplink_firmware(prefix, size)), // TP-LINK router firmware (vendor string)
            [0x56, 0x00, 0x00, 0x00] => super::probe(|| verify_tangopro_lib(prefix, size)), // ACCEL EDA / TangoPRO library V2.0 (.lib)
            b"W\x90u6" => super::probe(|| verify_audible_aa_offset(prefix, size)), // Audible legacy audiobook (`0x57907536` BE)
            b"chan" => super::probe(|| verify_re_engine_chain(prefix, size)), // Capcom RE Engine physics chain (.chain.*)
            b"clst" => super::probe(|| verify_re_engine_mcamlist(prefix, size)), // Capcom RE Engine motion camera list (.mcamlist.*)
            b"eGON" => super::probe(|| verify_allwinner_egon(prefix, size)), // Allwinner `eGON.BT0`/`eGON.BT1` bootloader
            b"jumb" => super::probe(|| verify_jumbf_c2pa(prefix, size)), // ISO 19566-5 JUMBF / C2PA superbox
            b"relo" => super::probe(|| verify_terraria_world(prefix, size)), // Terraria world save ("relogic")
            b"srcs" => super::probe(|| verify_cals_declaration(prefix, size)), // CALS MIL-STD-1840 declaration ("srcsys:")
            b"ver." => super::probe(|| verify_tplink_firmware(prefix, size)), // TP-LINK router firmware (version string)
            b"|Lda" => super::probe(|| verify_beyondtrust_scc_ui(prefix, size)), // BeyondTrust UI RPC cache (`|Ldato::...`)
            [0x85, 0xD2, 0x41, 0x09] => super::probe(|| verify_nba2k_cmg(prefix, size)), // NBA 2K save/mod archive (.cmg)
            [0xAB, 0xAD, 0xCA, 0xFE] => super::probe(|| verify_lcio_sio(prefix, size)), // Linear Collider I/O SIO record (`0xABADCAFE` BE)
            [0xCA, 0x23, 0x3C, 0x3C] => super::probe(|| verify_allegro_fasl(prefix, size)), // Allegro Common Lisp FASL bytecode
            [0xED, 0xBE, 0xAD, 0xDE] => super::probe(|| verify_nsis_firstheader(prefix, size)), // NSIS carved firstheader (`0xDEADBEED` LE)
            [0xEF, 0xBE, 0xAD, 0xDE] => super::probe(|| verify_nsis_firstheader(prefix, size)), // NSIS carved firstheader (`0xDEADBEEF` LE)
            [0xFF, 0xFF, 0x00, 0x00] => super::probe(|| verify_topsolid_rcf(prefix, size)), // TopSolid MFC `CArchive` (`wNewClassTag + schema 0`)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @4..6: 2-byte signatures at offset 4 (checked after 4-byte `@4..8` above).
    if let Some(w) = prefix.get(4..6) {
        let hit = match w {
            [0x00, 0x00] => super::probe(|| verify_alto_run(prefix, size)), // Xerox Alto BCPL `.run` (`w2 == 0` + 20 zero bytes)
            b"RM" => super::probe(|| verify_valve_hammer_rmf_with_stage(prefix, size)), // Valve Hammer Rich Map Format (`.rmf`)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @5..9: 4-byte signatures starting at offset 5.
    if let Some(w) = prefix.get(5..9) {
        let hit = match w {
            b"    " => super::probe(|| verify_aor_dv10_fw(prefix, size)), // AOR AR-DV10 SDR firmware
            b"*hi\xB3" => super::probe(|| verify_pdp10_failsafe(prefix, size)), // DEC PDP-10 FAILSAFE tape (SIXBIT "*FAILSAFE")
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @6..10: 4-byte signatures starting at offset 6.
    if let Some(w) = prefix.get(6..10) {
        let hit = match w {
            [0x00, 0x00, 0x00, 0x10] => super::probe(|| verify_acta_outline_doc(prefix, size)), // Symmetry Acta Mac outline
            [0x01, 0x00, 0x00, 0x00] => super::probe(|| verify_twin_track_dmo(prefix, size)), // TwinTeam Twin TrackPlayer OPL2 (.dmo)
            b"TCSO" => super::probe(|| verify_flash_sol(prefix, size)), // Adobe Flash Local Shared Object (.sol)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @7..11: 4-byte signatures starting at offset 7.
    if let Some(w) = prefix.get(7..11) {
        let hit = match w {
            b"SONG" => super::probe(|| verify_zx_spectrum_stc(prefix, size)), // ZX Spectrum Sound Tracker Compiled ("SONG BY ...")
            b"SOUN" => super::probe(|| verify_zx_ay_st13(prefix, size)), // ZX Spectrum Sound Tracker 1.3 AY (.st13)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @8..12: 4-byte signatures starting at offset 8 (after Stage 1's `DUCK` at `@8..12`).
    if let Some(w) = prefix.get(8..12) {
        let hit = match w {
            [0x00, 0x00, 0x00, 0x00] => super::probe(|| verify_acronis_msg_catalog(prefix, size))
                .or_else(|| super::probe(|| verify_sunos_toc(prefix, size))), // Acronis UTF-16LE catalog or SunOS XDR TOC (`version == 0`)
            [0x00, 0x00, 0x00, 0x01] => super::probe(|| verify_supf_510_index(prefix, size)), // Apple FairPlay `.supf` key container
            [0x00, 0x00, 0x00, 0x7C] => super::probe(|| verify_sony_pmp(prefix, size)), // Sony DSC-F1 PMP image (`jpeg_offset == 124`)
            [0x01, 0x00, 0x00, 0x00] => super::probe(|| verify_kontakt_hsin_nki(prefix, size)), // Native Instruments Kontakt monolith (.nki)
            [0x19, 0x04, 0x00, 0x10] => super::probe(|| verify_symbian_sis_offset(prefix, size)), // Symbian OS SIS (`UID3 == 0x10000419` LE)
            b".FIT" => super::probe(|| verify_garmin_fit(prefix, size)), // Garmin FIT activity/settings file
            [0x56, 0x19, 0xB5, 0x27] => super::probe(|| verify_amlogic_upgrade(prefix, size)), // Amlogic USB Burning Tool upgrade package
            b"YET " => super::probe(|| verify_laxity_pt(prefix, size)), // Laxity Amiga Powertracker (.pt)
            b"bvx-" => super::probe(|| verify_lzfse_length_prefixed(prefix, size)), // Length-prefixed Apple LZFSE (uncompressed block)
            b"bvx1" => super::probe(|| verify_lzfse_length_prefixed(prefix, size)), // Length-prefixed Apple LZFSE (v1 block)
            b"bvx2" => super::probe(|| verify_lzfse_length_prefixed(prefix, size)), // Length-prefixed Apple LZFSE (v2 block)
            b"bvxn" => super::probe(|| verify_lzfse_length_prefixed(prefix, size)), // Length-prefixed Apple LZFSE (LZVN block)
            b"yp|v" => super::probe(|| verify_leica_dmr_firmware(prefix, size)), // Leica Digital Modul-R XOR-0x35 firmware
            [0x8E, 0xFC, 0x24, 0x48] => super::probe(|| verify_sims4_thumbnail(prefix, size)), // EA The Sims 4 tray thumbnail
            [0xFF, 0x00, 0xFF, 0x00] => super::probe(|| verify_sound_images_sig(prefix, size)), // Sound Images Gen 2 AdLib module (.sig)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @8..10: 2-byte signatures at offset 8 (checked after 4-byte `@8..12` above).
    if let Some(w) = prefix.get(8..10) {
        let hit = match w {
            [0x00, 0x00] => super::probe(|| verify_cdfm_670(prefix, size)), // Renaissance CDFM / Composer 670 packed module
            [0x00, 0x02] => super::probe(|| verify_ctos_volume(prefix, size)), // Convergent CTOS/BTOS disk (`sector_size == 512` LE)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @12..16: 4-byte signatures at offset 12 (after Stage 1's Berkeley DB at `@12..16`).
    if let Some(w) = prefix.get(12..16) {
        let hit = match w {
            [0x00, 0x00, 0x00, 0x00] => super::probe(|| verify_digital_sonix_dsc(prefix, size)), // Digital Sonix & Chrome Amiga module (.dsc)
            [0x00, 0x00, 0x05, 0x0F] => {
                super::probe(|| verify_musical_enlightenment_tme(prefix, size)) // Musical Enlightenment Amiga module (.tme)
            }
            b"LATI" => super::probe(|| verify_kyocera_raw(prefix, size)), // Kyocera Contax N Digital RAW (byte-reversed "N DIGITAL")
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @16..20: 4-byte signatures at offset 16 (after Stage 1's LMDB / Realm at `@16..20`).
    if let Some(w) = prefix.get(16..20) {
        let hit = match w {
            b"$PS1" => super::probe(|| verify_amd_psp_firmware(prefix, size)), // AMD Platform Security Processor signed firmware
            [0x32, 0x00, 0x00, 0x00] => super::probe(|| verify_durager_uv_real(prefix, size)), // Durager UV lamp calibration record (.real)
            b"ALPS" => super::probe(|| verify_mediatek_bt_firmware(prefix, size)), // MediaTek / ALPS Bluetooth combo firmware
            b"SYSB" => super::probe(|| verify_apollo_sysboot(prefix, size)), // Apollo Domain/OS SYSBOOT tape ("SYSBOOT REV ")
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @36..40: 4-byte signatures at offset 36 (after Stage 1's ICC `"acsp"` at `@36..40`).
    if let Some(w) = prefix.get(36..40) {
        let hit = match w {
            [0x00, 0x00, 0x00, 0x00] => super::probe(|| verify_sound_control_sc(prefix, size)), // Holger Gehrmann Sound Control Amiga (.sc)
            b"!SOP" => super::probe(|| verify_ione_soprol(prefix, size)), // Ione SOPROL Amiga module (.spl)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @44..48: Tracker module 4-byte signatures at offset 44.
    if let Some(w) = prefix.get(44..48) {
        let hit = match w {
            b"PTMF" => super::probe(|| verify_polytracker_module(prefix, size)), // PolyTracker module (.ptm)
            b"SCRM" => super::probe(|| verify_scream_tracker_3(prefix, size)), // Scream Tracker 3 module (.s3m)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // @1080..1082: Offset 1080 (0x438) signatures — Ext2/3/4 superblock magic (`0xEF53` LE,
    // when the 1 KiB boot block is non-zero) and Images Music System Amiga module.
    if let Some(w) = prefix.get(1080..1082) {
        let hit = match w {
            [0x00, 0x00] => super::probe(|| verify_images_music_system(prefix, size)), // Images Music System / Unicorn Tracker (.ims)
            [0x53, 0xEF] => super::probe(|| verify_ext234_offset(prefix, size)), // Linux Ext2/Ext3/Ext4 (`s_magic == 0xEF53` LE)
            _ => None,
        };
        if hit.is_some() {
            return hit;
        }
    }

    // =========================================================================
    // Stage 3: Unique-offset inline-guarded probes.
    // Each format below has a fixed signature at an offset shared by no other
    // format (so a `match` table would be a single arm), ordered from exact
    // disk-image size checks and lowest offset (`@1..5`) to highest (`@1464`).
    // =========================================================================

    // Xerox Alto Diablo 31 / Diablo 44 disk pack: exact 4,872 or 9,744 × 534-byte sector geometry.
    if (size == 2_601_648 || size == 5_203_296)
        && let Some(lbl) = super::probe(|| verify_xerox_alto_disk(prefix, size))
    {
        return Some(lbl);
    }
    // Neo Geo / FBNeo ABKCPUR battery SRAM: constant 4-byte signature at @1..5.
    if prefix.get(1..5) == Some([0x09, 0x00, 0x00, 0x32].as_slice())
        && let Some(lbl) = super::probe(|| verify_neogeo_abkcpur_srm(prefix, size))
    {
        return Some(lbl);
    }
    // Quarterdeck DESQview Help (.hlp): "/FT" first topic marker starting at @6..8.
    if prefix.get(6..8) == Some(b"/F".as_slice())
        && let Some(lbl) = super::probe(|| verify_desqview_hlp(prefix, size))
    {
        return Some(lbl);
    }
    // Atari ST Quartet 4-voice module (.4v/.qpa): time-signature byte at @7,
    // 9 reserved zero bytes at @8..17, and zero event high byte at @28.
    if prefix.len() >= 40
        && matches!(prefix[7], 0x02 | 0x04 | 0x08 | 0x10)
        && prefix[8..17] == [0u8; 9]
        && prefix[28] == 0
        && let Some(lbl) = super::probe(|| verify_quartet_st_module(prefix, size))
    {
        return Some(lbl);
    }
    // Tomy Tracker Amiga module (.sg): reserved zero u16 at @10..12.
    if prefix.get(10..12) == Some([0x00, 0x00].as_slice())
        && let Some(lbl) = super::probe(|| verify_tomy_tracker_sg(prefix, size))
    {
        return Some(lbl);
    }
    // Advantec SoundFactory Amiga synth module (.psf): zero u32 at @10..14.
    if prefix.get(10..14) == Some([0x00, 0x00, 0x00, 0x00].as_slice())
        && let Some(lbl) = super::probe(|| verify_soundfactory_psf(prefix, size))
    {
        return Some(lbl);
    }
    // DEC VAX/VMS Executable/Shareable Image: ASCII major version "02" at @12..14.
    if prefix.get(12..14) == Some(b"02".as_slice())
        && let Some(lbl) = super::probe(|| verify_vms_exe(prefix, size))
    {
        return Some(lbl);
    }
    // Agile Pumatracker Amiga module (.puma): reserved zero u16 at @18..20.
    if prefix.get(18..20) == Some([0x00, 0x00].as_slice())
        && let Some(lbl) = super::probe(|| verify_pumatracker(prefix, size))
    {
        return Some(lbl);
    }
    // BSD / Unix UFS `dump(8)` archive: `c_magic` at @24..28 (OFS/NFS/UFS2/etc., LE or BE).
    if prefix.get(24..28).is_some_and(|s| {
        matches!(
            s,
            [0x00, 0x00, 0x60, 0x01]
                | [0x00, 0x00, 0x60, 0x02]
                | [0x00, 0x00, 0xEA, 0x6B]
                | [0x00, 0x00, 0xEA, 0x6C]
                | [0x00, 0x19, 0x54, 0x01]
                | [0x01, 0x54, 0x19, 0x00]
                | [0x01, 0x60, 0x00, 0x00]
                | [0x02, 0x60, 0x00, 0x00]
                | [0x6B, 0xEA, 0x00, 0x00]
                | [0x6C, 0xEA, 0x00, 0x00]
        )
    }) && let Some(lbl) = super::probe(|| verify_bsd_ufs_dump(prefix, size))
    {
        return Some(lbl);
    }
    // Decompressed NSIS binary script header: `strings.num == 0` u32 at @32..36.
    if prefix.get(32..36) == Some([0x00, 0x00, 0x00, 0x00].as_slice())
        && let Some(lbl) = super::probe(|| verify_nsis_script_header(prefix, size))
    {
        return Some(lbl);
    }
    // Acorn RISC OS Font `IntMetrics`: fixed header flag/offset word `0x00000010` LE at @40..44.
    if prefix.get(40..44) == Some([0x10, 0x00, 0x00, 0x00].as_slice())
        && let Some(lbl) = super::probe(|| verify_riscos_intmetrics(prefix, size))
    {
        return Some(lbl);
    }
    // Ken Silverman AdLib music (.ksm): 5-voice OPL2 percussion channel flags `[1; 5]` at @43..48.
    if prefix.get(43..47) == Some([0x01, 0x01, 0x01, 0x01].as_slice())
        && let Some(lbl) = super::probe(|| verify_kens_adlib_ksm(prefix, size))
    {
        return Some(lbl);
    }
    // Amiga SidMon II module: "SIDMON II - THE MIDI VERSION" banner starting at @58..62.
    if prefix.get(58..62) == Some(b"SIDM".as_slice())
        && let Some(lbl) = super::probe(|| verify_sidmon2_module(prefix, size))
    {
        return Some(lbl);
    }
    // Imago Orpheus tracker module (.imf): "IM10" signature at @60..64.
    if prefix.get(60..64) == Some(b"IM10".as_slice())
        && let Some(lbl) = super::probe(|| verify_imago_imf(prefix, size))
    {
        return Some(lbl);
    }
    // Actionamics Amiga Sound Tool (.ast): "ACTIONAMICS SOUND TOOL" starting at @62..66.
    if prefix.get(62..66) == Some(b"ACTI".as_slice())
        && let Some(lbl) = super::probe(|| verify_actionamics_ast(prefix, size))
    {
        return Some(lbl);
    }
    // MacBinary-wrapped Apple DiskCopy 6.0 NDIF: MacBinary type "dimg"/"rohd" at @65..69.
    if prefix.get(65..69).is_some_and(|s| matches!(s, b"dimg" | b"rohd"))
        && let Some(lbl) = super::probe(|| verify_apple_ndif_offset(prefix, size))
    {
        return Some(lbl);
    }
    // MRC / CCP4 3D electron microscopy density map: "MAP\0" or "MAP " at @208..212.
    if prefix.get(208..212).is_some_and(|s| matches!(s, b"MAP\x00" | b"MAP "))
        && let Some(lbl) = super::probe(|| verify_mrc_density_map(prefix, size))
    {
        return Some(lbl);
    }
    // IBM i / OS/400 Save File (SAVF): block 1 descriptor word `0x00000001` BE at @512..516.
    if prefix.get(512..516) == Some([0x00, 0x00, 0x00, 0x01].as_slice())
        && let Some(lbl) = super::probe(|| verify_ibm_savf(prefix, size))
    {
        return Some(lbl);
    }
    // Kris Hatlelid Chiptracker Amiga module: "KRIS" signature at @952..956.
    if prefix.get(952..956) == Some(b"KRIS".as_slice())
        && let Some(lbl) = super::probe(|| verify_chiptracker_kris(prefix, size))
    {
        return Some(lbl);
    }
    // Mahoney & Kaktus His Master's Noise Amiga module: "FEST" or "M&K!" at @1080..1084.
    if prefix.get(1080..1084).is_some_and(|s| matches!(s, b"FEST" | b"M&K!"))
        && let Some(lbl) = super::probe(|| verify_his_masters_noise(prefix, size))
    {
        return Some(lbl);
    }
    // Soundtracker 2.6 / Ice Tracker Amiga module: "IT10" or "MTN\0" at @1464..1468.
    if prefix.get(1464..1468).is_some_and(|s| matches!(s, b"IT10" | b"MTN\x00"))
        && let Some(lbl) = super::probe(|| verify_soundtracker_26(prefix, size))
    {
        return Some(lbl);
    }

    // =========================================================================
    // Stage 4: Structural, multi-offset, and numeric-header probes.
    // Runs last because these formats lack a single fixed constant signature
    // (they use multiple variant offsets, numeric field ranges, block-size
    // multiples, or `memchr` scans). Grouped into length buckets (`>= 512`,
    // `>= 64`, then smaller) so short files skip entire sections in one branch.
    // =========================================================================

    // 4a. Deep-header structural probes (`prefix.len() >= 512`).
    if prefix.len() >= 512 {
        // Xerox Alto bitmap font (.al): even 16-bit word length >= 516, 256 self-relative glyph ptrs.
        if size >= 516
            && size.is_multiple_of(2)
            && prefix.len() >= 516
            && let Some(lbl) = super::probe(|| verify_alto_al(prefix, size))
        {
            return Some(lbl);
        }
        // DEC PDP-11 RT-11 disk image: Block 1 Home Block at @0x200..0x400 ("DECRT11A    ").
        if size >= 1024
            && prefix.len() >= 1024
            && let Some(lbl) = super::probe(|| verify_dec_rt11(prefix, size))
        {
            return Some(lbl);
        }
        // Fujitsu FM Towns Euphony song (.eup): 2054-byte header + FM/PCM channel tables.
        if size >= 2060
            && prefix.len() >= 2054
            && let Some(lbl) = super::probe(|| verify_fm_towns_euphony(prefix, size))
        {
            return Some(lbl);
        }
        // HSC-Tracker OPL2 module (.hsc): 1587-byte header + N × 1152-byte pattern geometry.
        if prefix.len() >= 1587
            && let Some(lbl) = super::probe(|| verify_hsc_tracker(prefix, size))
        {
            return Some(lbl);
        }
        // IBM 3740 8-inch floppy image: 256,256 B exact size + Track 0 EBCDIC ERMAP/VOL1/HDR1 labels.
        if prefix.len() >= 1028
            && let Some(lbl) = super::probe(|| verify_ibm_3740(prefix, size))
        {
            return Some(lbl);
        }
        // MultiMate document: date separators '/' at @406 and @409 (and "D$     " at @420/@492).
        if size >= 512
            && prefix.len() >= 512
            && prefix[406] == b'/'
            && prefix[409] == b'/'
            && let Some(lbl) = super::probe(|| verify_multimate_offset(prefix, size))
        {
            return Some(lbl);
        }
        // DEC PDP-8 OS/8 Save Core Image (.sv): 512-byte Core Control Block of 12-bit LE words.
        if size >= 1024
            && prefix.len() >= 512
            && let Some(lbl) = super::probe(|| verify_pdp8_sv(prefix, size))
        {
            return Some(lbl);
        }
        // DEC TOPS-10 BACKUP 36-bit tape stream: 544 × 5-byte = 2,720-byte core-dump records.
        if size >= 2720
            && prefix.len() >= 2720
            && let Some(lbl) = super::probe(|| verify_tops10_backup(prefix, size))
        {
            return Some(lbl);
        }
        // DEC TOPS-20 DUMPER 36-bit tape saveset: 518 × 5-byte = 2,590-byte core-dump records.
        if size >= 2590
            && prefix.len() >= 2590
            && let Some(lbl) = super::probe(|| verify_tops20_dumper(prefix, size))
        {
            return Some(lbl);
        }
        // UCSD Pascal p-Machine codefile (.code): 512-byte aligned, Block 0 16-slot segment dict.
        if size >= 1024
            && size.is_multiple_of(512)
            && prefix.len() >= 512
            && let Some(lbl) = super::probe(|| verify_ucsd_code(prefix, size))
        {
            return Some(lbl);
        }
        // UCSD Pascal p-System disk volume (.vol/.dsk): Block 2 directory header at @1024.
        if size >= 2048
            && prefix.len() >= 1076
            && let Some(lbl) = super::probe(|| verify_ucsd_vol(prefix, size))
        {
            return Some(lbl);
        }
        // Tektronix 4404 / TSC UniFLEX disk image: big-endian SIR at Block 1 (@512..566).
        if size >= 1024
            && prefix.len() >= 566
            && let Some(lbl) = super::probe(|| verify_uniflex_img(prefix, size))
        {
            return Some(lbl);
        }
    }

    // 4b. Medium-header structural probes (`prefix.len() >= 64`).
    if prefix.len() >= 64 {
        // Alpha Networks WRGG router firmware: magic `0x20080321` (LE or BE) at @32..36.
        if size >= 116
            && prefix
                .get(32..36)
                .is_some_and(|s| s == [0x21, 0x03, 0x08, 0x20] || s == [0x20, 0x08, 0x03, 0x21])
            && let Some(lbl) = super::probe(|| verify_alpha_wrgg(prefix, size))
        {
            return Some(lbl);
        }
        // Xerox Alto `FONTS.WIDTHS` table: even word-aligned IX directory record chain.
        if size >= 64
            && size.is_multiple_of(2)
            && let Some(lbl) = super::probe(|| verify_alto_widths(prefix, size))
        {
            return Some(lbl);
        }
        // Borland Turbo C/Assembler Help (.tch/.tah): DOS EOF `0x1A` inside `"HELP FILE.\0\x1A$*$*"` in @0..64.
        if size >= 64
            && memchr::memchr(0x1A, &prefix[..64]).is_some()
            && let Some(lbl) = super::probe(|| verify_borland_turbo_help_tch(prefix, size))
        {
            return Some(lbl);
        }
        // CERNLIB ZEBRA RZ direct-access file: `NREC1 == 1` (LE or BE) at @0..4 + 16 control words.
        if size >= 1024
            && matches!(&prefix[0..4], [0x01, 0x00, 0x00, 0x00] | [0x00, 0x00, 0x00, 0x01])
            && let Some(lbl) = super::probe(|| verify_cern_zebra_rz(prefix, size))
        {
            return Some(lbl);
        }
        // Fastgraph/Image resource library (.fgi): array of 24-byte directory entries starting at @0.
        if size >= 96
            && prefix.len() >= 72
            && let Some(lbl) = super::probe(|| verify_fastgraph_fgi(prefix, size))
        {
            return Some(lbl);
        }
        // Game Music Creator Amiga module (.gmc): song order count @240 + 100 pattern offsets @244..444.
        if prefix.len() >= 444
            && let Some(lbl) = super::probe(|| verify_game_music_creator(prefix, size))
        {
            return Some(lbl);
        }
        // IBM z/OS IEBCOPY unloaded PDS: 3-byte `COPYR1` ID `CA 6D 0F` at @1 (raw), @5 (RDW), or @9 (BDW+RDW).
        if size >= 340
            && prefix.len() >= 68
            && (prefix[1..4] == [0xCA, 0x6D, 0x0F]
                || prefix[5..8] == [0xCA, 0x6D, 0x0F]
                || prefix[9..12] == [0xCA, 0x6D, 0x0F])
            && let Some(lbl) = super::probe(|| verify_iebcopy_unloaded_pds(prefix, size))
        {
            return Some(lbl);
        }
        // Decompressed Inno Setup `TSetupHeader` stream: first u32 LE string length in `2..=512` (`@1 <= 2`, `@2..4 == 0`).
        if size >= 256
            && prefix.len() >= 128
            && prefix[2] == 0
            && prefix[3] == 0
            && prefix[1] <= 2
            && let Some(lbl) = super::probe(|| verify_inno_setup_header_stream(prefix, size))
        {
            return Some(lbl);
        }
        // Intel ME Flash Partition Table: "$FPT" at @0 (also in Tier 1 `magic32`) or @16 (after 16B ROM bypass).
        if size >= 64
            && (&prefix[0..4] == b"$FPT" || &prefix[16..20] == b"$FPT")
            && let Some(lbl) = super::probe(|| verify_intel_me_fpt(prefix, size))
        {
            return Some(lbl);
        }
        // MSX / DOS OPLLDRV FM chiptune (.opx): 126-byte DOS `TYPE` title block + FM pointer table at @126..144.
        if size >= 160
            && prefix.len() >= 144
            && let Some(lbl) = super::probe(|| verify_msx_oplldrv_offset(prefix, size))
        {
            return Some(lbl);
        }
        // Pierre Adane Packer Amiga module (.pap): 4 even BE16 channel offsets at @0..8 (all low bits == 0).
        if (64..=65_536).contains(&size)
            && prefix.len() >= 72
            && ((prefix[1] | prefix[3] | prefix[5] | prefix[7]) & 1 == 0)
            && let Some(lbl) = super::probe(|| verify_pierre_adane_pap(prefix, size))
        {
            return Some(lbl);
        }
        // Atari ST Quartet sample set (.set): replay rate `4..=32` at @0 and voice/sample count `1..=16` at @1.
        if size >= 256
            && prefix.len() >= 222
            && (0x04..=0x20).contains(&prefix[0])
            && (0x01..=0x10).contains(&prefix[1])
            && let Some(lbl) = super::probe(|| verify_quartet_set(prefix, size))
        {
            return Some(lbl);
        }
        // Unity Engine `SerializedFile` (.assets): BE32 format version `9..=32` at @8..12,
        // endianness flag `<= 1` at @16, and 3 reserved zero bytes at @17..20.
        if size >= 64
            && prefix[8] == 0
            && prefix[9] == 0
            && prefix[10] == 0
            && (9..=32).contains(&prefix[11])
            && prefix[16] <= 1
            && prefix[17..20] == [0, 0, 0]
            && let Some(lbl) = super::probe(|| verify_unity_serialized_assets(prefix, size))
        {
            return Some(lbl);
        }
    }

    // 4c. Small-header and multi-offset structural probes (each guarded by cheap
    // length, alignment, or multi-byte field pre-checks).

    // AMD Radeon / AMDGPU microcode: IP Discovery (`07 14 21 28` + `"IPDS"` @60),
    // or `common_firmware_header` with LE16 major version `1..=5` @8..10 and minor `0..=10` @10..12.
    if ((size >= 64
        && prefix.len() >= 64
        && prefix[0..4] == [0x07, 0x14, 0x21, 0x28]
        && &prefix[60..64] == b"IPDS")
        || (size >= 256
            && prefix.len() >= 256
            && prefix[9] == 0
            && (1..=5).contains(&prefix[8])
            && prefix[11] == 0
            && prefix[10] <= 10))
        && let Some(lbl) = super::probe(|| verify_amdgpu_microcode(prefix, size))
    {
        return Some(lbl);
    }
    // Assetto Corsa `data.acd` archive: optional `-1111` (`A9 FB FF FF`) v2 marker or LE32 filename length `5..=64`.
    if size >= 48
        && prefix.len() >= 48
        && (prefix[0..4] == [0xA9, 0xFB, 0xFF, 0xFF]
            || (prefix[1] == 0
                && prefix[2] == 0
                && prefix[3] == 0
                && (5..=64).contains(&prefix[0])))
        && let Some(lbl) = super::probe(|| verify_assettocorsa_acd(prefix, size))
    {
        return Some(lbl);
    }
    // Chromium Simple Cache: entry/index magic (`30 5C 72 A7...`) or `"oy retne"` at @0 or @8.
    if (prefix.get(0..4) == Some([0x30, 0x5C, 0x72, 0xA7].as_slice())
        || prefix.get(8..12) == Some(b"oy r".as_slice())
        || prefix.get(0..4) == Some(b"oy r".as_slice()))
        && let Some(lbl) = super::probe(|| verify_chromium_simple_cache(prefix, size))
    {
        return Some(lbl);
    }
    // DESY FPACK / BOS event stream: `0x04030201` endianness sentinel at @8..12 (BE or LE).
    if size >= 68
        && prefix.len() >= 28
        && matches!(&prefix[8..12], [0x04, 0x03, 0x02, 0x01] | [0x01, 0x02, 0x03, 0x04])
        && let Some(lbl) = super::probe(|| verify_desy_fpack_bos(prefix, size))
    {
        return Some(lbl);
    }
    // Data General Nova relocatable binary (.rb): optional zero paper-tape leader or LE16 block type `1..=9`.
    if size >= 36
        && prefix.len() >= 2
        && (prefix[0] == 0 || (prefix[1] == 0 && (1..=9).contains(&prefix[0])))
        && let Some(lbl) = super::probe(|| verify_dg_nova_rb(prefix, size))
    {
        return Some(lbl);
    }
    // `fio` verify state (.state): LE u64 format version V3..=V7 at @0..8.
    if size >= 64
        && prefix.len() >= 32
        && (3..=7).contains(&prefix[0])
        && prefix[1] == 0
        && prefix[2] == 0
        && prefix[3] == 0
        && let Some(lbl) = super::probe(|| verify_fio_verify_state(prefix, size))
    {
        return Some(lbl);
    }
    // JP Software 4DOS / 4OS2 / 4NT help (.hlp): v4 `"4DH"/"4OH"/"4NH"` at @0 or v5 `0x08` + tag at @1..4.
    if size >= 64
        && prefix.len() >= 28
        && (matches!(&prefix[0..3], b"4DH" | b"4OH" | b"4NH")
            || (prefix[0] == 0x08 && matches!(&prefix[1..4], b"4DH" | b"4OH" | b"4NH")))
        && let Some(lbl) = super::probe(|| verify_fourdos_help(prefix, size))
    {
        return Some(lbl);
    }
    // Ashton-Tate FullWrite Professional Mac document: v1 (`00 00 00 5C`) or v2 (`00 00 00 03`) header.
    if size >= 64
        && prefix.len() >= 36
        && prefix[0] == 0
        && prefix[1] == 0
        && prefix[2] == 0
        && matches!(prefix[3], 0x5C | 0x03)
        && let Some(lbl) = super::probe(|| verify_fullwrite_doc(prefix, size))
    {
        return Some(lbl);
    }
    // Herad / Cryo Interactive AdLib music (.agd/.sdb/.sqx): LE16 format/channel tag `0x0032` or `0x0052` at @10..12.
    if size >= 64
        && prefix.len() >= 48
        && prefix[11] == 0
        && (prefix[10] == 0x32 || prefix[10] == 0x52)
        && let Some(lbl) = super::probe(|| verify_herad_adlib(prefix, size))
    {
        return Some(lbl);
    }
    // HP 2100 absolute binary paper tape: leading zero leader or `count` byte followed by `@1 == 0`.
    if size >= 24
        && prefix.len() >= 2
        && (prefix[0] == 0 || prefix[1] == 0)
        && let Some(lbl) = super::probe(|| verify_hp2100_abs(prefix, size))
    {
        return Some(lbl);
    }
    // HP 2100 relocatable binary paper tape: leading zero leader or record word count `>= 4` with `@1 == 0`.
    if size >= 24
        && prefix.len() >= 2
        && (prefix[0] == 0 || (prefix[1] == 0 && prefix[0] >= 4))
        && let Some(lbl) = super::probe(|| verify_hp2100_rel(prefix, size))
    {
        return Some(lbl);
    }
    // IBM z/OS AMATERSE compatibility mode (`RECFM=FB, LRECL=1024`; native 4-byte magics are in Tier 1 `magic32`):
    // 1 KiB size multiple, mode `0x02|0x05`, flags `<= 1`, PDS/flat marker at @2..4, reserved zero word at @10..12.
    if size >= 1024
        && size.is_multiple_of(1024)
        && prefix.len() >= 16
        && matches!(prefix[0], 0x02 | 0x05)
        && prefix[1] <= 1
        && matches!(&prefix[2..4], [0x00, 0x00] | [0x89, 0x69])
        && prefix[10..12] == [0x00, 0x00]
        && let Some(lbl) = super::probe(|| verify_ibm_terse(prefix, size))
    {
        return Some(lbl);
    }
    // Infogrames / Rob Hubbard II Amiga sample bank (.ins): small BE32 table offset at @4..8 + matching sample ptrs @8..16.
    if size >= 48
        && prefix.len() >= 24
        && prefix[4] == 0
        && prefix[5] == 0
        && prefix[6] <= 4
        && prefix[8..12] == prefix[12..16]
        && let Some(lbl) = super::probe(|| verify_infogrames_ins(prefix, size))
    {
        return Some(lbl);
    }
    // Intel SPI Flash Descriptor (IFD): `0x0FF0A55A` LE (`FLVALSIG`) at @16 after 16-byte SPI preamble.
    // (Unprefixed IFD at @0 is in Tier 1 `magic32`.)
    if size >= 64
        && prefix.get(16..20) == Some([0x5A, 0xA5, 0xF0, 0x0F].as_slice())
        && let Some(lbl) = super::probe(|| verify_intel_flash_descriptor_at(prefix, size, 16))
    {
        return Some(lbl);
    }
    // Intel PRO/Wireless & Wi-Fi 6/7 microcode (.ucode/.pnvm): TLV/Windows/PNVM prefix or ARC/Tensilica runtime @24/@28.
    if size >= 28
        && prefix.len() >= 32
        && (matches!(&prefix[0..4], [0, 0, 0, 0] | b"*WES" | [0x40, 0, 0, 0] | [0x3E, 0, 0, 0])
            || prefix[24..28] == [0x20, 0x20, 0x80, 0x0F]
            || prefix[28..32] == [0x20, 0x20, 0x80, 0x0F])
        && let Some(lbl) = super::probe(|| verify_iwlwifi_ucode(prefix, size))
    {
        return Some(lbl);
    }
    // Oracle Java Web Start cache index (.idx): two boolean flags (`<= 1`) at @0..2 + BE32 version `602..=606` (`@2..4 == 0`).
    if size >= 32
        && prefix.len() >= 32
        && prefix[0] <= 1
        && prefix[1] <= 1
        && prefix[2] == 0
        && prefix[3] == 0
        && let Some(lbl) = super::probe(|| verify_java_webstart_idx(prefix, size))
    {
        return Some(lbl);
    }
    // Jeroen Tel Amiga module (.jt): M68k `ANDI.B #1,...` (`02 39 00 01 00`) or CIA LED off `BSET #1,$BFE001`.
    if size >= 64
        && prefix.len() >= 24
        && (prefix.starts_with(&[0x02, 0x39, 0x00, 0x01, 0x00])
            || prefix.starts_with(&[0x08, 0xF9, 0x00, 0x01, 0x00, 0xBF, 0xE0, 0x01]))
        && let Some(lbl) = super::probe(|| verify_jeroen_tel_jt_offset(prefix, size))
    {
        return Some(lbl);
    }
    // JetBrains `PersistentHashMapValueStorage` (.values.at): `"/\0Header Record..."` at @0 or @4.
    if size >= 48
        && prefix.len() >= 48
        && (&prefix[0..4] == b"/\0He" || &prefix[4..8] == b"/\0He")
        && let Some(lbl) = super::probe(|| verify_jetbrains_phm_at(prefix, size))
    {
        return Some(lbl);
    }
    // ARM Mbed `littlefs` flash filesystem: `"littlefs"` tag at @8..16 (v2) or @40..48 (v1).
    if size >= 256
        && prefix.len() >= 56
        && (&prefix[8..16] == b"littlefs" || &prefix[40..48] == b"littlefs")
        && let Some(lbl) = super::probe(|| verify_littlefs(prefix, size))
    {
        return Some(lbl);
    }
    // Marvell OCTEON CPT crypto microcode: `"CPT01"` at @0 or `"OCPT-0"` at @4.
    if size >= 64
        && prefix.len() >= 48
        && (&prefix[0..4] == b"CPT0" || &prefix[4..8] == b"OCPT")
        && let Some(lbl) = super::probe(|| verify_marvell_cpt_microcode(prefix, size))
    {
        return Some(lbl);
    }
    // MegaZeux world (.mzx): protection mode `0..=3` at @25 + `'M'` magic start at @26 (unprotected) or @41/@42 (protected).
    if size >= 256
        && prefix.len() >= 49
        && prefix[25] <= 3
        && (prefix[26] == b'M' || prefix[41] == b'M' || prefix[42] == b'M')
        && let Some(lbl) = super::probe(|| verify_megazeux_world(prefix, size))
    {
        return Some(lbl);
    }
    // MSX MoonBlaster module (.mbm): `"-pc1-"` at @5, legacy voice tag `11 11 14` at @3/@4, or channel mask at @328/@329.
    if size >= 360
        && prefix.len() >= 16
        && (&prefix[5..10] == b"-pc1-"
            || prefix[3..6] == [0x11, 0x11, 0x14]
            || prefix[4..7] == [0x11, 0x11, 0x14]
            || (prefix.len() >= 346 && (prefix[328] == 0x10 || prefix[329] == 0x10)))
        && let Some(lbl) = super::probe(|| verify_moonblaster_module(prefix, size))
    {
        return Some(lbl);
    }
    // Nintendo Wii IMET opening banner (.bnr): `"IMET"` at @0x40 (opening.bnr) or @0x80 (WAD 00000000.app).
    if (prefix.get(0x40..0x44) == Some(b"IMET".as_slice())
        || prefix.get(0x80..0x84) == Some(b"IMET".as_slice()))
        && let Some(lbl) = super::probe(|| verify_nintendo_wii_opening_banner(prefix, size))
    {
        return Some(lbl);
    }
    // Nokia BB5 PPM firmware pack: two LE32 block lengths `<= 0x1000` at @0..4 and @4..8 (upper u16s == 0).
    if size >= 64
        && prefix.len() >= 48
        && prefix[2] == 0
        && prefix[3] == 0
        && prefix[6] == 0
        && prefix[7] == 0
        && let Some(lbl) = super::probe(|| verify_nokia_bb5_ppmg(prefix, size))
    {
        return Some(lbl);
    }
    // Microsoft Word/Office owner lock (`~$*.doc`/`~$*.docx`): ANSI length `n` in `1..=52` at @0 matching LE16 UTF-16 length at @54..56.
    if size >= 72
        && prefix.len() >= 72
        && (1..=52).contains(&prefix[0])
        && prefix[54] == prefix[0]
        && prefix[55] == 0
        && let Some(lbl) = super::probe(|| verify_office_owner_lock(prefix, size))
    {
        return Some(lbl);
    }
    // Eclipse OSGi `framework.info.*` bundle cache: BE32 state format version `1..=20` at @0..4.
    if size >= 64
        && prefix.len() >= 32
        && prefix[0] == 0
        && prefix[1] == 0
        && prefix[2] == 0
        && (1..=20).contains(&prefix[3])
        && let Some(lbl) = super::probe(|| verify_osgi_framework_info4(prefix, size))
    {
        return Some(lbl);
    }
    // DEC PDP-11 Absolute Loader paper tape (.lda): optional zero leader or block sync word `0x0001` LE.
    if size >= 14
        && prefix.len() >= 2
        && (prefix[0] == 0 || (prefix[0] == 1 && prefix[1] == 0))
        && let Some(lbl) = super::probe(|| verify_pdp11_lda(prefix, size))
    {
        return Some(lbl);
    }
    // Pixel Painters AdLib music (.fmf): LE16 OPL2 patch count `2..=32` at @0..2.
    if size >= 36
        && prefix.len() >= 34
        && prefix[1] == 0
        && (2..=32).contains(&prefix[0])
        && let Some(lbl) = super::probe(|| verify_pixel_painters_fmf(prefix, size))
    {
        return Some(lbl);
    }
    // Pixel Painters Xatax FM module (.ftf): instrument count `1..=16` at @0 and track count `2..=14` at @1.
    if size >= 64
        && prefix.len() >= 32
        && (0x01..=0x10).contains(&prefix[0])
        && (0x02..=0x0E).contains(&prefix[1])
        && let Some(lbl) = super::probe(|| verify_pixel_painters_ftf(prefix, size))
    {
        return Some(lbl);
    }
    // Qualcomm Atheros Bluetooth rampatch/NVM TLV: type `1|2|4` at @0 and exact `4 + u24_le(@1..4) == size`.
    if size >= 16
        && prefix.len() >= 16
        && matches!(prefix[0], 1 | 2 | 4)
        && 4 + ((prefix[1] as u64) | ((prefix[2] as u64) << 8) | ((prefix[3] as u64) << 16)) == size
        && let Some(lbl) = super::probe(|| verify_qca_bt_tlv(prefix, size))
    {
        return Some(lbl);
    }
    // RagTime document: v4–6 `"C#+D"` at @0 (also in Tier 1 `magic32`) or Classic v2/v3 zero u16 at @0..2.
    if size >= 64
        && prefix.len() >= 36
        && (&prefix[0..4] == b"C#+D" || prefix[0..2] == [0x00, 0x00])
        && let Some(lbl) = super::probe(|| verify_ragtime_doc(prefix, size))
    {
        return Some(lbl);
    }
    // Realtek Wi-Fi firmware (`rtlwifi`/`rtw88`/`rtw89`): section tag `FF <n> 00 00` at @0 or @32,
    // chip signature `xx 88` at @0..2, or 12 reserved zero bytes at @20..32.
    if size >= 32
        && prefix.len() >= 32
        && ((prefix[0] == 0xFF && prefix[2] == 0 && prefix[3] == 0)
            || (prefix.len() >= 64 && prefix[32] == 0xFF && prefix[34] == 0 && prefix[35] == 0)
            || (prefix[1] == 0x88 && matches!(prefix[0], 0x14 | 0x21 | 0x22))
            || prefix[20..32] == [0u8; 12])
        && let Some(lbl) = super::probe(|| verify_realtek_wifi_firmware(prefix, size))
    {
        return Some(lbl);
    }
    // SAM Coupé E-Tracker module (.cop/.etc): Z80 player entry opcode (`01|21|3E`), init sequence at @6, or `"ETracker"` at @10.
    if size >= 64
        && prefix.len() >= 30
        && (matches!(prefix[0], 0x01 | 0x21 | 0x3E)
            || prefix[6..10] == [0x3E, 0x01, 0x3D, 0x20]
            || &prefix[10..14] == b"ETra")
        && let Some(lbl) = super::probe(|| verify_sam_coupe_cop(prefix, size))
    {
        return Some(lbl);
    }
    // Akella Storm Engine texture (.tx): LE32 flags `<= 0xFF` at @0..4 + DXT1/3/5 or D3DFMT `21..=26` at @16..20.
    if size >= 64
        && prefix.len() >= 24
        && prefix[1] == 0
        && prefix[2] == 0
        && prefix[3] == 0
        && (matches!(&prefix[16..20], b"DXT1" | b"DXT3" | b"DXT5")
            || (prefix[17] == 0
                && prefix[18] == 0
                && prefix[19] == 0
                && (21..=26).contains(&prefix[16])))
        && let Some(lbl) = super::probe(|| verify_seadogs_dxt1_tx(prefix, size))
    {
        return Some(lbl);
    }
    // SIMH magnetic tape image (.tap): headerless stream of symmetrically length-framed LE32 records.
    if size >= 26
        && prefix.len() >= 26
        && let Some(lbl) = super::probe(|| verify_simh_tape(prefix, size))
    {
        return Some(lbl);
    }
    // Apple FairPlay `.supp` key container: version `1` or `4` at @0 + ASCII build digit at @1.
    if size >= 46
        && prefix.len() >= 46
        && matches!(prefix[0], 1 | 4)
        && prefix[1].is_ascii_digit()
        && let Some(lbl) = super::probe(|| verify_supp_510_pkg(prefix, size))
    {
        return Some(lbl);
    }
    // TensorFlow TFRecord: headerless stream verified by masked CRC32C of the leading LE64 length at @8..12.
    if size >= 17
        && prefix.len() >= 16
        && let Some(lbl) = super::probe(|| verify_tensorflow_record(prefix, size))
    {
        return Some(lbl);
    }
    // TeX Font Metrics (.tfm): 32-bit word-aligned (`size % 4 == 0`), exact BE16 section-length sum.
    if size >= 64
        && size.is_multiple_of(4)
        && prefix.len() >= 32
        && let Some(lbl) = super::probe(|| verify_tex_tfm(prefix, size))
    {
        return Some(lbl);
    }
    // TI TAS2781/TAS2563 SmartAmp DSP firmware: magic `2A 9F 00 00` / `"5552"` at @0 or RCA tag `00 00 01 05` at @8..12.
    if size >= 64
        && prefix.len() >= 32
        && (matches!(&prefix[0..4], [0x2A, 0x9F, 0x00, 0x00] | b"5552")
            || prefix[8..12] == [0x00, 0x00, 0x01, 0x05])
        && let Some(lbl) = super::probe(|| verify_ti_smartamp_firmware(prefix, size))
    {
        return Some(lbl);
    }
    // Unreal Engine 2 UZ2 chunked zlib archive: LE32 compressed chunk `<= 65536` (`@2 <= 1, @3 == 0`) + uncompressed `<= 32768` (`@6..8 == 0`).
    if size >= 24
        && prefix.len() >= 12
        && prefix[2] <= 1
        && prefix[3] == 0
        && prefix[6] == 0
        && prefix[7] == 0
        && let Some(lbl) = super::probe(|| verify_unreal_uz2(prefix, size))
    {
        return Some(lbl);
    }
    // Windows CE ROM image: `"B000FF\n"` at @0 (also in Tier 1 `magic32`) or `"ECEC"` TOC signature at @64..68.
    if (prefix.starts_with(b"B000FF\n") || prefix.get(64..68) == Some(b"ECEC".as_slice()))
        && let Some(lbl) = super::probe(|| verify_wince_b000ff(prefix, size))
    {
        return Some(lbl);
    }
    // Sharp X68000 MXDRV music (.mdx): Shift-JIS song title terminated by `0D 0A 1A` within @0..512.
    if size >= 48
        && prefix.len() >= 28
        && memchr::memchr(0x1A, &prefix[..prefix.len().min(512)]).is_some()
        && let Some(lbl) = super::probe(|| verify_x68000_mdx(prefix, size))
    {
        return Some(lbl);
    }
    // ZX Spectrum SQ-Tracker AY module (.sqt): LE16 declared size @0..2 + monotonic Z80 RAM pointers @2..12.
    if (128..=65_552).contains(&size)
        && prefix.len() >= 24
        && let Some(lbl) = super::probe(|| verify_zx_sq_tracker(prefix, size))
    {
        return Some(lbl);
    }
    // ZX Spectrum KSA Sound Tracker Pro STP2 (.stp/.stp2): 28-byte header banner at @10..38.
    if (64..=65536).contains(&size)
        && prefix.get(10..38) == Some(b"KSA SOFTWARE COMPILATION OF ".as_slice())
        && let Some(lbl) = super::probe(|| verify_zx_stp_module(prefix, size))
    {
        return Some(lbl);
    }
    None
}
