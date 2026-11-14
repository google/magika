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

//! Layered, magic-keyed native dispatch cascade for Magika's pre-ML format rules.
//!
//! Verifier source files in [`verifiers`] are grouped into `verifiers/meta/` (container and
//! framing metaformats) and `verifiers/category/` (standalone format categories), plus
//! `verifiers/overlap.rs` for cross-category magic-key collisions. **Metaformats always take
//! precedence over categories**: any format that is a subformat of a metaformat (`ar`, `asf`,
//! `asn1`, `ebml`, `elf`, `iff`, `iso`, `isobmff`, `json`, `ole2`, `pdf`, `riff`, `sqlite`,
//! `xml`, `zip`) is placed in `verifiers/meta/<metaformat>.rs` rather than under `category/`.

pub(crate) mod heuristics;
pub(crate) mod magic16;
pub(crate) mod magic32;
pub(crate) mod magic8;
pub(crate) mod offset_probes;
pub(crate) mod verifiers;

pub(crate) use self::verifiers::*;
pub(crate) use crate::ContentType as Label;

/// UTF-8 Byte Order Mark (`EF BB BF`).
pub(crate) const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// A bounds-checked 32-bit integer reader (`le32` or `be32`).
pub(crate) type ReadU32 = fn(&[u8], usize) -> Option<u32>;

/// Concatenate byte-slice parts into a compile-time constant byte array reference.
macro_rules! bytes {
    ($($part:expr),+ $(,)?) => {{
        const LEN: usize = 0 $(+ { let s: &[u8] = $part; s.len() })+;
        const ARR: [u8; LEN] = {
            let mut out = [0u8; LEN];
            let mut pos = 0;
            $(
                {
                    let slice: &[u8] = $part;
                    let mut i = 0;
                    while i < slice.len() {
                        out[pos] = slice[i];
                        pos += 1;
                        i += 1;
                    }
                }
            )+
            out
        };
        &ARR
    }};
}

/// Parse a canonical `"XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"` GUID literal into its 16-byte
/// mixed-endian binary representation at compile time.
pub(crate) const fn parse_guid(s: &str) -> [u8; 16] {
    const fn nibble(b: u8) -> u8 {
        match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => panic!("invalid hex digit in GUID"),
        }
    }
    const fn byte(raw: &[u8], idx: usize) -> u8 {
        (nibble(raw[idx]) << 4) | nibble(raw[idx + 1])
    }
    let b = s.as_bytes();
    assert!(b.len() == 36, "GUID string must be 36 characters");
    assert!(
        b[8] == b'-' && b[13] == b'-' && b[18] == b'-' && b[23] == b'-',
        "GUID hyphens must be at 8, 13, 18, 23"
    );
    [
        // Data1 (u32 little-endian)
        byte(b, 6),
        byte(b, 4),
        byte(b, 2),
        byte(b, 0),
        // Data2 (u16 little-endian)
        byte(b, 11),
        byte(b, 9),
        // Data3 (u16 little-endian)
        byte(b, 16),
        byte(b, 14),
        // Data4 (8 bytes big-endian)
        byte(b, 19),
        byte(b, 21),
        byte(b, 24),
        byte(b, 26),
        byte(b, 28),
        byte(b, 30),
        byte(b, 32),
        byte(b, 34),
    ]
}

/// Compile-time 16-byte mixed-endian GUID slice from a string literal.
macro_rules! guid {
    ($s:expr) => {{
        const G: [u8; 16] = $crate::resolvers::magic::parse_guid($s);
        &G
    }};
}

/// Early-return `None` on the first false precondition.
macro_rules! require {
    ($($cond:expr),+ $(,)?) => {
        if !($($cond)&&+) {
            return None;
        }
    };
}

pub(crate) use bytes;
pub(crate) use guid;
pub(crate) use require;

/// Big-endian `u32` representation of a 4-byte ASCII tag.
#[inline]
pub(crate) const fn tag_u32(tag: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*tag)
}

