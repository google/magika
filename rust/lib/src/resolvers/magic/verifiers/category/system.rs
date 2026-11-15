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

//! Operating-system metadata, registry, and shortcut format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// macOS Finder `.DS_Store`: alignment word `1` + `"Bud1\0"` Buddy allocator magic and root block
/// offset / size bounds.
pub(crate) fn dsstore(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(&1u32.to_be_bytes(), b"Bud1\0"); // @0..9: alignment word + Bud1 + high zero byte
    const ROOT_OFFSET: usize = 8; // @8: u32 BE — allocator book-keeping offset (≥ 32)
    const ROOT_SIZE: usize = 12; // @12: u32 BE — allocator book-keeping size (≥ 12)
    const MIN_LEN: usize = 36;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, HEADER),
        be32(prefix, ROOT_OFFSET)? >= 32,
        be32(prefix, ROOT_SIZE)? >= 12,
    );
    Some(Label::Dsstore)
}

/// Windows NT registry hive (`regf`): `"regf"` base block magic, major version 1, minor version
/// `0..=6`, primary file type `0`, and direct memory load format `1`.
pub(crate) fn hve(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"regf"; // @0..4: registry hive base block signature
    const MAJOR: usize = 20; // @20: u32 LE — major version (1)
    const MINOR: usize = 24; // @24: u32 LE — minor version (0..=6)
    const FILE_TYPE: usize = 28; // @28: u32 LE — file type (0 = primary)
    const FORMAT: usize = 32; // @32: u32 LE — format (1 = direct memory load)
    const MIN_LEN: usize = 48;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        le32(prefix, MAJOR)? == 1,
        le32(prefix, MINOR)? <= 6,
        le32(prefix, FILE_TYPE)? == 0,
        le32(prefix, FORMAT)? == 1,
    );
    Some(Label::Hve)
}

/// Windows Shell Link (`.lnk`): 76-byte `ShellLinkHeader` opening with size `0x4C` (76) and
/// link CLSID `00021401-0000-0000-C000-000000000046`.
pub(crate) fn lnk(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] =
        bytes!(&76u32.to_le_bytes(), guid!("00021401-0000-0000-C000-000000000046"),); // @0..20: HeaderSize (76) + LinkCLSID
    const MIN_LEN: usize = 76;

    require!(prefix.len() >= MIN_LEN, starts(prefix, HEADER));
    Some(Label::Lnk)
}

/// Windows Minidump (`MINIDUMP_HEADER`): `"MDMP"` + version `0xA793`, stream count `1..=255` at
/// `@8`, and stream directory RVA `32` immediately following the 32-byte header at `@12`.
pub(crate) fn minidump(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"MDMP", &0xA793u16.to_le_bytes()); // @0..6: MDMP signature + 0xA793 version
    const STREAM_COUNT: usize = 8; // @8: u32 LE — NumberOfStreams (1..=255)
    const STREAM_DIR_RVA: usize = 12; // @12: u32 LE — StreamDirectoryRva (32)
    const HEADER_LEN: usize = 32;

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, HEADER),
        (1..=255).contains(&le32(prefix, STREAM_COUNT)?),
        le32(prefix, STREAM_DIR_RVA)? == HEADER_LEN as u32,
    );
    Some(Label::Minidump)
}

/// International Color Consortium (ICC) color profile: 128-byte profile header + 4-byte tag count,
/// signature `"acsp"` at `@36`, and a known device/profile class at `@12`.
pub(crate) fn icc(prefix: &[u8]) -> Option<Label> {
    const DEVICE_CLASS: usize = 12; // @12..16: 4-byte profile/device class signature
    const MAGIC_OFFSET: usize = 36; // @36..40: "acsp" profile file signature
    const MAGIC: &[u8] = b"acsp";
    const DEVICE_CLASSES: &[&[u8; 4]] = &[
        b"\0\0\0\0",
        b"scnr",
        b"mntr",
        b"prtr",
        b"link",
        b"abst",
        b"spac",
        b"nmcl",
        b"cenc",
        b"mid ",
        b"mlnk",
        b"mvis",
    ];
    const MIN_LEN: usize = 132; // 128-byte header + 4-byte tag count

    require!(prefix.len() >= MIN_LEN, u8_at(prefix, 0)? == 0, eq_at(prefix, MAGIC_OFFSET, MAGIC),);
    let dev: &[u8; 4] = get_slice(prefix, DEVICE_CLASS, DEVICE_CLASS + 4)?.try_into().ok()?;
    require!(DEVICE_CLASSES.contains(&dev));
    Some(Label::Icc)
}

