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

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const ALZIP_V1_MAGIC: &[u8; 4] = bytes!(b"ALZ", &[1]);
const ALZIP_V2_MAGIC: &[u8; 4] = bytes!(b"ALZ", &[2]);
const AVAST_VPU_FIELD1_OFFSET: usize = 16;
const AVAST_VPU_FIELD2_OFFSET: usize = 20;
const AVAST_VPU_MAGIC: &[u8; 16] = b"ASWsetupFPkgFile";
const AVAST_VPU_ZLIB_OFFSET: usize = 24;
const BIOWARE_ERF_UTF16_MAGIC: &[u8; 10] = hex2bin!("45 00 52 00 46 00 20 00 56 00");
const BIOWARE_ERF_V20: &[u8; 6] = hex2bin!("32 00 2E 00 30 00");
const BIOWARE_ERF_V30: &[u8; 6] = hex2bin!("33 00 2E 00 30 00");
const BSD_DUMP_BLOCK_SIZE: usize = 1024;
const BSD_DUMP_CHECKSUM_TARGET: u32 = 84446;
const BSD_DUMP_MAGIC_OFFSET: usize = 24;
const BSD_DUMP_NFS_MAGIC: u32 = 60012;
const BSD_DUMP_OFS_MAGIC: u32 = 60011;
const BSD_DUMP_TYPE_OFFSET: usize = 0;
const BSD_DUMP_UFS2_MAGIC: u32 = 0x1954_0119;
const BSD_DUMP_VOLUME_OFFSET: usize = 12;
const CPM_LBR_DIR_ENTRY_MAGIC: &[u8; 12] = b"\x00           ";
const EMTEX_FLI_MAGIC: &[u8; 4] = b"FLIB";
const FAIRYGUI_COMP_OFFSET: usize = 8;
const FAIRYGUI_ID_LEN_OFFSET: usize = 9;
const FAIRYGUI_MAGIC: &[u8; 4] = b"FGUI";
const FAIRYGUI_VERSION_OFFSET: usize = 4;
const FIFA_FIFAMOD_MAGIC: &[u8; 4] = b"FETM";
const FREEARC_MAGIC: &[u8; 4] = bytes!(b"ArC", &[1]);
const FROMSOFT_DCX_MAGIC: &[u8; 4] = b"DCX\0";
const FROSTY_DATA_OFF_OFFSET: usize = 12;
const FROSTY_FBMOD_MAGIC: &[u8; 8] = bytes!(b"FROSTY", &[0x00, 0x01]);
const FROSTY_VERSION_OFFSET: usize = 8;
const GAMEMAKER_GMK_MAGIC: &[u8; 4] = hex2bin!("91 D5 12 00");
const GBA_GBFS_DIR_NMUL_OFFSET: usize = 22;
const GBA_GBFS_DIR_OFF_OFFSET: usize = 20;
const GBA_GBFS_MAGIC: &[u8; 16] = bytes!(b"PinEightGBFS", &[0x0D, 0x0A, 0x1A, 0x0A]);
const GBA_GBFS_TOTAL_LEN_OFFSET: usize = 16;
const HALO_BLF_MAGIC: &[u8; 4] = b"_blf";
const LARIAN_LSPK_MAGIC: &[u8; 4] = b"LSPK";
const MARMALADE_DIRS_OFFSET: usize = 6;
const MARMALADE_DZ_MAGIC: &[u8; 4] = b"DTRZ";
const MARMALADE_FILES_OFFSET: usize = 4;
const MAXIS_DBPF_MAGIC: &[u8; 4] = b"DBPF";
const MIN_1C_ENTERPRISE_CF_SIZE: u64 = 36;
const MIN_2DKGT2_DEMO_SIZE: u64 = 32;
const MIN_ABCOMP_SIZE: u64 = 52;
const MIN_AIDAIM_SFS_SIZE: u64 = 32;
const MIN_ALZIP_SIZE: u64 = 12;
const MIN_AMG_ARCHIVE_SIZE: u64 = 46;
const MIN_AMIGA_IMPLODER_SIZE: u64 = 50;
const MIN_AMIGA_SQUASH_SIZE: u64 = 12;
const MIN_ANC_CRUNCHER_SIZE: u64 = 16;
const MIN_APIS_IQ_ICE_SIZE: u64 = 32;
const MIN_APPLE_AEA_SIZE: u64 = 32;
const MIN_APPLE_ARCHIVE_SIZE: u64 = 12;
const MIN_APPLE_BINARY_II_SIZE: u64 = 128;
const MIN_APPLE_MKEXT_SIZE: u64 = 44;
const MIN_ARCHIVE_HASH_RTM_SIZE: u64 = 32;
const MIN_ARCV_SIZE: u64 = 16;
const MIN_ASHAMPOO_ASHLANG_SIZE: u64 = 36;
const MIN_ASSETTOCORSA_ACD_SIZE: u64 = 48;
const MIN_ASZ_PACKAGE_SIZE: u64 = 24;
const MIN_ATOMIK_CRUNCHER_SIZE: u64 = 16;
const MIN_AVAST_VPU_SIZE: u64 = 32;
const MIN_BCL_COMPRESSED_SIZE: u64 = 13;
const MIN_BETHESDA_ARCHIVE_SIZE: u64 = 24;
const MIN_BIOWARE_BIFF_SIZE: u64 = 20;
const MIN_BIOWARE_RIM_SIZE: u64 = 32;
const MIN_BITDEFENDER_AVXS_SIZE: u64 = 24;
const MIN_BLACKDRAGON_ZL_SIZE: u64 = 36;
const MIN_BLACKHOLE_ARCHIVE_SIZE: u64 = 38;
const MIN_BLIZZARD_WOO_TBL_SIZE: u64 = 28;
const MIN_BLUEBYTE_TPWM_SIZE: u64 = 12;
const MIN_BNS_DIALOG_CBF_SIZE: u64 = 64;
const MIN_BSDIFF_SIZE: u64 = 32;
const MIN_BSD_DUMP_SIZE: u64 = BSD_DUMP_BLOCK_SIZE as u64;
const MIN_CAPCOM_RE_PAK_SIZE: u64 = 32;
const MIN_CARV_VRAC_SIZE: u64 = 28;
const MIN_CASIO_CONTAINER_SIZE: u64 = 64;
const MIN_CAZIP_SIZE: u64 = 16;
const MIN_CFL3_SIZE: u64 = 16;
const MIN_CIGAMES_DPK4_SIZE: u64 = 20;
const MIN_CLICKTEAM_MFA_SIZE: u64 = 32;
const MIN_CMAP_CMP_SIZE: u64 = 16;
const MIN_COCOS2D_CCZ_SIZE: u64 = 20;
const MIN_COD_FASTFILE_SIZE: u64 = 24;
const MIN_COKTEL_STK2_SIZE: u64 = 28;
const MIN_CPM_LBR_SIZE: u64 = 128;
const MIN_CRIWARE_CPK_SIZE: u64 = 64;
const MIN_CRUNCH_MANIA_SIZE: u64 = 16;
const MIN_CRUSH_SIZE: u64 = 26;
const MIN_CRYENGINE_SOC_SIZE: u64 = 32;
const MIN_CRYO_BIGFILE_SIZE: u64 = 32;
const MIN_CRYO_CLIB_VOX_SIZE: u64 = 28;
const MIN_CTRADER_ALGO_SIZE: u64 = 20;
const MIN_CYAN_MOHAWK_SIZE: u64 = 28;
const MIN_DAGOR_DXP_SIZE: u64 = 48;
const MIN_DAGOR_GRP_SIZE: u64 = 48;
const MIN_DAGOR_VROMFS_SIZE: u64 = 24;
const MIN_DAN1_ARCHIVE_SIZE: u64 = 32;
const MIN_DELTA_RPM_SIZE: u64 = 32;
const MIN_DESCENT_HOG_SIZE: u64 = 20;
const MIN_DIFSTREAM_SVF_SIZE: u64 = 44;
const MIN_EA_ABK_SIZE: u64 = 36;
const MIN_EA_VIV4_BH_SIZE: u64 = 28;
const MIN_ECM_SIZE: u64 = 13;
const MIN_EDI_LZSS_SIZE: u64 = 16;
const MIN_EGG_ARCHIVE_SIZE: u64 = 14;
const MIN_EMTEX_FLI_SIZE: u64 = 32;
const MIN_ENSEMBLE_DRS_SIZE: u64 = 64;
const MIN_EXBN_EBN_SIZE: u64 = 24;
const MIN_FAIRYGUI_FUI_SIZE: u64 = 32;
const MIN_FASTGRAPH_FGI_SIZE: u64 = 96;
const MIN_FBCHUNKS_NGP_SIZE: u64 = 24;
const MIN_FIFA_FIFAMOD_SIZE: u64 = 32;
const MIN_FIGHTER_MAKER_KGT_SIZE: u64 = 32;
const MIN_FM_FMF_SIZE: u64 = 36;
const MIN_FREEARC_SIZE: u64 = 8;
const MIN_FROMSOFT_BINDER_SIZE: u64 = 48;
const MIN_FROMSOFT_DCX_SIZE: u64 = 80;
const MIN_FROSTY_FBMOD_SIZE: u64 = 32;
const MIN_GAMEMAKER_GMK_SIZE: u64 = 32;
const MIN_GBA_GBFS_SIZE: u64 = 64;
const MIN_GCA_ARCHIVE_SIZE: u64 = 8;
const MIN_GEA_ARCHIVE_SIZE: u64 = 36;
const MIN_GMOD_GMA_SIZE: u64 = 28;
const MIN_GMOTOR_C8CF_MAS_SIZE: u64 = 64;
const MIN_GODOT_GDEC_SIZE: u64 = 44;
const MIN_GODOT_PCK_SIZE: u64 = 96;
const MIN_HALO_BLF_SIZE: u64 = 60;
const MIN_HA_ARCHIVE_SIZE: u64 = 22;
const MIN_HDIFFPATCH_SIZE: u64 = 14;
const MIN_HHD_LPACK_SIZE: u64 = 32;
const MIN_HIGHWAY_PURSUIT_HPDT_SIZE: u64 = 16;
const MIN_HYPER_ARCHIVE_SIZE: u64 = 24;
const MIN_IBM_MRNZ_SIZE: u64 = 14;
const MIN_IBM_SAVF_SIZE: u64 = 1056;
const MIN_IBM_TERSE_COMPAT_SIZE: u64 = 1024;
const MIN_IBM_TERSE_NATIVE_SIZE: u64 = 16;
const MIN_ICY_TOWER_ITR_SIZE: u64 = 32;
const MIN_ID_TED5_SIZE: u64 = 10;
const MIN_IEBCOPY_PDS_SIZE: u64 = 340;
const MIN_IMP_ARCHIVE_SIZE: u64 = 42;
const MIN_INSTALLER_VISE_XYZ_SIZE: u64 = 24;
const MIN_INSTALLSHIELD_IBT_SIZE: u64 = 38;
const MIN_INSTALLSHIELD_PKG_SIZE: u64 = 32;
const MIN_INSTALLSHIELD_V3_SIZE: u64 = 256;
const MIN_ISI_FRAC_CAR_SIZE: u64 = 36;
const MIN_ISI_GMOTOR_MAS_SIZE: u64 = 40;
const MIN_IW4X_HEAT_SIZE: u64 = 32;
const MIN_JAM_PACKER_SIZE: u64 = 12;
const MIN_JFX1_SIZE: u64 = 16;
const MIN_JGXP_PACKAGE_SIZE: u64 = 32;
const MIN_JSOS_MD8_SIZE: u64 = 48;
const MIN_JWLAUNCH_P2_SIZE: u64 = 44;
const MIN_KASPERSKY_UPDS_SIZE: u64 = 24;
const MIN_KD_RED_SIZE: u64 = 64;
const MIN_KGB_SIZE: u64 = 16;
const MIN_KILL_LA_KILL_DGKP_SIZE: u64 = 32;
const MIN_KIRIKIRI_XP3_SIZE: u64 = 32;
const MIN_KOEI_LS11_S11_SIZE: u64 = 48;
const MIN_KOHAN_SND_SIZE: u64 = 32;
const MIN_KROM_MORK_RES_SIZE: u64 = 48;
const MIN_LARIAN_LSPK_SIZE: u64 = 32;
const MIN_LIBLZG_SIZE: u64 = 17;
const MIN_LITHTECH_REZ_SIZE: u64 = 128;
const MIN_LOB_MULTIPAK_SIZE: u64 = 16;
const MIN_LOGITECH_COMPRESS_SIZE: u64 = 16;
const MIN_LOTUS_CMZ_SIZE: u64 = 22;
const MIN_LUCASARTS_GOB_SIZE: u64 = 12;
const MIN_LZF_SIZE: u64 = 7;
const MIN_LZM_COMPRESSED_SIZE: u64 = 16;
const MIN_LZP1_SIZE: u64 = 8;
const MIN_LZSA_SIZE: u64 = 6;
const MIN_MARMALADE_DZ_SIZE: u64 = 24;
const MIN_MAXIS_DBPF_SIZE: u64 = 96;
const MIN_MDCD_ARCHIVE_SIZE: u64 = 26;
const MIN_MINDWARE_MFS_SIZE: u64 = 32;
const MIN_MIR_MAP_SIZE: u64 = 32;
const MIN_MMCMP_SIZE: u64 = 24;
const MIN_MOZLZ4_SIZE: u64 = 13;
const MIN_MPQ_SIZE: u64 = 64;
const MIN_MSDELTA_PATCH_SIZE: u64 = 12;
const MIN_MSTOR_IBX_SIZE: u64 = 32;
const MIN_NBA2K_CMG_SIZE: u64 = 64;
const MIN_NCBI_SRA_SIZE: u64 = 16;
const MIN_NINTENDO_BCRES_SIZE: u64 = 64;
const MIN_NINTENDO_BFRES_SIZE: u64 = 64;
const MIN_NINTENDO_NARC_SIZE: u64 = 44;
const MIN_NINTENDO_RARC_SIZE: u64 = 64;
const MIN_NINTENDO_SARC_SIZE: u64 = 32;
const MIN_NINTENDO_U8_SIZE: u64 = 44;
const MIN_NITROPLUS_NPA_SIZE: u64 = 32;
const MIN_NOS_INSTALLER_ARCHIVE_SIZE: u64 = 32;
const MIN_OMEGA_LOGIC_CORK_SIZE: u64 = 34;
const MIN_OS2_FTCOMP_SIZE: u64 = 48;
const MIN_OS2_PACK_SIZE: u64 = 32;
const MIN_PACK_ICE_SIZE: u64 = 16;
const MIN_PANGOLIN_PRR_SIZE: u64 = 32;
const MIN_PAQ_ARCHIVE_SIZE: u64 = 16;
const MIN_PAR1_SIZE: u64 = 96;
const MIN_PAR2_SIZE: u64 = 64;
const MIN_PARSONS_WRK_SIZE: u64 = 32;
const MIN_PAZLIGSAW_PZL_SIZE: u64 = 32;
const MIN_PC88_T88_SIZE: u64 = 32;
const MIN_PCX_LIBRARY_SIZE: u64 = 64;
const MIN_PDP10_FAILSAFE_SIZE: u64 = 25;
const MIN_PEA_ARCHIVE_SIZE: u64 = 10;
const MIN_PFPART3_SIZE: u64 = 32;
const MIN_PLAYSTATION_3_PKG_SIZE: u64 = 192;
const MIN_PLAYSTATION_4_PKG_SIZE: u64 = 256;
const MIN_PMC_CRUNCHER_SIZE: u64 = 16;
const MIN_PODBOT_PWF_SIZE: u64 = 32;
const MIN_POKEMON_GARC_SIZE: u64 = 48;
const MIN_POKEMON_GFPAK_SIZE: u64 = 48;
const MIN_POPSTARTER_VCD_SIZE: u64 = 64;
const MIN_POWERBUILDER_PBD_SIZE: u64 = 36;
const MIN_POWERPACKER_SIZE: u64 = 16;
const MIN_PROXMOX_PBS_SIZE: u64 = 64;
const MIN_PROX_USB_XHA_SIZE: u64 = 40;
const MIN_PUFFIN_PATCH_SIZE: u64 = 20;
const MIN_PXHG_PACKAGE_SIZE: u64 = 24;
const MIN_RADICAL_RCF_SIZE: u64 = 60;
const MIN_RAGE_RSC7_SIZE: u64 = 32;
const MIN_RCBOT_RCW_SIZE: u64 = 32;
const MIN_REBELLION_ASURA_SIZE: u64 = 28;
const MIN_REDENGINE_RDAR_SIZE: u64 = 40;
const MIN_REFPACK_BIG_SIZE: u64 = 24;
const MIN_RELIC_ARCHIVE_SIZE: u64 = 32;
const MIN_RELIC_RBH_SIZE: u64 = 36;
const MIN_RENPY_RPA_SIZE: u64 = 34;
const MIN_RESCSEG_CMF_SIZE: u64 = 32;
const MIN_RESFILE_RSC_SIZE: u64 = 28;
const MIN_RFZPACK_RFZ_SIZE: u64 = 28;
const MIN_RGSSAD_SIZE: u64 = 12;
const MIN_RNC_SIZE: u64 = 18;
const MIN_ROCKSTAR_RPF_SIZE: u64 = 32;
const MIN_RPGMV_SIZE: u64 = 32;
const MIN_RPG_WOLF_SIZE: u64 = 48;
const MIN_SAR_CAR_SIZE: u64 = 32;
const MIN_SCHINDLER_SZIP_SIZE: u64 = 8;
const MIN_SCRATCH_V1_SIZE: u64 = 28;
const MIN_SCS_HASHFS_SIZE: u64 = 32;
const MIN_SCUMM_ROOM_LFL_SIZE: u64 = 32;
const MIN_SC_REP_SIZE: u64 = 36;
const MIN_SEGA_AFS_SIZE: u64 = 16;
const MIN_SEGA_PVM_SIZE: u64 = 14;
const MIN_SENJUTSU_S7_SIZE: u64 = 24;
const MIN_SERPENT_STONE_SIZE: u64 = 32;
const MIN_SETUP_SPECIALIST_TGCF_SIZE: u64 = 24;
const MIN_SIERRA_AGI_VOL_SIZE: u64 = 32;
const MIN_SIMS1_IFF_SIZE: u64 = 64;
const MIN_SISX_SIZE: u64 = 28;
const MIN_SLIM_ARCHIVE_SIZE: u64 = 34;
const MIN_SOFTLIB_SLB_SIZE: u64 = 32;
const MIN_SOFTMAKER_DWR_SIZE: u64 = 100;
const MIN_SOLARIS_PKG_SIZE: u64 = 48;
const MIN_SONY_PICTUREGEAR_PTS_SIZE: u64 = 36;
const MIN_SPIS_LH5_SIZE: u64 = 24;
const MIN_SQZ_ARCHIVE_SIZE: u64 = 9;
const MIN_SSBU_PARACOBN_PRC_SIZE: u64 = 32;
const MIN_SSI_GFF_ARCHIVE_SIZE: u64 = 28;
const MIN_STARBOUND_PAK_SIZE: u64 = 24;
const MIN_STEELMC_STLR_SRG_SIZE: u64 = 32;
const MIN_STOS_AMOS_BANK_SIZE: u64 = 16;
const MIN_SUN_XRS_SIZE: u64 = 28;
const MIN_SWITCH_PFS0_SIZE: u64 = 40;
const MIN_SYMBOLICS_CARRY_SIZE: u64 = 48;
const MIN_SYNOPSE_SYNLZ_SIZE: u64 = 28;
const MIN_TERRARIA_TMOD_SIZE: u64 = 40;
const MIN_THILMERA7_ZPAC_T7P_SIZE: u64 = 64;
const MIN_TIME_OF_WAR_D3E_SIZE: u64 = 512;
const MIN_TOPS10_BACKUP_SIZE: u64 = 2720;
const MIN_TOPS20_DUMPER_SIZE: u64 = 2590;
const MIN_TOTAL_WAR_PFH_SIZE: u64 = 28;
const MIN_TRAINZ_CDP_SIZE: u64 = 32;
const MIN_TZIP_COMPRESSED_SIZE: u64 = 16;
const MIN_UBISOFT_ANVIL_ACV_SIZE: u64 = 48;
const MIN_UBISOFT_JADE_BF_SIZE: u64 = 40;
const MIN_UHARC_SIZE: u64 = 10;
const MIN_UNITY_SERIALIZED_ASSETS_SIZE: u64 = 64;
const MIN_UNITY_WEBDATA_SIZE: u64 = 36;
const MIN_UNREAL_PACKAGE_SIZE: u64 = 64;
const MIN_UNREAL_UTOC_SIZE: u64 = 144;
const MIN_UNREAL_UZ2_SIZE: u64 = 24;
const MIN_VALVE_VPK_SIZE: u64 = 12;
const MIN_VIC2_HUFFMAN_SIZE: u64 = 12;
const MIN_VMS_LIB_SIZE: u64 = 512;
const MIN_WARCRAFT3_MAP_SIZE: u64 = 544;
const MIN_WATCOM_WPK_SIZE: u64 = 16;
const MIN_WEBBUNDLE_SIZE: u64 = 24;
const MIN_WELTENBAUER_WBP_SIZE: u64 = 48;
const MIN_WESTWOOD_CSF_SIZE: u64 = 32;
const MIN_WINCE_CAB_SIZE: u64 = 100;
const MIN_WITCHER2_DZIP_SIZE: u64 = 32;
const MIN_WITCHER3_BUNDLE_SIZE: u64 = 64;
const MIN_WOLFRPG_MAP_MPS_SIZE: u64 = 64;
const MIN_XBOX_BDES_XZP_SIZE: u64 = 28;
const MIN_XCR_ARCHIVE_SIZE: u64 = 16;
const MIN_XNA_COURSE_CRS_SIZE: u64 = 64;
const MIN_XNA_XNB_SIZE: u64 = 14;
const MIN_XPK_SIZE: u64 = 36;
const MIN_XPLAPP_PACKAGE_SIZE: u64 = 44;
const MIN_YURIS_YPF_SIZE: u64 = 36;
const MIN_YZ1_ARCHIVE_SIZE: u64 = 8;
const MIN_ZCHUNK_SIZE: u64 = 32;
const MIN_ZEDAL_LIBRARY_LV_SIZE: u64 = 512;
const MIN_ZIM_ARCHIVE_SIZE: u64 = 80;
const MIN_ZISOFS_FILE_SIZE: u64 = 20;
const MIN_ZOS_EBCDIC_AR_SIZE: u64 = 68;
const MIN_ZP1_ARCHIVE_SIZE: u64 = 28;
const MIN_ZPAQ_SIZE: u64 = 13;
const MIN_ZUCCHINI_PATCH_SIZE: u64 = 28;
const PDP10_FAILSAFE_W1_HI: &[u8; 4] = hex2bin!("2A 68 69 B3");
const PDP10_FAILSAFE_W2_HI: &[u8; 2] = hex2bin!("86 69");
const PDP10_WORD36_MASK: u64 = (1u64 << 36) - 1;
const POKEMON_GARC_MAGIC: &[u8; 4] = b"CRAG";
const RADICAL_ALIGN_OR_DIR_OFFSET: usize = 36;
const RADICAL_ATG_MAGIC: &[u8; 24] = b"ATG CORE CEMENT LIBRARY\0";
const RADICAL_BIG_ENDIAN_OFFSET: usize = 34;
const RADICAL_HEADER_POS_OFFSET: usize = 44;
const RADICAL_IDENT_PAD_START: usize = 24;
const RADICAL_RAD_MAGIC: &[u8; 24] = b"RADCORE CEMENT LIBRARY\0\0";
const RADICAL_VALID_FLAG_OFFSET: usize = 35;
const RADICAL_VERSION_MAJOR_OFFSET: usize = 32;
const RADICAL_VERSION_MINOR_OFFSET: usize = 33;
const RAGE_RSC7_GFX_FLAGS_OFFSET: usize = 12;
const RAGE_RSC7_MAGIC: &[u8; 4] = b"RSC7";
const RAGE_RSC7_SYS_FLAGS_OFFSET: usize = 8;
const RAGE_RSC7_VERSION_OFFSET: usize = 4;
const SOFTMAKER_DWR_BANNER: &[u8; 67] =
    b"(c) SoftMaker Software GmbH and its licensors. All rights reserved.";
const SOFTMAKER_DWR_TAG_OFFSET: usize = 80;
const SQZ_MAGIC: &[u8; 5] = b"HLSQZ";
const SYMBOLICS_BLOK_MARKER: &[u8; 9] = b"\x8dEND\x8dBLOK";
const SYMBOLICS_ELS_PREFIX: &[u8; 31] = b"EMBEDDED-LENGTH-STREAM VERSION ";
const TEX_PK_PREAMBLE: &[u8; 2] = hex2bin!("F7 59");
const TUANJIE_WEBDATA_10_MAGIC: &[u8; 18] = b"TuanjieWebData1.0\0";
const UHARC_PREFIX: &[u8; 3] = b"UHA";
const UNITY_WEBDATA_10_MAGIC: &[u8; 16] = b"UnityWebData1.0\0";
const VMS_LIB_MAGIC: &[u8; 4] = le32_bytes!(0x0DEC_2581);
const WARCRAFT3_MAP_MAGIC: &[u8; 4] = b"HM3W";
const WITCHER2_DZIP_MAGIC: &[u8; 4] = b"DZIP";
const XPK_MAGIC: &[u8; 4] = b"XPKF";

/// Checks whether a null-terminated slice at `ver_slice` starts with a valid Unity engine version (`<major>.<minor>.<patch>`).
fn is_valid_unity_version_str(ver_slice: &[u8]) -> bool {
    let Some(nul_pos) = ver_slice.iter().position(|&b| b == 0) else {
        return false;
    };
    let s = &ver_slice[..nul_pos];
    if !(5..=24).contains(&s.len()) || !s.iter().all(|&b| b.is_ascii_graphic()) {
        return false;
    }
    let mut parts = s.split(|&b| b == b'.');
    let Some(major) = parts.next() else {
        return false;
    };
    let Some(minor) = parts.next() else {
        return false;
    };
    let Some(patch) = parts.next() else {
        return false;
    };
    !major.is_empty()
        && major.len() <= 4
        && major.iter().all(|&b| b.is_ascii_digit())
        && !minor.is_empty()
        && minor.len() <= 2
        && minor.iter().all(|&b| b.is_ascii_digit())
        && !patch.is_empty()
        && patch[0].is_ascii_digit()
}

fn parse_fastgraph_fgi_entry(entry: &[u8]) -> Option<(u32, u32)> {
    let nul_pos = entry[..12].iter().position(|&b| b == 0)?;
    require!(nul_pos >= 3, entry[nul_pos..12].iter().all(|&b| b == 0));
    let name = &entry[..nul_pos];
    let dot_pos = name.iter().position(|&b| b == b'.')?;
    let ext_len = name.len() - dot_pos - 1;
    require!(
        (1..=8).contains(&dot_pos),
        (1..=3).contains(&ext_len),
        name.iter().enumerate().all(|(i, &b)| {
            i == dot_pos
                || b.is_ascii_alphanumeric()
                || matches!(b, b'_' | b'-' | b'$' | b'~' | b'!')
        })
    );
    let data_off = le32(entry, 12)?;
    let data_len = le32(entry, 16)?;
    let file_type = entry[20];
    let reserved = entry[21];
    let spr_width = le16(entry, 22)?;
    require!(
        data_off >= 72,
        data_len > 0,
        matches!(file_type, 0 | 1 | 3..=10),
        reserved == 0,
        if matches!(file_type, 3 | 4) { (1..=4096).contains(&spr_width) } else { spr_width == 0 }
    );
    Some((data_off, data_len))
}

/// 1C:Enterprise 8 Configuration Container (.cf).
pub(crate) fn verify_1c_enterprise_cf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_1C_ENTERPRISE_CF_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\xff\xff\xff\x7f\x00\x02\0\0"),
        le32(prefix, 12) == Some(0),
        get_slice(prefix, 16, 18)? == b"\r\n"
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::_1cEnterpriseCf)
}

/// 2D Fighter Maker 2nd (2DKGT2) Replay Demo (.demo, 2002) Enterbrain.
pub(crate) fn verify_2dkgt2_demo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_2DKGT2_DEMO_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"2DKGT2"),
        matches!(u8_at(prefix, 6)?, b'G' | b'K'),
        get_slice(prefix, 7, 16)? == b"\0\0\0\0\0\0\0\0\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::_2dkgt2Demo)
}

/// ABComp v2.x: `"\x03AB2"` with reserved zero words at `@46` and `@50`.
pub(crate) fn verify_abcomp(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = b"\x03AB2"; // @0: 0x03 + "AB2"
    const RESERVED_A: usize = 46; // @46: u16 == 0
    const RESERVED_B: usize = 50; // @50: u16 == 0
    require!(size >= MIN_ABCOMP_SIZE, starts(prefix, MAGIC));
    require!(le16(prefix, RESERVED_A)? == 0, le16(prefix, RESERVED_B)? == 0);
    Some(Label::Abcomp)
}

/// Acronis True Image Backup Container (`.tib`).
pub(crate) fn verify_acronis_tib(prefix: &[u8], size: u64) -> Option<Label> {
    const VOLUME_SIG: &[u8; 8] = bytes!(le32_bytes!(0xA2B9_24CE), &32u32.to_le_bytes());
    const LEGACY_SIG: &[u8; 8] = b"\x00sacronx";
    const CHUNK_SIG: &[u8; 4] = le32_bytes!(0x4467_6EB4);
    require!(
        size >= 16,
        prefix.len() >= 8,
        starts(prefix, VOLUME_SIG) || starts(prefix, LEGACY_SIG) || starts(prefix, CHUNK_SIG),
    );
    Some(Label::AcronisTib)
}

/// AidAim Software SingleFileSystem (SFS) Container (AAMVHFSS, .edb / .plr / .loader).
pub(crate) fn verify_aidaim_sfs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AIDAIM_SFS_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"AAMVHFSS\xfe\xff\xff\xff\xff\xff\xff\x7f"),
        get_slice(prefix, 16, 32)? == b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::AidaimSfs)
}

/// ESTsoft ALZip Archive (`.alz`, 1999) ESTsoft (`"ALZ\x01"` / `"ALZ\x02"`).
///
/// Header layout:
/// - `@0..4`: `"ALZ\x01"` or `"ALZ\x02"` archive magic
/// - `@4..6`: `u16 LE` format version (typically `0x000A`, non-zero)
/// - `@6..8`: `u16 LE` archive ID (typically `0x0000`)
/// - `@8..12`: first record tag — `"BLZ\x01"` (local file header), `"CLZ\x01"` (central directory), `"CLZ\x02"` (EOF), or `"DLZ\x01"` (comment)
pub(crate) fn verify_alzip(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFFSET: usize = 4; // @4..6: u16 LE ALZip archive version (typically 0x000A)
    const NEXT_SIG_OFFSET: usize = 8; // @8..12: next record signature ("BLZ\x01" / "CLZ\x01" / "CLZ\x02" / "DLZ\x01")
    const BLZ_LFH_MAGIC: &[u8; 4] = bytes!(b"BLZ", &[1]);
    const CLZ_CD_MAGIC: &[u8; 4] = bytes!(b"CLZ", &[1]);
    const CLZ_EOF_MAGIC: &[u8; 4] = bytes!(b"CLZ", &[2]);
    const DLZ_COMMENT_MAGIC: &[u8; 4] = bytes!(b"DLZ", &[1]);
    const HEADER_LEN: usize = MIN_ALZIP_SIZE as usize;

    require!(
        size >= MIN_ALZIP_SIZE,
        prefix.len() >= HEADER_LEN,
        starts(prefix, ALZIP_V1_MAGIC) || starts(prefix, ALZIP_V2_MAGIC),
        le16(prefix, VERSION_OFFSET)? > 0,
        eq_at(prefix, NEXT_SIG_OFFSET, BLZ_LFH_MAGIC)
            || eq_at(prefix, NEXT_SIG_OFFSET, CLZ_CD_MAGIC)
            || eq_at(prefix, NEXT_SIG_OFFSET, CLZ_EOF_MAGIC)
            || eq_at(prefix, NEXT_SIG_OFFSET, DLZ_COMMENT_MAGIC)
    );
    Some(Label::Alz)
}

/// Milen Georgiev AMG Compressed Archive (`.amg`, 1995 Milen Georgiev, `AD 36 22`).
pub(crate) fn verify_amg_archive(prefix: &[u8], size: u64) -> Option<Label> {
    const AMG_MAGIC: &[u8; 3] = hex2bin!("AD 36 22"); // @0..3
    require!(
        size >= MIN_AMG_ARCHIVE_SIZE,
        prefix.len() >= 45,
        starts(prefix, AMG_MAGIC),
        eq_at(prefix, 3, &[0u8; 9]),
        (0x21..=0x7E).contains(&u8_at(prefix, 33)?)
    );
    Some(Label::AmgArchive)
}

