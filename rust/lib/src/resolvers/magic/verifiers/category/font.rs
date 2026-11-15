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

//! Font and typeface container format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

pub(crate) const TTF_SFNT_V1_MAGIC: &[u8; 4] = &[0x00, 0x01, 0x00, 0x00];

pub(crate) const TTF_TRUE_MAGIC: &[u8; 4] = b"true";

pub(crate) const TTF_TYP1_MAGIC: &[u8; 4] = b"typ1";

pub(crate) const OTF_MAGIC: &[u8; 4] = b"OTTO";

pub(crate) const TTCF_MAGIC: &[u8; 4] = b"ttcf";

const SFNT_1_0: u32 = tag_u32(TTF_SFNT_V1_MAGIC);

const SFNT_OTTO: u32 = tag_u32(OTF_MAGIC);

const SFNT_TRUE: u32 = tag_u32(TTF_TRUE_MAGIC);

const SFNT_TYP1: u32 = tag_u32(TTF_TYP1_MAGIC);

const SFNT_TTCF: u32 = tag_u32(TTCF_MAGIC);

/// W3C Web Open Font Format 1.0 (WOFF): `"wOFF"` magic, known sfnt flavor at `@4`, length ≥ 64 at
/// `@8`, `1..=4095` tables at `@12`, reserved `0` at `@14`, and totalSfntSize ≥ 28 at `@16`.
pub(crate) fn woff(prefix: &[u8]) -> Option<Label> {
    static WOFF_TRUETYPE: Subformat = Subformat::new("WOFF 1.0 (TrueType)");
    static WOFF_CFF: Subformat = Subformat::new("WOFF 1.0 (OpenType CFF)");
    static WOFF_APPLE_TT: Subformat = Subformat::new("WOFF 1.0 (Apple TrueType)");
    static WOFF_TYP1: Subformat = Subformat::new("WOFF 1.0 (PostScript Type 1)");
    const MAGIC: &[u8] = b"wOFF"; // @0..4: WOFF 1.0 signature
    const FLAVOR: usize = 4; // @4: u32 BE — sfnt flavor
    const LENGTH: usize = 8; // @8: u32 BE — total WOFF file size (≥ 64)
    const NUM_TABLES: usize = 12; // @12: u16 BE — number of tables (1..=4095)
    const RESERVED: usize = 14; // @14: u16 BE — reserved (0)
    const TOTAL_SFNT_SIZE: usize = 16; // @16: u32 BE — uncompressed sfnt size (≥ 28)
    const MIN_LEN: usize = 64; // 44-byte WOFF header + 1 × 20-byte table directory entry

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        be32(prefix, LENGTH)? >= MIN_LEN as u32,
        (1..=4095).contains(&be16(prefix, NUM_TABLES)?),
        be16(prefix, RESERVED)? == 0,
        be32(prefix, TOTAL_SFNT_SIZE)? >= 28,
    );
    let sub = match be32(prefix, FLAVOR)? {
        SFNT_1_0 => &WOFF_TRUETYPE,
        SFNT_OTTO => &WOFF_CFF,
        SFNT_TRUE => &WOFF_APPLE_TT,
        SFNT_TYP1 => &WOFF_TYP1,
        _ => return None,
    };
    with_subformat(Label::Woff, sub)
}

/// W3C Web Open Font Format 2.0 (WOFF2): `"wOF2"` magic, known sfnt/collection flavor at `@4`,
/// length ≥ 51 at `@8`, `1..=4095` tables at `@12`, reserved `0` at `@14`, totalSfntSize ≥ 28 at
/// `@16`, and nonzero totalCompressedSize at `@20`.
pub(crate) fn woff2(prefix: &[u8]) -> Option<Label> {
    static WOFF2_TRUETYPE: Subformat = Subformat::new("WOFF 2.0 (TrueType)");
    static WOFF2_CFF: Subformat = Subformat::new("WOFF 2.0 (OpenType CFF)");
    static WOFF2_APPLE_TT: Subformat = Subformat::new("WOFF 2.0 (Apple TrueType)");
    static WOFF2_TYP1: Subformat = Subformat::new("WOFF 2.0 (PostScript Type 1)");
    static WOFF2_TTC: Subformat = Subformat::new("WOFF 2.0 (TrueType Collection)");
    const MAGIC: &[u8] = b"wOF2"; // @0..4: WOFF 2.0 signature
    const FLAVOR: usize = 4; // @4: u32 BE — sfnt flavor
    const LENGTH: usize = 8; // @8: u32 BE — total WOFF2 file size (≥ 51)
    const NUM_TABLES: usize = 12; // @12: u16 BE — number of tables (1..=4095)
    const RESERVED: usize = 14; // @14: u16 BE — reserved (0)
    const TOTAL_SFNT_SIZE: usize = 16; // @16: u32 BE — uncompressed size (≥ 28)
    const TOTAL_COMP_SIZE: usize = 20; // @20: u32 BE — compressed stream size (≥ 1)
    const MIN_LEN: usize = 51; // 48-byte WOFF2 header + ≥ 2-byte table entry + ≥ 1 payload byte

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        be32(prefix, LENGTH)? >= MIN_LEN as u32,
        (1..=4095).contains(&be16(prefix, NUM_TABLES)?),
        be16(prefix, RESERVED)? == 0,
        be32(prefix, TOTAL_SFNT_SIZE)? >= 28,
        be32(prefix, TOTAL_COMP_SIZE)? >= 1,
    );
    let sub = match be32(prefix, FLAVOR)? {
        SFNT_1_0 => &WOFF2_TRUETYPE,
        SFNT_OTTO => &WOFF2_CFF,
        SFNT_TRUE => &WOFF2_APPLE_TT,
        SFNT_TYP1 => &WOFF2_TYP1,
        SFNT_TTCF => &WOFF2_TTC,
        _ => return None,
    };
    with_subformat(Label::Woff2, sub)
}

