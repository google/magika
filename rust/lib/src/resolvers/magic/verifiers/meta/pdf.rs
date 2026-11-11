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

//! Portable Document Format (`PDF`) metaformat verifiers (`pdf`, `ai`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// 4-byte Adobe PDF magic (`"%PDF"`).
pub(crate) const PDF_MAGIC: &[u8; 4] = b"%PDF";

pub(crate) const PDF_V1_MAGIC: &[u8] = b"%PDF-1.";

pub(crate) const PDF_V2_MAGIC: &[u8] = b"%PDF-2.";

pub(crate) const PDF_V0_MAGIC: &[u8] = b"%PDF-0.";

pub(crate) const PDF_NULL_MAGIC: &[u8] = b"%PDF-\0";

pub(crate) const PDF_XREF_MAGIC: &[u8] = b"%PDF-xref";

pub(crate) const AI_PRIVATE_DATA: &[u8] = b"AIPrivateData";

pub(crate) const AI_META_DATA: &[u8] = b"AIMetaData";

pub(crate) const AI_CREATOR_TAG: &[u8] = b"/Illustrator ";

pub(crate) const AI_CREATOR_NAME: &[u8] = b"Adobe Illustrator";

/// Adobe PDF (`pdf`, 1993) and PDF-backed Adobe Illustrator Artwork (`ai`) starting at `offset`:
/// validates `%PDF-x.y` (or minimal `%PDF`) header and disambiguates Illustrator private data tags.
pub(crate) fn pdf_or_ai_at(prefix: &[u8], size: u64, offset: usize) -> Option<Label> {
    const MIN_PDF_SIZE: u64 = 8; // "%PDF-1." + delimiter or "%PDF-\0..."
    const PDF_DICT_OPEN: &[u8] = b"<<";
    const PDF_OBJ_TAG: &[u8] = b"obj";
    const PDF_ENDOBJ_TAG: &[u8] = b"endobj";
    const PDF_TRAILER_TAG: &[u8] = b"trailer";
    const PDF_PAGES_TAG: &[u8] = b"/Pages";
    const PDF_ROOT_TAG: &[u8] = b"/Root";

    require!(
        size >= MIN_PDF_SIZE,
        offset + 5 <= prefix.len(),
        eq_at(prefix, offset, PDF_MAGIC),
        // Reject PostScript comments (`%%PDF-...`) or quoted string literals (`"%PDF-..."`, `'%PDF-...'`)
        offset == 0 || !matches!(u8_at(prefix, offset - 1)?, b'%' | b'"' | b'\''),
    );
    let rest = &prefix[offset..];
    let has_body_tokens = contains(rest, PDF_DICT_OPEN)
        && (contains(rest, PDF_ENDOBJ_TAG)
            || contains(rest, PDF_TRAILER_TAG)
            || contains(rest, PDF_PAGES_TAG)
            || contains(rest, PDF_ROOT_TAG));
    let valid_ver =
        if starts(rest, PDF_V1_MAGIC) || starts(rest, PDF_V2_MAGIC) || starts(rest, PDF_V0_MAGIC) {
            if rest.len() >= 8 && u8_at(rest, 7)?.is_ascii_digit() {
                rest.len() == 8
                    || matches!(
                        u8_at(rest, 8)?,
                        b'\r' | b'\n' | b' ' | b'\t' | b'%' | b'/' | 0x00 | 0x80..=0xFF
                    )
            } else {
                // Truncated minor version (`%PDF-1.\r`, `%PDF-1.\n`, `%PDF-1. `, `%PDF-1.\0`)
                rest.len() >= 8
                    && matches!(u8_at(rest, 7)?, b'\r' | b'\n' | b' ' | b'\t' | 0x00)
                    && (offset == 0 || has_body_tokens)
            }
        } else if starts(rest, PDF_NULL_MAGIC) || starts(rest, PDF_XREF_MAGIC) {
            // Minimal/truncated `%PDF-\0` or `%PDF-xref` header with PDF object/trailer syntax
            contains(rest, PDF_DICT_OPEN)
                && (contains(rest, PDF_OBJ_TAG) || contains(rest, PDF_TRAILER_TAG))
        } else {
            // Minimal `%PDF` header (accepted by PDFium/Chrome when followed by PDF object dictionary)
            (offset == 0 || matches!(u8_at(prefix, offset - 1)?, b'\r' | b'\n'))
                && rest.len() >= 5
                && rest[4..rest.len().min(20)]
                    .iter()
                    .any(|&b| matches!(b, b'\r' | b'\n' | b' ' | b'\t' | 0x00))
                && has_body_tokens
        };
    require!(valid_ver);
    let _ = get_slice(prefix, offset, prefix.len().min(offset + 64));
    if contains(prefix, AI_PRIVATE_DATA)
        || contains(prefix, AI_META_DATA)
        || contains(prefix, AI_CREATOR_TAG)
        || contains(prefix, AI_CREATOR_NAME)
    {
        return Some(Label::Ai);
    }
    Some(Label::Pdf)
}

/// Adobe PDF (`pdf`) and PDF-backed Adobe Illustrator Artwork (`ai`) at `@0`
/// (or preceded by a 3-byte UTF-8 BOM `\xEF\xBB\xBF`).
pub(crate) fn pdf_or_ai(prefix: &[u8], size: u64) -> Option<Label> {
    let off = if starts(prefix, UTF8_BOM) { 3 } else { 0 };
    pdf_or_ai_at(prefix, size, off)
}

/// Last-resort heuristic (`heuristic_last`): scans the first 1024 bytes for a `%PDF`
/// signature if no earlier magic or structural format matched at offset 0.
pub(crate) fn pdf_or_ai_scan_1024(prefix: &[u8], size: u64) -> Option<Label> {
    const PDF_SCAN_WINDOW: usize = 1024;
    let win_len = prefix.len().min(PDF_SCAN_WINDOW);
    require!(win_len >= 5);
    let win = &prefix[..win_len];
    let mut search_from = 0usize;
    while search_from + 4 <= win.len() {
        let pos = find_pos(&win[search_from..], PDF_MAGIC)?;
        let off = search_from + pos;
        if let Some(label) = pdf_or_ai_at(prefix, size, off) {
            return Some(label);
        }
        search_from = off + 1;
    }
    None
}