/// Amiga File Imploder Compressed File (IMP!).
pub(crate) fn verify_amiga_imploder(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_AMIGA_IMPLODER_SIZE && prefix.len() >= 12);
    let unpacked_size = be32(prefix, 4)? as u64;
    let packed_offset = be32(prefix, 8)? as u64;
    require!(
        (4..=0x0400_0000).contains(&unpacked_size) && packed_offset >= 12 && packed_offset <= size
    );
    Some(Label::Imploder)
}

/// Squash Amiga Data Compressor stream (`.sqsh`, 1991).
pub(crate) fn verify_amiga_squash(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AMIGA_SQUASH_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"SQSH"),
        be32(prefix, 4)? > 0,
        has_non_ascii_text_byte(prefix.get(4..12)?)
    );
    Some(Label::AmigaSquash)
}

/// ANC Cruncher Amiga Data Stream (`.anc` / `"FVL0"`, 1991).
pub(crate) fn verify_anc_cruncher(prefix: &[u8], size: u64) -> Option<Label> {
    const ANC_MAGIC_A: &[u8; 4] = hex2bin!("18 0A 00 18"); // @0..4: ANC Cruncher v1 magic
    const ANC_MAGIC_B: &[u8; 4] = hex2bin!("18 0A 00 02"); // @0..4: ANC Cruncher v2 magic
    const FVL0_MAGIC: &[u8; 4] = b"FVL0"; // @0..4: ANC Cruncher FVL0 header
    const ORIG_LEN_OFFSET: usize = 4; // @4..8: u32 BE uncompressed/payload size

    require!(size >= MIN_ANC_CRUNCHER_SIZE, prefix.len() >= 16);
    if starts(prefix, FVL0_MAGIC) {
        let payload_len = be32(prefix, ORIG_LEN_OFFSET)? as u64;
        require!(payload_len >= 8, payload_len.saturating_add(8) == size);
        return Some(Label::AncCruncher);
    }
    require!(
        starts(prefix, ANC_MAGIC_A) || starts(prefix, ANC_MAGIC_B),
        (16..=67_108_864).contains(&be32(prefix, ORIG_LEN_OFFSET)?)
    );
    Some(Label::AncCruncher)
}

/// APIS IQ-Software (IQ-FMEA / IQ-RM) Fridge Container (`.ice` / `.cfg`, v1–v4).
pub(crate) fn verify_apis_iq_ice(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APIS_IQ_ICE_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"{U>P<6} Fridge"),
        (1..=4).contains(&u8_at(prefix, 14)?),
        eq_at(prefix, 15, b"F\0\0\0 ")
    );
    Some(Label::ApisIqIce)
}

/// Apple Encrypted Archive / Signed Shortcut (.aea / .shortcut, 2021) Apple.
pub(crate) fn verify_apple_aea(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APPLE_AEA_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"AEA1\0\0\0\0"),
        le32(prefix, 8).unwrap_or(0) >= 16,
        get_slice(prefix, 12, 20)? == b"bplist00"
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::AppleAea)
}

/// Apple Archive (`.aar` / `.yaa`, 2020 Apple, `"AA01"` / `"YAA1"`).
pub(crate) fn verify_apple_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APPLE_ARCHIVE_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"AA01") || starts(prefix, b"YAA1"),
        le16(prefix, 4)? >= 6,
        eq_at(prefix, 6, b"TYP1")
    );
    Some(Label::AppleArchive)
}

/// Apple II Binary II Archive (`.bny` / `.bqy`, 1986 Gary Little, `0A 47 4C`).
pub(crate) fn verify_apple_binary_ii(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APPLE_BINARY_II_SIZE,
        size.is_multiple_of(128),
        prefix.len() >= 128,
        starts(prefix, b"\x0AGL"),
        u8_at(prefix, 18) == Some(0x02),
        (1..=64).contains(&u8_at(prefix, 23)?),
        u8_at(prefix, 109)? <= 0x0D,
        (0x21..=0x7E).contains(&u8_at(prefix, 24)?)
    );
    Some(Label::AppleBinaryIi)
}

/// Apple Darwin Multi-Kext Kernel Extension Archive (`.mkext`, `"MKXTMOSX"`).
pub(crate) fn verify_apple_mkext(prefix: &[u8], size: u64) -> Option<Label> {
    const MKEXT_MAGIC: &[u8; 8] = b"MKXTMOSX"; // @0..8: MKEXT_MAGIC ("MKXT") + MKEXT_SIGN ("MOSX")
    const LENGTH_OFF: usize = 8; // @8..12: u32 BE total archive length
    const ADLER32_OFF: usize = 12; // @12..16: u32 BE Adler-32 checksum (!= 0)
    const VERSION_OFF: usize = 16; // @16..20: u32 BE version (0x01008000 mkext1 or 0x02xxxxxx mkext2)
    const NUMKEXTS_OFF: usize = 20; // @20..24: u32 BE number of bundled kexts (1..=4096)
    require!(size >= MIN_APPLE_MKEXT_SIZE, prefix.len() >= 24, starts(prefix, MKEXT_MAGIC));
    let len = be32(prefix, LENGTH_OFF)? as u64;
    let adler = be32(prefix, ADLER32_OFF)?;
    let ver = be32(prefix, VERSION_OFF)?;
    let n_kexts = be32(prefix, NUMKEXTS_OFF)?;
    require!(
        (MIN_APPLE_MKEXT_SIZE..=size).contains(&len),
        adler != 0,
        ver == 0x0100_8000 || (ver >> 24) == 2,
        (1..=4096).contains(&n_kexts)
    );
    Some(Label::AppleMkext)
}

/// ARCHIVE# v2 Resource Package (.rtm).
pub(crate) fn verify_archive_hash_rtm(prefix: &[u8], size: u64) -> Option<Label> {
    let is_rtc = prefix.len() >= 56
        && starts(prefix, b"ARCHIVE#")
        && contains(prefix.get(8..56)?, b"\x01\0\0\0\x04\0\0\0E\0x\0e\0\0\0");
    let rec_len = if is_rtc { 56 } else { 32 };
    require!(
        size >= MIN_ARCHIVE_HASH_RTM_SIZE,
        prefix.len() >= rec_len,
        (starts(prefix, b"ARCHIVE#\x02\0\0\0") && le32(prefix, 20) == Some(0x6F)) || is_rtc
    );
    Some(Label::ArchiveHashRtm)
}

/// Ashampoo URESFILE Localization Package (.ashlang) Ashampoo.
pub(crate) fn verify_ashampoo_ashlang(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ASHAMPOO_ASHLANG_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"URESFILE"),
        le32(prefix, 12) == Some(0),
        get_slice(prefix, 16, 20)? == b"lang"
    );
    Some(Label::AshampooAshlang)
}

/// Kunos Simulazioni Assetto Corsa `data.acd` Obfuscated Car Physics Archive (`.acd`, 2013 Kunos Simulazioni).
pub(crate) fn verify_assettocorsa_acd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ASSETTOCORSA_ACD_SIZE, prefix.len() >= 48);
    let off = if le32(prefix, 0) == Some(0xFFFF_FBA9) { 8usize } else { 0usize };
    let name_len = le32(prefix, off)? as usize;
    require!((5..=64).contains(&name_len));
    let data_off = off + 4 + name_len + 4;
    require!(data_off + 32 <= prefix.len());
    let name = get_slice(prefix, off + 4, off + 4 + name_len)?;
    require!(name.iter().all(|&b| {
        b.is_ascii_lowercase()
            || b.is_ascii_uppercase()
            || b.is_ascii_digit()
            || matches!(b, b'_' | b'-' | b'.')
    }));
    let lower_ext = name.rsplit(|&b| b == b'.').next().filter(|e| e.len() != name.len())?;
    require!(
        lower_ext.eq_ignore_ascii_case(b"ini")
            || lower_ext.eq_ignore_ascii_case(b"lut")
            || lower_ext.eq_ignore_ascii_case(b"rto")
            || lower_ext.eq_ignore_ascii_case(b"txt")
            || lower_ext.eq_ignore_ascii_case(b"cfg")
    );
    let content_len = le32(prefix, off + 4 + name_len)? as usize;
    require!(
        content_len >= 8,
        (data_off as u64).saturating_add((content_len as u64).saturating_mul(4)) <= size
    );
    let check_words = content_len.min(16).min((prefix.len() - data_off) / 4);
    require!(check_words >= 8);
    let mut non_zero = 0usize;
    for i in 0..check_words {
        let p = data_off + i * 4;
        require!(
            u8_at(prefix, p + 1)? == 0 && u8_at(prefix, p + 2)? == 0 && u8_at(prefix, p + 3)? == 0
        );
        if u8_at(prefix, p)? != 0 {
            non_zero += 1;
        }
    }
    require!(non_zero >= 4);
    Some(Label::AssettocorsaAcd)
}

/// ASzf Zlib-Compressed Resource Package (.asz).
pub(crate) fn verify_asz_package(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ASZ_PACKAGE_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"ASzf"),
        (1..=100_000).contains(&le32(prefix, 4).unwrap_or(0)),
        (1..=256).contains(&le32(prefix, 8).unwrap_or(0))
    );
    Some(Label::AszPackage)
}

/// Atari ST Atomik Cruncher v3.5 Compressed File (ATM5).
pub(crate) fn verify_atomik_cruncher(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ATOMIK_CRUNCHER_SIZE && prefix.len() >= 12);
    let unpacked_size = be32(prefix, 4)? as u64;
    let packed_size = be32(prefix, 8)? as u64;
    require!((4..=0x0400_0000).contains(&unpacked_size) && packed_size >= 4 && packed_size <= size);
    Some(Label::Atomik)
}

/// ALWIL / Avast Antivirus Setup Package (`.vpu`).
pub(crate) fn verify_avast_vpu(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AVAST_VPU_SIZE,
        prefix.len() >= 26,
        starts(prefix, AVAST_VPU_MAGIC),
        le32(prefix, AVAST_VPU_FIELD1_OFFSET)? > 0,
        le32(prefix, AVAST_VPU_FIELD2_OFFSET)? > 0,
        u8_at(prefix, AVAST_VPU_ZLIB_OFFSET) == Some(0x78),
        has_non_ascii_text_byte(prefix.get(16..26)?)
    );
    Some(Label::AvastVpu)
}

/// Marcus Geelnard Basic Compression Library File (`.bfc`, 2003 Marcus Geelnard, `"BCL1"`).
pub(crate) fn verify_bcl_compressed(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BCL_COMPRESSED_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"BCL1"),
        (1..=10).contains(&be32(prefix, 4)?),
        be32(prefix, 8)? > 0
    );
    Some(Label::BclBfc)
}

/// Bethesda Game Studios Creation Engine Archive (.bsa / .ba2).
pub(crate) fn verify_bethesda_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BETHESDA_ARCHIVE_SIZE && prefix.len() >= 24);
    if starts(prefix, b"BSA\0") {
        require!(size >= 36 && prefix.len() >= 36);
        let version = le32(prefix, 4)?;
        let header_size = le32(prefix, 8)?;
        let folder_count = le32(prefix, 16)?;
        let file_count = le32(prefix, 20)?;
        require!(
            (103..=105).contains(&version)
                && header_size == 36
                && folder_count > 0
                && file_count > 0
        );
    } else if starts(prefix, b"BTDX") {
        let version = le32(prefix, 4)?;
        let kind = get_slice(prefix, 8, 12)?;
        let file_count = le32(prefix, 12)?;
        let name_table_off = le64(prefix, 16)?;
        require!(
            (1..=8).contains(&version)
                && (kind == b"GNRL" || kind == b"DX10" || kind == b"GNMF")
                && file_count > 0
                && name_table_off <= size
        );
    } else {
        return None;
    }
    Some(Label::BsaBa2)
}

/// BioWare Infinity / Aurora Engine BIFF Archive (`b"BIFFV1  "`).
pub(crate) fn verify_bioware_biff(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BIOWARE_BIFF_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"BIFFV1  ") || starts(prefix, b"BIFFV1.1"),
        u8_at(prefix, 11)? == 0
    );
    let file_cnt = le32(prefix, 8)? as u64;
    let table_off = le32(prefix, 16)? as u64;
    require!(
        file_cnt > 0
            && table_off >= 20
            && table_off.saturating_add(file_cnt.saturating_mul(16)) <= size
    );
    Some(Label::Bif)
}

/// BioWare UTF-16LE ERF V2.0 / V3.0 or ASCII ERF V1.0 Resource Archive (`.rim` / `.erf`).
pub(crate) fn verify_bioware_rim(prefix: &[u8], size: u64) -> Option<Label> {
    if starts(prefix, BIOWARE_ERF_UTF16_MAGIC) {
        require!(
            size >= MIN_BIOWARE_RIM_SIZE,
            prefix.len() >= 24,
            eq_at(prefix, 10, BIOWARE_ERF_V20) || eq_at(prefix, 10, BIOWARE_ERF_V30),
            le32(prefix, 16)? >= 1 || le32(prefix, 18)? >= 1
        );
        Some(Label::BiowareRim)
    } else if starts(prefix, b"ERF ")
        || starts(prefix, b"MOD ")
        || starts(prefix, b"SAV ")
        || starts(prefix, b"HAK ")
    {
        require!(
            size >= 160,
            prefix.len() >= 32,
            eq_at(prefix, 4, b"V1.0"),
            (1..=1_000_000).contains(&le32(prefix, 16)?),
            le32(prefix, 24)? >= 160,
            (le32(prefix, 24)? as u64) < size
        );
        Some(Label::BiowareRim)
    } else {
        None
    }
}

/// Bitdefender Antivirus Engine Threat Signature Segment (AVXS, .008 / .ivd).
pub(crate) fn verify_bitdefender_avxs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BITDEFENDER_AVXS_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"AVXS\x02\0\x01\0"),
        is_valid_zlib_header(u8_at(prefix, 20)?, u8_at(prefix, 21)?)
    );
    require!(has_non_ascii_text_byte(prefix.get(..24)?));
    Some(Label::BitdefenderAvxs)
}

/// BlackDragon v3 Archive (.zl).
pub(crate) fn verify_blackdragon_zl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BLACKDRAGON_ZL_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"BlackDragon Version\x03"),
        (1..=1_000_000).contains(&le32(prefix, 20).unwrap_or(0)),
        (1..=1_000_000).contains(&le32(prefix, 24).unwrap_or(0))
    );
    Some(Label::BlackdragonZl)
}

/// ZipZag BlackHole Compressed Archive (.bh).
pub(crate) fn verify_blackhole_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BLACKHOLE_ARCHIVE_SIZE,
        prefix.len() >= 38,
        starts(prefix, b"BH\x05\x07"),
        u8_at(prefix, 6)? == 0x25
    );
    let hdr_sz = le16(prefix, 4)? as usize;
    let fn_len = le16(prefix, 33)? as usize;
    let comment_len = le16(prefix, 35)? as usize;
    require!(
        hdr_sz >= 38,
        (1..=256).contains(&fn_len),
        37usize.saturating_add(fn_len).saturating_add(comment_len) == hdr_sz,
        prefix.len() >= 37 + fn_len,
        get_slice(prefix, 37, 37 + fn_len)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::Bh)
}

/// Blizzard Entertainment Diablo II String Table (.tbl, 2000) Blizzard Entertainment.
pub(crate) fn verify_blizzard_woo_tbl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BLIZZARD_WOO_TBL_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"Woo!\x01\0\0\0"),
        le16(prefix, 8).unwrap_or(0) > 0
    );
    Some(Label::BlizzardWooTbl)
}

/// C-Blosc2 Contiguous (`b2frame`) or Sparse (`b2sframe`) compressed container.
pub(crate) fn verify_blosc_compressed(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= 90
        && prefix.len() >= 24
        && (0x9B..=0x9F).contains(&u8_at(prefix, 0)?)
        && matches!(u8_at(prefix, 1)?, 0xA8 | 0xA7)
        && (eq_at(prefix, 2, b"b2frame\0") || eq_at(prefix, 2, b"b2sframe"))
        && u8_at(prefix, 10)? == 0xD2
    {
        let hdr_len = be32(prefix, 11)?;
        require!(
            (64..=1024 * 1024).contains(&hdr_len),
            (hdr_len as u64) <= size,
            matches!(u8_at(prefix, 15)?, 0xCF | 0xD3),
        );
        return Some(Label::Blosc);
    }
    require!(
        size >= 24,
        prefix.len() >= 16,
        starts(prefix, b"b2frame\0") || starts(prefix, b"b2sframe"),
    );
    let hdr_len = be32(prefix, 8)?;
    require!((24..=1024 * 1024).contains(&hdr_len), (hdr_len as u64) <= size);
    Some(Label::Blosc)
}

/// Blue Byte Turbo Packer Wolfgang Mayerhofer compressed file (`.tpwm`, 1991 Blue Byte).
pub(crate) fn verify_bluebyte_tpwm(prefix: &[u8], size: u64) -> Option<Label> {
    const ORIG_LEN_OFFSET: usize = 4; // @4..8: u32 BE uncompressed size

    require!(
        size >= MIN_BLUEBYTE_TPWM_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"TPWM"),
        (8..=67_108_864).contains(&be32(prefix, ORIG_LEN_OFFSET)?),
        has_non_ascii_text_byte(prefix.get(4..12)?)
    );
    Some(Label::BluebyteTpwm)
}

/// Blade & Soul Dialog Screenshots Bundle (.cbf).
pub(crate) fn verify_bns_dialog_cbf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BNS_DIALOG_CBF_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 2)? == hex2bin!("AD 0C"),
        get_slice(prefix, 3, 25)?
            == hex2bin!("00 00 00 64 69 61 6C 6F 67 5F 73 63 72 65 65 6E 73 68 6F 74 73 5F")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::BnsDialogCbf)
}

/// BSD / Unix UFS `dump(8)` Backup Archive (`.dump`, 1983) UC Berkeley CSRG.
///
/// Validates the 1,024-byte `union u_spcl` header (`<protocols/dumprestore.h>`) in either little-
/// or big-endian byte order: `c_magic` at `@24`, `c_type` in `1..=6` at `@0`, `c_volume >= 1`, and
/// the exact 256-word 32-bit modular checksum equal to `84446` (`0x149DE`).
pub(crate) fn verify_bsd_ufs_dump(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BSD_DUMP_SIZE, prefix.len() >= BSD_DUMP_BLOCK_SIZE);
    let m_le = le32(prefix, BSD_DUMP_MAGIC_OFFSET)?;
    let m_be = be32(prefix, BSD_DUMP_MAGIC_OFFSET)?;
    let is_le = if matches!(m_le, BSD_DUMP_OFS_MAGIC | BSD_DUMP_NFS_MAGIC | BSD_DUMP_UFS2_MAGIC) {
        true
    } else if matches!(m_be, BSD_DUMP_OFS_MAGIC | BSD_DUMP_NFS_MAGIC | BSD_DUMP_UFS2_MAGIC) {
        false
    } else {
        return None;
    };
    let read32 = if is_le { le32 } else { be32 };
    let c_type = read32(prefix, BSD_DUMP_TYPE_OFFSET)?;
    let c_vol = read32(prefix, BSD_DUMP_VOLUME_OFFSET)?;
    require!((1..=6).contains(&c_type), (1..=10000).contains(&c_vol));
    let mut sum = 0u32;
    for off in (0..BSD_DUMP_BLOCK_SIZE).step_by(4) {
        sum = sum.wrapping_add(read32(prefix, off)?);
    }
    require!(sum == BSD_DUMP_CHECKSUM_TARGET);
    Some(Label::BsdDump)
}

/// Colin Percival bsdiff / Chromium Courgette Binary Patch.
pub(crate) fn verify_bsdiff(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BSDIFF_SIZE, prefix.len() >= 32);
    if starts(prefix, b"BSDIFF40") {
        // Standard bsdiff 4.0: 32-byte header (sign-magnitude s64 LE lengths at @8, @16, @24) + "BZh" bzip2 control stream at @32
        require!(
            size >= 46,
            prefix.len() >= 36,
            (u8_at(prefix, 15)? & 0x7F) == 0,
            (u8_at(prefix, 23)? & 0x7F) == 0,
            (u8_at(prefix, 31)? & 0x7F) == 0,
            eq_at(prefix, 32, b"BZh"),
            (b'1'..=b'9').contains(&u8_at(prefix, 35)?)
        );
        return Some(Label::Bsdiff);
    }
    if starts(prefix, b"BSDIFN40") && prefix.get(8..32)?.contains(&0) {
        return Some(Label::Bsdiff);
    }
    if starts(prefix, b"BSDF2") && prefix.get(5..16)?.contains(&0) {
        return Some(Label::Bsdiff);
    }
    None
}

/// Capcom RE Engine PAK Resource Archive (`.pak`, `"KPKA"`, 2017 Capcom).
pub(crate) fn verify_capcom_re_pak(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CAPCOM_RE_PAK_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"KPKA"),
        (2..=4).contains(&u8_at(prefix, 4)?),
        u8_at(prefix, 5)? <= 4,
        (1..=2_000_000).contains(&le32(prefix, 8)?),
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::CapcomRePak)
}

/// CARV VRAC Metadata Container (.vrac).
pub(crate) fn verify_carv_vrac(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CARV_VRAC_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"CARV\x1b\0\0\0\0\0\0\0"),
        le32(prefix, 20) == Some(0)
    );
    Some(Label::CarvVrac)
}

/// Casio Graphing Calculator Bitwise-Inverted `"USBPower"` Container (`.g1a`, `.g1m`, `.g2m`, `.g3a`, `.g3m`, `.g1e`, `.c2m`).
///
/// Standard 32-byte (`0x20`) header (stored bitwise-inverted `~b & 0xFF` on disk):
/// - `@0..8`: `AA AC BD AF 90 88 9A 8D` (bitwise-NOT of `"USBPower"`, `55 53 42 50 6F 77 65 72`).
/// - `@8`: Subtype byte on disk:
///   - `0x0C` (`~0xF3`): fx-9860G add-in application (`.g1a`)
///   - `0x82` (`~0x7D`): fx-CG picture / flipbook (`.g3p` / `.g3b`)
///   - `0x8A` (`~0x75`): fx-CG main memory archive (`.g3m` / `.g3r`)
///   - `0x9D` (`~0x62`): fx-9860G main memory archive (`.g1m` / `.g1r` / `.g2m` / `.g2r`)
///   - `0xB6` (`~0x49`): e-Activity file (`.g1e` / `.g2e` / `.g3e`)
///   - `0xCE` (`~0x31`): fx-CP ClassPad main memory archive (`.c2m`)
///   - `0xD3` (`~0x2C`): fx-CG Prizm add-in application (`.g3a`)
///   - `0xED` (`~0x12`): fx-9860G language file (`.g1l`)
/// - `@9..14`: `FF EF FF EF FF` (decoded `00 10 00 10 00`) or `FF FE FF FE FF` (decoded `00 01 00 01 00`).
/// - `@14`: Control byte 1 `C1` where `(!u8_at(prefix, 14)?) == (!u8_at(prefix, 19)?).wrapping_add(0x41)`.
/// - `@15`: Reserved `0xFF` (decoded `0x00`).
/// - `@16..20`: Bitwise-inverted `u32 BE` total file size `FS` (`>= 64`).
pub(crate) fn verify_casio_container(prefix: &[u8], size: u64) -> Option<Label> {
    const USBPOWER_INV_MAGIC: &[u8; 8] = hex2bin!("AA AC BD AF 90 88 9A 8D"); // @0..8: ~"USBPower"
    const SUBTYPE_OFFSET: usize = 8; // @8: inverted subtype byte
    const TAIL_TYPE_OFFSET: usize = 9; // @9..14: inverted 5-byte subtype tail
    const TAIL_10_INV: &[u8; 5] = hex2bin!("FF EF FF EF FF"); // @9..14: ~(00 10 00 10 00)
    const TAIL_01_INV: &[u8; 5] = hex2bin!("FF FE FF FE FF"); // @9..14: ~(00 01 00 01 00)
    const C1_OFFSET: usize = 14; // @14: ~( (FS + 0x41) & 0xFF )
    const RESERVED_15_OFFSET: usize = 15; // @15: ~0x01 = 0xFE in real-world Casio files (or ~0x00 = 0xFF in Cahute doc)
    const FS_OFFSET: usize = 16; // @16..20: ~(u32 BE file size)
    const FS_LSB_OFFSET: usize = 19; // @19: ~(FS & 0xFF)

    require!(
        size >= MIN_CASIO_CONTAINER_SIZE,
        prefix.len() >= 32,
        starts(prefix, USBPOWER_INV_MAGIC),
        matches!(
            u8_at(prefix, SUBTYPE_OFFSET)?,
            0x0C | 0x82 | 0x8A | 0x9D | 0xB6 | 0xCE | 0xD3 | 0xED
        ),
        eq_at(prefix, TAIL_TYPE_OFFSET, TAIL_10_INV)
            || eq_at(prefix, TAIL_TYPE_OFFSET, TAIL_01_INV),
        matches!(u8_at(prefix, RESERVED_15_OFFSET)?, 0xFE | 0xFF),
        (!u8_at(prefix, C1_OFFSET)?) == (!u8_at(prefix, FS_LSB_OFFSET)?).wrapping_add(0x41),
        (!be32(prefix, FS_OFFSET)?) >= MIN_CASIO_CONTAINER_SIZE as u32
    );
    Some(Label::CasioContainer)
}

/// Computer Associates CAZIP: `"\r\n\x1ACAZIP"` followed by a two-digit ASCII version (`"10"`, `"33"`).
pub(crate) fn verify_cazip(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = b"\r\n\x1ACAZIP"; // @0, 8 bytes
    const VERSION: usize = 8; // @8: two ASCII digits
    require!(size >= MIN_CAZIP_SIZE, starts(prefix, MAGIC));
    require!(
        u8_at(prefix, VERSION)?.is_ascii_digit(),
        u8_at(prefix, VERSION + 1)?.is_ascii_digit()
    );
    Some(Label::Cazip)
}

/// Jari Komppa Compressed File Library v3 archive (`.cfl`, 2002 Jari Komppa).
pub(crate) fn verify_cfl3(prefix: &[u8], size: u64) -> Option<Label> {
    const CFL3_MAGIC: &[u8; 4] = b"CFL3"; // @0..4: "CFL3" header magic
    const ZLIB_CMF_OFFSET: usize = 8; // @8: zlib CMF byte (0x78) or directory offset word

    require!(
        size >= MIN_CFL3_SIZE,
        prefix.len() >= 12,
        starts(prefix, CFL3_MAGIC),
        u8_at(prefix, ZLIB_CMF_OFFSET) == Some(0x78) || has_non_ascii_text_byte(prefix.get(4..12)?)
    );
    Some(Label::Cfl3)
}

/// CI Games DPK4 Package Archive (`.dpk`, 2007 CI Games).
pub(crate) fn verify_cigames_dpk4(prefix: &[u8], size: u64) -> Option<Label> {
    const TOTAL_SIZE_OFFSET: usize = 4; // @4..8: u32 LE total package size
    const DATA_OFF_OFFSET: usize = 8; // @8..12: u32 LE data start offset
    const ENTRY_COUNT_OFFSET: usize = 12; // @12..16: u32 LE file entry count

    require!(
        size >= MIN_CIGAMES_DPK4_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"DPK4"),
        le32(prefix, TOTAL_SIZE_OFFSET)? as u64 == size,
        (16..=size).contains(&(le32(prefix, DATA_OFF_OFFSET)? as u64)),
        (1..=262_144).contains(&le32(prefix, ENTRY_COUNT_OFFSET)?)
    );
    Some(Label::CigamesDpk4)
}

/// Clickteam Fusion / Multimedia Fusion 2 Unicode Project (.mfa, 2006) Clickteam.
pub(crate) fn verify_clickteam_mfa(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CLICKTEAM_MFA_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"MFU2"),
        (1..=16).contains(&le32(prefix, 4).unwrap_or(0)),
        (1..=32).contains(&le32(prefix, 8).unwrap_or(0))
    );
    Some(Label::ClickteamMfa)
}

/// CMAP Zlib-Compressed Collision Tile Map (.cmp).
pub(crate) fn verify_cmap_cmp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CMAP_CMP_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"CMAP"),
        (1..=4096).contains(&le16(prefix, 4).unwrap_or(0)),
        (1..=4096).contains(&le16(prefix, 6).unwrap_or(0)),
        is_valid_zlib_header(u8_at(prefix, 8)?, u8_at(prefix, 9)?)
    );
    Some(Label::CmapCmp)
}

/// Cocos2d / TexturePacker Compressed PVR Container (.ccz).
pub(crate) fn verify_cocos2d_ccz(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_COCOS2D_CCZ_SIZE && prefix.len() >= 16);
    let comp_type = be16(prefix, 4)?;
    let version = be16(prefix, 6)?;
    let uncomp_len = be32(prefix, 12)?;
    require!(comp_type <= 3 && (1..=2).contains(&version) && uncomp_len > 0);
    Some(Label::Ccz)
}

/// Infinity Ward / Activision Call of Duty FastFile Archive (.ff / .fp, 2007) Infinity Ward.
pub(crate) fn verify_cod_fastfile(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_COD_FASTFILE_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"IWff"),
        matches!(get_slice(prefix, 5, 8)?, b"100" | b"000"),
        (1..=10000).contains(&le32(prefix, 8).unwrap_or(0))
            || (1..=10000).contains(&be32(prefix, 8).unwrap_or(0))
    );
    Some(Label::CodFastfile)
}

/// Coktel Vision STK2.1 Archive (`.stk` / `.itk`, 1993 Coktel Vision).
pub(crate) fn verify_coktel_stk2(prefix: &[u8], size: u64) -> Option<Label> {
    const STK21_MAGIC: &[u8; 6] = b"STK2.1"; // @0..6: "STK2.1"
    const COUNT_OFFSET: usize = 6; // @6..8: u16 LE file count

    require!(
        size >= MIN_COKTEL_STK2_SIZE,
        prefix.len() >= 28,
        starts(prefix, STK21_MAGIC),
        (1..=16_384).contains(&le16(prefix, COUNT_OFFSET)?),
        has_non_ascii_text_byte(prefix.get(6..28)?)
    );
    Some(Label::CoktelStk2)
}

/// ASCII SVR4 (`070701`/`070702`) or POSIX ODC (`070707`) CPIO archive.
pub(crate) fn verify_cpio(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 26, prefix.len() >= 26);
    let hex_end = prefix.len().min(30);
    if starts(prefix, b"070701") || starts(prefix, b"070702") {
        let fields = get_slice(prefix, 6, hex_end)?;
        require!(fields.iter().all(|&b| b.is_ascii_hexdigit() || b == 0), contains(prefix, b"\0"),);
        return Some(Label::Cpio);
    }
    if starts(prefix, b"070707") {
        let fields = get_slice(prefix, 6, hex_end)?;
        require!(fields.iter().all(|&b| matches!(b, b'0'..=b'7' | 0)), contains(prefix, b"\0"),);
        return Some(Label::Cpio);
    }
    None
}

/// Unix System V old binary CPIO archive (`0x71C7` LE or BE).
pub(crate) fn verify_cpio_binary(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 26, prefix.len() >= 26);
    if starts(prefix, hex2bin!("C7 71")) {
        require!(le16(prefix, 2)? == 0);
        return Some(Label::Cpio);
    }
    if starts(prefix, hex2bin!("71 C7")) {
        require!(be16(prefix, 2)? == 0);
        return Some(Label::Cpio);
    }
    None
}

/// CP/M LU / NULU Library Archive (`.lbr`, 1982 Gary P. Novosielski).
///
/// Validates Entry 0 self-descriptor (`0x00` + 11 spaces at `@0..12`, `index == 0` at `@12..14`,
/// `dir_sectors >= 1` at `@14..16`), 128-byte CP/M sector alignment, and `dir_sectors * 128 <= size`.
pub(crate) fn verify_cpm_lbr(prefix: &[u8], size: u64) -> Option<Label> {
    const INDEX_OFF: usize = 12; // @12..14: u16 LE directory start sector (must be 0)
    const DIR_SECTORS_OFF: usize = 14; // @14..16: u16 LE directory length in 128B sectors (>= 1)
    const SECTOR_SIZE: u64 = 128;

    require!(
        size >= MIN_CPM_LBR_SIZE,
        size.is_multiple_of(SECTOR_SIZE),
        prefix.len() >= 16,
        starts(prefix, CPM_LBR_DIR_ENTRY_MAGIC),
        le16(prefix, INDEX_OFF)? == 0
    );
    let dir_sectors = le16(prefix, DIR_SECTORS_OFF)?;
    require!(dir_sectors >= 1, (dir_sectors as u64) * SECTOR_SIZE <= size);
    Some(Label::Lbr)
}