/// Apple / Microsoft TrueType Font (`ttf`, 1991) & TrueType Collection (`ttcf`):
/// sfnt version `00 01 00 00`, `"true"`, or `"typ1"` with valid `numTables`, power-of-two
/// `searchRange`, and printable 4-byte table tags; or `"ttcf"` header with version `1|2`
/// and font count `1..=1024`.
pub(crate) fn ttf(prefix: &[u8], size: u64) -> Option<Label> {
    static TTC_V1: Subformat = Subformat::new("TrueType Collection (v1.0)");
    static TTC_V2: Subformat = Subformat::new("TrueType Collection (v2.0)");
    static TTF_SFNT: Subformat = Subformat::with_ids("TrueType Font (sfnt)", "x-fmt/453", None);
    static TTF_APPLE: Subformat = Subformat::with_ids("Apple TrueType Font", "x-fmt/453", None);
    static TTF_TYPE1: Subformat = Subformat::new("PostScript Type 1 sfnt Font");
    const MIN_TTF_SIZE: u64 = 28; // 12B offset table + >=1 16B table directory record

    require!(size >= MIN_TTF_SIZE, prefix.len() >= 28);
    if starts(prefix, TTCF_MAGIC) {
        let ver_maj = be16(prefix, 4)?;
        let ver_min = be16(prefix, 6)?;
        let num_fonts = be32(prefix, 8)?;
        let first_off = be32(prefix, 12)?;
        let sub = match (ver_maj, ver_min) {
            (1, 0) => &TTC_V1,
            (2, 0) => &TTC_V2,
            _ => return None,
        };
        require!((1..=1024).contains(&num_fonts), first_off >= 12 + num_fonts * 4);
        return with_subformat(Label::Ttf, sub);
    }
    let sub = if starts(prefix, TTF_SFNT_V1_MAGIC) {
        &TTF_SFNT
    } else if starts(prefix, TTF_TRUE_MAGIC) {
        &TTF_APPLE
    } else if starts(prefix, TTF_TYP1_MAGIC) {
        &TTF_TYPE1
    } else {
        return None;
    };
    let num_tables = be16(prefix, 4)?;
    let search_range = be16(prefix, 6)?;
    require!(
        (1..=256).contains(&num_tables),
        search_range >= 16,
        (search_range / 16).is_power_of_two(),
    );
    let tag = get_slice(prefix, 12, 16)?;
    require!(tag.iter().all(|&b| (0x20..=0x7E).contains(&b)));
    with_subformat(Label::Ttf, sub)
}

/// Adobe / Microsoft OpenType Font (`otf`, 1996) with PostScript CFF outlines:
/// sfnt version `"OTTO"` at `@0`, `numTables` in `1..=256` at `@4`, power-of-two
/// `searchRange` at `@6`, and printable 4-byte table tag at `@12..16`.
pub(crate) fn otf(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_OTF_SIZE: u64 = 28; // 12B sfnt header + >=1 16B table record

    require!(size >= MIN_OTF_SIZE, prefix.len() >= 28, starts(prefix, OTF_MAGIC),);
    let num_tables = be16(prefix, 4)?;
    let search_range = be16(prefix, 6)?;
    require!(
        (1..=256).contains(&num_tables),
        search_range >= 16,
        (search_range / 16).is_power_of_two(),
    );
    let tag = get_slice(prefix, 12, 16)?;
    require!(tag.iter().all(|&b| (0x20..=0x7E).contains(&b)));
    Some(Label::Otf)
}

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const BANNERMANIA_NAME_LEN_OFFSET: usize = 4;
const BERKELEY_VFONT_MAGIC_BE: &[u8; 2] = be16_bytes!(0x011E);
const BERKELEY_VFONT_MAGIC_LE: &[u8; 2] = le16_bytes!(0x011E);
const BMF_FONT_MAGIC: &[u8; 4] = hex2bin!("E1 E6 D5 1A");
const BORLAND_BGI_CHR_MAGIC: &[u8; 4] = bytes!(b"PK", &[0x08, 0x08]);
const EA_FNTI_FILE_SIZE_OFFSET: usize = 12;
const EA_FONT_FILE_SIZE_OFFSET: usize = 4;
const FC_CACHE_SIZE_OFFSET: usize = 8;
const FC_CACHE_VERSION_OFFSET: usize = 4;
const HERCULES_WOF_MAGIC: &[u8; 4] = bytes!(b"ES", le16_bytes!(1));
const HP_PCL_ESC_PREFIX: &[u8; 2] = &[0x1B, b')'];
const HP_PCL_FONT_HDR_PREFIX: &[u8; 3] = bytes!(HP_PCL_ESC_PREFIX, b"s");
const MIN_AGFA_INTELLIFONT_SIZE: u64 = 128;
const MIN_ALTO_AL_SIZE: u64 = 516;
const MIN_ALTO_WIDTHS_SIZE: u64 = 64;
const MIN_BANNERMANIA_FONT_SIZE: u64 = 64;
const MIN_BERKELEY_VFONT_SIZE: u64 = 2571;
const MIN_BETHESDA_XCAR_FONT_SIZE: u64 = 72;
const MIN_BITSTREAM_PFR_SIZE: u64 = 58;
const MIN_BITSTREAM_SPEEDO_SIZE: u64 = 150;
const MIN_BMF_FONT_SIZE: u64 = 29;
const MIN_BORLAND_BGI_CHR_SIZE: u64 = 144;
const MIN_CHX_BIG_FONT_SIZE: u64 = 520;
const MIN_DAISYDOT_NLQ3_FONT_SIZE: u64 = 512;
const MIN_DAISYDOT_NLQ_FONT_SIZE: u64 = 64;
const MIN_DXTK_SPRITEFONT_SIZE: u64 = 44;
const MIN_EA_FONT_SIZE: u64 = 32;
const MIN_ENVISION_PUBLISHER_FONT_SIZE: u64 = 128;
const MIN_FONTCONFIG_CACHE_SIZE: u64 = 40;
const MIN_FONTLAB_VFB_SIZE: u64 = 64;
const MIN_FONTOGRAPHER_FOG_SIZE: u64 = 512;
const MIN_HERCULES_WOF_SIZE: u64 = 36;
const MIN_HIGHLOGIC_FCP_SIZE: u64 = 44;
const MIN_HP_PCL_SOFTFONT_SIZE: u64 = 70;
const MIN_LINUX_PSF1_SIZE: u64 = 1540;
const MIN_LINUX_PSF2_SIZE: u64 = 64;
const MIN_MCFT_CFT_SIZE: u64 = 28;
const MIN_NINTENDO_FONT_SIZE: u64 = 64;
const MIN_OPTIKS_PCR_FONT_SIZE: u64 = 64;
const MIN_OS2_PM_FONT_SIZE: u64 = 64;
const MIN_PICASA_YTF_SIZE: u64 = 64;
const MIN_PLIB_TXF_SIZE: u64 = 64;
const MIN_RAVEN_XBNF_BNX_SIZE: u64 = 48;
const MIN_RISCOS_INTMETRICS_SIZE: u64 = 100;
const MIN_RISCOS_OUTLINES_SIZE: u64 = 64;
const MIN_RTSTRIKE_FONT_SIZE: u64 = 24;
const MIN_SIGNUM_FONT_SIZE: u64 = 128;
const MIN_SINCLAIR_QL_F3_SIZE: u64 = 64;
const MIN_TEX_TFM_SIZE: u64 = 64;
const MIN_THEDRAW_FONT_SIZE: u64 = 252;
const MIN_UBF_FONT_SIZE: u64 = 64;
const MIN_WGT_FONT_SIZE: u64 = 32;
const MIN_X11_PCF_SIZE: u64 = 84;
const PFR_CRLF_OFFSET: usize = 6;
const PFR_HEADER_SIZE_OFFSET: usize = 8;
const PLIB_TXF_ENDIAN_OFFSET: usize = 4;
const PLIB_TXF_FORMAT_OFFSET: usize = 8;
const PLIB_TXF_HEIGHT_OFFSET: usize = 16;
const PLIB_TXF_WIDTH_OFFSET: usize = 12;
const RISCOS_INTMETRICS_XSCALE_OFFSET: usize = 40;
const RISCOS_INTMETRICS_YSCALE_OFFSET: usize = 44;
const SPEEDO_FONT_SIZE_OFFSET: usize = 8;
const SPEEDO_SUFFIX_OFFSET: usize = 4;
const TFM_DESIGN_SIZE_OFFSET: usize = 28;
const X11_PCF_TABLE_COUNT_OFFSET: usize = 4;
const X11_PCF_TOC_OFFSET: usize = 8;

