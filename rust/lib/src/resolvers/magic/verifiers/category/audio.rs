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
/// page sequence number `0` at `@18`, and ≥ 1 segment at `@26`. Names the logical bitstream codec
/// from the O(1) BOS identification packet header at `@(27 + page_segments)` when present.
pub(crate) fn ogg(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"OggS\x00"; // @0..5: Ogg capture pattern + stream_structure_version 0
    const HEADER_TYPE: usize = 5; // @5: u8 — BOS (0x02) or BOS|EOS (0x06)
    const PAGE_SEQ_NO: usize = 18; // @18: u32 LE — initial page sequence number (0)
    const PAGE_SEGMENTS: usize = 26; // @26: u8 — number of segment table entries (≥ 1)
    const HEADER_FIXED_LEN: usize = 27; // fixed Ogg page header preceding the segment table
    const MIN_LEN: usize = 28; // 27-byte Ogg page header + ≥ 1 segment table byte

    static OGG_VORBIS: Subformat = Subformat::with_ids("Ogg Vorbis", "fmt/203", Some("fdd000117"));
    static OGG_OPUS: Subformat = Subformat::with_ids("Ogg Opus", "fmt/946", None);
    static OGG_THEORA: Subformat = Subformat::with_ids("Ogg Theora", "fmt/945", None);
    static OGG_FLAC: Subformat = Subformat::with_ids("Ogg FLAC", "fmt/947", Some("fdd000227"));
    static OGG_SPEEX: Subformat = Subformat::with_ids("Ogg Speex", "fmt/948", Some("fdd000260"));
    static OGG_SKELETON: Subformat = Subformat::new("Ogg Skeleton");
    static OGG_KATE: Subformat = Subformat::new("Ogg Kate");
    static OGG_CELT: Subformat = Subformat::new("Ogg CELT");
    static OGG_DIRAC: Subformat = Subformat::new("Ogg Dirac");
    static OGG_DAALA: Subformat = Subformat::new("Ogg Daala");
    static OGG_MIDI: Subformat = Subformat::new("Ogg MIDI");
    static OGG_PCM: Subformat = Subformat::new("Ogg PCM");

    fn bos_subformat(payload: &[u8]) -> Option<&'static Subformat> {
        match payload.first().copied()? {
            0x01 if starts(payload, b"\x01vorbis") => Some(&OGG_VORBIS),
            b'O' if starts(payload, b"OpusHead") => Some(&OGG_OPUS),
            b'O' if starts(payload, b"OggMIDI\0") => Some(&OGG_MIDI),
            0x80 if starts(payload, b"\x80theora") => Some(&OGG_THEORA),
            0x80 if starts(payload, b"\x80kate\0\0\0") => Some(&OGG_KATE),
            0x80 if starts(payload, b"\x80daala") => Some(&OGG_DAALA),
            0x7F if starts(payload, b"\x7fFLAC") => Some(&OGG_FLAC),
            b'f' if starts(payload, b"fLaC") => Some(&OGG_FLAC),
            b'f' if starts(payload, b"fishead\0") => Some(&OGG_SKELETON),
            b'S' if starts(payload, b"Speex   ") => Some(&OGG_SPEEX),
            b'C' if starts(payload, b"CELT    ") => Some(&OGG_CELT),
            b'B' if starts(payload, b"BBCD") => Some(&OGG_DIRAC),
            b'P' if starts(payload, b"PCM     ") => Some(&OGG_PCM),
            _ => None,
        }
    }

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, HEADER_TYPE)?, 0x02 | 0x06),
        le32(prefix, PAGE_SEQ_NO)? == 0,
    );
    let page_segments = u8_at(prefix, PAGE_SEGMENTS)?;
    require!(page_segments >= 1);
    let payload_off = HEADER_FIXED_LEN + usize::from(page_segments);
    if let Some(sub) = prefix.get(payload_off..).and_then(bos_subformat) {
        return with_subformat(Label::Ogg, sub);
    }
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
