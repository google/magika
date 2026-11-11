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

//! Firmware, ROM, and microcontroller format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// Nintendo 3DS homebrew executable (3DSX): magic `"3DSX"`, header size (32 base or 44 extended),
/// relocation header size 8, version 0, flags 0, and 4-byte-aligned segment sizes.
pub(crate) fn threedsx(prefix: &[u8]) -> Option<Label> {
    const TAG: &[u8] = b"3DSX"; // @0..4: format signature
    const BASE_HDR: &[u8] = bytes!(TAG, &32u16.to_le_bytes(), &8u16.to_le_bytes(), &[0u8; 8]); // @0..16: 32-byte header
    const EXT_HDR: &[u8] = bytes!(TAG, &44u16.to_le_bytes(), &8u16.to_le_bytes(), &[0u8; 8]); // @0..16: 44-byte extended header
    const BASE_MIN_LEN: usize = 56; // base header (32) + 3 relocation headers (24)
    const EXT_MIN_LEN: usize = 68; // extended header (44) + 3 relocation headers (24)
    const CODE_SEG_SIZE: usize = 16; // @16: u32 LE — code segment size (nonzero, 4-byte aligned)
    const RODATA_SEG_SIZE: usize = 20; // @20: u32 LE — rodata segment size (4-byte aligned)
    const DATA_BSS_SIZE: usize = 24; // @24: u32 LE — data+bss segment size (4-byte aligned)
    const BSS_SIZE: usize = 28; // @28: u32 LE — bss size (4-byte aligned)

    let has_header = (starts(prefix, BASE_HDR) && prefix.len() >= BASE_MIN_LEN)
        || (starts(prefix, EXT_HDR) && prefix.len() >= EXT_MIN_LEN);
    require!(has_header);
    let code = le32(prefix, CODE_SEG_SIZE)?;
    require!(
        code >= 4,
        code % 4 == 0,
        le32(prefix, RODATA_SEG_SIZE)? % 4 == 0,
        le32(prefix, DATA_BSS_SIZE)? % 4 == 0,
        le32(prefix, BSS_SIZE)? % 4 == 0,
    );
    Some(Label::_3dsx)
}

/// Microsoft USB Flashing Format (UF2): 512-byte block with starting magics `"UF2\n"` +
/// `0x9E5D5157` LE, payload size ≤ 476 at `@16`, and final magic `0x0AB16F30` LE at `@508`.
pub(crate) fn uf2(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"UF2\n", &0x9E5D_5157u32.to_le_bytes()); // @0..8: magicStart0 + magicStart1
    const PAYLOAD_SIZE: usize = 16; // @16: u32 LE — payloadSize (≤ 476)
    const MAGIC_END_OFFSET: usize = 508; // @508: u32 LE — magicEnd (0x0AB16F30)
    const MAGIC_END: u32 = 0x0AB1_6F30;
    const BLOCK_LEN: usize = 512;

    require!(
        prefix.len() >= BLOCK_LEN,
        starts(prefix, HEADER),
        le32(prefix, PAYLOAD_SIZE)? <= 476,
        le32(prefix, MAGIC_END_OFFSET)? == MAGIC_END,
    );
    Some(Label::Uf2)
}

/// Palm OS PRC/PDB application database: NUL terminator at `@31`, type `"appl"` at `@60`, four
/// printable ASCII creator bytes at `@64..68`, and nonzero record count at `@76`.
pub(crate) fn palmos(prefix: &[u8]) -> Option<Label> {
    const NAME_NUL: usize = 31; // @31: u8 — NUL terminator of 32-byte database name
    const DB_TYPE: usize = 60; // @60..64: "appl" database type
    const APPL_TYPE: &[u8] = b"appl";
    const CREATOR: usize = 64; // @64..68: 4-byte printable ASCII creator ID
    const NUM_RECORDS: usize = 76; // @76: u16 BE — number of records/resources (≥ 1)
    const MIN_LEN: usize = 78;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, DB_TYPE, APPL_TYPE),
        u8_at(prefix, NAME_NUL)? == 0,
        be16(prefix, NUM_RECORDS)? >= 1,
        get_slice(prefix, CREATOR, CREATOR + 4)?.iter().all(|&b| (0x20..=0x7E).contains(&b)),
    );
    Some(Label::Palmos)
}

