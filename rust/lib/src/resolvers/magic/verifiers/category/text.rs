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

//! Source code, script, markup, and structured text format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// PHP source tag (`<?php` / `<?= `) at `@0`.
pub(crate) fn php_tag(prefix: &[u8], size: u64) -> Option<Label> {
    const PHP_OPEN_TAG: &[u8; 5] = b"<?php";

    require!(
        size >= 8,
        prefix.len() >= 8,
        starts(prefix, PHP_OPEN_TAG)
            && matches!(u8_at(prefix, 5).unwrap_or(b' '), b' ' | b'\t' | b'\r' | b'\n'),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Php)
}

/// MS-DOS / Windows Batch Script (`batch`, 1981): `@echo off`, `@echo on`, `@rem `, or `@setlocal`.
pub(crate) fn batch(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_BATCH_SIZE: u64 = 10; // "@echo off\n"
    const BATCH_ECHO_OFF: &[u8; 9] = b"@echo off";
    const BATCH_ECHO_ON: &[u8; 8] = b"@echo on";
    const BATCH_REM: &[u8; 5] = b"@rem ";
    const BATCH_SETLOCAL: &[u8; 9] = b"@setlocal";

    require!(size >= MIN_BATCH_SIZE, prefix.len() >= 10);
    let win = &prefix[..prefix.len().min(512)];
    let trimmed = win.trim_ascii_start();
    let trim_off = (trimmed.as_ptr() as usize) - (win.as_ptr() as usize);
    let matched_len = if trimmed.len() >= BATCH_ECHO_OFF.len()
        && trimmed[..BATCH_ECHO_OFF.len()].eq_ignore_ascii_case(BATCH_ECHO_OFF)
    {
        Some(BATCH_ECHO_OFF.len())
    } else if trimmed.len() >= BATCH_ECHO_ON.len()
        && trimmed[..BATCH_ECHO_ON.len()].eq_ignore_ascii_case(BATCH_ECHO_ON)
    {
        Some(BATCH_ECHO_ON.len())
    } else if trimmed.len() >= BATCH_REM.len()
        && trimmed[..BATCH_REM.len()].eq_ignore_ascii_case(BATCH_REM)
    {
        Some(BATCH_REM.len())
    } else if trimmed.len() >= BATCH_SETLOCAL.len()
        && trimmed[..BATCH_SETLOCAL.len()].eq_ignore_ascii_case(BATCH_SETLOCAL)
    {
        Some(BATCH_SETLOCAL.len())
    } else {
        None
    };
    let len = matched_len?;
    require!(is_utf8_text_header(prefix, size as usize));
    let _ = get_slice(prefix, trim_off, trim_off + len);
    Some(Label::Batch)
}

/// Hunspell / MySpell affix dictionary file (`.aff`, `aff`, 2002): `"SET UTF-8"` / `"SET ISO8859-"`
/// / `"SET KOI8-"` / `"SET microsoft-cp"` encoding directive followed by `\nTRY `, `\nSFX `, or `\nPFX `.
pub(crate) fn aff_hunspell(prefix: &[u8], size: u64) -> Option<Label> {
    const HUNSPELL_SET_UTF8: &[u8] = b"SET UTF-8";
    const HUNSPELL_SET_ISO8859: &[u8] = b"SET ISO8859-";
    const HUNSPELL_SET_KOI8: &[u8] = b"SET KOI8-";
    const HUNSPELL_SET_MSCP: &[u8] = b"SET microsoft-cp";
    const HUNSPELL_TRY_RULE: &[u8] = b"\nTRY ";
    const HUNSPELL_SFX_RULE: &[u8] = b"\nSFX ";
    const HUNSPELL_PFX_RULE: &[u8] = b"\nPFX ";

    require!(
        size >= 24,
        prefix.len() >= 24,
        starts(prefix, HUNSPELL_SET_UTF8)
            || starts(prefix, HUNSPELL_SET_ISO8859)
            || starts(prefix, HUNSPELL_SET_KOI8)
            || starts(prefix, HUNSPELL_SET_MSCP),
        contains(prefix, HUNSPELL_TRY_RULE)
            || contains(prefix, HUNSPELL_SFX_RULE)
            || contains(prefix, HUNSPELL_PFX_RULE),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Aff)
}

/// Martin Richards BCPL source code (`bpl`, 1967): `"//"` comment header with `"GLOBAL $("` or `"GET \"LIBHDR\""`.
pub(crate) fn bpl_source(prefix: &[u8], size: u64) -> Option<Label> {
    const BCPL_COMMENT_PREFIX: &[u8] = b"//";
    const BCPL_GLOBAL_DECL: &[u8] = b"GLOBAL $(";
    const BCPL_GET_LIBHDR: &[u8] = b"GET \"LIBHDR\"";

    require!(
        size >= 20,
        prefix.len() >= 20,
        starts(prefix, BCPL_COMMENT_PREFIX)
            && (contains(prefix, BCPL_GLOBAL_DECL) || contains(prefix, BCPL_GET_LIBHDR)),
        !prefix[..prefix.len().min(512)].contains(&0),
    );
    Some(Label::Bpl)
}

