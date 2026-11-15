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

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const ADVSYS_MAGIC_OFFSET: usize = 2;
const ADVSYS_OBFUSCATED_MAGIC: &[u8; 6] = hex2bin!("A0 9D 8B 8E 88 8E");
const AGT_AGX_MAGIC: &[u8; 4] = le32_bytes!(0x51C1_C758);
const ALAN3_MAGIC: &[u8; 4] = b"ALAN";
const BETHESDA_PEX_BE_MAGIC: &[u8; 4] = hex2bin!("FA 57 C0 DE");
const BETHESDA_PEX_LE_MAGIC: &[u8; 4] = hex2bin!("DE C0 57 FA");
const COD_GSCC_MAGIC: &[u8; 7] = bytes!(&[0x80], b"GSC", &[0x0D, 0x0A, 0x00]);
const GEOSOFT_CODE_OFF_OFFSET: usize = 12;
const GEOSOFT_GX_MAGIC: &[u8; 4] = b"GEOX";
const GEOSOFT_HDR_SIZE_OFFSET: usize = 8;
const GEOSOFT_VERSION_OFFSET: usize = 4;
const GHC_HI_32_MAGIC: &[u8; 4] = hex2bin!("01 FA CE 32");
const GHC_HI_64_MAGIC: &[u8; 4] = hex2bin!("01 FA CE 64");
const GLULX_MAGIC: &[u8; 4] = b"Glul";
const IL2CPP_METADATA_MAGIC: &[u8; 4] = hex2bin!("AF 1B B1 FA");
const JULIA_BOM_OFFSET: usize = 10;
const JULIA_JI_MAGIC: &[u8; 8] = bytes!(&[0xFB], b"jli", &[0x0D, 0x0A, 0x1A, 0x0A]);
const JULIA_VERSION_OFFSET: usize = 8;
const JULIA_WORD_BYTES_OFFSET: usize = 12;
const MAGNETIC_SCROLLS_GAME_MAGIC: &[u8; 4] = b"MaSc";
const MIN_ADVSYS_GAME_SIZE: u64 = 128;
const MIN_AGT_AGX_SIZE: u64 = 272;
const MIN_ALAN3_GAME_SIZE: u64 = 184;
const MIN_ALLEGRO_FASL_SIZE: u64 = 32;
const MIN_ANDROID_ART_PROFILE_SIZE: u64 = 12;
const MIN_APPLESCRIPT_COMPILED_SIZE: u64 = 32;
const MIN_ATARI_GFA_BASIC_SIZE: u64 = 165;
const MIN_AUTOCAD_VLX_SIZE: u64 = 32;
const MIN_AUTOLISP_FAS_SIZE: u64 = 64;
const MIN_BETHESDA_PEX_SIZE: u64 = 32;
const MIN_BIOWARE_NCS_SIZE: u64 = 14;
const MIN_BUN_PILE_SIZE: u64 = 24;
const MIN_COD_GSCC_SIZE: u64 = 64;
const MIN_CYBERLINK_KC_SIZE: u64 = 64;
const MIN_DOTNET_JIT_PROFILE_SIZE: u64 = 512;
const MIN_EMACS_ELC_SIZE: u64 = 16;
const MIN_GEOSOFT_GX_SIZE: u64 = 100;
const MIN_GHC_HI_SIZE: u64 = 32;
const MIN_GLULX_STORY_SIZE: u64 = 256;
const MIN_HSXG_06XD_BLS_SIZE: u64 = 64;
const MIN_IL2CPP_METADATA_SIZE: u64 = 256;
const MIN_INSTALLSHIELD_INS_SIZE: u64 = 48;
const MIN_INSTALLSHIELD_INX_SIZE: u64 = 64;
const MIN_INSTALLSHIELD_ISU_SIZE: u64 = 53;
const MIN_JPHP_MODULE_SIZE: u64 = 50;
const MIN_JULIA_JI_SIZE: u64 = 32;
const MIN_MAGNETIC_SCROLLS_GAME_SIZE: u64 = 64;
const MIN_METATRADER_EX4_SIZE: u64 = 32;
const MIN_METATRADER_EX5_SIZE: u64 = 32;
const MIN_MSVC_IFC_SIZE: u64 = 44;
const MIN_NINTENDO_AINB_SIZE: u64 = 64;
const MIN_OCAML_ARTIFACT_SIZE: u64 = 20;
const MIN_OSL_OSO_BINARY_SIZE: u64 = 28;
const MIN_PCBOARD_PPE_SIZE: u64 = 56;
const MIN_PSION_OPL_SIZE: u64 = 32;
const MIN_QUAKE3_QVM_SIZE: u64 = 36;
const MIN_QUEST_CAS_SIZE: u64 = 16;
const MIN_REALBASIC_RBBF_RBO_SIZE: u64 = 64;
const MIN_RENPY_RPYC_SIZE: u64 = 32;
const MIN_RUBY_RDOC_RI_SIZE: u64 = 28;
const MIN_RUBY_YARB_SIZE: u64 = 32;
const MIN_SAKURA_WARS_SBX_SIZE: u64 = 32;
const MIN_TADS3_IMAGE_SIZE: u64 = 79;
const MIN_TAG_ADVENTURE_SIZE: u64 = 32;
const MIN_TALLY_TCP_SIZE: u64 = 56;
const MIN_TORQUESCRIPT_DSO_SIZE: u64 = 32;
const MIN_UCSD_CODE_SIZE: u64 = 1024;
const MIN_V8_BYTECODE_SIZE: u64 = 32;
const MIN_VALVE_SHADER_VCS_SIZE: u64 = 64;
const MIN_WHITEWATER_ACTOR_IMA_SIZE: u64 = 64;
const NINTENDO_AINB_MAGIC: &[u8; 4] = b"AIB ";
const PCBOARD_PPE_BANNER: &[u8; 41] = b"PCBoard Programming Language Executable  ";
const PCBOARD_PPE_EOF_TERM: &[u8; 3] = hex2bin!("0D 0A 1A");
const QUAKE3_QVM_V1_MAGIC: &[u8; 4] = hex2bin!("44 14 72 12");
const QUAKE3_QVM_V2_MAGIC: &[u8; 4] = hex2bin!("45 14 72 12");
const QUEST_CAS_MAGIC: &[u8; 4] = b"QCGF";
const TADS3_IMAGE_MAGIC: &[u8; 11] = b"T3-image\r\n\x1a";
const TAG_ADVENTURE_MAGIC: &[u8; 7] = b"\x06t.a.g.";

/// AdvSys compiled adventure (`.dat`, 1986 David Michael Betz). Every header byte is stored
/// as `~b - 30` (decoded `b = !(c + 30)`):
/// - `@2..8`: `"ADVSYS"` (on disk `A0 9D 8B 8E 88 8E`)
/// - `@8`: `u16 LE` data structure version (`VERSION 102`, on disk `7B E1`)
pub(crate) fn verify_advsys_game(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_VERSION: usize = 8; // @8: obfuscated u16 LE version
    const OBFUSCATION_ADD: u8 = 30;
    let decode = |c: u8| !c.wrapping_add(OBFUSCATION_ADD);
    require!(
        size >= MIN_ADVSYS_GAME_SIZE,
        prefix.len() >= HDR_VERSION + 2,
        eq_at(prefix, ADVSYS_MAGIC_OFFSET, ADVSYS_OBFUSCATED_MAGIC)
    );
    let version = u16::from_le_bytes([
        decode(u8_at(prefix, HDR_VERSION)?),
        decode(u8_at(prefix, HDR_VERSION + 1)?),
    ]);
    require!((100..=110).contains(&version));
    Some(Label::Advsys)
}

