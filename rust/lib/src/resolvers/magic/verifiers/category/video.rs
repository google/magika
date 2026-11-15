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

//! Video and multimedia stream format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// Adobe Flash Video (FLV v1): `"FLV\x01"`, valid audio/video flags (`0x00 | 0x01 | 0x04 | 0x05`),
/// data offset 9, and initial `PreviousTagSize0` = 0.
pub(crate) fn flv(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"FLV\x01"; // @0..4: FLV version 1
    const FLAGS: usize = 4; // @4: u8 — audio (0x04) | video (0x01) flags
    const DATA_OFFSET: usize = 5; // @5: u32 BE — header length (9)
    const PREV_TAG_0: usize = 9; // @9: u32 BE — initial previous-tag size (0)
    const MIN_LEN: usize = 13;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, FLAGS)?, 0x00 | 0x01 | 0x04 | 0x05),
        be32(prefix, DATA_OFFSET)? == 9,
        be32(prefix, PREV_TAG_0)? == 0,
    );
    Some(Label::Flv)
}

/// Adobe Flash SWF (`"FWS"`, `"CWS"`, or `"ZWS"`): version 1..=50 at `@3`, declared file length
/// ≥ 15 at `@4`, and compression-specific header fields at `@8`.
pub(crate) fn swf(prefix: &[u8]) -> Option<Label> {
    static SWF_FWS: Subformat = Subformat::new("SWF (uncompressed, FWS)");
    static SWF_CWS: Subformat = Subformat::new("SWF (zlib compressed, CWS)");
    static SWF_ZWS: Subformat = Subformat::new("SWF (LZMA compressed, ZWS)");
    const FWS_MAGIC: &[u8] = b"FWS";
    const CWS_MAGIC: &[u8] = b"CWS";
    const ZWS_MAGIC: &[u8] = b"ZWS";
    const VERSION: usize = 3; // @3: u8 — SWF format version (1..=50)
    const FILE_LENGTH: usize = 4; // @4: u32 LE — uncompressed file length (≥ 15)
    const PAYLOAD: usize = 8; // @8: RECT (FWS), zlib header (CWS), or LZMA header (ZWS)
    const MIN_LEN: usize = 15;

    require!(prefix.len() >= MIN_LEN, le32(prefix, FILE_LENGTH)? >= MIN_LEN as u32);
    let ver = u8_at(prefix, VERSION)?;
    require!((1..=50).contains(&ver));

    let sub = if starts(prefix, FWS_MAGIC) {
        let rect0 = u8_at(prefix, PAYLOAD)?;
        let rect1 = u8_at(prefix, PAYLOAD + 1)?;
        require!((8..=248).contains(&rect0) && rect0 % 8 == 0 && rect1 <= 31);
        &SWF_FWS
    } else if starts(prefix, CWS_MAGIC) {
        require!(
            ver >= 6
                && u8_at(prefix, PAYLOAD)? == 0x78
                && matches!(u8_at(prefix, PAYLOAD + 1)?, 0x01 | 0x5E | 0x9C | 0xDA)
        );
        &SWF_CWS
    } else if starts(prefix, ZWS_MAGIC) {
        const LZMA_PROPS: usize = 12; // @12: u8 — LZMA lc/lp/pb properties (0x5D = 93)
        const LZMA_DICT: usize = 13; // @13: u32 LE — LZMA dictionary size (4 KiB .. 1 GiB)
        require!(
            prefix.len() >= 17
                && ver >= 13
                && le32(prefix, PAYLOAD)? >= 5
                && u8_at(prefix, LZMA_PROPS)? == 0x5D
                && (4096..=1_073_741_824).contains(&le32(prefix, LZMA_DICT)?)
        );
        &SWF_ZWS
    } else {
        return None;
    };
    with_subformat(Label::Swf, sub)
}

/// MPEG-2 Transport Stream (`.ts` or Blu-ray `.m2ts`): 21 consecutive 188-byte packets starting
/// at `@0` (prefix ≥ 3948) or 192-byte packets with 4-byte timecode starting at `@4` (prefix ≥
/// 4032), each with sync byte `0x47`, `transport_error_indicator == 0`, and nonzero
/// `adaptation_field_control`.
pub(crate) fn mpegts(prefix: &[u8]) -> Option<Label> {
    static MPEG_TS: Subformat =
        Subformat::with_ids("MPEG-2 Transport Stream (.ts)", "fmt/585", Some("fdd000635"));
    static MPEG_M2TS: Subformat =
        Subformat::with_ids("BDAV MPEG-2 Transport Stream (.m2ts)", "fmt/1055", Some("fdd000636"));
    const SYNC: u8 = 0x47; // MPEG-TS sync byte
    const PACKETS: usize = 21;
    const TS_STRIDE: usize = 188;
    const M2TS_STRIDE: usize = 192;
    const TS_MIN_LEN: usize = 3948; // 21 * 188
    const M2TS_MIN_LEN: usize = 4032; // 21 * 192

    #[inline]
    fn valid_packets(prefix: &[u8], start: usize, stride: usize) -> bool {
        let mut has_bin = false;
        for k in 0..PACKETS {
            let p = start + k * stride;
            let Some(hdr) = get_slice(prefix, p, p + 4) else {
                return false;
            };
            if hdr[0] != SYNC || hdr[1] & 0x80 != 0 || hdr[3] & 0x30 == 0 {
                return false;
            }
            if !has_bin && has_non_ascii_text_byte(&hdr[1..4]) {
                has_bin = true;
            }
        }
        has_bin
    }

    if prefix.len() >= TS_MIN_LEN
        && u8_at(prefix, 0) == Some(SYNC)
        && valid_packets(prefix, 0, TS_STRIDE)
    {
        return with_subformat(Label::Mpegts, &MPEG_TS);
    }
    if prefix.len() >= M2TS_MIN_LEN
        && u8_at(prefix, 4) == Some(SYNC)
        && valid_packets(prefix, 4, M2TS_STRIDE)
    {
        return with_subformat(Label::Mpegts, &MPEG_M2TS);
    }
    None
}

