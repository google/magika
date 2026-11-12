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

//! Virtual-machine and language bytecode format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// Dalvik Executable (DEX): `"dex\n"` followed by `"[0-9]{3}\x00"`, matching header size
/// (112 standard, 120 for v041 container) and `ENDIAN_CONSTANT` (`0x12345678`) in either byte order.
pub(crate) fn dex(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"dex\n"; // @0..4: DEX magic prefix
    const VERSION: usize = 4; // @4..8: 3 ASCII digits + NUL
    const VER_041: u32 = tag_u32(b"041\0"); // DEX v041 container version
    const HEADER_SIZE: usize = 36; // @36: u32 — header size (112 or 120)
    const ENDIAN_TAG: usize = 40; // @40: u32 — endianness constant
    const ENDIAN_CONSTANT: u32 = 0x1234_5678; // standard DEX endian marker
    const STD_HDR_LEN: usize = 112;
    const V041_HDR_LEN: usize = 120;

    require!(
        prefix.len() >= STD_HDR_LEN,
        starts(prefix, MAGIC),
        get_slice(prefix, 4, 7)?.iter().all(u8::is_ascii_digit),
        u8_at(prefix, 7)? == 0,
    );
    static DEX_STANDARD: Subformat = Subformat::new("Dalvik Executable (standard)");
    static DEX_041_CONTAINER: Subformat = Subformat::new("Dalvik Executable (v041 container)");
    let expected_hdr = if be32(prefix, VERSION)? == VER_041 {
        require!(prefix.len() >= V041_HDR_LEN);
        V041_HDR_LEN as u32
    } else {
        STD_HDR_LEN as u32
    };
    let valid_endian = (le32(prefix, HEADER_SIZE)? == expected_hdr
        && le32(prefix, ENDIAN_TAG)? == ENDIAN_CONSTANT)
        || (be32(prefix, HEADER_SIZE)? == expected_hdr
            && be32(prefix, ENDIAN_TAG)? == ENDIAN_CONSTANT);
    require!(valid_endian);
    let sub = if expected_hdr == V041_HDR_LEN as u32 { &DEX_041_CONTAINER } else { &DEX_STANDARD };
    with_subformat(Label::Dex, sub)
}

/// LLVM bitcode: 4-byte-aligned file size and either a raw bitcode stream (`"BC\xc0\xde"` with
/// non-zero initial 2-bit abbrev ID) or a Darwin/Clang bitcode wrapper (`0x0B17C0DE` LE).
pub(crate) fn llvm_bitcode(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    const RAW_MAGIC: &[u8] = b"BC\xc0\xde"; // @0..4: raw LLVM bitcode stream magic
    const WRAPPER_MAGIC: &[u8] = &0x0B17_C0DEu32.to_le_bytes(); // @0..4: LLVM bitcode wrapper magic
    const FIRST_BYTE: usize = 4; // @4: low 2 bits are initial abbrev ID (must not be END_BLOCK = 0)
    const WRAP_OFFSET: usize = 8; // @8: u32 LE — bitcode offset (≥ 16)
    const WRAP_SIZE: usize = 12; // @12: u32 LE — bitcode size (≥ 4)

    require!(ctx.size.is_multiple_of(4));
    let raw =
        prefix.len() >= 8 && starts(prefix, RAW_MAGIC) && (u8_at(prefix, FIRST_BYTE)? & 0b11) != 0;
    let wrapper = prefix.len() >= 20
        && starts(prefix, WRAPPER_MAGIC)
        && le32(prefix, WRAP_OFFSET)? >= 16
        && le32(prefix, WRAP_SIZE)? >= 4;
    require!(raw || wrapper);
    static LLVM_RAW: Subformat = Subformat::new("LLVM bitcode (raw)");
    static LLVM_WRAPPER: Subformat = Subformat::new("LLVM bitcode wrapper");
    with_subformat(Label::LlvmBitcode, if raw { &LLVM_RAW } else { &LLVM_WRAPPER })
}

