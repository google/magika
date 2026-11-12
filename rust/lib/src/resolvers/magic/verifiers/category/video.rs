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
