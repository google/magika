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

//! Audio and music format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// Sun/NeXT (`.snd`, big-endian) or DEC (`dns.`, little-endian) AU audio file: 24-byte header,
/// data offset ≥ 24, defined encoding code (1..=14 or 16..=27), nonzero sample rate and channels.
pub(crate) fn au(prefix: &[u8]) -> Option<Label> {
    static AU_BE: Subformat = Subformat::new("Sun/NeXT AU (big-endian)");
    static AU_LE: Subformat = Subformat::new("DEC AU (little-endian)");
    const BIG_MAGIC: &[u8] = b".snd"; // @0..4: big-endian AU magic
    const LITTLE_MAGIC: &[u8] = b"dns."; // @0..4: little-endian AU magic
    const MIN_LEN: usize = 24; // 6 × u32 header
    const DATA_OFFSET: usize = 4; // @4: u32 — byte offset to audio data (≥ 24)
    const ENCODING: usize = 12; // @12: u32 — sample encoding format (1..=14 or 16..=27)
    const SAMPLE_RATE: usize = 16; // @16: u32 — sample rate in Hz (> 0)
    const CHANNELS: usize = 20; // @20: u32 — number of channels (> 0)

    type ReadU32 = fn(&[u8], usize) -> Option<u32>;
    require!(prefix.len() >= MIN_LEN);
    let (read_u32, sub): (ReadU32, &Subformat) = if starts(prefix, BIG_MAGIC) {
        (be32, &AU_BE)
    } else if starts(prefix, LITTLE_MAGIC) {
        (le32, &AU_LE)
    } else {
        return None;
    };
    let enc = read_u32(prefix, ENCODING)?;
    require!(
        read_u32(prefix, DATA_OFFSET)? >= MIN_LEN as u32,
        (1..=14).contains(&enc) || (16..=27).contains(&enc),
        read_u32(prefix, SAMPLE_RATE)? > 0,
        read_u32(prefix, CHANNELS)? > 0,
    );
    with_subformat(Label::Au, sub)
}

/// Standard MIDI File (SMF): `"MThd"` chunk of length 6, valid format (`0` with 1 track, or
/// `1..=2` with ≥ 1 track), valid time division, and first `"MTrk"` track chunk of length ≥ 4.
pub(crate) fn midi(prefix: &[u8]) -> Option<Label> {
    static MIDI_FMT0: Subformat = Subformat::new("Standard MIDI (format 0)");
    static MIDI_FMT1: Subformat = Subformat::new("Standard MIDI (format 1)");
    static MIDI_FMT2: Subformat = Subformat::new("Standard MIDI (format 2)");
    const HEADER: &[u8] = bytes!(b"MThd", &6u32.to_be_bytes()); // @0..8: MThd tag + 6-byte header length
    const FORMAT: usize = 8; // @8: u16 BE — MIDI format (0, 1, or 2)
    const TRACKS: usize = 10; // @10: u16 BE — number of tracks
    const DIVISION: usize = 12; // @12: u16 BE — ticks per quarter-note or SMPTE timecode
    const MTRK_TAG: usize = 14; // @14..18: first track "MTrk" chunk magic
    const MTRK_MAGIC: &[u8] = b"MTrk";
    const MTRK_LEN: usize = 18; // @18: u32 BE — first track length (≥ 4)
    const MIN_LEN: usize = 26;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, HEADER),
        eq_at(prefix, MTRK_TAG, MTRK_MAGIC),
        be32(prefix, MTRK_LEN)? >= 4,
    );
    let (fmt, trk, div) = (be16(prefix, FORMAT)?, be16(prefix, TRACKS)?, be16(prefix, DIVISION)?);
    let valid_fmt = (fmt == 0 && trk == 1) || ((1..=2).contains(&fmt) && trk >= 1);
    let valid_div = (1..=32767).contains(&div)
        || (matches!(u8_at(prefix, 12)?, 226 | 227 | 231 | 232) && u8_at(prefix, 13)? >= 1);
    require!(valid_fmt, valid_div);
    let sub = match fmt {
        0 => &MIDI_FMT0,
        1 => &MIDI_FMT1,
        2 => &MIDI_FMT2,
        _ => unreachable!(),
    };
    with_subformat(Label::Midi, sub)
}

/// Free Lossless Audio Codec (FLAC): `"fLaC"` magic followed by a 34-byte `STREAMINFO` block
/// (type 0, optionally last block `0x80`), block sizes ≥ 16 at `@8` and `@10`, and sample rate /
/// channel word ≥ 4096 at `@18`.
pub(crate) fn flac(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"fLaC"; // @0..4: FLAC stream marker
    const BLOCK_HDR_TYPE: usize = 4; // @4: u8 — 0x00 or 0x80 (last-block flag + STREAMINFO type 0)
    const BLOCK_HDR_LEN: &[u8] = &[0x00, 0x00, 0x22]; // @5..8: u24 BE — STREAMINFO length = 34
    const MIN_BLOCK: usize = 8; // @8: u16 BE — minimum block size in samples (≥ 16)
    const MAX_BLOCK: usize = 10; // @10: u16 BE — maximum block size in samples (≥ 16)
    const RATE_CH_BPS: usize = 18; // @18: u32 BE — sample rate (20 bits) + channels + bps (≥ 4096)
    const MIN_LEN: usize = 42; // 4-byte magic + 4-byte metadata header + 34-byte STREAMINFO

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, BLOCK_HDR_TYPE)?, 0x00 | 0x80),
        eq_at(prefix, 5, BLOCK_HDR_LEN),
        be16(prefix, MIN_BLOCK)? >= 16,
        be16(prefix, MAX_BLOCK)? >= 16,
        be32(prefix, RATE_CH_BPS)? >= 4096,
    );
    Some(Label::Flac)
}

/// Ogg bitstream (`"OggS"`): version 0, BOS flag (`0x02`, optionally EOS `0x06`) at `@5`, initial
/// page sequence number `0` at `@18`, and ≥ 1 segment at `@26`.
pub(crate) fn ogg(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"OggS\x00"; // @0..5: Ogg capture pattern + stream_structure_version 0
    const HEADER_TYPE: usize = 5; // @5: u8 — BOS (0x02) or BOS|EOS (0x06)
    const PAGE_SEQ_NO: usize = 18; // @18: u32 LE — initial page sequence number (0)
    const PAGE_SEGMENTS: usize = 26; // @26: u8 — number of segment table entries (≥ 1)
    const MIN_LEN: usize = 28; // 27-byte Ogg page header + ≥ 1 segment table byte

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, HEADER_TYPE)?, 0x02 | 0x06),
        le32(prefix, PAGE_SEQ_NO)? == 0,
        u8_at(prefix, PAGE_SEGMENTS)? >= 1,
    );
    Some(Label::Ogg)
}

/// ID3v2-tagged MPEG Audio (`mp3`, 1998): `"ID3"` at `@0`, version `2..=4` at `@3`,
/// minor `< 0xFF` at `@4`, flags `(b & 0x0F) == 0` at `@5`, and 4 synchsafe size bytes
/// (`< 0x80`) at `@6..10`.
pub(crate) fn id3_mp3(prefix: &[u8], size: u64) -> Option<Label> {
    static MP3_ID3V2_2: Subformat = Subformat::new("MP3 (ID3v2.2)");
    static MP3_ID3V2_3: Subformat = Subformat::new("MP3 (ID3v2.3)");
    static MP3_ID3V2_4: Subformat = Subformat::new("MP3 (ID3v2.4)");
    const ID3V2_MAGIC: &[u8; 3] = b"ID3";

    require!(size >= 14, prefix.len() >= 10, starts(prefix, ID3V2_MAGIC));
    let sub = match u8_at(prefix, 3)? {
        2 => &MP3_ID3V2_2,
        3 => &MP3_ID3V2_3,
        4 => &MP3_ID3V2_4,
        _ => return None,
    };
    require!(
        u8_at(prefix, 4)? < 0xFF,
        (u8_at(prefix, 5)? & 0x0F) == 0,
        ((u8_at(prefix, 6)? | u8_at(prefix, 7)? | u8_at(prefix, 8)? | u8_at(prefix, 9)?) & 0x80)
            == 0,
    );
    with_subformat(Label::Mp3, sub)
}

/// Verifies at least two consecutive valid MPEG-1/2/2.5 Layer I/II/III audio frame headers.
pub(crate) fn verify_mpeg_audio_frames(buf: &[u8], start: usize) -> bool {
    const BITRATES_V1_L3: [u32; 16] =
        [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0];
    const BITRATES_V2_L3: [u32; 16] =
        [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0];
    const SRATES_V1: [u32; 4] = [44100, 48000, 32000, 0];
    const SRATES_V2: [u32; 4] = [22050, 24000, 16000, 0];
    const SRATES_V25: [u32; 4] = [11025, 12000, 8000, 0];

    if buf.len() < start + 4 {
        return false;
    }
    let Some(hdr) = be32(buf, start) else { return false };
    if (hdr & 0xFFE0_0000) != 0xFFE0_0000 {
        return false;
    }
    let ver_bits = ((hdr >> 19) & 0x03) as u8;
    let layer_bits = ((hdr >> 17) & 0x03) as u8;
    let br_idx = ((hdr >> 12) & 0x0F) as usize;
    let sr_idx = ((hdr >> 10) & 0x03) as usize;
    let padding = (hdr >> 9) & 0x01;
    if ver_bits == 1 || layer_bits != 1 || br_idx == 0 || br_idx == 15 || sr_idx == 3 {
        return false;
    }
    let (bitrate, srate, coeff) = match ver_bits {
        3 => (BITRATES_V1_L3[br_idx] * 1000, SRATES_V1[sr_idx], 144u32),
        2 => (BITRATES_V2_L3[br_idx] * 1000, SRATES_V2[sr_idx], 72u32),
        0 => (BITRATES_V2_L3[br_idx] * 1000, SRATES_V25[sr_idx], 72u32),
        _ => return false,
    };
    if bitrate == 0 || srate == 0 {
        return false;
    }
    let frame_len = ((coeff * bitrate) / srate + padding) as usize;
    if frame_len < 24 || start + frame_len + 2 > buf.len() {
        return false;
    }
    let Some(next_sync) = be16(buf, start + frame_len) else { return false };
    (next_sync & 0xFFE0) == 0xFFE0
}

/// Verifies at least two consecutive valid MPEG-1/2/2.5 Layer II (`mp2`) audio frame headers.
pub(crate) fn verify_mpeg_audio_layer2(buf: &[u8], start: usize) -> bool {
    const BITRATES_V1_L2: [u32; 16] =
        [0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 0];
    const BITRATES_V2_L2: [u32; 16] =
        [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0];
    const SRATES_V1: [u32; 4] = [44100, 48000, 32000, 0];
    const SRATES_V2: [u32; 4] = [22050, 24000, 16000, 0];
    const SRATES_V25: [u32; 4] = [11025, 12000, 8000, 0];

    if buf.len() < start + 4 {
        return false;
    }
    let Some(hdr) = be32(buf, start) else { return false };
    if (hdr & 0xFFE0_0000) != 0xFFE0_0000 {
        return false;
    }
    let ver_bits = ((hdr >> 19) & 0x03) as u8;
    let layer_bits = ((hdr >> 17) & 0x03) as u8;
    let br_idx = ((hdr >> 12) & 0x0F) as usize;
    let sr_idx = ((hdr >> 10) & 0x03) as usize;
    let padding = (hdr >> 9) & 0x01;
    if ver_bits == 1 || layer_bits != 2 || br_idx == 0 || br_idx == 15 || sr_idx == 3 {
        return false;
    }
    let (bitrate, srate, coeff) = match ver_bits {
        3 => (BITRATES_V1_L2[br_idx] * 1000, SRATES_V1[sr_idx], 144u32),
        2 => (BITRATES_V2_L2[br_idx] * 1000, SRATES_V2[sr_idx], 144u32),
        0 => (BITRATES_V2_L2[br_idx] * 1000, SRATES_V25[sr_idx], 144u32),
        _ => return false,
    };
    if bitrate == 0 || srate == 0 {
        return false;
    }
    let frame_len = ((coeff * bitrate) / srate + padding) as usize;
    if frame_len < 24 || start + frame_len + 4 > buf.len() {
        return false;
    }
    let Some(next_hdr) = be32(buf, start + frame_len) else { return false };
    let next_br = ((next_hdr >> 12) & 0x0F) as usize;
    let next_sr = ((next_hdr >> 10) & 0x03) as usize;
    ((next_hdr >> 16) as u16 & 0xFFE6) == ((hdr >> 16) as u16 & 0xFFE6)
        && next_br != 0
        && next_br != 15
        && next_sr == sr_idx
}

/// Extended M3U / HLS Multimedia Playlist (`m3u`, 1997): `"#EXTM3U"` header (with optional UTF-8 BOM).
pub(crate) fn m3u(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_M3U_SIZE: u64 = 12; // "#EXTM3U\na.mp3"
    const EXTM3U_LF: &[u8] = b"#EXTM3U\n";
    const EXTM3U_CRLF: &[u8] = b"#EXTM3U\r\n";

    require!(size >= MIN_M3U_SIZE, prefix.len() >= 12);
    let off = if starts(prefix, UTF8_BOM) { 3 } else { 0 };
    require!(
        eq_at(prefix, off, EXTM3U_LF) || eq_at(prefix, off, EXTM3U_CRLF),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::M3u)
}

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const ANDERS_OLAND_MAGIC: &[u8; 4] = b"mpl1";
const BEAVER_SWEEPER_MAGIC: &[u8; 4] = b"+SNT";
const BEROTRACKER_BRT_MAGIC: &[u8; 4] = b"BRTF";
const BETHESDA_FUZ_AUDIO_SIZE_OFFSET: usize = 12;
const BETHESDA_FUZ_LIP_SIZE_OFFSET: usize = 8;
const BETHESDA_FUZ_MAGIC: &[u8; 8] = bytes!(b"FUZE", &[0x01, 0x00, 0x00, 0x00]);
const BUZZIC2_MAGIC: &[u8; 4] = b"buz2";
const BWSB_GDM_MAGIC: &[u8; 4] = b"GDM\xFE";
const BWSB_GMFS_SECONDARY_MAGIC: &[u8; 7] = b"\r\n\x1AGMFS";
const CBA_MODULE_MAGIC: &[u8; 4] = bytes!(b"CBA", &[0xF9]);
const COMPOSER_667_MAGIC: &[u8; 2] = b"gf";
const CUBIC_MXM_MAGIC: &[u8; 4] = b"MXM\x00";
const DARKWAVE_DWP_MAGIC: &[u8; 4] = b" PWD";
const DAVE_LOWE_DLN_PREFIX: &[u8; 4] = hex2bin!("00 04 00 48");
const DIGITAL_FM_MAGIC: &[u8; 4] = bytes!(b"DFM", &[0x1A]);
const DIRECTWAVE_DWP_MAGIC: &[u8; 4] = b"DwPr";
const DIRECTWAVE_HDR_LEN_OFFSET: usize = 12;
const DIRECTWAVE_RESERVED_OFFSET: usize = 16;
const DIRECTWAVE_SUBVER_OFFSET: usize = 8;
const DIRECTWAVE_VERSION_OFFSET: usize = 4;
const DIRK_BIALLUCH_MSMP_MAGIC: &[u8; 4] = b"MSMP";
const DIRK_BIALLUCH_MSNG_MAGIC: &[u8; 4] = b"MSNG";
const DRUMTRAKER_DTL_MAGIC: &[u8; 4] = b"DTL\x00";
const DRUMWARE_K1_MAGIC: &[u8; 30] = hex2bin!(
    "00 4B 00 31 00 49 00 44 00 52 00 55 00 4D 00 57 00 41 00 52 00 45 00 49 00 6E 00 63 00 20"
);
const DYNAMIC_STUDIO_DSM_MAGIC: &[u8; 4] = bytes!(b"DSm", &[0x1A]);
const EASE_GLL_MAGIC: &[u8; 18] =
    bytes!(b"EGLL", &[0x00, 0x00, 0x00, 0x00, 0x08, 0x00], b"EASE_GLL");
const EASE_GLL_VERSION_OFFSET: usize = 18;
const EDLIB_EDL_MAGIC: &[u8; 4] = hex2bin!("00 06 FE FD");
const EMS_V1_MAGIC_22: &[u8; 2] = hex2bin!("01 22");
const EMS_V1_MAGIC_25: &[u8; 2] = hex2bin!("01 25");
const ENTIS_CONTAINER_MAGIC: &[u8; 8] = bytes!(b"Entis", &[0x1A, 0x00, 0x00]);
const ENTIS_HEADER_CHUNK: &[u8; 8] = b"Header  ";
const ENTIS_MIO_DESC: &[u8; 44] = b"Music Interleaved and Orthogonal transformed";
const EXOTIC_XAD_MAGIC: &[u8; 4] = b"XAD!";
const FMP_M_MAGIC: &[u8; 22] =
    bytes!(b"MUSIC DRV V1.0", &[0x00, 0x00, 0x03, 0x01, 0x00, 0x00, 0x00, 0x00]);
const FM_KINGTRACKER_MAGIC: &[u8; 4] = b"FMK!";
const FUNKTRACKER_MAGIC: &[u8; 4] = b"Funk";
const GAMEBOY_GBR_MAGIC: &[u8; 4] = b"GBRF";
const GPX_UNCOMP_LEN_OFFSET: usize = 4;
const JAYTRAX_JXS_V0_MAGIC: &[u8; 4] = le32_bytes!(0x0000_0D80);
const JAYTRAX_JXS_V1_MAGIC: &[u8; 4] = le32_bytes!(0x0000_0D81);
const KONTAKT_NCW_BITS_OFFSET: usize = 10;
const KONTAKT_NCW_BLOCKS_OFF_OFFSET: usize = 20;
const KONTAKT_NCW_CHANNELS_OFFSET: usize = 8;
const KONTAKT_NCW_FLAGS_OFFSET: usize = 6;
const KONTAKT_NCW_MAGIC: &[u8; 4] = hex2bin!("01 A8 9E D6");
const KONTAKT_NCW_RATE_OFFSET: usize = 12;
const KONTAKT_NCW_SAMPLES_OFFSET: usize = 16;
const LPAC_MAGIC: &[u8; 4] = b"LPAC";
const MARK_COOKSEY_PREFIX: &[u8; 4] = hex2bin!("D0 40 D0 40");
const MIN_ABLETON_0649_ASD_SIZE: u64 = 64;
const MIN_ABLETON_AMXD_SIZE: u64 = 36;
const MIN_AC1D_PACKER_SIZE: u64 = 64;
const MIN_ACE_TRACKER_SIZE: u64 = 64;
const MIN_ACTIONAMICS_AST_SIZE: u64 = 128;
const MIN_ADLIB_A2M_SIZE: u64 = 64;
const MIN_ADLIB_BAM_SIZE: u64 = 16;
const MIN_ADLIB_CMF_SIZE: u64 = 40;
const MIN_ADLIB_DRO_SIZE: u64 = 20;
const MIN_ADLIB_RAD_SIZE: u64 = 32;
const MIN_ADLIB_RAW_SIZE: u64 = 16;
const MIN_ADLIB_ROL_SIZE: u64 = 128;
const MIN_ADLIB_SBI_SIZE: u64 = 47;
const MIN_ALL_SOUND_TRACKER_SIZE: u64 = 128;
const MIN_AMIGA_AHX_SIZE: u64 = 32;
const MIN_AMIGA_DBM_SIZE: u64 = 64;
const MIN_AMIGA_FC_SIZE: u64 = 100;
const MIN_AMIGA_MCMD_SIZE: u64 = 32;
const MIN_AMIGA_OKT_SIZE: u64 = 64;
const MIN_AMPLESOUND_UMSK_SIZE: u64 = 32;
const MIN_AMX_SYNTH_SIZE: u64 = 64;
const MIN_AM_COMPOSER_SIZE: u64 = 64;
const MIN_ANDERS_OLAND_SIZE: u64 = 128;
const MIN_AON_FM_SIZE: u64 = 24;
const MIN_APROSYS_APS_SIZE: u64 = 64;
const MIN_ARGONAUT_ASF_SIZE: u64 = 28;
const MIN_ART_OF_NOISE_SIZE: u64 = 48;
const MIN_ATARI_SAP_SIZE: u64 = 32;
const MIN_ATARI_YMST_SIZE: u64 = 32;
const MIN_ATARI_YM_SIZE: u64 = 32;
const MIN_AUDFPRINT_SIZE: u64 = 32;
const MIN_AUDIBLE_AA_SIZE: u64 = 64;
const MIN_AURORADSP_STATE_SIZE: u64 = 36;
const MIN_AVR_AUDIO_SIZE: u64 = 128;
const MIN_BANDAI_NAMCO_NUS3_SIZE: u64 = 32;
const MIN_BEATMAKER_INST_BLOB_SIZE: u64 = 64;
const MIN_BEAVER_SWEEPER_SIZE: u64 = 64;
const MIN_BEEPOLA_SIZE: u64 = 32;
const MIN_BEROTRACKER_BRT_SIZE: u64 = 32;
const MIN_BETHESDA_FUZ_SIZE: u64 = 32;
const MIN_BITWIG_PRESET_SIZE: u64 = 54;
const MIN_BOOMTRACKER_SIZE: u64 = 64;
const MIN_BOYSCOUT_BSF_SIZE: u64 = 1_024;
const MIN_BRSAR_SIZE: u64 = 64;
const MIN_BUZZIC2_SIZE: u64 = 64;
const MIN_BWSB_GDM_SIZE: u64 = 157;
const MIN_CAKEWALK_WRK_SIZE: u64 = 32;
const MIN_CBA_MODULE_SIZE: u64 = 64;
const MIN_CDFM_670_SIZE: u64 = 64;
const MIN_CHIPTRACKER_KRIS_SIZE: u64 = 1_982;
const MIN_COMPOSER_667_SIZE: u64 = 1_400;
const MIN_COOLEDIT_SIZE: u64 = 16;
const MIN_CRIWARE_AFS2_SIZE: u64 = 32;
const MIN_CRI_HCA_SIZE: u64 = 32;
const MIN_CRYO_APC_SIZE: u64 = 32;
const MIN_CUBIC_MXM_SIZE: u64 = 128;
const MIN_CYBERTRACKER_C64_SIZE: u64 = 64;
const MIN_DARKWAVE_DWP_SIZE: u64 = 64;
const MIN_DAVE_LOWE_DLN_SIZE: u64 = 128;
const MIN_DAVID_HANNEY_SIZE: u64 = 32;
const MIN_DAVID_WHITTAKER_SIZE: u64 = 64;
const MIN_DAWESOME_ABYSS2_SIZE: u64 = 28;
const MIN_DAWESOME_KULT_SIZE: u64 = 28;
const MIN_DEFY_TRACKER_SIZE: u64 = 64;
const MIN_DESKTOP_TRACKER_SIZE: u64 = 128;
const MIN_DIGITAL_FM_SIZE: u64 = 64;
const MIN_DIGITAL_MUGICIAN_SIZE: u64 = 128;
const MIN_DIGITAL_SONIX_DSC_SIZE: u64 = 64;
const MIN_DIGITAL_TRACKER_DTM_SIZE: u64 = 64;
const MIN_DIGITRAKKER_MDL_SIZE: u64 = 32;
const MIN_DIRECTWAVE_DWP_SIZE: u64 = 32;
const MIN_DIRK_BIALLUCH_SIZE: u64 = 64;
const MIN_DISORDER_PLM_SIZE: u64 = 96;
const MIN_DREAMSTATION_DSS_SIZE: u64 = 32;
const MIN_DRUMTRAKER_DTL_SIZE: u64 = 64;
const MIN_DRUMWARE_K1_SIZE: u64 = 64;
const MIN_DSDIFF_AUDIO_SIZE: u64 = 28;
const MIN_DSF_AUDIO_SIZE: u64 = 92;
const MIN_DSIK_DSM_SIZE: u64 = 64;
const MIN_DSMI_AMF_SIZE: u64 = 74;
const MIN_DSYM_SIZE: u64 = 32;
const MIN_DYNAMIC_STUDIO_DSM_SIZE: u64 = 128;
const MIN_EARACHE_SIZE: u64 = 32;
const MIN_EASE_GLL_SIZE: u64 = 32;
const MIN_EASYTRAX_SIZE: u64 = 32;
const MIN_EA_BNK_SIZE: u64 = 32;
const MIN_EA_SCHL_SIZE: u64 = 32;
const MIN_EDLIB_D00_SIZE: u64 = 128;
const MIN_EDLIB_EDL_SIZE: u64 = 64;
const MIN_EMS_V1_SIZE: u64 = 320;
const MIN_EMS_V6_SIZE: u64 = 128;
const MIN_ENCORE_ENC_SIZE: u64 = 32;
const MIN_ENSONIQ_PARIS_SIZE: u64 = 28;
const MIN_ENTIS_MIO_SIZE: u64 = 128;
const MIN_EPIC_PSM_SIZE: u64 = 76;
const MIN_EXOTIC_XAD_SIZE: u64 = 84;
const MIN_EXTENDED_MOD_EMD_SIZE: u64 = 64;
const MIN_EXTREMES_AMS_SIZE: u64 = 32;
const MIN_FACE_THE_MUSIC_SIZE: u64 = 82;
const MIN_FAMITRACKER_SIZE: u64 = 64;
const MIN_FARBRAUSCH_V2M_SIZE: u64 = 128;
const MIN_FASTTRACKER2_XM_SIZE: u64 = 60;
const MIN_FAUST_FMC_SIZE: u64 = 64;
const MIN_FERNANDEZ_ADLIB_SIZE: u64 = 20;
const MIN_FLMOBILE_FLMRACK_SIZE: u64 = 40;
const MIN_FL_STUDIO_FLP_SIZE: u64 = 22;
const MIN_FMOD_DESIGNER_FEV_SIZE: u64 = 32;
const MIN_FMP_M_SIZE: u64 = 32;
const MIN_FMTOWNS_EUPHONY_PMB_SIZE: u64 = 520;
const MIN_FM_KINGTRACKER_SIZE: u64 = 128;
const MIN_FM_TOWNS_EUPHONY_SIZE: u64 = 2060;
const MIN_FM_TRACKER_SIZE: u64 = 64;
const MIN_FUNKTRACKER_SIZE: u64 = 128;
const MIN_FUTURE_COMPOSER_BSI_SIZE: u64 = 64;
const MIN_GALAXY_MUSE_SIZE: u64 = 32;
const MIN_GAMEBOY_GBR_SIZE: u64 = 64;
const MIN_GAMEBOY_GBS_SIZE: u64 = 112;
const MIN_GAME_MUSIC_CREATOR_SIZE: u64 = 1_468;
const MIN_GENH_AUDIO_SIZE: u64 = 64;
const MIN_GLUEMON_SIZE: u64 = 64;
const MIN_GOATTRACKER_INS_SIZE: u64 = 16;
const MIN_GOATTRACKER_SNG_SIZE: u64 = 105;
const MIN_GRAOUMF_GT2_SIZE: u64 = 64;
const MIN_GT_GAME_SYSTEMS_DUX_SIZE: u64 = 64;
const MIN_GUITAR_PRO_GPX_SIZE: u64 = 32;
const MIN_GUS_PATCH_SIZE: u64 = 239;
const MIN_HALPST_AUDIO_SIZE: u64 = 64;
const MIN_HERAD_ADLIB_SIZE: u64 = 64;
const MIN_HES_AUDIO_SIZE: u64 = 36;
const MIN_HIPPEL_7V_SIZE: u64 = 64;
const MIN_HIPPEL_COSO_SIZE: u64 = 64;
const MIN_HIS_MASTERS_NOISE_SIZE: u64 = 1_084;
const MIN_HIVELY_TRACKER_SIZE: u64 = 32;
const MIN_HOLY_NOISE_SIZE: u64 = 32;
const MIN_HSC_TRACKER_SIZE: u64 = 2_739;
const MIN_IMAGES_MUSIC_SYSTEM_SIZE: u64 = 1_852;
const MIN_IMAGO_IMF_SIZE: u64 = 128;
const MIN_IMUSE_MCMP_SIZE: u64 = 24;
const MIN_INFOGRAMES_INS_SIZE: u64 = 48;
const MIN_INTERPLAY_ACM_SIZE: u64 = 16;
const MIN_IN_STEREO_SIZE: u64 = 64;
const MIN_IONE_SOPROL_SIZE: u64 = 64;
const MIN_IT_SIZE: u64 = 192;
const MIN_IXALANCE_SIZE: u64 = 32;
const MIN_JAMCRACKER_SIZE: u64 = 32;
const MIN_JASON_PAGE_OLD_SIZE: u64 = 128;
const MIN_JAYTRAX_JXS_SIZE: u64 = 3_456;
const MIN_JEROEN_TEL_JT_SIZE: u64 = 64;
const MIN_JESKOLA_BUZZ_SIZE: u64 = 32;
const MIN_JUCE_VC2_PRESET_SIZE: u64 = 24;
const MIN_KENS_ADLIB_KSM_SIZE: u64 = 86;
const MIN_KENS_DIGITAL_KDM_SIZE: u64 = 64;
const MIN_KLYSTRACK_SIZE: u64 = 32;
const MIN_KOEI_TECMO_XWS_SIZE: u64 = 48;
const MIN_KONAMI_SCC_MUSIXX_SIZE: u64 = 64;
const MIN_KONTAKT_HSIN_NKI_SIZE: u64 = 64;
const MIN_KONTAKT_NCW_SIZE: u64 = 128;
const MIN_KSS_AUDIO_SIZE: u64 = 20;
const MIN_KUGOU_MUSIC_KGM_SIZE: u64 = 64;
const MIN_KVAG_AUDIO_SIZE: u64 = 16;
const MIN_LABGRUPPEN_IPXPRESET_SIZE: u64 = 28;
const MIN_LAXITY_PT_SIZE: u64 = 128;
const MIN_LA_LOSSLESS_AUDIO_SIZE: u64 = 52;
const MIN_LEGGLESS_LME_SIZE: u64 = 64;
const MIN_LEGO_ALP_AUDIO_SIZE: u64 = 16;
const MIN_LIONHEART_LION_SIZE: u64 = 64;
const MIN_LIQUID_TRACKER_SIZE: u64 = 96;
const MIN_LORICIEL_BNKS_SIZE: u64 = 128;
const MIN_LPAC_SIZE: u64 = 58;
const MIN_LUCASARTS_LAA_SIZE: u64 = 16;
const MIN_LUCASARTS_SOU_SIZE: u64 = 32;
const MIN_LUXONIX_PURITY_PTH_SIZE: u64 = 44;
const MIN_MADTRACKER2_SIZE: u64 = 256;
const MIN_MARK2_SIZE: u64 = 64;
const MIN_MARK_COOKSEY_SIZE: u64 = 128;
const MIN_MASTERTRACKER_MTR_SIZE: u64 = 64;
const MIN_MASTER_TRACKS_MTS_SIZE: u64 = 64;
const MIN_MAXTRAX_SIZE: u64 = 32;
const MIN_MEDLEY_MSO_SIZE: u64 = 32;
const MIN_MEGASTATION_SIZE: u64 = 32;
const MIN_MEGATRACKER_MGT_SIZE: u64 = 64;
const MIN_MIKMOD_UNI_SIZE: u64 = 32;
const MIN_MKJAMZ_SIZE: u64 = 32;
const MIN_MLAT_MAD_SIZE: u64 = 32;
const MIN_MO3_SIZE: u64 = 32;
const MIN_MOBILE_XMF_SIZE: u64 = 16;
const MIN_MONOTONE_SIZE: u64 = 128;
const MIN_MOONBLASTER_SIZE: u64 = 360;
const MIN_MPU401_MTK_SIZE: u64 = 32;
const MIN_MSX_FAC_SOUNDTRACKER_SIZE: u64 = 64;
const MIN_MSX_MBMS_SIZE: u64 = 80;
const MIN_MSX_MGS_SIZE: u64 = 32;
const MIN_MSX_MPK_SIZE: u64 = 32;
const MIN_MSX_OPLLDRV_SIZE: u64 = 32;
const MIN_MTC_CHIPTUNE_SIZE: u64 = 32;
const MIN_MUSICAL_ENLIGHTENMENT_TME_SIZE: u64 = 512;
const MIN_MUSICLINE_SIZE: u64 = 64;
const MIN_MUSIC_STUDIO_SNG_SIZE: u64 = 32;
const MIN_MVS_TRACKER_SIZE: u64 = 32;
const MIN_MVX_MVM_SIZE: u64 = 32;
const MIN_NAMX_PROFILE_SIZE: u64 = 28;
const MIN_NERDTRACKER2_SIZE: u64 = 64;
const MIN_NES_NSF_SIZE: u64 = 28;
const MIN_NINTENDO_AST_SIZE: u64 = 96;
const MIN_NINTENDO_BRSTM_SIZE: u64 = 68;
const MIN_NINTENDO_BRWAV_SIZE: u64 = 64;
const MIN_NINTENDO_BWAV_SIZE: u64 = 92;
const MIN_NIST_SPHERE_SIZE: u64 = 1024;
const MIN_NOTATOR_SONG_HEADER_SIZE: u64 = 16;
const MIN_NTSP_SYSTEM_SIZE: u64 = 64;
const MIN_OBARSKI_OSP_SIZE: u64 = 64;
const MIN_ONYX_OMF_SIZE: u64 = 32;
const MIN_OPTIMFROG_SIZE: u64 = 20;
const MIN_ORGANYA_SIZE: u64 = 114;
const MIN_PALLADIX_PCS_SIZE: u64 = 32;
const MIN_PAUL_ROBOTHAM_DAT_SIZE: u64 = 1_364;
const MIN_PC98_PVI_SIZE: u64 = 528;
const MIN_PCMS_CCOL_WK_SIZE: u64 = 64;
const MIN_PIAPRO_NTDB_SIZE: u64 = 28;
const MIN_PIERRE_ADANE_PAP_SIZE: u64 = 64;
const MIN_PIXEL_PAINTERS_FMF_SIZE: u64 = 36;
const MIN_PIXEL_PAINTERS_FTF_SIZE: u64 = 64;
const MIN_PLAYERPRO_MADG_SIZE: u64 = 64;
const MIN_PMD_M_SIZE: u64 = 64;
const MIN_POLYTRACKER_SIZE: u64 = 608;
const MIN_PRESET_SAP_SIZE: u64 = 32;
const MIN_PRETRACKER_SIZE: u64 = 64;
const MIN_PRONOM_SONY_OMA_SIZE: u64 = 24;
const MIN_PROTOOLS_SIZE: u64 = 16;
const MIN_PROTREKKR_SIZE: u64 = 64;
const MIN_PRO_SOUND_CREATOR_SIZE: u64 = 64;
const MIN_PS2_HD_SIZE: u64 = 80;
const MIN_PSA_MODULE_SIZE: u64 = 64;
const MIN_PSX_SPU_DUMP_SIZE: u64 = 64;
const MIN_PSYCLE_SIZE: u64 = 32;
const MIN_PUMATRACKER_SIZE: u64 = 112;
const MIN_PXTONE_SIZE: u64 = 16;
const MIN_QUARTET_SET_SIZE: u64 = 256;
const MIN_QUARTET_ST_SIZE: u64 = 64;
const MIN_RADICAL_RSD_SIZE: u64 = 128;
const MIN_RAD_BINK_AUDIO_SIZE: u64 = 24;
const MIN_RAGE_AWC_SIZE: u64 = 32;
const MIN_RAILWORKS_DAV_SIZE: u64 = 44;
const MIN_RAMTRACKER_TRK_SIZE: u64 = 32;
const MIN_REAL_TRACKER_RTM_SIZE: u64 = 96;
const MIN_RECOMPOSER_RCP_SIZE: u64 = 64;
const MIN_RENOISE_OLD_SIZE: u64 = 64;
const MIN_RICHARD_JOSEPH_SIZE: u64 = 32;
const MIN_RK_AUDIO_SIZE: u64 = 16;
const MIN_ROBOPLAY_PLY_SIZE: u64 = 28;
const MIN_ROB_HUBBARD_RHO_SIZE: u64 = 256;
const MIN_ROLAND_SVQ_SIZE: u64 = 32;
const MIN_S3M_SIZE: u64 = 96;
const MIN_S98_AUDIO_SIZE: u64 = 32;
const MIN_SAM_COUPE_COP_SIZE: u64 = 64;
const MIN_SBSTUDIO_PAC_SIZE: u64 = 32;
const MIN_SC68_MUSIC_SIZE: u64 = 72;
const MIN_SCUMM_AMIGA_SIZE: u64 = 64;
const MIN_SEAN_CONRAN_SCR_SIZE: u64 = 128;
const MIN_SEGA_GENESIS_GYM_SIZE: u64 = 428;
const MIN_SEGA_SGC_SIZE: u64 = 160;
const MIN_SHORTEN_SIZE: u64 = 12;
const MIN_SHROOM_SIZE: u64 = 32;
const MIN_SIDMON2_SIZE: u64 = 128;
const MIN_SIERRA_SOL_SIZE: u64 = 16;
const MIN_SNES_SPC700_SIZE: u64 = 65984;
const MIN_SNEVENTRACKER_SNM_SIZE: u64 = 64;
const MIN_SNGPLAY_OBSM_SIZE: u64 = 32;
const MIN_SONARC_PCM_SIZE: u64 = 40;
const MIN_SONARC_SIZE: u64 = 32;
const MIN_SONIX_INSTR_SIZE: u64 = 84;
const MIN_SONY_OMA_BARE_SIZE: u64 = 96;
const MIN_SONY_SGXD_SIZE: u64 = 32;
const MIN_SONY_WAVE64_SIZE: u64 = 40;
const MIN_SOUNDFACTORY_PSF_SIZE: u64 = 280;
const MIN_SOUNDPLAYER_SJS_SIZE: u64 = 128;
const MIN_SOUNDSMITH_SIZE: u64 = 512;
const MIN_SOUNDTRACKER_26_SIZE: u64 = 1_468;
const MIN_SOUND_CLUB_SIZE: u64 = 32;
const MIN_SOUND_CONTROL_SC_SIZE: u64 = 64;
const MIN_SOUND_IMAGES_SIG_SIZE: u64 = 48;
const MIN_SOX_AUDIO_SIZE: u64 = 32;
const MIN_SQEX_AKB_SIZE: u64 = 32;
const MIN_SQUARE_ENIX_SCD_SIZE: u64 = 64;
const MIN_SQUARE_ENIX_SPW_SIZE: u64 = 48;
const MIN_STARKOS_SIZE: u64 = 32;
const MIN_STARTREKKER_BANK_SIZE: u64 = 264;
const MIN_STONETRACKER_SIZE: u64 = 64;
const MIN_STONETRACKER_SPS_SIZE: u64 = 64;
const MIN_SUNVOX_SIZE: u64 = 24;
const MIN_SYMPHONIE_SIZE: u64 = 24;
const MIN_SYNDER_SNG_SIZE: u64 = 32;
const MIN_SYNTH_DREAM_SDR_SIZE: u64 = 64;
const MIN_TFMX_MOD_SIZE: u64 = 64;
const MIN_TFM_MUSIC_MAKER_SIZE: u64 = 32;
const MIN_TOMOFON_TOMODEL_SIZE: u64 = 64;
const MIN_TOMOFON_TOPATCH_SIZE: u64 = 24;
const MIN_TOMY_TRACKER_SG_SIZE: u64 = 128;
const MIN_TRACKERPACKER_SIZE: u64 = 32;
const MIN_TSOUNDSYSTEM_SIZE: u64 = 64;
const MIN_TUNEFISH2_TF2M_SIZE: u64 = 72;
const MIN_TWINVQ_SIZE: u64 = 32;
const MIN_TWIN_TRACK_DMO_SIZE: u64 = 128;
const MIN_ULTIMA6_M_SIZE: u64 = 48;
const MIN_ULTRATRACKER_SIZE: u64 = 64;
const MIN_UNIQUE_DEV_SIZE: u64 = 64;
const MIN_UTAU_FRQ_SIZE: u64 = 40;
const MIN_VAG_SIZE: u64 = 64;
const MIN_VGM_SIZE: u64 = 64;
const MIN_VIO2_SCORE_SIZE: u64 = 36;
const MIN_VIRTUALDJ_VDJSAMPLE_SIZE: u64 = 36;
const MIN_VOODOO_VSS_SIZE: u64 = 64;
const MIN_VOYETRA_BANK_SIZE: u64 = 102;
const MIN_VST2_FXP_SIZE: u64 = 28;
const MIN_VST3_VSTPRESET_SIZE: u64 = 64;
const MIN_WITCHER3_SPEECH_SIZE: u64 = 24;
const MIN_WWISE_BNK_SIZE: u64 = 24;
const MIN_X68000_MDX_SIZE: u64 = 48;
const MIN_XACT_WAVE_BANK_SIZE: u64 = 52;
const MIN_XACT_XGS_SIZE: u64 = 77;
const MIN_XACT_XSB_SIZE: u64 = 52;
const MIN_XBOX_SDNS_SIZE: u64 = 32;
const MIN_XILAM_DERF_SIZE: u64 = 16;
const MIN_XSM_MUSIC_SIZE: u64 = 32;
const MIN_XTRACKER_DMF_SIZE: u64 = 66;
const MIN_XVAG_AUDIO_SIZE: u64 = 64;
const MIN_ZX_AY_ST13_SIZE: u64 = 64;
const MIN_ZX_CHIP_SIZE: u64 = 128;
const MIN_ZX_FTC_SIZE: u64 = 128;
const MIN_ZX_FXSM_SIZE: u64 = 64;
const MIN_ZX_GTR_SIZE: u64 = 64;
const MIN_ZX_SQT_SIZE: u64 = 128;
const MIN_ZX_STC_SIZE: u64 = 27;
const MIN_ZX_STP_SIZE: u64 = 64;
const MIN_ZX_VORTEX_VTX_SIZE: u64 = 32;
const NOTATOR_RESERVED_ZERO_PAD: &[u8; 8] = &[0u8; 8];
const PS2_HD_HEAD_MAGIC: &[u8; 12] = bytes!(b"IECSdaeH", &[0x40, 0x00, 0x00, 0x00]);
const PS2_HD_HEAD_OFFSET: usize = 16;
const PS2_HD_TOTAL_SIZE_OFFSET: usize = 28;
const PS2_HD_VERS_MAGIC: &[u8; 12] = bytes!(b"IECSsreV", &[0x10, 0x00, 0x00, 0x00]);
const RAGE_AWC_DATA_OFF_OFFSET: usize = 12;
const RAGE_AWC_FLAGS_HI_OFFSET: usize = 7;
const RAGE_AWC_MAGIC: &[u8; 6] = bytes!(b"ADAT", &[0x01, 0x00]);
const RAGE_AWC_STREAMS_OFFSET: usize = 8;
const RK_AUDIO_MAGIC: &[u8; 4] = b"RKA7";
const SEAN_CONRAN_SCR_V1_MAGIC: &[u8; 4] = hex2bin!("0F FF 0F E2");
const SEAN_CONRAN_SCR_V2_MAGIC: &[u8; 4] = hex2bin!("0F 1C 0F 0E");
const SIERRA_SCI_PATCH_MAGIC: &[u8; 2] = hex2bin!("89 00");
const SNGPLAY_OBSM_MAGIC: &[u8; 4] = b"ObsM";
const SONARC_PCM_MAGIC: &[u8; 26] = bytes!(b"Sonarc-squeezed PCM file.", &[0x1A]);
const SONARC_VOC_MAGIC: &[u8; 20] = bytes!(b"Sonarc-squeezed VOC", &[0x1A]);
const SONY_OMA_BARE_PREFIX: &[u8; 3] = b"EA3";
const SONY_OMA_TAGGED_PREFIX: &[u8; 3] = b"ea3";
const STONETRACKER_SPS_MAGIC: &[u8; 4] = bytes!(b"SPS", &[2]);
const VOODOO_VSS_MAGIC: &[u8; 4] = b"HERE";
const VOYETRA_BANK_MAGIC_D50: &[u8; 2] = hex2bin!("E0 05");
const VOYETRA_BANK_MAGIC_STD: &[u8; 2] = hex2bin!("DF 05");
const VST2_BYTE_SIZE_OFFSET: usize = 4;
const VST2_CCNK_MAGIC: &[u8; 4] = b"CcnK";
const VST2_FX_ID_OFFSET: usize = 16;
const VST2_VERSION_OFFSET: usize = 12;
const VST3_CLASS_ID_RANGE: core::ops::Range<usize> = 8..40;
const VST3_LIST_OFFSET_OFFSET: usize = 40;
const VST3_PRESET_MAGIC: &[u8; 8] = bytes!(b"VST3", &[0x01, 0x00, 0x00, 0x00]);
const WITCHER3_SPEECH_MAGIC: &[u8; 4] = b"CPSW";
const WITCHER3_SPEECH_META_MAGIC: &[u8; 4] = b"TEMS";
const ZX_FXSM_MAGIC: &[u8; 4] = b"FXSM";
const ZX_VTX_AY_MAGIC: &[u8; 2] = b"ay";
const ZX_VTX_YM_MAGIC: &[u8; 2] = b"ym";