/// Lua (v2.4..v5.5) and LuaJIT (v1/v2) compiled bytecode chunks.
pub(crate) fn luabytecode(prefix: &[u8]) -> Option<Label> {
    const LUA_MAGIC: &[u8] = b"\x1bLua"; // @0..4: standard Lua bytecode signature
    const LUAJIT_MAGIC: &[u8] = b"\x1bLJ"; // @0..3: LuaJIT bytecode dump signature
    const LUAC_DATA: &[u8] = b"\x19\x93\r\n\x1a\n"; // Lua 5.2+ conversion-check tail
    const MIN_LEN: usize = 8;
    static LUA_2_4: Subformat = Subformat::new("Lua 2.4 bytecode");
    static LUA_2_5: Subformat = Subformat::new("Lua 2.5 bytecode");
    static LUA_3_1: Subformat = Subformat::new("Lua 3.1 bytecode");
    static LUA_3_2: Subformat = Subformat::new("Lua 3.2 bytecode");
    static LUA_4_0: Subformat = Subformat::new("Lua 4.0 bytecode");
    static LUA_5_0: Subformat = Subformat::new("Lua 5.0 bytecode");
    static LUA_5_1: Subformat = Subformat::new("Lua 5.1 bytecode");
    static LUA_5_2: Subformat = Subformat::new("Lua 5.2 bytecode");
    static LUA_5_3: Subformat = Subformat::new("Lua 5.3 bytecode");
    static LUA_5_4: Subformat = Subformat::new("Lua 5.4 bytecode");
    static LUA_5_5: Subformat = Subformat::new("Lua 5.5 bytecode");
    static LUAJIT_2_0: Subformat = Subformat::new("LuaJIT 2.0 bytecode");
    static LUAJIT_2_1: Subformat = Subformat::new("LuaJIT 2.1 bytecode");

    require!(prefix.len() >= MIN_LEN);
    if starts(prefix, LUA_MAGIC) {
        let ver = u8_at(prefix, 4)?;
        let ok = match ver {
            0x23 => prefix.len() >= 11 && matches!(be16(prefix, 5)?, 0x1234 | 0x3412),
            0x25 => {
                prefix.len() >= 14
                    && u8_at(prefix, 5)? == 2
                    && u8_at(prefix, 6)? == 4
                    && u8_at(prefix, 7)? >= 1
                    && matches!(be16(prefix, 8)?, 0x1234 | 0x3412)
            }
            0x31 => {
                matches!(u8_at(prefix, 5)?, 0x6C | 0x66 | 0x64 | 0x3F) && u8_at(prefix, 6)? >= 1
            }
            0x32 => true,
            0x40 => {
                prefix.len() >= 14
                    && u8_at(prefix, 5)? <= 1
                    && get_slice(prefix, 6, 13).is_some_and(|s| s.iter().all(|&b| b >= 1))
            }
            0x50 => {
                prefix.len() >= 15
                    && u8_at(prefix, 5)? <= 1
                    && get_slice(prefix, 6, 14).is_some_and(|s| s.iter().all(|&b| b >= 1))
            }
            0x51 => {
                prefix.len() >= 12
                    && u8_at(prefix, 5)? == 0
                    && u8_at(prefix, 6)? <= 1
                    && get_slice(prefix, 7, 11).is_some_and(|s| s.iter().all(|&b| b >= 1))
                    && matches!(
                        u8_at(prefix, 11)?,
                        0x00 | 0x01 | 0x02 | 0x04 | 0x08 | 0x82 | 0x84 | 0x88
                    )
            }
            0x52 => {
                prefix.len() >= 18
                    && u8_at(prefix, 5)? == 0
                    && u8_at(prefix, 6)? <= 1
                    && prefix[7..11].iter().all(|&b| b >= 1)
                    && matches!(
                        u8_at(prefix, 11)?,
                        0x00 | 0x01 | 0x02 | 0x04 | 0x08 | 0x82 | 0x84 | 0x88
                    )
                    && eq_at(prefix, 12, LUAC_DATA)
            }
            0x53 => {
                prefix.len() >= 17
                    && u8_at(prefix, 5)? == 0
                    && eq_at(prefix, 6, LUAC_DATA)
                    && get_slice(prefix, 12, 17).is_some_and(|s| s.iter().all(|&b| b >= 1))
            }
            0x54 => {
                prefix.len() >= 15
                    && u8_at(prefix, 5)? == 0
                    && eq_at(prefix, 6, LUAC_DATA)
                    && get_slice(prefix, 12, 15).is_some_and(|s| s.iter().all(|&b| b >= 1))
            }
            0x55 => {
                prefix.len() >= 13
                    && u8_at(prefix, 5)? == 0
                    && eq_at(prefix, 6, LUAC_DATA)
                    && u8_at(prefix, 12)? >= 1
            }
            _ => false,
        };
        require!(ok);
        let sub = match ver {
            0x23 => &LUA_2_4,
            0x25 => &LUA_2_5,
            0x31 => &LUA_3_1,
            0x32 => &LUA_3_2,
            0x40 => &LUA_4_0,
            0x50 => &LUA_5_0,
            0x51 => &LUA_5_1,
            0x52 => &LUA_5_2,
            0x53 => &LUA_5_3,
            0x54 => &LUA_5_4,
            _ => &LUA_5_5,
        };
        return with_subformat(Label::Luabytecode, sub);
    }

    require!(starts(prefix, LUAJIT_MAGIC), matches!(u8_at(prefix, 3)?, 1 | 2));
    let flags = u8_at(prefix, 4)?;
    require!(flags <= 0x0F);
    let b5 = u8_at(prefix, 5)?;
    if (flags & 0x02) != 0 {
        // Stripped LuaJIT dump: ULEB128 proto length (1..=3 bytes) + proto flags/params/framesize
        let after = if (0x08..=0x7F).contains(&b5) {
            6
        } else if b5 >= 0x80 {
            let b6 = u8_at(prefix, 6)?;
            if (0x01..=0x7F).contains(&b6) {
                7
            } else if b6 >= 0x80 && (0x01..=0x7F).contains(&u8_at(prefix, 7)?) {
                8
            } else {
                return None;
            }
        } else {
            return None;
        };
        require!(
            u8_at(prefix, after)? <= 0x1F,
            u8_at(prefix, after + 1).is_some(),
            matches!(u8_at(prefix, after + 2)?, 0x01..=0xFA),
        );
    } else {
        // Named LuaJIT dump: ULEB128 name length (1..=2 bytes) + '@' or '=' + printable ASCII
        let after = if (0x02..=0x7F).contains(&b5) {
            6
        } else if b5 >= 0x80 && (0x01..=0x7F).contains(&u8_at(prefix, 6)?) {
            7
        } else {
            return None;
        };
        require!(
            matches!(u8_at(prefix, after)?, b'@' | b'='),
            matches!(u8_at(prefix, after + 1)?, 0x20..=0x7E),
        );
    }
    let sub = if u8_at(prefix, 3)? == 1 { &LUAJIT_2_0 } else { &LUAJIT_2_1 };
    with_subformat(Label::Luabytecode, sub)
}