/// CRI Middleware CRIWARE FileMajik PRO Package (.cpk).
pub(crate) fn verify_criware_cpk(prefix: &[u8], size: u64) -> Option<Label> {
    const CPK_MAGIC: &[u8; 4] = b"CPK "; // @0..4: CRIWARE CPK chunk magic
    const UTF_MAGIC: &[u8; 4] = b"@UTF"; // @16..20: unencrypted @UTF table header
    const UTF_ENC_LE_MAGIC: &[u8; 4] = hex2bin!("1F 9E F3 F5"); // @16..20: XOR-encrypted @UTF table (v1 key)
    const UTF_ENC_ALT_MAGIC: &[u8; 4] = hex2bin!("1F 9E 62 95"); // @16..20: XOR-encrypted @UTF table (v2 key)
    const FLAG_OFFSET: usize = 4; // @4..8: u32 LE table encryption/type flag (0x00 or 0xFF)
    const UTF_SIZE_OFFSET: usize = 8; // @8..16: u64 LE @UTF packet byte length
    const UTF_MAGIC_OFFSET: usize = 16; // @16..20: @UTF table signature

    require!(size >= MIN_CRIWARE_CPK_SIZE, prefix.len() >= 24, starts(prefix, CPK_MAGIC));
    let flag = le32(prefix, FLAG_OFFSET)?;
    let utf_size = le64(prefix, UTF_SIZE_OFFSET)?;
    require!(
        matches!(flag, 0x00 | 0xFF),
        utf_size >= 32,
        16 + utf_size <= size,
        eq_at(prefix, UTF_MAGIC_OFFSET, UTF_MAGIC)
            || eq_at(prefix, UTF_MAGIC_OFFSET, UTF_ENC_LE_MAGIC)
            || eq_at(prefix, UTF_MAGIC_OFFSET, UTF_ENC_ALT_MAGIC)
    );
    Some(Label::Cpk)
}

/// Thomas Schwarz Crunch-Mania Amiga Data Archive (`.crm`, 1991 Thomas Schwarz).
pub(crate) fn verify_crunch_mania(prefix: &[u8], size: u64) -> Option<Label> {
    const ORIG_LEN_OFFSET: usize = 6; // @6..10: u32 BE uncompressed size
    const CMPR_LEN_OFFSET: usize = 10; // @10..14: u32 BE compressed stream size

    require!(
        size >= MIN_CRUNCH_MANIA_SIZE,
        prefix.len() >= 14,
        starts(prefix, b"CrM!")
            || starts(prefix, b"CrM2")
            || starts(prefix, b"Crm!")
            || starts(prefix, b"Crm2")
    );
    let orig_len = be32(prefix, ORIG_LEN_OFFSET)? as u64;
    let cmpr_len = be32(prefix, CMPR_LEN_OFFSET)? as u64;
    require!(orig_len > 0, cmpr_len > 0, cmpr_len + 14 <= size);
    Some(Label::CrunchMania)
}

/// CRUSH archive / index: `"CRUSH v"` + version digit, with a `0x1A` (EOF) terminator at `@11`.
pub(crate) fn verify_crush(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = b"CRUSH v"; // @0, 7 bytes
    const VERSION_DIGIT: usize = 7; // @7: ASCII digit
    const EOF_MARK: usize = 11; // @11: 0x1A DOS EOF after the version string
    require!(size >= MIN_CRUSH_SIZE, starts(prefix, MAGIC));
    require!(u8_at(prefix, VERSION_DIGIT)?.is_ascii_digit(), u8_at(prefix, EOF_MARK)? == 0x1A);
    Some(Label::Crush)
}

/// Crytek CryEngine CrChF Object Container (.soc / .cgf, 2004) Crytek.
pub(crate) fn verify_cryengine_soc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CRYENGINE_SOC_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"CrChF\x07\0\0"),
        (1..=8).contains(&le32(prefix, 8).unwrap_or(0)),
        le32(prefix, 12) == Some(0x10)
    );
    Some(Label::CryengineSoc)
}

/// Cryo Interactive BigFile Game Archive (`b"BigFile 1.00\0\0\0\0"`).
pub(crate) fn verify_cryo_bigfile(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CRYO_BIGFILE_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"BigFile 1.00\x00\x00\x00\x00")
    );
    let file_cnt = le32(prefix, 16)? as u64;
    require!(file_cnt > 0 && le32(prefix, 28) == Some(0));
    Some(Label::CryoBigfile)
}

/// Adventure Game Studio (AGS) CLIB Resource Archive (`.ags` / `speech.vox` / `audio.vox`).
/// Supports all AGS CLIB format revisions (`v6`, `v10`, `v11`, `v15`, `v20`, `v21`, `v30`).
pub(crate) fn verify_cryo_clib_vox(prefix: &[u8], size: u64) -> Option<Label> {
    const AGS_CLIB_MAGIC: &[u8; 5] = bytes!(b"CLIB", &[0x1A]); // @0..5: "CLIB\x1a" ("Chris's Library")
    const VERSION_OFF: usize = 5; // @5: u8 CLIB format version
    const V30_RESERVED_OFF: usize = 7; // @7..11: u32 LE reserved zero in v30 header
    const V30_LIB_COUNT_OFF: usize = 11; // @11..15: u32 LE data file count (1..=256) in v30

    require!(size >= MIN_CRYO_CLIB_VOX_SIZE, prefix.len() >= 28, starts(prefix, AGS_CLIB_MAGIC));
    let version = u8_at(prefix, VERSION_OFF)?;
    match version {
        30 => {
            require!(
                le32(prefix, V30_RESERVED_OFF) == Some(0),
                (1..=256).contains(&le32(prefix, V30_LIB_COUNT_OFF)?)
            );
        }
        20 | 21 => {
            require!(u8_at(prefix, 6)? <= 128);
        }
        6 | 10 | 11 | 15 => {
            // Note: v6/v10/v11/v15 obfuscate bytes @6.. with an "Avis Durgan" seed.
        }
        _ => return None,
    }
    Some(Label::CryoClibVox)
}

/// Spotware cTrader / cAlgo Compiled Trading Algorithm (.algo).
pub(crate) fn verify_ctrader_algo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CTRADER_ALGO_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"algo\x02\0"),
        get_slice(prefix, 9, 13)? == b"\x1f\x8b\x08\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..20)?));
    Some(Label::CtraderAlgo)
}

/// Cyan Worlds / Brøderbund Mohawk Resource Archive (`.mhk`, `"MHWK"`, 1994 Brøderbund / Cyan Worlds).
pub(crate) fn verify_cyan_mohawk_mhk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CYAN_MOHAWK_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"MHWK"),
        be32(prefix, 4)? >= 20,
        eq_at(prefix, 8, b"RSRC"),
        be16(prefix, 12)? <= 0x0200,
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::CyanMohawk)
}

/// Gaijin Entertainment Dagor Engine DirectX Texture Pack (`.dxp.bin`, `"DxP2"`, 2012 Gaijin Entertainment).
pub(crate) fn verify_dagor_dxp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DAGOR_DXP_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"DxP2"),
        (1..=4).contains(&le32(prefix, 4)?),
        (1..=1_000_000).contains(&le32(prefix, 8)?),
        le32(prefix, 12)? >= 16,
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::DagorDxp)
}

/// Gaijin Entertainment Dagor Engine Game Resource Pack (`.grp`, `"GRP2"`/`"GRP3"`, 2012 Gaijin Entertainment).
pub(crate) fn verify_dagor_grp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DAGOR_GRP_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"GRP2") || starts(prefix, b"GRP3"),
        le32(prefix, 4)? >= 16,
        le32(prefix, 8)? >= 16,
        le32(prefix, 12)? >= 16,
        le32(prefix, 16)? >= 16,
        has_non_ascii_text_byte(prefix.get(..20)?)
    );
    Some(Label::DagorGrp)
}

/// Gaijin Entertainment Dagor Engine VROMFS Archive (`.vromfs.bin`, `"VRFx"`/`"VRFs"`, 2012 Gaijin Entertainment).
pub(crate) fn verify_dagor_vromfs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DAGOR_VROMFS_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"VRFx") || starts(prefix, b"VRFs"),
        matches!(
            get_slice(prefix, 4, 8)?,
            b"\0\0PC"
                | b"PC\0\0"
                | b"iOS\0"
                | b"and\0"
                | b"PS4\0"
                | b"PS5\0"
                | b"X360"
                | b"XBX1"
                | b"MAC\0"
                | b"NS\0\0"
        ),
        le32(prefix, 8)? > 0,
        (le32(prefix, 12)? & 0x03FF_FFFF) > 0,
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::DagorVromfs)
}

/// DAN1 Character Animation Package (.dan).
pub(crate) fn verify_dan1_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DAN1_ARCHIVE_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"DAN1"),
        le32(prefix, 8) == Some(0x34),
        get_slice(prefix, 16, 24)? == b"HUMAN1\0\0"
    );
    Some(Label::Dan1Archive)
}

/// Red Hat / SUSE Delta RPM (`.drpm`, `"drpm"` / `"DLT3"`).
pub(crate) fn verify_delta_rpm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DELTA_RPM_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"drpm") || starts(prefix, b"DLT3"),
        has_non_ascii_text_byte(prefix.get(4..12)?)
    );
    Some(Label::DeltaRpm)
}

/// Parallax Software Descent HOG Archive (`.hog`, 1994 Parallax Software).
pub(crate) fn verify_descent_hog(prefix: &[u8], size: u64) -> Option<Label> {
    const DHF_MAGIC: &[u8; 3] = b"DHF"; // @0..3: "DHF" ("Descent Hog File")
    const NAME_RANGE: core::ops::Range<usize> = 3..16; // @3..16: 13-byte null-terminated 8.3 filename
    const ENTRY_LEN_OFFSET: usize = 16; // @16..20: u32 LE first lump byte length

    require!(size >= MIN_DESCENT_HOG_SIZE, prefix.len() >= 20, starts(prefix, DHF_MAGIC));
    let name_slice = get_slice(prefix, NAME_RANGE.start, NAME_RANGE.end)?;
    let nul_pos = name_slice.iter().position(|&b| b == 0)?;
    let fname = &name_slice[..nul_pos];
    let entry_len = le32(prefix, ENTRY_LEN_OFFSET)? as u64;
    require!(
        (3..=12).contains(&fname.len()),
        fname.contains(&b'.'),
        fname.iter().all(|&b| b.is_ascii_graphic()),
        entry_len > 0 && 20 + entry_len <= size
    );
    Some(Label::DescentHog)
}

/// DifStream Smart Version Patch (DFS File, .svf, Gentee / DifStream).
pub(crate) fn verify_difstream_svf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DIFSTREAM_SVF_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"DFS File\r\nhttp://www.difstream.com\r\n\0\0\0\0")
    );
    require!(has_non_ascii_text_byte(prefix.get(..44)?));
    Some(Label::DifstreamSvf)
}

/// Electronic Arts Sports Audio Bank (.abk, 2004) EA Sports.
pub(crate) fn verify_ea_abk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EA_ABK_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"ABKC\x02\x01\x02\x02\x06\x00"),
        le32(prefix, 20).unwrap_or(0) >= 36
    );
    Some(Label::EaAbk)
}

/// Electronic Arts ViV4 BIG Archive Index Header (.bh, 2005) EA Sports.
pub(crate) fn verify_ea_viv4_bh(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EA_VIV4_BH_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"ViV4"),
        le32(prefix, 4).unwrap_or(0) >= 28,
        le32(prefix, 4) == be32(prefix, 12),
        has_non_ascii_text_byte(prefix.get(..28)?)
    );
    Some(Label::EaViv4Bh)
}

/// Error Code Modeler: `"ECM\0"` followed by (type, count) varint records that strip regenerable CD-ROM EDC/ECC.
pub(crate) fn verify_ecm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ECM_SIZE, starts(prefix, b"ECM\0"));
    Some(Label::Ecm)
}

/// EDI Install packed file / EDI LZSSLib: `"EDILZSS"` + version digit, followed by the printable original filename.
pub(crate) fn verify_edi_lzss(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = b"EDILZSS"; // @0, 7 bytes
    const VERSION: usize = 7; // @7: '1' or '2'
    const FILENAME: usize = 8; // @8: first character of the stored filename or 0xFF flag
    require!(size >= MIN_EDI_LZSS_SIZE, starts(prefix, MAGIC));
    let ver = u8_at(prefix, VERSION)?;
    require!(matches!(ver, b'1' | b'2'));
    let fbyte = u8_at(prefix, FILENAME)?;
    require!(
        fbyte == 0xFF
            || (fbyte.is_ascii_graphic() && prefix.get(8..prefix.len().min(24))?.contains(&0))
    );
    Some(Label::Edilzss)
}

/// ESTsoft ALZip EGG Archive (`.egg`, 2009) ESTsoft (`"EGGA"`).
pub(crate) fn verify_egg_archive(prefix: &[u8], size: u64) -> Option<Label> {
    (size >= MIN_EGG_ARCHIVE_SIZE
        && prefix.len() >= MIN_EGG_ARCHIVE_SIZE as usize
        && le16(prefix, 4) == Some(0x0100)
        && le32(prefix, 10) == Some(0))
    .then_some(Label::Egg)
}

/// emTeX Font Library (`.fli`, 1990, Eberhard Mattes).
///
/// Big-endian 14-byte header (`"FLIB"`, `version` 1 or 2, `0`, `dir_size`, `dir_count`,
/// `total_fonts`, `0x0000`) followed at `@14` by `dir_count` DPI directory blocks totaling
/// `dir_size` bytes, then concatenated TeX `.pk` font payloads (`0xF7 0x59`) starting at
/// `@14 + dir_size`.
pub(crate) fn verify_emtex_font_library(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_LEN: usize = 14;
    const VER_OFF: usize = 4; // @4: u8 format version (1 or 2)
    const PAD0_OFF: usize = 5; // @5: u8 reserved zero
    const DIR_SIZE_OFF: usize = 6; // @6..8: u16 BE total directory size in bytes
    const DIR_COUNT_OFF: usize = 8; // @8..10: u16 BE number of DPI subdirectories
    const TOTAL_FONTS_OFF: usize = 10; // @10..12: u16 BE total font entries across subdirectories
    const PAD1_OFF: usize = 12; // @12..14: u16 BE reserved zero

    require!(
        size >= MIN_EMTEX_FLI_SIZE,
        prefix.len() >= 22,
        starts(prefix, EMTEX_FLI_MAGIC),
        matches!(u8_at(prefix, VER_OFF)?, 1 | 2),
        u8_at(prefix, PAD0_OFF)? == 0,
        be16(prefix, PAD1_OFF)? == 0
    );
    let dir_size = be16(prefix, DIR_SIZE_OFF)? as usize;
    let dir_count = be16(prefix, DIR_COUNT_OFF)? as usize;
    let total_fonts = be16(prefix, TOTAL_FONTS_OFF)? as usize;
    let dir_end = HDR_LEN.checked_add(dir_size)?;
    require!(
        dir_size >= 18,
        (1..=64).contains(&dir_count),
        (1..=4096).contains(&total_fonts),
        (dir_end as u64) < size
    );

    if dir_end <= prefix.len() {
        let mut cursor = HDR_LEN;
        let mut seen_fonts: usize = 0;
        for _ in 0..dir_count {
            require!(cursor + 8 <= dir_end);
            let sub_sz = be16(prefix, cursor)? as usize; // @cursor..+2: byte size of font entries following the 8B subdir header
            let cnt = be16(prefix, cursor + 2)? as usize; // @cursor+2..+4: font count in subdir
            let _dpi = be32(prefix, cursor + 4)?; // @cursor+4..+8: 16.16 fixed-point DPI
            cursor += 8;
            let sub_end = cursor.checked_add(sub_sz)?;
            require!(sub_end <= dir_end);
            for _ in 0..cnt {
                require!(cursor + 9 <= sub_end);
                let font_size = be32(prefix, cursor)? as u64;
                let font_off = be32(prefix, cursor + 4)? as u64;
                let name_len = u8_at(prefix, cursor + 8)? as usize;
                require!(
                    font_size >= 2,
                    font_off >= dir_end as u64,
                    font_off.checked_add(font_size)? <= size,
                    (1..=64).contains(&name_len),
                    cursor + 9 + name_len <= sub_end
                );
                cursor += 9 + name_len;
                seen_fonts += 1;
            }
            require!(cursor == sub_end);
        }
        require!(cursor == dir_end, seen_fonts == total_fonts);
        if dir_end + 2 <= prefix.len() {
            require!(eq_at(prefix, dir_end, TEX_PK_PREAMBLE));
        }
    }

    Some(Label::EmtexFontLibrary)
}

/// Ensemble Studios Age of Empires I / II DRS Resource Archive (`.drs`).
///
/// Header layout:
/// - `@0..37`: `"Copyright (c) 1997 Ensemble Studios.\x1a"` (padded to 40 bytes)
/// - `@40..44`: `"1.00"` version string
/// - `@44..49`: `"tribe"` or `"swbg\0"` type tag
/// - `@56..60`: `u32 LE` `table_count` (`1..=64`)
/// - `@60..64`: `u32 LE` `first_file_offset` (`>= 64`, `< file_size`)
pub(crate) fn verify_ensemble_drs(prefix: &[u8], size: u64) -> Option<Label> {
    const COPYRIGHT_SIG: &[u8; 37] = bytes!(b"Copyright (c) 1997 Ensemble Studios.", &[0x1A]); // @0..37
    const VER_OFF: usize = 40; // @40..44: "1.00"
    const VER_TAG: &[u8; 4] = b"1.00";
    const TABLE_COUNT_OFF: usize = 56; // @56..60: u32 LE resource table count
    const FIRST_OFF_OFF: usize = 60; // @60..64: u32 LE offset to first resource payload

    require!(
        size >= MIN_ENSEMBLE_DRS_SIZE,
        prefix.len() >= 64,
        starts(prefix, COPYRIGHT_SIG),
        eq_at(prefix, VER_OFF, VER_TAG),
        (1..=64).contains(&le32(prefix, TABLE_COUNT_OFF)?)
    );
    let first_off = le32(prefix, FIRST_OFF_OFF)? as u64;
    require!(first_off >= 64, first_off < size);
    Some(Label::AoeDrs)
}

/// Eschalon Setup / EDI Install ARCV: `"ARCV"`, `@5` format byte `0x01` (EDI Install) or `0x02` (Eschalon Setup).
pub(crate) fn verify_eschalon_arcv(prefix: &[u8], size: u64) -> Option<Label> {
    const FORMAT: usize = 5; // @5: 0x01 EDI Install archive, 0x02 Eschalon Setup archive
    require!(size >= MIN_ARCV_SIZE, starts(prefix, b"ARCV"));
    require!(matches!(u8_at(prefix, FORMAT)?, 0x01 | 0x02));
    Some(Label::Arcv)
}

/// EXBN Binary Resource Container (.ebn).
pub(crate) fn verify_exbn_ebn(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EXBN_EBN_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"EXBN\x01\0\0\0\0\0\0\0\0\0\0\0"),
        (1..=65536).contains(&le32(prefix, 16).unwrap_or(0))
    );
    Some(Label::ExbnEbn)
}

/// FairyGUI Binary UI Package (`.fui`).
pub(crate) fn verify_fairygui_fui(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FAIRYGUI_FUI_SIZE,
        prefix.len() >= 24,
        starts(prefix, FAIRYGUI_MAGIC),
        (1..=16).contains(&be32(prefix, FAIRYGUI_VERSION_OFFSET)?),
        u8_at(prefix, FAIRYGUI_COMP_OFFSET)? <= 1,
        be16(prefix, FAIRYGUI_ID_LEN_OFFSET) == Some(8),
        get_slice(prefix, 11, 19)?.iter().all(|&b| b.is_ascii_alphanumeric())
    );
    Some(Label::FairyguiFui)
}

/// Fastgraph/Image Resource Library (`.fgi`, 1991) Ted Gruber Software.
pub(crate) fn verify_fastgraph_fgi(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_FASTGRAPH_FGI_SIZE && prefix.len() >= 72);
    let (off0, len0) = parse_fastgraph_fgi_entry(get_slice(prefix, 0, 24)?)?;
    let (off1, _len1) = parse_fastgraph_fgi_entry(get_slice(prefix, 24, 48)?)?;
    let idx_end = off0 as usize;
    require!(
        (72..=3096).contains(&off0),
        off0 % 24 == 0,
        off1 == off0.checked_add(len0)?,
        size >= off0 as u64,
        prefix.len() >= idx_end
    );
    let ptr_off = idx_end - 24;
    let ptr_slot = get_slice(prefix, ptr_off, idx_end)?;
    let is_terminal = ptr_slot[0] == 0xFF && ptr_slot[1..16] == [0u8; 15];
    let is_chained = ptr_slot[0..12] == [0xFFu8; 12] && le32(ptr_slot, 12)? > off0;
    require!(
        (is_terminal || is_chained) && ptr_slot[16..24] == [0u8; 8],
        has_non_ascii_text_byte(prefix.get(..48)?)
    );
    Some(Label::FastgraphFgi)
}

/// FBCHUNKS Chunked Package (.ngp).
pub(crate) fn verify_fbchunks_ngp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FBCHUNKS_NGP_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"FBCHUNKS\x01\0"),
        (1..=65536).contains(&le32(prefix, 10).unwrap_or(0))
    );
    Some(Label::FbchunksNgp)
}

/// FIFA Editor Tool Mod Package (`.fifamod`).
pub(crate) fn verify_fifa_fifamod(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FIFA_FIFAMOD_SIZE,
        prefix.len() >= 24,
        starts(prefix, FIFA_FIFAMOD_MAGIC),
        (1..=8).contains(&u8_at(prefix, 4)?),
        (4..=16).contains(&u8_at(prefix, 5)?),
        get_slice(prefix, 6, 10)?.iter().all(|&b| b.is_ascii_alphanumeric())
    );
    Some(Label::FifaFifamod)
}

/// ASCII / Enterbrain 2D Fighter Maker 95 Game Data container (`.kgt` / `.bg`).
///
/// Header layout:
/// - `@0..16`: `"KGTGAME"` followed by 9 NUL padding bytes
pub(crate) fn verify_fighter_maker_kgt(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 16] = bytes!(b"KGTGAME", &[0, 0, 0, 0, 0, 0, 0, 0, 0]); // @0..16

    require!(size >= MIN_FIGHTER_MAKER_KGT_SIZE, prefix.len() >= 16, starts(prefix, MAGIC));
    Some(Label::FighterMakerKgt)
}

/// Sports Interactive Football Manager Data Archive (.fmf, Sports Interactive).
pub(crate) fn verify_fm_fmf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FM_FMF_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\x02\x01afe.\x08\0\0"),
        get_slice(prefix, 13, 28)? == b"\0\0\0\0\x11\0\0\0\0\0\0\0\0\x10\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::FmFmf)
}

/// FreeArc Archive (2006) Bulat Ziganshin (`"ArC\x01"`).
///
/// Header layout:
/// - `@0..4`: `"ArC\x01"` (`aSIGNATURE`)
/// - `@4`: `u8` block type (`0` = DATA, `1` = DIR, `2` = FOOTER, `3` = DESCR, `4` = HEADER, `5` = RECOVERY, `6` = DEBUG)
/// - `@5..8`: block descriptor bytes (non-zero binary metadata in valid archives)
pub(crate) fn verify_freearc(prefix: &[u8], size: u64) -> Option<Label> {
    const BLOCK_TYPE_OFFSET: usize = 4; // @4: u8 FreeArc block type (0..=6)
    const MAX_BLOCK_TYPE: u8 = 6; // DATA(0)..DEBUG(6)
    const HEADER_LEN: usize = MIN_FREEARC_SIZE as usize; // @0..8: 4B magic + 4B block prefix
    require!(
        size >= MIN_FREEARC_SIZE,
        prefix.len() >= HEADER_LEN,
        starts(prefix, FREEARC_MAGIC),
        u8_at(prefix, BLOCK_TYPE_OFFSET)? <= MAX_BLOCK_TYPE,
        get_slice(prefix, 4, HEADER_LEN)?.iter().any(|&b| b != 0)
    );
    Some(Label::Freearc)
}

/// FromSoftware Binder (`BND3`/`BND4`/`BHF3`/`BHF4`/`BHD5`) or TPF Texture Package (`TPF\0`, 2009 FromSoftware).
pub(crate) fn verify_fromsoft_binder(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_FROMSOFT_BINDER_SIZE, prefix.len() >= 24);
    let tag = get_slice(prefix, 0, 4)?;
    let ok = match tag {
        b"BND3" | b"BHF3" => {
            let ascii_ver = get_slice(prefix, 4, 12)?.iter().take_while(|&&b| b != 0).count();
            let printable =
                get_slice(prefix, 4, 4 + ascii_ver)?.iter().all(|b| b.is_ascii_graphic());
            let count_ok = le32(prefix, 16).is_some_and(|c| c <= 1_000_000)
                || be32(prefix, 16).is_some_and(|c| c <= 1_000_000);
            ascii_ver >= 4 && printable && count_ok && has_non_ascii_text_byte(prefix.get(..24)?)
        }
        b"BND4" | b"BHF4" => {
            get_slice(prefix, 4, 8)? == [0, 0, 0, 0]
                && u8_at(prefix, 8)? <= 1
                && u8_at(prefix, 9)? <= 1
                && (le64(prefix, 16) == Some(0x40) || be64(prefix, 16) == Some(0x40))
        }
        b"BHD5" => {
            u8_at(prefix, 4)? <= 1
                && u8_at(prefix, 5)? <= 1
                && (le32(prefix, 8) == Some(1) || be32(prefix, 8) == Some(1))
                && has_non_ascii_text_byte(prefix.get(..20)?)
        }
        b"TPF\0" => {
            let sz_ok = le32(prefix, 4).is_some_and(|s| s >= 16)
                || be32(prefix, 4).is_some_and(|s| s >= 16);
            sz_ok
                && u8_at(prefix, 12)? <= 5
                && u8_at(prefix, 14)? <= 2
                && has_non_ascii_text_byte(prefix.get(..20)?)
        }
        _ => false,
    };
    require!(ok);
    Some(Label::FromsoftBinder)
}

/// FromSoftware DCX Compressed Asset Wrapper (`.dcx`, 2009 FromSoftware).
pub(crate) fn verify_fromsoft_dcx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FROMSOFT_DCX_SIZE,
        prefix.len() >= 76,
        starts(prefix, FROMSOFT_DCX_MAGIC),
        eq_at(prefix, 24, b"DCS\0"),
        eq_at(prefix, 36, b"DCP\0"),
        matches!(get_slice(prefix, 40, 44)?, b"DFLT" | b"EDGE" | b"ZSTD" | b"KRAK"),
        eq_at(prefix, 68, b"DCA\0")
    );
    Some(Label::FromsoftDcx)
}

/// Frosty Mod Manager Package (`.fbmod`).
pub(crate) fn verify_frosty_fbmod(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FROSTY_FBMOD_SIZE,
        prefix.len() >= 24,
        starts(prefix, FROSTY_FBMOD_MAGIC),
        (1..=16).contains(&le32(prefix, FROSTY_VERSION_OFFSET)?),
        le32(prefix, FROSTY_DATA_OFF_OFFSET)? >= 16
    );
    Some(Label::FrostyFbmod)
}

/// Mark Overmars / YoYo Games GameMaker Legacy Project (`.gm6`, `.gmk`, `.gm81`).
///
/// Header layout:
/// - `@0..4`: `91 D5 12 00` (`u32 LE` `1234321` = `0x0012D591`)
/// - `@4..8`: `u32 LE` `version` (`530` GM5.3, `600` GM6, `700` GM7, `800` GM8, `810` GM8.1, `820` GM8.2)
pub(crate) fn verify_gamemaker_gmk(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4..8: u32 LE GameMaker project format version

    require!(
        size >= MIN_GAMEMAKER_GMK_SIZE,
        prefix.len() >= 12,
        starts(prefix, GAMEMAKER_GMK_MAGIC),
        matches!(le32(prefix, VERSION_OFF)?, 530 | 600 | 700 | 800 | 810 | 820)
    );
    Some(Label::GamemakerGmk)
}

/// Facepunch Garry's Mod Addon Archive (.gma).
pub(crate) fn verify_garrysmod_gma(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_GMOD_GMA_SIZE && prefix.len() >= 24);
    let version = u8_at(prefix, 4)?;
    let timestamp = le64(prefix, 13)?;
    require!((1..=3).contains(&version) && (1_000_000_000..=4_000_000_000).contains(&timestamp));
    Some(Label::Gma)
}

/// Damian Yerrick Game Boy Advance PinEight GBFS Archive (`.gbfs`).
pub(crate) fn verify_gba_gbfs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GBA_GBFS_SIZE,
        prefix.len() >= 32,
        starts(prefix, GBA_GBFS_MAGIC),
        le32(prefix, GBA_GBFS_TOTAL_LEN_OFFSET)? >= 32,
        (32..=4096).contains(&le16(prefix, GBA_GBFS_DIR_OFF_OFFSET)?),
        le16(prefix, GBA_GBFS_DIR_NMUL_OFFSET)? >= 1
    );
    Some(Label::GbaGbfs)
}

/// GCA / DGCA Compressed Archive (2000) Shin-ichi Tsuruta.
pub(crate) fn verify_gca_archive(prefix: &[u8], size: u64) -> Option<Label> {
    (size >= MIN_GCA_ARCHIVE_SIZE
        && prefix.len() >= MIN_GCA_ARCHIVE_SIZE as usize
        && has_non_ascii_text_byte(prefix.get(3..8)?))
    .then_some(Label::Gca)
}

/// GEA Installer Uninstall Package (.gea).
pub(crate) fn verify_gea_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GEA_ARCHIVE_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"GEA\0\x1f\0\x01\0\0\0\0\0"),
        le32(prefix, 27) == Some(u32::MAX)
    );
    Some(Label::GeaArchive)
}

/// ISI gMotor 2 MAS Game Archive (`.mas`, obfuscated `"CUBEMAS2.00"` 16-byte signature).
pub(crate) fn verify_gmotor_c8cf_mas(prefix: &[u8], size: u64) -> Option<Label> {
    const GMOTOR2_MAS_MAGIC: &[u8; 16] =
        hex2bin!("C8 CF D2 D8 CE D8 E6 C9 CA DD D8 BE BB A6 BF 90"); // @0..16

    require!(
        size >= MIN_GMOTOR_C8CF_MAS_SIZE,
        prefix.len() >= 32,
        starts(prefix, GMOTOR2_MAS_MAGIC),
        le32(prefix, 16) == Some(0), // @16..20: reserved/flags
        (1..=100_000).contains(&le32(prefix, 20)?), // @20..24: file entry count
        le32(prefix, 24)? > 0,       // @24..28: data size / table offset
        (1..=260).contains(&le32(prefix, 28)?)  // @28..32: first entry name length / metadata
    );
    Some(Label::IsiGmotor2Mas)
}

/// Godot Engine Encrypted File (`GDEC`, `FileAccessEncrypted`, Godot 3 AES-ECB / Godot 4 AES-CBC).
pub(crate) fn verify_godot_gdec(prefix: &[u8], size: u64) -> Option<Label> {
    const GODOT_GDEC_MAGIC: &[u8; 4] = b"GDEC"; // @0..4: Godot FileAccessEncrypted magic
    const MD5_OFF: usize = 4; // @4..20: 16-byte MD5 digest of plaintext
    const LEN_OFF: usize = 20; // @20..28: u64 LE unpadded plaintext byte length
    const V3_HDR_LEN: u64 = 28; // Godot 3 (AES-256-ECB): 4 magic + 16 MD5 + 8 length
    const V4_HDR_LEN: u64 = 44; // Godot 4 (AES-256-CBC): + 16-byte IV

    require!(
        size >= MIN_GODOT_GDEC_SIZE,
        prefix.len() >= 32,
        starts(prefix, GODOT_GDEC_MAGIC),
        has_non_ascii_text_byte(prefix.get(MD5_OFF..LEN_OFF)?)
    );
    let unpadded_len = le64(prefix, LEN_OFF)?;
    require!((1..=1_073_741_824).contains(&unpadded_len));
    let padded_len = unpadded_len.div_ceil(16).saturating_mul(16);
    require!(size == V3_HDR_LEN + padded_len || size == V4_HDR_LEN + padded_len);
    Some(Label::GodotGdec)
}

/// Godot Engine Resource Pack (.pck).
pub(crate) fn verify_godot_pck(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_GODOT_PCK_SIZE && prefix.len() >= 20);
    let pack_ver = le32(prefix, 4)?;
    let major = le32(prefix, 8)?;
    let minor = le32(prefix, 12)?;
    let patch = le32(prefix, 16)?;
    require!(pack_ver <= 4 && (1..=5).contains(&major) && minor <= 30 && patch <= 100);
    Some(Label::GodotPck)
}

/// Valve GoldSrc / Half-Life Texture WAD3 Archive (`.wad`).
pub(crate) fn verify_goldsrc_wad3(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 28,
        prefix.len() >= 28,
        starts(prefix, b"WAD3"),
        (1..=65_536).contains(&le32(prefix, 4)?),
        le32(prefix, 8)? >= 12,
    );
    Some(Label::GoldsrcWad3)
}

/// Harri Hirvola HA Compressed Archive (`.ha`, 1995 Harri Hirvola, `"HA"`).
pub(crate) fn verify_ha_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HA_ARCHIVE_SIZE,
        prefix.len() >= 22,
        starts(prefix, b"HA"),
        (1..=4096).contains(&le16(prefix, 2)?)
    );
    let vm = u8_at(prefix, 4)?;
    let method = vm & 0x0F;
    let comp_sz = le32(prefix, 5)? as u64;
    let orig_sz = le32(prefix, 9)? as u64;
    require!(
        (vm >> 4) == 2,
        (method <= 2 && comp_sz > 0 && comp_sz < size)
            || ((method == 14 || method == 15) && comp_sz == 0 && orig_sz == 0)
    );
    Some(Label::HaArchive)
}

