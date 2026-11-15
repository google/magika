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
    static ITSF_V2: Subformat = Subformat::new("CHM (ITSF v2)");
    static ITSF_V3: Subformat = Subformat::new("CHM (ITSF v3)");
    static ITSF_V4: Subformat = Subformat::new("CHM (ITSF v4)");
    const MIN_CHM_SIZE: u64 = 96; // 56B ITSF v2 / 96B ITSF v3 header
    const CHM_MAGIC: &[u8; 4] = b"ITSF";

    require!(size >= MIN_CHM_SIZE, prefix.len() >= 56, starts(prefix, CHM_MAGIC));
    let sub = match le32(prefix, 4)? {
        2 => &ITSF_V2,
        3 => &ITSF_V3,
        4 => &ITSF_V4,
        _ => return None,
    };
    require!(le32(prefix, 8)? >= 56, le32(prefix, 12)? == 1);
    with_subformat(Label::Chm, sub)
}

/// Microsoft Outlook Personal Folders PST / OST (`outlook`, 1996): `"!BDN"` (`21 42 44 4E`)
/// at `@0`, `"SM"` / `"SO"` client magic at `@8..10`, and `wVer` `14..=64` LE at `@10..12`.
pub(crate) fn outlook_pst(prefix: &[u8], size: u64) -> Option<Label> {
    static OUTLOOK_PST_ANSI: Subformat =
        Subformat::with_ids("Outlook Personal Folders (PST, ANSI)", "x-fmt/248", Some("fdd000377"));
    static OUTLOOK_PST_UNICODE: Subformat = Subformat::with_ids(
        "Outlook Personal Folders (PST, Unicode)",
        "x-fmt/249",
        Some("fdd000378"),
    );
    static OUTLOOK_PST: Subformat = Subformat::new("Outlook Personal Folders (PST)");
    static OUTLOOK_OST: Subformat = Subformat::new("Outlook Offline Folders (OST)");
    const PST_MAGIC: &[u8; 4] = b"!BDN";
    const PST_CLIENT_SM: &[u8; 2] = b"SM";
    const PST_CLIENT_SO: &[u8; 2] = b"SO";

    require!(size >= 512, prefix.len() >= 16, starts(prefix, PST_MAGIC));
    let is_pst = if eq_at(prefix, 8, PST_CLIENT_SM) {
        true
    } else if eq_at(prefix, 8, PST_CLIENT_SO) {
        false
    } else {
        return None;
    };
    let ver = le16(prefix, 10)?;
    require!((14..=64).contains(&ver));
    let sub = if !is_pst {
        &OUTLOOK_OST
    } else if ver <= 15 {
        &OUTLOOK_PST_ANSI
    } else if ver >= 23 {
        &OUTLOOK_PST_UNICODE
    } else {
        &OUTLOOK_PST
    };
    with_subformat(Label::Outlook, sub)
}

/// Microsoft Word for DOS (`doc`, 1983): `31 BE 00 00 00 AB` at `@0..6`.
pub(crate) fn word_dos(prefix: &[u8], size: u64) -> Option<Label> {
    static WORD_DOS: Subformat = Subformat::new("Microsoft Word for MS-DOS");
    const WORD_DOS_MAGIC: &[u8] = &[0x31, 0xBE, 0x00, 0x00, 0x00, 0xAB];

    require!(size >= 128, prefix.len() >= 16, starts(prefix, WORD_DOS_MAGIC),);
    with_subformat(Label::Doc, &WORD_DOS)
}

/// Microsoft Word for Windows 1.x / 2.0 (`doc`, 1989): `wIdent` `9B A5` (v1) or `DB A5` (v2)
/// at `@0`, `nFib` in `0x0021..=0x0045` at `@2`, and `lid` language ID at `@6`.
pub(crate) fn word_win12(prefix: &[u8], size: u64) -> Option<Label> {
    static WORD_WIN_1: Subformat =
        Subformat::with_ids("Microsoft Word for Windows 1.x", "fmt/37", None);
    static WORD_WIN_2: Subformat =
        Subformat::with_ids("Microsoft Word for Windows 2.0", "fmt/38", None);
    const WORD_WIN1_MAGIC: &[u8] = &[0x9B, 0xA5];
    const WORD_WIN2_MAGIC: &[u8] = &[0xDB, 0xA5];

    require!(size >= 64, prefix.len() >= 32);
    let sub = if starts(prefix, WORD_WIN1_MAGIC) {
        &WORD_WIN_1
    } else if starts(prefix, WORD_WIN2_MAGIC) {
        &WORD_WIN_2
    } else {
        return None;
    };
    require!((0x0020..=0x0065).contains(&le16(prefix, 2)?));
    with_subformat(Label::Doc, sub)
}

/// Standalone Microsoft Excel BIFF2/3/4/5/8 worksheet/workbook (`xls`, 1987):
/// BOF record opcode (`0x0009`, `0x0209`, `0x0409`, `0x0809` LE) at `@0`, record length
/// `4..=20` LE at `@2`, and valid substream type (`0x0005`, `0x0006`, `0x0010`, `0x0020`,
/// `0x0040`, `0x0100`) at `@6`.
pub(crate) fn xls_biff(prefix: &[u8], size: u64) -> Option<Label> {
    static EXCEL_BIFF2: Subformat =
        Subformat::with_ids("Microsoft Excel 2.x (BIFF2)", "fmt/55", None);
    static EXCEL_BIFF3: Subformat =
        Subformat::with_ids("Microsoft Excel 3.0 (BIFF3)", "fmt/56", None);
    static EXCEL_BIFF4_SHEET: Subformat =
        Subformat::with_ids("Microsoft Excel 4.0 Worksheet (BIFF4)", "fmt/57", None);
    static EXCEL_BIFF4_BOOK: Subformat =
        Subformat::with_ids("Microsoft Excel 4.0 Workbook (BIFF4)", "fmt/58", None);
    static EXCEL_BIFF5_8: Subformat = Subformat::new("Microsoft Excel BIFF5/BIFF8 (standalone)");

    require!(size >= 16, prefix.len() >= 12);
    let opcode = le16(prefix, 0)?;
    require!(
        matches!(opcode, 0x0009 | 0x0209 | 0x0409 | 0x0809),
        (4..=20).contains(&le16(prefix, 2)?),
    );
    let substream = le16(prefix, 6)?;
    require!(matches!(substream, 0x0005 | 0x0006 | 0x0010 | 0x0020 | 0x0040 | 0x0100));
    let sub = match opcode {
        0x0009 => &EXCEL_BIFF2,
        0x0209 => &EXCEL_BIFF3,
        0x0409 if substream == 0x0100 => &EXCEL_BIFF4_BOOK,
        0x0409 => &EXCEL_BIFF4_SHEET,
        0x0809 => &EXCEL_BIFF5_8,
        _ => unreachable!(),
    };
    with_subformat(Label::Xls, sub)
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
    static HWP_V3: Subformat = Subformat::with_ids("HWP 3.0", "fmt/1083", None);
    const HWP_V3_MAGIC: &[u8; 25] = b"HWP Document File V3.00 \x1A";

    require!(size >= 32, prefix.len() >= 30, starts(prefix, HWP_V3_MAGIC),);
    with_subformat(Label::Hwp, &HWP_V3)
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

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const BORLAND_TV_HELP_FBHF_MAGIC: &[u8; 4] = b"FBHF";
const BORLAND_TV_HELP_FBRW_MAGIC: &[u8; 4] = b"FBRW";
const BORLAND_TV_HELP_THELPTOPIC_SIG: &[u8; 13] = b"\x02[\x0aTHelpTopic";
const CALS_DCL_SRCDOCID: &[u8; 9] = b"srcdocid:";
const CALS_DCL_SRCSYS: &[u8; 7] = b"srcsys:";
const CXL_HELP_MAGIC: &[u8; 4] = bytes!(b"*I", &[0x0D, 0x0A]);
const FOURDOS_HELP_V4_DOS_MAGIC: &[u8; 4] = b"4DH4";
const FOURDOS_HELP_V4_NT_MAGIC: &[u8; 4] = b"4NH4";
const FOURDOS_HELP_V4_OS2_MAGIC: &[u8; 4] = b"4OH4";
const FOURDOS_HELP_V5_SIG_LEN: u8 = 0x08;
const GUEMAP_MAGIC: &[u8; 8] = hex2bin!("C7 55 C5 6D E1 70 83 0D");
const INDESIGN_GUID: &[u8; 16] = &[
    0x06, 0x06, 0xED, 0xF5, 0xD8, 0x1D, 0x46, 0xE5, 0xBD, 0x31, 0xEF, 0xE7, 0xFE, 0x74, 0xB7, 0x1D,
];
const MIN_ACTA_OUTLINE_SIZE: u64 = 32;
const MIN_ADOBE_TECHPOOL_TUTORIAL_TPP_SIZE: u64 = 32;
const MIN_ATARI_ST_GUIDE_SIZE: u64 = 32;
const MIN_BAHN_NT3_SIZE: u64 = 80;
const MIN_BARTENDER_BTW_SIZE: u64 = 800;
const MIN_BEAGLEWORKS_SIZE: u64 = 64;
const MIN_BIZTREE_BTD_SIZE: u64 = 64;
const MIN_BORLAND_TB_TBH_SIZE: u64 = 32;
const MIN_BORLAND_TURBO_DEBUG_HELP_TDH_SIZE: u64 = 64;
const MIN_BORLAND_TURBO_HELP_TCH_SIZE: u64 = 64;
const MIN_BWD1_DOC_SIZE: u64 = 28;
const MIN_CALS_DCL_SIZE: u64 = 28;
const MIN_CEDIT_UCVIEW_HIS_SIZE: u64 = 64;
const MIN_CEDIT_UCVIEW_HLP_SIZE: u64 = 64;
const MIN_CHEMDRAW_CDX_SIZE: u64 = 16;
const MIN_CLARION_HLP_SIZE: u64 = 32;
const MIN_CLARIS_RESOLVE_SIZE: u64 = 48;
const MIN_CORELDRAW_CPX_SIZE: u64 = 16;
const MIN_CRYSTAL_REPORTS_V4_RPT_SIZE: u64 = 64;
const MIN_CXL_HELP_SIZE: u64 = 76;
const MIN_CZ_HELP_SIZE: u64 = 32;
const MIN_DELPHI_CARD_PUBLISHER_LCP_SIZE: u64 = 32;
const MIN_DESI_DDF_SIZE: u64 = 128;
const MIN_DESQVIEW_HLP_SIZE: u64 = 32;
const MIN_DISPLAYWRITE_DCA_SIZE: u64 = 14;
const MIN_DOS_EPS_SIZE: u64 = 30;
const MIN_ENDNOTE_ENF_SIZE: u64 = 36;
const MIN_ENVISION_SIZE: u64 = 64;
const MIN_ENVOY_SIZE: u64 = 16;
const MIN_EXPRESS_PUBLISHER_SIZE: u64 = 16;
const MIN_FOURDOS_HELP_SIZE: u64 = 64;
const MIN_FREEBASIC_HF_SIZE: u64 = 32;
const MIN_FULLWRITE_SIZE: u64 = 64;
const MIN_GARMIN_FIT_SIZE: u64 = 14;
const MIN_GUEMAP_SIZE: u64 = 16;
const MIN_HANMAC_WORD_SIZE: u64 = 64;
const MIN_HARVARD_GRAPHICS_SIZE: u64 = 16;
const MIN_HYPERWRITER_SIZE: u64 = 32;
const MIN_IBM_DISPLAYWRITE_SIZE: u64 = 62;
const MIN_IBM_OS2_MKMSGF_MSG_SIZE: u64 = 32;
const MIN_INDESIGN_SIZE: u64 = 8192;
const MIN_ION_PML_DGM_SIZE: u64 = 36;
const MIN_KAPEL_MDF_SIZE: u64 = 512;
const MIN_LIVECODE_SIZE: u64 = 16;
const MIN_LOCOSCRIPT_SIZE: u64 = 64;
const MIN_LOTUS_123_INSTALL_DAT_SIZE: u64 = 32;
const MIN_MACWRITE2_SIZE: u64 = 32;
const MIN_MAC_WORD_SIZE: u64 = 64;
const MIN_MANUSCRIPT_SIZE: u64 = 16;
const MIN_MARINER_WRITE_SIZE: u64 = 32;
const MIN_MCS_STIMULUS_STM_SIZE: u64 = 36;
const MIN_MILESTONES_MLB_SIZE: u64 = 36;
const MIN_MORE_OUTLINE_SIZE: u64 = 48;
const MIN_MOSHIDRAW_YZ_SIZE: u64 = 24;
const MIN_MOUSEWRITE_SIZE: u64 = 32;
const MIN_MS_ADVISOR_HELP_SIZE: u64 = 74;
const MIN_MS_PUBLISHER_BORDERART_BDR_SIZE: u64 = 32;
const MIN_MS_PUBLISHER_PAGEWIZARD_POC_SIZE: u64 = 32;
const MIN_MS_PUBLISHER_PREVIEW_PRV_SIZE: u64 = 32;
const MIN_MS_WORKS_DOS_WPS_SIZE: u64 = 128;
const MIN_MULTIMATE_SIZE: u64 = 24;
const MIN_MULTIPLAN_SIZE: u64 = 16;
const MIN_NBI_SIZE: u64 = 16;
const MIN_NORTON_GUIDES_HLP_SIZE: u64 = 64;
const MIN_NOVELL_NETWARE_MESSAGE_MSG_SIZE: u64 = 48;
const MIN_NOVELL_SYS_HELP_HEP_SIZE: u64 = 160;
const MIN_OBERON_ODC_SIZE: u64 = 40;
const MIN_ONENOTE_TOC2_SIZE: u64 = 1024;
const MIN_OS2_HELP_INF_SIZE: u64 = 144;
const MIN_PCTOOLS_MSG_SIZE: u64 = 32;
const MIN_PLATO_LESSON_SIZE: u64 = 32;
const MIN_POWERBUILDER_PSR_SIZE: u64 = 48;
const MIN_PRONOM_ADOBE_SWATCH_EXCHANGE_ASE_SIZE: u64 = 16;
const MIN_PRONOM_ADRIFT_TAF_SIZE: u64 = 20;
const MIN_PRONOM_ADVANCED_FORENSIC_FORMAT_AFF_SIZE: u64 = 16;
const MIN_PRONOM_ASYLUM_MUSIC_FORMAT_AMF_SIZE: u64 = 23;
const MIN_PRONOM_ATARI_MPP_SIZE: u64 = 16;
const MIN_PRONOM_AUTORUN_MAESTRO_MENU_FILE_MNU_SIZE: u64 = 16;
const MIN_PRONOM_AVCHD_THUMBNAIL_INDEX_FILE_TID_SIZE: u64 = 16;
const MIN_PRONOM_BIM_METADATA_FILE_SIZE: u64 = 90;
const MIN_PRONOM_BIOSEMI_24_BIT_BDF_SIZE: u64 = 16;
const MIN_PRONOM_BLITZ3D_FILE_FORMAT_B3D_SIZE: u64 = 16;
const MIN_PRONOM_CATIA_DRAWING_SIZE: u64 = 16;
const MIN_PRONOM_CHEMSTATION_GAS_CHROMATOGRAPHY_MASS_MS_SIZE: u64 = 256;
const MIN_PRONOM_COMMON_INSTRUMENT_CI1_SIZE: u64 = 40;
const MIN_PRONOM_COMMON_LOUDSPEAKER_CF1_SIZE: u64 = 40;
const MIN_PRONOM_DIFFRACPLUS_RAW_DATA_SIZE: u64 = 16;
const MIN_PRONOM_DNA_SEQUENCE_CHROMATOGRAM_FILE_SCF_SIZE: u64 = 16;
const MIN_PRONOM_FINEPRINT_FP_SIZE: u64 = 16;
const MIN_PRONOM_G9B_BITMAP_SIZE: u64 = 16;
const MIN_PRONOM_GARMIN_GMN_SIZE: u64 = 16;
const MIN_PRONOM_GSSI_DZT_SIZE: u64 = 1024;
const MIN_PRONOM_GX2_BITMAP_SIZE: u64 = 20;
const MIN_PRONOM_HAIKU_VECTOR_ICON_FORMAT_HVIF_SIZE: u64 = 16;
const MIN_PRONOM_INFOFILE_FLR_SIZE: u64 = 28;
const MIN_PRONOM_INTERLEAVED_ADX_AUDIO_AIX_SIZE: u64 = 20;
const MIN_PRONOM_LEADTOOLS_CMP_SIZE: u64 = 24;
const MIN_PRONOM_LEONARDO_IMAGE_FORMAT_LEO_SIZE: u64 = 20;
const MIN_PRONOM_MAPSFORGE_BINARY_VECTOR_MAP_SIZE: u64 = 24;
const MIN_PRONOM_MAR_ARCHIVE_MAC_SIZE: u64 = 16;
const MIN_PRONOM_PLEXTALK_PROJECT_FILE_IMDN_SIZE: u64 = 84;
const MIN_PRONOM_PORTABLE_DATABASE_PDB_SIZE: u64 = 16;
const MIN_PRONOM_PORTABLE_FORM_FILE_PFF_SIZE: u64 = 16;
const MIN_PRONOM_POWERVR_OBJECT_DATA_POD_SIZE: u64 = 31;
const MIN_PRONOM_QUICKEN_3_DATABASE_FILE_QST_SIZE: u64 = 20;
const MIN_PRONOM_Q_A_WORD_PROCESSOR_SIZE: u64 = 16;
const MIN_PRONOM_ROS_BAG_FORMAT_SIZE: u64 = 17;
const MIN_PRONOM_SERIF_PAGEPLUS_PUBLICATION_PPP_SIZE: u64 = 16;
const MIN_PRONOM_SPRITEPAD_SPD_SIZE: u64 = 20;
const MIN_PRONOM_SUPERSCAPE_VIRTUAL_REALITY_FORMAT_SVR_SIZE: u64 = 16;
const MIN_PRONOM_TRUE_COLOUR_PICTURE_SPOOKY_TRP_SIZE: u64 = 16;
const MIN_PRONOM_TUNDRA_TND_SIZE: u64 = 16;
const MIN_PRONOM_VIPS_IMAGE_V_SIZE: u64 = 16;
const MIN_PRONOM_WINDOWS_ADDRESS_BOOK_WAB_SIZE: u64 = 20;
const MIN_PRONOM_XAR_IMAGE_FORMAT_SIZE: u64 = 16;
const MIN_PRONOM_XL_PAINT_MAX_SIZE: u64 = 16;
const MIN_PRONOM_XL_PAINT_RAW_SIZE: u64 = 16;
const MIN_PRONOM_XV_THUMBNAIL_P7_SIZE: u64 = 17;
const MIN_PRONOM_ZOOMBROWSER_EX_THUMBNAIL_CACHE_INFO_SIZE: u64 = 16;
const MIN_PSION3_WORD_SIZE: u64 = 32;
const MIN_QUATTRO_PRO_WB1_SIZE: u64 = 28;
const MIN_RAGTIME_SIZE: u64 = 64;
const MIN_RILLSOFT_RPJ_SIZE: u64 = 36;
const MIN_SANKOM_OZD_SIZE: u64 = 32;
const MIN_SCREENWRITER_SIZE: u64 = 16;
const MIN_SHOWPARTNER_HLP_SIZE: u64 = 32;
const MIN_SIGNUM_SIZE: u64 = 64;
const MIN_SOFTPLAN_SPV_SIZE: u64 = 40;
const MIN_STARWRITER_LEGACY_VOR_SIZE: u64 = 600;
const MIN_STUDENT_WRITING_CENTER_SIZE: u64 = 64;
const MIN_TAXPREP_TFD_SIZE: u64 = 44;
const MIN_TNEF_SIZE: u64 = 17;
const MIN_TURBOPOWER_POPHELP_SIZE: u64 = 44;
const MIN_TURBOPOWER_TPH2_SIZE: u64 = 64;
const MIN_TURBOPOWER_TPHELP_SIZE: u64 = 60;
const MIN_WINDOWS_CAL_SIZE: u64 = 64;
const MIN_WINOLS_KP_SIZE: u64 = 28;
const MIN_WOODWORKS_CWB_SIZE: u64 = 28;
const MIN_WORDSTAR_2000_SIZE: u64 = 64;
const MIN_WPSPLUS_SIZE: u64 = 16;
const MIN_ZBR1BGJB_PRN_SIZE: u64 = 64;
const MS_ADVISOR_HELP_MAGIC: &[u8; 4] = bytes!(b"LN", le16_bytes!(2));
const ONENOTE_TOC2_FILE_TYPE_GUID: &[u8; 16] = guid!("43FF2FA1-EFD9-4C76-9EE2-10EA5722765F");
const ONENOTE_TOC2_FORMAT_GUID: &[u8; 16] = guid!("109ADD3F-911B-49F5-A5D0-1791EDC8AED8");
const TURBOPOWER_POPHELP_MAGIC: &[u8; 4] = b"INFO";
const TURBOPOWER_TPH2_MAGIC: &[u8; 4] = b"TPH2";
const TURBOPOWER_TPHELP_MAGIC: &[u8; 4] = b"RMP1";

/// Computes the Garmin FIT 16-bit CRC (nibble-driven CRC-16/ARC with polynomial `0x8005` reflected).
#[inline]
fn crc16_fit(data: &[u8]) -> u16 {
    const CRC_TABLE: [u16; 16] = [
        0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401, 0xA001, 0x6C00, 0x7800,
        0xB401, 0x5000, 0x9C01, 0x8801, 0x4400,
    ];
    let mut crc = 0u16;
    for &byte in data {
        let tmp = CRC_TABLE[(crc & 0xF) as usize];
        crc = (crc >> 4) & 0x0FFF;
        crc = crc ^ tmp ^ CRC_TABLE[(byte & 0xF) as usize];

        let tmp = CRC_TABLE[(crc & 0xF) as usize];
        crc = (crc >> 4) & 0x0FFF;
        crc = crc ^ tmp ^ CRC_TABLE[((byte >> 4) & 0xF) as usize];
    }
    crc
}

/// Symmetry Acta / Acta Advantage Macintosh Outline (`.acta`).
pub(crate) fn verify_acta_outline_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ACTA_OUTLINE_SIZE,
        prefix.len() >= 28,
        matches!(be16(prefix, 0)?, 1..=3),
        be16(prefix, 2)? == 1,
        be16(prefix, 4)? == 1,
        be32(prefix, 6)? == 0x0000_0010,
        be32(prefix, 10)? == 0x0000_000C,
        get_slice(prefix, 14, 22)?.iter().all(|&b| b == 0)
    );
    let topic_len = be16(prefix, 22)? as usize;
    require!((1..=4096).contains(&topic_len), 24 + (topic_len as u64) <= size);
    let check_len = topic_len.min(prefix.len() - 24).min(32);
    require!(
        check_len > 0,
        get_slice(prefix, 24, 24 + check_len)?
            .iter()
            .all(|&b| b == b'\r' || b == b'\t' || (0x20..=0x7E).contains(&b) || b >= 0x80)
    );
    Some(Label::ActaOutline)
}