/// WebAssembly binary module: `"\0asm"` magic followed by version 1 (`0x00000001` LE).
pub(crate) fn wasm(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"\0asm", &1u32.to_le_bytes()); // @0..8: WASM magic + version 1

    require!(starts(prefix, HEADER));
    Some(Label::Wasm)
}

/// CPython bytecode (`.pyc`): one of the 89 historical magic values at `@0..4` paired with the
/// marshal `TYPE_CODE` tag (`'C'` at `@8` for Python 1.0..1.2, `'c'` at `@8` for 1.5..3.2, `'c'`
/// at `@12` for 3.3..3.3a, or `'c'`/`0xE3` at `@12` for 3.4..3.6).
pub(crate) fn pythonbytecode(prefix: &[u8]) -> Option<Label> {
    const MIN_LEN: usize = 9;
    static PY_1_0: Subformat = Subformat::new("Python 1.0-1.2 bytecode");
    static PY_1_3: Subformat = Subformat::new("Python 1.3-3.2 bytecode");
    static PY_3_3: Subformat = Subformat::new("Python 3.3 bytecode");
    static PY_3_4: Subformat = Subformat::new("Python 3.4-3.6 bytecode");

    require!(prefix.len() >= MIN_LEN);
    // All 89 magics end in either 0x99 0x00 (Python 1.0..1.2) or 0x0D 0x0A ("\r\n").
    let marker = (u8_at(prefix, 2)?, u8_at(prefix, 3)?);
    if marker == (0x99, 0x00) {
        require!(
            u8_at(prefix, 1)? == 0x09,
            matches!(u8_at(prefix, 0)?, 0x02 | 0x03),
            u8_at(prefix, 8)? == 0x43,
        );
        return with_subformat(Label::Pythonbytecode, &PY_1_0);
    }
    require!(marker == (0x0D, 0x0A));
    let ver = le16(prefix, 0)?;

    // 59 timestamp-only magics (marshal 'c' = 0x63 at @8)
    const TIMESTAMP_VERSIONS: &[u16] = &[
        0xF303, 0x1704, 0xF304, 0x0C09, 0xF30A, 0x0C13, 0x0C1D, 0x0C1F, 0x0C27, 0xEB2A, 0xEB2B,
        0xED2D, 0xED2E, 0x0C3B, 0xF23B, 0xF23C, 0x0C45, 0xF245, 0x0C4F, 0x0C58, 0xF259, 0x0C62,
        0xF263, 0x0C6C, 0xF26D, 0xF26E, 0x0C76, 0xF277, 0x0C80, 0xF281, 0xC687, 0xC688, 0x2E89,
        0xF28B, 0xF28C, 0xF295, 0x4E99, 0xF29F, 0xF2A9, 0xF2B3, 0xF2B4, 0x0BB8, 0x0BC2, 0xF2C7,
        0x0BCC, 0xF2D1, 0xF2D2, 0x0BD6, 0xF2DB, 0x0BE0, 0xF2E5, 0x0BEA, 0xF2EF, 0x0BF4, 0x0BF5,
        0xF2F9, 0xC4FC, 0xC4FD, 0x0BFF,
    ];
    if u8_at(prefix, 8) == Some(0x63) && TIMESTAMP_VERSIONS.contains(&ver) {
        return with_subformat(Label::Pythonbytecode, &PY_1_3);
    }

    // 3 sized-header magics (marshal 'c' = 0x63 at @12)
    if matches!(ver, 0x0C8A | 0x0C94 | 0x0C9E) && u8_at(prefix, 12) == Some(0x63) {
        return with_subformat(Label::Pythonbytecode, &PY_3_3);
    }

    // 25 reference-enabled magics (marshal 'c' = 0x63 or FLAG_REF|'c' = 0xE3 at @12)
    const REFERENCE_VERSIONS: &[u16] = &[
        0x0D02, 0x0D0C, 0x0D16, 0x0D17, 0x0D20, 0x0D21, 0x0D2A, 0x0D2B, 0x0D2C, 0x0D2D, 0x0D2F,
        0x0D30, 0x0D31, 0x0D32, 0x0D33, 0x0D3E, 0x0D3F, 0x0CB2, 0x0CBC, 0x0CC6, 0x0CD0, 0x0CDA,
        0x0CE4, 0x0CEE, 0x0CF8,
    ];
    require!(matches!(u8_at(prefix, 12), Some(0x63 | 0xE3)), REFERENCE_VERSIONS.contains(&ver),);
    with_subformat(Label::Pythonbytecode, &PY_3_4)
}

