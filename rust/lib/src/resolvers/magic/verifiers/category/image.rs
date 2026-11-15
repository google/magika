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

//! Raster, vector, and scientific image format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// Better Portable Graphics (BPG): `"BPG\xfb"` magic, valid pixel format / bit depth byte,
/// color space / flags byte, and two nonzero `ue7(32)` dimensions followed by at least one byte.
pub(crate) fn bpg(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"BPG\xfb"; // @0..4: BPG header magic
    const FORMAT_DEPTH: usize = 4; // @4: pixel_format (bits 7..5, 0..=5) | alpha1 | bit_depth_minus_8 (0..=6)
    const COLOR_FLAGS: usize = 5; // @5: color_space | extension_present | alpha2 | limited_range | animation
    const DIMS_OFFSET: usize = 6; // @6: width and height encoded as ue7(32) varints
    const MIN_LEN: usize = 9;

    /// Advance past one nonzero `ue7(32)` varint at `pos`: `[0x01..=0x7F]` or
    /// `[0x81..=0xFF][0x80..=0xFF]{0,2}[0x00..=0x7F]` or `[0x81..=0x8F][0x80..=0xFF]{3}[0x00..=0x7F]`.
    fn skip_ue7(h: &[u8], pos: usize) -> Option<usize> {
        let first = u8_at(h, pos)?;
        if first == 0x00 || first == 0x80 {
            return None;
        }
        if first < 0x80 {
            return Some(pos + 1);
        }
        let max_cont = if first <= 0x8F { 3 } else { 2 };
        for cont in 0..=max_cont {
            if u8_at(h, pos + 1 + cont)? < 0x80 {
                return Some(pos + 2 + cont);
            }
        }
        None
    }

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    let b4 = u8_at(prefix, FORMAT_DEPTH)?;
    let b5 = u8_at(prefix, COLOR_FLAGS)?;
    require!((b4 & 0x0F) <= 6, (b4 >> 4) <= 0x0B, b5 < 80, b4 >= 32 || b5 < 16,);
    let after_w = skip_ue7(prefix, DIMS_OFFSET)?;
    let after_h = skip_ue7(prefix, after_w)?;
    require!(after_h < prefix.len());
    Some(Label::Bpg)
}

/// Flexible Image Transport System (FITS): complete 2880-byte primary header block opening with a
/// `"SIMPLE  = "` card (`T` or `F`) and a `"BITPIX  = "` card at offset 80 or 160.
pub(crate) fn fits(prefix: &[u8]) -> Option<Label> {
    const BLOCK_LEN: usize = 2880; // one 36 × 80-byte FITS header block
    const CARD_LEN: usize = 80; // 80-byte FITS header card
    const SECOND_CARD_PAD: usize = 89; // @89..91: two spaces in second card value field
    const SIMPLE_KEY: &[u8] = b"SIMPLE  =";
    const BITPIX_KEY: &[u8] = b"BITPIX  =";
    const BITPIX_VALUES: &[&[u8]] = &[b"8", b"16", b"32", b"64", b"-32", b"-64"];

    fn match_simple(card: &[u8]) -> bool {
        let Some(rest) = card.strip_prefix(SIMPLE_KEY) else { return false };
        let spaces = rest.iter().take_while(|&&b| b == b' ').count();
        (1..=20).contains(&spaces)
            && matches!(rest.get(spaces), Some(b'T' | b'F'))
            && matches!(rest.get(spaces + 1), Some(b' ' | b'/'))
    }

    fn match_bitpix(card: &[u8]) -> bool {
        let Some(rest) = card.strip_prefix(BITPIX_KEY) else { return false };
        let spaces = rest.iter().take_while(|&&b| b == b' ').count();
        if !(1..=20).contains(&spaces) {
            return false;
        }
        let after_spaces = &rest[spaces..];
        for &val in BITPIX_VALUES {
            if let Some(tail) = after_spaces.strip_prefix(val)
                && matches!(tail.first(), Some(b' ' | b'/'))
            {
                return true;
            }
        }
        false
    }

    require!(
        prefix.len() >= BLOCK_LEN,
        be16(prefix, SECOND_CARD_PAD)? == 0x2020,
        match_simple(prefix),
        match_bitpix(get_slice(prefix, CARD_LEN, prefix.len())?)
            || match_bitpix(get_slice(prefix, 2 * CARD_LEN, prefix.len())?),
    );
    Some(Label::Fits)
}

/// Apple Icon Image (`icns`): `"icns"` magic, container length (8 for an empty container, or
/// ≥ 16 with a first element of length ≥ 8 at offset 12).
pub(crate) fn icns(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"icns"; // @0..4: Apple icon container magic
    const FILE_LEN: usize = 4; // @4: u32 BE — total file length
    const FIRST_ELEM_LEN: usize = 12; // @12: u32 BE — first icon element length
    const EMPTY_LEN: usize = 8;
    const ELEM_MIN_LEN: usize = 16;

    require!(prefix.len() >= EMPTY_LEN, starts(prefix, MAGIC));
    let total = be32(prefix, FILE_LEN)?;
    let valid = total == EMPTY_LEN as u32
        || (prefix.len() >= ELEM_MIN_LEN
            && (ELEM_MIN_LEN as u32..0x0800_0000).contains(&total)
            && (EMPTY_LEN as u32..0x0800_0000).contains(&be32(prefix, FIRST_ELEM_LEN)?));
    require!(valid);
    Some(Label::Icns)
}

/// JPEG Network Graphics (JNG): 8-byte JNG signature followed by a 16-byte `JHDR` chunk header.
pub(crate) fn jng(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"\x8bJNG\r\n\x1a\n", &16u32.to_be_bytes(), b"JHDR"); // @0..16: JNG sig + JHDR chunk

    require!(starts(prefix, HEADER));
    Some(Label::Jng)
}

/// Adobe Photoshop PSD (v1) / PSB (v2): `"8BPS"` signature, version 1 or 2, 6 reserved zero bytes,
/// `1..=56` channels, bounded dimensions, bit depth `1 | 8 | 16 | 32`, and color mode `0..=9`.
pub(crate) fn psd(prefix: &[u8]) -> Option<Label> {
    static PSD: Subformat = Subformat::new("Adobe Photoshop (PSD)");
    static PSB: Subformat =
        Subformat::with_ids("Adobe Photoshop Large Document (PSB)", "fmt/996", Some("fdd000523"));
    const MAGIC: &[u8] = b"8BPS"; // @0..4: Photoshop signature
    const VERSION: usize = 4; // @4: u16 BE — 1 = PSD, 2 = PSB
    const RESERVED: usize = 6; // @6..12: 6 reserved zero bytes
    const CHANNELS: usize = 12; // @12: u16 BE — channel count (1..=56)
    const HEIGHT: usize = 14; // @14: u32 BE — height in pixels
    const WIDTH: usize = 18; // @18: u32 BE — width in pixels
    const DEPTH: usize = 22; // @22: u16 BE — bits per channel (1, 8, 16, 32)
    const COLOR_MODE: usize = 24; // @24: u16 BE — color mode (0..=9)
    const HEADER_LEN: usize = 26;

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, MAGIC),
        eq_at(prefix, RESERVED, &[0; 6]),
        (1..=56).contains(&be16(prefix, CHANNELS)?),
        matches!(be16(prefix, DEPTH)?, 1 | 8 | 16 | 32),
        be16(prefix, COLOR_MODE)? <= 9,
    );
    let (ver, h, w) = (be16(prefix, VERSION)?, be32(prefix, HEIGHT)?, be32(prefix, WIDTH)?);
    let max_dim = match ver {
        1 => 30_000,
        2 => 300_000,
        _ => return None,
    };
    require!((1..=max_dim).contains(&h), (1..=max_dim).contains(&w));
    with_subformat(Label::Psd, if ver == 1 { &PSD } else { &PSB })
}

/// Quite OK Image (QOI): `"qoif"` magic, nonzero width and height, 3 or 4 channels, and colorspace
/// `0` (sRGB) or `1` (linear).
pub(crate) fn qoi(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"qoif"; // @0..4: QOI header magic
    const WIDTH: usize = 4; // @4: u32 BE — width (> 0)
    const HEIGHT: usize = 8; // @8: u32 BE — height (> 0)
    const CHANNELS: usize = 12; // @12: u8 — 3 (RGB) or 4 (RGBA)
    const COLORSPACE: usize = 13; // @13: u8 — 0 or 1
    const MIN_LEN: usize = 23; // 14-byte header + ≥1 chunk byte + 8-byte end marker

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        be32(prefix, WIDTH)? > 0,
        be32(prefix, HEIGHT)? > 0,
        matches!(u8_at(prefix, CHANNELS)?, 3 | 4),
        u8_at(prefix, COLORSPACE)? <= 1,
    );
    Some(Label::Qoi)
}

/// GIMP XCF image: `"gimp xcf "` followed by `"file\0"` or `"v[0-9]{3}\0"`, width/height words,
/// and base image type `0..=2` (RGB, grayscale, indexed) at `@22`.
pub(crate) fn xcf(prefix: &[u8]) -> Option<Label> {
    static XCF_V0: Subformat = Subformat::new("GIMP XCF v0");
    static XCF_VNNN: Subformat = Subformat::new("GIMP XCF vNNN");
    const MAGIC: &[u8] = b"gimp xcf "; // @0..9: XCF magic prefix
    const VERSION: usize = 9; // @9..14: "file\0" or "vNNN\0"
    const VERSION_V0: &[u8] = b"file\0";
    const IMAGE_TYPE: usize = 22; // @22: u32 BE — base image type (0..=2)
    const MIN_LEN: usize = 26;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC), be32(prefix, IMAGE_TYPE)? <= 2);
    let is_v0 = eq_at(prefix, VERSION, VERSION_V0);
    let ver_ok = is_v0
        || (u8_at(prefix, VERSION)? == b'v'
            && get_slice(prefix, 10, 13).is_some_and(|s| s.iter().all(u8::is_ascii_digit))
            && u8_at(prefix, 13)? == 0);
    require!(ver_ok);
    with_subformat(Label::Xcf, if is_v0 { &XCF_V0 } else { &XCF_VNNN })
}

/// CompuServe GIF (`"GIF87a"` or `"GIF89a"`): nonzero width and height at `@6` and `@8`, and a
/// valid block introducer (`0x21` extension, `0x2C` image descriptor, or `0x3B` trailer) right
/// after the optional Global Color Table.
pub(crate) fn gif(prefix: &[u8]) -> Option<Label> {
    static GIF_87A: Subformat = Subformat::with_ids("GIF 87a", "fmt/3", None);
    static GIF_89A: Subformat = Subformat::with_ids("GIF 89a", "fmt/4", Some("fdd000133"));
    const GIF87A: &[u8] = b"GIF87a"; // @0..6: GIF87a header
    const GIF89A: &[u8] = b"GIF89a"; // @0..6: GIF89a header
    const WIDTH: usize = 6; // @6: u16 LE — logical screen width (> 0)
    const HEIGHT: usize = 8; // @8: u16 LE — logical screen height (> 0)
    const PACKED: usize = 10; // @10: u8 — bit 7 = GCT present, bits 2..0 = GCT size exponent
    const END_OF_LSD: usize = 13; // @13: first byte after 6-byte header + 7-byte Logical Screen Descriptor
    const MIN_LEN: usize = 14;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, GIF87A) || starts(prefix, GIF89A),
        le16(prefix, WIDTH)? > 0,
        le16(prefix, HEIGHT)? > 0,
        has_non_ascii_text_byte(&prefix[6..END_OF_LSD]),
    );
    let packed = u8_at(prefix, PACKED)?;
    let first_block =
        if packed < 0x80 { END_OF_LSD } else { END_OF_LSD + 3 * (1usize << ((packed & 0x07) + 1)) };
    require!(matches!(u8_at(prefix, first_block)?, 0x21 | 0x2C | 0x3B));
    with_subformat(Label::Gif, if starts(prefix, GIF87A) { &GIF_87A } else { &GIF_89A })
}

/// Windows Icon (`ICONDIR` + first `ICONDIRENTRY`): reserved `0`, type `1` (`00 00 01 00`),
/// image count ≥ 1 at `@4`, reserved `0` at `@9`, bytes in resource ≥ 8 at `@14`, and image
/// offset ≥ 22 at `@18`.
pub(crate) fn ico(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = &[0x00, 0x00, 0x01, 0x00]; // @0..4: idReserved = 0, idType = 1 (icon)
    const ID_COUNT: usize = 4; // @4: u16 LE — number of images (≥ 1)
    const ENTRY_RESERVED: usize = 9; // @9: u8 — first ICONDIRENTRY.bReserved (0)
    const BYTES_IN_RES: usize = 14; // @14: u32 LE — resource byte size (≥ 8)
    const IMAGE_OFFSET: usize = 18; // @18: u32 LE — offset to image data (≥ 22)
    const MIN_LEN: usize = 22; // 6-byte ICONDIR + 16-byte ICONDIRENTRY

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, HEADER),
        le16(prefix, ID_COUNT)? >= 1,
        u8_at(prefix, ENTRY_RESERVED)? == 0,
        le32(prefix, BYTES_IN_RES)? >= 8,
        le32(prefix, IMAGE_OFFSET)? >= MIN_LEN as u32,
    );
    Some(Label::Ico)
}

/// Commodore 64 compressed Koala Painter image: fixed 4-byte header `FF 80 C9 C7` and ≥ 16 bytes.
pub(crate) fn koala(prefix: &[u8]) -> Option<Label> {
    static KOALA_COMPRESSED: Subformat = Subformat::new("Koala Painter (compressed)");
    const MAGIC: &[u8] = &[0xFF, 0x80, 0xC9, 0xC7]; // @0..4: Koala compressed load header
    const MIN_LEN: usize = 16;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    with_subformat(Label::Koala, &KOALA_COMPRESSED)
}

/// Commodore 64 uncompressed Koala Painter image (`.koa` / `.kla` / `.pic`): 2-byte load address
/// `00 60` (`$6000`) at `@0..2`, exact 10,003-byte file size, and background color nibble `<= 0x0F` at `@10002`.
pub(crate) fn koala_uncompressed(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    static KOALA_UNCOMPRESSED: Subformat = Subformat::new("Koala Painter (uncompressed)");
    const LOAD_ADDR_6000: &[u8] = &[0x00, 0x60];
    require!(
        ctx.size == 10_003,
        prefix.len() >= 10_003,
        starts(prefix, LOAD_ADDR_6000),
        u8_at(prefix, 10_002)? <= 0x0F,
    );
    let _ = get_slice(prefix, 0, 16);
    with_subformat(Label::Koala, &KOALA_UNCOMPRESSED)
}

/// Netpbm image (`"P1"`..`"P6"`): 1..=8 whitespace bytes, up to 2 `#...\n` comment lines (each ≤
/// 128 comment bytes followed by `\n` and 0..=4 whitespace bytes), width (1..=6 ASCII digits),
/// 1..=4 whitespace bytes, height (1..=6 ASCII digits), and a trailing whitespace byte.
pub(crate) fn pbm(prefix: &[u8]) -> Option<Label> {
    static PBM_ASCII: Subformat = Subformat::with_ids("PBM (ASCII)", "x-fmt/164", None);
    static PGM_ASCII: Subformat = Subformat::with_ids("PGM (ASCII)", "fmt/407", None);
    static PPM_ASCII: Subformat = Subformat::with_ids("PPM (ASCII)", "x-fmt/178", None);
    static PBM_BINARY: Subformat = Subformat::with_ids("PBM (binary)", "fmt/409", None);
    static PGM_BINARY: Subformat = Subformat::with_ids("PGM (binary)", "fmt/406", None);
    static PPM_BINARY: Subformat = Subformat::with_ids("PPM (binary)", "fmt/408", None);
    #[inline]
    fn is_ws(b: u8) -> bool {
        matches!(b, b' ' | b'\t' | b'\r' | b'\n')
    }

    #[inline]
    fn take_run(s: &[u8], i: &mut usize, pred: fn(u8) -> bool) -> usize {
        let start = *i;
        while *i < s.len() && pred(s[*i]) {
            *i += 1;
        }
        *i - start
    }

    require!(prefix.len() >= 8, prefix[0] == b'P', matches!(prefix[1], b'1'..=b'6'));
    let mut i = 2;

    // [ \t\r\n]{1,8}
    let ws0 = take_run(prefix, &mut i, is_ws);
    require!((1..=8).contains(&ws0));

    // (#[^\n]{0,128}\n[ \t\r\n]{0,4}){0,2}
    for _ in 0..2 {
        if u8_at(prefix, i) != Some(b'#') {
            break;
        }
        i += 1;
        let comment_len = take_run(prefix, &mut i, |b| b != b'\n');
        require!(comment_len <= 128, u8_at(prefix, i) == Some(b'\n'));
        i += 1;
        let ws_after = take_run(prefix, &mut i, is_ws);
        require!(ws_after <= 4);
    }

    // [0-9]{1,6}[ \t\r\n]{1,4}[0-9]{1,6}[ \t\r\n]
    let width_digits = take_run(prefix, &mut i, |b| b.is_ascii_digit());
    require!((1..=6).contains(&width_digits));
    let ws_mid = take_run(prefix, &mut i, is_ws);
    require!((1..=4).contains(&ws_mid));
    let height_digits = take_run(prefix, &mut i, |b| b.is_ascii_digit());
    require!((1..=6).contains(&height_digits), is_ws(u8_at(prefix, i)?),);
    let _ = get_slice(prefix, 0, i + 1);
    let sub = match prefix[1] {
        b'1' => &PBM_ASCII,
        b'2' => &PGM_ASCII,
        b'3' => &PPM_ASCII,
        b'4' => &PBM_BINARY,
        b'5' => &PGM_BINARY,
        _ => &PPM_BINARY,
    };
    with_subformat(Label::Pbm, sub)
}

/// Windows or OS/2 BMP image (`"BM"`): zero reserved words at `@6..10`, file size and pixel
/// offset at `@2` and `@10`, followed at `@14` by either a 12-byte OS/2 `BITMAPCOREHEADER` or an
/// uncompressed Windows DIB header (40, 52, 56, 108, or 124 bytes).
pub(crate) fn bmp(prefix: &[u8]) -> Option<Label> {
    static BMP_CORE: Subformat =
        Subformat::with_ids("BMP (OS/2 1.x / Windows 2.x)", "fmt/115", None);
    static BMP_V3: Subformat = Subformat::with_ids("BMP v3 (BITMAPINFOHEADER)", "fmt/116", None);
    static BMP_V3_V2: Subformat = Subformat::new("BMP v3 (BITMAPV2INFOHEADER)");
    static BMP_V3_V3: Subformat = Subformat::new("BMP v3 (BITMAPV3INFOHEADER)");
    static BMP_V4: Subformat = Subformat::with_ids("BMP v4 (BITMAPV4HEADER)", "fmt/118", None);
    static BMP_V5: Subformat =
        Subformat::with_ids("BMP v5 (BITMAPV5HEADER)", "fmt/119", Some("fdd000189"));
    const MAGIC: &[u8] = b"BM"; // @0..2: BITMAPFILEHEADER magic
    const FILE_SIZE: usize = 2; // @2: u32 LE — bfSize (≥ 26)
    const RESERVED: usize = 6; // @6: u32 LE — bfReserved1 + bfReserved2 (0)
    const OFF_BITS: usize = 10; // @10: u32 LE — bfOffBits (pixel array offset)
    const DIB_SIZE: usize = 14; // @14: u32 LE — DIB header size
    const MIN_CORE_LEN: usize = 26; // 14-byte file header + 12-byte BITMAPCOREHEADER
    const MIN_WIN_LEN: usize = 54; // 14-byte file header + 40-byte BITMAPINFOHEADER

    require!(
        prefix.len() >= MIN_CORE_LEN,
        starts(prefix, MAGIC),
        le32(prefix, RESERVED)? == 0,
        le32(prefix, FILE_SIZE)? >= MIN_CORE_LEN as u32,
    );
    let off_bits = le32(prefix, OFF_BITS)?;
    require!(off_bits >= MIN_CORE_LEN as u32);
    let dib_size = le32(prefix, DIB_SIZE)?;
    let valid = if dib_size == 12 {
        // OS/2 BITMAPCOREHEADER: u16 width @18, u16 height @20, u16 planes @22, u16 bpp @24
        le16(prefix, 18)? >= 1
            && le16(prefix, 20)? >= 1
            && le16(prefix, 22)? == 1
            && matches!(le16(prefix, 24)?, 1 | 4 | 8 | 24)
    } else {
        // Windows BITMAPINFOHEADER / V2 / V3 / V4 / V5
        prefix.len() >= MIN_WIN_LEN
            && off_bits >= MIN_WIN_LEN as u32
            && matches!(dib_size, 40 | 52 | 56 | 108 | 124)
            && (1..=0x7FFF_FFFF).contains(&le32(prefix, 18)?)
            && le32(prefix, 22)? >= 1
            && le16(prefix, 26)? == 1
            && matches!(le16(prefix, 28)?, 1 | 4 | 8 | 16 | 24 | 32)
            && le32(prefix, 30)? == 0
    };
    require!(valid);
    let sub = match dib_size {
        12 => &BMP_CORE,
        40 => &BMP_V3,
        52 => &BMP_V3_V2,
        56 => &BMP_V3_V3,
        108 => &BMP_V4,
        _ => &BMP_V5,
    };
    with_subformat(Label::Bmp, sub)
}

/// DICOM medical image (PS3.10): 128-byte preamble followed by `"DICM"` at `@128`.
pub(crate) fn dicom(prefix: &[u8]) -> Option<Label> {
    const MAGIC_OFFSET: usize = 128; // @128..132: "DICM" after 128-byte preamble
    const MAGIC: &[u8] = b"DICM";
    const MIN_LEN: usize = 132;

    require!(prefix.len() >= MIN_LEN, u8_at(prefix, 0)? == 0, eq_at(prefix, MAGIC_OFFSET, MAGIC),);
    Some(Label::Dicom)
}

/// Microsoft Enhanced Metafile (MS-EMF): `EMR_HEADER` record type `1` at `@0`, signature `" EMF"`
/// and version `0x00010000` at `@40..48`, aligned header/file sizes, ≥ 2 records, and consistent
/// base/extension1/extension2 descriptor offsets (88, 100, or 108 bytes).
pub(crate) fn emf(prefix: &[u8]) -> Option<Label> {
    const EMR_HEADER: &[u8] = &1u32.to_le_bytes(); // @0..4: u32 LE — iType = EMR_HEADER (1)
    const HEADER_SIZE: usize = 4; // @4: u32 LE — nSize of EMR_HEADER record (≥ 88, multiple of 4)
    const SIGNATURE_OFFSET: usize = 40; // @40..48: dSignature (" EMF") + nVersion (0x00010000)
    const SIGNATURE: &[u8] = bytes!(b" EMF", &0x0001_0000u32.to_le_bytes());
    const N_BYTES: usize = 48; // @48: u32 LE — metafile byte size (≥ header + 20-byte EOF, multiple of 4)
    const N_RECORDS: usize = 52; // @52: u32 LE — number of records (≥ 2)
    const N_DESC: usize = 60; // @60: u32 LE — nDescription length
    const OFF_DESC: usize = 64; // @64: u32 LE — offDescription offset
    const OFF_PFD: usize = 92; // @92: u32 LE — offPixelFormat offset (HeaderExtension1)
    const MIN_LEN: usize = 88;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, SIGNATURE_OFFSET, SIGNATURE),
        starts(prefix, EMR_HEADER),
    );
    let hdr_size = le32(prefix, HEADER_SIZE)?;
    let n_bytes = le32(prefix, N_BYTES)?;
    let off_desc = le32(prefix, OFF_DESC)?;
    require!(
        hdr_size >= 88,
        hdr_size % 4 == 0,
        n_bytes >= 108,
        n_bytes % 4 == 0,
        le32(prefix, N_RECORDS)? >= 2,
    );
    let valid_layout = ((hdr_size == 88 && off_desc == 0) || off_desc == 88)
        || (prefix.len() >= 100
            && hdr_size >= 100
            && n_bytes >= 120
            && ((hdr_size == 100 && le32(prefix, N_DESC)? == 0 && le32(prefix, OFF_PFD)? == 0)
                || off_desc == 100
                || le32(prefix, OFF_PFD)? == 100))
        || (prefix.len() >= 108
            && hdr_size >= 108
            && n_bytes >= 128
            && ((hdr_size == 108 && le32(prefix, N_DESC)? == 0 && le32(prefix, OFF_PFD)? == 0)
                || off_desc == 108
                || le32(prefix, OFF_PFD)? == 108));
    require!(valid_layout);
    Some(Label::Emf)
}