fn match_jeroen_tel_jt_at(prefix: &[u8], off: usize) -> bool {
    prefix.len() >= off + 12
        && eq_at(prefix, off, hex2bin!("02 39 00 01 00"))
        && u8_at(prefix, off + 8) == Some(0x66)
        && u8_at(prefix, off + 9).is_some_and(|b| b >= 2 && b % 2 == 0)
        && eq_at(prefix, off + 10, hex2bin!("4E 75"))
}

/// Ableton Live Audio Analysis & Warp Sidecar (`.asd`, `06 49` + stepped peak offsets).
pub(crate) fn verify_ableton_0649_asd(prefix: &[u8], size: u64) -> Option<Label> {
    const ABLETON_ASD_MAGIC: &[u8; 2] = hex2bin!("06 49"); // @0..2

    require!(
        size >= MIN_ABLETON_0649_ASD_SIZE,
        prefix.len() >= 26,
        starts(prefix, ABLETON_ASD_MAGIC),
        le16(prefix, 2)? >= 1, // @2..4: sample/frame rate version token
        get_slice(prefix, 4, 10)? == [0u8; 6]
    );
    let o1 = le32(prefix, 10)?;
    let o2 = le32(prefix, 14)?;
    let o3 = le32(prefix, 18)?;
    let o4 = le32(prefix, 22)?;
    require!(
        o1 > 0 && o1 < o2 && o2 < o3 && o3 < o4 && o4 <= 0x0100_0000,
        o2 - o1 < 0x10_0000,
        o3 - o2 < 0x10_0000,
        o4 - o3 < 0x10_0000
    );
    Some(Label::AbletonAsd)
}

/// Ableton Live / Cycling '74 Max for Live Device (.amxd, 2009) Ableton & Cycling '74.
pub(crate) fn verify_ableton_amxd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ABLETON_AMXD_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"ampf\x04\0\0\0mmmmmeta\x04\0\0\0\x04\0\0\0ptch")
    );
    Some(Label::AbletonAmxd)
}

/// Anarchy Amiga AC1D Packer Module (`.ac1d`): `@2..4 == 0xAC1D` (`be16`), `@0` subsong/status `< 0x20`.
pub(crate) fn verify_ac1d_packer(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AC1D_PACKER_SIZE,
        prefix.len() >= 16,
        u8_at(prefix, 0)? < 0x20,
        be16(prefix, 2) == Some(0xAC1D),
        (be32(prefix, 4)? as u64) < size
    );
    Some(Label::Ac1dPacker)
}

/// Amiga Ace Tracker Module (`.am`): `"AM01\0\0\0\x7F"` at `@0..8`.
pub(crate) fn verify_ace_tracker_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ACE_TRACKER_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"AM01\x00\x00\x00\x7F")
    );
    Some(Label::AmAcetracker)
}

/// Actionamics Amiga Sound Tool Module (`.ast`): `"ACTIONAMICS SOUND TOOL"` at `@62..84`.
pub(crate) fn verify_actionamics_ast(prefix: &[u8], size: u64) -> Option<Label> {
    const BANNER_OFFSET: usize = 62; // @62: "ACTIONAMICS SOUND TOOL"
    require!(
        size >= MIN_ACTIONAMICS_AST_SIZE,
        prefix.len() >= 84,
        eq_at(prefix, BANNER_OFFSET, b"ACTIONAMICS SOUND TOOL")
    );
    Some(Label::ActionamicsAst)
}

/// Subz3ro AdLib Tracker II Module (`.a2m`, 2000 Subz3ro).
pub(crate) fn verify_adlib_a2m(prefix: &[u8], size: u64) -> Option<Label> {
    const A2M_MAGIC: &[u8; 10] = b"_A2module_"; // @0..10: AdLib Tracker II signature
    const FFVER_OFF: usize = 14; // @14: u8 file format version (1..=14)
    const NPAT_OFF: usize = 15; // @15: u8 number of patterns (1..=128)
    require!(
        size >= MIN_ADLIB_A2M_SIZE,
        prefix.len() >= 18,
        starts(prefix, A2M_MAGIC),
        (1..=14).contains(&u8_at(prefix, FFVER_OFF)?),
        (1..=128).contains(&u8_at(prefix, NPAT_OFF)?)
    );
    Some(Label::AdlibA2m)
}

/// Creative Labs AdLib / Sound Blaster Creative Music File (.cmf).
pub(crate) fn verify_adlib_cmf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ADLIB_CMF_SIZE && prefix.len() >= 20 && starts(prefix, b"CTMF"));
    let version = le16(prefix, 4)?;
    let inst_off = le16(prefix, 6)? as u64;
    let music_off = le16(prefix, 8)? as u64;
    require!(
        matches!(version, 0x0100 | 0x0101 | 0x0110)
            && inst_off >= 20
            && inst_off < size
            && music_off >= 20
            && music_off < size
    );
    Some(Label::AdlibCmf)
}

/// Creative Labs Sound Blaster CMF `"A.H."` variant (`.cmf`): `"A.H."` at `@0..4` + `u16 BE` pattern table (`<= 0x007F`) terminated by `0x0063` within `@4..64`.
pub(crate) fn verify_adlib_cmf_ah(prefix: &[u8], size: u64) -> Option<Label> {
    const AH_LEGACY_HDR: &[u8] = b"A.H.\x00\x00\x00\x00\x00\x01\x00\x02\x00\x03\x00\x04"; // @0: 16B sequential order header
    require!(size >= MIN_ADLIB_CMF_SIZE, prefix.len() >= 16, starts(prefix, b"A.H."));
    if starts(prefix, AH_LEGACY_HDR) {
        return Some(Label::AdlibCmf);
    }
    require!(prefix.len() >= 64);
    let mut found_end = false;
    let mut off = 4usize;
    while off + 2 <= 64 {
        let w = be16(prefix, off)?;
        if found_end {
            require!(w == 0);
        } else if w == 0x0063 {
            found_end = true;
        } else {
            require!(w <= 0x007F);
        }
        off += 2;
    }
    require!(found_end);
    Some(Label::AdlibCmf)
}

/// DOSBox AdLib OPL2/OPL3 Raw Capture (.dro).
pub(crate) fn verify_adlib_dro(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ADLIB_DRO_SIZE && prefix.len() >= 17 && starts(prefix, b"DBRAWOPL"));
    let is_v1_v2 = le16(prefix, 8).unwrap_or(99) <= 2 && le16(prefix, 10).unwrap_or(99) <= 2;
    let is_v0_1 = le32(prefix, 8).unwrap_or(0) > 0
        && le32(prefix, 12).unwrap_or(0) > 0
        && u8_at(prefix, 16)? <= 2;
    require!(is_v1_v2 || is_v0_1);
    Some(Label::AdlibDro)
}

/// Reality AdLib Tracker OPL2/OPL3 Module (.rad).
pub(crate) fn verify_adlib_rad(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ADLIB_RAD_SIZE
            && prefix.len() >= 18
            && starts(prefix, b"RAD by REALiTY!!")
            && matches!(u8_at(prefix, 16)?, 0x10 | 0x20 | 0x21)
            && get_slice(prefix, 16, prefix.len().min(32))?
                .iter()
                .any(|&b| (b < 0x20 && !matches!(b, b'\t' | b'\n' | b'\r')) || b >= 0x7F)
    );
    Some(Label::AdlibRad)
}

/// RdosPlay AdLib OPL2 Raw Register Capture (.raw).
pub(crate) fn verify_adlib_raw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ADLIB_RAW_SIZE
            && prefix.len() >= 16
            && starts(prefix, b"RAWADATA")
            && le16(prefix, 8).unwrap_or(0) > 0
            && has_non_ascii_text_byte(prefix.get(8..16)?)
    );
    Some(Label::AdlibRaw)
}

/// AdLib Visual Composer FM Song (`.rol`): `@0..4 == 00 00 04 00` (version 0.4) + `"\\roll\\default"` at `@4..17`.
pub(crate) fn verify_adlib_rol(prefix: &[u8], size: u64) -> Option<Label> {
    const COMMENT_OFFSET: usize = 4; // @4..44: 40-byte NUL-padded comment (often "\\roll\\default")
    require!(
        size >= MIN_ADLIB_ROL_SIZE,
        prefix.len() >= 48,
        starts(prefix, hex2bin!("00 00 04 00"))
    );
    let is_default_comment = eq_at(prefix, COMMENT_OFFSET, b"\\roll\\default");
    let is_valid_grid = prefix.len() >= 53
        && get_slice(prefix, 4, 44)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b))
        && u8_at(prefix, 43)? == 0
        && (1..=60).contains(&le16(prefix, 44).unwrap_or(0))
        && (1..=32).contains(&le16(prefix, 46).unwrap_or(0))
        && (1..=256).contains(&le16(prefix, 48).unwrap_or(0))
        && (1..=256).contains(&le16(prefix, 50).unwrap_or(0))
        && u8_at(prefix, 52)? <= 1;
    require!(is_default_comment || is_valid_grid);
    Some(Label::AdlibRol)
}

/// Creative Labs Sound Blaster FM Instrument Patch (.sbi).
pub(crate) fn verify_adlib_sbi(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ADLIB_SBI_SIZE
            && prefix.len() >= 47
            && (starts(prefix, b"SBI\x1A")
                || starts(prefix, b"2OP\x1A")
                || starts(prefix, b"4OP\x1A"))
            && prefix.get(4..34)?.contains(&0)
            && get_slice(prefix, 4, 34)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b))
    );
    Some(Label::AdlibSbi)
}

/// All Sound Tracker Module (`.ast`, 1993).
pub(crate) fn verify_all_sound_tracker(prefix: &[u8], size: u64) -> Option<Label> {
    const AST_PREFIX: &[u8; 7] = b"\x08AST 00"; // @0..7: Pascal String[8] "AST 00xx"
    require!(
        size >= MIN_ALL_SOUND_TRACKER_SIZE,
        prefix.len() >= 16,
        starts(prefix, AST_PREFIX),
        u8_at(prefix, 7)?.is_ascii_digit(),
        u8_at(prefix, 8)?.is_ascii_digit(),
        u8_at(prefix, 9)? == 0x04
    );
    Some(Label::AllSoundTracker)
}

/// Amiga A.M. Composer 1.2 Module (`.amc`, 1990).
pub(crate) fn verify_am_composer(prefix: &[u8], size: u64) -> Option<Label> {
    const AMC_MAGIC: &[u8; 16] = b"AMC V1.2 REPLAY!"; // @0..16: A.M. Composer 1.2 replay header
    require!(
        size >= MIN_AM_COMPOSER_SIZE,
        prefix.len() >= 24,
        starts(prefix, AMC_MAGIC),
        prefix.get(16..24)?.contains(&0)
    );
    Some(Label::AmComposer)
}

/// Amiga Abyss Highest Experience Chiptune Synthesizer Module (.ahx / .thx).
pub(crate) fn verify_amiga_ahx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AMIGA_AHX_SIZE
            && prefix.len() >= 14
            && (starts(prefix, b"THX\x00")
                || starts(prefix, b"THX\x01")
                || starts(prefix, b"AHX\x00")
                || starts(prefix, b"AHX\x01"))
    );
    let title_off = be16(prefix, 4)? as u64;
    let track_len = u8_at(prefix, 10)?;
    require!(title_off >= 14 && title_off < size && (1..=64).contains(&track_len));
    Some(Label::AmigaAhx)
}

/// Amiga DigiBooster / DigiBooster Pro Tracker Module (.dbm / .digi).
pub(crate) fn verify_amiga_dbm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_AMIGA_DBM_SIZE && prefix.len() >= 20);
    if starts(prefix, b"DBM0") {
        let trk_ver = be16(prefix, 4)?;
        require!(
            (0x0200..=0x0399).contains(&trk_ver)
                && (eq_at(prefix, 8, b"NAME") || eq_at(prefix, 8, b"INFO"))
        );
        return Some(Label::AmigaDbm);
    }
    if starts(prefix, b"DIGI Booster module\0") {
        return Some(Label::AmigaDbm);
    }
    None
}

/// Amiga Future Composer v1.0-v1.4 Synth Module (.fc / .fc13 / .fc14 / .smod).
pub(crate) fn verify_amiga_fc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AMIGA_FC_SIZE
            && prefix.len() >= 40
            && (starts(prefix, b"SMOD") || starts(prefix, b"FC14"))
    );
    let seq_len = be32(prefix, 4)? as u64;
    let pat_off = be32(prefix, 8)? as u64;
    let pat_len = be32(prefix, 12)? as u64;
    require!(
        seq_len > 0
            && (seq_len.is_multiple_of(13) || (seq_len % 13 == 1 && seq_len.is_multiple_of(2)))
            && seq_len < size
            && pat_off >= 40
            && pat_off.saturating_add(pat_len) <= size
    );
    Some(Label::AmigaFc)
}

/// Amiga MCMD Chiptune Module (`.mcmd`): `"MCMD\0"` at `@0..5`.
pub(crate) fn verify_amiga_mcmd_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AMIGA_MCMD_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"MCMD\x00"),
        be16(prefix, 6)? > 0
    );
    Some(Label::Mcmd)
}

/// Amiga Oktalyzer 8-Channel Tracker Module (.okt).
pub(crate) fn verify_amiga_okt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AMIGA_OKT_SIZE
            && prefix.len() >= 24
            && starts(prefix, b"OKTASONG")
            && eq_at(prefix, 8, b"CMOD")
            && be32(prefix, 12) == Some(8)
    );
    Some(Label::AmigaOkt)
}

/// Ample Sound Guitar/Bass Keyswitch Articulation Map (.umsk) Ample Sound.
pub(crate) fn verify_amplesound_umsk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AMPLESOUND_UMSK_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"TBTS"),
        le32(prefix, 8) == Some(100),
        get_slice(prefix, 12, 24)? == b"Unnamed\0KEYS"
    );
    Some(Label::AmplesoundUmsk)
}

/// And X Synth Module (`.amx`, 1996).
pub(crate) fn verify_amx_synth(prefix: &[u8], size: u64) -> Option<Label> {
    const AMX_MAGIC: &[u8; 6] = b"AMX \x11\x1A"; // @0..6: AMX synth header
    require!(size >= MIN_AMX_SYNTH_SIZE, prefix.len() >= 16, starts(prefix, AMX_MAGIC));
    Some(Label::AmxSynth)
}

/// Anders 0land Amiga Music Module (`.hot`, 1990 Anders 0land).
pub(crate) fn verify_anders_oland(prefix: &[u8], size: u64) -> Option<Label> {
    const BANNER: &[u8; 20] = b"-=*> Music & Player "; // @18..38: embedded player banner
    require!(
        size >= MIN_ANDERS_OLAND_SIZE,
        prefix.len() >= 40,
        starts(prefix, ANDERS_OLAND_MAGIC),
        be16(prefix, 4) == Some(0),
        eq_at(prefix, 18, BANNER)
    );
    Some(Label::AndersOland)
}

/// Art of Noise FM Instrument Definition file (`.fm`, 1995 Bastian Spiegel).
pub(crate) fn verify_aon_fm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AON_FM_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"AON-FM1") && matches!(u8_at(prefix, 7), Some(b'0'..=b'9')),
        has_non_ascii_text_byte(prefix.get(8..24)?),
        get_slice(prefix, 8, 24)?.iter().any(|&b| b != 0)
    );
    Some(Label::AonFm)
}

/// Apple Core Audio Format container (`.caf`).
pub(crate) fn verify_apple_caf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 8, prefix.len() >= 8, starts(prefix, b"caff\x00\x01\x00\x00"));
    Some(Label::Caf)
}

/// Petter Urstad AProSys Amiga Synth Module (`.aps`, 1990 Petter Urstad).
pub(crate) fn verify_aprosys_aps(prefix: &[u8], size: u64) -> Option<Label> {
    const APROSYS_MAGIC: &[u8; 8] = b"ADRVPACK"; // @0..8: AProSys packed module signature
    require!(
        size >= MIN_APROSYS_APS_SIZE,
        prefix.len() >= 16,
        starts(prefix, APROSYS_MAGIC),
        (1..=256).contains(&be32(prefix, 8)?)
    );
    Some(Label::AprosysAps)
}

/// Argonaut Games ASF Audio (`.asf`, `"ASF\0"`).
pub(crate) fn verify_argonaut_asf_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ARGONAUT_ASF_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"ASF\0"),
        matches!(le16(prefix, 4), Some(1 | 2)) && le16(prefix, 6)? <= 2,
        (1..=65535).contains(&le32(prefix, 8)?)
    );
    Some(Label::ArgonautAsf)
}

/// Bastian Spiegel Art Of Noise Amiga Module (`"AON4"` / `"AON8"`).
pub(crate) fn verify_art_of_noise_module(prefix: &[u8], size: u64) -> Option<Label> {
    const BANNER_OFFSET: usize = 4; // @4: "NAME" or 42-byte "artofnoise by bastian spiegel (twice/lego)"
    const CHUNK_AFTER_BANNER: usize = 46; // @46: "NAME" / "AUTH" / "DATE" / "RMRK" chunk after 42B banner
    require!(
        size >= MIN_ART_OF_NOISE_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"AON4") || starts(prefix, b"AON8")
    );
    let valid_direct = eq_at(prefix, BANNER_OFFSET, b"NAME") && prefix.get(8..12)?.contains(&0);
    let valid_banner = prefix.len() >= 54
        && eq_at(prefix, BANNER_OFFSET, b"artofnoise by ")
        && (eq_at(prefix, CHUNK_AFTER_BANNER, b"NAME")
            || eq_at(prefix, CHUNK_AFTER_BANNER, b"AUTH")
            || eq_at(prefix, CHUNK_AFTER_BANNER, b"DATE")
            || eq_at(prefix, CHUNK_AFTER_BANNER, b"RMRK"))
        && prefix.get(50..54)?.contains(&0);
    require!(valid_direct || valid_banner);
    Some(Label::ArtOfNoiseAon)
}

/// Atari 8-Bit Slight Atari Player POKEY Chiptune (.sap).
pub(crate) fn verify_atari_sap(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ATARI_SAP_SIZE
            && prefix.len() >= 24
            && (starts(prefix, b"SAP\r\n") || starts(prefix, b"SAP\n"))
    );
    let scan = get_slice(prefix, 0, prefix.len().min(512))?;
    require!(
        find_pos(scan, b"AUTHOR ").is_some()
            || find_pos(scan, b"NAME ").is_some()
            || find_pos(scan, b"TYPE ").is_some()
    );
    Some(Label::AtariSap)
}

/// Atari ST / Amstrad CPC YM2149 Sound Chip Music (.ym).
pub(crate) fn verify_atari_ym(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ATARI_YM_SIZE && prefix.len() >= 32);
    if matches!(get_slice(prefix, 0, 4)?, b"YM4!" | b"YM5!" | b"YM6!" | b"YMT1" | b"YMT2") {
        require!(get_slice(prefix, 4, 12)? == b"LeOnArD!" && prefix.get(12..32)?.contains(&0));
    } else if matches!(get_slice(prefix, 0, 4)?, b"YM2!" | b"YM3!" | b"YM3b") {
        // YM2! / YM3! / YM3b raw 14-register VBL frame dumps
        require!(
            prefix.get(4..32)?.contains(&0)
                && ((size - 4).is_multiple_of(14)
                    || (starts(prefix, b"YM3b") && (size - 8).is_multiple_of(14)))
        );
    } else {
        return None;
    }
    Some(Label::AtariYm)
}

/// Atari ST YMST YM2149 Audio (`.ymst`): `"YMST"` at `@0..4` + `u16 BE` flags (`0x8000` or `0x0000`) at `@4..6`.
pub(crate) fn verify_atari_ymst_audio(prefix: &[u8], size: u64) -> Option<Label> {
    const FLAGS_OFFSET: usize = 4; // @4: u16 BE frame flags
    require!(
        size >= MIN_ATARI_YMST_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"YMST"),
        (be16(prefix, FLAGS_OFFSET)? & 0x7FFF) == 0
    );
    Some(Label::Ymst)
}

/// Audfprint: `"audfprinthash"` (hash table) or `"audfprintpeak"` (peak list) + `"V00"` version.
pub(crate) fn verify_audfprint(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AUDFPRINT_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"audfprinthash") || starts(prefix, b"audfprintpeak"),
        eq_at(prefix, 13, b"V00"),
        prefix.get(16..24)?.contains(&0)
    );
    Some(Label::Audfprint)
}

/// Audible legacy DRM audiobook container (`.aa`).
///
/// Header layout:
/// - `@0..4`: `u32 BE file_size` (total file size in bytes)
/// - `@4..8`: `0x57907536` BE (`57 90 75 36` Audible `.aa` magic number)
/// - `@8..12`: `u32 BE toc_count` (`1..=64` Table of Contents entries)
/// - `@12..16`: `u32 BE reserved` (`== 0`)
/// - `@16..16 + 12 * toc_count`: 12-byte TOC entries `(u32 BE chunk_type, u32 BE offset, u32 BE length)`.
pub(crate) fn verify_audible_aa_offset(prefix: &[u8], size: u64) -> Option<Label> {
    const AUDIBLE_AA_MAGIC_OFFSET: usize = 4; // @4..8: 0x57907536 BE magic
    const AUDIBLE_AA_MAGIC: &[u8; 4] = hex2bin!("57 90 75 36"); // @4..8
    const TOC_COUNT_OFFSET: usize = 8; // @8..12: u32 BE number of TOC entries (1..=64)
    const RESERVED_OFFSET: usize = 12; // @12..16: u32 BE reserved (0)
    const FIRST_TOC_TYPE_OFFSET: usize = 16; // @16..20: u32 BE first TOC entry chunk type (0..=32)

    require!(
        size >= MIN_AUDIBLE_AA_SIZE,
        prefix.len() >= 28,
        eq_at(prefix, AUDIBLE_AA_MAGIC_OFFSET, AUDIBLE_AA_MAGIC),
        be32(prefix, RESERVED_OFFSET) == Some(0)
    );
    let declared_size = be32(prefix, 0)? as u64;
    let toc_count = be32(prefix, TOC_COUNT_OFFSET)? as u64;
    let first_type = be32(prefix, FIRST_TOC_TYPE_OFFSET)?;
    require!(
        declared_size >= MIN_AUDIBLE_AA_SIZE,
        (1..=64).contains(&toc_count),
        16 + toc_count * 12 <= size,
        first_type <= 32
    );
    Some(Label::AudibleAa)
}

/// Sync Audio Sculpture Synth Instrument Bank (`.as`, 1991 Sync).
pub(crate) fn verify_audio_sculpture_as(prefix: &[u8], size: u64) -> Option<Label> {
    const AS_MAGIC: &[u8; 16] = b"AudioSculpture10"; // @0..16: Audio Sculpture 1.0 bank header
    const WORD_STRIDE: &[u8; 8] = hex2bin!("00 18 00 18 00 18 00 18"); // @16..24: 24-byte synth slot stride words
    require!(
        size >= MIN_STARTREKKER_BANK_SIZE,
        prefix.len() >= 32,
        starts(prefix, AS_MAGIC),
        eq_at(prefix, 16, WORD_STRIDE)
    );
    Some(Label::AudioSculptureAs)
}

/// Aurora DSP Audio Plugin State Preset (.afx / .lpa) Aurora DSP.
pub(crate) fn verify_auroradsp_state(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AURORADSP_STATE_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"Grain Scaper_state\0\x01\x01scale_fac_id\0")
            || starts(prefix, b"Menace Drive_state\0\x01\x01scale_fac_id\0")
            || ((1..=32).contains(&u8_at(prefix, 20).unwrap_or(0))
                && get_slice(prefix, 21, 34)? == b"scale_fac_id\0")
    );
    Some(Label::AuroradspState)
}

/// Atari ST Audio Visual Research Sound (`.avr`, `"2BIT"`).
pub(crate) fn verify_avr_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AVR_AUDIO_SIZE,
        prefix.len() >= 26,
        starts(prefix, b"2BIT"),
        matches!(be16(prefix, 12), Some(0x0000 | 0xFFFF)),
        matches!(be16(prefix, 14), Some(8 | 12 | 16)),
        matches!(be16(prefix, 16), Some(0x0000 | 0xFFFF))
    );
    Some(Label::AvrAudio)
}

/// Bandai Namco NUS3 Audio Container (`.nus3audio` / `.nus3bank`, 2014 Bandai Namco, `"NUS3"`).
///
/// Header layout:
/// - `@0..4`: `"NUS3"`
/// - `@4..8`: `u32 LE` `payload_size` (`>= 24`)
/// - `@8..16`: `"AUDIINDX"` (`.nus3audio`) with `le32(@16)` chunk size in `4..=64` and `le32(@20)` subsong count in `1..=65536`,
///   OR `"BANKTOC "` (`.nus3bank`) with `le32(@16)` TOC size in `8..=4096` and `le32(@20)` chunk count in `1..=64`
pub(crate) fn verify_bandai_namco_nus3(prefix: &[u8], size: u64) -> Option<Label> {
    const PAYLOAD_SIZE_OFF: usize = 4; // @4..8: u32 LE payload byte length (>= 24)
    const SUBTAG_OFF: usize = 8; // @8..16: "AUDIINDX" or "BANKTOC "
    const CHUNK_LEN_OFF: usize = 16; // @16..20: u32 LE index/TOC chunk byte length
    const COUNT_OFF: usize = 20; // @20..24: u32 LE subsong count (AUDIINDX) or section count (BANKTOC)

    require!(
        size >= MIN_BANDAI_NAMCO_NUS3_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"NUS3"),
        le32(prefix, PAYLOAD_SIZE_OFF)? >= 24
    );
    let chunk_len = le32(prefix, CHUNK_LEN_OFF)?;
    let count = le32(prefix, COUNT_OFF)?;
    let is_audio = eq_at(prefix, SUBTAG_OFF, b"AUDIINDX")
        && (4..=64).contains(&chunk_len)
        && (1..=65536).contains(&count);
    let is_bank = eq_at(prefix, SUBTAG_OFF, b"BANKTOC ")
        && (8..=4096).contains(&chunk_len)
        && (1..=64).contains(&count);
    require!(is_audio || is_bank);
    Some(Label::BandaiNamcoNus3)
}

/// UJAM Beatmaker Instrument Sample Bank (`.blob`, `67 45 B1 62` + `u32le json_len` + `{"instruments":[`).
pub(crate) fn verify_beatmaker_inst_blob(prefix: &[u8], size: u64) -> Option<Label> {
    const BEATMAKER_BLOB_MAGIC: &[u8; 4] = hex2bin!("67 45 B1 62"); // @0..4
    const INSTRUMENTS_JSON_PREFIX: &[u8; 16] = b"{\"instruments\":["; // @8..24

    require!(
        size >= MIN_BEATMAKER_INST_BLOB_SIZE,
        prefix.len() >= 24,
        starts(prefix, BEATMAKER_BLOB_MAGIC)
    );
    let json_len = le32(prefix, 4)? as u64;
    require!(
        (16..=0x0100_0000).contains(&json_len),
        8 + json_len <= size,
        eq_at(prefix, 8, INSTRUMENTS_JSON_PREFIX)
    );
    Some(Label::UjamBeatmakerBlob)
}

/// Beaver Sweeper Tracker Module (`.gtk`, 1992).
pub(crate) fn verify_beaver_sweeper(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BEAVER_SWEEPER_SIZE,
        prefix.len() >= 24,
        starts(prefix, BEAVER_SWEEPER_MAGIC),
        has_non_ascii_text_byte(prefix.get(4..24)?)
    );
    Some(Label::BeaverSweeper)
}

/// ZX Spectrum Beepola Beeper Song (`.bbsong`): `"BBSONG\0"` at `@0..7`.
pub(crate) fn verify_beepola_song(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BEEPOLA_SIZE, prefix.len() >= 16, starts(prefix, b"BBSONG\x00"));
    Some(Label::Bbsong)
}

/// Benjamin Rosseaux BeRoTracker Module (`.brt`, 2004 Benjamin Rosseaux).
pub(crate) fn verify_berotracker_brt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BEROTRACKER_BRT_SIZE,
        prefix.len() >= 16,
        starts(prefix, BEROTRACKER_BRT_MAGIC),
        le32(prefix, 4)? > 0,
        eq_at(prefix, 8, b"NAME")
    );
    Some(Label::BerotrackerBrt)
}

/// Bethesda Creation Engine FUZE Voice + Lip-Sync container (`.fuz`).
pub(crate) fn verify_bethesda_fuz(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BETHESDA_FUZ_SIZE,
        prefix.len() >= 16,
        starts(prefix, BETHESDA_FUZ_MAGIC),
        le32(prefix, BETHESDA_FUZ_LIP_SIZE_OFFSET)? <= 16_777_216,
        le32(prefix, BETHESDA_FUZ_AUDIO_SIZE_OFFSET)? >= 1
    );
    Some(Label::BethesdaFuz)
}

/// Bitwig Studio Preset, Device, or Modulator container (`"BtWg000"` + `"meta"`).
pub(crate) fn verify_bitwig_studio_preset(prefix: &[u8], size: u64) -> Option<Label> {
    const BTWG_MAGIC: &[u8; 7] = b"BtWg000"; // @0..7: Bitwig container signature + version prefix
    const HEX_FIELDS_RANGE: core::ops::Range<usize> = 7..40; // @7..40: ASCII hex header offsets
    const META_TAG_OFFSET_A: usize = 48; // @48..52: "meta" tag offset variant A
    const META_TAG_OFFSET_B: usize = 50; // @50..54: "meta" tag offset variant B

    require!(
        size >= MIN_BITWIG_PRESET_SIZE,
        prefix.len() >= 54,
        starts(prefix, BTWG_MAGIC),
        get_slice(prefix, HEX_FIELDS_RANGE.start, HEX_FIELDS_RANGE.end)?
            .iter()
            .all(|&b| b.is_ascii_hexdigit() || b == b' '),
        eq_at(prefix, META_TAG_OFFSET_A, b"meta") || eq_at(prefix, META_TAG_OFFSET_B, b"meta")
    );
    Some(Label::BitwigPreset)
}

/// Bob's AdLib Music Module (.bam).
pub(crate) fn verify_bobs_adlib_bam(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ADLIB_BAM_SIZE, prefix.len() >= 16, starts(prefix, b"CBMF"));
    // Binary OPL2 register/opcode bytes (< 0x20 or >= 0x80) follow the 4-byte signature.
    require!(has_non_ascii_text_byte(prefix.get(4..16)?));
    Some(Label::AdlibBam)
}

/// CUD BoomTracker 4.0 Module (`"<CUD-FM-File>"`).
pub(crate) fn verify_boomtracker_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BOOMTRACKER_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"<CUD-FM-File>"),
        has_non_ascii_text_byte(prefix.get(13..24)?)
    );
    Some(Label::BoomtrackerCff)
}

/// PID BoyScout Game Boy Chiptune Module (`.bsf`, 2001 PID).
pub(crate) fn verify_boyscout_bsf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BOYSCOUT_BSF_SIZE,
        prefix.len() >= 52,
        le32(prefix, 0) == Some(1),
        (1..=64).contains(&le32(prefix, 4)?),
        (1..=64).contains(&le32(prefix, 8)?),
        le32(prefix, 12) == Some(0),
        (1..=4096).contains(&le32(prefix, 16)?),
        (1..=16).contains(&le32(prefix, 20)?),
        (1..=16).contains(&le32(prefix, 24)?),
        eq_at(prefix, 28, &[0u8; 24])
    );
    Some(Label::BoyscoutBsf)
}

