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

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const CDF_MS_SIZE_OFFSET: usize = 20;
const CIV3_BICQ_MAGIC: &[u8; 4] = b"BICQ";
const CIV3_BICX_MAGIC: &[u8; 4] = b"BICX";
const CIV3_BIC_MAGIC: &[u8; 4] = b"BIC ";
const EMTEX_DOT_MAGIC: &[u8; 4] = b"DOT\x00";
const EMTEX_TCP_MAGIC: &[u8; 4] = b"tcp\x02";
const GETTEXT_MO_MAGIC_LE: &[u8; 4] = le32_bytes!(0x9504_12DE);
const HIKVISION_XOR_KEY: [u8; 16] = [
    0xBA, 0xCD, 0xBC, 0xFE, 0xD6, 0xCA, 0xDD, 0xD3, 0xBA, 0xB9, 0xA3, 0xAB, 0xBF, 0xCB, 0xB5, 0xBE,
];
const IFD_SIGNATURE_LE: &[u8; 4] = le32_bytes!(0x0FF0_A55A);
const LEICA_DMR_XOR35_BANNER: &[u8; 20] =
    hex2bin!("79 70 7C 76 74 15 73 7C 67 78 62 74 67 70 15 7C 78 74 72 70");
const LEICA_FW_BE48_MAGIC: &[u8; 4] = hex2bin!("1E 1C AF 2E");
const LEICA_FW_LE56_MAGIC: &[u8; 4] = hex2bin!("3E AF 1C 1E");
const LEICA_FW_S2_MAGIC: &[u8; 4] = hex2bin!("1E 1C AF 1E");
const MIN_ADOBE_RNA_SIZE: u64 = 48;
const MIN_ALLWINNER_EGON_SIZE: u64 = 512;
const MIN_ALLWINNER_IMAGEWTY_SIZE: u64 = 1024;
const MIN_ALPHA_WRGG_SIZE: u64 = 116;
const MIN_ALSA_SOC_TOPOLOGY_SIZE: u64 = 36;
const MIN_AMD_MICROCODE_SIZE: u64 = 52;
const MIN_AMD_PSP_FIRMWARE_SIZE: u64 = 256;
const MIN_AMLOGIC_MPT_SIZE: u64 = 64;
const MIN_AMLOGIC_UPGRADE_SIZE: u64 = 128;
const MIN_ANDROID_BOOTLOADER_IMG_SIZE: u64 = 260;
const MIN_ANDROID_DTBO_SIZE: u64 = 64;
const MIN_ANDROID_HYB_SIZE: u64 = 32;
const MIN_ANDROID_VBMETA_SIZE: u64 = 256;
const MIN_ANDROID_WINSCOPE_TRACE_SIZE: u64 = 16;
const MIN_AOR_DV10_FW_SIZE: u64 = 64;
const MIN_APPLE_8900_SIZE: u64 = 0x800;
const MIN_APPLE_ASL_SIZE: u64 = 80;
const MIN_APPLE_BOM_SIZE: u64 = 512;
const MIN_APPLE_VMP4_SIZE: u64 = 20;
const MIN_APS_PATCH_SIZE: u64 = 78;
const MIN_ARCADYAN_FW_SIZE: u64 = 128;
const MIN_ARCSOFT_UI_AUI_SIZE: u64 = 32;
const MIN_ARM_MALI_CSFFW_SIZE: u64 = 64;
const MIN_AUL_DSC_SIZE: u64 = 56;
const MIN_AUL_TIMESYNC_SIZE: u64 = 48;
const MIN_AUL_UUIDTEXT_SIZE: u64 = 20;
const MIN_AUTEL_ECC_SIZE: u64 = 32;
const MIN_BEYONDTRUST_SCC_UI_SIZE: u64 = 44;
const MIN_BLUESTRIPE_HB_SIZE: u64 = 28;
const MIN_BROADCOM_CFE_SIZE: u64 = 256;
const MIN_BROADCOM_HCD_SIZE: u64 = 32;
const MIN_BROADCOM_STB_BOOTLOADER_SIZE: u64 = 64;
const MIN_BROADCOM_TRX_SIZE: u64 = 32;
const MIN_BR_AS3TRACE_SIZE: u64 = 36;
const MIN_BURNPIPE_BA_SIZE: u64 = 64;
const MIN_CASLOG_DLOG_SIZE: u64 = 36;
const MIN_CHROMEOS_FMAP_SIZE: u64 = 98;
const MIN_CHROMEOS_VB21_SIZE: u64 = 36;
const MIN_CHROMEOS_VBOOT_SIZE: u64 = 72;
const MIN_CHROME_SNSS_SIZE: u64 = 14;
const MIN_CIV3_BIC_SIZE: u64 = 32;
const MIN_COMBI_BCF_SIZE: u64 = 32;
const MIN_COREBOOT_CBFS_SIZE: u64 = 32;
const MIN_CRASHPAD_SETTINGS_SIZE: u64 = 28;
const MIN_CREATIVE_XFI_FW_SIZE: u64 = 24;
const MIN_CSMAN_DAT_SIZE: u64 = 20;
const MIN_CSR_DFU2_SIZE: u64 = 64;
const MIN_CUPS_IPP_SIZE: u64 = 32;
const MIN_CYPRESS_CLM_BLOB_SIZE: u64 = 68;
const MIN_DART_DSK_SIZE: u64 = 24;
const MIN_DISKOVERY_SCAN_SIZE: u64 = 36;
const MIN_DLINK_AIH0_SIZE: u64 = 64;
const MIN_DLINK_DEAFBEAD_SIZE: u64 = 16;
const MIN_DLINK_FPKG_SIZE: u64 = 64;
const MIN_DLINK_MH01_SIZE: u64 = 64;
const MIN_DLINK_SHRS_SIZE: u64 = 1756;
const MIN_DLINK_TLV_SIZE: u64 = 128;
const MIN_DMTF_PLDM_FIRMWARE_SIZE: u64 = 36;
const MIN_DOS_HAYES_MDM_SIZE: u64 = 512;
const MIN_DTB_SIZE: u64 = 40;
const MIN_DURAGER_UV_REAL_SIZE: u64 = 40;
const MIN_EMTEX_DOT_SIZE: u64 = 157;
const MIN_EMTEX_TCP_SIZE: u64 = 776;
const MIN_EO_WEBENGINE_SIZE: u64 = 48;
const MIN_ESP_PARTTABLE_SIZE: u64 = 64;
const MIN_EUREKALOG_EL_SIZE: u64 = 36;
const MIN_FBA_FS_SIZE: u64 = 48;
const MIN_FIELDLOG_FL_SIZE: u64 = 28;
const MIN_FIO_VERIFY_STATE_SIZE: u64 = 64;
const MIN_FSECURE_FS02_MF_SIZE: u64 = 36;
const MIN_GECKO_STARTUPCACHE_SIZE: u64 = 64;
const MIN_GETTEXT_MO_SIZE: u64 = 28;
const MIN_GIT_INDEX_SIZE: u64 = 32;
const MIN_GIT_PACK_IDX_SIZE: u64 = 1072;
const MIN_GLIBC_LD_SO_CACHE_SIZE: u64 = 64;
const MIN_GNUNET_GND_SIZE: u64 = 16;
const MIN_GNU_LOCATE_DB_SIZE: u64 = 16;
const MIN_GOBJECT_TYPELIB_SIZE: u64 = 112;
const MIN_GO_TELEMETRY_COUNT_SIZE: u64 = 48;
const MIN_GPO_REGISTRY_POL_SIZE: u64 = 8;
const MIN_HELIOS_SAVE_SIZE: u64 = 36;
const MIN_HIEW_EXT_INFO_SIZE: u64 = 24;
const MIN_HIKVISION_FIRMWARE_SIZE: u64 = 108;
const MIN_HP_BDL_SIZE: u64 = 64;
const MIN_HP_IPKG_SIZE: u64 = 64;
const MIN_HUAWEI_UPDATE_APP_SIZE: u64 = 128;
const MIN_HXNF_TROUBLESHOOT_SIZE: u64 = 32;
const MIN_HYTERA_FUDX_SIZE: u64 = 40;
const MIN_IBM_OS2_EXTENDED_ATTR_EA_SIZE: u64 = 64;
const MIN_IBM_STAP_CTL_SIZE: u64 = 48;
const MIN_IMX_SDMA_FW_SIZE: u64 = 216;
const MIN_INFOCRYPT_ICSL_SIZE: u64 = 112;
const MIN_INSTAR_BNEG_SIZE: u64 = 24;
const MIN_INTEL_CPD_FIRMWARE_SIZE: u64 = 40;
const MIN_INTEL_ESIF_DV_SIZE: u64 = 48;
const MIN_INTEL_FLASH_DESCRIPTOR_SIZE: u64 = 64;
const MIN_INTEL_ISCU_FIRMWARE_SIZE: u64 = 32;
const MIN_INTEL_IVSC_FIRMWARE_SIZE: u64 = 64;
const MIN_INTEL_ME_FPT_SIZE: u64 = 64;
const MIN_IRLOG1_LOG_SIZE: u64 = 36;
const MIN_ISWLOG_SWL_SIZE: u64 = 28;
const MIN_IWLWIFI_UCODE_SIZE: u64 = 28;
const MIN_JAVA_WEBSTART_IDX_SIZE: u64 = 32;
const MIN_JBOOT_SCH2_SIZE: u64 = 40;
const MIN_JODA_TZ_SIZE: u64 = 16;
const MIN_JUCE_PROP_SIZE: u64 = 24;
const MIN_KEYENCE_LJX_LJHP_SIZE: u64 = 64;
const MIN_KODAK_DC20_PRESET_PST_SIZE: u64 = 32;
const MIN_KONTEXT_LAT_SIZE: u64 = 512;
const MIN_LATTICE_BITSTREAM_SIZE: u64 = 64;
const MIN_LEICA_APSC_FW_SIZE: u64 = 96;
const MIN_LEICA_CAMERA_FW_SIZE: u64 = 48;
const MIN_LEICA_DMR_SIZE: u64 = 528;
const MIN_LEICA_LENS_DB_SIZE: u64 = 40;
const MIN_LENOVO_LOGDAT_SIZE: u64 = 64;
const MIN_LINUX_KMOD_INDEX_SIZE: u64 = 16;
const MIN_LINUX_PERF_DATA_SIZE: u64 = 104;
const MIN_LSSTAT_STAT_SIZE: u64 = 24;
const MIN_MACOS_FSEVENTS_SIZE: u64 = 24;
const MIN_MACOS_UTMPX_SIZE: u64 = 628;
const MIN_MACRO_EXPRESS_MEX_SIZE: u64 = 48;
const MIN_MARVELL_CPT_UCODE_SIZE: u64 = 64;
const MIN_MATTER_OTA_SIZE: u64 = 32;
const MIN_MCUBOOT_IMAGE_SIZE: u64 = 32;
const MIN_MEDIATEK_BRLYT_SIZE: u64 = 32;
const MIN_MEDIATEK_BT_FW_SIZE: u64 = 64;
const MIN_MEDIATEK_IMAGE_SIZE: u64 = 64;
const MIN_MEDIATEK_PRELOADER_SIZE: u64 = 64;
const MIN_MEDIATEK_SIGNED_IMAGE_SIZE: u64 = 128;
const MIN_MELEE_SLIPPI_SIZE: u64 = 32;
const MIN_MERCEDES_CFF_SIZE: u64 = 64;
const MIN_MICROHOUSE_PROSEC_BIN_SIZE: u64 = 32;
const MIN_MIKROTIK_NPK_SIZE: u64 = 36;
const MIN_MOTOROLA_MOTOLOGO_SIZE: u64 = 45;
const MIN_MOTOROLA_PIV_IMAGE_SIZE: u64 = 288;
const MIN_MOXA_FRM_SIZE: u64 = 96;
const MIN_MSDOS_COUNTRY_SYS_SIZE: u64 = 32;
const MIN_MSDOS_KEYBOARD_SYS_SIZE: u64 = 32;
const MIN_MSIE_INDEX_DAT_SIZE: u64 = 128;
const MIN_MSSSCARD_AX_LOG_SIZE: u64 = 64;
const MIN_MS_SETUP_TDF_SIZE: u64 = 32;
const MIN_MS_WINDOWS_POSTSCRIPT_WPD_SIZE: u64 = 64;
const MIN_NAF_BIO_SIZE: u64 = 10;
const MIN_NCSTUDIO_DYN_SIZE: u64 = 64;
const MIN_NEOGEO_ABKCPUR_SRM_SIZE: u64 = 64;
const MIN_NETGEAR_CHK_SIZE: u64 = 58;
const MIN_NETRON_NFW_SIZE: u64 = 36;
const MIN_NEWWORD_OVR_SIZE: u64 = 32;
const MIN_NEXTSTEP_WMD_SIZE: u64 = 64;
const MIN_NEXUS_IMGDATA_SIZE: u64 = 64;
const MIN_NFDUMP_FLOW_SIZE: u64 = 140;
const MIN_NINTENDO_WII_BNR_SIZE: u64 = 1_568;
const MIN_NORTON_COMMANDER_MSG_SIZE: u64 = 32;
const MIN_OFFICE_OWNER_LOCK_SIZE: u64 = 72;
const MIN_ONEDRIVE_ODL_SIZE: u64 = 256;
const MIN_OPENSSH_KRL_SIZE: u64 = 32;
const MIN_OSGI_FRAMEWORK_INFO4_SIZE: u64 = 64;
const MIN_PANASONIC_UPD_SIZE: u64 = 512;
const MIN_PASSMARK_PT_SIZE: u64 = 28;
const MIN_PDFANNOTATOR_SIL_SIZE: u64 = 36;
const MIN_PKCS15_KY_SIZE: u64 = 28;
const MIN_PLAYSTATION_SFO_SIZE: u64 = 20;
const MIN_POD5_SIZE: u64 = 64;
const MIN_PPM_MPD_SIZE: u64 = 64;
const MIN_PRINCED_POP_PLV_SIZE: u64 = 64;
const MIN_PROCMON_PMC_SIZE: u64 = 36;
const MIN_PROJECT64_PJ_SIZE: u64 = 1024;
const MIN_PS1_MEMORY_CARD_SIZE: u64 = PS1_MEMORY_CARD_SIZE;
const MIN_PS2_MEMORY_CARD_SIZE: u64 = 8_388_608;
const MIN_PSG_SLEEP_SIZE: u64 = 36;
const MIN_QCA_ATH_FIRMWARE_SIZE: u64 = 24;
const MIN_QCA_BT_TLV_SIZE: u64 = 16;
const MIN_QNX_IFS_SIZE: u64 = 256;
const MIN_QUALCOMM_ACDB_SIZE: u64 = 48;
const MIN_QUALCOMM_FBPK_SIZE: u64 = 88;
const MIN_QUALCOMM_META_IMAGE_SIZE: u64 = 76;
const MIN_QUALCOMM_MIBIB_SIZE: u64 = 64;
const MIN_QUALCOMM_SPLASH_SIZE: u64 = 512;
const MIN_QUARTUS_POF_SIZE: u64 = 64;
const MIN_REALTEK_CSYS_SIZE: u64 = 20;
const MIN_REALTEK_RTK0_SIZE: u64 = 32;
const MIN_REALTEK_WIFI_FW_SIZE: u64 = 32;
const MIN_RHIDE_GDT_SIZE: u64 = 32;
const MIN_ROCKCHIP_RKAF_SIZE: u64 = 136;
const MIN_ROCKCHIP_RKFW_SIZE: u64 = 102;
const MIN_ROCKCHIP_RSCE_SIZE: u64 = 544;
const MIN_ROUTER_WLK_CONFIG_SIZE: u64 = 64;
const MIN_RUP_PATCH_SIZE: u64 = 2048;
const MIN_SAMSUNG_PIT_SIZE: u64 = 160;
const MIN_SAWGRASS_DM_SIZE: u64 = 48;
const MIN_SDW2_SNP_SIZE: u64 = 28;
const MIN_SEAMA_FW_SIZE: u64 = 44;
const MIN_SECURITY_CFG_CCH_SIZE: u64 = 64;
const MIN_SELINUX_POLICY_SIZE: u64 = 36;
const MIN_SELINUX_PP_SIZE: u64 = 28;
const MIN_SERCOMM_FW_SIZE: u64 = 64;
const MIN_SFF_SIZE: u64 = 40;
const MIN_SH3_SMART_SIZE: u64 = 36;
const MIN_SILABS_GBL_SIZE: u64 = 24;
const MIN_SOF_FIRMWARE_SIZE: u64 = 48;
const MIN_SOPHOS_IDE_SIZE: u64 = 44;
const MIN_SPREADTRUM_DHTB_SIZE: u64 = 512;
const MIN_SPREADTRUM_PAC_SIZE: u64 = 2124;
const MIN_ST_DFUSE_SIZE: u64 = 309;
const MIN_SUNOS_TOC_SIZE: u64 = 32;
const MIN_SUNXI_PACKAGE_SIZE: u64 = 44;
const MIN_SYSTEMD_JOURNAL_SIZE: u64 = 240;
const MIN_TEKTRONIX_WFM_SIZE: u64 = 838;
const MIN_TELIX_SALT_SLC_SIZE: u64 = 20;
const MIN_THEBAT_SRB_SIZE: u64 = 32;
const MIN_TI_AIS_SIZE: u64 = 24;
const MIN_TI_BTS_SCRIPT_SIZE: u64 = 36;
const MIN_TI_MULTICORE_SIZE: u64 = 52;
const MIN_TI_OAD_SIZE: u64 = 44;
const MIN_TI_RPRC_SIZE: u64 = 40;
const MIN_TI_SMARTAMP_FW_SIZE: u64 = 64;
const MIN_TI_TPS_UPG_SIZE: u64 = 48;
const MIN_TOPSOLID_RCF_SIZE: u64 = 32;
const MIN_TOYOTA_CUW_SIZE: u64 = 64;
const MIN_TPLINK_FW_SIZE: u64 = 512;
const MIN_TRACKMANIA_GBX_SIZE: u64 = 32;
const MIN_TRESORIT_PROFILE_BACKUP_SIZE: u64 = 36;
const MIN_TRPH_XPD_SIZE: u64 = 32;
const MIN_TTC_XCFG_SIZE: u64 = 28;
const MIN_TUNEUP_TUN_SIZE: u64 = 40;
const MIN_TVRW_PROJECT_SIZE: u64 = 32;
const MIN_TZIF_SIZE: u64 = 44;
const MIN_UBICOM_FW_SIZE: u64 = 64;
const MIN_UBIQUITI_FW_SIZE: u64 = 264;
const MIN_UEFI_CAPSULE_SIZE: u64 = 64;
const MIN_UNITY_PREVIEW_INFO_SIZE: u64 = 36;
const MIN_UNREAL_BUILDPATCH_SIZE: u64 = 41;
const MIN_UNREAL_GVAS_SIZE: u64 = 64;
const MIN_UNREAL_LOCMETA_SIZE: u64 = 27;
const MIN_UNREAL_LOCRES_SIZE: u64 = 25;
const MIN_UNREAL_PIPELINE_CACHE_SIZE: u64 = 44;
const MIN_UNREAL_UTRACE_SIZE: u64 = 32;
const MIN_V129_RDT_SIZE: u64 = 36;
const MIN_VIM_SWAP_SIZE: u64 = 1024;
const MIN_VTECH_VRP_LOG_SIZE: u64 = 512;
const MIN_VW_SGO_SIZE: u64 = 0x1E0;
const MIN_WARCRAFT3_REPLAY_SIZE: u64 = 68;
const MIN_WII_GECKO_GCT_SIZE: u64 = 16;
const MIN_WIN31_REG_DAT_SIZE: u64 = 64;
const MIN_WINCE_B000FF_SIZE: u64 = 32;
const MIN_WINDOWS_CDF_MS_SIZE: u64 = 64;
const MIN_WINDOWS_CODEPAGE_NLS_SIZE: u64 = 64;
const MIN_WINDOWS_EVTX_SIZE: u64 = 4096;
const MIN_WINDOWS_EVT_SIZE: u64 = 48;
const MIN_WINDOWS_GRP_SIZE: u64 = 36;
const MIN_WINDOWS_IME_MB_SIZE: u64 = 256;
const MIN_WINDOWS_INSTALLER_SCRIPT_SIZE: u64 = 64;
const MIN_WINDOWS_KERNEL_DUMP_SIZE: u64 = 4096;
const MIN_WINDOWS_PNF_SIZE: u64 = 48;
const MIN_WINDOWS_PRI_SIZE: u64 = 54;
const MIN_WINDOWS_SDI_SIZE: u64 = 512;
const MIN_WIN_RESTORE_RP_LOG_SIZE: u64 = 536;
const MIN_WIN_SUPERFETCH_SIZE: u64 = 32;
const MIN_WIN_TASK_JOB_SIZE: u64 = 72;
const MIN_WIRELESS_REGDB_SIZE: u64 = 32;
const MIN_WIRESYNC_MOMV2_OMO_SIZE: u64 = 512;
const MIN_WITCHER3_SAV_SIZE: u64 = 64;
const MIN_WORLDCRAFT_CMDSEQ_SIZE: u64 = 171;
const MIN_X64DBG_TRACE_SIZE: u64 = 30;
const MIN_XIAOMI_HDR_SIZE: u64 = 48;
const MIN_XILINX_BITSTREAM_SIZE: u64 = 64;
const MIN_XILINX_XCLBIN_SIZE: u64 = 304;
const MIN_ZEBRA_TPD_SIZE: u64 = 32;
const MIN_ZEROPLUS_ALS_SIZE: u64 = 40;
const MIN_ZIGBEE_OTA_SIZE: u64 = 62;
const MIN_ZYXEL_BIX_SIZE: u64 = 64;
const NEXTSTEP_WMD_KEY_LEN_OFFSET: usize = 4;
const NEXTSTEP_WMD_KEY_OFFSET: usize = 8;
const NEXTSTEP_WMD_MAGIC: &[u8; 4] = hex2bin!("69 02 FC AE");
const ODL_FLAGS_OFFSET: usize = 24;
const ODL_RESERVED_OFFSET: usize = 16;
const ODL_VERSION_OFFSET: usize = 8;
const ONEDRIVE_ODL_MAGIC: &[u8; 8] = b"EBFGONED";
const PANASONIC_UPD_XORFF_MAGIC: &[u8; 4] = hex2bin!("AA AF BB FF");
const POD5_SIGNATURE: &[u8; 8] = b"\x8BPOD\r\n\x1A\n";
const PROJECT64_PJ_MAGIC: &[u8; 4] = le32_bytes!(0x23D8_A6C8);
const PS1_MEMORY_CARD_SIZE: u64 = 131_072;
const SUNOS_TOC_MAGIC: &[u8; 4] = be32_bytes!(0x674D_2309);
const TELIX_SLC_MAGIC: &[u8; 6] = hex2bin!("0B 1D 2B 1A 00 00");
const TOPSOLID_CLASS_LEN_OFFSET: usize = 8;
const TOPSOLID_CLASS_NAME_OFFSET: usize = 10;
const TOPSOLID_CLASS_PREFIX: &[u8; 2] = b"TS";
const TOPSOLID_MFC_NEW_CLASS_TAG: &[u8; 4] = bytes!(le16_bytes!(0xFFFF), le16_bytes!(0));
const TOPSOLID_MFC_TAG_OFFSET: usize = 4;
const TVRW_PROJECT_MAGIC: &[u8; 4] = b"TVRW";
const WINDOWS_CDF_MS_MAGIC: &[u8; 12] = bytes!(b"PcmH", le32_bytes!(1), le32_bytes!(0));
const WITCHER3_SAV_MAGIC: &[u8; 4] = b"SNFH";
const WORLDCRAFT_CMDSEQ_MAGIC: &[u8; 31] = b"Worldcraft Command Sequences\r\n\x1a";
const X64DBG_TRACE_MAGIC: &[u8; 4] = b"TRAC";
const XILINX_BIT_PREAMBLE: &[u8; 13] = hex2bin!("00 09 0F F0 0F F0 0F F0 0F F0 00 00 01");

#[inline]
const fn hikvision_dec_byte(b: u8, off: usize) -> u8 {
    b ^ HIKVISION_XOR_KEY[(off.wrapping_add(off >> 4)) & 0x0F]
}

#[inline]
fn hikvision_dec_le32(prefix: &[u8], off: usize) -> Option<u32> {
    let b = get_slice(prefix, off, off.checked_add(4)?)?;
    Some(u32::from_le_bytes([
        hikvision_dec_byte(b[0], off),
        hikvision_dec_byte(b[1], off + 1),
        hikvision_dec_byte(b[2], off + 2),
        hikvision_dec_byte(b[3], off + 3),
    ]))
}

/// Adobe Acrobat Reader RNA IPC Message (.13ec / .7a4).
pub(crate) fn verify_adobe_rna(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ADOBE_RNA_SIZE,
        prefix.len() >= 48,
        get_slice(prefix, 4, 12)? == b"\x03\0\0\0\x05\0\0\0",
        le32(prefix, 16) == Some(2),
        get_slice(prefix, 24, 41)? == b"\x01\x01\0\0\0\x01\x01\x64\x01\x02\x01\x05rna-d"
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::AdobeRna)
}

/// Allwinner `eGON.BT0` / `eGON.BT1` Bootloader Image (`.bin`).
pub(crate) fn verify_allwinner_egon(prefix: &[u8], size: u64) -> Option<Label> {
    const ARM_BRANCH_BYTE3: usize = 3; // @3: 0xEA (ARM unconditional branch opcode byte)
    const EGON_MAGIC_OFF: usize = 4; // @4..12: "eGON.BT0" (SPL) or "eGON.BT1" (U-Boot/TOC1)
    const LEN_OFF: usize = 16; // @16: u32 LE image length (multiple of 512, >= 512)
    require!(
        size >= MIN_ALLWINNER_EGON_SIZE,
        prefix.len() >= 32,
        u8_at(prefix, ARM_BRANCH_BYTE3) == Some(0xEA),
        eq_at(prefix, EGON_MAGIC_OFF, b"eGON.BT0") || eq_at(prefix, EGON_MAGIC_OFF, b"eGON.BT1")
    );
    let len = le32(prefix, LEN_OFF)?;
    require!(len >= 512 && len % 32 == 0);
    Some(Label::AllwinnerEgon)
}

/// Allwinner LiveSuit / PhoenixSuit IMAGEWTY Firmware Image (`.img`).
pub(crate) fn verify_allwinner_imagewty(prefix: &[u8], size: u64) -> Option<Label> {
    const IMAGEWTY_MAGIC: &[u8; 8] = b"IMAGEWTY"; // @0: 8-byte LiveSuit image signature
    const VER_OFF: usize = 8; // @8: u32 LE header version (0x100234 or 0x000300)
    const HDR_SIZE_OFF: usize = 12; // @12: u32 LE header size (0x50 or 0x60)
    require!(
        size >= MIN_ALLWINNER_IMAGEWTY_SIZE,
        prefix.len() >= 64,
        starts(prefix, IMAGEWTY_MAGIC),
        matches!(le32(prefix, VER_OFF), Some(0x0010_0234 | 0x0000_0300)),
        matches!(le32(prefix, HDR_SIZE_OFF), Some(0x50 | 0x60))
    );
    Some(Label::AllwinnerImagewty)
}

/// Alpha Networks WRGG03 / DKBS Router Firmware Image (`.bin`).
pub(crate) fn verify_alpha_wrgg(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC1_OFF: usize = 32; // @32: u32 (LE or BE) 0x20080321
    const MAGIC2_OFF: usize = 36; // @36: u32 (LE or BE) 0x20080321
    const WRGG_MAGIC_VAL: u32 = 0x2008_0321;
    require!(
        size >= MIN_ALPHA_WRGG_SIZE,
        prefix.len() >= 64,
        get_slice(prefix, 0, 8)?.iter().all(|&b| b.is_ascii_graphic())
    );
    let is_le = le32(prefix, MAGIC1_OFF) == Some(WRGG_MAGIC_VAL)
        && le32(prefix, MAGIC2_OFF) == Some(WRGG_MAGIC_VAL);
    let is_be = be32(prefix, MAGIC1_OFF) == Some(WRGG_MAGIC_VAL)
        && be32(prefix, MAGIC2_OFF) == Some(WRGG_MAGIC_VAL);
    require!(is_le || is_be);
    Some(Label::AlphaWrgg)
}

/// Linux ALSA SoC Audio DSP Topology Binary (`.tplg`, `"CoSA"` at `@0..4`).
pub(crate) fn verify_alsa_soc_topology(prefix: &[u8], size: u64) -> Option<Label> {
    const ABI_OFF: usize = 4; // @4..8: u32 LE ABI version (4..=8)
    const PAYLOAD_SIZE_OFF: usize = 12; // @12..16: u32 LE first block payload size (> 0)
    const HDR_SIZE_OFF: usize = 16; // @16..20: u32 LE sizeof(snd_soc_tplg_hdr) == 0x24 (36)

    require!(
        size >= MIN_ALSA_SOC_TOPOLOGY_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"CoSA"),
        (4..=8).contains(&le32(prefix, ABI_OFF)?),
        le32(prefix, HDR_SIZE_OFF) == Some(0x24)
    );
    let blk_sz = le32(prefix, PAYLOAD_SIZE_OFF)? as u64;
    require!(blk_sz > 0, 36 + blk_sz <= size);
    Some(Label::AlsaSocTopology)
}

/// Altera / Intel Quartus Programmer Object File (`.pof`).
///
/// Header layout:
/// - `@0..4`: `"POF\0"` (`50 4F 46 00`)
/// - `@4..8`: `u32 LE version` (`0x00010000`, `0x00000001`, or `0x00020000`)
/// - `@8..12`: `u32 LE packet_count` (`1..=65536`)
/// - `@12..14`: `u16 LE first_tag` (`1` = creator software string packet)
/// - `@14..18`: `u32 LE first_length` (`4..=512`, `18 + first_length <= size`)
pub(crate) fn verify_altera_quartus_pof(prefix: &[u8], size: u64) -> Option<Label> {
    const POF_MAGIC: &[u8; 4] = b"POF\0"; // @0..4: Altera/Intel Quartus POF magic
    const VERSION_OFFSET: usize = 4; // @4..8: u32 LE format version
    const PACKET_COUNT_OFFSET: usize = 8; // @8..12: u32 LE total TLV packets
    const FIRST_TAG_OFFSET: usize = 12; // @12..14: u16 LE first packet tag (1 = software banner)
    const FIRST_LEN_OFFSET: usize = 14; // @14..18: u32 LE first packet byte length

    require!(
        size >= MIN_QUARTUS_POF_SIZE,
        prefix.len() >= 24,
        starts(prefix, POF_MAGIC),
        matches!(le32(prefix, VERSION_OFFSET), Some(0x0001_0000 | 0x0000_0001 | 0x0002_0000)),
        (1..=65536).contains(&le32(prefix, PACKET_COUNT_OFFSET)?),
        (1..=16).contains(&le16(prefix, FIRST_TAG_OFFSET)?)
    );
    let first_len = le32(prefix, FIRST_LEN_OFFSET)? as u64;
    require!((4..=512).contains(&first_len), 18 + first_len <= size);
    Some(Label::QuartusPof)
}

/// AMD x86-64 Processor Microcode Update Container (`.bin`, `"DMA\0"`).
pub(crate) fn verify_amd_cpu_microcode(prefix: &[u8], size: u64) -> Option<Label> {
    const EQ_TABLE_TYPE_OFF: usize = 4; // @4..8: u32 LE equivalence table section type (0)
    const EQ_TABLE_LEN_OFF: usize = 8; // @8..12: u32 LE equivalence table byte length (multiple of 16)
    const CONTAINER_HDR_LEN: usize = 12; // @0..12: 12-byte AMD microcode container header

    require!(
        size >= MIN_AMD_MICROCODE_SIZE,
        prefix.len() >= 64,
        starts(prefix, b"DMA\0"),
        le32(prefix, EQ_TABLE_TYPE_OFF) == Some(0)
    );
    let eq_len = le32(prefix, EQ_TABLE_LEN_OFF)? as usize;
    require!(
        eq_len >= 16,
        eq_len.is_multiple_of(16),
        CONTAINER_HDR_LEN + eq_len + 8 <= prefix.len(),
        (CONTAINER_HDR_LEN + eq_len + 64) as u64 <= size,
        le32(prefix, CONTAINER_HDR_LEN + eq_len) == Some(1)
    );
    let patch_len = le32(prefix, CONTAINER_HDR_LEN + eq_len + 4)? as u64;
    require!(patch_len >= 64, (CONTAINER_HDR_LEN + eq_len + 8) as u64 + patch_len <= size);
    Some(Label::AmdMicrocode)
}

/// AMD Platform Security Processor / NPU / TEE Signed Firmware Image (`"$PS1"` at `@0x10..0x14`).
pub(crate) fn verify_amd_psp_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const PS1_MAGIC_OFF: usize = 0x10; // @0x10..0x14: "$PS1" magic
    const BODY_SIZE_OFF: usize = 0x14; // @0x14..0x18: u32 LE payload body size
    const ZERO_PAD_OFF: usize = 0x18; // @0x18..0x30: 24 bytes zero padding
    const SIGNED_FLAG_OFF: usize = 0x30; // @0x30..0x34: u32 LE fw_type / signed flag (1)

    require!(
        size >= MIN_AMD_PSP_FIRMWARE_SIZE,
        prefix.len() >= 0x40,
        eq_at(prefix, PS1_MAGIC_OFF, b"$PS1"),
        (1..size).contains(&(le32(prefix, BODY_SIZE_OFF)? as u64)),
        get_slice(prefix, ZERO_PAD_OFF, ZERO_PAD_OFF + 24)? == [0u8; 24],
        le32(prefix, SIGNED_FLAG_OFF) == Some(1)
    );
    Some(Label::AmdPspFirmware)
}