/// Atari ST DEGAS / DEGAS Elite uncompressed screen dump (`.pi1`/`.pi2`/`.pi3`): resolution word
/// `0..=2` BE at `@0` and exact file size `32034` (raw) or `32066` (with animation footer).
pub(crate) fn degas(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    static DEGAS: Subformat = Subformat::new("DEGAS");
    static DEGAS_ELITE: Subformat = Subformat::new("DEGAS Elite");
    const RESOLUTION: usize = 0; // @0: u16 BE — 0 (low), 1 (medium), 2 (high)
    const MIN_LEN: usize = 34; // 2-byte resolution + 32-byte palette
    const RAW_SIZE: u64 = 32034;
    const ANIM_SIZE: u64 = 32066;

    require!(
        prefix.len() >= MIN_LEN,
        matches!(ctx.size, RAW_SIZE | ANIM_SIZE),
        be16(prefix, RESOLUTION)? <= 2,
    );
    let _ = get_slice(prefix, 0, MIN_LEN);
    with_subformat(Label::Degas, if ctx.size == RAW_SIZE { &DEGAS } else { &DEGAS_ELITE })
}

/// W3C Portable Network Graphics (`png`, 1996, RFC 2083): 8-byte PNG signature at `@0`,
/// 13-byte `IHDR` chunk at `@8..33`, nonzero dimensions at `@16..24`, valid bit depth
/// (`1|2|4|8|16`) at `@24`, valid color type (`0|2|3|4|6`) at `@25`, and zero
/// compression/filter methods at `@26..28`.
pub(crate) fn png(prefix: &[u8], size: u64) -> Option<Label> {
    static PNG: Subformat = Subformat::new("PNG");
    static PNG_CGBI: Subformat = Subformat::new("Apple CgBI PNG");
    const MIN_PNG_SIZE: u64 = 57; // 8B sig + 25B IHDR + 12B IDAT + 12B IEND
    const PNG_SIG: &[u8] = b"\x89PNG\r\n\x1A\n";
    const CGBI_TAG: &[u8; 4] = b"CgBI";
    const IHDR_TAG: &[u8; 4] = b"IHDR";

    require!(size >= MIN_PNG_SIZE, prefix.len() >= 33, starts(prefix, PNG_SIG));
    // Apple CgBI PNGs prepend a 16-byte `CgBI` chunk (`length == 4`) at `@8..24` before `IHDR`.
    let ihdr = if be32(prefix, 8) == Some(4) && eq_at(prefix, 12, CGBI_TAG) { 24 } else { 8 };
    require!(
        prefix.len() >= ihdr + 25,
        be32(prefix, ihdr)? == 13,
        eq_at(prefix, ihdr + 4, IHDR_TAG),
        be32(prefix, ihdr + 8)? >= 1,
        be32(prefix, ihdr + 12)? >= 1,
        matches!(u8_at(prefix, ihdr + 16)?, 1 | 2 | 4 | 8 | 16),
        matches!(u8_at(prefix, ihdr + 17)?, 0 | 2 | 3 | 4 | 6),
        u8_at(prefix, ihdr + 18)? == 0,
        u8_at(prefix, ihdr + 19)? == 0,
        u8_at(prefix, ihdr + 20)? <= 1,
    );
    with_subformat(Label::Png, if ihdr == 24 { &PNG_CGBI } else { &PNG })
}

/// ITU-T T.81 / ISO/IEC 10918-1 JPEG (`jpeg`, 1992): SOI marker `FF D8` followed by a
/// valid JPEG marker `FF C0..=FE` (excluding `FF F7` JPEG-LS SOF55).
pub(crate) fn jpeg(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_JPEG_SIZE: u64 = 20; // SOI (2B) + minimal frame/quant/huffman/EOI markers
    const JPEG_SOI_PREFIX: &[u8] = &[0xFF, 0xD8, 0xFF];

    require!(size >= MIN_JPEG_SIZE, prefix.len() >= 4, starts(prefix, JPEG_SOI_PREFIX),);
    let m = u8_at(prefix, 3)?;
    require!((0xC0..=0xFE).contains(&m), m != 0xF7);
    Some(Label::Jpeg)
}

/// Classic TIFF little-endian magic (`"II*\0"`).
pub(crate) const TIFF_LE_MAGIC: &[u8; 4] = &[b'I', b'I', 42, 0];

/// Classic TIFF big-endian magic (`"MM\0*"`).
pub(crate) const TIFF_BE_MAGIC: &[u8; 4] = &[b'M', b'M', 0, 42];

/// BigTIFF little-endian magic (`"II+\0"`).
pub(crate) const BIGTIFF_LE_MAGIC: &[u8; 4] = &[b'I', b'I', 43, 0];

/// BigTIFF big-endian magic (`"MM\0+"`).
pub(crate) const BIGTIFF_BE_MAGIC: &[u8; 4] = &[b'M', b'M', 0, 43];

/// Aldus / Adobe Tagged Image File Format (`tiff`, 1986) & BigTIFF (2007):
/// `"II"` (little-endian) or `"MM"` (big-endian) at `@0`, version `42` (classic TIFF with
/// IFD offset `>= 8` at `@4`) or `43` (BigTIFF with offset byte size `8` at `@4` and `0` at `@6`).
pub(crate) fn tiff(prefix: &[u8], size: u64) -> Option<Label> {
    static TIFF_LE: Subformat = Subformat::new("TIFF (little-endian)");
    static TIFF_BE: Subformat = Subformat::new("TIFF (big-endian)");
    static BIGTIFF_LE: Subformat =
        Subformat::with_ids("BigTIFF (little-endian)", "fmt/1917", Some("fdd000328"));
    static BIGTIFF_BE: Subformat =
        Subformat::with_ids("BigTIFF (big-endian)", "fmt/1917", Some("fdd000328"));
    const MIN_TIFF_SIZE: u64 = 14; // 8B TIFF header + 2B entry count + 4B next IFD offset

    require!(size >= MIN_TIFF_SIZE, prefix.len() >= 8);
    let (valid, sub) = if starts(prefix, TIFF_LE_MAGIC) {
        let off = le32(prefix, 4)?;
        (off >= 8 && off % 2 == 0 && (off as u64) + 14 <= size, &TIFF_LE)
    } else if starts(prefix, TIFF_BE_MAGIC) {
        let off = be32(prefix, 4)?;
        (off >= 8 && off % 2 == 0 && (off as u64) + 14 <= size, &TIFF_BE)
    } else if starts(prefix, BIGTIFF_LE_MAGIC) {
        let valid = prefix.len() >= 16
            && le16(prefix, 4)? == 8
            && le16(prefix, 6)? == 0
            && le64(prefix, 8)? >= 16;
        (valid, &BIGTIFF_LE)
    } else if starts(prefix, BIGTIFF_BE_MAGIC) {
        let valid = prefix.len() >= 16
            && be16(prefix, 4)? == 8
            && be16(prefix, 6)? == 0
            && be64(prefix, 8)? >= 16;
        (valid, &BIGTIFF_BE)
    } else {
        (false, &TIFF_LE)
    };
    require!(valid);
    with_subformat(Label::Tiff, sub)
}

/// Aldus Placeable Windows Metafile (`wmf`, 1988): magic `D7 CD C6 9A` at `@0`, `hmf == 0`
/// at `@4`, XOR-16 checksum over the first 10 words at `@20`, followed by standard WMF header
/// (`mtType` `1|2`, `mtHeaderSize == 9`, `mtVersion` `0x0100|0x0300`) at `@22`.
pub(crate) fn wmf_placeable(prefix: &[u8], size: u64) -> Option<Label> {
    static WMF_PLACEABLE: Subformat = Subformat::new("Aldus Placeable Metafile");
    const MIN_WMF_PLACEABLE_SIZE: u64 = 46; // 22B Aldus header + 18B WMF header + 6B META_EOF
    const WMF_ALDUS_MAGIC: &[u8] = &[0xD7, 0xCD, 0xC6, 0x9A];

    require!(
        size >= MIN_WMF_PLACEABLE_SIZE,
        prefix.len() >= 40,
        starts(prefix, WMF_ALDUS_MAGIC),
        le16(prefix, 4)? == 0,
    );
    let mut xor_sum = 0u16;
    for i in 0..10 {
        xor_sum ^= le16(prefix, i * 2)?;
    }
    require!(
        xor_sum == le16(prefix, 20)?,
        matches!(le16(prefix, 22)?, 1 | 2),
        le16(prefix, 24)? == 9,
        matches!(le16(prefix, 26)?, 0x0100 | 0x0300),
        le32(prefix, 28)? >= 9,
    );
    with_subformat(Label::Wmf, &WMF_PLACEABLE)
}

/// Standard Windows GDI Metafile (`wmf`, 1988): `mtType` `1` or `2` LE at `@0`,
/// `mtHeaderSize == 9` (18 bytes) at `@2`, `mtVersion` `0x0100` or `0x0300` at `@4`,
/// `mtSize >= 9` words at `@6`, `mtMaxRecord >= 3` words at `@12`, `mtNoParameters == 0`
/// at `@16`, and first record size `>= 3` at `@18`.
pub(crate) fn wmf_standard(prefix: &[u8], size: u64) -> Option<Label> {
    static WMF_STANDARD: Subformat = Subformat::new("Windows Metafile (standard)");
    const MIN_WMF_STD_SIZE: u64 = 24; // 18B METAHEADER + 6B META_EOF record

    require!(
        size >= MIN_WMF_STD_SIZE,
        prefix.len() >= 24,
        matches!(le16(prefix, 0)?, 1 | 2),
        le16(prefix, 2)? == 9,
        matches!(le16(prefix, 4)?, 0x0100 | 0x0300),
        le32(prefix, 6)? >= 9,
        le32(prefix, 12)? >= 3,
        le16(prefix, 16)? == 0,
        le32(prefix, 18)? >= 3,
    );
    with_subformat(Label::Wmf, &WMF_STANDARD)
}

/// Truevision TGA (`tga`, 1984 / v2.0 1989): either carries the 26-byte v2.0 footer ending
/// with `"TRUEVISION-XFILE.\0"` in `tail`, or satisfies strict 18-byte TGA v1.0 header
/// + exact file size geometry.
pub(crate) fn tga(prefix: &[u8], size: u64, tail: Option<&[u8]>) -> Option<Label> {
    static TGA_2: Subformat = Subformat::with_ids("TGA 2.0", "fmt/402", Some("fdd000180"));
    static TGA_1: Subformat = Subformat::with_ids("TGA 1.0", "x-fmt/367", Some("fdd000179"));
    static TGA_UNSIGNED_FOOTER: Subformat = Subformat::new("TGA (unsigned 26-byte footer)");
    const MIN_TGA_SIZE: u64 = 19; // 18B header + >= 1B pixel data
    const TGA_V2_FOOTER_MAGIC: &[u8; 18] = b"TRUEVISION-XFILE.\0";

    require!(size >= MIN_TGA_SIZE, prefix.len() >= 18);
    let cmap_type = u8_at(prefix, 1)?;
    let img_type = u8_at(prefix, 2)?;
    let cmap_len = le16(prefix, 5)? as u64;
    let cmap_bpp = u8_at(prefix, 7)?;
    let width = le16(prefix, 12)? as u64;
    let height = le16(prefix, 14)? as u64;
    let bpp = u8_at(prefix, 16)?;
    let desc = u8_at(prefix, 17)?;

    require!(
        cmap_type <= 1,
        matches!(img_type, 1 | 2 | 3 | 9 | 10 | 11),
        (1..=16384).contains(&width),
        (1..=16384).contains(&height),
        matches!(bpp, 8 | 15 | 16 | 24 | 32),
        (desc & 0xC0) == 0,
    );
    if cmap_type == 1 {
        require!(cmap_len >= 1, matches!(cmap_bpp, 15 | 16 | 24 | 32));
    } else {
        require!(cmap_len == 0, cmap_bpp == 0, !matches!(img_type, 1 | 9));
    }

    // Check TGA v2.0 26-byte footer ("TRUEVISION-XFILE.\0" at tail end - 18)
    if let Some(t) = tail
        && t.len() >= 26
        && eq_at(t, t.len() - 18, TGA_V2_FOOTER_MAGIC)
    {
        return with_subformat(Label::Tga, &TGA_2);
    }

    // Uncompressed TGA v1.0 exact size match (18B header + ID + colormap + width*height*(bpp/8))
    if matches!(img_type, 1..=3) {
        let id_len = u8_at(prefix, 0)? as u64;
        let cmap_bytes = cmap_len * ((cmap_bpp as u64).div_ceil(8));
        let pixel_bytes = width * height * ((bpp as u64).div_ceil(8));
        let expected = 18 + id_len + cmap_bytes + pixel_bytes;
        if size == expected || size == expected + 26 {
            let sub = if size == expected { &TGA_1 } else { &TGA_UNSIGNED_FOOTER };
            return with_subformat(Label::Tga, sub);
        }
    }
    None
}

/// Checks if a buffer starting with `FF 0A` is a Borland Delphi 1 16-bit `.DFM` binary form
/// or QuickReport `.QRP` (`FF 0A 00` + NUL-terminated or Pascal length-prefixed class name `"T..."` + `"TPF0"`).
#[inline]
pub(crate) fn is_borland_delphi1_dfm_or_qrp(b: &[u8]) -> bool {
    if b.len() < 6 || b[0] != 0xFF || b[1] != 0x0A || b[2] != 0x00 {
        return false;
    }
    if b[3] == b'T' && b[4].is_ascii_alphanumeric() {
        let mut i = 3usize;
        while i < b.len().min(64) && b[i] != 0 {
            if !(b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                return false;
            }
            i += 1;
        }
        return i >= 5 && i < b.len().min(64) && b[i] == 0;
    }
    let name_len = b[3] as usize;
    if !(2..=64).contains(&name_len) || 4 + name_len > b.len() || b[4] != b'T' {
        return false;
    }
    b[4..4 + name_len].iter().all(|&c| c.is_ascii_alphanumeric() || c == b'_')
}

/// Validates a raw JPEG XL codestream (`FF 0A`) `SizeHeader` + `ImageMetadata` + `CustomTransformData`
/// + `ZeroPadToByte` + `FrameHeader` / `Toc` bitstream (ISO/IEC 18181-1 Annex A & B).
pub(crate) fn verify_jxl_codestream(b: &[u8], size: u64) -> Option<Label> {
    static JXL_CODESTREAM: Subformat =
        Subformat::with_ids("JPEG XL codestream", "fmt/1484", Some("fdd000536"));
    const JXL_CODESTREAM_MAGIC: &[u8] = &[0xFF, 0x0A];
    if size < 8
        || b.len() < 8
        || !starts(b, JXL_CODESTREAM_MAGIC)
        || is_borland_delphi1_dfm_or_qrp(b)
    {
        return None;
    }
    let mut val = 0u128;
    let max_bytes = b.len().min(18) - 2;
    for i in 0..max_bytes {
        val |= (b[2 + i] as u128) << (8 * i);
    }
    let max_bits = (max_bytes * 8) as u32;
    let mut pos = 0u32;
    #[inline(always)]
    fn read_bits(val: u128, pos: &mut u32, max_bits: u32, n_bits: u32) -> Option<u32> {
        if *pos + n_bits > max_bits {
            return None;
        }
        let res = ((val >> *pos) & ((1u128 << n_bits) - 1)) as u32;
        *pos += n_bits;
        Some(res)
    }
    macro_rules! u {
        ($n:expr) => {{ read_bits(val, &mut pos, max_bits, $n)? }};
    }
    macro_rules! u_enum {
        () => {{
            match u!(2) {
                0 => 0u32,
                1 => 1u32,
                2 => 2 + u!(4),
                _ => 18 + u!(6),
            }
        }};
    }
    macro_rules! u64_var {
        () => {{
            match u!(2) {
                0 => 0u64,
                1 => 1 + (u!(4) as u64),
                2 => 17 + (u!(8) as u64),
                _ => {
                    let mut r = u!(12) as u64;
                    let mut sh = 12u32;
                    while u!(1) == 1 {
                        if sh >= 60 {
                            r |= (u!(4) as u64) << sh;
                            break;
                        }
                        r |= (u!(8) as u64) << sh;
                        sh += 8;
                    }
                    r
                }
            }
        }};
    }

    let div8 = u!(1) != 0;
    let ysize = if div8 {
        8 * (1 + u!(5))
    } else {
        let dist = u!(2);
        if dist == 3 {
            return None;
        }
        let h_bits = [9u32, 13, 18, 30][dist as usize];
        1 + u!(h_bits)
    };
    let ratio = u!(3);
    let xsize = if ratio == 0 {
        if div8 {
            8 * (1 + u!(5))
        } else {
            let dist = u!(2);
            if dist == 3 {
                return None;
            }
            let w_bits = [9u32, 13, 18, 30][dist as usize];
            1 + u!(w_bits)
        }
    } else {
        match ratio {
            1 => ysize,
            2 => (ysize * 12) / 10,
            3 => (ysize * 4) / 3,
            4 => (ysize * 3) / 2,
            5 => (ysize * 16) / 9,
            6 => (ysize * 5) / 4,
            _ => ysize * 2,
        }
    };
    if ysize == 0
        || xsize == 0
        || ysize > 65536
        || xsize > 65536
        || (b[2] == 0 && b[3] == 0 && ysize == 1 && xsize == 1)
    {
        return None;
    }

    let all_default = u!(1) != 0;
    let mut want_icc = false;
    let mut xyb_encoded = true;
    let mut num_ec = 0u32;
    let mut have_anim = false;
    let mut have_timecode = false;

    if !all_default {
        let extra_fields = u!(1) != 0;
        if extra_fields {
            let _orientation = u!(3);
            let have_intr = u!(1) != 0;
            if have_intr {
                let idiv8 = u!(1) != 0;
                if idiv8 {
                    let _ = u!(5);
                } else {
                    let d = u!(2);
                    if d == 3 {
                        return None;
                    }
                    let _ = u!([9u32, 13, 18, 30][d as usize]);
                }
                if u!(3) == 0 {
                    if idiv8 {
                        let _ = u!(5);
                    } else {
                        let d = u!(2);
                        if d == 3 {
                            return None;
                        }
                        let _ = u!([9u32, 13, 18, 30][d as usize]);
                    }
                }
            }
            let have_prev = u!(1) != 0;
            if have_prev {
                return None;
            }
            have_anim = u!(1) != 0;
            if have_anim {
                let tps_num_sel = u!(2);
                if tps_num_sel == 2 {
                    let _ = u!(10);
                } else if tps_num_sel == 3 {
                    let _ = u!(30);
                }
                let tps_den_sel = u!(2);
                if tps_den_sel == 2 {
                    let _ = u!(8);
                } else if tps_den_sel == 3 {
                    let _ = u!(10);
                }
                let loops_sel = u!(2);
                if loops_sel == 1 {
                    let _ = u!(3);
                } else if loops_sel == 2 {
                    let _ = u!(16);
                } else if loops_sel == 3 {
                    let _ = u!(32);
                }
                have_timecode = u!(1) != 0;
            }
        }
        let fp = u!(1) != 0;
        let bps_sel = u!(2);
        let bps = match bps_sel {
            0 => 8u32,
            1 => 10,
            2 => 12,
            _ => 1 + u!(6),
        };
        if bps > 32 {
            return None;
        }
        if fp && (1 + u!(4)) > 10 {
            return None;
        }
        let _mod16 = u!(1);
        num_ec = match u!(2) {
            0 => 0,
            1 => 1,
            2 => 2 + u!(4),
            _ => return None,
        };
        if num_ec > 4 {
            return None;
        }
        for _ in 0..num_ec {
            let ec_all_default = u!(1) != 0;
            if !ec_all_default {
                let ec_type = u_enum!();
                if ec_type > 16 {
                    return None;
                }
                let ec_fp = u!(1) != 0;
                if u!(2) == 3 && (1 + u!(6)) > 32 {
                    return None;
                }
                if ec_fp && (1 + u!(4)) > 10 {
                    return None;
                }
                if u!(2) == 3 && (1 + u!(3)) > 3 {
                    return None;
                }
                if u!(2) != 0 {
                    return None;
                }
                if ec_type == 0 {
                    let _ = u!(1);
                } else if ec_type == 2 || ec_type == 5 {
                    return None;
                }
            }
        }
        xyb_encoded = u!(1) != 0;
        let ce_all_default = u!(1) != 0;
        if !ce_all_default {
            want_icc = u!(1) != 0;
            let cs = u_enum!();
            if cs > 1 {
                return None;
            }
            if !want_icc {
                let wp = u_enum!();
                if !matches!(wp, 1 | 2 | 10 | 11) {
                    return None;
                }
                if wp == 2 {
                    for _ in 0..2 {
                        let s = u!(2);
                        let _ = u!([19u32, 19, 20, 21][s as usize]);
                    }
                }
                if cs != 1 {
                    let prim = u_enum!();
                    if !matches!(prim, 1 | 2 | 9 | 11) {
                        return None;
                    }
                    if prim == 2 {
                        for _ in 0..6 {
                            let s = u!(2);
                            let _ = u!([19u32, 19, 20, 21][s as usize]);
                        }
                    }
                }
                let have_gamma = u!(1) != 0;
                if have_gamma {
                    let gamma = u!(24);
                    if gamma == 0 || gamma > 10_000_000 {
                        return None;
                    }
                } else {
                    let tf = u_enum!();
                    if !matches!(tf, 1 | 8 | 13 | 16 | 17 | 18) {
                        return None;
                    }
                }
                let ri = u_enum!();
                if ri > 3 {
                    return None;
                }
            }
        }
        if extra_fields && u!(1) == 0 {
            return None;
        }
        if u64_var!() != 0 {
            return None;
        }
    }

    let tr_def = u!(1);
    if tr_def != 1 {
        return None;
    }

    if want_icc {
        let icc_len = u64_var!();
        if (32..=(1 << 20)).contains(&icc_len) {
            let _ = get_slice(b, 0, b.len().min(18));
            return with_subformat(Label::Jxl, &JXL_CODESTREAM);
        }
        return None;
    }

    let rem = (8 - (pos & 7)) & 7;
    if rem > 0 && u!(rem) != 0 {
        return None;
    }

    let fh_all_default = u!(1) != 0;
    if fh_all_default {
        if u!(1) != 0 {
            return None;
        }
        let rem2 = (8 - (pos & 7)) & 7;
        if rem2 > 0 && u!(rem2) != 0 {
            return None;
        }
        if xsize <= 256 && ysize <= 256 {
            let s = u!(2);
            let e0 = u!([10u32, 14, 22, 30][s as usize]) + [0u32, 1024, 17408, 4211712][s as usize];
            if e0 == 0 || e0 > 65536 {
                return None;
            }
            let rem3 = (8 - (pos & 7)) & 7;
            if rem3 > 0 && u!(rem3) != 0 {
                return None;
            }
        }
        let _ = get_slice(b, 0, b.len().min(18));
        return with_subformat(Label::Jxl, &JXL_CODESTREAM);
    }

    let frame_type = u!(2);
    let encoding = u!(1);
    let flags = u64_var!();
    if (flags & !0xB3) != 0 {
        return None;
    }
    let do_ycbcr = if !xyb_encoded { u!(1) != 0 } else { false };
    if do_ycbcr && (flags & 0x20) == 0 {
        let _ = u!(6);
    }
    if (flags & 0x20) == 0 {
        let _ = u!(2);
        for _ in 0..num_ec {
            let _ = u!(2);
        }
    }
    if encoding == 1 {
        let _ = u!(2);
    } else if xyb_encoded {
        let _ = u!(6);
    }
    if frame_type != 2 {
        let p_sel = u!(2);
        let num_passes = if p_sel < 3 { p_sel + 1 } else { 4 + u!(3) };
        if num_passes > 4 {
            return None;
        }
        if num_passes != 1 {
            let ds_sel = u!(2);
            let num_ds = if ds_sel < 3 { ds_sel } else { 3 + u!(1) };
            for _ in 0..(num_passes - 1) {
                let _ = u!(2);
            }
            for _ in 0..num_ds {
                let _ = u!(2);
            }
            for _ in 0..num_ds {
                if u!(2) == 3 {
                    let _ = u!(3);
                }
            }
        }
    }
    let have_crop = if frame_type == 1 {
        let _ = u!(2);
        false
    } else {
        u!(1) != 0
    };
    if have_crop {
        let n_crop = if frame_type == 2 { 2 } else { 4 };
        for _ in 0..n_crop {
            let s = u!(2);
            let _ = u!([8u32, 11, 14, 30][s as usize]);
        }
    }
    let (is_last, duration, b_mode0) = if frame_type == 0 || frame_type == 3 {
        let mut bm0 = 0u32;
        for i in 0..(1 + num_ec) {
            let b_sel = u!(2);
            let b_mode = if b_sel < 3 { b_sel } else { 3 + u!(2) };
            if b_mode > 4 {
                return None;
            }
            if i == 0 {
                bm0 = b_mode;
            }
            if num_ec > 0 && (b_mode == 2 || b_mode == 3) && u!(2) == 3 {
                let _ = u!(3);
            }
            if (num_ec > 0 && (b_mode == 2 || b_mode == 3)) || b_mode == 4 {
                let _ = u!(1);
            }
            if have_crop || b_mode != 0 {
                let _ = u!(2);
            }
        }
        let dur = if have_anim {
            match u!(2) {
                0 => 0,
                1 => 1,
                2 => u!(8),
                _ => u!(32),
            }
        } else {
            0
        };
        if have_timecode {
            let _ = u!(32);
        }
        (u!(1) != 0, dur, bm0)
    } else {
        (false, 0, 0)
    };
    let save_ref = if frame_type != 1 && !is_last { u!(2) } else { 0 };
    let can_be_ref = !is_last && frame_type != 1 && (duration == 0 || save_ref != 0);
    let save_before_ct_def_false =
        can_be_ref && b_mode0 == 0 && !have_crop && (frame_type == 0 || frame_type == 3);
    if frame_type == 2 || save_before_ct_def_false {
        let _ = u!(1);
    }
    if u!(2) != 0 {
        return None;
    }
    let rf_def = u!(1) != 0;
    if !rf_def {
        let gab = u!(1) != 0;
        if gab && u!(1) != 0 {
            return None;
        }
        let epf = u!(2);
        if epf > 0 {
            if encoding == 0 && u!(1) != 0 {
                return None;
            }
            if u!(1) != 0 || u!(1) != 0 {
                return None;
            }
        }
        if u64_var!() != 0 {
            return None;
        }
    }
    if u64_var!() == 0 {
        let _ = get_slice(b, 0, b.len().min(18));
        with_subformat(Label::Jxl, &JXL_CODESTREAM)
    } else {
        None
    }
}