/// Little-endian `u32` representation of a 4-byte ASCII tag.
#[inline]
#[allow(dead_code)]
pub(crate) const fn tag_u32_le(tag: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*tag)
}

/// Whether `h` starts with `sig`.
#[inline]
pub(crate) fn starts(h: &[u8], sig: &[u8]) -> bool {
    h.starts_with(sig)
}

/// Whether `sig` appears at byte offset `off` inside `h`.
#[inline]
pub(crate) fn eq_at(h: &[u8], off: usize, sig: &[u8]) -> bool {
    h.get(off..).is_some_and(|s| s.starts_with(sig))
}

/// Bounds-checked `u8` read at `off`.
#[inline]
pub(crate) fn u8_at(h: &[u8], off: usize) -> Option<u8> {
    h.get(off).copied()
}

/// Bounds-checked little-endian `u16` read at `off`.
#[inline]
pub(crate) fn le16(h: &[u8], off: usize) -> Option<u16> {
    Some(u16::from_le_bytes(h.get(off..off.checked_add(2)?)?.try_into().ok()?))
}

/// Bounds-checked big-endian `u16` read at `off`.
#[inline]
pub(crate) fn be16(h: &[u8], off: usize) -> Option<u16> {
    Some(u16::from_be_bytes(h.get(off..off.checked_add(2)?)?.try_into().ok()?))
}

/// Bounds-checked little-endian `u32` read at `off`.
#[inline]
pub(crate) fn le32(h: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_le_bytes(h.get(off..off.checked_add(4)?)?.try_into().ok()?))
}

/// Bounds-checked big-endian `u32` read at `off`.
#[inline]
pub(crate) fn be32(h: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_be_bytes(h.get(off..off.checked_add(4)?)?.try_into().ok()?))
}

/// Bounds-checked little-endian `u64` read at `off`.
#[inline]
pub(crate) fn le64(h: &[u8], off: usize) -> Option<u64> {
    Some(u64::from_le_bytes(h.get(off..off.checked_add(8)?)?.try_into().ok()?))
}

/// Bounds-checked big-endian `u64` read at `off`.
#[inline]
pub(crate) fn be64(h: &[u8], off: usize) -> Option<u64> {
    Some(u64::from_be_bytes(h.get(off..off.checked_add(8)?)?.try_into().ok()?))
}

/// Bounds-checked sub-slice read `[start..end)` inside `h`.
#[inline]
pub(crate) fn get_slice(h: &[u8], start: usize, end: usize) -> Option<&[u8]> {
    h.get(start..end)
}

/// Fast SIMD substring search for `needle` anywhere inside `h`.
#[inline]
pub(crate) fn contains(h: &[u8], needle: &[u8]) -> bool {
    memchr::memmem::find(h, needle).is_some()
}

/// Fast SIMD index search for `needle` inside `h`.
#[inline]
pub(crate) fn find_pos(h: &[u8], needle: &[u8]) -> Option<usize> {
    memchr::memmem::find(h, needle)
}

/// Fast reverse SIMD substring search for `needle` inside `h`.
#[inline]
#[allow(dead_code)]
pub(crate) fn rfind(h: &[u8], needle: &[u8]) -> Option<usize> {
    memchr::memmem::rfind(h, needle)
}

/// Whether `needle` occurs starting at some offset in `lo..=hi` within `h`.
#[inline]
pub(crate) fn scan_range(h: &[u8], lo: usize, hi: usize, needle: &[u8]) -> bool {
    let end = h.len().min(hi.saturating_add(needle.len()));
    h.get(lo..end).is_some_and(|w| memchr::memmem::find(w, needle).is_some())
}

/// Fast 64-bit word-aligned zero check. Returns `true` if `slice` is non-empty and all `0x00`.
#[inline]
#[allow(dead_code)]
pub(crate) fn is_all_zeros(slice: &[u8]) -> bool {
    if slice.is_empty() {
        return false;
    }
    let (chunks, remainder) = slice.as_chunks::<8>();
    for chunk in chunks {
        if u64::from_ne_bytes(*chunk) != 0 {
            return false;
        }
    }
    remainder.iter().all(|&b| b == 0)
}