/// AGT (Adventure Game Toolkit, 1987 David Malmberg) game encapsulated as AGX
/// (`.agx`, 1996 Robert Masenten, AGiliTy / Magx). All integers are little-endian.
/// - `@0..4`: `58 C7 C1 51`; `@4..8`: version owner, version, extension owner, extension
/// - `@8..10`: `"\n\r"` download-corruption canary; `@10`: extension fallback; `@11..16`: reserved zeros
/// - `@16`: file index block offset (always `16`); `@28`: index record size (always `16`)
pub(crate) fn verify_agt_agx(prefix: &[u8], size: u64) -> Option<Label> {
    const CANARY: usize = 8; // @8..10: "\n\r"
    const AGX_CANARY: &[u8; 2] = b"\n\r";
    const RESERVED: usize = 11; // @11..16: 5 reserved zero bytes
    const INDEX_OFFSET: usize = 16; // @16: u32 LE block 0 (file index) offset
    const INDEX_SIZE: usize = 20; // @20: u32 LE block 0 size
    const INDEX_RECSIZE: usize = 28; // @28: u32 LE block 0 record size
    const FILE_HEADER_SIZE: u32 = 16;
    const INDEX_RECORD_SIZE: u32 = 16;
    require!(
        size >= MIN_AGT_AGX_SIZE,
        prefix.len() >= 32,
        starts(prefix, AGT_AGX_MAGIC),
        eq_at(prefix, CANARY, AGX_CANARY),
        get_slice(prefix, RESERVED, INDEX_OFFSET)?.iter().all(|&b| b == 0),
        le32(prefix, INDEX_OFFSET)? == FILE_HEADER_SIZE,
        le32(prefix, INDEX_RECSIZE)? == INDEX_RECORD_SIZE
    );
    let index_size = le32(prefix, INDEX_SIZE)?;
    require!(
        index_size % INDEX_RECORD_SIZE == 0,
        (1..=64).contains(&(index_size / INDEX_RECORD_SIZE))
    );
    Some(Label::AgtAgx)
}

/// Alan 3 compiled adventure (`.a3c`, 2006 Thomas Nilsson). All words are big-endian.
/// - `@0..4`: `"ALAN"`
/// - `@4..8`: compiler version bytes (`major == 3`, `minor <= 9`; `03 00 04 61` = 3.0beta4)
/// - `@8`: `u32 BE` game uid
/// - `@12`: `u32 BE` file size in Awords (`1..0x0100_0000`)
/// - `@16`: `Abool` text packing flag (`0` or `1`)
pub(crate) fn verify_alan3_game(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_MAJOR: usize = 4; // @4: compiler major version
    const VERSION_MINOR: usize = 5; // @5: compiler minor version
    const SIZE_AWORDS: usize = 12; // @12: u32 BE size in 4-byte Awords
    const PACK: usize = 16; // @16: u32 BE Abool pack
    const ALAN3_MAJOR: u8 = 3;
    require!(
        size >= MIN_ALAN3_GAME_SIZE,
        prefix.len() >= 20,
        starts(prefix, ALAN3_MAGIC),
        u8_at(prefix, VERSION_MAJOR)? == ALAN3_MAJOR,
        u8_at(prefix, VERSION_MINOR)? <= 9,
        (1..0x0100_0000).contains(&be32(prefix, SIZE_AWORDS)?),
        be32(prefix, PACK)? <= 1
    );
    Some(Label::Alan3Game)
}

/// Franz Allegro Common Lisp Fast-Load Bytecode (.fasl, 1988) Franz.
pub(crate) fn verify_allegro_fasl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ALLEGRO_FASL_SIZE,
        prefix.len() >= 32,
        u8_at(prefix, 0)? == 0xF3,
        get_slice(prefix, 4, 14)? == b"\xca#<<AcL>> "
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::AllegroFasl)
}

/// Android Runtime (ART) Baseline Profile or Metadata file (`.prof` / `.profm`, Google).
pub(crate) fn verify_android_art_profile(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ANDROID_ART_PROFILE_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"pro\0") || starts(prefix, b"prm\0"),
        get_slice(prefix, 4, 7)?.iter().all(|b| b.is_ascii_digit()),
        u8_at(prefix, 7)? == 0
    );
    Some(Label::AndroidArtProfile)
}

/// Apple Compiled AppleScript Bytecode (.scpt).
pub(crate) fn verify_applescript_compiled(prefix: &[u8], size: u64) -> Option<Label> {
    const APPLESCRIPT_MAGIC: &[u8; 8] = b"FasdUAS "; // @0..8: Fast Save Data / AppleScript header
    require!(
        size >= MIN_APPLESCRIPT_COMPILED_SIZE,
        prefix.len() >= 24,
        starts(prefix, APPLESCRIPT_MAGIC)
    );
    let v110_hdr =
        get_slice(prefix, 8, 16)?.iter().all(|&b| b.is_ascii_digit() || matches!(b, b'.' | b' '))
            && get_slice(prefix, 16, prefix.len().min(32))?
                .iter()
                .any(|&b| (b < 0x20 && !matches!(b, b'\t' | b'\n' | b'\r')) || b >= 0x7F);
    // Classic Mac OS AppleScript 1.00 uses a 4-byte ASCII version ("1.00") followed immediately by binary data at @12.
    let v100_hdr =
        get_slice(prefix, 8, 12)?.iter().all(|&b| b.is_ascii_digit() || matches!(b, b'.' | b' '))
            && u8_at(prefix, 8)?.is_ascii_digit()
            && u8_at(prefix, 9)? == b'.'
            && get_slice(prefix, 12, prefix.len().min(24))?
                .iter()
                .any(|&b| (b < 0x20 && !matches!(b, b'\t' | b'\n' | b'\r')) || b >= 0x7F);
    require!(v110_hdr || v100_hdr);
    Some(Label::Scpt)
}

/// GFA Systemtechnik Atari ST GFA-BASIC 3.x Tokenized Program (`.gfa`, 1988).
pub(crate) fn verify_atari_gfa_basic(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC10_OFF: usize = 2; // @2..12: "GFA-BASIC3"
    const RESERVED_OFF: usize = 12; // @12..16: 4 reserved zero bytes
    const TABLE_OFF: usize = 16; // @16..164: 37 x u32 BE section offsets
    const HEADER_SIZE: usize = 164;
    const BODY_END_IDX: usize = 18; // @88..92: end of tokenized body relative to @164
    const SIGNATURE: &[u8; 10] = b"GFA-BASIC3";

    require!(
        size >= MIN_ATARI_GFA_BASIC_SIZE,
        prefix.len() >= HEADER_SIZE,
        matches!(u8_at(prefix, 0)?, 0x00 | 0xFF),
        matches!(u8_at(prefix, 1)?, 0x03 | 0x04),
        eq_at(prefix, MAGIC10_OFF, SIGNATURE),
        eq_at(prefix, RESERVED_OFF, &[0u8; 4])
    );

    if u8_at(prefix, 0)? == 0xFF {
        // `PSAVE` protected files zero `@12..64` (header table cleared before encrypted tokens).
        require!(get_slice(prefix, RESERVED_OFF, 64)?.iter().all(|&b| b == 0));
    } else {
        // Unprotected `SAVE`: 37 monotonically non-decreasing u32 BE offsets, with entry 18 bounded by EOF.
        let mut prev = 0u32;
        for i in 0..37 {
            let ptr = be32(prefix, TABLE_OFF + i * 4)?;
            require!(ptr >= prev);
            prev = ptr;
        }
        let body_end = be32(prefix, TABLE_OFF + BODY_END_IDX * 4)? as u64;
        require!(body_end > 0, (HEADER_SIZE as u64).checked_add(body_end)? <= size);
    }

    Some(Label::AtariGfaBasic)
}