/// Rebels Buzzic 2 Synthesizer Module (`.buz2`, 2008 Rebels).
pub(crate) fn verify_buzzic2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BUZZIC2_SIZE,
        prefix.len() >= 16,
        starts(prefix, BUZZIC2_MAGIC),
        (1..=4).contains(&le16(prefix, 4)?),
        le16(prefix, 6)? <= 9
    );
    Some(Label::Buzzic2)
}

/// BWSB General Digital Music Tracker Module (`.gdm`, 1994 Edward Schlunder).
pub(crate) fn verify_bwsb_gdm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BWSB_GDM_SIZE
            && prefix.len() >= 77
            && starts(prefix, BWSB_GDM_MAGIC)
            && eq_at(prefix, 68, BWSB_GMFS_SECONDARY_MAGIC)
    );
    Some(Label::Gdm)
}

/// Twelve Tone Systems Cakewalk Project (`.wrk`): `"CAKEWALK\x1A"` at `@0..9`.
pub(crate) fn verify_cakewalk_wrk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_CAKEWALK_WRK_SIZE, prefix.len() >= 16, starts(prefix, b"CAKEWALK\x1A"));
    Some(Label::CakewalkWrk)
}

/// Chuck Biscuits / Black Artist Tracker Module (`.cba`, 1999).
pub(crate) fn verify_cba_module(prefix: &[u8], size: u64) -> Option<Label> {
    const EOF_OFF: usize = 36; // @36: 0x1A DOS EOF marker after 32B ASCII title
    require!(
        size >= MIN_CBA_MODULE_SIZE,
        prefix.len() >= 40,
        starts(prefix, CBA_MODULE_MAGIC),
        get_slice(prefix, 4, 36)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b)),
        u8_at(prefix, EOF_OFF)? == 0x1A
    );
    Some(Label::CbaModule)
}

/// Renaissance CDFM / Composer 670 Packed Module (`.670`, 1992 Renaissance).
pub(crate) fn verify_cdfm_670(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CDFM_670_SIZE,
        prefix.len() >= 32,
        (1..=15).contains(&u8_at(prefix, 0)?),  // @0: speed
        (1..=128).contains(&u8_at(prefix, 1)?), // @1: nord
        (1..=128).contains(&u8_at(prefix, 2)?), // @2: npat
        u8_at(prefix, 3)? <= u8_at(prefix, 1)?, // @3: loop_ord
        u8_at(prefix, 4)? <= 32,                // @4: num PCM samples
        u8_at(prefix, 5)? <= 32,                // @5: num FM instruments
        be16(prefix, 8) == Some(0)              // @8..10: reserved [0, 0]
    );
    let nord = u8_at(prefix, 1)? as usize;
    let npat = u8_at(prefix, 2)? as usize;
    let pat_sz = le16(prefix, 6)? as u64;
    let pat_tbl = 10 + nord;
    require!(
        pat_sz > 0 && pat_sz < size,
        prefix.len() >= pat_tbl + npat * 4,
        get_slice(prefix, 10, pat_tbl)?.iter().all(|&b| (b as usize) < npat)
    );
    let p0 = le32(prefix, pat_tbl)?;
    require!(p0 == 0);
    let mut prev = p0;
    for i in 1..npat {
        let p = le32(prefix, pat_tbl + i * 4)?;
        require!(p > prev, (p as u64) < pat_sz);
        prev = p;
    }
    Some(Label::Cdfm670)
}

/// Kris Hatlelid Chiptracker Amiga Module (`.mod` / `.kris`, 1990 Kris Hatlelid).
/// Layout: 22B title + `31 × 30`B sample headers (`vol <= 0x40`) + `"KRIS"` at `@952..956` + `song_len` (`1..=128`) at `@956`.
pub(crate) fn verify_chiptracker_kris(prefix: &[u8], size: u64) -> Option<Label> {
    const KRIS_TAG_OFF: usize = 952; // @952..956 (0x3B8): "KRIS"
    const KRIS_TAG: &[u8; 4] = b"KRIS";
    require!(
        size >= MIN_CHIPTRACKER_KRIS_SIZE,
        prefix.len() >= 958,
        eq_at(prefix, KRIS_TAG_OFF, KRIS_TAG),
        (1..=128).contains(&u8_at(prefix, 956)?),
        has_non_ascii_text_byte(prefix.get(..952)?)
    );
    for i in 0..31usize {
        let off = 22 + i * 30;
        require!(u8_at(prefix, off + 25)? <= 0x40);
    }
    Some(Label::ChiptrackerKris)
}

/// Future Crew Composer 667 OPL2 Module (`.667`, 1994 Future Crew).
pub(crate) fn verify_composer_667(prefix: &[u8], size: u64) -> Option<Label> {
    const SPEED_OFF: usize = 514; // @514: initial speed (1..=15)
    const ORDERS_OFF: usize = 515; // @515: order count (1..=128)
    const PAT_OFF_TBL: usize = 516; // @516..: strictly increasing u16 LE pattern offsets (first == 0)
    require!(
        (MIN_COMPOSER_667_SIZE..=65_536).contains(&size),
        prefix.len() >= 524,
        starts(prefix, COMPOSER_667_MAGIC),
        get_slice(prefix, 2, 514)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b)),
        (1..=15).contains(&u8_at(prefix, SPEED_OFF)?),
        (1..=128).contains(&u8_at(prefix, ORDERS_OFF)?),
        le16(prefix, PAT_OFF_TBL) == Some(0)
    );
    let p1 = le16(prefix, PAT_OFF_TBL + 2)? as u64;
    let p2 = le16(prefix, PAT_OFF_TBL + 4)? as u64;
    require!(p1 > 0 && p1 < p2 && p2 < size);
    Some(Label::Composer667)
}

/// Syntrillium Cool Edit / Adobe Audition Session (.ses).
pub(crate) fn verify_cooledit(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_COOLEDIT_SIZE
            && prefix.len() >= 16
            && starts(prefix, b"COOLNESS")
            && prefix.get(8..16)?.contains(&0)
    );
    Some(Label::Ses)
}

/// CRI Middleware High Compression Audio (.hca).
pub(crate) fn verify_cri_hca(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CRI_HCA_SIZE
            && prefix.len() >= 16
            && be16(prefix, 6).unwrap_or(0) >= 32
            && ((u8_at(prefix, 8)? & 0x7F) == b'f'
                && (u8_at(prefix, 9)? & 0x7F) == b'm'
                && (u8_at(prefix, 10)? & 0x7F) == b't'
                && u8_at(prefix, 11)? == 0)
    );
    Some(Label::CriHca)
}

/// CRI Middleware ADX2 AFS2 / AWB Streaming Audio Archive (`.awb`, `"AFS2"`, 2010 CRI Middleware).
pub(crate) fn verify_criware_afs2_awb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CRIWARE_AFS2_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"AFS2"),
        (1..=2).contains(&u8_at(prefix, 4)?),
        matches!(u8_at(prefix, 5)?, 2 | 4),
        matches!(u8_at(prefix, 6)?, 2 | 4),
        (1..=1_000_000).contains(&le32(prefix, 8)?),
        (16..=4096).contains(&le16(prefix, 12)?)
    );
    Some(Label::CriwareAwb)
}

/// Cryo Interactive IMA ADPCM Audio (`.apc`, `"CRYO_APC1.20"`).
pub(crate) fn verify_cryo_apc_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CRYO_APC_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"CRYO_APC1.20"),
        (4000..=96000).contains(&le32(prefix, 16)?)
    );
    Some(Label::CryoApc)
}

/// Cubic Team Tiny XM Module (`.mxm`, 1996 Niklas Beisert).
pub(crate) fn verify_cubic_mxm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CUBIC_MXM_SIZE,
        prefix.len() >= 48,
        starts(prefix, CUBIC_MXM_MAGIC),
        (1..=256).contains(&le32(prefix, 4)?),
        le32(prefix, 12)? <= 32,
        le32(prefix, 16)? <= 256,
        le32(prefix, 20)? <= 256
    );
    Some(Label::CubicMxm)
}

/// Commodore 64 Noname Cybertracker SID Module (`.ct`, 2001 Noname).
pub(crate) fn verify_cybertracker_c64(prefix: &[u8], size: u64) -> Option<Label> {
    const CT_C64_MAGIC: &[u8; 10] = b"\x00\x04NNTRKMZX"; // @0..10: PRG $0400 + "NNTRKMZX"
    require!(size >= MIN_CYBERTRACKER_C64_SIZE, prefix.len() >= 16, starts(prefix, CT_C64_MAGIC));
    Some(Label::CybertrackerC64)
}

/// ExperimentalScene DarkWave Studio Project (`.dwp`, 2010 ExperimentalScene).
pub(crate) fn verify_darkwave_dwp(prefix: &[u8], size: u64) -> Option<Label> {
    const MASTER_OFF: usize = 14; // @14..21: "Master\0" root machine name
    require!(
        size >= MIN_DARKWAVE_DWP_SIZE,
        prefix.len() >= 24,
        starts(prefix, DARKWAVE_DWP_MAGIC),
        (1..=10).contains(&le32(prefix, 4)?),
        eq_at(prefix, MASTER_OFF, b"Master\x00")
    );
    Some(Label::DarkwaveDwp)
}

/// Dave Lowe New Uncle Art Amiga Module (`.dln`, 1992 Dave Lowe).
pub(crate) fn verify_dave_lowe_dln(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DAVE_LOWE_DLN_SIZE,
        prefix.len() >= 56,
        starts(prefix, DAVE_LOWE_DLN_PREFIX),
        be32(prefix, 8) == Some(0x0000_0066),
        get_slice(prefix, 24, 39)?.iter().all(|&b| b == 0),
        be32(prefix, 40) == Some(0x0000_0048)
    );
    Some(Label::DaveLoweDln)
}

/// David Hanney Amiga Chiptune Module (`.dh`): `"DSNGSEQU"` at `@0..8`.
pub(crate) fn verify_david_hanney_dh(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_DAVID_HANNEY_SIZE, prefix.len() >= 16, starts(prefix, b"DSNGSEQU"));
    Some(Label::DavidHanneyDh)
}

/// David Whittaker Amiga 68000 Music Module (`.dw`): `48 E7 F1 FE 61 00` or `48 E7 F0 F0 42 40 47 FA`.
pub(crate) fn verify_david_whittaker_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DAVID_WHITTAKER_SIZE,
        prefix.len() >= 16,
        (starts(prefix, hex2bin!("48 E7 F1 FE 61 00"))
            && eq_at(prefix, 8, hex2bin!("4C DF 7F 8F 4E 75")))
            || starts(prefix, hex2bin!("48 E7 F0 F0 42 40 47 FA"))
    );
    Some(Label::DavidWhittaker)
}

/// Tracktion / Dawesome Abyss 2 Synthesizer Preset (.abyss2) Peter V / Dawesome.
pub(crate) fn verify_dawesome_abyss2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DAWESOME_ABYSS2_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"Abyss2_Plugin\0\x01\x02Q_TAGS\0")
    );
    Some(Label::DawesomeAbyss2)
}

/// Tracktion / Dawesome KULT Synthesizer Preset (.kult) Peter V / Dawesome.
pub(crate) fn verify_dawesome_kult(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DAWESOME_KULT_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"Kult_Plugin\0\0\x02K\x01PARAM\0")
    );
    Some(Label::DawesomeKult)
}

/// DeFy AdLib Tracker Module (`"DeFy DTM "` / `"DeFy DT "`).
pub(crate) fn verify_defy_tracker_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DEFY_TRACKER_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"DeFy DTM ") || starts(prefix, b"DeFy DT "),
        has_non_ascii_text_byte(prefix.get(8..20)?)
    );
    Some(Label::DefyDt)
}

/// Equivalence Acorn Archimedes Desktop Tracker Module (`.dtt`): `"DskT"` at `@0..4` + printable/null ASCII title at `@4..36`.
pub(crate) fn verify_desktop_tracker(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DESKTOP_TRACKER_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"DskT"),
        u8_at(prefix, 4)?.is_ascii_graphic(),
        has_non_ascii_text_byte(prefix.get(..36)?)
    );
    Some(Label::DesktopTracker)
}

/// Roel Vanhout Digital-FM AdLib Module (`.dfm`, 1994 Roel Vanhout).
pub(crate) fn verify_digital_fm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DIGITAL_FM_SIZE,
        prefix.len() >= 40,
        starts(prefix, DIGITAL_FM_MAGIC),
        u8_at(prefix, 4)? <= 2,
        u8_at(prefix, 6)? <= 32
    );
    Some(Label::DigitalFm)
}

/// Softeyes / Thalamus Amiga Digital Mugician Module (`.dmu`): `" MUGICIAN/SOFTEYES 1990 "` or `" MUGICIAN2/SOFTEYES 1990"` at `@0..24`.
pub(crate) fn verify_digital_mugician_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DIGITAL_MUGICIAN_SIZE,
        prefix.len() >= 24,
        starts(prefix, b" MUGICIAN/SOFTEYES 1990 ")
            || starts(prefix, b" MUGICIAN2/SOFTEYES 1990")
            || starts(prefix, b" MUGICIAN3/SOFTEYES 1990")
    );
    Some(Label::Dmu)
}

/// Digital Sonix & Chrome Amiga Module (`.dsc`, 1990 Digital Sonix & Chrome).
pub(crate) fn verify_digital_sonix_dsc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DIGITAL_SONIX_DSC_SIZE,
        prefix.len() >= 20,
        u8_at(prefix, 0)? == 0x01,
        (8..=96).contains(&u8_at(prefix, 1)?),
        u8_at(prefix, 2)? >= 1,
        u8_at(prefix, 3)? >= 1,
        (0x40..size).contains(&(be32(prefix, 4)? as u64)),
        be16(prefix, 8) == Some(0),
        be32(prefix, 12) == Some(0),
        u8_at(prefix, 16)? == 0x01,
        (1..=0x40).contains(&u8_at(prefix, 17)?)
    );
    Some(Label::DigitalSonixDsc)
}

/// Digital Symphony: fixed 8-byte magic `02 01 13 13 14 12 01 0B` (bytes of "BASSTRAK" in alphabet positions), version `0..=1`.
pub(crate) fn verify_digital_symphony(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = hex2bin!("02 01 13 13 14 12 01 0B"); // @0, 8 bytes
    const VERSION: usize = 8; // @8: u8
    require!(size >= MIN_DSYM_SIZE, starts(prefix, MAGIC), u8_at(prefix, VERSION)? <= 1);
    Some(Label::Dsym)
}

/// Softjee Atari Falcon Digital Tracker Module (`.dtm`): `"D.T.\0\0"` at `@0..6` + `u16 BE` header size (`0x18..=0x100`) at `@6..8`.
pub(crate) fn verify_digital_tracker_dtm(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_LEN_OFFSET: usize = 6; // @6: u16 BE header chunk length
    require!(
        size >= MIN_DIGITAL_TRACKER_DTM_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"D.T.\x00\x00"),
        (0x14..=0x200).contains(&be16(prefix, HDR_LEN_OFFSET)?)
    );
    Some(Label::Dtm)
}

/// Prodatron Digitrakker Module (`.mdl`): `"DMDL"` at `@0..4` + version (`0x00`, `0x10`, `0x11`) at `@4` + 2-char uppercase block ID (`"IN"`, `"ME"`, `"PA"`, `"TR"`, `"II"`, `"IS"`, `"SA"`) at `@5..7`.
pub(crate) fn verify_digitrakker_mdl(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFFSET: usize = 4; // @4: u8 format version (0x00, 0x10, 0x11)
    const BLOCK_ID_OFFSET: usize = 5; // @5: 2-byte uppercase ASCII block tag
    require!(
        size >= MIN_DIGITRAKKER_MDL_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"DMDL"),
        matches!(u8_at(prefix, VERSION_OFFSET)?, 0x00 | 0x10 | 0x11),
        u8_at(prefix, BLOCK_ID_OFFSET)?.is_ascii_uppercase(),
        u8_at(prefix, BLOCK_ID_OFFSET + 1)?.is_ascii_uppercase()
    );
    Some(Label::MdlDigitrakker)
}

/// Image-Line DirectWave Sampler Program file (`.dwp`).
pub(crate) fn verify_directwave_dwp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DIRECTWAVE_DWP_SIZE,
        prefix.len() >= 24,
        starts(prefix, DIRECTWAVE_DWP_MAGIC),
        (1..=64).contains(&le32(prefix, DIRECTWAVE_VERSION_OFFSET)?),
        (1..=16).contains(&le32(prefix, DIRECTWAVE_SUBVER_OFFSET)?),
        le32(prefix, DIRECTWAVE_HDR_LEN_OFFSET) == Some(16),
        le32(prefix, DIRECTWAVE_RESERVED_OFFSET) == Some(0)
    );
    Some(Label::DirectwaveDwp)
}

/// Dirk Bialluch Amiga Sample Bank (`.smp`, 1991 Dirk Bialluch).
pub(crate) fn verify_dirk_bialluch_smp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DIRK_BIALLUCH_SIZE,
        prefix.len() >= 16,
        starts(prefix, DIRK_BIALLUCH_MSMP_MAGIC),
        u8_at(prefix, 4)? == 0,
        (1..=64).contains(&u8_at(prefix, 5)?),
        get_slice(prefix, 6, 12)?.iter().all(|&b| b == 0)
    );
    Some(Label::DirkBialluchSmp)
}

/// Dirk Bialluch Amiga Music Module (`.tpu`, 1991 Dirk Bialluch).
pub(crate) fn verify_dirk_bialluch_tpu(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DIRK_BIALLUCH_SIZE,
        prefix.len() >= 16,
        starts(prefix, DIRK_BIALLUCH_MSNG_MAGIC),
        (1..=64).contains(&be32(prefix, 4)?),
        eq_at(prefix, 8, b"MSEQ")
    );
    Some(Label::DirkBialluchTpu)
}

/// Cubic Team / Statix DisorderTracker 2 Module (`.plm`): `"PLM\x1A"` at `@0..4`, `hdr_size >= 0x60` at `@4`, version `0x10` at `@5`.
pub(crate) fn verify_disorder_plm_module(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_SIZE_OFFSET: usize = 4; // @4: u8 header size in bytes (>= 0x60)
    const VERSION_OFFSET: usize = 5; // @5: u8 format version (0x10)
    require!(
        size >= MIN_DISORDER_PLM_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"PLM\x1A"),
        u8_at(prefix, HDR_SIZE_OFFSET)? >= 0x60,
        u8_at(prefix, VERSION_OFFSET) == Some(0x10)
    );
    Some(Label::Plm)
}

/// Audio Simulation DreamStation Synthesizer Song (`.dss`): `"DSFmt1\r\n"` at `@0..8`.
pub(crate) fn verify_dreamstation_dss(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_DREAMSTATION_DSS_SIZE, prefix.len() >= 16, starts(prefix, b"DSFmt1\r\n"));
    Some(Label::DssDreamstation)
}

/// AdLib DrumTraker Rhythm Module (`.dtl`, 1993).
pub(crate) fn verify_drumtraker_dtl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DRUMTRAKER_DTL_SIZE,
        prefix.len() >= 28,
        starts(prefix, DRUMTRAKER_DTL_MAGIC),
        get_slice(prefix, 4, 24)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b)),
        u8_at(prefix, 23)? == 0,
        eq_at(prefix, 24, hex2bin!("00 10 00"))
    );
    Some(Label::DrumtrakerDtl)
}

/// Drumware Kawai K1 Synthesizer Sound Bank (`.k1i`, 1988, Drumware).
///
/// Header opens with the 30-byte 16-bit big-endian-padded `"K1IDRUMWAREInc "` signature at `@0..30`.
pub(crate) fn verify_drumware_k1_bank(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DRUMWARE_K1_SIZE,
        prefix.len() >= DRUMWARE_K1_MAGIC.len(),
        starts(prefix, DRUMWARE_K1_MAGIC)
    );
    Some(Label::DrumwareK1Bank)
}

/// Sony Direct Stream Digital Stream File (`.dsf`, `"DSD "` + `"fmt "`).
pub(crate) fn verify_dsd_stream_file(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DSF_AUDIO_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"DSD "),
        le64(prefix, 4) == Some(28),
        eq_at(prefix, 28, b"fmt ")
    );
    Some(Label::DsfAudio)
}

/// Philips / Sony Direct Stream Digital Interchange File Format (DSDIFF).
pub(crate) fn verify_dsdiff_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DSDIFF_AUDIO_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"FRM8"),
        get_slice(prefix, 4, 8)? == [0, 0, 0, 0],
        get_slice(prefix, 12, 16)? == b"DSD "
    );
    let frm_size = be64(prefix, 4)?;
    let sub_id = get_slice(prefix, 16, 20)?;
    require!(
        frm_size >= 16,
        matches!(sub_id, b"FVER" | b"PROP" | b"COMT" | b"DIIN" | b"DSD " | b"DST ")
    );
    Some(Label::Dsdiff)
}

/// Carlos Hasan Digital Sound Interface Kit Module (`.dsm`): `"DSM\x10"` at `@0..4`.
pub(crate) fn verify_dsik_dsm_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_DSIK_DSM_SIZE, prefix.len() >= 36, starts(prefix, b"DSM\x10"));
    Some(Label::DsmDsik)
}

/// Otto Chrons DSMI Advanced Module Format (`.amf`): `"AMF"` at `@0..3` + version `0x08..=0x0E` at `@3` + 32B ASCII title at `@4..36` + `num_channels` (`1..=32`) at `@40`.
pub(crate) fn verify_dsmi_amf_module(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFFSET: usize = 3; // @3: u8 format version (0x08..=0x0E)
    const NUM_SAMPLES_OFFSET: usize = 36; // @36: u8 sample count (>= 1)
    const NUM_CHANNELS_OFFSET: usize = 40; // @40: u8 channel count (1..=32)
    const COMPACT_DMF_MAGIC: &[u8; 4] = b"DMF\x0E"; // @0..4: DSMI Compact AMF v14 magic (omits 32B title)
    if size >= 42
        && prefix.len() >= 16
        && starts(prefix, COMPACT_DMF_MAGIC)
        && u8_at(prefix, 4)? >= 1
        && u8_at(prefix, 7)? <= 1
        && (1..=32).contains(&u8_at(prefix, 8)?)
    {
        return Some(Label::AmfDsmi);
    }
    require!(
        size >= MIN_DSMI_AMF_SIZE,
        prefix.len() >= 42,
        starts(prefix, b"AMF"),
        (0x08..=0x0E).contains(&u8_at(prefix, VERSION_OFFSET)?),
        u8_at(prefix, NUM_SAMPLES_OFFSET)? >= 1,
        (1..=32).contains(&u8_at(prefix, NUM_CHANNELS_OFFSET)?)
    );
    Some(Label::AmfDsmi)
}

/// Dynamic Studio Professional Tracker Module (`.dsm`, 1994).
pub(crate) fn verify_dynamic_studio_dsm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DYNAMIC_STUDIO_DSM_SIZE,
        prefix.len() >= 64,
        starts(prefix, DYNAMIC_STUDIO_DSM_MAGIC),
        matches!(u8_at(prefix, 4)?, 0x20 | 0x10)
    );
    Some(Label::DynamicStudioDsm)
}

/// Electronic Arts BNK Audio Sample Bank (`"BNKl"` LE or `"BNKb"` BE).
pub(crate) fn verify_ea_bnk_audio_bank(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EA_BNK_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"BNKl") || starts(prefix, b"BNKb"),
        (1..=16).contains(&u8_at(prefix, 4)?),
        has_non_ascii_text_byte(prefix.get(..12)?)
    );
    Some(Label::EaBnk)
}

/// Electronic Arts SCHl / SHEN / SEAD / kVGT Multimedia Audio/Video (`"SCHl"`, `"SHEN"`, `"SEAD"`, `"kVGT"`).
pub(crate) fn verify_ea_schl_multimedia(prefix: &[u8], size: u64) -> Option<Label> {
    const EA_SCHL_MAGIC: &[u8; 4] = b"SCHl"; // @0..4: standard SCHl stream block
    const EA_SHEN_MAGIC: &[u8; 4] = b"SHEN"; // @0..4: big-endian / SHEN stream block
    const EA_SEAD_MAGIC: &[u8; 4] = b"SEAD"; // @0..4: SEAD stream header block
    const EA_KVGT_MAGIC: &[u8; 4] = b"kVGT"; // @0..4: kVGT stream header block
    const BLOCK_SIZE_OFFSET: usize = 4; // @4..8: u32 LE/BE initial header block byte length

    require!(
        size >= MIN_EA_SCHL_SIZE,
        prefix.len() >= 12,
        starts(prefix, EA_SCHL_MAGIC)
            || starts(prefix, EA_SHEN_MAGIC)
            || starts(prefix, EA_SEAD_MAGIC)
            || starts(prefix, EA_KVGT_MAGIC),
        prefix.get(4..12)?.contains(&0)
    );
    let blk_len = le32(prefix, BLOCK_SIZE_OFFSET).unwrap_or(0) as u64;
    let blk_len_be = be32(prefix, BLOCK_SIZE_OFFSET).unwrap_or(0) as u64;
    require!((16..=size).contains(&blk_len) || (16..=size).contains(&blk_len_be));
    Some(Label::EaSchl)
}

/// Casper Amiga EarAche Module (`.ea`): `"EASO"` at `@0..4` + three strictly increasing `u32 BE` table offsets at `@4..16` (`< size`).
pub(crate) fn verify_earache_ea(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_EARACHE_SIZE && prefix.len() >= 16 && starts(prefix, b"EASO"));
    let o1 = be32(prefix, 4)? as u64;
    let o2 = be32(prefix, 8)? as u64;
    let o3 = be32(prefix, 12)? as u64;
    require!(o1 >= 16 && o1 < o2 && o2 < o3 && o3 < size);
    Some(Label::EaracheEa)
}

/// AFMG EASE Generic Loudspeaker Library file (`.gll`).
pub(crate) fn verify_ease_gll(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EASE_GLL_SIZE,
        prefix.len() >= 24,
        starts(prefix, EASE_GLL_MAGIC),
        (1..=16).contains(&le32(prefix, EASE_GLL_VERSION_OFFSET)?)
    );
    Some(Label::EaseGll)
}

/// EasyTrax Module (`.etx`): `"EASYTRAX 1.0"` at `@0..12`.
pub(crate) fn verify_easytrax_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_EASYTRAX_SIZE, prefix.len() >= 16, starts(prefix, b"EASYTRAX 1.0"));
    Some(Label::Etx)
}

/// Vibrants EdLib AdLib FM Audio (`.d00`): `"JCH\x26"` at `@0..4` + version `<= 4` at `@4`.
pub(crate) fn verify_edlib_d00(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EDLIB_D00_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"JCH\x26"),
        u8_at(prefix, 4)? <= 4
    );
    Some(Label::EdlibD00)
}

/// Jens-Christian Huus EdLib Packed OPL2 Module (`.edl`, 1992 Jens-Christian Huus).
pub(crate) fn verify_edlib_edl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EDLIB_EDL_SIZE,
        prefix.len() >= 16,
        starts(prefix, EDLIB_EDL_MAGIC),
        be16(prefix, 6) == Some(0)
    );
    Some(Label::EdlibEdl)
}

/// Milanerny Electronic Music System v1 Amiga Module (`.ems`, 1989 Milanerny).
pub(crate) fn verify_ems_v1(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EMS_V1_SIZE,
        prefix.len() >= 150,
        starts(prefix, EMS_V1_MAGIC_22) || starts(prefix, EMS_V1_MAGIC_25)
    );
    let song_len = be16(prefix, 2)? as usize;
    let max_pat = be16(prefix, 4)?;
    require!(
        (1..=128).contains(&song_len),
        max_pat <= 63,
        get_slice(prefix, 6, 6 + song_len)?.iter().all(|&b| (b as u16) <= max_pat),
        get_slice(prefix, 6 + song_len, 134)?.iter().all(|&b| b == 0)
    );
    Some(Label::EmsV1)
}

/// Amiga Electronic Music System v6 Module (`.ems`, 1991).
pub(crate) fn verify_ems_v6(prefix: &[u8], size: u64) -> Option<Label> {
    const EMS_MAGIC: &[u8; 10] = b"E.M.S. V6."; // @0..10: Electronic Music System v6.xx
    require!(
        size >= MIN_EMS_V6_SIZE,
        prefix.len() >= 20,
        starts(prefix, EMS_MAGIC),
        u8_at(prefix, 10)?.is_ascii_digit(),
        u8_at(prefix, 11)?.is_ascii_digit(),
        eq_at(prefix, 12, hex2bin!("00 01 00 00"))
    );
    Some(Label::EmsV6)
}

/// Passport Designs Encore Musical Notation Score (`.enc`): `"SCOW"` or `"SCO5"` at `@0..4` + `\0\0\0` at `@5..8`.
pub(crate) fn verify_encore_enc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ENCORE_ENC_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"SCOW") || starts(prefix, b"SCO5"),
        eq_at(prefix, 5, b"\x00\x00\x00")
    );
    Some(Label::EncoreEnc)
}

/// Ensoniq PARIS Digital Audio File (`.paf` / `.fap`, 1998 Ensoniq).
pub(crate) fn verify_ensoniq_paris_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ENSONIQ_PARIS_SIZE, prefix.len() >= 28);
    let valid = if starts(prefix, b" paf\0\0\0\0") {
        be32(prefix, 8) == Some(0)
            && (8000..=192000).contains(&be32(prefix, 12)?)
            && be32(prefix, 16)? <= 2
            && (1..=64).contains(&be32(prefix, 20)?)
    } else if starts(prefix, b"fap \0\0\0\0") {
        matches!(le32(prefix, 8), Some(0 | 1))
            && (8000..=192000).contains(&le32(prefix, 12)?)
            && le32(prefix, 16)? <= 2
            && (1..=64).contains(&le32(prefix, 20)?)
    } else {
        false
    };
    require!(valid);
    Some(Label::EnsoniqPaf)
}

/// Entis MIO Music Interleaved and Orthogonal Transformed Audio (`.mio`).
pub(crate) fn verify_entis_mio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ENTIS_MIO_SIZE,
        prefix.len() >= 72,
        starts(prefix, ENTIS_CONTAINER_MAGIC),
        le32(prefix, 8)? != 0,
        eq_at(prefix, 16, ENTIS_MIO_DESC),
        eq_at(prefix, 64, ENTIS_HEADER_CHUNK)
    );
    Some(Label::EntisMio)
}

/// Epic MegaGames MASI ProTracker Studio Module (`.psm`): `"PSM "` + `u32 LE` size + `"FILE"` at `@8..12` (New PSM), or `"PSM\xFE"` + `@63 == 0x1A` or `@64 == 0x1A` (Old PSM).
pub(crate) fn verify_epic_psm_module(prefix: &[u8], size: u64) -> Option<Label> {
    const FILE_TAG_OFFSET: usize = 8; // @8: "FILE" in MASI New PSM
    require!(size >= MIN_EPIC_PSM_SIZE && prefix.len() >= 68);
    let is_new_psm = starts(prefix, b"PSM ") && eq_at(prefix, FILE_TAG_OFFSET, b"FILE");
    let is_old_psm = starts(prefix, b"PSM\xFE")
        && (u8_at(prefix, 63) == Some(0x1A) || u8_at(prefix, 64) == Some(0x1A));
    require!(is_new_psm || is_old_psm);
    Some(Label::PsmMasi)
}

/// Riven the Mage eXotic AdLib Module (`.xad`, 1997 Riven the Mage).
pub(crate) fn verify_exotic_xad(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EXOTIC_XAD_SIZE,
        prefix.len() >= 80,
        starts(prefix, EXOTIC_XAD_MAGIC),
        get_slice(prefix, 4, 76)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b)),
        u8_at(prefix, 39)? == 0,
        (1..=5).contains(&le16(prefix, 76)?)
    );
    Some(Label::ExoticXad)
}

/// ABT Extended MOD Module (`.emd`): `"EMOD\x10"` at `@0..5`.
pub(crate) fn verify_extended_mod_emd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_EXTENDED_MOD_EMD_SIZE, prefix.len() >= 16, starts(prefix, b"EMOD\x10"));
    Some(Label::Emd)
}

/// eXtra Simple Music Module (`.xsm`, `"ofTAZ!"`).
pub(crate) fn verify_extra_simple_music_xsm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_XSM_MUSIC_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"ofTAZ!"),
        le16(prefix, 6)? > 0,
        prefix.get(6..16)?.contains(&0)
    );
    Some(Label::XsmMusic)
}

/// Velvet Development Extreme's Tracker Module (`.ams`): `"Extreme"` at `@0..7` + `u8` low version + `0x01` high version at `@8`.
pub(crate) fn verify_extremes_tracker_ams(prefix: &[u8], size: u64) -> Option<Label> {
    const VER_HI_OFFSET: usize = 8; // @8: u8 major version (1)
    require!(
        size >= MIN_EXTREMES_AMS_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"Extreme"),
        u8_at(prefix, VER_HI_OFFSET) == Some(1)
    );
    Some(Label::AmsExtreme)
}

/// Jörg Schmidt / Maxon Amiga Face The Music 8-Channel Module (`.ftm`): `"FTMN"` at `@0..4` + version `3` at `@4`.
pub(crate) fn verify_face_the_music_module(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFFSET: usize = 4; // @4: u8 format version (3)
    const MEASURES_OFFSET: usize = 5; // @5: u8 number of measures (>= 1)
    require!(
        size >= MIN_FACE_THE_MUSIC_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"FTMN"),
        u8_at(prefix, VERSION_OFFSET) == Some(3),
        u8_at(prefix, MEASURES_OFFSET)? >= 1
    );
    Some(Label::Ftmn)
}

/// NES FamiTracker Module (`.ftm`): `"FamiTracker Module"` at `@0..18` + `u32 LE` version `<= 0x0450` at `@18..22`.
pub(crate) fn verify_famitracker_module(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFFSET: usize = 18; // @18: u32 LE format version (e.g. 0x0440)
    require!(
        size >= MIN_FAMITRACKER_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"FamiTracker Module"),
        (1..=0x0500).contains(&le32(prefix, VERSION_OFFSET)?)
    );
    Some(Label::Ftm)
}

/// Farbrausch V2 Synthesizer Module (`.v2m`, 2000 Tammo Hinrichs / Farbrausch).
pub(crate) fn verify_farbrausch_v2m(prefix: &[u8], size: u64) -> Option<Label> {
    const V2_GLOBALS_SIG: &[u8; 6] = hex2bin!("AB 05 00 04 04 08"); // V2 default global synth parameters
    require!(size >= MIN_FARBRAUSCH_V2M_SIZE, prefix.len() >= 32);
    let timediv = le32(prefix, 0)?;
    let maxtime = le32(prefix, 4)?;
    let gdnum = le32(prefix, 8)? as usize;
    require!(matches!(timediv, 32..=1024), maxtime > 0, (1..=32).contains(&gdnum));
    let globals_end = 12 + gdnum * 10;
    require!(prefix.len() >= globals_end, eq_at(prefix, 16, V2_GLOBALS_SIG));
    Some(Label::FarbrauschV2m)
}

/// Triton FastTracker II Extended Module (`.xm`, 1994): `"Extended Module: "` + `0x1A` at `@37` + version `0x0102..=0x0104` at `@58`.
pub(crate) fn verify_fasttracker2_xm(prefix: &[u8], size: u64) -> Option<Label> {
    const DOS_EOF_OFFSET: usize = 37; // @37: 0x1A DOS EOF marker after 20-byte module name
    const VERSION_OFFSET: usize = 58; // @58: u16 LE version (0x0102..=0x0104)
    require!(
        size >= MIN_FASTTRACKER2_XM_SIZE,
        prefix.len() >= 60,
        starts(prefix, b"Extended Module: "),
        u8_at(prefix, DOS_EOF_OFFSET) == Some(0x1A),
        (0x0102..=0x0104).contains(&le16(prefix, VERSION_OFFSET)?)
    );
    Some(Label::Xm)
}

/// Faust Music Creator AdLib Module (`.sng`): `"FMC!"` at `@0..4` + 20-byte printable/null ASCII title at `@4..24`.
pub(crate) fn verify_faust_music_creator(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FAUST_FMC_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"FMC!"),
        get_slice(prefix, 4, 24)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b)),
        has_non_ascii_text_byte(prefix.get(..32)?)
    );
    Some(Label::Fmc)
}

/// Martin Fernandez AdLib Music Module (.adlib).
pub(crate) fn verify_fernandez_adlib(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_FERNANDEZ_ADLIB_SIZE, prefix.len() >= 16, starts(prefix, b"ADLIB\x01"));
    let scan = get_slice(prefix, 6, prefix.len().min(128))?;
    let nul1 = scan.iter().position(|&b| b == 0)?;
    require!(nul1 >= 1, scan[..nul1].iter().all(|&b| (0x20..=0x7E).contains(&b)));
    let after1 = &scan[nul1 + 1..];
    let nul2 = after1.iter().position(|&b| b == 0)?;
    require!(
        after1[..nul2].iter().all(|&b| (0x20..=0x7E).contains(&b)),
        after1.get(nul2 + 1) == Some(&0x01)
    );
    Some(Label::AdlibFernandez)
}

/// Image-Line FruityLoops / FL Studio Project (`.flp`, `"FLhd"` + `"FLdt"`).
pub(crate) fn verify_fl_studio_project(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FL_STUDIO_FLP_SIZE,
        prefix.len() >= 18,
        starts(prefix, b"FLhd"),
        le32(prefix, 4) == Some(6),
        le16(prefix, 10)? >= 1,
        eq_at(prefix, 14, b"FLdt")
    );
    Some(Label::FlStudioFlp)
}

/// Image-Line FL Studio Mobile Effect/Instrument Rack (.flmrack) Image-Line.
pub(crate) fn verify_flmobile_flmrack(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FLMOBILE_FLMRACK_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"RMOd"),
        get_slice(prefix, 16, 24)? == b"HEAD\x09\0\0\0",
        get_slice(prefix, 33, 37)? == b"DESC"
    );
    Some(Label::FlmobileFlmrack)
}

