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
        matches!(be32(prefix, FLAVOR)?, SFNT_1_0 | SFNT_OTTO | SFNT_TRUE | SFNT_TYP1),
    );
    Some(Label::Woff)
}

/// W3C Web Open Font Format 2.0 (WOFF2): `"wOF2"` magic, known sfnt/collection flavor at `@4`,
/// length ≥ 51 at `@8`, `1..=4095` tables at `@12`, reserved `0` at `@14`, totalSfntSize ≥ 28 at
/// `@16`, and nonzero totalCompressedSize at `@20`.
pub(crate) fn woff2(prefix: &[u8]) -> Option<Label> {
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
        matches!(be32(prefix, FLAVOR)?, SFNT_1_0 | SFNT_OTTO | SFNT_TRUE | SFNT_TYP1 | SFNT_TTCF),
    );
    Some(Label::Woff2)
}

/// Apple / Microsoft TrueType Font (`ttf`, 1991) & TrueType Collection (`ttcf`):
/// sfnt version `00 01 00 00`, `"true"`, or `"typ1"` with valid `numTables`, power-of-two
/// `searchRange`, and printable 4-byte table tags; or `"ttcf"` header with version `1|2`
/// and font count `1..=1024`.
pub(crate) fn ttf(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_TTF_SIZE: u64 = 28; // 12B offset table + >=1 16B table directory record

    require!(size >= MIN_TTF_SIZE, prefix.len() >= 28);
    if starts(prefix, TTCF_MAGIC) {
        let ver_maj = be16(prefix, 4)?;
        let ver_min = be16(prefix, 6)?;
        let num_fonts = be32(prefix, 8)?;
        let first_off = be32(prefix, 12)?;
        require!(
            matches!((ver_maj, ver_min), (1, 0) | (2, 0)),
            (1..=1024).contains(&num_fonts),
            first_off >= 12 + num_fonts * 4,
        );
        return Some(Label::Ttf);
    }
    require!(
        starts(prefix, TTF_SFNT_V1_MAGIC)
            || starts(prefix, TTF_TRUE_MAGIC)
            || starts(prefix, TTF_TYP1_MAGIC),
    );
    let num_tables = be16(prefix, 4)?;
    let search_range = be16(prefix, 6)?;
    require!(
        (1..=256).contains(&num_tables),
        search_range >= 16,
        (search_range / 16).is_power_of_two(),
    );
    let tag = get_slice(prefix, 12, 16)?;
    require!(tag.iter().all(|&b| (0x20..=0x7E).contains(&b)));
    Some(Label::Ttf)
}

/// Adobe / Microsoft OpenType Font (`otf`, 1996) with PostScript CFF outlines:
/// sfnt version `"OTTO"` at `@0`, `numTables` in `1..=256` at `@4`, power-of-two
/// `searchRange` at `@6`, and printable 4-byte table tag at `@12..16`.
pub(crate) fn otf(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_OTF_SIZE: u64 = 28; // 12B sfnt header + >=1 16B table record

    require!(
        size >= MIN_OTF_SIZE,
        prefix.len() >= 28,
        starts(prefix, OTF_MAGIC),
    );
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