/// Adobe / TechPool Interactive Tutorial Presentation (`.tpp`, 1995) TechPool Studios / Adobe.
pub(crate) fn verify_adobe_techpool_tutorial_tpp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ADOBE_TECHPOOL_TUTORIAL_TPP_SIZE,
        prefix.len() >= 16,
        starts(prefix, &[0xFE, 0xCA, 0xBE, 0xBA]),
        eq_at(prefix, 6, b"N8")
    );
    Some(Label::AdobeTechpoolTutorialTpp)
}

/// Atari ST-Guide Compiled Hypertext (`.hyp`, `"HDOC"`).
pub(crate) fn verify_atari_st_guide_hyp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ATARI_ST_GUIDE_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"HDOC"),
        be32(prefix, 4)? > 0,
        be16(prefix, 8)? > 0,
        u8_at(prefix, 10)? <= 4,
        u8_at(prefix, 11)? <= 5
    );
    Some(Label::AtariStGuideHyp)
}

/// JB Software BAHN 3.2 Railway Simulation Network Layout (.nt3, 1995) Jan Bochmann.
pub(crate) fn verify_bahn_nt3(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BAHN_NT3_SIZE,
        prefix.len() >= 80,
        starts(prefix, b"JB Software Saxony BAHN 3.2 Netzdatei\r\n"),
        u8_at(prefix, 74)? == 0x1A
    );
    Some(Label::BahnNt3)
}

/// Seagull Scientific BarTender Barcode Label Document (.btw, 1992) Seagull Scientific.
pub(crate) fn verify_bartender_btw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BARTENDER_BTW_SIZE,
        prefix.len() >= 800,
        starts(prefix, b"\r\nBar Tender Format File\r\n(c) 1992"),
        has_non_ascii_text_byte(prefix.get(760..800)?)
    );
    Some(Label::BartenderBtw)
}

/// Beagle Bros BeagleWorks / WordPerfect Works Document (`"BWks"` + repeated module tag).
pub(crate) fn verify_beagleworks_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BEAGLEWORKS_SIZE, prefix.len() >= 18, starts(prefix, b"BWks"));
    let module = get_slice(prefix, 4, 8)?;
    require!(
        matches!(module, b"BWwp" | b"BWss" | b"BWdb" | b"BWdr" | b"BWpt"),
        get_slice(prefix, 8, 12)? == module,
        get_slice(prefix, 12, 18)? == [0x00, 0x00, 0x00, 0x00, 0x00, 0x01]
    );
    Some(Label::BeagleworksDoc)
}

/// BizTree Business-in-a-Box Encrypted Document Template (.btd).
pub(crate) fn verify_biztree_btd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BIZTREE_BTD_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 8)? == hex2bin!("CB E0 10 39 05 40 07 5A"),
        get_slice(prefix, 16, 32)? == hex2bin!("04 91 9C 2F EC D1 20 AC 27 4A CA 7F 40 08 3B BE")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::BiztreeBtd)
}

/// Borland Turbo Basic Help File (`"TURBO BASIC HELP FILE.\0\x1A"` or `"TURBO BASIC HELP FILE\r\n\x1A"`).
pub(crate) fn verify_borland_tb_tbh(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BORLAND_TB_TBH_SIZE,
        starts(prefix, b"TURBO BASIC HELP FILE.\x00\x1A")
            || starts(prefix, b"TURBO BASIC HELP FILE\r\n\x1A")
    );
    Some(Label::TbTbh)
}

/// Borland Turbo Debugger Help File (`.tdh`, 1988) Borland.
pub(crate) fn verify_borland_turbo_debug_help_tdh(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BORLAND_TURBO_DEBUG_HELP_TDH_SIZE,
        prefix.len() >= 24,
        eq_at(prefix, 2, &[0x00, 0x01]),
        eq_at(prefix, 11, &[0xC4, 0x00, 0xC7, 0x00, 0xBC, 0x00])
            || eq_at(prefix, 12, &[0xC4, 0x00, 0xC7, 0x00, 0xBC, 0x00])
    );
    Some(Label::BorlandTurboDebugHelpTdh)
}

/// Borland Turbo C / Turbo Assembler Help File (`.tch` / `.tah`, 1988) Borland.
pub(crate) fn verify_borland_turbo_help_tch(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BORLAND_TURBO_HELP_TCH_SIZE,
        prefix.len() >= 64,
        memchr::memmem::find(get_slice(prefix, 0, 64)?, b"HELP FILE.\x00\x1A$*$* &&&&$").is_some()
    );
    Some(Label::BorlandTurboHelpTch)
}

/// BWD1 Structured Document (.bwd).
pub(crate) fn verify_bwd1_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BWD1_DOC_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"BWD1\0\0\0\0info"),
        le32(prefix, 12).unwrap_or(0) > 0
    );
    Some(Label::Bwd1Doc)
}

/// CALS MIL-STD-1840 raster declaration file (`.dcl`, 1991, e.g. Formtek CALS toolkit).
///
/// A sequence of records, each `u32 BE length` + ASCII `keyword: value` (`srcsys`, `srcdocid`, `srcrelid`,
/// `chglvl`, `dteisu`, `dstsys`, ..., `doccls`, `docttl`), companion of the `.ras` CALS Type 1 raster.
pub(crate) fn verify_cals_declaration(prefix: &[u8], size: u64) -> Option<Label> {
    const LEN_SIZE: usize = 4; // u32 BE record length prefix
    const MAX_RECORD_LEN: u32 = 1024;
    require!(size >= MIN_CALS_DCL_SIZE, eq_at(prefix, LEN_SIZE, CALS_DCL_SRCSYS));
    let len1 = be32(prefix, 0)?;
    require!((CALS_DCL_SRCSYS.len() as u32..=MAX_RECORD_LEN).contains(&len1));
    let rec2 = LEN_SIZE + len1 as usize;
    let len2 = be32(prefix, rec2)?;
    require!(
        (CALS_DCL_SRCDOCID.len() as u32..=MAX_RECORD_LEN).contains(&len2),
        eq_at(prefix, rec2 + LEN_SIZE, CALS_DCL_SRCDOCID)
    );
    Some(Label::CalsDcl)
}

/// Chinese DOS CEdit / UCVIEW Revision History (.his).
pub(crate) fn verify_cedit_ucview_his(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CEDIT_UCVIEW_HIS_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 5)? == hex2bin!("01 AA A9 C5 76"),
        get_slice(prefix, 9, 10)? == hex2bin!("A1")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::CeditUcviewHis)
}

/// Chinese DOS CEdit / UCVIEW Help File (.hlp).
pub(crate) fn verify_cedit_ucview_hlp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CEDIT_UCVIEW_HLP_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 21)?
            == hex2bin!("01 B9 73 A1 42 B7 C7 B3 C6 28 A9 D2 BB DD C0 C9 AE D7 29 A1 47")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::CeditUcviewHlp)
}