/// LLVM Intermediate Representation assembly (`.ll`, `llvm_ir`, 2003): `"; ModuleID = "`,
/// `"source_filename = \""`, or `"target datalayout = \""` header with `target triple`, `define `, or `declare `.
pub(crate) fn llvm_ir(prefix: &[u8], size: u64) -> Option<Label> {
    const LLVM_MODULE_ID: &[u8] = b"; ModuleID = ";
    const LLVM_SOURCE_FILENAME: &[u8] = b"source_filename = \"";
    const LLVM_TARGET_DATALAYOUT: &[u8] = b"target datalayout = \"";
    const LLVM_TARGET_TRIPLE: &[u8] = b"target triple = \"";
    const LLVM_DEFINE_KW: &[u8] = b"define ";
    const LLVM_DECLARE_KW: &[u8] = b"declare ";

    require!(
        size >= 24,
        prefix.len() >= 24,
        (starts(prefix, LLVM_MODULE_ID)
            || starts(prefix, LLVM_SOURCE_FILENAME)
            || starts(prefix, LLVM_TARGET_DATALAYOUT))
            && (contains(prefix, LLVM_TARGET_TRIPLE)
                || contains(prefix, LLVM_DEFINE_KW)
                || contains(prefix, LLVM_DECLARE_KW)),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::LlvmIr)
}

/// Windows Script Encoder (`vbe` / `jse`): `"#@~^"` marker at `@0..4`, 6 Base64 length
/// characters (`[A-Za-z0-9+/]{6}`) at `@4..10`, `"=="` at `@10..12`, and valid text/script
/// buffer encoding (printable bytes + `\t`/`\n`/`\r`, optional trailing `0x00`/`0x1A` at EOF,
/// and `< 50%` high bytes unless valid UTF-8).
pub(crate) fn vbe(prefix: &[u8], size: u64) -> Option<Label> {
    const VBE_START_MARKER: &[u8; 4] = b"#@~^";
    const VBE_LEN_SUFFIX: &[u8; 2] = b"==";

    require!(
        size >= 16,
        prefix.len() >= 16,
        starts(prefix, VBE_START_MARKER),
        get_slice(prefix, 4, 10)?
            .iter()
            .all(|&b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/')),
        eq_at(prefix, 10, VBE_LEN_SUFFIX),
    );
    let mut end = prefix.len();
    if prefix.len() as u64 == size {
        while end > 12 && matches!(prefix[end - 1], 0x00 | 0x1A) {
            end -= 1;
        }
    }
    require!(end >= 16, is_mostly_printable_header(&prefix[..end], end));
    Some(Label::Vbe)
}

/// OGC Well-Known Text geometry or spatial reference system (`wkt`, 1999): `POINT`, `LINESTRING`,
/// `POLYGON`, `MULTIPOLYGON`, `GEOMETRYCOLLECTION`, `PROJCS["`, or `GEOGCS["` expression.
pub(crate) fn wkt(prefix: &[u8], size: u64) -> Option<Label> {
    const WKT_POINT_SP: &[u8] = b"POINT (";
    const WKT_POINT_NOSP: &[u8] = b"POINT(";
    const WKT_LINESTRING_SP: &[u8] = b"LINESTRING (";
    const WKT_LINESTRING_NOSP: &[u8] = b"LINESTRING(";
    const WKT_POLYGON_SP: &[u8] = b"POLYGON ((";
    const WKT_POLYGON_NOSP: &[u8] = b"POLYGON((";
    const WKT_MULTIPOLYGON_SP: &[u8] = b"MULTIPOLYGON (((";
    const WKT_GEOMETRYCOLLECTION_SP: &[u8] = b"GEOMETRYCOLLECTION (";
    const WKT_PROJCS_OPEN: &[u8] = b"PROJCS[\"";
    const WKT_GEOGCS_OPEN: &[u8] = b"GEOGCS[\"";

    require!(
        size >= 10,
        prefix.len() >= 10,
        (starts(prefix, WKT_POINT_SP)
            || starts(prefix, WKT_POINT_NOSP)
            || starts(prefix, WKT_LINESTRING_SP)
            || starts(prefix, WKT_LINESTRING_NOSP)
            || starts(prefix, WKT_POLYGON_SP)
            || starts(prefix, WKT_POLYGON_NOSP)
            || starts(prefix, WKT_MULTIPOLYGON_SP)
            || starts(prefix, WKT_GEOMETRYCOLLECTION_SP)
            || starts(prefix, WKT_PROJCS_OPEN)
            || starts(prefix, WKT_GEOGCS_OPEN))
            && (contains(prefix, b")") || contains(prefix, b"]")),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Wkt)
}
