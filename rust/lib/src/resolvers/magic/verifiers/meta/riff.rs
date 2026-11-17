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

//! Resource Interchange File Format (`RIFF` / `RIFX` / `RF64`) metaformat verifiers (`ani`, `avi`, `wav`, `webp`, `cdf`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

pub(crate) const RIFF_MAGIC: &[u8; 4] = b"RIFF"; // @0..4: little-endian RIFF container
pub(crate) const RIFX_MAGIC: &[u8; 4] = b"RIFX"; // @0..4: big-endian RIFX container

/// RIFF / RIFX / RF64 container overlap verifier: dispatches on the form type at offset 8
/// between AVI (`"AVI "` + `"LIST"`/`"hdrl"`), WAV (`"WAVE"`), and ANI (`"ACON"`).
pub(crate) fn riff(prefix: &[u8]) -> Option<Label> {
    static WAV_RIFF: Subformat = Subformat::with_ids("RIFF WAVE", "fmt/6", Some("fdd000001"));
    static WAV_RIFX: Subformat = Subformat::new("RIFX WAVE (big-endian)");
    static WAV_RF64: Subformat = Subformat::with_ids("RF64 WAVE", "fmt/712", None);
    const RF64_HDR: &[u8] = bytes!(b"RF64", &u32::MAX.to_le_bytes(), b"WAVEds64"); // @0..16: RF64 WAVE + ds64 chunk
    const CHUNK_SIZE: usize = 4; // @4: u32 — container chunk size
    const FORM_TYPE: usize = 8; // @8..12: 4-byte form type ("AVI ", "WAVE", "ACON")
    const AVI_LIST: &[u8] = b"AVI LIST"; // @8..16: AVI form + initial LIST chunk
    const WAVE_FORM: &[u8] = b"WAVE"; // @8..12: WAVE form type
    const ACON_FORM: &[u8] = b"ACON"; // @8..12: animated cursor form type
    const LIST_SIZE: usize = 16; // @16: u32 LE — LIST/ds64 chunk size
    const HDRL_TAG: &[u8] = b"hdrl"; // @20..24: AVI header list type
    const AVI_MIN_LEN: usize = 24;
    const WAV_MIN_LEN: usize = 12;
    const RF64_MIN_LEN: usize = 48;

    require!(prefix.len() >= WAV_MIN_LEN);
    if starts(prefix, RIFF_MAGIC) {
        if prefix.len() >= AVI_MIN_LEN
            && eq_at(prefix, FORM_TYPE, AVI_LIST)
            && eq_at(prefix, 20, HDRL_TAG)
            && (16..0x0800_0000).contains(&le32(prefix, CHUNK_SIZE)?)
            && (4..0x0800_0000).contains(&le32(prefix, LIST_SIZE)?)
        {
            return Some(Label::Avi);
        }
        if eq_at(prefix, FORM_TYPE, WAVE_FORM)
            && (4..0x0800_0000).contains(&le32(prefix, CHUNK_SIZE)?)
        {
            return with_subformat(Label::Wav, &WAV_RIFF);
        }
        if eq_at(prefix, FORM_TYPE, ACON_FORM)
            && (4..0x0800_0000).contains(&le32(prefix, CHUNK_SIZE)?)
        {
            return Some(Label::Ani);
        }
    } else if starts(prefix, RIFX_MAGIC) {
        if eq_at(prefix, FORM_TYPE, WAVE_FORM)
            && (4..0x0800_0000).contains(&be32(prefix, CHUNK_SIZE)?)
        {
            return with_subformat(Label::Wav, &WAV_RIFX);
        }
    } else if prefix.len() >= RF64_MIN_LEN
        && starts(prefix, RF64_HDR)
        && le32(prefix, LIST_SIZE)? >= 28
    {
        return with_subformat(Label::Wav, &WAV_RF64);
    }
    None
}

/// Google WebP (`webp`, 2010): `"RIFF"` at `@0`, chunk size `12..0x0800_0000` at `@4`, `"WEBP"` form
/// type at `@8`, and `"VP8 "`, `"VP8L"`, `"VP8X"`, or `"ANIM"` chunk tag at `@12`.
pub(crate) fn webp(prefix: &[u8], size: u64) -> Option<Label> {
    static WEBP_LOSSY: Subformat =
        Subformat::with_ids("WebP (lossy)", "fmt/566", Some("fdd000577"));
    static WEBP_LOSSLESS: Subformat =
        Subformat::with_ids("WebP (lossless)", "fmt/567", Some("fdd000577"));
    static WEBP_EXTENDED: Subformat =
        Subformat::with_ids("WebP (extended)", "fmt/568", Some("fdd000577"));
    static WEBP_ANIMATED: Subformat = Subformat::with_fdd("WebP (animated)", "fdd000577");
    const MIN_WEBP_SIZE: u64 = 20; // 12B RIFF WEBP header + 8B VP8/VP8L chunk header
    const WEBP_FORM: &[u8; 4] = b"WEBP";
    const VP8_LOSSY_CHUNK: &[u8; 4] = b"VP8 ";
    const VP8_LOSSLESS_CHUNK: &[u8; 4] = b"VP8L";
    const VP8_EXTENDED_CHUNK: &[u8; 4] = b"VP8X";
    const VP8_ANIM_CHUNK: &[u8; 4] = b"ANIM";

    require!(
        size >= MIN_WEBP_SIZE,
        prefix.len() >= 16,
        starts(prefix, RIFF_MAGIC),
        (12..0x0800_0000).contains(&le32(prefix, 4)?),
        eq_at(prefix, 8, WEBP_FORM),
    );
    let sub = if eq_at(prefix, 12, VP8_LOSSY_CHUNK) {
        &WEBP_LOSSY
    } else if eq_at(prefix, 12, VP8_LOSSLESS_CHUNK) {
        &WEBP_LOSSLESS
    } else if eq_at(prefix, 12, VP8_EXTENDED_CHUNK) {
        &WEBP_EXTENDED
    } else if eq_at(prefix, 12, VP8_ANIM_CHUNK) {
        &WEBP_ANIMATED
    } else {
        return None;
    };
    with_subformat(Label::Webp, sub)
}

/// Disambiguates `RIFF`/`RIFX`/`RF64` containers (`avi`, `wav`, `ani`, `cdr`, `webp`, `cdf`).
#[inline]
pub(crate) fn riff_or_webp(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| riff(prefix))
        .or_else(|| probe(|| webp(prefix, size)))
        .or_else(|| probe(|| cdf(prefix, size)))
}