/// Agfa Compugraphic Intellifont Scalable Font Library (`.lib` / `.fnt` / `.type`, 1990 Agfa).
pub(crate) fn verify_agfa_intellifont(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AGFA_INTELLIFONT_SIZE
            && prefix.len() >= 20
            && starts(prefix, hex2bin!("00 44 00 01 00"))
    );
    let is_ascii_id = eq_at(prefix, 5, hex2bin!("06 60")) && eq_at(prefix, 7, b"000000");
    let hdr_len = be16(prefix, 6).unwrap_or(0) as u32;
    let is_lib_hdr = u8_at(prefix, 5)? == 0
        && (16..=64).contains(&hdr_len)
        && be16(prefix, 10) == Some(2)
        && be32(prefix, 12) == Some(0x0200 + hdr_len)
        && eq_at(prefix, 16, hex2bin!("00 14 FF FF"));
    require!(is_ascii_id || is_lib_hdr);
    Some(Label::Intellifont)
}

/// Xerox Alto bitmap font (`.al`, 1974) Xerox PARC.
///
/// Validates big-endian `height` (`4..=64`), `baseline` (`1..=height`), `max_width` (`1..=64`),
/// even 16-bit word file length, all 256 self-relative glyph pointers (`2 + ch + rel[ch]`)
/// pointing past the pointer table and inside the file, non-zero `XW` width descriptors,
/// and at least 16 distinct glyph targets.
pub(crate) fn verify_alto_al(prefix: &[u8], size: u64) -> Option<Label> {
    const HEIGHT_OFF: usize = 0; // @0..2: u16 BE font height in scanlines
    const PROP_BASE_OFF: usize = 2; // @2: (proportional << 7) | baseline
    const MAX_WIDTH_OFF: usize = 3; // @3: u8 max character advance width
    const PTR_TABLE_OFF: usize = 4; // @4..516: 256 × u16 BE self-relative word offsets
    const PTR_COUNT: usize = 256;
    const PTR_END_WORD: usize = 2 + PTR_COUNT; // word 258 (@516)

    require!(
        size >= MIN_ALTO_AL_SIZE,
        size.is_multiple_of(2),
        size <= 131_072,
        prefix.len() >= PTR_TABLE_OFF + PTR_COUNT * 2,
        has_non_ascii_text_byte(prefix.get(..64)?)
    );

    let height = be16(prefix, HEIGHT_OFF)?;
    let baseline = (u8_at(prefix, PROP_BASE_OFF)? & 0x7F) as u16;
    let max_width = u8_at(prefix, MAX_WIDTH_OFF)?;
    require!(
        (4..=64).contains(&height),
        baseline >= 1 && baseline <= height,
        (1..=64).contains(&max_width)
    );

    let total_words = (size / 2) as usize;
    let prefix_words = prefix.len() / 2;
    let mut seen = [false; 4096];
    let mut distinct = 0usize;

    for ch in 0..PTR_COUNT {
        let rel = be16(prefix, PTR_TABLE_OFF + ch * 2)? as usize;
        let tgt = 2 + ch + rel;
        require!(tgt >= PTR_END_WORD, tgt < total_words);
        if tgt < prefix_words {
            let xw = be16(prefix, tgt * 2)?;
            require!(xw != 0);
            if tgt < seen.len() && !seen[tgt] {
                seen[tgt] = true;
                distinct += 1;
            } else if tgt >= seen.len() {
                distinct += 1;
            }
        }
    }
    require!(distinct >= 16);

    Some(Label::AltoAl)
}

/// Xerox Alto Font Widths Table (`FONTS.WIDTHS`, 1975) Xerox PARC.
///
/// Walks the big-endian IX directory record chain starting at `@0`: `IXN` (type `1`, `wlen == 12`,
/// BCPL printable ASCII family name) and `IXW`/`IXS`/`IXC` (types `2..=4`, `wlen >= 9`),
/// terminating cleanly at exact `0x0001` (`IXE` end-of-index marker) with `ixn >= 1 && ixw >= 1`.
pub(crate) fn verify_alto_widths(prefix: &[u8], size: u64) -> Option<Label> {
    const IXE_END_MARKER: u16 = 0x0001; // type 0, word_len 1
    const IXN_TYPE: u16 = 1; // Name Index (12 words = 24 bytes)
    const IXN_WORDS: usize = 12;
    const IXW_TYPE: u16 = 4; // Width Index (9 words = 18 bytes)

    require!(
        size >= MIN_ALTO_WIDTHS_SIZE,
        size.is_multiple_of(2),
        prefix.len() >= MIN_ALTO_WIDTHS_SIZE as usize,
        has_non_ascii_text_byte(prefix.get(..32)?)
    );

    let mut off = 0usize;
    let mut ixn_count = 0usize;
    let mut ixw_count = 0usize;
    let mut found_end = false;
    let mut iters = 0usize;

    while off + 2 <= prefix.len() && iters < 2048 {
        iters += 1;
        let hdr = be16(prefix, off)?;
        if hdr == IXE_END_MARKER {
            found_end = true;
            break;
        }
        let ix_type = hdr >> 12;
        let wlen = (hdr & 0x0FFF) as usize;
        let blen = wlen * 2;
        require!(wlen >= 2, off + blen <= prefix.len());
        match ix_type {
            IXN_TYPE => {
                require!(wlen == IXN_WORDS);
                let nlen = u8_at(prefix, off + 4)? as usize;
                require!(
                    (1..=19).contains(&nlen),
                    get_slice(prefix, off + 5, off + 5 + nlen)?
                        .iter()
                        .all(|&b| (0x20..=0x7E).contains(&b))
                );
                ixn_count += 1;
            }
            2 | 3 | IXW_TYPE => {
                require!(wlen >= 9);
                if ix_type == IXW_TYPE {
                    require!(wlen == 9);
                    ixw_count += 1;
                }
            }
            _ => return None,
        }
        off += blen;
    }

    require!(found_end, ixn_count >= 1, ixw_count >= 1);
    Some(Label::AltoWidths)
}

/// Pixellite / Broderbund BannerMania Vector Font (`.fnt`, 1989 Pixellite Group & Broderbund).
pub(crate) fn verify_bannermania_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BANNERMANIA_FONT_SIZE
            && prefix.len() >= 36
            && starts(prefix, hex2bin!("AB CD 00 05"))
    );
    let name_len = u8_at(prefix, BANNERMANIA_NAME_LEN_OFFSET)? as usize;
    require!(
        (1..=31).contains(&name_len)
            && get_slice(prefix, 5, 5 + name_len)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::BannermaniaFnt)
}

