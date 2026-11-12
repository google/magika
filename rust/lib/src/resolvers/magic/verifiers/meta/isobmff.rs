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

//! ISO Base Media File Format (`ISOBMFF`), QuickTime, and JPEG 2000/XL box metaformat verifiers (`3gp`, `avif`, `heif`, `jp2`, `jxl`, `mp4`, `qt`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

pub(crate) const FTYP_BOX: &[u8; 4] = b"ftyp"; // @4..8: ISO-BMFF file-type box tag

/// ISO Base Media File Format `ftyp` box at `@4` (3GPP, AVIF, or HEIF/HEVC): 4-byte-aligned box
/// length `16..=4096` at `@0`, `"ftyp"` at `@4`, and a 4-byte major brand at `@8`.
pub(crate) fn ftyp(prefix: &[u8]) -> Option<Label> {
    static AVIF_STILL: Subformat = Subformat::new("AVIF still image");
    static AVIF_SEQUENCE: Subformat = Subformat::new("AVIF image sequence");
    const BOX_LEN: usize = 0; // @0: u32 BE — ftyp box byte length (16..=4096, multiple of 4)
    const MAJOR_BRAND: usize = 8; // @8..12: 4-byte major brand code
    const MIN_LEN: usize = 16; // 8-byte box header + 4-byte major brand + 4-byte minor version
    const BRANDS_3GP: &[&[u8; 4]] = &[
        b"3ge6", b"3ge7", b"3ge9", b"3gg6", b"3gg9", b"3gh9", b"3gm9", b"3gmA", b"3gp1", b"3gp2",
        b"3gp3", b"3gp4", b"3gp5", b"3gp6", b"3gp7", b"3gp8", b"3gp9", b"3gr6", b"3gr9", b"3gs6",
        b"3gs7", b"3gs9", b"3gt8", b"3gt9", b"3gtv",
    ];
    const BRANDS_HEIF: &[&[u8; 4]] =
        &[b"heic", b"heix", b"heim", b"heis", b"hevc", b"hevx", b"hevm", b"hevs"];

    require!(prefix.len() >= MIN_LEN, eq_at(prefix, 4, FTYP_BOX));
    let box_len = be32(prefix, BOX_LEN)?;
    require!((MIN_LEN as u32..=0x00FF_FFFF).contains(&box_len), box_len % 4 == 0);
    let brand: &[u8; 4] = get_slice(prefix, MAJOR_BRAND, MAJOR_BRAND + 4)?.try_into().ok()?;
    if BRANDS_3GP.contains(&brand) {
        return Some(Label::_3gp);
    }
    if brand == b"avif" {
        return with_subformat(Label::Avif, &AVIF_STILL);
    }
    if brand == b"avis" {
        return with_subformat(Label::Avif, &AVIF_SEQUENCE);
    }
    if BRANDS_HEIF.contains(&brand) {
        return Some(Label::Heif);
    }
    None
}

/// ISO-BMFF / QuickTime `ftyp` or top-level atom probe for `mp4` and `qt` (after `3gp`,
/// `avif`, and `heif`).
pub(crate) fn isobmff_mp4_or_qt(prefix: &[u8], size: u64) -> Option<Label> {
    static QT_FTYP: Subformat = Subformat::new("QuickTime movie (ftyp)");
    static QT_LEGACY: Subformat = Subformat::new("QuickTime movie (legacy atom)");
    const MOOV_BOX: &[u8; 4] = b"moov";
    const MVHD_BOX: &[u8; 4] = b"mvhd";
    const CMOV_BOX: &[u8; 4] = b"cmov";
    const WIDE_BOX: &[u8; 4] = b"wide";
    const MDAT_BOX: &[u8; 4] = b"mdat";
    const QT_BRANDS: &[&[u8; 4]] = &[b"qt  ", b"mqt "];
    const MP4_BRANDS: &[&[u8; 4]] = &[
        b"isom", b"iso2", b"iso3", b"iso4", b"iso5", b"iso6", b"iso7", b"iso8", b"iso9", b"mp41",
        b"mp42", b"mp71", b"avc1", b"M4V ", b"M4A ", b"M4P ", b"M4B ", b"M4VH", b"M4VP", b"f4v ",
        b"f4p ", b"dash", b"msdh", b"msix", b"ndas", b"ndsc", b"ndsh", b"ndsm", b"ndsp", b"ndss",
        b"ndxc", b"ndxh", b"ndxm", b"ndxp", b"ndxs",
    ];

    require!(size >= 16, prefix.len() >= 16);
    let box_len = be32(prefix, 0)?;
    if eq_at(prefix, 4, FTYP_BOX) && (16..=0x00FF_FFFF).contains(&box_len) && box_len % 4 == 0 {
        let brand: &[u8; 4] = get_slice(prefix, 8, 12)?.try_into().ok()?;
        if QT_BRANDS.contains(&brand) {
            return with_subformat(Label::Qt, &QT_FTYP);
        }
        if MP4_BRANDS.contains(&brand) {
            return Some(Label::Mp4);
        }
        return None;
    }
    // Legacy QuickTime movie starting directly with moov, wide, free, or mdat atom
    if (eq_at(prefix, 4, MOOV_BOX)
        && (16..0x0800_0000).contains(&box_len)
        && (eq_at(prefix, 12, MVHD_BOX) || eq_at(prefix, 12, CMOV_BOX)))
        || (eq_at(prefix, 4, WIDE_BOX) && box_len == 8 && eq_at(prefix, 12, MDAT_BOX))
    {
        return with_subformat(Label::Qt, &QT_LEGACY);
    }
    None
}