/// Scanntronik Printfox / Pagefox C64 bitmap (`printfox`, 1986): `'B'` (`320x200`, `8000`B) or `'G'`
/// (`640x400`, `32000`B) type byte at `@0` followed by `0x9B` 16-bit LE RLE stream decoding to exact canvas size.
pub(crate) fn printfox(prefix: &[u8], size: u64) -> Option<Label> {
    static PRINTFOX_B: Subformat = Subformat::new("Printfox bitmap (320x200)");
    static PRINTFOX_G: Subformat = Subformat::new("Printfox/Pagefox bitmap (640x400)");
    require!((64..=40208).contains(&size), prefix.len() >= size as usize);
    let slice = &prefix[..size as usize];
    let rle16 = |start: usize, bounds: &[usize], totals: &[usize]| -> bool {
        if start >= slice.len() || !slice[start..slice.len().min(start + 32)].contains(&0x9B) {
            return false;
        }
        let (mut pos, mut out_b, mut runs, mut hit) = (start, 0usize, 0usize, false);
        while pos < slice.len() {
            if bounds.contains(&out_b) {
                hit = true;
            }
            if slice[pos] == 0x9B {
                if pos + 4 > slice.len() {
                    return false;
                }
                let cnt = u16::from_le_bytes([slice[pos + 1], slice[pos + 2]]) as usize;
                if cnt == 0 {
                    return false;
                }
                let _ = get_slice(prefix, pos, pos + 3);
                out_b = out_b.saturating_add(cnt);
                runs += 1;
                pos += 4;
            } else {
                out_b += 1;
                pos += 1;
            }
            if bounds.contains(&out_b) {
                hit = true;
            }
        }
        runs >= 2 && runs * 50 >= (slice.len() - start) && hit && totals.contains(&out_b)
    };
    let ok = match slice[0] {
        b'B' => (64..=9220).contains(&size) && rle16(1, &[8000, 8192], &[8000, 8192, 9216]),
        b'G' => {
            (64..=40200).contains(&size) && rle16(1, &[32000, 32768], &[32000, 32768, 36096, 40192])
        }
        _ => false,
    };
    require!(ok);
    let _ = get_slice(prefix, 0, 1);
    with_subformat(Label::Printfox, if slice[0] == b'B' { &PRINTFOX_B } else { &PRINTFOX_G })
}

/// Neuroimaging Informatics Technology Initiative NIfTI-1 volume (`nifti`, 2004): `sizeof_hdr == 348`
/// (LE or BE) at `@0..4` paired with `"n+1\0"` or `"ni1\0"` magic at `@344..348`.
pub(crate) fn nifti(prefix: &[u8], size: u64) -> Option<Label> {
    static NIFTI1_SINGLE: Subformat = Subformat::new("NIfTI-1 single file (.nii)");
    static NIFTI1_PAIR: Subformat = Subformat::new("NIfTI-1 header/image pair (.hdr)");
    const NIFTI1_SINGLE_MAGIC: &[u8; 4] = b"n+1\0";
    const NIFTI1_PAIR_MAGIC: &[u8; 4] = b"ni1\0";

    require!(
        size >= 348,
        prefix.len() >= 348,
        (eq_at(prefix, 344, NIFTI1_SINGLE_MAGIC) || eq_at(prefix, 344, NIFTI1_PAIR_MAGIC))
            && (le32(prefix, 0)? == 348 || be32(prefix, 0)? == 348),
    );
    let sub = if eq_at(prefix, 344, NIFTI1_SINGLE_MAGIC) { &NIFTI1_SINGLE } else { &NIFTI1_PAIR };
    with_subformat(Label::Nifti, sub)
}

/// Eastman Kodak Photo CD image (`pcd`, 1992): 2048-byte preamble followed by `"PCD_IPI"` or
/// `"PCD_OPA"` signature at `@2048..2055`.
pub(crate) fn pcd_kodak(prefix: &[u8], size: u64) -> Option<Label> {
    static KODAK_PHOTO_CD: Subformat = Subformat::with_ids("Kodak Photo CD", "fmt/211", None);
    const KODAK_PCD_IPI_MAGIC: &[u8; 7] = b"PCD_IPI";
    const KODAK_PCD_OPA_MAGIC: &[u8; 7] = b"PCD_OPA";

    require!(
        size >= 2055,
        prefix.len() >= 2055,
        has_non_ascii_text_byte(&prefix[..4]),
        eq_at(prefix, 2048, KODAK_PCD_IPI_MAGIC) || eq_at(prefix, 2048, KODAK_PCD_OPA_MAGIC),
    );
    with_subformat(Label::Pcd, &KODAK_PHOTO_CD)
}

/// Point Cloud Library (PCL) Point Cloud Data file (`pcd`, 2010): `"# .PCD v"` or `"VERSION 0.7"`
/// header followed by `\nFIELDS `, `\nPOINTS `, and `\nDATA ` directives.
pub(crate) fn pcd_ascii(prefix: &[u8], size: u64) -> Option<Label> {
    static PCL_POINT_CLOUD: Subformat = Subformat::new("PCL Point Cloud Data");
    const PCD_COMMENT_MAGIC: &[u8] = b"# .PCD v";
    const PCD_VER_DOT7: &[u8] = b"VERSION .7";
    const PCD_VER_0DOT7: &[u8] = b"VERSION 0.7";
    const PCD_FIELDS_LF: &[u8] = b"\nFIELDS ";
    const PCD_POINTS_LF: &[u8] = b"\nPOINTS ";
    const PCD_DATA_LF: &[u8] = b"\nDATA ";

    require!(
        size >= 40,
        prefix.len() >= 40,
        (starts(prefix, PCD_COMMENT_MAGIC)
            || starts(prefix, PCD_VER_DOT7)
            || starts(prefix, PCD_VER_0DOT7))
            && contains(prefix, PCD_FIELDS_LF)
            && contains(prefix, PCD_POINTS_LF)
            && contains(prefix, PCD_DATA_LF),
        is_utf8_text_header(prefix, size as usize),
    );
    with_subformat(Label::Pcd, &PCL_POINT_CLOUD)
}

/// Disambiguates Aldus Placeable and Standard Windows Metafiles (`wmf`).
#[inline]
pub(crate) fn wmf(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| wmf_placeable(prefix, size)).or_else(|| probe(|| wmf_standard(prefix, size)))
}

/// Disambiguates `01 00 00 00` between Enhanced Metafile (`emf`) and Standard Windows Metafile (`wmf`).
#[inline]
pub(crate) fn emf_or_wmf(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| emf(prefix)).or_else(|| probe(|| wmf_standard(prefix, size)))
}

/// ERDAS IMAGINE Hierarchical File Architecture raster (`erdas`, 1991): `"EHFA_HEADER_TAG\0"` at
/// `@0..16` and root dictionary pointer `>= 16` LE at `@16..20`.
pub(crate) fn erdas(prefix: &[u8], size: u64) -> Option<Label> {
    const ERDAS_HFA_MAGIC: &[u8; 16] = b"EHFA_HEADER_TAG\0";

    require!(
        size >= 24,
        prefix.len() >= 20,
        starts(prefix, ERDAS_HFA_MAGIC),
        le32(prefix, 16)? >= 16,
    );
    Some(Label::Erdas)
}

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const COSMIGO_BRC_MAGIC: &[u8; 4] = b"BRCN";
const COSMIGO_STM_MAGIC: &[u8; 4] = b"STMP";
const ECW_WAVELET_MAGIC: &[u8; 4] = hex2bin!("65 02 01 02");
const EXIV2_EXV_MAGIC: &[u8; 7] = b"\xff\x01Exiv2";
const FASTGRAPH_PPR_MAGIC: &[u8; 9] = b"P\0\0P\0\0R\0\0";
const FASTGRAPH_SPR_MAGIC: &[u8; 16] = b"F\0A\0S\0T\0G\0R\0A\0F\0";
const FIGMA_CHUNK0_LEN_OFFSET: usize = 12;
const FIGMA_JAM_MAGIC: &[u8; 8] = b"fig-jam.";
const FIGMA_KIWI_MAGIC: &[u8; 8] = b"fig-kiwi";
const FIGMA_VERSION_OFFSET: usize = 8;
const KODAK_CINEON_MAGIC_BE: &[u8; 4] = be32_bytes!(0x802A_5FD7);
const KRITA_TILE_DATA_HEADER: &[u8; 20] = b"VERSION 2\nTILEWIDTH ";
const KYOCERA_ARECOYK_MAGIC: &[u8; 7] = b"ARECOYK";
const KYOCERA_LATIGID_MAGIC: &[u8; 9] = b"LATIGID N";
const MAGNETIC_SCROLLS_GFX_MAP2_MAGIC: &[u8; 4] = b"MaP2";
const MAGNETIC_SCROLLS_GFX_MAP4_MAGIC: &[u8; 4] = b"MaP4";
const MAGNETIC_SCROLLS_GFX_MAPI_MAGIC: &[u8; 4] = b"MaPi";
const MIN_AEROFLY_TMT_SIZE: u64 = 64;
const MIN_ALCHEMY_GALLERY_SIZE: u64 = 32;
const MIN_ALPHA_BMP_SIZE: u64 = 32;
const MIN_AM6_AMP_SIZE: u64 = 32;
const MIN_ANSI_CSI_ART_SIZE: u64 = 32;
const MIN_AOL_ART_IMAGE_SIZE: u64 = 24;
const MIN_ARCSOFT_PSF_SIZE: u64 = 8;
const MIN_ARM_ASTC_SIZE: u64 = 32;
const MIN_ATARI_CRACKART_SIZE: u64 = 20;
const MIN_ATARI_RIP_SIZE: u64 = 32;
const MIN_BASIS_UNIVERSAL_SIZE: u64 = 100;
const MIN_BNG_SIZE: u64 = 22;
const MIN_BOHEMIA_PAA_SIZE: u64 = 32;
const MIN_BRUNSLI_JPEG_SIZE: u64 = 12;
const MIN_C64_GODOT_SIZE: u64 = 32;
const MIN_CANON_VRD_SIZE: u64 = 64;
const MIN_CLIP_STUDIO_PAINT_SIZE: u64 = 64;
const MIN_COREL_CMX3_SIZE: u64 = 34;
const MIN_COSMIGO_BRC_SIZE: u64 = 12;
const MIN_COSMIGO_STM_SIZE: u64 = 12;
const MIN_CRYENGINE_DDS_MIP6_SIZE: u64 = 64;
const MIN_DDI1_BGRA_SIZE: u64 = 28;
const MIN_DIMPS_EMB_SIZE: u64 = 36;
const MIN_DWIN_DGUS3_ICL_SIZE: u64 = 28;
const MIN_EA_FSH_IMAGE_SIZE: u64 = 24;
const MIN_ECW_WAVELET_SIZE: u64 = 24;
const MIN_ERICSSON_PKM_SIZE: u64 = 24;
const MIN_EXIV2_EXV_SIZE: u64 = 9;
const MIN_EZART_SIZE: u64 = 32;
const MIN_FASTGRAPH_PPR_SIZE: u64 = 27;
const MIN_FASTGRAPH_SPR_SIZE: u64 = 28;
const MIN_FIF_SIZE: u64 = 64;
const MIN_FIGMA_FIG_SIZE: u64 = 32;
const MIN_FLIR_FPF_SIZE: u64 = 892;
const MIN_GEM_VDI_METAFILE_SIZE: u64 = 36;
const MIN_GIANTS_GDM_SIZE: u64 = 64;
const MIN_GIANTS_GRLE_SIZE: u64 = 22;
const MIN_GLIDE_3DF_SIZE: u64 = 32;
const MIN_GRAFX2_PKM_SIZE: u64 = 780;
const MIN_GX1_IMAGE_SIZE: u64 = 32;
const MIN_HEMERA_HTA_SIZE: u64 = 16;
const MIN_HITACHI_HRF_SIZE: u64 = 512;
const MIN_HOMEWORLD_LIF_SIZE: u64 = 32;
const MIN_HSI_JPEG_SIZE: u64 = 64;
const MIN_HSI_RAW_SIZE: u64 = 33;
const MIN_JBIG2_SIZE: u64 = 19;
const MIN_JIF99A_SIZE: u64 = 14;
const MIN_JP2_PGX_SIZE: u64 = 13;
const MIN_KLEI_KTEX_SIZE: u64 = 16;
const MIN_KODAK_CINEON_SIZE: u64 = 1024;
const MIN_KODAK_PHOTO_CD_INFO_PCD_SIZE: u64 = 64;
const MIN_KODI_XBT_SIZE: u64 = 24;
const MIN_KOEI_TECMO_G1T_SIZE: u64 = 32;
const MIN_KRITA_TILE_DATA_SIZE: u64 = 66;
const MIN_KWIKDRAW_SIZE: u64 = 64;
const MIN_KYOCERA_RAW_SIZE: u64 = 156;
const MIN_LURAWAVE_SIZE: u64 = 32;
const MIN_LYTRO_LFP_SIZE: u64 = 64;
const MIN_MAGICK_MIFF_SIZE: u64 = 32;
const MIN_MAGNETIC_SCROLLS_GFX_SIZE: u64 = 16;
const MIN_MM_GEOV1_R565_SIZE: u64 = 28;
const MIN_MOM_GRF_SIZE: u64 = 36;
const MIN_MRC_DENSITY_MAP_SIZE: u64 = 1024;
const MIN_MRSID_SIZE: u64 = 32;
const MIN_MSP_SIZE: u64 = 32;
const MIN_MS_WINDOWS_BOOT_LOGO_LGO_SIZE: u64 = 64;
const MIN_NDS_NITRO_RESOURCE_SIZE: u64 = 24;
const MIN_NINTENDO_BNTX_SIZE: u64 = 64;
const MIN_NINTENDO_WIIU_GTX_SIZE: u64 = 64;
const MIN_NOKIA_NGG_SIZE: u64 = 16;
const MIN_NOKIA_NLM_SIZE: u64 = 16;
const MIN_NOKIA_NPM_SIZE: u64 = 16;
const MIN_OGL_SIZE: u64 = 64;
const MIN_OPENEXR_SIZE: u64 = 8;
const MIN_PABLOPAINT_SIZE: u64 = 48;
const MIN_PDN3_SIZE: u64 = 32;
const MIN_PFM_SIZE: u64 = 16;
const MIN_PGF_IMAGE_SIZE: u64 = 24;
const MIN_PHM_SIZE: u64 = 14;
const MIN_PHOTOSHOP_ABR_SIZE: u64 = 20;
const MIN_PIXINSIGHT_XISF_SIZE: u64 = 64;
const MIN_PM_PICTURE_SIZE: u64 = 28;
const MIN_POPCAP_PAM_SIZE: u64 = 24;
const MIN_POWERVR_PVR_SIZE: u64 = 32;
const MIN_PROSHAPE_SIZE: u64 = 32;
const MIN_PRVW_PRW_SIZE: u64 = 36;
const MIN_PSP_GIM_SIZE: u64 = 32;
const MIN_PTEX_SIZE: u64 = 64;
const MIN_QUICKLINK_QFX_SIZE: u64 = 256;
const MIN_RADIANCE_HDR_SIZE: u64 = 11;
const MIN_RADIOCEF_RCF_SIZE: u64 = 856;
const MIN_RAPISCAN_RCF_SIZE: u64 = 56;
const MIN_RED_THUMB_SIZE: u64 = 12;
const MIN_REKO_CARDSET_SIZE: u64 = 24;
const MIN_RIVE_RIV_SIZE: u64 = 16;
const MIN_RLI_SIZE: u64 = 32;
const MIN_RPGMAKER_XYZ1_SIZE: u64 = 22;
const MIN_SEADOGS_DXT1_TX_SIZE: u64 = 64;
const MIN_SGI_RGB_SIZE: u64 = 512;
const MIN_SIERRA_SCI32_PIC_SIZE: u64 = 64;
const MIN_SIERRA_SCI32_VIEW_SIZE: u64 = 32;
const MIN_SIGMA_X3F_SIZE: u64 = 40;
const MIN_SIMS4_THUMBNAIL_SIZE: u64 = 40;
const MIN_SMOOTHMOVE_PAN_SIZE: u64 = 68;
const MIN_SMPTE_DPX_SIZE: u64 = 1024;
const MIN_SONY_PMP_SIZE: u64 = 132;
const MIN_SPACE_RANGERS_GI_SIZE: u64 = 48;
const MIN_STAD_PAC_SIZE: u64 = 16;
const MIN_STALKER_XRAY_THM_SIZE: u64 = 64;
const MIN_STATISTICA_STG_SIZE: u64 = 64;
const MIN_TENCENT_TAP_SIZE: u64 = 32;
const MIN_UBISOFT_XBT_SIZE: u64 = 64;
const MIN_VALVE_VTF_SIZE: u64 = 64;
const MIN_VCL_METAFILE_SIZE: u64 = 16;
const MIN_XBIN_SIZE: u64 = 13;
const MIN_YGLF_FILE_SIZE: u64 = 28;
const MIN_ZONER_ZMF_SIZE: u64 = 92;
const MRC_MAP_MAGIC_NUL: &[u8; 4] = b"MAP\0";
const MRC_MAP_MAGIC_SPACE: &[u8; 4] = b"MAP ";
const NDS_NSBTX_MAGIC: &[u8; 4] = b"BTX0";
const PM_PICTURE_MAGIC_BE: &[u8; 4] = b"VIEW";
const PM_PICTURE_MAGIC_LE: &[u8; 4] = b"WEIV";
const POWERVR_V2_HEADER_LEN_LE: &[u8; 4] = le32_bytes!(52);
const RADIOCEF_LEAD_BPP_OFFSET: usize = 0x0350;
const RADIOCEF_LEAD_HEIGHT_OFFSET: usize = 0x034E;
const RADIOCEF_LEAD_MAGIC: &[u8; 4] = b"LEAD";
const RADIOCEF_LEAD_MAGIC_OFFSET: usize = 0x033C;
const RADIOCEF_LEAD_OFFSET_FIELD: usize = 0x0280;
const RADIOCEF_LEAD_SIZE_FIELD: usize = 0x0284;
const RADIOCEF_LEAD_VER_OFFSET: usize = 0x0340;
const RADIOCEF_LEAD_WIDTH_OFFSET: usize = 0x034C;
const RADIOCEF_RCF_MAGIC: &[u8; 36] = bytes!(&[35], b"Arquivo de cefalometria do RADIOCEF");
const RADIOCEF_TOTAL_SIZE_FIELD: usize = 0x02A0;
const RIVE_MAGIC: &[u8; 5] = bytes!(b"RIVE", &[0x07]);
const RIVE_MINOR_OFFSET: usize = 5;
const SIERRA_SCI32_PIC_MAGIC: &[u8; 4] = hex2bin!("81 00 FE 02");
const SIERRA_SCI32_VIEW_MAGIC: &[u8; 4] = hex2bin!("80 80 00 00");
const SIMS4_THUMBNAIL_PAYLOAD_SIG: &[u8; 8] = hex2bin!("BE FD 19 2D 47 AA F8 5C");
const SIMS4_THUMBNAIL_SIG: &[u8; 8] = hex2bin!("8E FC 24 48 9B 78 0E 3C");
const SONY_PMP_JPEG_DQT_MAGIC: &[u8; 4] = b"\xff\xd8\xff\xdb";