/// Berkeley Unix / SunOS `vfont` Bitmap Font (`.vfont`, 1979 UC Berkeley CSRG / Sun Microsystems).
///
/// Used by `vtroff`, `versatec`, SunView `pixfont`, and BSD/SunOS raster console/printer tools.
/// File size satisfies `size == 2570 + dsz` (`@2..4`).
pub(crate) fn verify_berkeley_vfont(prefix: &[u8], size: u64) -> Option<Label> {
    const VFONT_HEADER_AND_DISPATCH_SIZE: usize = 2570; // 10B header + 256 * 10B dispatch
    const DSZ_OFF: usize = 2; // @2..4: u16 total bitmap payload byte size
    const MAXX_OFF: usize = 4; // @4..6: u16 maximum glyph width
    const MAXY_OFF: usize = 6; // @6..8: u16 maximum glyph height
    const DISPATCH_OFF: usize = 10; // @10..2570: 256 x 10-byte dispatch descriptors

    require!(size >= MIN_BERKELEY_VFONT_SIZE, prefix.len() >= VFONT_HEADER_AND_DISPATCH_SIZE);
    let is_le = starts(prefix, BERKELEY_VFONT_MAGIC_LE);
    let is_be = starts(prefix, BERKELEY_VFONT_MAGIC_BE);
    require!(is_le || is_be);

    let read_u16 = |off: usize| -> u16 {
        if is_le { le16(prefix, off) } else { be16(prefix, off) }.unwrap_or(0)
    };
    let dsz = read_u16(DSZ_OFF);
    let maxx = read_u16(MAXX_OFF);
    let maxy = read_u16(MAXY_OFF);
    require!(
        dsz > 0,
        size == (VFONT_HEADER_AND_DISPATCH_SIZE as u64) + (dsz as u64),
        maxx >= 1,
        (1..=512).contains(&maxy)
    );

    let is_valid_glyph = |code: usize| -> Option<usize> {
        let off = DISPATCH_OFF + code * 10;
        let addr = read_u16(off) as u32;
        let nbytes = read_u16(off + 2) as i16;
        if nbytes > 0 && addr + (nbytes as u32) <= (dsz as u32) { Some(off) } else { None }
    };
    [b'A' as usize, b'a' as usize, b'0' as usize, b'!' as usize]
        .into_iter()
        .find_map(is_valid_glyph)
        .or_else(|| (0..256).find_map(is_valid_glyph))?;

    Some(Label::BerkeleyVfont)
}

/// Bethesda Softworks XnGine `FNHD`/`BPAL`/`FNDT` Bitmap Font (`.fnt`, 1995 Bethesda Softworks).
pub(crate) fn verify_bethesda_xcar_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BETHESDA_XCAR_FONT_SIZE && prefix.len() >= 72 && starts(prefix, b"FNHD"));
    let hdr_len = be32(prefix, 4)? as usize;
    require!(
        (32..=256).contains(&hdr_len)
            && prefix.len() >= 12 + hdr_len
            && (eq_at(prefix, 8 + hdr_len, b"BPAL") || eq_at(prefix, 8 + hdr_len, b"FNDT"))
    );
    Some(Label::XcarFnt)
}

/// Bitstream Portable Font Resource (`.pfr`, 1996 Bitstream & W3C).
pub(crate) fn verify_bitstream_pfr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BITSTREAM_PFR_SIZE
            && prefix.len() >= 16
            && (starts(prefix, b"PFR0") || starts(prefix, b"PFR1"))
            && ((u8_at(prefix, 4)?.is_ascii_digit() && u8_at(prefix, 5)?.is_ascii_digit())
                || (u8_at(prefix, 4).unwrap_or(0) == 0
                    && (1..=16).contains(&u8_at(prefix, 5).unwrap_or(0))))
            && eq_at(prefix, PFR_CRLF_OFFSET, b"\r\n")
    );
    let hdr_size = be16(prefix, PFR_HEADER_SIZE_OFFSET)? as u64;
    require!(hdr_size >= 43 && hdr_size <= size);
    Some(Label::Pfr)
}

/// Bitstream Speedo Scalable Outline Font (`.spd`, 1990 Bitstream).
pub(crate) fn verify_bitstream_speedo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BITSTREAM_SPEEDO_SIZE
            && prefix.len() >= 36
            && (starts(prefix, b"D1.0") || starts(prefix, b"D1.1"))
            && eq_at(prefix, SPEEDO_SUFFIX_OFFSET, b"\r\n\0\0")
    );
    let font_size = be32(prefix, SPEEDO_FONT_SIZE_OFFSET)? as u64;
    require!(font_size >= MIN_BITSTREAM_SPEEDO_SIZE && font_size <= size.saturating_add(16));
    Some(Label::Spd)
}

/// Tomaž Kac / CRS ByteMap Font v1.1 (`.bmf`, 1998 Tomaž Kac, supported in Cosmigo Pro Motion NG).
pub(crate) fn verify_bmf_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BMF_FONT_SIZE,
        prefix.len() >= 20,
        starts(prefix, BMF_FONT_MAGIC),
        u8_at(prefix, 4) == Some(0x11),
        u8_at(prefix, 5)? > 0,
        le32(prefix, 12) == Some(0)
    );
    let pal_colors = u8_at(prefix, 16)? as usize;
    let pal_end = 17usize.checked_add(pal_colors.checked_mul(3)?)?;
    require!(
        pal_colors >= 1,
        prefix.len() >= pal_end + 3,
        get_slice(prefix, 17, pal_end)?.iter().all(|&c| c <= 63)
    );
    let info_len = u8_at(prefix, pal_end)? as usize;
    let num_chars_off = pal_end + 1 + info_len;
    require!(prefix.len() >= num_chars_off + 2);
    let num_chars = le16(prefix, num_chars_off)?;
    require!((1..=256).contains(&num_chars));
    Some(Label::BmfFont)
}

/// Borland Graphics Interface Stroked Vector Font (`.chr`, 1987 Borland).
///
/// Distinguishes `.chr` vector fonts (uppercase `"PK\x08\x08"`, `0x1A` EOF followed by
/// `u16 LE header_size == 0x0080`, and `'+'` at `@0x80`) from BGI graphics drivers (`.bgi`),
/// which use lowercase `"pk\x08\x08"` (v2) or `"FBGD\x08\x08\x08\x08"` (v3).
pub(crate) fn verify_borland_bgi_chr(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_BLOCK_SIZE: usize = 0x80; // @0x80: '+' (0x2B) stroke header marker
    const STROKE_HEADER_MARKER: u8 = b'+';

    require!(
        size >= MIN_BORLAND_BGI_CHR_SIZE,
        prefix.len() > HDR_BLOCK_SIZE,
        starts(prefix, BORLAND_BGI_CHR_MAGIC),
        u8_at(prefix, HDR_BLOCK_SIZE)? == STROKE_HEADER_MARKER
    );
    let eof_pos = 4 + get_slice(prefix, 4, HDR_BLOCK_SIZE - 2)?.iter().position(|&b| b == 0x1A)?;
    require!(le16(prefix, eof_pos + 1)? == HDR_BLOCK_SIZE as u16);
    Some(Label::BgiChr)
}