/// AMD Radeon / AMDGPU Graphics & Compute Microcode Binary (`common_firmware_header` or IP Discovery `"IPDS"`).
pub(crate) fn verify_amdgpu_microcode(prefix: &[u8], size: u64) -> Option<Label> {
    const IPDS_MAGIC: &[u8; 4] = hex2bin!("07 14 21 28"); // @0..4: 0x28211407 LE IP Discovery Binary signature
    const IPDS_TAG_OFF: usize = 60; // @60..64: "IPDS" ASCII tag

    if size >= 64
        && prefix.len() >= 64
        && starts(prefix, IPDS_MAGIC)
        && eq_at(prefix, IPDS_TAG_OFF, b"IPDS")
    {
        return Some(Label::AmdgpuUcode);
    }

    const FILE_SIZE_OFF: usize = 0; // @0..4: u32 LE size_bytes
    const HDR_SIZE_OFF: usize = 4; // @4..8: u32 LE header_size_bytes
    const HDR_VER_MAJ_OFF: usize = 8; // @8..10: u16 LE header_version_major (1..=5)
    const HDR_VER_MIN_OFF: usize = 10; // @10..12: u16 LE header_version_minor (0..=10)
    const IP_VER_MAJ_OFF: usize = 12; // @12..14: u16 LE ip_version_major (1..=32)
    const IP_VER_MIN_OFF: usize = 14; // @14..16: u16 LE ip_version_minor (0..=32)
    const UCODE_SIZE_OFF: usize = 20; // @20..24: u32 LE ucode_size_bytes
    const UCODE_OFF_OFF: usize = 24; // @24..28: u32 LE ucode_array_offset_bytes
    const CRC32_OFF: usize = 28; // @28..32: u32 LE crc32 (!= 0)

    require!(size >= 256, prefix.len() >= 256);
    let size_bytes = le32(prefix, FILE_SIZE_OFF)? as u64;
    let hdr_size = le32(prefix, HDR_SIZE_OFF)?;
    let h_maj = le16(prefix, HDR_VER_MAJ_OFF)?;
    let h_min = le16(prefix, HDR_VER_MIN_OFF)?;
    let ip_maj = le16(prefix, IP_VER_MAJ_OFF)?;
    let ip_min = le16(prefix, IP_VER_MIN_OFF)?;
    let ucode_size = le32(prefix, UCODE_SIZE_OFF)? as u64;
    let ucode_off = le32(prefix, UCODE_OFF_OFF)?;
    let csum = le32(prefix, CRC32_OFF)?;

    require!(
        size_bytes >= 256,
        size_bytes <= size,
        size - size_bytes <= 0x400,
        (0x1C..=0x400).contains(&hdr_size),
        hdr_size % 4 == 0,
        (1..=5).contains(&h_maj),
        h_min <= 10,
        (1..=32).contains(&ip_maj),
        ip_min <= 32,
        csum != 0,
        (hdr_size.max(0x100)..=0x1000).contains(&ucode_off),
        ucode_off % 4 == 0,
        ucode_size > 0 || (size_bytes == size && ucode_off == 0x100),
        (ucode_off as u64) + ucode_size <= size_bytes
    );
    let pad_start = (hdr_size as usize).min(256);
    let pad_end = (ucode_off as usize).min(256);
    if pad_start < pad_end {
        let pad = get_slice(prefix, pad_start, pad_end)?;
        require!(pad.iter().all(|&b| b == 0));
    }
    Some(Label::AmdgpuUcode)
}

/// Amlogic eMMC Partition Table (`"MPT\0"`).
pub(crate) fn verify_amlogic_partition_table(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AMLOGIC_MPT_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"MPT\0"),
        get_slice(prefix, 4, 16)?.iter().any(|&b| b != 0)
    );
    Some(Label::AmlogicMpt)
}

/// Amlogic USB Burning Tool Upgrade Package (`aml_upgrade_package.img`) Amlogic.
pub(crate) fn verify_amlogic_upgrade(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AMLOGIC_UPGRADE_SIZE,
        prefix.len() >= 64,
        eq_at(prefix, 8, hex2bin!("56 19 B5 27")),
        (1..=2).contains(&le32(prefix, 4)?),
        matches!(le32(prefix, 0x14)?, 4 | 8 | 16),
        (1..=256).contains(&le32(prefix, 0x18)?),
    );
    Some(Label::AmlogicUpgrade)
}

/// Android Bootloader Signed Image Wrapper (`BOOTAND`, `0x12345678` LE).
pub(crate) fn verify_android_bootloader_img(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ANDROID_BOOTLOADER_IMG_SIZE,
        prefix.len() >= 260,
        starts(prefix, hex2bin!("78 56 34 12")),
        le32(prefix, 4) == Some(256) || le32(prefix, 12) == Some(256)
    );
    let sig_len = le32(prefix, 4)? as u64;
    let zip_len = le32(prefix, 8)? as u64;
    require!(256 + sig_len + zip_len == size || eq_at(prefix, 256, b"PK\x03\x04"));
    Some(Label::AndroidBootloaderImg)
}

/// Android Device-Tree Blob Overlay (`0x1EABB7D7` BE = `D7 B7 AB 1E`).
pub(crate) fn verify_android_dtbo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ANDROID_DTBO_SIZE,
        prefix.len() >= 20,
        starts(prefix, &[0xD7, 0xB7, 0xAB, 0x1E]),
        be32(prefix, 8) == Some(32),
        be32(prefix, 12) == Some(32),
        be32(prefix, 16)? >= 1
    );
    Some(Label::AndroidDtbo)
}

/// Android Minikin Compiled Hyphenation Dictionary (`.hyb`, 2015 Google, `"hy\xADb"`).
pub(crate) fn verify_android_hyphenation_dict(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ANDROID_HYB_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"hy\xADb"),
        le32(prefix, 4) == Some(0)
    );
    let (a, b, c, d) = (le32(prefix, 8)?, le32(prefix, 12)?, le32(prefix, 16)?, le32(prefix, 20)?);
    require!(a >= 24, a <= b, b <= c, c <= d, (d as u64) <= size);
    Some(Label::AndroidHyb)
}

/// Google Android Verified Boot 2.0 VBMeta Image (AVB0).
pub(crate) fn verify_android_vbmeta(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ANDROID_VBMETA_SIZE
            && prefix.len() >= 32
            && starts(prefix, b"AVB0")
            && be32(prefix, 4) == Some(1)
            && be32(prefix, 8).unwrap_or(99) <= 20
    );
    Some(Label::AndroidVbmeta)
}

/// Android AOSP Winscope WindowManager / SurfaceFlinger / ProtoLog Trace (`\x09WINTRACE`, `\x09LYRTRACE`, `\x09PROTOLOG`, etc.).
pub(crate) fn verify_android_winscope_trace(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ANDROID_WINSCOPE_TRACE_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"\x09PROTOLOG")
            || starts(prefix, b"\x09WINTRACE")
            || starts(prefix, b"\x09LYRTRACE")
            || starts(prefix, b"\x09TNXTRACE")
            || starts(prefix, b"\x09TRNTRACE")
            || starts(prefix, b"\x09WMSTRACE")
            || starts(prefix, b"\x09IMCTRACE")
            || starts(prefix, b"\x09IMSTRACE")
            || starts(prefix, b"\x09IMMTRACE")
            || starts(prefix, b"\x09A11YTRAC"),
        matches!(u8_at(prefix, 9)?, 0x0A | 0x10 | 0x12 | 0x18 | 0x1A | 0x22)
    );
    Some(Label::AndroidWinscopeTrace)
}

/// AOR AR-DV10 SDR Receiver Encrypted Firmware Image (`.d10`) AOR.
pub(crate) fn verify_aor_dv10_fw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AOR_DV10_FW_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 4)?.iter().all(|&b| b.is_ascii_digit()),
        u8_at(prefix, 4)?.is_ascii_uppercase(),
        eq_at(prefix, 5, b"     "),
        get_slice(prefix, 10, 16)?.iter().all(|&b| b.is_ascii_digit()),
        u8_at(prefix, 16)? < 6,
        u8_at(prefix, 17)? < 2,
        (32..=0x0200_0000).contains(&be32(prefix, 22)?),
    );
    Some(Label::AorDv10Fw)
}

/// Apple S5L8900 Secure Boot Firmware Container (`89001.0`, `.dfu` / `.img2`) Apple.
pub(crate) fn verify_apple_8900(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APPLE_8900_SIZE,
        prefix.len() >= 64,
        starts(prefix, b"89001.0"),
        matches!(u8_at(prefix, 7)?, 0x01..=0x04),
    );
    let size_of_data = le32(prefix, 0x0C)?;
    let footer_sig_off = le32(prefix, 0x10)?;
    require!((1..=0x1000_0000).contains(&size_of_data) && footer_sig_off <= size_of_data);
    Some(Label::Apple8900)
}

/// Apple System Log (`.asl`, 2005 Apple) binary database.
pub(crate) fn verify_apple_asl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APPLE_ASL_SIZE,
        prefix.len() >= 80,
        starts(prefix, b"ASL DB\0\0\0\0\0\0"),
        be32(prefix, 12) == Some(2)
    );
    Some(Label::AppleAsl)
}

/// Apple / NeXTSTEP Bill of Materials (BOMStore).
pub(crate) fn verify_apple_bom(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APPLE_BOM_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"BOMStore"),
        be32(prefix, 8) == Some(1)
    );
    let num_blocks = be32(prefix, 12)? as u64;
    let index_off = be32(prefix, 16)? as u64;
    let index_len = be32(prefix, 20)? as u64;
    let vars_off = be32(prefix, 24)? as u64;
    let vars_len = be32(prefix, 28)? as u64;
    let is_empty_bom = size == 512
        && num_blocks == 0
        && index_off == 0
        && index_len == 0
        && vars_off == 0
        && vars_len == 0
        && get_slice(prefix, 12, prefix.len().min(512))?.iter().all(|&b| b == 0);
    let is_populated_bom = index_off >= 32
        && index_len >= 4
        && index_off.saturating_add(index_len) <= size
        && vars_off >= 32
        && vars_len >= 4
        && vars_off.saturating_add(vars_len) <= size;
    require!(is_empty_bom || is_populated_bom);
    Some(Label::Bom)
}

/// Apple Maps Vector Map Tile Container (`VMP4`, `.vf`) Apple.
pub(crate) fn verify_apple_vmp4(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APPLE_VMP4_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"VMP4\x00\x00"),
        (8..=64).contains(&le16(prefix, 6)?),
    );
    let num_secs = le16(prefix, 8)? as usize;
    let first_type = le16(prefix, 10)?;
    let first_off = le32(prefix, 12)? as usize;
    require!(
        (1..=512).contains(&num_secs)
            && (1..=256).contains(&first_type)
            && first_off >= 10 + num_secs * 10
    );
    Some(Label::AppleVmp4)
}

/// Nintendo 64 APS 1.0 ROM Patch (`.aps`, `"APS10"`).
pub(crate) fn verify_aps_rom_patch(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APS_PATCH_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"APS10"),
        u8_at(prefix, 5)? <= 1,
        u8_at(prefix, 6)? == 0
    );
    Some(Label::ApsN64Patch)
}

/// Arcadyan Obfuscated LZMA Router Firmware Image (`.bin`).
pub(crate) fn verify_arcadyan_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const ARCADYAN_MAGIC: &[u8; 4] = hex2bin!("30 05 11 20"); // @0: Arcadyan header signature
    const LZMA_SWAP_OFF: usize = 0x68; // @0x68 (104): byte-swapped LZMA properties + dictionary size (00 D5 08 00)
    require!(
        size >= MIN_ARCADYAN_FW_SIZE,
        prefix.len() >= 112,
        starts(prefix, ARCADYAN_MAGIC),
        eq_at(prefix, LZMA_SWAP_OFF, hex2bin!("00 D5 08 00"))
    );
    Some(Label::ArcadyanFw)
}

/// ArcSoft PhotoStudio UI Layout Definition (`.aui`, 1998) ArcSoft.
pub(crate) fn verify_arcsoft_ui_aui(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ARCSOFT_UI_AUI_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"AUI\x00\x00\x01\x00\x00Arcsoft UI file\x00")
    );
    Some(Label::ArcsoftUiAui)
}

/// ARM Mali GPU Command Stream Frontend MCU Firmware Image (`6E 3A F1 C3` at `@0..4`).
pub(crate) fn verify_arm_mali_csffw(prefix: &[u8], size: u64) -> Option<Label> {
    const MALI_CSF_MAGIC: &[u8; 4] = hex2bin!("6E 3A F1 C3"); // @0..4: 0xC3F13A6E LE Mali CSF firmware magic
    const MINOR_VER_OFF: usize = 4; // @4..8: u32 LE header version (0..=16)
    const ZERO_OFF: usize = 12; // @12..16: u32 LE reserved zero
    const PAYLOAD_SIZE_OFF: usize = 20; // @20..24: u32 LE header/section offset (64..=size)

    require!(
        size >= MIN_ARM_MALI_CSFFW_SIZE,
        prefix.len() >= 32,
        starts(prefix, MALI_CSF_MAGIC),
        le32(prefix, MINOR_VER_OFF)? <= 16,
        le32(prefix, ZERO_OFF) == Some(0),
        (64..=size).contains(&(le32(prefix, PAYLOAD_SIZE_OFF)? as u64))
    );
    Some(Label::ArmMaliCsffw)
}

/// Apple Unified Logging Shared-Cache Strings (`.dsc`, `"hcsd"`, 2016 Apple) catalog.
pub(crate) fn verify_aul_dsc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AUL_DSC_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"hcsd"),
        matches!(le16(prefix, 4), Some(1 | 2)),
        le16(prefix, 6) == Some(0),
        (1..=0x0010_0000).contains(&le32(prefix, 8)?),
        (1..=0x0010_0000).contains(&le32(prefix, 12)?)
    );
    Some(Label::AulDsc)
}

/// Apple Unified Logging `.timesync` (`0xBBB0` LE = `B0 BB`, 2016 Apple) clock synchronization log.
pub(crate) fn verify_aul_timesync(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AUL_TIMESYNC_SIZE,
        prefix.len() >= 48,
        le16(prefix, 0) == Some(0xBBB0),
        le16(prefix, 2) == Some(48),
        le32(prefix, 4) == Some(0),
        le32(prefix, 24)? > 0,
        le32(prefix, 28)? > 0
    );
    Some(Label::AulTimesync)
}

/// Apple Unified Logging `uuidtext` (`0x66778899` LE = `99 88 77 66`, 2016 Apple) string catalog.
pub(crate) fn verify_aul_uuidtext(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AUL_UUIDTEXT_SIZE,
        prefix.len() >= 20,
        le32(prefix, 0) == Some(0x6677_8899),
        le32(prefix, 4) == Some(2),
        le32(prefix, 8) == Some(1)
    );
    let entry_count = le32(prefix, 12)?;
    require!(
        entry_count <= 0x0010_0000,
        entry_count > 0 || u8_at(prefix, 16).unwrap_or(0).is_ascii_graphic()
    );
    Some(Label::AulUuidtext)
}

/// Autel Diagnostic Encrypted Firmware Container (`.ecc`).
pub(crate) fn verify_autel_ecc(prefix: &[u8], size: u64) -> Option<Label> {
    const ECC_MAGIC: &[u8; 8] = b"ECC0101\0"; // @0: 8-byte versioned magic
    const PAYLOAD_SIZE_OFF: usize = 8; // @8: u32 LE encrypted payload size (> 0)
    const HDR_SIZE_OFF: usize = 12; // @12: u32 LE header size (0x20 = 32)
    const COPYRIGHT_OFF: usize = 16; // @16..32: "Copyright Autel\0"
    require!(
        size >= MIN_AUTEL_ECC_SIZE,
        prefix.len() >= 32,
        starts(prefix, ECC_MAGIC),
        le32(prefix, PAYLOAD_SIZE_OFF)? > 0,
        le32(prefix, HDR_SIZE_OFF) == Some(0x20),
        eq_at(prefix, COPYRIGHT_OFF, b"Copyright Autel\0")
    );
    Some(Label::AutelEcc)
}

/// `Ldato::rp::detail::*` UI Cache (`.cache-scc-ui`) header (`u32 LE record_len` `@0..4` + `"|Ldato::rp::detail::"` `@4..24`).
pub(crate) fn verify_beyondtrust_scc_ui(prefix: &[u8], size: u64) -> Option<Label> {
    const LDATO_DETAIL_MAGIC: &[u8; 20] = b"|Ldato::rp::detail::"; // @4..24: Ldato RPC cache type prefix
    const MAGIC_OFFSET: usize = 4; // @4: follows u32 LE record byte length

    require!(size >= MIN_BEYONDTRUST_SCC_UI_SIZE, prefix.len() >= 44);
    let rec_len = le32(prefix, 0)?;
    require!(
        (24..=65536).contains(&rec_len),
        eq_at(prefix, MAGIC_OFFSET, LDATO_DETAIL_MAGIC),
        has_non_ascii_text_byte(prefix.get(..4)?)
    );
    Some(Label::BeyondtrustSccUi)
}

/// BlueStripe FactFinder / Microsoft Monitoring BlackBox Capture (.hb) BlueStripe Software.
pub(crate) fn verify_bluestripe_hb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BLUESTRIPE_HB_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"\xff\x11BlueStripeBlackBox\x01\0\0\0")
    );
    Some(Label::BluestripeHb)
}

/// B&R Industrial Automation Studio 3 Trace Capture (.as3trace) B&R Industrial Automation.
pub(crate) fn verify_br_as3trace(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BR_AS3TRACE_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"as3trace"),
        le32(prefix, 12) == Some(0),
        le64(prefix, 20) == Some(0)
    );
    Some(Label::BrAs3trace)
}

/// Broadcom CFE ImageTag Router Firmware Image (`.bin`).
pub(crate) fn verify_broadcom_cfe(prefix: &[u8], size: u64) -> Option<Label> {
    const SIGN1_OFF: usize = 4; // @4..24: "Broadcom Corporatio\0" (20B signiture_1 field in bcmTag.h)
    require!(
        size >= MIN_BROADCOM_CFE_SIZE,
        prefix.len() >= 64,
        matches!(u8_at(prefix, 0), Some(b'6'..=b'8')),
        get_slice(prefix, 1, 4)? == [0, 0, 0],
        eq_at(prefix, SIGN1_OFF, b"Broadcom Corporatio\0")
    );
    Some(Label::BroadcomCfe)
}

/// Broadcom / Cypress Bluetooth HCI RAM Patch Download Image (`.hcd`, `0xFC4C` `Write_RAM` HCI command stream).
pub(crate) fn verify_broadcom_hcd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BROADCOM_HCD_SIZE,
        prefix.len() >= 32,
        starts(prefix, hex2bin!("4C FC")),
        u8_at(prefix, 2)? >= 4
    );
    let mut pos = 0usize;
    let mut cmds = 0usize;
    while pos + 3 <= prefix.len() && cmds < 4 {
        let op = le16(prefix, pos)?;
        let plen = u8_at(prefix, pos + 2)? as usize;
        require!(matches!(op, 0xFC4C | 0xFC4E), plen >= 4);
        pos += 3 + plen;
        cmds += 1;
    }
    require!(cmds >= 3);
    Some(Label::BroadcomHcd)
}

/// Broadcom Set-Top Box Bootloader Image (`"%BL!"`).
pub(crate) fn verify_broadcom_stb_bootloader(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BROADCOM_STB_BOOTLOADER_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"%BL!"),
        (1..=16).contains(&le32(prefix, 4)?),
        le32(prefix, 8) == Some(0x1000)
    );
    Some(Label::BroadcomStbBootloader)
}

/// Broadcom TRX Router Firmware Image (HDR0).
pub(crate) fn verify_broadcom_trx(prefix: &[u8], size: u64) -> Option<Label> {
    const LEN_OFF: usize = 4; // @4: u32 LE total TRX image byte length (>= 28 and <= file_size)
    const CRC_OFF: usize = 8; // @8: u32 LE CRC-32 checksum over bytes[12..len]
    const VER_OFF: usize = 14; // @14: u16 LE TRX header version (1 or 2)
    const OFF0_OFF: usize = 16; // @16: u32 LE offset to first partition (>= 28 and < len)

    require!(size >= MIN_BROADCOM_TRX_SIZE, prefix.len() >= 28, starts(prefix, b"HDR0"));
    let total_len = le32(prefix, LEN_OFF)? as u64;
    let crc = le32(prefix, CRC_OFF)?;
    let ver = le16(prefix, VER_OFF)?;
    let off0 = le32(prefix, OFF0_OFF)? as u64;
    require!(
        (28..=0x2000_0000).contains(&total_len),
        crc != 0,
        ver == 1 || ver == 2,
        off0 >= 28,
        off0 < total_len
    );
    Some(Label::BroadcomTrx)
}

/// WiX Burn Elevation Pipe Manifest (.ba).
pub(crate) fn verify_burnpipe_ba(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BURNPIPE_BA_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 20)?
            == hex2bin!("00 00 00 00 01 00 01 00 6E 01 00 00 62 01 00 00 05 00 00 00"),
        get_slice(prefix, 21, 32)? == hex2bin!("00 00 00 06 00 00 00 04 00 00 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::BurnpipeBa)
}

/// CASLOGv1 Binary Diagnostic / Usage Log (`.dlog` / `.ulog`).
pub(crate) fn verify_caslog_dlog(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CASLOG_DLOG_SIZE,
        prefix.len() >= 20,
        starts(prefix, b".-= CASLOGv1 =-."),
        matches!(le32(prefix, 16), Some(0x10 | 0x20))
    );
    Some(Label::CaslogDlog)
}

/// Google Chrome / Chromium Session & Tab Restore File (SNSS).
pub(crate) fn verify_chrome_snss(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_CHROME_SNSS_SIZE && prefix.len() >= 12 && starts(prefix, b"SNSS"));
    let version = le32(prefix, 4)?;
    let first_cmd_len = le16(prefix, 8)? as u64;
    require!((1..=5).contains(&version) && first_cmd_len > 0 && 10 + first_cmd_len <= size);
    Some(Label::ChromeSnss)
}

/// Google ChromeOS / Coreboot Flashrom Flash Map (`"__FMAP__"`).
pub(crate) fn verify_chromeos_fmap(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CHROMEOS_FMAP_SIZE,
        prefix.len() >= 56,
        starts(prefix, b"__FMAP__"),
        u8_at(prefix, 8)? == 1,
        u8_at(prefix, 9)? <= 2
    );
    let flash_size = le32(prefix, 18)?;
    let name = get_slice(prefix, 22, 54)?;
    let nul_pos = name.iter().position(|&b| b == 0).unwrap_or(32);
    let nareas = le16(prefix, 54)? as u64;
    require!(
        flash_size >= 64,
        nul_pos >= 1,
        name[..nul_pos].iter().all(|&b| (0x20..=0x7E).contains(&b)),
        (1..=256).contains(&nareas),
        size >= 56 + nareas * 42
    );
    Some(Label::ChromeosFmap)
}

/// Google ChromeOS Verified Boot 2.1 structure (`Vb2B`, `Vb2F`, `Vb2I`, `Vb2K`, `Vb2P`, `Vb2S`, v3.x).
pub(crate) fn verify_chromeos_vb21(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_CHROMEOS_VB21_SIZE, prefix.len() >= 36);
    let min_fixed = match prefix.get(0..4)? {
        b"Vb2P" | b"Vb2S" => 56u32,
        b"Vb2I" => 52u32,
        b"Vb2F" => 40u32,
        b"Vb2B" | b"Vb2K" => 36u32,
        _ => return None,
    };
    require!(prefix.len() >= min_fixed as usize);

    let ver_major = le16(prefix, 4)?;
    let ver_minor = le16(prefix, 6)?;
    let total_size = le32(prefix, 8)?;
    let fixed_size = le32(prefix, 12)?;
    let desc_size = le32(prefix, 16)?;
    let fixed_plus_desc = fixed_size.checked_add(desc_size)?;

    require!(
        ver_major == 3,
        ver_minor <= 16,
        total_size % 4 == 0,
        fixed_size % 4 == 0,
        desc_size % 4 == 0,
        fixed_size >= min_fixed,
        fixed_plus_desc <= total_size,
        (total_size as u64) <= size,
        total_size <= 64 * 1024 * 1024
    );

    if desc_size > 0 && (fixed_plus_desc as usize) <= prefix.len() {
        let desc = get_slice(prefix, fixed_size as usize, fixed_plus_desc as usize)?;
        let nul_pos = desc.iter().position(|&b| b == 0)?;
        require!(desc[..nul_pos].iter().all(|&b| (0x20..=0x7E).contains(&b)));
    }

    match prefix.get(0..4)? {
        b"Vb2P" | b"Vb2I" => {
            let key_offset = le32(prefix, 20)?;
            let key_size = le32(prefix, 24)?;
            let sig_alg = le16(prefix, 28)?;
            let hash_alg = le16(prefix, 30)?;
            require!(
                key_offset >= fixed_plus_desc,
                key_size >= 16,
                key_offset.checked_add(key_size)? <= total_size,
                sig_alg <= 16,
                hash_alg <= 8
            );
        }
        b"Vb2S" => {
            let sig_offset = le32(prefix, 20)?;
            let sig_size = le32(prefix, 24)?;
            let sig_alg = le16(prefix, 32)?;
            let hash_alg = le16(prefix, 34)?;
            require!(
                sig_offset >= fixed_plus_desc,
                sig_size >= 16,
                sig_offset.checked_add(sig_size)? <= total_size,
                sig_alg <= 16,
                hash_alg <= 8
            );
        }
        b"Vb2B" => {
            let key_offset = le32(prefix, 24)?;
            let sig_count = le32(prefix, 28)?;
            let sig_offset = le32(prefix, 32)?;
            require!(
                key_offset >= fixed_plus_desc,
                sig_offset >= key_offset,
                sig_offset <= total_size,
                (1..=32).contains(&sig_count)
            );
        }
        b"Vb2F" => {
            let sig_offset = le32(prefix, 28)?;
            let hash_count = le32(prefix, 32)?;
            let hash_offset = le32(prefix, 36)?;
            require!(
                hash_offset >= fixed_plus_desc,
                sig_offset >= hash_offset,
                sig_offset <= total_size,
                (1..=64).contains(&hash_count)
            );
        }
        b"Vb2K" => {}
        _ => return None,
    }

    Some(Label::ChromeosVb21)
}

/// Google ChromeOS Verified Boot Keyblock (`"CHROMEOS"`, `vb2_keyblock`).
pub(crate) fn verify_chromeos_vboot_keyblock(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_CHROMEOS_VBOOT_SIZE, prefix.len() >= 32, starts(prefix, b"CHROMEOS"));
    let major = le32(prefix, 8)?;
    let minor = le32(prefix, 12)?;
    let keyblock_size = le32(prefix, 16)? as u64;
    require!(
        (1..=3).contains(&major),
        minor <= 2,
        (72..=65536).contains(&keyblock_size),
        keyblock_size <= size
    );
    Some(Label::ChromeosVboot)
}

/// Firaxis Games Civilization III Editor Scenario (`.bic`, `.bix`, `.biq`).
///
/// Header layout:
/// - `@0..4`: `"BIC "` (vanilla), `"BICX"` (Play the World), or `"BICQ"` (Conquests)
/// - `@4..8`: `"VER#"` version section tag
/// - `@8..12`: `u32 LE` `num_headers` (`1..=16`)
/// - `@12..16`: `u32 LE` `header_size` (`>= 16`)
pub(crate) fn verify_civ3_bic(prefix: &[u8], size: u64) -> Option<Label> {
    const VER_TAG: &[u8; 4] = b"VER#"; // @4..8: version section header
    const VER_OFF: usize = 4;
    const NUM_HDRS_OFF: usize = 8; // @8..12: u32 LE header count (1..=16)
    const HDR_SIZE_OFF: usize = 12; // @12..16: u32 LE header byte length

    require!(
        size >= MIN_CIV3_BIC_SIZE,
        prefix.len() >= 16,
        starts(prefix, CIV3_BIC_MAGIC)
            || starts(prefix, CIV3_BICX_MAGIC)
            || starts(prefix, CIV3_BICQ_MAGIC),
        eq_at(prefix, VER_OFF, VER_TAG),
        (1..=16).contains(&le32(prefix, NUM_HDRS_OFF)?),
        le32(prefix, HDR_SIZE_OFF)? >= 16
    );
    Some(Label::Civ3Bic)
}

/// SMS-Soft CombiLoader Binary Container File (`.bcf`) SMS-Soft.
pub(crate) fn verify_combiloader_bcf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_COMBI_BCF_SIZE,
        prefix.len() >= 26,
        starts(prefix, b"SMS-SoftContFile"),
        (1..=0x1000_0000).contains(&le32(prefix, 20)?),
        (1..=32).contains(&le16(prefix, 24)?),
    );
    Some(Label::CombiloaderBcf)
}

/// coreboot CBFS Component (`"LARCHIVE"`).
pub(crate) fn verify_coreboot_cbfs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_COREBOOT_CBFS_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"LARCHIVE"),
        be32(prefix, 8)? > 0,
        be32(prefix, 20)? >= 24,
        has_non_ascii_text_byte(prefix.get(8..24)?)
    );
    Some(Label::CorebootCbfs)
}

/// Google Chrome / Chromium Crashpad Database `settings.dat` (`0x43506473` LE `"sdPC"`).
pub(crate) fn verify_crashpad_settings(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        (MIN_CRASHPAD_SETTINGS_SIZE..=4096).contains(&size),
        prefix.len() >= 28,
        starts(prefix, b"sdPC") || starts(prefix, b"CPds") || starts(prefix, b"CPAD")
    );
    let version = le32(prefix, 4)?;
    let options = le32(prefix, 8)?;
    let pad0 = le32(prefix, 12)?;
    require!((1..=2).contains(&version), options <= 0xFF, pad0 == 0);
    Some(Label::CrashpadCpad)
}

/// Creative Sound Blaster X-Fi / CA0132 DSP Microcode Container (`"XFi\0"` at `@0..4` or `"MXFL"` at `@0..4`).
pub(crate) fn verify_creative_xfi_microcode(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_CREATIVE_XFI_FW_SIZE, prefix.len() >= 16);
    if starts(prefix, b"XFi\0")
        && eq_at(prefix, 8, b"OCP\0")
        && (16..=size).contains(&(le32(prefix, 4)? as u64))
    {
        return Some(Label::CreativeXfiFw);
    }
    if starts(prefix, b"MXFL")
        && ((le32(prefix, 4) == Some(0xFFFF_FF01) && le32(prefix, 8) == Some(12))
            || (le32(prefix, 4) == Some(0x0004_0000) && le32(prefix, 8) == Some(0x0000_0400)))
    {
        return Some(Label::CreativeXfiFw);
    }
    None
}

/// Amit / Edimax CSMan Router Configuration Data (`.dat`).
pub(crate) fn verify_csman_dat(prefix: &[u8], size: u64) -> Option<Label> {
    const CS_LE_MAGIC: &[u8; 4] = b"CS\0\0"; // @0: little-endian CSMan signature
    const SC_BE_MAGIC: &[u8; 4] = b"SC\0\0"; // @0: big-endian CSMan signature
    const ZLIB_CMF_OFF: usize = 16; // @16: zlib CMF header byte (0x78)
    require!(
        size >= MIN_CSMAN_DAT_SIZE,
        prefix.len() >= 20,
        u8_at(prefix, ZLIB_CMF_OFF) == Some(0x78)
    );
    let (comp_sz, uncomp_sz) = if starts(prefix, CS_LE_MAGIC) {
        (le32(prefix, 4)?, le32(prefix, 12)?)
    } else if starts(prefix, SC_BE_MAGIC) {
        (be32(prefix, 4)?, be32(prefix, 12)?)
    } else {
        return None;
    };
    require!(comp_sz > 0, uncomp_sz > 0);
    Some(Label::CsmanDat)
}

/// Cambridge Silicon Radio `CSR-dfu2` Bluetooth Firmware Image (`.dfu`).
pub(crate) fn verify_csr_dfu2(prefix: &[u8], size: u64) -> Option<Label> {
    const CSR_DFU2_MAGIC: &[u8; 8] = b"CSR-dfu2"; // @0: 8-byte CSR DFU v2 signature
    const VER_OFF: usize = 8; // @8: u16 LE format version (1..=10)
    const FILE_LEN_OFF: usize = 10; // @10: u32 LE total file size (>= 64)
    const DESC_OFF: usize = 16; // @16..32: printable ASCII description string
    require!(
        size >= MIN_CSR_DFU2_SIZE,
        prefix.len() >= 48,
        starts(prefix, CSR_DFU2_MAGIC),
        (1..=10).contains(&le16(prefix, VER_OFF)?),
        le32(prefix, FILE_LEN_OFF)? >= 64,
        get_slice(prefix, DESC_OFF, DESC_OFF + 16)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::CsrDfu2)
}

/// CUPS Internet Printing Protocol (IPP, RFC 8010) spool control file (`c00001`).
pub(crate) fn verify_cups_ipp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CUPS_IPP_SIZE,
        prefix.len() >= 32,
        matches!(u8_at(prefix, 0)?, 1 | 2),
        u8_at(prefix, 1)? <= 2,
        be32(prefix, 4)? >= 1,
        eq_at(prefix, 8, b"\x01\x47\x00\x12attributes-charset")
    );
    Some(Label::CupsIpp)
}