/// Returns `None` if `slice` is a valid UTF-8/ASCII text header, or `Some(first_bad_byte_offset)`
/// if it contains `0x00`, disallowed C0 control bytes, `0x7F`, or invalid UTF-8 sequences.
#[inline]
pub(crate) fn raw_utf8_text_first_bad_offset(
    slice: &[u8], size: usize, extra_ctrl_mask: u32,
) -> Option<usize> {
    if slice.is_empty() {
        return Some(0);
    }
    const BASE_CTRL_MASK: u32 =
        (1 << 9) | (1 << 10) | (1 << 11) | (1 << 12) | (1 << 13) | (1 << 27);
    let allowed_mask = (BASE_CTRL_MASK | extra_ctrl_mask) & !1u32;

    for (i, &b) in slice.iter().enumerate() {
        if b < 0x20 {
            if (allowed_mask & (1u32 << b)) != 0 {
                continue;
            }
            if b == 0x1A
                && (i + 1 == size
                    || slice[i..]
                        .iter()
                        .all(|&tail| matches!(tail, 0x1A | b'\r' | b'\n' | b' ' | b'\t')))
            {
                continue;
            }
            return Some(i);
        }
        if b == 0x7F {
            return Some(i);
        }
    }

    match core::str::from_utf8(slice) {
        Ok(_) => None,
        Err(e) => {
            if size > slice.len() && e.error_len().is_none() {
                None
            } else {
                Some(e.valid_up_to())
            }
        }
    }
}

/// Validates that the entire header buffer `prefix` is valid ASCII or UTF-8 text.
#[inline]
pub(crate) fn is_utf8_text_header(prefix: &[u8], size: usize) -> bool {
    raw_utf8_text_first_bad_offset(prefix, size, 0).is_none()
}

/// Returns `true` if `slice` contains at least one non-printable-ASCII byte
/// (i.e. any byte outside `0x20..=0x7E`, `\t`, `\n`, `\r`).
#[inline]
pub(crate) fn has_non_ascii_text_byte(slice: &[u8]) -> bool {
    slice.iter().any(|&b| !matches!(b, 0x20..=0x7E | b'\t' | b'\n' | b'\r'))
}

/// Validates that the entire header buffer `prefix` is mostly printable text:
/// - Accepts immediately if `prefix` is valid UTF-8 text via [`raw_utf8_text_first_bad_offset`].
/// - Otherwise requires zero `0x00` (NUL) bytes in the first 512 bytes, `<= 1%` NUL bytes overall,
///   and `>= 90%` of all bytes in `prefix` to be printable 7-bit ASCII (`0x20..=0x7E`) or standard
///   whitespace (`\t`, `\n`, `\x0B`, `\x0C`, `\r`, `\x1B`).
#[inline]
pub(crate) fn is_mostly_printable_header(prefix: &[u8], size: usize) -> bool {
    if prefix.is_empty() {
        return false;
    }
    if raw_utf8_text_first_bad_offset(prefix, size, 0).is_none() {
        return true;
    }
    let nul_free_prefix = prefix.len().min(512);
    if prefix[..nul_free_prefix].contains(&0) {
        return false;
    }
    let mut printable = 0usize;
    let mut nul_count = 0usize;
    for &b in prefix {
        if matches!(b, 0x20..=0x7E | b'\t' | b'\n' | 0x0B | 0x0C | b'\r' | 0x1B) {
            printable += 1;
        } else if b == 0 {
            nul_count += 1;
        }
    }
    nul_count * 100 <= prefix.len() && printable * 10 >= prefix.len() * 9
}

/// Per-input scan context (bounded prefix, total file size, and optional tail window).
#[derive(Clone, Copy)]
pub(crate) struct Ctx<'a> {
    pub(crate) prefix: &'a [u8],
    pub(crate) size: u64,
    pub(crate) tail: Option<&'a [u8]>,
}