/// FM-Kingtracker AdLib Module (`.fmk`, 1995).
pub(crate) fn verify_fm_kingtracker(prefix: &[u8], size: u64) -> Option<Label> {
    const FMK_STATE_1: &[u8; 4] = hex2bin!("F4 01 03 32"); // @60..64: OPL2 init mode 1
    const FMK_STATE_2: &[u8; 4] = hex2bin!("F4 02 03 32"); // @60..64: OPL2 init mode 2
    require!(
        size >= MIN_FM_KINGTRACKER_SIZE,
        prefix.len() >= 96,
        starts(prefix, FM_KINGTRACKER_MAGIC),
        get_slice(prefix, 4, 60)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b))
            || eq_at(prefix, 60, FMK_STATE_1)
            || eq_at(prefix, 60, FMK_STATE_2),
        has_non_ascii_text_byte(prefix.get(60..96)?)
    );
    Some(Label::FmKingtracker)
}

/// Fujitsu FM Towns Euphony Song Module (`.eup`, 1989 Fujitsu).
pub(crate) fn verify_fm_towns_euphony(prefix: &[u8], size: u64) -> Option<Label> {
    const MUTE_OFF: usize = 852; // @852..884: 32 track enable/mute bytes (0x00 or 0xFF)
    const PORT_OFF: usize = 884; // @884..916: 32 track port indices (0x00..=0x08 or 0xFF)
    const CHAN_OFF: usize = 916; // @916..948: 32 track MIDI channels (0x00..=0x0F or 0xFF)
    const ZERO_OFF: usize = 1780; // @1780..2036: 256-byte zero-filled reserved block
    const BODY_PARAM_OFF: usize = 2048; // @2048..2052: u32 LE initial bar/body parameter

    require!(size >= MIN_FM_TOWNS_EUPHONY_SIZE, prefix.len() >= 2054);
    require!(
        get_slice(prefix, MUTE_OFF, MUTE_OFF + 32)?.iter().all(|&b| b == 0x00 || b == 0xFF),
        get_slice(prefix, PORT_OFF, PORT_OFF + 32)?.iter().all(|&b| b <= 0x08 || b == 0xFF),
        get_slice(prefix, CHAN_OFF, CHAN_OFF + 32)?.iter().all(|&b| b <= 0x0F || b == 0xFF),
        get_slice(prefix, CHAN_OFF, CHAN_OFF + 32)?
            .iter()
            .filter(|&&b| (1..=15).contains(&b))
            .count()
            >= 3,
        get_slice(prefix, ZERO_OFF, ZERO_OFF + 256)?.iter().all(|&b| b == 0),
        (1..size as u32).contains(&le32(prefix, BODY_PARAM_OFF)?)
    );
    Some(Label::EuphonyEup)
}

/// Fujitsu FM Towns Euphony PCM Instrument Bank (`.pmb`, 1989 Fujitsu).
///
/// Validates the 8-byte null preamble (`@0..8`) and Voice Slot 0 (`@8..136`, 128 bytes):
/// - `@8..16`: empty `[0; 8]` or NUL-padded ASCII instrument name (`0x20..=0x7E`)
/// - `@16..22`: reserved `[0; 6]`
/// - `@22..40`: 9 `u16 LE` monotonically non-decreasing MIDI key boundaries `<= 0x7F`, ending in `0x007F`
/// - `@40..72`: 8 `u32 LE` PCM sample IDs (`< 64`)
/// - `@72..136`: 8 x 8B envelope blocks with bytes `0..6 <= 0x7F` and bytes `6..8 == [0, 0]`.
pub(crate) fn verify_fm_towns_euphony_pmb(prefix: &[u8], size: u64) -> Option<Label> {
    const SLOT0_OFF: usize = 8; // @8: first 128-byte PCM voice slot
    const PAD6_OFF: usize = SLOT0_OFF + 8; // @16..22: 6 reserved NUL bytes
    const KEYSPLIT_OFF: usize = SLOT0_OFF + 14; // @22..40: 9 x u16 LE keysplit table
    const SAMPLE_IDS_OFF: usize = SLOT0_OFF + 32; // @40..72: 8 x u32 LE wave IDs
    const ENV_OFF: usize = SLOT0_OFF + 64; // @72..136: 8 x 8B envelope parameters
    const SLOT0_END: usize = SLOT0_OFF + 128; // @136

    require!(
        size >= MIN_FMTOWNS_EUPHONY_PMB_SIZE,
        prefix.len() >= SLOT0_END,
        eq_at(prefix, 0, &[0u8; 8]),
        eq_at(prefix, PAD6_OFF, &[0u8; 6])
    );

    // Slot 0 name (`@8..16`): all zeros or ASCII name padded with trailing NULs.
    let name = get_slice(prefix, SLOT0_OFF, SLOT0_OFF + 8)?;
    let mut seen_nul = false;
    let mut has_name = false;
    for &b in name {
        if b == 0 {
            seen_nul = true;
        } else if seen_nul || !(0x20..=0x7E).contains(&b) {
            return None;
        } else {
            has_name = true;
        }
    }

    // 9 x u16 LE monotonically non-decreasing MIDI key limits ending with `0x007F`.
    let mut prev_key = 0u16;
    for i in 0..9 {
        let k = le16(prefix, KEYSPLIT_OFF + i * 2)?;
        require!(k <= 0x007F && k >= prev_key);
        prev_key = k;
    }
    require!(prev_key == 0x007F);

    // 8 x u32 LE wave sample indices (`< 64`).
    for i in 0..8 {
        let id = le32(prefix, SAMPLE_IDS_OFF + i * 4)?;
        require!(id < 64);
    }

    // 8 x 8B envelope blocks: bytes `0..6` in `0..=0x7F`, trailing `u16 == 0`.
    let mut env_non_zero = false;
    for i in 0..8 {
        let blk = get_slice(prefix, ENV_OFF + i * 8, ENV_OFF + (i + 1) * 8)?;
        require!(blk[0..6].iter().all(|&b| b <= 0x7F), blk[6] == 0, blk[7] == 0);
        if blk[0..6].iter().any(|&b| b != 0) {
            env_non_zero = true;
        }
    }
    require!(has_name || env_non_zero);

    Some(Label::FmtownsEuphonyPmb)
}

/// FM Tracker AdLib Module (`.fmt`, 1994, `"FMTracker\x01\x01"` or `"FMTRK\x1A"`).
pub(crate) fn verify_fm_tracker(prefix: &[u8], size: u64) -> Option<Label> {
    const FMT_LONG_MAGIC: &[u8; 11] = b"FMTracker\x01\x01"; // @0..11: FMTracker v1.1 header
    const FMT_SHORT_MAGIC: &[u8; 6] = b"FMTRK\x1A"; // @0..6: FMTRK AdLib header
    require!(
        size >= MIN_FM_TRACKER_SIZE,
        prefix.len() >= 26,
        starts(prefix, FMT_LONG_MAGIC)
            || (starts(prefix, FMT_SHORT_MAGIC) && eq_at(prefix, 14, &[0, 0]))
    );
    Some(Label::FmTracker)
}

/// Firelight Technologies FMOD Designer Event Metadata Bank (`.fev`, `"FEV1"`, 2006 Firelight Technologies).
pub(crate) fn verify_fmod_designer_fev(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_FMOD_DESIGNER_FEV_SIZE, prefix.len() >= 24, starts(prefix, b"FEV1"));
    let ver = le32(prefix, 4)?;
    let name_len = le32(prefix, 16)? as usize;
    require!(
        ver > 0 && ver <= 0x0100_0000,
        (2..=128).contains(&name_len),
        20 + name_len <= prefix.len(),
        u8_at(prefix, 20 + name_len - 1)? == 0,
        get_slice(prefix, 20, 20 + name_len - 1)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::FmodFev)
}

/// NEC PC-98 / Sharp X68000 MUSIC DRV V1.0 FM/SSG chiptune module (`.m`).
pub(crate) fn verify_fmp_m(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_FMP_M_SIZE, prefix.len() >= 32, starts(prefix, FMP_M_MAGIC));
    Some(Label::FmpM)
}

/// Jason Nunn Funktracker Module (`.fnk`, 1995 Jason Nunn).
pub(crate) fn verify_funktracker(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FUNKTRACKER_SIZE,
        prefix.len() >= 24,
        starts(prefix, FUNKTRACKER_MAGIC),
        u8_at(prefix, 12)? == b'F',
        has_non_ascii_text_byte(prefix.get(4..20)?)
    );
    Some(Label::FunktrackerFnk)
}

/// Superions Future Composer BSI Amiga Module (`.bsi`, 1990 Superions).
pub(crate) fn verify_future_composer_bsi(prefix: &[u8], size: u64) -> Option<Label> {
    const FUCO_MAGIC: &[u8; 5] = b"FUCO\xC1"; // @0..5: Future Composer BSI signature
    require!(size >= MIN_FUTURE_COMPOSER_BSI_SIZE, prefix.len() >= 16, starts(prefix, FUCO_MAGIC));
    Some(Label::FutureComposerBsi)
}

/// Carlo Vogelsang Galaxy Music System (`"MUSE"` + `0xDEADBEEF`, `0xDEADBEAF`, or `0xDEADBABE` BE).
pub(crate) fn verify_galaxy_music_system(prefix: &[u8], size: u64) -> Option<Label> {
    const GALAXY_MUSE_MAGIC: &[u8; 4] = b"MUSE"; // @0..4: Galaxy Music System container magic
    const SUBHEADER_TAG_OFFSET: usize = 4; // @4..8: u32 BE format revision sentinel

    require!(
        size >= MIN_GALAXY_MUSE_SIZE,
        prefix.len() >= 8,
        starts(prefix, GALAXY_MUSE_MAGIC),
        matches!(be32(prefix, SUBHEADER_TAG_OFFSET), Some(0xDEAD_BEEF | 0xDEAD_BEAF | 0xDEAD_BABE))
    );
    Some(Label::GalaxyMusicMuse)
}

/// Andreas Tadic Game Music Creator Amiga Module (`.gmc`, 1990 Andreas Tadic).
pub(crate) fn verify_game_music_creator(prefix: &[u8], size: u64) -> Option<Label> {
    const SONG_LEN_OFF: usize = 240; // @240..244: u32 BE song order count (1..=100)
    const PAT_TBL_OFF: usize = 244; // @244..444: 100 x u16 BE pattern byte offsets (% 1024 == 0)
    require!(size >= MIN_GAME_MUSIC_CREATOR_SIZE, prefix.len() >= 444);
    let mut nonzero_smp = 0usize;
    for i in 0..15usize {
        let off = i * 16;
        let len_w = be16(prefix, off + 4)?;
        let vol = be16(prefix, off + 6)?;
        require!(u8_at(prefix, off)? == 0, vol <= 0x40);
        if len_w > 0 {
            nonzero_smp += 1;
        }
    }
    let song_len = be32(prefix, SONG_LEN_OFF)? as usize;
    require!(nonzero_smp >= 1, (1..=100).contains(&song_len));
    let mut max_pat = 0u64;
    for i in 0..song_len {
        let po = be16(prefix, PAT_TBL_OFF + i * 2)? as u64;
        require!(po.is_multiple_of(1024));
        if po > max_pat {
            max_pat = po;
        }
    }
    require!(size >= 444 + max_pat + 1024);
    Some(Label::GameMusicCreator)
}

/// Game Boy Ripped Sound Format (`.gbr`, 1999).
pub(crate) fn verify_gameboy_gbr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GAMEBOY_GBR_SIZE,
        prefix.len() >= 32,
        starts(prefix, GAMEBOY_GBR_MAGIC),
        u8_at(prefix, 4)? >= 1,
        u8_at(prefix, 5)? == 0,
        u8_at(prefix, 6)? <= 3,
        get_slice(prefix, 12, 32)?.iter().all(|&b| b == 0)
    );
    Some(Label::GameboyGbr)
}

/// Nintendo Game Boy Sound System Chiptune Rip (.gbs).
pub(crate) fn verify_gameboy_gbs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_GAMEBOY_GBS_SIZE && prefix.len() >= 16 && starts(prefix, b"GBS\x01"));
    let num_songs = u8_at(prefix, 4)?;
    let first_song = u8_at(prefix, 5)?;
    let load_addr = le16(prefix, 6)?;
    let init_addr = le16(prefix, 8)?;
    let play_addr = le16(prefix, 10)?;
    require!(
        num_songs >= 1
            && first_song >= 1
            && play_addr > 0
            && load_addr <= 0x7FFF
            && init_addr <= 0x7FFF
    );
    Some(Label::GameboyGbs)
}

/// GlueMon Amiga Chiptune Module (`.glue`, 1991).
pub(crate) fn verify_gluemon(prefix: &[u8], size: u64) -> Option<Label> {
    const GLUEMON_HDR: &[u8; 8] = hex2bin!("47 4C 55 45 B8 B3 AA BA"); // @0..8: "GLUE" + 4-byte inverted tag
    require!(size >= MIN_GLUEMON_SIZE, prefix.len() >= 20, starts(prefix, GLUEMON_HDR));
    Some(Label::Gluemon)
}

/// Commodore 64 GoatTracker Chiptune Song / Instrument (.sng / .ins).
pub(crate) fn verify_goattracker(prefix: &[u8], size: u64) -> Option<Label> {
    if starts(prefix, b"GTI!")
        || starts(prefix, b"GTI2")
        || starts(prefix, b"GTI3")
        || starts(prefix, b"GTI4")
        || starts(prefix, b"GTI5")
    {
        require!(
            size >= MIN_GOATTRACKER_INS_SIZE
                && prefix.len() >= 16
                && u8_at(prefix, 4)? <= 64
                && has_non_ascii_text_byte(prefix.get(4..16)?)
        );
        return Some(Label::Goattracker);
    }
    require!(
        size >= MIN_GOATTRACKER_SNG_SIZE
            && prefix.len() >= 101
            && (starts(prefix, b"GTS!")
                || starts(prefix, b"GTS2")
                || starts(prefix, b"GTS3")
                || starts(prefix, b"GTS4")
                || starts(prefix, b"GTS5"))
            && (1..=32).contains(&u8_at(prefix, 100)?)
            && prefix.get(4..100)?.contains(&0)
    );
    Some(Label::Goattracker)
}

/// Laurent de Soras Graoumf Tracker Module (`"GT2"` / `"GTK"` + version `1..=9`).
pub(crate) fn verify_graoumf_tracker_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GRAOUMF_GT2_SIZE,
        prefix.len() >= 8,
        (starts(prefix, b"GT2") || starts(prefix, b"GTK"))
            && (1..=9).contains(&u8_at(prefix, 3).unwrap_or(0))
    );
    Some(Label::GraoumfGt2)
}

/// GT Game Systems Amiga Music Module (`.dux`, 1991 GT Game Systems).
pub(crate) fn verify_gt_game_systems_dux(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GT_GAME_SYSTEMS_DUX_SIZE,
        prefix.len() >= 20,
        (be32(prefix, 0)? as u64) == size,
        be32(prefix, 4) == Some(0)
    );
    let len0 = be16(prefix, 8)? as u32;
    let vol0 = be16(prefix, 10)?;
    require!(len0 > 0, vol0 <= 0x40, be32(prefix, 16) == Some(len0));
    Some(Label::GtGameSystemsDux)
}

/// Arobas Music Guitar Pro 6 `.gpx` (`BCFZ` / `BCFS`) container.
pub(crate) fn verify_guitar_pro_gpx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GUITAR_PRO_GPX_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"BCFZ") || starts(prefix, b"BCFS"),
        (256..=67_108_864).contains(&le32(prefix, GPX_UNCOMP_LEN_OFFSET)?),
        has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    Some(Label::GuitarProGpx)
}

/// Advanced Gravis Ultrasound GF1 Instrument Patch (`.pat`, 1992) Advanced Gravis & Forte Technologies.
pub(crate) fn verify_gus_patch(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GUS_PATCH_SIZE
            && prefix.len() >= 90
            && (starts(prefix, b"GF1PATCH110\0") || starts(prefix, b"GF1PATCH100\0"))
            && eq_at(prefix, 12, b"ID#000002\0")
    );
    let instruments = u8_at(prefix, 82)?;
    let voices = u8_at(prefix, 83)?;
    let waveforms = le16(prefix, 85)?;
    let master_volume = le16(prefix, 87)?;
    require!(
        instruments <= 64 && voices <= 64 && (1..=512).contains(&waveforms) && master_volume <= 512
    );
    Some(Label::GusPat)
}

/// HAL Laboratory GameCube HALPST Audio Stream (.hps).
pub(crate) fn verify_halpst_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_HALPST_AUDIO_SIZE, prefix.len() >= 16, starts(prefix, b" HALPST\0"));
    let sample_rate = be32(prefix, 8)?;
    let channels = be32(prefix, 12)?;
    require!((4_000..=96_000).contains(&sample_rate), (1..=8).contains(&channels));
    Some(Label::Halpst)
}

/// Herad / Cryo Interactive AdLib Music (`.agd` / `.sdb` / `.sqx`, 1992 Cryo Interactive).
pub(crate) fn verify_herad_adlib(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_HERAD_ADLIB_SIZE, prefix.len() >= 48);
    let is_agd_sdb = u8_at(prefix, 2)? == 0
        && (le16(prefix, 3)? as u64) == size
        && le16(prefix, 6) == Some(0xFFFF)
        && matches!(le16(prefix, 10), Some(0x0032 | 0x0052));
    let is_sqx = u8_at(prefix, 2)? <= 1
        && u8_at(prefix, 4)? == 2
        && (3..=4).contains(&u8_at(prefix, 5)?)
        && le16(prefix, 10) == Some(0x0032)
        && (memchr::memmem::find(get_slice(prefix, 24, 48)?, hex2bin!("00 F0 FF 15")).is_some()
            || memchr::memmem::find(get_slice(prefix, 24, 48)?, hex2bin!("00 F8 FF 15")).is_some());
    require!(is_agd_sdb || is_sqx);
    Some(Label::HeradAdlib)
}

/// Jochen Hippel Amiga 68000 / 7V Module (`.hip` / `.hip7` / `.7v`):
/// - `"7V\0\0"` 4-byte 7-voice header magic
/// - `"TFMX\0"` unpacked Hippel TFMX / 7V binary table header (`@4..16` u16 BE table counts, `@20..28 == 0`)
/// - Short M68000 branch table `60 <even d1 != 0> 60 <even d2 != 0>` followed by `48 E7 FF FE` / `48 E7 80 80` (`MOVEM.L`) or `41 FA ... 30 80 4E 75` (`LEA d16(PC),A0; ...; MOVE.W D0,(A0); RTS`)
/// - Word M68000 branch table `60 00 <even d1 != 0> 60 00 <even d2 != 0>` followed by `48 E7 FF FE` / `48 E7 80 80`
/// - `60 00 00 2A` (`BRA.W +$2A`) with `"Player by Jochen Hippel"` banner within `@4..48`.
pub(crate) fn verify_hippel_7v_module(prefix: &[u8], size: u64) -> Option<Label> {
    const HIPPEL_7V_MAGIC: &[u8; 4] = b"7V\x00\x00"; // @0..4: 7-voice module header magic
    const HIPPEL_TFMX_MAGIC: &[u8; 5] = b"TFMX\x00"; // @0..5: unpacked Hippel TFMX / 7V song table header
    const BRA_SHORT_OP: u8 = 0x60; // M68000 BRA.S opcode
    const BRA_WORD_OP: &[u8; 2] = hex2bin!("60 00"); // M68000 BRA.W opcode
    const MOVEM_ALL_REGS: &[u8; 4] = hex2bin!("48 E7 FF FE"); // MOVEM.L D0-D7/A0-A6,-(SP)
    const MOVEM_D0_A0: &[u8; 4] = hex2bin!("48 E7 80 80"); // MOVEM.L D0/A0,-(SP)
    const LEA_PC_REL: &[u8; 2] = hex2bin!("41 FA"); // LEA d16(PC),A0
    const MOVE_RTS_TAIL: &[u8; 4] = hex2bin!("30 80 4E 75"); // MOVE.W D0,(A0); RTS
    const BANNER_BRANCH: &[u8; 4] = hex2bin!("60 00 00 2A"); // BRA.W +$2A skipping player banner
    const PLAYER_BANNER: &[u8] = b"Player by Jochen Hippel"; // @4..48: embedded replay copyright string

    require!(size >= MIN_HIPPEL_7V_SIZE, prefix.len() >= 16);
    let is_7v_magic = starts(prefix, HIPPEL_7V_MAGIC);
    let is_unpacked_tfmx = prefix.len() >= 32
        && starts(prefix, HIPPEL_TFMX_MAGIC)
        && be16(prefix, 4).unwrap_or(0) > 0
        && be16(prefix, 6).unwrap_or(0) > 0
        && be16(prefix, 20).unwrap_or(99) <= 8
        && eq_at(prefix, 22, &[0u8; 6]);
    let is_short_branch_player = u8_at(prefix, 0)? == BRA_SHORT_OP
        && u8_at(prefix, 1)? != 0
        && (u8_at(prefix, 1)? & 1) == 0
        && u8_at(prefix, 2)? == BRA_SHORT_OP
        && u8_at(prefix, 3)? != 0
        && (u8_at(prefix, 3)? & 1) == 0
        && (eq_at(prefix, 4, MOVEM_ALL_REGS)
            || eq_at(prefix, 4, MOVEM_D0_A0)
            || (eq_at(prefix, 4, LEA_PC_REL) && eq_at(prefix, 12, MOVE_RTS_TAIL)));
    let is_word_branch_player = starts(prefix, BRA_WORD_OP)
        && matches!(be16(prefix, 2), Some(d1) if d1 != 0 && (d1 & 1) == 0)
        && eq_at(prefix, 4, BRA_WORD_OP)
        && matches!(be16(prefix, 6), Some(d2) if d2 != 0 && (d2 & 1) == 0)
        && (eq_at(prefix, 8, MOVEM_ALL_REGS) || eq_at(prefix, 8, MOVEM_D0_A0));
    let is_banner_7v = starts(prefix, BANNER_BRANCH)
        && prefix.len() >= 32
        && scan_range(prefix, 4, 48, PLAYER_BANNER);
    require!(
        is_7v_magic
            || is_unpacked_tfmx
            || is_short_branch_player
            || is_word_branch_player
            || is_banner_7v
    );
    Some(Label::HippelHip)
}

/// Jochen Hippel Amiga/Atari ST COSO Compressed Module (`.hipc` / `.soc`): `"COSO"` at `@0..4` + increasing `u32 BE` table offsets at `@4..28` (`0x20..size`).
pub(crate) fn verify_hippel_coso_module(prefix: &[u8], size: u64) -> Option<Label> {
    const OFF1: usize = 4; // @4: u32 BE offset 1 (typically 0x40)
    const OFF2: usize = 8; // @8: u32 BE offset 2
    require!(size >= MIN_HIPPEL_COSO_SIZE, prefix.len() >= 36, starts(prefix, b"COSO"));
    let o1 = be32(prefix, OFF1)? as u64;
    let o2 = be32(prefix, OFF2)? as u64;
    require!(o1 >= 0x20 && o1 < o2 && o2 < size);
    Some(Label::Hipc)
}

/// Mahoney & Kaktus His Master's Noise Amiga Module (`.mod`, 1990 Mahoney & Kaktus).
/// Layout: `31 × 30`B sample headers (`vol <= 0x40`, allowing `0xFF` chip finetune), `song_len` at `@950`, `"M&K!"` or `"FEST"` at `@1080..1084`.
pub(crate) fn verify_his_masters_noise(prefix: &[u8], size: u64) -> Option<Label> {
    const HMN_TAG_OFF: usize = 1080; // @1080..1084: "M&K!" or "FEST"
    const HMN_TAG_MK: &[u8; 4] = b"M&K!";
    const HMN_TAG_FEST: &[u8; 4] = b"FEST";
    require!(
        size >= MIN_HIS_MASTERS_NOISE_SIZE,
        prefix.len() >= 1084,
        eq_at(prefix, HMN_TAG_OFF, HMN_TAG_MK) || eq_at(prefix, HMN_TAG_OFF, HMN_TAG_FEST),
        (1..=128).contains(&u8_at(prefix, 950)?),
        has_non_ascii_text_byte(prefix.get(..1080)?)
    );
    for i in 0..31usize {
        let off = 20 + i * 30;
        require!(u8_at(prefix, off + 25)? <= 0x40);
    }
    Some(Label::HisMastersNoise)
}

/// Amiga / Cross-Platform HivelyTracker Chiptune Module (.hvl).
pub(crate) fn verify_hively_tracker(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HIVELY_TRACKER_SIZE
            && prefix.len() >= 16
            && (starts(prefix, b"HVL\x00") || starts(prefix, b"HVL\x01"))
    );
    let title_off = be16(prefix, 4)? as u64;
    let pos_nr = be16(prefix, 6)? & 0x0FFF;
    let track_len = u8_at(prefix, 10)?;
    let ins_nr = u8_at(prefix, 12)?;
    require!(
        (16..size).contains(&title_off)
            && pos_nr <= 1000
            && (1..=64).contains(&track_len)
            && ins_nr <= 63
    );
    Some(Label::HivelyTracker)
}

/// The Holy Noise Amiga Module (`.thn`): `"ARP."` at `@0..4`.
pub(crate) fn verify_holy_noise_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HOLY_NOISE_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"ARP."),
        be16(prefix, 8)? > 0,
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::HolyNoise)
}

/// Hannes Seifert HSC-Tracker OPL2 Module (`.hsc`, 1992 Hannes Seifert).
/// Diophantine invariant: `(size - 1587) % 1152 == 0` with `1..=50` patterns, valid initial OPL2 slots & active orders.
pub(crate) fn verify_hsc_tracker(prefix: &[u8], size: u64) -> Option<Label> {
    const ORDER_LIST_OFF: usize = 1536; // @1536..1587: 51-byte pattern order list (after 128 x 12B instruments)
    const HEADER_SIZE: u64 = 1587; // 1536 + 51
    const PATTERN_SIZE: u64 = 1152; // 64 rows * 9 channels * 2 bytes
    require!(
        size >= MIN_HSC_TRACKER_SIZE,
        prefix.len() >= 1587,
        (size - HEADER_SIZE).is_multiple_of(PATTERN_SIZE)
    );
    let npats = (size - HEADER_SIZE) / PATTERN_SIZE;
    require!((1..=50).contains(&npats));
    // First 4 instrument slots must be valid OPL2 definitions
    let mut valid_nonzero = 0usize;
    for i in 0..128usize {
        let off = i * 12;
        let inst = get_slice(prefix, off, off + 12)?;
        let is_valid = (inst[8] & 0xF0) == 0 && inst[9] <= 3 && inst[10] <= 3;
        if i < 4 {
            require!(is_valid);
        }
        if is_valid && inst.iter().any(|&b| b != 0) {
            valid_nonzero += 1;
        }
    }
    require!(valid_nonzero >= 8);
    // Validate active pattern order list up to 0x80+ end/jump marker
    let first_ord = u8_at(prefix, ORDER_LIST_OFF)?;
    require!((first_ord as u64) < npats);
    for &ord in get_slice(prefix, ORDER_LIST_OFF, ORDER_LIST_OFF + 51)? {
        if ord >= 0x80 {
            break;
        }
        require!((ord as u64) < npats);
    }
    Some(Label::HscTracker)
}

/// Images Design Images Music System / Unicorn Tracker Amiga Module (`.ims`, 1990 Images Design).
pub(crate) fn verify_images_music_system(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_IMAGES_MUSIC_SYSTEM_SIZE,
        prefix.len() >= 1084,
        (1..=128).contains(&u8_at(prefix, 950)?),
        u8_at(prefix, 951)? <= 0x7F,
        be16(prefix, 1080) == Some(0),
        (1..=64).contains(&u8_at(prefix, 1082)?),
        u8_at(prefix, 1083)? == 0x3C, // @1083: '<' IMS sentinel
        has_non_ascii_text_byte(prefix.get(..1080)?)
    );
    let song_len = u8_at(prefix, 950)? as usize;
    for &o in get_slice(prefix, 952, 952 + song_len)? {
        require!(o < 64);
    }
    for i in 0..31usize {
        let off = 20 + i * 30;
        require!(u8_at(prefix, off + 25)? <= 0x40);
    }
    Some(Label::ImagesMusicSystem)
}

/// Lutz Roeder Imago Orpheus Tracker Module (`.imf`, 1995 Lutz Roeder).
pub(crate) fn verify_imago_imf(prefix: &[u8], size: u64) -> Option<Label> {
    const IM10_OFF: usize = 60; // @60..64: "IM10" signature
    const IM10_MAGIC: &[u8; 4] = b"IM10";
    require!(
        size >= MIN_IMAGO_IMF_SIZE,
        prefix.len() >= 64,
        eq_at(prefix, IM10_OFF, IM10_MAGIC),
        get_slice(prefix, 0, 32)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b)),
        u8_at(prefix, 31)? == 0,
        (1..=256).contains(&le16(prefix, 32)?),
        le16(prefix, 34)? <= 256,
        le16(prefix, 36)? <= 256
    );
    Some(Label::ImagoImf)
}

/// Impulse Tracker Music Module (.it).
pub(crate) fn verify_impulse_tracker(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_IT_SIZE && prefix.len() >= 64);
    require!(starts(prefix, b"IMPM"));

    let ord_num = le16(prefix, 32)?;
    let ins_num = le16(prefix, 34)?;
    let smp_num = le16(prefix, 36)?;
    let pat_num = le16(prefix, 38)?;
    let cwt_v = le16(prefix, 40)?;
    let cmwt = le16(prefix, 42)?;
    let gv = u8_at(prefix, 48)?;
    let mv = u8_at(prefix, 49)?;

    // Compatible version cmwt must be <= cwt_v and >= 0x0100
    require!(cmwt >= 0x0100 && cmwt <= cwt_v);
    // Standard limits: ord_num <= 256, ins_num <= 256, smp_num <= 256, pat_num <= 256
    require!(ord_num <= 256 && ins_num <= 256 && smp_num <= 256 && pat_num <= 256);
    // Global and mix volumes: 0..=128
    require!(gv <= 128 && mv <= 128);

    Some(Label::It)
}

/// LucasArts iMUSE MCMP Compressed Audio (`"MCMP"`).
pub(crate) fn verify_imuse_mcmp_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_IMUSE_MCMP_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"MCMP"),
        (1..=16384).contains(&be16(prefix, 4)?),
        prefix.get(4..12)?.contains(&0)
    );
    Some(Label::ImuseMcmp)
}

/// In Stereo! 1.0/2.0 Amiga Module (`"ISM!V1.2"` / `"IS20DF10"`).
pub(crate) fn verify_in_stereo_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_IN_STEREO_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"ISM!V1.2") || starts(prefix, b"IS20DF10"),
        prefix.get(8..20)?.contains(&0)
    );
    Some(Label::InStereoIs)
}

/// Infogrames / Rob Hubbard II Amiga Sample Bank (`.ins`, 1989 Infogrames).
pub(crate) fn verify_infogrames_ins(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_INFOGRAMES_INS_SIZE, prefix.len() >= 24);
    let end = be32(prefix, 0)? as u64;
    let tbl_off = be32(prefix, 4)?;
    let smp0_a = be32(prefix, 8)?;
    let smp0_b = be32(prefix, 12)?;
    let smp0_words = be16(prefix, 18)? as u32;
    let smp1_off = be32(prefix, 20)?;
    require!(
        end >= MIN_INFOGRAMES_INS_SIZE && end <= size && size - end <= 16,
        (0x10..=0x400).contains(&tbl_off),
        smp0_a == smp0_b && smp0_a >= 24 && (smp0_a as u64) < size,
        smp0_words > 0,
        smp1_off == smp0_b.checked_add(smp0_words * 2)?
    );
    Some(Label::InfogramesIns)
}

/// Interplay Entertainment ACM Compressed Audio stream (`.acm`, 1997 Interplay, `97 28 03 01`).
pub(crate) fn verify_interplay_acm(prefix: &[u8], size: u64) -> Option<Label> {
    const ACM_MAGIC: &[u8; 4] = hex2bin!("97 28 03 01"); // @0..4: 0x01032897 LE signature
    const SAMPLES_OFFSET: usize = 4; // @4..8: u32 LE total PCM sample count (> 0)
    const CHANNELS_OFFSET: usize = 8; // @8..10: u16 LE channel count (1 or 2)
    const RATE_OFFSET: usize = 10; // @10..12: u16 LE sample rate in Hz (4000..=96000)

    require!(
        size >= MIN_INTERPLAY_ACM_SIZE,
        prefix.len() >= 14,
        starts(prefix, ACM_MAGIC),
        le32(prefix, SAMPLES_OFFSET)? > 0,
        matches!(le16(prefix, CHANNELS_OFFSET)?, 1 | 2),
        le16(prefix, RATE_OFFSET)? >= 4000
    );
    Some(Label::InterplayAcm)
}

/// Ione Sound Programming Language Amiga Module (.spl).
pub(crate) fn verify_ione_soprol(prefix: &[u8], size: u64) -> Option<Label> {
    const SOPROL_OFF: usize = 0x24; // @0x24 (36): "!SOPROL!" 8-byte ASCII signature
    require!(
        size >= MIN_IONE_SOPROL_SIZE,
        prefix.len() >= 0x2E,
        eq_at(prefix, SOPROL_OFF, b"!SOPROL!")
    );
    Some(Label::SoprolSpl)
}

/// Shortcut Ixalance Impulse Tracker Synth Module (`.ixs`): `"IXS!"` at `@0..4` + `00 00 00 00` at `@4..8` + non-zero `u32 LE` lengths at `@8..16`.
pub(crate) fn verify_ixalance_ixs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_IXALANCE_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"IXS!"),
        le32(prefix, 4) == Some(0),
        le32(prefix, 8)? > 0
    );
    Some(Label::IxalanceIxs)
}

/// M. Gemmel JamCracker Pro Amiga Module (`"BeEp"`).
pub(crate) fn verify_jamcracker_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JAMCRACKER_SIZE,
        prefix.len() >= 8,
        starts(prefix, b"BeEp"),
        (1..=256).contains(&be16(prefix, 4)?)
    );
    Some(Label::JamcrackerJam)
}

/// Jason Page Old Amiga Music Module (`.jpo`, 1988 Jason Page).
pub(crate) fn verify_jason_page_old(prefix: &[u8], size: u64) -> Option<Label> {
    const JPO_TAIL: &[u8; 8] = hex2bin!("30 3C 00 FF 32 00 4E B9"); // @32..40: MOVE.W #$FF,D0; MOVE.W D0,D1; JSR
    require!(
        size >= MIN_JASON_PAGE_OLD_SIZE,
        prefix.len() >= 40,
        be16(prefix, 0) == Some(0x2B7C),
        be16(prefix, 8) == Some(0x2B7C),
        be16(prefix, 16) == Some(0x2B7C),
        be16(prefix, 24) == Some(0x2B7C),
        eq_at(prefix, 32, JPO_TAIL)
    );
    Some(Label::JasonPageOld)
}

/// Johannes Bjerregaard JayTrax Synth Module (`.jxs`, 1999 Johannes Bjerregaard).
pub(crate) fn verify_jaytrax_jxs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JAYTRAX_JXS_SIZE,
        prefix.len() >= 40,
        starts(prefix, JAYTRAX_JXS_V0_MAGIC) || starts(prefix, JAYTRAX_JXS_V1_MAGIC),
        (1..=256).contains(&le32(prefix, 4)?),
        le32(prefix, 8) == Some(1),
        (1..=128).contains(&le32(prefix, 12)?),
        eq_at(prefix, 16, &[0u8; 24])
    );
    Some(Label::JaytraxJxs)
}

/// Jeroen Tel / Maniacs of Noise Amiga Module (`.jt`, 1990 Jeroen Tel) at `@0` or `@8`/`@12` (`offset_probe`).
pub(crate) fn verify_jeroen_tel_jt_offset(prefix: &[u8], size: u64) -> Option<Label> {
    const CIA_LED_OFF: &[u8; 8] = hex2bin!("08 F9 00 01 00 BF E0 01"); // BSET #1,$BFE001
    require!(
        size >= MIN_JEROEN_TEL_JT_SIZE,
        prefix.len() >= 24,
        match_jeroen_tel_jt_at(prefix, 0)
            || (starts(prefix, CIA_LED_OFF) && match_jeroen_tel_jt_at(prefix, 8))
    );
    Some(Label::JeroenTelJt)
}

/// Oskari Tammelin Jeskola Buzz Modular Song (`.bmx` / `.bmw`): `"Buzz"` at `@0..4`, `num_sections` (`1..=64`) at `@4..8`, `"BVER"` / `"PARA"` / `"MACH"` at `@8..12`.
pub(crate) fn verify_jeskola_buzz_module(prefix: &[u8], size: u64) -> Option<Label> {
    const NUM_SECTIONS_OFFSET: usize = 4; // @4: u32 LE section count (1..=64)
    const FIRST_SEC_OFFSET: usize = 8; // @8: 4-byte section tag ("BVER", "PARA", "MACH")
    require!(
        size >= MIN_JESKOLA_BUZZ_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"Buzz"),
        (1..=64).contains(&le32(prefix, NUM_SECTIONS_OFFSET)?),
        eq_at(prefix, FIRST_SEC_OFFSET, b"BVER")
            || eq_at(prefix, FIRST_SEC_OFFSET, b"PARA")
            || eq_at(prefix, FIRST_SEC_OFFSET, b"MACH")
    );
    Some(Label::Bmx)
}

/// JUCE AudioProcessorValueTreeState Binary XML Preset (VC2!, 2015) Raw Material Software / ROLI.
pub(crate) fn verify_juce_vc2_preset(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JUCE_VC2_PRESET_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"VC2!"),
        (8..=10_000_000).contains(&le32(prefix, 4).unwrap_or(0)),
        u8_at(prefix, 8)? == b'<'
    );
    Some(Label::JuceVc2Preset)
}

/// Ken Silverman AdLib Music (`.ksm`, 1993 Ken Silverman).
/// Layout: `@11..16` drum instruments (`0xFA..=0xFF`), `@16..32` quant (`1..=32 | 0xF0`),
/// `@43..48 == [1; 5]` 5-voice OPL2 percussion channels, `@48..64 <= 1`, `@64..80 <= 0x3F`.
pub(crate) fn verify_kens_adlib_ksm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KENS_ADLIB_KSM_SIZE,
        prefix.len() >= 80,
        get_slice(prefix, 11, 16)?.iter().all(|&b| (0xFA..=0xFF).contains(&b)),
        get_slice(prefix, 16, 32)?.iter().all(|&b| (1..=32).contains(&b) || b == 0xF0),
        eq_at(prefix, 43, &[1u8; 5]),
        get_slice(prefix, 48, 64)?.iter().all(|&b| b <= 1),
        get_slice(prefix, 64, 80)?.iter().all(|&b| b <= 0x3F),
        has_non_ascii_text_byte(prefix.get(..48)?)
    );
    Some(Label::KensAdlibKsm)
}