#[inline]
fn is_valid_rcf_schema_date(d: u32) -> bool {
    let year = d / 10_000;
    let month = (d / 100) % 100;
    let day = d % 100;
    (1995..=2099).contains(&year) && (1..=12).contains(&month) && (1..=31).contains(&day)
}

#[inline]
fn rcf_rot_le32(prefix: &[u8], off: usize) -> Option<u32> {
    let raw = le32(prefix, off)?.to_le_bytes();
    Some(u32::from_le_bytes([
        raw[0].rotate_left(3),
        raw[1].rotate_left(5),
        raw[2].rotate_left(2),
        raw[3].rotate_left(7),
    ]))
}

/// IPACS Aerofly Texture Map (`.tmt`).
pub(crate) fn verify_aerofly_tmt(prefix: &[u8], size: u64) -> Option<Label> {
    const AEROFLY_TMT_MAGIC: u32 = 12345; // @0..4: 0x00003039 magic
    const AEROFLY_TMT_VER: u32 = 3; // @4..8: format version 3
    const PIXEL_DATA_OFF: u64 = 31; // 31-byte header preceding uncompressed raster plane

    require!(size >= MIN_AEROFLY_TMT_SIZE, prefix.len() >= 24);
    let magic = le32(prefix, 0)?;
    let ver = le32(prefix, 4)?;
    let width = le32(prefix, 8)?;
    let height = le32(prefix, 12)?;
    let levels = le32(prefix, 16)?;
    let bpp = le32(prefix, 20)?;
    require!(
        magic == AEROFLY_TMT_MAGIC,
        ver == AEROFLY_TMT_VER,
        (1..=8192).contains(&width),
        (1..=8192).contains(&height),
        (1..=14).contains(&levels),
        matches!(bpp, 16 | 24 | 32)
    );
    let expected_pixels =
        (width as u64).checked_mul(height as u64)?.checked_mul((bpp / 8) as u64)?;
    require!(size == PIXEL_DATA_OFF.checked_add(expected_pixels)?);
    Some(Label::AeroflyTmt)
}

/// Alchemy Mindworks Image Gallery: `"ALCHGLRY"`.
pub(crate) fn verify_alchemy_gallery(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ALCHEMY_GALLERY_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"ALCHGLRY"),
        has_non_ascii_text_byte(prefix.get(8..16)?) || scan_range(prefix, 8, 64, b"Image Gallery")
    );
    Some(Label::AlchGal)
}

/// Alpha Microsystems AMOS Bitmap (`.bmp`).
pub(crate) fn verify_alpha_bmp(prefix: &[u8], size: u64) -> Option<Label> {
    const ALPHA_BMP_MAGIC: &[u8; 4] = bytes!(hex2bin!("FF FF"), be16_bytes!(1)); // @0..4: FF FF 00 01 magic
    const HDR_SIZE_OFFSET: usize = 4; // @4..8: u32 LE header size in bytes (>= 20)
    const COMP_OFFSET: usize = 8; // @8..10: u16 LE compression mode (1..=4)
    const WIDTH_OFFSET: usize = 10; // @10..12: u16 LE width in pixels
    const HEIGHT_OFFSET: usize = 12; // @12..14: u16 LE height in pixels
    const BPP_OFFSET: usize = 14; // @14..16: u16 LE bits per pixel (1, 4, 8, 16, or 24)
    const PLANES_OFFSET: usize = 16; // @16..18: u16 LE color planes (1)

    require!(size >= MIN_ALPHA_BMP_SIZE, prefix.len() >= 20, starts(prefix, ALPHA_BMP_MAGIC));
    let hdr_size = le32(prefix, HDR_SIZE_OFFSET)?;
    let comp = le16(prefix, COMP_OFFSET)?;
    let width = le16(prefix, WIDTH_OFFSET)?;
    let height = le16(prefix, HEIGHT_OFFSET)?;
    let bpp = le16(prefix, BPP_OFFSET)?;
    let planes = le16(prefix, PLANES_OFFSET)?;
    require!(
        (20..=4096).contains(&hdr_size),
        (hdr_size as u64) < size,
        (1..=4).contains(&comp),
        (1..=16_384).contains(&width),
        (1..=16_384).contains(&height),
        matches!(bpp, 1 | 4 | 8 | 16 | 24),
        planes == 1
    );
    Some(Label::AlphaBmp)
}

/// AM6 Wrapped Windows DIB Bitmap (.amp).
pub(crate) fn verify_am6_amp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AM6_AMP_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"AM6\xec\x03\0"),
        le32(prefix, 14) == Some(40),
        le16(prefix, 26) == Some(1)
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::Am6Amp)
}

/// ANSI X3.64 / ECMA-48 CSI Escape BBS Text Art (.ans / .imp).
pub(crate) fn verify_ansi_csi_art(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ANSI_CSI_ART_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"\x1b["),
        get_slice(prefix, 2, prefix.len().min(64))?.windows(2).any(|w| w == b"\x1b[")
    );
    require!(has_non_ascii_text_byte(prefix.get(..prefix.len().min(512))?));
    Some(Label::AnsiCsiArt)
}

/// America Online Johnson-Grace Compressed Image (`.art`, 1994 Johnson-Grace / AOL, PRONOM `fmt/666`).
pub(crate) fn verify_aol_art_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AOL_ART_IMAGE_SIZE
            && prefix.len() >= 16
            && starts(prefix, b"JG")
            && matches!(u8_at(prefix, 2)?, 3 | 4)
            && u8_at(prefix, 3)? == 0x0E
    );
    Some(Label::AolArt)
}

/// ArcSoft PhotoStudio Image (.psf, PRONOM fmt/1832).
pub(crate) fn verify_arcsoft_psf(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_ARCSOFT_PSF_SIZE
        && prefix.len() >= MIN_ARCSOFT_PSF_SIZE as usize
        && starts(prefix, b"FSPA\x03\x00\x00\x00")
    {
        Some(Label::Psf)
    } else {
        None
    }
}

/// ARM Adaptive Scalable Texture Compression Image (.astc).
pub(crate) fn verify_arm_astc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ARM_ASTC_SIZE
            && prefix.len() >= 16
            && starts(prefix, &[0x13, 0xAB, 0xA1, 0x5C])
    );
    let bx = u8_at(prefix, 4)?;
    let by = u8_at(prefix, 5)?;
    let bz = u8_at(prefix, 6)?;
    let xsize = (u8_at(prefix, 7).unwrap_or(0) as u32)
        | ((u8_at(prefix, 8).unwrap_or(0) as u32) << 8)
        | ((u8_at(prefix, 9).unwrap_or(0) as u32) << 16);
    let ysize = (u8_at(prefix, 10).unwrap_or(0) as u32)
        | ((u8_at(prefix, 11).unwrap_or(0) as u32) << 8)
        | ((u8_at(prefix, 12).unwrap_or(0) as u32) << 16);
    let zsize = (u8_at(prefix, 13).unwrap_or(0) as u32)
        | ((u8_at(prefix, 14).unwrap_or(0) as u32) << 8)
        | ((u8_at(prefix, 15).unwrap_or(0) as u32) << 16);
    require!(
        (3..=12).contains(&bx)
            && (3..=12).contains(&by)
            && (1..=6).contains(&bz)
            && xsize > 0
            && ysize > 0
            && zsize > 0
    );
    Some(Label::Astc)
}

/// Atari ST CrackArt Raster Image (.ca1 / .ca2 / .ca3).
pub(crate) fn verify_atari_crackart(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ATARI_CRACKART_SIZE
            && prefix.len() >= 20
            && starts(prefix, b"CA")
            && u8_at(prefix, 2)? <= 1
            && u8_at(prefix, 3)? <= 2
    );
    let compressed = u8_at(prefix, 2)?;
    let res = u8_at(prefix, 3)?;
    let pal_bytes: usize = match res {
        0 => 32, // 16 colors * 2 bytes
        1 => 8,  // 4 colors * 2 bytes
        2 => 0,  // monochrome (no palette)
        _ => return None,
    };
    require!(prefix.len() >= 4 + pal_bytes + 4);
    // Validate Atari ST 9-bit / STE 12-bit RGB palette words (upper 4 bits of each 16-bit BE word must be 0)
    for i in (0..pal_bytes).step_by(2) {
        let w = be16(prefix, 4 + i)?;
        require!((w & 0xF000) == 0);
    }
    if compressed == 0 {
        require!(size == 32004 + (pal_bytes as u64));
    } else {
        let delta = u8_at(prefix, 4 + pal_bytes + 2)?;
        let speed = u8_at(prefix, 4 + pal_bytes + 3)?;
        require!(speed >= 1 && (speed != 0 || delta != 0) && size <= 32768);
    }
    Some(Label::AtariCrackart)
}

/// Atari 8-bit Rocky Interlace Picture (.rip).
pub(crate) fn verify_atari_rocky_interlace(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ATARI_RIP_SIZE
            && prefix.len() >= 20
            && (starts(prefix, b"RIP1") || starts(prefix, b"RIP2"))
            && matches!(u8_at(prefix, 4)?, b'.' | 0)
    );
    let valid_be = (20..=size).contains(&(be16(prefix, 10)? as u64))
        && (1..=320).contains(&be16(prefix, 12)?)
        && (1..=300).contains(&be16(prefix, 14)?);
    let valid_le = (20..=size).contains(&(le16(prefix, 10)? as u64))
        && (16..=320).contains(&le16(prefix, 13)?)
        && (1..=240).contains(&le16(prefix, 15)?);
    require!(valid_be || valid_le);
    Some(Label::RockyRip)
}

/// Binomial Basis Universal Supercompressed GPU Texture (`.basis`, 2019 Binomial).
pub(crate) fn verify_basis_universal(prefix: &[u8], size: u64) -> Option<Label> {
    const VER_OFF: usize = 2; // @2: u16 LE version (0x0010..=0x0020, standard 0x0013)
    const HDR_SIZE_OFF: usize = 4; // @4: u16 LE header size (must be 0x004D = 77)
    const DATA_SIZE_OFF: usize = 8; // @8: u32 LE total payload size after 77B header
    const SLICE_DESCS_OFF: usize = 65; // @65: u32 LE file offset to slice descriptor array

    require!(size >= MIN_BASIS_UNIVERSAL_SIZE && prefix.len() >= 77 && starts(prefix, b"sB"));
    let ver = le16(prefix, VER_OFF)?;
    let hdr_size = le16(prefix, HDR_SIZE_OFF)?;
    let data_size = le32(prefix, DATA_SIZE_OFF)? as u64;
    // 24-bit little-endian counts at @14..17 (`total_slices`) and @17..20 (`total_images`)
    let total_slices = (u8_at(prefix, 14).unwrap_or(0) as u32)
        | ((u8_at(prefix, 15).unwrap_or(0) as u32) << 8)
        | ((u8_at(prefix, 16).unwrap_or(0) as u32) << 16);
    let total_images = (u8_at(prefix, 17).unwrap_or(0) as u32)
        | ((u8_at(prefix, 18).unwrap_or(0) as u32) << 8)
        | ((u8_at(prefix, 19).unwrap_or(0) as u32) << 16);
    let tex_format = u8_at(prefix, 20)?;
    let flags = le16(prefix, 21)?;
    let tex_type = u8_at(prefix, 23)?;
    let slice_ofs = le32(prefix, SLICE_DESCS_OFF)? as u64;

    require!(
        (0x0010..=0x0020).contains(&ver)
            && hdr_size == 77
            && data_size > 0
            && 77u64.saturating_add(data_size) <= size
            && total_slices > 0
            && (1..=total_slices).contains(&total_images)
            && tex_format <= 32
            && ((tex_format == 0) == ((flags & 1) != 0))
            && flags <= 0x01FF
            && tex_type < 5
            && slice_ofs >= 77
            && slice_ofs.saturating_add(23u64.saturating_mul(total_slices as u64)) <= size
    );
    Some(Label::Basis)
}

/// Better than PNG Lossless Image (`.bng`, 2026, Sam H. Smith / Serēnum `BTPNGF` v1):
/// - `@0..8`: `FF 0E 38 30 2A 1A 18 01` (`0xFF` + `"BTPNGF"` + `"1"` in Serēnum `CLEAR` character encoding)
/// - `@8..12`: `u32le` `width >= 1`
/// - `@12..16`: `u32le` `height >= 1` (`width * height <= 2^28`)
/// - `@16..20`: `u32le` `stream_byte_count` (`bit 31` = `RAW` uncompressed planar RGBA flag; `bits 0..30` = `body_length`)
/// - Conformance check: `size >= 20 + body_length`, and `body_length == 4 * width * height` when `RAW` is set or `2 <= body_length <= 4 * width * height` when coded.
pub(crate) fn verify_bng(prefix: &[u8], size: u64) -> Option<Label> {
    const BNG_MAGIC: &[u8; 8] = hex2bin!("FF 0E 38 30 2A 1A 18 01"); // @0..8: 0xFF + "BTPNGF1" in Serēnum CLEAR encoding
    const BNG_WIDTH_OFFSET: usize = 8; // @8..12: u32le image width in pixels (>= 1)
    const BNG_HEIGHT_OFFSET: usize = 12; // @12..16: u32le image height in pixels (>= 1)
    const BNG_STREAM_COUNT_OFFSET: usize = 16; // @16..20: u32le stream_byte_count (bit 31 = RAW flag, bits 0..30 = body_length)
    const BNG_HEADER_SIZE: usize = 20; // @0..20: fixed 20-byte BNG v1 container header
    const BNG_RAW_FLAG_MASK: u32 = 0x8000_0000; // bit 31: uncompressed planar RGBA mode
    const BNG_BODY_LEN_MASK: u32 = 0x7FFF_FFFF; // bits 0..30: encoded or raw body byte count
    const BNG_MAX_PIXELS: u64 = 1 << 28; // reference decoder pixel budget (268,435,456 pixels)

    require!(size >= MIN_BNG_SIZE, prefix.len() >= BNG_HEADER_SIZE, starts(prefix, BNG_MAGIC));
    let width = le32(prefix, BNG_WIDTH_OFFSET)? as u64;
    let height = le32(prefix, BNG_HEIGHT_OFFSET)? as u64;
    let stream_byte_count = le32(prefix, BNG_STREAM_COUNT_OFFSET)?;
    let pixel_count = width.checked_mul(height)?;
    let raw_bytes = pixel_count.checked_mul(4)?;
    let is_raw = (stream_byte_count & BNG_RAW_FLAG_MASK) != 0;
    let body_len = (stream_byte_count & BNG_BODY_LEN_MASK) as u64;

    require!(
        width >= 1,
        height >= 1,
        pixel_count <= BNG_MAX_PIXELS,
        size >= (BNG_HEADER_SIZE as u64).saturating_add(body_len),
        if is_raw { body_len == raw_bytes } else { body_len >= 2 && body_len <= raw_bytes }
    );
    Some(Label::Bng)
}

/// Bohemia Interactive Real Virtuality PAA/PAC Texture (`.paa`/`.pac`, 2001 Bohemia Interactive).
pub(crate) fn verify_bohemia_paa(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BOHEMIA_PAA_SIZE,
        prefix.len() >= 16,
        matches!(
            get_slice(prefix, 0, 2)?,
            [0x01, 0xFF]
                | [0x02, 0xFF]
                | [0x03, 0xFF]
                | [0x04, 0xFF]
                | [0x05, 0xFF]
                | [0x44, 0x44]
                | [0x55, 0x15]
                | [0x80, 0x80]
                | [0x88, 0x88]
        ),
        eq_at(prefix, 2, b"GGAT"),
        matches!(get_slice(prefix, 6, 10)?, b"CGVA" | b"CXAM" | b"GALF" | b"SFFO" | b"ZIWS"),
        (4..=64).contains(&le32(prefix, 10)?)
    );
    Some(Label::BohemiaPaa)
}

/// Google Brunsli Lossless Repacked JPEG (`.brn`, 2019 Google, `0A 04 42 D2 D5 4E 12`).
pub(crate) fn verify_brunsli_jpeg(prefix: &[u8], size: u64) -> Option<Label> {
    const BRUNSLI_MAGIC: &[u8; 7] = hex2bin!("0A 04 42 D2 D5 4E 12"); // @0..7
    require!(size >= MIN_BRUNSLI_JPEG_SIZE, prefix.len() >= 8, starts(prefix, BRUNSLI_MAGIC));
    Some(Label::Brunsli)
}

/// Commodore 64 GoDot Image / Clip (`"GOD0"` / `"GOD1"`).
pub(crate) fn verify_c64_godot_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_C64_GODOT_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"GOD0") || starts(prefix, b"GOD1"),
        has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    Some(Label::C64Godot)
}

/// Canon Digital Photo Professional Recipe Data file (`.vrd` v1–v3 or `.dr4` v4).
///
/// Header layouts:
/// - `.vrd` (DPP 1.x–3.x): `@0..28 == "CANON OPTIONAL DATA\0\x00\x01\x00\x00..."` followed by `0xFFFF00F4` (`EditData`) / `0xFFFF00F6` (`XMP`) / `0xFFFF00F7` (`Edit4Data`) section markers.
/// - `.dr4` (DPP 4.x): `@0..4 == "IIII"` (little-endian), `@4..8 == \x04\x00\x04\x00` or `\x05\x00\x04\x00`, `@12..16 == 28` (`0x1C` header length), `@20..24 == 0` (`u32 LE`), `@28..32 == u32 LE entry_count` (`1..=4096`).
pub(crate) fn verify_canon_vrd(prefix: &[u8], size: u64) -> Option<Label> {
    const VRD_MAGIC: &[u8; 24] = bytes!(b"CANON OPTIONAL DATA", &[0x00], be32_bytes!(0x0001_0000)); // @0..24: .vrd header
    const DR4_V4_MAGIC: &[u8; 8] = bytes!(b"IIII", le16_bytes!(4), le16_bytes!(4)); // @0..8: .dr4 v4 header
    const DR4_V5_MAGIC: &[u8; 8] = bytes!(b"IIII", le16_bytes!(5), le16_bytes!(4)); // @0..8: .dr4 v5 header

    require!(size >= MIN_CANON_VRD_SIZE && prefix.len() >= 36);
    if starts(prefix, VRD_MAGIC) {
        let first_block_tag = be32(prefix, 28)?;
        require!(matches!(first_block_tag, 0xFFFF_00F4..=0xFFFF_00F7));
        return Some(Label::CanonVrd);
    }
    if starts(prefix, DR4_V4_MAGIC) || starts(prefix, DR4_V5_MAGIC) {
        let hdr_len = le32(prefix, 12)?;
        let count = le32(prefix, 28)?;
        let first_val_off = le32(prefix, 56).unwrap_or(0);
        let valid_layout = (hdr_len == 28 && le32(prefix, 20) == Some(0))
            || first_val_off == 36u32.saturating_add(count.saturating_mul(28));
        require!((1..=4096).contains(&count), valid_layout);
        return Some(Label::CanonVrd);
    }
    None
}

/// Celsys Clip Studio Paint illustration document (`.clip`, `"CSFCHUNK"`).
///
/// Header layout:
/// - `@0..8`: `"CSFCHUNK"` (`8B` container signature)
/// - `@8..16`: `u64 BE total_size` (`>= 64`)
/// - `@16..24`: `u64 BE first_chunk_offset` (`== 24`)
/// - `@24..32`: `"CHNKHead"` (`8B` header chunk signature)
pub(crate) fn verify_clip_studio_paint(prefix: &[u8], size: u64) -> Option<Label> {
    const CSFCHUNK_MAGIC: &[u8; 8] = b"CSFCHUNK"; // @0..8: container signature
    const TOTAL_SIZE_OFFSET: usize = 8; // @8..16: u64 BE total file size
    const FIRST_CHUNK_OFFSET: usize = 16; // @16..24: u64 BE offset to CHNKHead (24)
    const CHNKHEAD_OFFSET: usize = 24; // @24..32: "CHNKHead" first chunk signature

    require!(
        size >= MIN_CLIP_STUDIO_PAINT_SIZE,
        prefix.len() >= 40,
        starts(prefix, CSFCHUNK_MAGIC),
        be64(prefix, TOTAL_SIZE_OFFSET)? >= MIN_CLIP_STUDIO_PAINT_SIZE,
        be64(prefix, FIRST_CHUNK_OFFSET) == Some(24),
        eq_at(prefix, CHNKHEAD_OFFSET, b"CHNKHead")
    );
    Some(Label::ClipStudioPaint)
}

/// Corel Presentation Exchange Metafile (CMX).
pub(crate) fn verify_corel_cmx3(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_COREL_CMX3_SIZE, prefix.len() >= 34, starts(prefix, b"CMX3"));
    let is_trid_cmx3 = le32(prefix, 4) == Some(27)
        && le32(prefix, 8) == Some(22)
        && get_slice(prefix, 12, 34)? == b"Corel Binary Meta File";
    let has_corel_id = memchr::memmem::find(
        get_slice(prefix, 4, prefix.len().min(64))?,
        b"Corel Binary Meta File",
    )
    .is_some();
    require!(is_trid_cmx3 || has_corel_id);
    Some(Label::Cmx)
}

/// Cosmigo Pro Motion / Pro Motion NG Brush Container (`.brc`, `~pmotion.brc`).
pub(crate) fn verify_cosmigo_brc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_COSMIGO_BRC_SIZE,
        prefix.len() >= 12,
        starts(prefix, COSMIGO_BRC_MAGIC),
        le32(prefix, 4) == Some(12),
        le32(prefix, 8) == Some(0)
    );
    if size > 12 {
        require!(prefix.len() >= 20);
        let chunk_tag_ok = eq_at(prefix, 12, b"BCI2")
            || eq_at(prefix, 12, b"BCIA")
            || eq_at(prefix, 12, b"BCA3")
            || eq_at(prefix, 12, b"BCAA")
            || eq_at(prefix, 12, b"BCIM");
        let chunk_len = le32(prefix, 16)?;
        require!(chunk_tag_ok, chunk_len > 0);
    }
    Some(Label::CosmigoBrc)
}

/// Cosmigo Pro Motion / Pro Motion NG Simple Tile Map (`.stm`).
pub(crate) fn verify_cosmigo_stm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_COSMIGO_STM_SIZE, prefix.len() >= 12, starts(prefix, COSMIGO_STM_MAGIC));
    let width = le16(prefix, 4)? as u64;
    let height = le16(prefix, 6)? as u64;
    let _tile0 = le16(prefix, 8)?;
    let flip_x = u8_at(prefix, 10)?;
    let flip_y = u8_at(prefix, 11)?;
    let expected_size = 8u64.checked_add(width.checked_mul(height)?.checked_mul(4)?)?;
    require!(
        (1..=16384).contains(&width),
        (1..=16384).contains(&height),
        flip_x <= 1,
        flip_y <= 1,
        size == expected_size,
        has_non_ascii_text_byte(prefix.get(..12)?)
    );
    Some(Label::CosmigoStm)
}

/// CryEngine Split DDS Mipmap Stream (.dds.6).
pub(crate) fn verify_cryengine_dds_mip6(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CRYENGINE_DDS_MIP6_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 8)? == hex2bin!("FF FF 00 00 00 00 00 00"),
        get_slice(prefix, 10, 24)? == hex2bin!("00 00 00 00 00 00 FF FF 00 00 00 00 00 00"),
        get_slice(prefix, 26, 32)? == hex2bin!("00 00 00 00 00 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::CryengineDdsMip6)
}