/// Cypress / Broadcom Wi-Fi Country Locale Matrix Regulatory Blob (`.clm_blob`, `"BLOB<\0\0\0"` at `@0..8` + `"CLM DATA"` at `@60..68`).
pub(crate) fn verify_cypress_clm_blob(prefix: &[u8], size: u64) -> Option<Label> {
    const BLOB_MAGIC: &[u8; 8] = bytes!(b"BLOB", le32_bytes!(0x3C)); // @0..8: "BLOB" + header size 60 (0x3C)
    const PAYLOAD_OFF_OFF: usize = 24; // @24..28: u32 LE payload offset (0x3C = 60)
    const PAYLOAD_LEN_OFF: usize = 28; // @28..32: u32 LE payload size (size - 64)
    const CLM_DATA_OFF: usize = 60; // @60..68: "CLM DATA" tag

    require!(
        size >= MIN_CYPRESS_CLM_BLOB_SIZE,
        prefix.len() >= 68,
        starts(prefix, BLOB_MAGIC),
        le32(prefix, PAYLOAD_OFF_OFF) == Some(0x3C),
        (le32(prefix, PAYLOAD_LEN_OFF)? as u64) + 64 == size,
        eq_at(prefix, CLM_DATA_OFF, b"CLM DATA")
    );
    Some(Label::CypressClmBlob)
}

/// Dart IDE Desktop State File (`"Dart desktop file.\x1A"`).
pub(crate) fn verify_dart_dsk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_DART_DSK_SIZE, starts(prefix, b"Dart desktop file.\x1A"));
    Some(Label::DartDsk)
}

/// Diskovery Disk Inventory Snapshot (`.diskovery`, Aleksandr Gusev).
pub(crate) fn verify_diskovery_scan(prefix: &[u8], size: u64) -> Option<Label> {
    const DISKOVERY_MAGIC: &[u8; 4] = hex2bin!("CA FE 00 09"); // @0..4: Diskovery snapshot magic
    const RECORD_TAG: &[u8; 4] = hex2bin!("00 00 01 E0"); // @20..24: invariant snapshot header tag
    const SYSTEMTIME_OFF: usize = 4; // @4..20: big-endian Windows SYSTEMTIME (8 x u16)

    require!(
        size >= MIN_DISKOVERY_SCAN_SIZE,
        prefix.len() >= 36,
        starts(prefix, DISKOVERY_MAGIC),
        eq_at(prefix, 20, RECORD_TAG)
    );
    let year = be16(prefix, SYSTEMTIME_OFF)?;
    let month = be16(prefix, SYSTEMTIME_OFF + 2)?;
    let dow = be16(prefix, SYSTEMTIME_OFF + 4)?;
    let day = be16(prefix, SYSTEMTIME_OFF + 6)?;
    let hour = be16(prefix, SYSTEMTIME_OFF + 8)?;
    let min = be16(prefix, SYSTEMTIME_OFF + 10)?;
    let sec = be16(prefix, SYSTEMTIME_OFF + 12)?;
    let ms = be16(prefix, SYSTEMTIME_OFF + 14)?;
    require!(
        (2000..=2100).contains(&year),
        (1..=12).contains(&month),
        dow <= 6,
        (1..=31).contains(&day),
        hour <= 23,
        min <= 59,
        sec <= 59,
        ms <= 999
    );
    Some(Label::DiskoveryScan)
}

/// D-Link AIH0 Router Firmware Image (`.bin`).
pub(crate) fn verify_dlink_aih0(prefix: &[u8], size: u64) -> Option<Label> {
    const AIH0_MAGIC: &[u8; 4] = b"AIH0"; // @0: "AIH0" header tag
    const LOAD_MARKER_OFF: usize = 8; // @8: 0x80000100 BE load marker
    const IMG_SIZE_OFF: usize = 12; // @12: u32 BE payload length (> 0)
    const MODEL_OFF: usize = 32; // @32: printable ASCII model identifier (e.g. "TW100-S4W1CA")
    require!(
        size >= MIN_DLINK_AIH0_SIZE,
        prefix.len() >= 48,
        starts(prefix, AIH0_MAGIC),
        eq_at(prefix, LOAD_MARKER_OFF, hex2bin!("80 00 01 00")),
        be32(prefix, IMG_SIZE_OFF)? > 0,
        get_slice(prefix, MODEL_OFF, MODEL_OFF + 8)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::DlinkAih0)
}

/// D-Link DEAFBEAD Firmware Archive Container (`.deafbead`).
pub(crate) fn verify_dlink_deafbead(prefix: &[u8], size: u64) -> Option<Label> {
    const DEAFBEAD_MAGIC: &[u8; 4] = hex2bin!("DE AF BE AD"); // @0: 0xADBEAFDE LE / 0xDEAFBEAD BE
    const NODE_TYPE_OFF: usize = 4; // @4: u8 entry type (0x86 = directory, 0x87 = file)
    const NAME_LEN_OFF: usize = 5; // @5: u16 LE entry name length (1..=256)
    const NAME_OFF: usize = 7; // @7: ASCII entry path
    require!(
        size >= MIN_DLINK_DEAFBEAD_SIZE,
        prefix.len() >= 16,
        starts(prefix, DEAFBEAD_MAGIC),
        matches!(u8_at(prefix, NODE_TYPE_OFF), Some(0x86 | 0x87))
    );
    let name_len = le16(prefix, NAME_LEN_OFF)? as usize;
    let check_len = name_len.min(prefix.len().saturating_sub(NAME_OFF)).min(16);
    require!(
        (1..=256).contains(&name_len),
        check_len >= 1,
        get_slice(prefix, NAME_OFF, NAME_OFF + check_len)?.iter().all(|&b| b.is_ascii_graphic())
    );
    Some(Label::DlinkDeafbead)
}

/// D-Link FPKG / CPKG NetDefend Firewall Firmware Package (`.pkg`).
pub(crate) fn verify_dlink_fpkg(prefix: &[u8], size: u64) -> Option<Label> {
    const FPKG_MAGIC: &[u8; 8] = bytes!(b"FPKG", le32_bytes!(1)); // @0: "FPKG" + version 1 LE
    const CPKG_MAGIC: &[u8; 8] = bytes!(b"CPKG", le32_bytes!(1)); // @0: "CPKG" + version 1 LE
    const HDR_LEN_OFF: usize = 8; // @8: u32 BE header length (0x20..=0x10000)
    const MODEL_OFF: usize = 20; // @20: printable ASCII device model (e.g. "DFL-2560G")
    require!(
        size >= MIN_DLINK_FPKG_SIZE,
        prefix.len() >= 48,
        starts(prefix, FPKG_MAGIC) || starts(prefix, CPKG_MAGIC)
    );
    let hdr_len = be32(prefix, HDR_LEN_OFF)?;
    require!(
        (0x20..=0x10000).contains(&hdr_len),
        get_slice(prefix, MODEL_OFF, MODEL_OFF + 6)?.iter().all(|&b| b.is_ascii_graphic())
    );
    Some(Label::DlinkFpkg)
}

/// D-Link MH01 Encrypted Router Firmware Image (`.bin`).
pub(crate) fn verify_dlink_mh01(prefix: &[u8], size: u64) -> Option<Label> {
    const MH01_MAGIC: &[u8; 4] = b"MH01"; // @0 and @16: "MH01" tag
    const INNER_MH01_OFF: usize = 16; // @16: second "MH01" descriptor
    const HEX_DIGEST_OFF: usize = 32; // @32..64: 32-char ASCII hex digest
    require!(
        size >= MIN_DLINK_MH01_SIZE,
        prefix.len() >= 64,
        starts(prefix, MH01_MAGIC),
        eq_at(prefix, INNER_MH01_OFF, MH01_MAGIC),
        get_slice(prefix, HEX_DIGEST_OFF, 64)?.iter().all(|&b| b.is_ascii_hexdigit())
    );
    Some(Label::DlinkMh01)
}

/// D-Link SHRS Encrypted Router Firmware Image (`.bin`).
pub(crate) fn verify_dlink_shrs(prefix: &[u8], size: u64) -> Option<Label> {
    const SHRS_MAGIC: &[u8; 4] = b"SHRS"; // @0: "SHRS" signature
    const DEC_SIZE_OFF: usize = 4; // @4: u32 BE decrypted payload size (> 0)
    const ENC_SIZE_OFF: usize = 8; // @8: u32 BE AES-128-CBC encrypted payload size (>= dec_size, multiple of 16)
    require!(
        size >= MIN_DLINK_SHRS_SIZE,
        prefix.len() >= 32,
        starts(prefix, SHRS_MAGIC),
        has_non_ascii_text_byte(prefix.get(4..16)?)
    );
    let dec_sz = be32(prefix, DEC_SIZE_OFF)?;
    let enc_sz = be32(prefix, ENC_SIZE_OFF)?;
    require!(dec_sz > 0, enc_sz >= dec_sz, enc_sz % 16 == 0);
    Some(Label::DlinkShrs)
}

/// D-Link TLV Board-Tagged Router Firmware Image (`.bin`).
pub(crate) fn verify_dlink_tlv(prefix: &[u8], size: u64) -> Option<Label> {
    const DLINK_TLV_MAGIC: &[u8; 4] = hex2bin!("64 80 19 40"); // @0: D-Link TLV header magic
    const MODEL_OFF: usize = 4; // @4..12: printable ASCII model name (e.g. "COVR-1900")
    const BOARD_OFF: usize = 36; // @36..44: printable ASCII SoC/board tag (e.g. "MT7629B_")
    require!(
        size >= MIN_DLINK_TLV_SIZE,
        prefix.len() >= 64,
        starts(prefix, DLINK_TLV_MAGIC),
        get_slice(prefix, MODEL_OFF, MODEL_OFF + 8)?.iter().all(|&b| b.is_ascii_graphic()),
        get_slice(prefix, BOARD_OFF, BOARD_OFF + 8)?.iter().all(|&b| b.is_ascii_graphic())
    );
    Some(Label::DlinkTlv)
}

/// DMTF DSP0267 PLDM Firmware Update Package (`.pldm`).
pub(crate) fn verify_dmtf_pldm_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const PLDM_V1_0_UUID: &[u8; 16] = hex2bin!("F0 18 87 8C CB 7D 49 43 98 00 A0 2F 05 9A CA 02");
    const PLDM_V1_1_UUID: &[u8; 16] = hex2bin!("12 44 D2 64 8D 7D 47 18 A0 30 FC 8A 56 58 7D 5B");
    const PLDM_V1_2_UUID: &[u8; 16] = hex2bin!("31 19 CE 2F E8 0A 4A 99 AF 6D 46 F8 B1 21 F6 BF");
    const PLDM_V1_3_UUID: &[u8; 16] = hex2bin!("7B 29 1C 99 6D B6 42 08 80 1B 02 02 6E 46 3C 78");
    require!(
        size >= MIN_DMTF_PLDM_FIRMWARE_SIZE,
        prefix.len() >= 36,
        starts(prefix, PLDM_V1_0_UUID)
            || starts(prefix, PLDM_V1_1_UUID)
            || starts(prefix, PLDM_V1_2_UUID)
            || starts(prefix, PLDM_V1_3_UUID),
        (1..=4).contains(&u8_at(prefix, 16)?),
        (36..=4096).contains(&le16(prefix, 17)?)
    );
    Some(Label::DmtfPldmFirmware)
}

/// DOS Hayes AT Command Modem Definition File (.mdm).
pub(crate) fn verify_dos_hayes_mdm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DOS_HAYES_MDM_SIZE,
        prefix.len() >= 96,
        get_slice(prefix, 0, 24)?
            == hex2bin!("5E 4D 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..96)?));
    Some(Label::DosHayesMdm)
}

/// Flattened Device Tree Blob (.dtb).
pub(crate) fn verify_dtb(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_DTB_SIZE && prefix.len() >= 12 && starts(prefix, &[0xD0, 0x0D, 0xFE, 0xED]) {
        let totalsize = be32(prefix, 4).unwrap_or(0);
        if totalsize >= MIN_DTB_SIZE as u32 {
            return Some(Label::Dtb);
        }
    }
    None
}

/// Durager UV Lamp Calibration Record (.real).
pub(crate) fn verify_durager_uv_real(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DURAGER_UV_REAL_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 1, 14)? == hex2bin!("00 00 00 03 02 01 04 05 05 00 05 00 29"),
        get_slice(prefix, 16, 32)? == hex2bin!("32 00 00 00 00 00 00 00 00 00 00 00 E8 03 00 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::DuragerUvReal)
}

/// emTeX Dot-Matrix Printer Driver Table (`.dot`, 1990, Eberhard Mattes).
///
/// Header layout: `@0..4 = b"DOT\0"`, `@4..6 u16 LE version` (`2` or `3`),
/// `@6..8 u16 LE sub_version` (`1`), `@8..10 u16 LE header_size` (`156` in v2, `220` in v3),
/// `@10..12 u16 LE data_size`, satisfying exact `header_size + data_size == size`.
pub(crate) fn verify_emtex_dot_driver(prefix: &[u8], size: u64) -> Option<Label> {
    const VER_OFF: usize = 4; // @4..6: u16 LE driver format version (2 or 3)
    const SUB_OFF: usize = 6; // @6..8: u16 LE sub-version (1)
    const HDR_SZ_OFF: usize = 8; // @8..10: u16 LE fixed header size (156 or 220)
    const DATA_SZ_OFF: usize = 10; // @10..12: u16 LE escape-string/data pool size

    require!(size >= MIN_EMTEX_DOT_SIZE, prefix.len() >= 12, starts(prefix, EMTEX_DOT_MAGIC));
    let version = le16(prefix, VER_OFF)?;
    let sub_version = le16(prefix, SUB_OFF)?;
    let hdr_size = le16(prefix, HDR_SZ_OFF)?;
    let data_size = le16(prefix, DATA_SZ_OFF)?;
    require!(
        sub_version == 1,
        (version == 2 && hdr_size == 156) || (version == 3 && hdr_size == 220),
        data_size >= 1,
        (hdr_size as u64) + (data_size as u64) == size
    );
    Some(Label::EmtexDotDriver)
}

/// emTeX TeX Codepage Translation Table (`.tcp`, 1991, Eberhard Mattes).
///
/// Layout: `@0..4 = b"tcp\x02"`, `@4..260 xlat_in[256]`, `@260..516 xlat_out[256]`,
/// `@516..772 u16 LE macro_offsets[128]` (`0xFFFF` = unmapped, else `< pool_size`),
/// `@772..776 u32 LE pool_size`, followed by exact `pool_size` bytes (`776 + pool_size == size`).
pub(crate) fn verify_emtex_tcp_codepage(prefix: &[u8], size: u64) -> Option<Label> {
    const MACRO_OFFS_START: usize = 516; // @516..772: 128 x u16 LE replacement macro offsets
    const POOL_SIZE_OFF: usize = 772; // @772..776: u32 LE replacement string pool size
    const FIXED_HDR_SIZE: usize = 776;

    require!(
        size >= MIN_EMTEX_TCP_SIZE,
        prefix.len() >= FIXED_HDR_SIZE,
        starts(prefix, EMTEX_TCP_MAGIC)
    );
    let pool_size = le32(prefix, POOL_SIZE_OFF)? as u64;
    require!((FIXED_HDR_SIZE as u64).checked_add(pool_size)? == size);
    for i in 0..128 {
        let off = le16(prefix, MACRO_OFFS_START + i * 2)?;
        require!(off == 0xFFFF || (off as u64) < pool_size);
    }
    Some(Label::EmtexTcpCodepage)
}

/// Essential Objects EO.WebEngine IPC Cache (.0_1 / .0_1_x64) Essential Objects.
pub(crate) fn verify_eo_webengine(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EO_WEBENGINE_SIZE,
        prefix.len() >= 48,
        get_slice(prefix, 12, 44)? == b"E\0O\0.\0W\0e\0b\0E\0n\0g\0i\0n\0e\0,\0 \0V\0e\0"
            || get_slice(prefix, 16, 48)? == b"E\0O\0.\0W\0e\0b\0E\0n\0g\0i\0n\0e\0,\0 \0V\0e\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::EoWebengine)
}

/// Espressif ESP-IDF Flash Partition Table Binary.
pub(crate) fn verify_espressif_partition_table(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ESP_PARTTABLE_SIZE && prefix.len() >= 64);
    require!(u8_at(prefix, 0)? == 0xAA && u8_at(prefix, 1)? == 0x50);
    // Validate first two 32-byte entries (either standard 0xAA 0x50 entry or 0xEB 0xEB MD5 trailer)
    for entry_idx in 0..2 {
        let base = entry_idx * 32;
        if u8_at(prefix, base)? == 0xEB && u8_at(prefix, base + 1)? == 0xEB {
            break;
        }
        require!(u8_at(prefix, base)? == 0xAA && u8_at(prefix, base + 1)? == 0x50);
        let ptype = u8_at(prefix, base + 2)?;
        require!(matches!(ptype, 0x00 | 0x01 | 0x40..=0xFE));
        let offset = le32(prefix, base + 4)?;
        let length = le32(prefix, base + 8)?;
        require!(length > 0 && (offset == 0 || (offset & 0xFFF) == 0));
        let label = get_slice(prefix, base + 12, base + 28)?;
        let nul_pos = label.iter().position(|&b| b == 0).unwrap_or(16);
        require!(nul_pos > 0 && label[..nul_pos].iter().all(|&b| (0x20..=0x7E).contains(&b)));
    }
    Some(Label::EspParttable)
}

/// Neos Eureka S.r.l. EurekaLog Delphi/C++Builder Exception Log (.el, 2011) EurekaLog.
pub(crate) fn verify_eurekalog_el(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EUREKALOG_EL_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\xff\xfeE\0u\0r\0e\0k\0a\0L\0o\0g\0 \x007\0.\0")
    );
    Some(Label::EurekalogEl)
}

/// FinalBurn Alpha / FBNeo Emulator Savestate (.fs, 2002) Team FBAlpha.
pub(crate) fn verify_fba_fs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FBA_FS_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"FB1 FS1 "),
        le32(prefix, 8).unwrap_or(0) >= 64,
        get_slice(prefix, 36, 48)? == b"\0\0\0\0\0\0\0\0\0\0\0\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::FbaFs)
}

/// NetManila / Deventec FieldLog Binary Diagnostic Trace (.fl).
pub(crate) fn verify_fieldlog_fl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FIELDLOG_FL_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"FieldLog\0\x02\x10\0\0!")
    );
    Some(Label::FieldlogFl)
}

/// `fio` (Flexible I/O Tester) Verify State (`.state`, `struct verify_state_hdr`).
pub(crate) fn verify_fio_verify_state(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_LEN: u64 = 24; // 3 x u64: version, payload_size, crc32c
    const VERSION_OFF: usize = 0; // @0..8: u64 LE format version (V3..V7)
    const PAYLOAD_SIZE_OFF: usize = 8; // @8..16: u64 LE payload byte length (size - 24)
    const CRC_OFF: usize = 16; // @16..24: u64 LE holding 32-bit CRC32C in low dword
    const NO_COMPS_OFF: usize = 24; // @24..32: u64 LE completion count in first thread_io_list

    require!(size >= MIN_FIO_VERIFY_STATE_SIZE, prefix.len() >= 32);
    let version = le64(prefix, VERSION_OFF)?;
    require!((3..=7).contains(&version));
    let payload_size = le64(prefix, PAYLOAD_SIZE_OFF)?;
    require!(payload_size >= 40, payload_size.checked_add(HDR_LEN) == Some(size));
    let crc = le64(prefix, CRC_OFF)?;
    let no_comps = le64(prefix, NO_COMPS_OFF)?;
    require!(crc != 0 && (crc >> 32) == 0, (1..=1_048_576).contains(&no_comps));
    Some(Label::FioVerifyState)
}

/// F-Secure Signed Update Manifest (FS02, .mf).
pub(crate) fn verify_fsecure_fs02_mf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FSECURE_FS02_MF_SIZE,
        prefix.len() >= 13,
        starts(prefix, b"FS02\x01\0\0\0\x08\0\0\0"),
        u8_at(prefix, 12)?.is_ascii_graphic()
    );
    require!(has_non_ascii_text_byte(prefix.get(..13)?));
    Some(Label::FsecureFs02Mf)
}

/// Mozilla Firefox / Gecko Startup Cache (`"startupcache"` + 4 ASCII version digits).
pub(crate) fn verify_gecko_startupcache(prefix: &[u8], size: u64) -> Option<Label> {
    const GECKO_STARTUPCACHE_PREFIX: &[u8; 12] = b"startupcache"; // @0..12

    require!(
        size >= MIN_GECKO_STARTUPCACHE_SIZE,
        prefix.len() >= 24,
        starts(prefix, GECKO_STARTUPCACHE_PREFIX),
        get_slice(prefix, 12, 16)?.iter().all(u8::is_ascii_digit),
        has_non_ascii_text_byte(prefix.get(16..24)?)
    );
    Some(Label::GeckoStartupcache)
}

/// GNU gettext Machine Object Compiled Translation Catalog (.mo).
pub(crate) fn verify_gettext_mo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GETTEXT_MO_SIZE
            && prefix.len() >= 28
            && (get_slice(prefix, 0, 4)? == GETTEXT_MO_MAGIC_LE
                || get_slice(prefix, 0, 4)? == [0x95, 0x04, 0x12, 0xDE])
    );
    let (rev_major, orig_off, trans_off) = if get_slice(prefix, 0, 4)? == GETTEXT_MO_MAGIC_LE {
        (
            le16(prefix, 6).unwrap_or(99),
            le32(prefix, 12).unwrap_or(0),
            le32(prefix, 16).unwrap_or(0),
        )
    } else {
        (
            be16(prefix, 4).unwrap_or(99),
            be32(prefix, 12).unwrap_or(0),
            be32(prefix, 16).unwrap_or(0),
        )
    };
    require!(rev_major <= 1 && orig_off >= 28 && trans_off >= 28);
    Some(Label::GettextMo)
}

/// Git Staging Index (`.git/index`, `"DIRC"` + `be32(4) in 2..=4`).
pub(crate) fn verify_git_index(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GIT_INDEX_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"DIRC"),
        matches!(be32(prefix, 4), Some(2..=4))
    );
    Some(Label::GitIndex)
}

/// Git Packfile v2 Index (`.idx`, `\xFFtOc` + `be32(4) == 2`).
pub(crate) fn verify_git_pack_index(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GIT_PACK_IDX_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"\xFFtOc"),
        be32(prefix, 4) == Some(2)
    );
    Some(Label::GitPackIdx)
}

/// GNU C Library Dynamic Linker Cache (`ld.so.cache`, `"glibc-ld.so.cache1.1"`).
pub(crate) fn verify_glibc_ld_so_cache(prefix: &[u8], size: u64) -> Option<Label> {
    const GLIBC_LD_CACHE_MAGIC: &[u8; 20] = b"glibc-ld.so.cache1.1"; // @0..20

    require!(
        size >= MIN_GLIBC_LD_SO_CACHE_SIZE,
        prefix.len() >= 32,
        starts(prefix, GLIBC_LD_CACHE_MAGIC),
        (1..=1_000_000).contains(&le32(prefix, 20)?), // @20..24: nlibs
        (1..=0x1000_0000).contains(&le32(prefix, 24)?), // @24..28: len_strings
        u8_at(prefix, 28)? <= 3,                      // @28: flags (endianness / extension bits)
        get_slice(prefix, 29, 32)? == [0u8; 3]
    );
    Some(Label::GlibcLdSoCache)
}

/// GNU `findutils` `locate` Database (`\0LOCATE02\0`).
pub(crate) fn verify_gnu_locate_db(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_GNU_LOCATE_DB_SIZE, starts(prefix, b"\x00LOCATE02\x00"));
    Some(Label::GnuLocateDb)
}

/// GNUnet Directory (`.gnd`, `\x89GND\r\n\x1A\n`).
pub(crate) fn verify_gnunet_directory(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_GNUNET_GND_SIZE, starts(prefix, b"\x89GND\r\n\x1A\n"));
    Some(Label::GnunetGnd)
}

/// Go Runtime Telemetry Counter File (.count, 2023) Go Authors.
pub(crate) fn verify_go_telemetry_count(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GO_TELEMETRY_COUNT_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"# telemetry/counter file v1\n"),
        u8_at(prefix, 28)? == 0xC0,
        get_slice(prefix, 32, 43)? == b"TimeBegin: "
    );
    Some(Label::GoTelemetryCount)
}

/// GNOME GObject Introspection Binary Typelib (`.typelib`, 2005 GNOME Project).
pub(crate) fn verify_gobject_typelib(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GOBJECT_TYPELIB_SIZE
            && prefix.len() >= 44
            && starts(prefix, b"GOBJ\nMETADATA\r\n\x1A")
    );
    let major = le16(prefix, 16)?;
    let size_at_40 = le32(prefix, 40)? as u64;
    let size_at_28 = le32(prefix, 28)? as u64;
    require!(major == 4 && (size_at_40 == size || size_at_28 == size));
    Some(Label::GobjectTypelib)
}

/// Windows Group Policy Registry Policy (`registry.pol`, `"PReg\x01\0\0\0"`).
pub(crate) fn verify_group_policy_registry_pol(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GPO_REGISTRY_POL_SIZE,
        starts(prefix, b"PReg\x01\x00\x00\x00"),
        size == 8 || (prefix.len() >= 10 && eq_at(prefix, 8, b"[\x00"))
    );
    Some(Label::GpoRegistryPol)
}

/// HELIOS Save State Container (.save).
pub(crate) fn verify_helios_save(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HELIOS_SAVE_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"HELIOSSV"),
        (1..=16).contains(&le32(prefix, 8).unwrap_or(0)),
        le32(prefix, 12).unwrap_or(0) >= 12
    );
    Some(Label::HeliosSave)
}

/// SEN Hiew External Information / Names / Save File.
pub(crate) fn verify_hiew_external_info(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_HIEW_EXT_INFO_SIZE, prefix.len() >= 24);
    if starts(prefix, b"Hiew External Information") {
        require!(size >= 42, prefix.len() >= 36, get_slice(prefix, 25, 36)?.contains(&0x1A));
        return Some(Label::HiewExt);
    }
    // Hiew32 v6.x–v8.x binary external files:
    // 14-byte NUL-padded subformat tag at @0..14 + 2-byte BCD version (`0x05..=0x09` major at @14)
    let bcd_major = u8_at(prefix, 14)?;
    require!((0x05..=0x09).contains(&bcd_major));
    if starts(prefix, b"HiewNames\0\0\0\0\0") {
        // Followed by 16-byte BriefLZ `BLZ\x01` container header at @16..32
        require!(
            size >= 33,
            prefix.len() >= 32,
            get_slice(prefix, 16, 20)? == b"BLZ\x01",
            le32(prefix, 20).unwrap_or(0) > 0
        );
        return Some(Label::HiewExt);
    }
    if starts(prefix, b"HiewSave\0") {
        require!(
            size >= 36,
            prefix.len() >= 36,
            get_slice(prefix, 16, 20)? == b"BLZ\x01" || get_slice(prefix, 20, 24)? == b"BLZ\x01"
        );
        return Some(Label::HiewExt);
    }
    if starts(prefix, b"HiewEditUndo\0\0") {
        require!((size - 16).is_multiple_of(8));
        return Some(Label::HiewExt);
    }
    if starts(prefix, b"HiewCMarkers\0\0") {
        require!(size >= 28 && (size - 16).is_multiple_of(12));
        return Some(Label::HiewExt);
    }
    if starts(prefix, b"HiewMacro\0\0\0\0\0") {
        let count = le16(prefix, 16)? as u64;
        require!(size >= 69 && size == 69 + 5 * count);
        return Some(Label::HiewExt);
    }
    if starts(prefix, b"HiewCHV\0") {
        require!(size >= 96 && (size - 16).is_multiple_of(80));
        return Some(Label::HiewExt);
    }
    if starts(prefix, b"HiewXlat\0") {
        require!(size >= 800 && (size - 16).is_multiple_of(0x310));
        return Some(Label::HiewExt);
    }
    None
}

/// Hikvision DVR/NVR/IPC Firmware Package (`digicap.dav`) obfuscated with
/// Hikvision's shifting 16-byte XOR schedule `key[(i + (i >> 4)) & 0x0F]`.
pub(crate) fn verify_hikvision_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_SUM_OFF: usize = 4; // @4..8: u32 LE bytewise sum over dec[12..hdr_len]
    const HDR_LEN_OFF: usize = 8; // @8..12: u32 LE plaintext header length (>= 108)
    const FILE_COUNT_OFF: usize = 12; // @12..16: u32 LE embedded file entry count
    const LANGUAGE_OFF: usize = 16; // @16..20: u32 LE firmware language ID
    const FIRST_ENTRY_NAME_OFF: usize = 64; // @64: first 44-byte entry `fileName[32]`

    require!(
        size >= MIN_HIKVISION_FIRMWARE_SIZE,
        prefix.len() >= MIN_HIKVISION_FIRMWARE_SIZE as usize
    );
    let magic_dec = [
        hikvision_dec_byte(u8_at(prefix, 0)?, 0),
        hikvision_dec_byte(u8_at(prefix, 1)?, 1),
        hikvision_dec_byte(u8_at(prefix, 2)?, 2),
        hikvision_dec_byte(u8_at(prefix, 3)?, 3),
    ];
    require!(matches!(&magic_dec, b"SWKH" | b"02KH"));

    let hdr_checksum = hikvision_dec_le32(prefix, HDR_SUM_OFF)?;
    let hdr_len = hikvision_dec_le32(prefix, HDR_LEN_OFF)? as usize;
    let file_count = hikvision_dec_le32(prefix, FILE_COUNT_OFF)? as usize;
    let language = hikvision_dec_le32(prefix, LANGUAGE_OFF)?;
    require!(
        (MIN_HIKVISION_FIRMWARE_SIZE as usize..=65_536).contains(&hdr_len),
        (hdr_len as u64) <= size,
        (1..=512).contains(&file_count),
        0x40 + file_count * 44 <= hdr_len,
        language <= 16,
        (0x21..=0x7E).contains(&hikvision_dec_byte(
            u8_at(prefix, FIRST_ENTRY_NAME_OFF)?,
            FIRST_ENTRY_NAME_OFF
        ))
    );
    if prefix.len() >= hdr_len {
        let computed: u32 = (12..hdr_len)
            .map(|i| hikvision_dec_byte(u8_at(prefix, i).unwrap_or(0), i) as u32)
            .fold(0u32, u32::wrapping_add);
        require!(computed == hdr_checksum);
    }

    Some(Label::HikvisionFirmware)
}

/// HP FutureSmart Printer Firmware Bundle (`.bdl`).
pub(crate) fn verify_hp_bdl(prefix: &[u8], size: u64) -> Option<Label> {
    const BDL_MAGIC: &[u8; 8] = bytes!(b"ibdl", le16_bytes!(1), le16_bytes!(1)); // @0: "ibdl" + v1.1 LE
    const TOC_OFF_POS: usize = 8; // @8: u32 LE TOC offset (>= 32)
    const TOC_CNT_POS: usize = 16; // @16: u32 LE TOC entry count (1..=4096)
    require!(
        size >= MIN_HP_BDL_SIZE,
        prefix.len() >= 40,
        starts(prefix, BDL_MAGIC),
        le32(prefix, TOC_OFF_POS)? >= 32,
        (1..=4096).contains(&le32(prefix, TOC_CNT_POS)?)
    );
    Some(Label::HpBdl)
}

/// HP LaserJet IPKG Firmware Package (`.ipkg`).
pub(crate) fn verify_hp_ipkg(prefix: &[u8], size: u64) -> Option<Label> {
    const IPKG_MAGIC: &[u8; 8] = bytes!(b"ipkg", le16_bytes!(1), le16_bytes!(3)); // @0: "ipkg" + v1.3 LE
    const TOC_OFF_POS: usize = 8; // @8: u32 LE TOC offset (>= 32)
    const TOC_CNT_POS: usize = 16; // @16: u32 LE TOC entry count (1..=4096)
    require!(
        size >= MIN_HP_IPKG_SIZE,
        prefix.len() >= 48,
        starts(prefix, IPKG_MAGIC),
        le32(prefix, TOC_OFF_POS)? >= 32,
        (1..=4096).contains(&le32(prefix, TOC_CNT_POS)?)
    );
    Some(Label::HpIpkg)
}

/// Huawei / HiSilicon Firmware Update Package (`UPDATE.APP`) Huawei.
pub(crate) fn verify_huawei_update_app(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HUAWEI_UPDATE_APP_SIZE,
        prefix.len() >= 112,
        get_slice(prefix, 0, 92)? == [0u8; 92],
        eq_at(prefix, 92, hex2bin!("55 AA 5A A5")),
        (98..=4096).contains(&le32(prefix, 96)?),
        le32(prefix, 100)? == 1,
    );
    Some(Label::HuaweiUpdateApp)
}

/// HXNF Troubleshoot Diagnostic Dump (.troubleshoot).
pub(crate) fn verify_hxnf_troubleshoot(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HXNF_TROUBLESHOOT_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"HXNF\x01\0\x05\x04\x19\x04!\0\x14\x13\0\0"),
        get_slice(prefix, 16, 32)? == [0u8; 16]
    );
    Some(Label::HxnfTroubleshoot)
}

/// Hytera DMR Radio Firmware Upgrade Package (.fudx) Hytera Communications.
pub(crate) fn verify_hytera_fudx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HYTERA_FUDX_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"Hytera Upgrade Fud Files5\0\0\0\0\0V")
    );
    Some(Label::HyteraFudx)
}

/// IBM OS/2 Extended Attributes Data File (`EA DATA. SF`, 1989) IBM.
pub(crate) fn verify_ibm_os2_extended_attr_ea(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_IBM_OS2_EXTENDED_ATTR_EA_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"ED\x00\x00"),
        get_slice(prefix, 4, 32)?.iter().all(|&b| b == 0),
        le16(prefix, 32)? > 0
    );
    Some(Label::IbmOs2ExtendedAttrEa)
}