/// ChemDraw Chemical Draw Exchange (.cdx, PRONOM fmt/378, LoC fdd000582).
pub(crate) fn verify_chemdraw_cdx(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_CHEMDRAW_CDX_SIZE
        && prefix.len() >= 16
        && starts(prefix, b"VjCD0100")
        && prefix.get(8..16)?.contains(&0)
    {
        Some(Label::Cdx)
    } else {
        None
    }
}

/// Clarion Professional Developer Help File (`.hlp`, `E0 49` `0x49E0` LE signature + `u32 LE` directory offset).
pub(crate) fn verify_clarion_hlp(prefix: &[u8], size: u64) -> Option<Label> {
    const CLARION_HLP_MAGIC: &[u8; 2] = hex2bin!("E0 49"); // @0..2: 0x49E0 LE Clarion Help signature
    const DIR_OFFSET_FIELD: usize = 2; // @2..6: u32 LE topic directory offset within file
    const TOPIC_COUNT_OFFSET: usize = 6; // @6..8: u16 LE non-zero header metric
    const INDEX_COUNT_OFFSET: usize = 8; // @8..10: u16 LE non-zero header metric

    let dir_off = le32(prefix, DIR_OFFSET_FIELD)? as u64;
    require!(
        size >= MIN_CLARION_HLP_SIZE,
        prefix.len() >= 16,
        starts(prefix, CLARION_HLP_MAGIC),
        (12..size).contains(&dir_off),
        le16(prefix, TOPIC_COUNT_OFFSET)? > 0,
        le16(prefix, INDEX_COUNT_OFFSET)? > 0,
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::ClarionHlp)
}

/// Claris Resolve Spreadsheet (`"BAT_BT"` + 2-char module + 4 ASCII version digits).
pub(crate) fn verify_claris_resolve(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CLARIS_RESOLVE_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"BAT_BT"),
        get_slice(prefix, 6, 8)?.iter().all(|b| b.is_ascii_uppercase()),
        get_slice(prefix, 8, 12)?.iter().all(|b| b.is_ascii_digit()),
        prefix.get(12..16)?.contains(&0)
    );
    Some(Label::ClarisResolve)
}

/// CorelDRAW Compressed Drawing (.cpx, PRONOM x-fmt/36).
pub(crate) fn verify_coreldraw_cpx(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_CORELDRAW_CPX_SIZE
        && prefix.len() >= 16
        && (starts(prefix, b"CDRCOMP1") || starts(prefix, b"CDRCOMP2"))
        && has_non_ascii_text_byte(prefix.get(8..16)?)
    {
        Some(Label::Cpx)
    } else {
        None
    }
}

/// Seagate Crystal Reports 4.x/5.x Report (.rpt).
pub(crate) fn verify_crystal_reports_v4_rpt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CRYSTAL_REPORTS_V4_RPT_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 14)? == hex2bin!("00 10 41 00 04 00 00 00 00 04 00 00 00 00"),
        get_slice(prefix, 15, 22)? == hex2bin!("00 00 00 00 00 00 00"),
        get_slice(prefix, 23, 24)? == hex2bin!("00"),
        get_slice(prefix, 25, 32)? == hex2bin!("00 00 00 00 00 00 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::CrystalReportsV4Rpt)
}

/// C eXtended Library (CXL 5.2) Compiled Help Database (`.hlp`, 1988 Mike Smedley / Innovative Data Concepts).
///
/// Header layout: `"*I\r\n"` at `@0..4`, followed by fixed 36-byte index records consisting of
/// `u16 LE context_id` (`> 0`), NUL-padded 30-byte ASCII title (`char[30]`, containing at least
/// one trailing `0x00`), and `u32 LE body_offset` (`40 <= body_offset < size`).
pub(crate) fn verify_cxl_help(prefix: &[u8], size: u64) -> Option<Label> {
    const CTX_OFF: usize = 4; // @4: u16 LE first topic context ID (> 0)
    const TITLE_OFF: usize = 6; // @6..36: char[30] NUL-padded ASCII title
    const TITLE_END: usize = 36;
    const BODY_OFF_OFF: usize = 36; // @36..40: u32 LE offset to first topic body

    require!(size >= MIN_CXL_HELP_SIZE, prefix.len() >= 40, starts(prefix, CXL_HELP_MAGIC));
    let ctx = le16(prefix, CTX_OFF)?;
    let title = get_slice(prefix, TITLE_OFF, TITLE_END)?;
    let nul_pos = title.iter().position(|&b| b == 0)?;
    let body_off = le32(prefix, BODY_OFF_OFF)?;
    require!(
        ctx > 0,
        nul_pos >= 1,
        title[..nul_pos].iter().all(|&b| (0x20..=0x7E).contains(&b)),
        title[nul_pos..].iter().all(|&b| b == 0),
        body_off >= 40,
        (body_off as u64) < size
    );
    Some(Label::CxlHelp)
}

/// CZ Help: `"CZ_HELP!"`.
pub(crate) fn verify_cz_help(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CZ_HELP_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"CZ_HELP!"),
        prefix.get(8..16)?.contains(&0)
    );
    Some(Label::CzHelp)
}

/// DEC WPS-Plus Word Processing Document (.wps / .wpl / .dx).
pub(crate) fn verify_dec_wpsplus(prefix: &[u8], size: u64) -> Option<Label> {
    let valid = starts(prefix, &[0x48, 0x42, 0xD1, 0x07])
        || (size >= 184
            && prefix.len() >= 124
            && starts(prefix, b"\x74\x00_ _ _ _____")
            && eq_at(prefix, 118, b"\x40\x00|N"));
    require!(size >= MIN_WPSPLUS_SIZE && prefix.len() >= 4 && valid);
    Some(Label::Wpsplus)
}

/// Borland Delphi TCardDocument / Desktop Card Publisher (`.pub`, 1995) header (`LCP\x01\x00` or `LCP\x02\x00`).
pub(crate) fn verify_delphi_card_publisher_lcp(prefix: &[u8], size: u64) -> Option<Label> {
    const LCP_V1_MAGIC: &[u8; 5] = b"LCP\x01\x00"; // @0..5: Desktop Card Publisher v1 header
    const LCP_V2_MAGIC: &[u8; 5] = b"LCP\x02\x00"; // @0..5: Desktop Card Publisher v2 header
    const SENTINEL_OFFSET: usize = 11; // @11..13: 0xFFFF record sentinel

    require!(
        size >= MIN_DELPHI_CARD_PUBLISHER_LCP_SIZE,
        prefix.len() >= 32,
        starts(prefix, LCP_V1_MAGIC) || starts(prefix, LCP_V2_MAGIC),
        eq_at(prefix, SENTINEL_OFFSET, &[0xFF, 0xFF])
    );
    Some(Label::DelphiCardPublisherLcp)
}

/// DESI Telephone Label Definition File (.ddf) DESI Telephone Labels.
pub(crate) fn verify_desi_ddf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DESI_DDF_SIZE,
        prefix.len() >= 128,
        starts(prefix, b"DESI Definition File 2.10\r\nCopyright"),
        has_non_ascii_text_byte(prefix.get(36..128)?)
    );
    Some(Label::DesiDdf)
}

/// Quarterdeck DESQview / DESQview/X Help File (`.hlp`):
/// `@0..4` `u32 LE` topic text size (`>= 32`, `u8_at(prefix, 3)? == 0`), `@4..6` `u16 LE` topic index size (`>= 16`),
/// `@6..9` `"/FT"` first topic marker followed by printable topic key and index/EOF boundary.
pub(crate) fn verify_desqview_hlp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DESQVIEW_HLP_SIZE,
        prefix.len() >= 16,
        u8_at(prefix, 3)? == 0,
        eq_at(prefix, 6, b"/FT"),
        u8_at(prefix, 9)?.is_ascii_graphic()
    );
    let text_len = le32(prefix, 0)? as u64;
    let index_len = le16(prefix, 4)? as u64;
    let win = get_slice(prefix, 9, prefix.len().min(128))?;
    require!(
        text_len >= 32,
        index_len >= 16,
        text_len.saturating_add(index_len) == size || contains(win, b"/F")
    );
    Some(Label::DesqviewHlp)
}

/// IBM DisplayWrite DCA RFT/FFT (.dca, PRONOM x-fmt/148).
pub(crate) fn verify_displaywrite_dca(prefix: &[u8], size: u64) -> Option<Label> {
    const DCA_BEGIN_UNIT: &[u8; 6] = hex2bin!("00 05 E1 03 00 00"); // @0..6: Begin Formatted Text Unit (len=5, class=E1, type=03)
    const DOC_PARAMS_LEN_OFFSET: usize = 6; // @6: u8 Document Parameters structured field length (0x10..=0x80)
    const DOC_PARAMS_PREFIX: &[u8; 3] = hex2bin!("E2 05 00"); // @7..10: Document Parameters class/type/flags
    const VERSION_MARKER: &[u8; 2] = hex2bin!("01 00"); // @12..14: DCA RFT format version marker

    require!(
        size >= MIN_DISPLAYWRITE_DCA_SIZE,
        prefix.len() >= 14,
        starts(prefix, DCA_BEGIN_UNIT),
        (0x10..=0x80).contains(&u8_at(prefix, DOC_PARAMS_LEN_OFFSET)?),
        eq_at(prefix, 7, DOC_PARAMS_PREFIX),
        eq_at(prefix, 12, VERSION_MARKER)
    );
    Some(Label::Dca)
}

/// IBM Document Content Architecture Final-Form Text (`.fft` / `.dca`, 1981 IBM):
/// contiguous EBCDIC Control Sequence Prefix (`0x2B`, Class `0xD1..=0xD4`, `cnt >= 3`) TLV records from offset 0.
pub(crate) fn verify_displaywrite_dca_fft(prefix: &[u8], size: u64) -> Option<Label> {
    const CSP_INTRODUCER: u8 = 0x2B;
    require!(size >= MIN_DISPLAYWRITE_DCA_SIZE, prefix.len() >= 15);
    let mut pos = 0usize;
    let mut records = 0usize;
    while records < 3 {
        require!(pos + 4 <= prefix.len());
        let intro = u8_at(prefix, pos)?;
        let class = u8_at(prefix, pos + 1)?;
        let count = u8_at(prefix, pos + 2)? as usize;
        require!(
            intro == CSP_INTRODUCER,
            (0xD1..=0xD4).contains(&class),
            (3..=128).contains(&count)
        );
        let next = pos + 2 + count;
        require!((next as u64) <= size, next <= prefix.len());
        pos = next;
        records += 1;
    }
    Some(Label::Dca)
}

/// MS-DOS Binary Encapsulated PostScript (.eps).
pub(crate) fn verify_dos_eps(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_DOS_EPS_SIZE && prefix.len() >= 30 { Some(Label::EpsDos) } else { None }
}

/// Clarivate / Thomson Reuters EndNote Connection/Filter File (.enf / .enz) ISI ResearchSoft.
pub(crate) fn verify_endnote_enf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ENDNOTE_ENF_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"NDNEEL2s\0\0\0\x04\0\0\0\x02"),
        le32(prefix, 26) == Some(u32::MAX)
    );
    Some(Label::EndnoteEnf)
}

/// EnVision Publisher: banner `"EnVision Publisher DTP document file."` terminated by DOS EOF `0x1A`.
pub(crate) fn verify_envision_publisher(prefix: &[u8], size: u64) -> Option<Label> {
    const BANNER: &[u8] = b"EnVision Publisher DTP document file."; // @0, 37 bytes
    require!(
        size >= MIN_ENVISION_SIZE,
        starts(prefix, BANNER),
        u8_at(prefix, BANNER.len())? == 0x1A
    );
    Some(Label::Envision)
}

/// Power Up Express Publisher Document (.epd).
pub(crate) fn verify_express_publisher(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EXPRESS_PUBLISHER_SIZE
            && prefix.len() >= 4
            && starts(prefix, &[0xCA, 0xFE, 0xBE, 0xEF])
    );
    Some(Label::Epd)
}

/// Fabylon Book container (`FABYLON_BOOK_02!` / `FABYLON_REALM_01`).
pub(crate) fn verify_fabylon(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 24,
        prefix.len() >= 24,
        starts(prefix, b"FABYLON_BOOK_02!") || starts(prefix, b"FABYLON_REALM_01"),
        prefix.get(16..24)?.contains(&0),
    );
    Some(Label::Fabylon)
}

/// JP Software 4DOS / 4OS2 / 4NT Compressed Help Database (`.hlp`, 1991, JP Software).
///
/// On-disk header generations:
/// - v4.x (`4DOS 4.0`): `b"4DH4"` / `b"4OH4"` / `b"4NH4"`, `@4..6 u16 LE topic_count >= 1`,
///   `@11` screen width (`40..=132`), `@12..28` 16-byte nibble-encoding token table starting with `' '`.
/// - v5.x+ (`4DOS 5.52`): `@0 == 0x08`, `@1..4` in `{"4DH","4OH","4NH"}`, `@4..7` ASCII version
///   digits, `@7..9` ASCII uppercase/digit revision, `@17` screen width (`40..=132`), `@18..34`
///   frequency token table starting with `' '`.
pub(crate) fn verify_fourdos_help(prefix: &[u8], size: u64) -> Option<Label> {
    const V4_TOPICS_OFF: usize = 4; // @4..6: u16 LE topic count
    const V4_WIDTH_OFF: usize = 11; // @11: u8 help display width (80)
    const V4_TOKENS_OFF: usize = 12; // @12..28: 16-byte Huffman/nibble token table
    const V5_WIDTH_OFF: usize = 17; // @17: u8 help display width (80)
    const V5_TOKENS_OFF: usize = 18; // @18..34: frequency token table prefix

    require!(size >= MIN_FOURDOS_HELP_SIZE);

    if prefix.len() >= 28
        && (starts(prefix, FOURDOS_HELP_V4_DOS_MAGIC)
            || starts(prefix, FOURDOS_HELP_V4_OS2_MAGIC)
            || starts(prefix, FOURDOS_HELP_V4_NT_MAGIC))
    {
        let topics = le16(prefix, V4_TOPICS_OFF)?;
        let width = u8_at(prefix, V4_WIDTH_OFF)?;
        let tokens = get_slice(prefix, V4_TOKENS_OFF, V4_TOKENS_OFF + 16)?;
        let text_tokens =
            tokens.iter().filter(|&&b| b.is_ascii_lowercase() || b == b' ' || b == b'\r').count();
        require!(
            topics >= 1,
            (40..=132).contains(&width),
            tokens[0] == b' ',
            text_tokens >= 10,
            has_non_ascii_text_byte(prefix.get(..28)?)
        );
        return Some(Label::FourdosHelp);
    }

    if prefix.len() >= 34 && u8_at(prefix, 0) == Some(FOURDOS_HELP_V5_SIG_LEN) {
        let tag = get_slice(prefix, 1, 9)?;
        let width = u8_at(prefix, V5_WIDTH_OFF)?;
        let tokens = get_slice(prefix, V5_TOKENS_OFF, V5_TOKENS_OFF + 16)?;
        let text_tokens =
            tokens.iter().filter(|&&b| b.is_ascii_lowercase() || b == b' ' || b == b'\r').count();
        require!(
            matches!(&tag[0..3], b"4DH" | b"4OH" | b"4NH"),
            tag[3..6].iter().all(|b| b.is_ascii_digit()),
            tag[6..8].iter().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()),
            (40..=132).contains(&width),
            tokens[0] == b' ',
            text_tokens >= 10,
            has_non_ascii_text_byte(prefix.get(..34)?)
        );
        return Some(Label::FourdosHelp);
    }
    None
}