/// Bungie Halo 3 / ODST / Reach BLF Container / Forge Variant (`.blf`, `.map`).
///
/// Header layout:
/// - `@0..4`: `"_blf"` chunk tag
/// - `@4..8`: `u32 BE` `chunk_size` (`0x30` = 48 bytes)
/// - `@8..10`: `u16 BE` `major_version` (`1`)
/// - `@10..12`: `u16 BE` `minor_version` (`2`)
/// - `@12..14`: Byte-order mark (`FF FE` or `FE FF`)
pub(crate) fn verify_halo_blf(prefix: &[u8], size: u64) -> Option<Label> {
    const CHUNK_LEN_OFF: usize = 4; // @4..8: u32 BE chunk size (0x30 = 48)
    const MAJOR_OFF: usize = 8; // @8..10: u16 BE major version (1)
    const MINOR_OFF: usize = 10; // @10..12: u16 BE minor version (2)
    const BOM_OFF: usize = 12; // @12..14: byte-order mark (FF FE or FE FF)
    const BOM_LE: &[u8; 2] = hex2bin!("FF FE");
    const BOM_BE: &[u8; 2] = hex2bin!("FE FF");

    require!(
        size >= MIN_HALO_BLF_SIZE,
        prefix.len() >= 48,
        starts(prefix, HALO_BLF_MAGIC),
        be32(prefix, CHUNK_LEN_OFF)? == 0x30,
        be16(prefix, MAJOR_OFF)? == 1,
        be16(prefix, MINOR_OFF)? == 2,
        eq_at(prefix, BOM_OFF, BOM_LE) || eq_at(prefix, BOM_OFF, BOM_BE)
    );
    Some(Label::HaloBlf)
}

/// HouSisong HDiffPatch Binary Differential Patch.
pub(crate) fn verify_hdiffpatch(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_HDIFFPATCH_SIZE, prefix.len() >= 12);
    if starts(prefix, b"HDIFF13\0") {
        return Some(Label::Hdiffpatch);
    }
    let hdr_len = if starts(prefix, b"HDIFFSF20&")
        || (starts(prefix, b"HDIFFW26")
            && u8_at(prefix, 8).unwrap_or(0).is_ascii_graphic()
            && u8_at(prefix, 9).unwrap_or(0) == 0)
    {
        10
    } else if starts(prefix, b"HDIFF13&")
        || starts(prefix, b"HDIFF19&")
        || starts(prefix, b"HDIFF20&")
    {
        8
    } else {
        return None;
    };
    let comp_slice = get_slice(prefix, hdr_len, prefix.len().min(36))?;
    let nul_pos = comp_slice.iter().position(|&b| b == 0)?;
    require!(
        comp_slice[..nul_pos]
            .iter()
            .all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'&')
    );
    Some(Label::Hdiffpatch)
}

/// LPACK 915827FD Resource Pack (.lpack).
pub(crate) fn verify_hhd_lpack(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HHD_LPACK_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"\x91\x58\x27\xfd\x72\xa1\xc1\x4c\x8a\xba\x56\x4e\xfc\x98\xdd\x21"),
        (1..=4096).contains(&le32(prefix, 16).unwrap_or(0))
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::HhdLpack)
}

/// Adam Dawson Highway Pursuit Game Data Archive (`b"HPDT"`).
pub(crate) fn verify_highway_pursuit_hpdt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_HIGHWAY_PURSUIT_HPDT_SIZE, prefix.len() >= 16, starts(prefix, b"HPDT"));
    let entry_cnt = le32(prefix, 4)? as u64;
    require!((1..=10_000).contains(&entry_cnt) && u8_at(prefix, 7)? == 0);
    Some(Label::Hpdt)
}

/// Hyper Compressed Archive (`.hyp`, 1989 Peter Sawatzki & Klaas Teschauer, `"\x1AHP"` / `"\x1AST"`).
pub(crate) fn verify_hyper_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HYPER_ARCHIVE_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"\x1AHP") || starts(prefix, b"\x1AST"),
        matches!(u8_at(prefix, 3), Some(0x25..=0x27)),
        (le32(prefix, 4)? as u64) < size
    );
    let fn_len = u8_at(prefix, 21)? as usize;
    require!(
        (1..=64).contains(&fn_len),
        prefix.len() >= 22 + fn_len,
        get_slice(prefix, 22, 22 + fn_len)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::HyperArchive)
}

/// IBM PC-DOS 6.3 / OS/2 MRNZ obfuscated installer file (`.co_` / `.ex_` / `.sy_`):
/// `"MRNZ\x88\xF0\x27\x33"` at `@0..8` + `u32 LE obfuscated_count` (`1..=255`) at `@8..12`.
pub(crate) fn verify_ibm_mrnz(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = bytes!(b"MRNZ", hex2bin!("88 F0 27 33")); // @0, 8 bytes
    const OBF_COUNT: usize = 8; // @8: u32 LE count of 0xAE-XORed bytes
    require!(size >= MIN_IBM_MRNZ_SIZE, starts(prefix, MAGIC));
    let obf_count = le32(prefix, OBF_COUNT)?;
    require!((1..=255).contains(&obf_count), size >= 12 + obf_count as u64);
    Some(Label::IbmMrnz)
}

/// IBM i / OS/400 Save File (`SAVF`, 1988 IBM).
pub(crate) fn verify_ibm_savf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_IBM_SAVF_SIZE && prefix.len() >= 1056);
    // 528-byte physical record RRN stride lock: Record 1 has RRN=1 at @512..516, Record 2 has RRN=2 at @1040..1044
    require!(be32(prefix, 512) == Some(1) && be32(prefix, 1040) == Some(2));
    // Record 1 payload check: either standard XOR-scrambled signature at @0..8 or @8..16 (`AE 31 6D C6 5C 4C AD 98`)
    // or unscrambled `0x19DB` + EBCDIC `"QSRDSSPC"` at @2..12.
    const XOR_KEY: &[u8; 8] = hex2bin!("AE 31 6D C6 5C 4C AD 98");
    let is_scrambled = get_slice(prefix, 0, 8)? == XOR_KEY || get_slice(prefix, 8, 16)? == XOR_KEY;
    let is_unscrambled = get_slice(prefix, 2, 12)? == b"\x19\xDB\xD8\xE2\xD9\xC4\xE2\xE2\xD7\xC3";
    require!(is_scrambled || is_unscrambled);
    Some(Label::IbmSavf)
}

/// IBM z/OS `AMATERSE` / `TRSMAIN` Tersed Data Set (`PACK`/`SPACK`, 1984 IBM).
pub(crate) fn verify_ibm_terse(prefix: &[u8], size: u64) -> Option<Label> {
    // 1. Native Mode (`01 89 69 A5` non-PDS or `07 89 69 A5` PDS) or 4-byte VM/CMS TERSE (`05 01 01 00`):
    if size >= MIN_IBM_TERSE_NATIVE_SIZE
        && prefix.len() >= 12
        && ((matches!(u8_at(prefix, 0)?, 0x01 | 0x07)
            && get_slice(prefix, 1, 4)? == [0x89, 0x69, 0xA5])
            || get_slice(prefix, 0, 4)? == [0x05, 0x01, 0x01, 0x00])
    {
        let first_code = be16(prefix, 4)? >> 4;
        require!(first_code < 260 && get_slice(prefix, 4, 12)?.iter().any(|&b| b != 0));
        return Some(Label::IbmTerse);
    }

    // 2. Compatibility Mode (12-byte header, `RECFM=FB, LRECL=1024` => `size % 1024 == 0`):
    require!(
        size >= MIN_IBM_TERSE_COMPAT_SIZE
            && size.is_multiple_of(1024)
            && prefix.len() >= 16
            && matches!(u8_at(prefix, 0)?, 0x02 | 0x05)
            && matches!(u8_at(prefix, 1)?, 0x00 | 0x01)
    );
    let pds_marker = be16(prefix, 2)?;
    let lrecl = be16(prefix, 4)?;
    let recfm = u8_at(prefix, 6)?;
    let ext_flag = u8_at(prefix, 7)?;
    let blksize = be16(prefix, 8)?;
    let reserved = be16(prefix, 10)?;
    let first_code = be16(prefix, 12)? >> 4;

    let is_recfm_u = (recfm & 0xC0) == 0xC0;
    let is_recfm_v = (recfm & 0xC0) == 0x40;
    let is_recfm_f = (recfm & 0xC0) == 0x80;

    require!(
        matches!(pds_marker, 0x0000 | 0x8969)
            && (is_recfm_u || (1..=32760).contains(&lrecl))
            && (recfm & 0xC0) != 0
            && (recfm & 0x01) == 0
            && ((u8_at(prefix, 1)? == 0x01 && is_recfm_v)
                || (u8_at(prefix, 1)? == 0x00 && (is_recfm_f || is_recfm_u)))
            && matches!(ext_flag, 0x00 | 0x01 | 0x04 | 0x80)
            && (blksize == 0 || (pds_marker == 0x8969 && (1..=32760).contains(&blksize)))
            && reserved == 0
            && first_code < 260
            && get_slice(prefix, 12, 16)?.iter().any(|&b| b != 0)
    );
    Some(Label::IbmTerse)
}

/// Free Lunch Design Icy Tower Replay Recording (ITR140, .itr).
pub(crate) fn verify_icy_tower_itr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ICY_TOWER_ITR_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"ITR140"),
        (1..=4096).contains(&le32(prefix, 6).unwrap_or(0)),
        prefix.get(10..32)?.contains(&0)
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::IcyTowerItr)
}

/// id Software TED5 Tile Editor Game Maps file (`"TED5"`, 1991 id Software).
pub(crate) fn verify_id_ted5(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ID_TED5_SIZE,
        prefix.len() >= 10,
        starts(prefix, b"TED5"),
        has_non_ascii_text_byte(prefix.get(4..10)?)
    );
    Some(Label::IdTed5)
}

/// IBM z/OS `IEBCOPY` Unloaded Partitioned Data Set (`COPYR1` / `COPYR2`, 1967 IBM).
pub(crate) fn verify_iebcopy_unloaded_pds(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_IEBCOPY_PDS_SIZE && prefix.len() >= 68);
    // `COPYR1` can start at `@0` (raw), `@4` (after 4B RDW), or `@8` (after 4B BDW + 4B RDW).
    let copyr1_off = [0usize, 4usize, 8usize].into_iter().find(|&off| {
        prefix.len() >= off + 56
            && u8_at(prefix, off).is_some_and(|b| (b & 0x3E) == 0)
            && eq_at(prefix, off + 1, hex2bin!("CA 6D 0F"))
    })?;
    if copyr1_off == 4 {
        let rdw_len = be16(prefix, 0)?;
        require!(matches!(rdw_len, 60 | 64) && be16(prefix, 2) == Some(0));
    } else if copyr1_off == 8 {
        let bdw_len = be16(prefix, 0)?;
        let rdw_len = be16(prefix, 4)?;
        require!(
            matches!(rdw_len, 60 | 64)
                && bdw_len >= rdw_len + 4
                && (be16(prefix, 2).unwrap_or(0) & 0x7FFF) == 0
                && be16(prefix, 6) == Some(0)
        );
    }

    let c1 = get_slice(prefix, copyr1_off, prefix.len())?;
    let ds1dsorg = be16(c1, 4)?;
    let ds1blkl = be16(c1, 6)?;
    let ds1lrecl = be16(c1, 8)?;
    let ds1recfm = c1[10];
    let unld_blkl = be16(c1, 14)?;
    let ucb_tbyt1 = c1[16];
    let ucb_tbyt3 = c1[18];
    let tracks_per_cyl = be16(c1, 26)?;

    // Validate Partitioned Organization (`DS1DSORG` = `0x0200` PO or `0x0300` POU),
    // valid MVS block/record lengths, MVS DASD UCB device class (`0x30` / `0x20`), and tracks/cyl (`1..=64`).
    require!(
        (ds1dsorg & 0xFEFF) == 0x0200
            && (1..=32760).contains(&ds1blkl)
            && ds1lrecl <= 32760
            && (ds1recfm & 0xC0) != 0
            && (ds1recfm & 0x01) == 0
            && (60..=32760).contains(&unld_blkl)
            && ucb_tbyt1 == 0x30
            && ucb_tbyt3 == 0x20
            && (1..=64).contains(&tracks_per_cyl)
    );
    Some(Label::IebcopyPds)
}

/// Technelysium IMP Compressed Archive (`.imp`, 1998 Technelysium, `"IMP\n"`).
pub(crate) fn verify_imp_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_IMP_ARCHIVE_SIZE,
        prefix.len() >= 42,
        starts(prefix, b"IMP\x0A"),
        (36..size).contains(&(le32(prefix, 4)? as u64)),
        le32(prefix, 8)? > 0
    );
    Some(Label::ImpArchive)
}

/// Checks whether a slice after `zlb\x1A` begins with a valid Inno Setup compressed chunk header.
fn is_inno_zlb_payload(payload: &[u8]) -> bool {
    const LZMA_PROPS_LC3_LP0_PB2: u8 = 0x5D;
    const LZMA_MAX_PROPS: u8 = 224;
    const LZMA2_MAX_DICT_PROP: u8 = 40;
    const BZIP2_BLOCK_MAGIC: &[u8; 6] = hex2bin!("31 41 59 26 53 59");
    const BZIP2_EOS_MAGIC: &[u8; 6] = hex2bin!("17 72 45 38 50 90");
    const ZSTD_FRAME_MAGIC: &[u8; 4] = hex2bin!("28 B5 2F FD");

    if payload.len() < 12 {
        return false;
    }
    let Some(b0) = u8_at(payload, 0) else {
        return false;
    };
    (b0 == 0x78
        && u8_at(payload, 1).is_some_and(|b1| (u16::from(b0) * 256 + u16::from(b1)) % 31 == 0))
        || (b0 == LZMA_PROPS_LC3_LP0_PB2 && u8_at(payload, 5) == Some(0x00))
        || (b0 <= LZMA2_MAX_DICT_PROP
            && u8_at(payload, 6) == Some(LZMA_PROPS_LC3_LP0_PB2)
            && u8_at(payload, 7) == Some(0x00))
        || (starts(payload, b"BZh")
            && u8_at(payload, 3).is_some_and(|lvl| (b'1'..=b'9').contains(&lvl))
            && (eq_at(payload, 4, BZIP2_BLOCK_MAGIC) || eq_at(payload, 4, BZIP2_EOS_MAGIC)))
        || (b0 <= LZMA2_MAX_DICT_PROP
            && (u8_at(payload, 1) == Some(0x01)
                || (u8_at(payload, 1).is_some_and(|ctrl| ctrl >= 0xE0)
                    && u8_at(payload, 6).is_some_and(|props| props <= LZMA_MAX_PROPS)
                    && u8_at(payload, 7) == Some(0x00))))
        || starts(payload, ZSTD_FRAME_MAGIC)
        || starts(payload, b"MZ")
}

/// Inno Setup data slice, setup banner, or `zlb\x1A` compressed stream.
pub(crate) fn verify_inno_setup(prefix: &[u8], size: u64) -> Option<Label> {
    const SLICE_IDS: [&[u8; 8]; 3] = [b"idska16\x1A", b"idska32\x1A", b"idskb32\x1A"];
    const BANNERS: [&[u8]; 5] = [
        b"Inno Setup Setup Data (",
        b"Inno Setup Messages (",
        b"Inno Setup Uninstall Log (",
        b"My Inno Setup Extensions Setup Data (",
        b"i1.2.10--",
    ];
    require!(size >= 16, prefix.len() >= 16);
    if (BANNERS.iter().any(|b| starts(prefix, b))
        && prefix.get(..prefix.len().min(64))?.iter().any(|&b| b == 0x1A || b == 0))
        || SLICE_IDS.iter().any(|id| starts(prefix, *id))
    {
        return Some(Label::InnoSetup);
    }
    require!(starts(prefix, b"zlb\x1A"), is_inno_zlb_payload(prefix.get(4..)?));
    Some(Label::InnoSetup)
}

/// Decompressed Inno Setup `TSetupHeader` stream (`[Inno_Setup_Header.bin]`):
/// starts at `@0` with a chain of `u32 LE` length-prefixed `TSetupHeader` strings (`AppName`, `AppVerName`,
/// `AppId`, `AppCopyright`, `AppPublisher`, …, `DefaultDirName`, `DefaultGroupName`) containing Inno Setup
/// brace constants (`{app}`, `{pf}`, `{tmp}`, `{group}`, `{autopf}`, in ANSI or UTF-16LE).
pub(crate) fn verify_inno_setup_header_stream(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_INNO_HDR_STREAM_SIZE: u64 = 256;
    const CONST_APP_A: &[u8] = b"{app}";
    const CONST_PF_A: &[u8] = b"{pf}";
    const CONST_TMP_A: &[u8] = b"{tmp}";
    const CONST_GROUP_A: &[u8] = b"{group}";
    const CONST_APP_W: &[u8] = b"{\0a\0p\0p\0}\0";
    const CONST_PF_W: &[u8] = b"{\0p\0f\0}\0";
    const CONST_TMP_W: &[u8] = b"{\0t\0m\0p\0}\0";
    const CONST_GROUP_W: &[u8] = b"{\0g\0r\0o\0u\0p\0}\0";

    require!(size >= MIN_INNO_HDR_STREAM_SIZE, prefix.len() >= 128);
    let len0 = le32(prefix, 0)? as usize;
    require!((2..=512).contains(&len0)); // guarantees get_slice(prefix, 2, 4)? == [0, 0] (rejects printable ASCII)

    let mut pos = 0usize;
    let mut non_empty = 0usize;
    for _ in 0..12 {
        let slen = le32(prefix, pos)? as usize;
        require!(slen <= 2048, pos + 4 + slen <= prefix.len());
        if slen >= 2 {
            non_empty += 1;
        }
        pos += 4 + slen;
    }
    require!(non_empty >= 3);
    let window = get_slice(prefix, 0, prefix.len().min(1024))?;
    let has_inno_const = contains(window, CONST_APP_A)
        || contains(window, CONST_PF_A)
        || contains(window, CONST_TMP_A)
        || contains(window, CONST_GROUP_A)
        || contains(window, CONST_APP_W)
        || contains(window, CONST_PF_W)
        || contains(window, CONST_TMP_W)
        || contains(window, CONST_GROUP_W);
    require!(has_inno_const);
    Some(Label::InnoSetup)
}

/// MindVision Installer VISE Metadata Container (ESIVCSIM, miscdata.xyz).
pub(crate) fn verify_installer_vise_xyz(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_INSTALLER_VISE_XYZ_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"ESIVCSIM\x01\0\0\0"),
        (1..=256).contains(&le16(prefix, 12).unwrap_or(0))
    );
    require!(has_non_ascii_text_byte(prefix.get(..24)?));
    Some(Label::InstallerViseXyz)
}

/// InstallShield Cabinet archive (`ISc(`).
pub(crate) fn verify_installshield_cab(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 24,
        prefix.len() >= 24,
        starts(prefix, b"ISc("),
        prefix.get(4..12)?.contains(&0),
    );
    let _ = le32(prefix, 4)?;
    let _ = le32(prefix, 12)?;
    let _ = le32(prefix, 16)?;
    Some(Label::Iscab)
}

/// InstallShield `setup.ibt` / `setup.boot` multi-member `SZDD` archive: starts with `"setup.dl_\0setup.dll\0"`
/// (or `"setup.dl_\0Setup.dll\0"`) followed by version and member size strings and an embedded `SZDD\x88\xF0\x27\x33` stream.
pub(crate) fn verify_installshield_ibt(prefix: &[u8], size: u64) -> Option<Label> {
    const HEADER_LOWER: &[u8] = b"setup.dl_\0setup.dll\0"; // @0, 20 bytes
    const HEADER_MIXED: &[u8] = b"setup.dl_\0Setup.dll\0"; // @0, 20 bytes (InstallShield 7.x setup.boot)
    const SZDD: &[u8] = bytes!(b"SZDD", hex2bin!("88 F0 27 33"));
    require!(
        size >= MIN_INSTALLSHIELD_IBT_SIZE,
        starts(prefix, HEADER_LOWER) || starts(prefix, HEADER_MIXED)
    );
    let hi = prefix.len().min(64);
    memchr::memmem::find(get_slice(prefix, 20, hi)?, SZDD)?;
    Some(Label::IsIbt)
}

/// InstallShield Package Directory (`setup.pkg`, 1995 Stirling Technologies / InstallShield).
pub(crate) fn verify_installshield_pkg(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 2] = hex2bin!("4A A3"); // @0..2: 0xA34A LE magic
    const SIZE1_OFFSET: usize = 2; // @2..6: u32 LE total directory table size
    const VERSION_OFFSET: usize = 6; // @6..10: u32 LE format version (== 2)
    const SIZE2_OFFSET: usize = 10; // @10..14: u32 LE inner table size (<= size1)
    const COUNT_OFFSET: usize = 14; // @14..16: u16 LE package entry count (> 0)
    require!(
        size >= MIN_INSTALLSHIELD_PKG_SIZE,
        prefix.len() >= 16,
        starts(prefix, MAGIC),
        le32(prefix, VERSION_OFFSET)? == 2
    );
    let s1 = le32(prefix, SIZE1_OFFSET)?;
    let s2 = le32(prefix, SIZE2_OFFSET)?;
    let count = le16(prefix, COUNT_OFFSET)?;
    require!(s2 > 0, s2 <= s1, s1 <= 16_777_216, (1..=4096).contains(&count));
    Some(Label::InstallshieldPkg)
}

/// InstallShield v3 compressed installer archive (`_INST32I.EX_`): `2A AB 79 D8`, `u16 BE version == 1` @4,
/// banner `"Copyright (c) 1990-1995 Stirling Technologies"` @8.
pub(crate) fn verify_installshield_v3(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = hex2bin!("2A AB 79 D8"); // @0
    const VERSION: usize = 4; // @4: u16 BE
    const BANNER: usize = 8; // @8
    require!(size >= MIN_INSTALLSHIELD_V3_SIZE, starts(prefix, MAGIC), be16(prefix, VERSION)? == 1);
    require!(
        eq_at(prefix, BANNER, b"Copyright (c) 1990-"),
        scan_range(prefix, BANNER, 96, b"Stirling Technologies")
    );
    Some(Label::Is3Archive)
}

/// InstallShield Z (`.z`) or Stirling TSComp archive.
pub(crate) fn verify_installshield_z(prefix: &[u8], size: u64) -> Option<Label> {
    const ISZ_MAGIC: &[u8; 4] = le32_bytes!(0x8C65_5D13);
    const TSCOMP_MAGIC: &[u8; 4] = le32_bytes!(0x8C13_5D65);
    require!(size >= 30, prefix.len() >= 30);
    if starts(prefix, ISZ_MAGIC) {
        require!(prefix.len() >= 51, eq_at(prefix, 4, &[0x3A, 0x01, 0x02, 0x00]));
        return Some(Label::Isz);
    }
    require!(
        starts(prefix, TSCOMP_MAGIC),
        eq_at(prefix, 4, &[0x08, 0x01, 0x03, 0x00]),
        u8_at(prefix, 8)? <= 2,
    );
    Some(Label::Isz)
}

/// Image Space Incorporated FRAC Encrypted Vehicle Archive (.car, 2005) ISI.
pub(crate) fn verify_isi_frac_car(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ISI_FRAC_CAR_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"FRAC\0\0\0\0"),
        get_slice(prefix, 12, 20)? == b"PYTC\0\0\0\0",
        get_slice(prefix, 28, 36)? == b"INIC\0\0\0\0"
    );
    Some(Label::IsiFracCar)
}

/// ISI gMotor / rFactor MAS Archive (`.mas` / `.gtl`, 2001 Image Space Incorporated; `"GMOTORMAS10\0\0\0\0\0"` or `"CUBEMAS4.10\0"`).
pub(crate) fn verify_isi_gmotor_mas(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ISI_GMOTOR_MAS_SIZE, prefix.len() >= 32);
    if starts(prefix, b"GMOTORMAS10\x00\x00\x00\x00\x00") {
        require!(le32(prefix, 16)? == 40);
        let entry_cnt = le32(prefix, 24)? as u64;
        require!((1..=100_000).contains(&entry_cnt));
        return Some(Label::Mas);
    }
    if starts(prefix, b"CUBEMAS")
        && u8_at(prefix, 7)?.is_ascii_digit()
        && u8_at(prefix, 8)? == b'.'
        && u8_at(prefix, 9)?.is_ascii_digit()
        && get_slice(prefix, 11, 16)? == [0u8; 5]
    {
        let entry_cnt = le32(prefix, 16)? as u64;
        require!((1..=100_000).contains(&entry_cnt));
        return Some(Label::Mas);
    }
    None
}

/// IW4x Call of Duty: Modern Warfare 2 Multiplayer Heatmap (.iw4heat).
pub(crate) fn verify_iw4x_heat(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_IW4X_HEAT_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"IW4XHEAT\x04\0\0\0"),
        (1..=64).contains(&le32(prefix, 12).unwrap_or(0))
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::Iw4xHeat)
}

/// Atari ST JAM Packer compressed stream (`"LSD!"`, `"LZH!"`, or `"LZW!"`, 1990 JAM / Automation).
pub(crate) fn verify_jam_packer(prefix: &[u8], size: u64) -> Option<Label> {
    const UNPACKED_OFFSET: usize = 4; // @4..8: u32 BE unpacked size
    const PACKED_OFFSET: usize = 8; // @8..12: u32 BE packed size

    require!(
        size >= MIN_JAM_PACKER_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"LSD!") || starts(prefix, b"LZH!") || starts(prefix, b"LZW!")
    );
    let unpacked = be32(prefix, UNPACKED_OFFSET)? as u64;
    let packed = be32(prefix, PACKED_OFFSET)? as u64;
    require!((1..=0x0400_0000).contains(&unpacked), (1..=size).contains(&packed));
    Some(Label::JamPacker)
}

/// GMM Entertainment JFX1 / JPK1: `"JFX1"` followed by a non-empty header.
pub(crate) fn verify_jfx1(prefix: &[u8], size: u64) -> Option<Label> {
    const HEADER: core::ops::Range<usize> = 4..12;
    require!(size >= MIN_JFX1_SIZE, prefix.len() >= HEADER.end, starts(prefix, b"JFX1"));
    require!(
        get_slice(prefix, HEADER.start, HEADER.end)?.iter().any(|&b| b != 0),
        get_slice(prefix, HEADER.start, HEADER.end)?.contains(&0)
    );
    Some(Label::Jfx1)
}

/// JGXP Game Resource Package (.gxp).
pub(crate) fn verify_jgxp_package(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JGXP_PACKAGE_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"JGXP\x7f\xfd\xff\xff\x1f\xfe\xff\xff"),
        le32(prefix, 16) == Some(0x10)
    );
    Some(Label::JgxpPackage)
}

/// JSOS SOS Standard Filesystem 1.x (.md8, MatchWare).
pub(crate) fn verify_jsos_md8(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JSOS_MD8_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"JSOS"),
        get_slice(prefix, 22, 46)? == b"SOS Standard filesystem "
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::JsosMd8)
}

/// Java Web Start Standalone Launcher Bundle (.p2).
pub(crate) fn verify_jwlaunch_p2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JWLAUNCH_P2_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"0\0lib/jwstandalonelaunch.jar\0"),
        get_slice(prefix, 34, 38)? == b"PK\x03\x04"
    );
    require!(has_non_ascii_text_byte(prefix.get(..44)?));
    Some(Label::JwlaunchP2)
}

/// Kaspersky Anti-Virus UPDS Gzip Signature Database (.kdc, 2006) Kaspersky Lab.
pub(crate) fn verify_kaspersky_upds(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KASPERSKY_UPDS_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"UPDS"),
        get_slice(prefix, 8, 16)? == b"\0\0\x02\0\x1f\x8b\x08\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..24)?));
    Some(Label::KasperskyUpds)
}

/// Knowledge Dynamics RED: `"RR\x01\x29"`, `u16 LE name_len` @24 (1..=64), then the printable stored 8.3 filename
/// (containing a `.`) at `@26`.
pub(crate) fn verify_kd_red(prefix: &[u8], size: u64) -> Option<Label> {
    const NAME_LEN: usize = 24; // @24: u16 LE
    const NAME: usize = 26; // @26: stored filename
    require!(size >= MIN_KD_RED_SIZE, starts(prefix, b"RR\x01\x29"));
    let name_len = le16(prefix, NAME_LEN)? as usize;
    require!((1..=64).contains(&name_len), prefix.len() >= NAME + name_len);
    let name = get_slice(prefix, NAME, NAME + name_len)?;
    let printable = name.iter().take_while(|&&b| b != 0).count();
    require!(printable >= 1, name[..printable].iter().all(|&b| b.is_ascii_graphic() || b == b' '));
    Some(Label::KdRed)
}

/// KGB Archiver Compressed Archive (.kgb).
pub(crate) fn verify_kgb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KGB_SIZE
            && prefix.len() >= 12
            && starts(prefix, b"KGB_arch -")
            && get_slice(prefix, 10, prefix.len().min(256))?
                .iter()
                .any(|&b| (b < 0x20 && !matches!(b, b'\t' | b'\n' | b'\r')) || b >= 0x7F)
    );
    Some(Label::Kgb)
}

/// APLUS / Arc System Works Kill la Kill: IF DGKP Asset Archive (.pac / .lyt).
pub(crate) fn verify_kill_la_kill_dgkp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KILL_LA_KILL_DGKP_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"DGKP\0\0\x02\0\0\0\0\0"),
        le32(prefix, 16) == Some(0x30)
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::KillLaKillDgkp)
}

/// KiriKiri2 / KiriKiriZ Visual Novel Archive (.xp3).
pub(crate) fn verify_kirikiri_xp3(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KIRIKIRI_XP3_SIZE
            && prefix.len() >= 19
            && get_slice(prefix, 0, 11)? == b"XP3\r\n \n\x1A\x8B\x67\x01"
    );
    let index_off = le64(prefix, 11)?;
    require!(index_off >= 19 && (index_off < size || index_off == 0x17));
    Some(Label::Xp3)
}

/// KOEI Romance of the Three Kingdoms XI LS11 Data Archive (.s11).
pub(crate) fn verify_koei_ls11_s11(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KOEI_LS11_S11_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"LS11"),
        get_slice(prefix, 4, 48)?.iter().all(|&b| b == 0)
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::KoeiLs11S11)
}

/// TimeGate Studios Kohan KA Sound Archive (.snd, 2001) TimeGate Studios.
pub(crate) fn verify_kohan_snd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KOHAN_SND_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"KA Archive\0\0\0\0\0\0\0\0\0\0"),
        (1..=4096).contains(&le16(prefix, 20).unwrap_or(0))
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::KohanSnd)
}

/// KROM / MORK 64-Bit Resource Table (.1 / .10d0).
pub(crate) fn verify_krom_mork_res(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KROM_MORK_RES_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"KROM\0\0\0\0") || starts(prefix, b"MORK\0\0\0\0"),
        (1..=0x10_0000).contains(&le64(prefix, 8).unwrap_or(0)),
        (1..=0x1_0000).contains(&le64(prefix, 16).unwrap_or(0)),
        (1..=0x10_0000).contains(&le64(prefix, 24).unwrap_or(0))
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::KromMorkRes)
}

/// Microsoft Compress KWAJ archive (`KWAJ\x88\xF0\x27`).
pub(crate) fn verify_kwaj(prefix: &[u8], size: u64) -> Option<Label> {
    const PREFIX: &[u8; 7] = bytes!(b"KWAJ", &[0x88, 0xF0, 0x27]);
    require!(
        size >= 14,
        prefix.len() >= 14,
        starts(prefix, PREFIX),
        matches!(u8_at(prefix, 7)?, 0xD1 | 0x33),
    );
    Some(Label::Kwaj)
}

/// Larian Studios LSPK Package (`.pak` / `.lsv`, Divinity: Original Sin 1/2, Baldur's Gate 3).
///
/// Header layout:
/// - `@0..4`: `"LSPK"` magic
/// - `@4..8`: `u32 LE` `version` (`7..=20`, e.g., `7, 9, 10, 13, 15, 16, 18`)
pub(crate) fn verify_larian_lspk(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4..8: u32 LE LSPK format version (7..=20)

    require!(
        size >= MIN_LARIAN_LSPK_SIZE,
        prefix.len() >= 16,
        starts(prefix, LARIAN_LSPK_MAGIC),
        (7..=20).contains(&le32(prefix, VERSION_OFF)?)
    );
    Some(Label::LarianLspk)
}

/// Marcus Geelnard `liblzg` Compressed Stream (`.lzg`, 2010 Marcus Geelnard, `"LZG"`).
pub(crate) fn verify_liblzg_compressed(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LIBLZG_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"LZG"),
        be32(prefix, 3)? > 0,
        (be32(prefix, 7)? as u64) + 16 == size,
        u8_at(prefix, 15)? <= 1
    );
    Some(Label::Liblzg)
}