/// CHX Big Font (`.chx`, 1995).
pub(crate) fn verify_chx_big_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_CHX_BIG_FONT_SIZE && prefix.len() >= 517 && starts(prefix, b"CHX\x00"));
    // Offset table at @5..517 contains 256 u16 LE offsets; the first non-zero offset is 0x0205 (517)
    let mut found_first = false;
    let mut off = 5usize;
    while off + 2 <= 517 {
        let v = le16(prefix, off)?;
        if v != 0 {
            require!(v == 0x0205);
            found_first = true;
            break;
        }
        off += 2;
    }
    require!(found_first);
    Some(Label::ChxFont)
}

/// Atari 8-Bit Daisy-Dot III NLQ Font (`.nlq`, 1992 Roy E. Myers).
pub(crate) fn verify_daisydot_nlq3_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DAISYDOT_NLQ3_FONT_SIZE
            && prefix.len() >= 512
            && u8_at(prefix, 0)? == b'3'
            && u8_at(prefix, 1)? == 0x9B
    );
    // Walk at least 64 consecutive variable-stride glyph records starting at @2:
    // each record starts with hdr where bit 7 == 0, width = hdr & 0x3F in 1..=32,
    // and rows = 4 if (hdr & 0x40) != 0 else 2.
    let mut pos = 2usize;
    let mut count = 0usize;
    while pos < prefix.len() && count < 64 {
        let hdr = u8_at(prefix, pos)?;
        let w = (hdr & 0x3F) as usize;
        require!((hdr & 0x80) == 0 && (1..=32).contains(&w));
        let rows = if (hdr & 0x40) != 0 { 4 } else { 2 };
        pos += 1 + rows * w;
        count += 1;
    }
    require!(count >= 64);
    Some(Label::DaisydotNlq)
}

/// Atari Daisy-Dot NLQ Font (`.nlq` / `.fnt`, 1986–1992 Roy E. Myers).
pub(crate) fn verify_daisydot_nlq_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DAISYDOT_NLQ_FONT_SIZE
            && prefix.len() >= 28
            && (starts(prefix, b"DAISY-DOT NLQ FONT\x9B")
                || starts(prefix, b"DAISY-DOT II NLQ FONT FILE\x1A")
                || (starts(prefix, b"DOT_MAGIC NLQ FONT")
                    && has_non_ascii_text_byte(prefix.get(18..28)?))
                || (starts(prefix, b"DOT_FILE") && has_non_ascii_text_byte(prefix.get(8..24)?)))
    );
    Some(Label::DaisydotNlq)
}

/// Microsoft DirectXTK `MakeSpriteFont` Binary (`.spritefont`, 2012 Microsoft).
pub(crate) fn verify_dxtk_spritefont(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DXTK_SPRITEFONT_SIZE
            && prefix.len() >= 16
            && starts(prefix, b"DXTKfont")
            && le32(prefix, 8).is_some_and(|c| (1..=65536).contains(&c))
    );
    Some(Label::DxtkSpritefont)
}

/// Electronic Arts Game Bitmap Font (`.fnt` / `.ffn` / `.vfn`, 1994 Electronic Arts).
pub(crate) fn verify_ea_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EA_FONT_SIZE
            && prefix.len() >= 18
            && (starts(prefix, b"FNTF")
                || starts(prefix, b"FntF")
                || starts(prefix, b"FNTS")
                || starts(prefix, b"FNTI"))
    );
    let len_le = le32(prefix, EA_FONT_FILE_SIZE_OFFSET)? as u64;
    let len_be = be32(prefix, EA_FONT_FILE_SIZE_OFFSET)? as u64;
    let fnti_len = le32(prefix, EA_FNTI_FILE_SIZE_OFFSET)? as u64;
    require!(
        (len_le >= MIN_EA_FONT_SIZE && (len_le == size || (len_le + 16 >= size && len_le <= size)))
            || (len_be >= MIN_EA_FONT_SIZE && len_be == size)
            || (starts(prefix, b"FNTI") && fnti_len == size && le16(prefix, 16) == Some(1))
    );
    Some(Label::EaFfn)
}

/// Envision Publisher Scalable Vector Font (`.svf`, 1991 Software Publishing / Envision Solutions).
pub(crate) fn verify_envision_publisher_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ENVISION_PUBLISHER_FONT_SIZE
            && prefix.len() >= 38
            && starts(prefix, hex2bin!("44 83 F9 F8 F8 F9 F9 F9 11 FA F8 F9 F8 F9 D9 F9 19 F9"))
            && eq_at(prefix, 26, hex2bin!("61 BD FD FB CF F9 D7 F9 9D F9 E9 F9"))
    );
    Some(Label::EnvisionSvf)
}

/// Fontconfig Binary Font Cache (`.cache-4`..`.cache-9`, 2002 Keith Packard / freedesktop.org).
pub(crate) fn verify_fontconfig_cache(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_FONTCONFIG_CACHE_SIZE && prefix.len() >= 32);
    let m_le = le32(prefix, 0)?;
    let m_be = be32(prefix, 0)?;
    let is_le = matches!(m_le, 0xFC02_FC04 | 0xFC02_FC05);
    let is_be = matches!(m_be, 0xFC02_FC04 | 0xFC02_FC05);
    require!(is_le || is_be);

    let version = if is_le {
        le32(prefix, FC_CACHE_VERSION_OFFSET)?
    } else {
        be32(prefix, FC_CACHE_VERSION_OFFSET)?
    };
    require!((1..=16).contains(&version));

    let declared_size = if is_le {
        let s64 = le64(prefix, FC_CACHE_SIZE_OFFSET)?;
        if s64 == size { s64 } else { le32(prefix, FC_CACHE_SIZE_OFFSET)? as u64 }
    } else {
        let s64 = be64(prefix, FC_CACHE_SIZE_OFFSET)?;
        if s64 == size { s64 } else { be32(prefix, FC_CACHE_SIZE_OFFSET)? as u64 }
    };
    require!(declared_size >= MIN_FONTCONFIG_CACHE_SIZE && declared_size <= size);
    Some(Label::FontconfigCache)
}

/// FontLab Studio Vector Font Binary (`.vfb`, 1993 Yuri Yarmola / FontLab).
pub(crate) fn verify_fontlab_vfb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_FONTLAB_VFB_SIZE && prefix.len() >= 12 && starts(prefix, b"\x1AWLF10"));
    Some(Label::Vfb)
}