/// DDI1 Uncompressed 32-Bit BGRA Bitmap (.bgra).
pub(crate) fn verify_ddi1_bgra(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DDI1_BGRA_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"DDI1"),
        (1..=8192).contains(&le32(prefix, 4).unwrap_or(0)),
        (1..=8192).contains(&le32(prefix, 8).unwrap_or(0))
    );
    Some(Label::Ddi1Bgra)
}

/// Dimps #EMB Texture/Resource Container (.emb, 2008) Dimps.
pub(crate) fn verify_dimps_emb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DIMPS_EMB_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"#EMB\xfe\xff \0\x01\0\x01\0"),
        le32(prefix, 24) == Some(0x20)
    );
    Some(Label::DimpsEmb)
}

/// DWIN Technology DGUS II/III T5L HMI Icon Library (.icl, 2017) DWIN Technology.
pub(crate) fn verify_dwin_dgus3_icl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DWIN_DGUS3_ICL_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"DGUS_3"),
        get_slice(prefix, 12, 14)? == b"\x04\0",
        (1..=65535).contains(&le16(prefix, 14).unwrap_or(0))
    );
    Some(Label::DwinDgus3Icl)
}

/// Electronic Arts FSH / SHPI Shape Image container (`.fsh` / `.ssh`, 1994 Electronic Arts).
pub(crate) fn verify_ea_fsh_image(prefix: &[u8], size: u64) -> Option<Label> {
    const TOTAL_SIZE_OFFSET: usize = 4; // @4..8: u32 LE total file size
    const COUNT_OFFSET: usize = 8; // @8..12: u32 LE number of shape entries
    const DIR_ID_RANGE: core::ops::Range<usize> = 12..16; // @12..16: 4-byte ASCII directory ID ("GIMX", "G356", "G264", etc.)

    require!(
        size >= MIN_EA_FSH_IMAGE_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"SHPI")
            || starts(prefix, b"SHPP")
            || starts(prefix, b"SHPS")
            || starts(prefix, b"SHPM")
    );
    let total_sz = le32(prefix, TOTAL_SIZE_OFFSET)? as u64;
    let count = le32(prefix, COUNT_OFFSET)? as u64;
    require!(
        (total_sz..=total_sz + 64).contains(&size),
        (1..=4096).contains(&count),
        16 + count * 8 <= size,
        get_slice(prefix, DIR_ID_RANGE.start, DIR_ID_RANGE.end)?
            .iter()
            .all(|&b| b == 0 || (0x20..=0x7E).contains(&b))
    );
    Some(Label::EaFsh)
}

/// Enhanced Compression Wavelet Geospatial Image (`.ecw`, 1998) Earth Resource Mapping (`65 02 01 02`).
///
/// Header layout:
/// - `@0..4`: `65 02 01 02` (`'e'` + version `2` + cell type + blocking)
/// - `@4`: `u8` `n_levels` (`1..=31`)
/// - `@5`: `u8` `n_subbands` (`1..=16`)
/// - `@6..10`: `u32 BE` `width` (`1..=0x1000_0000`)
/// - `@10..14`: `u32 BE` `height` (`1..=0x1000_0000`)
/// - `@14..16`: `u16 BE` `n_bands` (`1..=256`)
/// - `@16..18`: `u16 BE` `color_space` (`<= 16`)
/// - `@18..20`: `u16 BE` `x_block_size` (power of 2 in `16..=4096`)
/// - `@20..22`: `u16 BE` `y_block_size` (power of 2 in `16..=4096`)
pub(crate) fn verify_ecw_wavelet(prefix: &[u8], size: u64) -> Option<Label> {
    const N_LEVELS_OFFSET: usize = 4; // @4: u8 wavelet decomposition levels (1..=31)
    const N_SUBBANDS_OFFSET: usize = 5; // @5: u8 wavelet subbands per level (1..=16)
    const WIDTH_OFFSET: usize = 6; // @6..10: u32 BE raster width in pixels
    const HEIGHT_OFFSET: usize = 10; // @10..14: u32 BE raster height in pixels
    const N_BANDS_OFFSET: usize = 14; // @14..16: u16 BE spectral band count (1..=256)
    const COLOR_SPACE_OFFSET: usize = 16; // @16..18: u16 BE color space enum (<= 16)
    const X_BLOCK_OFFSET: usize = 18; // @18..20: u16 BE horizontal block size (power of 2, 16..=4096)
    const Y_BLOCK_OFFSET: usize = 20; // @20..22: u16 BE vertical block size (power of 2, 16..=4096)
    const HEADER_LEN: usize = 22;

    require!(
        size >= MIN_ECW_WAVELET_SIZE,
        prefix.len() >= HEADER_LEN,
        starts(prefix, ECW_WAVELET_MAGIC)
    );
    let n_levels = u8_at(prefix, N_LEVELS_OFFSET)?;
    let n_subbands = u8_at(prefix, N_SUBBANDS_OFFSET)?;
    let width = be32(prefix, WIDTH_OFFSET)?;
    let height = be32(prefix, HEIGHT_OFFSET)?;
    let n_bands = be16(prefix, N_BANDS_OFFSET)?;
    let color_space = be16(prefix, COLOR_SPACE_OFFSET)?;
    let x_blk = be16(prefix, X_BLOCK_OFFSET)?;
    let y_blk = be16(prefix, Y_BLOCK_OFFSET)?;

    require!(
        (1..=31).contains(&n_levels),
        (1..=16).contains(&n_subbands),
        (1..=0x1000_0000).contains(&width),
        (1..=0x1000_0000).contains(&height),
        (1..=256).contains(&n_bands),
        color_space <= 16,
        x_blk.is_power_of_two() && (16..=4096).contains(&x_blk),
        y_blk.is_power_of_two() && (16..=4096).contains(&y_blk)
    );
    Some(Label::Ecw)
}

/// Ericsson Texture Compression ETC1/ETC2 PKM Container (.pkm).
pub(crate) fn verify_ericsson_pkm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ERICSSON_PKM_SIZE
            && prefix.len() >= 16
            && starts(prefix, b"PKM ")
            && matches!(get_slice(prefix, 4, 6)?, b"10" | b"20")
            && be16(prefix, 6).unwrap_or(99) <= 16
            && be16(prefix, 8).unwrap_or(0) > 0
            && be16(prefix, 10).unwrap_or(0) > 0
            && be16(prefix, 12).unwrap_or(0) > 0
            && be16(prefix, 14).unwrap_or(0) > 0
    );
    Some(Label::Pkm)
}

/// Exiv2 Metadata Sidecar Container (`.exv`, 2004 Andreas Huggel).
///
/// Header layout:
/// - `@0..7`: `"\xFF\x01Exiv2"` (`FF 01 45 78 69 76 32`)
/// - `@7..9`: JPEG marker (`FF E0`..`FF EF` APPn, `FF FE` COM, `FF DB` DQT, or `FF D9` EOI).
pub(crate) fn verify_exiv2_exv(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EXIV2_EXV_SIZE,
        prefix.len() >= 9,
        starts(prefix, EXIV2_EXV_MAGIC),
        u8_at(prefix, 7)? == 0xFF,
        matches!(u8_at(prefix, 8)?, 0xD8..=0xFE)
    );
    Some(Label::Exiv2Exv)
}

/// EZ-Art Professional `.eza` (v2 container): `"EZA_2.0\0"`.
pub(crate) fn verify_ezart(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_EZART_SIZE, starts(prefix, b"EZA_2.0\0"));
    Some(Label::Ezart)
}

/// Fastgraph Packed Pixel Run / Young Picasso Bitmap (`.yp` / `.ppr`, 1991) Ted Gruber Software / Blueview Software.
pub(crate) fn verify_fastgraph_ppr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FASTGRAPH_PPR_SIZE,
        prefix.len() >= 27,
        starts(prefix, FASTGRAPH_PPR_MAGIC),
        u8_at(prefix, 10)? == 0,
        u8_at(prefix, 11)? == 0,
        u8_at(prefix, 13)? == 0,
        u8_at(prefix, 14)? == 0,
        u8_at(prefix, 16)? == 0,
        u8_at(prefix, 17)? == 0,
        u8_at(prefix, 19)? == 0,
        u8_at(prefix, 20)? == 0,
        u8_at(prefix, 21)? == 4,
        u8_at(prefix, 22)? == 0,
        u8_at(prefix, 23)? == 0,
        u8_at(prefix, 25).unwrap_or(0) != 0 || u8_at(prefix, 26).unwrap_or(0) != 0
    );
    let width = 256u32 * (u8_at(prefix, 9)? as u32) + (u8_at(prefix, 12)? as u32);
    let height = 256u32 * (u8_at(prefix, 15)? as u32) + (u8_at(prefix, 18)? as u32);
    require!(
        (1..=4096).contains(&width),
        (1..=4096).contains(&height),
        has_non_ascii_text_byte(prefix.get(..27)?)
    );
    Some(Label::FastgraphPpr)
}

/// Fastgraph Standard Pixel Run Bitmap (`.prf` / `.spr`, 1991) Ted Gruber Software.
pub(crate) fn verify_fastgraph_spr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FASTGRAPH_SPR_SIZE,
        prefix.len() >= 28,
        starts(prefix, FASTGRAPH_SPR_MAGIC),
        u8_at(prefix, 17)? == 0,
        u8_at(prefix, 19)? == 0,
        u8_at(prefix, 21)? == 0,
        u8_at(prefix, 23)? == 0,
        u8_at(prefix, 24)? == 0,
        u8_at(prefix, 25)? == 0,
        u8_at(prefix, 27)? > 0
    );
    let width = 1u32 + (u8_at(prefix, 16)? as u32) + 256u32 * (u8_at(prefix, 18)? as u32);
    let height = 1u32 + (u8_at(prefix, 20)? as u32) + 256u32 * (u8_at(prefix, 22)? as u32);
    require!(
        (1..=4096).contains(&width),
        (1..=4096).contains(&height),
        has_non_ascii_text_byte(prefix.get(..28)?)
    );
    Some(Label::FastgraphSpr)
}

/// FBI CJIS Wavelet Scalar Quantization fingerprint image (`.wsq`).
pub(crate) fn verify_fbi_wsq(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 16, prefix.len() >= 8, starts(prefix, hex2bin!("FF A0")));
    let m2 = be16(prefix, 2)?;
    require!(matches!(m2, 0xFFA8 | 0xFFA4 | 0xFFA5 | 0xFFA6 | 0xFFA2 | 0xFFA3));
    let seg_len = be16(prefix, 4)? as usize;
    let next_off = 4usize.checked_add(seg_len)?;
    require!(seg_len >= 2, next_off + 2 <= prefix.len());
    let m3 = be16(prefix, next_off)?;
    require!(matches!(m3, 0xFFA1..=0xFFA8));
    Some(Label::Wsq)
}

/// Figma Canvas Document (`.fig`).
pub(crate) fn verify_figma_fig(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FIGMA_FIG_SIZE,
        prefix.len() >= 20,
        starts(prefix, FIGMA_KIWI_MAGIC) || starts(prefix, FIGMA_JAM_MAGIC),
        (1..=1000).contains(&le32(prefix, FIGMA_VERSION_OFFSET)?),
        le32(prefix, FIGMA_CHUNK0_LEN_OFFSET)? >= 4
    );
    Some(Label::FigmaFig)
}

/// Teledyne FLIR Public Image Format thermal radiometric image (`.fpf`).
///
/// Header layout (`FPF_HEADER_T`, 892 bytes):
/// - `@0..24`: `"FPF Public Image Format\0"` (`24B` `fpf_id` NUL-terminated string)
/// - `@32..36`: `u32 LE version` (`1` or `2`, or BE)
/// - `@36..40`: `u32 LE image_data_offset` (`== 892` = `0x0000037C`, or BE)
/// - `@40..42`: `u16 LE image_type` (`0..=8`)
/// - `@42..44`: `u16 LE pixel_format` (`0..=5`, e.g., `2` = 4-byte `f32` Kelvin temperatures)
/// - `@44..46`: `u16 LE x_size` (`> 0`)
/// - `@46..48`: `u16 LE y_size` (`> 0`)
pub(crate) fn verify_flir_fpf(prefix: &[u8], size: u64) -> Option<Label> {
    const FPF_MAGIC: &[u8; 24] = b"FPF Public Image Format\0"; // @0..24: fpf_id
    const VERSION_OFFSET: usize = 32; // @32..36: u32 version (1 or 2)
    const DATA_OFF_OFFSET: usize = 36; // @36..40: u32 image_data_offset (892)
    const WIDTH_OFFSET: usize = 44; // @44..46: u16 x_size (> 0)
    const HEIGHT_OFFSET: usize = 46; // @46..48: u16 y_size (> 0)

    require!(size >= MIN_FLIR_FPF_SIZE, prefix.len() >= 48, starts(prefix, FPF_MAGIC));
    let ver_le = le32(prefix, VERSION_OFFSET)?;
    let ver_be = be32(prefix, VERSION_OFFSET)?;
    let off_le = le32(prefix, DATA_OFF_OFFSET)?;
    let off_be = be32(prefix, DATA_OFF_OFFSET)?;
    let valid_le = matches!(ver_le, 1 | 2)
        && off_le == 892
        && le16(prefix, WIDTH_OFFSET)? > 0
        && le16(prefix, HEIGHT_OFFSET)? > 0;
    let valid_be = matches!(ver_be, 1 | 2)
        && off_be == 892
        && be16(prefix, WIDTH_OFFSET)? > 0
        && be16(prefix, HEIGHT_OFFSET)? > 0;
    require!(valid_le || valid_be);
    Some(Label::FlirFpf)
}

/// France Logiciels Thomson compressed image (`FLOGICIELS OU IMAGES COD`).
pub(crate) fn verify_flogiciels(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 32,
        prefix.len() >= 32,
        starts(prefix, b"FLOGICIELS OU IMAGES COD"),
        has_non_ascii_text_byte(prefix.get(24..prefix.len().min(128))?),
    );
    Some(Label::Flog)
}

/// Iterated Systems FIF: `"FIF\x01"`, `u32 LE width` @6 and `u32 LE height` @10 (1..=16384), and two
/// NUL-terminated 3-letter colour-space tags at `@20` / `@24`.
pub(crate) fn verify_fractal_image(prefix: &[u8], size: u64) -> Option<Label> {
    const WIDTH: usize = 6; // @6: u32 LE
    const HEIGHT: usize = 10; // @10: u32 LE
    const COLOR_TAGS: core::ops::Range<usize> = 20..28; // @20: "EUV\0" "BGR\0"
    require!(size >= MIN_FIF_SIZE, prefix.len() >= COLOR_TAGS.end, starts(prefix, b"FIF\x01"));
    require!(
        (1..=16384).contains(&le32(prefix, WIDTH)?),
        (1..=16384).contains(&le32(prefix, HEIGHT)?)
    );
    let tags = get_slice(prefix, COLOR_TAGS.start, COLOR_TAGS.end)?;
    require!(tags[0..3].iter().all(u8::is_ascii_uppercase), tags[3] == 0);
    require!(tags[4..7].iter().all(u8::is_ascii_uppercase), tags[7] == 0);
    Some(Label::Fif)
}

/// Digital Research GEM VDI Metafile (`.gem`, 1985 Digital Research, PRONOM `fmt/542` & `fmt/543`).
pub(crate) fn verify_gem_vdi_metafile(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GEM_VDI_METAFILE_SIZE
            && prefix.len() >= 36
            && u8_at(prefix, 0)? == 0xFF
            && u8_at(prefix, 1)? == 0xFF
    );
    let hdr_words_le = le16(prefix, 2)?;
    let hdr_words_be = be16(prefix, 2)?;
    let (is_le, hdr_words) = if (14..=128).contains(&hdr_words_le) {
        (true, hdr_words_le as usize)
    } else if (14..=128).contains(&hdr_words_be) {
        (false, hdr_words_be as usize)
    } else {
        return None;
    };

    let read16 =
        |b: &[u8], off: usize| -> Option<u16> { if is_le { le16(b, off) } else { be16(b, off) } };
    let version = read16(prefix, 4)?;
    let coord_type = read16(prefix, 6)?;
    require!(version <= 500 && matches!(coord_type, 0 | 2));

    let cmd_off = hdr_words * 2;
    require!((cmd_off as u64) + 8 <= size);
    if cmd_off + 8 <= prefix.len() {
        let opcode = read16(prefix, cmd_off)?;
        let ptsin = read16(prefix, cmd_off + 2)? as u64;
        let intin = read16(prefix, cmd_off + 4)? as u64;
        let subopcode = read16(prefix, cmd_off + 6)?;
        require!(
            ((1..=205).contains(&opcode) || opcode == 0xFFFF)
                && ptsin <= 4096
                && intin <= 4096
                && subopcode <= 256
                && (cmd_off as u64)
                    .saturating_add(8)
                    .saturating_add(ptsin * 4)
                    .saturating_add(intin * 2)
                    <= size
        );
    }
    Some(Label::GemVdi)
}

/// GIANTS Software Farming Simulator GDM Foliage Density Map (`.gdm`, `"!MDF"`/`"\"MDF"`, 2016 GIANTS Software).
pub(crate) fn verify_giants_gdm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GIANTS_GDM_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"!MDF") || starts(prefix, b"\"MDF"),
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::GiantsGdm)
}

/// GIANTS Software Farming Simulator GRLE RLE Map (`.grle`, `"GRLE"`, 2011 GIANTS Software).
///
/// 21-byte header layout:
/// - `@0..4`: `"GRLE"`
/// - `@4`: `u8 version` (`1..=4`)
/// - `@5..9`: `u32 LE width` (`1..=65536`)
/// - `@9..13`: `u32 LE height` (`1..=65536`)
/// - `@13..17`: `u32 LE num_channels` (`1..=16`)
/// - `@17..21`: `u32 LE data_size` (`21 + data_size == size`)
pub(crate) fn verify_giants_grle(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GIANTS_GRLE_SIZE,
        prefix.len() >= 21,
        starts(prefix, b"GRLE"),
        (1..=4).contains(&u8_at(prefix, 4)?),
        (1..=65536).contains(&le32(prefix, 5)?),
        (1..=65536).contains(&le32(prefix, 9)?),
        (1..=16).contains(&le32(prefix, 13)?),
        21u64.saturating_add(le32(prefix, 17)? as u64) == size
    );
    Some(Label::GiantsGrle)
}

/// 3dfx Glide Texture (`.3df`, `"3df v"`).
pub(crate) fn verify_glide_3dfx_texture(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GLIDE_3DF_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"3df v1."),
        u8_at(prefix, 7)?.is_ascii_digit(),
        matches!(u8_at(prefix, 8)?, b'\n' | b'\r')
    );
    Some(Label::Glide3df)
}

/// Sunset Design GrafX2 PKM Paletted Image (`.pkm`, `"PKM\0"`).
pub(crate) fn verify_grafx2_pkm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GRAFX2_PKM_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"PKM\0"),
        (u8_at(prefix, 4)? == 0
            && u8_at(prefix, 5)? <= 1
            && be16(prefix, 6)? > 0
            && be16(prefix, 8)? > 0)
            || (u8_at(prefix, 4).unwrap_or(0) != u8_at(prefix, 5).unwrap_or(0)
                && (1..=16384).contains(&le16(prefix, 6)?)
                && (1..=16384).contains(&le16(prefix, 8)?)
                && get_slice(prefix, 10, prefix.len().min(778))?.iter().all(|&b| b <= 0x3F))
    );
    Some(Label::Grafx2Pkm)
}

/// Show Partner GX1 CGA BSAVE Screen Image (`.gx1`, `FA 04 30` + `(FF 00)*8`).
pub(crate) fn verify_gx1_image(prefix: &[u8], size: u64) -> Option<Label> {
    const GX1_HEADER: &[u8; 19] =
        hex2bin!("FA 04 30 FF 00 FF 00 FF 00 FF 00 FF 00 FF 00 FF 00 FF 00");
    require!(size >= MIN_GX1_IMAGE_SIZE, prefix.len() >= 20, starts(prefix, GX1_HEADER));
    Some(Label::Gx1)
}

/// Hemera Thumbnails Archive: PNG-style `"\x89HTA\r\n\x1A"` signature.
pub(crate) fn verify_hemera_hta(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_HEMERA_HTA_SIZE, starts(prefix, b"\x89HTA\r\n\x1A"));
    Some(Label::HemeraHta)
}

/// Hitachi CADCore / Tracer Raster Format Image (`.hrf`).
pub(crate) fn verify_hitachi_hrf(prefix: &[u8], size: u64) -> Option<Label> {
    const HRF_MAGIC: &[u8; 16] = b"CADC/KR RST     "; // @0..16: 16-byte Hitachi raster signature
    const VERSION_OFFSET: usize = 16; // @16..20: 4-character ASCII version (e.g. "0300")
    const DATE_SLASH1_OFFSET: usize = 22; // @22: '/' separator in MM/DD/YY date string
    const DATE_SLASH2_OFFSET: usize = 25; // @25: '/' separator in MM/DD/YY date string
    const RESERVED_OFFSET: usize = 28; // @28..32: 4 reserved zero bytes

    require!(
        size >= MIN_HITACHI_HRF_SIZE,
        prefix.len() >= 64,
        starts(prefix, HRF_MAGIC),
        get_slice(prefix, VERSION_OFFSET, VERSION_OFFSET + 4)?.iter().all(|&b| b.is_ascii_digit()),
        u8_at(prefix, DATE_SLASH1_OFFSET) == Some(b'/'),
        u8_at(prefix, DATE_SLASH2_OFFSET) == Some(b'/'),
        le32(prefix, RESERVED_OFFSET) == Some(0)
    );
    Some(Label::HitachiHrf)
}

/// Relic Entertainment Homeworld Library Image Format Texture (`.lif`).
pub(crate) fn verify_homeworld_lif_texture(prefix: &[u8], size: u64) -> Option<Label> {
    const LIF_MAGIC: &[u8; 12] = bytes!(b"Willy 7", &[0x00], le32_bytes!(0x0104)); // @0..12: "Willy 7\0" + version 0x0104 LE
    const WIDTH_OFFSET: usize = 16; // @16..20: u32 LE texture width
    const HEIGHT_OFFSET: usize = 20; // @20..24: u32 LE texture height

    require!(
        size >= MIN_HOMEWORLD_LIF_SIZE,
        prefix.len() >= 24,
        starts(prefix, LIF_MAGIC),
        (1..=16_384).contains(&le32(prefix, WIDTH_OFFSET)?),
        (1..=16_384).contains(&le32(prefix, HEIGHT_OFFSET)?)
    );
    Some(Label::HomeworldLif)
}

/// Handmade Software Image Alchemy HSI Proprietary JPEG (`.jpg` / `.hsi`).
pub(crate) fn verify_hsi_jpeg(prefix: &[u8], size: u64) -> Option<Label> {
    const HSI_JPEG_MAGIC: &[u8; 6] = bytes!(b"hsi1", be16_bytes!(0)); // @0..6: "hsi1\0\0" signature
    const WIDTH_OFFSET: usize = 6; // @6..8: u16 BE width in pixels
    const HEIGHT_OFFSET: usize = 8; // @8..10: u16 BE height in pixels

    require!(
        size >= MIN_HSI_JPEG_SIZE,
        prefix.len() >= 16,
        starts(prefix, HSI_JPEG_MAGIC),
        (1..=16_384).contains(&be16(prefix, WIDTH_OFFSET)?),
        (1..=16_384).contains(&be16(prefix, HEIGHT_OFFSET)?)
    );
    Some(Label::HsiJpeg)
}

