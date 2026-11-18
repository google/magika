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
/// length `16..=0x00FF_FFFF` at `@0`, `"ftyp"` at `@4`, and a 4-byte major brand at `@8`.
pub(crate) fn ftyp(prefix: &[u8]) -> Option<Label> {
    static AVIF_STILL: Subformat = Subformat::new("AVIF still image");
    static AVIF_SEQUENCE: Subformat = Subformat::new("AVIF image sequence");
    static HEIF_HEIC: Subformat = Subformat::with_fdd("HEIC image (heic)", "fdd000526");
    static HEIF_HEIX: Subformat = Subformat::with_fdd("HEIC extended image (heix)", "fdd000526");
    static HEIF_HEIM: Subformat = Subformat::new("HEIF image collection (heim)");
    static HEIF_HEIS: Subformat = Subformat::new("HEIF scalable image (heis)");
    static HEIF_HEVC: Subformat = Subformat::new("HEVC image sequence (hevc)");
    static HEIF_HEVX: Subformat = Subformat::new("HEVC extended image sequence (hevx)");
    static HEIF_HEVM: Subformat = Subformat::new("HEVC image sequence collection (hevm)");
    static HEIF_HEVS: Subformat = Subformat::new("HEVC scalable image sequence (hevs)");
    static THREEGP_R4: Subformat = Subformat::new("3GPP Release 4");
    static THREEGP_R5: Subformat = Subformat::new("3GPP Release 5");
    static THREEGP_R6: Subformat = Subformat::new("3GPP Release 6");
    static THREEGP_R7: Subformat = Subformat::new("3GPP Release 7");
    static THREEGP_R8: Subformat = Subformat::new("3GPP Release 8");
    static THREEGP_R9: Subformat = Subformat::new("3GPP Release 9");
    static THREEGP_PROGRESSIVE: Subformat = Subformat::new("3GPP progressive download");
    static THREEGP_GENERAL: Subformat = Subformat::new("3GPP general profile");
    static THREEGP_STREAMING: Subformat = Subformat::new("3GPP streaming server");
    static THREEGP_EXTENDED: Subformat = Subformat::new("3GPP extended presentation");
    static THREEGP_AHS: Subformat = Subformat::new("3GPP adaptive HTTP streaming");
    static THREEGP_SEGMENT: Subformat = Subformat::new("3GPP media segment");
    static THREEGP_MOBILE_TV: Subformat = Subformat::new("3GPP mobile TV");
    const BOX_LEN: usize = 0; // @0: u32 BE — ftyp box byte length (16..=0x00FF_FFFF, multiple of 4)
    const MAJOR_BRAND: usize = 8; // @8..12: 4-byte major brand code
    const MIN_LEN: usize = 16; // 8-byte box header + 4-byte major brand + 4-byte minor version

    require!(prefix.len() >= MIN_LEN, eq_at(prefix, 4, FTYP_BOX));
    let box_len = be32(prefix, BOX_LEN)?;
    require!((MIN_LEN as u32..=0x00FF_FFFF).contains(&box_len), box_len % 4 == 0);
    let brand: &[u8; 4] = get_slice(prefix, MAJOR_BRAND, MAJOR_BRAND + 4)?.try_into().ok()?;
    match brand {
        b"3gp1" | b"3gp2" | b"3gp3" | b"3gp4" => with_subformat(Label::_3gp, &THREEGP_R4),
        b"3gp5" => with_subformat(Label::_3gp, &THREEGP_R5),
        b"3gp6" => with_subformat(Label::_3gp, &THREEGP_R6),
        b"3gp7" => with_subformat(Label::_3gp, &THREEGP_R7),
        b"3gp8" => with_subformat(Label::_3gp, &THREEGP_R8),
        b"3gp9" => with_subformat(Label::_3gp, &THREEGP_R9),
        b"3gr6" | b"3gr9" => with_subformat(Label::_3gp, &THREEGP_PROGRESSIVE),
        b"3gg6" | b"3gg9" => with_subformat(Label::_3gp, &THREEGP_GENERAL),
        b"3gs6" | b"3gs7" | b"3gs9" => with_subformat(Label::_3gp, &THREEGP_STREAMING),
        b"3ge6" | b"3ge7" | b"3ge9" => with_subformat(Label::_3gp, &THREEGP_EXTENDED),
        b"3gh9" => with_subformat(Label::_3gp, &THREEGP_AHS),
        b"3gm9" | b"3gmA" => with_subformat(Label::_3gp, &THREEGP_SEGMENT),
        b"3gt8" | b"3gt9" | b"3gtv" => with_subformat(Label::_3gp, &THREEGP_MOBILE_TV),
        b"avif" => with_subformat(Label::Avif, &AVIF_STILL),
        b"avis" => with_subformat(Label::Avif, &AVIF_SEQUENCE),
        b"heic" => with_subformat(Label::Heif, &HEIF_HEIC),
        b"heix" => with_subformat(Label::Heif, &HEIF_HEIX),
        b"heim" => with_subformat(Label::Heif, &HEIF_HEIM),
        b"heis" => with_subformat(Label::Heif, &HEIF_HEIS),
        b"hevc" => with_subformat(Label::Heif, &HEIF_HEVC),
        b"hevx" => with_subformat(Label::Heif, &HEIF_HEVX),
        b"hevm" => with_subformat(Label::Heif, &HEIF_HEVM),
        b"hevs" => with_subformat(Label::Heif, &HEIF_HEVS),
        _ => None,
    }
}

