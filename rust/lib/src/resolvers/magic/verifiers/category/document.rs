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

//! Office, page-description, mail, calendar, and help document format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

pub(crate) const MIME_VERSION_HDR: &[u8] = b"MIME-Version:";

pub(crate) const FROM_HDR: &[u8] = b"From:";

const VERSION_PROP: &[u8] = b"VERSION:";

/// Microsoft OneNote revision store file (`.one`): 16-byte file-type GUID at offset 0.
pub(crate) fn one(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = guid!("7B5C52E4-D88C-4DA7-AEB1-5378D02996D3"); // @0..16: OneNote header GUID

    require!(starts(prefix, MAGIC));
    Some(Label::One)
}

/// Windows Help (`.hlp`): magic `0x00035F3F` LE, directory start ≥ 16 at `@4`, no-free-chain
/// sentinel `00 00 FF FF FF FF` at `@6..12`, and entire file size ≥ 16 at `@12`.
pub(crate) fn hlp(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = &0x0003_5F3Fu32.to_le_bytes(); // @0..4: WinHelp magic (3F 5F 03 00)
    const DIR_START: usize = 4; // @4: u32 LE — offset of internal directory (≥ 16)
    const NO_FREE_CHAIN: &[u8] = &[0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF]; // @6..12: high bytes of DirStart + FirstFreeBlock = -1
    const FILE_SIZE: usize = 12; // @12: u32 LE — entire file size (≥ 16)
    const HEADER_LEN: usize = 16;

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, MAGIC),
        eq_at(prefix, 6, NO_FREE_CHAIN),
        le32(prefix, DIR_START)? >= HEADER_LEN as u32,
        le32(prefix, FILE_SIZE)? >= HEADER_LEN as u32,
    );
    Some(Label::Hlp)
}

/// Adobe PostScript Document Structuring Conventions: opens with `"%!PS-Adobe"`.
pub(crate) fn postscript(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"%!PS-Adobe"; // @0..10: conforming PostScript DSC header

    require!(starts(prefix, MAGIC));
    Some(Label::Postscript)
}

/// Truncated 4-byte Rich Text Format magic (`"{\\rt"`, accepted by Microsoft Word).
pub(crate) const RTF_MAGIC: &[u8; 4] = b"{\\rt";

/// Full 5-byte Rich Text Format magic (`"{\\rtf"`).
pub(crate) const RTF_FULL_MAGIC: &[u8; 5] = b"{\\rtf";

/// Microsoft Rich Text Format (`rtf`, 1987): `{\rtf` (or truncated `{\rt` accepted by Word)
/// control word at `@0` followed by version digit or control delimiter.
pub(crate) fn rtf(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_RTF_SIZE: u64 = 6; // "{\rt}\n" or "{\rtf1}"
    let text = prefix.strip_suffix(b"\0").unwrap_or(prefix);

    require!(size >= MIN_RTF_SIZE, prefix.len() >= 6, starts(prefix, RTF_MAGIC),);
    let next_byte =
        if starts(prefix, RTF_FULL_MAGIC) { u8_at(prefix, 5)? } else { u8_at(prefix, 4)? };
    require!(
        matches!(next_byte, b'0'..=b'9' | b'\\' | b'{' | b'}' | b' ' | b'\t' | b'\r' | b'\n'),
        is_utf8_text_header(text, text.len()),
    );
    Some(Label::Rtf)
}

/// Microsoft Compiled HTML Help (`chm`, 1997): `"ITSF"` magic at `@0`, version `2..=4` LE
/// at `@4`, header length `>= 56` LE at `@8`, and unknown_1 `== 1` LE at `@12`.
pub(crate) fn chm(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_CHM_SIZE: u64 = 96; // 56B ITSF v2 / 96B ITSF v3 header
    const CHM_MAGIC: &[u8; 4] = b"ITSF";

    require!(
        size >= MIN_CHM_SIZE,
        prefix.len() >= 56,
        starts(prefix, CHM_MAGIC),
        (2..=4).contains(&le32(prefix, 4)?),
        le32(prefix, 8)? >= 56,
        le32(prefix, 12)? == 1,
    );
    Some(Label::Chm)
}