/// Autodesk AutoCAD Visual LISP Compiled Application Package (.vlx, 1999) Autodesk.
pub(crate) fn verify_autocad_vlx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AUTOCAD_VLX_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"VRTLIB-1"),
        has_non_ascii_text_byte(prefix.get(8..19)?),
        get_slice(prefix, 19, 25)? == b"LOAD(("
    );
    Some(Label::AutocadVlx)
}

/// Autodesk AutoCAD Visual LISP Compiled Fast-Load Bytecode (.fas, 1999) Autodesk.
pub(crate) fn verify_autolisp_fas(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AUTOLISP_FAS_SIZE,
        prefix.len() >= 64,
        starts(prefix, b"\r\n FAS4-FILE ; Do not change it!\r\n")
            || starts(prefix, b"\r\n FAS5-FILE ; Do not change it!\r\n"),
        has_non_ascii_text_byte(prefix.get(34..64)?)
    );
    Some(Label::AutolispFas)
}

/// Bethesda Game Studios Creation Engine Compiled Papyrus Script (`.pex`, 2011 Bethesda Game Studios).
pub(crate) fn verify_bethesda_pex(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BETHESDA_PEX_SIZE, prefix.len() >= 20);
    let is_be = starts(prefix, BETHESDA_PEX_BE_MAGIC);
    let is_le = starts(prefix, BETHESDA_PEX_LE_MAGIC);
    require!(
        is_be || is_le,
        u8_at(prefix, 4).unwrap_or(0) == 3,
        (1..=9).contains(&u8_at(prefix, 5).unwrap_or(0))
    );
    let (game_id, src_len) = if is_be {
        (be16(prefix, 6)?, be16(prefix, 16)? as usize)
    } else {
        (le16(prefix, 6)?, le16(prefix, 16)? as usize)
    };
    require!(
        (1..=4).contains(&game_id),
        (1..=260).contains(&src_len),
        18 + src_len <= prefix.len(),
        get_slice(prefix, 18, 18 + src_len)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::BethesdaPex)
}

/// BioWare Aurora / Odyssey NWScript Compiled Bytecode file (`.ncs`, Neverwinter Nights, KotOR, The Witcher 1).
///
/// Header layout:
/// - `@0..8`: `"NCS V1.0"`
/// - `@8`: `0x42` (`'B'`, `T` total-file-size header opcode)
/// - `@9..13`: `u32 BE` `total_size` (`>= 13`, `<= file_size`, `<= 64 MiB`)
pub(crate) fn verify_bioware_ncs(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 8] = b"NCS V1.0"; // @0..8: NWScript Compiled V1.0 header
    const SIZE_OPCODE_OFF: usize = 8; // @8: 0x42 header size opcode
    const SIZE_OPCODE: u8 = 0x42;
    const TOTAL_SIZE_OFF: usize = 9; // @9..13: u32 BE bytecode image size
    const MAX_NCS_SIZE: u32 = 0x0400_0000; // 64 MiB cap (guarantees @9 <= 0x04, non-ASCII)

    require!(
        size >= MIN_BIOWARE_NCS_SIZE,
        prefix.len() >= 13,
        starts(prefix, MAGIC),
        u8_at(prefix, SIZE_OPCODE_OFF)? == SIZE_OPCODE
    );
    let declared_size = be32(prefix, TOTAL_SIZE_OFF)?;
    require!((13..=MAX_NCS_SIZE).contains(&declared_size), (declared_size as u64) <= size);
    Some(Label::BiowareNcs)
}

/// PILE 0x2B Archive (.pile).
pub(crate) fn verify_bun_pile(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BUN_PILE_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"\x2b\0\0\0"),
        matches!(u8_at(prefix, 4)?, 1 | 2),
        get_slice(prefix, 5, 14)? == b"\x03\xee\xcf\x1d\x27\xa6\x4d\xcb\x51"
    );
    require!(has_non_ascii_text_byte(prefix.get(..24)?));
    Some(Label::BunPile)
}

/// Activision / Treyarch Call of Duty Compiled Game Script (`.gscc` / `.gscbin`).
pub(crate) fn verify_cod_gscc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_COD_GSCC_SIZE,
        prefix.len() >= 24,
        starts(prefix, COD_GSCC_MAGIC),
        u8_at(prefix, 7)? != 0,
        le32(prefix, 12) == Some(0) || le32(prefix, 16)? >= 32
    );
    Some(Label::CodGscc)
}

/// CyberLink Compiled UI / Script Resource (`.kc`).
pub(crate) fn verify_cyberlink_kc(prefix: &[u8], size: u64) -> Option<Label> {
    const KC_MAGIC: &[u8; 12] = hex2bin!("1B 58 00 07 0A 00 00 00 00 00 00 00"); // @0..12: CyberLink KC header
    const KC_SENTINEL: &[u8; 16] = hex2bin!("E3 E3 00 00 00 00 00 00 00 00 00 00 00 00 00 00"); // @16..32: constant marker + reserved zeros
    const PAYLOAD_ID_OFF: usize = 12; // @12..16: non-zero u32 LE resource parameter

    require!(
        size >= MIN_CYBERLINK_KC_SIZE,
        prefix.len() >= 32,
        starts(prefix, KC_MAGIC),
        le32(prefix, PAYLOAD_ID_OFF)? > 0,
        eq_at(prefix, 16, KC_SENTINEL)
    );
    Some(Label::CyberlinkKc)
}

/// .NET Runtime Multicore JIT Startup Profile (.profile).
pub(crate) fn verify_dotnet_jit_profile(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DOTNET_JIT_PROFILE_SIZE,
        prefix.len() >= 24,
        get_slice(prefix, 0, 16)? == hex2bin!("40 00 00 01 65 00 00 00 00 00 00 00 47 00 00 00"),
        get_slice(prefix, 17, 20)? == hex2bin!("0E 00 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..24)?));
    Some(Label::DotnetJitProfile)
}

/// GNU Emacs Byte-Compiled Lisp (`.elc`, `";ELC"` + version `19..=35` + `\0\0\0`, or Emacs 18 `\n(def` + `"(byte-code \""` within prefix).
pub(crate) fn verify_emacs_elc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_EMACS_ELC_SIZE, prefix.len() >= 16);
    if starts(prefix, b";ELC") {
        require!((19..=35).contains(&u8_at(prefix, 4)?), get_slice(prefix, 5, 8)? == [0, 0, 0]);
        return Some(Label::Elc);
    }
    let win = get_slice(prefix, 0, prefix.len().min(8192))?;
    require!(
        (starts(prefix, b"\n(def")
            || starts(prefix, b";;; Byte-compiled")
            || starts(prefix, b"(byte-code "))
            && contains(win, b"(byte-code \"")
    );
    Some(Label::Elc)
}

