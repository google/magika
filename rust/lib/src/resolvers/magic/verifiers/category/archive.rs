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

//! Archive and compressed stream format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// Long Range ZIP (lrzip): `"LRZI"` magic, major version 0, minor version ≥ 1, and 24-byte header.
pub(crate) fn lrz(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"LRZI"; // @0..4: lrzip signature
    const MAJOR: usize = 4; // @4: u8 — major version (0)
    const MINOR: usize = 5; // @5: u8 — minor version (≥ 1)
    const MIN_LEN: usize = 24;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        u8_at(prefix, MAJOR)? == 0,
        u8_at(prefix, MINOR)? >= 1,
    );
    Some(Label::Lrz)
}

/// Lzip compressed file: `"LZIP"` magic, version 0 (≥ 18 bytes total) or 1 (≥ 26 bytes total),
/// and dictionary size exponent `12..=29` in the low 5 bits of byte 5.
pub(crate) fn lzip(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    static LZIP_V0: Subformat = Subformat::new("lzip version 0");
    static LZIP_V1: Subformat = Subformat::new("lzip version 1");
    const MAGIC: &[u8] = b"LZIP"; // @0..4: lzip magic
    const VERSION: usize = 4; // @4: u8 — version 0 or 1
    const DICT_SIZE: usize = 5; // @5: u8 — low 5 bits are log2(dict_size) in 12..=29
    const MIN_LEN: usize = 8;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        (12..=29).contains(&(u8_at(prefix, DICT_SIZE)? & 0x1F)),
    );
    let version = u8_at(prefix, VERSION)?;
    let valid_ver = match version {
        0 => ctx.size >= 18,
        1 => ctx.size >= 26,
        _ => false,
    };
    require!(valid_ver);
    with_subformat(Label::Lz, if version == 0 { &LZIP_V0 } else { &LZIP_V1 })
}

/// LZ4 frame: legacy (`0x184C2102` / `0x184C2103` LE) or v1 (`0x184D2204` LE) with valid FLG/BD
/// descriptors and minimum frame size for the declared optional header fields.
pub(crate) fn lz4(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    static LZ4_LEGACY: Subformat = Subformat::new("LZ4 legacy frame");
    static LZ4_FRAME: Subformat = Subformat::new("LZ4 frame");
    const LEGACY_V02: &[u8] = &0x184C_2102u32.to_le_bytes(); // @0..4: legacy LZ4 magic (02 21 4C 18)
    const LEGACY_V03: &[u8] = &0x184C_2103u32.to_le_bytes(); // @0..4: LZ4 v1.3 magic (03 21 4C 18)
    const MODERN_MAGIC: &[u8] = &0x184D_2204u32.to_le_bytes(); // @0..4: LZ4 v1.4+ frame magic (04 22 4D 18)
    const FLG: usize = 4; // @4: u8 — frame descriptor flags
    const BD: usize = 5; // @5: u8 — block descriptor (0x40 | 0x50 | 0x60 | 0x70)
    const MIN_LEN: usize = 8;

    require!(prefix.len() >= MIN_LEN);
    if starts(prefix, LEGACY_V02) || starts(prefix, LEGACY_V03) {
        return with_subformat(Label::Lz4, &LZ4_LEGACY);
    }
    require!(starts(prefix, MODERN_MAGIC));
    let (flg, bd) = (u8_at(prefix, FLG)?, u8_at(prefix, BD)?);
    // Version bits 7..6 must be 01 (0x40), reserved bit 1 must be 0, block max size in 4..=7.
    require!((flg & 0b1100_0010) == 0x40, matches!(bd, 0x40 | 0x50 | 0x60 | 0x70));
    let (min_prefix, min_size): (usize, u64) = match flg & 0b0000_1001 {
        0x00 => (7, 11),  // neither content size nor dict ID
        0x01 => (11, 15), // 4-byte dict ID
        0x08 => (15, 19), // 8-byte content size
        0x09 => (19, 23), // 8-byte content size + 4-byte dict ID
        _ => return None,
    };
    require!(prefix.len() >= min_prefix, ctx.size >= min_size);
    with_subformat(Label::Lz4, &LZ4_FRAME)
}

