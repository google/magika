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
    Some(Label::Psd)
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
    const MAGIC: &[u8] = b"gimp xcf "; // @0..9: XCF magic prefix
    const VERSION: usize = 9; // @9..14: "file\0" or "vNNN\0"
    const VERSION_V0: &[u8] = b"file\0";
    const IMAGE_TYPE: usize = 22; // @22: u32 BE — base image type (0..=2)
    const MIN_LEN: usize = 26;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC), be32(prefix, IMAGE_TYPE)? <= 2);
    let ver_ok = eq_at(prefix, VERSION, VERSION_V0)
        || (u8_at(prefix, VERSION)? == b'v'
            && get_slice(prefix, 10, 13).is_some_and(|s| s.iter().all(u8::is_ascii_digit))
            && u8_at(prefix, 13)? == 0);
    require!(ver_ok);
    Some(Label::Xcf)
}

/// CompuServe GIF (`"GIF87a"` or `"GIF89a"`): nonzero width and height at `@6` and `@8`, and a
/// valid block introducer (`0x21` extension, `0x2C` image descriptor, or `0x3B` trailer) right
/// after the optional Global Color Table.
pub(crate) fn gif(prefix: &[u8]) -> Option<Label> {
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
    Some(Label::Gif)
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
    const MAGIC: &[u8] = &[0xFF, 0x80, 0xC9, 0xC7]; // @0..4: Koala compressed load header
    const MIN_LEN: usize = 16;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    Some(Label::Koala)
}

/// Commodore 64 uncompressed Koala Painter image (`.koa` / `.kla` / `.pic`): 2-byte load address
/// `00 60` (`$6000`) at `@0..2`, exact 10,003-byte file size, and background color nibble `<= 0x0F` at `@10002`.
pub(crate) fn koala_uncompressed(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    const LOAD_ADDR_6000: &[u8] = &[0x00, 0x60];
    require!(
        ctx.size == 10_003,
        prefix.len() >= 10_003,
        starts(prefix, LOAD_ADDR_6000),
        u8_at(prefix, 10_002)? <= 0x0F,
    );
    let _ = get_slice(prefix, 0, 16);
    Some(Label::Koala)
}

/// Netpbm image (`"P1"`..`"P6"`): 1..=8 whitespace bytes, up to 2 `#...\n` comment lines (each ≤
/// 128 comment bytes followed by `\n` and 0..=4 whitespace bytes), width (1..=6 ASCII digits),
/// 1..=4 whitespace bytes, height (1..=6 ASCII digits), and a trailing whitespace byte.
pub(crate) fn pbm(prefix: &[u8]) -> Option<Label> {
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
    Some(Label::Pbm)
}

/// Windows or OS/2 BMP image (`"BM"`): zero reserved words at `@6..10`, file size and pixel
/// offset at `@2` and `@10`, followed at `@14` by either a 12-byte OS/2 `BITMAPCOREHEADER` or an
/// uncompressed Windows DIB header (40, 52, 56, 108, or 124 bytes).
pub(crate) fn bmp(prefix: &[u8]) -> Option<Label> {
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
    Some(Label::Bmp)
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
    Some(Label::Degas)
}

/// W3C Portable Network Graphics (`png`, 1996, RFC 2083): 8-byte PNG signature at `@0`,
/// 13-byte `IHDR` chunk at `@8..33`, nonzero dimensions at `@16..24`, valid bit depth
/// (`1|2|4|8|16`) at `@24`, valid color type (`0|2|3|4|6`) at `@25`, and zero
/// compression/filter methods at `@26..28`.
pub(crate) fn png(prefix: &[u8], size: u64) -> Option<Label> {
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
    Some(Label::Png)
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
    const MIN_TIFF_SIZE: u64 = 14; // 8B TIFF header + 2B entry count + 4B next IFD offset

    require!(size >= MIN_TIFF_SIZE, prefix.len() >= 8);
    let valid = if starts(prefix, TIFF_LE_MAGIC) {
        let off = le32(prefix, 4)?;
        off >= 8 && off % 2 == 0 && (off as u64) + 14 <= size
    } else if starts(prefix, TIFF_BE_MAGIC) {
        let off = be32(prefix, 4)?;
        off >= 8 && off % 2 == 0 && (off as u64) + 14 <= size
    } else if starts(prefix, BIGTIFF_LE_MAGIC) {
        prefix.len() >= 16
            && le16(prefix, 4)? == 8
            && le16(prefix, 6)? == 0
            && le64(prefix, 8)? >= 16
    } else if starts(prefix, BIGTIFF_BE_MAGIC) {
        prefix.len() >= 16
            && be16(prefix, 4)? == 8
            && be16(prefix, 6)? == 0
            && be64(prefix, 8)? >= 16
    } else {
        false
    };
    require!(valid);
    Some(Label::Tiff)
}