/// Windows Vista+ Explorer Thumbnail Cache (`thumbsdb`, 2006): `"CMMM"` at `@0`,
/// format version (`20|21|30|31|32`) LE at `@4`, and cache type `<= 16` LE at `@8`.
pub(crate) fn thumbcache(prefix: &[u8], size: u64) -> Option<Label> {
    const THUMBCACHE_MAGIC: &[u8; 4] = b"CMMM";

    require!(
        size >= 24,
        prefix.len() >= 24,
        starts(prefix, THUMBCACHE_MAGIC),
        (1..=64).contains(&le32(prefix, 4)?),
        le32(prefix, 8)? <= 32,
    );
    static THUMBCACHE: Subformat = Subformat::new("Windows thumbcache (CMMM)");
    with_subformat(Label::Thumbsdb, &THUMBCACHE)
}

/// Microsoft Windows Registry Script (`winregistry`, 1995): `"REGEDIT4"` or
/// `"Windows Registry Editor Version 5.00"` (ASCII/UTF-8 or UTF-16LE) with `"[HKEY_"` key path.
pub(crate) fn winregistry(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_WIN_REG_SIZE: u64 = 29; // "REGEDIT4\n\n[HKEY_CLASSES_ROOT]"
    const REGEDIT4_HDR: &[u8; 8] = b"REGEDIT4";
    const REGEDIT5_HDR: &[u8; 36] = b"Windows Registry Editor Version 5.00";
    const HKEY_OPEN: &[u8] = b"[HKEY_";
    const HKEY_DEL: &[u8] = b"[-HKEY_";
    const UTF16LE_REGEDIT5_HDR: &[u8] = b"\xFF\xFEW\0i\0n\0d\0o\0w\0s\0 \0R\0e\0g\0i\0s\0t\0r\0y\0 \0E\0d\0i\0t\0o\0r\0 \0V\0e\0r\0s\0i\0o\0n\0 \x005\0.\x000\x000\0";
    const UTF16LE_HKEY_OPEN: &[u8] = b"[\0H\0K\0E\0Y\0_\0";
    const UTF16LE_HKEY_DEL: &[u8] = b"[\0-\0H\0K\0E\0Y\0_\0";

    static REGEDIT4: Subformat = Subformat::new("REGEDIT4");
    static REGEDIT5: Subformat = Subformat::new("Registry Editor 5.00");
    static REGEDIT5_UTF16LE: Subformat = Subformat::new("Registry Editor 5.00 (UTF-16LE)");

    require!(size >= MIN_WIN_REG_SIZE, prefix.len() >= 29);
    if (starts(prefix, REGEDIT4_HDR) && matches!(u8_at(prefix, 8), Some(b'\r' | b'\n')))
        || (starts(prefix, REGEDIT5_HDR) && matches!(u8_at(prefix, 36), Some(b'\r' | b'\n')))
    {
        require!(
            contains(prefix, HKEY_OPEN) || contains(prefix, HKEY_DEL),
            is_utf8_text_header(prefix, size as usize),
        );
        let sub = if starts(prefix, REGEDIT4_HDR) { &REGEDIT4 } else { &REGEDIT5 };
        return with_subformat(Label::Winregistry, sub);
    }
    if starts(prefix, UTF16LE_REGEDIT5_HDR) {
        require!(contains(prefix, UTF16LE_HKEY_OPEN) || contains(prefix, UTF16LE_HKEY_DEL));
        return with_subformat(Label::Winregistry, &REGEDIT5_UTF16LE);
    }
    None
}

/// AppleDouble resource fork header: magic `0x00051607`, version 1 (`0x00010000`) or 2 (`0x00020000`),
/// and complete 26-byte fixed header.
pub(crate) fn appledouble(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = &0x0005_1607u32.to_be_bytes(); // @0..4: AppleDouble magic
    const VERSION: usize = 4; // @4..8: u32 BE — version 1 or 2
    const V1: u32 = 0x0001_0000; // version 1.0
    const V2: u32 = 0x0002_0000; // version 2.0
    const MIN_LEN: usize = 26; // fixed 26-byte header

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(be32(prefix, VERSION)?, V1 | V2),
    );
    static APPLEDOUBLE_V1: Subformat =
        Subformat::with_ids("AppleDouble version 1", "fmt/966", Some("fdd000625"));
    static APPLEDOUBLE_V2: Subformat =
        Subformat::with_ids("AppleDouble version 2", "fmt/503", Some("fdd000625"));
    let sub = if be32(prefix, VERSION)? == V1 { &APPLEDOUBLE_V1 } else { &APPLEDOUBLE_V2 };
    with_subformat(Label::Appledouble, sub)
}