/// Monolith LithTech Engine RezMgr Resource Archive (.rez, 1998) Monolith Productions.
pub(crate) fn verify_lithtech_rez(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LITHTECH_REZ_SIZE,
        prefix.len() >= 128,
        starts(prefix, b"\r\nRezMgr Version 1 Copyright (C) 1995 MONOLITH INC")
            || starts(prefix, b"\r\nWinRez "),
        u8_at(prefix, 126)? == 0x1A
    );
    Some(Label::LithtechRez)
}

/// LOB's File Compressor / Multipak Amiga Data Stream (`.lob`, 1991).
pub(crate) fn verify_lob_multipak(prefix: &[u8], size: u64) -> Option<Label> {
    const METHOD_OFFSET: usize = 4; // @4: compression method (1..=6)
    const CMPR_LEN_OFFSET: usize = 8; // @8..12: u32 BE compressed stream length

    require!(
        size >= MIN_LOB_MULTIPAK_SIZE,
        prefix.len() >= 12,
        matches!(u8_at(prefix, 0)?, 0x01..=0x03) && eq_at(prefix, 1, b"LOB"),
        (1..=6).contains(&u8_at(prefix, METHOD_OFFSET)?)
    );
    let orig_len_24 = be32(prefix, 4)? & 0x00FF_FFFF;
    let cmpr_len = be32(prefix, CMPR_LEN_OFFSET)? as u64;
    require!(orig_len_24 > 0, cmpr_len == 0 || cmpr_len + 12 <= size);
    Some(Label::LobMultipak)
}

/// Logitech `LGEXPAND` v2 Compressed Installer File (`.ex_` / `.dl_` / `.cu_`, 1995 Logitech).
pub(crate) fn verify_logitech_compress(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 2] = hex2bin!("DA FA"); // @0..2: 0xFADA LE magic
    const FLAGS_OFFSET: usize = 2; // @2..4: u16 LE flags / MS-DOS time
    const ORIG_SIZE_OFFSET: usize = 4; // @4..8: u32 LE uncompressed size
    const DCL_MODE_OFFSET: usize = 8; // @8: PKWARE DCL Implode mode (0x00 binary)
    const DCL_DICT_OFFSET: usize = 9; // @9: PKWARE DCL dictionary size (0x04=1KB, 0x05=2KB, 0x06=4KB)
    require!(
        size >= MIN_LOGITECH_COMPRESS_SIZE,
        prefix.len() >= 12,
        starts(prefix, MAGIC),
        le16(prefix, FLAGS_OFFSET)? != 0,
        (16..=16_777_216).contains(&le32(prefix, ORIG_SIZE_OFFSET)?),
        u8_at(prefix, DCL_MODE_OFFSET)? == 0x00,
        matches!(u8_at(prefix, DCL_DICT_OFFSET)?, 0x04..=0x06)
    );
    Some(Label::LogitechCompress)
}

/// Lotus Ami Pro CMZ Compressed Archive (`.cmz`, 1992 Lotus, `"Clay"`).
pub(crate) fn verify_lotus_cmz(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LOTUS_CMZ_SIZE,
        prefix.len() >= 22,
        starts(prefix, b"Clay"),
        (le32(prefix, 4)? as u64) <= size,
        le32(prefix, 8)? > 0,
        u8_at(prefix, 19)? == 0
    );
    let fn_len = le16(prefix, 16)? as usize;
    require!(
        (1..=64).contains(&fn_len),
        prefix.len() >= 20 + fn_len,
        get_slice(prefix, 20, 20 + fn_len)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::LotusCmz)
}

/// LucasArts Dark Forces GOB / Grim Fandango LAB Archive (`.gob` / `.lab`, 1995 LucasArts).
pub(crate) fn verify_lucasarts_gob(prefix: &[u8], size: u64) -> Option<Label> {
    const GOB_MAGIC_LF: &[u8; 4] = b"GOB\n"; // @0..4: Dark Forces v1.0 GOB\x0A
    const GOB_MAGIC_20: &[u8; 4] = b"GOB "; // @0..4: GOB\x20 variant
    const LAB_MAGIC: &[u8; 8] = b"LABN\0\0\x01\0"; // @0..8: Outlaws / Grim Fandango LABN v1.0
    const DIR_OFF_OFFSET: usize = 4; // @4..8: u32 LE master directory offset

    require!(size >= MIN_LUCASARTS_GOB_SIZE, prefix.len() >= 12);
    if starts(prefix, LAB_MAGIC) {
        require!(
            prefix.len() >= 16,
            (1..=65_536).contains(&le32(prefix, 8)?),
            le32(prefix, 12)? > 0
        );
        return Some(Label::LucasartsGob);
    }
    require!(starts(prefix, GOB_MAGIC_LF) || starts(prefix, GOB_MAGIC_20));
    let dir_off = le32(prefix, DIR_OFF_OFFSET)? as u64;
    require!((8..size).contains(&dir_off));
    Some(Label::LucasartsGob)
}

/// LZF compressed stream.
pub(crate) fn verify_lzf(prefix: &[u8], size: u64) -> Option<Label> {
    // LibLZF framed compressed stream ("ZV\x00" uncompressed / "ZV\x01" compressed, 2000) Marc Alexander Lehmann
    if size >= MIN_LZF_SIZE
        && prefix.len() >= MIN_LZF_SIZE as usize
        && ((u8_at(prefix, 2)? == 0 && be16(prefix, 3).unwrap_or(0) > 0)
            || (u8_at(prefix, 2).unwrap_or(0) == 1
                && be16(prefix, 3).unwrap_or(0) > 0
                && be16(prefix, 5).unwrap_or(0) > 0))
    {
        Some(Label::Lzf)
    } else {
        None
    }
}

/// Apple LZFSE / LZVN compressed block stream (`bvx-` / `bvx1` / `bvx2` / `bvxn`).
pub(crate) fn verify_lzfse(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 8,
        prefix.len() >= 8,
        starts(prefix, b"bvx-")
            || starts(prefix, b"bvx1")
            || starts(prefix, b"bvx2")
            || starts(prefix, b"bvxn"),
        le32(prefix, 4)? <= 0x1000_0000,
    );
    Some(Label::Lzfse)
}

/// 8-byte `u64 LE` uncompressed-size prefixed Apple LZFSE stream (`@8`: `bvx*`).
pub(crate) fn verify_lzfse_length_prefixed(prefix: &[u8], size: u64) -> Option<Label> {
    const STREAM_OFFSET: usize = 8;
    const MAX_UNCOMPRESSED_SIZE: u64 = 0x4000_0000;
    require!(
        size >= 16,
        prefix.len() >= 16,
        eq_at(prefix, STREAM_OFFSET, b"bvx2")
            || eq_at(prefix, STREAM_OFFSET, b"bvx1")
            || eq_at(prefix, STREAM_OFFSET, b"bvx-")
            || eq_at(prefix, STREAM_OFFSET, b"bvxn"),
    );
    let uncomp_len = le64(prefix, 0)?;
    require!(
        (1..=MAX_UNCOMPRESSED_SIZE).contains(&uncomp_len),
        le32(prefix, STREAM_OFFSET + 4)? <= 0x1000_0000,
    );
    Some(Label::Lzfse)
}

/// `LZM1` Chunked Compressed Stream (`.lzm`, `"LZM1"`).
pub(crate) fn verify_lzm_compressed(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LZM_COMPRESSED_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"LZM1"),
        le32(prefix, 4) == Some(1)
    );
    let chunk_sz = le32(prefix, 8)?;
    require!((1024..=0x4000_0000).contains(&chunk_sz), chunk_sz.is_power_of_two());
    Some(Label::LzmCompressed)
}

/// lzop LZO compressed file (`\x89LZO\0\r\n\x1a\n`).
pub(crate) fn verify_lzop(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 9] = b"\x89LZO\x00\r\n\x1a\n";
    require!(size >= 18, prefix.len() >= 18, starts(prefix, MAGIC));
    let version = be16(prefix, 9)?;
    let flags_off = if version >= 0x0940 { 17usize } else { 15usize };
    require!(prefix.len() >= flags_off + 4);
    let _ = be32(prefix, flags_off)?;
    Some(Label::Lzop)
}

/// Charles Bloom `LZP1` Compressed Stream (`.lzp`, 1996 Charles Bloom, `"LZP1"`).
pub(crate) fn verify_lzp1_compressed(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LZP1_SIZE,
        prefix.len() >= 8,
        starts(prefix, b"LZP1"),
        has_non_ascii_text_byte(prefix.get(4..prefix.len().min(16))?)
    );
    Some(Label::Lzp1)
}

/// Emmanuel Marty `LZSA` Compressed Stream (`.lzsa`, 2019 Emmanuel Marty, `7B 9E`).
pub(crate) fn verify_lzsa_compressed(prefix: &[u8], size: u64) -> Option<Label> {
    const LZSA_MAGIC: &[u8; 2] = hex2bin!("7B 9E"); // @0..2
    require!(size >= MIN_LZSA_SIZE, prefix.len() >= 6, starts(prefix, LZSA_MAGIC));
    let flags = u8_at(prefix, 2)?;
    let blk1_len = (le16(prefix, 3)? as usize) | ((u8_at(prefix, 5)? as usize) << 16);
    require!(
        (flags & 0x18) == 0,
        (flags & 0xE0) >= 0x20
            || (flags == 0
                && blk1_len > 0
                && 6 + blk1_len <= size as usize
                && (prefix.len() < 9 + blk1_len || u8_at(prefix, 8 + blk1_len) == Some(0)))
    );
    Some(Label::Lzsa)
}

/// Ideaworks3D / Marmalade SDK Derbh Archive (`.dz`).
pub(crate) fn verify_marmalade_dz(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MARMALADE_DZ_SIZE,
        prefix.len() >= 16,
        starts(prefix, MARMALADE_DZ_MAGIC),
        le16(prefix, MARMALADE_FILES_OFFSET)? >= 1,
        le16(prefix, MARMALADE_DIRS_OFFSET)? >= 1,
        u8_at(prefix, 8) == Some(0),
        u8_at(prefix, 9)?.is_ascii_graphic()
    );
    Some(Label::MarmaladeDz)
}

/// Maxis Database Packed File (`DBPF`, `.package` / `.dat`, 2003 Maxis / Electronic Arts).
pub(crate) fn verify_maxis_dbpf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_MAXIS_DBPF_SIZE && prefix.len() >= 68 && starts(prefix, MAXIS_DBPF_MAGIC));
    let major = le32(prefix, 4)?;
    let minor = le32(prefix, 8)?;
    let idx_count = le32(prefix, 36)?;
    let idx_off_v1 = le32(prefix, 40)? as u64;
    let idx_size = le32(prefix, 44)? as u64;
    let idx_off_v2 = le32(prefix, 64)? as u64;
    let idx_off = if major >= 2 && idx_off_v1 == 0 { idx_off_v2 } else { idx_off_v1 };
    require!(
        (1..=3).contains(&major)
            && minor <= 4
            && idx_count <= 1_000_000
            && (idx_count == 0 || (idx_off >= 96 && idx_off + idx_size <= size))
    );
    Some(Label::Dbpf)
}

/// Dietrich MDCD Compressed Archive (`.mdcd` / `.lz`, 1990).
pub(crate) fn verify_mdcd_archive(prefix: &[u8], size: u64) -> Option<Label> {
    const MDCD_MAGIC_UPPER: &[u8; 4] = b"MDCD"; // @0..4: "MDCD"
    const MDCD_MAGIC_MIXED: &[u8; 6] = b"MDmd\x0A\x01"; // @0..6: "MDmd\x0A\x01"

    require!(size >= MIN_MDCD_ARCHIVE_SIZE, prefix.len() >= 26);
    if starts(prefix, MDCD_MAGIC_MIXED) {
        require!(prefix.len() >= 54);
        let pascal_len = u8_at(prefix, 41)? as usize;
        let is_pascal_fn = (1..=12).contains(&pascal_len)
            && get_slice(prefix, 42, 42 + pascal_len)?.iter().all(|&b| b.is_ascii_graphic());
        let is_space_fn = get_slice(prefix, 41, 53)?.contains(&b'.')
            && get_slice(prefix, 41, 53)?.iter().all(|&b| (0x20..=0x7E).contains(&b));
        require!(is_pascal_fn || is_space_fn);
        return Some(Label::Mdcd);
    }
    require!(starts(prefix, MDCD_MAGIC_UPPER));
    let fn_len = u8_at(prefix, 12)? as usize;
    require!(
        (1..=12).contains(&fn_len),
        get_slice(prefix, 13, 13 + fn_len)?.iter().all(|&b| b.is_ascii_graphic())
    );
    Some(Label::Mdcd)
}

/// Mindware Studios Meng Engine Archive (`.mfs`, 2005 Mindware Studios).
pub(crate) fn verify_mindware_mfs_archive(prefix: &[u8], size: u64) -> Option<Label> {
    const MENG_PACK_MAGIC: &[u8; 12] = bytes!(b"MENG", le32_bytes!(0x1234_5678), b"PACK"); // @0..12: "MENG" + 0x12345678 LE + "PACK"
    const VERSION_CHAR_OFFSET: usize = 12; // @12: ASCII version digit ('1')
    const NULL_SEP_OFFSET: usize = 13; // @13: 0x00 null terminator after version digit

    require!(
        size >= MIN_MINDWARE_MFS_SIZE,
        prefix.len() >= 20,
        starts(prefix, MENG_PACK_MAGIC),
        u8_at(prefix, VERSION_CHAR_OFFSET)?.is_ascii_digit(),
        u8_at(prefix, NULL_SEP_OFFSET) == Some(0)
    );
    Some(Label::MindwareMfs)
}

/// WeMade Entertainment The Legend of Mir 2 Tile Map (.map, 2001) WeMade.
pub(crate) fn verify_mir_map(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MIR_MAP_SIZE,
        prefix.len() >= 32,
        (1..=2048).contains(&le16(prefix, 0).unwrap_or(0)),
        (1..=2048).contains(&le16(prefix, 2).unwrap_or(0)),
        get_slice(prefix, 4, 20)? == b"\x0fLegend of mir\r\n"
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::MirMap)
}

/// Emmanuel Giasson Music Module Compressor file (`.mmcmp` / `.zig`, 1997).
pub(crate) fn verify_mmcmp(prefix: &[u8], size: u64) -> Option<Label> {
    const MMCMP_MAGIC: &[u8; 8] = b"ziRCONia"; // @0..8: 8-byte MMCMP signature
    const HDR_SIZE_OFFSET: usize = 8; // @8..10: u16 LE header size (14)
    const NBLOCKS_OFFSET: usize = 12; // @12..14: u16 LE block count (>= 1)
    const FILESIZE_OFFSET: usize = 14; // @14..18: u32 LE uncompressed module size
    const BLKTABLE_OFFSET: usize = 18; // @18..22: u32 LE block offset table pointer

    require!(
        size >= MIN_MMCMP_SIZE,
        prefix.len() >= 24,
        starts(prefix, MMCMP_MAGIC),
        le16(prefix, HDR_SIZE_OFFSET) == Some(14),
        le16(prefix, NBLOCKS_OFFSET)? >= 1,
        le32(prefix, FILESIZE_OFFSET)? > 24,
        (24..size).contains(&(le32(prefix, BLKTABLE_OFFSET)? as u64))
    );
    Some(Label::Mmcmp)
}

/// Mozilla LZ4: Firefox session, bookmark backup and search-engine JSON compressed as one raw LZ4 block.
pub(crate) fn verify_mozlz4(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = b"mozLz40\0"; // @0, 8 bytes
    const DECOMPRESSED_SIZE: usize = 8; // @8: u32 LE — size of the original JSON
    require!(size >= MIN_MOZLZ4_SIZE, starts(prefix, MAGIC));
    require!(le32(prefix, DECOMPRESSED_SIZE)? > 0);
    Some(Label::Mozlz4)
}

/// Blizzard MoPaQ Archive (.mpq).
pub(crate) fn verify_mpq(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_MPQ_SIZE && prefix.len() >= 32);

    let actual_prefix = if starts(prefix, b"MPQ\x1B") {
        // MPQ User Data Header (16 bytes):
        // 0..4: "MPQ\x1B"
        // 4..8: user_data_size (u32 LE)
        // 8..12: mpq_header_offset (u32 LE)
        require!(prefix.len() >= 16);
        let header_offset = le32(prefix, 8)? as usize;
        require!(header_offset >= 16 && (header_offset as u64) + 32 <= size);
        if prefix.len() >= header_offset + 32 {
            get_slice(prefix, header_offset, prefix.len())?
        } else {
            return Some(Label::Mpq);
        }
    } else {
        prefix
    };

    require!(starts(actual_prefix, b"MPQ\x1A") && actual_prefix.len() >= 32);
    let header_size = u32::from_le_bytes(actual_prefix[4..8].try_into().ok()?);
    let archive_size = u32::from_le_bytes(actual_prefix[8..12].try_into().ok()?);
    let format_version = u16::from_le_bytes(actual_prefix[12..14].try_into().ok()?);
    let block_size_shift = u16::from_le_bytes(actual_prefix[14..16].try_into().ok()?);
    let hash_table_entries = u32::from_le_bytes(actual_prefix[24..28].try_into().ok()?);

    // Header size must be >= 32 (0x20)
    require!(header_size >= 32);
    // Format version: 0 = v1, 1 = v2, 2 = v3, 3 = v4
    require!(format_version <= 4);
    // Block size shift: 512 << shift, typically 3 (4096B sectors), maximum 16
    require!(block_size_shift <= 16);
    // Hash table entries must be non-zero power of 2
    require!(hash_table_entries > 0 && hash_table_entries.is_power_of_two());

    // If archive_size is non-zero, it shouldn't wildly exceed actual file size
    if archive_size > 0 && (archive_size as u64) > size + 1024 {
        return None;
    }

    Some(Label::Mpq)
}

/// Microsoft Windows Update Delta Compression Patch (PA30 / PA19).
pub(crate) fn verify_msdelta_patch(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_MSDELTA_PATCH_SIZE, prefix.len() >= 12);
    if starts(prefix, b"DCM\x01PA30")
        || starts(prefix, b"DCM\x01PA31")
        || starts(prefix, b"DCM\x01PA19")
    {
        return Some(Label::Msdelta);
    }
    if prefix.len() >= 16
        && starts(prefix, b"DCD\x01")
        && matches!(get_slice(prefix, 12, 16)?, b"PA30" | b"PA31" | b"PA19")
    {
        return Some(Label::Msdelta);
    }
    require!(
        (starts(prefix, b"PA30") || starts(prefix, b"PA31") || starts(prefix, b"PA19"))
            && has_non_ascii_text_byte(prefix.get(4..12)?)
    );
    Some(Label::Msdelta)
}

/// mstor Indexed Mailbox Archive (.ibx).
pub(crate) fn verify_mstor_ibx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MSTOR_IBX_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"mstor\0\0\0\x61\0\x60\0"),
        get_slice(prefix, 16, 24)? == b"info\0\xff\xff\xff"
    );
    Some(Label::MstorIbx)
}

/// NBA 2K Career Game Save / Mod Archive (.cmg).
pub(crate) fn verify_nba2k_cmg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NBA2K_CMG_SIZE,
        prefix.len() >= 24,
        be32(prefix, 4) == Some(0x85D2_4109),
        be32(prefix, 8) == Some(0x11),
        (be32(prefix, 12).map(|p| p as u64 + 24 == size).unwrap_or(false)
            || size >= MIN_NBA2K_CMG_SIZE),
        be32(prefix, 16) == Some(0)
    );
    require!(has_non_ascii_text_byte(prefix.get(..24)?));
    Some(Label::Nba2kCmg)
}

/// NCBI Sequence Read Archive (`NCBI.sra` + byte-order marker `0x05031988`).
pub(crate) fn verify_ncbi_sra(prefix: &[u8], size: u64) -> Option<Label> {
    const BYTE_ORDER: u32 = 0x0503_1988; // @8: u32 eByteOrderTag
    require!(size >= MIN_NCBI_SRA_SIZE, starts(prefix, b"NCBI.sra"));
    let version = if le32(prefix, 8)? == BYTE_ORDER {
        le32(prefix, 12)? // @12: u32 version in the archive's byte order
    } else if be32(prefix, 8)? == BYTE_ORDER {
        be32(prefix, 12)?
    } else {
        return None;
    };
    require!((1..=16).contains(&version));
    Some(Label::Sra)
}

/// Nintendo 3DS NW4C Binary CTR Resource Package (.bcres / .bcmdl / .bctex).
pub(crate) fn verify_nintendo_bcres(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NINTENDO_BCRES_SIZE
            && prefix.len() >= 24
            && matches!(get_slice(prefix, 4, 6)?, &[0xFF, 0xFE] | &[0xFE, 0xFF])
            && le16(prefix, 6) == Some(0x14)
            && le32(prefix, 12)? as u64 == size
            && get_slice(prefix, 20, 24)? == b"DATA"
    );
    Some(Label::Bcres)
}

/// Nintendo Wii U / Switch NW4F Binary Cafe Resource Package (.bfres).
pub(crate) fn verify_nintendo_bfres(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_NINTENDO_BFRES_SIZE && prefix.len() >= 32);
    let is_switch = get_slice(prefix, 4, 8)? == b"    "
        && matches!(get_slice(prefix, 12, 14)?, &[0xFF, 0xFE] | &[0xFE, 0xFF])
        && le32(prefix, 28)? as u64 == size;
    let is_wiiu = matches!(get_slice(prefix, 8, 10)?, &[0xFE, 0xFF] | &[0xFF, 0xFE])
        && (be32(prefix, 12)? as u64 == size || le32(prefix, 12)? as u64 == size);
    require!(is_switch || is_wiiu);
    Some(Label::Bfres)
}

/// Nintendo DS Nitro Archive (`.narc`, `"NARC"` + `FE FF` BOM + version `0x0000`/`0x0100` + `"BTAF"`).
pub(crate) fn verify_nintendo_narc_archive(prefix: &[u8], size: u64) -> Option<Label> {
    const NARC_MAGIC: &[u8; 4] = b"NARC"; // @0..4: Nitro Archive chunk magic
    const BTAF_MAGIC: &[u8; 4] = b"BTAF"; // @16..20: File Allocation Table block magic

    require!(
        size >= MIN_NINTENDO_NARC_SIZE,
        prefix.len() >= 20,
        starts(prefix, NARC_MAGIC),
        le16(prefix, 4) == Some(0xFFFE),
        matches!(le16(prefix, 6), Some(0x0000 | 0x0100)),
        le16(prefix, 12) == Some(16),
        eq_at(prefix, 16, BTAF_MAGIC)
    );
    Some(Label::NintendoNarc)
}

/// Nintendo GameCube / Wii JKR Archive (.rarc / .arc).
pub(crate) fn verify_nintendo_rarc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_NINTENDO_RARC_SIZE && prefix.len() >= 32);
    let valid = if starts(prefix, b"RARC") {
        be32(prefix, 4)? as u64 == size && be32(prefix, 8) == Some(0x20)
    } else {
        le32(prefix, 4)? as u64 == size && le32(prefix, 8) == Some(0x20)
    };
    require!(valid);
    Some(Label::Rarc)
}

/// Nintendo Wii U / 3DS / Switch SARC Resource Archive (.sarc).
pub(crate) fn verify_nintendo_sarc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NINTENDO_SARC_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"SARC"),
        get_slice(prefix, 20, 24)? == b"SFAT"
    );
    let bom = be16(prefix, 6)?;
    require!(bom == 0xFEFF || bom == 0xFFFE);
    let is_be = bom == 0xFEFF;
    let hdr_sz = if is_be { be16(prefix, 4)? } else { le16(prefix, 4)? };
    let file_sz = if is_be { be32(prefix, 8)? } else { le32(prefix, 8)? } as u64;
    let data_off = if is_be { be32(prefix, 12)? } else { le32(prefix, 12)? } as u64;
    let sfat_hdr_sz = if is_be { be16(prefix, 24)? } else { le16(prefix, 24)? };
    require!(
        hdr_sz == 0x14,
        sfat_hdr_sz == 0x0C,
        file_sz >= MIN_NINTENDO_SARC_SIZE,
        (0x20..=size).contains(&data_off)
    );
    Some(Label::Sarc)
}

/// Nintendo Wii / Wii U U8 Archive (`55 AA 38 2D` = `"U\xAA8-"`).
pub(crate) fn verify_nintendo_u8_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NINTENDO_U8_SIZE,
        prefix.len() >= 16,
        starts(prefix, &[0x55, 0xAA, 0x38, 0x2D]),
        be32(prefix, 4) == Some(0x20),
        be32(prefix, 8)? >= 12,
        be32(prefix, 12)? >= 0x20
    );
    Some(Label::NintendoU8)
}

/// Nintendo `Yay0` SZP compressed stream.
pub(crate) fn verify_nintendo_yay0(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 20, prefix.len() >= 20, starts(prefix, b"Yay0"));
    let uncomp_size = be32(prefix, 4)?;
    let link_off = be32(prefix, 8)? as u64;
    let chunk_off = be32(prefix, 12)? as u64;
    require!(
        (1..=0x1000_0000).contains(&uncomp_size),
        link_off >= 16,
        (link_off & 1) == 0,
        chunk_off >= link_off,
        chunk_off < size,
    );
    Some(Label::Yay0)
}

/// Nintendo `Yaz0` run-length compressed stream.
pub(crate) fn verify_nintendo_yaz0(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 18, prefix.len() >= 18, starts(prefix, b"Yaz0"));
    let uncomp_size = be32(prefix, 4)?;
    let align_field = be32(prefix, 8)?;
    let reserved = be32(prefix, 12)?;
    require!(
        (1..=0x4000_0000).contains(&uncomp_size),
        reserved == 0,
        align_field == 0 || (align_field.is_power_of_two() && align_field <= 0x10000),
    );
    Some(Label::Yaz0)
}

/// Nitroplus Visual Novel Engine NPA Archive (.npa, 2004) Nitroplus.
pub(crate) fn verify_nitroplus_npa(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NITROPLUS_NPA_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"NPA\x01\0\0\0"),
        matches!(get_slice(prefix, 15, 17)?, [1, 1] | [0, 1] | [1, 0]),
        (1..=100_000).contains(&le32(prefix, 17).unwrap_or(0))
    );
    Some(Label::NitroplusNpa)
}

/// Netopsystems FEAD Optimizer (`NOS_PO` / `PK??NOS_PO` / XOR-`0x9C` `"IEu\x81=%"`) Archive payload.
pub(crate) fn verify_nos_installer_archive(prefix: &[u8], size: u64) -> Option<Label> {
    const FEAD_XOR9C_MAGIC: &[u8; 6] = hex2bin!("D5 D9 E9 1D A1 B9"); // "IEu\x81=%" XOR 0x9C
    require!(size >= MIN_NOS_INSTALLER_ARCHIVE_SIZE, prefix.len() >= 26);
    let stream_off = if starts(prefix, b"NOS_PO\x01") {
        if eq_at(prefix, 7, FEAD_XOR9C_MAGIC) {
            7
        } else if u8_at(prefix, 7)?.is_ascii_digit() && eq_at(prefix, 8, FEAD_XOR9C_MAGIC) {
            8
        } else {
            return None;
        }
    } else if starts(prefix, b"PK") && eq_at(prefix, 4, b"NOS_PO\x01") {
        if eq_at(prefix, 11, FEAD_XOR9C_MAGIC) {
            11
        } else if u8_at(prefix, 11)?.is_ascii_digit() && eq_at(prefix, 12, FEAD_XOR9C_MAGIC) {
            12
        } else {
            return None;
        }
    } else if starts(prefix, FEAD_XOR9C_MAGIC) {
        0
    } else {
        return None;
    };
    // At stream_off + 8..10 is the XOR-0x9C u16 LE header size (0x0038 ^ 0x9C9C = A4 9C, or A4 9A):
    require!(
        u8_at(prefix, stream_off + 8)? == 0xA4,
        (u8_at(prefix, stream_off + 9)? ^ 0x9C) <= 0x0F
    );
    Some(Label::NosInstallerData)
}

/// NSIS `firstheader` starting directly with `0xDEADBEEF` / `0xDEADBEED` at offset 0:
/// - **NSIS 1.x (1999–2002)**: no leading `flags` dword;
///   `@0 u32 0xDEADBEEF`, `@4 "nsisinstall\0"` (12B), `@16 u32 0xDEADF00D` (1.4x–1.98) or
///   `0x0BADF00D` (1.1x–1.3x), `@20 u32 length_of_header`, `@24 u32 length_of_all_following_data`.
/// - **Signature-carved NSIS 2.x / 3.x**: carved starting at `@0 0xDEADBEEF` / `0xDEADBEED`
///   followed by `@4 "NullsoftInst"` / `"NullSoftInst"` (skipping the 4-byte `flags` field),
///   `@16 u32 length_of_header`, `@20 u32 length_of_all_following_data`.
pub(crate) fn verify_nsis_at_deadbeef(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_NSIS_V1_SIZE: u64 = 28;
    const MIN_CARVED_V2_SIZE: u64 = 24;
    require!(size >= MIN_CARVED_V2_SIZE, prefix.len() >= 24);
    let sig = le32(prefix, 0)?;
    require!(sig == 0xDEAD_BEEF || sig == 0xDEAD_BEED);
    if eq_at(prefix, 4, b"nsisinstall\0") {
        require!(size >= MIN_NSIS_V1_SIZE, prefix.len() >= 28);
        let magic2 = le32(prefix, 16)?;
        require!(magic2 == 0xDEAD_F00D || magic2 == 0x0BAD_F00D);
        let header_len = le32(prefix, 20)?;
        let total = le32(prefix, 24)?;
        require!(header_len > 0 && total >= 64);
        return Some(Label::Nsis);
    }
    if eq_at(prefix, 4, b"NullsoftInst") || eq_at(prefix, 4, b"NullSoftInst") {
        let header_len = le32(prefix, 16)?;
        let total = le32(prefix, 20)?;
        require!(header_len > 0 && total >= 64);
        return Some(Label::Nsis);
    }
    None
}

/// NSIS 2.x / 3.x / Park / NSISBI `firstheader`,
/// the start of NSIS installer data carved from an overlay:
/// `@0 u32 flags` (`FH_FLAGS_*` bits 0..3 + NSISBI bits 4..7; `exehead` tests `flags & 0xFFFFFE00`),
/// `@4 u32 siginfo = 0xDEADBEEF` (or `0xDEADBEED`), `@8 "NullsoftInst"` (or `"NullSoftInst"` / `"nsisinstall\0"`),
/// `@20 u32 length_of_header`, `@24 u32 length_of_all_following_data` (or 36-byte wide header when `flags & 0x10 != 0`).
pub(crate) fn verify_nsis_firstheader(prefix: &[u8], size: u64) -> Option<Label> {
    const FLAGS: usize = 0; // @0: u32 flags
    const SIGINFO: usize = 4; // @4: FH_SIG (0xDEADBEEF / 0xDEADBEED)
    const NSINST_ID: usize = 8; // @8: 12-byte signature ("NullsoftInst" / "NullSoftInst" / "nsisinstall\0")
    const LENGTH_OF_HEADER: usize = 20; // @20: u32 inflated header length
    const LENGTH_OF_ALL: usize = 24; // @24: u32 total data length
    const FH_FLAGS_ACCEPTED: u32 = 0x1FF; // exehead tests `flags & 0xFFFFFE00` (includes NSISBI flags 0x10..0x80)
    const FH_FLAGS_BI_LONG_OFFSET: u32 = 0x10; // NSISBI 36-byte wide firstheader
    const FIRSTHEADER_SIZE: u64 = 28;
    const MIN_NSIS_DATA_SIZE: u64 = 64;
    require!(size >= FIRSTHEADER_SIZE, prefix.len() >= 28);
    let flags = le32(prefix, FLAGS)?;
    require!(flags & !FH_FLAGS_ACCEPTED == 0);
    let sig = le32(prefix, SIGINFO)?;
    require!(sig == 0xDEAD_BEEF || sig == 0xDEAD_BEED);
    require!(
        eq_at(prefix, NSINST_ID, b"NullsoftInst")
            || eq_at(prefix, NSINST_ID, b"NullSoftInst")
            || eq_at(prefix, NSINST_ID, b"nsisinstall\0")
    );
    let (header_len, total) = if flags & FH_FLAGS_BI_LONG_OFFSET != 0 && prefix.len() >= 36 {
        let h = u64::from(le32(prefix, LENGTH_OF_HEADER)?) | (u64::from(le32(prefix, 28)?) << 32);
        let t = u64::from(le32(prefix, LENGTH_OF_ALL)?) | (u64::from(le32(prefix, 32)?) << 32);
        (h, t)
    } else {
        (u64::from(le32(prefix, LENGTH_OF_HEADER)?), u64::from(le32(prefix, LENGTH_OF_ALL)?))
    };
    // `length_of_header` is the *inflated* header size, so it may exceed the total of tiny installers.
    require!(header_len > 0, total >= MIN_NSIS_DATA_SIZE.min(size));
    Some(Label::Nsis)
}