/// Android Optimized Dalvik Executable (`odex`, 2008): `"dey\n035\0"`, `"dey\n036\0"`, or
/// `"dey\n037\0"` header magic at `@0..8` and ≥ 40-byte `DexOptHeader`.
pub(crate) fn odex(prefix: &[u8], size: u64) -> Option<Label> {
    const ODEX_035_MAGIC: &[u8; 8] = b"dey\n035\0";
    const ODEX_036_MAGIC: &[u8; 8] = b"dey\n036\0";
    const ODEX_037_MAGIC: &[u8; 8] = b"dey\n037\0";

    require!(
        size >= 40,
        prefix.len() >= 40,
        starts(prefix, ODEX_035_MAGIC)
            || starts(prefix, ODEX_036_MAGIC)
            || starts(prefix, ODEX_037_MAGIC),
    );
    static ODEX_035: Subformat = Subformat::new("Optimized DEX 035");
    static ODEX_036: Subformat = Subformat::new("Optimized DEX 036");
    static ODEX_037: Subformat = Subformat::new("Optimized DEX 037");
    let sub = if starts(prefix, ODEX_035_MAGIC) {
        &ODEX_035
    } else if starts(prefix, ODEX_036_MAGIC) {
        &ODEX_036
    } else {
        &ODEX_037
    };
    with_subformat(Label::Odex, sub)
}

/// Khronos SPIR-V binary module: 4-byte-aligned size, 5-word header with magic `0x07230203` in
/// either endianness, version `1.0`..=`1.6` (`0x00010000..=0x00010600` with low byte 0), nonzero
/// ID bound at `@12`, and reserved schema `0` at `@16`.
pub(crate) fn spirv(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    const MAGIC: u32 = 0x0723_0203; // @0: u32 — SPIR-V magic number
    const VERSION: usize = 4; // @4: u32 — version word (0x00010000..=0x00010600, step 0x100)
    const BOUND: usize = 12; // @12: u32 — ID bound (> 0)
    const SCHEMA: usize = 16; // @16: u32 — instruction schema (0)
    const HEADER_LEN: usize = 20;

    require!(prefix.len() >= HEADER_LEN, ctx.size.is_multiple_of(4));
    let read_u32: ReadU32 = if le32(prefix, 0)? == MAGIC {
        le32
    } else if be32(prefix, 0)? == MAGIC {
        be32
    } else {
        return None;
    };
    let ver = read_u32(prefix, VERSION)?;
    require!(
        (0x0001_0000..=0x0001_0600).contains(&ver),
        ver & 0xFF == 0,
        read_u32(prefix, BOUND)? > 0,
        read_u32(prefix, SCHEMA)? == 0,
    );
    static SPIRV_LE: Subformat = Subformat::new("SPIR-V (little-endian)");
    static SPIRV_BE: Subformat = Subformat::new("SPIR-V (big-endian)");
    with_subformat(Label::Spirv, if le32(prefix, 0)? == MAGIC { &SPIRV_LE } else { &SPIRV_BE })
}