/// IBM Guardium Windows S-TAP Circular Trace Log (`.ctl`).
pub(crate) fn verify_ibm_stap_ctl(prefix: &[u8], size: u64) -> Option<Label> {
    const IBM_STAP_CTL_MAGIC: &[u8; 8] = hex2bin!("39 C8 3B 07 FE 1D A9 38"); // @0..8: S-TAP trace magic
    const VERSION_OFF: usize = 8; // @8..16: u64 LE header version (e.g., 2)
    const HEAD_OFF: usize = 16; // @16..24: u64 LE ring-buffer read head offset
    const MAX_SIZE_OFF: usize = 24; // @24..32: u64 LE ring-buffer capacity (default 20 MiB)
    const WRITE_POS_OFF: usize = 32; // @32..40: u64 LE current write cursor (<= file size)
    const FILE_SIZE_OFF: usize = 40; // @40..48: u64 LE recorded file length (== file size)

    require!(size >= MIN_IBM_STAP_CTL_SIZE, prefix.len() >= 48, starts(prefix, IBM_STAP_CTL_MAGIC));
    let version = le64(prefix, VERSION_OFF)?;
    let head_off = le64(prefix, HEAD_OFF)?;
    let max_size = le64(prefix, MAX_SIZE_OFF)?;
    let write_pos = le64(prefix, WRITE_POS_OFF)?;
    let file_size = le64(prefix, FILE_SIZE_OFF)?;
    require!(
        (1..=8).contains(&version),
        head_off <= size,
        (4096..=1_073_741_824).contains(&max_size),
        write_pos <= size,
        file_size == size
    );
    Some(Label::IbmStapCtl)
}

/// NXP i.MX Smart Direct Memory Access Controller Firmware Image (`"SDMA"` at `@0..4`).
pub(crate) fn verify_imx_sdma_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_MAJOR_OFF: usize = 4; // @4..8: u32 LE version_major (1..=10)
    const RAM_CODE_START_OFF: usize = 20; // @20..24: u32 LE ram_code_start (0xD8 = 216)
    const RAM_CODE_SIZE_OFF: usize = 24; // @24..28: u32 LE ram_code_size

    require!(
        size >= MIN_IMX_SDMA_FW_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"SDMA"),
        (1..=10).contains(&le32(prefix, VERSION_MAJOR_OFF)?),
        le32(prefix, RAM_CODE_START_OFF) == Some(0xD8),
        0xD8 + (le32(prefix, RAM_CODE_SIZE_OFF)? as u64) <= size
    );
    Some(Label::ImxSdmaFw)
}

/// Infocrypt ICS Encrypted Audit Log (.icsl).
pub(crate) fn verify_infocrypt_icsl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_INFOCRYPT_ICSL_SIZE,
        prefix.len() >= 112,
        starts(prefix, b"ICS version 1."),
        get_slice(prefix, 15, 32)? == b"\r\n(c) Infocrypt, ",
        has_non_ascii_text_byte(prefix.get(32..112)?)
    );
    Some(Label::InfocryptIcsl)
}

/// INSTAR IP Camera BNEG Firmware Container (`.pkg`).
pub(crate) fn verify_instar_bneg(prefix: &[u8], size: u64) -> Option<Label> {
    const BNEG_MAGIC: &[u8; 12] = bytes!(b"BNEG", le32_bytes!(1), le32_bytes!(1)); // @0: "BNEG" + 1 LE + 1 LE
    const PART1_SIZE_OFF: usize = 12; // @12: u32 LE first partition size (> 0)
    const PART2_SIZE_OFF: usize = 16; // @16: u32 LE second partition size (> 0)
    require!(
        size >= MIN_INSTAR_BNEG_SIZE,
        prefix.len() >= 24,
        starts(prefix, BNEG_MAGIC),
        le32(prefix, PART1_SIZE_OFF)? > 0,
        le32(prefix, PART2_SIZE_OFF)? > 0
    );
    Some(Label::InstarBneg)
}

/// Intel Code Partition Directory Firmware Container (`"$CPD"` at `@0..4`).
pub(crate) fn verify_intel_cpd_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const NUM_ENTRIES_OFF: usize = 4; // @4..8: u32 LE num_of_entries (1..=64)
    const HDR_VER_OFF: usize = 8; // @8: u8 header_version (1 or 2)
    const ENTRY_VER_OFF: usize = 9; // @9: u8 entry_version (1)
    const HDR_LEN_OFF: usize = 10; // @10: u8 header_length (0x10 or 0x14)
    const PART_NAME_OFF: usize = 12; // @12..16: 4-byte ASCII partition_name

    require!(
        size >= MIN_INTEL_CPD_FIRMWARE_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"$CPD"),
        (1..=64).contains(&le32(prefix, NUM_ENTRIES_OFF)?),
        matches!(u8_at(prefix, HDR_VER_OFF), Some(1 | 2)),
        u8_at(prefix, ENTRY_VER_OFF) == Some(1),
        matches!(u8_at(prefix, HDR_LEN_OFF), Some(0x10 | 0x14))
    );
    let part_name = get_slice(prefix, PART_NAME_OFF, PART_NAME_OFF + 4)?;
    require!(part_name.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b)));
    Some(Label::IntelCpdFirmware)
}

/// Intel Dynamic Tuning Technology (DPTF) ESIF DataVault (`.dv`).
pub(crate) fn verify_intel_esif_dv(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_INTEL_ESIF_DV_SIZE,
        prefix.len() >= 45,
        starts(prefix, hex2bin!("E5 1F 94 00")),
        get_slice(prefix, 4, 7)? == [0, 0, 0],
        (1..=4).contains(&u8_at(prefix, 7)?)
    );
    let name_len = le32(prefix, 8)? as usize;
    require!(
        (2..=32).contains(&name_len),
        get_slice(prefix, 12, 12 + name_len - 1)?.iter().all(|&b| (0x20..=0x7E).contains(&b)),
        u8_at(prefix, 12 + name_len - 1)? == 0,
        u8_at(prefix, 44)?.is_ascii_graphic()
    );
    Some(Label::IntelEsifDv)
}

/// Intel SPI Flash Descriptor (IFD, 2007) Intel (`0x0FF0A55A` LE) at `base` (`0` or `16`).
///
/// Descriptor Map layout:
/// - `@0..16` (when `base == 16`): 16 erased SPI preamble bytes (`0xFF` or `0x00`)
/// - `@base..base+4`: `0x0FF0A55A` LE (`FLVALSIG`)
/// - `@base+4..base+8`: `u32 LE` `FLMAP0` (`FCBA` bits `7:0` in `1..=0x7F`, reserved bits `15:10 == 0`, `FRBA` bits `23:16` in `1..=0x7F`, upper nibble `31:28 == 0`)
/// - `@base+8..base+12`: `u32 LE` `FLMAP1` (`FMBA` bits `7:0` in `1..=0x7F`, reserved bits `15:11 == 0`)
pub(crate) fn verify_intel_flash_descriptor_at(
    prefix: &[u8], size: u64, base: usize,
) -> Option<Label> {
    const FLMAP0_REL: usize = 4; // +4..+8: u32 LE Flash Map 0 (FCBA / NC / FRBA / NR)
    const FLMAP1_REL: usize = 8; // +8..+12: u32 LE Flash Map 1 (FMBA / NM / FPSBA / PSL)
    const MAP_END_REL: usize = 12;

    require!(
        size >= MIN_INTEL_FLASH_DESCRIPTOR_SIZE,
        prefix.len() >= base + MAP_END_REL,
        eq_at(prefix, base, IFD_SIGNATURE_LE)
    );
    if base > 0 {
        require!(get_slice(prefix, 0, base)?.iter().all(|&b| b == 0xFF || b == 0x00));
    }
    let flmap0 = le32(prefix, base + FLMAP0_REL)?;
    let flmap1 = le32(prefix, base + FLMAP1_REL)?;
    let fcba = flmap0 & 0xFF;
    let frba = (flmap0 >> 16) & 0xFF;
    let fmba = flmap1 & 0xFF;

    require!(
        (1..=0x7F).contains(&fcba),
        ((flmap0 >> 8) & 0xFC) == 0,
        (1..=0x7F).contains(&frba),
        ((flmap0 >> 24) & 0xF0) == 0,
        (1..=0x7F).contains(&fmba),
        ((flmap1 >> 8) & 0xF8) == 0
    );
    Some(Label::Ifd)
}

/// Intel C600 Storage Controller Unit Cable Management OEM Firmware Binary (`"ISCUOEMB"` at `@0..8`).
pub(crate) fn verify_intel_iscu_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_INTEL_ISCU_FIRMWARE_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"ISCUOEMB"),
        le16(prefix, 8)? as u64 == size
    );
    Some(Label::IntelIscuFirmware)
}

/// Intel Vision Sensing Controller Firmware Image (`"SCVF"` at `@0..4` and `"SFWI"` at `@20..24` or `@28..32`).
pub(crate) fn verify_intel_ivsc_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_INTEL_IVSC_FIRMWARE_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"SCVF"),
        eq_at(prefix, 20, b"SFWI") || eq_at(prefix, 28, b"SFWI")
    );
    Some(Label::IntelIvscFirmware)
}

/// Intel Management Engine Flash Partition Table (`$FPT`, `.bin`) Intel.
pub(crate) fn verify_intel_me_fpt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_INTEL_ME_FPT_SIZE && prefix.len() >= 64);
    let off = if starts(prefix, b"$FPT") {
        0usize
    } else if prefix.len() >= 80 && eq_at(prefix, 16, b"$FPT") {
        16usize
    } else {
        return None;
    };
    let num_entries = le32(prefix, off + 4)?;
    let hdr_ver = u8_at(prefix, off + 8)?;
    let entry_ver = u8_at(prefix, off + 9)?;
    let hdr_len = u8_at(prefix, off + 10)?;
    require!(
        (1..=64).contains(&num_entries)
            && matches!(hdr_ver, 0x10 | 0x20 | 0x21)
            && entry_ver == 0x10
            && matches!(hdr_len, 0x20 | 0x30)
            && u8_at(prefix, off + 32)?.is_ascii_graphic()
            && get_slice(prefix, off + 32, off + 36)?
                .iter()
                .all(|&b| b.is_ascii_graphic() || b == 0)
    );
    Some(Label::IntelMeFpt)
}

/// IRLOG1 ECDSA-Signed Incident Response Log (.irlog).
pub(crate) fn verify_irlog1_log(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_IRLOG1_LOG_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"IRLOG1\x01[\0\x30\x59\x30\x13\x06\x07\x2a\x86\x48\xce\x3d")
    );
    Some(Label::Irlog1Log)
}

/// ISWLOG Encrypted Diagnostic Trace (.swl).
pub(crate) fn verify_iswlog_swl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ISWLOG_SWL_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"ISWLOG."),
        (b'1'..=b'9').contains(&u8_at(prefix, 7)?),
        le32(prefix, 8) == Some(0x40)
    );
    Some(Label::IswlogSwl)
}

/// Intel PRO/Wireless & Wi-Fi 6/7 Microcode Image (`.ucode`, `00 00 00 00 "IWL\n"` TLV or legacy v1/v2 header),
/// Payloaded NVM (`.pnvm`, headerless `iwl_ucode_tlv` starting with `PNVM_SKU` `0x40` or `PNVM_VERSION` `0x3E`),
/// or Windows Intel Wi-Fi Firmware (`"*WESTOPFORNOONE*"` at `@0..16`).
pub(crate) fn verify_iwlwifi_ucode(prefix: &[u8], size: u64) -> Option<Label> {
    const IWL_TLV_MAGIC: &[u8; 8] = b"\0\0\0\0IWL\n"; // @0..8: iwl_tlv_ucode_header zero + "IWL\n"
    const IWL_WIN_MAGIC: &[u8; 16] = b"*WESTOPFORNOONE*"; // @0..16: Windows Intel Wi-Fi firmware magic
    const PNVM_SKU_TLV: &[u8; 8] = bytes!(le32_bytes!(0x40), le32_bytes!(12)); // @0..8: iwl_ucode_tlv PNVM_SKU (type=0x40, len=12)
    const PNVM_HW_TYPE_TLV: &[u8; 8] = bytes!(le32_bytes!(0x3A), le32_bytes!(12)); // @20..28: iwl_ucode_tlv HW_TYPE (type=0x3A, len=12)
    const PNVM_VERSION_TLV: &[u8; 8] = bytes!(le32_bytes!(0x3E), le32_bytes!(4)); // @0..8: iwl_ucode_tlv PNVM_VERSION (type=0x3E, len=4)
    const ARC_INSN_SIG: &[u8; 4] = hex2bin!("20 20 80 0F"); // ARC/Tensilica runtime text start

    require!(size >= MIN_IWLWIFI_UCODE_SIZE, prefix.len() >= 32);
    if starts(prefix, IWL_TLV_MAGIC) {
        return Some(Label::IwlwifiUcode);
    }
    if starts(prefix, IWL_WIN_MAGIC) {
        let tlv_type = le32(prefix, 16)?;
        let tlv_len = le32(prefix, 20)?;
        require!(
            (1..=0x100).contains(&tlv_type),
            tlv_len > 0 && 24 + (tlv_len as u64) <= size,
            has_non_ascii_text_byte(prefix.get(16..32)?)
        );
        return Some(Label::IwlwifiUcode);
    }
    if (starts(prefix, PNVM_SKU_TLV) && eq_at(prefix, 20, PNVM_HW_TYPE_TLV))
        || (starts(prefix, PNVM_VERSION_TLV) && eq_at(prefix, 12, PNVM_SKU_TLV))
    {
        return Some(Label::IwlwifiUcode);
    }
    // Legacy iwl_ucode_header v1 (24-byte header) or v2 (28-byte header)
    let sum_v1 = (1..6).try_fold(0u64, |acc, i| Some(acc + le32(prefix, i * 4)? as u64))?;
    if 24 + sum_v1 == size && eq_at(prefix, 24, ARC_INSN_SIG) {
        return Some(Label::IwlwifiUcode);
    }
    let sum_v2 = (2..7).try_fold(0u64, |acc, i| Some(acc + le32(prefix, i * 4)? as u64))?;
    if le32(prefix, 4) == Some(0) && 28 + sum_v2 == size && eq_at(prefix, 28, ARC_INSN_SIG) {
        return Some(Label::IwlwifiUcode);
    }
    None
}

/// Oracle / Sun Java Web Start Deployment Cache Index (`.idx`, versions 602..=606).
pub(crate) fn verify_java_webstart_idx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JAVA_WEBSTART_IDX_SIZE,
        prefix.len() >= 32,
        u8_at(prefix, 0)? <= 1,
        u8_at(prefix, 1)? <= 1,
        (602..=606).contains(&be32(prefix, 2)?)
    );
    Some(Label::JavaWebstartIdx)
}

/// JBOOT SCH2 Second-Stage Kernel Header Image (`.bin`).
pub(crate) fn verify_jboot_sch2(prefix: &[u8], size: u64) -> Option<Label> {
    const SCH2_MAGIC: &[u8; 4] = le32_bytes!(0x0203_2124); // @0: 0x02032124 LE ("$!\x03\x02")
    const ENTRY_OFF: usize = 12; // @12: u32 LE kernel entry point / CRC (!= 0)
    const LEN_OFF: usize = 16; // @16: u32 LE kernel payload size (> 0)
    require!(
        size >= MIN_JBOOT_SCH2_SIZE,
        prefix.len() >= 32,
        starts(prefix, SCH2_MAGIC),
        le32(prefix, ENTRY_OFF)? != 0,
        le32(prefix, LEN_OFF)? > 0
    );
    Some(Label::JbootSch2)
}

/// Decodes a Joda-Time variable-length millisecond value (`DateTimeZoneBuilder.readMillis`).
fn read_joda_millis(buf: &[u8], pos: &mut usize) -> Option<i64> {
    const THIRTY_MIN_MS: i64 = 30 * 60 * 1_000;
    const ONE_MIN_MS: i64 = 60 * 1_000;
    let v = u8_at(buf, *pos)?;
    *pos += 1;
    match v >> 6 {
        0 => {
            // Form 00: signed 6-bit value in 30-minute units (±16h, 1 byte total).
            let units = ((v as i8) << 2 >> 2) as i64;
            Some(units * THIRTY_MIN_MS)
        }
        1 => {
            // Form 01: signed 30-bit value in minute units (4 bytes total).
            let b1 = u8_at(buf, *pos)? as u32;
            let b2 = u8_at(buf, *pos + 1)? as u32;
            let b3 = u8_at(buf, *pos + 2)? as u32;
            *pos += 3;
            let hi = (((v as i8) << 2 >> 2) as i32) << 24;
            let mins = (hi | ((b1 << 16) | (b2 << 8) | b3) as i32) as i64;
            Some(mins * ONE_MIN_MS)
        }
        2 => {
            // Form 10: signed 38-bit value in second units (5 bytes total).
            let lo = be32(buf, *pos)? as u64;
            *pos += 4;
            let hi = (((v as i8) << 2 >> 2) as i64) << 32;
            let secs = hi | (lo as i64);
            Some(secs * 1_000)
        }
        _ => {
            // Form 11: full 64-bit signed milliseconds (9 bytes total).
            // DateTimeZoneBuilder.writeMillis emits 0xFF (< 0) or 0xC0 (>= 0).
            require!(v == 0xFF || v == 0xC0);
            let ms = be64(buf, *pos)? as i64;
            *pos += 8;
            require!((v == 0xFF) == (ms < 0));
            Some(ms)
        }
    }
}

/// Reads a Java `DataOutput.writeUTF` printable ASCII zone abbreviation from a Joda-Time zone file.
fn read_joda_utf<'a>(buf: &'a [u8], pos: &mut usize) -> Option<&'a [u8]> {
    let slen = be16(buf, *pos)? as usize;
    *pos += 2;
    require!((1..=64).contains(&slen));
    let s = get_slice(buf, *pos, *pos + slen)?;
    require!(s.iter().all(|&b| (0x20..=0x7E).contains(&b)));
    *pos += slen;
    Some(s)
}

/// Joda-Time Compiled Time Zone (`DateTimeZoneBuilder.PrecalculatedZone` / `CachedDateTimeZone`).
///
/// Layout (`DateTimeZoneBuilder.java`): `@0` type tag (`'C'` = `CachedDateTimeZone`, `'P'` =
/// `PrecalculatedZone`), `@1..3` `u16 BE poolSize` (with `@1 == 0x00` for real tzdata pools
/// `< 256`, which selects 1-byte `nameKey` indices), `poolSize` Java-UTF strings,
/// `i32 BE numTransitions`, `numTransitions` `(millis, wallOffset, stdOffset, nameKeyIdx)`
/// records, and a trailing `u8 hasTail` flag + optional `DSTZone` rule pair.
pub(crate) fn verify_joda_timezone_info(prefix: &[u8], size: u64) -> Option<Label> {
    const MAX_TZ_OFFSET_MS: i64 = 24 * 60 * 60 * 1_000;
    const MAX_TRANSITIONS: usize = 4_096;

    require!(size >= MIN_JODA_TZ_SIZE, prefix.len() >= 16);
    let tag = u8_at(prefix, 0)?;
    require!(matches!(tag, b'C' | b'P'), u8_at(prefix, 1)? == 0);
    // Byte @1 == 0x00 pins pool_size to 1..=255, which guarantees 1-byte nameKey indices below.
    let pool_size = u8_at(prefix, 2)? as usize;
    require!(pool_size >= 1);

    let mut pos = 3;
    for _ in 0..pool_size {
        read_joda_utf(prefix, &mut pos)?;
    }

    let num_transitions = be32(prefix, pos)? as usize;
    pos += 4;
    require!((1..=MAX_TRANSITIONS).contains(&num_transitions));

    let full_in_prefix = size <= prefix.len() as u64;
    let mut prev_t: Option<i64> = None;
    let mut checked_transitions = 0usize;
    for _ in 0..num_transitions {
        // For files larger than prefix (none in standard tzdata), stop before running off prefix.
        if !full_in_prefix && pos + 32 > prefix.len() {
            break;
        }
        let t = read_joda_millis(prefix, &mut pos)?;
        let wall = read_joda_millis(prefix, &mut pos)?;
        let std_off = read_joda_millis(prefix, &mut pos)?;
        let name_idx = u8_at(prefix, pos)? as usize;
        pos += 1;
        require!(
            prev_t.is_none_or(|p| t > p),
            (-MAX_TZ_OFFSET_MS..=MAX_TZ_OFFSET_MS).contains(&wall),
            (-MAX_TZ_OFFSET_MS..=MAX_TZ_OFFSET_MS).contains(&std_off),
            name_idx < pool_size
        );
        prev_t = Some(t);
        checked_transitions += 1;
    }
    require!(checked_transitions >= 1);

    if full_in_prefix {
        let has_tail = u8_at(prefix, pos)?;
        pos += 1;
        require!(has_tail <= 1);
        if has_tail == 1 {
            // DSTZone: stdOffset + start Recurrence + end Recurrence.
            let std_off = read_joda_millis(prefix, &mut pos)?;
            require!((-MAX_TZ_OFFSET_MS..=MAX_TZ_OFFSET_MS).contains(&std_off));
            for _ in 0..2 {
                // OfYear: mode ('u'|'w'|'s'), month (1..=12), dayOfMonth (-31..=31),
                // dayOfWeek (0..=7), advanceDayOfWeek (0|1), millisOfDay (0..=24h).
                let mode = u8_at(prefix, pos)?;
                let month = u8_at(prefix, pos + 1)?;
                let dom = u8_at(prefix, pos + 2)? as i8;
                let dow = u8_at(prefix, pos + 3)?;
                let advance = u8_at(prefix, pos + 4)?;
                pos += 5;
                require!(
                    matches!(mode, b'u' | b'w' | b's'),
                    (1..=12).contains(&month),
                    (-31..=31).contains(&dom),
                    dow <= 7,
                    advance <= 1
                );
                let millis_of_day = read_joda_millis(prefix, &mut pos)?;
                require!((0..=MAX_TZ_OFFSET_MS).contains(&millis_of_day));
                read_joda_utf(prefix, &mut pos)?;
                let save_ms = read_joda_millis(prefix, &mut pos)?;
                require!((-MAX_TZ_OFFSET_MS..=MAX_TZ_OFFSET_MS).contains(&save_ms));
            }
        }
        require!(pos as u64 == size);
    }

    Some(Label::JodaTz)
}

/// JUCE PropertySet Binary Configuration (.settings, 2004) Raw Material Software.
pub(crate) fn verify_juce_prop(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JUCE_PROP_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"PROP\x01\0\0\0"),
        u8_at(prefix, 8)?.is_ascii_alphabetic(),
        prefix.get(9..24)?.contains(&0)
    );
    require!(has_non_ascii_text_byte(prefix.get(..24)?));
    Some(Label::JuceProp)
}

/// Keyence LJ-X8000 / LJ-V7000 Series Laser Profiler Settings file (`.ljhp`).
pub(crate) fn verify_keyence_ljx_ljhp(prefix: &[u8], size: u64) -> Option<Label> {
    const KEYENCE_LJX8_PREFIX: &[u8; 5] = b"LJ-X8"; // @0..5
    const KEYENCE_LJV7_PREFIX: &[u8; 5] = b"LJ-V7"; // @0..5

    require!(
        size >= MIN_KEYENCE_LJX_LJHP_SIZE,
        prefix.len() >= 32,
        starts(prefix, KEYENCE_LJX8_PREFIX) || starts(prefix, KEYENCE_LJV7_PREFIX),
        get_slice(prefix, 5, 8)?.iter().all(u8::is_ascii_digit), // @5..8: head model suffix (e.g. "080", "200", "400")
        get_slice(prefix, 8, 32)? == [0u8; 24]
    );
    Some(Label::KeyenceLjLjhp)
}

/// Eastman Kodak DC20 Digital Camera Preset (`.pst`, 1996) Kodak.
pub(crate) fn verify_kodak_dc20_preset_pst(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KODAK_DC20_PRESET_PST_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"TSRP\x01\x00\x00\x00"),
        u8_at(prefix, 8)?.is_ascii_graphic()
    );
    Some(Label::KodakDc20PresetPst)
}

/// KONTEXT DOS Word Processor Printer/Keyboard Driver (.lat).
pub(crate) fn verify_kontext_lat(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KONTEXT_LAT_SIZE,
        prefix.len() >= 48,
        get_slice(prefix, 0, 22)?
            == hex2bin!("0D 0A 20 20 20 20 20 20 20 20 20 20 20 20 20 20 20 20 20 20 20 20"),
        get_slice(prefix, 31, 32)? == hex2bin!("20")
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::KontextLat)
}

/// Lattice Semiconductor FPGA Configuration Bitstream (`.bit`).
pub(crate) fn verify_lattice_bitstream(prefix: &[u8], size: u64) -> Option<Label> {
    const LATTICE_PREFIX: &[u8; 9] = bytes!(hex2bin!("FF 00"), b"Lattice"); // @0..9: 0xFF 0x00 + "Lattice" comment block
    require!(
        size >= MIN_LATTICE_BITSTREAM_SIZE,
        prefix.len() >= 48,
        starts(prefix, LATTICE_PREFIX),
        eq_at(prefix, 9, b" Semiconductor Corporation Bitstream\0")
            || eq_at(prefix, 9, b"\0iCEcube2 ")
    );
    Some(Label::LatticeBitstream)
}

/// Leica APS-C ASCII-headered camera firmware updates (`X1`, `X2`, `X Vario`, `T 701`,
/// `TL`, `TL2`): `"LENGTH=<bytes> <model> VER=<ver> [DVR=<dvr>] SUM=<sum> ...\r\n"` space-padded to
/// 96 (`0x60`) or 256 (`0x100`) bytes.
pub(crate) fn verify_leica_apsc_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_LEICA_APSC_FW_SIZE, prefix.len() >= 96, starts(prefix, b"LENGTH="));
    let win = get_slice(prefix, 0, prefix.len().min(256))?;
    let crlf = win.windows(2).position(|w| w == b"\r\n")?;
    require!((24..160).contains(&crlf));
    let hdr_str = std::str::from_utf8(&win[..crlf]).ok()?;
    let mut length_val: Option<u64> = None;
    let mut has_ver = false;
    let mut sum_val: Option<u64> = None;
    let mut hw_model: Option<&str> = None;
    for tok in hdr_str.split_ascii_whitespace() {
        if let Some(v) = tok.strip_prefix("LENGTH=") {
            length_val = v.parse::<u64>().ok().filter(|&n| n >= 64);
        } else if tok.starts_with("VER=") && tok.len() > 4 {
            has_ver = true;
        } else if let Some(v) = tok.strip_prefix("SUM=") {
            sum_val = v.parse::<u64>().ok().filter(|&n| n > 0);
        } else if tok.starts_with("DVR=") {
            continue;
        } else if hw_model.is_none() && !tok.contains('=') && tok.len() >= 3 {
            hw_model = Some(tok);
        }
    }
    length_val?;
    sum_val?;
    require!(has_ver, hw_model.is_some());
    // Space padding immediately after `\r\n` up to at least offset 96 (`0x60`, X1/X2/XV) or 256 (`0x100`, T/TL/TL2)
    // `crlf` may be anywhere in 24..160, so the header line must itself end before the 96-byte pad boundary.
    require!(crlf + 2 <= 96, win[crlf + 2..96].iter().all(|&b| b == b' '));
    Some(Label::LeicaApscFirmware)
}

/// Leica Camera Maestro / Fujitsu-Socionext firmware containers (`.fw` / `.s2`):
/// - `1E 1C AF 2E`: 48B BE header (`M 240`, `M 246`, `M 262`, `S 006`, `unk2 = 0x07E1EA5E BE`, `!magic` at `@44..48`)
/// - `3E AF 1C 1E`: 56B LE header + `"COMP"` stream (`S 007`, `S3`, `M10`, `unk2 = 0x07E1EA5E LE`, `!magic` at `@52..56`)
/// - `1E 1C AF 1E`: 300B (`0x12C`) BE section table (`S2`, CRC32-verified sections at `@0x24`).
pub(crate) fn verify_leica_camera_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const LEICA_UNK2: u32 = 0x07E1_EA5E;
    const BE48_INV_MAGIC: u32 = !0x1E1C_AF2E;
    const LE56_INV_MAGIC: u32 = !0x1E1C_AF3E;

    require!(size >= MIN_LEICA_CAMERA_FW_SIZE && prefix.len() >= 48);
    if starts(prefix, LEICA_FW_BE48_MAGIC) {
        let unk2 = be32(prefix, 8)?;
        let pk_sz = be32(prefix, 12)?;
        let unpk_sz = be32(prefix, 16)?;
        let blocks = be32(prefix, 36)?;
        let body_off = be32(prefix, 40)?;
        let inv_magic = be32(prefix, 44)?;
        require!(
            unk2 == LEICA_UNK2,
            inv_magic == BE48_INV_MAGIC,
            pk_sz > 0,
            unpk_sz > 0,
            blocks >= 1,
            body_off == 48
        );
        return Some(Label::LeicaFirmware);
    }

    if starts(prefix, LEICA_FW_LE56_MAGIC) {
        require!(size >= 60 && prefix.len() >= 60);
        let unk2 = le32(prefix, 8)?;
        let pk_sz = le32(prefix, 12)?;
        let unpk_sz = le32(prefix, 16)?;
        let blocks = le32(prefix, 36)?;
        let body_off = le32(prefix, 40)?;
        let inv_magic = le32(prefix, 52)?;
        require!(
            unk2 == LEICA_UNK2,
            inv_magic == LE56_INV_MAGIC,
            pk_sz > 0,
            unpk_sz > 0,
            blocks >= 1,
            body_off == 56,
            eq_at(prefix, 56, b"COMP")
        );
        return Some(Label::LeicaFirmware);
    }

    if starts(prefix, LEICA_FW_S2_MAGIC) {
        const S2_HDR_SIZE: usize = 0x12C;
        require!(size >= S2_HDR_SIZE as u64 && prefix.len() >= S2_HDR_SIZE);
        let hdr_id = be32(prefix, 4)?;
        let flags = be32(prefix, 12)?;
        let rec0_off = be32(prefix, 0x2C)?;
        let rec0_len = be32(prefix, 0x30)?;
        let rec0_crc = be32(prefix, 0x34)?;
        require!(
            hdr_id > 0 && hdr_id <= 0xFFFF,
            flags >= 1,
            rec0_off == 0,
            rec0_len > 0 && rec0_crc != 0
        );
        return Some(Label::LeicaFirmware);
    }

    None
}

/// Leica Digital Modul-R XOR-`0x35` Firmware Image (`.cim`).
pub(crate) fn verify_leica_dmr_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const XOR_KEY: u8 = 0x35;
    require!(
        size >= MIN_LEICA_DMR_SIZE,
        prefix.len() >= MIN_LEICA_DMR_SIZE as usize,
        eq_at(prefix, 8, LEICA_DMR_XOR35_BANNER)
    );
    let mut hdr = [0u8; 96];
    for (dst, &src) in hdr.iter_mut().zip(get_slice(prefix, 0, 96)?.iter()) {
        *dst = src ^ XOR_KEY;
    }
    require!(hdr[28..40] == [0u8; 12]);
    let format_ver = u32::from_le_bytes([hdr[40], hdr[41], hdr[42], hdr[43]]);
    let table_slots = u32::from_le_bytes([hdr[44], hdr[45], hdr[46], hdr[47]]);
    let active_secs = u32::from_le_bytes([hdr[48], hdr[49], hdr[50], hdr[51]]);
    let payload_sz = u32::from_le_bytes([hdr[52], hdr[53], hdr[54], hdr[55]]);
    require!(
        format_ver == 1,
        (1..=32).contains(&table_slots),
        (1..=table_slots).contains(&active_secs),
        payload_sz >= 64
    );
    Some(Label::LeicaDmrFirmware)
}

/// Extracted Leica Camera Lens Database (`"LEICA-LENS-FILE-SOF:"`, S / T / TL / TL2).
pub(crate) fn verify_leica_lens_db(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 20] = b"LEICA-LENS-FILE-SOF:";
    require!(size >= MIN_LEICA_LENS_DB_SIZE, prefix.len() >= 40, starts(prefix, MAGIC));
    let ver = be16(prefix, 20)?;
    require!((1..=16).contains(&ver), eq_at(prefix, 22, b"LEICA_"));
    Some(Label::LeicaLensDb)
}

/// Lenovo Smart Engine XOR-Obfuscated Installer Log (.logdat).
pub(crate) fn verify_lenovo_logdat(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LENOVO_LOGDAT_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 4)? == hex2bin!("4C 00 00 00"),
        get_slice(prefix, 6, 7)? == hex2bin!("D2"),
        get_slice(prefix, 9, 10)? == hex2bin!("DF"),
        get_slice(prefix, 12, 13)? == hex2bin!("C5"),
        get_slice(prefix, 15, 16)? == hex2bin!("C5"),
        get_slice(prefix, 18, 19)? == hex2bin!("C5"),
        get_slice(prefix, 22, 25)? == hex2bin!("DF C5 DF")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::LenovoLogdat)
}

/// Linux `kmod` Binary Module Index (`modules.dep.bin` / `modules.alias.bin` / `modules.symbols.bin`, `0xB007F457` BE).
pub(crate) fn verify_linux_kmod_index(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LINUX_KMOD_INDEX_SIZE,
        prefix.len() >= 12,
        starts(prefix, hex2bin!("B0 07 F4 57")),
        (1..=2).contains(&be16(prefix, 4)?),
        be16(prefix, 6)? <= 16
    );
    let root = be32(prefix, 8)?;
    let root_off = (root & 0x0FFF_FFFF) as u64;
    require!((12..=size).contains(&root_off), (root & 0xE000_0000) != 0);
    Some(Label::LinuxKmodIndex)
}