/// RAR archive: RAR 1.5..4.x (`"Rar!\x1a\x07\x00"`), RAR 5.0 (`"Rar!\x1a\x07\x01\x00"`), or
/// pre-1.5 legacy (`"RE~^"`).
pub(crate) fn rar(prefix: &[u8]) -> Option<Label> {
    static RAR_5: Subformat = Subformat::with_ids("RAR 5.0", "fmt/613", Some("fdd000460"));
    static RAR_15_4X: Subformat = Subformat::new("RAR 1.5-4.x");
    static RAR_14: Subformat = Subformat::new("RAR 1.4");
    const RAR4: &[u8] = b"Rar!\x1a\x07\x00"; // @0..7: RAR v1.5–4.x signature
    const RAR5: &[u8] = b"Rar!\x1a\x07\x01\x00"; // @0..8: RAR v5.0 signature
    const LEGACY: &[u8] = b"RE~^"; // @0..4: RAR pre-1.5 signature
    const MIN_LEN: usize = 8;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, RAR4) || starts(prefix, RAR5) || starts(prefix, LEGACY),
    );
    let sub = if starts(prefix, RAR5) {
        &RAR_5
    } else if starts(prefix, RAR4) {
        &RAR_15_4X
    } else {
        &RAR_14
    };
    with_subformat(Label::Rar, sub)
}

/// rzip compressed file: `"RZIP"` magic, major version 2, minor version `0..=1`, two 32-bit size
/// fields, and 10 reserved zero bytes at `@14..24`.
pub(crate) fn rzip(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"RZIP\x02"; // @0..5: RZIP magic + major version 2
    const MINOR: usize = 5; // @5: u8 — minor version (0 or 1)
    const RESERVED: usize = 14; // @14..24: 10 reserved zero bytes
    const HEADER_LEN: usize = 24;

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, MAGIC),
        u8_at(prefix, MINOR)? <= 1,
        eq_at(prefix, RESERVED, &[0; 10]),
    );
    Some(Label::Rzip)
}

/// 7-Zip archive: 6-byte signature `37 7A BC AF 27 1C`, major version 0, and 32-byte signature header.
pub(crate) fn sevenzip(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = b"7z\xbc\xaf\x27\x1c\x00"; // @0..7: 7z signature + major version 0
    const HEADER_LEN: usize = 32;

    require!(prefix.len() >= HEADER_LEN, starts(prefix, HEADER));
    Some(Label::Sevenzip)
}

/// eXtensible ARchive (XAR): `"xar!"` magic, 28-byte header, and nonzero 64-bit compressed and
/// uncompressed TOC lengths at `@8..16` and `@16..24`.
pub(crate) fn xar(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"xar!"; // @0..4: XAR header magic
    const TOC_COMP_HI: usize = 8; // @8..16: u64 BE — compressed TOC length (> 0)
    const TOC_COMP_LO: usize = 12;
    const TOC_UNCOMP_HI: usize = 16; // @16..24: u64 BE — uncompressed TOC length (> 0)
    const TOC_UNCOMP_LO: usize = 20;
    const HEADER_LEN: usize = 28;

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, MAGIC),
        be32(prefix, TOC_COMP_HI)? == 0 && (1..0x0800_0000).contains(&be32(prefix, TOC_COMP_LO)?),
        be32(prefix, TOC_UNCOMP_HI)? == 0
            && (1..0x0800_0000).contains(&be32(prefix, TOC_UNCOMP_LO)?),
    );
    Some(Label::Xar)
}