/// Borland Turbo Vision / FreeVision Compiled Help File (`.hlp` / `.hf`, 1990, Borland).
///
/// Supports `"FBHF"` and `"FBRW"` (Borland Resource Workshop) headers:
/// - C++ `opstream` variant: `@12..25 == b"\x02[\x0aTHelpTopic"`.
/// - Pascal `TStream` / Resource Workshop variant: `@4..8 u32 LE eof_sentinel_pos` (`<= size`,
///   typically `size - 8`) and `@8..12 u32 LE index_pos` (`12 <= index_pos < eof_sentinel_pos`).
pub(crate) fn verify_freebasic_hf(prefix: &[u8], size: u64) -> Option<Label> {
    const EOF_SENTINEL_OFF: usize = 4; // @4..8: u32 LE EOF sentinel / index end offset
    const INDEX_POS_OFF: usize = 8; // @8..12: u32 LE THelpIndex stream offset
    const STREAM_START: u64 = 12; // @12: first serialized THelpTopic record
    const CPP_SIG_OFF: usize = 12; // @12..25: C++ opstream "\x02[\x0aTHelpTopic"

    require!(
        size >= MIN_FREEBASIC_HF_SIZE,
        prefix.len() >= 32,
        starts(prefix, BORLAND_TV_HELP_FBHF_MAGIC) || starts(prefix, BORLAND_TV_HELP_FBRW_MAGIC)
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));

    if eq_at(prefix, CPP_SIG_OFF, BORLAND_TV_HELP_THELPTOPIC_SIG) {
        return Some(Label::FreebasicHf);
    }

    let eof_pos = le32(prefix, EOF_SENTINEL_OFF)? as u64;
    let idx_pos = le32(prefix, INDEX_POS_OFF)? as u64;
    require!(idx_pos >= STREAM_START, idx_pos < eof_pos, eof_pos <= size, size - eof_pos <= 64);
    Some(Label::FreebasicHf)
}

/// Ashton-Tate / Akimbo FullWrite Professional 1.0 & 2.0 Macintosh Document.
pub(crate) fn verify_fullwrite_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_FULLWRITE_SIZE, prefix.len() >= 36);
    // FullWrite 1.0: 00 00 00 5C 00 00 02 00 + [0; 10] + 00 11 + [0xFF; 8]
    if get_slice(prefix, 0, 8)? == [0x00, 0x00, 0x00, 0x5C, 0x00, 0x00, 0x02, 0x00]
        && get_slice(prefix, 8, 18)?.iter().all(|&b| b == 0)
        && get_slice(prefix, 18, 20)? == [0x00, 0x11]
        && get_slice(prefix, 20, 28)?.iter().all(|&b| b == 0xFF)
    {
        return Some(Label::FullwriteDoc);
    }
    // FullWrite 2.0: 00 00 00 03 00 1A 00 00 00 01 ... 3E 80 3E 80 3E 80 at @26..32 (or @28..34)
    if get_slice(prefix, 0, 10)? == [0x00, 0x00, 0x00, 0x03, 0x00, 0x1A, 0x00, 0x00, 0x00, 0x01]
        && (get_slice(prefix, 26, 32)? == [0x3E, 0x80, 0x3E, 0x80, 0x3E, 0x80]
            || get_slice(prefix, 28, 34)? == [0x3E, 0x80, 0x3E, 0x80, 0x3E, 0x80])
    {
        return Some(Label::FullwriteDoc);
    }
    None
}

/// Garmin Flexible and Interoperable Data Transfer (.fit) activity/settings file.
pub(crate) fn verify_garmin_fit(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_GARMIN_FIT_SIZE && prefix.len() >= 12);
    let hdr_size = u8_at(prefix, 0)? as usize;
    require!(hdr_size == 12 || hdr_size == 14);
    require!(eq_at(prefix, 8, b".FIT"));
    let data_size = le32(prefix, 4)? as u64;
    let expected_min_size = (hdr_size as u64).checked_add(data_size)?.checked_add(2)?;
    require!(size >= expected_min_size);

    if hdr_size == 14 && prefix.len() >= 14 {
        let hdr_crc = le16(prefix, 12)?;
        if hdr_crc != 0 {
            require!(hdr_crc == crc16_fit(get_slice(prefix, 0, 12)?));
        }
    }

    if size == expected_min_size && size <= prefix.len() as u64 {
        let file_crc = le16(prefix, (size - 2) as usize)?;
        require!(file_crc == crc16_fit(get_slice(prefix, 0, (size - 2) as usize)?));
    }

    Some(Label::Fit)
}

/// GUEmap interactive fiction map (`.gmp`, 1996 Christopher J. Madsen):
/// 8-byte binary magic `C7 55 C5 6D E1 70 83 0D`, `u16 LE` file version @8 (`1..=16`) and
/// `u16 LE` revision @10 (`1..=16`).
pub(crate) fn verify_guemap(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION: usize = 8; // @8: u16 LE file format version
    const REVISION: usize = 10; // @10: u16 LE revision
    require!(
        size >= MIN_GUEMAP_SIZE,
        prefix.len() >= 12,
        starts(prefix, GUEMAP_MAGIC),
        (1..=16).contains(&le16(prefix, VERSION)?),
        (1..=16).contains(&le16(prefix, REVISION)?)
    );
    Some(Label::Guemap)
}

/// HanMac Word-J (`"YLYS\x01\x00"`) and HanMac Word-K (`"HYLS\x02\x10"`) Documents.
pub(crate) fn verify_hanmac_word_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_HANMAC_WORD_SIZE, prefix.len() >= 16);
    if starts(prefix, b"YLYS\x01\x00") && get_slice(prefix, 6, 16)?.iter().all(|&b| b == 0) {
        return Some(Label::HanmacWord);
    }
    if starts(prefix, b"HYLS\x02\x10") && get_slice(prefix, 6, 10)?.iter().all(|&b| b == 0) {
        return Some(Label::HanmacWord);
    }
    None
}

/// Software Publishing Harvard Graphics Presentation / Chart (.ch3 / .sh3).
pub(crate) fn verify_harvard_graphics(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HARVARD_GRAPHICS_SIZE,
        prefix.len() >= 6,
        starts(prefix, b"HGB3\0\0") || starts(prefix, b"HGB1\0\0") || starts(prefix, b"Falc\0\0")
    );
    Some(Label::Harvard)
}

/// Ntergaid HyperWriter! Document (`"HyperWriter!\x1A"` or `"Hyperwriter!\x1A"`).
pub(crate) fn verify_hyperwriter_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HYPERWRITER_SIZE,
        starts(prefix, b"HyperWriter!\x1A") || starts(prefix, b"Hyperwriter!\x1A")
    );
    Some(Label::Hyperwriter)
}

/// IBM DisplayWrite 2/3/4/5 native binary document (`.doc` / `.dw4`).
///
/// Header layout:
/// - `@0..10`: `80 00 00 09 20 00 4F 7B 4A 5D` (DisplayWrite record prefix + EBCDIC `"O{J]"`).
/// - `@10`: `0x02` (DisplayWrite 2/3/4) or `0x18` (DisplayWrite 5).
/// - `@11`: `0x00`.
pub(crate) fn verify_ibm_displaywrite(prefix: &[u8], size: u64) -> Option<Label> {
    const DW_PREFIX: &[u8; 10] = bytes!(&[0x80, 0x00, 0x00, 0x09, 0x20, 0x00], b"O{J]"); // @0..10
    const VER_OFFSET: usize = 10; // @10: u8 version flag (0x02 = DW2/3/4, 0x18 = DW5)
    const ZERO_OFFSET: usize = 11; // @11: u8 reserved zero

    require!(
        size >= MIN_IBM_DISPLAYWRITE_SIZE,
        prefix.len() >= 12,
        starts(prefix, DW_PREFIX),
        matches!(u8_at(prefix, VER_OFFSET)?, 0x02 | 0x18),
        u8_at(prefix, ZERO_OFFSET) == Some(0)
    );
    Some(Label::IbmDisplaywrite)
}

/// IBM OS/2 & PC DOS MKMSGF Compiled Message Catalog (`.msg`, 1987) IBM.
pub(crate) fn verify_ibm_os2_mkmsgf_msg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_IBM_OS2_MKMSGF_MSG_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"\xFFMKMSGF\x00")
    );
    Some(Label::IbmOs2MkmsgfMsg)
}

/// Adobe InDesign Document / Book / Library (.indd / .indb / .indl).
pub(crate) fn verify_indesign(prefix: &[u8], size: u64) -> Option<Label> {
    const SUBTYPE_OFFSET: usize = 16; // @16..24: 8-byte ASCII subtype tag
    const ENDIAN_OFFSET: usize = 24; // @24: byte order flag (1=LE, 2=BE)
    require!(
        size >= MIN_INDESIGN_SIZE
            && prefix.len() >= 25
            && starts(prefix, INDESIGN_GUID)
            && (eq_at(prefix, SUBTYPE_OFFSET, b"DOCUMENT")
                || eq_at(prefix, SUBTYPE_OFFSET, b"BOOKBOOK")
                || eq_at(prefix, SUBTYPE_OFFSET, b"LIBRARY4")
                || eq_at(prefix, SUBTYPE_OFFSET, b"LIBRARY2"))
            && matches!(u8_at(prefix, ENDIAN_OFFSET), Some(1 | 2))
    );
    Some(Label::Indd)
}

/// Power Measurement / Schneider Electric ION Enterprise PML User Diagram (.dgm) Power Measurement.
pub(crate) fn verify_ion_pml_dgm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ION_PML_DGM_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"PML USER DIAGRAM\x01\0Version 2.00\0")
    );
    Some(Label::IonPmlDgm)
}

/// Lumex Kapel Capillary Electrophoresis Measurement (.mdf).
pub(crate) fn verify_kapel_mdf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KAPEL_MDF_SIZE,
        prefix.len() >= 64,
        get_slice(prefix, 0, 24)?
            == hex2bin!("5B 4D 65 61 73 75 72 65 6D 65 6E 74 5D 0A 54 79 70 65 3D 45 6C 65 63 74")
    );
    require!(has_non_ascii_text_byte(prefix.get(..64)?));
    Some(Label::KapelMdf)
}

/// Runtime Revolution / LiveCode Stack (.rev / .livecode).
pub(crate) fn verify_livecode(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LIVECODE_SIZE
            && prefix.len() >= 16
            && (starts(prefix, b"REVO") || starts(prefix, b"REVS"))
            && get_slice(prefix, 4, 8)?.iter().all(|b| b.is_ascii_digit())
            && prefix.get(8..16)?.contains(&0)
    );
    Some(Label::Livecode)
}

/// Locomotive Software LocoScript Document (`.loc`, `"JOY!pe"` or `"JOY\x01\x01"`..`"JOY\x01\x03"`).
pub(crate) fn verify_locoscript_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LOCOSCRIPT_SIZE,
        prefix.len() >= 16,
        (starts(prefix, b"JOY!pe") && has_non_ascii_text_byte(prefix.get(6..16)?))
            || (starts(prefix, b"JOY\x01") && matches!(u8_at(prefix, 4).unwrap_or(0), 1..=3))
    );
    Some(Label::Locoscript)
}

/// Lotus 1-2-3 Release 3 Installation Driver Database (`LOTUS.DAT`, 1989) Lotus.
pub(crate) fn verify_lotus_123_install_dat(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LOTUS_123_INSTALL_DAT_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"\xA9\x09LOTUS   \x06\x00")
    );
    Some(Label::Lotus123InstallDat)
}

/// Lotus Manuscript Technical Document (.doc).
pub(crate) fn verify_lotus_manuscript(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MANUSCRIPT_SIZE
            && prefix.len() >= 5
            && (starts(prefix, b"*R1\r\n")
                || starts(prefix, b"*R2\r\n")
                || starts(prefix, b"*R1\r")
                || starts(prefix, b"*R2\r"))
    );
    Some(Label::Manuscript)
}

/// Microsoft Word for Macintosh v1.05..v5.1 (`FE 32 00`, `FE 34 00`, `FE 37 00 1C`, `FE 37 00 23`).
pub(crate) fn verify_mac_word_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MAC_WORD_SIZE,
        prefix.len() >= 18,
        starts(prefix, &[0xFE, 0x32, 0x00])
            || starts(prefix, &[0xFE, 0x34, 0x00])
            || starts(prefix, &[0xFE, 0x37, 0x00, 0x1C])
            || starts(prefix, &[0xFE, 0x37, 0x00, 0x23])
    );
    let end14 = be32(prefix, 14)? as u64;
    let v1_ok = (32..=size).contains(&end14);
    let v4_ok = prefix.len() >= 32
        && end14 == 0
        && be32(prefix, 24)? > 0
        && (32..=size).contains(&(be32(prefix, 28)? as u64));
    require!(v1_ok || v4_ok);
    Some(Label::MacWord)
}

/// Claris MacWrite II Document (.mw2).
pub(crate) fn verify_macwrite2(prefix: &[u8], size: u64) -> Option<Label> {
    let is_raw = prefix.len() >= 32
        && starts(prefix, &[0x00, 0x2E, 0x00, 0x2E])
        && matches!(be16(prefix, 4), Some(1..=3))
        && be16(prefix, 6) == Some(0)
        && eq_at(prefix, 8, &[0x00, 0x48, 0x00, 0x48]);
    let is_macbin = prefix.len() >= 73
        && starts(prefix, &[0x00, 0x2E, 0x00, 0x2E])
        && (eq_at(prefix, 65, b"MW2DMWII") || eq_at(prefix, 65, b"MW2SMWII"));
    require!(size >= MIN_MACWRITE2_SIZE && (is_raw || is_macbin));
    Some(Label::Macwrite2)
}

/// Mariner Write Document (`.mwd`, `"MRNW"` or `legacy Mac document` v1.6–3.5 `"FFFF00000006000100"` + `\x0177AA\x03`).
pub(crate) fn verify_mariner_write_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_MARINER_WRITE_SIZE, prefix.len() >= 26);
    if starts(prefix, b"MRNW")
        && get_slice(prefix, 4, 12)?.iter().any(|&b| b != 0)
        && prefix.get(4..12)?.contains(&0)
    {
        return Some(Label::MarinerWrite);
    }
    if starts(prefix, b"FFFF00000006000100") && get_slice(prefix, 20, 26)? == b"\x0177AA\x03" {
        return Some(Label::MarinerWrite);
    }
    None
}