/// Ken Silverman Digital Music (`.kdm`, 1995 Ken Silverman).
/// Layout: `version = le32(0) == 0`, `nnotes = le32(4)` (`1..=65536`), `ntracks = le32(8)` (`1..=32`),
/// exact `size == 12 + 4 * ntracks + nnotes * (10 | 11)`, monotonic `u32 LE` timestamps.
pub(crate) fn verify_kens_digital_kdm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_KENS_DIGITAL_KDM_SIZE, prefix.len() >= 48);
    let version = le32(prefix, 0)?; // @0..4: KDM version (0)
    let nnotes = le32(prefix, 4)? as u64; // @4..8: total note events
    let ntracks = le32(prefix, 8)? as u64; // @8..12: track count
    require!(version == 0, (1..=65_536).contains(&nnotes), (1..=32).contains(&ntracks));
    let hdr_end = 12 + 4 * ntracks;
    require!(size == hdr_end + nnotes * 10 || size == hdr_end + nnotes * 11);
    let t_off = hdr_end as usize;
    let check_notes = (nnotes as usize).min(6);
    if prefix.len() >= t_off + check_notes * 4 {
        let mut prev = 0u32;
        for i in 0..check_notes {
            let t = le32(prefix, t_off + i * 4)?;
            require!(t >= prev);
            prev = t;
        }
    }
    Some(Label::KensDigitalKdm)
}

/// kometbomb Klystrack Chiptune Song (`.kt`): `"cyd!song"` at `@0..8` + version `1..=32` at `@8` + channels `1..=32` at `@9`.
pub(crate) fn verify_klystrack_module(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFFSET: usize = 8; // @8: u8 format version (1..=32)
    const CHANNELS_OFFSET: usize = 9; // @9: u8 channel count (1..=32)
    require!(
        size >= MIN_KLYSTRACK_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"cyd!song"),
        (1..=32).contains(&u8_at(prefix, VERSION_OFFSET)?),
        (1..=32).contains(&u8_at(prefix, CHANNELS_OFFSET)?)
    );
    Some(Label::Kt)
}

/// Koei Tecmo XWS Wave Stream Audio bank (`.xws`, 2010 Koei Tecmo, `"XWSFILE\0"`).
///
/// Header layout:
/// - `@0..8`: `"XWSFILE\0"`
/// - `@8`: `u8` endianness flag (`0x00` = LE, `0xFF` = BE)
/// - `@12..16`: `u32` `tables_start` (`0x30..=0x1000`)
/// - `@16..20`: `u32` `file_size` (`>= 0x30`)
/// - `@20..24`: `u32` `chunks2` (`== chunks` at `@24..28`, in `1..=4096`)
/// - `@32..36`: `u32` `table1_offset` (`>= 0x30`)
pub(crate) fn verify_koei_tecmo_xws(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 8] = b"XWSFILE\0"; // @0..8
    const ENDIAN_OFF: usize = 8; // @8: 0x00 = LE, 0xFF = BE
    const TABLES_START_OFF: usize = 12; // @12..16: u32 tables start offset
    const FILE_SIZE_OFF: usize = 16; // @16..20: u32 declared file size
    const CHUNKS2_OFF: usize = 20; // @20..24: u32 chunk table count copy
    const CHUNKS_OFF: usize = 24; // @24..28: u32 chunk table count
    const TABLE1_OFF: usize = 32; // @32..36: u32 table 1 offset

    require!(size >= MIN_KOEI_TECMO_XWS_SIZE, prefix.len() >= 36, starts(prefix, MAGIC));
    let is_be = match u8_at(prefix, ENDIAN_OFF)? {
        0x00 => false,
        0xFF => true,
        _ => return None,
    };
    let read32 = |off| if is_be { be32(prefix, off) } else { le32(prefix, off) };
    let tables_start = read32(TABLES_START_OFF)?;
    let file_size = read32(FILE_SIZE_OFF)?;
    let chunks2 = read32(CHUNKS2_OFF)?;
    let chunks = read32(CHUNKS_OFF)?;
    let table1_off = read32(TABLE1_OFF)?;
    require!(
        (0x30..=0x1000).contains(&tables_start),
        file_size >= 0x30,
        chunks == chunks2,
        (1..=4096).contains(&chunks),
        table1_off >= 0x30
    );
    Some(Label::KoeiTecmoXws)
}

/// MSX Konami SCC-Musixx Song (`.sng`): 16-byte descending envelope ramp `78 70 68 60 58 50 48 40 38 30 28 20 18 10 08 00` at `@0..16`.
pub(crate) fn verify_konami_scc_musixx(prefix: &[u8], size: u64) -> Option<Label> {
    const RAMP: &[u8] = hex2bin!("78 70 68 60 58 50 48 40 38 30 28 20 18 10 08 00"); // @0: 16B SCC envelope ramp
    require!(size >= MIN_KONAMI_SCC_MUSIXX_SIZE, prefix.len() >= 16, starts(prefix, RAMP));
    Some(Label::SccMusixx)
}

/// Native Instruments Kontakt Monolith Instrument (`.nki`, `hsin` + `DSIN`).
pub(crate) fn verify_kontakt_hsin_nki(prefix: &[u8], size: u64) -> Option<Label> {
    const KONTAKT_HSIN_SIG: &[u8; 16] = hex2bin!("01 00 00 00 68 73 69 6E 01 00 00 00 00 00 00 00"); // @8..24
    const KONTAKT_DSIN_MAGIC: &[u8; 4] = b"DSIN"; // @48..52

    require!(
        size >= MIN_KONTAKT_HSIN_NKI_SIZE,
        prefix.len() >= 52,
        (64..=0x4_0000_0000).contains(&le64(prefix, 0)?), // @0..8: container payload length
        eq_at(prefix, 8, KONTAKT_HSIN_SIG),
        eq_at(prefix, 48, KONTAKT_DSIN_MAGIC)
    );
    Some(Label::KontaktHsinNki)
}

/// Native Instruments Kontakt Lossless Compressed Wave file (`.ncw`).
pub(crate) fn verify_kontakt_ncw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KONTAKT_NCW_SIZE,
        prefix.len() >= 32,
        starts(prefix, KONTAKT_NCW_MAGIC),
        le16(prefix, KONTAKT_NCW_FLAGS_OFFSET) == Some(0),
        (1..=16).contains(&le16(prefix, KONTAKT_NCW_CHANNELS_OFFSET)?),
        matches!(le16(prefix, KONTAKT_NCW_BITS_OFFSET)?, 8 | 16 | 24 | 32),
        (4000..=384_000).contains(&le32(prefix, KONTAKT_NCW_RATE_OFFSET)?),
        le32(prefix, KONTAKT_NCW_SAMPLES_OFFSET)? >= 1,
        le32(prefix, KONTAKT_NCW_BLOCKS_OFF_OFFSET)? >= 0x78
    );
    Some(Label::KontaktNcw)
}

/// KuGou Music Encrypted Audio Container (`.kgm` or `.vpr`).
pub(crate) fn verify_kugou_music_kgm(prefix: &[u8], size: u64) -> Option<Label> {
    const KUGOU_KGM_MAGIC: &[u8; 16] = hex2bin!("7C D5 32 EB 86 02 7F 4B A8 AF A6 8E 0F FF 99 14"); // @0..16 (.kgm)
    const KUGOU_VPR_MAGIC: &[u8; 16] = hex2bin!("05 28 BC 96 E9 E4 5A 43 91 AA BD D0 7A F5 36 31"); // @0..16 (.vpr)

    require!(
        size >= MIN_KUGOU_MUSIC_KGM_SIZE,
        prefix.len() >= 24,
        starts(prefix, KUGOU_KGM_MAGIC) || starts(prefix, KUGOU_VPR_MAGIC),
        (64..=0x10_0000).contains(&le32(prefix, 16)?), // @16..20: header size (typically 0x400)
        (1..=10).contains(&le32(prefix, 20)?)          // @20..24: encryption version
    );
    Some(Label::KugouMusicKgm)
}

/// La Lossless Audio Compressor file (`.la`, 2002–2004 Michael Bevin).
pub(crate) fn verify_la_lossless_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LA_LOSSLESS_AUDIO_SIZE
            && prefix.len() >= 28
            && matches!(get_slice(prefix, 0, 4)?, b"LA02" | b"LA03" | b"LA04")
    );
    let wav_size = le32(prefix, 4)? as u64;
    let fmt_off = if prefix.len() >= 40
        && get_slice(prefix, 8, 12)? == b"WAVE"
        && le32(prefix, 12).unwrap_or(0) >= 24
        && get_slice(prefix, 16, 20)? == b"fmt "
        && le32(prefix, 20).unwrap_or(0) >= 16
    {
        24
    } else {
        8
    };
    let fmt_tag = le16(prefix, fmt_off)?;
    let channels = le16(prefix, fmt_off + 2)?;
    let sample_rate = le32(prefix, fmt_off + 4)?;
    let bits_per_sample = le16(prefix, fmt_off + 14)?;
    require!(
        wav_size >= 44
            && fmt_tag == 1
            && (1..=8).contains(&channels)
            && (4000..=192_000).contains(&sample_rate)
            && matches!(bits_per_sample, 8 | 16 | 24 | 32)
    );
    Some(Label::LaAudio)
}

/// Lab.gruppen IPX Series Amplifier DSP Preset (.ipxpreset) Lab.gruppen.
pub(crate) fn verify_labgruppen_ipxpreset(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LABGRUPPEN_IPXPRESET_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"IPX\0\0\0\0\x02\x08\0\0\0\x07\0\0\0")
    );
    Some(Label::LabgruppenIpxpreset)
}

/// Laxity Amiga Powertracker Module (`.pt`, 1991 Thomas E. Petersen).
pub(crate) fn verify_laxity_pt(prefix: &[u8], size: u64) -> Option<Label> {
    const BANNER_1: &[u8; 16] = b"YET ANOTHER COOL"; // @8..24
    const BANNER_2: &[u8; 16] = b"LAXITY PRODUCT!\x00"; // @32..48
    require!(
        size >= MIN_LAXITY_PT_SIZE,
        prefix.len() >= 56,
        be16(prefix, 6) == Some(0),
        eq_at(prefix, 8, BANNER_1),
        eq_at(prefix, 32, BANNER_2)
    );
    Some(Label::LaxityPt)
}

/// Leggless / Silents UK Amiga Music Editor Module (`.lme`): `"LME\0(c)1990 Leggless"` at `@0..20`.
pub(crate) fn verify_leggless_lme(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LEGGLESS_LME_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"LME\x00(c)1990 Leggless")
    );
    Some(Label::LegglessLme)
}

/// High Voltage Software / LEGO Racers ALP ADPCM Audio file (`.tun`, `"ALP "` + `"ADPC"`).
pub(crate) fn verify_lego_alp_audio(prefix: &[u8], size: u64) -> Option<Label> {
    const ALP_MAGIC: &[u8; 4] = b"ALP "; // @0..4: "ALP "
    const ADPC_TAG_OFFSET: usize = 8; // @8..12: "ADPC" codec tag

    require!(
        size >= MIN_LEGO_ALP_AUDIO_SIZE,
        prefix.len() >= 16,
        starts(prefix, ALP_MAGIC),
        eq_at(prefix, ADPC_TAG_OFFSET, b"ADPC")
    );
    Some(Label::LegoAlpTun)
}

/// Thalion Lionheart Amiga Music Module (`.lion`, 1992 Thalion).
pub(crate) fn verify_lionheart_lion(prefix: &[u8], size: u64) -> Option<Label> {
    const LION_TAG: &[u8; 8] = hex2bin!("21 44 FF FF 00 00 00 00"); // @32..40
    require!(
        size >= MIN_LIONHEART_LION_SIZE,
        prefix.len() >= 40,
        be32(prefix, 0) == Some(0x28),
        eq_at(prefix, 32, LION_TAG)
    );
    let o1 = be32(prefix, 4)? as u64;
    let o2 = be32(prefix, 8)? as u64;
    let o3 = be32(prefix, 12)? as u64;
    let o4 = be32(prefix, 16)? as u64;
    let o5 = be32(prefix, 20)? as u64;
    let o6 = be32(prefix, 24)? as u64;
    let o7 = be32(prefix, 28)? as u64;
    require!(
        o1 >= 0x40
            && o1 < o2
            && o2 < o3
            && o3 < o4
            && o4 <= o5
            && o5 <= o6
            && o6 == o7
            && o7 < size
    );
    Some(Label::LionheartLion)
}

/// Nir Oren Liquid Tracker Module (`.liq`): `"Liquid Module:"` at `@0..14` + `@64 == 0x1A` (DOS EOF).
pub(crate) fn verify_liquid_tracker_liq(prefix: &[u8], size: u64) -> Option<Label> {
    const DOS_EOF_OFFSET: usize = 64; // @64: 0x1A DOS EOF after 30B title and 20B editor name
    const LIQ_V0_MAGIC: &[u8; 5] = b"NO\x00\x00\x1A"; // @0..5: Liquid Tracker v0.0 header magic
    if size >= MIN_LIQUID_TRACKER_SIZE
        && prefix.len() >= 48
        && starts(prefix, LIQ_V0_MAGIC)
        && get_slice(prefix, 5, 35)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b))
    {
        return Some(Label::Liq);
    }
    require!(
        size >= MIN_LIQUID_TRACKER_SIZE,
        prefix.len() >= 66,
        starts(prefix, b"Liquid Module:"),
        u8_at(prefix, DOS_EOF_OFFSET) == Some(0x1A)
    );
    Some(Label::Liq)
}

/// Loriciel Amiga Sample Bank (`.bnk`, 1990 Michel Winogradoff / Loriciel).
pub(crate) fn verify_loriciel_bnks(prefix: &[u8], size: u64) -> Option<Label> {
    const BNKS_HDR: &[u8; 8] = hex2bin!("42 4E 4B 53 00 00 00 80"); // @0..8: "BNKS" + 0x00000080 table size
    require!(size >= MIN_LORICIEL_BNKS_SIZE, prefix.len() >= 16, starts(prefix, BNKS_HDR));
    Some(Label::LoricielBnks)
}

/// Tilman Liebchen LPAC Lossless Predictive Audio Compressor (`.pac`, TU Berlin):
/// - `@0..4`: `"LPAC"`
/// - `@4`: version (`1..=10`; v1.40 writes `0x06`)
/// - `@6..10`: `u32 BE` total sample frames (`> 0`)
/// - `@14..`: embedded uncompressed `"RIFF"...."WAVEfmt "` or `"FORM"...."AIF"` header.
pub(crate) fn verify_lpac(prefix: &[u8], size: u64) -> Option<Label> {
    let has_riff_wav = eq_at(prefix, 14, b"RIFF") && eq_at(prefix, 22, b"WAVEfmt ");
    let has_aiff = eq_at(prefix, 14, b"FORM") && eq_at(prefix, 22, b"AIF");
    require!(
        size >= MIN_LPAC_SIZE,
        prefix.len() >= 30,
        starts(prefix, LPAC_MAGIC),
        (1..=10).contains(&u8_at(prefix, 4)?),
        be32(prefix, 6)? > 0,
        has_riff_wav || has_aiff,
        has_non_ascii_text_byte(prefix.get(4..14)?)
    );
    Some(Label::Lpac)
}

/// LucasArts AdLib Audio Music (.laa).
pub(crate) fn verify_lucasarts_laa(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_LUCASARTS_LAA_SIZE && prefix.len() >= 12);
    let is_v2 = starts(prefix, b"ADL ")
        && u8_at(prefix, 4)? <= 16
        && has_non_ascii_text_byte(prefix.get(..12)?);
    let is_v1 = prefix.len() >= 16
        && u8_at(prefix, 0)? == 0xF3
        && (1..=16).contains(&u8_at(prefix, 1)?)
        && eq_at(prefix, 2, hex2bin!("00 00 41 44 C8 00"))
        && eq_at(prefix, 10, &[0u8; 6]);
    require!(is_v2 || is_v1);
    Some(Label::LucasartsLaa)
}

/// LucasArts SCUMM `SOU ` / `VCTL` Voice Audio stream (`.sou`, 1993 LucasArts).
pub(crate) fn verify_lucasarts_sou(prefix: &[u8], size: u64) -> Option<Label> {
    const SOU_MAGIC: &[u8; 8] = b"SOU \0\0\0\0"; // @0..8: Monster.SOU container header
    const VCTL_MAGIC: &[u8; 4] = b"VCTL"; // @0..4: "VCTL" lip-sync control chunk
    const VCTL_LEN_OFFSET: usize = 4; // @4..8: u32 BE chunk size (includes 8-byte header)

    require!(size >= MIN_LUCASARTS_SOU_SIZE, prefix.len() >= 32);
    if starts(prefix, SOU_MAGIC) {
        require!(eq_at(prefix, 8, b"VCTL") || eq_at(prefix, 8, b"VTTL"), be32(prefix, 12)? >= 8);
        return Some(Label::LucasartsSou);
    }
    require!(starts(prefix, VCTL_MAGIC));
    let vctl_len = be32(prefix, VCTL_LEN_OFFSET)? as usize;
    require!(
        (8..=4096).contains(&vctl_len),
        vctl_len + 16 <= prefix.len(),
        eq_at(prefix, vctl_len, b"Creative Voice File\x1A") || eq_at(prefix, vctl_len, b"VTLK")
    );
    Some(Label::LucasartsSou)
}

/// Luxonix / Sonic Cat Purity Rompler Synthesizer Patch (.pth, 2005) Luxonix.
pub(crate) fn verify_luxonix_purity_pth(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LUXONIX_PURITY_PTH_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"LUXONIX -Purity    -Patches          \x1a\0L")
    );
    Some(Label::LuxonixPurityPth)
}

/// Yannick Delwiche MadTracker 2 Module (`.mt2`): `"MT20"` at `@0..4`, `u16 LE` version `0x0200..=0x02FF` at `@8..10`, `"MadTracker 2."` at `@10..23`.
pub(crate) fn verify_madtracker2_module(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFFSET: usize = 8; // @8: u16 LE version (0x0200..=0x02FF)
    const TRACKER_NAME_OFFSET: usize = 10; // @10: "MadTracker 2."
    require!(
        size >= MIN_MADTRACKER2_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"MT20"),
        (0x0200..=0x02FF).contains(&le16(prefix, VERSION_OFFSET)?),
        eq_at(prefix, TRACKER_NAME_OFFSET, b"MadTracker 2.")
    );
    Some(Label::Mt2)
}

/// Cachet Software Amiga Mark II Music System (`.mk2`) or Darius Zendeh (`.dz`): `48 E7 00 F0 41 FA` (`MOVEM.L D0-D3,-(SP); LEA (d16,PC),A0`) + `4C D8 06 00` at `@8..12`.
pub(crate) fn verify_mark2_music_system(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MARK2_SIZE,
        prefix.len() >= 24,
        starts(prefix, hex2bin!("48 E7 00 F0 41 FA")),
        eq_at(prefix, 8, hex2bin!("4C D8 06 00"))
    );
    Some(Label::Mark2Mk2)
}

/// Mark Cooksey Amiga Music Module (`.mc`, 1988 Mark Cooksey).
pub(crate) fn verify_mark_cooksey(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MARK_COOKSEY_SIZE,
        prefix.len() >= 24,
        starts(prefix, MARK_COOKSEY_PREFIX),
        be16(prefix, 4) == Some(0x4EFB),
        matches!(be16(prefix, 6), Some(0x0002 | 0x0006)),
        be16(prefix, 8) == Some(0x6000)
    );
    Some(Label::MarkCooksey)
}

/// Passport Designs Master Tracks Pro Sequence (`.mts`): `"SCOR\0\0\0@"` at `@0..8`.
pub(crate) fn verify_master_tracks_mts(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MASTER_TRACKS_MTS_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"SCOR\x00\x00\x00@")
    );
    Some(Label::MasterTracksMts)
}

/// Kane Wood / Arkha Master Tracker Module (`.mtr`): `"MTRAC "` at `@0..6`.
pub(crate) fn verify_mastertracker_mtr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MASTERTRACKER_MTR_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"MTRAC ") || starts(prefix, b"MTRACK "),
        get_slice(prefix, 6, 24)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b))
    );
    Some(Label::MastertrackerMtr)
}

/// Taliesin Amiga MaxTrax Module (`.mxtx`): `"MXTX\0"` at `@0..5` + `u16 BE` (`1..=256`) or `u32 BE` (`1..=256`) score count at `@6`.
pub(crate) fn verify_maxtrax_module(prefix: &[u8], size: u64) -> Option<Label> {
    const MAXTRAX_MAGIC: &[u8; 5] = b"MXTX\x00"; // @0..5: MaxTrax module signature
    const FLAGS_OFFSET: usize = 5; // @5: u8 header flags/revision
    const COUNT_OFFSET: usize = 6; // @6: u16 BE (or u32 BE when flags > 0) score count (1..=256)

    require!(
        size >= MIN_MAXTRAX_SIZE,
        prefix.len() >= 12,
        starts(prefix, MAXTRAX_MAGIC),
        (1..=256).contains(&be16(prefix, COUNT_OFFSET)?)
            || (u8_at(prefix, FLAGS_OFFSET).unwrap_or(0) > 0
                && (1..=256).contains(&be32(prefix, COUNT_OFFSET)?))
    );
    Some(Label::Mxtx)
}

/// Medley Sound Amiga Module (`.mso`): `"MSOB"` at `@0..4` + increasing `u32 BE` offsets at `@4..16` (`< size`).
pub(crate) fn verify_medley_mso(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_MEDLEY_MSO_SIZE && prefix.len() >= 16 && starts(prefix, b"MSOB"));
    let o1 = be32(prefix, 4)? as u64;
    let o2 = be32(prefix, 8)? as u64;
    let o3 = be32(prefix, 12)? as u64;
    require!(o1 >= 16 && o1 < o2 && o2 < o3 && o3 < size);
    Some(Label::MedleyMso)
}

/// MegaStation Compressed MIDI Module (`.msm`, 1997, `"SNGM0.10"` + `"COMP"`/`"NORM"`).
pub(crate) fn verify_megastation_msm(prefix: &[u8], size: u64) -> Option<Label> {
    const MSM_MAGIC: &[u8; 8] = b"SNGM0.10"; // @0..8: MegaStation MIDI header
    require!(
        size >= MIN_MEGASTATION_SIZE,
        prefix.len() >= 16,
        starts(prefix, MSM_MAGIC),
        (eq_at(prefix, 8, b"COMP") && u8_at(prefix, 12)? == 0x78)
            || (eq_at(prefix, 8, b"NORM") && has_non_ascii_text_byte(prefix.get(12..16)?))
    );
    Some(Label::MegastationMsm)
}

/// Simplet / Fatal Design Atari Falcon MegaTracker Module (`.mgt`): `"MGT\x11\xBDMCS"` at `@0..8`.
pub(crate) fn verify_megatracker_mgt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MEGATRACKER_MGT_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"MGT\x11\xBDMCS")
    );
    Some(Label::MegatrackerMgt)
}

/// Jean-Paul Mikkers MikMod UniMod Module (`.uni`): `"UN04"`, `"UN05"`, or `"UN06"` at `@0..4` + `num_channels` (`1..=64`) at `@4`.
pub(crate) fn verify_mikmod_uni_module(prefix: &[u8], size: u64) -> Option<Label> {
    const CHANNELS_OFFSET: usize = 4; // @4: u8 number of channels (1..=64)
    require!(
        size >= MIN_MIKMOD_UNI_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"UN04") || starts(prefix, b"UN05") || starts(prefix, b"UN06"),
        (1..=64).contains(&u8_at(prefix, CHANNELS_OFFSET)?),
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::UniMikmod)
}

/// MKJamz AdLib FM Module (`.mkj`): `"MKJamz"` at `@0..6`.
pub(crate) fn verify_mkjamz_mkj(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_MKJAMZ_SIZE, prefix.len() >= 16, starts(prefix, b"MKJamz"));
    Some(Label::MkjamzMkj)
}

/// Søren Lund MLAT AdLib Tracker Module (`.mad`): `"MAD+"` at `@0..4` + 8-byte ASCII driver identifier at `@4..12` + binary song data.
pub(crate) fn verify_mlat_mad(prefix: &[u8], size: u64) -> Option<Label> {
    const MLAT_MAD_MAGIC: &[u8; 4] = b"MAD+"; // @0..4: MLAT AdLib Tracker header magic
    const DRIVER_TAG_END: usize = 12; // @4..12: 8-char ASCII driver identifier (e.g. "MEM4    ")

    require!(
        size >= MIN_MLAT_MAD_SIZE,
        prefix.len() >= 16,
        starts(prefix, MLAT_MAD_MAGIC),
        get_slice(prefix, 4, DRIVER_TAG_END)?
            .iter()
            .all(|&b| b == b' ' || b.is_ascii_alphanumeric()),
        has_non_ascii_text_byte(prefix.get(DRIVER_TAG_END..prefix.len().min(64))?)
    );
    Some(Label::MlatMad)
}

/// Un4seen Developments MO3 Compressed Tracker Module (.mo3).
pub(crate) fn verify_mo3(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_MO3_SIZE && prefix.len() >= 16);
    let music_hdr_size = le32(prefix, 4)?;
    require!((16..=0x0400_0000).contains(&music_hdr_size));
    Some(Label::Mo3)
}

/// Mobile eXtensible Music Format (.mxmf, PRONOM fmt/961, LoC fdd000121).
pub(crate) fn verify_mobile_xmf(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_MOBILE_XMF_SIZE
        && prefix.len() >= 16
        && starts(prefix, b"XMF_2.00")
        && prefix.get(8..16)?.contains(&0)
    {
        Some(Label::Mxmf)
    } else {
        None
    }
}

/// Trixter MONOTONE PC Speaker / OPL Tracker Module (`.mon`, 2007 Jim Leonard).
pub(crate) fn verify_monotone_module(prefix: &[u8], size: u64) -> Option<Label> {
    const MONOTONE_MAGIC: &[u8; 9] = b"\x08MONOTONE"; // @0..9: Turbo Pascal String[8] signature
    const VER_OFF: usize = 91; // @91: u8 format version (1)
    const PATS_OFF: usize = 92; // @92: u8 total patterns (1..=128)
    const TRKS_OFF: usize = 93; // @93: u8 total tracks (1..=12)
    const CELL_OFF: usize = 94; // @94: u8 cell byte size (2)

    require!(
        size >= MIN_MONOTONE_SIZE,
        prefix.len() >= 96,
        starts(prefix, MONOTONE_MAGIC),
        u8_at(prefix, VER_OFF)? == 1,
        (1..=128).contains(&u8_at(prefix, PATS_OFF)?),
        (1..=12).contains(&u8_at(prefix, TRKS_OFF)?),
        u8_at(prefix, CELL_OFF)? == 2
    );
    Some(Label::MonotoneMon)
}

/// Remco Schrijvers MSX MoonBlaster Module (`.mbm`): `@5..10 == "-pc1-"` or `@3..11 == "\x11\x11\x14"` + `@7..11 == "\xC0\xB2\x01\xF4"`.
pub(crate) fn verify_moonblaster_module(prefix: &[u8], size: u64) -> Option<Label> {
    const MBM_CH_MASK: &[u8; 16] = hex2bin!("10 08 04 02 01 18 14 12 11 0C 0A 09 06 05 03 00"); // @328 (User) / @329 (Edit): MSX-MUSIC channel bitmask table
    require!(size >= MIN_MOONBLASTER_SIZE && prefix.len() >= 16);
    let is_pc1 = eq_at(prefix, 5, b"-pc1-");
    let is_legacy_voice = (eq_at(prefix, 3, hex2bin!("11 11 14"))
        && eq_at(prefix, 7, hex2bin!("C0 B2 01 F4")))
        || (eq_at(prefix, 4, hex2bin!("11 11 14")) && eq_at(prefix, 8, hex2bin!("C0 B2 01 F4")));
    if is_pc1 || is_legacy_voice {
        return Some(Label::MoonblasterMbm);
    }
    if prefix.len() >= 346 {
        let ch_off = if u8_at(prefix, 0)? == 0xFF { 329usize } else { 328usize };
        if eq_at(prefix, ch_off, MBM_CH_MASK) {
            return Some(Label::MoonblasterMbm);
        }
    }
    None
}

/// MPU-401 Tracker Compressed Module (`.mtk`): `"mpu401tr\x92kk\xEEr@data"` at `@0..18`.
pub(crate) fn verify_mpu401_mtk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MPU401_MTK_SIZE,
        prefix.len() >= 18,
        starts(prefix, b"mpu401tr"),
        eq_at(prefix, 9, b"kk\xEEr@data")
    );
    Some(Label::Mpu401Mtk)
}

/// MSX FAC SoundTracker Music Module (`.mus` / `.sm1` / `.sm2`, 1991 FAC).
pub(crate) fn verify_msx_fac_soundtracker(prefix: &[u8], size: u64) -> Option<Label> {
    const BSAVE_HDR: &[u8; 7] = hex2bin!("FE 00 80 FF BF 00 80"); // @0..7: BSAVE 0x8000..0xBFFF exec 0x8000
    require!(
        size >= MIN_MSX_FAC_SOUNDTRACKER_SIZE,
        prefix.len() >= 32,
        starts(prefix, BSAVE_HDR),
        get_slice(prefix, 7, 32)?.iter().any(|&b| b != 0)
    );
    Some(Label::MsxFacMus)
}

/// MSX / Sega Master System KSS Chiptune Audio (KSCC / KSSX).
pub(crate) fn verify_msx_kss_audio(prefix: &[u8], size: u64) -> Option<Label> {
    const INIT_LEN: usize = 6; // @6: u16le initial data length
    const EXTRA_HDR_SZ: usize = 14; // @14: u8 extra header size (0x00 or 0x10)
    const DEVICE_FLAGS: usize = 15; // @15: u8 sound chip flags
    require!(
        size >= MIN_KSS_AUDIO_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"KSCC") || starts(prefix, b"KSSX"),
        has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    let init_len = le16(prefix, INIT_LEN)?;
    let extra_sz = u8_at(prefix, EXTRA_HDR_SZ)?;
    let dev_flags = u8_at(prefix, DEVICE_FLAGS)?;
    let is_kscc = starts(prefix, b"KSCC");
    if is_kscc {
        require!(extra_sz == 0, (dev_flags & 0x80) == 0);
    } else {
        require!((extra_sz & 0xEF) == 0);
    }
    let hdr_total = 16usize + extra_sz as usize;
    require!(init_len > 0, (hdr_total as u64) + 4 <= size);
    Some(Label::Kss)
}

/// MSX MoonBlaster for MoonSound OPL4 Music (.mbm).
pub(crate) fn verify_msx_mbms_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MSX_MBMS_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"MBMS\x10"),
        u8_at(prefix, 5)? > 0,
        le16(prefix, 6)? > 0
    );
    Some(Label::Mbms)
}

/// MSX MGSDRV Chiptune Music Data (.mgs).
pub(crate) fn verify_msx_mgsdrv_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MSX_MGS_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"MGS"),
        get_slice(prefix, 3, 6)?.iter().all(|b| matches!(b, 0x20..=0x7E)),
        get_slice(prefix, 6, 8)? == b"\r\n",
        has_non_ascii_text_byte(prefix.get(8..prefix.len().min(512))?)
    );
    Some(Label::Mgsdrv)
}

/// MSX K-kaz MPK Chiptune Music Data (.mpk).
pub(crate) fn verify_msx_mpk_audio(prefix: &[u8], size: u64) -> Option<Label> {
    const MPK_MAGIC: &[u8; 3] = b"MPK"; // @0..3: MSX K-kaz MPK magic
    const BINARY_SCAN_END: usize = 512; // @8..512: binary channel pointer table & song data after optional title

    require!(
        size >= MIN_MSX_MPK_SIZE,
        prefix.len() >= 16,
        starts(prefix, MPK_MAGIC),
        get_slice(prefix, 3, 6)?.iter().all(|b| b.is_ascii_digit()),
        get_slice(prefix, 6, 8)? == b"\r\n",
        has_non_ascii_text_byte(prefix.get(8..prefix.len().min(BINARY_SCAN_END))?)
    );
    Some(Label::MsxMpk)
}

/// MSX / DOS OPLLDRV FM Chiptune (`.opx`): `"TSW12 "` or `"OPXAPI for DOS v"` at `@0`.
pub(crate) fn verify_msx_oplldrv_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MSX_OPLLDRV_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"TSW12 ") || starts(prefix, b"OPXAPI for DOS v"),
        has_non_ascii_text_byte(prefix.get(6..prefix.len().min(64))?)
    );
    Some(Label::Opx)
}

/// MSX / DOS OPLLDRV FM Chiptune (`.opx`) with 126-byte DOS `TYPE`-able title block followed at `@126`
/// by binary OPLLDRV header and strictly increasing `u16 LE` FM channel pointer table (`@136..142`).
pub(crate) fn verify_msx_oplldrv_offset(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 160 && prefix.len() >= 144);
    let has_type_banner = eq_at(prefix, 53, b"\r\n")
        && eq_at(prefix, 108, b"\r\n")
        && eq_at(prefix, 123, b"\r\n\x1A");
    let has_blank_banner = get_slice(prefix, 0, 126)? == [b' '; 126];
    require!(
        has_type_banner || has_blank_banner,
        u8_at(prefix, 134)? <= 2,
        has_non_ascii_text_byte(prefix.get(126..140)?)
    );
    let ch0 = le16(prefix, 136)?;
    let ch1 = le16(prefix, 138)?;
    let ch2 = le16(prefix, 140)?;
    require!(ch0 < ch1 && ch1 < ch2);
    Some(Label::Opx)
}

/// MTC Chiptune Module (`.mtc`): `"MTC1"` at `@0..4` + `"PROP"` at `@8..12`.
pub(crate) fn verify_mtc_chiptune(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MTC_CHIPTUNE_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"MTC1"),
        eq_at(prefix, 8, b"PROP")
    );
    Some(Label::MtcChiptune)
}

/// Activision The Music Studio Song (`.sng`): `\xCDMstudio\xCD\x02Blocks` at `@0..16`.
pub(crate) fn verify_music_studio_sng(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = b"\xCDMstudio\xCD\x02Blocks"; // @0: 16B header
    require!(size >= MIN_MUSIC_STUDIO_SNG_SIZE, prefix.len() >= 16, starts(prefix, MAGIC));
    Some(Label::MusicStudioSng)
}

/// N. J. Luuring Jr. The Musical Enlightenment Amiga Module (`.tme`, 1990 N. J. Luuring Jr.).
pub(crate) fn verify_musical_enlightenment_tme(prefix: &[u8], size: u64) -> Option<Label> {
    const TME_VOL_RAMP: &[u8; 16] = hex2bin!("00 00 05 0F 00 00 05 0F 00 00 05 0F 00 00 05 0F"); // @12..28
    require!(
        size >= MIN_MUSICAL_ENLIGHTENMENT_TME_SIZE,
        prefix.len() >= 28,
        (be32(prefix, 0)? as u64) == size,
        u8_at(prefix, 4)? == 0,
        (1..=16).contains(&u8_at(prefix, 6)?),
        u8_at(prefix, 7)? == 0x1F,
        eq_at(prefix, 12, TME_VOL_RAMP)
    );
    Some(Label::MusicalEnlightenmentTme)
}

/// Conny & Christian Cyrus Musicline Editor Amiga Module (`"MLEDMODL"` + variable `MODL` size + `"VERS"`).
pub(crate) fn verify_musicline_module(prefix: &[u8], size: u64) -> Option<Label> {
    const MODL_LEN_OFFSET: usize = 8; // @8: u32 BE MODL header payload length (0, 4, or 8)
    require!(size >= MIN_MUSICLINE_SIZE, prefix.len() >= 16, starts(prefix, b"MLEDMODL"));
    let modl_len = be32(prefix, MODL_LEN_OFFSET)? as usize;
    require!(modl_len <= 64, prefix.len() >= 16 + modl_len, eq_at(prefix, 12 + modl_len, b"VERS"));
    Some(Label::MusiclineMl)
}

/// MVS Tracker Module (`.mus`): `"MVSM1"` at `@0..5` + printable ASCII title at `@5..12`.
pub(crate) fn verify_mvs_tracker(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MVS_TRACKER_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"MVSM1"),
        u8_at(prefix, 5)?.is_ascii_graphic()
    );
    Some(Label::MvsTracker)
}

/// MVX Tracker Module (`.mvm`): `"MVM\0"` at `@0..4`.
pub(crate) fn verify_mvx_mvm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MVX_MVM_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"MVM\x00"),
        le16(prefix, 4)? > 0
    );
    Some(Label::MvxMvm)
}

/// NAMX Neural Amp Modeler Extended Profile (.namx).
pub(crate) fn verify_namx_profile(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NAMX_PROFILE_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"NAMX01\0\0"),
        has_non_ascii_text_byte(prefix.get(8..28)?)
    );
    Some(Label::NamxProfile)
}

/// Bananmos NES NerdTracker II Module (`.ned`): `"NED: "` at `@0..5` + ASCII title at `@5..21`.
pub(crate) fn verify_nerdtracker2_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NERDTRACKER2_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"NED: "),
        has_non_ascii_text_byte(prefix.get(..24)?)
    );
    Some(Label::Ned)
}

/// Nintendo Entertainment System Sound Format (.nsf / .nsfe).
pub(crate) fn verify_nes_nsf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_NES_NSF_SIZE && prefix.len() >= 28);
    if starts(prefix, b"NESM\x1A") {
        require!(size >= 132 && prefix.len() >= 128);
        let version = u8_at(prefix, 5)?;
        let total_songs = u8_at(prefix, 6)?;
        let start_song = u8_at(prefix, 7)?;
        let load_addr = le16(prefix, 8)?;
        let init_addr = le16(prefix, 10)?;
        let play_addr = le16(prefix, 12)?;
        require!(
            (1..=2).contains(&version)
                && total_songs >= 1
                && start_song >= 1
                && load_addr >= 0x6000
                && init_addr >= 0x6000
                && play_addr >= 0x6000
        );
    } else if starts(prefix, b"NSFE") {
        let first_chunk_len = le32(prefix, 4)? as u64;
        require!(
            prefix.get(4..8)?.contains(&0)
                && get_slice(prefix, 8, 12)?
                    .iter()
                    .all(|&b| b.is_ascii_alphanumeric() || b == b' ')
                && first_chunk_len + 12 <= size
        );
    } else {
        return None;
    }
    Some(Label::Nsf)
}