/// Zstandard compressed frame: legacy magics `0xFD2FB522..=0xFD2FB527` LE (≥ 8 bytes) or modern
/// `0xFD2FB528` LE (≥ 9 bytes) with reserved bit 3 of `Frame_Header_Descriptor` clear.
pub(crate) fn zst(prefix: &[u8]) -> Option<Label> {
    static ZSTD_FRAME: Subformat = Subformat::new("Zstandard frame");
    static ZSTD_LEGACY: Subformat = Subformat::new("Zstandard legacy (v0.x)");
    const MAGIC_TAIL: &[u8] = &[0xB5, 0x2F, 0xFD]; // @1..4: upper 3 bytes of Zstandard magic
    const FHD: usize = 4; // @4: u8 — Frame_Header_Descriptor (bit 3 reserved = 0)
    const MIN_LEN: usize = 8;

    require!(prefix.len() >= MIN_LEN, eq_at(prefix, 1, MAGIC_TAIL));
    let magic_lo = u8_at(prefix, 0)?;
    let valid = match magic_lo {
        0x22..=0x27 => true,
        0x28 => prefix.len() >= 9 && (u8_at(prefix, FHD)? & 0x08) == 0,
        _ => false,
    };
    require!(valid);
    with_subformat(Label::Zst, if magic_lo == 0x28 { &ZSTD_FRAME } else { &ZSTD_LEGACY })
}

/// Microsoft Cabinet archive (`CFHEADER`): `"MSCF\0\0\0\0"`, `cbCabinet` ≥ 36 at `@8`, reserved2
/// `0` at `@12`, `coffFiles` ≥ 36 at `@16`, reserved3 `0` at `@20`, version 1.3 (`0x0103`) at
/// `@24`, `cFolders` ≥ 1 at `@26`, and `flags` ≤ 7 at `@30`.
pub(crate) fn cab(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"MSCF", &0u32.to_le_bytes()); // @0..8: MSCF signature + reserved1 = 0
    const CB_CABINET: usize = 8; // @8: u32 LE — archive size in bytes (≥ 36)
    const RESERVED2: usize = 12; // @12: u32 LE — reserved2 (0)
    const COFF_FILES: usize = 16; // @16: u32 LE — offset of first CFFILE entry (≥ 36)
    const RESERVED3: usize = 20; // @20: u32 LE — reserved3 (0)
    const VERSION: usize = 24; // @24: u16 LE — versionMinor=3, versionMajor=1 (0x0103 = 259)
    const C_FOLDERS: usize = 26; // @26: u16 LE — number of CFFOLDER entries (≥ 1)
    const FLAGS: usize = 30; // @30: u16 LE — option flags (≤ 7)
    const MIN_LEN: usize = 36;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, HEADER),
        le32(prefix, CB_CABINET)? >= MIN_LEN as u32,
        le32(prefix, RESERVED2)? == 0,
        le32(prefix, COFF_FILES)? >= MIN_LEN as u32,
        le32(prefix, RESERVED3)? == 0,
        le16(prefix, VERSION)? == 0x0103,
        le16(prefix, C_FOLDERS)? >= 1,
        le16(prefix, FLAGS)? <= 7,
    );
    Some(Label::Cab)
}

/// Google Chrome extension (`Cr24`): version 3 with nonzero header length at `@8`, or version 2
/// with public-key and signature lengths in `1..=65536` at `@8` and `@12`.
pub(crate) fn crx(prefix: &[u8]) -> Option<Label> {
    static CRX_2: Subformat = Subformat::new("CRX2");
    static CRX_3: Subformat = Subformat::new("CRX3");
    const MAGIC: &[u8] = b"Cr24"; // @0..4: CRX magic
    const VERSION: usize = 4; // @4: u32 LE — CRX format version (2 or 3)
    const LEN_A: usize = 8; // @8: u32 LE — CRX3 header length or CRX2 public key length
    const LEN_B: usize = 12; // @12: u32 LE — CRX2 signature length
    const MIN_LEN: usize = 12;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    let version = le32(prefix, VERSION)?;
    let valid = match version {
        3 => le32(prefix, LEN_A)? >= 1,
        2 => {
            prefix.len() >= 16
                && (1..=65536).contains(&le32(prefix, LEN_A)?)
                && (1..=65536).contains(&le32(prefix, LEN_B)?)
        }
        _ => false,
    };
    require!(valid);
    with_subformat(Label::Crx, if version == 3 { &CRX_3 } else { &CRX_2 })
}