/// AppleSingle archive: magic `0x00051600`, version 1 (`0x00010000`) or 2 (`0x00020000`),
/// and complete 26-byte fixed header.
pub(crate) fn applesingle(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = &0x0005_1600u32.to_be_bytes(); // @0..4: AppleSingle magic
    const VERSION: usize = 4; // @4..8: u32 BE — version 1 or 2
    const V1: u32 = 0x0001_0000; // version 1.0
    const V2: u32 = 0x0002_0000; // version 2.0
    const MIN_LEN: usize = 26; // fixed 26-byte header

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(be32(prefix, VERSION)?, V1 | V2),
    );
    static APPLESINGLE_V1: Subformat =
        Subformat::with_ids("AppleSingle version 1", "fmt/967", None);
    static APPLESINGLE_V2: Subformat =
        Subformat::with_ids("AppleSingle version 2", "fmt/968", None);
    let sub = if be32(prefix, VERSION)? == V1 { &APPLESINGLE_V1 } else { &APPLESINGLE_V2 };
    with_subformat(Label::Applesingle, sub)
}

/// Microsoft Windows Internet Shortcut (`internetshortcut`, 1995): `"[InternetShortcut]"`
/// or `"[{000214A0-0000-0000-C000-000000000046}]"` section header with a `"URL="` entry.
pub(crate) fn internetshortcut(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_WIN_URL_SIZE: u64 = 24; // "[InternetShortcut]\nURL=x"
    const URL_SECTION_CRLF: &[u8] = b"[InternetShortcut]\r\n";
    const URL_SECTION_LF: &[u8] = b"[InternetShortcut]\n";
    const URL_SECTION_BARE: &[u8] = b"[InternetShortcut]";
    const URL_PROPSTORE_GUID_SEC: &[u8] = b"[{000214A0-0000-0000-C000-000000000046}]";
    const URL_KEY_UPPER: &[u8] = b"\nURL=";
    const URL_KEY_LOWER: &[u8] = b"\nurl=";
    const URL_KEY_MIXED: &[u8] = b"\nUrl=";

    require!(size >= MIN_WIN_URL_SIZE, prefix.len() >= 24);
    let is_direct = starts(prefix, URL_SECTION_CRLF) || starts(prefix, URL_SECTION_LF);
    let is_prop_first =
        starts(prefix, URL_PROPSTORE_GUID_SEC) && contains(prefix, URL_SECTION_BARE);
    require!(
        is_direct || is_prop_first,
        contains(prefix, URL_KEY_UPPER)
            || contains(prefix, URL_KEY_LOWER)
            || contains(prefix, URL_KEY_MIXED),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Internetshortcut)
}

/// Microsoft Remote Desktop Protocol connection profile (`.rdp`, `rdp`, 2001): ASCII or UTF-16LE
/// key-value settings (`"screen mode id:i:"`, `"full address:s:"`, `"desktopwidth:i:"`).
pub(crate) fn rdp(prefix: &[u8], size: u64) -> Option<Label> {
    const RDP_SCREEN_MODE_ASCII: &[u8] = b"screen mode id:i:";
    const RDP_FULL_ADDRESS_ASCII: &[u8] = b"full address:s:";
    const RDP_DESKTOPWIDTH_ASCII: &[u8] = b"desktopwidth:i:";
    const RDP_SCREEN_UTF16LE: &[u8] = b"\xFF\xFEs\0c\0r\0e\0e\0n\0";
    const RDP_FULL_ADDRESS_UTF16LE: &[u8] = b"f\0u\0l\0l\0 \0a\0d\0d\0r\0e\0s\0s\0:\0s\0:";

    require!(
        size >= 24,
        prefix.len() >= 24,
        (starts(prefix, RDP_SCREEN_MODE_ASCII)
            || starts(prefix, RDP_FULL_ADDRESS_ASCII)
            || starts(prefix, RDP_SCREEN_UTF16LE))
            && (contains(prefix, RDP_FULL_ADDRESS_ASCII)
                || contains(prefix, RDP_DESKTOPWIDTH_ASCII)
                || contains(prefix, RDP_FULL_ADDRESS_UTF16LE)),
    );
    static RDP_ASCII: Subformat = Subformat::new("RDP connection file (ASCII)");
    static RDP_UTF16LE: Subformat = Subformat::new("RDP connection file (UTF-16LE)");
    with_subformat(
        Label::Rdp,
        if starts(prefix, RDP_SCREEN_UTF16LE) { &RDP_UTF16LE } else { &RDP_ASCII },
    )
}

