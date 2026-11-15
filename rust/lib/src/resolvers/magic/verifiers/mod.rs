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

//! Format verifiers organized into `meta/` (container/framing metaformats) and
//! `category/` (standalone format categories), plus `overlap` for cross-category
//! magic-key collisions.
//!
//! **Metaformat precedence over categories**: whenever a format is a subformat of a
//! container or framing metaformat (`ar`, `asf`, `asn1`, `ebml`, `elf`, `iff`, `iso`,
//! `isobmff`, `json`, `ole2`, `pdf`, `riff`, `sqlite`, `xml`, `zip`), its verifier MUST
//! be placed in `meta/<metaformat>.rs` rather than `category/<category>.rs`.

pub(crate) mod category;
pub(crate) mod meta;
pub(crate) mod overlap;

pub(crate) use self::category::*;
pub(crate) use self::meta::*;
pub(crate) use self::overlap::*;

/// Validates an RFC 1950 2-byte zlib header (`CMF` + `FLG`).
#[inline]
pub(crate) const fn is_valid_zlib_header(cmf: u8, flg: u8) -> bool {
    (cmf & 0x0F) == 8 && (cmf >> 4) <= 7 && (cmf as u16 * 256 + flg as u16).is_multiple_of(31)
}

/// Parses an ASN.1 BER definite length field, returning `(header_bytes, content_len)`.
pub(crate) fn parse_asn1_ber_len(buf: &[u8]) -> Option<(usize, usize)> {
    let b0 = super::u8_at(buf, 0)?;
    if b0 < 0x80 {
        return Some((1, b0 as usize));
    }
    let n = (b0 & 0x7F) as usize;
    super::require!((1..=4).contains(&n), buf.len() > n);
    let bytes = super::get_slice(buf, 1, 1 + n)?;
    let mut val = 0usize;
    for &b in bytes {
        val = (val << 8) | (b as usize);
    }
    Some((1 + n, val))
}