/// Nintendo GameCube / Wii AST Audio Stream (.ast).
pub(crate) fn verify_nintendo_ast(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NINTENDO_AST_SIZE,
        prefix.len() >= 96,
        starts(prefix, b"STRM"),
        get_slice(prefix, 64, 68)? == b"BLCK",
        get_slice(prefix, 80, 96)?.iter().all(|&b| b == 0)
    );
    let data_sz = be32(prefix, 4)?;
    let codec = be16(prefix, 8)?;
    let bit_depth = be16(prefix, 10)?;
    let channels = be16(prefix, 12)?;
    let sample_rate = be32(prefix, 16)?;
    require!(
        data_sz > 0,
        codec <= 1,
        matches!(bit_depth, 4 | 8 | 16),
        (1..=8).contains(&channels),
        (4_000..=96_000).contains(&sample_rate)
    );
    Some(Label::NintendoAst)
}

/// Nintendo GameCube / Wii Sound Archive (.brsar).
pub(crate) fn verify_nintendo_brsar(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BRSAR_SIZE && prefix.len() >= 16);
    require!(starts(prefix, b"RSAR"));

    let bom = be16(prefix, 4)?;
    // Byte order mark must be 0xFEFF (BE) or 0xFFFE (LE)
    require!(bom == 0xFEFF || bom == 0xFFFE);

    let is_be = bom == 0xFEFF;
    let file_size = if is_be { be32(prefix, 8)? } else { le32(prefix, 8)? };
    let header_size = if is_be { be16(prefix, 12)? } else { le16(prefix, 12)? };

    require!(header_size >= 64);
    if file_size > 0 {
        require!((file_size as u64) <= size + 1024);
    }

    Some(Label::Brsar)
}

/// Nintendo Wii Binary Revolution Stream Audio (.brstm).
pub(crate) fn verify_nintendo_brstm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_NINTENDO_BRSTM_SIZE, prefix.len() >= 68, starts(prefix, b"RSTM"));
    let bom = be16(prefix, 4)?;
    require!(bom == 0xFEFF || bom == 0xFFFE);
    let is_be = bom == 0xFEFF;
    let file_sz = if is_be { be32(prefix, 8)? } else { le32(prefix, 8)? } as u64;
    let hdr_sz = if is_be { be16(prefix, 12)? } else { le16(prefix, 12)? };
    let num_blks = if is_be { be16(prefix, 14)? } else { le16(prefix, 14)? };
    require!(
        hdr_sz == 0x40,
        (2..=8).contains(&num_blks),
        file_sz >= MIN_NINTENDO_BRSTM_SIZE,
        get_slice(prefix, 0x40, 0x44)? == b"HEAD"
    );
    Some(Label::Brstm)
}

/// Nintendo Wii Binary Revolution Wave Audio (.brwav).
pub(crate) fn verify_nintendo_brwav(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_NINTENDO_BRWAV_SIZE, prefix.len() >= 40, starts(prefix, b"RWAV"));
    let bom = be16(prefix, 4)?;
    require!(bom == 0xFEFF || bom == 0xFFFE);
    let is_be = bom == 0xFEFF;
    let file_sz = if is_be { be32(prefix, 8)? } else { le32(prefix, 8)? } as u64;
    let hdr_sz = if is_be { be16(prefix, 12)? } else { le16(prefix, 12)? };
    let num_blks = if is_be { be16(prefix, 14)? } else { le16(prefix, 14)? };
    require!(
        hdr_sz == 0x20,
        num_blks == 2,
        file_sz >= MIN_NINTENDO_BRWAV_SIZE,
        get_slice(prefix, 0x20, 0x24)? == b"INFO"
    );
    Some(Label::Brwav)
}

/// Nintendo Switch / Wii U Binary Wave Audio file (`.bwav`, 2017 Nintendo, `"BWAV"`).
///
/// Header layout:
/// - `@0..4`: `"BWAV"`
/// - `@4..6`: BOM (`FF FE` LE or `FE FF` BE)
/// - `@6..8`: `u16` `version` (`1..=0x0100`)
/// - `@12..14`: `u16` `prefetch_flag` (`0..=1`)
/// - `@14..16`: `u16` `channels` (`1..=16`)
/// - `@16..18`: `u16` `codec` (`0` = PCM16, `1` = DSP-ADPCM, `2` = Opus)
/// - `@20..24`: `u32` `sample_rate` (`1000..=384_000`)
pub(crate) fn verify_nintendo_bwav(prefix: &[u8], size: u64) -> Option<Label> {
    const BOM_OFF: usize = 4; // @4..6: 0xFEFF (LE) or 0xFFFE (BE)
    const VER_OFF: usize = 6; // @6..8: u16 version
    const PREFETCH_OFF: usize = 12; // @12..14: u16 prefetch flag (0..=1)
    const CHANNELS_OFF: usize = 14; // @14..16: u16 channel count (1..=16)
    const CODEC_OFF: usize = 16; // @16..18: u16 codec id (0..=2)
    const RATE_OFF: usize = 20; // @20..24: u32 sample rate Hz

    require!(size >= MIN_NINTENDO_BWAV_SIZE, prefix.len() >= 24, starts(prefix, b"BWAV"));
    let is_be = match le16(prefix, BOM_OFF)? {
        0xFEFF => false,
        0xFFFE => true,
        _ => return None,
    };
    let read16 = |off| if is_be { be16(prefix, off) } else { le16(prefix, off) };
    let read32 = |off| if is_be { be32(prefix, off) } else { le32(prefix, off) };
    let version = read16(VER_OFF)?;
    let prefetch = read16(PREFETCH_OFF)?;
    let channels = read16(CHANNELS_OFF)?;
    let codec = read16(CODEC_OFF)?;
    let sample_rate = read32(RATE_OFF)?;
    require!(
        (1..=0x0100).contains(&version),
        prefetch <= 1,
        (1..=16).contains(&channels),
        codec <= 2,
        (1000..=384_000).contains(&sample_rate)
    );
    Some(Label::NintendoBwav)
}

/// NIST SPHERE Speech Audio (`"NIST_1A\n   1024\n"`).
pub(crate) fn verify_nist_sphere_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NIST_SPHERE_SIZE,
        starts(prefix, b"NIST_1A\n   1024\n") || starts(prefix, b"NIST_1A\r\n   1024\r\n")
    );
    Some(Label::NistSphere)
}

/// C-Lab Notator / Creator MIDI song arrangement.
pub(crate) fn verify_notator_song(prefix: &[u8], size: u64) -> Option<Label> {
    // Atari ST C-Lab Notator .son header: 0x3B9E BE, byte 2 == 0x00, tempo BPM at 6..8 in 10..=400, reserved 8..16 all zeros
    if size >= MIN_NOTATOR_SONG_HEADER_SIZE
        && prefix.len() >= MIN_NOTATOR_SONG_HEADER_SIZE as usize
        && u8_at(prefix, 0)? == 0x3B
        && u8_at(prefix, 1)? == 0x9E
        && u8_at(prefix, 2)? == 0x00
        && (10..=400).contains(&be16(prefix, 6).unwrap_or(0))
        && get_slice(prefix, 8, MIN_NOTATOR_SONG_HEADER_SIZE as usize)? == NOTATOR_RESERVED_ZERO_PAD
    {
        Some(Label::NotatorSong)
    } else {
        None
    }
}

/// Amiga NTSP-system Audio Module (`.ntsp`): `"SPNT\0"` at `@0..5` + `00*8` at `@8..16`.
pub(crate) fn verify_ntsp_system(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NTSP_SYSTEM_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"SPNT\x00"),
        be64(prefix, 8) == Some(0)
    );
    Some(Label::NtspSystem)
}

/// Karsten Obarski Synth Pack Bank (`.osp`, 1988 Karsten Obarski).
pub(crate) fn verify_obarski_osp(prefix: &[u8], size: u64) -> Option<Label> {
    const OSP_MAGIC: &[u8; 12] = b"OBISYNTHPACK"; // @0..12: Obarski Synth Pack signature
    require!(
        size >= MIN_OBARSKI_OSP_SIZE,
        prefix.len() >= 20,
        starts(prefix, OSP_MAGIC),
        has_non_ascii_text_byte(prefix.get(12..20)?)
    );
    Some(Label::ObarskiOsp)
}

/// Onyx Music File (`.omf`): `"Onyx Music File\x1A"` at `@0..16`.
pub(crate) fn verify_onyx_omf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ONYX_OMF_SIZE, prefix.len() >= 20, starts(prefix, b"Onyx Music File\x1A"));
    Some(Label::OmfOnyx)
}

/// OptimFROG Lossless Audio (.ofr / .ofs).
pub(crate) fn verify_optimfrog(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_OPTIMFROG_SIZE && prefix.len() >= 20);
    require!(starts(prefix, b"OFR ") || starts(prefix, b"OFRX"));

    let channels = u8_at(prefix, 15)?;
    let sample_rate = le32(prefix, 16)?;

    // Valid channels: 1 (mono), 2 (stereo), up to 8 (surround)
    require!((1..=8).contains(&channels));
    // Valid sample rates: 4000 Hz to 384000 Hz
    require!((4000..=384000).contains(&sample_rate));

    Some(Label::Ofr)
}

/// Studio Pixel Cave Story Organya Music (`.org`): `"Org-01"`, `"Org-02"`, or `"Org-03"` at `@0..6` + `wait` (`1..=2000`) at `@6..8`.
pub(crate) fn verify_organya_audio(prefix: &[u8], size: u64) -> Option<Label> {
    const WAIT_OFFSET: usize = 6; // @6: u16 LE wait/tempo ms per tick (1..=2000)
    const STEPS_OFFSET: usize = 8; // @8: u8 steps per bar (1..=32)
    const BEATS_OFFSET: usize = 9; // @9: u8 beats per step (1..=32)
    require!(
        size >= MIN_ORGANYA_SIZE,
        prefix.len() >= 18,
        starts(prefix, b"Org-01") || starts(prefix, b"Org-02") || starts(prefix, b"Org-03"),
        (1..=2000).contains(&le16(prefix, WAIT_OFFSET)?),
        (1..=32).contains(&u8_at(prefix, STEPS_OFFSET)?),
        (1..=32).contains(&u8_at(prefix, BEATS_OFFSET)?)
    );
    Some(Label::Org)
}

/// Palladix Amiga Audio Module (`.pcs`): `"PLX\0"` at `@0..4` + increasing `u16 BE` offsets at `@8..16`.
pub(crate) fn verify_palladix_pcs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PALLADIX_PCS_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"PLX\x00") || starts(prefix, b"S.")
    );
    if starts(prefix, b"PLX\x00") {
        let be_ok = match (be16(prefix, 8), be16(prefix, 10), be16(prefix, 12)) {
            (Some(o1), Some(o2), Some(o3)) => o1 > 0 && o1 < o2 && o2 < o3,
            _ => false,
        };
        let le_ok = match (le16(prefix, 8), le16(prefix, 10), le16(prefix, 12)) {
            (Some(o1), Some(o2), Some(o3)) => o1 > 0 && o1 < o2 && o2 < o3,
            _ => false,
        };
        require!(be_ok || le_ok);
        return Some(Label::PalladixPcs);
    }
    None
}

/// Paul Robotham Amiga Music Module (`.dat`, 1994 Paul Robotham).
pub(crate) fn verify_paul_robotham_dat(prefix: &[u8], size: u64) -> Option<Label> {
    const PR_PREFIX_12: &[u8; 12] = hex2bin!("00 04 00 01 00 03 00 19 00 00 05 54"); // @0..12
    const PR_CHAN_OFFS: &[u8; 16] = hex2bin!("00 00 04 54 00 00 01 54 00 00 02 54 00 00 03 54"); // @24..40
    require!(
        size >= MIN_PAUL_ROBOTHAM_DAT_SIZE,
        prefix.len() >= 40,
        starts(prefix, PR_PREFIX_12),
        eq_at(prefix, 24, PR_CHAN_OFFS)
    );
    let o12 = be32(prefix, 12)? as u64;
    let o16 = be32(prefix, 16)? as u64;
    let o20 = be32(prefix, 20)? as u64;
    require!(o12 >= 0x0554 && o12 < o16 && o16 < o20 && o20 < size);
    Some(Label::PaulRobothamDat)
}

/// NEC PC-98 FMP ADPCM Sample Bank (`.pvi`): `"PVI1"` / `"PVI2"` at `@0..4`, `@11..16 == 00*5`.
pub(crate) fn verify_pc98_pvi_sample_bank(prefix: &[u8], size: u64) -> Option<Label> {
    const RESERVED_12: usize = 12; // @12: u32 LE reserved zero
    require!(
        size >= MIN_PC98_PVI_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"PVI2") || starts(prefix, b"PVI1"),
        le32(prefix, RESERVED_12) == Some(0)
    );
    Some(Label::Pvi)
}

/// NEC PC-88 / PC-98 S98 FM Sound Chip Register Log (.s98).
pub(crate) fn verify_pc98_s98_audio(prefix: &[u8], size: u64) -> Option<Label> {
    const COMPRESSING: usize = 12; // @12: u32le compressing (0 in v3)
    const TAG_OFFSET: usize = 16; // @16: u32le song title / UTF-8 tag offset
    const DUMP_OFFSET: usize = 20; // @20: u32le register dump data offset
    const LOOP_OFFSET: usize = 24; // @24: u32le loop point offset
    require!(
        size >= MIN_S98_AUDIO_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"S98"),
        matches!(u8_at(prefix, 3)?, b'0'..=b'3')
    );
    let comp = le32(prefix, COMPRESSING)?;
    let tag_off = le32(prefix, TAG_OFFSET)? as u64;
    let dump_off = le32(prefix, DUMP_OFFSET)? as u64;
    let loop_off = le32(prefix, LOOP_OFFSET)? as u64;
    if u8_at(prefix, 3)? == b'3' {
        require!(comp == 0);
    } else {
        require!(comp <= 65_536);
    }
    require!(
        (0x1C..size).contains(&dump_off),
        tag_off == 0 || (0x1C..size).contains(&tag_off),
        loop_off == 0 || (dump_off..size).contains(&loop_off)
    );
    Some(Label::S98)
}

/// PC Engine / TurboGrafx-16 Hudson Entertainment System Sound Format (.hes).
pub(crate) fn verify_pc_engine_hes_audio(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION: usize = 4; // @4: u8 version (must be 0)
    const SUBHDR_MAGIC: usize = 16; // @16..20: "DATA"
    const DATA_SIZE: usize = 20; // @20: u32le data_size
    const RESERVED: usize = 28; // @28: u32le reserved (0)
    require!(
        size >= MIN_HES_AUDIO_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"HESM"),
        u8_at(prefix, VERSION)? == 0,
        get_slice(prefix, SUBHDR_MAGIC, SUBHDR_MAGIC + 4)? == b"DATA"
    );
    let data_sz = le32(prefix, DATA_SIZE)? as u64;
    let reserved = le32(prefix, RESERVED)?;
    require!(data_sz > 0, data_sz <= size, reserved == 0);
    Some(Label::Hes)
}

/// Single Cell Software Caustic PCMSynth Preset (`.pcmsynth`, `PCMS` + `CCOL` + `CTRL`).
pub(crate) fn verify_pcms_ccol_wk(prefix: &[u8], size: u64) -> Option<Label> {
    const PCMS_HDR_MAGIC: &[u8; 8] = hex2bin!("00 00 02 00 50 43 4D 53"); // @0..8: chunk hdr + "PCMS"
    const CCOL_HDR_MAGIC: &[u8; 8] = hex2bin!("00 00 02 00 43 43 4F 4C"); // @12..20: chunk hdr + "CCOL"
    const CTRL_CHUNK_MAGIC: &[u8; 4] = b"CTRL"; // @24..28

    require!(
        size >= MIN_PCMS_CCOL_WK_SIZE,
        prefix.len() >= 28,
        starts(prefix, PCMS_HDR_MAGIC),
        le32(prefix, 8)? >= 32,
        eq_at(prefix, 12, CCOL_HDR_MAGIC),
        eq_at(prefix, 24, CTRL_CHUNK_MAGIC)
    );
    Some(Label::CausticPcmsynthWk)
}

/// Crypton Future Media Piapro Studio NT Voice Database (.ntdb, 2020) Crypton Future Media.
pub(crate) fn verify_piapro_ntdb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PIAPRO_NTDB_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"NTDB\0\0\0\x01\x07\0\0\0\0\0\0\0")
    );
    Some(Label::PiaproNtdb)
}

/// Pierre Adane Packer Amiga Module (`.pap`, 1990 Pierre Adane).
pub(crate) fn verify_pierre_adane_pap(prefix: &[u8], size: u64) -> Option<Label> {
    const PAP_MIN_SMP_END: usize = 8 + 8 * 8; // 8B track ptrs + first 8 x 8B sample descriptors
    require!((MIN_PIERRE_ADANE_PAP_SIZE..=65_536).contains(&size), prefix.len() >= PAP_MIN_SMP_END);
    let o0 = be16(prefix, 0)? as u64;
    let o1 = be16(prefix, 2)? as u64;
    let o2 = be16(prefix, 4)? as u64;
    let o3 = be16(prefix, 6)? as u64;
    require!(
        o0 >= PAP_MIN_SMP_END as u64 && o0 < o1 && o1 < o2 && o2 < o3 && o3 < size,
        o0.is_multiple_of(2)
            && o1.is_multiple_of(2)
            && o2.is_multiple_of(2)
            && o3.is_multiple_of(2)
    );
    let ch_len = size - o3;
    require!(
        ch_len >= 2 && ch_len.is_multiple_of(2) && ch_len <= 512,
        o3 - o2 == ch_len,
        o2 - o1 == ch_len,
        o1 - o0 == ch_len || o1 - o0 == ch_len + 2
    );
    let mut active = 0usize;
    for i in 0..8usize {
        let off = 8 + i * 8;
        let len_w = be16(prefix, off)?;
        let finetune = u8_at(prefix, off + 2)?;
        let vol = u8_at(prefix, off + 3)?;
        let loop_start = be16(prefix, off + 4)?;
        let loop_len = be16(prefix, off + 6)?;
        require!(finetune <= 15, vol <= 0x40);
        if len_w > 0 {
            require!(loop_start <= len_w, loop_len >= 1);
            active += 1;
        }
    }
    require!(active >= 1);
    Some(Label::PierreAdanePap)
}

/// Pixel Painters AdLib Music (`.fmf`, 1993 Pixel Painters).
/// Layout: `n = le16(0)` (`2..=32`) followed by `n × 16`B OPL2 patches (`p[4..8]` all nonzero ADSR with nonzero carrier attack rate `(p[5] >> 4) != 0`, `p[8..10] <= 3`, `(p[10] & 0xF1) == 0`, `+11..+16 == [0; 5]`).
pub(crate) fn verify_pixel_painters_fmf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PIXEL_PAINTERS_FMF_SIZE, prefix.len() >= 34);
    let n = le16(prefix, 0)? as usize;
    require!((2..=32).contains(&n), (2 + n * 16) as u64 <= size, prefix.len() >= 2 + n * 16);
    for i in 0..n {
        let p = get_slice(prefix, 2 + i * 16, 2 + (i + 1) * 16)?;
        require!(
            p[4..8].iter().all(|&b| b != 0),
            (p[5] >> 4) != 0,
            p[8] <= 3 && p[9] <= 3,
            (p[10] & 0xF1) == 0,
            p[11..16] == [0u8; 5]
        );
    }
    Some(Label::PixelPaintersFmf)
}

/// Pixel Painters Xatax FM Module (`.ftf`, 1994 Pixel Painters).
pub(crate) fn verify_pixel_painters_ftf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        (MIN_PIXEL_PAINTERS_FTF_SIZE..=65_536).contains(&size),
        prefix.len() >= 32,
        (1..=16).contains(&u8_at(prefix, 0)?),
        (2..=14).contains(&u8_at(prefix, 1)?),
        le16(prefix, 2)? > 0
    );
    let ntrk = u8_at(prefix, 1)? as usize;
    let mut prev = 0x3Fu64;
    for i in 0..ntrk {
        let off = le16(prefix, 4 + i * 2)? as u64;
        require!(off > prev && off < size);
        prev = off;
    }
    require!(get_slice(prefix, 4 + ntrk * 2, 32)?.iter().all(|&b| b == 0));
    Some(Label::PixelPaintersFtf)
}

/// Antoine Rosset Macintosh PlayerPRO Module (`.madg` / `.madi` / `.madk`): `"MADG"`, `"MADI"`, or `"MADK"` at `@0..4`.
pub(crate) fn verify_playerpro_madg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PLAYERPRO_MADG_SIZE,
        prefix.len() >= 64,
        starts(prefix, b"MADG") || starts(prefix, b"MADI") || starts(prefix, b"MADK"),
        get_slice(prefix, 4, 36)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b)),
        has_non_ascii_text_byte(prefix.get(..64)?)
    );
    Some(Label::PlayerproMadg)
}

/// Sony PlayStation SPU VAG Audio (.vag).
pub(crate) fn verify_playstation_vag(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_VAG_SIZE && prefix.len() >= 48);
    require!(starts(prefix, b"VAGp") || starts(prefix, b"VAGi"));

    let data_size = be32(prefix, 12)?;
    let sample_rate = be32(prefix, 16)?;

    // Sample rate: 4000 Hz to 96000 Hz
    require!((4000..=96000).contains(&sample_rate));

    // Data size must be non-zero, a multiple of 16 (SPU ADPCM block size), and not exceed file size
    require!(data_size > 0 && data_size % 16 == 0);
    require!((data_size as u64) + 48 <= size + 1024);

    Some(Label::Vag)
}

/// Sony PlayStation 3 / PS Vita / PS4 XVAG Audio Stream (.xvag).
pub(crate) fn verify_playstation_xvag(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_XVAG_AUDIO_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"XVAG"),
        get_slice(prefix, 32, 36)? == b"fmat" || get_slice(prefix, 32, 36)? == b"FMAT"
    );
    let is_be_vgmstream = (u8_at(prefix, 8)? & 1) == 1;
    let hdr_sz_vgm = if is_be_vgmstream { be32(prefix, 4)? } else { le32(prefix, 4)? };
    let channels_vgm = if is_be_vgmstream { be32(prefix, 40)? } else { le32(prefix, 40)? };
    let sr_vgm = if prefix.len() >= 64 {
        if is_be_vgmstream { be32(prefix, 60)? } else { le32(prefix, 60)? }
    } else {
        0
    };
    let valid_real_xvag =
        hdr_sz_vgm >= 36 && (1..=16).contains(&channels_vgm) && (4_000..=192_000).contains(&sr_vgm);

    let is_be_legacy = (u8_at(prefix, 4)? & 1) == 1;
    let hdr_sz_leg =
        if is_be_legacy { be32(prefix, 4)? & 0x00FF_FFFF } else { le32(prefix, 4)? >> 8 };
    let channels_leg = if is_be_legacy { be32(prefix, 36)? } else { le32(prefix, 36)? };
    let sr_leg = if is_be_legacy { be32(prefix, 44)? } else { le32(prefix, 44)? };
    let valid_legacy =
        hdr_sz_leg >= 36 && (1..=16).contains(&channels_leg) && (4_000..=192_000).contains(&sr_leg);

    require!(valid_real_xvag || valid_legacy);
    Some(Label::Xvag)
}

/// Masahiro Kajihara (KAJA) NEC PC-98 PMD FM Module (`.m`): `@0` version (`0..=2`), `@1..3 == 0x001A` (26B offset to Part A after 12-entry `u16 LE` part table), strictly non-decreasing `u16 LE` part offsets `@1..25` all `< size`.
pub(crate) fn verify_pmd_m_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        (MIN_PMD_M_SIZE..=65_536).contains(&size),
        prefix.len() >= 26,
        u8_at(prefix, 0)? <= 2,
        le16(prefix, 1) == Some(0x001A)
    );
    let mut prev = 0x001Au16;
    let mut i = 3;
    while i + 2 <= 25 {
        let cur = le16(prefix, i)?;
        require!(cur >= prev && (cur as u64) <= size);
        prev = cur;
        i += 2;
    }
    Some(Label::PmdM)
}

/// Lone Ranger / Acme PolyTracker Module (`.ptm`): `@28 == 0x1A` (DOS EOF), `"PTMF"` at `@44..48`, `num_channels` (`1..=32`) at `@38..40`.
pub(crate) fn verify_polytracker_module(prefix: &[u8], size: u64) -> Option<Label> {
    const DOS_EOF_OFFSET: usize = 28; // @28: 0x1A DOS EOF after 28-byte ASCII title
    const CHANNELS_OFFSET: usize = 38; // @38: u16 LE number of channels (1..=32)
    const MAGIC_OFFSET: usize = 44; // @44: "PTMF" magic
    require!(
        size >= MIN_POLYTRACKER_SIZE,
        prefix.len() >= 48,
        u8_at(prefix, DOS_EOF_OFFSET) == Some(0x1A),
        (1..=32).contains(&le16(prefix, CHANNELS_OFFSET)?),
        eq_at(prefix, MAGIC_OFFSET, b"PTMF")
    );
    Some(Label::Ptm)
}

/// Sound Particles / Audio Plugin Preset Data Tree (.sap / .afx).
pub(crate) fn verify_preset_sap(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRESET_SAP_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"preset_data_tree\0\x01\x05preset_name\0")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::PresetSap)
}

/// Pink / Abyss preTracker Amiga Module (`"PRT"` + version byte `0x18..=0x22`).
pub(crate) fn verify_pretracker_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRETRACKER_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"PRT"),
        (0x18..=0x22).contains(&u8_at(prefix, 3)?),
        prefix.get(4..16)?.contains(&0)
    );
    Some(Label::PretrackerPrt)
}

/// ZX Spectrum Pro Sound Creator Module (`.psc`): `"PSC V1.0"` at `@0..8` + `" COMPILATIO"` at `@9..20`.
pub(crate) fn verify_pro_sound_creator_psc(prefix: &[u8], size: u64) -> Option<Label> {
    const COMP_OFFSET: usize = 9; // @9: " COMPILATIO"
    require!(
        size >= MIN_PRO_SOUND_CREATOR_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"PSC V1.0"),
        eq_at(prefix, COMP_OFFSET, b" COMPILATIO")
    );
    Some(Label::Psc)
}

/// Sony OpenMG Audio (`.oma` / `.aa3`):
/// - Tagged (`"ea3"`): `@3` ID3 version (`2..=4`), `@4 == 0x00`, `@6..10` 28-bit synchsafe tag length.
/// - Bare untagged (`"EA3"`): `@3 <= 3`, `@4..6 == 0x00 0x60` (`EA3_HEADER_SIZE = 96`; `@6..8` is `eid`, e.g. `0xFFFF`).
pub(crate) fn verify_pronom_sony_oma(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_PRONOM_SONY_OMA_SIZE
        && prefix.len() >= 16
        && starts(prefix, SONY_OMA_TAGGED_PREFIX)
        && matches!(u8_at(prefix, 3)?, 0x02..=0x04)
        && u8_at(prefix, 4)? == 0x00
        && (u8_at(prefix, 6).unwrap_or(0)
            | u8_at(prefix, 7).unwrap_or(0)
            | u8_at(prefix, 8).unwrap_or(0)
            | u8_at(prefix, 9).unwrap_or(0))
            & 0x80
            == 0
    {
        return Some(Label::SonyOma);
    }
    if size >= MIN_SONY_OMA_BARE_SIZE
        && prefix.len() >= 16
        && starts(prefix, SONY_OMA_BARE_PREFIX)
        && u8_at(prefix, 3)? <= 0x03
        && u8_at(prefix, 4)? == 0x00
        && u8_at(prefix, 5)? == 0x60
    {
        return Some(Label::SonyOma);
    }
    None
}

/// Digidesign / Avid Pro Tools Session (`.pts` / `.ptf` / `.ptx`, PRONOM `fmt/1950`, `fmt/1951`, `fmt/1952`).
pub(crate) fn verify_protools(prefix: &[u8], size: u64) -> Option<Label> {
    const PROTOOLS_BITSTRING_MAGIC: &[u8; 17] = bytes!(&[0x03], b"0010111100101011"); // @0..17: 0x03 + 16-byte binary bitstring
    const PROTOOLS_BITSTRING_HDR_LEN: usize = 24; // @0..24: 17B magic + 7B version/encryption header

    if size >= PROTOOLS_BITSTRING_HDR_LEN as u64
        && prefix.len() >= PROTOOLS_BITSTRING_HDR_LEN
        && starts(prefix, PROTOOLS_BITSTRING_MAGIC)
    {
        require!(has_non_ascii_text_byte(
            prefix.get(PROTOOLS_BITSTRING_MAGIC.len()..PROTOOLS_BITSTRING_HDR_LEN)?
        ));
        return Some(Label::Ptf);
    }
    require!(
        size >= MIN_PROTOOLS_SIZE
            && prefix.len() >= 16
            && (starts(prefix, b"ptf ") || starts(prefix, b"Ptf "))
            && has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    Some(Label::Ptf)
}

/// Franck Charlet ProTrekkr / NoiseTrekkr Module (`"PROTREKK"`..`"PROTREKO"` / `"TWNN"`).
pub(crate) fn verify_protrekkr_module(prefix: &[u8], size: u64) -> Option<Label> {
    const PROTREKKR_PREFIX: &[u8; 7] = b"PROTREK"; // @0..7: ProTrekkr v1.x/v2.x module prefix
    const NOISETREKKR_MAGIC: &[u8; 4] = b"TWNN"; // @0..4: NoiseTrekkr (`TWNNSNG`) module magic

    require!(
        size >= MIN_PROTREKKR_SIZE,
        prefix.len() >= 20,
        (starts(prefix, PROTREKKR_PREFIX) && matches!(u8_at(prefix, 7)?, b'K'..=b'P'))
            || starts(prefix, NOISETREKKR_MAGIC),
        prefix.get(4..20)?.contains(&0)
    );
    Some(Label::ProtrekkrPtk)
}

/// Sony PlayStation 2 CSL / Jam Soundbank Header file (`.hd`).
pub(crate) fn verify_ps2_hd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PS2_HD_SIZE,
        prefix.len() >= 36,
        starts(prefix, PS2_HD_VERS_MAGIC),
        eq_at(prefix, PS2_HD_HEAD_OFFSET, PS2_HD_HEAD_MAGIC),
        le32(prefix, PS2_HD_TOTAL_SIZE_OFFSET)? >= 64
    );
    Some(Label::Ps2Hd)
}

/// Professional Sound Artists Amiga Module (`.psa`): `"PSA\0"` at `@0..4` + 20B printable ASCII title at `@4..24`.
pub(crate) fn verify_psa_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PSA_MODULE_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"PSA\x00"),
        u8_at(prefix, 4)?.is_ascii_graphic(),
        get_slice(prefix, 4, 24)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b))
    );
    Some(Label::PsaModule)
}

/// Sony PlayStation SPU Register Dump (`.spu`): `"SPU1"` or `"SPU\0"` at `@0..4` + ASCII/Shift-JIS game string + binary register dump.
pub(crate) fn verify_psx_spu_dump(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PSX_SPU_DUMP_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"SPU1") || starts(prefix, b"SPU\x00"),
        u8_at(prefix, 4)? != 0,
        has_non_ascii_text_byte(prefix.get(..prefix.len().min(64))?)
    );
    Some(Label::Spu)
}

/// Psycle Modular Music Creation Studio Song (`.psy`): `"PSY3SONG"` or `"PSY2SONG"` at `@0..8`.
pub(crate) fn verify_psycle_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PSYCLE_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"PSY3SONG") || starts(prefix, b"PSY2SONG")
    );
    Some(Label::Psy)
}

/// Agile Pumatracker Amiga Module (`.puma`, 1991 Agile).
/// Layout: 12B ASCII/NUL name, `last_ord = be16(12) <= 255`, `be16(14)` in `1..=128`,
/// `be16(16)` in `1..=64`, `be16(18) == 0`, exact `"patt"` at `@(80 + (last_ord + 1) * 14)`.
pub(crate) fn verify_pumatracker(prefix: &[u8], size: u64) -> Option<Label> {
    const PATT_TAG: &[u8; 4] = b"patt"; // @80 + (last_ord + 1) * 14
    require!(size >= MIN_PUMATRACKER_SIZE, prefix.len() >= 98);
    require!(get_slice(prefix, 0, 12)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b)));
    let last_ord = be16(prefix, 12)? as usize;
    let num_pats = be16(prefix, 14)?;
    let num_inst = be16(prefix, 16)?;
    let zero_pad = be16(prefix, 18)?;
    require!(
        last_ord <= 255,
        (1..=128).contains(&num_pats),
        (1..=64).contains(&num_inst),
        zero_pad == 0
    );
    let patt_off = 80 + (last_ord + 1) * 14;
    require!(prefix.len() >= patt_off + 4, eq_at(prefix, patt_off, PATT_TAG));
    Some(Label::Pumatracker)
}

/// Studio Pixel PxTone / Piston Collage (`.ptcop`, `.pttune`, `.ptnoise`, `.ptvoice`, 2005 Daisuke Amaya):
/// `"PTCOLLAGE-"` / `"PTTUNE--"` / `"PTNOISE-"` / `"PTVOICE-"` followed by an ASCII or binary `YYYYMMDD` version date.
pub(crate) fn verify_pxtone(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_8: usize = 8; // @8: ASCII digit or binary u32 LE YYYYMMDD date for 8-byte prefixes
    const VERSION_10: usize = 10; // @10: ASCII digit for 10-byte "PTCOLLAGE-" prefix
    require!(
        size >= MIN_PXTONE_SIZE,
        prefix.len() >= 16,
        has_non_ascii_text_byte(prefix.get(8..prefix.len().min(24))?)
    );
    if starts(prefix, b"PTCOLLAGE-") {
        require!(u8_at(prefix, VERSION_10)?.is_ascii_digit());
        return Some(Label::Pxtone);
    }
    require!(
        starts(prefix, b"PTNOISE-") || starts(prefix, b"PTVOICE-") || starts(prefix, b"PTTUNE--")
    );
    let is_ascii_date = u8_at(prefix, VERSION_8)?.is_ascii_digit();
    let is_bin_date = (20_040_101..=20_301_231).contains(&le32(prefix, VERSION_8)?);
    require!(is_ascii_date || is_bin_date);
    Some(Label::Pxtone)
}

/// MicroDeal Atari ST Quartet Sample Set (`.set`, 1989 MicroDeal).
pub(crate) fn verify_quartet_set(prefix: &[u8], size: u64) -> Option<Label> {
    const QUARTET_SET_HEADER_SIZE: u64 = 342; // 2B header + 20*7B names + 20*4B offsets + 20*6B loop/len
    require!(
        size >= MIN_QUARTET_SET_SIZE,
        prefix.len() >= 222,
        (0x04..=0x20).contains(&u8_at(prefix, 0)?),
        (0x01..=0x10).contains(&u8_at(prefix, 1)?)
    );
    let off0 = be32(prefix, 142)? as u64;
    let is_std_hdr = off0 == QUARTET_SET_HEADER_SIZE
        && size > QUARTET_SET_HEADER_SIZE
        && u8_at(prefix, 2)? == b' '
        && get_slice(prefix, 3, 8)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
        && u8_at(prefix, 8)? == 0;
    let is_full_table =
        get_slice(prefix, 2, 142)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b))
            && (0..20usize).all(|i| {
                let off = be32(prefix, 142 + i * 4).unwrap_or(0) as u64;
                off == 0 || (off >= 222 && off <= size)
            })
            && (0..20usize).any(|i| be32(prefix, 142 + i * 4).unwrap_or(0) != 0);
    require!(is_std_hdr || is_full_table);
    Some(Label::QuartetSet)
}

/// MicroDeal Atari ST Quartet 4-Voice Module (`.4v` / `.qpa`): `@0..2` step rate (`4..=32`), `@2..4` tempo, `@4..6` bar length (`4..=64`), `@6..8` time signature, `@8..16 == 00*8`, followed by 12-byte big-endian voice events (`0x00` high byte + `cmd` in `'P'`, `'V'`, `'R'`, `'S'`, `'L'`, `'F'`, `'l'`, `'r'`).
pub(crate) fn verify_quartet_st_module(prefix: &[u8], size: u64) -> Option<Label> {
    const RATE_OFFSET: usize = 0; // @0: u16 BE replay rate (4..=32)
    const BAR_OFFSET: usize = 4; // @4: u16 BE bar length (4..=64)
    require!(
        size >= MIN_QUARTET_ST_SIZE,
        prefix.len() >= 40,
        (4..=32).contains(&be16(prefix, RATE_OFFSET)?),
        (4..=64).contains(&be16(prefix, BAR_OFFSET)?),
        (1..=16).contains(&u8_at(prefix, 6)?),
        matches!(u8_at(prefix, 7)?, 2 | 4 | 8 | 16),
        be64(prefix, 8) == Some(0),
        u8_at(prefix, 16) == Some(0),
        matches!(u8_at(prefix, 17)?, b'P' | b'V' | b'R' | b'S' | b'L' | b'F' | b'l' | b'r'),
        u8_at(prefix, 28) == Some(0),
        matches!(u8_at(prefix, 29)?, b'P' | b'V' | b'R' | b'S' | b'L' | b'F' | b'l' | b'r')
    );
    Some(Label::QuartetSt)
}