/// Geosoft Oasis montaj Compiled GX Executable (`.gx`).
pub(crate) fn verify_geosoft_gx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GEOSOFT_GX_SIZE,
        prefix.len() >= 24,
        starts(prefix, GEOSOFT_GX_MAGIC),
        (1000..=1050).contains(&le32(prefix, GEOSOFT_VERSION_OFFSET)?),
        le32(prefix, GEOSOFT_HDR_SIZE_OFFSET) == Some(20),
        le32(prefix, GEOSOFT_CODE_OFF_OFFSET) == Some(100)
    );
    Some(Label::GeosoftGx)
}

/// Glasgow Haskell Compiler (GHC) Binary Interface file (`.hi` / `.p_hi` / `.dyn_hi`).
pub(crate) fn verify_ghc_hi(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GHC_HI_SIZE,
        prefix.len() >= 20,
        starts(prefix, GHC_HI_64_MAGIC) || starts(prefix, GHC_HI_32_MAGIC),
        (be32(prefix, 4) == Some(0) && (1..=16).contains(&be32(prefix, 8)?))
            || (1..=16).contains(&u8_at(prefix, 4)?)
    );
    Some(Label::GhcHi)
}

/// Glulx virtual machine story file (`.ulx`, 1999 Andrew Plotkin).
/// - `@0..4`: `"Glul"`
/// - `@4`: `u32 BE` spec version (`0x0002_0000..=0x0003_FFFF`, upper byte always 0)
/// - `@8/@12/@16/@20`: RAMSTART / EXTSTART / ENDMEM / stack size (`u32 BE`, multiples of 256,
///   `36 <= RAMSTART <= EXTSTART <= ENDMEM`)
pub(crate) fn verify_glulx_story(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION: usize = 4; // @4: u32 BE Glulx spec version
    const RAMSTART: usize = 8; // @8: u32 BE start of RAM
    const EXTSTART: usize = 12; // @12: u32 BE end of game file / start of extension memory
    const ENDMEM: usize = 16; // @16: u32 BE end of memory
    const STACKSIZE: usize = 20; // @20: u32 BE stack size
    const HEADER_SIZE: u32 = 36; // 9 × u32 header
    const PAGE: u32 = 256; // Glulx memory page granularity
    const SUPPORTED_VERSIONS: core::ops::Range<u32> = 0x0002_0000..0x0004_0000;
    require!(
        size >= MIN_GLULX_STORY_SIZE,
        prefix.len() >= HEADER_SIZE as usize,
        starts(prefix, GLULX_MAGIC),
        SUPPORTED_VERSIONS.contains(&be32(prefix, VERSION)?)
    );
    let ram = be32(prefix, RAMSTART)?;
    let ext = be32(prefix, EXTSTART)?;
    let end = be32(prefix, ENDMEM)?;
    let stack = be32(prefix, STACKSIZE)?;
    require!(
        ram >= HEADER_SIZE,
        ram <= ext,
        ext <= end,
        stack != 0,
        ram % PAGE == 0,
        ext % PAGE == 0,
        end % PAGE == 0,
        stack % PAGE == 0
    );
    Some(Label::Glulx)
}

/// Blizzard World of Warcraft Compiled Shader Bundle (`.bls`, `"HSXG"`).
pub(crate) fn verify_hsxg_06xd_bls(prefix: &[u8], size: u64) -> Option<Label> {
    const BLIZZARD_BLS_MAGIC: &[u8; 4] = b"HSXG"; // @0..4

    require!(
        size >= MIN_HSXG_06XD_BLS_SIZE,
        prefix.len() >= 20,
        starts(prefix, BLIZZARD_BLS_MAGIC),
        (0x0001_0001..=0x0002_0020).contains(&le32(prefix, 4)?), // @4..8: version (e.g. 0x0001000E)
        get_slice(prefix, 8, 12)?.iter().all(|&b| b.is_ascii_alphanumeric() || b == b' '), // @8..12: shader profile FourCC (06XD/05XD/...)
        (16..=4096).contains(&le32(prefix, 12)?), // @12..16: header/table offset
        le32(prefix, 16)? >= 1                    // @16..20: permutation count
    );
    Some(Label::BlizzardWowBls)
}

/// InstallShield 2 / 3 / 5 Compiled Script (`.ins`, 1993–1997 Stirling Technologies / InstallShield).
pub(crate) fn verify_installshield_ins(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC_B8C9: &[u8] = hex2bin!("B8 C9 0C 00"); // @0..4: InstallShield 3 / 5 .ins magic
    const MAGIC_FFFF: &[u8] = hex2bin!("FF FF 0C 00"); // @0..4: InstallShield 2 .ins magic
    const MAGIC_IS2_16BIT: &[u8] = hex2bin!("FF FF 0C 00 14 00 34 12"); // @0..8: InstallShield 2/3 16-bit .ins header
    const SUBVER_OFFSET: usize = 6; // @6..8: u16 LE subversion (0x1234 IS2/IS3 or 0x0001 IS5)
    const BANNER_START: usize = 14; // @14..48: copyright/vendor string
    require!(
        size >= MIN_INSTALLSHIELD_INS_SIZE,
        prefix.len() >= 48,
        starts(prefix, MAGIC_B8C9) || starts(prefix, MAGIC_FFFF),
        matches!(le16(prefix, SUBVER_OFFSET)?, 0x1234 | 0x0001),
        scan_range(prefix, BANNER_START, 48, b"Stirling Technologies")
            || scan_range(prefix, BANNER_START, 48, b"InstallSHIELD")
            || (starts(prefix, MAGIC_IS2_16BIT)
                && le16(prefix, 8)? > 0
                && le16(prefix, 10)? > 0
                && le16(prefix, 12)? > 0)
    );
    Some(Label::InstallshieldIns)
}

/// InstallShield Compiled Rules Script (`.inx`, 1990 Stirling Technologies / InstallShield, PRONOM `fmt/197`).
/// Supports both plaintext `"aLuZ\0\0Copyright (c) "` and the InstallShield 6/7+ XOR/ROR-encrypted `"aLuZ\0\0Copyright (c) 1990-"` header.
pub(crate) fn verify_installshield_inx(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC_PLAIN: &[u8] = b"aLuZ\0\0Copyright (c) "; // @0..20: plaintext aLuZ header
    const MAGIC_ENC: &[u8] =
        hex2bin!("74 C4 2C 84 E1 E5 D4 28 10 FB 00 20 3C 24 FB 4D 11 20 1D 3D E4 C8 CC EC E4"); // @0..25: encrypted "aLuZ\0\0Copyright (c) 1990-"
    require!(
        size >= MIN_INSTALLSHIELD_INX_SIZE,
        starts(prefix, MAGIC_PLAIN) || starts(prefix, MAGIC_ENC)
    );
    Some(Label::InstallshieldInx)
}