/// W3C WebVTT Timed Text Subtitle Track (`vtt`, 2011): `"WEBVTT"` header (with optional UTF-8 BOM)
/// followed by cue arrow `"-->"` within the first 512 bytes.
pub(crate) fn vtt(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_WEBVTT_SIZE: u64 = 16; // "WEBVTT\n\n0 --> 1\n"
    const WEBVTT_LF: &[u8] = b"WEBVTT\n";
    const WEBVTT_CRLF: &[u8] = b"WEBVTT\r\n";
    const WEBVTT_SPACE: &[u8] = b"WEBVTT ";
    const WEBVTT_CUE_ARROW: &[u8] = b"-->";

    require!(size >= MIN_WEBVTT_SIZE, prefix.len() >= 16);
    let off = if starts(prefix, UTF8_BOM) { 3 } else { 0 };
    require!(
        eq_at(prefix, off, WEBVTT_LF)
            || eq_at(prefix, off, WEBVTT_CRLF)
            || eq_at(prefix, off, WEBVTT_SPACE),
        scan_range(prefix, off + 6, 512, WEBVTT_CUE_ARROW),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Vtt)
}

/// SubRip Text Video Subtitle Track (`srt`, 2000): cue index followed by
/// `HH:MM:SS,mmm --> HH:MM:SS,mmm` timestamp line.
pub(crate) fn srt(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_SRT_SIZE: u64 = 32; // "1\n00:00:00,000 --> 00:00:01,000\nA"
    const SRT_CUE_ARROW: &[u8; 5] = b" --> ";

    require!(size >= MIN_SRT_SIZE, prefix.len() >= 32);
    let win = &prefix[..prefix.len().min(256)];
    let win = win.strip_prefix(UTF8_BOM).unwrap_or(win);
    let win = win.trim_ascii_start();
    let mut idx_len = 0usize;
    while idx_len < win.len() && win[idx_len].is_ascii_digit() {
        idx_len += 1;
    }
    require!((1..=6).contains(&idx_len));
    let after_idx = &win[idx_len..];
    let after_nl = after_idx.strip_prefix(b"\r\n").or_else(|| after_idx.strip_prefix(b"\n"))?;
    require!(
        after_nl.len() >= 29,
        after_nl[2] == b':',
        after_nl[5] == b':',
        matches!(after_nl[8], b',' | b'.'),
        &after_nl[12..17] == SRT_CUE_ARROW,
        after_nl[19] == b':',
        after_nl[22] == b':',
        matches!(after_nl[25], b',' | b'.'),
        is_utf8_text_header(prefix, size as usize),
    );
    let end_off = (after_nl.as_ptr() as usize) + 29 - (prefix.as_ptr() as usize);
    let _ = get_slice(prefix, 0, end_off.min(prefix.len()));
    Some(Label::Srt)
}

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const DV_DIF_HEADER_PREFIX: &[u8; 3] = b"\x1f\x07\x00";
const INTEL_DVI_AVL_FILE_ID: &[u8; 4] = le32_bytes!(0x4156_5353);
const INTEL_DVI_FILE_MAGIC: &[u8; 4] = le32_bytes!(0x5644_5649);
const MIN_ARGONAUT_BRP_SIZE: u64 = 24;
const MIN_BETHESDA_VID_SIZE: u64 = 32;
const MIN_BFI_VIDEO_SIZE: u64 = 36;
const MIN_BLURAY_CLPI_SIZE: u64 = 64;
const MIN_BLURAY_MOBJ_SIZE: u64 = 56;
const MIN_BLURAY_MPLS_SIZE: u64 = 58;
const MIN_CHRONOMASTER_DFA_SIZE: u64 = 128;
const MIN_CRI_USM_SIZE: u64 = 32;
const MIN_CRYO_HNM4_SIZE: u64 = 64;
const MIN_CYBERPAINT_SEQ_SIZE: u64 = 132;
const MIN_DEMOMAKER_FLM_SIZE: u64 = 32;
const MIN_DUCK_IVF_SIZE: u64 = 44;
const MIN_DVD_VIDEO_IFO_SIZE: u64 = 2048;
const MIN_DV_VIDEO_SIZE: u64 = 160;
const MIN_DXA_SIZE: u64 = 32;
const MIN_ID_ROQ_SIZE: u64 = 16;
const MIN_INTEL_DVI_SIZE: u64 = 132;
const MIN_JVC_EVERIO_MOI_SIZE: u64 = 256;
const MIN_LOTUS_SCREENCAM_SIZE: u64 = 8;
const MIN_NUPPELVIDEO_SIZE: u64 = 72;
const MIN_NUT_CONTAINER_SIZE: u64 = 48;
const MIN_PS2_IPU_SIZE: u64 = 20;
const MIN_RAD_BINK_SIZE: u64 = 44;
const MIN_RAD_SMACKER_SIZE: u64 = 104;
const MIN_REALMEDIA_SIZE: u64 = 18;
const MIN_REDCODE_R3D_SIZE: u64 = 68;
const MIN_SCALEFORM_GFX_V8_SIZE: u64 = 20;
const MIN_SCUMM_CUTSCENE_CUT_SIZE: u64 = 64;
const MIN_SEGA_FILM_SIZE: u64 = 64;
const MIN_SGI_MOVIE_SIZE: u64 = 8;
const MIN_SIERRA_RBT_SIZE: u64 = 60;
const MIN_WINK_WNK_SIZE: u64 = 60;
const MIN_WINK_WTP_SIZE: u64 = 168;
const MIN_WTV_SIZE: u64 = 4096;
const SEGA_FILM_FDSC_CHUNK_TAG: &[u8; 4] = b"FDSC";
const SEGA_FILM_MAGIC: &[u8; 4] = b"FILM";
const WINK_WNK_MAGIC: &[u8; 4] = b"RPKW";
const WINK_WTP_V1_MAGIC: &[u8; 4] = b"WTP ";
const WINK_WTP_V2_MAGIC: &[u8; 4] = b"WTP2";
const WINK_WTP_V3_MAGIC: &[u8; 4] = b"WTP3";
const WINK_WTP_V4_MAGIC: &[u8; 4] = b"WTP4";