/// RAD Game Tools Bink Audio Stream (`.binka`, 2007 RAD Game Tools, `"1FCB"`).
///
/// Header layout (little-endian):
/// - `@0..4`: `"1FCB"` (reversed `"BCF1"`)
/// - `@4`: `u8` `version` (`1..=2`)
/// - `@5`: `u8` `channels` (`1..=8`)
/// - `@6..8`: `u16 LE` `sample_rate` (`4000..=65535`)
/// - `@8..12`: `u32 LE` `num_samples` (`> 0`)
/// - `@12..16`: `u32 LE` `max_frame_size` (`> 0`)
/// - `@16..20`: `u32 LE` `file_size` (`>= 24`)
pub(crate) fn verify_rad_bink_audio(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4: u8 format version (1..=2)
    const CHANNELS_OFF: usize = 5; // @5: u8 channels (1..=8)
    const RATE_OFF: usize = 6; // @6..8: u16 LE sample rate Hz
    const SAMPLES_OFF: usize = 8; // @8..12: u32 LE total samples (> 0)
    const MAX_FRAME_OFF: usize = 12; // @12..16: u32 LE max frame size (> 0)
    const FILE_SIZE_OFF: usize = 16; // @16..20: u32 LE declared file size (>= 24)

    require!(
        size >= MIN_RAD_BINK_AUDIO_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"1FCB"),
        (1..=2).contains(&u8_at(prefix, VERSION_OFF)?),
        (1..=8).contains(&u8_at(prefix, CHANNELS_OFF)?),
        (4000..=65535).contains(&le16(prefix, RATE_OFF)?),
        le32(prefix, SAMPLES_OFF)? > 0,
        le32(prefix, MAX_FRAME_OFF)? > 0,
        le32(prefix, FILE_SIZE_OFF)? >= 24
    );
    Some(Label::RadBinkAudio)
}

/// Radical Entertainment RSD Audio Stream (`.rsd`, 2000 Radical Entertainment, `"RSD2"` / `"RSD3"` / `"RSD4"` / `"RSD6"`).
///
/// Header layout:
/// - `@0..4`: `"RSD2"`, `"RSD3"`, `"RSD4"`, or `"RSD6"`
/// - `@4..8`: 4-byte codec FourCC (`"PCM "`, `"PCMB"`, `"VAG "`, `"XADP"`, `"GADP"`, `"WADP"`, `"RADP"`, `"OOGV"`, `"WMA "`, `"AT3+"`, `"XMA "`)
/// - `@8..12`: `u32 LE` `channels` (`1..=8`)
/// - `@12..16`: `u32 LE` `bits_per_sample` (`4 | 8 | 16 | 24 | 32`)
/// - `@16..20`: `u32 LE` `sample_rate` (`4000..=192_000`)
pub(crate) fn verify_radical_rsd(prefix: &[u8], size: u64) -> Option<Label> {
    const CODEC_OFF: usize = 4; // @4..8: 4-byte codec FourCC
    const CHANNELS_OFF: usize = 8; // @8..12: u32 LE channels (1..=8)
    const BITS_OFF: usize = 12; // @12..16: u32 LE bits per sample
    const RATE_OFF: usize = 16; // @16..20: u32 LE sample rate Hz

    require!(
        size >= MIN_RADICAL_RSD_SIZE,
        prefix.len() >= 20,
        matches!(get_slice(prefix, 0, 4)?, b"RSD2" | b"RSD3" | b"RSD4" | b"RSD6"),
        matches!(
            get_slice(prefix, CODEC_OFF, CODEC_OFF + 4)?,
            b"PCM "
                | b"PCMB"
                | b"VAG "
                | b"XADP"
                | b"GADP"
                | b"WADP"
                | b"RADP"
                | b"OOGV"
                | b"WMA "
                | b"AT3+"
                | b"XMA "
        ),
        (1..=8).contains(&le32(prefix, CHANNELS_OFF)?),
        matches!(le32(prefix, BITS_OFF), Some(4 | 8 | 16 | 24 | 32)),
        (4000..=192_000).contains(&le32(prefix, RATE_OFF)?)
    );
    Some(Label::RadicalRsd)
}

/// Rockstar Games RAGE Audio Wave Container (`.awc`).
pub(crate) fn verify_rage_awc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RAGE_AWC_SIZE,
        prefix.len() >= 16,
        starts(prefix, RAGE_AWC_MAGIC),
        u8_at(prefix, RAGE_AWC_FLAGS_HI_OFFSET) == Some(0xFF),
        (1..=65_535).contains(&le32(prefix, RAGE_AWC_STREAMS_OFFSET)?),
        le32(prefix, RAGE_AWC_DATA_OFF_OFFSET)? >= 16
    );
    Some(Label::RageAwc)
}

/// RailWorks / Train Simulator XOR-Obfuscated WAV Audio (`.dav`, single-byte XOR key `0x7B`).
pub(crate) fn verify_railworks_dav(prefix: &[u8], size: u64) -> Option<Label> {
    const XOR_KEY: u8 = 0x7B;
    const RIFF_XOR: &[u8; 4] = hex2bin!("29 32 3D 3D");
    const WAVE_FMT_XOR: &[u8; 8] = hex2bin!("2C 3A 2D 3E 1D 16 0F 5B");

    require!(
        size >= MIN_RAILWORKS_DAV_SIZE,
        prefix.len() >= 32,
        starts(prefix, RIFF_XOR),
        eq_at(prefix, 8, WAVE_FMT_XOR),
    );
    let dec_u32 = |off: usize| -> Option<u32> {
        let s = get_slice(prefix, off, off + 4)?;
        Some(u32::from_le_bytes([s[0] ^ XOR_KEY, s[1] ^ XOR_KEY, s[2] ^ XOR_KEY, s[3] ^ XOR_KEY]))
    };
    let dec_u16 = |off: usize| -> Option<u16> {
        let s = get_slice(prefix, off, off + 2)?;
        Some(u16::from_le_bytes([s[0] ^ XOR_KEY, s[1] ^ XOR_KEY]))
    };
    let riff_size = dec_u32(4)?;
    let fmt_size = dec_u32(16)?;
    let audio_fmt = dec_u16(20)?;
    let channels = dec_u16(22)?;
    let sample_rate = dec_u32(24)?;
    let byte_rate = dec_u32(28)?;
    require!(
        riff_size >= 36,
        (16..=64).contains(&fmt_size),
        matches!(audio_fmt, 1..=3 | 6 | 7 | 0x11 | 0xFFFE),
        (1..=8).contains(&channels),
        (4_000..=192_000).contains(&sample_rate),
        byte_rate > 0,
    );
    Some(Label::RailworksDav)
}

/// RamTracker Module (`.trk`): `"TRK01/TV.ES."` at `@0..12`.
pub(crate) fn verify_ramtracker_trk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_RAMTRACKER_TRK_SIZE, prefix.len() >= 16, starts(prefix, b"TRK01/TV.ES."));
    Some(Label::RamtrackerTrk)
}

/// RealTech Real Tracker Module (`.rtm`): `"RTMM "` at `@0..5` + `@37 == 0x1A` (DOS EOF).
pub(crate) fn verify_real_tracker_rtm(prefix: &[u8], size: u64) -> Option<Label> {
    const DOS_EOF_OFFSET: usize = 37; // @37: 0x1A DOS EOF after 32-byte object name
    require!(
        size >= MIN_REAL_TRACKER_RTM_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"RTMM "),
        u8_at(prefix, DOS_EOF_OFFSET) == Some(0x1A)
    );
    Some(Label::Rtm)
}

/// Come On Music Recomposer PC-98 / X68000 MIDI Sequence (`"RCM-PC98V2.0"`).
pub(crate) fn verify_recomposer_rcp_music(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RECOMPOSER_RCP_SIZE,
        prefix.len() >= 32,
        (starts(prefix, b"RCM-PC98V2.0") || starts(prefix, b"COME ON MUSIC"))
            && has_non_ascii_text_byte(prefix.get(12..32)?)
    );
    Some(Label::RecomposerRcp)
}

/// Renoise Legacy Binary Song (`.rns`): `"RNS0"` at `@0..4` + `">>> Chunk Start <<<"` at `@9..28` or `@10..29`.
pub(crate) fn verify_renoise_old_rns(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RENOISE_OLD_SIZE,
        prefix.len() >= 30,
        starts(prefix, b"RNS0"),
        eq_at(prefix, 9, b">>> Chunk Start <<<") || eq_at(prefix, 10, b">>> Chunk Start <<<")
    );
    Some(Label::Rns)
}

/// Richard Joseph / Vectordean Amiga Audio (`.rjp` / `.sng` / `.ins`): `"RJP1"` at `@0..4` + `"SMOD"` or `00 00 00 00` at `@4..8`.
pub(crate) fn verify_richard_joseph_audio(prefix: &[u8], size: u64) -> Option<Label> {
    const SUBTAG_OFFSET: usize = 4; // @4: "SMOD" (song) or 0x00000000 (instrument bank)
    require!(
        size >= MIN_RICHARD_JOSEPH_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"RJP1"),
        eq_at(prefix, SUBTAG_OFFSET, b"SMOD") || be32(prefix, SUBTAG_OFFSET) == Some(0)
    );
    Some(Label::Rjp)
}

/// Malcolm Taylor RK Audio Lossless/Lossy Compressor (`.rka`, `"RKA7"`).
pub(crate) fn verify_rk_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RK_AUDIO_SIZE,
        prefix.len() >= 16,
        starts(prefix, RK_AUDIO_MAGIC),
        le32(prefix, 4)? > 0,
        (1..=384_000).contains(&le32(prefix, 8)?),
        matches!(u8_at(prefix, 12)?, 1 | 2),
        matches!(u8_at(prefix, 13)?, 8 | 16),
        (u8_at(prefix, 15)? & 2) != 0
    );
    Some(Label::RkaAudio)
}

/// Rob Hubbard Atari ST Music Module (`.rho`, 1988 Rob Hubbard).
pub(crate) fn verify_rob_hubbard_rho(prefix: &[u8], size: u64) -> Option<Label> {
    const RHO_SINE: &[u8; 16] = hex2bin!("00 40 7F 40 00 C0 81 C0 FF 00 24 49 6C 8E B1 D6"); // @0..16: vibrato sine table
    require!(size >= MIN_ROB_HUBBARD_RHO_SIZE, prefix.len() >= 24, starts(prefix, RHO_SINE));
    Some(Label::RobHubbardRho)
}

/// ROBOPLAY AdLib OPL2 Chiptune Module (.ply).
pub(crate) fn verify_roboplay_ply(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ROBOPLAY_PLY_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"ROBOPLAY\x01\x07\0A")
    );
    Some(Label::RoboplayPly)
}

/// Roland XP/JV MRC-Pro Sequencer Song (`.svq`): `"SV00\0\x60"` or `"SV01\0\x60"` at `@2..8`.
pub(crate) fn verify_roland_svq(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC_OFFSET: usize = 2; // @2: "SV00\0\x60" or "SV01\0\x60"
    require!(
        size >= MIN_ROLAND_SVQ_SIZE,
        prefix.len() >= 16,
        eq_at(prefix, MAGIC_OFFSET, b"SV00\x00\x60")
            || eq_at(prefix, MAGIC_OFFSET, b"SV01\x00\x60")
    );
    Some(Label::RolandSvq)
}

/// SAM Coupé E-Tracker Compiled SAA1099 Module (`.cop` / `.etc`): compiled Z80 SAA1099 player stubs or `"ETracker (C) BY ESI"` banner.
pub(crate) fn verify_sam_coupe_cop(prefix: &[u8], size: u64) -> Option<Label> {
    const ETRACKER_Z80_INIT: &[u8; 7] = hex2bin!("3E 01 3D 20 1F DD 21"); // @6..13: LD A,1 / DEC A / JR NZ,+31 / LD IX,nn
    const ETRACKER_Z80_LOOP: &[u8; 4] = hex2bin!("06 06 C5 CD"); // @15..19: LD B,6 / PUSH BC / CALL nn

    require!(size >= MIN_SAM_COUPE_COP_SIZE && prefix.len() >= 30);
    let is_z80_stub =
        u8_at(prefix, 0) == Some(0x01) && eq_at(prefix, 3, hex2bin!("3E FF 32 01 80 79 FE"));
    let is_etc_banner = eq_at(prefix, 10, b"ETracker (C) BY ESI");
    let is_etracker_z80 = eq_at(prefix, 6, ETRACKER_Z80_INIT)
        && ((u8_at(prefix, 0) == Some(0x21)
            && u8_at(prefix, 3) == Some(0xC3)
            && eq_at(prefix, 15, ETRACKER_Z80_LOOP))
            || contains(prefix.get(..prefix.len().min(256))?, b"ETracker"));
    let is_saa_player_z80 = u8_at(prefix, 0) == Some(0x3E)
        && eq_at(prefix, 2, hex2bin!("3D C2 23 81 21"))
        && eq_at(prefix, 9, hex2bin!("7E 0F 30 15"));
    require!(is_z80_stub || is_etc_banner || is_etracker_z80 || is_saa_player_z80);
    Some(Label::SamCoupeCop)
}

/// Henning Sauer SBStudio PAC Module (`.pac`): `"PACG"` at `@0..4` + `"PAIN"` at `@8..12`.
pub(crate) fn verify_sbstudio_pac(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SBSTUDIO_PAC_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"PACG"),
        eq_at(prefix, 8, b"PAIN")
    );
    Some(Label::SbstudioPac)
}

/// sc68 Atari ST / Amiga Chip-Music (`"SC68 Music-file "`).
pub(crate) fn verify_sc68_music(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SC68_MUSIC_SIZE,
        prefix.len() >= 64,
        starts(prefix, b"SC68 Music-file "),
        prefix.get(16..64)?.contains(&0)
    );
    Some(Label::Sc68Music)
}

pub(crate) fn verify_scream_tracker_3(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_S3M_SIZE && prefix.len() >= 48);
    require!(eq_at(prefix, 44, b"SCRM"));
    require!(u8_at(prefix, 28)? == 0x1A && u8_at(prefix, 29)? == 0x10);

    let ord_num = le16(prefix, 32)?;
    let smp_num = le16(prefix, 34)?;
    let pat_num = le16(prefix, 36)?;
    let ffi = le16(prefix, 42)?;
    // ffi (File Format Information): 1 = signed samples, 2 = unsigned samples
    require!(ffi == 1 || ffi == 2);
    // Reasonable module limits: orders <= 256, samples <= 256, patterns <= 256
    require!(ord_num <= 256 && smp_num <= 256 && pat_num <= 256);

    Some(Label::S3m)
}

/// Lucasfilm Games SCUMM Amiga Music Module (`.scumm`, 1989 Lucasfilm Games).
/// Layout: `be16(2) <= size` (`size - be16(2) <= 64`), `48 E7 7F FE` (`MOVEM.L D0-D6/A0-A6,-(SP)`) at `@4..8` + Paula `00 DF F0 96` in `@16..96`.
pub(crate) fn verify_scumm_amiga(prefix: &[u8], size: u64) -> Option<Label> {
    const MOVEM_SCUMM: &[u8; 4] = hex2bin!("48 E7 7F FE"); // @4..8: MOVEM.L D0-D6/A0-A6,-(SP)
    const PAULA_DMACON: &[u8; 4] = hex2bin!("00 DF F0 96"); // Paula DMACON register address
    require!(
        (MIN_SCUMM_AMIGA_SIZE..=65_600).contains(&size),
        prefix.len() >= 48,
        eq_at(prefix, 4, MOVEM_SCUMM)
    );
    let decl_sz = be16(prefix, 2)? as u64;
    require!(decl_sz >= MIN_SCUMM_AMIGA_SIZE, decl_sz <= size, size - decl_sz <= 64);
    let win_end = prefix.len().min(96);
    memchr::memmem::find(get_slice(prefix, 16, win_end)?, PAULA_DMACON)?;
    Some(Label::ScummAmiga)
}

/// Sean Conran Amiga Music Module (`.scr`, 1989 Sean Conran).
pub(crate) fn verify_sean_conran_scr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SEAN_CONRAN_SCR_SIZE,
        prefix.len() >= 24,
        starts(prefix, SEAN_CONRAN_SCR_V1_MAGIC) || starts(prefix, SEAN_CONRAN_SCR_V2_MAGIC)
    );
    // Strictly descending 12 x u16 BE Amiga Paula period table at @0..24
    let mut prev = be16(prefix, 0)?;
    for i in 1..12usize {
        let p = be16(prefix, i * 2)?;
        require!(p < prev && p >= 0x0700);
        prev = p;
    }
    Some(Label::SeanConranScr)
}

/// Sega Genesis YM2612 Audio Log (.gym).
pub(crate) fn verify_sega_genesis_gym(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SEGA_GENESIS_GYM_SIZE && prefix.len() >= 0x44);
    require!(starts(prefix, b"GYMX") && prefix.get(4..0x44)?.contains(&0));
    let is_valid_ascii = |s: &[u8]| s.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b));
    require!(is_valid_ascii(get_slice(prefix, 0x04, 0x24)?));
    require!(is_valid_ascii(get_slice(prefix, 0x24, 0x44)?));
    Some(Label::GenesisGym)
}

/// Sega Master System / Game Gear / ColecoVision SGC Audio (`.sgc`): `"SGC\x1A"` + version `1` at `@4` + system type `0..=2` at `@24`.
pub(crate) fn verify_sega_sgc_audio(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFFSET: usize = 4; // @4: u8 version (1)
    const NTSC_PAL_OFFSET: usize = 5; // @5: u8 0=NTSC, 1=PAL
    const SYS_TYPE_OFFSET: usize = 24; // @24: u8 0=SMS, 1=Game Gear, 2=ColecoVision
    require!(
        size >= MIN_SEGA_SGC_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"SGC\x1A"),
        u8_at(prefix, VERSION_OFFSET) == Some(1),
        u8_at(prefix, NTSC_PAL_OFFSET)? <= 1,
        u8_at(prefix, SYS_TYPE_OFFSET)? <= 2
    );
    Some(Label::Sgc)
}

/// Shorten Lossless Audio (.shn).
pub(crate) fn verify_shorten(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SHORTEN_SIZE && prefix.len() >= 5);
    require!(starts(prefix, b"ajkg"));
    let version = u8_at(prefix, 4)?;
    // Format version: 0..=3.
    require!(version <= 3);
    Some(Label::Shn)
}

/// Shroom Player Module (`.sho`): `"shro\x02\0\0"` at `@0..7`.
pub(crate) fn verify_shroom_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SHROOM_SIZE, prefix.len() >= 16, starts(prefix, b"shro\x02\x00\x00"));
    Some(Label::Sho)
}

/// Reinier van Vliet Amiga SidMon II Module (`.sid2`): `"SIDMON II - THE MIDI VERSION"` at `@58..86`.
pub(crate) fn verify_sidmon2_module(prefix: &[u8], size: u64) -> Option<Label> {
    const BANNER_OFFSET: usize = 58; // @58: 28-byte "SIDMON II - THE MIDI VERSION" signature
    require!(
        size >= MIN_SIDMON2_SIZE,
        prefix.len() >= 86,
        eq_at(prefix, BANNER_OFFSET, b"SIDMON II - THE MIDI VERSION")
    );
    Some(Label::Sidmon2)
}

/// Sierra On-Line SCI0 AdLib Patch Bank (`.003`, 1988 Sierra On-Line).
pub(crate) fn verify_sierra_sci_patch(prefix: &[u8], size: u64) -> Option<Label> {
    const PATCH_COUNT_SMALL: u64 = 1_346; // 2 + 48 * 28
    const PATCH_COUNT_LARGE: u64 = 2_692; // 2 + 96 * 28 + 2B padding (2690 or 2692)
    require!(
        matches!(size, PATCH_COUNT_SMALL | 2_690 | PATCH_COUNT_LARGE),
        prefix.len() >= 2 + 48 * 28,
        starts(prefix, SIERRA_SCI_PATCH_MAGIC)
    );
    for i in 0..48usize {
        let p = get_slice(prefix, 2 + i * 28, 2 + (i + 1) * 28)?;
        require!(
            p[0] <= 3,
            p[2] <= 7,
            p[3] <= 15,
            p[4] <= 15,
            p[5] <= 1,
            p[6] <= 15,
            p[7] <= 15,
            p[8] <= 63,
            p[9] <= 1 && p[10] <= 1 && p[11] <= 1 && p[12] <= 1,
            p[13] <= 3,
            p[18] <= 1,
            p[21] <= 63,
            p[22] <= 1 && p[23] <= 1 && p[24] <= 1 && p[25] <= 1,
            p[26] <= 3 && p[27] <= 3
        );
    }
    Some(Label::SierraSciPatch)
}

/// Sierra On-Line SCI1.1 / SCI2 SOL Digital Audio file (`.aud` / `.sfx`).
pub(crate) fn verify_sierra_sol(prefix: &[u8], size: u64) -> Option<Label> {
    const RES_TYPE_OFFSET: usize = 0; // @0: SCI resource type (0x0D or 0x8D)
    const HDR_LEN_OFFSET: usize = 1; // @1: header size after 2-byte resource prefix (11..=16)
    const SOL_MAGIC_OFFSET: usize = 2; // @2..6: "SOL\0" signature
    const SOL_MAGIC: &[u8; 4] = b"SOL\0"; // @2..6: "SOL\0"
    const SAMPLE_RATE_OFFSET: usize = 6; // @6..8: u16 LE sample rate in Hz (4000..=48000)
    const DATA_SIZE_OFFSET: usize = 9; // @9..13: u32 LE audio payload size

    require!(
        size >= MIN_SIERRA_SOL_SIZE,
        prefix.len() >= 14,
        matches!(u8_at(prefix, RES_TYPE_OFFSET)?, 0x0D | 0x8D),
        (11..=16).contains(&u8_at(prefix, HDR_LEN_OFFSET)?),
        eq_at(prefix, SOL_MAGIC_OFFSET, SOL_MAGIC),
        (4000..=48_000).contains(&le16(prefix, SAMPLE_RATE_OFFSET)?),
        le32(prefix, DATA_SIZE_OFFSET)? > 0
    );
    Some(Label::SierraSol)
}

/// Simon & Schuster Interactive KVAG ADPCM Audio (`.vag`, `"KVAG"`).
pub(crate) fn verify_simon_schuster_kvag(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KVAG_AUDIO_SIZE,
        prefix.len() >= 14,
        starts(prefix, b"KVAG"),
        le32(prefix, 4)? > 0,
        (4000..=96000).contains(&le32(prefix, 8)?),
        le16(prefix, 12)? <= 1
    );
    Some(Label::KvagAudio)
}

/// Super Nintendo SPC700 Sound Chip State (.spc).
pub(crate) fn verify_snes_spc700(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SNES_SPC700_SIZE
            && prefix.len() >= 37
            && starts(prefix, b"SNES-SPC700 Sound File Data")
            && ((u8_at(prefix, 33)? == 0x1A && u8_at(prefix, 34)? == 0x1A)
                || eq_at(prefix, 27, b" 0.10\x00\x00\x00"))
    );
    Some(Label::Spc700)
}

/// HertzDevil SnevenTracker Chiptune Module (`.snm`, 2019 HertzDevil).
pub(crate) fn verify_sneventracker_snm(prefix: &[u8], size: u64) -> Option<Label> {
    const FULL_MAGIC: &[u8; 20] = b"SnevenTracker Module"; // @0..20
    const PARAMS_TAG: &[u8; 8] = b"PARAMS\x00\x00"; // @24..32
    require!(
        size >= MIN_SNEVENTRACKER_SNM_SIZE,
        prefix.len() >= 32,
        starts(prefix, FULL_MAGIC),
        (0x20..=0x4000).contains(&le32(prefix, 20)?),
        eq_at(prefix, 24, PARAMS_TAG)
    );
    Some(Label::SneventrackerSnm)
}

/// ObsM SNGPlay AdLib Chiptune (`.sng`, 1994).
pub(crate) fn verify_sngplay_obsm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SNGPLAY_OBSM_SIZE,
        prefix.len() >= 16,
        starts(prefix, SNGPLAY_OBSM_MAGIC),
        prefix.get(6..12)?.contains(&0)
    );
    Some(Label::SngplayObsm)
}

/// Sonarc lossless squeezed Creative Voice (`.vc`) or raw PCM (`.snc`) audio.
pub(crate) fn verify_sonarc(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_SONARC_SIZE && starts(prefix, SONARC_VOC_MAGIC) {
        return Some(Label::Sonarc);
    }
    if size >= MIN_SONARC_PCM_SIZE && starts(prefix, SONARC_PCM_MAGIC) {
        return Some(Label::Sonarc);
    }
    None
}

/// Aegis Sonix Instrument Descriptor (`.instr`, 1987 Mark Riley / Aegis).
pub(crate) fn verify_sonix_instrument(prefix: &[u8], size: u64) -> Option<Label> {
    const SAMPLED_TAG: &[u8; 13] = b"SampledSound\x00"; // @0..13: sampled-sound instrument
    const MIDI_TAG: &[u8; 5] = b"MIDI\x00"; // @0..5: MIDI patch instrument
    require!(size >= MIN_SONIX_INSTR_SIZE, prefix.len() >= 48);
    let is_sampled = starts(prefix, SAMPLED_TAG)
        && get_slice(prefix, 13, 32)?.iter().all(|&b| b == 0)
        && be16(prefix, 32) == Some(2);
    let is_midi = starts(prefix, MIDI_TAG)
        && get_slice(prefix, 5, 32)?.iter().all(|&b| b == 0)
        && be16(prefix, 32) == Some(3)
        && eq_at(prefix, 38, b"DIPatch\x00");
    require!(is_sampled || is_midi);
    Some(Label::SonixInstr)
}

/// Sony SCEI SGXD Sound Group Audio Container (.sgd / .sgb, 2006) Sony Computer Entertainment.
pub(crate) fn verify_sony_sgxd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SONY_SGXD_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"SGXD"),
        le32(prefix, 4).unwrap_or(0) > 0,
        matches!(
            get_slice(prefix, 16, 20)?,
            b"RGND" | b"WAVE" | b"NAME" | b"SEQD" | b"WSUR" | b"WMKR" | b"BUFS"
        )
    );
    Some(Label::SonySgxd)
}

/// Sony Wave64 64-bit audio container (.w64).
pub(crate) fn verify_sony_wave64(prefix: &[u8], size: u64) -> Option<Label> {
    // Sony Wave64 RIFF GUID: 66666972-912E-11CF-A5D6-28DB04C10000 (Data1 = "riff" LE)
    const SONY_WAVE64_RIFF_GUID: &[u8; 16] = guid!("66666972-912E-11CF-A5D6-28DB04C10000");
    if size >= MIN_SONY_WAVE64_SIZE && prefix.len() >= 16 && starts(prefix, SONY_WAVE64_RIFF_GUID) {
        Some(Label::W64)
    } else {
        None
    }
}

/// Bluemoon Sound Club Module (`.sn` / `.sn2`): `"SNGs"`, `"SN2\0"`, or `"SN2"` with `"NAM"` chunk tag at `@31..34`.
pub(crate) fn verify_sound_club_module(prefix: &[u8], size: u64) -> Option<Label> {
    const SOUND_CLUB_V1_MAGIC: &[u8; 4] = b"SNGs"; // @0..4: Sound Club v1 (.sn) module magic
    const SOUND_CLUB_V2_MAGIC: &[u8; 3] = b"SN2"; // @0..3: Sound Club v2 (.sn2) module magic
    const NAM_TAG_OFFSET: usize = 31; // @31..34: Sound Club v2 "NAM" song name chunk marker
    const NAM_TAG: &[u8; 3] = b"NAM";

    let is_v1 = starts(prefix, SOUND_CLUB_V1_MAGIC)
        && has_non_ascii_text_byte(prefix.get(4..prefix.len().min(64))?);
    let is_v2_null = starts(prefix, SOUND_CLUB_V2_MAGIC) && prefix.get(3) == Some(&0);
    let is_v2_nam = starts(prefix, SOUND_CLUB_V2_MAGIC)
        && prefix.len() >= 35
        && eq_at(prefix, NAM_TAG_OFFSET, NAM_TAG)
        && has_non_ascii_text_byte(prefix.get(3..31)?);
    require!(size >= MIN_SOUND_CLUB_SIZE, prefix.len() >= 16, is_v1 || is_v2_null || is_v2_nam);
    Some(Label::Sn2)
}

/// Holger Gehrmann Sound Control Amiga Module (`.sc`, 1994 Holger Gehrmann).
pub(crate) fn verify_sound_control_sc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SOUND_CONTROL_SC_SIZE,
        prefix.len() >= 40,
        u8_at(prefix, 0)?.is_ascii_graphic(),
        get_slice(prefix, 0, 16)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b))
    );
    let o16 = be32(prefix, 16)? as u64;
    let o20 = be32(prefix, 20)? as u64;
    let o24 = be32(prefix, 24)? as u64;
    require!(
        o16 >= 0x40 && o16 < o20 && o20 < size,
        o24 > 0 && o24 < o16,
        matches!(be16(prefix, 32), Some(2 | 3)),
        be32(prefix, 36) == Some(0),
        has_non_ascii_text_byte(prefix.get(16..40)?)
    );
    Some(Label::SoundControlSc)
}

/// Tony Williams Sound Images Generation 2 AdLib Module (`.sig`, 1992 Tony Williams).
pub(crate) fn verify_sound_images_sig(prefix: &[u8], size: u64) -> Option<Label> {
    const SIG_SEQ_HDR: &[u8; 12] = hex2bin!("FF 00 FF 00 91 00 FF 00 FF 00 80 00"); // @8..20
    require!(
        size >= MIN_SOUND_IMAGES_SIG_SIZE,
        prefix.len() >= 24,
        le16(prefix, 2)? == le16(prefix, 0)?.wrapping_add(1),
        u8_at(prefix, 4)? == 0x05,
        u8_at(prefix, 7)? == 0x00,
        eq_at(prefix, 8, SIG_SEQ_HDR)
    );
    Some(Label::SoundImagesSig)
}

/// Advantec SoundFactory Amiga Synth Module (`.psf`, 1990 Advantec).
pub(crate) fn verify_soundfactory_psf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SOUNDFACTORY_PSF_SIZE,
        prefix.len() >= 276,
        (be32(prefix, 0)? as u64) == size,
        eq_at(prefix, 10, &[0u8; 10])
    );
    let mut past_0114 = 0usize;
    for i in 0..64usize {
        let off = be32(prefix, 20 + i * 4)? as u64;
        require!((0x0114..size).contains(&off));
        if off > 0x0114 {
            past_0114 += 1;
        }
    }
    require!(past_0114 >= 1);
    Some(Label::SoundfactoryPsf)
}

/// Scott Johnston SoundPlayer Amiga Module (`sjs.*`, 1990 Scott Johnston).
pub(crate) fn verify_soundplayer_sjs(prefix: &[u8], size: u64) -> Option<Label> {
    const SJS_VOICE_HDR: &[u8; 12] = hex2bin!("07 00 00 1E 00 00 1E 00 00 1E 00 00"); // @2..14
    require!(
        size >= MIN_SOUNDPLAYER_SJS_SIZE,
        prefix.len() >= 16,
        (0x80..=0xBF).contains(&u8_at(prefix, 0)?),
        eq_at(prefix, 2, SJS_VOICE_HDR)
    );
    Some(Label::SoundplayerSjs)
}

/// Apple IIgs Huibert Aalbers / Ian Schmidt SoundSmith Module (`"SONGOK"` / `"IAN9OK"` / `"IAN92a"`, 1990 Huibert Aalbers).
pub(crate) fn verify_soundsmith_module(prefix: &[u8], size: u64) -> Option<Label> {
    const SOUNDSMITH_MAGIC: &[u8; 6] = b"SONGOK"; // @0..6: SoundSmith v1.0 signature
    const SOUNDSMITH_IAN9OK: &[u8; 6] = b"IAN9OK"; // @0..6: SoundSmith v0.95 signature
    const SOUNDSMITH_IAN92A: &[u8; 6] = b"IAN92a"; // @0..6: SoundSmith v0.92a signature
    require!(
        size >= MIN_SOUNDSMITH_SIZE,
        prefix.len() >= 48,
        starts(prefix, SOUNDSMITH_MAGIC)
            || starts(prefix, SOUNDSMITH_IAN9OK)
            || starts(prefix, SOUNDSMITH_IAN92A),
        get_slice(prefix, 10, 20)?.iter().all(|&b| b == 0),
        u8_at(prefix, 20)? <= 21
    );
    Some(Label::Soundsmith)
}

/// MnemoTroN Soundtracker 2.6 / Ice Tracker Amiga Module (`.mod` / `.mtn`, 1990 MnemoTroN).
/// Layout: `31 × 30`B sample headers (`vol <= 0x40`), `song_len` (`1..=128`) at `@950`, `"MTN\0"` or `"IT10"` at `@1464..1468`.
pub(crate) fn verify_soundtracker_26(prefix: &[u8], size: u64) -> Option<Label> {
    const ST26_TAG_OFF: usize = 1464; // @1464..1468 (0x5B8): "MTN\0" or "IT10"
    const ST26_TAG_MTN: &[u8; 4] = b"MTN\x00";
    const ST26_TAG_IT10: &[u8; 4] = b"IT10";
    require!(
        size >= MIN_SOUNDTRACKER_26_SIZE,
        prefix.len() >= 1468,
        eq_at(prefix, ST26_TAG_OFF, ST26_TAG_MTN) || eq_at(prefix, ST26_TAG_OFF, ST26_TAG_IT10),
        (1..=128).contains(&u8_at(prefix, 950)?),
        has_non_ascii_text_byte(prefix.get(..1084)?)
    );
    for i in 0..31usize {
        let off = 20 + i * 30;
        require!(u8_at(prefix, off + 25)? <= 0x40);
    }
    Some(Label::Soundtracker26)
}

/// Sound eXchange Native Audio (`.sox`, `".SoX"` LE or `"XoS."` BE).
pub(crate) fn verify_sox_native_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SOX_AUDIO_SIZE, prefix.len() >= 28);
    let (hdr_size, ch) = if starts(prefix, b".SoX") {
        let c24 = le32(prefix, 24)?;
        let c20 = le32(prefix, 20)?;
        (le32(prefix, 4)?, if (1..=256).contains(&c24) { c24 } else { c20 })
    } else if starts(prefix, b"XoS.") {
        let c24 = be32(prefix, 24)?;
        let c20 = be32(prefix, 20)?;
        (be32(prefix, 4)?, if (1..=256).contains(&c24) { c24 } else { c20 })
    } else {
        return None;
    };
    require!((20..=65536).contains(&hdr_size), (1..=256).contains(&ch));
    Some(Label::SoxAudio)
}

/// Square Enix AKB Audio Bank Container (.akb, 2010) Square Enix.
pub(crate) fn verify_sqex_akb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SQEX_AKB_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"AKB "),
        matches!(le16(prefix, 6), Some(0x44 | 0x9C)),
        le32(prefix, 28) == le16(prefix, 6).map(u32::from)
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::SqexAkb)
}

/// Square Enix Sound Container Data audio bank (`.scd`, 2007 Square Enix, `"SEDBSSCF"`).
///
/// Header layout:
/// - `@0..8`: `"SEDBSSCF"`
/// - `@8..12`: `u32` (LE or BE per `@12`) `version` (`2..=4`)
/// - `@12`: `u8` `endianness` (`0` = LE, `1` = BE)
/// - `@13`: `u8` `sscf_flag` (`0x04`)
/// - `@14..16`: `u16` `tables_offset` (`0x20..=0x200`, 16-byte aligned, typically `0x30`)
/// - `@16..20` (LE) or `@20..24` (BE): `u32` `file_size` (`>= 64`)
pub(crate) fn verify_square_enix_scd(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 8] = b"SEDBSSCF"; // @0..8
    const VERSION_OFF: usize = 8; // @8..12: u32 version (2..=4)
    const ENDIAN_OFF: usize = 12; // @12: 0 = LE, 1 = BE
    const FLAG_OFF: usize = 13; // @13: 0x04
    const TABLES_OFF: usize = 14; // @14..16: u16 tables header offset (16-byte aligned)

    require!(
        size >= MIN_SQUARE_ENIX_SCD_SIZE,
        prefix.len() >= 24,
        starts(prefix, MAGIC),
        u8_at(prefix, FLAG_OFF) == Some(0x04)
    );
    let endian = u8_at(prefix, ENDIAN_OFF)?;
    let (version, tables_off, file_size) = match endian {
        0 => (le32(prefix, VERSION_OFF)?, le16(prefix, TABLES_OFF)?, le32(prefix, 16)?),
        1 => (be32(prefix, VERSION_OFF)?, be16(prefix, TABLES_OFF)?, be32(prefix, 20)?),
        _ => return None,
    };
    require!(
        (2..=4).contains(&version),
        (0x20..=0x200).contains(&tables_off),
        tables_off % 16 == 0,
        file_size >= 64
    );
    Some(Label::SquareEnixScd)
}

/// Square Enix SeWave Audio Stream (`.spw`, 2002 Square Enix, `"SeWave\0\0"`).
///
/// Header layout:
/// - `@0..8`: `"SeWave\0\0"`
/// - `@8..12`: `u32 LE` `file_size` (`>= 48`)
/// - `@12..16`: `u32 LE` `codec` (`0` = PS-ADPCM, `1` = PCM, `3` = ATRAC3)
/// - `@36..40`: `u32 LE` `start_offset` (`0x30..=0x1000`)
/// - `@42`: `u8` `channels` (`1..=8`)
pub(crate) fn verify_square_enix_spw(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 8] = b"SeWave\0\0"; // @0..8
    const FILE_SIZE_OFF: usize = 8; // @8..12: u32 LE total size (>= 48)
    const CODEC_OFF: usize = 12; // @12..16: u32 LE codec (0, 1, or 3)
    const START_OFF: usize = 36; // @36..40: u32 LE stream start offset (0x30..=0x1000)
    const CHANNELS_OFF: usize = 42; // @42: u8 channel count (1..=8)

    require!(
        size >= MIN_SQUARE_ENIX_SPW_SIZE,
        prefix.len() >= 44,
        starts(prefix, MAGIC),
        le32(prefix, FILE_SIZE_OFF)? >= 48,
        matches!(le32(prefix, CODEC_OFF), Some(0 | 1 | 3)),
        (0x30..=0x1000).contains(&le32(prefix, START_OFF)?),
        (1..=8).contains(&u8_at(prefix, CHANNELS_OFF)?)
    );
    Some(Label::SquareEnixSpw)
}

/// Targhan / Arkos Amstrad CPC Starkos Song (`.sks`): `"STK1.0SONG"` at `@0..10`.
pub(crate) fn verify_starkos_song(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_STARKOS_SIZE, prefix.len() >= 16, starts(prefix, b"STK1.0SONG"));
    Some(Label::Sks)
}