/// InstallShield Uninstall Script / Log (`.isu`, 1995 Stirling Technologies / InstallShield).
pub(crate) fn verify_installshield_isu(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = hex2bin!("71 A8 00 00 01 02"); // @0..6: InstallShield .isu magic + version 1.2
    const BANNER_LEN_OFFSET: usize = 10; // @10..12: u16 LE vendor banner length (typically 41 = 0x0029)
    const BANNER_OFFSET: usize = 12; // @12..34: "Stirling Technologies,"
    require!(
        size >= MIN_INSTALLSHIELD_ISU_SIZE,
        prefix.len() >= 34,
        starts(prefix, MAGIC),
        (22..=128).contains(&le16(prefix, BANNER_LEN_OFFSET)?),
        eq_at(prefix, BANNER_OFFSET, b"Stirling Technologies,")
    );
    Some(Label::InstallshieldIsu)
}

/// JPHP (DevelNext) compiled PHP-on-JVM module (`.phb`, 2014 Dmitriy Zayceff / jphp-group).
///
/// `php.runtime.loader.dump.ModuleDumper` writes `DUMP_STAMP` + `DUMP_VERSION`, a legacy `LangMode`
/// enum ordinal (`writeEnum`: `-1` when null), then the module name via `writeName` (`i32` UTF-16
/// char count + `writeUTF` `u16` modified-UTF-8 byte length + bytes, or a bare `i32 -1` when null).
/// Modified UTF-8 never contains `0x00` and spends 1..=3 bytes per UTF-16 unit, which ties the
/// two length fields together.
pub(crate) fn verify_jphp_module(prefix: &[u8], size: u64) -> Option<Label> {
    const DUMP_STAMP_AND_VERSION: &[u8; 8] =
        bytes!(be32_bytes!(479_873_682), be32_bytes!(20_141_011)); // @0..8
    const LANG_MODE_OFF: usize = 8; // @8: i32 LangMode ordinal (-1 = null)
    const MODULE_NAME_LEN_OFF: usize = 12; // @12: i32 UTF-16 char count (-1 = null)
    const MODULE_NAME_UTF_OFF: usize = 16; // @16: u16 modified-UTF-8 byte length
    const MAX_LANG_MODE: i32 = 7;
    require!(
        size >= MIN_JPHP_MODULE_SIZE,
        prefix.len() >= MIN_JPHP_MODULE_SIZE as usize,
        starts(prefix, DUMP_STAMP_AND_VERSION)
    );
    let lang_mode = be32(prefix, LANG_MODE_OFF)? as i32;
    require!((-1..=MAX_LANG_MODE).contains(&lang_mode));
    let name_chars = be32(prefix, MODULE_NAME_LEN_OFF)? as i32;
    if name_chars != -1 {
        let utf_len = be16(prefix, MODULE_NAME_UTF_OFF)? as usize;
        let chars = usize::try_from(name_chars).ok()?;
        require!(chars <= utf_len && utf_len <= chars.saturating_mul(3));
        let name_end = MODULE_NAME_UTF_OFF + 2 + utf_len;
        require!(name_end as u64 <= size);
        if let Some(name) = get_slice(prefix, MODULE_NAME_UTF_OFF + 2, name_end.min(prefix.len())) {
            require!(!name.contains(&0));
        }
    }
    Some(Label::JphpModule)
}

/// Julia Precompiled Package Image (`.ji`).
pub(crate) fn verify_julia_ji(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JULIA_JI_SIZE,
        prefix.len() >= 24,
        starts(prefix, JULIA_JI_MAGIC),
        (1..=32).contains(&le16(prefix, JULIA_VERSION_OFFSET)?),
        le16(prefix, JULIA_BOM_OFFSET) == Some(0xFEFF),
        matches!(u8_at(prefix, JULIA_WORD_BYTES_OFFSET)?, 4 | 8)
    );
    Some(Label::JuliaJi)
}

/// Magnetic Scrolls game file (`.mag`, 1985 Magnetic Scrolls; container by Niclas Karlsson).
/// All words are big-endian.
/// - `@0..4`: `"MaSc"`; `@4`: `u32 BE` file size; `@8`: `u32 BE` header size (always 42)
/// - `@12`: `u16 BE` game version (`0` Pawn .. `4` Magnetic Windows)
/// - `@14`: `u32 BE` 68000 code size (non-zero)
pub(crate) fn verify_magnetic_scrolls_game(prefix: &[u8], size: u64) -> Option<Label> {
    const FILE_SIZE: usize = 4; // @4: u32 BE file size
    const HEADER_SIZE_FIELD: usize = 8; // @8: u32 BE header size
    const GAME_VERSION: usize = 12; // @12: u16 BE game version
    const CODE_SIZE: usize = 14; // @14: u32 BE code size
    const MAG_HEADER_SIZE: u32 = 42;
    const MAX_GAME_VERSION: u16 = 4;
    require!(
        size >= MIN_MAGNETIC_SCROLLS_GAME_SIZE,
        prefix.len() >= MAG_HEADER_SIZE as usize,
        starts(prefix, MAGNETIC_SCROLLS_GAME_MAGIC),
        be32(prefix, HEADER_SIZE_FIELD)? == MAG_HEADER_SIZE,
        be32(prefix, FILE_SIZE)? >= MAG_HEADER_SIZE,
        be16(prefix, GAME_VERSION)? <= MAX_GAME_VERSION,
        be32(prefix, CODE_SIZE)? != 0
    );
    Some(Label::MagneticScrollsMag)
}

/// MetaQuotes MetaTrader 4 Compiled MQL4 Expert Advisor Bytecode (.ex4, 2005) MetaQuotes.
pub(crate) fn verify_metatrader_ex4(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_METATRADER_EX4_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"EX4\0"),
        get_slice(prefix, 12, 22)? == b"Copyright "
    );
    Some(Label::MetatraderEx4)
}

/// MetaQuotes MetaTrader 5 Compiled MQL5 Expert Advisor Bytecode (.ex5, 2010) MetaQuotes.
pub(crate) fn verify_metatrader_ex5(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_METATRADER_EX5_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"EX5\x02q\0"),
        le16(prefix, 14) == Some(1),
        le32(prefix, 20).unwrap_or(99) <= 1
    );
    Some(Label::MetatraderEx5)
}

/// Microsoft Visual C++ C++20 Built Module Interface (`.ifc`, IPR/IFC).
pub(crate) fn verify_msvc_ifc(prefix: &[u8], size: u64) -> Option<Label> {
    const MSVC_IFC_MAGIC: &[u8; 4] = bytes!(b"TQE", &[0x1A]); // @0..4: MSVC IFC magic
    const SHA256_OFF: usize = 4; // @4..36: 32-byte SHA-256 checksum
    const MAJOR_OFF: usize = 36; // @36: format major version (0)
    const MINOR_OFF: usize = 37; // @37: format minor version (e.g., 42..44)
    const ABI_OFF: usize = 38; // @38: C++ ABI version (0)
    const ARCH_OFF: usize = 39; // @39: target architecture enum (0..=8)

    require!(
        size >= MIN_MSVC_IFC_SIZE,
        prefix.len() >= 44,
        starts(prefix, MSVC_IFC_MAGIC),
        u8_at(prefix, MAJOR_OFF)? == 0,
        (1..=64).contains(&u8_at(prefix, MINOR_OFF)?),
        u8_at(prefix, ABI_OFF)? == 0,
        u8_at(prefix, ARCH_OFF)? <= 8,
        has_non_ascii_text_byte(prefix.get(SHA256_OFF..MAJOR_OFF)?)
    );
    Some(Label::MsvcIfc)
}