/// Argonaut Games BRPP Video (`.brp`, `"BRPP"`).
pub(crate) fn verify_argonaut_brp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ARGONAUT_BRP_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"BRPP"),
        matches!(le32(prefix, 4), Some(1 | 2)),
        (1..=16).contains(&le32(prefix, 8)?)
            || (le32(prefix, 8)? > 0 && matches!(get_slice(prefix, 12, 16)?, b"BVID" | b"BASF"))
    );
    Some(Label::ArgonautBrp)
}

/// Bethesda Softworks VID Video (`.vid`, `"VID\0"`).
pub(crate) fn verify_bethesda_vid(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BETHESDA_VID_SIZE, prefix.len() >= 14, starts(prefix, b"VID\0"));
    let packed_ok = u8_at(prefix, 4)? == 2
        && le16(prefix, 5)? > 0
        && (16..=4096).contains(&le16(prefix, 7)?)
        && (16..=4096).contains(&le16(prefix, 9)?);
    let aligned_ok = le16(prefix, 6)? > 0
        && (16..=4096).contains(&le16(prefix, 8)?)
        && (16..=4096).contains(&le16(prefix, 10)?);
    require!(packed_ok || aligned_ok);
    Some(Label::BethesdaVid)
}

/// Blue Byte Brute Force & Ignorance Video (`.bfi`, `"BF&I"`).
pub(crate) fn verify_bfi_video(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BFI_VIDEO_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"BF&I"),
        le32(prefix, 8)? > 0,
        prefix.get(4..20)?.contains(&0)
    );
    Some(Label::BfiVideo)
}

/// Blu-ray Disc Clip Information (`.clpi`, `"HDMV0100"` / `"HDMV0200"` / `"HDMV0300"`).
pub(crate) fn verify_bluray_clpi(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BLURAY_CLPI_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"HDMV0100") || starts(prefix, b"HDMV0200") || starts(prefix, b"HDMV0300"),
        be32(prefix, 8)? >= 40,
        prefix.get(8..20)?.contains(&0)
    );
    Some(Label::BlurayClpi)
}

/// Blu-ray Disc MovieObject (`MovieObject.bdmv`, `"MOBJ0100"` / `"MOBJ0200"` / `"MOBJ0300"`).
pub(crate) fn verify_bluray_movie_object(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BLURAY_MOBJ_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"MOBJ0100") || starts(prefix, b"MOBJ0200") || starts(prefix, b"MOBJ0300"),
        matches!(be32(prefix, 8), Some(0 | 40..)),
        get_slice(prefix, 12, 40)?.iter().all(|&b| b == 0),
        be32(prefix, 40)? > 0
    );
    Some(Label::BlurayMobj)
}

/// Blu-ray Disc Movie PlayList (`.mpls`, `"MPLS0100"` / `"MPLS0200"` / `"MPLS0300"`).
pub(crate) fn verify_bluray_mpls(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BLURAY_MPLS_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"MPLS0100") || starts(prefix, b"MPLS0200") || starts(prefix, b"MPLS0300"),
        be32(prefix, 8)? >= 40,
        prefix.get(8..20)?.contains(&0)
    );
    Some(Label::BlurayMpls)
}

/// DreamForge Chronomaster DFA Video (`.dfa`, `"DFIA"`).
pub(crate) fn verify_chronomaster_dfa(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CHRONOMASTER_DFA_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"DFIA"),
        le16(prefix, 6)? > 0,
        (16..=4096).contains(&le16(prefix, 8)?),
        (16..=4096).contains(&le16(prefix, 10)?)
    );
    Some(Label::ChronomasterDfa)
}

/// CRI Middleware Sofdec2 USM Video Container (.usm).
pub(crate) fn verify_cri_usm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CRI_USM_SIZE
            && prefix.len() >= 32
            && starts(prefix, b"CRID")
            && be32(prefix, 4).unwrap_or(0) >= 24
            && prefix.get(4..16)?.contains(&0)
            && (memchr::memmem::find(prefix, b"@UTF").is_some()
                || memchr::memmem::find(prefix, b"@SFV").is_some()
                || u8_at(prefix, 9).unwrap_or(0) == 0x18)
    );
    Some(Label::CriUsm)
}