/// Aldus Placeable Windows Metafile (`wmf`, 1988): magic `D7 CD C6 9A` at `@0`, `hmf == 0`
/// at `@4`, XOR-16 checksum over the first 10 words at `@20`, followed by standard WMF header
/// (`mtType` `1|2`, `mtHeaderSize == 9`, `mtVersion` `0x0100|0x0300`) at `@22`.
pub(crate) fn wmf_placeable(prefix: &[u8], size: u64) -> Option<Label> {
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
    Some(Label::Wmf)
}

/// Standard Windows GDI Metafile (`wmf`, 1988): `mtType` `1` or `2` LE at `@0`,
/// `mtHeaderSize == 9` (18 bytes) at `@2`, `mtVersion` `0x0100` or `0x0300` at `@4`,
/// `mtSize >= 9` words at `@6`, `mtMaxRecord >= 3` words at `@12`, `mtNoParameters == 0`
/// at `@16`, and first record size `>= 3` at `@18`.
pub(crate) fn wmf_standard(prefix: &[u8], size: u64) -> Option<Label> {
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
    Some(Label::Wmf)
}

/// Truevision TGA (`tga`, 1984 / v2.0 1989): either carries the 26-byte v2.0 footer ending
/// with `"TRUEVISION-XFILE.\0"` in `tail`, or satisfies strict 18-byte TGA v1.0 header
/// + exact file size geometry.
pub(crate) fn tga(prefix: &[u8], size: u64, tail: Option<&[u8]>) -> Option<Label> {
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
        return Some(Label::Tga);
    }

    // Uncompressed TGA v1.0 exact size match (18B header + ID + colormap + width*height*(bpp/8))
    if matches!(img_type, 1..=3) {
        let id_len = u8_at(prefix, 0)? as u64;
        let cmap_bytes = cmap_len * ((cmap_bpp as u64).div_ceil(8));
        let pixel_bytes = width * height * ((bpp as u64).div_ceil(8));
        let expected = 18 + id_len + cmap_bytes + pixel_bytes;
        if size == expected || size == expected + 26 {
            return Some(Label::Tga);
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
            return Some(Label::Jxl);
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
        return Some(Label::Jxl);
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
        Some(Label::Jxl)
    } else {
        None
    }
}

/// Scanntronik Printfox / Pagefox C64 bitmap (`printfox`, 1986): `'B'` (`320x200`, `8000`B) or `'G'`
/// (`640x400`, `32000`B) type byte at `@0` followed by `0x9B` 16-bit LE RLE stream decoding to exact canvas size.
pub(crate) fn printfox(prefix: &[u8], size: u64) -> Option<Label> {
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
    Some(Label::Printfox)
}

/// Neuroimaging Informatics Technology Initiative NIfTI-1 volume (`nifti`, 2004): `sizeof_hdr == 348`
/// (LE or BE) at `@0..4` paired with `"n+1\0"` or `"ni1\0"` magic at `@344..348`.
pub(crate) fn nifti(prefix: &[u8], size: u64) -> Option<Label> {
    const NIFTI1_SINGLE_MAGIC: &[u8; 4] = b"n+1\0";
    const NIFTI1_PAIR_MAGIC: &[u8; 4] = b"ni1\0";

    require!(
        size >= 348,
        prefix.len() >= 348,
        (eq_at(prefix, 344, NIFTI1_SINGLE_MAGIC) || eq_at(prefix, 344, NIFTI1_PAIR_MAGIC))
            && (le32(prefix, 0)? == 348 || be32(prefix, 0)? == 348),
    );
    Some(Label::Nifti)
}

/// Eastman Kodak Photo CD image (`pcd`, 1992): 2048-byte preamble followed by `"PCD_IPI"` or
/// `"PCD_OPA"` signature at `@2048..2055`.
pub(crate) fn pcd_kodak(prefix: &[u8], size: u64) -> Option<Label> {
    const KODAK_PCD_IPI_MAGIC: &[u8; 7] = b"PCD_IPI";
    const KODAK_PCD_OPA_MAGIC: &[u8; 7] = b"PCD_OPA";

    require!(
        size >= 2055,
        prefix.len() >= 2055,
        has_non_ascii_text_byte(&prefix[..4]),
        eq_at(prefix, 2048, KODAK_PCD_IPI_MAGIC) || eq_at(prefix, 2048, KODAK_PCD_OPA_MAGIC),
    );
    Some(Label::Pcd)
}

/// Point Cloud Library (PCL) Point Cloud Data file (`pcd`, 2010): `"# .PCD v"` or `"VERSION 0.7"`
/// header followed by `\nFIELDS `, `\nPOINTS `, and `\nDATA ` directives.
pub(crate) fn pcd_ascii(prefix: &[u8], size: u64) -> Option<Label> {
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
    Some(Label::Pcd)
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
