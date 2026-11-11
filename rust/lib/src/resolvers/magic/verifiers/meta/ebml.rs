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

//! Extensible Binary Meta Language (`EBML`) metaformat verifiers (`mkv`, `webm`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

pub(crate) const EBML_MAGIC: &[u8; 4] = &[0x1A, 0x45, 0xDF, 0xA3]; // @0..4: EBML Header element ID

/// Matroska container (`.mkv`): EBML header `1A 45 DF A3`, non-zero/non-FF VINT byte at `@4`,
/// a `DocType` element (`42 82` + VINT size 8 + `"matroska"`) starting in `@5..=28`, and
/// `"matroska"` aligned at offset 8, 24, or 31.
pub(crate) fn mkv(prefix: &[u8]) -> Option<Label> {
    const VINT_FIRST: usize = 4; // @4: u8 — first byte of EBML header size VINT (1..=254)
    const DOCTYPE_ID: &[u8] = &[0x42, 0x82]; // DocType element ID
    const MATROSKA: &[u8] = b"matroska"; // 8-byte DocType string
    const MIN_LEN: usize = 16;

    fn has_doctype_at(h: &[u8], pos: usize) -> bool {
        if !eq_at(h, pos, DOCTYPE_ID) {
            return false;
        }
        let vint_start = pos + 2;
        // Size 8 encoded in 1..=8-byte EBML VINT: leading byte `(0x80 >> k)` followed by `k - 1`
        // zero bytes and final byte `0x08` (or single byte `0x88` for width 1).
        for width in 1..=8usize {
            let lead = 0x80u8 >> (width - 1);
            if width == 1 {
                if u8_at(h, vint_start) == Some(lead | 0x08) && eq_at(h, vint_start + 1, MATROSKA) {
                    return true;
                }
            } else if u8_at(h, vint_start) == Some(lead)
                && h.get(vint_start + 1..vint_start + width - 1)
                    .is_some_and(|zeros| zeros.iter().all(|&b| b == 0))
                && u8_at(h, vint_start + width - 1) == Some(0x08)
                && eq_at(h, vint_start + width, MATROSKA)
            {
                return true;
            }
        }
        false
    }

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, EBML_MAGIC),
        (1..=254).contains(&u8_at(prefix, VINT_FIRST)?),
        eq_at(prefix, 8, MATROSKA) || eq_at(prefix, 24, MATROSKA) || eq_at(prefix, 31, MATROSKA),
        (5..=28).any(|pos| has_doctype_at(prefix, pos)),
    );
    Some(Label::Mkv)
}

/// WebM EBML media container (`webm`, 2010): EBML header `1A 45 DF A3` at `@0` with
/// `DocType` element `42 82 84 "webm"` in the header.
pub(crate) fn webm(prefix: &[u8], size: u64) -> Option<Label> {
    const WEBM_DOCTYPE_ELEMENT: &[u8] = b"\x42\x82\x84webm";

    require!(
        size >= 32,
        prefix.len() >= 16,
        starts(prefix, EBML_MAGIC),
        scan_range(prefix, 4, 64, WEBM_DOCTYPE_ELEMENT),
    );
    Some(Label::Webm)
}

/// Disambiguates EBML media containers (`1A 45 DF A3`) between Matroska (`mkv`) and WebM (`webm`).
#[inline]
pub(crate) fn mkv_or_webm(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| mkv(prefix)).or_else(|| probe(|| webm(prefix, size)))
}