/// Nintendo AI Node Binary Graph (`.ainb`).
///
/// Header layout:
/// - `@0..4`: `"AIB "` magic
/// - `@4..8`: `u32 LE` `version` (`0x0400..=0x0500`, e.g., `0x0407` in TotK)
/// - `@8..12`: `u32 LE` `filename_offset` (string-pool offset, `< file_size`)
/// - `@32..36`: `u32 LE` `string_pool_offset` (`>= 40`, `< file_size`)
pub(crate) fn verify_nintendo_ainb(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4..8: u32 LE format version (0x0400..=0x0500)
    const FILENAME_OFF: usize = 8; // @8..12: u32 LE filename offset in string pool
    const STRPOOL_OFF: usize = 32; // @32..36: u32 LE string pool offset

    require!(
        size >= MIN_NINTENDO_AINB_SIZE,
        prefix.len() >= 40,
        starts(prefix, NINTENDO_AINB_MAGIC),
        (0x0400..=0x0500).contains(&le32(prefix, VERSION_OFF)?),
        (le32(prefix, FILENAME_OFF)? as u64) < size
    );
    let strpool_off = le32(prefix, STRPOOL_OFF)? as u64;
    require!(strpool_off >= 40, strpool_off < size);
    Some(Label::NintendoAinb)
}

/// OCaml Compiler Artifact (`.cmi` / `.cmo` / `.cma` / `.cmx` / `.cmt`, 1996 INRIA).
pub(crate) fn verify_ocaml_compiler_artifact(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_OCAML_ARTIFACT_SIZE
            && prefix.len() >= 16
            && starts(prefix, b"Caml1999")
            && matches!(
                u8_at(prefix, 8)?,
                b'I' | b'O' | b'A' | b'X' | b'Y' | b'Z' | b'M' | b'N' | b'T' | b'D'
            )
            && get_slice(prefix, 9, 12)?.iter().all(|&b| b.is_ascii_digit())
            && get_slice(prefix, 12, prefix.len().min(20))?
                .iter()
                .any(|&b| (b < 0x20 && !matches!(b, b'\t' | b'\n' | b'\r')) || b >= 0x7F)
    );
    Some(Label::OcamlCmi)
}

/// Compiled Binary OSO Shader (.oso).
pub(crate) fn verify_osl_oso_binary(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_OSL_OSO_BINARY_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"OSO_"),
        le16(prefix, 4).unwrap_or(0) > 0,
        le32(prefix, 6) == Some(0),
        le64(prefix, 12) == Some(0)
    );
    Some(Label::OslOsoBinary)
}

/// Clark Development PCBoard Programming Language Executable (`.ppe`, 1993–1996, PPL 1.00–3.40).
///
/// Header layout (`v1.00`, `v2.00`, `v3.01`, `v3.10`):
/// - `@0..41`: `b"PCBoard Programming Language Executable  "` (two spaces before version)
/// - `@41..45`: ASCII version `X.XX` (`'1'..='9'`, `'.'`, `'0'..='9'`, `'0'..='9'`)
/// - `@45..48`: `0D 0A 1A` (`\r\n\x1A`, binary DOS EOF guard rejecting printable ASCII)
/// - `@48..50`: `u16 LE num_vars` (`>= 1`)
pub(crate) fn verify_pcboard_ppe(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 41; // @41..45: ASCII "X.XX" PPL compiler version
    const TERM_OFF: usize = 45; // @45..48: CR LF EOF (0D 0A 1A)
    const NUM_VARS_OFF: usize = 48; // @48..50: u16 LE variable table slot count
    require!(
        size >= MIN_PCBOARD_PPE_SIZE,
        prefix.len() >= NUM_VARS_OFF + 2,
        starts(prefix, PCBOARD_PPE_BANNER),
        (b'1'..=b'9').contains(&u8_at(prefix, VERSION_OFF)?),
        u8_at(prefix, VERSION_OFF + 1)? == b'.',
        u8_at(prefix, VERSION_OFF + 2)?.is_ascii_digit(),
        u8_at(prefix, VERSION_OFF + 3)?.is_ascii_digit(),
        eq_at(prefix, TERM_OFF, PCBOARD_PPE_EOF_TERM),
        le16(prefix, NUM_VARS_OFF)? >= 1
    );
    Some(Label::PcboardPpe)
}

/// Psion Organiser II / Series 3 OPL Translated Bytecode file (`.opo` / `.opa`, `"OPLObjectFile**\0"`).
pub(crate) fn verify_psion_opl(prefix: &[u8], size: u64) -> Option<Label> {
    const OPL_MAGIC: &[u8; 16] = b"OPLObjectFile**\0"; // @0..16: 16-byte OPL object signature
    const VERSION_OFFSET: usize = 16; // @16..18: u16 LE format version (1)

    require!(
        size >= MIN_PSION_OPL_SIZE,
        prefix.len() >= 24,
        starts(prefix, OPL_MAGIC),
        (1..=16).contains(&le16(prefix, VERSION_OFFSET)?)
    );
    Some(Label::PsionOpl)
}

/// id Software id Tech 3 Quake III Virtual Machine Bytecode Module (`.qvm`, 1999 id Software).
pub(crate) fn verify_quake3_qvm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_QUAKE3_QVM_SIZE,
        prefix.len() >= 32,
        starts(prefix, QUAKE3_QVM_V1_MAGIC) || starts(prefix, QUAKE3_QVM_V2_MAGIC)
    );
    let inst_count = le32(prefix, 4)?;
    let code_off = le32(prefix, 8)? as u64;
    let code_len = le32(prefix, 12)? as u64;
    let data_off = le32(prefix, 16)? as u64;
    let data_len = le32(prefix, 20)? as u64;
    let lit_len = le32(prefix, 24)? as u64;
    require!(
        inst_count > 0,
        code_off >= 32,
        code_off.saturating_add(code_len) == data_off,
        data_off.saturating_add(data_len).saturating_add(lit_len) <= size
    );
    Some(Label::Quake3Qvm)
}

/// Quest 3.x/4.x compiled game (`.cas`, 1998 Axe Software / Alex Warren).
/// - `@0..4`: `"QCGF"`; `@4..7`: 3 ASCII digits; `@7`: NUL
/// - `@8`: first keyword token (`0x01..=0xFE`, `0x08` = `define`); the stream mixes keyword bytes with
///   `0x0A`/`0xFE`-introduced `255 - c` obfuscated strings, with a binary byte within 16 bytes.
pub(crate) fn verify_quest_cas(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION: usize = 4; // @4..7: 3 ASCII digits
    const VERSION_NUL: usize = 7; // @7: NUL
    const FIRST_TOKEN: usize = 8; // @8: first keyword byte
    require!(
        size >= MIN_QUEST_CAS_SIZE,
        prefix.len() >= 16,
        starts(prefix, QUEST_CAS_MAGIC),
        get_slice(prefix, VERSION, VERSION_NUL)?.iter().all(u8::is_ascii_digit),
        u8_at(prefix, VERSION_NUL)? == 0,
        (0x01..=0xFE).contains(&u8_at(prefix, FIRST_TOKEN)?)
    );
    Some(Label::QuestCas)
}