/// ISO-BMFF / QuickTime `ftyp` or top-level atom probe for `mp4` and `qt` (after `3gp`,
/// `avif`, and `heif`).
pub(crate) fn isobmff_mp4_or_qt(prefix: &[u8], size: u64) -> Option<Label> {
    static QT_FTYP: Subformat = Subformat::new("QuickTime movie (ftyp)");
    static QT_LEGACY: Subformat = Subformat::new("QuickTime movie (legacy atom)");
    static MP4_ISOM: Subformat = Subformat::new("ISO Base Media v1 (isom)");
    static MP4_ISO2: Subformat = Subformat::new("ISO Base Media v2 (iso2)");
    static MP4_ISO3_PLUS: Subformat = Subformat::new("ISO Base Media (iso3+)");
    static MP4_V1: Subformat = Subformat::with_fdd("MP4 v1 (mp41)", "fdd000037");
    static MP4_V2: Subformat = Subformat::new("MP4 v2 (mp42)");
    static MP4_MP71: Subformat = Subformat::new("MPEG-7 (mp71)");
    static MP4_AVC1: Subformat = Subformat::with_fdd("MP4 AVC (avc1)", "fdd000137");
    static MP4_M4A: Subformat = Subformat::with_ids("Apple M4A audio (.m4a)", "fmt/2094", None);
    static MP4_M4V: Subformat = Subformat::new("Apple M4V video (.m4v)");
    static MP4_M4B: Subformat = Subformat::new("Apple M4B audiobook (.m4b)");
    static MP4_M4P: Subformat = Subformat::new("Apple M4P protected audio (.m4p)");
    static MP4_F4V: Subformat = Subformat::new("Adobe Flash Video (.f4v)");
    static MP4_F4P: Subformat = Subformat::new("Adobe Flash Protected Video (.f4p)");
    static MP4_DASH: Subformat = Subformat::new("MPEG-DASH media segment");
    static MP4_NERO: Subformat = Subformat::new("Nero Digital MP4");
    const MOOV_BOX: &[u8; 4] = b"moov";
    const MVHD_BOX: &[u8; 4] = b"mvhd";
    const CMOV_BOX: &[u8; 4] = b"cmov";
    const WIDE_BOX: &[u8; 4] = b"wide";
    const MDAT_BOX: &[u8; 4] = b"mdat";

    require!(size >= 16, prefix.len() >= 16);
    let box_len = be32(prefix, 0)?;
    if eq_at(prefix, 4, FTYP_BOX) && (16..=0x00FF_FFFF).contains(&box_len) && box_len % 4 == 0 {
        let brand: &[u8; 4] = get_slice(prefix, 8, 12)?.try_into().ok()?;
        return match brand {
            b"qt  " | b"mqt " => with_subformat(Label::Qt, &QT_FTYP),
            b"isom" => with_subformat(Label::Mp4, &MP4_ISOM),
            b"iso2" => with_subformat(Label::Mp4, &MP4_ISO2),
            b"iso3" | b"iso4" | b"iso5" | b"iso6" | b"iso7" | b"iso8" | b"iso9" => {
                with_subformat(Label::Mp4, &MP4_ISO3_PLUS)
            }
            b"mp41" => with_subformat(Label::Mp4, &MP4_V1),
            b"mp42" => with_subformat(Label::Mp4, &MP4_V2),
            b"mp71" => with_subformat(Label::Mp4, &MP4_MP71),
            b"avc1" => with_subformat(Label::Mp4, &MP4_AVC1),
            b"M4A " => with_subformat(Label::Mp4, &MP4_M4A),
            b"M4V " | b"M4VH" | b"M4VP" => with_subformat(Label::Mp4, &MP4_M4V),
            b"M4B " => with_subformat(Label::Mp4, &MP4_M4B),
            b"M4P " => with_subformat(Label::Mp4, &MP4_M4P),
            b"f4v " => with_subformat(Label::Mp4, &MP4_F4V),
            b"f4p " => with_subformat(Label::Mp4, &MP4_F4P),
            b"dash" | b"msdh" | b"msix" => with_subformat(Label::Mp4, &MP4_DASH),
            b"ndas" | b"ndsc" | b"ndsh" | b"ndsm" | b"ndsp" | b"ndss" | b"ndxc" | b"ndxh"
            | b"ndxm" | b"ndxp" | b"ndxs" => with_subformat(Label::Mp4, &MP4_NERO),
            _ => None,
        };
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