/// Linux Kernel perf.data Performance Profile (PERFILE2).
pub(crate) fn verify_linux_perf_data(prefix: &[u8], size: u64) -> Option<Label> {
    let is_v2 = size >= MIN_LINUX_PERF_DATA_SIZE
        && prefix.len() >= 24
        && (starts(prefix, b"PERFILE2") || starts(prefix, b"2ELIFREP"))
        && (le64(prefix, 8) == Some(104) || be64(prefix, 8) == Some(104));
    let is_pipe = size >= 16
        && prefix.len() >= 16
        && (starts(prefix, b"PERFILE2") || starts(prefix, b"2ELIFREP"))
        && (le64(prefix, 8) == Some(16) || be64(prefix, 8) == Some(16));
    let is_v1 = size >= 96
        && prefix.len() >= 24
        && starts(prefix, b"PERFFILE")
        && (64..=128).contains(&le64(prefix, 8).unwrap_or(0));
    require!(is_v2 || is_pipe || is_v1);
    Some(Label::LinuxPerf)
}

/// LSSTAT License/System Status Record (.stat).
pub(crate) fn verify_lsstat_stat(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LSSTAT_STAT_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"LSSTAT\x01\x01\0\0\0"),
        be32(prefix, 20) == Some(1)
    );
    Some(Label::LsstatStat)
}

/// Apple macOS FSEvents file system event log (`1SLD` / `2SLD` / `3SLD`, 2007 Apple).
pub(crate) fn verify_macos_fsevents(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MACOS_FSEVENTS_SIZE,
        prefix.len() >= 24,
        matches!(get_slice(prefix, 0, 4)?, b"1SLD" | b"2SLD" | b"3SLD")
    );
    let stream_len = le32(prefix, 8)?;
    require!(
        (13..=16_777_215).contains(&stream_len),
        stream_len as u64 <= size,
        u8_at(prefix, 12).unwrap_or(0) == 0 || u8_at(prefix, 12).unwrap_or(0).is_ascii_graphic()
    );
    Some(Label::MacosFsevents)
}

/// Apple macOS `utmpx` user accounting log (`"utmpx-1.00\0"`, 2005 Apple).
pub(crate) fn verify_macos_utmpx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MACOS_UTMPX_SIZE,
        size.is_multiple_of(628),
        prefix.len() >= 304,
        starts(prefix, b"utmpx-1.00\0"),
        le16(prefix, 296) == Some(10)
    );
    Some(Label::MacosUtmpx)
}

/// ME 0x0FA0 Padded Record (.mex).
pub(crate) fn verify_macro_express_mex(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MACRO_EXPRESS_MEX_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"ME\xa0\x0f\0\0<\0"),
        get_slice(prefix, 8, 48)?.iter().all(|&b| b == 0)
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::MacroExpressMex)
}

/// Marvell OCTEON TX2 / CN10K Cryptographic Acceleration Engine Microcode Image (`"CPT01"` at `@0..5` or `"OCPT-0"` at `@4..10`).
pub(crate) fn verify_marvell_cpt_microcode(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MARVELL_CPT_UCODE_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"CPT01") || eq_at(prefix, 4, b"OCPT-0"),
        get_slice(prefix, 4, 32)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b))
    );
    Some(Label::MarvellCptUcode)
}

/// Connectivity Standards Alliance Matter OTA Software Update Image (`.ota`).
pub(crate) fn verify_matter_ota(prefix: &[u8], size: u64) -> Option<Label> {
    const MATTER_MAGIC: &[u8; 4] = le32_bytes!(0x1BEE_F11E); // @0: 0x1BEEF11E LE (1E F1 EE 1B)
    const TOTAL_SIZE_OFF: usize = 4; // @4: u64 LE total OTA file size (>= 32)
    const HDR_SIZE_OFF: usize = 12; // @12: u32 LE TLV header size (16..=4096)
    const TLV_TAG_OFF: usize = 16; // @16: u8 Matter TLV anonymous structure tag (0x15)
    require!(
        size >= MIN_MATTER_OTA_SIZE,
        prefix.len() >= 20,
        starts(prefix, MATTER_MAGIC),
        u8_at(prefix, TLV_TAG_OFF) == Some(0x15)
    );
    let total_sz = le64(prefix, TOTAL_SIZE_OFF)?;
    let hdr_sz = le32(prefix, HDR_SIZE_OFF)?;
    require!(total_sz >= 32, (16..=4096).contains(&hdr_sz));
    Some(Label::MatterOta)
}

/// MCUboot / Apache Mynewt Secure Bootloader Firmware Image (`.bin`).
pub(crate) fn verify_mcuboot_image(prefix: &[u8], size: u64) -> Option<Label> {
    const MCUBOOT_MAGIC: &[u8; 4] = le32_bytes!(0x96F3_B83D); // @0: IMAGE_MAGIC 0x96F3B83D LE
    const HDR_SIZE_OFF: usize = 8; // @8: u16 LE ih_hdr_size (32..=4096)
    const IMG_SIZE_OFF: usize = 12; // @12: u32 LE ih_img_size (> 0)
    require!(
        size >= MIN_MCUBOOT_IMAGE_SIZE,
        prefix.len() >= 32,
        starts(prefix, MCUBOOT_MAGIC),
        (32..=4096).contains(&le16(prefix, HDR_SIZE_OFF)?),
        le32(prefix, IMG_SIZE_OFF)? > 0
    );
    Some(Label::McubootImg)
}

/// MediaTek BootROM Layout Descriptor Header (`.bin`).
pub(crate) fn verify_mediatek_brlyt(prefix: &[u8], size: u64) -> Option<Label> {
    const BRLYT_MAGIC: &[u8; 12] = bytes!(b"BRLYT\0\0\0", le32_bytes!(1)); // @0: "BRLYT\0\0\0" + version 1 LE
    const BBBB_OFF: usize = 20; // @20..24: "BBBB" (0x42424242 BootROM descriptor tag)
    require!(
        size >= MIN_MEDIATEK_BRLYT_SIZE,
        prefix.len() >= 32,
        starts(prefix, BRLYT_MAGIC),
        eq_at(prefix, BBBB_OFF, b"BBBB")
    );
    Some(Label::MediatekBrlyt)
}

/// MediaTek / ALPS Bluetooth Combo Controller Firmware Image (`YYYYMMDDHHMMSS` at `@0..14` + `"ALPS"` at `@16..20`).
pub(crate) fn verify_mediatek_bt_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MEDIATEK_BT_FW_SIZE,
        prefix.len() >= 32,
        eq_at(prefix, 16, b"ALPS"),
        get_slice(prefix, 0, 14)?.iter().all(|b| b.is_ascii_digit())
    );
    Some(Label::MediatekBtFw)
}

/// MediaTek Partition Image (`0x58881688`).
pub(crate) fn verify_mediatek_image(prefix: &[u8], size: u64) -> Option<Label> {
    const MTK_PART_MAGIC: &[u8; 4] = le32_bytes!(0x5888_1688); // @0..4: PART_MAGIC 0x58881688 LE
    const DATA_SIZE_OFF: usize = 4; // @4..8: u32 LE payload size (> 0)
    const NAME_OFF: usize = 8; // @8..40: 32-byte NUL-padded ASCII partition name
    const EXT_MAGIC_OFF: usize = 48; // @48..52: PART_EXT_MAGIC 0x58891689 LE or 0
    require!(
        size >= MIN_MEDIATEK_IMAGE_SIZE,
        prefix.len() >= 64,
        starts(prefix, MTK_PART_MAGIC),
        le32(prefix, DATA_SIZE_OFF)? > 0,
        u8_at(prefix, NAME_OFF)?.is_ascii_graphic(),
        get_slice(prefix, NAME_OFF, NAME_OFF + 32)?
            .iter()
            .all(|&b| b == 0 || (0x20..=0x7E).contains(&b)),
        matches!(le32(prefix, EXT_MAGIC_OFF), Some(0 | 0x5889_1689))
    );
    Some(Label::MediatekImage)
}

/// MediaTek BootROM Preloader Image (`EMMC_BOOT` / `SF_BOOT`).
pub(crate) fn verify_mediatek_preloader(prefix: &[u8], size: u64) -> Option<Label> {
    const EMMC_BOOT_MAGIC: &[u8; 10] = b"EMMC_BOOT\0"; // @0..10: EMMC BootROM header identifier
    const SF_BOOT_MAGIC: &[u8; 8] = b"SF_BOOT\0"; // @0..8: Serial Flash BootROM header identifier
    const VER_OFF: usize = 12; // @12..16: u32 LE header version (1)
    const RW_UNIT_OFF: usize = 16; // @16..20: u32 LE device sector size (0x200 = 512)
    require!(
        size >= MIN_MEDIATEK_PRELOADER_SIZE,
        prefix.len() >= 32,
        starts(prefix, EMMC_BOOT_MAGIC) || starts(prefix, SF_BOOT_MAGIC),
        le32(prefix, VER_OFF) == Some(1),
        le32(prefix, RW_UNIT_OFF) == Some(0x200)
    );
    Some(Label::MediatekPreloader)
}

/// MediaTek Signed Partition Image (`"BFBF"` or `"SSSS"` with `0x58881688` LE, `"ANDROID!"` inner header, or `"SSSS"` v1-v4 `secExtMagic`).
pub(crate) fn verify_mediatek_signed_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MEDIATEK_SIGNED_IMAGE_SIZE,
        prefix.len() >= 72,
        starts(prefix, b"BFBF") || starts(prefix, b"SSSS"),
        le32(prefix, 64) == Some(0x5888_1688)
            || eq_at(prefix, 64, b"ANDROID!")
            || (prefix.len() >= 0x4048
                && (le32(prefix, 0x4040) == Some(0x5888_1688)
                    || eq_at(prefix, 0x4040, b"ANDROID!")
                    || eq_at(prefix, 0x4000, b"SSSS")))
            || (starts(prefix, b"SSSS")
                && (le32(prefix, 36) == Some(0x7A79_7A79)
                    || matches!(le32(prefix, 48), Some(0x7A79_7A79 | 0x7B79_7B79)))
                && le32(prefix, 60).unwrap_or(0) > 0)
    );
    Some(Label::MediatekSignedImg)
}

/// Project Slippi Super Smash Bros. Melee Replay (`.slp`, UBJSON container).
///
/// Header layout:
/// - `@0..11`: `"{U\x03raw[$U#l"` (UBJSON object with `"raw"` byte-array key and `int32` length tag)
/// - `@11..15`: `u32 BE` `raw_payload_length`
pub(crate) fn verify_melee_slippi(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 11] = bytes!(b"{U", &[3], b"raw[$U#l"); // @0..11: UBJSON raw payload envelope

    require!(size >= MIN_MELEE_SLIPPI_SIZE, prefix.len() >= 15, starts(prefix, MAGIC));
    Some(Label::MeleeSlippi)
}

/// Mercedes-Benz Caesar Flash File (`.cff`).
pub(crate) fn verify_mercedes_cff(prefix: &[u8], size: u64) -> Option<Label> {
    const CFF_MAGIC: &[u8; 23] = b"CFF-TRANSLATOR-VERSION:"; // @0..23: Caesar Flash File header key
    require!(
        size >= MIN_MERCEDES_CFF_SIZE,
        prefix.len() >= 48,
        starts(prefix, CFF_MAGIC),
        scan_range(prefix, 23, 64, b"\nDATE:")
    );
    Some(Label::MercedesCff)
}

/// Micro House Technical Library Pro-Sec Hardware Database (`.bin`, 1994) Micro House.
pub(crate) fn verify_microhouse_prosec_bin(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MICROHOUSE_PROSEC_BIN_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"#Pro-Sec#\x10\x00\x04")
    );
    Some(Label::MicrohouseProsecBin)
}

/// MikroTik RouterOS NPK Firmware Package (`.npk`).
pub(crate) fn verify_mikrotik_npk(prefix: &[u8], size: u64) -> Option<Label> {
    const NPK_MAGIC: &[u8; 4] = le32_bytes!(0xBAD0_F11E); // @0: 0xBAD0F11E LE (1E F1 D0 BA)
    const TOTAL_LEN_OFF: usize = 4; // @4: u32 LE remaining file length (> 28)
    const PART_ID_OFF: usize = 8; // @8: u16 LE first TLV part ID (1..=64)
    const PART_LEN_OFF: usize = 14; // @14: u32 LE first part size (> 0)
    require!(size >= MIN_MIKROTIK_NPK_SIZE, prefix.len() >= 36, starts(prefix, NPK_MAGIC));
    let total_len = le32(prefix, TOTAL_LEN_OFF)?;
    let part_id = le16(prefix, PART_ID_OFF)?;
    let part_len = le32(prefix, PART_LEN_OFF)?;
    require!(total_len > 28, (1..=64).contains(&part_id), part_len > 0);
    Some(Label::MikrotikNpk)
}

/// Motorola `MotoLogo` Boot Logo Container (`.bin`).
pub(crate) fn verify_motorola_motologo(prefix: &[u8], size: u64) -> Option<Label> {
    const MOTOLOGO_MAGIC: &[u8; 9] = b"MotoLogo\0"; // @0: 9-byte NUL-terminated magic
    const HDR_END_OFF: usize = 9; // @9: u32 LE directory end offset (>= 45)
    const FIRST_NAME_OFF: usize = 13; // @13: 24-byte NUL-padded ASCII logo name (e.g. "logo_boot")
    require!(
        size >= MIN_MOTOROLA_MOTOLOGO_SIZE,
        prefix.len() >= 45,
        starts(prefix, MOTOLOGO_MAGIC),
        le32(prefix, HDR_END_OFF)? >= 45,
        u8_at(prefix, FIRST_NAME_OFF)?.is_ascii_graphic(),
        get_slice(prefix, FIRST_NAME_OFF, FIRST_NAME_OFF + 8)?
            .iter()
            .all(|&b| b == 0 || b.is_ascii_graphic())
    );
    Some(Label::MotorolaMotologo)
}

/// Motorola PIV Signed Partition Image (`"MOTO"` + `"MOT_PIV_FULL256\0"`).
pub(crate) fn verify_motorola_piv_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MOTOROLA_PIV_IMAGE_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"MOTO"),
        eq_at(prefix, 8, b"MOT_PIV_FULL256\0")
    );
    Some(Label::MotorolaPivImg)
}

/// Moxa Industrial Device Firmware Image (`.frm`).
pub(crate) fn verify_moxa_frm(prefix: &[u8], size: u64) -> Option<Label> {
    const MOXA_MAGIC: &[u8; 8] = bytes!(b"*FRM", be32_bytes!(1)); // @0: "*FRM" + 0x00000001 BE
    const FILE_SIZE_OFF: usize = 8; // @8: u32 LE total file size (>= 96)
    const HDR_SIZE_OFF: usize = 12; // @12: u16 LE header length (0x60 = 96)
    const SEC_COUNT_OFF: usize = 14; // @14: u16 LE section count (1..=64)
    require!(size >= MIN_MOXA_FRM_SIZE, prefix.len() >= 32, starts(prefix, MOXA_MAGIC));
    let file_sz = le32(prefix, FILE_SIZE_OFF)?;
    let hdr_sz = le16(prefix, HDR_SIZE_OFF)?;
    let sec_cnt = le16(prefix, SEC_COUNT_OFF)?;
    require!(file_sz >= 96, hdr_sz == 0x60, (1..=64).contains(&sec_cnt));
    Some(Label::MoxaFrm)
}

/// Microsoft Setup Font/Typeface Definition (`SETUP.TDF`, 1991) Microsoft.
pub(crate) fn verify_ms_setup_tdf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MS_SETUP_TDF_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"QQ\x00\x00\x81\x81\x00\x00")
    );
    Some(Label::MsSetupTdf)
}

/// Microsoft Windows 3.1 PostScript Printer Description (`.wpd`, 1992) Microsoft.
pub(crate) fn verify_ms_windows_postscript_wpd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MS_WINDOWS_POSTSCRIPT_WPD_SIZE,
        prefix.len() >= 48,
        starts(prefix, &[0x12, 0x00, 0x00, 0x00]),
        le16(prefix, 4)? > 0,
        u8_at(prefix, 20)?.is_ascii_alphabetic()
    );
    let nul_pos = get_slice(prefix, 20, 48)?.iter().position(|&b| b == 0)?;
    require!(
        nul_pos >= 4,
        get_slice(prefix, 20, 20 + nul_pos)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::MsWindowsPostscriptWpd)
}

/// MS-DOS / PC DOS Country Information Table (`COUNTRY.SYS`, 1987) Microsoft / IBM.
pub(crate) fn verify_msdos_country_sys(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MSDOS_COUNTRY_SYS_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"\xFFCOUNTRY\x00\x00\x00\x00")
    );
    Some(Label::MsdosCountrySys)
}

/// MS-DOS / PC DOS Keyboard Layout Table (`KEYBOARD.SYS`, 1987) Microsoft / IBM.
pub(crate) fn verify_msdos_keyboard_sys(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MSDOS_KEYBOARD_SYS_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"\xFFKEYB   \x00\x00\x00\x00")
    );
    Some(Label::MsdosKeyboardSys)
}

/// Microsoft Internet Explorer WinINet URL Cache (`"Client UrlCache MMF Ver "`).
pub(crate) fn verify_msie_cache_index_dat(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MSIE_INDEX_DAT_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"Client UrlCache MMF Ver "),
        u8_at(prefix, 24)?.is_ascii_digit(),
        prefix.get(25..32)?.contains(&0)
    );
    Some(Label::MsieIndexDat)
}

/// Microsoft SmartCard Provider Trace Record (.ax).
pub(crate) fn verify_mssscard_ax_log(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MSSSCARD_AX_LOG_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 7)? == hex2bin!("00 01 00 01 C0 A8 7A"),
        get_slice(prefix, 8, 16)? == hex2bin!("EA 07 0A 00 01 00 0C 00"),
        get_slice(prefix, 17, 18)? == hex2bin!("00"),
        get_slice(prefix, 19, 20)? == hex2bin!("00"),
        get_slice(prefix, 21, 22)? == hex2bin!("00"),
        get_slice(prefix, 24, 32)? == hex2bin!("00 00 00 00 00 00 FA 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::MssscardAxLog)
}

/// Nucleotide Archival Format file (`.naf`, 2019 Kirill Kryukov, `01 F9 EC 01` / `01 F9 EC 02`).
pub(crate) fn verify_naf_bio(prefix: &[u8], size: u64) -> Option<Label> {
    const NAF_MAGIC: &[u8; 3] = hex2bin!("01 F9 EC"); // @0..3
    require!(
        size >= MIN_NAF_BIO_SIZE,
        prefix.len() >= 8,
        starts(prefix, NAF_MAGIC),
        matches!(u8_at(prefix, 3), Some(1 | 2))
    );
    Some(Label::NafBio)
}

/// Weihong NcStudio CNC Dynamic State File (.dyn).
pub(crate) fn verify_ncstudio_dyn(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NCSTUDIO_DYN_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 32)?
            == hex2bin!(
                "06 00 00 00 01 20 05 00 00 FF FF FF FF 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00"
            )
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::NcstudioDyn)
}

/// Neo Geo / FBNeo ABKCPUR MAO Battery Save RAM (.srm).
pub(crate) fn verify_neogeo_abkcpur_srm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NEOGEO_ABKCPUR_SRM_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 1, 32)?
            == hex2bin!(
                "09 00 00 32 56 02 80 00 01 CA 00 41 42 4B 43 50 55 52 20 4D 41 4F C2 20 4B 80 21 DA 00 C6 01"
            )
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::NeogeoAbkcpurSrm)
}

/// NETGEAR CHK Router Firmware Image (`.chk`).
pub(crate) fn verify_netgear_chk(prefix: &[u8], size: u64) -> Option<Label> {
    const CHK_MAGIC: &[u8; 4] = b"*#$^"; // @0: 0x2A23245E big-endian ("*#$^")
    const HDR_LEN_OFF: usize = 4; // @4: u32 BE header length (40..=512)
    const BOARD_ID_OFF: usize = 40; // @40: printable ASCII board_id string
    require!(size >= MIN_NETGEAR_CHK_SIZE, prefix.len() >= 58, starts(prefix, CHK_MAGIC));
    let hdr_len = be32(prefix, HDR_LEN_OFF)?;
    require!(
        (40..=512).contains(&hdr_len),
        get_slice(prefix, BOARD_ID_OFF, BOARD_ID_OFF + 8)?.iter().all(|&b| b.is_ascii_graphic())
    );
    Some(Label::NetgearChk)
}

/// Obsidian Control Systems Netron DMX Node Firmware Image (.nfw) Obsidian Control Systems.
pub(crate) fn verify_netron_nfw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NETRON_NFW_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"Firmware File   "),
        le32(prefix, 20) == Some(1),
        le64(prefix, 24) == Some(0)
    );
    Some(Label::NetronNfw)
}

/// MicroPro Newword 2 Message Overlay (`"Newword 2"` + `"Messages"` at `@15..23`).
pub(crate) fn verify_newword_ovr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NEWWORD_OVR_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"Newword 2"),
        eq_at(prefix, 15, b"Messages")
    );
    Some(Label::NewwordOvr)
}

/// NeXTSTEP Workspace Manager window attributes file (`.wmd`, binary, 1989 NeXT).
///
/// Header layout:
/// - `@0..4`: `69 02 FC AE`.
/// - `@4..8`: `u32 LE` key length `n` (`1..=64`), `@8..8+n`: printable key (`"MainBrowserRoot"`).
/// - then `u32 LE` value length `m` (`1..=255`) + value starting with `/` (absolute path).
pub(crate) fn verify_nextstep_wmd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_NEXTSTEP_WMD_SIZE, starts(prefix, NEXTSTEP_WMD_MAGIC));
    let key_len = le32(prefix, NEXTSTEP_WMD_KEY_LEN_OFFSET)? as usize;
    require!((1..=64).contains(&key_len));
    let key_end = NEXTSTEP_WMD_KEY_OFFSET + key_len;
    let key = get_slice(prefix, NEXTSTEP_WMD_KEY_OFFSET, key_end)?;
    require!(key.iter().all(|b| b.is_ascii_graphic()));
    let val_len = le32(prefix, key_end)? as usize;
    require!((1..=255).contains(&val_len), u8_at(prefix, key_end + 4)? == b'/');
    Some(Label::NextstepWmd)
}

/// Google Nexus `IMGDATA!` Boot Splash Container (`.img`).
pub(crate) fn verify_nexus_imgdata(prefix: &[u8], size: u64) -> Option<Label> {
    const IMGDATA_MAGIC: &[u8; 12] = bytes!(b"IMGDATA!", le32_bytes!(1)); // @0: "IMGDATA!" + version 1 LE
    const COUNT_OFF: usize = 12; // @12: u32 LE image count (1..=128)
    const FIRST_NAME_OFF: usize = 24; // @24: first image ASCII name (e.g. "boot")
    require!(
        size >= MIN_NEXUS_IMGDATA_SIZE,
        prefix.len() >= 48,
        starts(prefix, IMGDATA_MAGIC),
        (1..=128).contains(&le32(prefix, COUNT_OFF)?),
        get_slice(prefix, FIRST_NAME_OFF, FIRST_NAME_OFF + 4)?
            .iter()
            .all(|&b| b.is_ascii_graphic())
    );
    Some(Label::NexusImgdata)
}

/// nfdump NetFlow Binary Data (`0C A5 01 00` or `0C A5 02 00`).
pub(crate) fn verify_nfdump_flow_data(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NFDUMP_FLOW_SIZE,
        prefix.len() >= 24,
        (starts(prefix, &[0x0C, 0xA5, 0x01, 0x00])
            && le32(prefix, 4)? <= 0xFF
            && le32(prefix, 8)? >= 1)
            || (starts(prefix, &[0x0C, 0xA5, 0x02, 0x00])
                && ((le32(prefix, 4)? <= 0xFF && le32(prefix, 8)? >= 1)
                    || (u8_at(prefix, 16).unwrap_or(0) <= 4
                        && u8_at(prefix, 17).unwrap_or(0) == 0
                        && le32(prefix, 20)? <= 10)))
    );
    Some(Label::NfdumpFlow)
}

/// Nintendo Wii IMET Opening Banner (`.bnr`, 2006 Nintendo, `"IMET"` at `@0x40` or `@0x80`).
///
/// Header layout:
/// - `imet_off` in `{0x40 (opening.bnr), 0x80 (WAD 00000000.app)}`
/// - `@imet_off..+4`: `"IMET"`
/// - `@imet_off+4..+8`: `u32 BE` `hash_size` (`0x600`)
/// - `@imet_off+8..+12`: `u32 BE` `unk` (`3`)
/// - `@imet_off+12..+24`: `u32 BE` `icon_size`, `banner_size`, `sound_size` (sum `> 0`)
/// - `@imet_off+0x5C0..+0x5C4`: embedded Nintendo U8 archive magic (`55 AA 38 2D`)
pub(crate) fn verify_nintendo_wii_opening_banner(prefix: &[u8], size: u64) -> Option<Label> {
    const U8_MAGIC: &[u8; 4] = hex2bin!("55 AA 38 2D");
    const IMET_TO_U8_DELTA: usize = 0x5C0; // 0x40 -> 0x600, 0x80 -> 0x640

    require!(size >= MIN_NINTENDO_WII_BNR_SIZE);
    for imet_off in [0x40usize, 0x80usize] {
        let u8_off = imet_off + IMET_TO_U8_DELTA;
        if prefix.len() < u8_off + 4 {
            continue;
        }
        if eq_at(prefix, imet_off, b"IMET")
            && be32(prefix, imet_off + 4) == Some(0x600)
            && be32(prefix, imet_off + 8) == Some(3)
            && eq_at(prefix, u8_off, U8_MAGIC)
        {
            let icon_sz = be32(prefix, imet_off + 12)?;
            let banner_sz = be32(prefix, imet_off + 16)?;
            let sound_sz = be32(prefix, imet_off + 20)?;
            if icon_sz.saturating_add(banner_sz).saturating_add(sound_sz) > 0 {
                return Some(Label::NintendoWiiBnr);
            }
        }
    }
    None
}

/// Norton Commander Message Catalog (`"Abort\0nc.cfg\0Error "` or `"Abort\0"` + `"nc.cfg\0"`).
pub(crate) fn verify_norton_commander_msg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NORTON_COMMANDER_MSG_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"Abort\0nc.cfg\0Error ")
    );
    Some(Label::NcMsg)
}

/// Microsoft Word / Office Owner Lock File (~$*.doc / ~$*.docx).
pub(crate) fn verify_office_owner_lock(prefix: &[u8], size: u64) -> Option<Label> {
    let n = prefix.first().copied().unwrap_or(0) as usize;
    let rec_len = (56 + 2 * n).max(72);
    require!(
        size >= MIN_OFFICE_OWNER_LOCK_SIZE,
        prefix.len() >= rec_len,
        (1..=52).contains(&n),
        get_slice(prefix, 1, 1 + n)?.iter().all(|&b| (0x20..=0x7E).contains(&b)),
        get_slice(prefix, 1 + n, 54)?.iter().all(|&b| b == 0)
            || (size == 165 && get_slice(prefix, 1 + n, 54)?.iter().all(|&b| b == b' ')),
        le16(prefix, 54) == Some(n as u16),
        (0..n).all(|i| u8_at(prefix, 56 + 2 * i).unwrap_or(0) == u8_at(prefix, 1 + i).unwrap_or(0)
            && u8_at(prefix, 56 + 2 * i + 1).unwrap_or(0) == 0)
    );
    require!(has_non_ascii_text_byte(prefix.get(..rec_len)?));
    Some(Label::OfficeOwnerLock)
}

/// Microsoft OneDrive Obfuscated Diagnostic Log file (`.odl` / `.odlgz` / `.odlsent` / `.aodl`).
pub(crate) fn verify_onedrive_odl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ONEDRIVE_ODL_SIZE,
        prefix.len() >= 32,
        starts(prefix, ONEDRIVE_ODL_MAGIC),
        (1..=8).contains(&le32(prefix, ODL_VERSION_OFFSET)?),
        le64(prefix, ODL_RESERVED_OFFSET) == Some(0),
        le32(prefix, ODL_FLAGS_OFFSET) == Some(1)
    );
    Some(Label::OnedriveOdl)
}

/// OpenSSH Key Revocation List (`.krl`, `"SSHKRL\n\0"` + `be32(8) == 1`).
pub(crate) fn verify_openssh_krl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_OPENSSH_KRL_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"SSHKRL\n\0"),
        be32(prefix, 8) == Some(1)
    );
    Some(Label::OpensshKrl)
}

/// Eclipse OSGi Framework Bundle Info Cache (`framework.info.*`, `DataInputStream` header + `"osgi."`).
pub(crate) fn verify_osgi_framework_info4(prefix: &[u8], size: u64) -> Option<Label> {
    const OSGI_PROP_PREFIX: &[u8; 5] = b"osgi.";

    require!(
        size >= MIN_OSGI_FRAMEWORK_INFO4_SIZE,
        prefix.len() >= 32,
        (1..=20).contains(&be32(prefix, 0)?) // @0..4: framework state cache format version
    );
    let l1 = be16(prefix, 4)? as usize;
    require!(
        (3..=32).contains(&l1),
        6 + l1 + 2 <= prefix.len(),
        get_slice(prefix, 6, 6 + l1)?
            .iter()
            .all(|&b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
    );
    let l2 = be16(prefix, 6 + l1)? as usize;
    let off = 8 + l1 + l2;
    require!(
        (3..=32).contains(&l2),
        off + 9 <= prefix.len(),
        get_slice(prefix, 8 + l1, off)?
            .iter()
            .all(|&b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_')),
        (16..=1_000_000).contains(&be32(prefix, off)?),
        eq_at(prefix, off + 4, OSGI_PROP_PREFIX)
    );
    Some(Label::OsgiFrameworkInfo4)
}

/// Panasonic / Leica `UPD\0` (`55 50 44 00`, D-Lux 5 / D-Lux 8 / Lumix) and bitwise
/// XOR-`0xFF` inverted (`AA AF BB FF`, Q / CL / SL / SL2 `.quf`/`.lfu`) camera firmware containers.
pub(crate) fn verify_panasonic_upd_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_LEN: usize = 68;
    require!(size >= MIN_PANASONIC_UPD_SIZE && prefix.len() >= HDR_LEN);
    let xor_mask: u8 = if starts(prefix, b"UPD\0") {
        0x00
    } else if starts(prefix, PANASONIC_UPD_XORFF_MAGIC) {
        0xFF
    } else {
        return None;
    };
    let mut hdr = [0u8; HDR_LEN];
    for (dst, &src) in hdr.iter_mut().zip(get_slice(prefix, 0, HDR_LEN)?.iter()) {
        *dst = src ^ xor_mask;
    }
    let header_size = u32::from_le_bytes([hdr[4], hdr[5], hdr[6], hdr[7]]);
    let parts = u32::from_le_bytes([hdr[52], hdr[53], hdr[54], hdr[55]]);
    let data_off = u32::from_le_bytes([hdr[56], hdr[57], hdr[58], hdr[59]]);
    let data_len = u32::from_le_bytes([hdr[60], hdr[61], hdr[62], hdr[63]]);
    require!(header_size == 0x200, (1..=64).contains(&parts), data_off == 0x200, data_len >= 64);
    // @12..28: NUL-terminated printable ASCII camera code (e.g. "DL5", "CAYMN", "MC671", "CRERA")
    let model_raw = &hdr[12..28];
    let nul_pos = model_raw.iter().position(|&b| b == 0)?;
    require!(
        (2..=15).contains(&nul_pos),
        model_raw[..nul_pos].iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    );
    std::str::from_utf8(&model_raw[..nul_pos]).ok()?;
    Some(Label::PanasonicUpdFirmware)
}

/// PassMark PerformanceTest Baseline Results (PTRESFILE, .pt).
pub(crate) fn verify_passmark_pt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PASSMARK_PT_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"PTRESFILE\0\0\0") || starts(prefix, b"PTRESY\xf0\xf3\x0fY")
    );
    require!(has_non_ascii_text_byte(prefix.get(..28)?));
    Some(Label::PassmarkPt)
}

/// GRAHL Software Design PDF Annotator Stamp/Item Library (.sil) GRAHL Software Design.
pub(crate) fn verify_pdfannotator_sil(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PDFANNOTATOR_SIL_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"SILF"),
        (1..=16).contains(&le16(prefix, 4).unwrap_or(0)),
        (0x20..=0x80).contains(&le16(prefix, 6).unwrap_or(0)),
        le16(prefix, 8) == Some(0)
    );
    Some(Label::PdfannotatorSil)
}

/// PKCS #15 Cryptographic Token Info (OID 1.2.840.113549.1.15.3.1, .ky).
pub(crate) fn verify_pkcs15_ky(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PKCS15_KY_SIZE,
        prefix.len() >= 28,
        u8_at(prefix, 0)? == 0x30,
        u8_at(prefix, 1)? == 0x82,
        get_slice(prefix, 4, 16)? == b"\x06\x0a\x2a\x86\x48\x86\xf7\x0d\x01\x0f\x03\x01"
    );
    require!(has_non_ascii_text_byte(prefix.get(..28)?));
    Some(Label::Pkcs15Ky)
}

/// Sony PlayStation System File Object (PARAM.SFO / .sfo).
pub(crate) fn verify_playstation_sfo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PLAYSTATION_SFO_SIZE && prefix.len() >= 20);
    if u8_at(prefix, 0)? != 0 || get_slice(prefix, 1, 4)? != b"PSF" {
        return None;
    }
    let version = le32(prefix, 4)?;
    if !matches!(version, 0x0000_0101 | 0x0000_0100 | 0x0001_0100 | 0x0001_0000) {
        return None;
    }
    let key_table_start = le32(prefix, 8)? as u64;
    let data_table_start = le32(prefix, 12)? as u64;
    let num_entries = le32(prefix, 16)? as u64;

    if 20 + num_entries * 16 > key_table_start
        || key_table_start > data_table_start
        || data_table_start > size
    {
        return None;
    }

    Some(Label::PsSfo)
}

/// Oxford Nanopore POD5 raw-signal container: the full 8-byte signature (binary `0x8B` lead
/// byte plus `\r\n\x1A\n` transport guard) is required.
pub(crate) fn verify_pod5(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_POD5_SIZE, starts(prefix, POD5_SIGNATURE));
    Some(Label::Pod5)
}