/// Handmade Software Image Alchemy HSI Raw Bitmap (`.raw`).
pub(crate) fn verify_hsi_raw(prefix: &[u8], size: u64) -> Option<Label> {
    const HSI_RAW_MAGIC: &[u8; 8] = bytes!(b"mhwanh", be16_bytes!(4)); // @0..8: "mhwanh\0\x04" signature + version 4 BE
    const WIDTH_OFFSET: usize = 8; // @8..10: u16 BE width in pixels
    const HEIGHT_OFFSET: usize = 10; // @10..12: u16 BE height in pixels
    const PAL_COLORS_OFFSET: usize = 12; // @12..14: u16 BE palette color count (0..=256)
    const RESERVED_OFFSET: usize = 20; // @20..32: 12 reserved zero bytes

    require!(
        size >= MIN_HSI_RAW_SIZE,
        prefix.len() >= 32,
        starts(prefix, HSI_RAW_MAGIC),
        (1..=16_384).contains(&be16(prefix, WIDTH_OFFSET)?),
        (1..=16_384).contains(&be16(prefix, HEIGHT_OFFSET)?),
        be16(prefix, PAL_COLORS_OFFSET)? <= 256,
        eq_at(prefix, RESERVED_OFFSET, &[0u8; 12])
    );
    Some(Label::HsiRaw)
}

/// ITU-T T.88 / ISO/IEC 14492 JBIG2 Bi-Level Image (.jb2 / .jbig2).
pub(crate) fn verify_jbig2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JBIG2_SIZE
            && prefix.len() >= 13
            && starts(prefix, b"\x97JB2\r\n\x1A\n")
            && (u8_at(prefix, 8)? & 0xFC) == 0
    );
    Some(Label::Jbig2)
}

/// Jeff's Image Format (`.jif`, `"JIF99a"`).
pub(crate) fn verify_jeffs_image_format(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JIF99A_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"JIF99a"),
        le16(prefix, 6)? > 0,
        le16(prefix, 8)? > 0,
        has_non_ascii_text_byte(prefix.get(6..12)?)
    );
    Some(Label::Jif99a)
}

/// ISO/IEC 15444-4 JPEG 2000 PGX verification component image (`.pgx`, `"PG ML "` / `"PG LM "`).
pub(crate) fn verify_jp2_pgx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JP2_PGX_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"PG ML ") || starts(prefix, b"PG LM ")
    );
    let max_hdr = prefix.len().min(64);
    let rel_nl = get_slice(prefix, 6, max_hdr)?.iter().position(|&b| b == b'\n')?;
    let nl_pos = 6 + rel_nl;
    let mut hdr_slice = get_slice(prefix, 6, nl_pos)?;
    if hdr_slice.ends_with(b"\r") {
        hdr_slice = &hdr_slice[..hdr_slice.len() - 1];
    }
    while let Some((&first, rest)) = hdr_slice.split_first() {
        if first == b' ' || first == b'\t' {
            hdr_slice = rest;
        } else {
            break;
        }
    }
    if let Some((&sign, rest)) = hdr_slice.split_first()
        && (sign == b'+' || sign == b'-')
    {
        hdr_slice = rest;
    }
    let hdr_str = core::str::from_utf8(hdr_slice).ok()?;
    let mut parts = hdr_str.split_ascii_whitespace();
    let bitdepth: u64 = parts.next()?.parse().ok()?;
    let width: u64 = parts.next()?.parse().ok()?;
    let height: u64 = parts.next()?.parse().ok()?;
    require!(
        parts.next().is_none(),
        (1..=38).contains(&bitdepth),
        (1..=65535).contains(&width),
        (1..=65535).contains(&height)
    );
    let bytes_per_sample = bitdepth.div_ceil(8);
    let payload_bytes = width.checked_mul(height)?.checked_mul(bytes_per_sample)?;
    let min_total = (nl_pos as u64 + 1).checked_add(payload_bytes)?;
    require!(size >= min_total);
    Some(Label::Jp2Pgx)
}

/// ISO/IEC 21122-1 JPEG XS raw codestream (`FF 10 FF 50`).
pub(crate) fn verify_jxs_codestream(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 16, prefix.len() >= 8, starts(prefix, hex2bin!("FF 10 FF 50")));
    let lcap = be16(prefix, 4)? as usize;
    require!(lcap >= 2);
    let pih_off = 4usize.checked_add(lcap)?;
    if pih_off + 2 <= prefix.len() {
        require!(be16(prefix, pih_off)? == 0xFF51);
    }
    Some(Label::Jxs)
}

/// Klei Entertainment KTEX Game Texture (`.tex`, Don't Starve / Oxygen Not Included, 2013 Klei Entertainment).
pub(crate) fn verify_klei_ktex(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KLEI_KTEX_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"KTEX"),
        (1..=16384).contains(&le16(prefix, 8)?),
        (1..=16384).contains(&le16(prefix, 10)?),
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::KleiKtex)
}

/// Kodak Cineon Digital Film Image (.cin).
pub(crate) fn verify_kodak_cineon(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KODAK_CINEON_SIZE
            && prefix.len() >= 26
            && (get_slice(prefix, 0, 4)? == KODAK_CINEON_MAGIC_BE
                || get_slice(prefix, 0, 4)? == [0xD7, 0x5F, 0x2A, 0x80])
    );
    let data_offset = if get_slice(prefix, 0, 4)? == KODAK_CINEON_MAGIC_BE {
        be32(prefix, 4).unwrap_or(0)
    } else {
        le32(prefix, 4).unwrap_or(0)
    };
    // Cineon v4.5 File Information Header:
    // @0..4 magic, @4..8 image_data_offset (>= 712), @8..12 generic_hdr_len,
    // @12..16 industry_hdr_len, @16..20 variable_hdr_len, @20..24 file_size,
    // @24..32 version string (e.g. "V4.5\0\0\0\0" or "V4.5    ").
    require!(
        data_offset >= 712 && u8_at(prefix, 24)? == b'V' && u8_at(prefix, 25)?.is_ascii_digit()
    );
    Some(Label::Cineon)
}

/// Eastman Kodak Photo CD Disc Information (`INFO.PCD`, 1992) Kodak.
pub(crate) fn verify_kodak_photo_cd_info_pcd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KODAK_PHOTO_CD_INFO_PCD_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"PHOTO_CD\x00"),
        get_slice(prefix, 10, 22)?.iter().all(|&b| b.is_ascii_digit() || b == b' ')
    );
    Some(Label::KodakPhotoCdInfoPcd)
}

/// XBMC / Kodi Texture Bundle (`.xbt`, `"XBTF"`).
pub(crate) fn verify_kodi_xbt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KODI_XBT_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"XBTF2"),
        (1..=65535).contains(&le32(prefix, 5)?)
    );
    Some(Label::KodiXbt)
}

/// Koei Tecmo G1T Texture Archive (`.g1t`, 2011 Koei Tecmo, `"GT1G"` LE or `"G1TG"` BE).
///
/// Header layout:
/// - `@0..4`: `"GT1G"` (LE) or `"G1TG"` (BE)
/// - `@4..8`: 4 ASCII digits version string (e.g., `"0600"`, `"0500"`)
/// - `@8..12`: `u32` `file_size` (`>= 32`)
/// - `@12..16`: `u32` `data_offset` (`0x1C..=0x10000`)
/// - `@16..20`: `u32` `tex_count` (`1..=65536`)
/// - `@20..24`: `u32` `platform_id` (`<= 0x20`)
pub(crate) fn verify_koei_tecmo_g1t(prefix: &[u8], size: u64) -> Option<Label> {
    const FILE_SIZE_OFF: usize = 8; // @8..12: u32 total file size (>= 32)
    const DATA_OFF_FIELD: usize = 12; // @12..16: u32 texture table offset
    const TEX_COUNT_OFF: usize = 16; // @16..20: u32 texture count (1..=65536)
    const PLATFORM_OFF: usize = 20; // @20..24: u32 target platform id (<= 0x20)

    require!(size >= MIN_KOEI_TECMO_G1T_SIZE, prefix.len() >= 24);
    let is_be = if starts(prefix, b"GT1G") {
        false
    } else if starts(prefix, b"G1TG") {
        true
    } else {
        return None;
    };
    let read32 = |off| if is_be { be32(prefix, off) } else { le32(prefix, off) };
    let file_size = read32(FILE_SIZE_OFF)?;
    let data_offset = read32(DATA_OFF_FIELD)?;
    let tex_count = read32(TEX_COUNT_OFF)?;
    let platform_id = read32(PLATFORM_OFF)?;
    require!(
        get_slice(prefix, 4, 8)?.iter().all(|b| b.is_ascii_digit()),
        file_size >= 32,
        (0x1C..=0x10000).contains(&data_offset),
        (1..=65536).contains(&tex_count),
        platform_id <= 0x20
    );
    Some(Label::KoeiTecmoG1t)
}

/// KDE Krita Paint Device Layer Tile Data Stream (`layers/layerN` / `.paintdevice`).
pub(crate) fn verify_krita_tile_data(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KRITA_TILE_DATA_SIZE,
        prefix.len() >= 66,
        starts(prefix, KRITA_TILE_DATA_HEADER),
        scan_range(prefix, 20, 48, b"\nTILEHEIGHT "),
        scan_range(prefix, 32, 64, b"\nPIXELSIZE "),
        scan_range(prefix, 44, 80, b"\nDATA ")
    );
    let lzf_pos = find_pos(prefix.get(..prefix.len().min(96))?, b",LZF,")?;
    let after_lzf = lzf_pos + 5;
    let rel_nl = prefix.get(after_lzf..prefix.len().min(112))?.iter().position(|&b| b == b'\n')?;
    let bin_off = after_lzf + rel_nl + 1;
    let hdr = get_slice(prefix, 0, bin_off)?;
    require!(
        rel_nl >= 1,
        hdr.iter().all(|&b| b.is_ascii_graphic() || matches!(b, b' ' | b'\n' | b'\r')),
        u8_at(prefix, bin_off)? <= 1,
        has_non_ascii_text_byte(prefix.get(..bin_off + 1)?)
    );
    Some(Label::KritaTileData)
}

/// KwikDraw: banner `"KwikDraw - An Object Oriented Drawing Program"`.
pub(crate) fn verify_kwikdraw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KWIKDRAW_SIZE,
        starts(prefix, b"KwikDraw - An Object Oriented Drawing Program")
            || (starts(prefix, b"KwikDraw - ")
                && has_non_ascii_text_byte(prefix.get(11..prefix.len().min(64))?))
    );
    Some(Label::Kwikdraw)
}

/// Kyocera Contax N Digital RAW Image (`.raw`, 2002 Kyocera).
///
/// Header layout:
/// - `@5..10`: `" .reV"` (byte-reversed `"Ver. "` inside 4-byte big-endian swapped groups)
/// - `@12..21`: `"LATIGID N"` (byte-reversed `"N DIGITAL"`)
/// - `@25..32`: `"ARECOYK"` (byte-reversed `"KYOCERA"`).
pub(crate) fn verify_kyocera_raw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KYOCERA_RAW_SIZE,
        prefix.len() >= 52,
        eq_at(prefix, 12, KYOCERA_LATIGID_MAGIC),
        eq_at(prefix, 25, KYOCERA_ARECOYK_MAGIC)
    );
    Some(Label::KyoceraRaw)
}

/// LuraTech LuraWave Wavelet Bitmap (`.lwf`, `57 56 F2 00` or `57 56 02 00 47 45 00 0E`).
pub(crate) fn verify_lurawave_lwf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LURAWAVE_SIZE,
        prefix.len() >= 16,
        starts(prefix, bytes!(b"WV", &[0xF2, 0x00]))
            || (starts(prefix, bytes!(b"WV", &[0x02, 0x00]))
                && eq_at(prefix, 4, bytes!(b"GE", &[0x00, 0x0E]))),
        get_slice(prefix, 4, 16)?.iter().any(|&b| b != 0)
    );
    Some(Label::LurawaveLwf)
}

/// Lytro Light Field Picture container (`.lfp` / `.lfr`).
///
/// Header layout:
/// - `@0..16`: `\x89LFP\r\n\x1A\n\x00\x00\x00\x01\x00\x00\x00\x00` (`16B` file header with version 1 BE and length 0)
/// - `@16..24`: `\x89LFM\r\n\x1A\n` (metadata section) or `\x89LFC\r\n\x1A\n` (component section)
pub(crate) fn verify_lytro_lfp(prefix: &[u8], size: u64) -> Option<Label> {
    const LFP_HEADER: &[u8; 16] =
        bytes!(&[0x89], b"LFP\r\n", &[0x1A], b"\n", be32_bytes!(1), be32_bytes!(0)); // @0..16: LFP header
    const SECTION_MAGIC_OFFSET: usize = 16; // @16..20: "\x89LFM" or "\x89LFC"

    require!(
        size >= MIN_LYTRO_LFP_SIZE,
        prefix.len() >= 28,
        starts(prefix, LFP_HEADER),
        eq_at(prefix, SECTION_MAGIC_OFFSET, b"\x89LFM\r\n\x1A\n")
            || eq_at(prefix, SECTION_MAGIC_OFFSET, b"\x89LFC\r\n\x1A\n")
    );
    Some(Label::LytroLfp)
}

/// ImageMagick Machine Independent File Format (.miff).
pub(crate) fn verify_magick_miff(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MAGICK_MIFF_SIZE
            && prefix.len() >= 20
            && (starts(prefix, b"id=ImageMagick")
                || memchr::memmem::find(prefix, b"id=ImageMagick").is_some())
            && (scan_range(prefix, 14, 512, b"columns=")
                || scan_range(prefix, 14, 512, b"class=")
                || prefix.get(14..prefix.len().min(32))?.contains(&0))
    );
    Some(Label::Miff)
}

/// Magnetic Scrolls picture container (`.gfx`, 1985 Magnetic Scrolls / Niclas Karlsson):
/// - `"MaPi"`: `u32 BE` total size @4 (`16..0x0100_0000`) + `u32 BE` picture offsets @8 (`< size`)
/// - `"MaP2"`: `u16 BE` directory byte size @4 (non-zero multiple of 16-byte entries)
/// - `"MaP4"`: `u16 LE` directory entry count @4 (`1..=512`, 14-byte entries)
pub(crate) fn verify_magnetic_scrolls_gfx(prefix: &[u8], size: u64) -> Option<Label> {
    const SIZE_OR_DIR: usize = 4; // @4: total size (MaPi) / directory size (MaP2) / count (MaP4)
    const FIRST_PICTURE: usize = 8; // @8: MaPi first picture offset
    const MAP2_ENTRY_SIZE: u16 = 16;
    require!(size >= MIN_MAGNETIC_SCROLLS_GFX_SIZE, prefix.len() >= 12);
    if starts(prefix, MAGNETIC_SCROLLS_GFX_MAPI_MAGIC) {
        let total = be32(prefix, SIZE_OR_DIR)?;
        require!((16..0x0100_0000).contains(&total), be32(prefix, FIRST_PICTURE)? < total);
    } else if starts(prefix, MAGNETIC_SCROLLS_GFX_MAP2_MAGIC) {
        let dir = be16(prefix, SIZE_OR_DIR)?;
        require!(dir != 0, dir % MAP2_ENTRY_SIZE == 0, has_non_ascii_text_byte(prefix.get(..12)?));
    } else if starts(prefix, MAGNETIC_SCROLLS_GFX_MAP4_MAGIC) {
        require!((1..=512).contains(&le16(prefix, SIZE_OR_DIR)?));
    } else {
        return None;
    }
    Some(Label::MagneticScrollsGfx)
}

/// Magneti Marelli RT6/SMEG Automotive Navigation GEOV1 RGB565 Image (.r565) Magneti Marelli.
pub(crate) fn verify_mm_geov1_r565(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MM_GEOV1_R565_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"GEOV1"),
        (1..=4096).contains(&le16(prefix, 5).unwrap_or(0)),
        (1..=4096).contains(&le16(prefix, 7).unwrap_or(0)),
        le32(prefix, 9) == Some(0),
        le16(prefix, 13) == le16(prefix, 7)
    );
    Some(Label::MmGeov1R565)
}

/// Simtex Master of Magic Graphics Resource (.grf, 1994) Simtex / MicroProse.
pub(crate) fn verify_mom_grf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MOM_GRF_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"Master of Magic\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"),
        le32(prefix, 30).unwrap_or(0) > 0
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::MomGrf)
}

/// MRC / CCP4 3D Electron Microscopy & Crystallography Density Map (`.mrc` / `.map` / `.ccp4`, 1982 MRC LMB / CCP4).
///
/// Header layout (1024-byte header):
/// - `@0..12`: `u32 LE/BE NX, NY, NZ` (`> 0`, grid dimensions)
/// - `@12..16`: `u32 LE/BE MODE` (`0..=12` or `101`: `0` = int8, `1` = int16, `2` = float32, `3` = complex int16, `4` = complex float32, `6` = uint16, `12` = float16, `101` = 4-bit packed)
/// - `@64..76`: `u32 LE/BE MAPC, MAPR, MAPS` (axis mapping permutation of `{1, 2, 3}`)
/// - `@208..212`: `"MAP "` or `"MAP\0"`
/// - `@212..214`: Machine stamp (`44 44` or `44 41` little-endian, `11 11` or `22 22` big-endian).
pub(crate) fn verify_mrc_density_map(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MRC_DENSITY_MAP_SIZE,
        prefix.len() >= 224,
        eq_at(prefix, 208, MRC_MAP_MAGIC_SPACE) || eq_at(prefix, 208, MRC_MAP_MAGIC_NUL)
    );
    let mach = get_slice(prefix, 212, 214)?;
    let be = match mach {
        [0x44, 0x44] | [0x44, 0x41] => false,
        [0x11, 0x11] | [0x22, 0x22] => true,
        _ => return None,
    };
    let r32 = |off: usize| if be { be32(prefix, off) } else { le32(prefix, off) };
    let nx = r32(0)?;
    let ny = r32(4)?;
    let nz = r32(8)?;
    let mode = r32(12)?;
    let mapc = r32(64)?;
    let mapr = r32(68)?;
    let maps = r32(72)?;
    require!(
        (1..=0x00FF_FFFF).contains(&nx),
        (1..=0x00FF_FFFF).contains(&ny),
        (1..=0x00FF_FFFF).contains(&nz),
        matches!(mode, 0..=12 | 101),
        (1..=3).contains(&mapc),
        (1..=3).contains(&mapr),
        (1..=3).contains(&maps),
        mapc + mapr + maps == 6
    );
    Some(Label::MrcMap)
}

/// LizardTech MrSID Multiresolution Seamless Image (.sid).
pub(crate) fn verify_mrsid(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_MRSID_SIZE && prefix.len() >= 16);
    let is_msid = starts(prefix, b"msid") || starts(prefix, b"MSID");
    require!(is_msid);

    // Gen 2/3/4 MrSID format versions:
    // In Gen 2/3: bytes @4..6 are version (e.g. 2, 3, or 4 in BE or LE)
    let ver = be16(prefix, 4)?;
    let ver_le = le16(prefix, 4)?;
    require!(
        (1..=10).contains(&ver)
            || (1..=10).contains(&ver_le)
            || has_non_ascii_text_byte(prefix.get(4..16)?)
    );

    Some(Label::Sid)
}

/// Microsoft Windows 3.x EGA/VGA Startup Logo Code Stub (`.lgo`, 1990) Microsoft.
pub(crate) fn verify_ms_windows_boot_logo_lgo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MS_WINDOWS_BOOT_LOGO_LGO_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"LOGO\xE9")
    );
    Some(Label::MsWindowsBootLogoLgo)
}

/// Microsoft Paint v1/v2 bitmap (.msp).
pub(crate) fn verify_msp(prefix: &[u8], size: u64) -> Option<Label> {
    const WIDTH_OFF: usize = 4; // @4: u16 LE bitmap width in pixels (> 0)
    const HEIGHT_OFF: usize = 6; // @6: u16 LE bitmap height in pixels (> 0)
    const CKSUM_OFF: usize = 24; // @24: u16 LE XOR checksum of the preceding 12 words (@0..24)
    const PAD_OFF: usize = 26; // @26..32: 3 reserved padding words (must be 0x0000)

    require!(size >= MIN_MSP_SIZE, prefix.len() >= 32);
    let w = le16(prefix, WIDTH_OFF)?;
    let h = le16(prefix, HEIGHT_OFF)?;
    require!(w > 0, h > 0);
    let cksum = le16(prefix, CKSUM_OFF)?;
    let mut xor13 = 0u16;
    for i in 0..13 {
        xor13 ^= le16(prefix, i * 2)?;
    }
    let cksum_valid = (xor13 == 0 && (cksum != 0 || eq_at(prefix, PAD_OFF, &[0u8; 6])))
        || (cksum == 0 && eq_at(prefix, 8, &[0u8; 16]) && eq_at(prefix, PAD_OFF, &[0u8; 6]));
    require!(cksum_valid);
    Some(Label::MspPaint)
}

/// Nintendo DS Nitro SDK 3D Binary Texture (`.nsbtx`, `"BTX0"` + `"TEX0"`).
pub(crate) fn verify_nds_nsbtx(prefix: &[u8], size: u64) -> Option<Label> {
    const BOM_LE: &[u8; 2] = hex2bin!("FF FE"); // @4..6: little-endian BOM
    const TEX0_MAGIC: &[u8; 4] = b"TEX0"; // @block0_off: Nitro 3D texture chunk tag
    const BOM_OFF: usize = 4;
    const FILE_SIZE_OFF: usize = 8;
    const HDR_SIZE_OFF: usize = 12;
    const BLOCK_COUNT_OFF: usize = 14;
    const BLOCK0_PTR_OFF: usize = 16;

    require!(
        size >= MIN_NDS_NITRO_RESOURCE_SIZE,
        prefix.len() >= 24,
        starts(prefix, NDS_NSBTX_MAGIC),
        eq_at(prefix, BOM_OFF, BOM_LE),
        le16(prefix, HDR_SIZE_OFF)? == 16
    );
    let blocks = le16(prefix, BLOCK_COUNT_OFF)? as usize;
    let decl_size = le32(prefix, FILE_SIZE_OFF)? as u64;
    let block0_off = le32(prefix, BLOCK0_PTR_OFF)? as usize;
    let expected_off = 16 + blocks * 4;
    require!(
        (1..=16).contains(&blocks),
        decl_size >= 24,
        decl_size <= size,
        block0_off == expected_off,
        eq_at(prefix, block0_off, TEX0_MAGIC)
    );
    Some(Label::NdsNsbtx)
}

/// Nintendo Switch Binary Texture Container (`.bntx`).
pub(crate) fn verify_nintendo_bntx_texture(prefix: &[u8], size: u64) -> Option<Label> {
    const BNTX_MAGIC: &[u8; 8] = bytes!(b"BNTX", le32_bytes!(0)); // @0..8: "BNTX\0\0\0\0"
    const BOM_OFFSET: usize = 12; // @12..14: u16 byte-order mark (0xFEFF or 0xFFFE)
    const TARGET_TAG_OFFSET: usize = 32; // @32..36: target platform/block tag ("NX  " or "BRTI")

    require!(
        size >= MIN_NINTENDO_BNTX_SIZE,
        prefix.len() >= 36,
        starts(prefix, BNTX_MAGIC),
        matches!(le16(prefix, BOM_OFFSET), Some(0xFEFF | 0xFFFE)),
        eq_at(prefix, TARGET_TAG_OFFSET, b"NX  ") || eq_at(prefix, TARGET_TAG_OFFSET, b"BRTI")
    );
    Some(Label::NintendoBntx)
}