/// Multi Channel Systems MC_Stimulus II Waveform Definition (.stm) Multi Channel Systems.
pub(crate) fn verify_mcs_stimulus_stm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MCS_STIMULUS_STM_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\x18MCStimulus Version 1.4.0")
    );
    Some(Label::McsStimulusStm)
}

/// KIDASA Software Milestones Professional Schedule (.mlb / .mlq) KIDASA Software.
pub(crate) fn verify_milestones_mlb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MILESTONES_MLB_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"MVB00") || starts(prefix, b"MVQ00"),
        get_slice(prefix, 5, 20)? == [0xFF; 15]
    );
    Some(Label::MilestonesMlb)
}

/// Living Videotext / Symantec MORE 2.0/3.0 Macintosh Outline (`00 03` / `00 06` + `"MOR2"`/`"MOR3"`/`"MORE"`).
pub(crate) fn verify_more_outline_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MORE_OUTLINE_SIZE,
        prefix.len() >= 16,
        u8_at(prefix, 0)? == 0x00,
        matches!(u8_at(prefix, 1)?, 2 | 3 | 4 | 6),
        matches!(get_slice(prefix, 2, 6)?, b"MOR2" | b"MOR3" | b"MORE"),
        matches!(be16(prefix, 6)?, 0x0080 | 0x0000)
    );
    Some(Label::MoreOutline)
}

/// YZ 6.0 GBK Document (.yz).
pub(crate) fn verify_moshidraw_yz(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MOSHIDRAW_YZ_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"\x036.0"),
        get_slice(prefix, 6, 16)? == b"\0*\xb4\xcb\xce\xc4\xbc\xfe\xce\xaa"
    );
    require!(has_non_ascii_text_byte(prefix.get(..24)?));
    Some(Label::MoshidrawYz)
}

/// Roger Wagner MouseWrite Apple IIe/IIc/IIGS / Macintosh Word Processor Document (`"MW\x01\x00"` / `"MW\x04\x00\x02\x00"` / `"\x04GM01"`..`"\x04GM04"`).
pub(crate) fn verify_mousewrite_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_MOUSEWRITE_SIZE, prefix.len() >= 16);
    if starts(prefix, b"MW\x04\x00\x02\x00") {
        return Some(Label::MousewriteDoc);
    }
    if starts(prefix, b"\x04GM0")
        && (b'1'..=b'4').contains(&u8_at(prefix, 4)?)
        && (be32(prefix, 6) == Some(0x0000_0080) || be32(prefix, 5) == Some(0x0000_0080))
    {
        return Some(Label::MousewriteDoc);
    }
    if starts(prefix, b"MW\x01\x00")
        && get_slice(prefix, 4, 16)?
            .iter()
            .filter(|&&b| b == 0x8D || (0xA0..=0xFE).contains(&b))
            .count()
            >= 8
    {
        return Some(Label::MousewriteDoc);
    }
    None
}

/// Microsoft QuickHelp / Advisor Help Database (`.hlp`, 1988, Microsoft `HELPMAKE`).
///
/// 70-byte (`0x46`) header geometry:
/// `version == 2`, `topic_count >= 1`, `display_width == 0 || 20..=160`, NUL-padded ASCII
/// `database_name` at `@16..30`, `topic_index_off == 0x46`, exact
/// `context_strings_off == 0x46 + (topic_count + 1) * 4`, monotonic non-zero section offsets,
/// and `0x46 < database_size <= size`.
pub(crate) fn verify_ms_advisor_help(prefix: &[u8], size: u64) -> Option<Label> {
    const TOPICS_OFF: usize = 8; // @8..10: u16 LE topic count (>= 1)
    const WIDTH_OFF: usize = 12; // @12: u8 screen width (0 or 20..=160)
    const DBNAME_OFF: usize = 16; // @16..30: 14-byte NUL-padded DOS filename
    const DBNAME_LEN: usize = 14;
    const TABLES_OFF: usize = 34; // @34..66: 8 x u32 LE section offsets
    const DB_SIZE_OFF: usize = 66; // @66..70: u32 LE logical database size
    const HDR_LEN: u32 = 70; // 0x46 bytes

    require!(
        size >= MIN_MS_ADVISOR_HELP_SIZE,
        prefix.len() >= HDR_LEN as usize,
        starts(prefix, MS_ADVISOR_HELP_MAGIC)
    );
    let n_topics = le16(prefix, TOPICS_OFF)?;
    let width = u8_at(prefix, WIDTH_OFF)?;
    let db_size = le32(prefix, DB_SIZE_OFF)?;
    let t0 = le32(prefix, TABLES_OFF)?;
    let t1 = le32(prefix, TABLES_OFF + 4)?;
    let expected_t1 = HDR_LEN.checked_add((n_topics as u32 + 1).checked_mul(4)?)?;
    require!(
        n_topics >= 1,
        width == 0 || (20..=160).contains(&width),
        db_size > HDR_LEN,
        (db_size as u64) <= size,
        t0 == HDR_LEN,
        t1 == expected_t1,
        t1 <= db_size
    );
    let mut prev = t1;
    for i in 2..8 {
        let off = le32(prefix, TABLES_OFF + i * 4)?;
        if off != 0 {
            require!(off >= prev, off <= db_size);
            prev = off;
        }
    }
    let dbname = get_slice(prefix, DBNAME_OFF, DBNAME_OFF + DBNAME_LEN)?;
    let nul_pos = dbname.iter().position(|&b| b == 0).unwrap_or(DBNAME_LEN);
    require!(
        dbname[..nul_pos].iter().all(|&b| (0x20..=0x7E).contains(&b)),
        dbname[nul_pos..].iter().all(|&b| b == 0),
        has_non_ascii_text_byte(prefix.get(..HDR_LEN as usize)?)
    );

    Some(Label::MsAdvisorHlp)
}

/// Microsoft Publisher BorderArt File (`.bdr`, 1993) Microsoft.
pub(crate) fn verify_ms_publisher_borderart_bdr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MS_PUBLISHER_BORDERART_BDR_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"XT\x01\x00"),
        le16(prefix, 4)? > 0,
        u8_at(prefix, 6)?.is_ascii_graphic()
    );
    Some(Label::MsPublisherBorderartBdr)
}

/// Microsoft Publisher PageWizard Design Assistant (`.poc`, 1995) Microsoft.
pub(crate) fn verify_ms_publisher_pagewizard_poc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MS_PUBLISHER_PAGEWIZARD_POC_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"POCPB3\x04\x00")
    );
    Some(Label::MsPublisherPagewizardPoc)
}

/// Microsoft Publisher Template Preview (`.prv`, 1995) Microsoft.
pub(crate) fn verify_ms_publisher_preview_prv(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MS_PUBLISHER_PREVIEW_PRV_SIZE,
        prefix.len() >= 16,
        starts(prefix, &[0x89, 0x6C, 0xA2, 0x00]),
        le16(prefix, 4)? > 0,
        le16(prefix, 6)? > 0
    );
    Some(Label::MsPublisherPreviewPrv)
}

/// Microsoft Works for DOS Word Processor document (`.wps`, 1987 Microsoft, PRONOM `fmt/163` / `fmt/259`).
pub(crate) fn verify_ms_works_dos_wps(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MS_WORKS_DOS_WPS_SIZE
            && prefix.len() >= 32
            && u8_at(prefix, 0)? == 0x01
            && u8_at(prefix, 1)? == 0xFE
    );
    let ver_flags = le16(prefix, 2)?;
    let cb_mac_26 = le32(prefix, 26)? as u64;
    let v2_ok =
        get_slice(prefix, 4, 12)?.iter().all(|&b| b == 0) && (128..=size).contains(&cb_mac_26);
    let itw_ok = prefix.len() >= 46
        && le32(prefix, 34)? == 0x0000_0100
        && (128..=size).contains(&(le32(prefix, 38)? as u64))
        && (128..=size).contains(&(le32(prefix, 42)? as u64));
    require!(ver_flags <= 0x0020 && (v2_ok || itw_ok));
    Some(Label::WpsDos)
}

/// MultiMate Text Document (.dox, PRONOM x-fmt/347).
pub(crate) fn verify_multimate(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_MULTIMATE_SIZE
        && prefix.len() >= 24
        && starts(prefix, b"D$     ")
        && prefix.get(7..24)?.contains(&0)
    {
        Some(Label::Multimate)
    } else {
        None
    }
}

pub(crate) fn verify_multimate_offset(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 512
            && prefix.len() >= 512
            && u8_at(prefix, 0)?.is_ascii_graphic()
            && get_slice(prefix, 0, 20)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
            && get_slice(prefix, 20, 50)?.iter().all(|&b| b == 0)
            && u8_at(prefix, 406)? == b'/'
            && u8_at(prefix, 409)? == b'/'
            && u8_at(prefix, 414)? == b'/'
            && u8_at(prefix, 417)? == b'/'
            && (eq_at(prefix, 492, b"D$     ") || eq_at(prefix, 420, b"D$     "))
    );
    Some(Label::Multimate)
}

/// Microsoft Multiplan Spreadsheet (.mp / .mod).
pub(crate) fn verify_multiplan(prefix: &[u8], size: u64) -> Option<Label> {
    let valid = starts(prefix, &[0x0C, 0xEF, 0x4D, 0x50])
        || (prefix.len() >= 8
            && (starts(prefix, &[0x0C, 0xE7, 0x00, 0x00, 0x66, 0x09, 0x01, 0x00])
                || starts(prefix, &[0x08, 0xE7, 0x00, 0x00, 0x66, 0x09, 0x01, 0x00])
                || starts(prefix, &[0x0C, 0xE9, 0x00, 0x00, 0x08, 0xAB, 0x08, 0x00])
                || starts(prefix, &[0x0C, 0xEC, 0x00, 0x00, 0x08, 0xAB, 0x08, 0x00])
                || starts(prefix, &[0x0C, 0xED, 0x00, 0x00, 0x08, 0xAB, 0x08, 0x00])
                || starts(prefix, &[0x0A, 0xEC, 0x00, 0x00, 0x08, 0xAB, 0x08, 0x00])
                || starts(prefix, &[0x09, 0x00, 0x04, 0x00, 0x40, 0x01, 0x10, 0x00])
                || starts(prefix, &[0x11, 0xAB, 0x00, 0x00, 0x13, 0xE8, 0x00, 0x00])));
    require!(size >= MIN_MULTIPLAN_SIZE && prefix.len() >= 4 && valid);
    Some(Label::Multiplan)
}

/// NBI Legacy Word Processing Document (.leg).
pub(crate) fn verify_nbi_legacy(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NBI_SIZE
            && prefix.len() >= 16
            && starts(prefix, b"# VE # 4.00")
            && has_non_ascii_text_byte(prefix.get(11..16)?)
    );
    Some(Label::Nbi)
}

/// Peter Norton Computing Norton Guides / Expert Help Database (`.ng` / `.hlp`, 1987) header (`NG\x00\x01` or `EH\x00\x01`).
pub(crate) fn verify_norton_guides_hlp(prefix: &[u8], size: u64) -> Option<Label> {
    const NORTON_GUIDE_MAGIC: &[u8; 4] = b"NG\x00\x01"; // @0..4: standard Norton Guides header
    const EXPERT_HELP_MAGIC: &[u8; 4] = b"EH\x00\x01"; // @0..4: Expert Help compatible database header
    const TITLE_START: usize = 8; // @8..48: 40-byte NUL-terminated database title (CP437 / ASCII)
    const TITLE_END: usize = 48;

    require!(
        size >= MIN_NORTON_GUIDES_HLP_SIZE,
        prefix.len() >= TITLE_END,
        starts(prefix, NORTON_GUIDE_MAGIC) || starts(prefix, EXPERT_HELP_MAGIC),
        u8_at(prefix, TITLE_START)?.is_ascii_graphic(),
        get_slice(prefix, TITLE_START, TITLE_END)?.contains(&0x00),
        get_slice(prefix, TITLE_START, TITLE_END)?.iter().all(|&b| b == 0x00 || b >= 0x20)
    );
    Some(Label::NortonGuidesHlp)
}

/// Novell / Tumbleweed Envoy Document (.evy).
pub(crate) fn verify_novell_envoy(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ENVOY_SIZE
            && prefix.len() >= 4
            && (starts(prefix, &[0x32, 0x5E, 0x10, 0x10])
                || starts(prefix, &[0xB2, 0x97, 0xE1, 0x69]))
    );
    Some(Label::Evy)
}

/// Novell NetWare / Message Librarian Message Catalog (`.msg`, 1989) Novell.
pub(crate) fn verify_novell_netware_message_msg(prefix: &[u8], size: u64) -> Option<Label> {
    let check_len = prefix.len().min(128);
    require!(
        size >= MIN_NOVELL_NETWARE_MESSAGE_MSG_SIZE,
        check_len >= 48,
        starts(prefix, b"NetWare Message File: ")
            || starts(prefix, b"Novell Message Librarian Data File Version")
            || starts(prefix, b"DOS Client Message File: "),
        get_slice(prefix, 22, check_len)?.contains(&0x1A)
    );
    Some(Label::NovellNetwareMessageMsg)
}

/// Novell NetWare SYS$HELP Help File (`.hep`, 1989) Novell.
pub(crate) fn verify_novell_sys_help_hep(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NOVELL_SYS_HELP_HEP_SIZE,
        prefix.len() >= 160,
        starts(prefix, b"SYS$HELP help file.\r\n\r"),
        get_slice(prefix, 22, 160)?.contains(&0x1A)
    );
    Some(Label::NovellSysHelpHep)
}

/// BlackBox Component Builder / Oberon-2 Compound Document (.odc, 1997) Oberon microsystems.
pub(crate) fn verify_oberon_odc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_OBERON_ODC_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"CDOo\0\0\0\0\x82\xf1Documents.StdDocumentDesc\0\xf1D")
    );
    require!(has_non_ascii_text_byte(prefix.get(..40)?));
    Some(Label::OberonOdc)
}

/// Microsoft OneNote Table of Contents v2 (.onetoc2).
pub(crate) fn verify_onenote_toc2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ONENOTE_TOC2_SIZE,
        prefix.len() >= 64,
        get_slice(prefix, 0, 16)? == ONENOTE_TOC2_FILE_TYPE_GUID,
        get_slice(prefix, 16, 32)?.iter().any(|&b| b != 0),
        get_slice(prefix, 32, 48)?.iter().all(|&b| b == 0),
        get_slice(prefix, 48, 64)? == ONENOTE_TOC2_FORMAT_GUID
    );
    Some(Label::Onetoc2)
}