/// Decompressed NSIS 2.x / 3.x Binary Script Header (`[NSIS_Script_Header.bin]`):
/// - `@0..4`: `u32 flags` (`CH_FLAGS_*`, `flags & !0x3FF == 0`)
/// - `@4..68`: 8 `block_header { u32 offset, u32 num }` records (`Pages` `@4`, `Sections` `@12`,
///   `Entries` `@20`, `Strings` `@28`, `LangTables` `@36`, `CtlColors` `@44`, `BgFont` `@52`, `Data` `@60`)
/// - Exact algebraic table closure: `entries.offset + entries.num * 28 == strings.offset`,
///   `strings.num == 0` (`@32..36 == 0`), and `langtables.offset > strings.offset`.
pub(crate) fn verify_nsis_script_header(prefix: &[u8], size: u64) -> Option<Label> {
    const CH_FLAGS: usize = 0; // @0: u32 common header flags
    const PAGES_OFF: usize = 4; // @4: u32 pages.offset (bh_len: 276 or 300 in 32-bit, 508 in 64-bit)
    const PAGES_NUM: usize = 8; // @8: u32 pages.num
    const SECTIONS_OFF: usize = 12; // @12: u32 sections.offset
    const SECTIONS_NUM: usize = 16; // @16: u32 sections.num
    const ENTRIES_OFF: usize = 20; // @20: u32 entries.offset
    const ENTRIES_NUM: usize = 24; // @24: u32 entries.num (28-byte EW_* instructions)
    const STRINGS_OFF: usize = 28; // @28: u32 strings.offset
    const STRINGS_NUM: usize = 32; // @32: u32 strings.num (always 0)
    const LANGTABLES_OFF: usize = 36; // @36: u32 langtables.offset
    const CH_FLAGS_MASK: u32 = 0x03FF;
    const EW_ENTRY_SIZE: u32 = 28; // 7 * sizeof(u32)
    const MIN_NSIS_SCRIPT_HDR_SIZE: u64 = 300;

    require!(size >= MIN_NSIS_SCRIPT_HDR_SIZE, prefix.len() >= 68);
    let flags = le32(prefix, CH_FLAGS)?;
    let pages_off = le32(prefix, PAGES_OFF)?;
    let pages_num = le32(prefix, PAGES_NUM)?;
    let sections_off = le32(prefix, SECTIONS_OFF)?;
    let sections_num = le32(prefix, SECTIONS_NUM)?;
    let entries_off = le32(prefix, ENTRIES_OFF)?;
    let entries_num = le32(prefix, ENTRIES_NUM)?;
    let strings_off = le32(prefix, STRINGS_OFF)?;
    let strings_num = le32(prefix, STRINGS_NUM)?;
    let langtables_off = le32(prefix, LANGTABLES_OFF)?;

    require!(
        flags & !CH_FLAGS_MASK == 0,
        (276..=600).contains(&pages_off),
        pages_num <= 256,
        sections_num <= 1024,
        sections_off >= pages_off,
        (1..=1_000_000).contains(&entries_num),
        strings_num == 0,
        entries_off.checked_add(entries_num.checked_mul(EW_ENTRY_SIZE)?) == Some(strings_off),
        langtables_off > strings_off,
        u64::from(langtables_off) <= size
    );
    if sections_num > 0 {
        let sec_bytes = entries_off.checked_sub(sections_off)?;
        require!(
            sec_bytes % sections_num == 0
                && matches!(sec_bytes / sections_num, 1048 | 2072 | 4120 | 16408)
        );
    } else {
        require!(entries_off == sections_off);
    }
    Some(Label::Nsis)
}

/// Apple II NuFX / ShrinkIt archive (`.shk` / `.sdk`).
pub(crate) fn verify_nufx(prefix: &[u8], size: u64) -> Option<Label> {
    const NUFILE_HI_MAGIC: &[u8; 6] = &[b'N', b'u' | 0x80, b'F', b'i' | 0x80, b'l', b'e' | 0x80];
    const NUFX_HI_MAGIC: &[u8; 4] = &[b'N', b'u' | 0x80, b'F', b'X' | 0x80];
    require!(size >= 16, prefix.len() >= 16);
    if starts(prefix, NUFILE_HI_MAGIC)
        || starts(prefix, NUFX_HI_MAGIC)
        || starts(prefix, b"\0NuFX")
        || starts(prefix, b"\0NuFile")
        || ((starts(prefix, b"NuFile") || starts(prefix, b"NuFX")) && u8_at(prefix, 15)? == 0)
    {
        return Some(Label::Shk);
    }
    None
}

/// Omega Logic CORK Compressed Installer File (`.dl$` / `.ex$`, 1993 Omega Logic).
pub(crate) fn verify_omega_logic_cork(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 6] = b"CORK\x01\x00"; // @0..6: "CORK" + u16 LE version 1
    const NAME_RANGE: core::ops::Range<usize> = 6..19; // @6..19: NUL-terminated 8.3 filename
    const ORIG_SIZE_OFFSET: usize = 25; // @25..29: u32 LE uncompressed size
    const COMP_SIZE_OFFSET: usize = 29; // @29..33: u32 LE compressed payload size
    require!(size >= MIN_OMEGA_LOGIC_CORK_SIZE, prefix.len() >= 33, starts(prefix, MAGIC));
    let nul_pos =
        get_slice(prefix, NAME_RANGE.start, NAME_RANGE.end)?.iter().position(|&b| b == 0)?;
    let orig_size = le32(prefix, ORIG_SIZE_OFFSET)?;
    let comp_size = le32(prefix, COMP_SIZE_OFFSET)?;
    require!(
        (1..=12).contains(&nul_pos),
        get_slice(prefix, NAME_RANGE.start, NAME_RANGE.start + nul_pos)?
            .iter()
            .all(u8::is_ascii_graphic),
        orig_size > 0,
        comp_size > 0,
        comp_size <= orig_size
    );
    Some(Label::OmegaLogicCork)
}

/// IBM OS/2 PACK2 FTCOMP Compressed Installation Archive (.ba_ / .ex_ / .dl_).
pub(crate) fn verify_os2_ftcomp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_OS2_FTCOMP_SIZE
            && prefix.len() >= 32
            && starts(prefix, hex2bin!("A5 96 FD FF"))
            && eq_at(prefix, 24, b"FTCOMP")
    );
    Some(Label::Os2Ftcomp)
}

/// IBM OS/2 PACK Compressed Installation File (.ex$ / .dl$ / .sy$).
pub(crate) fn verify_os2_pack(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_OS2_PACK_SIZE
            && prefix.len() >= 16
            && u8_at(prefix, 0)? == 0xA5
            && u8_at(prefix, 1)? == 0x96
            && matches!(
                (u8_at(prefix, 2)?, u8_at(prefix, 3)?),
                (0x00, 0x14) | (0x14, 0x0A) | (0xFF, 0xFF) | (0xFE, 0xFF)
            )
    );
    Some(Label::Os2Pack)
}

/// Atari ST Pack-Ice Compressed File (ICE! / Ice!).
pub(crate) fn verify_pack_ice(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PACK_ICE_SIZE && prefix.len() >= 12);
    let packed_size = be32(prefix, 4)? as u64;
    let unpacked_size = be32(prefix, 8)? as u64;
    require!(
        packed_size >= MIN_PACK_ICE_SIZE
            && packed_size <= size
            && (4..=0x0400_0000).contains(&unpacked_size)
    );
    Some(Label::PackIce)
}

/// Pangolin Laser Systems Request Package (.prr) Pangolin Laser Systems.
pub(crate) fn verify_pangolin_prr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PANGOLIN_PRR_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"PangolinReqPack\0(\0"),
        get_slice(prefix, 22, 26)? == b"PFCh"
    );
    Some(Label::PangolinPrr)
}

/// Matt Mahoney PAQ Archive (`"paq6"`, `"paq7"`, `"paq8"`, `"paq9a"`, `"pQ9\x01"`).
pub(crate) fn verify_paq_archive(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_PAQ_ARCHIVE_SIZE
        && prefix.len() >= 12
        && starts(prefix, b"pQ9\x01")
        && matches!(u8_at(prefix, 4)?, b'1'..=b'9')
    {
        return Some(Label::PaqArchive);
    }
    require!(
        size >= MIN_PAQ_ARCHIVE_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"paq8")
            || starts(prefix, b"paq7")
            || starts(prefix, b"paq6")
            || starts(prefix, b"paq9a")
    );
    let dash_ok = get_slice(prefix, 4, 12)?.contains(&b'-')
        || u8_at(prefix, 4).unwrap_or(0) == 0
        || u8_at(prefix, 5).unwrap_or(0) == 0;
    let scan = get_slice(prefix, 4, prefix.len().min(256))?;
    require!(
        dash_ok,
        scan.iter().any(|&b| (b < 0x20 && !matches!(b, b'\t' | b'\n' | b'\r')) || b >= 0x7F)
    );
    Some(Label::PaqArchive)
}

/// Parchive: PAR1 `"PAR\0\0\0\0\0"` + `u64 LE version` (`0x0001xxxx`), or PAR2/PAR3 `"PAR2\0PKT"` / `"PAR3\0PKT"`
/// + `u64 LE packet_length` (a multiple of 4, at least the 64-byte packet header).
pub(crate) fn verify_parchive(prefix: &[u8], size: u64) -> Option<Label> {
    const PAR1: &[u8] = b"PAR\0\0\0\0\0"; // @0, 8 bytes
    const PAR2: &[u8] = b"PAR2\0PKT"; // @0, 8 bytes
    const PAR3: &[u8] = b"PAR3\0PKT"; // @0, 8 bytes
    const FIELD: usize = 8; // @8: u64 LE — PAR1 version / PAR2+ packet length
    if starts(prefix, PAR1) {
        require!(size >= MIN_PAR1_SIZE, le32(prefix, FIELD)? >> 16 == 1);
    } else {
        require!(size >= MIN_PAR2_SIZE, starts(prefix, PAR2) || starts(prefix, PAR3));
        let packet_len = le64(prefix, FIELD)?;
        require!(packet_len >= 64, packet_len % 4 == 0);
    }
    Some(Label::Parchive)
}

/// Parsons Technology Resource Archive (`.wrk`, `"PARSONS TECHNOLOGY RESOURCE"`).
pub(crate) fn verify_parsons_wrk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PARSONS_WRK_SIZE, starts(prefix, b"PARSONS TECHNOLOGY RESOURCE"));
    Some(Label::ParsonsWrk)
}

/// Pazligsaw Jigsaw Puzzle Package (`BOX1`, `.pzl`).
pub(crate) fn verify_pazligsaw_pzl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PAZLIGSAW_PZL_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"BOX1"),
        (1..=10_000).contains(&le32(prefix, 16).unwrap_or(0)),
        (1..=16).contains(&le32(prefix, 20).unwrap_or(0)),
        (1..=16).contains(&le32(prefix, 24).unwrap_or(0)),
        le32(prefix, 28) == Some(4)
    );
    Some(Label::PazligsawPzl)
}

/// NEC PC-8801 T88 Cassette Tape Image (.t88, 1999).
pub(crate) fn verify_pc88_t88(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PC88_T88_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"PC-8801 Tape Image(T88)\0\x01\0")
    );
    Some(Label::Pc88T88)
}

/// Genus PCX Programmer's Toolkit library: `"pcxLib\0"` + header word, 50-byte label @10..60 (often `"Copyright (c) Genus Microprogramming"`), and `0x015E` (`5E 01` LE) entry sync word at `@60..62`.
pub(crate) fn verify_pcx_library(prefix: &[u8], size: u64) -> Option<Label> {
    const BANNER: usize = 10; // @10: 50-byte archive label
    const ENTRY_SYNC_OFFSET: usize = 60; // @60..62: u16 LE 0x015E first directory entry sync word
    require!(size >= MIN_PCX_LIBRARY_SIZE, prefix.len() >= 64, starts(prefix, b"pcxLib\0"));
    require!(
        eq_at(prefix, ENTRY_SYNC_OFFSET, hex2bin!("5E 01"))
            || eq_at(prefix, BANNER, b"Copyright (c) Genus Microprogramming")
    );
    Some(Label::PcxLibrary)
}

/// DEC PDP-10 TOPS-10 FAILSAFE Tape Stream (1970 Digital Equipment).
///
/// Header record layout (`5 * 5 = 25` core-dump 36-bit bytes; supports both clean ANSI core-dump
/// `(b4 & 0xF0) == 0` and real DEC TM10 hardware 6-bit low frame `(b4 >> 4) == (b3 & 0x03)`):
/// - `w[0]` (`@0..5`): right half `== 4` (4 following header words), left half = FAILSAFE version (`!= 0`)
/// - `w[1..2]` (`@5..13`): SIXBIT `"*FAILSAFE"` (`2A 68 69 B3 [03|33] 86 69 4x`).
pub(crate) fn verify_pdp10_failsafe(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_WORDS: usize = 5;
    const BYTES_PER_WORD: usize = 5;
    const HDR_BYTES: usize = HDR_WORDS * BYTES_PER_WORD; // 25 bytes
    const W1_OFF: usize = 5; // @5: SIXBIT "*FAILS"
    const W2_OFF: usize = 10; // @10: SIXBIT "AFE..."

    require!(
        size >= MIN_PDP10_FAILSAFE_SIZE,
        size.is_multiple_of(BYTES_PER_WORD as u64),
        prefix.len() >= HDR_BYTES,
        eq_at(prefix, W1_OFF, PDP10_FAILSAFE_W1_HI),
        matches!(u8_at(prefix, W1_OFF + 4)?, 0x03 | 0x33),
        eq_at(prefix, W2_OFF, PDP10_FAILSAFE_W2_HI),
        (u8_at(prefix, W2_OFF + 2)? & 0xF0) == 0x40,
        has_non_ascii_text_byte(prefix.get(..HDR_BYTES)?)
    );
    for idx in 0..HDR_WORDS {
        let base = idx * BYTES_PER_WORD;
        let b3 = u8_at(prefix, base + 3)?;
        let b4 = u8_at(prefix, base + 4)?;
        if (b4 & 0xC0) != 0 || ((b4 & 0x30) != 0 && ((b4 >> 4) & 0x03) != (b3 & 0x03)) {
            return None;
        }
    }
    let w0 = ((u8_at(prefix, 0)? as u64) << 28)
        | ((u8_at(prefix, 1)? as u64) << 20)
        | ((u8_at(prefix, 2)? as u64) << 12)
        | ((u8_at(prefix, 3)? as u64) << 4)
        | ((u8_at(prefix, 4)? & 0x0F) as u64);
    require!((w0 & 0x3_FFFF) == 4, (w0 >> 18) != 0);

    Some(Label::Pdp10Failsafe)
}

/// PeaZip PEA archive (.pea).
pub(crate) fn verify_pea_archive(prefix: &[u8], size: u64) -> Option<Label> {
    (size >= MIN_PEA_ARCHIVE_SIZE
        && prefix.len() >= 4
        && (starts(prefix, b"PEA\x01")
            || (u8_at(prefix, 0)? == 0xEA
                && u8_at(prefix, 1)? == 1
                && matches!(u8_at(prefix, 2)?, 0 | 1))))
    .then_some(Label::Pea)
}

/// PFPART3 Partial Download Container (.pfpart).
pub(crate) fn verify_pfpart3(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PFPART3_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"PFPART3\n"),
        get_slice(prefix, 16, 24)? == b"\0\0\x80\0\0\0\0\0"
    );
    Some(Label::Pfpart3)
}

/// Sony PlayStation 3 / PSP Package (.pkg).
pub(crate) fn verify_playstation_3_pkg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PLAYSTATION_3_PKG_SIZE && prefix.len() >= 32);
    require!(starts(prefix, b"\x7fPKG"));
    let pkg_rev = be16(prefix, 4)?;
    let pkg_type = be16(prefix, 6)?;
    // @4..6 is pkg_revision (0x8000 retail or 0x0000 debug) and @6..8 is pkg_type (1=PS3, 2=PSP/Vita);
    // also accept legacy layout where @4..6 is 1 or 2.
    require!(
        (matches!(pkg_rev, 0x8000 | 0x0000) && matches!(pkg_type, 1 | 2))
            || matches!(pkg_rev, 1 | 2)
    );
    let file_offset = be32(prefix, 8)? as u64;
    let file_count = be32(prefix, 12)?;
    require!(file_offset >= 0x80 && file_count >= 1);
    Some(Label::Ps3Pkg)
}

/// Sony PlayStation 4 / PS Vita Content Package (.pkg).
pub(crate) fn verify_playstation_4_pkg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PLAYSTATION_4_PKG_SIZE && prefix.len() >= 32);
    require!(starts(prefix, b"\x7fCNT"));
    let cnt_type = be32(prefix, 4)?;
    require!((1..=0x20).contains(&cnt_type));
    let file_count = be32(prefix, 12)?;
    let table_offset = be32(prefix, 20)? as u64;
    require!(table_offset >= 0x80 && file_count >= 1);
    Some(Label::Ps4Pkg)
}

/// Stephan Fuhrmann PowerPlayer Music Cruncher file (`.pmc`, 1992).
pub(crate) fn verify_pmc_cruncher(prefix: &[u8], size: u64) -> Option<Label> {
    const ORIG_LEN_OFFSET: usize = 4; // @4..8: u32 BE uncompressed module size
    const CMPR_LEN_OFFSET: usize = 8; // @8..12: u32 BE compressed data size

    require!(
        size >= MIN_PMC_CRUNCHER_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"SFHD") || starts(prefix, b"SFCD")
    );
    let orig_len = be32(prefix, ORIG_LEN_OFFSET)? as u64;
    let cmpr_len = be32(prefix, CMPR_LEN_OFFSET)? as u64;
    require!(orig_len > 0, cmpr_len > 0, (cmpr_len + 12..=cmpr_len + 16).contains(&size));
    Some(Label::PmcCruncher)
}

/// Counter-Strike POD-Bot Waypoint File (.pwf, 2001) Count Floyd.
pub(crate) fn verify_podbot_pwf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PODBOT_PWF_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"PODWAY!\0"),
        (5..=10).contains(&le32(prefix, 8).unwrap_or(0)),
        (1..=4096).contains(&le32(prefix, 12).unwrap_or(0))
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::PodbotPwf)
}

/// Game Freak Nintendo 3DS GARC Archive (`.garc`, Pokémon X/Y, OR/AS, Sun/Moon).
///
/// Header layout:
/// - `@0..4`: `"CRAG"` (little-endian `"GARC"`)
/// - `@4..8`: `u32 LE` `header_len` (`0x1C` = 28 for v4, `0x24` = 36 for v6)
/// - `@8..10`: `FF FE` little-endian BOM
/// - `@10..12`: `u16 LE` `version` (`0x0400` or `0x0600`)
/// - `@12..16`: `u32 LE` `chunk_count` (`4`)
/// - `@header_len..header_len+4`: `"OTAF"` (FATO section magic)
pub(crate) fn verify_pokemon_garc(prefix: &[u8], size: u64) -> Option<Label> {
    const BOM_LE: &[u8; 2] = hex2bin!("FF FE"); // @8..10: little-endian BOM
    const FATO_MAGIC: &[u8; 4] = b"OTAF"; // @header_len: FATO chunk tag reversed
    const HDR_LEN_OFF: usize = 4; // @4..8: u32 LE header length (28 or 36)
    const BOM_OFF: usize = 8; // @8..10: BOM
    const VERSION_OFF: usize = 10; // @10..12: u16 LE version (0x0400 or 0x0600)
    const CHUNK_COUNT_OFF: usize = 12; // @12..16: u32 LE section count (4)

    require!(
        size >= MIN_POKEMON_GARC_SIZE,
        prefix.len() >= 40,
        starts(prefix, POKEMON_GARC_MAGIC),
        eq_at(prefix, BOM_OFF, BOM_LE),
        matches!(le16(prefix, VERSION_OFF)?, 0x0400 | 0x0600),
        le32(prefix, CHUNK_COUNT_OFF)? == 4
    );
    let hdr_len = le32(prefix, HDR_LEN_OFF)? as usize;
    require!(matches!(hdr_len, 28 | 36), eq_at(prefix, hdr_len, FATO_MAGIC));
    Some(Label::PokemonGarc)
}

/// Game Freak Nintendo Switch GFPAK Archive (`.gfpak`, Pokémon Let's Go / Sword & Shield / PLA).
///
/// Header layout:
/// - `@0..8`: `"GFLXPACK"` magic
/// - `@8..12`: `u32 LE` `version` (`0x1000` or `0x2000`)
/// - `@12..16`: `u32 LE` `is_relocated` (`0` or `1`)
/// - `@16..20`: `u32 LE` `file_count` (`1..=1_000_000`)
pub(crate) fn verify_pokemon_gfpak(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 8] = b"GFLXPACK"; // @0..8: Game Freak GFLX pack signature
    const VERSION_OFF: usize = 8; // @8..12: u32 LE version (0x1000 or 0x2000)
    const RELOC_OFF: usize = 12; // @12..16: u32 LE relocation flag (0 or 1)
    const FILE_COUNT_OFF: usize = 16; // @16..20: u32 LE entry count

    require!(
        size >= MIN_POKEMON_GFPAK_SIZE,
        prefix.len() >= 24,
        starts(prefix, MAGIC),
        matches!(le32(prefix, VERSION_OFF)?, 0x1000 | 0x2000),
        le32(prefix, RELOC_OFF)? <= 1,
        (1..=1_000_000).contains(&le32(prefix, FILE_COUNT_OFF)?)
    );
    Some(Label::PokemonGfpak)
}

/// Sony PlayStation POPStarter Virtual CD-ROM Image (.vcd).
pub(crate) fn verify_popstarter_vcd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_POPSTARTER_VCD_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 4)? == hex2bin!("41 00 A0 00"),
        get_slice(prefix, 10, 14)? == hex2bin!("41 00 A1 00"),
        get_slice(prefix, 20, 24)? == hex2bin!("41 00 A2 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::PopstarterVcd)
}

/// Sybase / Appeon PowerBuilder Dynamic Library (.pbd / .pbl, 1991) Powersoft / Sybase.
pub(crate) fn verify_powerbuilder_pbd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_POWERBUILDER_PBD_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"HDR*"),
        get_slice(prefix, 4, 18)? == b"PowerBuilder\0\0"
            || get_slice(prefix, 4, 32)? == b"P\0o\0w\0e\0r\0B\0u\0i\0l\0d\0e\0r\0\0\0"
    );
    Some(Label::PowerbuilderPbd)
}

/// Amiga PowerPacker Compressed File (PP20).
pub(crate) fn verify_powerpacker(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_POWERPACKER_SIZE && prefix.len() >= 12);
    let valid = if starts(prefix, b"PP20") || starts(prefix, b"PP11") {
        get_slice(prefix, 4, 8)?.iter().all(|&b| (9..=15).contains(&b))
            && u8_at(prefix, 4)? <= u8_at(prefix, 5)?
            && u8_at(prefix, 5)? <= u8_at(prefix, 6)?
            && u8_at(prefix, 6)? <= u8_at(prefix, 7)?
    } else if starts(prefix, b"PX20") {
        get_slice(prefix, 6, 10)?.iter().all(|&b| (9..=15).contains(&b))
    } else if starts(prefix, b"PPbk") {
        let bank_num = be16(prefix, 4)?;
        let bank_len = be32(prefix, 8)?;
        bank_num <= 256 && bank_len > 0
    } else {
        false
    };
    require!(valid);
    Some(Label::Powerpacker)
}

/// PlayStation Patch File (`PPF10` / `PPF20` / `PPF30`).
pub(crate) fn verify_ppf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 56,
        prefix.len() >= 6,
        starts(prefix, b"PPF30") || starts(prefix, b"PPF20") || starts(prefix, b"PPF10"),
        u8_at(prefix, 5)? <= 2,
    );
    Some(Label::PpfPatch)
}

/// Pro-face PROX_USB_XS HMI Project Archive (.xha) Pro-face / Schneider Electric.
pub(crate) fn verify_prox_usb_xha(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PROX_USB_XHA_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"PROX_USB_XS\0\0\0\0\0"),
        get_slice(prefix, 32, 36)? == b"XPR\0"
    );
    Some(Label::ProxUsbXha)
}

/// Proxmox Backup Server `.pxar` (`EF AC 88 E5 74 64 95 D5` / `0D A4 16 DF 75 6C 0F 73`),
/// `.fidx` (`D5 95 E9 65 B6 1A 0C 0B`), `.didx` (`1F C3 87 21 E0 5D 9A 43`),
/// or `.blob` (`62 79 95 27 E4 E6 D8 C8` / `42 AB 38 07 BE 83 70 A8`).
pub(crate) fn verify_proxmox_backup_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PROXMOX_PBS_SIZE,
        starts(prefix, &[0xD5, 0x95, 0xE9, 0x65, 0xB6, 0x1A, 0x0C, 0x0B])
            || starts(prefix, &[0x1F, 0xC3, 0x87, 0x21, 0xE0, 0x5D, 0x9A, 0x43])
            || starts(prefix, &[0x62, 0x79, 0x95, 0x27, 0xE4, 0xE6, 0xD8, 0xC8])
            || starts(prefix, &[0x42, 0xAB, 0x38, 0x07, 0xBE, 0x83, 0x70, 0xA8])
            || (starts(prefix, &[0xEF, 0xAC, 0x88, 0xE5, 0x74, 0x64, 0x95, 0xD5])
                && le64(prefix, 8) == Some(56))
            || (starts(prefix, &[0x0D, 0xA4, 0x16, 0xDF, 0x75, 0x6C, 0x0F, 0x73])
                && le64(prefix, 8) == Some(24))
    );
    Some(Label::ProxmoxPbs)
}

/// Google ChromeOS / Android Puffin Deterministic Deflate Patch (PUF1).
pub(crate) fn verify_puffin_patch(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PUFFIN_PATCH_SIZE && prefix.len() >= 12);
    let hdr_size = be32(prefix, 4)? as u64;
    require!(
        hdr_size >= 4
            && (hdr_size + 8 <= size
                || (hdr_size <= 16_777_216 && matches!(u8_at(prefix, 10)?, 0x10 | 0x12 | 0x1A)))
            && u8_at(prefix, 8)? == 0x08
            && (1..=4).contains(&u8_at(prefix, 9)?)
    );
    Some(Label::PuffinPatch)
}

/// PXHG Multilingual String Package (.hg).
pub(crate) fn verify_pxhg_package(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PXHG_PACKAGE_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"PXHG\x01"),
        (1..=64).contains(&u8_at(prefix, 5)?),
        (1..=64).contains(&u8_at(prefix, 6)?)
    );
    Some(Label::PxhgPackage)
}

/// Radical Entertainment RadCore / ATG Core Cement Library game data archive (`.rcf`).
///
/// Header layout (`radCFFileInfo` v1.2 / ATG Core v2.x):
/// - `@0..32`:  `char m_CementLibraryIdentification[32]` (`"RADCORE CEMENT LIBRARY"` or `"ATG CORE CEMENT LIBRARY"`, NUL-padded)
/// - `@32`:     `u8 m_CementLibraryVersionMajor` (`1` for RadCore, `2` for ATG Core)
/// - `@33`:     `u8 m_CementLibraryVersionMinor` (`0..=9`, typically `2` for RadCore v1.2, `1` for ATG Core v2.1)
/// - `@34`:     `u8 m_BigEndian` (`0` = little-endian, `1` = big-endian)
/// - `@35`:     `u8 m_Valid` (`1` = valid)
/// - `@36..40`: `u32 m_Alignment` (v1.2, typically `0x800`) or `table1_offset` (v1.0/v2.x, typically `0x3C`)
/// - `@40..44`: `u32 m_PadNetSize` (v1.2) or `table1_size` (v2.x)
/// - `@44..48`: `u32 m_HeaderStartPos` (v1.2 directory offset) or `table2_offset` (v2.x filename table offset)
pub(crate) fn verify_radical_rcf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RADICAL_RCF_SIZE,
        prefix.len() >= 48,
        starts(prefix, RADICAL_ATG_MAGIC) || starts(prefix, RADICAL_RAD_MAGIC),
        get_slice(prefix, RADICAL_IDENT_PAD_START, RADICAL_VERSION_MAJOR_OFFSET)?
            .iter()
            .all(|&b| b == 0),
        (1..=4).contains(&u8_at(prefix, RADICAL_VERSION_MAJOR_OFFSET)?),
        u8_at(prefix, RADICAL_VERSION_MINOR_OFFSET)? <= 9,
        u8_at(prefix, RADICAL_BIG_ENDIAN_OFFSET)? <= 1,
        u8_at(prefix, RADICAL_VALID_FLAG_OFFSET)? != 0
    );
    let align_or_dir_ok = le32(prefix, RADICAL_ALIGN_OR_DIR_OFFSET)? >= 36
        || be32(prefix, RADICAL_ALIGN_OR_DIR_OFFSET)? >= 36;
    if align_or_dir_ok {
    } else {
        require!(
            le32(prefix, RADICAL_HEADER_POS_OFFSET)? >= 48
                || be32(prefix, RADICAL_HEADER_POS_OFFSET)? >= 48
        );
    }
    Some(Label::RadicalRcf)
}

/// Rockstar Games RAGE Engine RSC7 Resource Container.
pub(crate) fn verify_rage_rsc7(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RAGE_RSC7_SIZE,
        prefix.len() >= 16,
        starts(prefix, RAGE_RSC7_MAGIC),
        (1..=256).contains(&le32(prefix, RAGE_RSC7_VERSION_OFFSET)?),
        le32(prefix, RAGE_RSC7_SYS_FLAGS_OFFSET)? != 0,
        le32(prefix, RAGE_RSC7_GFX_FLAGS_OFFSET)? != 0
    );
    Some(Label::RageRsc7)
}

/// Half-Life RCBot Waypoint File (.rcw, 2004) Cheeseh.
pub(crate) fn verify_rcbot_rcw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RCBOT_RCW_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"RCBot\0"),
        get_slice(prefix, 8, 12)? == b"\x04\0\0\0",
        (1..=8192).contains(&le32(prefix, 16).unwrap_or(0))
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::RcbotRcw)
}

/// Rebellion Developments Asura Engine Resource Archive (.asr / .pc, 2005) Rebellion Developments.
pub(crate) fn verify_rebellion_asura(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_REBELLION_ASURA_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"Asura   "),
        matches!(get_slice(prefix, 8, 12)?, b"FNFO" | b"RSCF" | b"ZLIB" | b"ASFZ" | b"Sura")
    );
    Some(Label::RebellionAsura)
}

/// CD Projekt RED REDengine 4 Archive (.archive, 2020) CD Projekt RED.
pub(crate) fn verify_redengine_rdar(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_REDENGINE_RDAR_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"RDAR\x0c\0\0\0"),
        le64(prefix, 8).unwrap_or(0) >= 40,
        le32(prefix, 16).unwrap_or(0) > 0,
        get_slice(prefix, 20, 32)? == [0u8; 12]
    );
    Some(Label::RedengineRdar)
}

/// Electronic Arts RefPack-Compressed BIGF Archive (.big, Electronic Arts).
pub(crate) fn verify_refpack_big(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_REFPACK_BIG_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"\x10\xfb\x00\x71\x20\xe0BIGF")
    );
    require!(has_non_ascii_text_byte(prefix.get(..24)?));
    Some(Label::RefpackBig)
}

/// Relic Entertainment SGA / Big Archive (`"_ARCHIVE"`, versions 1–10).
pub(crate) fn verify_relic_archive(prefix: &[u8], size: u64) -> Option<Label> {
    const RELIC_ARCHIVE_MAGIC: &[u8; 8] = b"_ARCHIVE"; // @0..8: Relic SGA archive signature
    const VERSION_OFF: usize = 8; // @8..12: u32 LE major format version (1, 2, 4..=10)

    require!(
        size >= MIN_RELIC_ARCHIVE_SIZE,
        prefix.len() >= 32,
        starts(prefix, RELIC_ARCHIVE_MAGIC),
        matches!(le32(prefix, VERSION_OFF)?, 1 | 2 | 4 | 5 | 6 | 7 | 8 | 9 | 10)
    );
    Some(Label::RelicArchive)
}

/// Relic Entertainment PIFF RBHF RigidBody Header (.rbh, 2004) Relic Entertainment.
pub(crate) fn verify_relic_rbh(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RELIC_RBH_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"PIFF"),
        get_slice(prefix, 8, 16)? == b"RBHFRBHH",
        get_slice(prefix, 28, 32)? == b"\x02\0\x04\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::RelicRbh)
}

/// Ren'Py Visual Novel Archive (.rpa).
pub(crate) fn verify_renpy_rpa(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RENPY_RPA_SIZE
            && prefix.len() >= 25
            && matches!(get_slice(prefix, 4, 8)?, b"3.0 " | b"2.0 " | b"3.2 " | b"4.0 ")
            && get_slice(prefix, 8, 24)?.iter().all(|b| b.is_ascii_hexdigit())
            && (u8_at(prefix, 24).unwrap_or(0) == b' '
                || u8_at(prefix, 24).unwrap_or(0) == b'\n'
                || u8_at(prefix, 24).unwrap_or(0) == b'\r')
    );
    Some(Label::Rpa)
}