/// Nintendo Wii U GX2 Texture Container (`.gtx`, 2012 Nintendo, `"Gfx2"` + `"BLK{"`).
///
/// Header layout:
/// - `@0..4`: `"Gfx2"`
/// - `@4..8`: `u32 BE` `header_size` (`0x20`)
/// - `@8..12`: `u32 BE` `major_version` (`6` or `7`)
/// - `@12..16`: `u32 BE` `minor_version` (`<= 4`)
/// - `@16..20`: `u32 BE` `gpu_version` (`2` = Latte / Cafe)
/// - `@20..24`: `u32 BE` `align_mode` (`0..=1`)
/// - `@32..36`: `"BLK{"` first GX2 block magic
/// - `@36..40`: `u32 BE` `blk_header_size` (`0x20`)
/// - `@48..52`: `u32 BE` `block_type` (`1..=0x16`)
pub(crate) fn verify_nintendo_wiiu_gtx(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_SIZE_OFF: usize = 4; // @4..8: u32 BE header size (0x20)
    const MAJOR_OFF: usize = 8; // @8..12: u32 BE major version (6..=7)
    const MINOR_OFF: usize = 12; // @12..16: u32 BE minor version (<= 4)
    const GPU_VER_OFF: usize = 16; // @16..20: u32 BE GPU version (2)
    const ALIGN_OFF: usize = 20; // @20..24: u32 BE alignment mode (0..=1)
    const BLK_OFF: usize = 32; // @32..36: "BLK{" first block header magic
    const BLK_HDR_SIZE_OFF: usize = 36; // @36..40: u32 BE block header size (0x20)
    const BLK_TYPE_OFF: usize = 48; // @48..52: u32 BE GX2 block type (1..=0x16)

    require!(
        size >= MIN_NINTENDO_WIIU_GTX_SIZE,
        prefix.len() >= 52,
        starts(prefix, b"Gfx2"),
        be32(prefix, HDR_SIZE_OFF) == Some(0x20),
        matches!(be32(prefix, MAJOR_OFF), Some(6 | 7)),
        be32(prefix, MINOR_OFF)? <= 4,
        be32(prefix, GPU_VER_OFF) == Some(2),
        be32(prefix, ALIGN_OFF)? <= 1,
        eq_at(prefix, BLK_OFF, b"BLK{"),
        be32(prefix, BLK_HDR_SIZE_OFF) == Some(0x20),
        (1..=0x16).contains(&be32(prefix, BLK_TYPE_OFF)?)
    );
    Some(Label::NintendoWiiuGtx)
}

/// Nokia Group Graphics Image (`.ngg`, `"NGG\0\x01\0"`).
pub(crate) fn verify_nokia_ngg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NOKIA_NGG_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"NGG\0\x01\0"),
        (1..=255).contains(&le16(prefix, 6)?),
        (1..=255).contains(&le16(prefix, 8)?)
    );
    Some(Label::NokiaNgg)
}

/// Nokia Logo Manager Image (`.nlm`, `"NLM \x01"`).
pub(crate) fn verify_nokia_nlm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NOKIA_NLM_SIZE,
        prefix.len() >= 10,
        starts(prefix, b"NLM \x01"),
        u8_at(prefix, 5)? <= 3,
        ((1..=255).contains(&le16(prefix, 6)?) && (1..=255).contains(&le16(prefix, 8)?))
            || (u8_at(prefix, 7).unwrap_or(0) >= 1
                && u8_at(prefix, 8).unwrap_or(0) >= 1
                && (1..=8).contains(&u8_at(prefix, 9).unwrap_or(0)))
    );
    Some(Label::NokiaNlm)
}

/// Nokia Picture Message Image (`.npm`, `"NPM\0"`).
pub(crate) fn verify_nokia_npm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NOKIA_NPM_SIZE,
        prefix.len() >= 10,
        starts(prefix, b"NPM\0"),
        u8_at(prefix, 4)? <= 128
    );
    Some(Label::NokiaNpm)
}

/// ObjectGraphics OGL: `"OGL_FILE\0"`.
pub(crate) fn verify_objectgraphics_ogl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_OGL_SIZE, starts(prefix, b"OGL_FILE\0"));
    Some(Label::Ogl)
}

/// ILM OpenEXR high dynamic range image (.exr).
pub(crate) fn verify_openexr(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_OPENEXR_SIZE
        && prefix.len() >= MIN_OPENEXR_SIZE as usize
        && starts(prefix, bytes!(b"v/1", &[1]))
        && u8_at(prefix, 4)? == 2
    {
        Some(Label::Exr)
    } else {
        None
    }
}

/// PabloPaint: id line `"PABLO PACKED PICTURE: Groupe CDND \r\n"`.
pub(crate) fn verify_pablopaint(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PABLOPAINT_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"PABLO PACKED PICTURE:"),
        has_non_ascii_text_byte(prefix.get(21..prefix.len().min(48))?)
    );
    Some(Label::Pablopaint)
}

/// Paint.NET Layered Raster Image (.pdn).
pub(crate) fn verify_pdn3(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PDN3_SIZE
            && prefix.len() >= 20
            && starts(prefix, b"PDN3")
            && has_non_ascii_text_byte(prefix.get(4..7)?)
    );
    let xml_len = (u8_at(prefix, 4).unwrap_or(0) as u64)
        | ((u8_at(prefix, 5).unwrap_or(0) as u64) << 8)
        | ((u8_at(prefix, 6).unwrap_or(0) as u64) << 16);
    require!(
        (15..=0x00FF_FFFF).contains(&xml_len)
            && 7 + xml_len < size
            && (starts(prefix.get(7..)?, b"<pdnImage")
                || starts(prefix.get(7..)?, b"\xEF\xBB\xBF<pdnImage")
                || starts(prefix.get(7..)?, b"<?xml"))
    );
    Some(Label::Pdn)
}

/// Portable FloatMap HDR Raster Image (.pfm).
pub(crate) fn verify_pfm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PFM_SIZE
            && prefix.len() >= 12
            && (starts(prefix, b"PF") || starts(prefix, b"Pf"))
            && matches!(u8_at(prefix, 2)?, b'\n' | b'\r' | b' ')
    );
    let mut idx = 2;
    for _ in 0..2 {
        while idx < prefix.len() {
            if matches!(u8_at(prefix, idx)?, b' ' | b'\n' | b'\r' | b'\t') {
                idx += 1;
            } else if u8_at(prefix, idx)? == b'#' {
                while idx < prefix.len()
                    && u8_at(prefix, idx)? != b'\n'
                    && u8_at(prefix, idx)? != b'\r'
                {
                    idx += 1;
                }
            } else {
                break;
            }
        }
        let start = idx;
        let mut val: u32 = 0;
        while idx < prefix.len() && u8_at(prefix, idx)?.is_ascii_digit() && idx - start < 8 {
            val = val.saturating_mul(10).saturating_add((u8_at(prefix, idx)? - b'0') as u32);
            idx += 1;
        }
        require!(
            idx > start
                && val > 0
                && idx < prefix.len()
                && matches!(u8_at(prefix, idx)?, b' ' | b'\n' | b'\r' | b'\t')
        );
    }
    // Parse third token: non-zero floating-point scale/endianness factor (e.g. "-1.0" or "1.0") followed by newline
    while idx < prefix.len() && matches!(u8_at(prefix, idx)?, b' ' | b'\n' | b'\r' | b'\t') {
        idx += 1;
    }
    let scale_start = idx;
    if idx < prefix.len() && matches!(u8_at(prefix, idx)?, b'-' | b'+') {
        idx += 1;
    }
    let digit_start = idx;
    while idx < prefix.len()
        && (u8_at(prefix, idx).unwrap_or(0).is_ascii_digit()
            || u8_at(prefix, idx).unwrap_or(0) == b'.')
        && idx - scale_start < 16
    {
        idx += 1;
    }
    require!(
        idx > digit_start
            && idx < prefix.len()
            && matches!(u8_at(prefix, idx)?, b'\n' | b'\r' | b' ')
    );
    Some(Label::Pfm)
}

/// Portable HalfMap (`.phm`, 2015) IEEE 754 `f16` HDR raster header and geometry:
/// - `@0..2`: `"PH"` (3-channel RGB `f16`) or `"Ph"` (1-channel grayscale `f16`)
/// - `@2`: whitespace (`\n`, `\r`, or `' '`)
/// - Positive ASCII decimal `width` and `height` tokens (with optional `#` comment lines)
/// - Non-zero floating-point scale/endianness token (e.g. `"-1.0"` or `"1.0"`) followed by whitespace.
pub(crate) fn verify_phm(prefix: &[u8], size: u64) -> Option<Label> {
    const PHM_RGB_MAGIC: &[u8; 2] = b"PH"; // @0..2: 3-channel RGB IEEE 754 binary16 half-float
    const PHM_GRAY_MAGIC: &[u8; 2] = b"Ph"; // @0..2: 1-channel grayscale IEEE 754 binary16 half-float
    const MIN_PHM_HEADER_LEN: usize = 10; // @0..10: minimum ASCII header length before raster

    require!(
        size >= MIN_PHM_SIZE,
        prefix.len() >= MIN_PHM_HEADER_LEN,
        starts(prefix, PHM_RGB_MAGIC) || starts(prefix, PHM_GRAY_MAGIC),
        matches!(u8_at(prefix, 2)?, b'\n' | b'\r' | b' ')
    );
    let mut idx = 2;
    for _ in 0..2 {
        while idx < prefix.len() {
            if matches!(u8_at(prefix, idx)?, b' ' | b'\n' | b'\r' | b'\t') {
                idx += 1;
            } else if u8_at(prefix, idx)? == b'#' {
                while idx < prefix.len()
                    && u8_at(prefix, idx)? != b'\n'
                    && u8_at(prefix, idx)? != b'\r'
                {
                    idx += 1;
                }
            } else {
                break;
            }
        }
        let start = idx;
        let mut val: u32 = 0;
        while idx < prefix.len() && u8_at(prefix, idx)?.is_ascii_digit() && idx - start < 8 {
            val = val.saturating_mul(10).saturating_add((u8_at(prefix, idx)? - b'0') as u32);
            idx += 1;
        }
        require!(
            idx > start
                && val > 0
                && idx < prefix.len()
                && matches!(u8_at(prefix, idx)?, b' ' | b'\n' | b'\r' | b'\t')
        );
    }
    while idx < prefix.len() && matches!(u8_at(prefix, idx)?, b' ' | b'\n' | b'\r' | b'\t') {
        idx += 1;
    }
    let scale_start = idx;
    if idx < prefix.len() && matches!(u8_at(prefix, idx)?, b'-' | b'+') {
        idx += 1;
    }
    let digit_start = idx;
    while idx < prefix.len()
        && (u8_at(prefix, idx).unwrap_or(0).is_ascii_digit()
            || u8_at(prefix, idx).unwrap_or(0) == b'.')
        && idx - scale_start < 16
    {
        idx += 1;
    }
    require!(
        idx > digit_start
            && idx < prefix.len()
            && matches!(u8_at(prefix, idx)?, b'\n' | b'\r' | b' ')
    );
    Some(Label::Phm)
}

/// Adobe Photoshop Brush Preset (`.abr` v6..v10 with `"8BIM"` at `@4..8`).
pub(crate) fn verify_photoshop_abr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PHOTOSHOP_ABR_SIZE,
        prefix.len() >= 12,
        matches!(be16(prefix, 0), Some(6..=10)),
        matches!(be16(prefix, 2), Some(1 | 2)),
        eq_at(prefix, 4, b"8BIM")
    );
    Some(Label::PhotoshopAbr)
}

/// Pleiades Astrophoto PixInsight Extensible Image Serialization Format image (`.xisf`, `XISF 1.0`).
///
/// Header layout:
/// - `@0..8`: `"XISF0100"` (`8B` signature)
/// - `@8..12`: `u32 LE xml_header_length` (`>= 32`, `16 + xml_header_length <= size`)
/// - `@12..16`: `u32 LE reserved` (`== 0`)
/// - `@16..21`: `"<?xml"` or `"<xisf"`
pub(crate) fn verify_pixinsight_xisf(prefix: &[u8], size: u64) -> Option<Label> {
    const XISF_MAGIC: &[u8; 8] = b"XISF0100"; // @0..8: XISF 1.0 signature
    const HDR_LEN_OFFSET: usize = 8; // @8..12: u32 LE XML header byte length
    const RESERVED_OFFSET: usize = 12; // @12..16: u32 LE reserved (0)
    const XML_START_OFFSET: usize = 16; // @16..21: XML prologue or root element

    require!(
        size >= MIN_PIXINSIGHT_XISF_SIZE,
        prefix.len() >= 32,
        starts(prefix, XISF_MAGIC),
        le32(prefix, RESERVED_OFFSET) == Some(0),
        eq_at(prefix, XML_START_OFFSET, b"<?xml") || eq_at(prefix, XML_START_OFFSET, b"<xisf")
    );
    let xml_len = le32(prefix, HDR_LEN_OFFSET)? as u64;
    require!(xml_len >= 32, 16u64.saturating_add(xml_len) <= size);
    Some(Label::PixinsightXisf)
}

/// PM picture (`.pm`, 1989, "VIEW" format).
///
/// 7 int32 fields in writer byte order (`"VIEW"` BE or `"WEIV"` LE): `@4` planes (`1..=8`), `@8` rows,
/// `@12` columns, `@16` bands (`1..=16`), `@20` pixel format (`PM_A 0x8000`, `PM_C 0x8001`, `PM_S 0x8002`,
/// `PM_I 0x8004`, `PM_F 0xC004`), `@24` comment size; pixel planes and comment follow.
pub(crate) fn verify_pm_picture(prefix: &[u8], size: u64) -> Option<Label> {
    const NP_OFFSET: usize = 4; // @4..8: int32 number of planes
    const NROW_OFFSET: usize = 8; // @8..12: int32 rows
    const NCOL_OFFSET: usize = 12; // @12..16: int32 columns
    const NBAND_OFFSET: usize = 16; // @16..20: int32 bands
    const FORM_OFFSET: usize = 20; // @20..24: int32 pixel format
    const CMTSIZE_OFFSET: usize = 24; // @24..28: int32 comment bytes incl. NUL
    /// `PM_A` byte, `PM_C` char, `PM_S` short, `PM_I` int, `PM_F` float pixel formats.
    const PM_FORMS: [u32; 5] = [0x8000, 0x8001, 0x8002, 0x8004, 0xC004];
    require!(size >= MIN_PM_PICTURE_SIZE, prefix.len() >= MIN_PM_PICTURE_SIZE as usize);
    let big_endian = if starts(prefix, PM_PICTURE_MAGIC_BE) {
        true
    } else if starts(prefix, PM_PICTURE_MAGIC_LE) {
        false
    } else {
        return None;
    };
    let rd = |p: &[u8], off: usize| if big_endian { be32(p, off) } else { le32(p, off) };
    require!(
        (1..=8).contains(&rd(prefix, NP_OFFSET)?),
        (1..=32_768).contains(&rd(prefix, NROW_OFFSET)?),
        (1..=32_768).contains(&rd(prefix, NCOL_OFFSET)?),
        (1..=16).contains(&rd(prefix, NBAND_OFFSET)?),
        PM_FORMS.contains(&rd(prefix, FORM_OFFSET)?),
        rd(prefix, CMTSIZE_OFFSET)? <= 65_536
    );
    Some(Label::PmPicture)
}

/// PopCap Games PAM Sprite Animation (.pam) PopCap Games.
pub(crate) fn verify_popcap_pam(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_POPCAP_PAM_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"PAM!"),
        (1..=6).contains(&le16(prefix, 4).unwrap_or(0)),
        le16(prefix, 8) == Some(0),
        has_non_ascii_text_byte(prefix.get(10..24)?)
    );
    Some(Label::PopcapPam)
}

/// Imagination Technologies PowerVR Texture (.pvr).
pub(crate) fn verify_powervr_pvr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_POWERVR_PVR_SIZE && prefix.len() >= 16);
    let valid = if starts(prefix, b"PVR\x03") {
        size >= 52
            && prefix.len() >= 52
            && le32(prefix, 24).unwrap_or(0) > 0
            && le32(prefix, 28).unwrap_or(0) > 0
    } else if starts(prefix, b"\x03RVP") {
        size >= 52
            && prefix.len() >= 52
            && be32(prefix, 24).unwrap_or(0) > 0
            && be32(prefix, 28).unwrap_or(0) > 0
    } else if get_slice(prefix, 0, 4)? == POWERVR_V2_HEADER_LEN_LE {
        size >= 52 && prefix.len() >= 52 && get_slice(prefix, 44, 48)? == b"PVR!"
    } else if starts(prefix, b"PVRT") {
        let w = le16(prefix, 12).unwrap_or(0);
        let h = le16(prefix, 14).unwrap_or(0);
        w > 0 && h > 0 && prefix.get(4..16)?.contains(&0)
    } else if starts(prefix, b"GBIX") {
        prefix.len() >= 32
            && prefix.get(4..16)?.contains(&0)
            && (get_slice(prefix, 16, 20)? == b"PVRT" || get_slice(prefix, 12, 16)? == b"PVRT")
    } else {
        false
    };
    require!(valid);
    Some(Label::Pvr)
}

/// libPGF Progressive Graphics File (`.pgf`, `"PGF"` + version `'1'..'9'` or `0x01..=0x39`).
pub(crate) fn verify_progressive_graphics_file(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PGF_IMAGE_SIZE, prefix.len() >= 18, starts(prefix, b"PGF"));
    let v = u8_at(prefix, 3)?;
    let is_ascii_ver = v.is_ascii_digit() && v != b'0';
    let is_bin_ver = (1..=9).contains(&(v & 0x0F)) && (v & 0xC0) == 0;
    require!(is_ascii_ver || is_bin_ver);
    let v7_ok = (16..=65536).contains(&le32(prefix, 4)?)
        && (1..=65535).contains(&le32(prefix, 8)?)
        && (1..=65535).contains(&le32(prefix, 12)?);
    let v6_ok = (16..=4096).contains(&le16(prefix, 4)?)
        && (1..=65535).contains(&le32(prefix, 6)?)
        && (1..=65535).contains(&le32(prefix, 10)?)
        && u8_at(prefix, 14)? <= 32
        && (1..=64).contains(&u8_at(prefix, 16)?)
        && (1..=8).contains(&u8_at(prefix, 17)?);
    require!(v7_ok || v6_ok);
    Some(Label::Pgf)
}

/// ProShape: `"ProShape V1.0 Drawing"`, `"ProShape V1.x Drawing"` or `"ProShape Drawing"`, then `"\r\n\x1A"`.
pub(crate) fn verify_proshape(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PROSHAPE_SIZE, starts(prefix, b"ProShape "));
    require!(scan_range(prefix, 9, 32, b"Drawing\r\n\x1A"));
    Some(Label::Proshape)
}

/// PRVW Preview Image Container (.prw).
pub(crate) fn verify_prvw_prw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRVW_PRW_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"PRVW"),
        get_slice(prefix, 5, 12)? == b"\xff\x2b\x3e\0\0\0\0",
        get_slice(prefix, 24, 28)? == b"IMG\0"
    );
    Some(Label::PrvwPrw)
}

/// Ptex: `"Ptex"`, `u32 LE version == 1`, `u32 LE meshtype` (0 triangle, 1 quad), `u32 LE datatype` (0 u8, 1 u16, 2 half, 3 float).
pub(crate) fn verify_ptex(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION: usize = 4; // @4: u32 LE
    const MESH_TYPE: usize = 8; // @8: u32 LE
    const DATA_TYPE: usize = 12; // @12: u32 LE
    require!(size >= MIN_PTEX_SIZE, starts(prefix, b"Ptex"));
    require!(
        le32(prefix, VERSION)? == 1,
        le32(prefix, MESH_TYPE)? <= 1,
        le32(prefix, DATA_TYPE)? <= 3
    );
    Some(Label::Ptex)
}

/// Smith Micro QuickLink II Fax Image (`.qfx`).
pub(crate) fn verify_quicklink_qfx(prefix: &[u8], size: u64) -> Option<Label> {
    const QFX_MAGIC: &[u8; 8] = b"QLIIFAX "; // @0..8: "QLIIFAX " signature
    const VERSION_OFFSET: usize = 8; // @8..10: u16 LE format version (1 or 2)
    const WIDTH_OFFSET: usize = 10; // @10..12: u16 LE width in pixels (e.g. 1728)
    const HEIGHT_OFFSET: usize = 12; // @12..14: u16 LE height in lines
    const RESERVED_OFFSET: usize = 14; // @14..16: u16 LE reserved (0)
    const RES_FLAG_OFFSET: usize = 16; // @16..18: u16 LE resolution flag (1 = standard, 2 = fine)
    const PAGES_OFFSET: usize = 18; // @18..20: u16 LE page count (>= 1)

    require!(
        size >= MIN_QUICKLINK_QFX_SIZE,
        prefix.len() >= 32,
        starts(prefix, QFX_MAGIC),
        matches!(le16(prefix, VERSION_OFFSET), Some(1 | 2)),
        (1..=16_384).contains(&le16(prefix, WIDTH_OFFSET)?),
        (1..=32_768).contains(&le16(prefix, HEIGHT_OFFSET)?),
        le16(prefix, RESERVED_OFFSET) == Some(0),
        matches!(le16(prefix, RES_FLAG_OFFSET), Some(1 | 2)),
        le16(prefix, PAGES_OFFSET)? >= 1
    );
    Some(Label::QuicklinkQfx)
}

/// Radiance RGBE HDR image (.hdr/.pic).
pub(crate) fn verify_radiance_hdr(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_RADIANCE_HDR_SIZE
        && (starts(prefix, b"#?RADIANCE\n")
            || starts(prefix, b"#?RADIANCE\r\n")
            || starts(prefix, b"#?RGBE\n")
            || starts(prefix, b"#?RGBE\r\n"))
    {
        Some(Label::HdrRgbe)
    } else {
        None
    }
}

/// Radiocef Studio Dental X-Ray Cephalometry file (`.rcf`, Radio Tecnologia).
///
/// Binary layout:
/// - `@0..36`: Delphi `ShortString[35]` signature `\x23Arquivo de cefalometria do RADIOCEF`
/// - `@0x0280..0x0284`: `u32 LE` offset of the embedded `LEAD` dental radiograph stream (`0x0000033C` = 828)
/// - `@0x0284..0x0288`: `u32 LE` byte size of the embedded `LEAD` stream (`>= 28`)
/// - `@0x02A0..0x02A4`: `u32 LE` total `.rcf` file size (`>= 856`)
/// - `@0x033C..0x0352`: Embedded LEADTOOLS (`"LEAD"`) compressed dental radiograph header with
///   pixel width (`@0x034C`), pixel height (`@0x034E`), and bits per pixel (`@0x0350`, typically 8-bit grayscale).
pub(crate) fn verify_radiocef_rcf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RADIOCEF_RCF_SIZE,
        prefix.len() >= MIN_RADIOCEF_RCF_SIZE as usize,
        starts(prefix, RADIOCEF_RCF_MAGIC),
        le32(prefix, RADIOCEF_LEAD_OFFSET_FIELD)? == RADIOCEF_LEAD_MAGIC_OFFSET as u32,
        le32(prefix, RADIOCEF_LEAD_SIZE_FIELD)? >= 28,
        le32(prefix, RADIOCEF_TOTAL_SIZE_FIELD)? >= MIN_RADIOCEF_RCF_SIZE as u32,
        eq_at(prefix, RADIOCEF_LEAD_MAGIC_OFFSET, RADIOCEF_LEAD_MAGIC),
        le16(prefix, RADIOCEF_LEAD_VER_OFFSET)? == 0x1000
    );
    let width = le16(prefix, RADIOCEF_LEAD_WIDTH_OFFSET)? as u32;
    let height = le16(prefix, RADIOCEF_LEAD_HEIGHT_OFFSET)? as u32;
    let bpp = u8_at(prefix, RADIOCEF_LEAD_BPP_OFFSET)?;
    require!(
        (1..=16_384).contains(&width),
        (1..=16_384).contains(&height),
        matches!(bpp, 1 | 4 | 8 | 12 | 16 | 24)
    );

    Some(Label::RadiocefRcf)
}