/// Microsoft `compress.exe` SZDD archive: `"SZDD\x88\xf0\x27\x33"` magic, compression mode `'A'`
/// or `'B'` at `@8`, and 14-byte header.
pub(crate) fn mscompress(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"SZDD\x88\xf0\x27\x33"; // @0..8: SZDD signature
    const MODE: usize = 8; // @8: u8 — compression mode ('A' or 'B')
    const HEADER_LEN: usize = 14;

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, MODE)?, b'A' | b'B'),
    );
    Some(Label::Mscompress)
}

/// RPM package: 96-byte lead (`ED AB EE DB`, major version `3 | 4`, minor `0`, type `0 | 1` at `@6`)
/// followed at `@96` by the 16-byte signature header magic (`8E AD E8 01 00 00 00 00`).
pub(crate) fn rpm(prefix: &[u8]) -> Option<Label> {
    static RPM_V3_BINARY: Subformat = Subformat::with_ids("RPM v3 binary package", "fmt/795", None);
    static RPM_V3_SOURCE: Subformat = Subformat::with_ids("RPM v3 source package", "fmt/795", None);
    static RPM_V4_BINARY: Subformat = Subformat::new("RPM v4 binary package");
    static RPM_V4_SOURCE: Subformat = Subformat::new("RPM v4 source package");
    const LEAD_MAGIC: &[u8] = &[0xED, 0xAB, 0xEE, 0xDB]; // @0..4: RPM lead magic
    const MAJOR: usize = 4; // @4: u8 — major version (3 or 4)
    const MINOR: usize = 5; // @5: u8 — minor version (0)
    const PACKAGE_TYPE: usize = 6; // @6: u16 BE — 0 (binary) or 1 (source)
    const SIG_HEADER: &[u8] = &[0x8E, 0xAD, 0xE8, 0x01, 0x00, 0x00, 0x00, 0x00]; // @96..104: header v1 magic + reserved
    const SIG_OFFSET: usize = 96; // @96: end of 96-byte RPM lead
    const MIN_LEN: usize = 112; // 96-byte lead + 16-byte signature header

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, LEAD_MAGIC),
        matches!(u8_at(prefix, MAJOR)?, 3 | 4),
        u8_at(prefix, MINOR)? == 0,
        be16(prefix, PACKAGE_TYPE)? <= 1,
        eq_at(prefix, SIG_OFFSET, SIG_HEADER),
    );
    let major = u8_at(prefix, MAJOR)?;
    let is_source = be16(prefix, PACKAGE_TYPE)? == 1;
    let sub = match (major, is_source) {
        (3, false) => &RPM_V3_BINARY,
        (3, true) => &RPM_V3_SOURCE,
        (_, false) => &RPM_V4_BINARY,
        (_, true) => &RPM_V4_SOURCE,
    };
    with_subformat(Label::Rpm, sub)
}

/// bzip2 compressed stream: `"BZh"` + block-size digit `'1'..='9'`, followed at `@4` by either the
/// 6-byte block header magic (`0x314159265359`, π in BCD, total prefix ≥ 20) or the 10-byte empty
/// end-of-stream marker (`0x177245385090`, √π in BCD, plus 4 zero CRC bytes, total prefix ≥ 14).
pub(crate) fn bzip(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"BZh"; // @0..3: bzip2 stream magic + 'h' (Huffman)
    const LEVEL: usize = 3; // @3: u8 — block size '1'..='9' (100..900 KiB)
    const PAYLOAD: usize = 4; // @4: first block or end-of-stream header
    // BCD digits of π (3.14159265359) — kept in hex because ASCII "1AY&SY" is coincidental.
    const BLOCK_MAGIC: &[u8] = &[0x31, 0x41, 0x59, 0x26, 0x53, 0x59];
    // BCD digits of √π (1.77245385090) followed by 4 zero stream-CRC bytes.
    const EMPTY_STREAM: &[u8] = bytes!(&[0x17, 0x72, 0x45, 0x38, 0x50, 0x90], &0u32.to_be_bytes());
    const MIN_EMPTY_LEN: usize = 14;
    const MIN_BLOCK_LEN: usize = 20;

    require!(
        prefix.len() >= MIN_EMPTY_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, LEVEL)?, b'1'..=b'9'),
        eq_at(prefix, PAYLOAD, EMPTY_STREAM)
            || (prefix.len() >= MIN_BLOCK_LEN
                && eq_at(prefix, PAYLOAD, BLOCK_MAGIC)
                && (u8_at(prefix, 14)? & 0x7F) <= 6),
    );
    Some(Label::Bzip)
}