/// RESCSEG Segmented Resource Package (.cmf).
pub(crate) fn verify_rescseg_cmf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RESCSEG_CMF_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"RESCSEG\0"),
        le32(prefix, 8) == Some(1),
        le32(prefix, 16).unwrap_or(0) > 0,
        le32(prefix, 16) == le32(prefix, 20)
    );
    Some(Label::RescsegCmf)
}

/// ResFile Zlib-Compressed Resource Container (.rsc).
pub(crate) fn verify_resfile_rsc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RESFILE_RSC_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"ResFile\0\x03\0\0\0eD\0\0"),
        is_valid_zlib_header(u8_at(prefix, 24)?, u8_at(prefix, 25)?)
    );
    Some(Label::ResfileRsc)
}

/// RFZPACK Resource Archive (.rfz).
pub(crate) fn verify_rfzpack_rfz(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RFZPACK_RFZ_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"RFZPACK\0"),
        (1..=64).contains(&le32(prefix, 8).unwrap_or(0)),
        le32(prefix, 12).unwrap_or(0) > 0
    );
    Some(Label::RfzpackRfz)
}

/// Enterbrain RPG Maker XP / VX / VX Ace RGSS Archive (.rgssad / .rgss2a / .rgss3a).
pub(crate) fn verify_rgssad(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RGSSAD_SIZE
            && prefix.len() >= 12
            && get_slice(prefix, 0, 7)? == b"RGSSAD\0"
            && (1..=3).contains(&u8_at(prefix, 7)?)
    );
    Some(Label::Rgssad)
}

/// Rob Northen Computing ProPack Compressed File (RNC).
pub(crate) fn verify_rnc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_RNC_SIZE && prefix.len() >= 18);
    if starts(prefix, b"RNCA") {
        let dir_end = be16(prefix, 4)? as u64;
        require!(dir_end >= 12 && dir_end <= size);
    } else {
        let method = u8_at(prefix, 3)?;
        let unpacked_size = be32(prefix, 4)? as u64;
        let packed_size = be32(prefix, 8)? as u64;
        // Standard RNC header is 18 bytes; early RNC1 ("rnc1old") header is 12 bytes
        require!(
            method <= 2
                && unpacked_size > 0
                && unpacked_size <= 0x1000_0000
                && (method == 0 || (packed_size > 0 && packed_size + 12 <= size))
        );
    }
    Some(Label::Rnc)
}

/// Rockstar Games RAGE Package File Archive (`.rpf`, RPF0–RPF8 / LE `7FPR`/`8FPR`, 2006 Rockstar Games).
pub(crate) fn verify_rockstar_rpf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ROCKSTAR_RPF_SIZE, prefix.len() >= 16);
    let ok = match get_slice(prefix, 0, 4)? {
        b"7FPR" | b"8FPR" => {
            let entry_count = le32(prefix, 4)?;
            let names_len = le32(prefix, 8)?;
            let enc = le32(prefix, 12)?;
            let enc_ok = get_slice(prefix, 12, 16)? == b"OPEN"
                || get_slice(prefix, 12, 16)? == b"NG\0\0"
                || matches!(enc, 0x0FFF_FFF9 | 0x0FEF_FFFF | 0x0FFF_FF99);
            (1..=2_000_000).contains(&entry_count) && names_len <= 64 * 1024 * 1024 && enc_ok
        }
        b"RPF0" | b"RPF2" | b"RPF3" | b"RPF4" | b"RPF6" | b"RPF7" | b"RPF8" => {
            let le_ok = le32(prefix, 4).is_some_and(|s| (16..=0x1000_0000).contains(&s))
                && le32(prefix, 8).is_some_and(|c| (1..=2_000_000).contains(&c));
            let be_ok = be32(prefix, 4).is_some_and(|s| (16..=0x1000_0000).contains(&s))
                && be32(prefix, 8).is_some_and(|c| (1..=2_000_000).contains(&c));
            (le_ok || be_ok) && has_non_ascii_text_byte(prefix.get(..16)?)
        }
        _ => false,
    };
    require!(ok);
    Some(Label::RockstarRpf)
}

/// WOLF RPG Editor DXLib Encrypted Archive (.wolf, 2008) SmokingWOLF.
pub(crate) fn verify_rpg_wolf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RPG_WOLF_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"DX\x08\0"),
        le64(prefix, 8) == Some(0x40),
        get_slice(prefix, 40, 48)? == b"\xa4\x03\0\0\0\0\x3a\x01"
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::RpgWolf)
}

/// Kadokawa RPG Maker MV / MZ Encrypted Asset (.rpgmvp / .rpgmvo / .rpgmvm / .png_ / .ogg_).
pub(crate) fn verify_rpgmv(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RPGMV_SIZE
            && prefix.len() >= 16
            && get_slice(prefix, 0, 16)? == b"RPGMV\0\0\0\0\x03\x01\0\0\0\0\0"
    );
    Some(Label::Rpgmv)
}

/// SAR / CAR v2.0x archive: `"CAR 2.00"` or `"CAR 2.01"` followed by a binary member header (`@8..32` contains at least one non-text byte).
pub(crate) fn verify_sar_car(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = b"CAR 2.0"; // @0, 7 bytes
    const MINOR: usize = 7; // @7: '0' or '1'
    const HEADER: core::ops::Range<usize> = 8..32; // binary member header
    require!(size >= MIN_SAR_CAR_SIZE, prefix.len() >= HEADER.end, starts(prefix, MAGIC));
    require!(matches!(u8_at(prefix, MINOR)?, b'0' | b'1'));
    require!(
        get_slice(prefix, HEADER.start, HEADER.end)?.iter().any(|&b| !(0x09..0x80).contains(&b))
    );
    Some(Label::SarCar)
}

/// Blizzard StarCraft / StarCraft: Remastered Replay (.rep, 1998) Blizzard Entertainment.
pub(crate) fn verify_sc_rep(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SC_REP_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\xc2\x19\xc2\x93\x01\0\0\0\x04\0\0\0seRS"),
        is_valid_zlib_header(u8_at(prefix, 32)?, u8_at(prefix, 33)?)
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::ScRep)
}

/// Michael Schindler `SZIP` Block-Sorted Stream (`.sz`, 1997 Michael Schindler, `"SZ\n\x04"`).
pub(crate) fn verify_schindler_szip(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SCHINDLER_SZIP_SIZE,
        prefix.len() >= 8,
        starts(prefix, b"SZ\x0A\x04"),
        (1..=2).contains(&u8_at(prefix, 4)?)
    );
    Some(Label::SchindlerSzip)
}

/// MIT Scratch 1.x Squeak Binary Project (`.sb`).
///
/// Header layout:
/// - `@0..10`: `"ScratchV01"` or `"ScratchV02"`
/// - `@10..14`: `u32 BE` `info_obj_size` (`>= 14`)
/// - `@14..24`: `"ObjS\x01Stch\x01"` Squeak object stream header
pub(crate) fn verify_scratch_project_v1(prefix: &[u8], size: u64) -> Option<Label> {
    const SCRATCH_V01: &[u8; 10] = b"ScratchV01"; // @0..10: Scratch 1.0..1.3 header
    const SCRATCH_V02: &[u8; 10] = b"ScratchV02"; // @0..10: Scratch 1.4 header
    const INFO_SIZE_OFF: usize = 10; // @10..14: u32 BE info ObjStream byte size
    const OBJS_OFF: usize = 14; // @14..24: Squeak ObjS/Stch stream header
    const OBJS_MAGIC: &[u8; 10] = bytes!(b"ObjS", &[1], b"Stch", &[1]);

    require!(
        size >= MIN_SCRATCH_V1_SIZE,
        prefix.len() >= 24,
        starts(prefix, SCRATCH_V01) || starts(prefix, SCRATCH_V02),
        be32(prefix, INFO_SIZE_OFF)? >= 14,
        eq_at(prefix, OBJS_OFF, OBJS_MAGIC)
    );
    Some(Label::ScratchSb)
}

/// SCS Software Prism3D HashFS Game Archive (`.scs`, Euro Truck Simulator 2 / ATS, 2012 SCS Software).
pub(crate) fn verify_scs_hashfs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SCS_HASHFS_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"SCS#"),
        (1..=2).contains(&le16(prefix, 4)?),
        eq_at(prefix, 8, b"CITY") || eq_at(prefix, 8, b"MD5\0"),
        le32(prefix, 12)? >= 1
    );
    Some(Label::ScsHashfs)
}

/// LucasArts SCUMM v3 Room Resource File (.lfl, RO/HD/BX).
pub(crate) fn verify_scumm_room_lfl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SCUMM_ROOM_LFL_SIZE,
        prefix.len() >= 24,
        get_slice(prefix, 4, 12)? == b"RO\x0c\0\0\0HD",
        get_slice(prefix, 22, 24)? == b"BX"
    );
    require!(has_non_ascii_text_byte(prefix.get(..24)?));
    Some(Label::ScummRoomLfl)
}

/// Sega Dreamcast / PS2 AFS Archive (`.afs`, `"AFS\0"`).
pub(crate) fn verify_sega_afs_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SEGA_AFS_SIZE, prefix.len() >= 16, starts(prefix, b"AFS\0"));
    let count = le32(prefix, 4)? as u64;
    let first_off = le32(prefix, 8)? as u64;
    require!((1..=65535).contains(&count), first_off >= 8 + count * 8 && first_off <= size);
    Some(Label::SegaAfs)
}

/// Sega Dreamcast PVM Texture Archive (PVMH).
pub(crate) fn verify_sega_pvm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SEGA_PVM_SIZE, prefix.len() >= 14);
    let pvmh_off = if starts(prefix, b"PVMH") || starts(prefix, b"XVMH") {
        0usize
    } else if starts(prefix, b"GBIX") {
        let gbix_len = le32(prefix, 4)? as usize;
        let next = 8usize.checked_add(gbix_len)?;
        require!(
            (4..=64).contains(&gbix_len),
            next + 12 <= prefix.len(),
            get_slice(prefix, next, next + 4)? == b"PVMH"
                || get_slice(prefix, next, next + 4)? == b"XVMH"
        );
        next
    } else {
        return None;
    };
    let chunk_size = le32(prefix, pvmh_off + 4)?;
    let flags = le16(prefix, pvmh_off + 8)?;
    let tex_count = le16(prefix, pvmh_off + 10)?;
    require!(chunk_size >= 6, flags <= 0x0FFF, (1..=4096).contains(&tex_count));
    Some(Label::Pvm)
}

/// Senjutsu Tactical Campaign Map Data (SENJUTSUMAPDATA, .s7).
pub(crate) fn verify_senjutsu_s7(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SENJUTSU_S7_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"SENJUTSUMAPDATA\0\0\x01\0")
    );
    require!(has_non_ascii_text_byte(prefix.get(..24)?));
    Some(Label::SenjutsuS7)
}

/// Serpent OS / AerynOS Stone Package (`.stone`, `\0mos` + `be32(28) == 1`).
pub(crate) fn verify_serpent_stone_package(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SERPENT_STONE_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"\x00mos"),
        (1..=4).contains(&u8_at(prefix, 27).unwrap_or(0))
            || (1..=4).contains(&u8_at(prefix, 24).unwrap_or(0)),
        be32(prefix, 28) == Some(1)
    );
    Some(Label::SerpentStone)
}

/// TG-Soft Setup Specialist Multi-Part Archive Volume (`b"TGCF\0"`).
pub(crate) fn verify_setup_specialist_tgcf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SETUP_SPECIALIST_TGCF_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"TGCF\x00")
    );
    Some(Label::Tgcf)
}

/// Sierra On-Line Adventure Game Interpreter Resource Volume (`VOL.0`, 1984 Sierra On-Line).
pub(crate) fn verify_sierra_agi_vol(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SIERRA_AGI_VOL_SIZE,
        prefix.len() >= 16,
        starts(prefix, hex2bin!("12 34")),
        u8_at(prefix, 2)? <= 15
    );
    let vol = u8_at(prefix, 2)?;
    // AGI v2: 5-byte resource header (12 34 vol rlen_le16)
    let rlen_v2 = le16(prefix, 3)? as usize;
    let off2_v2 = 5 + rlen_v2;
    if rlen_v2 >= 8
        && off2_v2 + 5 <= prefix.len()
        && u8_at(prefix, off2_v2)? == 0x12
        && u8_at(prefix, off2_v2 + 1)? == 0x34
        && u8_at(prefix, off2_v2 + 2)? == vol
        && le16(prefix, off2_v2 + 3)? >= 4
    {
        return Some(Label::SierraAgiVol);
    }
    // AGI v3: 7-byte resource header (12 34 vol ulen_le16 clen_le16)
    let ulen_v3 = le16(prefix, 3)?;
    let clen_v3 = le16(prefix, 5)? as usize;
    let off2_v3 = 7 + clen_v3;
    if clen_v3 >= 8
        && (clen_v3 as u16) <= ulen_v3
        && off2_v3 + 7 <= prefix.len()
        && u8_at(prefix, off2_v3)? == 0x12
        && u8_at(prefix, off2_v3 + 1)? == 0x34
        && u8_at(prefix, off2_v3 + 2)? == vol
    {
        return Some(Label::SierraAgiVol);
    }
    None
}

/// Maxis The Sims 1 IFF 2.5 Resource Container (`.iff`, `.spf`, `.flr`, `.wll`).
///
/// Header layout:
/// - `@0..35`: `"IFF FILE 2.5:TYPE FOLLOWED BY SIZE\0"`
/// - `@60..64`: `u32 BE` resource map offset
pub(crate) fn verify_sims1_iff(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 35] = bytes!(b"IFF FILE 2.5:TYPE FOLLOWED BY SIZE", &[0]); // @0..35: IFF 2.5 banner + NUL

    require!(size >= MIN_SIMS1_IFF_SIZE, prefix.len() >= MAGIC.len(), starts(prefix, MAGIC));
    Some(Label::Sims1Iff)
}

/// MS-DOS Dominic Herity SLIM Compressed File (`.slm` / `.com`, `CD 20 73 4C 1A 02` or `CD 20 73 4C 69 4D 1A`).
pub(crate) fn verify_slim_archive(prefix: &[u8], size: u64) -> Option<Label> {
    const SLIM_MAGIC_1: &[u8; 6] = b"\xCD\x20sL\x1A\x02"; // @0..6: INT 20h + "sL\x1A\x02"
    const SLIM_MAGIC_2: &[u8; 7] = b"\xCD\x20sLiM\x1A"; // @0..7: INT 20h + "sLiM\x1A"

    require!(
        size >= MIN_SLIM_ARCHIVE_SIZE,
        prefix.len() >= 34,
        starts(prefix, SLIM_MAGIC_1) || starts(prefix, SLIM_MAGIC_2)
    );
    Some(Label::SlimArchive)
}

/// SoftLib Compressed Library Archive (`.slb`, `"SLIB"`).
pub(crate) fn verify_softlib_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SOFTLIB_SLB_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"SLIB"),
        le16(prefix, 4) == Some(2),
        (1..=4096).contains(&le16(prefix, 6)?),
        (0x21..=0x7E).contains(&u8_at(prefix, 8)?)
    );
    Some(Label::SoftlibSlb)
}

/// SoftMaker Office Resource Archive (`.dwr`).
pub(crate) fn verify_softmaker_dwr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SOFTMAKER_DWR_SIZE,
        prefix.len() >= 96,
        starts(prefix, SOFTMAKER_DWR_BANNER),
        get_slice(prefix, 67, 80)?.iter().all(|&b| b == 0),
        eq_at(prefix, SOFTMAKER_DWR_TAG_OFFSET, b"jklm")
    );
    Some(Label::SoftmakerDwr)
}

/// Sun Solaris / SVR4 Package Datastream (`.pkg`, 1989 Sun).
pub(crate) fn verify_solaris_pkg(prefix: &[u8], size: u64) -> Option<Label> {
    const PKG_MAGIC: &[u8; 20] = b"# PaCkAgE DaTaStReAm"; // @0..20: SVR4 pkgtrans header

    require!(
        size >= MIN_SOLARIS_PKG_SIZE,
        prefix.len() >= 32,
        starts(prefix, PKG_MAGIC),
        matches!(u8_at(prefix, 20)?, b'\n' | b'\r' | b':'),
        scan_range(prefix, 20, 512, b"# end of header")
    );
    Some(Label::SolarisPkg)
}

/// Sony PictureGear / VAIO Theme Package (PTM fnIA, .pts).
pub(crate) fn verify_sony_picturegear_pts(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SONY_PICTUREGEAR_PTS_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"PTM \x04\0\x01\0fnIAP\x03\0\0"),
        get_slice(prefix, 24, 28)? == b"fnIA"
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::SonyPicturegearPts)
}

/// SPIS LH5-Compressed Container (`SPIS\x1ALH5`, Delphi `TCompress`, `.lng`).
pub(crate) fn verify_spis_lh5(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SPIS_LH5_SIZE,
        prefix.len() >= 21,
        starts(prefix, b"SPIS\x1aLH5"),
        (1..=16).contains(&le32(prefix, 17).unwrap_or(0))
    );
    Some(Label::SpisLh5)
}

/// Squeeze It (SQZ) archive (`.sqz`, 1990–1993 Jonas I. Hammarberg).
///
/// Header layout:
/// - `@0..5`: `"HLSQZ"` signature
/// - `@5`: ASCII version (`'1'`)
/// - `@6`: host OS in bits 0..2 (0 PC-DOS, 1 OS/2, 2 MVS, 3 HPFS, 4 Amiga, 5 Macintosh, 6 Unix); bits 3..7 reserved = 0
/// - `@7`: flags (bit 0 byte order, bit 1 DOS date/time, bit 2 security envelope, bit 3 encrypted,
///   bit 4 unified data); bits 5..7 reserved = 0
/// - `@8`: first block: `0` end of archive, `1..=18` extra-data block type, `>= 19` file header size
///   followed by an 8-bit additive header checksum over the `size` header bytes (method in the low nibble, `0..=4`).
pub(crate) fn verify_sqz_archive(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION: usize = 5; // @5: ASCII archive version
    const OS: usize = 6; // @6: host OS (bits 0..2)
    const FLAGS: usize = 7; // @7: archive flags (bits 0..4)
    const FIRST_BLOCK: usize = 8; // @8: first block size / type byte
    const HEADER_SUM: usize = 9; // @9: additive checksum of a normal file header
    const HEADER_START: usize = 10; // @10..: file header body (method, sizes, date, attr, CRC, name)
    const SQZ_VERSION_1: u8 = b'1';
    const OS_RESERVED_MASK: u8 = 0xF8; // bits 3..7 reserved
    const FLAGS_RESERVED_MASK: u8 = 0xE0; // bits 5..7 reserved
    const MIN_FILE_HEADER_SIZE: u8 = 19; // 18 fixed bytes + >= 1 filename byte
    const METHOD_MASK: u8 = 0x0F;
    const MAX_METHOD: u8 = 4;

    require!(
        size >= MIN_SQZ_ARCHIVE_SIZE,
        starts(prefix, SQZ_MAGIC),
        u8_at(prefix, VERSION)? == SQZ_VERSION_1,
        u8_at(prefix, OS)? & OS_RESERVED_MASK == 0,
        u8_at(prefix, FLAGS)? & FLAGS_RESERVED_MASK == 0
    );
    let block = u8_at(prefix, FIRST_BLOCK)?;
    if block >= MIN_FILE_HEADER_SIZE {
        let sum = u8_at(prefix, HEADER_SUM)?;
        let header = get_slice(prefix, HEADER_START, HEADER_START + block as usize)?;
        require!(
            header.iter().fold(0u8, |acc, &b| acc.wrapping_add(b)) == sum,
            header[0] & METHOD_MASK <= MAX_METHOD
        );
    }
    Some(Label::SqzArchive)
}

/// Nintendo Super Smash Bros. Ultimate Binary Parameter (paracobn, .prc).
pub(crate) fn verify_ssbu_paracobn_prc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SSBU_PARACOBN_PRC_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"paracobn\0\x02\0\0"),
        get_slice(prefix, 16, 24)? == b"\0\0\0\0\0\0\0\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::SsbuParacobnPrc)
}

/// Strategic Simulations General File Format Archive (`.gff`, 1991 Strategic Simulations & Miles Design).
pub(crate) fn verify_ssi_gff_archive(prefix: &[u8], size: u64) -> Option<Label> {
    const GFFI_MAGIC: &[u8; 4] = b"GFFI"; // @0..4: "GFFI" container signature
    const VERSION_OFFSET: usize = 4; // @4..8: u32 LE format version (0x00030000 = v3.0)
    const HDR_SIZE_OFFSET: usize = 8; // @8..12: u32 LE header size (28 bytes)
    const TOC_OFFSET_OFFSET: usize = 16; // @16..20: u32 LE table-of-contents byte offset
    const EXPECTED_VERSION: u32 = 0x0003_0000; // GFF v3.0 version word
    const EXPECTED_HDR_SIZE: u32 = 28; // sizeof(GFFHeader)

    require!(
        size >= MIN_SSI_GFF_ARCHIVE_SIZE,
        prefix.len() >= 28,
        starts(prefix, GFFI_MAGIC),
        le32(prefix, VERSION_OFFSET) == Some(EXPECTED_VERSION),
        le32(prefix, HDR_SIZE_OFFSET) == Some(EXPECTED_HDR_SIZE)
    );
    let toc_offset = le32(prefix, TOC_OFFSET_OFFSET)? as u64;
    require!(toc_offset >= MIN_SSI_GFF_ARCHIVE_SIZE, toc_offset <= size);
    Some(Label::SsiGff)
}

/// Chucklefish Starbound Asset Package (`.pak`, `"SBAsset6"`, 2013 Chucklefish).
pub(crate) fn verify_starbound_pak(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_STARBOUND_PAK_SIZE, prefix.len() >= 16, starts(prefix, b"SBAsset6"));
    let idx_off = be64(prefix, 8)?;
    require!(idx_off >= 16 && idx_off < size);
    Some(Label::StarboundPak)
}

/// SteelMC Rust Minecraft Server Region (`STLR`, `.srg`).
pub(crate) fn verify_steelmc_stlr_srg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_STEELMC_STLR_SRG_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"STLR"),
        (1..=64).contains(&le32(prefix, 4).unwrap_or(0)),
        le32(prefix, 8).unwrap_or(u32::MAX) <= 1024
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::SteelmcStlrSrg)
}

/// STOS memory bank `"Lionpoubnk"`, or AMOS banks group `"AmBs"` + `u16 BE` bank count (`>= 1`).
pub(crate) fn verify_stos_amos_bank(prefix: &[u8], size: u64) -> Option<Label> {
    const AMOS_BANK_COUNT: usize = 4; // @4: u16 BE
    require!(size >= MIN_STOS_AMOS_BANK_SIZE, prefix.len() >= 16);
    if starts(prefix, b"AmBs") {
        require!(be16(prefix, AMOS_BANK_COUNT)? >= 1, prefix.get(4..12)?.contains(&0));
    } else {
        require!(starts(prefix, b"Lionpoubnk"), prefix.get(10..16)?.contains(&0));
    }
    Some(Label::StosAmos)
}

/// Sun Microsystems XRS Resource Archive (.xrs) Sun Microsystems.
pub(crate) fn verify_sun_xrs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SUN_XRS_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"Sun\xfe\xedRes\x01\0\0")
    );
    Some(Label::SunXrs)
}

/// Nintendo Switch Partition FS0 / NSP Package (PFS0).
pub(crate) fn verify_switch_pfs0(prefix: &[u8], size: u64) -> Option<Label> {
    let entry0_off = le64(prefix, 16)?;
    let entry0_size = le64(prefix, 24)?;
    require!(
        size >= MIN_SWITCH_PFS0_SIZE
            && prefix.len() >= 40
            && (1..=65535).contains(&le32(prefix, 4).unwrap_or(0))
            && (1..=0x10_0000).contains(&le32(prefix, 8).unwrap_or(0))
            && le32(prefix, 12) == Some(0)
            && entry0_off.saturating_add(entry0_size) <= size
            && le32(prefix, 32) == Some(0)
            && le32(prefix, 36) == Some(0)
    );
    Some(Label::SwitchPfs0)
}

/// Symbian SISX: UID1 `0x10201A7A`, UID2 `0x00000000`, UID3 = package UID (non-zero), UID checksum.
pub(crate) fn verify_symbian_sisx(prefix: &[u8], size: u64) -> Option<Label> {
    const UID1_SISX: u32 = 0x1020_1A7A; // @0: u32 LE
    const UID2: usize = 4; // @4: u32 LE == 0
    const UID3: usize = 8; // @8: u32 LE package UID
    require!(size >= MIN_SISX_SIZE, le32(prefix, 0)? == UID1_SISX);
    require!(le32(prefix, UID2)? == 0, le32(prefix, UID3)? != 0);
    Some(Label::Sisx)
}

/// Symbolics Lisp Machine Carry-Tape / Distribution Tape (`.carry`, 1982) Symbolics.
///
/// Opens with `EMBEDDED-LENGTH-STREAM VERSION <n>\x8dEND\x8dBLOK<12-digit-len>` where
/// `0x8D` is the Lisp Machine `#\Return` character.
pub(crate) fn verify_symbolics_carry(prefix: &[u8], size: u64) -> Option<Label> {
    const VER_OFF: usize = 31; // @31: ASCII version digits followed by \x8dEND\x8dBLOK
    const BLOK_LEN_DIGITS: usize = 12; // 12 zero-padded decimal ASCII digits after BLOK
    require!(
        size >= MIN_SYMBOLICS_CARRY_SIZE,
        prefix.len() >= MIN_SYMBOLICS_CARRY_SIZE as usize,
        starts(prefix, SYMBOLICS_ELS_PREFIX)
    );
    let mut i = VER_OFF;
    let ver_max = prefix.len().min(VER_OFF + 4);
    while i < ver_max && u8_at(prefix, i)?.is_ascii_digit() {
        i += 1;
    }
    require!(i > VER_OFF);
    let len_off = i + SYMBOLICS_BLOK_MARKER.len();
    let end_off = len_off + BLOK_LEN_DIGITS;
    require!(
        prefix.len() >= end_off,
        eq_at(prefix, i, SYMBOLICS_BLOK_MARKER),
        get_slice(prefix, len_off, end_off)?.iter().all(|b| b.is_ascii_digit())
    );
    Some(Label::SymbolicsCarry)
}

/// Synopse mORMot SynLZ Compressed Stream (.synlz, Arnaud Bouchez / Synopse).
pub(crate) fn verify_synopse_synlz(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SYNOPSE_SYNLZ_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"\x51\x10\xa5\xab"),
        (1..=0x1000_0000).contains(&le32(prefix, 4).unwrap_or(0)),
        le32(prefix, 12).unwrap_or(0) > 0,
        le32(prefix, 12) == le32(prefix, 20)
    );
    require!(has_non_ascii_text_byte(prefix.get(..28)?));
    Some(Label::SynopseSynlz)
}

/// tModLoader Terraria Mod Package (`.tmod`, 2015 tModLoader).
pub(crate) fn verify_terraria_tmod(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_TERRARIA_TMOD_SIZE, prefix.len() >= 32, starts(prefix, b"TMOD"));
    let vlen = u8_at(prefix, 4)? as usize;
    require!((3..=24).contains(&vlen), 5 + vlen + 20 <= prefix.len());
    let ver = get_slice(prefix, 5, 5 + vlen)?;
    require!(
        ver.iter().all(|&b| b.is_ascii_digit() || b == b'.'),
        ver.contains(&b'.'),
        has_non_ascii_text_byte(prefix.get(..5 + vlen + 20)?)
    );
    Some(Label::TerrariaTmod)
}

/// thilmera7 System Monitor ZPAC Resource Package (`.t7p`, `"G15:ZPAC"`).
pub(crate) fn verify_thilmera7_zpac_t7p(prefix: &[u8], size: u64) -> Option<Label> {
    const THILMERA7_ZPAC_MAGIC: &[u8; 8] = b"G15:ZPAC"; // @0..8

    require!(
        size >= MIN_THILMERA7_ZPAC_T7P_SIZE,
        prefix.len() >= 16,
        starts(prefix, THILMERA7_ZPAC_MAGIC),
        has_non_ascii_text_byte(prefix.get(8..16)?)
    );
    Some(Label::Thilmera7ZpacT7p)
}

/// The Time of War Engine Data File v4.00 (`.d3e`).
pub(crate) fn verify_time_of_war_d3e(prefix: &[u8], size: u64) -> Option<Label> {
    const TIME_OF_WAR_MAGIC: &[u8; 38] = b"The Time of War       Engine Data File"; // @0..38: banner magic

    require!(
        size >= MIN_TIME_OF_WAR_D3E_SIZE,
        prefix.len() >= 408,
        starts(prefix, TIME_OF_WAR_MAGIC),
        has_non_ascii_text_byte(prefix.get(..408)?)
    );
    Some(Label::TimeOfWarD3e)
}

/// DEC TOPS-10 BACKUP / Interchange 36-Bit Tape Stream (`.bck`, 1975 Digital Equipment).
///
/// Each 544-word record is stored as `544 * 5 = 2,720` core-dump bytes (bits `7..4` of every 5th byte `== 0`):
/// - `w[0]` (`G$TYPE`): `1..=7` (1=label, 2=start, 3=end, 4=file, 5=UFD, 6=end-of-vol, 7=comment)
/// - `w[1]` (`G$SEQ`): `1..=1_000_000` (1-based block sequence number)
/// - `w[4]` (`G$CHK`): exact 36-bit add-and-rotate-left-1 checksum over all 544 words (with `w[4] = 0`)
/// - `w[5]` (`G$SIZ`) + `w[6]` (`G$LND`): data/non-data payload words (`<= 512`, sum `<= 512`).
pub(crate) fn verify_tops10_backup(prefix: &[u8], size: u64) -> Option<Label> {
    const RECORD_WORDS: usize = 544; // 32 header words + 512 data words
    const BYTES_PER_WORD: usize = 5; // 36-bit ANSI core-dump packing
    const RECORD_BYTES: usize = RECORD_WORDS * BYTES_PER_WORD; // 2,720 bytes
    const WORD36_MASK: u64 = (1u64 << 36) - 1;

    require!(
        size >= MIN_TOPS10_BACKUP_SIZE,
        size.is_multiple_of(RECORD_BYTES as u64),
        prefix.len() >= RECORD_BYTES,
        has_non_ascii_text_byte(prefix.get(..64)?)
    );

    let mut acc: u64 = 0;
    let mut g_type: u64 = 0;
    let mut g_seq: u64 = 0;
    let mut g_chk: u64 = 0;
    let mut g_siz: u64 = 0;
    let mut g_lnd: u64 = 0;
    for idx in 0..RECORD_WORDS {
        let base = idx * BYTES_PER_WORD;
        let b3 = u8_at(prefix, base + 3)?;
        let b4 = u8_at(prefix, base + 4)?;
        // Clean ANSI core-dump has bits 7..4 == 0; real DEC TM10 hardware copies bits 1..0 of byte 3 into bits 5..4.
        if (b4 & 0xC0) != 0 || ((b4 & 0x30) != 0 && ((b4 >> 4) & 0x03) != (b3 & 0x03)) {
            return None;
        }
        let w = ((u8_at(prefix, base)? as u64) << 28)
            | ((u8_at(prefix, base + 1)? as u64) << 20)
            | ((u8_at(prefix, base + 2)? as u64) << 12)
            | ((b3 as u64) << 4)
            | ((b4 & 0x0F) as u64);
        match idx {
            0 => g_type = w,
            1 => g_seq = w,
            4 => g_chk = w,
            5 => g_siz = w,
            6 => g_lnd = w,
            _ => {}
        }
        let term = if idx == 4 { 0 } else { w };
        acc = acc.wrapping_add(term) & WORD36_MASK;
        acc = ((acc << 1) | (acc >> 35)) & WORD36_MASK;
    }

    require!(
        (1..=7).contains(&g_type),
        (1..=1_000_000).contains(&g_seq),
        g_siz <= 512,
        g_lnd <= 512,
        g_siz + g_lnd <= 512,
        acc == g_chk
    );

    Some(Label::Tops10Backup)
}