/// Microsoft Outlook Personal Folders PST / OST (`outlook`, 1996): `"!BDN"` (`21 42 44 4E`)
/// at `@0`, `"SM"` / `"SO"` client magic at `@8..10`, and `wVer` `14..=64` LE at `@10..12`.
pub(crate) fn outlook_pst(prefix: &[u8], size: u64) -> Option<Label> {
    const PST_MAGIC: &[u8; 4] = b"!BDN";
    const PST_CLIENT_SM: &[u8; 2] = b"SM";
    const PST_CLIENT_SO: &[u8; 2] = b"SO";

    require!(
        size >= 512,
        prefix.len() >= 16,
        starts(prefix, PST_MAGIC),
        eq_at(prefix, 8, PST_CLIENT_SM) || eq_at(prefix, 8, PST_CLIENT_SO),
        (14..=64).contains(&le16(prefix, 10)?),
    );
    Some(Label::Outlook)
}

/// Microsoft Word for DOS (`doc`, 1983): `31 BE 00 00 00 AB` at `@0..6`.
pub(crate) fn word_dos(prefix: &[u8], size: u64) -> Option<Label> {
    const WORD_DOS_MAGIC: &[u8] = &[0x31, 0xBE, 0x00, 0x00, 0x00, 0xAB];

    require!(size >= 128, prefix.len() >= 16, starts(prefix, WORD_DOS_MAGIC),);
    Some(Label::Doc)
}

/// Microsoft Word for Windows 1.x / 2.0 (`doc`, 1989): `wIdent` `9B A5` (v1) or `DB A5` (v2)
/// at `@0`, `nFib` in `0x0021..=0x0045` at `@2`, and `lid` language ID at `@6`.
pub(crate) fn word_win12(prefix: &[u8], size: u64) -> Option<Label> {
    const WORD_WIN1_MAGIC: &[u8] = &[0x9B, 0xA5];
    const WORD_WIN2_MAGIC: &[u8] = &[0xDB, 0xA5];

    require!(
        size >= 64,
        prefix.len() >= 32,
        starts(prefix, WORD_WIN1_MAGIC) || starts(prefix, WORD_WIN2_MAGIC),
        (0x0020..=0x0065).contains(&le16(prefix, 2)?),
    );
    Some(Label::Doc)
}

/// Standalone Microsoft Excel BIFF2/3/4/5/8 worksheet/workbook (`xls`, 1987):
/// BOF record opcode (`0x0009`, `0x0209`, `0x0409`, `0x0809` LE) at `@0`, record length
/// `4..=20` LE at `@2`, and valid substream type (`0x0005`, `0x0006`, `0x0010`, `0x0020`,
/// `0x0040`, `0x0100`) at `@6`.
pub(crate) fn xls_biff(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 16,
        prefix.len() >= 12,
        matches!(le16(prefix, 0)?, 0x0009 | 0x0209 | 0x0409 | 0x0809),
        (4..=20).contains(&le16(prefix, 2)?),
        matches!(le16(prefix, 6)?, 0x0005 | 0x0006 | 0x0010 | 0x0020 | 0x0040 | 0x0100),
    );
    Some(Label::Xls)
}

/// RFC 2557 MHTML Web Archive (`mht`, 1999): `"MIME-Version:"` or `"From: "` header with
/// `"Content-Type: multipart/related"`.
pub(crate) fn mht(prefix: &[u8], size: u64) -> Option<Label> {
    const MHT_MIME_1_0: &[u8] = b"MIME-Version: 1.0";
    const MHT_MULTIPART_RELATED: &[u8] = b"multipart/related";
    const MHT_SNAPSHOT_LOC: &[u8] = b"Snapshot-Content-Location:";

    require!(
        size >= 40,
        prefix.len() >= 32,
        starts(prefix, MIME_VERSION_HDR)
            || starts(prefix, FROM_HDR)
            || contains(prefix, MHT_MIME_1_0),
        contains(prefix, MHT_MULTIPART_RELATED) || contains(prefix, MHT_SNAPSHOT_LOC),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Mht)
}