/// Cryo Interactive HNM4 / HNM6 Video (`.hnm` / `.hns`, `"HNM4"` / `"HNM6"`).
pub(crate) fn verify_cryo_hnm4_video(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CRYO_HNM4_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"HNM4") || starts(prefix, b"HNM6"),
        ((64..=1920).contains(&le16(prefix, 8)?) && (48..=1080).contains(&le16(prefix, 10)?))
            || ((64..=1920).contains(&le16(prefix, 6)?) && (48..=1080).contains(&le16(prefix, 8)?)),
        le32(prefix, 16)? > 0
    );
    Some(Label::CryoHnm4)
}

/// Atari ST Antic Cyber Paint Sequence Animation (.seq).
pub(crate) fn verify_cyberpaint_seq(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CYBERPAINT_SEQ_SIZE
            && prefix.len() >= 64
            && (starts(prefix, hex2bin!("FE DB 00 00")) || starts(prefix, hex2bin!("FE DC 00 00")))
    );
    let frames16 = be16(prefix, 4)? as u32;
    let frames32 = be32(prefix, 4)?;
    require!(
        ((1..=4096).contains(&frames16) || (1..=4096).contains(&frames32))
            && get_slice(prefix, 10, 64)?.iter().all(|&b| b == 0)
    );
    Some(Label::CyberpaintSeq)
}

/// Atari ST DemoMaker / Cyber Paint Film Animation (.flm / .seq).
pub(crate) fn verify_demomaker_flm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DEMOMAKER_FLM_SIZE
            && prefix.len() >= 16
            && (starts(prefix, b"FLM\0\0")
                || starts(prefix, &[0xA9, 0xD1, 0x6A, 0x30, 0x00, 0x02, 0x0C])
                || (starts(prefix, &[0x03, 0x19, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00])
                    && eq_at(prefix, 32, b"SCREEN\0\x04")))
    );
    Some(Label::DemomakerFlm)
}

/// Duck TrueMotion / On2 / AOMedia IVF Video Container (.ivf).
pub(crate) fn verify_duck_ivf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DUCK_IVF_SIZE
            && prefix.len() >= 32
            && starts(prefix, b"DKIF")
            && le16(prefix, 4) == Some(0)
            && le16(prefix, 6) == Some(32)
            && get_slice(prefix, 8, 12)?.iter().all(|b| b.is_ascii_graphic())
            && le16(prefix, 12).unwrap_or(0) > 0
            && le16(prefix, 14).unwrap_or(0) > 0
    );
    Some(Label::Ivf)
}

/// IEC 61834 / SMPTE 314M Digital Video DIF Stream (`.dv` / `.dif`, 1995, PRONOM `x-fmt/152`).
///
/// Header layout:
/// - `@0..3`: `1F 07 00` (DIF Header block ID: `SCT = 0`, `Arb = 0xF`, `Dseq = 0`, `FSC = 0`, `DBN = 0`)
/// - `@3`: `DSF` & reserved bits (`0x3F` for 525/60 NTSC or `0xBF` for 625/50 PAL)
/// - `@8..80`: Reserved `0xFF` padding bytes inside the 80-byte Header DIF block
/// - `@80`: Next 80-byte DIF block (`SCT` in upper 3 bits: `0x3F` Subcode, `0x5F` VAUX, `0x7F` Audio, `0x9F` Video).
pub(crate) fn verify_dv_video(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DV_VIDEO_SIZE,
        prefix.len() >= 84,
        starts(prefix, DV_DIF_HEADER_PREFIX),
        matches!(u8_at(prefix, 3)?, 0x3F | 0xBF),
        get_slice(prefix, 16, 80)?.iter().all(|&b| b == 0xFF),
        matches!(u8_at(prefix, 80)? & 0x1F, 0x1F)
    );
    Some(Label::DvVideo)
}

/// DVD-Video VMGI / VTSI Navigation File (`.ifo` / `.bup`, `"DVDVIDEO-VMG"` / `"DVDVIDEO-VTS"`).
pub(crate) fn verify_dvd_video_ifo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DVD_VIDEO_IFO_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"DVDVIDEO-VMG") || starts(prefix, b"DVDVIDEO-VTS"),
        be32(prefix, 12)? >= 1,
        prefix.get(12..24)?.contains(&0)
    );
    Some(Label::DvdIfo)
}

/// DXA: `"DEXA"`, `u8 flags`, `u16 BE frame_count`, `i32 BE frame_rate`, `u16 BE width`, `u16 BE height`.
pub(crate) fn verify_dxa(prefix: &[u8], size: u64) -> Option<Label> {
    const FLAGS: usize = 4; // @4: u8 flags (<= 0x0F)
    const FRAME_COUNT: usize = 5; // @5: u16 BE
    const WIDTH: usize = 11; // @11: u16 BE
    const HEIGHT: usize = 13; // @13: u16 BE
    require!(size >= MIN_DXA_SIZE, prefix.len() >= 15, starts(prefix, b"DEXA"));
    require!(
        u8_at(prefix, FLAGS)? <= 0x0F,
        be16(prefix, FRAME_COUNT)? > 0,
        (1..=4096).contains(&be16(prefix, WIDTH)?),
        (1..=4096).contains(&be16(prefix, HEIGHT)?)
    );
    Some(Label::Dxa)
}