/// Apple binary property list: `"bplist"` magic followed by classic `"0?"` (with 32-byte trailer),
/// modern `"10"`..`"19"`, or binary version words `00 00`, `00 01`, or `40 00`.
pub(crate) fn applebplist(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"bplist"; // @0..6: binary plist signature
    const MIN_LEN: usize = 8; // 6-byte magic + 2-byte version
    const CLASSIC_MIN_LEN: usize = 41; // 8-byte header + ≥1 object byte + 32-byte trailer
    const VER_MAJOR: usize = 6; // @6: u8 — major version byte
    const VER_MINOR: usize = 7; // @7: u8 — minor version byte

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    let (v0, v1) = (u8_at(prefix, VER_MAJOR)?, u8_at(prefix, VER_MINOR)?);
    let valid = (v0 == b'0'
        && prefix.len() >= CLASSIC_MIN_LEN
        && has_non_ascii_text_byte(&prefix[8..prefix.len().min(CLASSIC_MIN_LEN)]))
        || (v0 == b'1'
            && v1.is_ascii_digit()
            && has_non_ascii_text_byte(&prefix[8..prefix.len().min(CLASSIC_MIN_LEN)]))
        || (v0 == 0x00 && matches!(v1, 0x00 | 0x01))
        || (v0 == 0x40 && v1 == 0x00);
    require!(valid);
    Some(Label::Applebplist)
}

/// Doom WAD archive: `"IWAD"` or `"PWAD"` magic plus lump count (`0..=1_000_000`) at `@4`
/// and directory offset (`12..=0x0800_0000`) at `@8` (≥ 12 bytes).
pub(crate) fn wad(prefix: &[u8]) -> Option<Label> {
    const IWAD: &[u8] = b"IWAD"; // @0..4: internal WAD signature
    const PWAD: &[u8] = b"PWAD"; // @0..4: patch WAD signature
    const HEADER_LEN: usize = 12; // 4-byte magic + numlumps (4) + infotableofs (4)

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, IWAD) || starts(prefix, PWAD),
        le32(prefix, 4)? <= 1_000_000,
        (HEADER_LEN as u32..=0x0800_0000).contains(&le32(prefix, 8)?),
    );
    static DOOM_IWAD: Subformat = Subformat::new("Doom IWAD");
    static DOOM_PWAD: Subformat = Subformat::new("Doom PWAD");
    with_subformat(Label::Wad, if starts(prefix, IWAD) { &DOOM_IWAD } else { &DOOM_PWAD })
}

/// Android Compiled Resource Table (`arsc`, 2008): `RES_TABLE_TYPE` (`0x0002`) with 12-byte
/// header (`0x000C`), exact total table size at `@4..8`, `1..=256` resource packages at `@8..12`,
/// and a valid first child chunk (`RES_STRING_POOL_TYPE` `0x0001` or `RES_TABLE_PACKAGE_TYPE`
/// `0x0200`) at `@12..20`.
pub(crate) fn arsc(prefix: &[u8], size: u64) -> Option<Label> {
    const HEADER: &[u8] = hex2bin!("02 00 0C 00"); // @0..4: RES_TABLE_TYPE (0x0002) + headerSize (12)
    const TABLE_SIZE: usize = 4; // @4: u32 LE — total ResTable_header chunk size
    const PKG_COUNT: usize = 8; // @8: u32 LE — packageCount (1..=256)
    const NEXT_TYPE: usize = 12; // @12: u16 LE — first child chunk type
    const NEXT_HDR_SIZE: usize = 14; // @14: u16 LE — first child headerSize (>= 16)
    const NEXT_SIZE: usize = 16; // @16: u32 LE — first child chunk size
    const MIN_SIZE: u64 = 24;
    const MIN_PREFIX: usize = 20;

    require!(size >= MIN_SIZE, prefix.len() >= MIN_PREFIX, starts(prefix, HEADER));
    let table_size = u64::from(le32(prefix, TABLE_SIZE)?);
    let pkg_count = le32(prefix, PKG_COUNT)?;
    let next_type = le16(prefix, NEXT_TYPE)?;
    let next_hdr = u64::from(le16(prefix, NEXT_HDR_SIZE)?);
    let next_size = u64::from(le32(prefix, NEXT_SIZE)?);
    require!(
        table_size == size,
        (1..=256).contains(&pkg_count),
        matches!(next_type, 0x0001 | 0x0200),
        next_hdr >= 16,
        next_size >= next_hdr,
        12 + next_size <= size,
    );
    Some(Label::Arsc)
}