/// Altsys / Macromedia Fontographer Database (`.fog`, 1986 / 1995 Altsys / Macromedia).
pub(crate) fn verify_fontographer_fog(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FONTOGRAPHER_FOG_SIZE
            && prefix.len() >= 16
            && starts(prefix, b"Cff1")
            && ((be16(prefix, 4) == Some(0x8000)
                && be16(prefix, 6).is_some_and(|v| (1..=16).contains(&v)))
                || eq_at(prefix, 4, hex2bin!("BC 99 99 CE 40 00")))
    );
    Some(Label::Fog)
}

/// Hercules *Write On!* RamFont Bitmap Font (`.wof`, 1987 Hercules Computer Technology).
///
/// Hercules Graphics Card Plus (GB112, 1986) and InColor Card (GB222, 1987) RamFont mode
/// stores up to 3,072 8x14 hardware character cells in video RAM; *Write On!* stores 8x14,
/// 16x14, and 16x28 multi-tile high-resolution bitmap fonts with a 22-byte header and exact
/// file size `22 + (last_char - first_char + 1) * ceil(width / 8) * height`.
pub(crate) fn verify_hercules_wof(prefix: &[u8], size: u64) -> Option<Label> {
    const WOF_HEADER_LEN: u16 = 22; // 22-byte fixed header size
    const HEADER_SIZE_OFF: usize = 4; // @4..6: u16 LE header_size (22)
    const WIDTH_OFF: usize = 6; // @6..8: u16 LE glyph width in pixels (1..=64)
    const HEIGHT_OFF: usize = 8; // @8..10: u16 LE glyph height in scanlines (1..=64)
    const TILE_HEIGHT_OFF: usize = 10; // @10..12: u16 LE hardware RamFont tile height (14)
    const FIRST_CHAR_OFF: usize = 12; // @12..14: u16 LE first character code (0..=4095)
    const LAST_CHAR_OFF: usize = 14; // @14..16: u16 LE last character code (0..=4095 across 48 KiB RamFont banks)
    const RAMFONT_TILE_HEIGHT: u16 = 14;

    require!(
        size >= MIN_HERCULES_WOF_SIZE,
        prefix.len() >= WOF_HEADER_LEN as usize,
        starts(prefix, HERCULES_WOF_MAGIC)
    );
    let hdr_sz = le16(prefix, HEADER_SIZE_OFF)?;
    let width = le16(prefix, WIDTH_OFF)?;
    let height = le16(prefix, HEIGHT_OFF)?;
    let tile_h = le16(prefix, TILE_HEIGHT_OFF)?;
    let first_ch = le16(prefix, FIRST_CHAR_OFF)?;
    let last_ch = le16(prefix, LAST_CHAR_OFF)?;
    require!(
        hdr_sz == WOF_HEADER_LEN,
        tile_h == RAMFONT_TILE_HEIGHT,
        (1..=64).contains(&width),
        (1..=64).contains(&height),
        first_ch <= last_ch,
        last_ch <= 4095
    );
    let num_glyphs = (last_ch - first_ch) as u64 + 1;
    let row_bytes = (width as u64).div_ceil(8);
    let expected_size = (WOF_HEADER_LEN as u64) + num_glyphs * row_bytes * (height as u64);
    require!(
        size == expected_size,
        has_non_ascii_text_byte(prefix.get(..WOF_HEADER_LEN as usize)?)
    );
    Some(Label::WriteonWof)
}

/// High-Logic FontCreator Font Project (.fcp, 2013) High-Logic.
pub(crate) fn verify_highlogic_fcp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HIGHLOGIC_FCP_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"\x89FCP\r\n\x1a\n"),
        get_slice(prefix, 20, 24)? == b"META",
        get_slice(prefix, 37, 41)? == b"FONT"
    );
    Some(Label::HighlogicFcp)
}

/// HP PCL / PaintJet Downloadable Soft Font (`.sfp` / `.sfl` / `.pjf`, 1985 Hewlett-Packard).
///
/// Begins with the PCL *Create Font Header* escape sequence `ESC ) s <len> W` (`1B 29 73 ... 57`)
/// followed by a big-endian binary Font Descriptor (`u16 BE font_descriptor_size >= 64`,
/// `u8 descriptor_format <= 20`, `u8 font_type`, `u8 orientation <= 3`, `u8 spacing <= 1`).
pub(crate) fn verify_hp_pcl_softfont(prefix: &[u8], size: u64) -> Option<Label> {
    const LEN_DIGITS_OFF: usize = 3; // @3..: ASCII decimal byte length of font header payload
    const DESC_SIZE_REL: usize = 0; // +0..2: u16 BE font_descriptor_size (>= 64, <= decl_len)
    const DESC_FMT_REL: usize = 2; // +2: u8 descriptor_format (0..=20)
    const FONT_TYPE_REL: usize = 3; // +3: u8 font_type (0=7-bit, 1=8-bit, 2=PC-8, 10/11=unbound)
    const ORIENTATION_REL: usize = 12; // +12: u8 orientation (0..=3: Portrait/Landscape/Reverse)
    const SPACING_REL: usize = 13; // +13: u8 spacing (0=fixed, 1=proportional)

    require!(
        size >= MIN_HP_PCL_SOFTFONT_SIZE,
        prefix.len() >= MIN_HP_PCL_SOFTFONT_SIZE as usize,
        starts(prefix, HP_PCL_FONT_HDR_PREFIX)
    );
    let digit_limit = prefix.len().min(LEN_DIGITS_OFF + 5);
    let mut i = LEN_DIGITS_OFF;
    let mut decl_len: usize = 0;
    while i < digit_limit && u8_at(prefix, i)?.is_ascii_digit() {
        decl_len = decl_len * 10 + (u8_at(prefix, i)? - b'0') as usize;
        i += 1;
    }
    require!(i > LEN_DIGITS_OFF, u8_at(prefix, i) == Some(b'W'), (64..=16_384).contains(&decl_len));
    let desc_off = i + 1;
    require!(prefix.len() >= desc_off + 16, size >= (desc_off + decl_len) as u64);
    let desc_size = be16(prefix, desc_off + DESC_SIZE_REL)? as usize;
    let desc_fmt = u8_at(prefix, desc_off + DESC_FMT_REL)?;
    let font_type = u8_at(prefix, desc_off + FONT_TYPE_REL)?;
    let orientation = u8_at(prefix, desc_off + ORIENTATION_REL)?;
    let spacing = u8_at(prefix, desc_off + SPACING_REL)?;
    require!(
        desc_size >= 64,
        desc_size <= decl_len,
        desc_fmt <= 20,
        matches!(font_type, 0..=2 | 10..=11),
        orientation <= 3,
        spacing <= 1,
        has_non_ascii_text_byte(prefix.get(desc_off..desc_off + 16)?)
    );
    Some(Label::HpPclSoftfont)
}