/// id Software Quake III / Doom 3 RoQ Video (`.roq`, `84 10 FF FF FF FF`).
pub(crate) fn verify_id_roq_video(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ID_ROQ_SIZE,
        prefix.len() >= 8,
        starts(prefix, &[0x84, 0x10, 0xFF, 0xFF, 0xFF, 0xFF]),
        (1..=240).contains(&le16(prefix, 6)?)
    );
    Some(Label::IdRoq)
}

/// Intel/IBM DVI (Digital Video Interactive) ActionMedia AVSS multimedia file (`.dvi` / `.avs`, 1989–1992).
///
/// Little-endian structures:
/// - `StdFileHdr`: `@0` `FileId` (`0x56445649`), `@4` `HdrSize` (`12`, known-bad files store other values),
///   `@6` `HdrVersion == 1`, `@8` `AnnOffset`.
/// - `AvLFile` @12: `HdrID` (`0x41565353`), `@16` `HdrSize` (`120`), `@18` `HdrVer` (`1..=3`).
pub(crate) fn verify_intel_dvi_avss(prefix: &[u8], size: u64) -> Option<Label> {
    const STD_HDR_VERSION_OFFSET: usize = 6; // @6..8: i16 LE StdFileHdr.HdrVersion (VSTD_HDR_VER = 1)
    const AVL_FILE_OFFSET: usize = 12; // @12..16: AvLFile.HdrID
    const AVL_HDR_SIZE_OFFSET: usize = 16; // @16..18: i16 LE AvLFile.HdrSize (120)
    const AVL_HDR_VER_OFFSET: usize = 18; // @18..20: i16 LE AvLFile.HdrVer (AVL_FILE_VER <= 3)
    require!(
        size >= MIN_INTEL_DVI_SIZE,
        starts(prefix, INTEL_DVI_FILE_MAGIC),
        le16(prefix, STD_HDR_VERSION_OFFSET)? == 1,
        eq_at(prefix, AVL_FILE_OFFSET, INTEL_DVI_AVL_FILE_ID),
        (20..=1024).contains(&le16(prefix, AVL_HDR_SIZE_OFFSET)?),
        (1..=3).contains(&le16(prefix, AVL_HDR_VER_OFFSET)?)
    );
    Some(Label::IntelDvi)
}

/// JVC Everio Camcorder Media Object Information sidecar file (`.moi`).
///
/// Header layout:
/// - `@0..2`: `"V6"` or `"V7"` (`2B` `MOIVersion`)
/// - `@2..6`: `u32 BE moi_file_size` (`== size`, with `@2..4 == 0x0000` for sidecar files `< 64 KiB`)
/// - `@6..8`: `u16 BE year` (`1999..=2100`)
/// - `@8`: `u8 month` (`1..=12`)
/// - `@9`: `u8 day` (`1..=31`)
/// - `@10`: `u8 hour` (`0..=23`)
/// - `@11`: `u8 minute` (`0..=59`)
/// - `@12..14`: `u16 BE milliseconds` (`0..=59999`)
pub(crate) fn verify_jvc_everio_moi(prefix: &[u8], size: u64) -> Option<Label> {
    const FILE_SIZE_OFFSET: usize = 2; // @2..6: u32 BE MOIFileSize
    const YEAR_OFFSET: usize = 6; // @6..8: u16 BE recording year (1999..=2100)
    const MONTH_OFFSET: usize = 8; // @8: u8 recording month (1..=12)
    const DAY_OFFSET: usize = 9; // @9: u8 recording day (1..=31)
    const HOUR_OFFSET: usize = 10; // @10: u8 recording hour (0..=23)
    const MINUTE_OFFSET: usize = 11; // @11: u8 recording minute (0..=59)
    const MSEC_OFFSET: usize = 12; // @12..14: u16 BE milliseconds (0..=59999)

    require!(
        size >= MIN_JVC_EVERIO_MOI_SIZE,
        prefix.len() >= 18,
        starts(prefix, b"V6\x00\x00") || starts(prefix, b"V7\x00\x00"),
        be32(prefix, FILE_SIZE_OFFSET)? as u64 == size,
        (1999..=2100).contains(&be16(prefix, YEAR_OFFSET)?),
        (1..=12).contains(&u8_at(prefix, MONTH_OFFSET)?),
        (1..=31).contains(&u8_at(prefix, DAY_OFFSET)?),
        u8_at(prefix, HOUR_OFFSET)? <= 23,
        u8_at(prefix, MINUTE_OFFSET)? <= 59,
        be16(prefix, MSEC_OFFSET)? <= 59_999
    );
    Some(Label::JvcEverioMoi)
}

/// Lotus ScreenCam Movie File (.scm, PRONOM fmt/1938).
pub(crate) fn verify_lotus_screencam(prefix: &[u8], size: u64) -> Option<Label> {
    const SCREENCAM_V3_MAGIC: &[u8; 8] = b"\x80SCMOVv3"; // @0..8: Lotus ScreenCam v3 header
    const SCREENCAM_V2_MAGIC: &[u8; 8] = b"\x80SCMOVv2"; // @0..8: Lotus ScreenCam v2 header
    if size >= MIN_LOTUS_SCREENCAM_SIZE
        && prefix.len() >= 8
        && (starts(prefix, SCREENCAM_V3_MAGIC) || starts(prefix, SCREENCAM_V2_MAGIC))
    {
        Some(Label::Screencam)
    } else {
        None
    }
}