/// IPS/IPS32 end-of-patch trailer matcher (`"EOF"` / `"EEOF"`, optional 3- or 4-byte truncation
/// size, or an EBP JSON metadata block `"EOF{...}"`).
fn has_valid_ips_tail(tail: &[u8], is_ips32: bool) -> bool {
    let eof_tag: &[u8] = if is_ips32 { b"EEOF" } else { b"EOF" };
    let eof_len = eof_tag.len();
    let trunc_len = if is_ips32 { 4 } else { 3 };
    if tail.len() >= eof_len && eq_at(tail, tail.len() - eof_len, eof_tag) {
        return true;
    }
    if tail.len() >= eof_len + trunc_len && eq_at(tail, tail.len() - eof_len - trunc_len, eof_tag) {
        return true;
    }
    if !is_ips32 && tail.last() == Some(&b'}') {
        let win_start = tail.len().saturating_sub(2048);
        if let Some(rel) = tail[win_start..].windows(4).rposition(|w| w == b"EOF{") {
            return eq_at(tail, win_start + rel, b"EOF{")
                && u8_at(tail, tail.len() - 1) == Some(b'}');
        }
    }
    false
}

/// International Patching System binary patch (`ips_patch`, 1993): `"PATCH"` (24-bit offsets,
/// `"EOF"` trailer) or `"IPS32"` (32-bit offsets, `"EEOF"` trailer), validated by walking up
/// to 64 hunk headers in the prefix and confirming the trailer in-prefix or via `ctx.tail`.
pub(crate) fn ips_patch(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    const IPS_MAGIC: &[u8; 5] = b"PATCH";
    const IPS32_MAGIC: &[u8; 5] = b"IPS32";
    const MAX_IPS_PREFIX_HUNKS: usize = 64;

    let is_ips32 = if starts(prefix, IPS_MAGIC) {
        false
    } else if starts(prefix, IPS32_MAGIC) {
        true
    } else {
        return None;
    };
    let off_len: usize = if is_ips32 { 4 } else { 3 };
    let eof_tag: &[u8] = if is_ips32 { b"EEOF" } else { b"EOF" };
    let min_len = 5 + off_len + 2 + 1 + off_len; // magic + >=1 hunk + EOF sentinel
    require!(ctx.size >= min_len as u64, prefix.len() >= 11);

    let mut pos = 5usize;
    let mut hunks = 0usize;
    let mut seen_non_ascii_hdr = false;
    let mut reached_eof = false;

    while hunks < MAX_IPS_PREFIX_HUNKS {
        if pos + off_len > prefix.len() {
            break;
        }
        if eq_at(prefix, pos, eof_tag) {
            let after_eof = pos + off_len;
            let rem = ctx.size.saturating_sub(after_eof as u64);
            if rem == 0
                || rem == off_len as u64
                || (!is_ips32 && after_eof < prefix.len() && u8_at(prefix, after_eof) == Some(b'{'))
            {
                reached_eof = true;
                break;
            }
            return None;
        }
        let hdr_len = off_len + 2;
        if pos + hdr_len > prefix.len() {
            break;
        }
        let hdr = get_slice(prefix, pos, pos + hdr_len)?;
        if !seen_non_ascii_hdr && has_non_ascii_text_byte(hdr) {
            seen_non_ascii_hdr = true;
        }
        let hunk_size = usize::from(be16(prefix, pos + off_len)?);
        pos += hdr_len;
        if hunk_size == 0 {
            // RLE hunk: 2-byte BE repeat count (> 0) + 1-byte fill value
            if pos + 3 > prefix.len() {
                break;
            }
            let rle_count = be16(prefix, pos)?;
            require!(rle_count >= 1);
            if !seen_non_ascii_hdr && has_non_ascii_text_byte(&prefix[pos..pos + 2]) {
                seen_non_ascii_hdr = true;
            }
            pos += 3;
        } else {
            let next = pos.checked_add(hunk_size)?;
            if (next as u64) > ctx.size {
                return None;
            }
            pos = next;
        }
        hunks += 1;
    }

    require!(hunks >= 1, seen_non_ascii_hdr);
    if !reached_eof {
        require!(hunks == MAX_IPS_PREFIX_HUNKS || ctx.size > prefix.len() as u64);
        if let Some(t) = ctx.tail {
            require!(has_valid_ips_tail(t, is_ips32));
        } else if ctx.size == prefix.len() as u64 {
            require!(has_valid_ips_tail(prefix, is_ips32));
        } else {
            return None;
        }
    }
    Some(Label::IpsPatch)
}