/// Rapiscan OS600 Dual-Energy X-Ray Scan container (`.rcf`, 2005 Rapiscan).
///
/// Header & chunk layout:
/// - `@0..8`: Unrotated file header `02 08 03 00 04 00 00 00` (`bytes!(&[0x02, 0x08, 0x03, 0x00], le32_bytes!(4))`)
/// - `@8..20`: Chunk 0 unrotated 12-byte header:
///   - `@8..12`: `0xFFFF1111` LE (`11 11 FF FF`) chunk sync marker
///   - `@12..16`: `u32 LE chunk_type` (`6` = Archive Stream Label, `2` = Acquisition Metadata)
///   - `@16..20`: `u32 LE schema_date` (decimal `YYYYMMDD`, e.g., `20060217` or `20131002`)
/// - Chunk payload bytes starting at chunk offset `+12` are obfuscated with a 4-byte periodic
///   8-bit left rotation `ROTS = [3, 5, 2, 7]`.
/// - Also supports carved/unrotated chunk streams starting directly with `11 11 FF FF` at `@0..4`.
pub(crate) fn verify_rapiscan_rcf(prefix: &[u8], size: u64) -> Option<Label> {
    const RCF_FILE_HEADER_TAIL: &[u8; 7] = bytes!(&[0x08, 0x03, 0x00], le32_bytes!(4)); // @1..8: file header flags (0x0308) + header version 4
    const RCF_CHUNK_SYNC: &[u8; 4] = &[0x11, 0x11, 0xFF, 0xFF]; // @+0..4: 0xFFFF1111 LE chunk sync
    const CHUNK0_OFF: usize = 8; // @8..20: first chunk header
    const CHUNK0_TYPE_OFF: usize = 12; // @12..16: u32 LE chunk_type
    const CHUNK0_DATE_OFF: usize = 16; // @16..20: u32 LE decimal YYYYMMDD schema_date
    const CHUNK0_STR_LEN_OFF: usize = 20; // @20..24: rotated u32 LE UTF-16LE byte length in Chunk Type 6

    if starts(prefix, RCF_CHUNK_SYNC) {
        require!(
            size >= MIN_RAPISCAN_RCF_SIZE,
            prefix.len() >= 56,
            (1..=255).contains(&le32(prefix, 4)?),
            le32(prefix, 20) == Some(0),
            (1..=65535).contains(&le32(prefix, 24)?),
            (1..=8).contains(&le32(prefix, 48)?),
            (16..=16384).contains(&le32(prefix, 52)?)
        );
        return Some(Label::RapiscanRcf);
    }

    require!(
        size >= 108,
        prefix.len() >= 108,
        (1..=4).contains(&u8_at(prefix, 0)?),
        eq_at(prefix, 1, RCF_FILE_HEADER_TAIL),
        eq_at(prefix, CHUNK0_OFF, RCF_CHUNK_SYNC)
    );
    let chunk0_type = le32(prefix, CHUNK0_TYPE_OFF)?;
    let chunk0_date = le32(prefix, CHUNK0_DATE_OFF)?;
    require!((1..=64).contains(&chunk0_type), is_valid_rcf_schema_date(chunk0_date));

    if chunk0_type == 6 {
        let str_byte_len = rcf_rot_le32(prefix, CHUNK0_STR_LEN_OFF)? as usize;
        require!((2..=256).contains(&str_byte_len), str_byte_len.is_multiple_of(2));
        let next_off = 24 + str_byte_len;
        require!(next_off + 12 <= prefix.len(), eq_at(prefix, next_off, RCF_CHUNK_SYNC));
        let chunk1_type = le32(prefix, next_off + 4)?;
        let chunk1_date = le32(prefix, next_off + 8)?;
        require!((1..=64).contains(&chunk1_type), is_valid_rcf_schema_date(chunk1_date));
    }

    Some(Label::RapiscanRcf)
}

/// RED Digital Cinema Thumbnail (.rtn, PRONOM fmt/1664).
pub(crate) fn verify_red_thumb(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_RED_THUMB_SIZE
        && prefix.len() >= MIN_RED_THUMB_SIZE as usize
        && starts(prefix, b"REDTHUMB")
        && has_non_ascii_text_byte(prefix.get(8..prefix.len().min(16))?)
    {
        Some(Label::Redthumb)
    } else {
        None
    }
}

/// Amiga / PC REKO Cardset (`.reko`, `"REKO"`).
pub(crate) fn verify_reko_cardset(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_REKO_CARDSET_SIZE,
        prefix.len() >= 22,
        starts(prefix, b"REKO"),
        be32(prefix, 4)? > 0,
        (16..=1024).contains(&be16(prefix, 12)?),
        (16..=1024).contains(&be16(prefix, 14)?),
        u8_at(prefix, 21)? >= 1
    );
    Some(Label::RekoCardset)
}

/// Rive Interactive Vector Animation file (`.riv`).
pub(crate) fn verify_rive_riv(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RIVE_RIV_SIZE,
        prefix.len() >= 16,
        starts(prefix, RIVE_MAGIC),
        u8_at(prefix, RIVE_MINOR_OFFSET)? <= 16
    );
    Some(Label::RiveRiv)
}

/// RealWorld Paint layered image: `"RWLayeredImage_1"`.
pub(crate) fn verify_rli(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RLI_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"RWLayeredImage_1"),
        has_non_ascii_text_byte(prefix.get(16..prefix.len().min(32))?)
            || eq_at(prefix, 16, b"CNVS")
    );
    Some(Label::Rli)
}

/// ASCII / Enterbrain RPG Maker 2000 / 2003 XYZ Compressed Raster Image (.xyz).
pub(crate) fn verify_rpgmaker_xyz1(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RPGMAKER_XYZ1_SIZE
            && prefix.len() >= 12
            && starts(prefix, b"XYZ1")
            && has_non_ascii_text_byte(prefix.get(4..12)?)
    );
    let width = le16(prefix, 4)?;
    let height = le16(prefix, 6)?;
    let cmf = u8_at(prefix, 8)?;
    let zlib_hdr = be16(prefix, 8)?;
    require!(width > 0 && height > 0 && (cmf & 0x0F) == 8 && (cmf >> 4) <= 7 && zlib_hdr % 31 == 0);
    Some(Label::RpgmakerXyz)
}

/// Akella Storm Engine Texture (`.tx`, DXT1/DXT3/DXT5 or D3DFMT pixel formats).
pub(crate) fn verify_seadogs_dxt1_tx(prefix: &[u8], size: u64) -> Option<Label> {
    const FMT_DXT1: u32 = u32::from_le_bytes(*b"DXT1");
    const FMT_DXT3: u32 = u32::from_le_bytes(*b"DXT3");
    const FMT_DXT5: u32 = u32::from_le_bytes(*b"DXT5");

    require!(size >= MIN_SEADOGS_DXT1_TX_SIZE, prefix.len() >= 24);
    let flags = le32(prefix, 0)?;
    let w = le32(prefix, 4)?;
    let h = le32(prefix, 8)?;
    let mips = le32(prefix, 12)?;
    let fmt = le32(prefix, 16)?;
    let mip0 = le32(prefix, 20)?;
    require!(
        flags <= 0xFF,
        (8..=8192).contains(&w) && w.is_power_of_two(),
        (8..=8192).contains(&h) && h.is_power_of_two(),
        (1..=14).contains(&mips)
    );
    let level_bytes = |lw: u32, lh: u32| -> Option<u64> {
        match fmt {
            FMT_DXT1 => Some((lw.div_ceil(4) as u64) * (lh.div_ceil(4) as u64) * 8),
            FMT_DXT3 | FMT_DXT5 => Some((lw.div_ceil(4) as u64) * (lh.div_ceil(4) as u64) * 16),
            21 | 22 => Some((lw as u64) * (lh as u64) * 4), // D3DFMT_A8R8G8B8 / X8R8G8B8
            23..=26 => Some((lw as u64) * (lh as u64) * 2), // D3DFMT_R5G6B5 / X1R5G5B5 / A1R5G5B5 / A4R4G4B4
            _ => None,
        }
    };
    let expected_mip0 = level_bytes(w, h)?;
    require!((mip0 as u64) == expected_mip0);
    // For numeric D3DFMT codes (without ASCII FourCC), enforce exact total mipmap payload size.
    if (21..=26).contains(&fmt) {
        let mut total: u64 = 0;
        let (mut lw, mut lh) = (w, h);
        for _ in 0..mips {
            total = total.checked_add(level_bytes(lw, lh)?)?;
            lw = (lw / 2).max(1);
            lh = (lh / 2).max(1);
        }
        require!(24u64.checked_add(total) == Some(size));
    }
    Some(Label::StormEngineTx)
}

/// Silicon Graphics IRIS RGB Image (.sgi / .rgb / .bw).
pub(crate) fn verify_sgi_rgb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SGI_RGB_SIZE && prefix.len() >= 108 && starts(prefix, &[0x01, 0xDA]));
    let storage = u8_at(prefix, 2)?;
    let bpc = u8_at(prefix, 3)?;
    let dimension = be16(prefix, 4)?;
    let xsize = be16(prefix, 6)?;
    let ysize = be16(prefix, 8)?;
    let zsize = be16(prefix, 10)?;
    let colormap = be32(prefix, 104)?;
    require!(
        storage <= 1
            && (1..=2).contains(&bpc)
            && (1..=3).contains(&dimension)
            && xsize > 0
            && ysize > 0
            && (1..=4).contains(&zsize)
            && colormap <= 3
    );
    Some(Label::SgiRgb)
}

/// Sierra On-Line SCI1.1/SCI32 Background Picture Patch (`.p56`, 1992 Sierra On-Line).
pub(crate) fn verify_sierra_sci32_pic(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SIERRA_SCI32_PIC_SIZE,
        prefix.len() >= 20,
        starts(prefix, SIERRA_SCI32_PIC_MAGIC),
        eq_at(prefix, 4, hex2bin!("00 01 02 03 04 05 06 07"))
    );
    Some(Label::SierraSci32Pic)
}

/// Sierra On-Line SCI32 Sprite View Patch (`.v56`, 1993 Sierra On-Line).
pub(crate) fn verify_sierra_sci32_view(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SIERRA_SCI32_VIEW_SIZE,
        prefix.len() >= 32,
        starts(prefix, SIERRA_SCI32_VIEW_MAGIC),
        (160..=1024).contains(&le16(prefix, 4)?),
        (100..=768).contains(&le16(prefix, 6)?),
        le16(prefix, 26) == Some(16),
        (1..=256).contains(&le16(prefix, 28)?)
    );
    Some(Label::SierraSci32View)
}

/// Sigma Foveon X3 Camera RAW (`.x3f`, `"FOVb"`).
pub(crate) fn verify_sigma_x3f_raw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SIGMA_X3F_SIZE, prefix.len() >= 36, starts(prefix, b"FOVb"));
    let ver = le32(prefix, 4)?;
    let cols = le32(prefix, 28)?;
    let rows = le32(prefix, 32)?;
    require!(
        (0x0001_0000..=0x0006_0000).contains(&ver),
        (1..=65535).contains(&cols),
        (1..=65535).contains(&rows)
    );
    Some(Label::SigmaX3f)
}

/// Electronic Arts The Sims 4 Tray Thumbnail Image (`.hhi`/`.bpi`/`.sgi`, 2014 Electronic Arts / Maxis).
pub(crate) fn verify_sims4_thumbnail(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SIMS4_THUMBNAIL_SIZE,
        prefix.len() >= 32,
        eq_at(prefix, 8, SIMS4_THUMBNAIL_SIG),
        le16(prefix, 16) == Some(2),
        eq_at(prefix, 24, SIMS4_THUMBNAIL_PAYLOAD_SIG),
        le32(prefix, 0)? as u64 + 24 == size
    );
    Some(Label::Sims4Thumbnail)
}

/// Infinite Pictures SmoothMove Pan Viewer Panorama (`.pan`, 1996 Infinite Pictures).
///
/// Header layout:
/// - `@0x00..0x04`: `b"IPSM"` (`49 50 53 4D`)
/// - `@0x04..0x08`: `u32 LE` `file_size` (`== size`)
/// - `@0x08..0x0C`: `u32 LE` `num_sections` (`2..=16`)
/// - `@0x0C..0x10`: `u32 LE` `reserved` (`0`)
/// - `@0x10..0x10 + num_sections * 16`: Section directory entries (`[u8; 4]` tag `BTMP`/`INIT`/`HSPT`,
///   `u32 LE` offset, `u32 LE` length, `u32 LE` reserved `0`), with `BTMP` pointing to an embedded JFIF JPEG (`FF D8 FF`).
pub(crate) fn verify_smoothmove_pan(prefix: &[u8], size: u64) -> Option<Label> {
    const FILE_SIZE_OFF: usize = 4; // @4: u32 LE total file size in bytes
    const NUM_SECTIONS_OFF: usize = 8; // @8: u32 LE number of 16-byte directory entries (2..=16)
    const RESERVED_OFF: usize = 12; // @12: u32 LE reserved (0)
    require!(
        size >= MIN_SMOOTHMOVE_PAN_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"IPSM"),
        le32(prefix, FILE_SIZE_OFF)? as u64 == size,
        le32(prefix, RESERVED_OFF)? == 0
    );
    let num_sections = le32(prefix, NUM_SECTIONS_OFF)? as usize;
    require!((2..=16).contains(&num_sections));
    let dir_end = 16 + num_sections * 16;
    require!(prefix.len() >= dir_end, size >= (dir_end as u64) + 20);

    let mut has_btmp = false;
    let mut has_init = false;
    for i in 0..num_sections {
        let entry_off = 16 + i * 16;
        let tag = get_slice(prefix, entry_off, entry_off + 4)?;
        let sec_off = le32(prefix, entry_off + 4)? as u64;
        let sec_len = le32(prefix, entry_off + 8)? as u64;
        let sec_res = le32(prefix, entry_off + 12)?;
        require!(
            tag.iter().all(|&b| b.is_ascii_uppercase() || b.is_ascii_digit()),
            sec_off >= dir_end as u64,
            sec_len > 0,
            sec_off.saturating_add(sec_len) <= size,
            sec_res == 0
        );
        if tag == b"BTMP" {
            has_btmp = true;
            if (sec_off as usize) + 3 <= prefix.len() {
                require!(eq_at(prefix, sec_off as usize, hex2bin!("FF D8 FF")));
            }
        } else if tag == b"INIT" {
            require!(sec_len == 16);
            has_init = true;
        }
    }
    require!(has_btmp && has_init);
    Some(Label::SmoothmovePan)
}

/// SMPTE 268M Digital Picture Exchange Image (.dpx).
pub(crate) fn verify_smpte_dpx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SMPTE_DPX_SIZE
            && prefix.len() >= 24
            && (starts(prefix, b"SDPX") || starts(prefix, b"XPDS"))
            && prefix.get(4..24)?.contains(&0)
    );
    let data_offset = if starts(prefix, b"SDPX") {
        be32(prefix, 4).unwrap_or(0)
    } else {
        le32(prefix, 4).unwrap_or(0)
    };
    require!(
        data_offset >= 1024 && u8_at(prefix, 8)? == b'V' && u8_at(prefix, 9)?.is_ascii_digit()
    );
    Some(Label::Dpx)
}

/// Sony Cyber-shot DSC-F1 PMP Still Image (`.pmp`, 1996 Sony).
///
/// Header layout:
/// - `@8..12`: `u32 BE jpeg_offset` (`0x0000007C` = `124`)
/// - `@12..16`: `u32 BE jpeg_length` (`> 0`, `124 + jpeg_length <= size`)
/// - `@22..24`: `u16 BE width` (`1..=8192`)
/// - `@24..26`: `u16 BE height` (`1..=8192`)
/// - `@124..128`: `FF D8 FF DB` (JPEG SOI followed immediately by DQT marker).
pub(crate) fn verify_sony_pmp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SONY_PMP_SIZE,
        prefix.len() >= 128,
        be32(prefix, 8) == Some(0x7C),
        eq_at(prefix, 124, SONY_PMP_JPEG_DQT_MAGIC),
        (1..=8192).contains(&be16(prefix, 22).unwrap_or(0)),
        (1..=8192).contains(&be16(prefix, 24).unwrap_or(0))
    );
    Some(Label::SonyPmp)
}

/// Sony PlayStation Portable GIM Texture (`.gim`, `"MIG.00.1PSP\0"` / `".GIM00.1PSP\0"`).
pub(crate) fn verify_sony_psp_gim(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PSP_GIM_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"MIG.00.1PSP\0") || starts(prefix, b".GIM00.1PSP\0"),
        le32(prefix, 12) == Some(0)
    );
    Some(Label::SonyPspGim)
}

/// GI 16-Bit RGB565 Sprite Image (.gi).
pub(crate) fn verify_space_rangers_gi(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SPACE_RANGERS_GI_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"gi\0\0\x01\0\0\0\0\0\0\0\0\0\0\0"),
        get_slice(prefix, 24, 36)? == b"\0\xf8\0\0\xe0\x07\0\0\x1f\0\0\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::SpaceRangersGi)
}

/// Atari ST STAD High-Resolution Compressed Graphic (.pac).
pub(crate) fn verify_stad_pac(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        (MIN_STAD_PAC_SIZE..=32768).contains(&size)
            && prefix.len() >= 16
            && (starts(prefix, b"pM85") || starts(prefix, b"pM86"))
            && has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    let id_byte = u8_at(prefix, 4)?;
    let special_byte = u8_at(prefix, 6)?;
    require!(id_byte != special_byte);
    Some(Label::StadPac)
}

/// GSC Game World X-Ray Engine Texture Descriptor (.thm, 0x0810).
pub(crate) fn verify_stalker_xray_thm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_STALKER_XRAY_THM_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 32)?
            == hex2bin!(
                "10 08 00 00 02 00 00 00 12 00 13 08 00 00 04 00 00 00 01 00 00 00 12 08 00 00 20 00 00 00 03 00"
            )
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::StalkerXrayThm)
}

/// StatSoft Statistica Graph (`.stg`, 1991 StatSoft).
///
/// Header layout:
/// - `0..7`: `05 04 03 06 57 6B 00` (`05 04 03 06 "Wk\0"`)
/// - `9..17`: 8 reserved NUL bytes (`00 * 8`)
/// - `17`: `0x0E` record tag
/// - `18..24`: 6 reserved NUL bytes (`00 * 6`)
pub(crate) fn verify_statistica_stg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_STATISTICA_STG_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"\x05\x04\x03\x06Wk\x00"),
        get_slice(prefix, 9, 17)?.iter().all(|&b| b == 0),
        u8_at(prefix, 17) == Some(0x0E),
        get_slice(prefix, 18, 24)?.iter().all(|&b| b == 0)
    );
    Some(Label::StatisticaStg)
}

/// Sun Microsystems Raster image (`.ras` / `.sun`, `0x59A66A95` BE).
pub(crate) fn verify_sunras(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 4] = be32_bytes!(0x59A6_6A95);
    require!(size >= 32, prefix.len() >= 32, starts(prefix, MAGIC));
    let _ = be32(prefix, 4)?;
    let _ = be32(prefix, 8)?;
    let _ = be32(prefix, 12)?;
    let _ = be32(prefix, 20)?;
    Some(Label::Sunras)
}

/// Tencent TAP Animated Image Container (`.tap`, `"TAP0"`).
pub(crate) fn verify_tencent_tap_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TENCENT_TAP_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"TAP0"),
        (1..=16384).contains(&le32(prefix, 4)?),
        (1..=16384).contains(&le32(prefix, 8)?),
        (1..=256).contains(&le32(prefix, 12)?),
        (1..=100).contains(&le32(prefix, 20)?)
    );
    Some(Label::TencentTap)
}

/// Ubisoft Dunia Engine XBT Texture (.xbt, 2008) Ubisoft Montreal.
pub(crate) fn verify_ubisoft_xbt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_UBISOFT_XBT_SIZE, prefix.len() >= 64, starts(prefix, b"TBX\0"));
    let dds_off = le32(prefix, 8)? as usize;
    require!(
        (32..=60).contains(&dds_off),
        le32(prefix, 12) == Some(0),
        get_slice(prefix, dds_off, dds_off + 4)? == b"DDS "
    );
    Some(Label::UbisoftXbt)
}

/// Valve Source Engine Texture Format (.vtf).
pub(crate) fn verify_valve_vtf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VALVE_VTF_SIZE
            && prefix.len() >= 24
            && starts(prefix, b"VTF\0")
            && le32(prefix, 4) == Some(7)
            && le32(prefix, 8).unwrap_or(99) <= 6
            && le32(prefix, 12).unwrap_or(0) >= 64
            && le16(prefix, 16).unwrap_or(0) > 0
            && le16(prefix, 18).unwrap_or(0) > 0
    );
    Some(Label::Vtf)
}

/// Borland VCL Metafile / StarView SVM (.svm / .vcl).
pub(crate) fn verify_vcl_metafile(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VCL_METAFILE_SIZE
            && prefix.len() >= 16
            && starts(prefix, b"VCLMTF")
            && has_non_ascii_text_byte(prefix.get(6..16)?)
    );
    Some(Label::Svm)
}

/// XBIN: `"XBIN\x1A"`, `u16 LE width > 0`, `u16 LE height > 0`, `u8 font_height` (1..=32), `u8 flags` (bits 5..7 reserved = 0).
pub(crate) fn verify_xbin(prefix: &[u8], size: u64) -> Option<Label> {
    const WIDTH: usize = 5; // @5: u16 LE columns
    const HEIGHT: usize = 7; // @7: u16 LE rows
    const FONT_HEIGHT: usize = 9; // @9: u8 scanlines per glyph
    const FLAGS: usize = 10; // @10: palette / font / compress / non-blink / 512-char
    require!(size >= MIN_XBIN_SIZE, starts(prefix, b"XBIN\x1A"));
    require!(
        le16(prefix, WIDTH)? > 0,
        le16(prefix, HEIGHT)? > 0,
        (1..=32).contains(&u8_at(prefix, FONT_HEIGHT)?)
    );
    require!(u8_at(prefix, FLAGS)? & 0xE0 == 0);
    Some(Label::Xbin)
}

/// yGLF Graphics/Glyph Format (.glf).
pub(crate) fn verify_yglf_file(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_YGLF_FILE_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"yGLF\r\n\x1a\n\0\x01\0\0\0\0\x02")
    );
    Some(Label::YglfFile)
}

/// Zoner ZMF: `u32 LE 92` header size, `u32 LE 0xD4015ADD` signature, `u32 LE 0x12345678` byte-order marker.
pub(crate) fn verify_zoner_zmf(prefix: &[u8], size: u64) -> Option<Label> {
    const HEADER: &[u8] =
        bytes!(le32_bytes!(0x5C), le32_bytes!(0xD401_5ADD), le32_bytes!(0x1234_5678)); // @0, 12 bytes
    require!(size >= MIN_ZONER_ZMF_SIZE, starts(prefix, HEADER));
    Some(Label::Zmf)
}