/// IBM OS/2 Presentation Manager IPF Hypertext Help / Book (.hlp / .inf).
pub(crate) fn verify_os2_help_inf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_OS2_HELP_INF_SIZE
            && prefix.len() >= 32
            && le32(prefix, 12).unwrap_or(0) >= 64
            && (le16(prefix, 16).unwrap_or(0) > 0 || le16(prefix, 24).unwrap_or(0) > 0)
    );
    Some(Label::Os2Inf)
}

/// Central Point PC Tools Desktop Help File (`"Desktop Help File    Version "`).
pub(crate) fn verify_pctools_msg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PCTOOLS_MSG_SIZE, starts(prefix, b"Desktop Help File    Version "));
    Some(Label::PctoolsMsg)
}

/// PLATO / NovaNET TUTOR Lesson Source Block (.1t / .1b).
pub(crate) fn verify_plato_lesson(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PLATO_LESSON_SIZE,
        prefix.len() >= 32,
        u8_at(prefix, 0)? == b' ',
        get_slice(prefix, 1, 4)?.iter().all(|b| b.is_ascii_digit()),
        get_slice(prefix, 4, 8)? == b" \r\n\x85",
        contains(prefix.get(8..32)?, b"\r\n\x84")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::PlatoLesson)
}

/// Sybase / Appeon PowerBuilder Standalone DataWindow PSR (Unicode or ANSI, `.psr` / `.dw`).
pub(crate) fn verify_powerbuilder_psr(prefix: &[u8], size: u64) -> Option<Label> {
    const PSR_UTF16: &[u8; 48] =
        b"\x82\0\0\0P\0l\0a\0t\0f\0o\0r\0m\0 \0i\0n\0d\0e\0p\0e\0n\0d\0e\0n\0t\0 \0P\0";
    const PSR_ANSI: &[u8; 41] = b"\x41\0\0\0Platform independent PSR File Format,";

    require!(size >= MIN_POWERBUILDER_PSR_SIZE, prefix.len() >= 41);
    if prefix.len() >= 48 && starts(prefix, PSR_UTF16) {
        return Some(Label::PowerbuilderPsr);
    }
    if starts(prefix, PSR_ANSI) {
        return Some(Label::PowerbuilderPsr);
    }
    None
}

/// Adobe Swatch Exchange (PRONOM `fmt/1816`).
pub(crate) fn verify_pronom_adobe_swatch_exchange_ase(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_ADOBE_SWATCH_EXCHANGE_ASE_SIZE,
        prefix.len() >= 6,
        starts(prefix, bytes!(b"ASEF", &[0x00, 0x01]))
    );
    Some(Label::AdobeSwatchExchangeAse)
}

/// ADRIFT Text Adventure File (.taf, PRONOM fmt/1584).
pub(crate) fn verify_pronom_adrift_taf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_ADRIFT_TAF_SIZE,
        prefix.len() >= 14,
        starts(prefix, bytes!(b"<B?", &[0xC9, 0x6A, 0x87, 0xC2, 0xCF]))
    );
    Some(Label::AdriftTaf)
}

/// Advanced Forensic Format (PRONOM `fmt/844`).
pub(crate) fn verify_pronom_advanced_forensic_format_aff(
    prefix: &[u8], size: u64,
) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_ADVANCED_FORENSIC_FORMAT_AFF_SIZE,
        prefix.len() >= 8,
        starts(prefix, bytes!(b"AFF10", &[0x0D, 0x0A, 0x00]))
    );
    Some(Label::AdvancedForensicFormatAff)
}

/// Asylum Music Format (PRONOM `fmt/1622`).
pub(crate) fn verify_pronom_asylum_music_format_amf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_ASYLUM_MUSIC_FORMAT_AMF_SIZE,
        prefix.len() >= 19,
        starts(prefix, b"ASYLUM Music Format")
    );
    Some(Label::AsylumMusicFormatAmf)
}

/// Atari ST Multi Palette Picture File (.mpp, PRONOM fmt/1471).
pub(crate) fn verify_pronom_atari_mpp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_ATARI_MPP_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"MPP"),
        u8_at(prefix, 3)? <= 3,
        u8_at(prefix, 4)? <= 7,
        eq_at(prefix, 5, hex2bin!("00 00 00"))
    );
    Some(Label::AtariMpp)
}

/// Autorun Maestro Menu File (PRONOM `fmt/1461`).
pub(crate) fn verify_pronom_autorun_maestro_menu_file_mnu(
    prefix: &[u8], size: u64,
) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_AUTORUN_MAESTRO_MENU_FILE_MNU_SIZE,
        prefix.len() >= 16,
        starts(prefix, bytes!(b"MENU", &[0x01])),
        has_non_ascii_text_byte(prefix.get(5..16)?)
    );
    Some(Label::AutorunMaestroMenuFileMnu)
}

/// AVCHD Thumbnail Index File (PRONOM `fmt/1077`).
pub(crate) fn verify_pronom_avchd_thumbnail_index_file_tid(
    prefix: &[u8], size: u64,
) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_AVCHD_THUMBNAIL_INDEX_FILE_TID_SIZE,
        prefix.len() >= 8,
        starts(prefix, b"TIDX0100")
    );
    Some(Label::AvchdThumbnailIndexFileTid)
}

/// BIM Metadata File (PRONOM `fmt/1722`).
pub(crate) fn verify_pronom_bim_metadata_file(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_BIM_METADATA_FILE_SIZE,
        prefix.len() >= 86,
        starts(
            prefix,
            bytes!(
                &[0x00, 0x1F, 0x01],
                b"4urn:schemas-professionalDisc:realTimeMeta:cameraMeta",
                &[0x1C],
                b"./RealTimeMetaCameraMeta.xsd",
                &[0x00]
            )
        )
    );
    Some(Label::BimMetadataFile)
}

/// BioSemi 24-Bit BDF Electrophysiology (PRONOM `fmt/2102`).
pub(crate) fn verify_pronom_biosemi_24_bit_bdf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_BIOSEMI_24_BIT_BDF_SIZE,
        prefix.len() >= 8,
        starts(prefix, bytes!(&[0xFF], b"BIOSEMI"))
    );
    Some(Label::Biosemi24BitBdf)
}

/// Blitz3D File Format (PRONOM `fmt/1182`).
pub(crate) fn verify_pronom_blitz3d_file_format_b3d(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_BLITZ3D_FILE_FORMAT_B3D_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"BB3D"),
        has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    Some(Label::Blitz3dFileFormatB3d)
}

/// CATIA Drawing (PRONOM `fmt/1615`).
pub(crate) fn verify_pronom_catia_drawing(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_CATIA_DRAWING_SIZE,
        prefix.len() >= 8,
        starts(prefix, bytes!(b"V5_CFV2", &[0x00]))
    );
    Some(Label::CatiaDrawing)
}

/// ChemStation Gas Chromatography/Mass Spectrum (GC/MS) Data File (PRONOM `fmt/2101`).
pub(crate) fn verify_pronom_chemstation_gas_chromatography_mass_ms(
    prefix: &[u8], size: u64,
) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_CHEMSTATION_GAS_CHROMATOGRAPHY_MASS_MS_SIZE,
        prefix.len() >= 256,
        starts(prefix, hex2bin!("01 32 00 00")),
        u8_at(prefix, 4)? == 0x08 || starts(prefix.get(4..)?, bytes!(&[0x11], b"GC / MS D"))
    );
    require!(
        eq_at(prefix, 248, hex2bin!("00 00 00 02"))
            || eq_at(prefix, 252, hex2bin!("00 00 02 00"))
            || eq_at(prefix, 276, hex2bin!("00 00 02 00"))
    );
    Some(Label::ChemstationGasChromatographyMassMs)
}

/// Common Instrument File Format (.ci1 / .ci2, PRONOM fmt/1887).
pub(crate) fn verify_pronom_common_instrument_ci1(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_COMMON_INSTRUMENT_CI1_SIZE,
        prefix.len() >= 26,
        starts(prefix, hex2bin!("42 BD 0A 00 01")),
        eq_at(prefix, 20, b"v1.0") || eq_at(prefix, 20, b"v2.0")
    );
    Some(Label::CommonInstrumentCi1)
}

/// Common Loudspeaker Format Binary (.cf1 / .cf2, PRONOM fmt/1944).
pub(crate) fn verify_pronom_common_loudspeaker_cf1(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_COMMON_LOUDSPEAKER_CF1_SIZE,
        prefix.len() >= 26,
        starts(prefix, hex2bin!("40 BD 0A 00 01")),
        eq_at(prefix, 20, b"v1.0") || eq_at(prefix, 20, b"v2.0")
    );
    Some(Label::CommonLoudspeakerCf1)
}

/// Bruker AXS / Siemens DIFFRAC.plus & DIFFRAC.suite Raw Data File Format (PRONOM `fmt/1120` v1, `fmt/1121` v2).
pub(crate) fn verify_pronom_diffracplus_raw_data(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_DIFFRACPLUS_RAW_DATA_SIZE,
        prefix.len() >= 8,
        starts(prefix, bytes!(b"RAW1.00", &[0x00]))
            || starts(prefix, bytes!(b"RAW1.01", &[0x00]))
            || starts(prefix, bytes!(b"RAW2.00", &[0x00]))
            || starts(prefix, bytes!(b"RAW4.00", &[0x00]))
    );
    Some(Label::DiffracplusRawData)
}

/// DNA Sequence Chromatogram File (PRONOM `fmt/1554`).
pub(crate) fn verify_pronom_dna_sequence_chromatogram_file_scf(
    prefix: &[u8], size: u64,
) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_DNA_SEQUENCE_CHROMATOGRAM_FILE_SCF_SIZE,
        prefix.len() >= 16,
        starts(prefix, b".scf"),
        has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    Some(Label::DnaSequenceChromatogramFileScf)
}

/// FinePrint Saved Output (.fp, PRONOM fmt/1396).
pub(crate) fn verify_pronom_fineprint_fp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_FINEPRINT_FP_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"FINC") || starts(prefix, b"FINE"),
        matches!(u8_at(prefix, 4)?, 0x01..=0x03),
        has_non_ascii_text_byte(prefix.get(5..16)?)
    );
    Some(Label::FineprintFp)
}

/// G9B MSX2+ V9990 Graphics Format Bitmap (.g9b, PRONOM fmt/1787).
pub(crate) fn verify_pronom_g9b_bitmap(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_G9B_BITMAP_SIZE,
        prefix.len() >= 14,
        starts(prefix, b"G9B"),
        matches!(u8_at(prefix, 3)?, 0x0B | 0x00),
        u8_at(prefix, 4)? == 0x00,
        matches!(u8_at(prefix, 5)?, 0 | 2 | 4 | 8 | 16)
    );
    Some(Label::G9bBitmap)
}

/// Garmin Track Log File (.gmn, PRONOM fmt/1679).
pub(crate) fn verify_pronom_garmin_gmn(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_GARMIN_GMN_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"<@gArMiN@>"),
        has_non_ascii_text_byte(prefix.get(10..16)?)
    );
    Some(Label::GarminGmn)
}

/// GSSI SIR-10 RADAN Ground Penetrating Radar Data (.dzt, PRONOM fmt/362).
pub(crate) fn verify_pronom_gssi_dzt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_GSSI_DZT_SIZE,
        prefix.len() >= 16,
        starts(prefix, hex2bin!("FF 00 00 04")) || starts(prefix, hex2bin!("FF 07 00 04")),
        (32..=16384).contains(&le16(prefix, 4).unwrap_or(0)),
        matches!(le16(prefix, 6), Some(8 | 12 | 16 | 32))
    );
    Some(Label::GssiDzt)
}

/// Show Partner GX2 Graphics File (.gx2, PRONOM fmt/1789).
pub(crate) fn verify_pronom_gx2_bitmap(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_GX2_BITMAP_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"GX2\x01"),
        u8_at(prefix, 5)? == 0x00,
        le16(prefix, 7).unwrap_or(0) > 0,
        le16(prefix, 9).unwrap_or(0) > 0
    );
    Some(Label::Gx2Bitmap)
}

/// Haiku Vector Icon Format (PRONOM `fmt/1791`).
pub(crate) fn verify_pronom_haiku_vector_icon_format_hvif(
    prefix: &[u8], size: u64,
) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_HAIKU_VECTOR_ICON_FORMAT_HVIF_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"ncif"),
        has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    Some(Label::HaikuVectorIconFormatHvif)
}

/// InfoFile FLR Database (.flr, PRONOM fmt/1785).
pub(crate) fn verify_pronom_infofile_flr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_INFOFILE_FLR_SIZE,
        prefix.len() >= 20,
        starts(prefix, bytes!(b"72249", &[0x00, 0x00, 0x01])),
        be32(prefix, 8).unwrap_or(0) > 0,
        be32(prefix, 12).unwrap_or(0) > 0
    );
    Some(Label::InfofileFlr)
}

/// Interleaved ADX Audio Format (AIX) (PRONOM `fmt/841`).
pub(crate) fn verify_pronom_interleaved_adx_audio_aix(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_INTERLEAVED_ADX_AUDIO_AIX_SIZE,
        prefix.len() >= 16,
        starts(
            prefix,
            bytes!(
                b"AIXF",
                &[0x00, 0x00, 0x17, 0xF8, 0x01, 0x00, 0x00, 0x14, 0x00, 0x00, 0x08, 0x00]
            )
        )
    );
    Some(Label::InterleavedAdxAudioAix)
}

/// LEADTOOLS Compressed Image (.cmp, PRONOM fmt/1337).
pub(crate) fn verify_pronom_leadtools_cmp(prefix: &[u8], size: u64) -> Option<Label> {
    const LEAD_UPPER_MAGIC: &[u8; 4] = b"LEAD"; // @0..4: uppercase LEADTOOLS CMP header magic
    const LEAD_TITLE_MAGIC: &[u8; 4] = b"Lead"; // @0..4: mixed-case LEADTOOLS CMP header magic
    const RESERVED_ZERO_OFFSET: usize = 8; // @8..16: 8 reserved NUL bytes before dimensions
    const WIDTH_OFFSET: usize = 16; // @16..18: u16 LE image width (> 0)
    const HEIGHT_OFFSET: usize = 18; // @18..20: u16 LE image height (> 0)

    require!(
        size >= MIN_PRONOM_LEADTOOLS_CMP_SIZE,
        prefix.len() >= 20,
        starts(prefix, LEAD_UPPER_MAGIC) || starts(prefix, LEAD_TITLE_MAGIC),
        eq_at(prefix, 4, hex2bin!("00 01 01 00"))
            || (u8_at(prefix, 4).unwrap_or(0) == 0
                && u8_at(prefix, 5).unwrap_or(0) <= 4
                && u8_at(prefix, 7).unwrap_or(0) == 0),
        eq_at(prefix, RESERVED_ZERO_OFFSET, &[0u8; 8]),
        le16(prefix, WIDTH_OFFSET).unwrap_or(0) > 0,
        le16(prefix, HEIGHT_OFFSET).unwrap_or(0) > 0
    );
    Some(Label::LeadtoolsCmp)
}