/// NuppelVideo / MythTV Recording (`.nuv`, `"NuppelVideo\0"` / `"MythTVVideo\0"`).
pub(crate) fn verify_nuppel_video(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NUPPELVIDEO_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"NuppelVideo\0") || starts(prefix, b"MythTVVideo\0"),
        eq_at(prefix, 12, b"0.0")
    );
    Some(Label::Nuppelvideo)
}

/// FFmpeg / MPlayer NUT Multimedia Container (`.nut`, `"nut/multimedia container\0"`).
pub(crate) fn verify_nut_container(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_NUT_CONTAINER_SIZE, starts(prefix, b"nut/multimedia container\0"));
    Some(Label::NutVideo)
}

/// Sony PlayStation 2 IPU Video Stream (`.ipu`, `"ipum"`).
pub(crate) fn verify_ps2_ipu_video(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PS2_IPU_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"ipum"),
        le32(prefix, 4)? > 0,
        (16..=1920).contains(&le16(prefix, 8)?),
        (16..=1088).contains(&le16(prefix, 10)?),
        le32(prefix, 12)? > 0
    );
    Some(Label::Ps2Ipu)
}

/// RAD Game Tools Bink Video Container (.bik / .bk2).
pub(crate) fn verify_rad_bink(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RAD_BINK_SIZE
            && prefix.len() >= 32
            && (starts(prefix, b"BIK") || starts(prefix, b"KB2"))
    );
    let frames = le32(prefix, 8).unwrap_or(0);
    let w = le32(prefix, 20).unwrap_or(0);
    let h = le32(prefix, 24).unwrap_or(0);
    require!(frames > 0 && (1..=16384).contains(&w) && (1..=16384).contains(&h));
    Some(Label::Bink)
}

/// RAD Game Tools Smacker Video Container (.smk).
pub(crate) fn verify_rad_smacker(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RAD_SMACKER_SIZE
            && prefix.len() >= 24
            && (starts(prefix, b"SMK2") || starts(prefix, b"SMK4"))
    );
    let w = le32(prefix, 4).unwrap_or(0);
    let h = le32(prefix, 8).unwrap_or(0);
    let frames = le32(prefix, 12).unwrap_or(0);
    require!((1..=4096).contains(&w) && (1..=4096).contains(&h) && frames > 0);
    Some(Label::Smacker)
}

/// RealNetworks RealMedia / RealAudio Stream (.rm / .rmvb / .ra).
pub(crate) fn verify_realmedia(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_REALMEDIA_SIZE
            && prefix.len() >= 18
            && (starts(prefix, b".RMF") || starts(prefix, b".ra\xFD"))
    );
    let valid = if starts(prefix, b".RMF") {
        be32(prefix, 4) == Some(18) && be16(prefix, 8).unwrap_or(99) <= 2
    } else {
        (3..=5).contains(&be16(prefix, 4).unwrap_or(0))
    };
    require!(valid);
    Some(Label::Realmedia)
}

/// RED Digital Cinema REDCODE RAW video file (`.r3d`).
///
/// Header layout:
/// - `@0..4`: `u32 BE atom_size` (`68..=65535`, `@0..2 == 0x0000`)
/// - `@4..8`: `"RED1"` (R3D v1) or `"RED2"` (R3D v2)
/// - `@10..12`: `"R1"` (when `"RED1"`) or `"R2"` (when `"RED2"`).
pub(crate) fn verify_redcode_r3d_offset(prefix: &[u8], size: u64) -> Option<Label> {
    const ATOM_TYPE_OFFSET: usize = 4; // @4..8: "RED1" or "RED2"
    const SUBVER_OFFSET: usize = 10; // @10..12: "R1" or "R2"

    require!(
        size >= MIN_REDCODE_R3D_SIZE,
        prefix.len() >= 16,
        (eq_at(prefix, ATOM_TYPE_OFFSET, b"RED1") && eq_at(prefix, SUBVER_OFFSET, b"R1"))
            || (eq_at(prefix, ATOM_TYPE_OFFSET, b"RED2") && eq_at(prefix, SUBVER_OFFSET, b"R2"))
    );
    let atom_sz = be32(prefix, 0)? as u64;
    require!((68..=65535).contains(&atom_sz), atom_sz <= size);
    Some(Label::RedcodeR3d)
}

/// Autodesk Scaleform GFx Movie (`.gfx`, uncompressed `"GFX"` or zlib `"CFX"`).
pub(crate) fn verify_scaleform_gfx_v8(prefix: &[u8], size: u64) -> Option<Label> {
    const SCALEFORM_GFX_PREFIX: &[u8; 3] = b"GFX"; // @0..3: uncompressed
    const SCALEFORM_CFX_PREFIX: &[u8; 3] = b"CFX"; // @0..3: zlib-compressed

    require!(
        size >= MIN_SCALEFORM_GFX_V8_SIZE,
        prefix.len() >= 10,
        (6..=20).contains(&u8_at(prefix, 3)?), // @3: Flash / Scaleform version
        (32..=0x2000_0000).contains(&le32(prefix, 4)?)  // @4..8: uncompressed file length
    );
    if starts(prefix, SCALEFORM_CFX_PREFIX) {
        require!(is_valid_zlib_header(u8_at(prefix, 8)?, u8_at(prefix, 9)?));
        return Some(Label::ScaleformGfx);
    }
    require!(starts(prefix, SCALEFORM_GFX_PREFIX));
    let nbits = (u8_at(prefix, 8)? >> 3) as usize;
    require!((1..=31).contains(&nbits));
    let rect_bytes = (5 + nbits * 4).div_ceil(8);
    let fps_off = 8 + rect_bytes;
    require!(
        prefix.len() >= fps_off + 4,
        (1..=120).contains(&u8_at(prefix, fps_off + 1)?), // frame rate integer part
        le16(prefix, fps_off + 2)? >= 1                   // frame count >= 1
    );
    Some(Label::ScaleformGfx)
}