/// Linux Console PC Screen Font (PSF1 / PSF2).
pub(crate) fn verify_linux_psf1(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_LINUX_PSF1_SIZE && prefix.len() >= 16);
    let mode = u8_at(prefix, 2)?;
    let charsize = u8_at(prefix, 3)? as u64;
    require!(mode <= 0x05 && (6..=32).contains(&charsize));
    let num_glyphs = if (mode & 0x01) != 0 { 512u64 } else { 256u64 };
    let glyph_bytes = 4 + num_glyphs * charsize;
    let valid_size = if (mode & 0x06) == 0 { size == glyph_bytes } else { size > glyph_bytes };
    require!(valid_size);
    Some(Label::LinuxPsf)
}

/// Linux Console PC Screen Font (PSF1 / PSF2).
pub(crate) fn verify_linux_psf2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_LINUX_PSF2_SIZE && prefix.len() >= 32);
    let version = le32(prefix, 4)?;
    let headersize = le32(prefix, 8)?;
    let flags = le32(prefix, 12)?;
    let length = le32(prefix, 16)?;
    let charsize = le32(prefix, 20)?;
    let height = le32(prefix, 24)?;
    let width = le32(prefix, 28)?;
    require!(
        version == 0
            && headersize >= 32
            && flags <= 1
            && (1..=4096).contains(&length)
            && charsize > 0
            && (4..=64).contains(&height)
            && (4..=64).contains(&width)
            && (headersize as u64) + (length as u64) * (charsize as u64) <= size
    );
    Some(Label::LinuxPsf)
}

/// MCFT Bitmap/Vector Font Resource (.cft).
pub(crate) fn verify_mcft_cft(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MCFT_CFT_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"MCFT\0\0\x04\x08\0\0\xc0@\x90\0\0\0")
    );
    Some(Label::McftCft)
}

/// Nintendo NW4R / NW4C / NW4F Binary Font (`.brfnt` / `.bcfnt` / `.bffnt`, 2006–2017 Nintendo).
pub(crate) fn verify_nintendo_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NINTENDO_FONT_SIZE
            && prefix.len() >= 20
            && (starts(prefix, b"CFNT")
                || starts(prefix, b"FFNT")
                || starts(prefix, b"RFNT")
                || starts(prefix, b"RTFN"))
            && matches!(le16(prefix, 4), Some(0xFEFF | 0xFFFE))
    );
    Some(Label::NintendoFont)
}

/// Keith P. Graham Optiks PCR Font (`.pcr`, 1988 Keith P. Graham).
pub(crate) fn verify_optiks_pcr_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_OPTIKS_PCR_FONT_SIZE
            && prefix.len() >= 11
            && starts(prefix, b"KPG\x01")
            && u8_at(prefix, 4)? <= 4
            && u8_at(prefix, 5)? == 0x20
    );
    Some(Label::PcrFont)
}

/// OS/2 Presentation Manager font: `FOCASIGNATURE` block `u32 LE 0xFFFFFFFE`, `u32 LE 20`, `"OS/2 FONT"`.
pub(crate) fn verify_os2_pm_font(prefix: &[u8], size: u64) -> Option<Label> {
    const SIGNATURE: &[u8] = bytes!(le32_bytes!(0xFFFF_FFFE), le32_bytes!(20), b"OS/2 FONT"); // @0, 17 bytes
    require!(size >= MIN_OS2_PM_FONT_SIZE, starts(prefix, SIGNATURE));
    Some(Label::Os2Font)
}

/// Google Picasa Rasterized Font Metrics & Glyph Atlas (`.ytf`).
pub(crate) fn verify_picasa_ytf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PICASA_YTF_SIZE,
        prefix.len() >= 48,
        le32(prefix, 0) == Some(100),
        eq_at(prefix, 20, hex2bin!("00 00 80 3F"))
    );
    let pt_size = le32(prefix, 4)?;
    let weight = le32(prefix, 8)?;
    let style = le32(prefix, 12)?;
    let flags = le32(prefix, 16)?;
    let atlas_w = le32(prefix, 28)?;
    let atlas_h = le32(prefix, 32)?;
    let name_len = le32(prefix, 40)? as usize;
    require!(
        (4..=512).contains(&pt_size),
        (100..=1000).contains(&weight),
        style <= 16,
        (1..=16).contains(&flags),
        (64..=4096).contains(&atlas_w),
        atlas_w.is_power_of_two(),
        (64..=4096).contains(&atlas_h),
        atlas_h.is_power_of_two(),
        (2..=64).contains(&name_len)
    );
    let check_len = name_len.min(4);
    require!(get_slice(prefix, 44, 44 + check_len)?.iter().all(|&b| (0x20..=0x7E).contains(&b)));
    Some(Label::PicasaYtf)
}

/// PLIB / OpenGL Texture Mapped Font (`.txf`, 1997 Mark J. Kilgard & Steve Baker).
pub(crate) fn verify_plib_txf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PLIB_TXF_SIZE && prefix.len() >= 28 && starts(prefix, bytes!(&[0xFF], b"txf"))
    );
    let is_le = le32(prefix, PLIB_TXF_ENDIAN_OFFSET) == Some(0x1234_5678);
    let is_be = be32(prefix, PLIB_TXF_ENDIAN_OFFSET) == Some(0x1234_5678);
    require!(is_le || is_be);

    let (fmt, w, h) = if is_le {
        (
            le32(prefix, PLIB_TXF_FORMAT_OFFSET)?,
            le32(prefix, PLIB_TXF_WIDTH_OFFSET)?,
            le32(prefix, PLIB_TXF_HEIGHT_OFFSET)?,
        )
    } else {
        (
            be32(prefix, PLIB_TXF_FORMAT_OFFSET)?,
            be32(prefix, PLIB_TXF_WIDTH_OFFSET)?,
            be32(prefix, PLIB_TXF_HEIGHT_OFFSET)?,
        )
    };
    require!(
        fmt <= 1
            && w.is_power_of_two()
            && h.is_power_of_two()
            && (8..=8192).contains(&w)
            && (8..=8192).contains(&h)
    );
    Some(Label::Txf)
}

/// Raven Software Engine XBNF Binary Font / Text Resource (.bnx).
pub(crate) fn verify_raven_xbnf_bnx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RAVEN_XBNF_BNX_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"XBNF\x01\0\0\0\0\0\0\0\x0c\0"),
        get_slice(prefix, 32, 40)? == b"\x00\x01\x02\x03\x04\x05\x06\x07"
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::RavenXbnfBnx)
}

/// Acorn RISC OS Font `IntMetrics` file (`IntMetrics` / `IntMetric0`, 1987 Acorn).
pub(crate) fn verify_riscos_intmetrics(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RISCOS_INTMETRICS_SIZE
            && prefix.len() >= 52
            && le32(prefix, RISCOS_INTMETRICS_XSCALE_OFFSET) == Some(16)
            && le32(prefix, RISCOS_INTMETRICS_YSCALE_OFFSET) == Some(16)
    );
    let cr_pos = get_slice(prefix, 0, 40)?.iter().position(|&b| b == 0x0D)?;
    require!(
        cr_pos >= 2 && get_slice(prefix, 0, cr_pos)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::RiscosFont)
}