/// Leonardo Image Format (PRONOM `fmt/1217`).
pub(crate) fn verify_pronom_leonardo_image_format_leo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_LEONARDO_IMAGE_FORMAT_LEO_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"Leo zone format ")
    );
    Some(Label::LeonardoImageFormatLeo)
}

/// Mapsforge Binary Vector Map (PRONOM `fmt/915`).
pub(crate) fn verify_pronom_mapsforge_binary_vector_map(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_MAPSFORGE_BINARY_VECTOR_MAP_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"mapsforge binary OSM")
    );
    Some(Label::MapsforgeBinaryVectorMap)
}

/// Mar Archive (PRONOM `fmt/1483`).
pub(crate) fn verify_pronom_mar_archive_mac(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_MAR_ARCHIVE_MAC_SIZE,
        prefix.len() >= 4,
        starts(prefix, bytes!(b"MAR", &[0x80]))
    );
    Some(Label::MarArchiveMac)
}

/// Plextalk Project File (imdn) (PRONOM `fmt/2036`).
pub(crate) fn verify_pronom_plextalk_project_file_imdn(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_PLEXTALK_PROJECT_FILE_IMDN_SIZE,
        prefix.len() >= 83,
        starts(prefix, hex2bin!("01 FF 00 FF")),
        eq_at(prefix, 36, b"ImdTxtTabl.imtt"),
        eq_at(prefix, 68, b"ImdPhrInfo.imph")
    );
    Some(Label::PlextalkProjectFileImdn)
}

/// LLNL PACT / Silo Portable Database (`.pdb` / `.silo`, PRONOM `fmt/1064` v1, `fmt/1065` v2, `fmt/1066` v3).
pub(crate) fn verify_pronom_portable_database_pdb(prefix: &[u8], size: u64) -> Option<Label> {
    const PDB_V1_MAGIC: &[u8; 11] = b"!<><PDB><>!"; // @0..11: PACT Portable Database v1 (PRONOM fmt/1064)
    const PDB_V3_MAGIC: &[u8; 18] = b"!<><>!<><>!<><>!\n$"; // @0..18: PACT Portable Database v3 (PRONOM fmt/1066)
    const PDB_V2_PREFIX: &[u8; 7] = b"!<<PDB:"; // @0..7: PACT / Silo Portable Database v2/v3 prefix

    require!(size >= MIN_PRONOM_PORTABLE_DATABASE_PDB_SIZE);
    if prefix.len() >= PDB_V1_MAGIC.len() && starts(prefix, PDB_V1_MAGIC) {
        return Some(Label::PortableDatabasePdb);
    }
    if prefix.len() >= PDB_V3_MAGIC.len() && starts(prefix, PDB_V3_MAGIC) {
        return Some(Label::PortableDatabasePdb);
    }
    if prefix.len() >= 16
        && starts(prefix, PDB_V2_PREFIX)
        && (eq_at(prefix, 9, b">>!\n$") || eq_at(prefix, 8, b">>!\n$"))
    {
        return Some(Label::PortableDatabasePdb);
    }
    None
}

/// Portable Form File (PRONOM `fmt/322`).
pub(crate) fn verify_pronom_portable_form_file_pff(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_PORTABLE_FORM_FILE_PFF_SIZE,
        prefix.len() >= 8,
        starts(prefix, bytes!(b"PFF", &[0x00, 0x07, 0x02, 0x00, 0x06]))
    );
    Some(Label::PortableFormFilePff)
}

/// PowerVR Object Data (PRONOM `fmt/782`).
pub(crate) fn verify_pronom_powervr_object_data_pod(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_POWERVR_OBJECT_DATA_POD_SIZE,
        prefix.len() >= 27,
        starts(
            prefix,
            bytes!(
                &[0xE8, 0x03, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00],
                b"AB.POD.2.0",
                &[0x00, 0xE8, 0x03, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00]
            )
        )
    );
    Some(Label::PowervrObjectDataPod)
}

/// Q&A Word Processor Document (PRONOM `fmt/1045`).
pub(crate) fn verify_pronom_q_a_word_processor(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_Q_A_WORD_PROCESSOR_SIZE,
        prefix.len() >= 16,
        starts(prefix, bytes!(b"TBWP", &[0x00]))
    );
    // NUL-terminated ASCII creation date string at @5..
    get_slice(prefix, 5, prefix.len().min(32))?.iter().position(|&b| b == 0)?;
    Some(Label::QAWordProcessor)
}

/// Quicken 3 Database File (PRONOM `fmt/1861`).
pub(crate) fn verify_pronom_quicken_3_database_file_qst(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_QUICKEN_3_DATABASE_FILE_QST_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"QW Ver. 3.00.00 ")
    );
    Some(Label::Quicken3DatabaseFileQst)
}

/// ROS Bag Format (PRONOM `fmt/2093`).
pub(crate) fn verify_pronom_ros_bag_format(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_ROS_BAG_FORMAT_SIZE,
        prefix.len() >= 13,
        starts(prefix, b"#ROSBAG V2.0"),
        matches!(prefix.get(12), Some(b'\r' | b'\n'))
    );
    Some(Label::RosBagFormat)
}

/// Serif PagePlus Publication (PRONOM `fmt/1537`).
pub(crate) fn verify_pronom_serif_pageplus_publication_ppp(
    prefix: &[u8], size: u64,
) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_SERIF_PAGEPLUS_PUBLICATION_PPP_SIZE,
        prefix.len() >= 4,
        starts(prefix, hex2bin!("CE 19 CE 19"))
    );
    Some(Label::SerifPageplusPublicationPpp)
}

/// SpritePad C64 Sprite Image (.spd, PRONOM fmt/1561, `"SPD"` + version `1..=5` + VIC-II color indices `<= 15`).
pub(crate) fn verify_pronom_spritepad_spd(prefix: &[u8], size: u64) -> Option<Label> {
    const SPRITEPAD_MAGIC: &[u8; 3] = b"SPD"; // @0..3: SpritePad C64 header signature
    const VERSION_OFFSET: usize = 3; // @3: u8 SpritePad format version (1..=5)
    const BG_COLOR_OFFSET: usize = 6; // @6: u8 VIC-II background color index (0..=15)
    const MC1_COLOR_OFFSET: usize = 7; // @7: u8 VIC-II multicolor 1 index (0..=15)
    const MC2_COLOR_OFFSET: usize = 8; // @8: u8 VIC-II multicolor 2 index (0..=15)

    require!(
        size >= MIN_PRONOM_SPRITEPAD_SPD_SIZE,
        prefix.len() >= 18,
        starts(prefix, SPRITEPAD_MAGIC),
        (1..=5).contains(&u8_at(prefix, VERSION_OFFSET)?),
        u8_at(prefix, BG_COLOR_OFFSET)? <= 15,
        u8_at(prefix, MC1_COLOR_OFFSET)? <= 15,
        u8_at(prefix, MC2_COLOR_OFFSET)? <= 15,
        has_non_ascii_text_byte(prefix.get(VERSION_OFFSET..prefix.len().min(24))?)
    );
    Some(Label::SpritepadSpd)
}

/// Superscape Virtual Reality Format (PRONOM `fmt/734`).
pub(crate) fn verify_pronom_superscape_virtual_reality_format_svr(
    prefix: &[u8], size: u64,
) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_SUPERSCAPE_VIRTUAL_REALITY_FORMAT_SVR_SIZE,
        prefix.len() >= 9,
        starts(prefix, bytes!(b"SVR", &[0x08, 0x00, 0x00, 0x00, 0x01, 0xEC]))
    );
    Some(Label::SuperscapeVirtualRealityFormatSvr)
}

/// True Colour Picture [Spooky Sprites] (Atari Falcon) (PRONOM `fmt/1607`).
pub(crate) fn verify_pronom_true_colour_picture_spooky_trp(
    prefix: &[u8], size: u64,
) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_TRUE_COLOUR_PICTURE_SPOOKY_TRP_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"tru?"),
        has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    Some(Label::TrueColourPictureSpookyTrp)
}

/// TUNDRA (PRONOM `fmt/1603`).
pub(crate) fn verify_pronom_tundra_tnd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_TUNDRA_TND_SIZE,
        prefix.len() >= 9,
        starts(prefix, bytes!(&[0x18], b"TUNDRA24"))
    );
    Some(Label::TundraTnd)
}

/// Vips Image (PRONOM `fmt/1811`).
pub(crate) fn verify_pronom_vips_image_v(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_VIPS_IMAGE_V_SIZE,
        prefix.len() >= 4,
        starts(prefix, hex2bin!("08 F2 A6 B6"))
    );
    Some(Label::VipsImageV)
}

/// Windows Address Book (PRONOM `fmt/1255`).
pub(crate) fn verify_pronom_windows_address_book_wab(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_WINDOWS_ADDRESS_BOOK_WAB_SIZE,
        prefix.len() >= 16,
        starts(prefix, hex2bin!("9C CB CB 8D 13 75 D2 11 91 58 00 C0 4F 79 56 A4"))
    );
    Some(Label::WindowsAddressBookWab)
}

/// Xar Image Format (PRONOM `fmt/922`).
pub(crate) fn verify_pronom_xar_image_format(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_XAR_IMAGE_FORMAT_SIZE,
        prefix.len() >= 8,
        starts(prefix, bytes!(b"XARA", &[0xA3, 0xA3, 0x0D, 0x0A]))
    );
    Some(Label::XarImageFormat)
}

/// XL-Paint MaX (PRONOM `fmt/1658`).
pub(crate) fn verify_pronom_xl_paint_max(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_XL_PAINT_MAX_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"XLPM"),
        has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    Some(Label::XlPaintMax)
}

/// XL-Paint (PRONOM `fmt/1659`).
pub(crate) fn verify_pronom_xl_paint_raw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PRONOM_XL_PAINT_RAW_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"XLPB"),
        has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    Some(Label::XlPaintRaw)
}

/// XV Thumbnail (PRONOM `fmt/1497`, `"P7 332\n#XVVERSION"` or `"P7 332\n#...\n#END_OF_COMMENTS"` + 8-bit 332 raster payload).
pub(crate) fn verify_pronom_xv_thumbnail_p7(prefix: &[u8], size: u64) -> Option<Label> {
    const XV_THUMB_MAGIC: &[u8; 7] = b"P7 332\n"; // @0..7: XV 332-palette thumbnail header magic
    const XV_VERSION_TAG: &[u8; 10] = b"#XVVERSION"; // @7..17: original xv `#XVVERSION` comment line
    const END_OF_COMMENTS_TAG: &[u8; 17] = b"\n#END_OF_COMMENTS"; // end of XV header comment block

    let has_xv_version = eq_at(prefix, 7, XV_VERSION_TAG);
    let has_end_comments =
        prefix.get(7) == Some(&b'#') && scan_range(prefix, 6, 256, END_OF_COMMENTS_TAG);
    require!(
        size >= MIN_PRONOM_XV_THUMBNAIL_P7_SIZE,
        prefix.len() >= 24,
        starts(prefix, XV_THUMB_MAGIC),
        has_xv_version || has_end_comments,
        has_non_ascii_text_byte(prefix.get(16..prefix.len().min(512))?)
    );
    Some(Label::XvThumbnailP7)
}

/// Canon ZoomBrowser EX Thumbnail Cache (`ZbThumbnail.info`, PRONOM `fmt/1496`).
pub(crate) fn verify_pronom_zoombrowser_ex_thumbnail_cache_info(
    prefix: &[u8], size: u64,
) -> Option<Label> {
    const ZBEX_V4_HDR: &[u8; 20] = bytes!(
        b"zbex",
        &[
            0x04, 0x00, 0x00, 0x4C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x04,
            0x00, 0x00
        ]
    );
    if size >= 20 && prefix.len() >= 20 && starts(prefix, ZBEX_V4_HDR) {
        return Some(Label::ZoombrowserExThumbnailCacheInfo);
    }
    require!(
        size >= MIN_PRONOM_ZOOMBROWSER_EX_THUMBNAIL_CACHE_INFO_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"zbex"),
        has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    Some(Label::ZoombrowserExThumbnailCacheInfo)
}

/// Psion Series 3 Word: `"PSIONWPDATAFILE\0"`.
pub(crate) fn verify_psion3_word(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PSION3_WORD_SIZE, starts(prefix, b"PSIONWPDATAFILE\0"));
    Some(Label::Psion3Word)
}

/// Borland Quattro Pro for Windows Encrypted/Packed Notebook (.wb1) Borland.
pub(crate) fn verify_quattro_pro_wb1(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_QUATTRO_PRO_WB1_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"WWBB1111\xf9\xd3\xf4\xe6\x15\x0dJ[\\S")
    );
    Some(Label::QuattroProWb1)
}

/// RagTime 4–6 (`43 23 2B 44 A4 43 4D A5 48 64 72`) and RagTime Classic 2.x/3.x (`68 8F 68 8F 68 8F` at `@16..22`).
pub(crate) fn verify_ragtime_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_RAGTIME_SIZE, prefix.len() >= 36);
    if starts(prefix, b"C#+D\xA4CM\xA5Hdr") {
        return Some(Label::Ragtime);
    }
    if get_slice(prefix, 0, 2)? == [0x00, 0x00]
        && get_slice(prefix, 6, 16)? == [0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        && get_slice(prefix, 16, 22)? == [0x68, 0x8F, 0x68, 0x8F, 0x68, 0x8F]
        && get_slice(prefix, 30, 36)? == [0x68, 0x8F, 0x68, 0x8F, 0x68, 0x8F]
    {
        return Some(Label::Ragtime);
    }
    None
}

/// Rillsoft Project Management Schedule (.rpj / .mbk) Rillsoft.
pub(crate) fn verify_rillsoft_rpj(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RILLSOFT_RPJ_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\xff\xfe\xff\x0cR\0I\0L\0L\0S\0O\0F\0T\0-\0P\0R\0J\0")
            || starts(
                prefix,
                b"\xff\xfe\xff\0\xff\xfe\xff\x0cR\0I\0L\0L\0S\0O\0F\0T\0-\0P\0R\0J\0"
            )
    );
    Some(Label::RillsoftRpj)
}

/// Sankom Audytor OZC/CO Heating & HVAC Engineering Project (.ozd) Sankom.
pub(crate) fn verify_sankom_ozd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SANKOM_OZD_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"PWFILEVERINFOZC__DAT7.00\x05")
    );
    Some(Label::SankomOzd)
}

/// Movie Magic Screenwriter Screenplay (.scw / .mms).
pub(crate) fn verify_screenwriter(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SCREENWRITER_SIZE
            && prefix.len() >= 16
            && (starts(prefix, b"SCWX") || starts(prefix, b"SCRIPT") || starts(prefix, b"SCRI"))
            && prefix.get(4..16)?.contains(&0)
    );
    Some(Label::Scw)
}