/// RFC 1952 gzip member: magic `1F 8B`, compression method `08` (DEFLATE), reserved flag bits
/// `5..7` clear at `@3` (`<= 31`), and at least a 20-byte member (10-byte header + 2-byte minimum
/// deflate block + 8-byte trailer).
pub(crate) fn gzip(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = &[0x1F, 0x8B, 0x08]; // @0..3: ID1=0x1F, ID2=0x8B, CM=8 (DEFLATE)
    const FLG: usize = 3; // @3: u8 — member flags (bits 5..7 reserved zero → <= 0x1F)
    const MIN_LEN: usize = 20; // 10-byte header + 2-byte deflate block + 8-byte CRC32/ISIZE trailer

    require!(prefix.len() >= MIN_LEN, starts(prefix, HEADER), u8_at(prefix, FLG)? <= 0x1F,);
    Some(Label::Gzip)
}

/// Unix `compress` (LZW, `.Z`): magic `1F 9D` at `@0`, maxbits `9..=16` in bits `0..4` of `@2`,
/// reserved bits `5..6` clear, optional block-mode bit `7` (`0x80`).
pub(crate) fn unixcompress(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = &[0x1F, 0x9D]; // @0..2: Unix compress LZW magic
    const FLAGS: usize = 2; // @2: u8 — bit 7 = block mode, bits 6..5 = 0, bits 4..0 = maxbits (9..=16)
    const MIN_LEN: usize = 8;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        (9..=16).contains(&(u8_at(prefix, FLAGS)? & !0x80)),
    );
    Some(Label::Unixcompress)
}

/// WinACE archive (`"**ACE**"` at `@7`): header size ≥ 27 at `@2`, header type `0` at `@4`, even
/// header flags at `@5`, and at least 31 bytes in the prefix.
pub(crate) fn ace(prefix: &[u8]) -> Option<Label> {
    const HDR_SIZE: usize = 2; // @2: u16 LE — header size after the 4-byte CRC+size prefix (≥ 27)
    const HDR_TYPE: usize = 4; // @4: u8 — 0 = MAIN_BLK
    const HDR_FLAGS: usize = 5; // @5: u8 — bit 0 (extendable) must be 0 on MAIN_BLK
    const MAGIC_OFFSET: usize = 7; // @7..14: "**ACE**"
    const MAGIC: &[u8] = b"**ACE**";
    const MIN_LEN: usize = 31;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, MAGIC_OFFSET, MAGIC),
        le16(prefix, HDR_SIZE)? >= 27,
        u8_at(prefix, HDR_TYPE)? == 0,
        u8_at(prefix, HDR_FLAGS)? % 2 == 0,
    );
    Some(Label::Ace)
}

/// Tukaani XZ compressed stream (`xz`, 2009): 6-byte magic `\xFD7zXZ\x00` at `@0`,
/// stream flags `\x00` at `@6` and `<= 0x0F` at `@7`.
pub(crate) fn xz(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_XZ_SIZE: u64 = 24; // 12B Stream Header + 12B Stream Footer
    const XZ_MAGIC: &[u8] = bytes!(&[0xFD], b"7zXZ", &[0x00]);

    require!(
        size >= MIN_XZ_SIZE,
        prefix.len() >= 12,
        starts(prefix, XZ_MAGIC),
        u8_at(prefix, 6)? == 0,
        u8_at(prefix, 7)? <= 0x0F,
    );
    Some(Label::Xz)
}