/// Disambiguates MHTML (`mht`) vs RFC 822 Mail (`eml`) for files starting with `"MIME"` or `"From"`.
pub(crate) fn mht_or_eml(prefix: &[u8], size: u64) -> Option<Label> {
    mht(prefix, size).or_else(|| eml(prefix, size))
}

/// Ashton-Tate Framework II / III document (`asd`, 1985): `"ASD01\x1A"` signature at `@0..6`.
pub(crate) fn asd(prefix: &[u8], size: u64) -> Option<Label> {
    const ASD_V01_MAGIC: &[u8; 6] = b"ASD01\x1A";

    require!(size >= 16, prefix.len() >= 12, starts(prefix, ASD_V01_MAGIC),);
    Some(Label::Asd)
}

/// Hancom Hangul Word Processor v3.0 document (`hwp`, 1995): `"HWP Document File V3.00 \x1A"` at `@0..25`.
pub(crate) fn hwp_v3(prefix: &[u8], size: u64) -> Option<Label> {
    const HWP_V3_MAGIC: &[u8; 25] = b"HWP Document File V3.00 \x1A";

    require!(size >= 32, prefix.len() >= 30, starts(prefix, HWP_V3_MAGIC),);
    Some(Label::Hwp)
}

/// IETF RFC 5545 iCalendar (`ics`, 1998) / IMC vCalendar 1.0 (`vcs`, 1996): `"BEGIN:VCALENDAR"`
/// header at `@0` with `"VERSION:"` or `"PRODID:"` property within the first 256 bytes.
pub(crate) fn ics(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_ICS_SIZE: u64 = 28; // "BEGIN:VCALENDAR\nVERSION:2.0\n"
    const VCALENDAR_BEGIN_CRLF: &[u8] = b"BEGIN:VCALENDAR\r\n";
    const VCALENDAR_BEGIN_LF: &[u8] = b"BEGIN:VCALENDAR\n";
    const VCALENDAR_VER_1_0: &[u8] = b"VERSION:1.0";
    const VCALENDAR_PRODID_PROP: &[u8] = b"PRODID:";

    require!(
        size >= MIN_ICS_SIZE,
        prefix.len() >= 24,
        starts(prefix, VCALENDAR_BEGIN_CRLF) || starts(prefix, VCALENDAR_BEGIN_LF),
        is_utf8_text_header(prefix, size as usize),
    );
    if scan_range(prefix, 16, 256, VCALENDAR_VER_1_0) {
        return Some(Label::Vcs);
    }
    require!(
        scan_range(prefix, 16, 256, VERSION_PROP)
            || scan_range(prefix, 16, 256, VCALENDAR_PRODID_PROP)
    );
    Some(Label::Ics)
}

/// IETF RFC 822 / 2822 / 5322 Electronic Mail Message (`eml`, 1982).
pub(crate) fn eml(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_EML_SIZE: u64 = 32;
    const MBOX_FROM_PREFIX: &[u8] = b"From ";
    const HTTP_STATUS_PREFIX: &[u8] = b"HTTP/";
    const STRONG_HDRS: &[&[u8]] = &[
        b"Return-Path:",
        b"Delivered-To:",
        b"Received:",
        b"DKIM-Signature:",
        b"ARC-Seal:",
        b"X-Original-To:",
    ];
    const STD_HDRS: &[&[u8]] = &[
        FROM_HDR,
        b"To:",
        b"Subject:",
        b"Date:",
        b"Message-ID:",
        MIME_VERSION_HDR,
        b"Content-Type:",
        b"Reply-To:",
    ];

    require!(size >= MIN_EML_SIZE, prefix.len() >= 32);
    let win = &prefix[..prefix.len().min(2048)];
    require!(!starts(win, MBOX_FROM_PREFIX) && !starts(win, HTTP_STATUS_PREFIX));

    #[inline]
    fn starts_ci(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.len() >= needle.len() && haystack[..needle.len()].eq_ignore_ascii_case(needle)
    }

    let mut mail_hdrs = 0usize;
    let mut has_strong_mail_hdr = false;
    let mut line_idx = 0usize;
    let mut matched_spans: Vec<(usize, usize)> = Vec::new();
    for raw_line in win.split(|&b| b == b'\n').take(35) {
        let line = raw_line.strip_suffix(b"\r").unwrap_or(raw_line);
        if line.is_empty() {
            break;
        }
        if line_idx > 0 && matches!(line[0], b' ' | b'\t') {
            continue;
        }
        let line_off = (line.as_ptr() as usize) - (win.as_ptr() as usize);
        if let Some(hdr) = STRONG_HDRS.iter().copied().find(|h| starts_ci(line, h)) {
            has_strong_mail_hdr = true;
            mail_hdrs += 1;
            matched_spans.push((line_off, line_off + hdr.len()));
        } else if let Some(hdr) = STD_HDRS.iter().copied().find(|h| starts_ci(line, h)) {
            mail_hdrs += 1;
            matched_spans.push((line_off, line_off + hdr.len()));
        } else if line_idx == 0 {
            return None;
        }
        line_idx += 1;
        if (has_strong_mail_hdr && mail_hdrs >= 2) || mail_hdrs >= 3 {
            break;
        }
    }
    require!(
        (has_strong_mail_hdr && mail_hdrs >= 2) || mail_hdrs >= 3,
        is_utf8_text_header(prefix, size as usize),
    );
    for (s, e) in matched_spans {
        let _ = get_slice(prefix, s, e);
    }
    Some(Label::Eml)
}