/// DEC TOPS-20 `DUMPER` 36-bit tape saveset (`.dumper` / `.tap`, 1976) Digital Equipment.
///
/// Validates SIMH 5-byte-per-36-bit-word core-dump packing across all 518 words of the first
/// 2,590-byte record (`(b4 & 0xF0) == 0` = 2,072 zero bits), a valid signed 36-bit record type
/// (`w[4]` in `-5..=0`), non-zero sequence number `w[5] >= 1`, and the exact 36-bit
/// one's-complement checksum `sum(w[0..518]) % (2^36 - 1) == 0`.
pub(crate) fn verify_tops20_dumper(prefix: &[u8], size: u64) -> Option<Label> {
    const RECORD_WORDS: usize = 518; // 6 header words + 512 data words = 518 × 36-bit words
    const BYTES_PER_WORD: usize = 5; // SIMH core-dump 5-byte packing
    const RECORD_BYTES: usize = RECORD_WORDS * BYTES_PER_WORD; // @0..2590
    require!(
        size >= MIN_TOPS20_DUMPER_SIZE,
        prefix.len() >= RECORD_BYTES,
        has_non_ascii_text_byte(prefix.get(..64)?)
    );

    let mut sum: u128 = 0;
    let mut any_nonzero = false;
    let mut w4: u64 = 0;
    let mut w5: u64 = 0;
    for idx in 0..RECORD_WORDS {
        let base = idx * BYTES_PER_WORD;
        let b3 = u8_at(prefix, base + 3)?;
        let b4 = u8_at(prefix, base + 4)?;
        // Clean ANSI core-dump has bits 7..4 == 0; real DEC TM10 hardware copies bits 1..0 of byte 3 into bits 5..4.
        if (b4 & 0xC0) != 0 || ((b4 & 0x30) != 0 && ((b4 >> 4) & 0x03) != (b3 & 0x03)) {
            return None;
        }
        let w = ((u8_at(prefix, base)? as u64) << 28)
            | ((u8_at(prefix, base + 1)? as u64) << 20)
            | ((u8_at(prefix, base + 2)? as u64) << 12)
            | ((b3 as u64) << 4)
            | ((b4 & 0x0F) as u64);
        if w != 0 {
            any_nonzero = true;
        }
        if idx == 4 {
            w4 = w;
        } else if idx == 5 {
            w5 = w;
        }
        sum += w as u128;
    }

    // Signed 36-bit record types: 0 (data), -1 (tape hdr), -2 (file hdr), -3 (trailer), -4 (EOT), -5 (dir)
    let valid_type = w4 == 0 || (0x000F_FFFF_FFFB..=PDP10_WORD36_MASK).contains(&w4);
    require!(
        any_nonzero,
        valid_type,
        (1..=1_000_000).contains(&w5),
        sum.is_multiple_of(PDP10_WORD36_MASK as u128)
    );

    Some(Label::Tops20Dumper)
}

/// Creative Assembly Total War PackFile (`.pack`, `"PFH0"`, `"PFH2"`, `"PFH3"`, `"PFH4"`, `"PFH5"`).
pub(crate) fn verify_total_war_pack_pfh(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TOTAL_WAR_PFH_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"PFH4")
            || starts(prefix, b"PFH5")
            || starts(prefix, b"PFH3")
            || starts(prefix, b"PFH2")
            || starts(prefix, b"PFH0"),
        (le32(prefix, 4)? & 0x0F) <= 8,
        has_non_ascii_text_byte(prefix.get(4..24)?)
    );
    Some(Label::TotalWarPfh)
}

/// N3V Games / Auran Trainz Content Dispatcher Pack (`.cdp`, 2001 Auran / N3V Games).
pub(crate) fn verify_trainz_cdp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TRAINZ_CDP_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"ACS$"),
        (1..=4).contains(&le32(prefix, 4)?),
        le32(prefix, 8) == Some(0),
        le32(prefix, 12)? >= 16,
        eq_at(prefix, 20, b"\x07assets\0")
    );
    Some(Label::TrainzCdp)
}

/// `TZIP` Compressed Stream (`.tz`, `"TZ\0\0"`).
pub(crate) fn verify_tzip_compressed(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TZIP_COMPRESSED_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"TZ\0\0"),
        le32(prefix, 4) == Some(4),
        eq_at(prefix, 8, hex2bin!("00 02"))
    );
    Some(Label::Tzip)
}

/// Ubisoft Anvil Engine Save Game Container (.acv).
pub(crate) fn verify_ubisoft_anvil_acv(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UBISOFT_ANVIL_ACV_SIZE,
        prefix.len() >= 32,
        starts(prefix, hex2bin!("33 AA FB 57")),
        matches!(le16(prefix, 8), Some(1..=16))
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::UbisoftAnvilAcv)
}

/// Ubisoft Jade Engine Bigfile Archive (.bf, 2003) Ubisoft Montpellier.
pub(crate) fn verify_ubisoft_jade_bf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UBISOFT_JADE_BF_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"BIG\0"),
        (0x20..=0x30).contains(&le32(prefix, 4).unwrap_or(0)),
        le64(prefix, 24) == Some(u64::MAX)
    );
    Some(Label::UbisoftJadeBf)
}

/// UHARC High-Compression Multimedia Archive (1997) Uwe Herklotz (`"UHA\x01"`..`"UHA\x07"`).
///
/// Header layout:
/// - `@0..3`: `"UHA"` ASCII signature
/// - `@3`: `u8` archive format version (`1..=7`, non-printable binary byte)
/// - `@4..10`: 6-byte bit-packed/encrypted archive header (non-zero in valid archives)
pub(crate) fn verify_uharc(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFFSET: usize = 3; // @3: u8 UHARC format version (1 = v0.1 .. 7 = v0.7)
    const HEADER_LEN: usize = MIN_UHARC_SIZE as usize; // @0..10: 4B magic + 6B packed archive header
    require!(
        size >= MIN_UHARC_SIZE,
        prefix.len() >= HEADER_LEN,
        starts(prefix, UHARC_PREFIX),
        matches!(u8_at(prefix, VERSION_OFFSET), Some(1..=7)),
        get_slice(prefix, 4, HEADER_LEN)?.iter().any(|&b| b != 0)
    );
    Some(Label::Uharc)
}

/// Standalone Unity Engine `SerializedFile` binary asset container
/// (`.assets`, `sharedassets0.assets`, `globalgamemanagers`, `level0`, `unity default resources`, `unity_builtin_extra`, 2005 Unity Technologies).
pub(crate) fn verify_unity_serialized_assets(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_UNITY_SERIALIZED_ASSETS_SIZE && prefix.len() >= 64);
    let ver = be32(prefix, 8)?;
    if (22..=32).contains(&ver) {
        // v22+ (Unity 2019.3+): 48-byte big-endian header:
        // @0..4: u32 BE legacy_metadata_size == 0
        // @4..8: u32 BE legacy_file_size == 0
        // @8..12: u32 BE m_Version (22..=32)
        // @12..16: u32 BE legacy_data_offset == 0
        // @16: u8 m_Endianess (0 = LE, 1 = BE)
        // @17..20: [u8; 3] m_Reserved == [0, 0, 0]
        // @20..24: u32 BE m_MetadataSize (> 16)
        // @24..32: u64 BE m_FileSize (>= m_DataOffset)
        // @32..40: u64 BE m_DataOffset (>= 48 + m_MetadataSize)
        // @40..48: i64 BE m_Unknown (0)
        // @48..: null-terminated ASCII unityVersion (e.g. "2021.3.4f1\0")
        require!(
            be32(prefix, 0) == Some(0),
            be32(prefix, 4) == Some(0),
            be32(prefix, 12) == Some(0),
            u8_at(prefix, 16)? <= 1,
            get_slice(prefix, 17, 20)? == [0, 0, 0]
        );
        let meta_size = be32(prefix, 20)? as u64;
        let file_size = be64(prefix, 24)?;
        let data_offset = be64(prefix, 32)?;
        let _unk = be64(prefix, 40)?;
        require!(
            meta_size >= 16,
            data_offset >= 48u64.saturating_add(meta_size),
            file_size >= data_offset
        );
        let ver_window = get_slice(prefix, 48, prefix.len().min(76))?;
        require!(is_valid_unity_version_str(ver_window));
        return Some(Label::UnityAssets);
    } else if (9..=21).contains(&ver) {
        // v9..=21 (Unity 3.5–2019.2): 20-byte big-endian header:
        // @0..4: u32 BE m_MetadataSize (> 16)
        // @4..8: u32 BE m_FileSize (>= m_DataOffset)
        // @8..12: u32 BE m_Version (9..=21)
        // @12..16: u32 BE m_DataOffset (>= 20 + m_MetadataSize)
        // @16: u8 m_Endianess (0 = LE, 1 = BE)
        // @17..20: [u8; 3] m_Reserved == [0, 0, 0]
        // @20..: null-terminated ASCII unityVersion (e.g. "5.6.7f1\0" or "2018.4.36f1\0")
        let meta_size = be32(prefix, 0)? as u64;
        let file_size = be32(prefix, 4)? as u64;
        let data_offset = be32(prefix, 12)? as u64;
        require!(
            u8_at(prefix, 16)? <= 1,
            get_slice(prefix, 17, 20)? == [0, 0, 0],
            meta_size >= 16,
            data_offset >= 20u64.saturating_add(meta_size),
            file_size >= data_offset
        );
        let ver_window = get_slice(prefix, 20, prefix.len().min(48))?;
        require!(is_valid_unity_version_str(ver_window));
        return Some(Label::UnityAssets);
    }
    None
}

/// Unity WebGL Data Package (`.data` / `.unityweb`, 2015 Unity Technologies / Unity China).
pub(crate) fn verify_unity_webdata(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_UNITY_WEBDATA_SIZE && prefix.len() >= 36);
    let hdr_pos = if starts(prefix, UNITY_WEBDATA_10_MAGIC) {
        16
    } else if starts(prefix, TUANJIE_WEBDATA_10_MAGIC) {
        18
    } else {
        return None;
    };
    let header_len = le32(prefix, hdr_pos)? as usize;
    let first_off = le32(prefix, hdr_pos + 4)? as usize;
    let first_size = le32(prefix, hdr_pos + 8)? as u64;
    let first_name_len = le32(prefix, hdr_pos + 12)? as usize;
    let name_start = hdr_pos + 16;
    require!(
        header_len >= name_start + first_name_len,
        first_off == header_len,
        (1..=256).contains(&first_name_len),
        (first_off as u64).saturating_add(first_size) <= size || size <= 4096
    );
    let name_end = (name_start + first_name_len).min(prefix.len());
    let name_slice = get_slice(prefix, name_start, name_end)?;
    require!(name_slice.iter().all(|&b| (0x20..=0x7E).contains(&b)));
    Some(Label::UnityWebdata)
}

/// Epic Games Unreal Engine Package / Asset (.uasset / .umap / .upk / .u).
pub(crate) fn verify_unreal_package(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_UNREAL_PACKAGE_SIZE && prefix.len() >= 36);
    let ver_le = le32(prefix, 4).unwrap_or(0) as i32;
    let ver_be = be32(prefix, 4).unwrap_or(0) as i32;
    let valid_ver = (-16..=-1).contains(&ver_le)
        || (1..=2000).contains(&(ver_le & 0xFFFF))
        || (-16..=-1).contains(&ver_be)
        || (1..=2000).contains(&(ver_be & 0xFFFF));
    require!(valid_ver);
    Some(Label::UnrealPkg)
}

/// Epic Games Unreal Engine 4.26 / 5 IoStore Table of Contents (.utoc).
pub(crate) fn verify_unreal_utoc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UNREAL_UTOC_SIZE
            && prefix.len() >= 24
            && starts(prefix, b"-==--==--==--==-")
            && (1..=12).contains(&u8_at(prefix, 16)?)
            && le32(prefix, 20) == Some(144)
    );
    Some(Label::UnrealUtoc)
}

/// Epic Games Unreal Engine 2 UZ2 Chunked Zlib Redirect Archive (`.uz2`, 2002 Epic Games).
pub(crate) fn verify_unreal_uz2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_UNREAL_UZ2_SIZE, prefix.len() >= 12);
    let clen1 = le32(prefix, 0)? as usize;
    let ulen1 = le32(prefix, 4)?;
    require!(
        (8..=65536).contains(&clen1),
        (1..=32768).contains(&ulen1),
        is_valid_zlib_header(u8_at(prefix, 8)?, u8_at(prefix, 9)?)
    );
    let off2 = 8 + clen1;
    if ulen1 == 32768 && off2 + 10 <= prefix.len() {
        let clen2 = le32(prefix, off2)?;
        let ulen2 = le32(prefix, off2 + 4)?;
        require!(
            (8..=65536).contains(&clen2),
            (1..=32768).contains(&ulen2),
            is_valid_zlib_header(u8_at(prefix, off2 + 8)?, u8_at(prefix, off2 + 9)?)
        );
    } else {
        require!(8 + (clen1 as u64) == size, u8_at(prefix, 8)? == 0x78);
    }
    Some(Label::UnrealUz2)
}

/// Valve Pak Source Engine Archive (.vpk).
pub(crate) fn verify_valve_vpk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_VALVE_VPK_SIZE && prefix.len() >= 12);
    let version = le32(prefix, 4)?;
    let tree_size = le32(prefix, 8)? as u64;
    let valid_v1 = version == 1 && 12 + tree_size <= size;
    let valid_v2 = version == 2 && size >= 28 && prefix.len() >= 28 && 28 + tree_size <= size;
    require!(valid_v1 || valid_v2);
    Some(Label::Vpk)
}

/// Atari ST Vic2 Huffman compressed stream (`"Vic2"`).
pub(crate) fn verify_vic2_huffman(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VIC2_HUFFMAN_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"Vic2"),
        has_non_ascii_text_byte(prefix.get(4..12)?)
    );
    Some(Label::Vic2Huffman)
}

/// DEC VAX/VMS Librarian Archive (`.OLB`/`.MLB`/`.HLB`/`.NLB`/`.TLB`, 1978 Digital Equipment).
///
/// Header (`LHD`) layout (`512`-byte block multiple):
/// - `@0x00`: `u8 lhd$b_type` (`1..=8`: 1=object, 2=macro, 3=help, 4=text, 5=shareable, 6=NCS, 7/8=ELF/Alpha)
/// - `@0x01`: `u8 lhd$b_nindex` (`>= 1` B-tree indices)
/// - `@0x02`: `u16 LE reserved == 0`
/// - `@0x04`: `u32 LE lhd$l_magic == 0x0DEC2581` (`81 25 EC 0D`)
/// - `@0x08`: `u32 LE lhd$l_version` (`1..=8`, typically `3`)
/// - `@0x0C`: `u8` counted ASCII creator string length (`8..=40`, e.g. `"VAX-11 Librarian V04-00"`).
pub(crate) fn verify_vms_lib(prefix: &[u8], size: u64) -> Option<Label> {
    const TYPE_OFF: usize = 0x00; // @0x00: u8 library type (1..=8)
    const NINDEX_OFF: usize = 0x01; // @0x01: u8 index count (>= 1)
    const RESERVED_OFF: usize = 0x02; // @0x02: u16 LE zero field
    const MAGIC_OFF: usize = 0x04; // @0x04: u32 LE 0x0DEC2581
    const VERSION_OFF: usize = 0x08; // @0x08: u32 LE format version (1..=8)
    const CREATOR_LEN_OFF: usize = 0x0C; // @0x0C: u8 counted ASCII string length
    const CREATOR_STR_OFF: usize = 0x0D; // @0x0D: ASCII creator banner

    require!(
        size >= MIN_VMS_LIB_SIZE,
        size.is_multiple_of(512),
        prefix.len() >= 0x38,
        eq_at(prefix, MAGIC_OFF, VMS_LIB_MAGIC),
        (1..=8).contains(&u8_at(prefix, TYPE_OFF)?),
        u8_at(prefix, NINDEX_OFF)? >= 1,
        le16(prefix, RESERVED_OFF) == Some(0),
        (1..=8).contains(&le32(prefix, VERSION_OFF)?)
    );
    let clen = u8_at(prefix, CREATOR_LEN_OFF)? as usize;
    require!(
        (8..=40).contains(&clen),
        CREATOR_STR_OFF + clen <= prefix.len(),
        get_slice(prefix, CREATOR_STR_OFF, CREATOR_STR_OFF + clen)?
            .iter()
            .all(|&b| (0x20..=0x7E).contains(&b)),
        has_non_ascii_text_byte(prefix.get(..CREATOR_STR_OFF)?)
    );
    Some(Label::VmsLib)
}

/// Blizzard Entertainment Warcraft III World Editor Map (`.w3m` / `.w3x`).
///
/// Header layout:
/// - `@0..4`: `"HM3W"` magic
/// - `@4..8`: `u32 LE` reserved (`0`)
/// - `@512..516`: `"MPQ\x1a"` embedded Blizzard MPQ archive header
pub(crate) fn verify_warcraft3_map(prefix: &[u8], size: u64) -> Option<Label> {
    const RESERVED_OFF: usize = 4; // @4..8: u32 LE unknown/reserved (0)
    const MPQ_OFF: usize = 512; // @512..516: embedded MPQ header offset
    const MPQ_MAGIC: &[u8; 4] = bytes!(b"MPQ", &[0x1A]); // @512..516: Blizzard MPQ signature

    require!(
        size >= MIN_WARCRAFT3_MAP_SIZE,
        prefix.len() >= MPQ_OFF + 4,
        starts(prefix, WARCRAFT3_MAP_MAGIC),
        le32(prefix, RESERVED_OFF)? == 0,
        eq_at(prefix, MPQ_OFF, MPQ_MAGIC)
    );
    Some(Label::Warcraft3Map)
}

/// Watcom Installer WPK Archive (`.wpk`, 1994 Watcom).
pub(crate) fn verify_watcom_wpk(prefix: &[u8], size: u64) -> Option<Label> {
    const WPK_V1_MAGIC: &[u8; 6] = b"\x03\x24\x01\x01\x01\x00"; // @0..6: Watcom WPK v1 binary header

    require!(size >= MIN_WATCOM_WPK_SIZE, prefix.len() >= 16);
    if starts(prefix, WPK_V1_MAGIC) {
        let hdr_len = le16(prefix, 6)? as u64;
        let payload_len = le32(prefix, 8)? as u64;
        require!(hdr_len >= 16, hdr_len.saturating_add(payload_len) == size);
        return Some(Label::WatcomWpk);
    }
    require!(starts(prefix, b"_WPK"), prefix.get(4..12)?.contains(&0));
    Some(Label::WatcomWpk)
}

/// W3C / Google Chrome WebBundle (`.wbn`, `"🌐📦"`) & Isolated Web App Signed WebBundle (`.swbn`, `"🖋📦"` / `"📝👑"`).
pub(crate) fn verify_webbundle(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WEBBUNDLE_SIZE,
        prefix.len() >= 15,
        matches!(u8_at(prefix, 0)?, 0x83..=0x86),
        u8_at(prefix, 1)? == 0x48
    );
    let magic8 = get_slice(prefix, 2, 10)?;
    let is_wbn = magic8 == b"\xF0\x9F\x8C\x90\xF0\x9F\x93\xA6"
        && u8_at(prefix, 10)? == 0x44
        && matches!(get_slice(prefix, 11, 15)?, b"b1\0\0" | b"b2\0\0" | b"1b\0\0" | b"2b\0\0");
    let is_swbn = (magic8 == b"\xF0\x9F\x96\x8B\xF0\x9F\x93\xA6"
        || magic8 == b"\xF0\x9F\x93\x9D\xF0\x9F\x91\x91")
        && (u8_at(prefix, 10)? != 0x44
            || matches!(get_slice(prefix, 11, 15)?, b"1b\0\0" | b"2b\0\0" | b"b1\0\0" | b"b2\0\0"));
    require!(is_wbn || is_swbn);
    Some(Label::Wbn)
}

/// Weltenbauer / Astragon Construction Simulator Persistence (`.wbp`) header (v1 and v2).
pub(crate) fn verify_weltenbauer_wbp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WELTENBAUER_WBP_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"\x89wbp\r\n\x1a\n")
    );
    let ver = le32(prefix, 8)?;
    require!((1..=16).contains(&ver), eq_at(prefix, 42, b"LIST") || eq_at(prefix, 34, b"LIST"));
    Some(Label::WeltenbauerWbp)
}

/// Westwood Studios / EA SAGE Engine Compiled String Table (.csf, 2000) Westwood Studios.
pub(crate) fn verify_westwood_csf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WESTWOOD_CSF_SIZE,
        prefix.len() >= 32,
        starts(prefix, b" FSC"),
        (2..=4).contains(&le32(prefix, 4).unwrap_or(0)),
        le32(prefix, 8).unwrap_or(0) > 0,
        get_slice(prefix, 24, 28)? == b" LBL"
    );
    Some(Label::WestwoodCsf)
}

/// Microsoft Windows CE Installation Cabinet Header (`"MSCE\0\0\0\0"`).
pub(crate) fn verify_wince_cab_header(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WINCE_CAB_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"MSCE\0\0\0\0"),
        le32(prefix, 8)? >= 100
    );
    Some(Label::WinceMsce)
}

/// CD Projekt RED REDengine 2 / The Witcher 2 DZIP Archive (`.dzip`).
///
/// Header layout:
/// - `@0..4`: `"DZIP"` magic
/// - `@4..8`: `u32 LE` `version` (`1..=2`)
/// - `@8..12`: `u32 LE` `entry_count` (`1..=1_000_000`)
pub(crate) fn verify_witcher2_dzip(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4..8: u32 LE archive format version (1 or 2)
    const ENTRY_COUNT_OFF: usize = 8; // @8..12: u32 LE packed file count

    require!(
        size >= MIN_WITCHER2_DZIP_SIZE,
        prefix.len() >= 16,
        starts(prefix, WITCHER2_DZIP_MAGIC),
        (1..=2).contains(&le32(prefix, VERSION_OFF)?),
        (1..=1_000_000).contains(&le32(prefix, ENTRY_COUNT_OFF)?)
    );
    Some(Label::Witcher2Dzip)
}

/// CD Projekt RED REDengine 3 / The Witcher 3 Asset Bundle (`.bundle`).
///
/// Header layout:
/// - `@0..8`: `"POTATO70"` magic
/// - `@8..12`: `u32 LE` `total_size` (`>= 32`)
/// - `@12..16`: `u32 LE` `dummy_size` (`0`)
/// - `@16..20`: `u32 LE` `toc_size` (`>= 32`, `< file_size`)
pub(crate) fn verify_witcher3_bundle(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 8] = b"POTATO70"; // @0..8: REDengine 3 bundle magic
    const TOTAL_SIZE_OFF: usize = 8; // @8..12: u32 LE declared bundle size
    const DUMMY_OFF: usize = 12; // @12..16: u32 LE reserved zero
    const TOC_SIZE_OFF: usize = 16; // @16..20: u32 LE table-of-contents byte length

    require!(
        size >= MIN_WITCHER3_BUNDLE_SIZE,
        prefix.len() >= 32,
        starts(prefix, MAGIC),
        le32(prefix, TOTAL_SIZE_OFF)? >= 32,
        le32(prefix, DUMMY_OFF)? == 0
    );
    let toc_size = le32(prefix, TOC_SIZE_OFF)? as u64;
    require!(toc_size >= 32, toc_size < size);
    Some(Label::Witcher3Bundle)
}

/// WOLF RPG Editor Map File (`.mps`, `00*10` + `"WOLFM\0"`).
pub(crate) fn verify_wolfrpg_map_mps(prefix: &[u8], size: u64) -> Option<Label> {
    const WOLFRPG_MAP_MAGIC: &[u8; 16] =
        hex2bin!("00 00 00 00 00 00 00 00 00 00 57 4F 4C 46 4D 00"); // @0..16

    require!(
        size >= MIN_WOLFRPG_MAP_MPS_SIZE,
        prefix.len() >= 16,
        starts(prefix, WOLFRPG_MAP_MAGIC)
    );
    Some(Label::WolfrpgMapMps)
}

/// BDES Encrypted Package (.xzp).
pub(crate) fn verify_xbox_bdes_xzp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_XBOX_BDES_XZP_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"BDES\0\0\0\x01"),
        has_non_ascii_text_byte(prefix.get(8..28)?)
    );
    Some(Label::XboxBdesXzp)
}

/// XCR Archive (`.xcr`, `"xcr File 1.00\0"` or `"xcr\xBC"`).
pub(crate) fn verify_xcr_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_XCR_ARCHIVE_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"xcr File 1.")
            || (starts(prefix, b"xcr\xBC") && prefix.get(4..16)?.contains(&0))
    );
    Some(Label::XcrArchive)
}

/// Windows Phone XNA 711x7.0 Course Package (.crs_).
pub(crate) fn verify_xna_course_crs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_XNA_COURSE_CRS_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 3)? == hex2bin!("37 31 31"),
        get_slice(prefix, 4, 28)?
            == hex2bin!("37 2E 30 2E 7D DD E8 68 76 76 EB CD B7 06 8A 92 A8 99 06 B2 40 B6 94 8E")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::XnaCourseCrs)
}

/// Microsoft XNA Game Studio / MonoGame Compiled Binary Asset (.xnb).
pub(crate) fn verify_xna_xnb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_XNA_XNB_SIZE && prefix.len() >= 10);
    let version = u8_at(prefix, 4)?;
    let flags = u8_at(prefix, 5)?;
    let declared_size = le32(prefix, 6)? as u64;
    require!((1..=6).contains(&version) && (flags & 0x3E) == 0 && declared_size == size);
    Some(Label::Xnb)
}

/// Amiga XPK: `"XPKF"`, `u32 BE` crunched length, 4-character sub-packer ID (`NUKE`, `SQSH`, `SHRI`, `RLEN`, ...),
/// `u32 BE` uncompressed length.
pub(crate) fn verify_xpk(prefix: &[u8], size: u64) -> Option<Label> {
    const CRUNCHED_LEN: usize = 4; // @4: u32 BE
    const PACKER_ID: core::ops::Range<usize> = 8..12; // @8: sub-library FourCC
    const UNPACKED_LEN: usize = 12; // @12: u32 BE
    require!(size >= MIN_XPK_SIZE, prefix.len() >= 16, starts(prefix, XPK_MAGIC));
    require!(be32(prefix, CRUNCHED_LEN)? >= 28, be32(prefix, UNPACKED_LEN)? > 0);
    require!(
        get_slice(prefix, PACKER_ID.start, PACKER_ID.end)?
            .iter()
            .all(|&b| b.is_ascii_uppercase() || b.is_ascii_digit()),
        prefix.get(4..8)?.contains(&0) || prefix.get(12..16)?.contains(&0)
    );
    Some(Label::Xpk)
}

/// XPLAPP Application Package (.xplapp).
pub(crate) fn verify_xplapp_package(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_XPLAPP_PACKAGE_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"XPLAPP\x01\0\0\0\0\0"),
        get_slice(prefix, 34, 40)? == b"LOCALE"
    );
    Some(Label::XplappPackage)
}

/// YU-RIS Visual Novel Engine YPF Archive (.ypf, 2005) Firstia / YU-RIS.
pub(crate) fn verify_yuris_ypf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_YURIS_YPF_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"YPF\0"),
        (100..=2000).contains(&le32(prefix, 4).unwrap_or(0)),
        (1..=100_000).contains(&le32(prefix, 8).unwrap_or(0)),
        get_slice(prefix, 16, 32)? == [0u8; 16]
    );
    Some(Label::YurisYpf)
}

/// DeepFreezer YZ1 Archive (1999) Yamazaki (`"yz1\x01"`, `"yz010600"`, `"yz020..."`).
pub(crate) fn verify_yz1_archive(prefix: &[u8], size: u64) -> Option<Label> {
    let valid = starts(prefix, b"yz1\x01")
        || (prefix.len() >= 16
            && (starts(prefix, b"yz010") || starts(prefix, b"yz020"))
            && get_slice(prefix, 5, 8)?.iter().all(|b| b.is_ascii_digit())
            && has_non_ascii_text_byte(prefix.get(..16)?));
    (size >= MIN_YZ1_ARCHIVE_SIZE && valid).then_some(Label::Yz1)
}

/// zchunk Delta-Compressed Container (`.zck`, `\0ZCK1`).
pub(crate) fn verify_zchunk_container(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ZCHUNK_SIZE, prefix.len() >= 12, starts(prefix, b"\0ZCK1"));
    Some(Label::ZchunkZck)
}

/// ZEDAL Library Timestamped Binary Export (.lv).
pub(crate) fn verify_zedal_library_lv(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ZEDAL_LIBRARY_LV_SIZE,
        prefix.len() >= 44,
        get_slice(prefix, 0, 9)? == hex2bin!("4C 69 62 72 61 72 79 0A 62"),
        get_slice(prefix, 24, 25)? == hex2bin!("36")
    );
    require!(has_non_ascii_text_byte(prefix.get(..44)?));
    Some(Label::ZedalLibraryLv)
}

/// openZIM Offline Wiki Archive (`.zim`, `"ZIM\x04"`).
pub(crate) fn verify_zim_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ZIM_ARCHIVE_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"ZIM\x04"),
        matches!(le16(prefix, 4), Some(5 | 6)),
        le32(prefix, 24)? > 0,
        le64(prefix, 32)? >= 80
    );
    Some(Label::OpenzimZim)
}

/// H. Peter Anvin `zisofs` Compressed File (`.zf`, 2001 H. Peter Anvin, `37 E4 53 96 C9 DB D6 07`).
pub(crate) fn verify_zisofs_compressed(prefix: &[u8], size: u64) -> Option<Label> {
    const ZISOFS_MAGIC: &[u8; 8] = hex2bin!("37 E4 53 96 C9 DB D6 07"); // @0..8
    require!(
        size >= MIN_ZISOFS_FILE_SIZE,
        prefix.len() >= 16,
        starts(prefix, ZISOFS_MAGIC),
        u8_at(prefix, 12) == Some(4),
        (15..=17).contains(&u8_at(prefix, 13)?),
        le16(prefix, 14) == Some(0)
    );
    Some(Label::ZisofsFile)
}

/// Rahul Dhesi ZOO archive (`.zoo`).
pub(crate) fn verify_zoo(prefix: &[u8], size: u64) -> Option<Label> {
    const ZOO_TAG: &[u8; 4] = &[0xDC, 0xA7, 0xC4, 0xFD];
    require!(
        size >= 24,
        prefix.len() >= 24,
        eq_at(prefix, 20, ZOO_TAG) || starts(prefix, b"\xFE\x07\x01"),
    );
    Some(Label::Zoo)
}

/// IBM z/OS USS EBCDIC Archiver Library (!<arch>).
pub(crate) fn verify_zos_ebcdic_ar(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC_IBM1047_PREFIX: &[u8] = hex2bin!("5A 4C 81 99 83 88 6E");
    const MAGIC_IBM037_PREFIX: &[u8] = hex2bin!("4F 4C 81 99 83 88 6E");
    require!(
        size >= MIN_ZOS_EBCDIC_AR_SIZE,
        prefix.len() >= 68,
        starts(prefix, MAGIC_IBM1047_PREFIX) || starts(prefix, MAGIC_IBM037_PREFIX),
        matches!(u8_at(prefix, 7)?, 0x15 | 0x25),
        matches!(u8_at(prefix, 66)?, 0x60 | 0x79 | 0x40),
        u8_at(prefix, 67)? == u8_at(prefix, 7)?
    );
    // Member size field at @56..66 in EBCDIC digits (0xF0..=0xF9) or spaces (0x40)
    require!(
        (0xF0..=0xF9).contains(&u8_at(prefix, 56)?),
        get_slice(prefix, 56, 66)?.iter().all(|&b| (0xF0..=0xF9).contains(&b) || b == 0x40)
    );
    Some(Label::ZosAr)
}

/// zp1 Compressed Data Package (.zp).
pub(crate) fn verify_zp1_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ZP1_ARCHIVE_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"zp1\0"),
        le32(prefix, 4).unwrap_or(0) > 0,
        get_slice(prefix, 8, 12)? == b"\0\0\x04\0",
        (1..=1024).contains(&le32(prefix, 12).unwrap_or(0))
    );
    Some(Label::Zp1Archive)
}

/// ZPAQ Journaling Archiver (2009) Matt Mahoney (`"zPaQ"` / `"7kSt"`).
pub(crate) fn verify_zpaq(prefix: &[u8], size: u64) -> Option<Label> {
    (size >= MIN_ZPAQ_SIZE
        && prefix.len() >= 6
        && ((starts(prefix, b"zPaQ") && matches!(u8_at(prefix, 4), Some(1 | 2)))
            || (starts(prefix, b"zPQ")
                && matches!(u8_at(prefix, 3), Some(1 | 2))
                && u8_at(prefix, 4) == Some(1))
            || (starts(prefix, b"7kSt")
                && (prefix.len() < 17 || eq_at(prefix, 13, b"zPQ") || eq_at(prefix, 13, b"zPaQ")))))
    .then_some(Label::Zpaq)
}

/// Google Chrome Zucchini Executable Differential Patch (Zuc\\\\0).
pub(crate) fn verify_zucchini_patch(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ZUCCHINI_PATCH_SIZE && prefix.len() >= 24);
    let major = le16(prefix, 4)?;
    let old_size = le32(prefix, 8)?;
    let new_size = le32(prefix, 16)?;
    require!((1..=4).contains(&major) && (old_size > 0 || new_size > 0));
    Some(Label::ZucchiniPatch)
}

/// Symbian OS v6-v8 / EPOC R3-R5 SIS installation package (`UID2` @4 + `UID3` 0x10000419 @8).
pub(crate) fn verify_symbian_sis_offset(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 68,
        prefix.len() >= 24,
        le32(prefix, 8)? == 0x1000_0419,
        matches!(le32(prefix, 4)?, 0x1000_3A12 | 0x1000_006D),
    );
    Some(Label::Sisx)
}