/// LucasArts SCUMM Cutscene Resource (.cut).
pub(crate) fn verify_scumm_cutscene_cut(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SCUMM_CUTSCENE_CUT_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 6)? == hex2bin!("01 00 00 00 20 00"),
        get_slice(prefix, 8, 10)? == hex2bin!("00 00"),
        get_slice(prefix, 16, 18)? == hex2bin!("50 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::ScummCutsceneCut)
}

/// Sega Saturn / Dreamcast FILM / CPK Multimedia Container (`.cpk`, 1994 Sega).
pub(crate) fn verify_sega_film(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SEGA_FILM_SIZE
            && prefix.len() >= 32
            && starts(prefix, SEGA_FILM_MAGIC)
            && eq_at(prefix, 16, SEGA_FILM_FDSC_CHUNK_TAG)
    );
    let hdr_len = be32(prefix, 4)? as u64;
    require!(
        hdr_len >= 32
            && hdr_len <= size
            && (eq_at(prefix, 8, b"1.0")
                || eq_at(prefix, 8, b"d1.0")
                || eq_at(prefix, 8, &[0, 0, 0, 0]))
    );
    Some(Label::SegaFilm)
}

/// Silicon Graphics Movie Format (.mv/.movie, PRONOM fmt/1901).
pub(crate) fn verify_sgi_movie(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_SGI_MOVIE_SIZE && prefix.len() >= 8 && starts(prefix, b"MOVI\x00\x00\x00\x03") {
        Some(Label::SgiMovie)
    } else {
        None
    }
}

/// Sierra On-Line SCI2.1 / SCI3 Robot Video Animation (`.rbt`, `16 00 "SOL\0"`).
pub(crate) fn verify_sierra_rbt(prefix: &[u8], size: u64) -> Option<Label> {
    const RBT_MAGIC: &[u8; 6] = b"\x16\x00SOL\0"; // @0..6: 0x0016 LE + "SOL\0"
    const VERSION_OFFSET: usize = 6; // @6..8: u16 LE Robot version (4, 5, or 6)
    const FRAMES_OFFSET: usize = 14; // @14..16: u16 LE total frame count (>= 1)

    require!(
        size >= MIN_SIERRA_RBT_SIZE,
        prefix.len() >= 60,
        starts(prefix, RBT_MAGIC),
        (4..=6).contains(&le16(prefix, VERSION_OFFSET)?),
        le16(prefix, FRAMES_OFFSET)? >= 1
    );
    Some(Label::SierraRbt)
}

/// DebugMode Wink Screen Capture Presentation (`.wnk`, `"RPKW"`, 2003) Satish Kumar / DebugMode.
///
/// Header layout:
/// - `@0..4`: `"RPKW"` (`52 50 4B 57`) — byte-reversed `"WKPR"` (*Wink Project*).
/// - `@4..6`: `version` (`u16 LE`): `0x0001..=0x0300` (`0x0100` v1.x, `0x0249` v2.0, `0x0300` v3.x).
/// - `@6..10`: `flag1` (`u32 LE` C++ `BOOL`: `0` or `1`).
/// - `@10..14` *(only when `version >= 0x0249`)*: `flag2` (`u32 LE` C++ `BOOL`: `0` or `1`).
/// - Next 32 bytes (`@10..42` for v1; `@14..46` for v2/v3): eight `u32 LE` fields —
///   `num_frames`, `width`, `height`, `output_width`, `output_height`, `crop_x`, `crop_y`, `bg_zlib_len`.
pub(crate) fn verify_wink_presentation(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4: u16 LE format version (0x0100 v1.x, 0x0249 v2.0, 0x0300 v3.x)
    const V2_MIN_VERSION: u16 = 0x0249; // Wink 2.0 adds 4-byte flag2 at @10..14
    const MAX_VERSION: u16 = 0x0300; // Wink 3.01 rejects version > 0x0300
    const FLAG1_OFF: usize = 6; // @6: u32 LE boolean flag1 (0 or 1)
    const FLAG2_OFF: usize = 10; // @10: u32 LE boolean flag2 (0 or 1, version >= 0x0249)
    const GEOM_OFF_V1: usize = 10; // @10..42: 8 x u32 LE geometry block (v1)
    const GEOM_OFF_V2: usize = 14; // @14..46: 8 x u32 LE geometry block (v2/v3)
    const GEOM_LEN: usize = 32; // 8 x u32 LE
    const MIN_V2_SIZE: u64 = 64; // 46 B v2/v3 fixed header + 2 B string lengths + 16 B settings
    const MAX_FRAMES: u32 = 1_000_000;
    const MAX_DIM: u32 = 16_384;

    require!(
        size >= MIN_WINK_WNK_SIZE,
        prefix.len() >= GEOM_OFF_V1 + GEOM_LEN,
        starts(prefix, WINK_WNK_MAGIC)
    );
    let version = le16(prefix, VERSION_OFF)?;
    require!((1..=MAX_VERSION).contains(&version));
    let flag1 = le32(prefix, FLAG1_OFF)?;
    require!(flag1 <= 1);

    let (geom_off, min_size) = if version >= V2_MIN_VERSION {
        let flag2 = le32(prefix, FLAG2_OFF)?;
        require!(flag2 <= 1);
        (GEOM_OFF_V2, MIN_V2_SIZE)
    } else {
        (GEOM_OFF_V1, MIN_WINK_WNK_SIZE)
    };
    let hdr_end = geom_off + GEOM_LEN;
    require!(size >= min_size, prefix.len() >= hdr_end);

    let num_frames = le32(prefix, geom_off)?;
    let width = le32(prefix, geom_off + 4)?;
    let height = le32(prefix, geom_off + 8)?;
    let output_width = le32(prefix, geom_off + 12)?;
    let output_height = le32(prefix, geom_off + 16)?;
    let crop_x = le32(prefix, geom_off + 20)?;
    let crop_y = le32(prefix, geom_off + 24)?;
    let bg_zlib_len = le32(prefix, geom_off + 28)?;

    require!(
        num_frames <= MAX_FRAMES,
        width <= MAX_DIM,
        height <= MAX_DIM,
        num_frames == 0 || (width >= 1 && height >= 1),
        output_width <= MAX_DIM,
        output_height <= MAX_DIM,
        crop_x <= width,
        crop_y <= height,
        (hdr_end as u64).saturating_add(bg_zlib_len as u64) <= size
    );

    Some(Label::WinkWnk)
}