/// IETF RFC 6350 / 2426 vCard Contact (`vcard`): `"BEGIN:VCARD"` header at `@0` with `"VERSION:"`.
pub(crate) fn vcard(prefix: &[u8], size: u64) -> Option<Label> {
    const VCARD_BEGIN_CRLF: &[u8] = b"BEGIN:VCARD\r\n";
    const VCARD_BEGIN_LF: &[u8] = b"BEGIN:VCARD\n";

    require!(
        size >= 24,
        prefix.len() >= 22,
        starts(prefix, VCARD_BEGIN_CRLF) || starts(prefix, VCARD_BEGIN_LF),
        scan_range(prefix, 12, 256, VERSION_PROP),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Vcard)
}

/// Disambiguates `"BEGIN:"` between iCalendar (`ics`), vCalendar (`vcs`), and vCard (`vcard`).
#[inline]
pub(crate) fn ics_or_vcard(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| ics(prefix, size)).or_else(|| probe(|| vcard(prefix, size)))
}

/// LaTeX2e / TeX Typesetting Document & Package Source (`latex`, 1984).
pub(crate) fn latex(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_LATEX_SIZE: u64 = 16;
    const LATEX_DOCUMENTCLASS: &[u8] = b"\\documentclass";
    const LATEX_DOCUMENTSTYLE: &[u8] = b"\\documentstyle";
    const LATEX_BEGIN_DOCUMENT: &[u8] = b"\\begin{document}";
    const LATEX_NEEDS_TEX_FORMAT: &[u8] = b"\\NeedsTeXFormat{LaTeX";
    const LATEX_PROVIDES_PACKAGE: &[u8] = b"\\ProvidesPackage{";
    const LATEX_PROVIDES_CLASS: &[u8] = b"\\ProvidesClass{";
    const LATEX_MAGIC_COMMENT: &[u8] = b"%!TEX ";
    const LATEX_USEPACKAGE: &[u8] = b"\\usepackage";
    const LATEX_SECTION: &[u8] = b"\\section";

    require!(size >= MIN_LATEX_SIZE, prefix.len() >= 15);
    let win = &prefix[..prefix.len().min(1024)];
    let has_marker = (memchr::memchr(b'\\', win).is_some()
        && (contains(win, LATEX_DOCUMENTCLASS)
            || contains(win, LATEX_DOCUMENTSTYLE)
            || contains(win, LATEX_BEGIN_DOCUMENT)
            || contains(win, LATEX_NEEDS_TEX_FORMAT)
            || contains(win, LATEX_PROVIDES_PACKAGE)
            || contains(win, LATEX_PROVIDES_CLASS)
            || (contains(win, LATEX_USEPACKAGE) && contains(win, LATEX_SECTION))))
        || contains(win, LATEX_MAGIC_COMMENT);
    require!(has_marker, is_utf8_text_header(prefix, size as usize));
    Some(Label::Latex)
}