/// Acorn RISC OS Outline Font (`Outlines`, 1987 Acorn).
pub(crate) fn verify_riscos_outlines(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RISCOS_OUTLINES_SIZE
            && prefix.len() >= 24
            && starts(prefix, b"FONT\x00")
            && u8_at(prefix, 5)? <= 8
            && le16(prefix, 6).unwrap_or(0) > 0
            && le32(prefix, 16).is_some_and(|off| (20..size as u32).contains(&off))
    );
    Some(Label::RiscosFont)
}

/// RTSTRK01 Zlib-Compressed Font Strike (.rtstrike).
pub(crate) fn verify_rtstrike_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RTSTRIKE_FONT_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"RTSTRK01"),
        le32(prefix, 8).unwrap_or(0) > 0,
        is_valid_zlib_header(u8_at(prefix, 12)?, u8_at(prefix, 13)?)
    );
    Some(Label::RtstrikeFont)
}

/// Application Systems Heidelberg Signum! Font (`.e24` / `.l30` / `.p24` / `.p9`, 1986 Application Systems Heidelberg).
pub(crate) fn verify_signum_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SIGNUM_FONT_SIZE
            && prefix.len() >= 16
            && (starts(prefix, b"eset0001")
                || starts(prefix, b"ls300001")
                || starts(prefix, b"ps240001")
                || starts(prefix, b"ps090001"))
            && prefix.get(8..16)?.contains(&0)
    );
    Some(Label::SignumFont)
}

/// Sinclair QL QLBAS Bitmap Font Resource (.f3).
pub(crate) fn verify_sinclair_ql_f3(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SINCLAIR_QL_F3_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 2, 32)?
            == hex2bin!(
                "03 00 00 03 07 1F 1F 3F 3F 01 04 7F FF FF 43 FF 04 7F 00 3F 3F 1F 1F 07 03 01 04 00 00 0F"
            )
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::SinclairQlF3)
}

/// TeX Font Metrics file (`.tfm`, 1982 Donald E. Knuth).
pub(crate) fn verify_tex_tfm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_TEX_TFM_SIZE && size.is_multiple_of(4) && prefix.len() >= 32);
    let lf = be16(prefix, 0)? as u64;
    let lh = be16(prefix, 2)? as u64;
    let bc = be16(prefix, 4)? as i64;
    let ec = be16(prefix, 6)? as i64;
    let nw = be16(prefix, 8)? as u64;
    let nh = be16(prefix, 10)? as u64;
    let nd = be16(prefix, 12)? as u64;
    let ni = be16(prefix, 14)? as u64;
    let nl = be16(prefix, 16)? as u64;
    let nk = be16(prefix, 18)? as u64;
    let ne = be16(prefix, 20)? as u64;
    let np = be16(prefix, 22)? as u64;

    require!(
        lf >= 16
            && lh >= 2
            && bc >= 0
            && ec <= 255
            && bc <= ec + 1
            && nw >= 1
            && nh >= 1
            && nd >= 1
            && ni >= 1
            && np <= 256
    );
    let char_count = (ec - bc + 1) as u64;
    let expected_lf = 6 + lh + char_count + nw + nh + nd + ni + nl + nk + ne + np;
    require!(lf == expected_lf && (lf * 4 == size || (lf * 4 <= size && lf * 4 + 4 >= size)));

    // Design size at @28..32 (second header word) is a big-endian fix_word (units of 2^-20 pt, >= 1.0 pt = 0x00100000)
    let design_size = be32(prefix, TFM_DESIGN_SIZE_OFFSET)?;
    require!((0x0008_0000..0x8000_0000).contains(&design_size));

    Some(Label::Tfm)
}

/// TheDraw ANSI/ASCII Block Font (`.tdf`, 1986 Ian E. Davis / TheSoft Programming Services).
pub(crate) fn verify_thedraw_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_THEDRAW_FONT_SIZE
            && prefix.len() >= 24
            && starts(prefix, b"\x13TheDraw FONTS file\x1A")
    );
    Some(Label::ThedrawTdf)
}

/// Symantec / Peter Norton Universal BitMap Font (`.ubf`, 1991 Symantec).
pub(crate) fn verify_ubf_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UBF_FONT_SIZE
            && prefix.len() >= 32
            && ((starts(prefix, b"UBF91a\r\n")
                && get_slice(prefix, 8, prefix.len().min(128))?.contains(&0x1A))
                || (starts(prefix, b"UBF92a")
                    && u8_at(prefix, 6)? > 0
                    && u8_at(prefix, 7)? > 0
                    && has_non_ascii_text_byte(prefix.get(6..16)?)))
    );
    Some(Label::UbfFont)
}

/// WordUp Graphics Toolkit font: `"WGT Font File"` followed by two spaces.
pub(crate) fn verify_wgt_font(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WGT_FONT_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"WGT Font File  "),
        has_non_ascii_text_byte(prefix.get(15..24)?)
    );
    Some(Label::WgtFont)
}

/// X11 / Linux Portable Compiled Font (`.pcf` / `.pmf`, 1991 Keith Packard / MIT X Consortium).
pub(crate) fn verify_x11_pcf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_X11_PCF_SIZE && prefix.len() >= 24 && starts(prefix, bytes!(&[0x01], b"fcp"))
    );
    let table_count = le32(prefix, X11_PCF_TABLE_COUNT_OFFSET)? as usize;
    require!((1..=32).contains(&table_count));
    let toc_end = X11_PCF_TOC_OFFSET + table_count * 16;
    require!(size >= toc_end as u64);

    let check_count = table_count.min((prefix.len().saturating_sub(X11_PCF_TOC_OFFSET)) / 16);
    require!(check_count >= 1);
    for i in 0..check_count {
        let base = X11_PCF_TOC_OFFSET + i * 16;
        let tbl_type = le32(prefix, base)?;
        let tbl_fmt = le32(prefix, base + 4)?;
        let tbl_size = le32(prefix, base + 8)? as u64;
        let tbl_off = le32(prefix, base + 12)? as u64;
        // PCF table type is a single bit power of two in 1..=256 (PCF_PROPERTIES..PCF_BDF_ACCELERATORS)
        require!(tbl_type.is_power_of_two() && tbl_type <= 0x0100);
        // PCF format mask uses only low bits 0x00..0xFF and byte/bit/scan masks 0x3300
        require!((tbl_fmt & !0x33FF) == 0);
        // PCF_ACCELERATORS (0x02) / PCF_BDF_ACCELERATORS (0x100) may omit the 28-byte ink metrics (72 bytes at EOF).
        if matches!(tbl_type, 0x02 | 0x100) {
            require!(tbl_off >= toc_end as u64 && tbl_off.checked_add(24)? <= size);
        } else {
            require!(tbl_off >= toc_end as u64 && tbl_off.checked_add(tbl_size)? <= size);
        }
    }
    Some(Label::Pcf)
}