/// Bauer+Kirch PPM-Optimierung Data (.mpd, Bauer+Kirch Monschau).
pub(crate) fn verify_ppm_mpd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PPM_MPD_SIZE,
        prefix.len() >= 64,
        starts(prefix, b"***PPM-OPTIMIERUNG***(C)BAUER+KIRCH,MONSCHAU(GER"),
        has_non_ascii_text_byte(prefix.get(48..64)?)
    );
    require!(has_non_ascii_text_byte(prefix.get(..64)?));
    Some(Label::PpmMpd)
}

/// Princed Prince of Persia Level (`.plv`, Apoplexy / RoomShaker / Princed).
///
/// Header layout:
/// - `@0..7`: `"POP_LVL"` magic
/// - `@7`: `u8` `pop_version` (`1` = PoP 1, `2` = PoP 2)
/// - `@8`: `u8` `plv_version` (`1` or `2`)
/// - `@9`: `u8` `level_number` (`0..=15`)
/// - `@10..14`: `u32 LE` `number_of_fields` (`1..=64`)
/// - `@14..18`: `u32 LE` `level_size` (`>= 16`)
pub(crate) fn verify_princed_pop_plv(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 7] = b"POP_LVL"; // @0..7: Princed PLV header
    const POP_VER_OFF: usize = 7; // @7: Prince of Persia game version (1 or 2)
    const PLV_VER_OFF: usize = 8; // @8: PLV format version (1 or 2)
    const LEVEL_NUM_OFF: usize = 9; // @9: level index (0..=15)
    const FIELDS_CNT_OFF: usize = 10; // @10..14: u32 LE metadata field count
    const LEVEL_SIZE_OFF: usize = 14; // @14..18: u32 LE level payload size

    require!(
        size >= MIN_PRINCED_POP_PLV_SIZE,
        prefix.len() >= 20,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, POP_VER_OFF)?, 1 | 2),
        matches!(u8_at(prefix, PLV_VER_OFF)?, 1 | 2),
        u8_at(prefix, LEVEL_NUM_OFF)? <= 15,
        (1..=64).contains(&le32(prefix, FIELDS_CNT_OFF)?),
        le32(prefix, LEVEL_SIZE_OFF)? >= 16
    );
    Some(Label::PopPlv)
}

/// Microsoft Sysinternals Process Monitor Configuration (`.pmc`):
/// contiguous sequence of 16-byte little-endian setting records (`u32 rec_len`, `u32 hdr_len == 16`,
/// `u32 val_offset`, `u32 val_bytes`, `val_offset >= 20`, `val_offset + val_bytes == rec_len`), where `@16..val_offset`
/// stores UTF-16LE NUL-terminated Procmon setting names (`Columns`, `ColumnMap`, `FilterRules`,
/// `HighlightRules`, `Autoscroll`, `DestructiveFilter`, `HistoryDepth`, `Logfile`).
pub(crate) fn verify_procmon_pmc(prefix: &[u8], size: u64) -> Option<Label> {
    const PROCMON_UTF16_KEYS: &[&[u8]] = &[
        b"C\0o\0l\0u\0m\0n\0s\0\0\0",
        b"C\0o\0l\0u\0m\0n\0M\0a\0p\0\0\0",
        b"F\0i\0l\0t\0e\0r\0R\0u\0l\0e\0s\0\0\0",
        b"H\0i\0g\0h\0l\0i\0g\0h\0t\0R\0u\0l\0e\0s\0\0\0",
        b"A\0u\0t\0o\0s\0c\0r\0o\0l\0l\0\0\0",
        b"D\0e\0s\0t\0r\0u\0c\0t\0i\0v\0e\0F\0i\0l\0t\0e\0r\0\0\0",
        b"H\0i\0s\0t\0o\0r\0y\0D\0e\0p\0t\0h\0\0\0",
    ];
    require!(size >= MIN_PROCMON_PMC_SIZE, prefix.len() >= 34, le32(prefix, 4)? == 16,);
    let mut pos = 0usize;
    let mut valid_records = 0usize;
    let mut known_key_hits = 0usize;
    while pos + 16 <= prefix.len() && valid_records < 8 {
        let rec_len = le32(prefix, pos)? as usize;
        let hdr_len = le32(prefix, pos + 4)? as usize;
        let val_offset = le32(prefix, pos + 8)? as usize;
        let val_bytes = le32(prefix, pos + 12)? as usize;
        let Some(name_bytes) = val_offset.checked_sub(16) else {
            break;
        };
        if hdr_len != 16
            || name_bytes < 4
            || name_bytes % 2 != 0
            || name_bytes > 256
            || val_offset.checked_add(val_bytes)? != rec_len
            || (pos as u64 + rec_len as u64) > size
        {
            break;
        }
        let name_end = (pos + 16 + name_bytes).min(prefix.len());
        let name_slice = get_slice(prefix, pos + 16, name_end)?;
        if PROCMON_UTF16_KEYS.iter().any(|k| name_slice.starts_with(k)) {
            known_key_hits += 1;
        }
        valid_records += 1;
        let Some(next_pos) = pos.checked_add(rec_len) else {
            break;
        };
        pos = next_pos;
        if pos as u64 == size {
            break;
        }
    }
    require!(valid_records >= 1, known_key_hits >= 1);
    Some(Label::ProcmonPmc)
}

/// Project64 Nintendo 64 Emulator Save State (`.pj`, 2001 Zilmar).
///
/// Header layout:
/// - `@0..4`: `u32 LE` magic `0x23D8A6C8` (`C8 A6 D8 23`)
/// - `@4..8`: `u32 LE` N64 RDRAM byte size (`0x0040_0000` = 4 MiB standard or `0x0080_0000` = 8 MiB Expansion Pak)
/// - `@8..12`:Embedded N64 ROM header PI BSD Domain 1 register word (`0x8037_1240` LE = `40 12 37 80` or byte-swapped)
pub(crate) fn verify_project64_pj(prefix: &[u8], size: u64) -> Option<Label> {
    const RDRAM_SIZE_OFF: usize = 4; // @4..8: u32 LE RDRAM size (4 MiB or 8 MiB)
    const RDRAM_4MB: u32 = 0x0040_0000;
    const RDRAM_8MB: u32 = 0x0080_0000;

    require!(
        size >= MIN_PROJECT64_PJ_SIZE,
        prefix.len() >= 40,
        starts(prefix, PROJECT64_PJ_MAGIC),
        matches!(le32(prefix, RDRAM_SIZE_OFF)?, RDRAM_4MB | RDRAM_8MB)
    );
    Some(Label::Project64Pj)
}

/// Sony PlayStation 1 Memory Card Image (.mcr / .mcd).
pub(crate) fn verify_ps1_memory_card(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PS1_MEMORY_CARD_SIZE && size.is_multiple_of(PS1_MEMORY_CARD_SIZE));
    require!(prefix.len() >= 128);

    if u8_at(prefix, 0).unwrap_or(0) != b'M' || u8_at(prefix, 1).unwrap_or(0) != b'C' {
        return None;
    }
    // In Frame 0, bytes 2..127 must be all 0x00
    for &b in get_slice(prefix, 2, 127)? {
        if b != 0x00 {
            return None;
        }
    }
    // Byte 127 is the checksum byte ('M' ^ 'C' = 0x0E)
    if u8_at(prefix, 127)? != 0x0E {
        return None;
    }

    Some(Label::Ps1Mcr)
}

/// Sony PlayStation 2 Memory Card Image (.ps2 / .vmc).
pub(crate) fn verify_ps2_memory_card(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PS2_MEMORY_CARD_SIZE && prefix.len() >= 64);
    if !prefix.starts_with(b"Sony PS2 Memory Card Format ") {
        return None;
    }
    if !get_slice(prefix, 0x1C, 0x28)?.starts_with(b"1.") {
        return None;
    }
    let page_len = le16(prefix, 0x28)?;
    if page_len != 512 && page_len != 1024 {
        return None;
    }
    let pages_per_cluster = le16(prefix, 0x2A)?;
    if pages_per_cluster != 1 && pages_per_cluster != 2 {
        return None;
    }

    Some(Label::Ps2Vmc)
}

/// Polysomnography (PSG) Sleep Study Recording (.psg).
pub(crate) fn verify_psg_sleep(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PSG_SLEEP_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\x50\x80\xfe\0\xab\x73\x12\x08\0\x0d\0"),
        get_slice(prefix, 21, 29)? == b"\x01\0\x0b\0\x0c\0\x01\x80"
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::PsgSleep)
}

/// Qualcomm Atheros WLAN / WiGig Firmware or Board Data Binary (`"QCA-ATH10K"`, `"QCA-ATH11K"`, `"QCA-ATH12K"`, `"QCA-ATH6KL"`, `"_wmi_cmd_rsp\0"`, or `wil6210`).
pub(crate) fn verify_qca_ath_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const WIL6210_MAGIC: &[u8; 16] = bytes!(le32_bytes!(6), le32_bytes!(0x34), b"0126\0\0\0\0"); // @0..16: wil6210 record 6 + len 0x34 + "0126"

    require!(size >= MIN_QCA_ATH_FIRMWARE_SIZE, prefix.len() >= 16);
    if starts(prefix, b"QCA-ATH10K")
        || starts(prefix, b"QCA-ATH11K")
        || starts(prefix, b"QCA-ATH12K")
        || starts(prefix, b"QCA-ATH6KL")
        || starts(prefix, b"_wmi_cmd_rsp\0")
        || starts(prefix, WIL6210_MAGIC)
    {
        return Some(Label::QcaAthFirmware);
    }
    None
}

/// Qualcomm Atheros Bluetooth Rampatch & NVM TLV Firmware Binary (`.bin` / `.tlv`).
pub(crate) fn verify_qca_bt_tlv(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_QCA_BT_TLV_SIZE, prefix.len() >= 16);
    let tlv_type = u8_at(prefix, 0)?;
    let top_len = (u8_at(prefix, 1)? as u64)
        | ((u8_at(prefix, 2)? as u64) << 8)
        | ((u8_at(prefix, 3)? as u64) << 16);
    require!(matches!(tlv_type, 1 | 2 | 4), 4 + top_len == size);
    if tlv_type == 1 {
        let total_sz = le32(prefix, 4)? as u64;
        let patch_sz = le32(prefix, 8)? as u64;
        require!(
            total_sz + 4 == size
                || (total_sz <= top_len && patch_sz < total_sz && (total_sz - patch_sz) <= 4096)
        );
        return Some(Label::QcaBtTlv);
    }
    // Type 2 / Type 4 NVM tag sequence: walk consecutive 12-byte (tag_id, tag_len, tag_ptr, tag_ex_flag) records
    let mut pos = if tlv_type == 4 && u8_at(prefix, 4) == Some(2) { 8usize } else { 4usize };
    let mut tags = 0usize;
    while pos < prefix.len() && (pos as u64) < size && tags < 4 {
        require!(pos + 12 <= prefix.len());
        let tag_id = le16(prefix, pos)?;
        let tag_len = le16(prefix, pos + 2)? as usize;
        require!(tag_id > 0, pos + 12 + tag_len <= size as usize, le32(prefix, pos + 4) == Some(0));
        pos += 12 + tag_len;
        tags += 1;
    }
    require!(tags >= 2 || (tags == 1 && pos as u64 == size));
    Some(Label::QcaBtTlv)
}

/// BlackBerry QNX Neutrino Image Filesystem Boot Image (`.ifs`).
pub(crate) fn verify_qnx_ifs(prefix: &[u8], size: u64) -> Option<Label> {
    const QNX_IFS_LE: &[u8; 4] = hex2bin!("EB 7E FF 00"); // @0: 0x00FF7EEB little-endian
    const QNX_IFS_BE: &[u8; 4] = hex2bin!("00 FF 7E EB"); // @0: 0x00FF7EEB big-endian
    const VERSION_OFF: usize = 4; // @4: u16 version (1)
    const HDR_SIZE_OFF: usize = 8; // @8: u32 startup_header size (256 = 0x100)
    require!(size >= MIN_QNX_IFS_SIZE, prefix.len() >= 64);
    let (ver, hdr_sz) = if starts(prefix, QNX_IFS_LE) {
        (le16(prefix, VERSION_OFF)?, le32(prefix, HDR_SIZE_OFF)?)
    } else if starts(prefix, QNX_IFS_BE) {
        (be16(prefix, VERSION_OFF)?, be32(prefix, HDR_SIZE_OFF)?)
    } else {
        return None;
    };
    require!(ver == 1, hdr_sz == 256);
    Some(Label::QnxIfs)
}

/// Qualcomm Audio Calibration Database (`.acdb`, `"QCMSNDDB"` + `"AVDB"`).
pub(crate) fn verify_qualcomm_acdb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_QUALCOMM_ACDB_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"QCMSNDDB"),
        eq_at(prefix, 16, b"AVDB")
    );
    Some(Label::QualcommAcdb)
}

/// Qualcomm Fastboot Firmware Pack (`"FBPK"` v0, v1, v2) or Partition Table (`"FBPT"`).
pub(crate) fn verify_qualcomm_fbpk(prefix: &[u8], size: u64) -> Option<Label> {
    if starts(prefix, b"FBPT") {
        require!(
            size >= MIN_QUALCOMM_FBPK_SIZE,
            prefix.len() >= 60,
            (0..=2).contains(&le32(prefix, 4)?),
            le64(prefix, 8) == Some(0),
            (1..=256).contains(&le32(prefix, 16)?),
            u8_at(prefix, 56)?.is_ascii_graphic()
        );
        return Some(Label::QualcommFbpk);
    }
    require!(size >= MIN_QUALCOMM_FBPK_SIZE, prefix.len() >= 40, starts(prefix, b"FBPK"));
    let ver = le32(prefix, 4)?;
    let cnt = match ver {
        0 => le32(prefix, 8)?,
        1 => {
            if prefix.len() >= 84 && (1..=256).contains(&le32(prefix, 76).unwrap_or(0)) {
                le32(prefix, 76)?
            } else {
                le32(prefix, 36)?
            }
        }
        2 => {
            if prefix.len() >= 112 && le32(prefix, 8) == Some(112) && le32(prefix, 12) == Some(104)
            {
                le32(prefix, 104)?
            } else {
                le32(prefix, 36)?
            }
        }
        _ => return None,
    };
    require!((1..=256).contains(&cnt));
    Some(Label::QualcommFbpk)
}

/// 40-byte Qualcomm MBN Bootloader / Modem Firmware Header (`qualcomm_sbl1`).
pub(crate) fn verify_qualcomm_mbn40(prefix: &[u8], size: u64) -> Option<Label> {
    const IMG_ID_OFF: usize = 0; // @0..4: u32 LE image_id (1..=64)
    const VER_OFF: usize = 4; // @4..8: u32 LE header_vsn_num (3)
    const DEST_PTR_OFF: usize = 12; // @12..16: u32 LE image_dest_ptr (!= 0)
    const IMG_SZ_OFF: usize = 16; // @16..20: u32 LE image_size (> 0)
    const CODE_SZ_OFF: usize = 20; // @20..24: u32 LE code_size (> 0)
    const SIG_PTR_OFF: usize = 24; // @24..28: u32 LE signature_ptr (== dest_ptr + code_size)
    const SIG_SZ_OFF: usize = 28; // @28..32: u32 LE signature_size
    const CERT_PTR_OFF: usize = 32; // @32..36: u32 LE cert_chain_ptr (== sig_ptr + sig_size)
    const CERT_SZ_OFF: usize = 36; // @36..40: u32 LE cert_chain_size
    require!(
        size >= 40,
        prefix.len() >= 40,
        (1..=64).contains(&le32(prefix, IMG_ID_OFF)?),
        le32(prefix, VER_OFF) == Some(3)
    );
    let dest_ptr = le32(prefix, DEST_PTR_OFF)?;
    let img_sz = le32(prefix, IMG_SZ_OFF)?;
    let code_sz = le32(prefix, CODE_SZ_OFF)?;
    let sig_ptr = le32(prefix, SIG_PTR_OFF)?;
    let sig_sz = le32(prefix, SIG_SZ_OFF)?;
    let cert_ptr = le32(prefix, CERT_PTR_OFF)?;
    let cert_sz = le32(prefix, CERT_SZ_OFF)?;
    require!(
        dest_ptr != 0,
        code_sz > 0,
        img_sz >= code_sz,
        sig_ptr == dest_ptr.wrapping_add(code_sz),
        cert_ptr == sig_ptr.wrapping_add(sig_sz),
        code_sz.wrapping_add(sig_sz).wrapping_add(cert_sz) == img_sz
    );
    Some(Label::QualcommSbl1)
}

/// Qualcomm Android Bootloader `meta_image` (`0xCE1AD63C` LE).
pub(crate) fn verify_qualcomm_meta_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_QUALCOMM_META_IMAGE_SIZE,
        prefix.len() >= 76,
        starts(prefix, hex2bin!("3C D6 1A CE")),
        le16(prefix, 4) == Some(1),
        le16(prefix, 6) == Some(0),
        le16(prefix, 72) == Some(76)
    );
    Some(Label::QualcommMetaImage)
}

/// Qualcomm MSM MIBIB Flash Partition Table (`.bin`).
pub(crate) fn verify_qualcomm_mibib(prefix: &[u8], size: u64) -> Option<Label> {
    const MIBIB_MAGIC: &[u8; 8] = hex2bin!("9A 1B 7D AA BC 48 7D 1F"); // @0: PART_TABLE_MAGIC1 + PART_TABLE_MAGIC2 LE
    const VER_OFF: usize = 8; // @8: u32 LE partition table version (1..=16)
    const NUMPARTS_OFF: usize = 12; // @12: u32 LE partition count (1..=128)
    const FIRST_NAME_OFF: usize = 16; // @16: first partition name prefix ("0:")
    require!(
        size >= MIN_QUALCOMM_MIBIB_SIZE,
        prefix.len() >= 44,
        starts(prefix, MIBIB_MAGIC),
        (1..=16).contains(&le32(prefix, VER_OFF)?),
        (1..=128).contains(&le32(prefix, NUMPARTS_OFF)?),
        eq_at(prefix, FIRST_NAME_OFF, b"0:")
    );
    Some(Label::QualcommMibib)
}

/// Qualcomm Secondary Bootloader SBL1 / NorCodeWord / MBN Image (`.mbn`).
pub(crate) fn verify_qualcomm_sbl1(prefix: &[u8], size: u64) -> Option<Label> {
    const SBL1_MAGIC: &[u8; 8] = hex2bin!("D1 DC 4B 84 34 10 D7 73"); // @0: 8-byte Qualcomm flash codeword + magic
    const VER_OFF: usize = 8; // @8: u32 LE header version (1..=64) or FLASH_PARTI_VERSION (0x7D0B435A)
    const PAD_OFF: usize = 12; // @12..20: 8 bytes of 0xFF flash padding
    require!(
        size >= 80,
        prefix.len() >= 64,
        starts(prefix, SBL1_MAGIC),
        matches!(le32(prefix, VER_OFF)?, 1..=64 | 0x7D0B_435A),
        eq_at(prefix, PAD_OFF, &[0xFF; 8])
    );
    Some(Label::QualcommSbl1)
}

/// Qualcomm Snapdragon Boot Splash Screen Image (`.img`).
pub(crate) fn verify_qualcomm_splash(prefix: &[u8], size: u64) -> Option<Label> {
    const SPLASH_MAGIC: &[u8; 8] = b"SPLASH!!"; // @0: 8-byte splash header signature
    const WIDTH_OFF: usize = 8; // @8: u32 LE frame width in pixels (1..=16384)
    const HEIGHT_OFF: usize = 12; // @12: u32 LE frame height in pixels (1..=16384)
    require!(
        size >= MIN_QUALCOMM_SPLASH_SIZE,
        prefix.len() >= 32,
        starts(prefix, SPLASH_MAGIC),
        (1..=16384).contains(&le32(prefix, WIDTH_OFF)?),
        (1..=16384).contains(&le32(prefix, HEIGHT_OFF)?)
    );
    Some(Label::QualcommSplash)
}

/// Realtek Bluetooth Controller Firmware Patch (`"Realtech"`) or Config (`0x8723AB55`).
pub(crate) fn verify_realtek_bt_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const RTK_CFG_MAGIC: &[u8; 4] = hex2bin!("55 AB 23 87"); // @0..4: 0x8723AB55 LE Realtek BT config signature

    if size >= 20 && prefix.len() >= 20 && starts(prefix, b"Realtech") {
        let np1 = le16(prefix, 12)?;
        let np2 = le16(prefix, 16)?;
        require!(
            ((1..=64).contains(&np1) && 14 + (np1 as u64) * 8 <= size)
                || ((1..=64).contains(&np2) && 18 + (np2 as u64) * 8 <= size)
        );
        return Some(Label::RealtekBtFw);
    }
    if size >= 10
        && prefix.len() >= 10
        && starts(prefix, RTK_CFG_MAGIC)
        && 6 + (le16(prefix, 4)? as u64) == size
    {
        return Some(Label::RealtekBtFw);
    }
    None
}

/// Realtek AP / Router CSYS Firmware Image (`.bin`).
pub(crate) fn verify_realtek_csys(prefix: &[u8], size: u64) -> Option<Label> {
    const CSYS_MAGIC: &[u8; 4] = b"CSYS"; // @0: "CSYS" signature
    const BURN_ADDR_OFF: usize = 4; // @4: u32 burn address (!= 0)
    const LEN_OFF: usize = 8; // @8: u32 payload length (> 0)
    require!(
        size >= MIN_REALTEK_CSYS_SIZE,
        prefix.len() >= 20,
        starts(prefix, CSYS_MAGIC),
        has_non_ascii_text_byte(prefix.get(4..12)?),
        le32(prefix, BURN_ADDR_OFF)? != 0,
        le32(prefix, LEN_OFF)? > 0,
        le32(prefix, LEN_OFF)? <= 0x1000_0000 || be32(prefix, LEN_OFF)? <= 0x1000_0000
    );
    Some(Label::RealtekCsys)
}

/// Realtek RTK0 Bluetooth / Wi-Fi SoC Firmware Patch (`.bin`).
pub(crate) fn verify_realtek_rtk0(prefix: &[u8], size: u64) -> Option<Label> {
    const RTK0_MAGIC: &[u8; 4] = b"RTK0"; // @0: "RTK0" signature
    const FILE_SIZE_OFF: usize = 4; // @4: u32 LE firmware size (>= 32)
    const VER_OFF: usize = 12; // @12: u32 LE version constant (0x00010000)
    const HDR_SIZE_OFF: usize = 16; // @16: u32 LE header size (0x1C = 28)
    require!(
        size >= MIN_REALTEK_RTK0_SIZE,
        prefix.len() >= 28,
        starts(prefix, RTK0_MAGIC),
        le32(prefix, FILE_SIZE_OFF)? >= 32,
        le32(prefix, VER_OFF) == Some(0x0001_0000),
        le32(prefix, HDR_SIZE_OFF) == Some(0x1C)
    );
    Some(Label::RealtekRtk0)
}

/// Realtek IEEE 802.11 Wireless MAC/BB/RF Firmware Image (`rtlwifi`, `rtw88`, or `rtw89`).
pub(crate) fn verify_realtek_wifi_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const RTW89_FWHDR_VER_OFF: usize = 5; // @5: rtw89 header version
    const RTW89_PART_CNT_OFF: usize = 9; // @9: rtw89 section count (1..=8)
    const RTW89_SEC_MAGIC_OFF: usize = 32; // @32..36: 0xFF <n> 00 00 section descriptor tag

    require!(size >= MIN_REALTEK_WIFI_FW_SIZE, prefix.len() >= 32);
    // 1a. rtw89 with section table at @0 (0xFF <n> 00 00)
    if size >= 64
        && prefix.len() >= 64
        && u8_at(prefix, 0) == Some(0xFF)
        && (1..=16).contains(&u8_at(prefix, 1)?)
        && le16(prefix, 2) == Some(0)
        && get_slice(prefix, 8, 16)? == [0u8; 8]
        && (0x30..=0x200).contains(&le32(prefix, 20)?)
    {
        return Some(Label::RealtekWifiFw);
    }
    // 1b. rtw89 (56B+ header with section table at @32)
    if size >= 64
        && prefix.len() >= 64
        && u8_at(prefix, RTW89_SEC_MAGIC_OFF) == Some(0xFF)
        && (1..=16).contains(&u8_at(prefix, RTW89_SEC_MAGIC_OFF + 1)?)
        && le16(prefix, RTW89_SEC_MAGIC_OFF + 2) == Some(0)
        && (1..=8).contains(&u8_at(prefix, RTW89_PART_CNT_OFF)?)
        && u8_at(prefix, RTW89_FWHDR_VER_OFF)? <= 16
    {
        return Some(Label::RealtekWifiFw);
    }
    // 2. rtw88 (64B header: chip_id in {0x8814, 0x8821, 0x8822})
    let sig = le16(prefix, 0)?;
    if size >= 64
        && prefix.len() >= 64
        && matches!(sig, 0x8814 | 0x8821 | 0x8822)
        && (1..=12).contains(&u8_at(prefix, 16)?)
        && (1..=31).contains(&u8_at(prefix, 17)?)
        && (1..=64).contains(&le32(prefix, 24)?)
    {
        return Some(Label::RealtekWifiFw);
    }
    // 3. rtlwifi (32B rtl_wifi_header: 32 + fw_size == size and @20..32 == 0)
    let fw_sz = le16(prefix, 12)? as u64;
    let chip = sig >> 4;
    let valid_chip = matches!(chip, 0x03B | 0x10B | 0x210 | 0x230 | 0x23D | 0x530 | 0x819)
        || matches!(chip, 0x872 | 0x881 | 0x882 | 0x885 | 0x88C | 0x88E | 0x88F)
        || matches!(chip, 0x920 | 0x92C | 0x92D | 0x92E | 0x92F | 0x950);
    if valid_chip && fw_sz > 0 && 32 + fw_sz == size && get_slice(prefix, 20, 32)? == [0u8; 12] {
        return Some(Label::RealtekWifiFw);
    }
    None
}

/// Robert Hoehne RHIDE DJGPP IDE Project (`.gpr`) or Desktop State (`.gdt`, 1996).
pub(crate) fn verify_rhide_gdt(prefix: &[u8], size: u64) -> Option<Label> {
    const RHIDE_DESKTOP_MAGIC: &[u8; 25] = b"\0\x17__RHIDE_desktop-file__\x1a"; // @0..25: .gdt desktop signature
    const RHIDE_PROJECT_MAGIC: &[u8; 24] = b"\x17__RHIDE_project-file__\x1a"; // @0..24: .gpr project signature

    require!(
        size >= MIN_RHIDE_GDT_SIZE,
        prefix.len() >= 32,
        starts(prefix, RHIDE_DESKTOP_MAGIC) || starts(prefix, RHIDE_PROJECT_MAGIC)
    );
    Some(Label::RhideGdt)
}

/// Rockchip RKAF Update Archive Image (`.img`).
pub(crate) fn verify_rockchip_rkaf(prefix: &[u8], size: u64) -> Option<Label> {
    const RKAF_MAGIC: &[u8; 4] = b"RKAF"; // @0: "RKAF" magic
    const LEN_OFF: usize = 4; // @4: u32 LE archive size (>= 136)
    const MODEL_OFF: usize = 8; // @8..24: NUL-padded printable ASCII model name
    require!(
        size >= MIN_ROCKCHIP_RKAF_SIZE,
        prefix.len() >= 64,
        starts(prefix, RKAF_MAGIC),
        le32(prefix, LEN_OFF)? >= 136,
        u8_at(prefix, MODEL_OFF)?.is_ascii_graphic(),
        get_slice(prefix, MODEL_OFF, MODEL_OFF + 16)?
            .iter()
            .all(|&b| b == 0 || b.is_ascii_graphic()),
        has_non_ascii_text_byte(prefix.get(..64)?)
    );
    Some(Label::RockchipRkaf)
}

/// Rockchip RKFW Bootable Firmware Package (`.img`).
pub(crate) fn verify_rockchip_rkfw(prefix: &[u8], size: u64) -> Option<Label> {
    const RKFW_MAGIC: &[u8; 4] = b"RKFW"; // @0: "RKFW" magic
    const HDR_LEN_POS: usize = 4; // @4: u16 LE header_len (0x0066 standard, 0x0028 compact)
    const BOOT_OFF_POS: usize = 25; // @25: u32 LE bootloader offset (0x00000066 or >= 0x28)
    const FW_OFF_POS: usize = 33; // @33: u32 LE embedded RKAF firmware offset
    require!(
        size >= MIN_ROCKCHIP_RKFW_SIZE,
        prefix.len() >= 64,
        starts(prefix, RKFW_MAGIC),
        matches!(le16(prefix, HDR_LEN_POS), Some(0x0028 | 0x0066)),
        le32(prefix, BOOT_OFF_POS) == Some(0x0066)
            || le32(prefix, FW_OFF_POS)
                .map(|off| (40..=512).contains(&off) && eq_at(prefix, off as usize, b"RKAF"))
                .unwrap_or(false)
    );
    Some(Label::RockchipRkfw)
}

/// Rockchip RSCE Android Resource Image (`.img`).
pub(crate) fn verify_rockchip_rsce(prefix: &[u8], size: u64) -> Option<Label> {
    const RSCE_MAGIC: &[u8; 8] = b"RSCE\0\0\0\0"; // @0: 8-byte RSCE header magic
    const ENTR_OFF: usize = 512; // @512: first index table entry tag ("ENTR")
    require!(
        size >= MIN_ROCKCHIP_RSCE_SIZE,
        prefix.len() >= 520,
        starts(prefix, RSCE_MAGIC),
        u8_at(prefix, 8) == Some(1),
        u8_at(prefix, 9) == Some(1),
        eq_at(prefix, ENTR_OFF, b"ENTR")
    );
    Some(Label::RockchipRsce)
}

/// TP-Link / Realtek WLK Encrypted Router Configuration (`"WLK5"`..`"WLK8"` `config.bin`).
pub(crate) fn verify_router_wlk_config(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC_PREFIX: &[u8; 3] = b"WLK"; // @0..3: "WLK" + ASCII version digit ('5'..='8')
    const VERSION_OFF: usize = 4; // @4..8: u32 LE repeating numeric version (5..=8)
    const PARAM_OFF: usize = 8; // @8..12: u32 LE fixed WLK format constant (0x0003_0D40 = 200_000)
    const BLOCK0_LEN_OFF: usize = 12; // @12..16: u32 LE 32-byte key/header block length
    const BLOCK1_LEN_OFF: usize = 48; // @48..52: u32 LE 32-byte block delimiter

    require!(
        size >= MIN_ROUTER_WLK_CONFIG_SIZE,
        prefix.len() >= 52,
        starts(prefix, MAGIC_PREFIX),
        (b'5'..=b'8').contains(&u8_at(prefix, 3)?)
    );
    let ver = (u8_at(prefix, 3)? - b'0') as u32;
    require!(
        le32(prefix, VERSION_OFF) == Some(ver),
        le32(prefix, PARAM_OFF) == Some(0x0003_0D40),
        le32(prefix, BLOCK0_LEN_OFF) == Some(32),
        le32(prefix, BLOCK1_LEN_OFF) == Some(32)
    );
    Some(Label::RouterWlkConfig)
}

/// NINJA 2.0 RUP ROM Patch (`.rup`, `"NINJA2"` + `0x00`/`0x01`/`'0'`).
pub(crate) fn verify_rup_rom_patch(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RUP_PATCH_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"NINJA2"),
        u8_at(prefix, 6).unwrap_or(0) <= 1
            || (u8_at(prefix, 6).unwrap_or(0) == b'0' && u8_at(prefix, 7).unwrap_or(0) <= 1),
        has_non_ascii_text_byte(prefix.get(6..16)?)
    );
    Some(Label::RupNinja2Patch)
}

/// Samsung Odin Partition Information Table (`.pit`, `0x12349876` LE).
pub(crate) fn verify_samsung_odin_pit(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SAMSUNG_PIT_SIZE,
        prefix.len() >= 28,
        starts(prefix, hex2bin!("76 98 34 12"))
    );
    let cnt = le32(prefix, 4)? as u64;
    require!((1..=256).contains(&cnt), 28 + cnt * 132 <= size);
    Some(Label::SamsungPit)
}

/// Sawgrass SG400/SG800 RPCS-R Printer Profile (.dm, Sawgrass Technologies).
pub(crate) fn verify_sawgrass_dm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SAWGRASS_DM_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"S\0A\0W\0G\0R\0A\0S\0S\0 \0S\0G\0"),
        get_slice(prefix, 30, 44)? == b"R\0P\0C\0S\0-\0R\0\0\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::SawgrassDm)
}

/// SDW v2 System Snapshot (.snp).
pub(crate) fn verify_sdw2_snp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SDW2_SNP_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"SDW\x02\x01\0\0\0]\0\0\x80\0"),
        le32(prefix, 16) == Some(0)
    );
    Some(Label::Sdw2Snp)
}

/// Alpha Networks SEAMA (`Seattle Image`) Firmware Container (`.seama`).
pub(crate) fn verify_seama_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const SEAMA_BE: &[u8; 4] = hex2bin!("5E A3 A4 17"); // @0: 0x5EA3A417 big-endian
    const SEAMA_LE: &[u8; 4] = hex2bin!("17 A4 A3 5E"); // @0: 0x5EA3A417 little-endian
    const META_SIZE_OFF: usize = 4; // @4: u32 (BE/LE) metadata size (1..=1024)
    const META_STR_OFF: usize = 12; // @12: printable ASCII metadata key=value (in outer header) or @28 (after MD5)
    require!(size >= MIN_SEAMA_FW_SIZE, prefix.len() >= 44);
    let meta_size = if starts(prefix, SEAMA_BE) {
        be32(prefix, META_SIZE_OFF)?
    } else if starts(prefix, SEAMA_LE) {
        le32(prefix, META_SIZE_OFF)?
    } else {
        return None;
    };
    require!(
        (1..=1024).contains(&meta_size),
        get_slice(prefix, META_STR_OFF, META_STR_OFF + 8)?.iter().all(|&b| b.is_ascii_graphic())
            || get_slice(prefix, 28, 36)?.iter().all(|&b| b.is_ascii_graphic())
    );
    Some(Label::SeamaFw)
}