/// Microsoft Windows Prefetch (`prefetch`, 2001): Windows 10/11 MAM-compressed header
/// (`"MAM\x04"` or `"MAM\x84"` with bounded uncompressed size `84..=64 MiB` at `@4..8`),
/// or uncompressed XP–Windows 8.1 header (version `17 | 23 | 26 | 30` LE at `@0..4`,
/// `"SCCA"` signature at `@4..8`, and file size `>= 84` at `@12..16`).
pub(crate) fn prefetch(prefix: &[u8], size: u64) -> Option<Label> {
    const MAM_MAGIC: &[u8; 3] = b"MAM"; // @0..3: Windows 10+ XPRESS Huffman container
    const SCCA_MAGIC: &[u8; 4] = b"SCCA"; // @4..8: uncompressed Prefetch signature
    const MIN_LEN: usize = 16;

    require!(size >= MIN_LEN as u64, prefix.len() >= MIN_LEN);
    if starts(prefix, MAM_MAGIC) {
        require!(
            matches!(u8_at(prefix, 3)?, 0x04 | 0x84),
            (84..=0x0400_0000).contains(&le32(prefix, 4)?),
        );
        return Some(Label::Prefetch);
    }
    require!(
        matches!(le32(prefix, 0)?, 17 | 23 | 26 | 30),
        eq_at(prefix, 4, SCCA_MAGIC),
        le32(prefix, 12)? >= 84,
    );
    Some(Label::Prefetch)
}

/// Microsoft Windows Jump List Custom Destinations (`custom_destinations`, 2009): header
/// version `2` at `@0..4`, `1..=64` categories at `@4..8`, reserved `0` at `@8..12`,
/// category type `<= 2` at `@12..16`, and an embedded Shell Link (`0x4C` + CLSID) or
/// category footer sentinel (`AB FB BF BA`) within `@16..512`.
pub(crate) fn custom_destinations(prefix: &[u8], size: u64) -> Option<Label> {
    const HEADER_VER: usize = 0; // @0: u32 LE — format version (2)
    const CAT_COUNT: usize = 4; // @4: u32 LE — category count (1..=64)
    const RESERVED: usize = 8; // @8: u32 LE — reserved (0)
    const CAT_TYPE: usize = 12; // @12: u32 LE — first category type (0..=2)
    const LNK_SIG: &[u8] = hex2bin!("4C 00 00 00 01 14 02 00");
    const FOOTER_SIG: &[u8] = hex2bin!("AB FB BF BA");
    const MIN_SIZE: u64 = 64;
    const MIN_PREFIX: usize = 36;

    require!(
        size >= MIN_SIZE,
        prefix.len() >= MIN_PREFIX,
        le32(prefix, HEADER_VER)? == 2,
        (1..=64).contains(&le32(prefix, CAT_COUNT)?),
        le32(prefix, RESERVED)? == 0,
        le32(prefix, CAT_TYPE)? <= 2,
        scan_range(prefix, 16, 512, LNK_SIG) || scan_range(prefix, 16, 512, FOOTER_SIG),
    );
    Some(Label::CustomDestinations)
}

/// freedesktop.org Shared MIME-info binary magic database (`freedesktop_mime_magic`, 2003):
/// `"MIME-Magic\x00\n["` header opening the first priority section.
pub(crate) fn freedesktop_mime_magic(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 13] = b"MIME-Magic\x00\n[";
    const MIN_SIZE: u64 = 16;

    require!(size >= MIN_SIZE, prefix.len() >= MAGIC.len(), starts(prefix, MAGIC));
    Some(Label::FreedesktopMimeMagic)
}