/// REALbasic / Xojo Binary Project or Object (`"RbBF"` + version 1 + `"Blok"` chunk).
pub(crate) fn verify_realbasic_rbbf_rbo(prefix: &[u8], size: u64) -> Option<Label> {
    const REALBASIC_RBBF_MAGIC: &[u8; 8] = bytes!(b"RbBF", be32_bytes!(1)); // @0..8
    const REALBASIC_BLOK_MAGIC: &[u8; 4] = b"Blok"; // @blk_off..blk_off+4

    require!(
        size >= MIN_REALBASIC_RBBF_RBO_SIZE,
        prefix.len() >= 24,
        starts(prefix, REALBASIC_RBBF_MAGIC),
        get_slice(prefix, 8, 16)? == [0u8; 8]
    );
    let blk_off = be32(prefix, 16)? as usize;
    require!(
        (20..=256).contains(&blk_off),
        blk_off + 4 <= prefix.len(),
        eq_at(prefix, blk_off, REALBASIC_BLOK_MAGIC)
    );
    Some(Label::RealbasicRbbf)
}

/// Ren'Py Compiled Script AST Bytecode (.rpyc).
pub(crate) fn verify_renpy_rpyc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RENPY_RPYC_SIZE
            && prefix.len() >= 22
            && (starts(prefix, b"RENPY RPC2") || starts(prefix, b"RPYC2\0\0\0\0\0"))
    );
    let slot_id = le32(prefix, 10)?;
    let slot_start = le32(prefix, 14)? as u64;
    let slot_len = le32(prefix, 18)? as u64;
    require!(slot_id == 1 && slot_start >= 22 && slot_len > 0 && slot_start + slot_len <= size);
    Some(Label::Rpyc)
}

/// Ruby RDoc RI Marshal Documentation Entry (.ri, 2003).
pub(crate) fn verify_ruby_rdoc_ri(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RUBY_RDOC_RI_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"\x04\x08U:"),
        (10..=64).contains(&u8_at(prefix, 4)?),
        get_slice(prefix, 5, 11)? == b"RDoc::"
    );
    Some(Label::RubyRdocRi)
}

/// CRuby YARV Compiled Instruction Sequence Bytecode (YARB).
pub(crate) fn verify_ruby_yarb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RUBY_YARB_SIZE
            && prefix.len() >= 20
            && (1..=10).contains(&le32(prefix, 4).unwrap_or(0))
            && le32(prefix, 8).unwrap_or(99) <= 20
            && le32(prefix, 12).unwrap_or(0) >= 32
    );
    Some(Label::RubyYarb)
}

/// SEGA / Overworks Sakura Wars 3 Compiled Script (ASCR, .sbx).
pub(crate) fn verify_sakura_wars_sbx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SAKURA_WARS_SBX_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"ASCR"),
        get_slice(prefix, 8, 12)? == b"\xba\xaf\x55\xcc",
        le32(prefix, 20) == Some(24)
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::SakuraWarsSbx)
}

/// TADS 3 T3 VM image file (`.t3`, 1999 Michael J. Roberts).
/// - `@0..11`: `"T3-image\r\n\x1A"`
/// - `@11`: `u16 LE` image format version (`1..=16`)
/// - `@45..69`: 24-char ASCII `asctime` build timestamp (`"Wed Oct 10 16:19:37 2007"`)
/// - `@69..73`: first block type (4 ASCII letters, normally `"ENTP"`)
pub(crate) fn verify_tads3_image(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION: usize = 11; // @11: u16 LE image format version
    const TIMESTAMP: usize = 45; // @45..69: asctime() string
    const TIMESTAMP_LEN: usize = 24;
    const TIME_COLON_1: usize = TIMESTAMP + 13; // "hh:" colon
    const TIME_COLON_2: usize = TIMESTAMP + 16; // "mm:" colon
    const FIRST_BLOCK: usize = 69; // @69..73: block type FourCC
    require!(
        size >= MIN_TADS3_IMAGE_SIZE,
        prefix.len() >= FIRST_BLOCK + 4,
        starts(prefix, TADS3_IMAGE_MAGIC),
        (1..=16).contains(&le16(prefix, VERSION)?),
        get_slice(prefix, TIMESTAMP, TIMESTAMP + TIMESTAMP_LEN)?
            .iter()
            .all(|b| (0x20..=0x7E).contains(b)),
        u8_at(prefix, TIME_COLON_1)? == b':',
        u8_at(prefix, TIME_COLON_2)? == b':',
        get_slice(prefix, FIRST_BLOCK, FIRST_BLOCK + 4)?.iter().all(u8::is_ascii_alphabetic)
    );
    Some(Label::Tads3Image)
}

/// T.A.G. (Text-Adventure-Generator, 1997 Martin Oehm) compiled game (`.tag`):
/// a sequence of Turbo Pascal length-prefixed strings `06 "t.a.g."`, version (`03 "2.0"`, digits/dots)
/// and build date (`0A "08-10-2000"`).
pub(crate) fn verify_tag_adventure(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_LEN: usize = 7; // @7: ShortString length of version
    require!(
        size >= MIN_TAG_ADVENTURE_SIZE,
        prefix.len() >= 32,
        starts(prefix, TAG_ADVENTURE_MAGIC)
    );
    let vlen = usize::from(u8_at(prefix, VERSION_LEN)?);
    require!((1..=8).contains(&vlen));
    let version = get_slice(prefix, VERSION_LEN + 1, VERSION_LEN + 1 + vlen)?;
    let date_len_at = VERSION_LEN + 1 + vlen;
    require!(
        version.iter().all(|b| b.is_ascii_digit() || *b == b'.'),
        (1..=10).contains(&u8_at(prefix, date_len_at)?)
    );
    Some(Label::TagAdventure)
}

/// Tally.ERP 9 Compiled Tally Definition Language (`.tcp`) bytecode file.
pub(crate) fn verify_tally_compiled_tdl(prefix: &[u8], size: u64) -> Option<Label> {
    const TALLY_SDB_MAGIC: &[u8; 8] = bytes!(&[0x89], b"SDB\r\n", &[0x1A], b"\n"); // @0..8: PNG-style 8-byte SDB signature
    const FORMAT_MAJOR_OFFSET: usize = 12; // @12..16: u32 LE major version (2)
    const FORMAT_MINOR_OFFSET: usize = 16; // @16..20: u32 LE minor version (2)
    const HDR_LEN_OFFSET: usize = 20; // @20..24: u32 LE header length (0x38 = 56 bytes)

    require!(
        size >= MIN_TALLY_TCP_SIZE,
        prefix.len() >= 28,
        starts(prefix, TALLY_SDB_MAGIC),
        le32(prefix, FORMAT_MAJOR_OFFSET) == Some(2),
        le32(prefix, FORMAT_MINOR_OFFSET) == Some(2),
        le32(prefix, HDR_LEN_OFFSET) == Some(0x38)
    );
    Some(Label::TallyTcp)
}