/// Brightbill-Roberts Show Partner Animator Help File (`"Show Partner Animator Help File"`).
pub(crate) fn verify_showpartner_hlp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SHOWPARTNER_HLP_SIZE, starts(prefix, b"Show Partner Animator Help File"));
    Some(Label::ShowpartnerHlp)
}

/// Application Systems Heidelberg Signum! 1/2 Document (`.sdo`, `"sdoc0001"` / `"sdoc0002"`).
pub(crate) fn verify_signum_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SIGNUM_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"sdoc0001") || starts(prefix, b"sdoc0002"),
        prefix.get(8..16)?.contains(&0)
    );
    Some(Label::SignumSdo)
}

/// SoftPlan Architectural Design reView Project (.spv) SoftPlan Systems.
pub(crate) fn verify_softplan_spv(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SOFTPLAN_SPV_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"\x68\x63\x60\xa2\xe6\x6f\x6c\xa7\x06\0\0\0\x1a\0\0\0SoftPlan_reView ")
    );
    Some(Label::SoftplanSpv)
}

/// Star Division StarWriter DOS Legacy Document Template (.vor, 1991) Star Division.
pub(crate) fn verify_starwriter_legacy_vor(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_STARWRITER_LEGACY_VOR_SIZE,
        prefix.len() >= 600,
        starts(prefix, b".\\\\\\ WRITER "),
        has_non_ascii_text_byte(prefix.get(12..600)?)
    );
    Some(Label::StarwriterLegacyVor)
}

/// The Learning Company Student Writing Center Document (`\x1ATLC\x01FF\x00` / `\x1ATLC\x02FF\x00`).
pub(crate) fn verify_student_writing_center(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_STUDENT_WRITING_CENTER_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"\x1ATLC")
            && u8_at(prefix, 4)? <= 2
            && get_slice(prefix, 5, 8)? == b"FF\x00",
        le16(prefix, 8)? <= 4 || be16(prefix, 8)? <= 4
    );
    Some(Label::StudentWritingCenter)
}

/// Wolters Kluwer Taxprep / Canadian CRA Tax Form Definition (`.tfd`).
pub(crate) fn verify_taxprep_tfd(prefix: &[u8], size: u64) -> Option<Label> {
    const TFD_PAGE_GEOM: &[u8; 16] = hex2bin!("FB 04 00 00 72 06 00 00 14 00 00 00 14 00 00 00");

    require!(
        size >= MIN_TAXPREP_TFD_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"\x04V1."),
        u8_at(prefix, 4)?.is_ascii_digit()
    );
    let tlen = u8_at(prefix, 5)? as usize;
    require!(
        (19..=24).contains(&tlen),
        prefix.len() >= 6 + tlen + 16,
        u8_at(prefix, 6)?.is_ascii_digit(),
        u8_at(prefix, 7)?.is_ascii_digit(),
        eq_at(prefix, 6 + tlen, TFD_PAGE_GEOM)
    );
    Some(Label::TaxprepTfd)
}

/// Microsoft TNEF: `u32 LE 0x223E9F78`, `u16` legacy key, then the first attribute's level byte
/// (`0x01` LVL_MESSAGE or `0x02` LVL_ATTACHMENT).
pub(crate) fn verify_tnef(prefix: &[u8], size: u64) -> Option<Label> {
    const TNEF_SIGNATURE: u32 = 0x223E_9F78; // @0: u32 LE
    const FIRST_LEVEL: usize = 6; // @6: attribute level
    require!(size >= MIN_TNEF_SIZE, le32(prefix, 0)? == TNEF_SIGNATURE);
    require!(matches!(u8_at(prefix, FIRST_LEVEL)?, 0x01 | 0x02));
    Some(Label::Tnef)
}

/// TurboPower PopHelp / Object Professional Help Database (`.hlp`, 1990, TurboPower).
///
/// Header layout:
/// - `@0..4`: `b"INFO"`
/// - `@4..18`: `7 × u16 LE` (`highest_topic`, `biggest_topic`, `index_topics`, `name_table_size`,
///   `max_name_info`, `flags`, `window_width`)
/// - `@18..34`: 16-byte nibble-compression frequency table (or all zeros when uncompressed)
/// - `tt_start = 34 + 2*highest_topic + name_table_size + 2*index_topics`: `highest_topic × 6B`
///   `(u32 LE file_offset, u16 LE comp_size)` topic table; first entry offset equals `tt_start + 6*highest_topic`.
pub(crate) fn verify_turbopower_pophelp(prefix: &[u8], size: u64) -> Option<Label> {
    const HIGHEST_OFF: usize = 4; // @4..6: u16 LE total topic count (>= 1)
    const BIGGEST_OFF: usize = 6; // @6..8: u16 LE largest uncompressed topic (>= 1)
    const INDEXED_OFF: usize = 8; // @8..10: u16 LE pick-list topic count (<= highest)
    const NAMETBL_OFF: usize = 10; // @10..12: u16 LE XOR-0xFF Pascal name table byte size
    const WIN_WIDTH_OFF: usize = 16; // @16..18: u16 LE display window width (10..=255)
    const HDR_LEN: u64 = 34; // 18B scalar header + 16B compression frequency table
    require!(
        size >= MIN_TURBOPOWER_POPHELP_SIZE,
        prefix.len() >= 18,
        starts(prefix, TURBOPOWER_POPHELP_MAGIC)
    );
    let highest = le16(prefix, HIGHEST_OFF)? as u64;
    let biggest = le16(prefix, BIGGEST_OFF)? as u64;
    let indexed = le16(prefix, INDEXED_OFF)? as u64;
    let name_tbl = le16(prefix, NAMETBL_OFF)? as u64;
    let win_width = le16(prefix, WIN_WIDTH_OFF)?;
    let tt_start = HDR_LEN + 2 * highest + name_tbl + 2 * indexed;
    let topic_data_start = tt_start + 6 * highest;
    require!(
        highest >= 1,
        biggest >= 1,
        indexed <= highest,
        (10..=255).contains(&win_width),
        size >= topic_data_start,
        has_non_ascii_text_byte(prefix.get(..18)?)
    );
    if tt_start + 6 <= prefix.len() as u64 {
        let tt_off = tt_start as usize;
        require!(le32(prefix, tt_off)? as u64 == topic_data_start);
    }
    Some(Label::TurbopowerPophelp)
}

/// TurboPower Turbo Professional Reference Manual / TPHelp v1 Database (`.hlp`, 1989, TurboPower).
///
/// Header layout:
/// - `@0..4`: `b"RMP1"`
/// - `@4..12`: `4 × u16 LE` (`highest_topic`, `biggest_topic`, `index_topics`, `extra`)
/// - `@12..28`: 16-byte nibble-compression frequency table
/// - `@28..`: `highest_topic × 24B` Pascal names (`len <= 23`), followed by `highest_topic × 6B`
///   `(u32 LE offset, u16 LE comp_size)` topic records and `highest_topic × 2B` index map;
///   first topic offset equals `28 + 32 * highest_topic`.
pub(crate) fn verify_turbopower_tphelp(prefix: &[u8], size: u64) -> Option<Label> {
    const HIGHEST_OFF: usize = 4; // @4..6: u16 LE total topic count (>= 1)
    const BIGGEST_OFF: usize = 6; // @6..8: u16 LE largest topic size (>= 1)
    const INDEXED_OFF: usize = 8; // @8..10: u16 LE indexed topic count (<= highest)
    const HDR_LEN: u64 = 28; // 12B scalar header + 16B frequency table
    const NAME0_LEN_OFF: usize = 28; // @28: first fixed 24B Pascal topic string length (<= 23)
    require!(
        size >= MIN_TURBOPOWER_TPHELP_SIZE,
        prefix.len() > NAME0_LEN_OFF,
        starts(prefix, TURBOPOWER_TPHELP_MAGIC)
    );
    let highest = le16(prefix, HIGHEST_OFF)? as u64;
    let biggest = le16(prefix, BIGGEST_OFF)? as u64;
    let indexed = le16(prefix, INDEXED_OFF)? as u64;
    let tt_start = HDR_LEN + 24 * highest;
    let topic_data_start = HDR_LEN + 32 * highest;
    require!(
        highest >= 1,
        biggest >= 1,
        indexed <= highest,
        u8_at(prefix, NAME0_LEN_OFF)? <= 23,
        size >= topic_data_start,
        has_non_ascii_text_byte(prefix.get(..12)?)
    );
    if tt_start + 6 <= prefix.len() as u64 {
        let tt_off = tt_start as usize;
        require!(le32(prefix, tt_off)? as u64 == topic_data_start);
    }
    Some(Label::TurbopowerTphelp)
}

/// TurboPower Object Professional / Turbo Professional Compiled Help v2 file (`.hlp`, 1991, TurboPower Software).
///
/// Header layout: `@0..4 = b"TPH2"`, `@4..6 u16 LE highest_topic`, `@6..8 u16 LE max_topic_size`,
/// `@8..10 u16 LE indexed_topics` (`<= highest_topic`), `@10 u8 name_slot_size`,
/// `@11 u8 max_width`, `@12..28` 16-byte nibble-compression frequency table. Following
/// `highest_topic × name_slot_size` title bytes sits the `highest_topic × 6`-byte topic index
/// `(u32 LE offset, u16 LE comp_len)` and `highest_topic × u16 LE` topic IDs, with contiguous
/// topic records spanning from `data_start` to exact `size`.
pub(crate) fn verify_turbopower_tphelp2(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_LEN: usize = 28;
    const HIGHEST_OFF: usize = 4; // @4..6: u16 LE highest_topic (>= 1)
    const MAX_TOPIC_OFF: usize = 6; // @6..8: u16 LE max_topic_size (>= 1)
    const INDEXED_OFF: usize = 8; // @8..10: u16 LE indexed_topics (<= highest_topic)
    const NAME_SLOT_OFF: usize = 10; // @10: u8 topic title slot width in bytes
    const MAX_WIDTH_OFF: usize = 11; // @11: u8 display width in columns

    require!(
        size >= MIN_TURBOPOWER_TPH2_SIZE,
        prefix.len() >= HDR_LEN,
        starts(prefix, TURBOPOWER_TPH2_MAGIC)
    );
    let highest = le16(prefix, HIGHEST_OFF)? as usize;
    let max_topic = le16(prefix, MAX_TOPIC_OFF)?;
    let indexed = le16(prefix, INDEXED_OFF)? as usize;
    let name_slot = u8_at(prefix, NAME_SLOT_OFF)? as usize;
    let max_width = u8_at(prefix, MAX_WIDTH_OFF)?;
    require!(
        (1..=4096).contains(&highest),
        max_topic >= 1,
        indexed <= highest,
        (1..=128).contains(&name_slot),
        (20..=255).contains(&max_width)
    );
    let rec_start = HDR_LEN.checked_add(highest.checked_mul(name_slot)?)?;
    let id_start = rec_start.checked_add(highest.checked_mul(6)?)?;
    let data_start = id_start.checked_add(highest.checked_mul(2)?)?;
    require!((data_start as u64) < size);

    if id_start <= prefix.len() {
        let mut expected = data_start as u64;
        for i in 0..highest {
            let base = rec_start + i * 6;
            let off = le32(prefix, base)? as u64;
            let comp_len = le16(prefix, base + 4)? as u64;
            require!(off == expected, comp_len >= 1);
            expected = expected.checked_add(comp_len)?;
            require!(expected <= size);
        }
        require!(expected == size);
    } else if rec_start + 6 <= prefix.len() {
        let first_off = le32(prefix, rec_start)? as u64;
        let first_len = le16(prefix, rec_start + 4)? as u64;
        require!(first_off == data_start as u64, first_len >= 1);
    } else {
        return None;
    }

    Some(Label::TurbopowerTphelp2)
}

/// Wildcat! BBS Compiled wcCODE module (`"GHSH"`).
pub(crate) fn verify_wildcat_wcx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 16,
        prefix.len() >= 16,
        starts(prefix, b"GHSH"),
        has_non_ascii_text_byte(prefix.get(4..16)?),
    );
    Some(Label::Wcx)
}

/// Microsoft Windows 1.x-3.1 Calendar (.cal).
pub(crate) fn verify_windows_calendar(prefix: &[u8], size: u64) -> Option<Label> {
    const CAL_MAGIC_8: &[u8; 8] = hex2bin!("B5 A2 B0 B3 B3 B0 A2 B5"); // @0..8: palindromic Calendar magic
    const RESERVED_OFF: usize = 8; // @8..12: 4 reserved zero bytes
    const CDD_OFF: usize = 12; // @12: u16 LE number of 12-byte Date Descriptors at @64

    require!(
        size >= MIN_WINDOWS_CAL_SIZE,
        prefix.len() >= 18,
        starts(prefix, CAL_MAGIC_8),
        le32(prefix, RESERVED_OFF) == Some(0)
    );
    let cdd = le16(prefix, CDD_OFF)? as u64;
    require!(64 + cdd * 12 <= size);
    Some(Label::WinCal)
}

/// EVC Electronic WinOLS ECU Map Pack (.kp) EVC electronic.
pub(crate) fn verify_winols_kp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WINOLS_KP_SIZE,
        prefix.len() >= 28,
        (starts(prefix, b"\x0b\0\0\0WinOLS File\0") && le32(prefix, 16).unwrap_or(0) > 0)
            || (starts(prefix, b"WinOLS File\0") && le32(prefix, 12).unwrap_or(0) > 0)
    );
    Some(Label::WinolsKp)
}

/// WoodWorks Sizer / Connections Project (.cwb) Canadian Wood Council.
pub(crate) fn verify_woodworks_cwb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WOODWORKS_CWB_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"cwc22\0") || starts(prefix, b"cwc21\0"),
        get_slice(prefix, 6, 14)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::WoodworksCwb)
}

/// MicroPro WordStar 2000 Document (`.ws2`, `\x1DWS2000` or `\x7F WS2000\xFF`).
pub(crate) fn verify_wordstar_2000_doc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WORDSTAR_2000_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"\x1DWS2000") || starts(prefix, b"\x7F WS2000\xFF")
    );
    Some(Label::Wordstar2000)
}

/// Zebra ZBR1BGJB XML-Headered Card Print Job (`.prn`, `"ZBR1BGJB"` + `<?xml`).
pub(crate) fn verify_zbr1bgjb_prn(prefix: &[u8], size: u64) -> Option<Label> {
    const ZEBRA_ZBR1_MAGIC: &[u8; 8] = b"ZBR1BGJB"; // @0..8
    const XML_DECL_PREFIX: &[u8; 5] = b"<?xml"; // @16..21

    require!(
        size >= MIN_ZBR1BGJB_PRN_SIZE,
        prefix.len() >= 21,
        starts(prefix, ZEBRA_ZBR1_MAGIC),
        le32(prefix, 8)? <= 0x0010_0000, // @8..12: flags / version
        (16..=0x0100_0000).contains(&le32(prefix, 12)?), // @12..16: XML metadata length
        eq_at(prefix, 16, XML_DECL_PREFIX)
    );
    Some(Label::Zbr1bgjbPrn)
}