/// DebugMode Wink Presentation Template (`.wtp`, `"WTP "`/`"WTP2"`/`"WTP3"`/`"WTP4"`, 2003) Satish Kumar / DebugMode.
///
/// Header layout:
/// - `@0..4`: `"WTP "` (v1), `"WTP2"` (v2), `"WTP3"` (v3), or `"WTP4"` (v4).
/// - `@4..8`: `flags` (`u32 LE`, upper 16 bits zero; bit `0x0001` = uncompressed RGB24 background image follows `@24`).
/// - `@8..16`: `outer_width`, `outer_height` (`2 x u32 LE`).
/// - `@16..24`: `inner_width`, `inner_height` (`2 x u32 LE`).
/// - `@24..`: if `(flags & 1) != 0`, `outer_width * outer_height * 3` bytes of RGB24 image data before template records.
pub(crate) fn verify_wink_template(prefix: &[u8], size: u64) -> Option<Label> {
    const FLAGS_OFF: usize = 4; // @4: u32 LE template feature bitmask (upper 16 bits 0)
    const OUTER_W_OFF: usize = 8; // @8: u32 LE outer canvas width
    const OUTER_H_OFF: usize = 12; // @12: u32 LE outer canvas height
    const INNER_W_OFF: usize = 16; // @16: u32 LE inner capture viewport width
    const INNER_H_OFF: usize = 20; // @20: u32 LE inner capture viewport height
    const HEADER_LEN: usize = 24; // 4 B magic + 5 x u32 LE
    const FLAG_HAS_BG_IMAGE: u32 = 0x0001; // bit 0: raw RGB24 background (outer_w * outer_h * 3 B)
    const MAX_DIM: u32 = 16_384;

    require!(
        size >= MIN_WINK_WTP_SIZE,
        prefix.len() >= HEADER_LEN,
        starts(prefix, WINK_WTP_V1_MAGIC)
            || starts(prefix, WINK_WTP_V2_MAGIC)
            || starts(prefix, WINK_WTP_V3_MAGIC)
            || starts(prefix, WINK_WTP_V4_MAGIC)
    );
    let flags = le32(prefix, FLAGS_OFF)?;
    let outer_w = le32(prefix, OUTER_W_OFF)?;
    let outer_h = le32(prefix, OUTER_H_OFF)?;
    let inner_w = le32(prefix, INNER_W_OFF)?;
    let inner_h = le32(prefix, INNER_H_OFF)?;

    require!(
        (flags >> 16) == 0,
        outer_w <= MAX_DIM,
        outer_h <= MAX_DIM,
        inner_w <= MAX_DIM,
        inner_h <= MAX_DIM
    );
    if (flags & FLAG_HAS_BG_IMAGE) != 0 {
        require!(outer_w >= 1, outer_h >= 1);
        let bg_bytes = (outer_w as u64).saturating_mul(outer_h as u64).saturating_mul(3);
        require!(size >= MIN_WINK_WTP_SIZE.saturating_add(bg_bytes));
    }

    Some(Label::WinkWtp)
}

/// Windows Recorded TV: header GUID `{2000D8B7-4937-11DA-A64E-0007E95EAD8D}` at `@0`.
pub(crate) fn verify_wtv(prefix: &[u8], size: u64) -> Option<Label> {
    const WTV_HEADER_GUID: &[u8] = hex2bin!("B7 D8 00 20 37 49 DA 11 A6 4E 00 07 E9 5E AD 8D"); // @0, 16 bytes
    require!(size >= MIN_WTV_SIZE, starts(prefix, WTV_HEADER_GUID));
    Some(Label::Wtv)
}