/// Fairlight Startrekker AM Synth Instrument Bank (`.nt`, 1990 Bjorn Wesen).
pub(crate) fn verify_startrekker_nt(prefix: &[u8], size: u64) -> Option<Label> {
    const NT_MAGIC: &[u8; 16] = b"ST1.2 ModuleINFO"; // @0..16: Startrekker 1.2 AM bank header
    const WORD_STRIDE: &[u8; 8] = hex2bin!("00 18 00 18 00 18 00 18"); // @16..24: 24-byte synth slot stride words
    require!(
        size >= MIN_STARTREKKER_BANK_SIZE,
        prefix.len() >= 32,
        starts(prefix, NT_MAGIC),
        eq_at(prefix, 16, WORD_STRIDE)
    );
    Some(Label::StartrekkerNt)
}

/// Stone Amiga Stonetracker Module (`.spm`): `"SPM\x02"` at `@0..4` + printable ASCII title at `@4`.
pub(crate) fn verify_stonetracker_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_STONETRACKER_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"SPM\x02"),
        u8_at(prefix, 4)?.is_ascii_graphic()
    );
    Some(Label::Spm)
}

/// Amiga Stonetracker Sample Bank (`.sps`, 1995).
pub(crate) fn verify_stonetracker_sps(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_STONETRACKER_SPS_SIZE,
        prefix.len() >= 24,
        starts(prefix, STONETRACKER_SPS_MAGIC),
        u8_at(prefix, 4)? <= 3,
        (1..=64).contains(&u8_at(prefix, 5)?),
        get_slice(prefix, 6, 18)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b)),
        be32(prefix, 18)? as u64 <= size
    );
    Some(Label::StonetrackerSps)
}

/// WarmPlace SunVox Modular Synthesizer Project (`.sunvox`): `"SVOX\0\0\0\0"` at `@0..8` + `"VERS"` or `"BVER"` at `@8..12`.
pub(crate) fn verify_sunvox_module(prefix: &[u8], size: u64) -> Option<Label> {
    const FIRST_CHUNK_OFFSET: usize = 8; // @8: "VERS" or "BVER" chunk tag
    require!(
        size >= MIN_SUNVOX_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"SVOX\x00\x00\x00\x00"),
        eq_at(prefix, FIRST_CHUNK_OFFSET, b"VERS") || eq_at(prefix, FIRST_CHUNK_OFFSET, b"BVER")
    );
    Some(Label::Sunvox)
}

/// Patrick Meng Amiga Symphonie Pro Module (`.symmod`): `"SymM\0\0\0\x01"` at `@0..8`.
pub(crate) fn verify_symphonie_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SYMPHONIE_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"SymM\x00\x00\x00\x01")
    );
    Some(Label::Symmod)
}

/// Synder SNG-Player Chiptune (`.sng`): `"SYNB"` / `"SYNC"` / `"SYND"` / `"SYNF"` / `"SYNH"` + LE16 replay Hz (`25..=1000`) at `@4..6`, `0` at `@8..10`, header length (`32..=512`) at `@10..12`.
pub(crate) fn verify_synder_sng_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SYNDER_SNG_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"SYNB")
            || starts(prefix, b"SYNC")
            || starts(prefix, b"SYND")
            || starts(prefix, b"SYNF")
            || starts(prefix, b"SYNH"),
        (25..=1000).contains(&le16(prefix, 4)?),
        le16(prefix, 8) == Some(0),
        (32..=512).contains(&le16(prefix, 10)?)
    );
    Some(Label::SynderSng)
}

/// Synth Dream Amiga Music Module (`sdr.*`, 1991 Synth Dream).
pub(crate) fn verify_synth_dream_sdr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SYNTH_DREAM_SDR_SIZE,
        prefix.len() >= 28,
        (0x28..size).contains(&(be32(prefix, 0)? as u64)),
        be32(prefix, 4) == Some(0x1C),
        be32(prefix, 8) == Some(0x28)
    );
    for i in 3..7usize {
        let off = be32(prefix, i * 4)? as u64;
        require!((0x28..size).contains(&off));
    }
    Some(Label::SynthDreamSdr)
}

/// Shiru TFM Music Maker YM2203/YM2612 Module (`.tfe`): `"TFMfmtV2"` or `"TFMfmtV1"` at `@0..8`.
pub(crate) fn verify_tfm_music_maker(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TFM_MUSIC_MAKER_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"TFMfmtV2") || starts(prefix, b"TFMfmtV1")
    );
    Some(Label::Tfe)
}

/// Chris Huelsbeck TFMX Amiga Module (`"TFMX-SONG"` / `"TFMX "` / `"tfmx "`).
pub(crate) fn verify_tfmx_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TFMX_MOD_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"TFMX-SONG")
            || starts(prefix, b"TFMX_SONG")
            || starts(prefix, b"TFMX ")
            || starts(prefix, b"tfmxsong")
            || starts(prefix, b"tfmx "),
        prefix.get(5..32)?.contains(&0)
            || has_non_ascii_text_byte(prefix.get(..prefix.len().min(512))?)
    );
    Some(Label::TfmxMod)
}

/// Klevgränd Tomofon Audio Model (`.tomodel`).
pub(crate) fn verify_tomofon_tomodel(prefix: &[u8], size: u64) -> Option<Label> {
    // @0..32: header size/tag 0x2C + 7 default normalized parameter floats (0.0, 0.0, 0.5, 0.5, 1.0, ~1.0, 0.5)
    const TOMOFON_HDR: &[u8; 32] = hex2bin!(
        "2C 00 00 00 00 00 00 00 00 00 00 00 00 00 00 3F 00 00 00 3F 00 00 80 3F 58 FF 7F 3F 00 00 00 3F"
    );

    require!(size >= MIN_TOMOFON_TOMODEL_SIZE, prefix.len() >= 32, starts(prefix, TOMOFON_HDR));
    Some(Label::TomofonTomodel)
}

/// Klevgränd Tomofon Audio Patch (velKqdiP, .topatch).
pub(crate) fn verify_tomofon_topatch(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TOMOFON_TOPATCH_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"velKqdiP\0\0\0\x01"),
        le32(prefix, 12).unwrap_or(0) >= 16
    );
    require!(has_non_ascii_text_byte(prefix.get(..24)?));
    Some(Label::TomofonTopatch)
}

/// Tomy Tracker Amiga Music Module (`.sg`, 1991 Tomy).
pub(crate) fn verify_tomy_tracker_sg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TOMY_TRACKER_SG_SIZE,
        prefix.len() >= 24,
        (be32(prefix, 0)? as u64) == size
    );
    let smp_off = be32(prefix, 4)? as u64;
    let pat_hdr = be16(prefix, 8)?;
    let o12 = be32(prefix, 12)? as u64;
    require!(
        (0x80..size).contains(&smp_off),
        pat_hdr > 0 && (pat_hdr & 0x0F) == 0,
        be16(prefix, 10) == Some(0),
        (0x40..smp_off).contains(&o12)
    );
    Some(Label::TomyTrackerSg)
}

/// Crazy Crack TrackerPacker Module (`"MEXX"` v1, `"MEXX_TP"` v2, or `"CPLX_TP3"` v3).
pub(crate) fn verify_trackerpacker_module(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_TRACKERPACKER_SIZE && prefix.len() >= 30);
    let is_tp2_tp3 = (starts(prefix, b"MEXX_TP") || starts(prefix, b"CPLX_TP"))
        && prefix.get(7..30)?.contains(&0);
    let is_tp1 = starts(prefix, b"MEXX")
        && be32(prefix, 4).is_some_and(|off| off >= 64 && (off as u64) < size)
        && prefix.get(8..28)?.contains(&0)
        && get_slice(prefix, 8, 28)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b));
    require!(is_tp2_tp3 || is_tp1);
    Some(Label::TrackerpackerMexx)
}

/// T'SoundSystem Tracker Module (`.tsd`, 1993).
pub(crate) fn verify_tsoundsystem(prefix: &[u8], size: u64) -> Option<Label> {
    const TSD_MAGIC: &[u8; 15] = b"T'SoundSystem\x00\x00"; // @0..15: T'SoundSystem header
    require!(size >= MIN_TSOUNDSYSTEM_SIZE, prefix.len() >= 20, starts(prefix, TSD_MAGIC));
    Some(Label::Tsoundsystem)
}

/// Brain Control Tunefish 2 Synth Module (`.tf2m`, 2009 Brain Control).
/// Layout: `le32(0)` in `0x48..size`, `le32(4) == 0x44`, 16 strictly increasing `u32 LE` section offsets `< size`.
pub(crate) fn verify_tunefish2_tf2m(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TUNEFISH2_TF2M_SIZE,
        prefix.len() >= 68,
        (0x48..=10_000_000).contains(&(le32(prefix, 0)? as u64)),
        le32(prefix, 4) == Some(0x44)
    );
    let mut prev = 0x44u64;
    for i in 1..16usize {
        let off = le32(prefix, 4 + i * 4)? as u64;
        require!(off > prev && off < size);
        prev = off;
    }
    Some(Label::Tunefish2Tf2m)
}

/// TwinTeam Twin TrackPlayer Compressed OPL2 Module (`.dmo`, 1995 TwinTeam).
pub(crate) fn verify_twin_track_dmo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TWIN_TRACK_DMO_SIZE,
        prefix.len() >= 16,
        (0x16..=0x17).contains(&u8_at(prefix, 1)?),
        (0x20..=0x2F).contains(&u8_at(prefix, 3)?),
        (0x02..=0x05).contains(&u8_at(prefix, 5)?),
        le32(prefix, 6) == Some(1),
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::TwinTrackDmo)
}

/// NTT TwinVQ Audio (`.vqf`, `"TWIN97012000"` / `"TWIN00052200"` + `"COMM"`).
pub(crate) fn verify_twinvq_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TWINVQ_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"TWIN97012000") || starts(prefix, b"TWIN00052200"),
        be32(prefix, 12)? >= 12,
        eq_at(prefix, 16, b"COMM")
    );
    Some(Label::TwinvqVqf)
}

/// Origin Systems Ultima VI AdLib Music (`.m`, 1990 Origin Systems).
pub(crate) fn verify_ultima6_m(prefix: &[u8], size: u64) -> Option<Label> {
    const U6_MARKER: &[u8; 5] = hex2bin!("00 00 00 07 01"); // @2..7: high u16 of decompressed size (0) + initial LZW stream header
    require!(size >= MIN_ULTIMA6_M_SIZE, prefix.len() >= 16);
    let uncomp_len = le32(prefix, 0)? as u64;
    require!(uncomp_len >= size && uncomp_len <= 65_536, eq_at(prefix, 2, U6_MARKER));
    Some(Label::Ultima6M)
}

/// MAS UltraTracker Module (`.ult`): `"MAS_UTrack_V00"` at `@0..14` + ASCII version `'1'..='4'` at `@14`.
pub(crate) fn verify_ultratracker_module(prefix: &[u8], size: u64) -> Option<Label> {
    const VER_DIGIT_OFFSET: usize = 14; // @14: '1'..'4' version character
    require!(
        size >= MIN_ULTRATRACKER_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"MAS_UTrack_V00"),
        (b'1'..=b'4').contains(&u8_at(prefix, VER_DIGIT_OFFSET)?)
    );
    Some(Label::Ult)
}

/// Unique Development Amiga Music Module (`.uds`, 1991 Unique Development).
pub(crate) fn verify_unique_dev_uds(prefix: &[u8], size: u64) -> Option<Label> {
    const UDS_MAGIC: &[u8; 8] = b"S\x8FNG.B4\x00"; // @0..8: Unique Development song header
    require!(size >= MIN_UNIQUE_DEV_SIZE, prefix.len() >= 16, starts(prefix, UDS_MAGIC));
    Some(Label::UniqueDevUds)
}

/// UTAU Singing Synthesizer Fundamental Frequency Table (.frq, 2008) Ameya/Ayame.
pub(crate) fn verify_utau_frq(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UTAU_FRQ_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"FREQ0003\0\x01\0\0"),
        get_slice(prefix, 20, 36)? == [0u8; 16]
    );
    Some(Label::UtauFrq)
}

/// Video Game Music Sound Chip Log (.vgm).
pub(crate) fn verify_vgm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_VGM_SIZE && prefix.len() >= 64 && starts(prefix, b"Vgm "));
    let eof_offset = le32(prefix, 4)? as u64;
    let version = le32(prefix, 8)?;
    require!(
        eof_offset >= 60 && (version & 0xFFFF_0000) == 0 && (0x0100..=0x0299).contains(&version)
    );
    Some(Label::Vgm)
}

/// vgmstream Generic Headered Audio Stream (.genh).
pub(crate) fn verify_vgmstream_genh(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_GENH_AUDIO_SIZE, prefix.len() >= 36, starts(prefix, b"GENH"));
    let channels = le32(prefix, 4)?;
    let sample_rate = le32(prefix, 12)?;
    let codec = le32(prefix, 24)?;
    let start_off = le32(prefix, 28)? as u64;
    let hdr_sz = le32(prefix, 32)? as u64;
    require!(
        (1..=64).contains(&channels),
        (1_000..=192_000).contains(&sample_rate),
        codec <= 64,
        (0x28..=size).contains(&start_off),
        (0x28..=start_off).contains(&hdr_sz)
    );
    Some(Label::Genh)
}

/// VIO v2 Music Score (.vio).
pub(crate) fn verify_vio2_score(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VIO2_SCORE_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"VIO\x02\0\0\x01\0"),
        get_slice(prefix, 8, 24)? == [0u8; 16]
    );
    Some(Label::Vio2Score)
}

/// Atomix VirtualDJ Audio/Video Sampler Clip (.vdjsample, 2014) Atomix Productions.
pub(crate) fn verify_virtualdj_vdjsample(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VIRTUALDJ_VDJSAMPLE_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"VDJ\0"),
        (0x100..=0x1000).contains(&le32(prefix, 4).unwrap_or(0)),
        le32(prefix, 8) == Some(0x70),
        get_slice(prefix, 16, 32)? == [0u8; 16]
    );
    Some(Label::VirtualdjVdjsample)
}

/// Thomas Winischhofer Voodoo Supreme Synthesizer Module (`.vss`, 1993).
pub(crate) fn verify_voodoo_vss(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VOODOO_VSS_SIZE,
        prefix.len() >= 24,
        starts(prefix, VOODOO_VSS_MAGIC),
        (16..size as u32).contains(&be32(prefix, 4)?),
        (16..size as u32).contains(&be32(prefix, 8)?)
    );
    Some(Label::VoodooVss)
}

/// Voyetra PatchMaster / Sideman MIDI Synthesizer Bank (`.b*` / `.c*`, 1987, Voyetra Technologies).
///
/// Header layout: `@0..2 u16 LE magic` (`0x05DF` or `0x05E0`), `@2..4 u16 LE version == 1`,
/// `@4..6 u16 LE device_id` (`1..=255`), `@6..22` NUL-padded ASCII synthesizer model name,
/// `@22 == 0`, `@23 u8 num_banks` (`1..=14`), `@24 == 0`, followed at `@25` by `(num_banks + 1)`
/// strictly increasing `u32 LE` offsets with `bank_offset[0] == 84` and exact
/// `bank_offset[num_banks] + 18 == size`.
pub(crate) fn verify_voyetra_patch_bank(prefix: &[u8], size: u64) -> Option<Label> {
    const VER_OFF: usize = 2; // @2..4: u16 LE format version (1)
    const DEV_ID_OFF: usize = 4; // @4..6: u16 LE synthesizer device ID (1..=255)
    const NAME_OFF: usize = 6; // @6..22: NUL-padded 16-byte ASCII synth model name
    const PAD22_OFF: usize = 22; // @22: reserved zero byte
    const NBANKS_OFF: usize = 23; // @23: u8 number of patch banks (1..=14)
    const PAD24_OFF: usize = 24; // @24: reserved zero byte
    const OFFS_START: usize = 25; // @25..: (num_banks + 1) x u32 LE offsets
    const BANK_DATA_START: u32 = 84; // 0x54 fixed bank data start offset
    const EOF_TRAILER_LEN: u64 = 18; // fixed 18-byte EOF checksum/comment trailer

    require!(
        size >= MIN_VOYETRA_BANK_SIZE,
        prefix.len() >= BANK_DATA_START as usize,
        starts(prefix, VOYETRA_BANK_MAGIC_STD) || starts(prefix, VOYETRA_BANK_MAGIC_D50),
        le16(prefix, VER_OFF)? == 1,
        (1..=255).contains(&le16(prefix, DEV_ID_OFF)?),
        u8_at(prefix, PAD22_OFF)? == 0,
        u8_at(prefix, PAD24_OFF)? == 0
    );
    let synth_name = get_slice(prefix, NAME_OFF, PAD22_OFF)?;
    let name_len = synth_name.iter().position(|&b| b == 0).unwrap_or(synth_name.len());
    require!(
        name_len >= 1,
        synth_name[..name_len].iter().all(|&b| (0x20..=0x7E).contains(&b)),
        synth_name[name_len..].iter().all(|&b| b == 0)
    );
    let num_banks = u8_at(prefix, NBANKS_OFF)? as usize;
    require!((1..=14).contains(&num_banks));
    let mut prev = le32(prefix, OFFS_START)?;
    require!(prev == BANK_DATA_START);
    for i in 1..=num_banks {
        let cur = le32(prefix, OFFS_START + i * 4)?;
        require!(cur > prev);
        prev = cur;
    }
    require!((prev as u64).checked_add(EOF_TRAILER_LEN)? == size);

    Some(Label::VoyetraPatchBank)
}

/// Steinberg VST 2.x Effect Preset or Bank file (`.fxp` / `.fxb`).
pub(crate) fn verify_vst2_fxp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VST2_FXP_SIZE,
        prefix.len() >= 28,
        starts(prefix, VST2_CCNK_MAGIC),
        be32(prefix, VST2_BYTE_SIZE_OFFSET)? >= 20,
        matches!(get_slice(prefix, 8, 12), Some(b"FxCk" | b"FPCh" | b"FxBk" | b"FBCh")),
        (1..=2).contains(&be32(prefix, VST2_VERSION_OFFSET)?),
        be32(prefix, VST2_FX_ID_OFFSET)? != 0
    );
    Some(Label::Vst2Fxp)
}

/// Steinberg VST 3 Preset file (`.vstpreset`).
pub(crate) fn verify_vst3_vstpreset(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VST3_VSTPRESET_SIZE,
        prefix.len() >= 48,
        starts(prefix, VST3_PRESET_MAGIC),
        get_slice(prefix, VST3_CLASS_ID_RANGE.start, VST3_CLASS_ID_RANGE.end)?
            .iter()
            .all(|&b| b.is_ascii_hexdigit()),
        (48..=268_435_456).contains(&le64(prefix, VST3_LIST_OFFSET_OFFSET)?)
    );
    Some(Label::Vst3Vstpreset)
}

/// CD Projekt RED REDengine 3 / The Witcher 3 Speech Container (`"CPSW"` / `"TEMS"`).
///
/// Header layout:
/// - `@0..4`: `"CPSW"` (`.w3speech`, reversed `"WSPC"`) or `"TEMS"` (`.w3speech.meta`)
/// - `@4..8`: `u32 LE` REDengine version (`100..=250`, e.g., `163`–`164`)
pub(crate) fn verify_witcher3_w3speech(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4..8: u32 LE REDengine version (100..=250; upper 2 bytes are 0x0000)

    require!(
        size >= MIN_WITCHER3_SPEECH_SIZE,
        prefix.len() >= 12,
        starts(prefix, WITCHER3_SPEECH_MAGIC) || starts(prefix, WITCHER3_SPEECH_META_MAGIC),
        (100..=250).contains(&le32(prefix, VERSION_OFF)?)
    );
    Some(Label::Witcher3W3speech)
}

/// Audiokinetic Wwise SoundBank / File Package (.bnk / .pck).
pub(crate) fn verify_wwise_bnk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WWISE_BNK_SIZE
            && prefix.len() >= 16
            && (starts(prefix, b"BKHD") || starts(prefix, b"AKPK"))
            && prefix.get(4..12)?.contains(&0)
    );
    let chunk_len_le = le32(prefix, 4).unwrap_or(0);
    let chunk_len_be = be32(prefix, 4).unwrap_or(0);
    require!(
        (chunk_len_le >= 8 && (chunk_len_le as u64) <= size)
            || (chunk_len_be >= 8 && (chunk_len_be as u64) <= size)
    );
    Some(Label::WwiseBnk)
}

/// Sharp X68000 MXDRV Music Module (`.mdx`, 1989 Katsuhiko Honda / Hudson Soft).
pub(crate) fn verify_x68000_mdx(prefix: &[u8], size: u64) -> Option<Label> {
    const TITLE_EOF_SEQ: &[u8; 3] = hex2bin!("0D 0A 1A"); // @eof: CR+LF+EOF terminating Shift-JIS song title
    const MAX_TITLE_SCAN: usize = 512; // @0..512: maximum title scan window
    const MAX_PDX_NAME_LEN: usize = 64; // maximum NUL-terminated .pdx filename length
    const CH0_OFF_STD: u16 = 0x0014; // 2 + 9 * 2 bytes (8 OPM + 1 ADPCM channel pointers)
    const CH0_OFF_EXPCM: u16 = 0x0026; // 2 + 18 * 2 bytes (EX-PCM expanded channel pointers)

    require!(size >= MIN_X68000_MDX_SIZE, prefix.len() >= 28);
    let win = get_slice(prefix, 0, prefix.len().min(MAX_TITLE_SCAN))?;
    let eof_pos = memchr::memmem::find(win, TITLE_EOF_SEQ)?;
    let pdx_start = eof_pos + 3;
    let pdx_limit = (pdx_start + MAX_PDX_NAME_LEN + 1).min(prefix.len());
    require!(pdx_limit > pdx_start);
    let nul_rel = get_slice(prefix, pdx_start, pdx_limit)?.iter().position(|&b| b == 0)?;
    let pdx_slice = get_slice(prefix, pdx_start, pdx_start + nul_rel)?;
    require!(pdx_slice.iter().all(|&b| b >= 0x20 && b != 0x7F));
    let base = pdx_start + nul_rel + 1;
    require!(prefix.len() >= base + 20);
    let voice_off = be16(prefix, base)?;
    let ch_a_off = be16(prefix, base + 2)?;
    require!(
        matches!(ch_a_off, CH0_OFF_STD | CH0_OFF_EXPCM),
        voice_off >= ch_a_off,
        (base as u64) + (voice_off as u64) <= size
    );
    let mut prev = ch_a_off;
    for ch in 1..9usize {
        let cur = be16(prefix, base + 2 + ch * 2)?;
        require!(cur >= prev && cur <= voice_off);
        prev = cur;
    }
    Some(Label::X68000Mdx)
}

/// Microsoft XACT Global Settings file (`.xgs`, 2002 Microsoft, `"XGSF"` LE or `"FSGX"` BE).
///
/// Header layout (LE `"XGSF"` or BE `"FSGX"`):
/// - `@0..4`: `"XGSF"` (LE) or `"FSGX"` (BE)
/// - `@4..6`: `u16` `tool_version` (`1..=64`)
/// - `@6..8`: `u16` `format_version` (`1..=64`)
/// - `@19..21`: `u16` `num_categories` (`<= 4096`)
/// - `@21..23`: `u16` `num_variables` (`<= 4096`)
/// - `@33..37`: `u32` `categories_offset` (`0x20..=0x200`, `0x4D` in XACT3)
pub(crate) fn verify_xact_global_settings(prefix: &[u8], size: u64) -> Option<Label> {
    const TOOL_VER_OFF: usize = 4; // @4..6: u16 tool version (1..=64)
    const FMT_VER_OFF: usize = 6; // @6..8: u16 format version (1..=64)
    const CAT_COUNT_OFF: usize = 19; // @19..21: u16 category count (<= 4096)
    const VAR_COUNT_OFF: usize = 21; // @21..23: u16 variable count (<= 4096)
    const CAT_OFF_FIELD: usize = 33; // @33..37: u32 categories table offset (0x20..=0x200)

    require!(size >= MIN_XACT_XGS_SIZE, prefix.len() >= 37);
    let is_be = if starts(prefix, b"XGSF") {
        false
    } else if starts(prefix, b"FSGX") {
        true
    } else {
        return None;
    };
    let read16 = |off| if is_be { be16(prefix, off) } else { le16(prefix, off) };
    let read32 = |off| if is_be { be32(prefix, off) } else { le32(prefix, off) };
    let tool_ver = read16(TOOL_VER_OFF)?;
    let fmt_ver = read16(FMT_VER_OFF)?;
    let num_cat = read16(CAT_COUNT_OFF)?;
    let num_var = read16(VAR_COUNT_OFF)?;
    let cat_off = read32(CAT_OFF_FIELD)?;
    require!(
        (1..=64).contains(&tool_ver),
        (1..=64).contains(&fmt_ver),
        num_cat <= 4096,
        num_var <= 4096,
        (0x20..=0x200).contains(&cat_off)
    );
    Some(Label::XactXgs)
}

/// Microsoft XACT Sound Bank (`.xsb`, 2002 Microsoft, `"SDBK"` LE or `"KBDS"` BE).
///
/// Header layout:
/// - `@0..4`: `"SDBK"` (little-endian PC/Xbox) or `"KBDS"` (big-endian Xbox 360)
/// - `@4..6`: `u16` `tool_version` (`1..=64`)
/// - `@6..8`: `u16` `format_version` (`1..=64` for XACT3 `tool_version >= 42`)
/// - `@18`: `u8` `platform` (`1` = PC, `2`/`3` = Xbox 360) for XACT3
/// - `@27`: `u8` `wavebanks_count` (`<= 64`) for XACT3
pub(crate) fn verify_xact_sound_bank(prefix: &[u8], size: u64) -> Option<Label> {
    const TOOL_VER_OFF: usize = 4; // @4..6: u16 tool version (1..=64)
    const FMT_VER_OFF: usize = 6; // @6..8: u16 format version (1..=64)
    const PLATFORM_OFF: usize = 18; // @18: u8 target platform (1..=3)
    const WB_COUNT_OFF: usize = 27; // @27: u8 referenced wavebank count (<= 64)

    require!(size >= MIN_XACT_XSB_SIZE, prefix.len() >= 32);
    let is_be = if starts(prefix, b"SDBK") {
        false
    } else if starts(prefix, b"KBDS") {
        true
    } else {
        return None;
    };
    let tool_ver = if is_be { be16(prefix, TOOL_VER_OFF)? } else { le16(prefix, TOOL_VER_OFF)? };
    require!((1..=64).contains(&tool_ver));
    if tool_ver >= 42 {
        let fmt_ver = if is_be { be16(prefix, FMT_VER_OFF)? } else { le16(prefix, FMT_VER_OFF)? };
        let platform = u8_at(prefix, PLATFORM_OFF)?;
        let wb_count = u8_at(prefix, WB_COUNT_OFF)?;
        require!((1..=64).contains(&fmt_ver), (1..=3).contains(&platform), wb_count <= 64);
    } else {
        require!(has_non_ascii_text_byte(prefix.get(4..32)?));
    }
    Some(Label::XactXsb)
}

/// Microsoft DirectX / Xbox Cross-Platform Audio Creation Tool Wave Bank (`.xwb`, `"WBND"` LE / `"DNBW"` BE).
pub(crate) fn verify_xact_wave_bank(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFFSET: usize = 4; // @4..8: u32 tool version (1..=64)
    const SEG0_OFF_OFFSET: usize = 8; // @8..12 (or @12..16 in v42+): first WAVEBANKREGION offset

    require!(size >= MIN_XACT_WAVE_BANK_SIZE, prefix.len() >= 52);
    let (ver, seg_off) = if starts(prefix, b"WBND") {
        (le32(prefix, VERSION_OFFSET)?, le32(prefix, SEG0_OFF_OFFSET)? as u64)
    } else if starts(prefix, b"DNBW") {
        (be32(prefix, VERSION_OFFSET)?, be32(prefix, SEG0_OFF_OFFSET)? as u64)
    } else {
        return None;
    };
    require!((1..=64).contains(&ver), seg_off <= size);
    Some(Label::XactXwb)
}

/// Xbox SDNS ADPCM Audio (`.sdns`, `"SDNS"`).
pub(crate) fn verify_xbox_sdns_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_XBOX_SDNS_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"SDNS"),
        get_slice(prefix, 4, 16)?.iter().any(|&b| b != 0),
        prefix.get(4..16)?.contains(&0)
    );
    Some(Label::XboxSdns)
}

/// Xilam DERF Audio (`.adp`, `"DERF"`).
pub(crate) fn verify_xilam_derf_audio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_XILAM_DERF_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"DERF"),
        matches!(le32(prefix, 4), Some(1 | 2)),
        le32(prefix, 8)? > 0
    );
    Some(Label::XilamDerf)
}

/// D-Lusion X-Tracker Delusion Digital Music File (`.dmf`): `"DDMF"` at `@0..4` + version `1..=10` at `@4`.
pub(crate) fn verify_xtracker_dmf_module(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFFSET: usize = 4; // @4: u8 format version (1..=10)
    require!(
        size >= MIN_XTRACKER_DMF_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"DDMF"),
        (1..=10).contains(&u8_at(prefix, VERSION_OFFSET)?),
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::DmfXtracker)
}

/// ZX Spectrum Sound Tracker 1.3 AY Module (`.st13`, 1992 KSA Software).
pub(crate) fn verify_zx_ay_st13(prefix: &[u8], size: u64) -> Option<Label> {
    const BANNER_OFF: usize = 7; // @7..25: "SOUND TRACKER v1.3"
    const BANNER: &[u8; 18] = b"SOUND TRACKER v1.3";
    require!(
        size >= MIN_ZX_AY_ST13_SIZE,
        prefix.len() >= 28,
        (1..=32).contains(&u8_at(prefix, 0)?),
        eq_at(prefix, BANNER_OFF, BANNER)
    );
    Some(Label::ZxSt13)
}

/// ZX Spectrum Exolon Chip Tracker Module (`.chi`, 1995 Exolon).
pub(crate) fn verify_zx_chip_tracker(prefix: &[u8], size: u64) -> Option<Label> {
    const CHIP_MAGIC: &[u8; 8] = b"CHIPv1.0"; // @0..8: Chip Tracker v1.0 header
    require!(
        size >= MIN_ZX_CHIP_SIZE,
        prefix.len() >= 96,
        starts(prefix, CHIP_MAGIC),
        has_non_ascii_text_byte(prefix.get(8..prefix.len().min(256))?)
    );
    Some(Label::ZxChi)
}

/// ZX Spectrum Digital Reality Fast Tracker Module (`.ftc`, 1997 Digital Reality).
pub(crate) fn verify_zx_fast_tracker(prefix: &[u8], size: u64) -> Option<Label> {
    const MODULE_TAG: &[u8; 8] = b"Module: "; // @0..8: Fast Tracker module prefix
    const EDITOR_OFF: usize = 51; // @51..67: "Fast Tracker v1."
    const EDITOR_TAG: &[u8; 16] = b"Fast Tracker v1.";
    require!(
        size >= MIN_ZX_FTC_SIZE,
        prefix.len() >= 80,
        starts(prefix, MODULE_TAG),
        eq_at(prefix, EDITOR_OFF, EDITOR_TAG),
        has_non_ascii_text_byte(prefix.get(68..80)?)
    );
    Some(Label::ZxFtc)
}

/// ZX Spectrum Fuxoft AY Music Module (`.fxm`, 1998 Frantisek Fuka & Patrik Rak).
pub(crate) fn verify_zx_fuxoft_fxm(prefix: &[u8], size: u64) -> Option<Label> {
    const LOAD_ADDR_OFF: usize = 4; // @4..6: u16 LE Z80 RAM load address (>= 0x4000)
    require!(
        size >= MIN_ZX_FXSM_SIZE,
        prefix.len() >= 20,
        starts(prefix, ZX_FXSM_MAGIC),
        le16(prefix, LOAD_ADDR_OFF)? >= 0x4000,
        has_non_ascii_text_byte(prefix.get(4..20)?)
    );
    Some(Label::ZxFxm)
}

/// ZX Spectrum Polaris Global Tracker Module (`.gtr`, 2001 Polaris).
pub(crate) fn verify_zx_global_tracker(prefix: &[u8], size: u64) -> Option<Label> {
    const GTR_TAG: &[u8; 3] = b"GTR"; // @1..4: Global Tracker signature
    require!(
        size >= MIN_ZX_GTR_SIZE,
        prefix.len() >= 40,
        (1..=64).contains(&u8_at(prefix, 0)?),
        eq_at(prefix, 1, GTR_TAG),
        matches!(u8_at(prefix, 4)?, 0x10 | 0x11),
        le16(prefix, 5) == Some(0)
    );
    Some(Label::ZxGtr)
}

/// ZX Spectrum Sound Tracker Compiled Module (`.stc`): `@0` tempo (`1..=64`), `@1..7` three `u16 LE` offsets, `@7..25` `"SONG BY ST COMPILE"` / `"SONG BY MB COMPILE"`.
pub(crate) fn verify_zx_spectrum_stc(prefix: &[u8], size: u64) -> Option<Label> {
    const TEMPO_OFFSET: usize = 0; // @0: u8 tempo (1..=64)
    const POS_OFF: usize = 1; // @1: u16 LE positions table offset
    const ORN_OFF: usize = 3; // @3: u16 LE ornaments table offset
    const PAT_OFF: usize = 5; // @5: u16 LE patterns table offset
    const BANNER_OFFSET: usize = 7; // @7: 18-byte compiler signature
    require!(
        size >= MIN_ZX_STC_SIZE,
        prefix.len() >= 27,
        (1..=64).contains(&u8_at(prefix, TEMPO_OFFSET)?),
        eq_at(prefix, BANNER_OFFSET, b"SONG BY ST COMPILE")
            || eq_at(prefix, BANNER_OFFSET, b"SONG BY MB COMPILE")
            || eq_at(prefix, BANNER_OFFSET, b"SONG BY ST-COMPILE")
    );
    let pos = le16(prefix, POS_OFF)? as u64;
    let orn = le16(prefix, ORN_OFF)? as u64;
    let pat = le16(prefix, PAT_OFF)? as u64;
    require!(pos >= 27 && pos < size && orn >= 27 && orn < size && pat >= 27 && pat < size);
    Some(Label::Stc)
}

/// ZX Spectrum SQ-Tracker AY Module (`.sqt`, 1993 George K.).
/// Layout: `le16(0) == size` (with up to 16B trailing padding), Z80 RAM pointers `smp < orn <= pat < pos <= loop` at `@2..12`.
pub(crate) fn verify_zx_sq_tracker(prefix: &[u8], size: u64) -> Option<Label> {
    require!((MIN_ZX_SQT_SIZE..=65_552).contains(&size), prefix.len() >= 24);
    let declared = le16(prefix, 0)? as u64;
    require!((MIN_ZX_SQT_SIZE..=size).contains(&declared), size - declared <= 16);
    let smp = le16(prefix, 2)?; // @2..4: sample pointer table base
    let orn = le16(prefix, 4)?; // @4..6: ornament pointer table base
    let pat = le16(prefix, 6)?; // @6..8: pattern pointer table base
    let pos = le16(prefix, 8)?; // @8..10: position list base
    let loop_p = le16(prefix, 10)?; // @10..12: loop position pointer
    let first_smp = le16(prefix, 12)?; // @12..14: first sample data pointer
    let orn_delta = orn.checked_sub(smp)?;
    require!(
        (2..=64).contains(&orn_delta) && orn_delta % 2 == 0,
        pat >= orn,
        pos > pat,
        loop_p >= pos,
        (loop_p.checked_sub(smp)? as u64) + 10 < declared,
        first_smp > smp && first_smp < pos,
        has_non_ascii_text_byte(prefix.get(..prefix.len().min(64))?)
    );
    Some(Label::ZxSqt)
}

/// ZX Spectrum KSA Sound Tracker Pro STP2 Module (`.stp` / `.stp2`, 1991 KSA Software).
pub(crate) fn verify_zx_stp_module(prefix: &[u8], size: u64) -> Option<Label> {
    const KSA_BANNER: &[u8; 28] = b"KSA SOFTWARE COMPILATION OF "; // @10..38: STP2 53B header banner
    require!(
        (MIN_ZX_STP_SIZE..=65536).contains(&size),
        prefix.len() >= 38,
        (1..=32).contains(&u8_at(prefix, 0)?),
        matches!(u8_at(prefix, 9)?, 0x20..=0x2F),
        eq_at(prefix, 10, KSA_BANNER)
    );
    let pos_off = le16(prefix, 1)? as u64;
    let pat_off = le16(prefix, 3)? as u64;
    let orn_off = le16(prefix, 5)? as u64;
    let smp_off = le16(prefix, 7)? as u64;
    require!(pat_off >= 63 && pos_off < size && orn_off < size && smp_off < size);
    Some(Label::ZxStp)
}

/// ZX Spectrum Vortex Project AY/YM Chiptune (`.vtx`, 2000 Roman Sherbakov).
pub(crate) fn verify_zx_vortex_vtx(prefix: &[u8], size: u64) -> Option<Label> {
    const STEREO_OFF: usize = 2; // @2: u8 stereo layout (0..=6)
    const CLOCK_OFF: usize = 5; // @5..9: u32 LE AY/YM chip clock Hz (500_000..=10_000_000)
    const INT_FREQ_OFF: usize = 9; // @9: u8 interrupt rate Hz (25..=200)
    const YEAR_OFF: usize = 10; // @10..12: u16 LE composition year (0 or 1975..=2100)
    const UNPACKED_OFF: usize = 12; // @12..16: u32 LE unpacked YM register dump size

    require!(
        size >= MIN_ZX_VORTEX_VTX_SIZE,
        prefix.len() >= 24,
        starts(prefix, ZX_VTX_AY_MAGIC) || starts(prefix, ZX_VTX_YM_MAGIC),
        u8_at(prefix, STEREO_OFF)? <= 6,
        (500_000..=10_000_000).contains(&le32(prefix, CLOCK_OFF)?),
        (25..=200).contains(&u8_at(prefix, INT_FREQ_OFF)?)
    );
    let year = le16(prefix, YEAR_OFF)?;
    let unpacked = le32(prefix, UNPACKED_OFF)?;
    require!(year == 0 || (1975..=2100).contains(&year), (32..=33_554_432).contains(&unpacked));
    Some(Label::ZxVtx)
}