/// Intel HEX object file: two consecutive leading records `:[0-9A-F]{6}0[0-5][0-9A-F]{2,512}\r?\n`
/// and `:[0-9A-F]{6}0[0-5]`.
pub(crate) fn intelhex(prefix: &[u8]) -> Option<Label> {
    #[inline]
    fn is_upper_hex(b: u8) -> bool {
        matches!(b, b'0'..=b'9' | b'A'..=b'F')
    }

    #[inline]
    fn record_head(s: &[u8], at: usize) -> bool {
        s.len() >= at + 9
            && s[at] == b':'
            && s[at + 1..at + 7].iter().copied().all(is_upper_hex)
            && s[at + 7] == b'0'
            && matches!(s[at + 8], b'0'..=b'5')
    }

    require!(record_head(prefix, 0));
    let mut i = 9;
    while i < prefix.len() && is_upper_hex(prefix[i]) {
        i += 1;
    }
    let hex_digits = i - 9;
    require!((2..=512).contains(&hex_digits));
    if u8_at(prefix, i) == Some(b'\r') {
        i += 1;
    }
    require!(u8_at(prefix, i) == Some(b'\n'));
    i += 1;
    require!(record_head(prefix, i));
    let _ = get_slice(prefix, 0, i + 9);
    Some(Label::Intelhex)
}

/// Motorola S-record hexadecimal firmware (`srecord`, 1975): `"S0"`..`"S3"` record header, hex byte
/// count `3..=64`, and verified ones'-complement record checksum.
pub(crate) fn srecord(prefix: &[u8], size: u64) -> Option<Label> {
    const SREC_S0: &[u8] = b"S0";
    const SREC_S1: &[u8] = b"S1";
    const SREC_S2: &[u8] = b"S2";
    const SREC_S3: &[u8] = b"S3";

    require!(
        size >= 10,
        prefix.len() >= 10,
        matches!(&prefix[0..2], SREC_S0 | SREC_S1 | SREC_S2 | SREC_S3),
        prefix[2..10].iter().all(|&b| b.is_ascii_hexdigit()),
    );
    let hex_byte = |off: usize| -> Option<u8> {
        let hi = (prefix[off] as char).to_digit(16)? as u8;
        let lo = (prefix[off + 1] as char).to_digit(16)? as u8;
        Some((hi << 4) | lo)
    };
    let count = hex_byte(2)? as usize;
    let line_end = 4 + count * 2;
    require!(
        (3..=64).contains(&count),
        prefix.len() > line_end,
        prefix[2..line_end].iter().all(|&b| b.is_ascii_hexdigit()),
        matches!(prefix[line_end], b'\r' | b'\n'),
    );
    let mut sum = count as u8;
    for idx in (4..line_end - 2).step_by(2) {
        sum = sum.wrapping_add(hex_byte(idx)?);
    }
    let cksum = hex_byte(line_end - 2)?;
    require!(sum ^ 0xFF == cksum);
    let _ = get_slice(prefix, 0, line_end + 1);
    Some(Label::Srecord)
}

/// Nintendo 3DS CTR Card Image / Content Container (`3ds`, 2011): `"NCSD"` or `"NCCH"` signature at
/// `@0x100..0x104` (after the 256-byte RSA-2048 signature) and non-zero media-unit size at `@0x104`.
pub(crate) fn nintendo_3ds(prefix: &[u8], size: u64) -> Option<Label> {
    const N3DS_NCSD_MAGIC: &[u8; 4] = b"NCSD";
    const N3DS_NCCH_MAGIC: &[u8; 4] = b"NCCH";

    require!(
        size >= 0x200,
        prefix.len() >= 0x110,
        eq_at(prefix, 0x100, N3DS_NCSD_MAGIC) || eq_at(prefix, 0x100, N3DS_NCCH_MAGIC),
        (1..0x0800_0000).contains(&le32(prefix, 0x104)?),
    );
    Some(Label::_3ds)
}

/// UEFI Platform Initialization Firmware Volume (`uefi_fv`, 2006): 16-byte zero vector at `@0..16`
/// and `"_FVH"` signature (`EFI_FVH_SIGNATURE`) at `@40..44`.
pub(crate) fn uefi_fv(prefix: &[u8], size: u64) -> Option<Label> {
    const UEFI_FVH_SIG: &[u8; 4] = b"_FVH";
    const UEFI_ZERO_VECTOR: &[u8; 16] = &[0u8; 16];

    require!(
        size >= 64,
        prefix.len() >= 48,
        eq_at(prefix, 40, UEFI_FVH_SIG),
        starts(prefix, UEFI_ZERO_VECTOR),
    );
    Some(Label::UefiFv)
}