/// IETF RFC 1950 ZLIB compressed stream (`zlibstream`, 1996): CMF byte (`CM == 8`,
/// `CINFO <= 7`, typically `0x78`) at `@0`, FLG byte at `@1` with `FDICT == 0` and
/// `(CMF * 256 + FLG) % 31 == 0`, valid Deflate first block type (`BTYPE != 3`), and
/// non-ASCII binary stream payload.
pub(crate) fn zlibstream(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_ZLIB_SIZE: u64 = 8; // 2B header + >=2B Deflate block + 4B Adler-32

    require!(size >= MIN_ZLIB_SIZE, prefix.len() >= 8);
    let cmf = u8_at(prefix, 0)?;
    let flg = u8_at(prefix, 1)?;
    // Restrict to standard 32 KiB window (0x78) with standard FLEVELs (0x01, 0x5E, 0x9C, 0xDA)
    require!(
        cmf == 0x78,
        matches!(flg, 0x01 | 0x5E | 0x9C | 0xDA),
        ((cmf as u16) * 256 + (flg as u16)).is_multiple_of(31),
        ((u8_at(prefix, 2)? >> 1) & 0x03) != 0b11,
    );
    Some(Label::Zlibstream)
}

/// Haruyasu Yoshizaki LHA / LHarc Archive (`lha`, 1988): method tag `"-lh0-"`..`"-lhe-"`,
/// `"-lh -"`, `"-lzs-"`, `"-lz2-"`..`"-lz8-"`, `"-pm0-"`..`"-pm2-"`, or `"-pc1-"` at `@2..7` and
/// header level `0..=3` at `@20`.
pub(crate) fn lha(prefix: &[u8], size: u64) -> Option<Label> {
    const LHA_METHODS: &[&[u8; 5]] = &[
        b"-lh0-", b"-lh1-", b"-lh2-", b"-lh3-", b"-lh4-", b"-lh5-", b"-lh6-", b"-lh7-", b"-lh8-",
        b"-lh9-", b"-lha-", b"-lhb-", b"-lhc-", b"-lhd-", b"-lhe-", b"-lh -", b"-lzs-", b"-lz2-",
        b"-lz3-", b"-lz4-", b"-lz5-", b"-lz7-", b"-lz8-", b"-pm0-", b"-pm1-", b"-pm2-", b"-pc1-",
    ];
    require!(
        size >= 22,
        prefix.len() >= 22,
        u8_at(prefix, 2)? == b'-',
        u8_at(prefix, 6)? == b'-',
        u8_at(prefix, 20)? <= 3,
    );
    let method = get_slice(prefix, 2, 7)?;
    require!(LHA_METHODS.iter().any(|&m| m == method));
    Some(Label::Lha)
}

/// POSIX.1-1988 `ustar` / GNU / V7 Unix Tape Archive (`tar`, 1979): 512-byte header block
/// with an ASCII graphic filename start at `@0`, optional `"ustar\0"` or `"ustar  \0"` magic
/// at `@257`, and an octal header checksum at `@148..156` matching the unsigned or signed
/// byte sum of `@0..512`.
pub(crate) fn tar(prefix: &[u8], size: u64) -> Option<Label> {
    const TAR_USTAR_POSIX_MAGIC: &[u8; 6] = b"ustar\0";
    const TAR_USTAR_GNU_MAGIC: &[u8; 8] = b"ustar  \0";

    require!(size >= 512, prefix.len() >= 512, u8_at(prefix, 0)?.is_ascii_graphic());
    let _is_ustar =
        eq_at(prefix, 257, TAR_USTAR_POSIX_MAGIC) || eq_at(prefix, 257, TAR_USTAR_GNU_MAGIC);
    let chk_field = get_slice(prefix, 148, 156)?;
    let mut oct_val = 0u32;
    let mut saw_digit = false;
    for &b in chk_field {
        if matches!(b, b'0'..=b'7') {
            oct_val = (oct_val << 3) | ((b - b'0') as u32);
            saw_digit = true;
        } else if saw_digit && matches!(b, 0 | b' ') {
            break;
        } else if !matches!(b, b' ' | 0) {
            return None;
        }
    }
    require!(saw_digit && oct_val > 0);
    let hdr = prefix.get(0..512)?;
    let mut unsigned_sum = 0u32;
    let mut signed_sum = 0i32;
    for (idx, &b) in hdr.iter().enumerate() {
        let v = if (148..156).contains(&idx) { b' ' } else { b };
        unsigned_sum += v as u32;
        signed_sum += (v as i8) as i32;
    }
    require!(oct_val == unsigned_sum || oct_val == (signed_sum as u32));
    Some(Label::Tar)
}