/// GarageGames Torque Game Engine Compiled TorqueScript (.dso, 2001) GarageGames.
pub(crate) fn verify_torquescript_dso(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TORQUESCRIPT_DSO_SIZE,
        prefix.len() >= 32,
        matches!(le32(prefix, 0), Some(33 | 36 | 46)),
        (8..=size as u32).contains(&le32(prefix, 4).unwrap_or(0)),
        contains(prefix.get(8..prefix.len().min(48))?, b"$pref::")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::TorquescriptDso)
}

/// UCSD Pascal p-Machine compiled codefile (`.code`, 1977) UC San Diego / SofTech Microsystems.
///
/// Supports both little-endian and big-endian 16-slot segment dictionaries (`@0..256`) across
/// UCSD p-System II.0–IV.2: validates active `(codeaddr, codeleng)` block bounds against `size`,
/// 8-byte uppercase space-padded segment names (`@64..192`), inactive slot hygiene (`b"        "`
/// or `[0; 8]`), and p-System `segkind` nibble/version encoding (`@192..224`).
pub(crate) fn verify_ucsd_code(prefix: &[u8], size: u64) -> Option<Label> {
    const DICT_LEN: usize = 512; // @0..512: Block 0 UCSD segment dictionary
    const SLOTS: usize = 16;
    const NAMES_OFF: usize = 64; // @64..192: 16 × 8-byte space-padded ASCII segment names
    const KINDS_OFF: usize = 192; // @192..224: 16 × u16 segkind (low nibble 0..=8, hi byte version)

    require!(
        size >= MIN_UCSD_CODE_SIZE,
        size.is_multiple_of(512),
        prefix.len() >= DICT_LEN,
        has_non_ascii_text_byte(prefix.get(..64)?)
    );

    let check_dict = |big_endian: bool| -> bool {
        let u16_at = |o: usize| -> Option<u16> {
            if big_endian { be16(prefix, o) } else { le16(prefix, o) }
        };
        let mut active = 0usize;
        for s in 0..SLOTS {
            let Some(codeaddr) = u16_at(s * 4) else { return false };
            let Some(codeleng) = u16_at(s * 4 + 2) else { return false };
            let Some(segkind) = u16_at(KINDS_OFF + s * 2) else { return false };
            let Some(name) = get_slice(prefix, NAMES_OFF + s * 8, NAMES_OFF + (s + 1) * 8) else {
                return false;
            };
            if codeleng == 0 && codeaddr == 0 {
                let all_sp = name.iter().all(|&b| b == b' ');
                let all_zero = name.iter().all(|&b| b == 0);
                if (!all_sp && !all_zero) || (segkind & 0x00FF) != 0 || (segkind >> 8) > 0x0F {
                    return false;
                }
            } else {
                if codeaddr == 0 || codeleng == 0 {
                    return false;
                }
                let end_byte = (codeaddr as u64) * 512 + (codeleng as u64);
                if end_byte > size {
                    return false;
                }
                // Low nibble: segment kind 0..=8; high byte: p-System version (0 for II.0, 2 for IV.x)
                if (segkind & 0x000F) > 8 || (segkind >> 8) > 0x0F {
                    return false;
                }
                // Segment name: starts with A-Z, followed by A-Z0-9_ then trailing spaces
                if !name[0].is_ascii_uppercase() {
                    return false;
                }
                let mut seen_space = false;
                for &b in name {
                    if b == b' ' {
                        seen_space = true;
                    } else if seen_space
                        || !(b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
                    {
                        return false;
                    }
                }
                active += 1;
            }
        }
        active >= 1
    };

    require!(check_dict(false) || check_dict(true));
    Some(Label::UcsdCode)
}

/// Unity Engine IL2CPP Global Metadata Binary (`global-metadata.dat`, 2015 Unity Technologies).
///
/// Header layout (`Il2CppGlobalMetadataHeader`, little-endian):
/// - `@0..4`: `u32 LE` `sanity` (`0xFAB11BAF` = `AF 1B B1 FA`)
/// - `@4..8`: `i32 LE` `version` (`16..=40`, Unity 5.0 through Unity 6)
/// - `@8..12`: `u32 LE` `stringLiteralOffset` (`>= 128`, 4-byte aligned)
/// - `@12..16`: `u32 LE` `stringLiteralSize` (multiple of 8: `sizeof(Il2CppStringLiteral) == 8`)
/// - `@16..20`: `u32 LE` `stringLiteralDataOffset` (`== stringLiteralOffset + stringLiteralSize`)
/// - `@20..24`: `u32 LE` `stringLiteralDataSize`
/// - `@24..28`: `u32 LE` `stringOffset` (`>= stringLiteralDataOffset + stringLiteralDataSize`)
pub(crate) fn verify_unity_il2cpp_metadata(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_IL2CPP_METADATA_SIZE,
        prefix.len() >= 32,
        starts(prefix, IL2CPP_METADATA_MAGIC)
    );
    let version = le32(prefix, 4)?;
    let str_lit_off = le32(prefix, 8)?;
    let str_lit_size = le32(prefix, 12)?;
    let str_lit_data_off = le32(prefix, 16)?;
    let str_lit_data_size = le32(prefix, 20)?;
    let str_off = le32(prefix, 24)?;
    require!(
        (16..=45).contains(&version),
        str_lit_off >= 128,
        str_lit_off % 4 == 0,
        str_lit_size % 8 == 0,
        str_lit_off.checked_add(str_lit_size) == Some(str_lit_data_off),
        str_off >= str_lit_data_off.saturating_add(str_lit_data_size)
    );
    Some(Label::Il2cppMetadata)
}

/// Google V8 JavaScript Engine Cached Bytecode (.jsc / .bin).
pub(crate) fn verify_v8_bytecode(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_V8_BYTECODE_SIZE
            && prefix.len() >= 28
            && u8_at(prefix, 2)? == 0xDE
            && u8_at(prefix, 3)? == 0xC0
            && (0x01..=0x0F).contains(&u8_at(prefix, 1)?)
    );
    let version_hash = le32(prefix, 4)?;
    let flag_hash = le32(prefix, 12)?;
    let p1 = le32(prefix, 16).unwrap_or(0);
    let p2 = le32(prefix, 20).unwrap_or(0);
    let p3 = le32(prefix, 24).unwrap_or(0);
    require!(
        version_hash > 0x0001_0000
            && flag_hash != 0
            && version_hash != flag_hash
            && (p1 > 0 || p2 > 0 || p3 > 0)
            && prefix.get(4..28)?.contains(&0)
    );
    Some(Label::V8Bytecode)
}

/// Valve Source Engine LZMA-Compiled Shader (.vcs, v6).
pub(crate) fn verify_valve_shader_vcs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VALVE_SHADER_VCS_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 24)?
            == hex2bin!("06 00 00 00 01 00 00 00 01 00 00 00 00 00 00 00 00 00 00 00 02 00 00 00"),
        get_slice(prefix, 28, 32)? == hex2bin!("00 00 00 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::ValveShaderVcs)
}

/// Whitewater Group Actor Object-Oriented Image (`.ima`, 1987) Whitewater Group.
pub(crate) fn verify_whitewater_actor_ima(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WHITEWATER_ACTOR_IMA_SIZE,
        prefix.len() >= 16,
        starts(prefix, &[0x01, 0x03, 0xFC, 0xFF, 0x28, 0x00, 0x3C, 0x00])
    );
    Some(Label::WhitewaterActorIma)
}