/// Fast-path Tier 1 (`magic32`) ISO-BMFF & QuickTime box dispatcher (`3gp`, `avif`, `heif`, `mp4`, `qt`).
#[inline]
pub(crate) fn isobmff(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| ftyp(prefix)).or_else(|| probe(|| isobmff_mp4_or_qt(prefix, size)))
}

/// ISO-BMFF 12-byte signature box overlap verifier: dispatches between JPEG 2000 (`"jP  \r\n\x87\n"`
/// + `"ftyp"` with `jp2 ` / `jpx ` / `jpm ` / `mjp2` / `jph `) and JPEG XL container (`"JXL \r\n\x87\n"` + `"ftypjxl "`).
pub(crate) fn jp2_or_jxl(prefix: &[u8]) -> Option<Label> {
    static JP2_PART1: Subformat =
        Subformat::with_ids("JP2 (JPEG 2000 Part 1)", "x-fmt/392", Some("fdd000143"));
    static JPX_PART2: Subformat =
        Subformat::with_ids("JPX (JPEG 2000 Part 2)", "fmt/151", Some("fdd000154"));
    static JPM_PART6: Subformat =
        Subformat::with_ids("JPM (JPEG 2000 Part 6)", "fmt/463", Some("fdd000144"));
    static MJP2_MOTION: Subformat =
        Subformat::with_ids("Motion JPEG 2000 (MJ2)", "fmt/337", Some("fdd000127"));
    static JPH_PART15: Subformat = Subformat::with_ids("JPH (JPEG 2000 Part 15)", "fmt/1964", None);
    static JXL_CONTAINER: Subformat =
        Subformat::with_ids("JPEG XL container", "fmt/1485", Some("fdd000538"));
    const JP2_SIG: &[u8] = bytes!(&12u32.to_be_bytes(), b"jP  \r\n\x87\n"); // @0..12: JPEG 2000 signature box
    const JXL_SIG: &[u8] = bytes!(&12u32.to_be_bytes(), b"JXL \r\n\x87\n"); // @0..12: JPEG XL signature box
    const FTYP_LEN: usize = 12; // @12: u32 BE — ftyp box length
    const FTYP_BRAND: usize = 16; // @16..24: "ftyp" + 4-byte major brand
    const JP2_MIN_LEN: usize = 28;
    const JXL_MIN_LEN: usize = 32;

    if prefix.len() >= JP2_MIN_LEN && starts(prefix, JP2_SIG) && eq_at(prefix, FTYP_BRAND, FTYP_BOX)
    {
        let sub = match get_slice(prefix, 20, 24)? {
            b"jp2 " => &JP2_PART1,
            b"jpx " | b"jpxb" => &JPX_PART2,
            b"jpm " => &JPM_PART6,
            b"mjp2" => &MJP2_MOTION,
            b"jph " => &JPH_PART15,
            _ => return None,
        };
        let ftyp_len = be32(prefix, FTYP_LEN)?;
        require!(ftyp_len >= 16, ftyp_len % 4 == 0);
        return with_subformat(Label::Jp2, sub);
    }
    if prefix.len() >= JXL_MIN_LEN
        && starts(prefix, JXL_SIG)
        && eq_at(prefix, FTYP_BRAND, b"ftypjxl ")
        && be32(prefix, FTYP_LEN)? >= 20
    {
        return with_subformat(Label::Jxl, &JXL_CONTAINER);
    }
    None
}