/// .NET Code Access Security Config Cache (.config.cch).
pub(crate) fn verify_security_cfg_cch(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SECURITY_CFG_CCH_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 12)? == hex2bin!("01 00 00 00 00 00 00 00 02 00 00 00"),
        get_slice(prefix, 20, 27)? == hex2bin!("01 00 00 00 00 00 00"),
        get_slice(prefix, 29, 32)? == hex2bin!("00 00 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::SecurityCfgCch)
}

/// NSA / Linux Kernel SELinux Binary Security Policy.
pub(crate) fn verify_selinux_policy(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 24 && prefix.len() >= 20);
    let is_kernel_policy = size >= MIN_SELINUX_POLICY_SIZE
        && starts(prefix, &[0x8C, 0xFF, 0x7C, 0xF9])
        && le32(prefix, 4) == Some(8)
        && get_slice(prefix, 8, 16)? == b"SE Linux"
        && (15..=45).contains(&le32(prefix, 16).unwrap_or(0));
    let is_modular_policy = size >= MIN_SELINUX_POLICY_SIZE
        && prefix.len() >= 28
        && starts(prefix, &[0x8D, 0xFF, 0x7C, 0xF9])
        && le32(prefix, 4) == Some(15)
        && get_slice(prefix, 8, 23)? == b"SE Linux Module"
        && (1..=2).contains(&le32(prefix, 23).unwrap_or(0));
    let is_file_contexts = starts(prefix, &[0x8E, 0xFF, 0x7C, 0xF9])
        && (1..=6).contains(&le32(prefix, 4).unwrap_or(0))
        && (1..=256).contains(&le32(prefix, 8).unwrap_or(0));
    require!(is_kernel_policy || is_modular_policy || is_file_contexts);
    Some(Label::SelinuxPolicy)
}

/// NSA SELinux Compiled Policy Package (`.pp`, `0xF97CFF8F` LE = `8F FF 7C F9`).
pub(crate) fn verify_selinux_policy_package(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SELINUX_PP_SIZE,
        prefix.len() >= 12,
        starts(prefix, &[0x8F, 0xFF, 0x7C, 0xF9]) || starts(prefix, &[0xF9, 0x7C, 0xFF, 0x8F]),
        le32(prefix, 4) == Some(1),
        (1..=16).contains(&le32(prefix, 8)?)
    );
    Some(Label::SelinuxPp)
}

/// Sercomm `sErCoMm` Router Firmware Image (`.bin`).
pub(crate) fn verify_sercomm_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const SERCOMM_MAGIC: &[u8; 8] = b"sErCoMm\0"; // @0: 8-byte alternating-case signature
    require!(
        size >= MIN_SERCOMM_FW_SIZE,
        prefix.len() >= 64,
        starts(prefix, SERCOMM_MAGIC),
        u8_at(prefix, 8)? <= 0x10
    );
    Some(Label::SercommFw)
}

/// Roche 454 Standard Flowgram Format (`.sff\0\0\0\x01`, big-endian): `header_length` must
/// equal `align8(31 + flows + key_length)` and the flowgram format code must be 1.
pub(crate) fn verify_sff(prefix: &[u8], size: u64) -> Option<Label> {
    const INDEX_OFFSET: usize = 8; // @8: u64 BE index_offset (0 = none)
    const INDEX_LENGTH: usize = 16; // @16: u32 BE index_length
    const HEADER_LENGTH: usize = 24; // @24: u16 BE header_length
    const KEY_LENGTH: usize = 26; // @26: u16 BE key_length
    const FLOWS: usize = 28; // @28: u16 BE number_of_flows_per_read
    const FORMAT_CODE: usize = 30; // @30: u8 flowgram_format_code == 1
    require!(size >= MIN_SFF_SIZE, starts(prefix, b".sff\x00\x00\x00\x01"));
    let index_offset = be64(prefix, INDEX_OFFSET)?;
    let index_length = be32(prefix, INDEX_LENGTH)? as u64;
    let header_length = be16(prefix, HEADER_LENGTH)? as u64;
    let key_length = be16(prefix, KEY_LENGTH)? as u64;
    let flows = be16(prefix, FLOWS)? as u64;
    require!(
        u8_at(prefix, FORMAT_CODE)? == 1,
        (1..=64).contains(&key_length),
        flows >= 1,
        header_length == (31 + flows + key_length).div_ceil(8) * 8,
        header_length <= size,
        index_offset == 0 || (index_offset >= header_length && index_offset + index_length <= size)
    );
    Some(Label::Sff)
}

/// SH3 R02.04 Smart Configuration Profile (.smart).
pub(crate) fn verify_sh3_smart(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SH3_SMART_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"SH3\0R02.04.00.00\0"),
        get_slice(prefix, 17, 33)? == [0u8; 16]
    );
    Some(Label::Sh3Smart)
}

/// Silicon Labs Gecko Bootloader Upgrade Image (`.gbl`) Silicon Labs.
pub(crate) fn verify_silabs_gbl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SILABS_GBL_SIZE,
        prefix.len() >= 24,
        starts(prefix, hex2bin!("EB 17 A6 03 08 00 00 00")),
        u8_at(prefix, 11)? == 0x03,
        matches!(u8_at(prefix, 19)?, 0xF4..=0xFE),
    );
    Some(Label::SilabsGbl)
}

/// Sound Open Firmware Audio DSP Image, Extended Manifest, or Debug Dictionary (`"$AE1"`, `"XMan"`, or `"Logs"`).
pub(crate) fn verify_sof_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SOF_FIRMWARE_SIZE, prefix.len() >= 32);
    if starts(prefix, b"$AE1") {
        let total_sz = le32(prefix, 4)? as u64;
        require!((40..=size).contains(&total_sz), le16(prefix, 8) == Some(1));
        return Some(Label::SofFirmware);
    }
    if starts(prefix, b"XMan") {
        let xman_sz = le32(prefix, 4)? as u64;
        require!((16..=size).contains(&xman_sz), le32(prefix, 8) == Some(16));
        return Some(Label::SofFirmware);
    }
    if starts(prefix, b"Logs") {
        require!(le32(prefix, 12) == Some(0x4C), le32(prefix, 16) == Some(0x3C));
        return Some(Label::SofFirmware);
    }
    None
}

/// Sophos Anti-Virus Virus Identity Definition (.ide, 1999) Sophos.
pub(crate) fn verify_sophos_ide(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SOPHOS_IDE_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"SOPH\x01\0"),
        get_slice(prefix, 22, 28)? == b"\x01\0TOC\0",
        get_slice(prefix, 36, 40)? == b"PLIS"
    );
    Some(Label::SophosIde)
}

/// Spreadtrum / Unisoc PAC Firmware Package (`.pac`).
pub(crate) fn verify_spreadtrum_pac(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SPREADTRUM_PAC_SIZE,
        prefix.len() >= 2124,
        starts(prefix, hex2bin!("42 00 50 00 5F 00 52 00")),
        eq_at(prefix, 2116, hex2bin!("FA FF FA FF"))
    );
    Some(Label::SpreadtrumPac)
}

/// Spreadtrum / Unisoc DHTB Signed Partition Image (`"DHTB\x01\0\0\0"`).
pub(crate) fn verify_spreadtrum_signed_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SPREADTRUM_DHTB_SIZE,
        prefix.len() >= 56,
        starts(prefix, b"DHTB\x01\0\0\0"),
        le64(prefix, 48)? > 0
    );
    Some(Label::SpreadtrumDhtbImg)
}

/// STMicroelectronics DfuSe USB Device Firmware Upgrade Image (.dfu).
pub(crate) fn verify_st_dfuse(prefix: &[u8], size: u64) -> Option<Label> {
    const IMG_SIZE_OFF: usize = 6; // @6: u32 LE DFUImageSize (total size excluding 16B suffix)
    const TARGETS_OFF: usize = 10; // @10: u8 number of targets (>= 1)
    const TARGET_SIG_OFF: usize = 11; // @11..17: "Target" signature

    require!(
        size >= MIN_ST_DFUSE_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"DfuSe\x01"),
        eq_at(prefix, TARGET_SIG_OFF, b"Target")
    );
    let img_size = le32(prefix, IMG_SIZE_OFF)? as u64;
    let targets = u8_at(prefix, TARGETS_OFF)?;
    require!(img_size >= 285, img_size <= size, targets >= 1);
    Some(Label::StDfuse)
}

/// SunOS 4.x XDR Distribution Tape Table of Contents (`.toc`, 1988 Sun Microsystems).
///
/// XDR Record-Marking header layout:
/// - `@0x00`: `u32 BE rm`: bit 31 (`0x8000_0000`) = last-fragment flag, bits `30..0` = record length (`24..=65536`)
/// - `@0x04`: `u32 BE 0x674D2309` (`TOC_MAGIC`)
/// - `@0x08`: `u32 BE version == 0`
/// - `@0x0C`: `u32 BE str_len` (`1..=64`) followed by printable ASCII OS distribution title (`"SunOS"`).
pub(crate) fn verify_sunos_toc(prefix: &[u8], size: u64) -> Option<Label> {
    const RM_OFF: usize = 0x00; // @0x00: u32 BE XDR record mark
    const MAGIC_OFF: usize = 0x04; // @0x04: u32 BE 0x674D2309
    const VER_OFF: usize = 0x08; // @0x08: u32 BE version 0
    const SLEN_OFF: usize = 0x0C; // @0x0C: u32 BE ASCII string length
    const STR_OFF: usize = 0x10; // @0x10: distribution name string

    require!(
        size >= MIN_SUNOS_TOC_SIZE,
        prefix.len() >= STR_OFF + 4,
        eq_at(prefix, MAGIC_OFF, SUNOS_TOC_MAGIC),
        be32(prefix, VER_OFF) == Some(0),
        has_non_ascii_text_byte(prefix.get(..STR_OFF)?)
    );
    let rm = be32(prefix, RM_OFF)?;
    let rec_len = (rm & 0x7FFF_FFFF) as u64;
    let slen = be32(prefix, SLEN_OFF)? as usize;
    require!(
        (rm & 0x8000_0000) != 0,
        (24..=0x1_0000).contains(&rec_len),
        rec_len + 4 <= size,
        (1..=64).contains(&slen),
        STR_OFF + slen <= prefix.len(),
        get_slice(prefix, STR_OFF, STR_OFF + slen)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::SunosToc)
}

/// Allwinner sunxi-package Boot Firmware Container (.fex, 2012) Allwinner Technology.
pub(crate) fn verify_sunxi_package(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SUNXI_PACKAGE_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"sunxi-package\0\0\0"),
        (1..=16).contains(&le32(prefix, 32).unwrap_or(0))
    );
    require!(has_non_ascii_text_byte(prefix.get(..44)?));
    Some(Label::SunxiPackage)
}

/// Linux systemd Structured Binary Journal Log (.journal).
pub(crate) fn verify_systemd_journal(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SYSTEMD_JOURNAL_SIZE
            && prefix.len() >= 96
            && starts(prefix, b"LPKSHHRH")
            && le64(prefix, 88).unwrap_or(0) >= 208
            && has_non_ascii_text_byte(prefix.get(8..32)?)
    );
    Some(Label::SystemdJournal)
}

/// Tektronix WFM Oscilloscope Waveform file (`.wfm`, WFM#001 / WFM#002 / WFM#003).
///
/// Header layout:
/// - `@0..2`: `0x0F0F` LE (`0F 0F`) or `0xF0F0` BE (`F0 F0`) byte-order verification word
/// - `@2..10`: `":WFM#001"`, `":WFM#002"`, or `":WFM#003"` version string
/// - `@10`: `u8 num_digits_in_byte_count` (`0..=9`)
/// - `@15`: `i8 bytes_per_point` (`1`, `2`, `4`, or `8`)
/// - `@16..20`: `u32 LE/BE byte_offset_to_curve_buffer` (`>= 78` and `<= size`)
pub(crate) fn verify_tektronix_wfm(prefix: &[u8], size: u64) -> Option<Label> {
    const TEK_WFM_LE_MAGIC: &[u8; 7] = bytes!(hex2bin!("0F0F"), b":WFM#"); // @0..7: 0x0F0F LE + ":WFM#"
    const TEK_WFM_BE_MAGIC: &[u8; 7] = bytes!(hex2bin!("F0F0"), b":WFM#"); // @0..7: 0xF0F0 BE + ":WFM#"
    const VER_DIGITS_OFFSET: usize = 7; // @7..10: "001", "002", or "003"
    const NUM_DIGITS_OFFSET: usize = 10; // @10: u8 number of digits in byte count (0..=9)
    const BYTES_PER_POINT_OFFSET: usize = 15; // @15: i8 bytes per point (1, 2, 4, or 8)
    const CURVE_OFFSET: usize = 16; // @16..20: u32 LE/BE byte offset to curve buffer (>= 78)

    let is_le = if starts(prefix, TEK_WFM_LE_MAGIC) {
        true
    } else if starts(prefix, TEK_WFM_BE_MAGIC) {
        false
    } else {
        return None;
    };
    require!(
        size >= MIN_TEKTRONIX_WFM_SIZE,
        prefix.len() >= 24,
        matches!(
            get_slice(prefix, VER_DIGITS_OFFSET, VER_DIGITS_OFFSET + 3)?,
            b"001" | b"002" | b"003"
        ),
        u8_at(prefix, NUM_DIGITS_OFFSET)? <= 9,
        matches!(u8_at(prefix, BYTES_PER_POINT_OFFSET), Some(1 | 2 | 4 | 8))
    );
    let curve_off =
        if is_le { le32(prefix, CURVE_OFFSET)? } else { be32(prefix, CURVE_OFFSET)? } as u64;
    require!((78..=size).contains(&curve_off));
    Some(Label::TekWfm)
}

/// Telix SALT Compiled Script (`.slc`, 1988 Exis / deltaComm).
///
/// Header layout:
/// - `@0..6`: `0B 1D 2B 1A 00 00` (4-byte magic + zero version/flags word)
/// - `@6..8`: `u16 LE code_size` (`> 0`, exact `12 + code_size == file_size`)
/// - `@8..10`: `u16 LE stack_size` (`> 0`, e.g. `0x0400`, `0x0800`, `0x0C00`)
/// - `@10..12`: `u16 LE entry_point` (`< code_size`)
pub(crate) fn verify_telix_salt_slc(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_SIZE: usize = 12; // 12-byte SALT bytecode header
    const CODE_SIZE_OFF: usize = 6; // @6: u16 LE bytecode length
    const STACK_SIZE_OFF: usize = 8; // @8: u16 LE runtime stack size
    const ENTRY_OFF: usize = 10; // @10: u16 LE main() entry offset
    const TELIX_BANNER_OFF: usize = 15; // @15..20: alternate Telix script/control banner
    const TELIX_BANNER: &[u8; 5] = b"Telix";

    require!(
        size >= MIN_TELIX_SALT_SLC_SIZE,
        prefix.len() >= HDR_SIZE,
        starts(prefix, TELIX_SLC_MAGIC)
    );
    let code_size = le16(prefix, CODE_SIZE_OFF)?;
    let stack_size = le16(prefix, STACK_SIZE_OFF)?;
    let entry = le16(prefix, ENTRY_OFF)?;
    let is_salt_v3 = code_size > 0
        && stack_size > 0
        && entry < code_size
        && (HDR_SIZE as u64) + (code_size as u64) == size;
    let is_telix_banner = prefix.len() >= TELIX_BANNER_OFF + TELIX_BANNER.len()
        && eq_at(prefix, TELIX_BANNER_OFF, TELIX_BANNER);
    require!(is_salt_v3 || is_telix_banner);
    Some(Label::TelixSaltSlc)
}

/// SRB 5A5A12AB Binary Container (`.srb`).
pub(crate) fn verify_thebat_srb(prefix: &[u8], size: u64) -> Option<Label> {
    const SRB_5A5A_MAGIC: &[u8; 20] =
        hex2bin!("5A 5A 12 AB 80 00 00 00 0C 00 00 00 07 00 00 00 00 00 00 00"); // @0..20: SRB container magic + header fields

    require!(size >= MIN_THEBAT_SRB_SIZE, prefix.len() >= 32, starts(prefix, SRB_5A5A_MAGIC));
    Some(Label::ThebatSrb)
}

/// Texas Instruments Application Image Script Boot Image (`.ais`) Texas Instruments.
pub(crate) fn verify_ti_ais(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_TI_AIS_SIZE && prefix.len() >= 16);
    if starts(prefix, b"TIPA") {
        let op = u8_at(prefix, 4)?;
        require!(eq_at(prefix, 5, b"YSX") && ((0x01..=0x0E).contains(&op) || op == 0x63));
        return Some(Label::TiAis);
    }
    if matches!(le32(prefix, 0)?, 0..=2) && eq_at(prefix, 4, b"TIPA") {
        let op = u8_at(prefix, 8)?;
        require!(eq_at(prefix, 9, b"YSX") && ((0x01..=0x0E).contains(&op) || op == 0x63));
        return Some(Label::TiAis);
    }
    None
}

/// Texas Instruments WiLink Bluetooth Init Script (`.bts`, `"BTSB\x01\0\0\0"` at `@0..8`).
pub(crate) fn verify_ti_bts_script(prefix: &[u8], size: u64) -> Option<Label> {
    const BTSB_MAGIC: &[u8; 8] = bytes!(b"BTSB", le32_bytes!(1)); // @0..8: "BTSB" + version 1 LE

    require!(
        size >= MIN_TI_BTS_SCRIPT_SIZE,
        prefix.len() >= 36,
        starts(prefix, BTSB_MAGIC),
        get_slice(prefix, 8, 32)? == [0u8; 24]
    );
    Some(Label::TiBtsScript)
}

/// Texas Instruments Multicore Meta Header Application Image (`.appimage`) Texas Instruments.
pub(crate) fn verify_ti_multicore(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_TI_MULTICORE_SIZE, prefix.len() >= 32, starts(prefix, b"MSTR"),);
    let num_files = le32(prefix, 4)? as usize;
    require!((1..=16).contains(&num_files));
    let mend_off_v1 = 12 + num_files * 8;
    let mend_off_v2 = 20 + num_files * 8;
    require!(
        (mend_off_v2 + 4 <= prefix.len() && eq_at(prefix, mend_off_v2, b"MEND"))
            || (mend_off_v1 + 4 <= prefix.len() && eq_at(prefix, mend_off_v1, b"MEND"))
    );
    Some(Label::TiMulticore)
}

/// Texas Instruments SimpleLink Over-the-Air Download Firmware Image (`.bin`) Texas Instruments.
pub(crate) fn verify_ti_oad(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TI_OAD_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"CC26x2R1")
            || starts(prefix, b"CC13x2R1")
            || starts(prefix, b"CC26X2R1")
            || starts(prefix, b"CC13X2R1")
            || starts(prefix, b"CC23x0R5")
            || starts(prefix, b"OAD IMG "),
        (1..=16).contains(&u8_at(prefix, 12)?),
        (1..=16).contains(&u8_at(prefix, 13)?),
        matches!(u8_at(prefix, 16)?, 0x00 | 0x02 | 0x03 | 0xFC | 0xFE | 0xFF),
        le32(prefix, 24)? >= 44,
        matches!(le16(prefix, 40)?, 44 | 52 | 60 | 128),
    );
    Some(Label::TiOad)
}

/// Texas Instruments Remote Processor Resource Core Firmware Image (`.rprc`) Texas Instruments.
pub(crate) fn verify_ti_rprc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TI_RPRC_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"RPRC"),
        (1..=64).contains(&le32(prefix, 12)?),
        (1..=16).contains(&le32(prefix, 16)?),
    );
    Some(Label::TiRprc)
}

/// Texas Instruments TAS2781 / TAS2563 Smart Amplifier DSP Firmware or RCA Config Binary (`.bin`).
pub(crate) fn verify_ti_smartamp_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const TAS_MAGIC: &[u8; 4] = hex2bin!("2A 9F 00 00"); // @0..4: TAS2781_BIN_MAGIC
    const RCA_INNER_TAG: &[u8; 8] = hex2bin!("00 00 01 05 00 00 01 01"); // @8..16: TI RCA version tag

    require!(size >= MIN_TI_SMARTAMP_FW_SIZE, prefix.len() >= 32);
    if starts(prefix, TAS_MAGIC)
        && (1..=16).contains(&le32(prefix, 4)?)
        && (1..=16).contains(&le32(prefix, 8)?)
    {
        return Some(Label::TiSmartampFw);
    }
    if starts(prefix, b"5552") && be32(prefix, 4)? as u64 == size {
        return Some(Label::TiSmartampFw);
    }
    if be32(prefix, 0)? as u64 == size && eq_at(prefix, 8, RCA_INNER_TAG) {
        return Some(Label::TiSmartampFw);
    }
    None
}

/// Dell / TI TPS65987 USB-C Monitor Firmware Upgrade (.upg, Texas Instruments).
pub(crate) fn verify_ti_tps_upg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TI_TPS_UPG_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"M2B10"),
        get_slice(prefix, 16, 30)? == b"\x02\x02\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff",
        get_slice(prefix, 33, 46)? == b"TI TPS65987DD"
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::TiTpsUpg)
}

/// Missler TopSolid MFC `CArchive` Resource Configuration file (`.rcf`).
///
/// Header layout:
/// - `@0..2`: `u16 LE version == 1` (`01 00`)
/// - `@2..4`: `u16 LE count` (`1..=4096`)
/// - `@4..8`: MFC `CArchive` `wNewClassTag = 0xFFFF` (`FF FF`) + `wSchema = 0x0000` (`00 00`)
/// - `@8..10`: `u16 LE class_len` (`8..=64`)
/// - `@10..10+class_len`: ASCII `CRuntimeClass` identifier starting with `"TS"` (e.g., `"TSElementManager"`, `"TSToolIconInfo"`)
pub(crate) fn verify_topsolid_rcf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TOPSOLID_RCF_SIZE,
        prefix.len() >= 32,
        le16(prefix, 0) == Some(1),
        (1..=4096).contains(&le16(prefix, 2)?),
        eq_at(prefix, TOPSOLID_MFC_TAG_OFFSET, TOPSOLID_MFC_NEW_CLASS_TAG),
        eq_at(prefix, TOPSOLID_CLASS_NAME_OFFSET, TOPSOLID_CLASS_PREFIX)
    );
    let class_len = le16(prefix, TOPSOLID_CLASS_LEN_OFFSET)? as usize;
    let class_end = TOPSOLID_CLASS_NAME_OFFSET + class_len;
    require!(
        (8..=64).contains(&class_len),
        class_end <= prefix.len(),
        get_slice(prefix, TOPSOLID_CLASS_NAME_OFFSET, class_end)?
            .iter()
            .all(|&b| b.is_ascii_alphanumeric() || b == b'_')
    );
    Some(Label::TopsolidRcf)
}

/// Toyota / Lexus Techstream Calibration Update Wizard Package (`.cuw`) Toyota.
pub(crate) fn verify_toyota_cuw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TOYOTA_CUW_SIZE,
        prefix.len() >= 34,
        starts(prefix, b"\0CALIBRATION\0"),
        matches!(u8_at(prefix, 13)?, 0x01 | 0x02 | 0x31 | 0x32),
        be32(prefix, 14)? != 0,
        be32(prefix, 18)? >= 64,
        eq_at(prefix, 22, b"\x00\x0Aattach.att"),
    );
    Some(Label::ToyotaCuw)
}

/// TP-LINK Router Firmware Image (`.bin`).
pub(crate) fn verify_tplink_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const VENDOR_OFF: usize = 4; // @4..11: "TP-LINK" (mktplinkfw) or "ver. 2.0\0" / "ver. 3.0\0" (mktplinkfw2)
    require!(
        size >= MIN_TPLINK_FW_SIZE,
        prefix.len() >= 64,
        eq_at(prefix, VENDOR_OFF, b"TP-LINK")
            || eq_at(prefix, VENDOR_OFF, b"ver. 2.0\0")
            || eq_at(prefix, VENDOR_OFF, b"ver. 3.0\0")
    );
    let ver_le = le32(prefix, 0)?;
    let ver_be = be32(prefix, 0)?;
    require!((1..=3).contains(&ver_le) || (1..=3).contains(&ver_be));
    Some(Label::TplinkFw)
}

/// Nadeo GameBox Binary Asset / TrackMania Map (`.gbx`).
///
/// Header layout:
/// - `@0..3`: `"GBX"` magic
/// - `@3..5`: `u16 LE` `version` (`3..=6`)
/// - `@5`: format (`'B'` binary or `'T'` text)
/// - `@6`: ref-table compression (`'U'` uncompressed or `'C'` compressed)
/// - `@7`: body compression (`'U'` uncompressed or `'C'` compressed)
pub(crate) fn verify_trackmania_gbx(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 3] = b"GBX"; // @0..3: GameBox magic
    const VER_OFF: usize = 3; // @3..5: u16 LE version (3..=6)
    const FMT_OFF: usize = 5; // @5: 'B' (binary) or 'T' (text)
    const REF_COMP_OFF: usize = 6; // @6: 'U' or 'C'
    const BODY_COMP_OFF: usize = 7; // @7: 'U' or 'C'

    require!(
        size >= MIN_TRACKMANIA_GBX_SIZE,
        prefix.len() >= 13,
        starts(prefix, MAGIC),
        (3..=6).contains(&le16(prefix, VER_OFF)?),
        matches!(u8_at(prefix, FMT_OFF)?, b'B' | b'T'),
        matches!(u8_at(prefix, REF_COMP_OFF)?, b'U' | b'C'),
        matches!(u8_at(prefix, BODY_COMP_OFF)?, b'U' | b'C')
    );
    Some(Label::TrackmaniaGbx)
}

/// Tresorit Encrypted Storage Profile Backup (.backup).
pub(crate) fn verify_tresorit_profile_backup(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TRESORIT_PROFILE_BACKUP_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\xff\x64\x80\xff\x65\x80\x06\x0b\x2b\x06\x01\x04\x01\x82\xc9\x7b\x04\x01\x01\x06\x0b\x2b\x06\x01\x04\x01\x82\xc9\x7b\x04")
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::TresoritProfileBackup)
}

/// TRPH Mobile Printer Configuration (.xpd).
pub(crate) fn verify_trph_xpd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TRPH_XPD_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"TRPH\xc0\0\0\0\x31\x01\0\0M\0O\0B\0I\0L\0E\0\0\0\0\0")
    );
    Some(Label::TrphXpd)
}

/// TCW40 Controller Configuration (.xcfg).
pub(crate) fn verify_ttc_xcfg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TTC_XCFG_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"TCW40-CONFIG_V."),
        get_slice(prefix, 17, 20)? == b"\0\0\0"
    );
    Some(Label::TtcXcfg)
}

/// TuneUp Utilities Installation/Undo Log (.tun, 2004) TuneUp Software.
pub(crate) fn verify_tuneup_tun(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TUNEUP_TUN_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"T\0U\0N\0_\0S\0Y\0S\0T\0E\0M\0\x05\0\0\0\0\x08\0\0"),
        get_slice(prefix, 32, 36)? == b"$BXM"
    );
    Some(Label::TuneupTun)
}

/// Turbo Vision Resource Workshop Project file (`.rwp`, 1993, BitSoft / Borland).
///
/// Mirrors Borland `TResourceFile`'s `"FBPR"` container with `"TVRW"` at `@0..4` and
/// `@4..8 u32 LE index_pos` pointing to the serialized `TResourceCollection` directory
/// (`u16 count`, `u16 limit`, `u16 delta`).
pub(crate) fn verify_turvision_rwp(prefix: &[u8], size: u64) -> Option<Label> {
    const INDEX_POS_OFF: usize = 4; // @4..8: u32 LE TResourceCollection directory offset

    require!(size >= MIN_TVRW_PROJECT_SIZE, prefix.len() >= 12, starts(prefix, TVRW_PROJECT_MAGIC));
    let idx_pos = le32(prefix, INDEX_POS_OFF)? as u64;
    require!(idx_pos >= 12, idx_pos + 6 <= size);
    if idx_pos + 6 <= prefix.len() as u64 {
        let count = le16(prefix, idx_pos as usize)?;
        let limit = le16(prefix, idx_pos as usize + 2)?;
        require!(limit >= count, limit <= 4096);
    }
    require!(has_non_ascii_text_byte(prefix.get(..prefix.len().min(64))?));
    Some(Label::TurvisionRwp)
}

/// IANA Time Zone Information Binary Format (TZif).
pub(crate) fn verify_tzif(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TZIF_SIZE
            && prefix.len() >= 44
            && starts(prefix, b"TZif")
            && matches!(u8_at(prefix, 4)?, 0 | b'2' | b'3' | b'4')
            && get_slice(prefix, 5, 20)?.iter().all(|&b| b == 0)
            && be32(prefix, 36).unwrap_or(0) > 0
            && be32(prefix, 40).unwrap_or(0) > 0
    );
    Some(Label::Tzif)
}

/// Ubicom IP3K / IP5K / IP7K Network Processor Firmware Image (`.bin`).
pub(crate) fn verify_ubicom_firmware(prefix: &[u8], size: u64) -> Option<Label> {
    const UBICOM_PREAMBLE: &[u8; 4] = hex2bin!("FA 32 00 80"); // @0: Ubicom boot vector word
    const UBICOM_MAGIC_OFF: usize = 12; // @12: 0xD3FAAAB6 BE magic
    require!(
        size >= MIN_UBICOM_FW_SIZE,
        prefix.len() >= 32,
        starts(prefix, UBICOM_PREAMBLE),
        eq_at(prefix, UBICOM_MAGIC_OFF, hex2bin!("D3 FA AA B6"))
    );
    Some(Label::UbicomFw)
}

/// Ubiquiti airOS / EdgeOS Firmware Image (`.bin`).
pub(crate) fn verify_ubiquiti_ubnt(prefix: &[u8], size: u64) -> Option<Label> {
    const UBNT_MAGIC: &[u8; 4] = b"UBNT"; // @0: "UBNT" magic
    const VER_STR_OFF: usize = 4; // @4..16: printable ASCII version string (e.g. "SW.ar7240...")
    const CRC_OFF: usize = 260; // @260: u32 BE header CRC-32 (!= 0)
    require!(
        size >= MIN_UBIQUITI_FW_SIZE,
        prefix.len() >= 264,
        starts(prefix, UBNT_MAGIC),
        get_slice(prefix, VER_STR_OFF, VER_STR_OFF + 12)?.iter().all(|&b| b.is_ascii_graphic()),
        be32(prefix, CRC_OFF)? != 0,
        has_non_ascii_text_byte(prefix.get(..264)?)
    );
    Some(Label::UbiquitiUbnt)
}

/// UEFI Update Capsule / AMI Aptio Signed Firmware Capsule (`.cap` / `.bin`) UEFI Forum.
pub(crate) fn verify_uefi_capsule(prefix: &[u8], size: u64) -> Option<Label> {
    const EFI_FMP_CAPSULE_GUID: &[u8; 16] =
        hex2bin!("BD 86 66 3B 76 0D 30 40 B7 0E B5 51 9E 2F C5 A0");
    const UEFI_CAPSULE_GUID: &[u8; 16] =
        hex2bin!("B9 82 91 53 B5 AB 91 43 B6 9A E3 A9 43 F7 2F CC");
    const WINDOWS_UX_CAPSULE_GUID: &[u8; 16] =
        hex2bin!("01 4D 8B 3B 90 B0 23 47 84 D3 AE 32 31 FC C6 E1");
    const APTIO_SIGNED_CAPSULE_GUID: &[u8; 16] =
        hex2bin!("8B A6 3C 4A 23 77 FB 48 80 3D 57 8C C1 FE C4 4D");
    const APTIO_UNSIGNED_CAPSULE_GUID: &[u8; 16] =
        hex2bin!("C9 3E B1 14 63 B9 9F 41 B7 7C 01 9A 24 91 D3 7E");
    require!(
        size >= MIN_UEFI_CAPSULE_SIZE,
        prefix.len() >= 28,
        starts(prefix, EFI_FMP_CAPSULE_GUID)
            || starts(prefix, UEFI_CAPSULE_GUID)
            || starts(prefix, WINDOWS_UX_CAPSULE_GUID)
            || starts(prefix, APTIO_SIGNED_CAPSULE_GUID)
            || starts(prefix, APTIO_UNSIGNED_CAPSULE_GUID),
    );
    let hdr_size = le32(prefix, 16)?;
    let flags = le32(prefix, 20)?;
    let img_size = le32(prefix, 24)?;
    require!(
        (28..=0x10_0000).contains(&hdr_size) && (flags & 0x0000_FFFE) == 0 && img_size >= hdr_size
    );
    Some(Label::UefiCapsule)
}

/// Unity Editor Library Asset Preview Descriptor (PreviewAssetData, .info).
pub(crate) fn verify_unity_preview_info(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UNITY_PREVIEW_INFO_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"PreviewAssetData\0\0\0\0BaseName:\n")
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::UnityPreviewInfo)
}

/// Epic Games Unreal Engine BuildPatch Binary Manifest.
pub(crate) fn verify_unreal_buildpatch_manifest(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UNREAL_BUILDPATCH_SIZE,
        prefix.len() >= 41,
        starts(prefix, hex2bin!("0C C0 BE 44"))
    );
    let hdr_size = le32(prefix, 4)?;
    let uncomp = le32(prefix, 8)?;
    let comp = le32(prefix, 12)?;
    let stored_as = u8_at(prefix, 36)?;
    require!(
        (37..=128).contains(&hdr_size),
        uncomp > 0,
        comp > 0,
        stored_as <= 7,
        (hdr_size as u64) + (comp as u64) <= size
    );
    Some(Label::UnrealManifest)
}