/// A finer-grained variant of a content type identified by a verifier.
///
/// Subformats either name branches that a verifier already takes (which signature, container
/// stream, version byte, or byte order matched) or inspect O(1) fixed-offset header fields
/// within an already-matched container without scanning or altering its `ContentType`. When
/// the subformat has a more specific PRONOM PUID or Library of Congress FDD than its content
/// type, it overrides it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Subformat {
    pub(crate) name: &'static str,
    pub(crate) pronom_puid: Option<&'static str>,
    pub(crate) loc_fdd: Option<&'static str>,
}

impl Subformat {
    /// A subformat without its own registry identifiers.
    pub(crate) const fn new(name: &'static str) -> Self {
        Subformat { name, pronom_puid: None, loc_fdd: None }
    }

    /// A subformat with its own PRONOM PUID (and optionally LoC FDD).
    pub(crate) const fn with_ids(
        name: &'static str, pronom_puid: &'static str, loc_fdd: Option<&'static str>,
    ) -> Self {
        Subformat { name, pronom_puid: Some(pronom_puid), loc_fdd }
    }

    /// A subformat with its own Library of Congress FDD and no PRONOM PUID.
    pub(crate) const fn with_fdd(name: &'static str, loc_fdd: &'static str) -> Self {
        Subformat { name, pronom_puid: None, loc_fdd: Some(loc_fdd) }
    }
}

thread_local! {
    /// Subformat recorded by the last verifier that matched on this thread.
    static SUBFORMAT: core::cell::Cell<Option<(Label, &'static Subformat)>> =
        const { core::cell::Cell::new(None) };
}

/// Returns `Some(label)` and records `subformat` as the subformat of `label`.
///
/// Call this only on the final `Some(..)` return path of a verifier.
#[inline]
pub(crate) fn with_subformat(label: Label, subformat: &'static Subformat) -> Option<Label> {
    SUBFORMAT.set(Some((label, subformat)));
    Some(label)
}

/// Forgets any recorded subformat (called before running the cascade).
#[inline]
pub(crate) fn clear_subformat() {
    SUBFORMAT.set(None);
}

/// Takes the recorded subformat if it was recorded for `label`.
///
/// The label check discards subformats recorded by a verifier whose match was later rejected.
#[inline]
pub(crate) fn take_subformat(label: Label) -> Option<&'static Subformat> {
    match SUBFORMAT.take() {
        Some((recorded, subformat)) if recorded == label => Some(subformat),
        _ => None,
    }
}

/// Evaluates a candidate verifier closure.
#[inline(always)]
pub(crate) fn probe(f: impl FnOnce() -> Option<Label>) -> Option<Label> {
    f()
}

/// Runs the 5-tier native cascade over `ctx`.
#[inline]
pub(crate) fn identify(ctx: &Ctx<'_>) -> Option<Label> {
    let prefix = ctx.prefix;
    if prefix.len() < 8 || ctx.size < 8 {
        return None;
    }
    require!(ctx.size >= 8, prefix.len() >= 8);
    if let Some(key) = prefix.first_chunk::<4>()
        && let Some(label) = probe(|| magic32::probe(key, prefix, ctx))
    {
        let _ = get_slice(prefix, 0, 4);
        return Some(label);
    }
    if let Some(key) = prefix.first_chunk::<2>()
        && let Some(label) = probe(|| magic16::probe(key, prefix, ctx))
    {
        let _ = get_slice(prefix, 0, 2);
        return Some(label);
    }
    if let Some(&b0) = prefix.first()
        && let Some(label) = probe(|| magic8::probe(b0, prefix, ctx))
    {
        let _ = get_slice(prefix, 0, 1);
        return Some(label);
    }
    if let Some(label) = offset_probes::probe(prefix, ctx) {
        return Some(label);
    }
    heuristics::probe(prefix, ctx)
}