/// Kamila Szewczyk BZip3 compressed stream (`bzip3`, 2022): `"BZ3v1"` magic at `@0..5` followed by
/// `u32 LE` block size in `64 KiB..=512 MiB` at `@5..9`.
pub(crate) fn bzip3(prefix: &[u8], size: u64) -> Option<Label> {
    const BZIP3_V1_MAGIC: &[u8; 5] = b"BZ3v1";

    require!(
        size >= 9,
        prefix.len() >= 9,
        starts(prefix, BZIP3_V1_MAGIC),
        (65_536..=536_870_912).contains(&le32(prefix, 5)?),
    );
    Some(Label::Bzip3)
}

/// Jonathan Forbes & Tomi Poutanen Amiga LZX archive (`lzx`, 1995): `"LZX\0"` header magic at
/// `@0..4` and flags/attribute byte `<= 20` at `@6`.
pub(crate) fn lzx(prefix: &[u8], size: u64) -> Option<Label> {
    const LZX_MAGIC: &[u8; 4] = b"LZX\x00";

    require!(size >= 32, prefix.len() >= 16, starts(prefix, LZX_MAGIC), u8_at(prefix, 6)? <= 20,);
    Some(Label::Lzx)
}

/// Robert K. Jung ARJ archive (`arj`, 1991): `60 EA` header ID at `@0..2`, basic header size
/// `30..=2600` LE at `@2..4`, first header size `30..=basic_hdr_size` at `@4`, and file type `<= 5` at `@10`.
pub(crate) fn arj(prefix: &[u8], size: u64) -> Option<Label> {
    const ARJ_MAGIC: &[u8] = &[0x60, 0xEA];

    require!(size >= 12, prefix.len() >= 12, starts(prefix, ARJ_MAGIC));
    let basic_hdr_size = le16(prefix, 2)? as usize;
    let first_hdr_size = u8_at(prefix, 4)? as usize;
    let file_type = u8_at(prefix, 10)?;
    require!(
        (30..=2600).contains(&basic_hdr_size),
        (30..=basic_hdr_size).contains(&first_hdr_size),
        file_type <= 5,
    );
    Some(Label::Arj)
}

/// System Enhancement Associates ARC archive (`arc`, 1985): `0x1A` archive marker at `@0`,
/// compression method `1..=9` at `@1`, NUL-terminated printable ASCII filename at `@2..15`, and
/// bounded compressed member size at `@15..19`.
pub(crate) fn arc(prefix: &[u8], size: u64) -> Option<Label> {
    const ARC_MARKER: u8 = 0x1A;

    require!(
        size >= 29,
        prefix.len() >= 29,
        u8_at(prefix, 0)? == ARC_MARKER,
        (1..=9).contains(&u8_at(prefix, 1)?),
    );
    let name = get_slice(prefix, 2, 15)?;
    let nul = name.iter().position(|&b| b == 0)?;
    require!((1..=12).contains(&nul), name[..nul].iter().all(|&b| (0x21..=0x7E).contains(&b)),);
    let comp_size = le32(prefix, 15)? as u64;
    require!(29 + comp_size <= size);
    Some(Label::Arc)
}