/// Epic Games Unreal Engine GVAS SaveGame (.sav).
pub(crate) fn verify_unreal_gvas(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_UNREAL_GVAS_SIZE, prefix.len() >= 32, starts(prefix, b"GVAS"));
    let sg_ver = le32(prefix, 4)?;
    let pkg_ver = le32(prefix, 8)?;
    // Note: in UE5 SaveGameFileVersion==3, @12..16 is PackageFileUE5Version before FEngineVersion at @16..22
    let eng_major_12 = le16(prefix, 12).unwrap_or(0);
    let eng_major_16 = le16(prefix, 16).unwrap_or(0);
    require!(
        (1..=4).contains(&sg_ver),
        (100..=2000).contains(&pkg_ver),
        (4..=6).contains(&eng_major_12) || (4..=6).contains(&eng_major_16)
    );
    Some(Label::UnrealGvas)
}

/// Epic Games Unreal Engine Localization Metadata (.locmeta).
pub(crate) fn verify_unreal_locmeta(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 16] = hex2bin!("4F EE 4C A1 68 48 55 83 6C 4C 46 BD 70 DA 50 7C");
    const MAGIC_ALT: &[u8; 16] = hex2bin!("4F EE 4C A1 68 83 55 4E B1 F3 DB 4D B4 E3 C2 1A");
    require!(
        size >= MIN_UNREAL_LOCMETA_SIZE,
        prefix.len() >= 25,
        starts(prefix, MAGIC) || starts(prefix, MAGIC_ALT)
    );
    let ver = u8_at(prefix, 16)?;
    let str_len = le32(prefix, 17)? as i32;
    let str_bytes = if (1..=64).contains(&str_len) {
        str_len as usize
    } else if (-64..=-1).contains(&str_len) {
        ((-str_len) as usize) * 2
    } else {
        return None;
    };
    require!(ver <= 2, (21usize.saturating_add(str_bytes).saturating_add(4) as u64) <= size);
    Some(Label::UnrealLocmeta)
}

/// Epic Games Unreal Engine Localization Resource (.locres).
pub(crate) fn verify_unreal_locres(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 16] = hex2bin!("0E 14 74 75 67 4A 03 FC 4A 15 90 9D C3 37 7F 1B");
    const MAGIC_ALT: &[u8; 16] = hex2bin!("0E 14 74 75 6C 4A 03 FC 4A 15 90 9D C3 37 7F 1B");
    require!(
        size >= MIN_UNREAL_LOCRES_SIZE,
        prefix.len() >= 25,
        starts(prefix, MAGIC) || starts(prefix, MAGIC_ALT)
    );
    let ver = u8_at(prefix, 16)?;
    let str_arr_off = le64(prefix, 17)?;
    require!(
        (1..=3).contains(&ver),
        str_arr_off == u64::MAX || (str_arr_off >= 25 && str_arr_off <= size)
    );
    Some(Label::UnrealLocres)
}

/// Epic Games Unreal Engine Pipeline State Object Cache (.upipelinecache).
pub(crate) fn verify_unreal_pipeline_cache(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UNREAL_PIPELINE_CACHE_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"HCACEPIP")
    );
    let ver = le32(prefix, 8)?;
    let platform = le32(prefix, 16)?;
    let toc_off_36 = le64(prefix, 36).unwrap_or(0);
    let toc_off_40 = le64(prefix, 40).unwrap_or(0);
    require!(
        (1..=64).contains(&ver),
        platform <= 512,
        (toc_off_36 >= 36 && toc_off_36 <= size) || (toc_off_40 >= 40 && toc_off_40 <= size)
    );
    Some(Label::UnrealPsoCache)
}

/// Epic Games Unreal Insights Trace Capture (.utrace).
pub(crate) fn verify_unreal_utrace(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UNREAL_UTRACE_SIZE,
        prefix.len() >= 16,
        matches!(get_slice(prefix, 0, 4)?, b"TRCE" | b"TRC2" | b"ECRT" | b"2CRT")
    );
    let meta_size = le16(prefix, 4)? as usize;
    let ver_off = 6usize.checked_add(meta_size)?;
    require!(meta_size <= 256, ver_off + 6 <= prefix.len(), (ver_off as u64) + 6 <= size);
    let transport_ver = u8_at(prefix, ver_off)?;
    let proto_ver = u8_at(prefix, ver_off + 1)?;
    let first_pkt_size = le16(prefix, ver_off + 2)?;
    require!((1..=4).contains(&transport_ver), proto_ver <= 8, first_pkt_size >= 4);
    Some(Label::UnrealUtrace)
}

/// DMR Radio Codeplug Data (V129 / V100, .rdt).
pub(crate) fn verify_v129_rdt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_V129_RDT_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"V129"),
        matches!(u8_at(prefix, 4)?, b'0'..=b'2'),
        u8_at(prefix, 9)? == b'D',
        get_slice(prefix, 19, 27)? == b"V100\x01\0\0\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::V129Rdt)
}

/// Vim swap block 0: `b0_id` = `"b0"`, `b0_version` = `"VIM <ver>"`, `b0_page_size` (`u32` @12, native endian,
/// power of two 1 KiB..64 KiB); the file is a whole number of pages.
pub(crate) fn verify_vim_swap(prefix: &[u8], size: u64) -> Option<Label> {
    const PAGE_SIZE: usize = 12; // @12: u32 in the writer's byte order
    require!(size >= MIN_VIM_SWAP_SIZE, starts(prefix, b"b0VIM "));
    let le = le32(prefix, PAGE_SIZE)?;
    let page = if le.is_power_of_two() && (1024..=65536).contains(&le) {
        le
    } else {
        be32(prefix, PAGE_SIZE)?
    };
    require!(
        page.is_power_of_two(),
        (1024..=65536).contains(&page),
        size.is_multiple_of(page as u64)
    );
    Some(Label::VimSwap)
}

/// VMware Virtual Machine NVRAM file (`"MRVN"`).
pub(crate) fn verify_vmware_nvram(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 16,
        prefix.len() >= 16,
        starts(prefix, b"MRVN"),
        prefix.get(4..16)?.contains(&0),
    );
    Some(Label::VmwareNvram)
}

/// VTech Multi-Section Install Log Container (.vrp, [HeadLen]).
pub(crate) fn verify_vtech_vrp_log(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VTECH_VRP_LOG_SIZE,
        prefix.len() >= 148,
        get_slice(prefix, 0, 24)?
            == hex2bin!("5B 48 65 61 64 4C 65 6E 5D 0D 0A 73 69 7A 65 3D 31 32 31 0D 0A 5B 5F 5F"),
        get_slice(prefix, 45, 69)?
            == hex2bin!("0D 0A 5B 5F 5F 64 61 6C 6C 6F 67 2E 74 78 74 5D 0D 0A 73 69 7A 65 3D 31"),
        get_slice(prefix, 72, 96)?
            == hex2bin!("0D 0A 5B 5F 5F 73 6C 77 6C 6F 67 2E 74 78 74 5D 0D 0A 73 69 7A 65 3D 30")
    );
    require!(has_non_ascii_text_byte(prefix.get(..148)?));
    Some(Label::VtechVrpLog)
}

/// Volkswagen / Audi ODIS / VAS-PC SGML Object ECU Flash Container (`.sgo`) Volkswagen.
pub(crate) fn verify_vw_sgo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VW_SGO_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"SGML Object File"),
        be16(prefix, 0x10)? == 0x0002,
        u8_at(prefix, 0x18)? >= 1,
        u8_at(prefix, 0x19)? == 0x31,
    );
    Some(Label::VwSgo)
}

/// Blizzard Entertainment Warcraft III Recorded Game Replay (`.w3g`).
///
/// Header layout:
/// - `@0..28`: `"Warcraft III recorded game\x1a\0"`
/// - `@28..32`: `u32 LE` `header_size` (`0x40` = 64 for v0, `0x44` = 68 for v1)
pub(crate) fn verify_warcraft3_replay(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 28] = bytes!(b"Warcraft III recorded game", &[0x1A, 0]); // @0..28
    const HDR_SIZE_OFF: usize = 28; // @28..32: u32 LE header size (0x40 or 0x44)

    require!(
        size >= MIN_WARCRAFT3_REPLAY_SIZE,
        prefix.len() >= 32,
        starts(prefix, MAGIC),
        matches!(le32(prefix, HDR_SIZE_OFF)?, 0x40 | 0x44)
    );
    Some(Label::Warcraft3Replay)
}

/// Nintendo Wii Gecko / Ocarina Cheat Code Table (`.gct`, 2008 Gecko OS / WiiBrew).
///
/// Header layout:
/// - `@0..8`: `00 D0 C0 DE 00 D0 C0 DE` (`GCT_MAGIC8_NUM`)
/// - 8-byte aligned cheat code pairs (`size >= 16` and `size % 8 == 0`)
/// - `@(size-8)..size`: `F0 00 00 00 00 00 00 00` (`GCT_TERM_NUM`, when `size <= prefix.len()`)
pub(crate) fn verify_wii_gecko_gct(prefix: &[u8], size: u64) -> Option<Label> {
    const GECKO_HDR: &[u8; 8] = hex2bin!("00 D0 C0 DE 00 D0 C0 DE");
    const GECKO_TERM: &[u8; 8] = hex2bin!("F0 00 00 00 00 00 00 00");

    require!(
        size >= MIN_WII_GECKO_GCT_SIZE,
        size.is_multiple_of(8),
        prefix.len() >= 8,
        starts(prefix, GECKO_HDR)
    );
    if size <= prefix.len() as u64 {
        let term_off = (size as usize) - 8;
        require!(eq_at(prefix, term_off, GECKO_TERM));
    }
    Some(Label::WiiGeckoGct)
}

/// Microsoft Windows XP System Restore Point log (`rp.log`).
pub(crate) fn verify_win_restore_rp_log(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size == MIN_WIN_RESTORE_RP_LOG_SIZE,
        prefix.len() >= 64,
        matches!(le32(prefix, 0), Some(100..=103)),
        le32(prefix, 4)? <= 18,
        le64(prefix, 8)? <= 0x0010_0000,
        u8_at(prefix, 16)?.is_ascii_graphic() && u8_at(prefix, 17)? == 0,
        u8_at(prefix, 18)?.is_ascii_graphic() && u8_at(prefix, 19)? == 0
    );
    Some(Label::WinRestoreRpLog)
}

/// Microsoft Windows Vista / 7 SuperFetch / ReadyBoot XPRESS Huffman stream (`AgGlGlobalHistory.db`, `"MEM0"` / `"MEMO"`).
pub(crate) fn verify_win_superfetch(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WIN_SUPERFETCH_SIZE,
        prefix.len() >= 32,
        matches!(get_slice(prefix, 0, 4)?, b"MEM0" | b"MEMO")
    );
    let uncomp_size = le32(prefix, 4)?;
    let chunk_size = le32(prefix, 8)?;
    require!(
        uncomp_size >= 16,
        (16..=65536).contains(&chunk_size),
        uncomp_size >= chunk_size,
        has_non_ascii_text_byte(prefix.get(12..32)?)
    );
    Some(Label::WinSuperfetch)
}

/// Microsoft Windows Task Scheduler Job Object (.job, 1997) Microsoft `FIXDLEN_DATA`.
pub(crate) fn verify_win_task_job(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WIN_TASK_JOB_SIZE,
        prefix.len() >= 72,
        matches!(
            le16(prefix, 0),
            Some(0x0400 | 0x0500 | 0x0501 | 0x0502 | 0x0600 | 0x0601 | 0x0602 | 0x0603 | 0x0A00)
        ),
        le16(prefix, 2) == Some(1),
        le16(prefix, 20) == Some(0x0046),
        matches!(le16(prefix, 52)?, 0 | 1980..=2100),
        le16(prefix, 54)? <= 12,
        le16(prefix, 58)? <= 31,
        le16(prefix, 66)? <= 999
    );
    let app_name_len = le16(prefix, 0x46)? as usize;
    require!(
        app_name_len == 0 || (size >= (0x48 + app_name_len * 2) as u64 && app_name_len <= 1024),
        has_non_ascii_text_byte(prefix.get(..72)?)
    );
    Some(Label::WinTaskJob)
}

/// Microsoft Windows CE ROM Binary Image Data Format (`.nb0` / `.bin`).
pub(crate) fn verify_wince_b000ff(prefix: &[u8], size: u64) -> Option<Label> {
    const B000FF_MAGIC: &[u8; 7] = b"B000FF\n"; // @0: 7-byte BLCommon signature
    const IMG_START_OFF: usize = 7; // @7: u32 LE image base address (> 0)
    const IMG_LEN_OFF: usize = 11; // @11: u32 LE total image span (> 0)
    const REC_ADDR_OFF: usize = 15; // @15: u32 LE first record destination address (>= image_start)
    const ECEC_OFF: usize = 64; // @64 (0x40): ROM_SIGNATURE ("ECEC" = 0x43454345) in raw .nb0 ROM images
    if size >= MIN_WINCE_B000FF_SIZE && prefix.len() >= 27 && starts(prefix, B000FF_MAGIC) {
        let img_start = le32(prefix, IMG_START_OFF)?;
        let img_len = le32(prefix, IMG_LEN_OFF)?;
        let rec_addr = le32(prefix, REC_ADDR_OFF)?;
        require!(img_start > 0, img_len > 0, rec_addr >= img_start);
        return Some(Label::WinceB000ff);
    }
    if size >= 128 && prefix.len() >= 76 && eq_at(prefix, ECEC_OFF, b"ECEC") {
        let ptoc = le32(prefix, 68)?;
        let ptoc_off = le32(prefix, 72)?;
        require!(ptoc > 0x0001_0000, ptoc_off >= 76, (ptoc & 0x0FFF) == (ptoc_off & 0x0FFF));
        return Some(Label::WinceB000ff);
    }
    None
}

/// Microsoft Windows 3.1 Registration Database (`REG.DAT`, `"SHCC3.10"`).
pub(crate) fn verify_windows_31_registry(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WIN31_REG_DAT_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"SHCC3.10"),
        has_non_ascii_text_byte(prefix.get(8..16)?)
    );
    Some(Label::Win31RegDat)
}

/// Microsoft Windows Component-Based Servicing Compiled Definition Form file (`.cdf-ms`).
pub(crate) fn verify_windows_cdf_ms(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WINDOWS_CDF_MS_SIZE,
        prefix.len() >= 28,
        starts(prefix, WINDOWS_CDF_MS_MAGIC),
        le32(prefix, CDF_MS_SIZE_OFFSET)? >= 16
    );
    Some(Label::WindowsCdfMs)
}

/// Microsoft Windows National Language Support Codepage Table (`C_*.NLS`, 1993) Microsoft.
pub(crate) fn verify_windows_codepage_nls(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WINDOWS_CODEPAGE_NLS_SIZE,
        prefix.len() >= 28,
        starts(prefix, &[0x0D, 0x00]),
        le16(prefix, 2)? > 0,
        matches!(le16(prefix, 4)?, 1 | 2),
        eq_at(prefix, 6, &[0x3F, 0x00, 0x3F, 0x00])
    );
    Some(Label::WindowsCodepageNls)
}

/// Windows NT/2000/XP Legacy Event Log (.evt).
pub(crate) fn verify_windows_evt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WINDOWS_EVT_SIZE
            && prefix.len() >= 48
            && le32(prefix, 0) == Some(0x30)
            && get_slice(prefix, 4, 8)? == b"LfLe"
            && le32(prefix, 8) == Some(1)
            && le32(prefix, 44) == Some(0x30)
    );
    Some(Label::Evt)
}

/// Windows XML Event Log (.evtx).
pub(crate) fn verify_windows_evtx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WINDOWS_EVTX_SIZE
            && prefix.len() >= 44
            && starts(prefix, b"ElfFile\0")
            && ((le32(prefix, 32) == Some(128) && le16(prefix, 40) == Some(4096))
                || le32(prefix, 32) == Some(4096))
            && le16(prefix, 38) == Some(3)
    );
    Some(Label::Evtx)
}

/// Microsoft Windows 3.x Program Manager Group (.grp).
pub(crate) fn verify_windows_grp(prefix: &[u8], size: u64) -> Option<Label> {
    const CB_GROUP_OFF: usize = 6; // @6: u16 LE size of base group structure (<= file size)
    const CMD_SHOW_OFF: usize = 8; // @8: u16 LE window show state (0..=11)
    const P_NAME_OFF: usize = 22; // @22: u16 LE offset to NUL-terminated ASCII group title
    const C_ITEMS_OFF: usize = 32; // @32: u16 LE count of item offset pointers at @34

    require!(size >= MIN_WINDOWS_GRP_SIZE, prefix.len() >= 36, starts(prefix, b"PMCC"));
    let cb_group = le16(prefix, CB_GROUP_OFF)? as u64;
    let cmd_show = le16(prefix, CMD_SHOW_OFF)?;
    let p_name = le16(prefix, P_NAME_OFF)? as u64;
    let c_items = le16(prefix, C_ITEMS_OFF)? as u64;
    require!(
        cb_group >= MIN_WINDOWS_GRP_SIZE,
        cb_group <= size,
        cmd_show <= 11,
        p_name >= 34 + 2 * c_items,
        p_name < cb_group
    );
    Some(Label::WinGrp)
}

/// Microsoft Windows 95 / NT / 2000 Input Method Editor compiled dictionary table (`.mb`).
///
/// Header layout:
/// - `@0..8`: `"95.0"` in UTF-16LE (`39 00 35 00 2E 00 30 00`)
/// - `@8..24`: Four `u32 LE` version/format constants (`1`, `2`, `1`, `7`)
/// - `@24..28`: `u32 LE` max key strokes (`1..=16`, e.g. `4` for Wubi / Cangjie /鄭碼, `6` for Array / Boshiamy)
/// - `@38..42`: Non-zero UTF-16LE IME description title.
pub(crate) fn verify_windows_ime_mb(prefix: &[u8], size: u64) -> Option<Label> {
    const IME_MB_MAGIC: &[u8; 24] = bytes!(
        b"9\x00",
        b"5\x00",
        b".\x00",
        b"0\x00",
        le32_bytes!(1),
        le32_bytes!(2),
        le32_bytes!(1),
        le32_bytes!(7)
    ); // @0..24: "95.0" UTF-16LE + [1, 2, 1, 7] u32 LE header
    const DICT_COUNT_OFFSET: usize = 24; // @24..28: u32 LE dictionary entry count (> 0)
    const SEC1_ID_OFFSET: usize = 28; // @28..32: u32 LE section 1 ID (1) or max key length

    require!(
        size >= MIN_WINDOWS_IME_MB_SIZE,
        prefix.len() >= 64,
        starts(prefix, IME_MB_MAGIC),
        le32(prefix, DICT_COUNT_OFFSET)? > 0,
        le32(prefix, SEC1_ID_OFFSET)? > 0
    );
    Some(Label::WindowsImeMb)
}

/// Microsoft Windows Installer Binary Execution / Rollback Script (`.ibs` / `.rbs`).
///
/// Header layout:
/// - `@0`: `0x02` (`ixoHeader` opcode)
/// - `@1`: `0x09` (9 typed arguments in the header record)
/// - `@2..4`: `0x4000` LE (`iasString` type tag for `Signature`, length 0x00004 = 4 ASCII chars)
/// - `@4..8`: `"IXOS"` (`0x534F5849` LE script signature)
/// - `@8..10`: `0x4000` or `0xC000` LE (next argument type tag)
pub(crate) fn verify_windows_installer_script(prefix: &[u8], size: u64) -> Option<Label> {
    const IBS_HEADER_MAGIC: &[u8; 8] = bytes!(&[0x02, 0x09], le16_bytes!(0x4000), b"IXOS"); // @0..8: ixoHeader(9 args) + "IXOS"
    const ARG2_TYPE_OFFSET: usize = 8; // @8..10: u16 LE second argument type tag (0x4000, 0x8000, or 0xC000)

    require!(
        size >= MIN_WINDOWS_INSTALLER_SCRIPT_SIZE,
        prefix.len() >= 32,
        starts(prefix, IBS_HEADER_MAGIC),
        matches!(le16(prefix, ARG2_TYPE_OFFSET), Some(0x4000 | 0x8000 | 0xC000))
    );
    Some(Label::WindowsInstallerScript)
}

/// Windows Kernel Memory Crash Dump (PAGEDUMP / PAGEDU64).
pub(crate) fn verify_windows_kernel_dump(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WINDOWS_KERNEL_DUMP_SIZE
            && prefix.len() >= 16
            && starts(prefix, b"PAGE")
            && matches!(get_slice(prefix, 4, 8)?, b"DUMP" | b"DU64" | b"HDMP")
            && matches!(le32(prefix, 8), Some(0x0F | 0x0C))
            && le32(prefix, 12).unwrap_or(0) > 0
    );
    Some(Label::WinKdump)
}

/// Microsoft Windows Precompiled Setup Information (.pnf, 2001) Microsoft.
pub(crate) fn verify_windows_pnf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WINDOWS_PNF_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"\x04\x03\x02\0\x83\0\0\x01\0\0\0\0\0\0"),
        matches!(le16(prefix, 14), Some(6 | 8)),
        matches!(le32(prefix, 40), Some(0x80 | 0x90))
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::WindowsPnf)
}

/// Microsoft Windows 8 / 10 / 11 Package Resource Index file (`.pri`, `"mrm_pri0"` / `"mrm_pri1"` / `"mrm_pri2"` / `"mrm_prif"`).
pub(crate) fn verify_windows_pri(prefix: &[u8], size: u64) -> Option<Label> {
    const MAJOR_OFFSET: usize = 8; // @8..10: u16 LE major version (0)
    const MINOR_OFFSET: usize = 10; // @10..12: u16 LE minor version (1)
    const TOTAL_SIZE_OFFSET: usize = 12; // @12..16: u32 LE total PRI file size (== size)
    const TOC_OFF_OFFSET: usize = 16; // @16..20: u32 LE table-of-contents offset (0x1E = 30)
    const NUM_SECTIONS_OFFSET: usize = 24; // @24..26: u16 LE number of sections (>= 1)
    const SENTINEL_OFFSET: usize = 26; // @26..30: u32 LE 0x00FFFFFF header sentinel

    require!(
        size >= MIN_WINDOWS_PRI_SIZE,
        prefix.len() >= 30,
        starts(prefix, b"mrm_pri0")
            || starts(prefix, b"mrm_pri1")
            || starts(prefix, b"mrm_pri2")
            || starts(prefix, b"mrm_prif"),
        le16(prefix, MAJOR_OFFSET) == Some(0),
        le16(prefix, MINOR_OFFSET) == Some(1),
        le32(prefix, TOTAL_SIZE_OFFSET)? as u64 <= size,
        matches!(le32(prefix, TOC_OFF_OFFSET), Some(0x1E | 0x20)),
        le16(prefix, NUM_SECTIONS_OFFSET)? >= 1,
        le16(prefix, SENTINEL_OFFSET) == Some(0xFFFF)
    );
    Some(Label::WindowsPri)
}

/// Microsoft Windows System Deployment Image (`.sdi`, `"$SDI0001"`).
pub(crate) fn verify_windows_sdi_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WINDOWS_SDI_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"$SDI0001"),
        matches!(le64(prefix, 8), Some(0..=3))
    );
    Some(Label::WindowsSdi)
}

/// Linux CRDA / cfg80211 Wireless Regulatory Database (`regulatory.db`, `"RGDB"` at `@0..4`).
pub(crate) fn verify_wireless_regdb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WIRELESS_REGDB_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"RGDB"),
        (19..=22).contains(&be32(prefix, 4)?)
    );
    Some(Label::WirelessRegdb)
}

/// momv2$[] WireSyncEngine Obfuscated Model (.omo).
pub(crate) fn verify_wiresync_momv2_omo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WIRESYNC_MOMV2_OMO_SIZE,
        prefix.len() >= 68,
        get_slice(prefix, 0, 24)?
            == hex2bin!("6D 6F 6D 76 32 24 5B 5D 71 72 65 39 37 34 33 6A 70 6F 39 61 64 33 71 3B")
    );
    require!(has_non_ascii_text_byte(prefix.get(..68)?));
    Some(Label::WiresyncMomv2Omo)
}

/// CD Projekt RED The Witcher 3 Saved Game (`.sav`).
///
/// Header layout:
/// - `@0..4`: `"SNFH"` magic (Save Header)
/// - `@4..8`: `"FZLC"` (LZ4 compressed chunks) or `"FZLB"` (legacy block stream)
/// - `@8..12`: `u32 LE` `chunk_count` (`1..=100_000`)
/// - `@12..16`: `u32 LE` `header_size` (`16..=0x00FF_FFFF`)
pub(crate) fn verify_witcher3_sav(prefix: &[u8], size: u64) -> Option<Label> {
    const STREAM_OFF: usize = 4; // @4..8: compression container tag ("FZLC" / "FZLB")
    const FZLC_TAG: &[u8; 4] = b"FZLC";
    const FZLB_TAG: &[u8; 4] = b"FZLB";
    const CHUNK_CNT_OFF: usize = 8; // @8..12: u32 LE compressed chunk count
    const HDR_SIZE_OFF: usize = 12; // @12..16: u32 LE chunk table size

    require!(
        size >= MIN_WITCHER3_SAV_SIZE,
        prefix.len() >= 16,
        starts(prefix, WITCHER3_SAV_MAGIC),
        eq_at(prefix, STREAM_OFF, FZLC_TAG) || eq_at(prefix, STREAM_OFF, FZLB_TAG),
        (1..=100_000).contains(&le32(prefix, CHUNK_CNT_OFF)?),
        (16..=0x00FF_FFFF).contains(&le32(prefix, HDR_SIZE_OFF)?)
    );
    Some(Label::Witcher3Sav)
}

/// Valve Hammer / Worldcraft Command Sequences (`CmdSeq.wc` / `.wc`, 1997 Valve): binary "Run Map"
/// compile-tool sequence store.
///
/// Layout: `@0..31` signature, `@31 f32 LE version` (0.1 / 0.2), `@35 u32 LE seq_count`, then
/// per sequence a 128-byte NUL-terminated name (`@39..167` for the first; bytes after the NUL may be
/// uninitialized editor memory) and `@167 u32 LE cmd_count`.
pub(crate) fn verify_worldcraft_cmdseq(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION: usize = 31; // @31: f32 LE format version
    const SEQ_COUNT: usize = 35; // @35: u32 LE number of sequences
    const SEQ_NAME: usize = 39; // @39..167: first sequence name (128B, NUL-terminated)
    const SEQ_NAME_LEN: usize = 128;
    const CMD_COUNT: usize = SEQ_NAME + SEQ_NAME_LEN; // @167: u32 LE commands in first sequence
    const MAX_SEQUENCES: u32 = 1024;
    const MAX_COMMANDS: u32 = 256;

    require!(
        size >= MIN_WORLDCRAFT_CMDSEQ_SIZE,
        prefix.len() >= MIN_WORLDCRAFT_CMDSEQ_SIZE as usize,
        starts(prefix, WORLDCRAFT_CMDSEQ_MAGIC)
    );
    let version = f32::from_bits(le32(prefix, VERSION)?);
    let seq_count = le32(prefix, SEQ_COUNT)?;
    require!(
        version.is_finite(),
        (0.05..=10.0).contains(&version),
        (1..=MAX_SEQUENCES).contains(&seq_count)
    );
    let name = get_slice(prefix, SEQ_NAME, SEQ_NAME + SEQ_NAME_LEN)?;
    let nul = name.iter().position(|&b| b == 0)?;
    require!(nul >= 1, name[..nul].iter().all(|&b| (0x20..=0x7E).contains(&b)));
    require!(le32(prefix, CMD_COUNT)? <= MAX_COMMANDS);
    Some(Label::WorldcraftCmdseq)
}

/// x64dbg Binary Execution Trace (`.trace32` / `.trace64`):
/// - `@0..4`: `"TRAC"`,
/// - `@4..8`: `u32 LE json_len` (`16..=102_400`, `8 + json_len <= size`, non-ASCII byte in `@4..8`),
/// - `@8 .. 8 + json_len`: UTF-8 JSON header object containing `"ver"` (`1`) and `"arch"` (`"x86"` or `"x64"`).
pub(crate) fn verify_x64dbg_trace(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_X64DBG_TRACE_SIZE,
        prefix.len() >= 28,
        starts(prefix, X64DBG_TRACE_MAGIC),
        has_non_ascii_text_byte(prefix.get(4..8)?),
    );
    let json_len = le32(prefix, 4)? as usize;
    require!((16..=102_400).contains(&json_len), 8u64.checked_add(json_len as u64)? <= size,);
    let j_end = (8 + json_len).min(prefix.len());
    let json_slice = get_slice(prefix, 8, j_end)?;
    let trimmed_start = json_slice.iter().position(|&b| !b.is_ascii_whitespace() && b != 0)?;
    memchr::memmem::find(json_slice, b"\"ver\"")?;
    memchr::memmem::find(json_slice, b"\"arch\"")?;
    memchr::memmem::find(json_slice, b"\"x64\"")
        .or_else(|| memchr::memmem::find(json_slice, b"\"x86\""))?;
    require!(json_slice[trimmed_start] == b'{');
    if 8 + json_len <= prefix.len() {
        let trimmed_end = json_slice.iter().rposition(|&b| !b.is_ascii_whitespace() && b != 0)?;
        require!(json_slice[trimmed_end] == b'}');
        if 8 + json_len < prefix.len() && (8 + json_len as u64) < size {
            let blk_type = u8_at(prefix, 8 + json_len)?;
            require!(blk_type == 0 || blk_type >= 0x80);
        }
    }
    Some(Label::X64dbgTrace)
}

/// Xiaomi MiWiFi HDR1 / HDR2 Router Firmware Image (`.bin`).
pub(crate) fn verify_xiaomi_hdr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_XIAOMI_HDR_SIZE, prefix.len() >= 32);
    if starts(prefix, b"HDR1") {
        let first_off = le32(prefix, 16)?;
        require!((0x30..=0x1000).contains(&first_off));
    } else if starts(prefix, b"HDR2") {
        require!(
            be32(prefix, 4)? != 0,
            be32(prefix, 8)? != 0,
            has_non_ascii_text_byte(prefix.get(..32)?)
        );
    } else {
        return None;
    }
    Some(Label::XiaomiHdr)
}

/// Xilinx FPGA Configuration Bitstream (`.bit`, Xilinx).
pub(crate) fn verify_xilinx_bitstream(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_XILINX_BITSTREAM_SIZE
            && prefix.len() >= 20
            && starts(prefix, XILINX_BIT_PREAMBLE)
            && u8_at(prefix, 13)? == b'a'
    );
    let design_len = be16(prefix, 14)? as usize;
    require!((2..=512).contains(&design_len));
    if 16 + design_len < prefix.len() {
        require!(u8_at(prefix, 16 + design_len)? == b'b');
    }
    Some(Label::XilinxBit)
}

/// AMD / Xilinx FPGA Accelerator Binary (`.xclbin`, `"xclbin2\0"`).
pub(crate) fn verify_xilinx_xclbin(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_XILINX_XCLBIN_SIZE, starts(prefix, b"xclbin2\0"));
    Some(Label::XilinxXclbin)
}

/// Zebra Technologies Printer Driver Profile (.tpd) Zebra Technologies.
pub(crate) fn verify_zebra_tpd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ZEBRA_TPD_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"TPD\x03\0\0\0"),
        get_slice(prefix, 9, 15)? == b"Zebra "
    );
    Some(Label::ZebraTpd)
}

/// LaProject 0x765D Project (.als).
pub(crate) fn verify_zeroplus_als(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ZEROPLUS_ALS_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"\x5d\x76\0\0\x01"),
        matches!(u8_at(prefix, 5)?, 1 | 2),
        get_slice(prefix, 6, 8)? == b"\0\0",
        get_slice(prefix, 28, 38)? == b"LaProject\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..40)?));
    Some(Label::ZeroplusAls)
}

/// Zigbee Alliance Cluster Library Over-the-Air Upgrade Image (`.ota` / `.zigbee`) Connectivity Standards Alliance.
pub(crate) fn verify_zigbee_ota(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ZIGBEE_OTA_SIZE,
        prefix.len() >= 62,
        starts(prefix, hex2bin!("1E F1 EE 0B 00 01")),
    );
    let hdr_len = le16(prefix, 6)?;
    let fc = le16(prefix, 8)?;
    let zs_ver = le16(prefix, 18)?;
    let total_size = le32(prefix, 52)?;
    require!(
        (56..=92).contains(&hdr_len)
            && (fc & !0x0007) == 0
            && (1..=16).contains(&zs_ver)
            && total_size >= (hdr_len as u32) + 6
    );
    Some(Label::ZigbeeOta)
}

/// ZyXEL BIX Router Firmware Image (`.bix`).
pub(crate) fn verify_zyxel_bix(prefix: &[u8], size: u64) -> Option<Label> {
    const BIX_MAGIC: &[u8; 4] = hex2bin!("83 80 00 00"); // @0: ZyXEL BIX magic
    const IMG_SIZE_OFF: usize = 12; // @12: u32 BE payload length (> 0)
    const NAME_OFF: usize = 32; // @32..40: printable ASCII firmware version (e.g. "ESX.1.2.0")
    require!(
        size >= MIN_ZYXEL_BIX_SIZE,
        prefix.len() >= 64,
        starts(prefix, BIX_MAGIC),
        be32(prefix, IMG_SIZE_OFF)? > 0,
        u8_at(prefix, NAME_OFF)?.is_ascii_graphic(),
        get_slice(prefix, NAME_OFF, NAME_OFF + 8)?.iter().all(|&b| b == 0 || b.is_ascii_graphic())
    );
    Some(Label::ZyxelBix)
}
