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

//! Disk, optical media, and filesystem image format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// Microsoft Virtual Hard Disk (VHD): 512-byte dynamic/differencing header copy of footer opening
/// with `"conectix"`, features `2 | 3` at `@8`, version `0x00010000` at `@12`, disk type `2..=4`
/// at `@60`, and saved state `0..=1` at `@84`.
pub(crate) fn vhd(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"conectix"; // @0..8: VHD cookie
    const FEATURES: usize = 8; // @8: u32 BE — features bitmask (2 or 3)
    const FORMAT_VERSION: usize = 12; // @12: u32 BE — file format version (0x00010000)
    const DISK_TYPE: usize = 60; // @60: u32 BE — disk type (2..=4)
    const SAVED_STATE: usize = 84; // @84: u8 — saved state flag (0 or 1)
    const FOOTER_LEN: usize = 512;

    require!(
        prefix.len() >= FOOTER_LEN,
        starts(prefix, MAGIC),
        matches!(be32(prefix, FEATURES)?, 2 | 3),
        be32(prefix, FORMAT_VERSION)? == 0x0001_0000,
        (2..=4).contains(&be32(prefix, DISK_TYPE)?),
        u8_at(prefix, SAVED_STATE)? <= 1,
    );
    Some(Label::Vhd)
}

/// SquashFS read-only compressed filesystem (`squashfs`, 2002): `"hsqs"` (LE v4),
/// `"sqsh"` (BE), `"shsq"`, or `"qshs"` at `@0`, nonzero `inode_count` at `@4`,
/// power-of-two `block_size` (`4096..=1048576`) at `@12`, and `s_major` `1..=4` at `@28`.
pub(crate) fn squashfs(prefix: &[u8], size: u64) -> Option<Label> {
    static SQUASHFS_LE: Subformat = Subformat::new("SquashFS (little-endian)");
    static SQUASHFS_BE: Subformat = Subformat::new("SquashFS (big-endian)");
    static SQUASHFS_LZMA_LE: Subformat = Subformat::new("SquashFS LZMA (little-endian)");
    static SQUASHFS_LZMA_BE: Subformat = Subformat::new("SquashFS LZMA (big-endian)");
    const MIN_SQUASHFS_SIZE: u64 = 96; // 96B squashfs_super_block
    const SQUASHFS_LE_MAGIC: &[u8; 4] = b"hsqs";
    const SQUASHFS_LZMA_LE_MAGIC: &[u8; 4] = b"shsq";
    const SQUASHFS_BE_MAGIC: &[u8; 4] = b"sqsh";
    const SQUASHFS_LZMA_BE_MAGIC: &[u8; 4] = b"qshs";
    const SNAP_YAML_PATH_1: &[u8] = b"snap/snap.yaml";
    const SNAP_YAML_PATH_2: &[u8] = b"meta/snap.yaml";
    const SNAP_YAML_NAME: &[u8] = b"snap.yaml";
    const SNAP_META_DIR: &[u8] = b"meta";

    require!(size >= MIN_SQUASHFS_SIZE, prefix.len() >= 32);
    let is_le = starts(prefix, SQUASHFS_LE_MAGIC) || starts(prefix, SQUASHFS_LZMA_LE_MAGIC);
    let is_be = starts(prefix, SQUASHFS_BE_MAGIC) || starts(prefix, SQUASHFS_LZMA_BE_MAGIC);
    require!(is_le || is_be);
    let inodes = if is_le { le32(prefix, 4)? } else { be32(prefix, 4)? };
    let block_size = if is_le { le32(prefix, 12)? } else { be32(prefix, 12)? };
    let major = if is_le { le16(prefix, 28)? } else { be16(prefix, 28)? };
    require!(
        inodes >= 1,
        (1024..=1_048_576).contains(&block_size),
        block_size.is_power_of_two(),
        (1..=4).contains(&major),
    );
    if contains(prefix, SNAP_YAML_PATH_1)
        || contains(prefix, SNAP_YAML_PATH_2)
        || (contains(prefix, SNAP_YAML_NAME) && contains(prefix, SNAP_META_DIR))
    {
        return Some(Label::Snap);
    }
    let sub = if starts(prefix, SQUASHFS_LE_MAGIC) {
        &SQUASHFS_LE
    } else if starts(prefix, SQUASHFS_BE_MAGIC) {
        &SQUASHFS_BE
    } else if starts(prefix, SQUASHFS_LZMA_LE_MAGIC) {
        &SQUASHFS_LZMA_LE
    } else {
        &SQUASHFS_LZMA_BE
    };
    with_subformat(Label::Squashfs, sub)
}

static DMG_2IMG: Subformat = Subformat::new("Apple II 2IMG disk image");
static DMG_DDM: Subformat = Subformat::new("Apple Driver Descriptor Map");
static DMG_UDIF: Subformat = Subformat::with_ids("Apple UDIF disk image", "fmt/1071", None);

/// Apple Disk Image header variants (`dmg`): Apple Driver Descriptor Map (`"ER"` +
/// block size `512` or `2048` BE at `@0..4`) or Apple II `"2IMG"` universal disk image.
pub(crate) fn dmg_header(prefix: &[u8], size: u64) -> Option<Label> {
    const TWO_IMG_MAGIC: &[u8; 4] = b"2IMG";
    const DMG_DDM_2048_MAGIC: &[u8] = bytes!(b"ER", &2048u16.to_be_bytes());
    const DMG_DDM_512_MAGIC: &[u8] = bytes!(b"ER", &512u16.to_be_bytes());

    require!(size >= 64, prefix.len() >= 32);
    if starts(prefix, TWO_IMG_MAGIC) {
        let hdr_len = le16(prefix, 8)?;
        let version = le16(prefix, 10)?;
        require!(matches!(hdr_len, 52 | 64), version <= 1);
        return with_subformat(Label::Dmg, &DMG_2IMG);
    }
    require!(
        size >= 512,
        starts(prefix, DMG_DDM_2048_MAGIC) || starts(prefix, DMG_DDM_512_MAGIC),
        be32(prefix, 4)? >= 1,
    );
    with_subformat(Label::Dmg, &DMG_DDM)
}

/// Apple UDIF Disk Image `koly` trailer (`dmg`, 2001): 512-byte trailer at `size - 512`
/// opening with `"koly"`, version `4` BE at `+4`, and header size `512` BE at `+8`.
pub(crate) fn dmg_koly(size: u64, tail: Option<&[u8]>) -> Option<Label> {
    const DMG_KOLY_MAGIC: &[u8] = bytes!(b"koly", &4u32.to_be_bytes(), &512u32.to_be_bytes());
    let t = tail?;
    require!(size >= 512, t.len() >= 512, eq_at(t, t.len() - 512, DMG_KOLY_MAGIC),);
    with_subformat(Label::Dmg, &DMG_UDIF)
}

/// Apple Hierarchical File System (`"BD"` classic HFS, `"H+"` HFS Plus, or `"HX"` HFSX) volume
/// image (`hfs`, 1985): volume header signature at `@1024..1026` with valid version or allocation block size.
pub(crate) fn hfs(prefix: &[u8], size: u64) -> Option<Label> {
    static HFS_PLUS: Subformat = Subformat::with_ids("HFS Plus", "fmt/1742", None);
    static HFSX: Subformat = Subformat::new("HFSX");
    static HFS_CLASSIC: Subformat = Subformat::with_ids("HFS", "fmt/1105", None);
    const HFS_PLUS_SIG: &[u8; 2] = b"H+";
    const HFSX_SIG: &[u8; 2] = b"HX";
    const HFS_CLASSIC_SIG: &[u8; 2] = b"BD";

    require!(
        size >= 1026,
        prefix.len() >= 1026,
        ((eq_at(prefix, 1024, HFS_PLUS_SIG) || eq_at(prefix, 1024, HFSX_SIG))
            && (u8_at(prefix, 0)? == 0
                || (prefix.len() >= 1028 && matches!(be16(prefix, 1026)?, 0 | 4 | 5))))
            || (size >= 2048
                && prefix.len() >= 1060
                && eq_at(prefix, 1024, HFS_CLASSIC_SIG)
                && (512..0x0800_0000).contains(&be32(prefix, 1044)?)
                && be32(prefix, 1044)? % 512 == 0),
    );
    let sub = if eq_at(prefix, 1024, HFS_PLUS_SIG) {
        &HFS_PLUS
    } else if eq_at(prefix, 1024, HFSX_SIG) {
        &HFSX
    } else {
        &HFS_CLASSIC
    };
    with_subformat(Label::Hfs, sub)
}

/// IBM PC / MS-DOS Master Boot Record sector (`dosmbr`, 1983): exact 512-byte sector starting with
/// an x86 bootstrap opcode at `@0`, ending with `55 AA` at `@510..512`, and not a FAT12 BPB boot sector.
pub(crate) fn dosmbr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size == 512,
        prefix.len() >= 512,
        matches!(prefix[0], 0xEB | 0xE9 | 0xEA | 0xE8 | 0xFA | 0xFC | 0x31 | 0x33),
        prefix[510] == 0x55 && prefix[511] == 0xAA,
    );
    // Reject if valid FAT12 BPB with matching total_sectors
    let bps = u16::from_le_bytes([prefix[11], prefix[12]]);
    let spc = prefix[13];
    require!(!(bps == 512 && spc.is_power_of_two() && matches!(prefix[21], 0xF0 | 0xF8..=0xFF)));
    let _ = get_slice(prefix, 0, 1);
    let _ = get_slice(prefix, 510, 512);
    Some(Label::Dosmbr)
}

/// FAT12 / FAT16 floppy or disk image (`img`, 1981): `EB`/`E9` jump at `@0`, 512-byte BIOS Parameter
/// Block (`@11..22`) with power-of-two `spc`, valid media descriptor, and `tot16 * 512 == size`.
pub(crate) fn img_fat12(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 2048 && size.is_multiple_of(512),
        prefix.len() >= 32,
        matches!(prefix[0], 0xEB | 0xE9),
    );
    let bps = u16::from_le_bytes([prefix[11], prefix[12]]);
    let spc = prefix[13];
    let rsvd = u16::from_le_bytes([prefix[14], prefix[15]]);
    let fats = prefix[16];
    let tot16 = u16::from_le_bytes([prefix[19], prefix[20]]) as u64;
    let media = prefix[21];
    require!(
        bps == 512,
        spc != 0 && spc.is_power_of_two(),
        rsvd >= 1,
        (1..=2).contains(&fats),
        matches!(media, 0xF0 | 0xF8..=0xFF),
        tot16 > 0 && tot16 * 512 == size,
    );
    let _ = get_slice(prefix, 0, 1);
    let _ = get_slice(prefix, 11, 22);
    Some(Label::Img)
}

/// Microsoft Windows Imaging Format (`wim`, 2006): `"MSWIM\0\0\0"` or `"WLPWM\0\0\0"` signature at
/// `@0..8`, header size `0xD0` (`208`) LE at `@8..12`, and non-zero version at `@12..16`.
pub(crate) fn wim(prefix: &[u8], size: u64) -> Option<Label> {
    static WIM: Subformat = Subformat::with_ids("WIM", "fmt/614", None);
    static WIM_PIPABLE: Subformat = Subformat::new("Pipable WIM");
    const MSWIM_MAGIC: &[u8; 8] = b"MSWIM\0\0\0";
    const WLPWM_MAGIC: &[u8; 8] = b"WLPWM\0\0\0";

    require!(
        size >= 72,
        prefix.len() >= 20,
        starts(prefix, MSWIM_MAGIC) || starts(prefix, WLPWM_MAGIC),
        le32(prefix, 8)? == 0xD0,
        le32(prefix, 12)? >= 1,
    );
    with_subformat(Label::Wim, if starts(prefix, MSWIM_MAGIC) { &WIM } else { &WIM_PIPABLE })
}

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const ALTO_DIABLO31_IMAGE_SIZE: u64 = 4872 * ALTO_SECTOR_SIZE as u64;
const ALTO_DIABLO44_IMAGE_SIZE: u64 = 2 * ALTO_DIABLO31_IMAGE_SIZE;
const ALTO_FID_DIRECTORY: u16 = 0x8000;
const ALTO_FID_IN_USE: u16 = 0x0001;
const ALTO_HEADER_RDA_WORD: usize = 2;
const ALTO_LABEL_FID0_WORD: usize = 8;
const ALTO_LABEL_FID1_WORD: usize = 9;
const ALTO_LABEL_FILEPAGE_WORD: usize = 7;
const ALTO_LEADER_NAME_OFFSET: usize = 22 + 12;
const ALTO_SECTOR_SIZE: usize = 534;
const ALTO_SYSDIR_NAME: &[u8; 8] = b"\x07SysDir.";
const ALTO_SYSDIR_NAME_SWAPPED: &[u8; 8] = b"S\x07syiD.r";
const ALTO_SYSDIR_RDA: u16 = 0x1000;
const ALTO_SYSDIR_VDA: usize = 1;
const AMIGA_PACKDEV_PREFIX: &[u8; 3] = b"PKD";
const AMIGA_ZOOM5_MAGIC: &[u8; 4] = b"ZOM5";
const AMIGA_ZOOM_HEADER_LEN: usize = 72;
const AMIGA_ZOOM_MAGIC: &[u8; 4] = b"ZOOM";
const ANSI_BINARY_GATE_WINDOW: usize = 4096;
const ANSI_HDR1_MAGIC: &[u8; 4] = b"HDR1";
const ANSI_LABEL_LEN: usize = 80;
const ANSI_VOL1_LABEL_STANDARD_OFFSET: usize = 79;
const ANSI_VOL1_MAGIC: &[u8; 4] = b"VOL1";
const ANSI_VOL1_VOLID_RANGE: core::ops::Range<usize> = 4..10;
const CTOS_VHB_CHECKSUM_TARGET: u16 = 0xDB5D;
const CTOS_VHB_WORDS: usize = 58;
const DEC_RT11_HDR_LEN: usize = 0x30;
const DEC_RT11_HDR_OFFSET: usize = 0x3D0;
const DEC_RT11_SYSID_ALT_OFFSET: usize = 0x3EC;
const DEC_RT11_SYSID_OFFSET: usize = 0x3F0;
const DEC_RT11_SYSTEM_ID: &[u8; 12] = b"DECRT11A    ";
const DEC_XXDP_BOOT_SIG: u16 = 0x00A0;
const EXT234_MAGIC_OFFSET: usize = EXT234_SB_OFFSET + 0x38;
const EXT234_SB_OFFSET: usize = 0x400;
const EXT234_SUPER_MAGIC: u16 = 0xEF53;
const IBM_3740_DDR1_EBCDIC: &[u8; 4] = ebcdic_bytes!("DDR1");
const IBM_3740_DISK_SIZE: u64 = 256_256;
const IBM_3740_ERMAP_EBCDIC: &[u8; 8] = ebcdic_bytes!("ERMAP   ");
const IBM_3740_HDR1_EBCDIC: &[u8; 4] = ebcdic_bytes!("HDR1");
const IBM_3740_VOL1_EBCDIC: &[u8; 4] = ebcdic_bytes!("VOL1");
const MAX_SIMH_LEADING_TAPE_MARKS: usize = 2;
const MAX_SIMH_RECORD_LEN: u32 = 0x1_0000;
const MIN_AARU_IMAGE_SIZE: u64 = 64;
const MIN_ALCOHOL_MDS_SIZE: u64 = 88;
const MIN_AMIGA_PACKDEV_SIZE: u64 = 40;
const MIN_AMIGA_ZOOM_SIZE: u64 = 80;
const MIN_ANDROID_OTA_SIZE: u64 = 32;
const MIN_ANDROID_SPARSE_SIZE: u64 = 28;
const MIN_ANSI_TAPE_SIZE: u64 = 160;
const MIN_APPLE_A2R_SIZE: u64 = 48;
const MIN_APPLE_NDIF_SIZE: u64 = 64;
const MIN_APPLE_SPARSEIMAGE_SIZE: u64 = 64;
const MIN_ATARI_7800_SIZE: u64 = 128;
const MIN_ATARI_MSA_SIZE: u64 = 522;
const MIN_ATARI_STX_SIZE: u64 = 32;
const MIN_BOCHS_DISK_SIZE: u64 = 512;
const MIN_BOX86_86F_SIZE: u64 = 512;
const MIN_BTRFS_SEND_SIZE: u64 = 23;
const MIN_C64_TAP_SIZE: u64 = 24;
const MIN_CAPS_IPF_SIZE: u64 = 108;
const MIN_CHD_SIZE: u64 = 16;
const MIN_COMMODORE_NIBTOOLS_SIZE: u64 = 256;
const MIN_CRAMFS_SIZE: u64 = 64;
const MIN_CSO_CONTAINER_SIZE: u64 = 24;
const MIN_CTOS_VOLUME_SIZE: u64 = 8192;
const MIN_DEC_RT11_SIZE: u64 = 1024;
const MIN_DEC_XXDP_SIZE: u64 = 8192;
const MIN_DISKDUPE_DDI_SIZE: u64 = 9728;
const MIN_DOS_DIM_ARCHIVE_SIZE: u64 = 596;
const MIN_ENCASE_E01_SIZE: u64 = 64;
const MIN_EXT234_VOLUME_SIZE: u64 = 2048;
const MIN_FDS_DISK_SIZE: u64 = 65_500;
const MIN_FTK_AD1_SIZE: u64 = 512;
const MIN_HERCULES_DASD_SIZE: u64 = 512;
const MIN_IBM_SAVEDSKF_SIZE: u64 = 64;
const MIN_KRYOFLUX_STREAM_SIZE: u64 = 48;
const MIN_LITTLEFS_SIZE: u64 = 256;
const MIN_MICROCHIP_MPFS_SIZE: u64 = 24;
const MIN_MICROSOFT_VHDX_SIZE: u64 = 512;
const MIN_MOOF_SIZE: u64 = 80;
const MIN_MSX_CAS_SIZE: u64 = 26;
const MIN_N64_ROM_SIZE: u64 = 64;
const MIN_NEO_GEO_POCKET_SIZE: u64 = 64;
const MIN_NES_ROM_SIZE: u64 = 16400;
const MIN_NTFSCLONE_SIZE: u64 = 64;
const MIN_ORIC_MFM_DISK_SIZE: u64 = ORIC_MFM_HEADER_SIZE + ORIC_MFM_TRACK_SIZE;
const MIN_PARALLELS_HDD_SIZE: u64 = 64;
const MIN_PC98_VFD_SIZE: u64 = 256;
const MIN_PISMO_CISO_SIZE: u64 = 64;
const MIN_POWERISO_DAA_SIZE: u64 = 72;
const MIN_PSP_DAX_SIZE: u64 = 36;
const MIN_QCOW2_SIZE: u64 = 72;
const MIN_QEMU_QED_SIZE: u64 = 64;
const MIN_QNX_DEFLATE_SIZE: u64 = 20;
const MIN_ROMFS_SIZE: u64 = 32;
const MIN_SABDU_SIZE: u64 = 32;
const MIN_SEGA_SMS_ROM_SIZE: u64 = 8_192;
const MIN_SIMH_RECORD_LEN: u32 = 14;
const MIN_SIMH_TAPE_SIZE: u64 = 26;
const MIN_SINCLAIR_QDOS_SIZE: u64 = 512;
const MIN_STACKER_SIZE: u64 = 512;
const MIN_T98NEXT_NFD_SIZE: u64 = 512;
const MIN_TELEDISK_SIZE: u64 = 16;
const MIN_TPLINK_MINIFS_SIZE: u64 = 64;
const MIN_TROC_FS_SIZE: u64 = 32;
const MIN_UBIFS_SIZE: u64 = 64;
const MIN_UCSD_VOL_SIZE: u64 = 2048;
const MIN_UNIFLEX_IMG_SIZE: u64 = 1024;
const MIN_UNIXWARE_BFS_SIZE: u64 = 512;
const MIN_VMDK_SIZE: u64 = 512;
const MIN_XFS_SIZE: u64 = 512;
const MIN_ZX_TZX_SIZE: u64 = 16;
const ORIC_MFM_HEADER_SIZE: u64 = 256;
const ORIC_MFM_TRACK_SIZE: u64 = 6400;
const SIMH_END_OF_MEDIUM: u32 = 0xFFFF_FFFF;
const SIMH_META_LEN: usize = 4;
const SIMH_RECORD_LEN_MASK: u32 = 0x00FF_FFFF;
const SIMH_TAPE_MARK: u32 = 0x0000_0000;
const TEKTRONIX_4404_BOOT_PREFIX: &[u8; 5] = b"4400P";

/// Augmented (non-reflected, init 0) CRC-16 poly `0x1021` (Zoom 1.x–4.x).
const fn amiga_crc_1021_table() -> [u16; 256] {
    let mut t = [0u16; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = (i as u16) << 8;
        let mut b = 0;
        while b < 8 {
            c = if c & 0x8000 != 0 { (c << 1) ^ 0x1021 } else { c << 1 };
            b += 1;
        }
        t[i] = c;
        i += 1;
    }
    t
}

fn amiga_xad_crc_1021_2(data: &[u8]) -> u16 {
    data.iter()
        .fold(0u16, |crc, &b| AMIGA_CRC_1021_TABLE[(crc >> 8) as usize] ^ ((crc << 8) ^ b as u16))
}

/// Reflected CRC-32 poly `0xEDB88320`, init 0, no final XOR (Zoom 5).
fn amiga_xad_crc32_id1(data: &[u8]) -> u32 {
    data.iter().fold(0u32, |mut crc, &b| {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
        crc
    })
}

fn crc16_teledisk(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0xA097;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Returns the offset just past the trailing length word of the SIMH record at `pos`,
/// requiring the trailing `u32 LE` length to repeat the leading one. SIMH pads odd-length
/// records to an even length; Ersatz-11 (E11) writes them unpadded — both are accepted.
fn simh_record_end(prefix: &[u8], pos: usize) -> Option<usize> {
    let meta = le32(prefix, pos)?;
    require!(meta & !SIMH_RECORD_LEN_MASK == 0);
    let len = meta & SIMH_RECORD_LEN_MASK;
    require!((MIN_SIMH_RECORD_LEN..=MAX_SIMH_RECORD_LEN).contains(&len));
    let data_end = pos + SIMH_META_LEN + len as usize;
    if len & 1 == 1 && le32(prefix, data_end + 1) == Some(meta) {
        return Some(data_end + 1 + SIMH_META_LEN); // SIMH even-padded odd record
    }
    (le32(prefix, data_end) == Some(meta)).then_some(data_end + SIMH_META_LEN)
}

static AMIGA_CRC_1021_TABLE: [u16; 256] = amiga_crc_1021_table();

/// Aaru / DiscImageChef Disk Image (`"DICF"` / `"AARUFRMT"` / `"DICMFRMT"`).
pub(crate) fn verify_aaru_disk_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AARU_IMAGE_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"DICF") || starts(prefix, b"AARUFRMT") || starts(prefix, b"DICMFRMT"),
        prefix.get(8..16)?.contains(&0)
    );
    Some(Label::AaruDicf)
}

/// AccessData FTK Logical Image (`.ad1`, `"ADSEGMENTEDFILE\0"`).
pub(crate) fn verify_accessdata_ftk_ad1(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_FTK_AD1_SIZE, starts(prefix, b"ADSEGMENTEDFILE\0"));
    Some(Label::FtkAd1)
}

/// ACT Apricot floppy disk image.
pub(crate) fn verify_act_apricot(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 32, prefix.len() >= 32, starts(prefix, b"ACT Apricot disk image\x1A\x04"));
    Some(Label::DskAct)
}

/// Alcohol / DAEMON Tools media descriptor: `"MEDIA DESCRIPTOR"`, version `u8 major` (1 = MDS, 2 = MDX/MDS v2) + `u8 minor`.
pub(crate) fn verify_alcohol_mds(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_MAJOR: usize = 16; // @16: u8
    require!(size >= MIN_ALCOHOL_MDS_SIZE, starts(prefix, b"MEDIA DESCRIPTOR"));
    require!(matches!(u8_at(prefix, VERSION_MAJOR)?, 1 | 2));
    Some(Label::Mds)
}

/// PackDev device archive (`.pkd` / `.dsq`, 1994–1997, Christian Wasner):
/// - `@0..3`: `"PKD"`, `@3`: `u8` BCD version/revision (`0x10..=0x19`)
/// - `@4`: `u32 BE` block count, `@8`: `u32 BE` block size (`256..=4096`, power of two)
/// - `@12`: `u32 BE` reserved blocks, `@16`: `u32 BE` track length (multiple of block size)
/// - `@20`: `u32 BE` XPK buffer size, `@24`: XPK packer name (`4` B for `v < 1.2`, `24` B for `v >= 1.2`)
/// - `@28`/`@48`: `u16 BE` XPK mode (`0..=100`), `@30`/`@50`: `u16 BE` `BAMPresent` (`0` stored, `1` BAM follows).
pub(crate) fn verify_amiga_packdev(prefix: &[u8], size: u64) -> Option<Label> {
    const VERS_REV: usize = 3; // @3: u8 BCD version (high nibble) / revision (low nibble)
    const BLOCK_COUNT: usize = 4; // @4: u32 BE number of blocks
    const BLOCK_SIZE: usize = 8; // @8: u32 BE bytes per block
    const RESERVED: usize = 12; // @12: u32 BE reserved (boot) blocks
    const TRACK_LEN: usize = 16; // @16: u32 BE bytes per track
    const XPK_PACKER: usize = 24; // @24: 4-byte XPK sub-packer FourCC (or 0 if uncompressed)
    require!(
        size >= MIN_AMIGA_PACKDEV_SIZE,
        prefix.len() >= 32,
        starts(prefix, AMIGA_PACKDEV_PREFIX)
    );
    let vers_rev = u8_at(prefix, VERS_REV)?;
    require!((0x10..=0x19).contains(&vers_rev));
    let (hdr_len, xpk_mode_off, known_fs_off) =
        if vers_rev < 0x12 { (32usize, 28usize, 30usize) } else { (52usize, 48usize, 50usize) };
    require!(size >= (hdr_len as u64) + 8, prefix.len() >= hdr_len);
    let blocks = be32(prefix, BLOCK_COUNT)?;
    let block_size = be32(prefix, BLOCK_SIZE)?;
    let reserved = be32(prefix, RESERVED)?;
    let track_len = be32(prefix, TRACK_LEN)?;
    let packer = get_slice(prefix, XPK_PACKER, XPK_PACKER + 4)?;
    let packer_valid = packer == [0, 0, 0, 0] || packer.iter().all(|b| b.is_ascii_graphic());
    require!(
        (256..=4096).contains(&block_size),
        block_size.is_power_of_two(),
        blocks > 0 && blocks < 0x0100_0000,
        reserved <= blocks,
        track_len >= block_size && track_len % block_size == 0 && track_len <= 0x0010_0000,
        packer_valid,
        be16(prefix, xpk_mode_off)? <= 100,
        be16(prefix, known_fs_off)? <= 1
    );
    Some(Label::AmigaPackdev)
}

/// Amiga Zoom disk archive (`.zom`, 1990–1995, Olaf Barthel):
/// - `@0`: `"ZOOM"` (v1–4) or `"ZOM5"` (v5)
/// - `@4`: `u8 From` / `@5`: `u8 To` — first/last cylinder (`From <= To <= 83`)
/// - `@6`: `u8 Version` (`>= 3` for `"ZOOM"`)
/// - `@72`: header checksum over `@0..72` — `u16 BE` CRC-16 (`"ZOOM"`) or `u32 BE` raw CRC-32 (`"ZOM5"`).
pub(crate) fn verify_amiga_zoom(prefix: &[u8], size: u64) -> Option<Label> {
    const FROM_CYL: usize = 4; // @4: u8 first cylinder
    const TO_CYL: usize = 5; // @5: u8 last cylinder
    const VERSION: usize = 6; // @6: u8 Zoom version that wrote the file
    const MAX_CYLINDER: u8 = 83; // 80 standard + 3 overscan cylinders
    const MIN_ZOOM_VERSION: u8 = 3; // XAD: data[6] >= 3
    require!(size >= MIN_AMIGA_ZOOM_SIZE, prefix.len() >= AMIGA_ZOOM_HEADER_LEN + 4);
    let from = u8_at(prefix, FROM_CYL)?;
    let to = u8_at(prefix, TO_CYL)?;
    require!(from <= to, to <= MAX_CYLINDER);
    let header = get_slice(prefix, 0, AMIGA_ZOOM_HEADER_LEN)?;
    if starts(prefix, AMIGA_ZOOM5_MAGIC) {
        require!(be32(prefix, AMIGA_ZOOM_HEADER_LEN)? == amiga_xad_crc32_id1(header));
    } else {
        require!(
            starts(prefix, AMIGA_ZOOM_MAGIC),
            u8_at(prefix, VERSION)? >= MIN_ZOOM_VERSION,
            be16(prefix, AMIGA_ZOOM_HEADER_LEN)? == amiga_xad_crc_1021_2(header)
        );
    }
    Some(Label::AmigaZoom)
}

/// Google ChromeOS / Android A/B OTA Update Payload (CrAU).
pub(crate) fn verify_android_ota(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ANDROID_OTA_SIZE && prefix.len() >= 24);
    let version = be64(prefix, 4)?;
    let manifest_size = be64(prefix, 12)?;
    require!((1..=2).contains(&version) && manifest_size > 0 && manifest_size + 20 <= size);
    Some(Label::AndroidOta)
}

/// Android Sparse Ext4/EROFS System Partition Image (.simg).
pub(crate) fn verify_android_sparse(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ANDROID_SPARSE_SIZE && prefix.len() >= 28);
    let major = le16(prefix, 4).unwrap_or(0);
    let file_hdr_sz = le16(prefix, 8).unwrap_or(0);
    let chunk_hdr_sz = le16(prefix, 10).unwrap_or(0);
    let blk_sz = le32(prefix, 12).unwrap_or(0);
    require!(
        major == 1
            && file_hdr_sz >= 28
            && chunk_hdr_sz >= 12
            && blk_sz >= 1024
            && blk_sz.is_power_of_two()
    );
    Some(Label::AndroidSparse)
}

/// ANSI X3.27 / ISO 1001 Labeled Magnetic Tape Volume dumped without record framing
/// (raw cartridge/9-track images, e.g. Apollo Domain/OS `.ct` QIC dumps).
///
/// Requires an 80-byte printable `VOL1` label with a non-blank volume identifier and a label
/// standard version `'1'..='4'` at `@79`, followed either by `HDR1` or by NUL block padding,
/// and at least one non-printable byte within the next 4 KiB.
pub(crate) fn verify_ansi_labeled_tape(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ANSI_TAPE_SIZE,
        prefix.len() > ANSI_LABEL_LEN,
        starts(prefix, ANSI_VOL1_MAGIC),
        get_slice(prefix, 0, ANSI_LABEL_LEN)?.iter().all(|&b| (0x20..=0x7E).contains(&b)),
        (b'1'..=b'4').contains(&u8_at(prefix, ANSI_VOL1_LABEL_STANDARD_OFFSET)?),
        get_slice(prefix, ANSI_VOL1_VOLID_RANGE.start, ANSI_VOL1_VOLID_RANGE.end)?
            .iter()
            .any(|&b| b != b' '),
        eq_at(prefix, ANSI_LABEL_LEN, ANSI_HDR1_MAGIC)
            || u8_at(prefix, ANSI_LABEL_LEN).unwrap_or(0) == 0
    );
    let gate_end = prefix.len().min(ANSI_LABEL_LEN + ANSI_BINARY_GATE_WINDOW);
    require!(
        get_slice(prefix, ANSI_LABEL_LEN, gate_end)?
            .iter()
            .any(|&b| !matches!(b, 0x20..=0x7E | b'\t' | b'\n' | b'\r'))
    );
    Some(Label::AnsiTape)
}

/// Applesauce A2R floppy flux capture.
pub(crate) fn verify_apple_a2r(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APPLE_A2R_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"A2R1") || starts(prefix, b"A2R2") || starts(prefix, b"A2R3"),
        eq_at(prefix, 4, b"\xFF\n\r\n"),
        eq_at(prefix, 8, b"INFO") || eq_at(prefix, 12, b"INFO")
    );
    Some(Label::A2r)
}

/// Apple DiskCopy 6.0 New Disk Image Format (.dmg / .img).
pub(crate) fn verify_apple_ndif(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_APPLE_NDIF_SIZE && prefix.len() >= 16);
    require!(starts(prefix, b"dImg"));

    let version = be32(prefix, 4)?;
    // NDIF DiskCopy 6.0 version is standard 1 (0x00000001 BE)
    require!(version == 1);

    Some(Label::Ndif)
}

/// Apple macOS Sparse Disk Image (`.sparseimage`, `"sprs"` + `be32(4) == 3`).
pub(crate) fn verify_apple_sparse_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APPLE_SPARSEIMAGE_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"sprs"),
        be32(prefix, 4) == Some(3),
        be32(prefix, 8)? > 0
    );
    Some(Label::AppleSparseimage)
}

/// Atari 7800 ProSystem Cartridge ROM (.a78).
pub(crate) fn verify_atari_7800_rom(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ATARI_7800_SIZE && prefix.len() >= 128);

    let is_a78 = starts(prefix, b"ATARI7800") || starts(prefix, b"\x01ATARI7800");
    require!(is_a78);

    // Confirmation magic at offset 100
    require!(eq_at(prefix, 100, b"ACTUAL CART DATA STARTS HERE"));

    Some(Label::A78Rom)
}

/// Atari ST Magic Shadow Archiver Compressed Floppy Image (.msa).
pub(crate) fn verify_atari_msa(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ATARI_MSA_SIZE && prefix.len() >= 16);
    let spt = be16(prefix, 2)?;
    let sides = be16(prefix, 4)?;
    let start_track = be16(prefix, 6)?;
    let end_track = be16(prefix, 8)?;
    let first_track_len = be16(prefix, 10)?;
    require!(
        (8..=22).contains(&spt)
            && sides <= 1
            && start_track <= end_track
            && end_track <= 86
            && first_track_len >= 1
            && first_track_len <= spt * 512
            && 12 + (first_track_len as u64) <= size
    );
    Some(Label::AtariMsa)
}

/// Atari ST Pasti Copy-Protected Floppy Disk Image (.stx).
pub(crate) fn verify_atari_stx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_ATARI_STX_SIZE && prefix.len() >= 16 && starts(prefix, b"RSY\0"));
    let version = le16(prefix, 4)?;
    let tracks = u8_at(prefix, 10)?;
    require!((1..=3).contains(&version) && (1..=172).contains(&tracks));
    Some(Label::AtariStx)
}

/// Vortex ATonce Atari ST PC-emulator virtual hard disk container.
pub(crate) fn verify_atonce_data(prefix: &[u8], size: u64) -> Option<Label> {
    const TAG: &[u8; 8] = &[0x04, 0x05, 0x06, 0x01, 0x00, 0x01, 0x02, 0x03];
    const RECORD_SIZE: usize = 32;
    require!(
        size >= 64,
        prefix.len() >= 64,
        eq_at(prefix, 8, TAG),
        u8_at(prefix, 18)? == 0,
        eq_at(prefix, RECORD_SIZE + 8, TAG),
        u8_at(prefix, RECORD_SIZE + 18)? == 1,
    );
    Some(Label::DsgAtonce)
}

/// Bochs Virtual HD Image (`"Bochs Virtual HD Image\0"` + `"Redolog\0"` at `@32`).
pub(crate) fn verify_bochs_disk_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BOCHS_DISK_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"Bochs Virtual HD Image\0"),
        eq_at(prefix, 32, b"Redolog\0")
    );
    Some(Label::BochsDisk)
}

/// 86Box Surface Floppy Image (`.86f`, `"86BF"`).
pub(crate) fn verify_box86_floppy_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BOX86_86F_SIZE,
        prefix.len() >= 8,
        starts(prefix, b"86BF"),
        (0x0100..=0x0300).contains(&le16(prefix, 4)?)
    );
    Some(Label::Box8686f)
}

/// Linux Btrfs Send Stream (`"btrfs-stream\0"` + `u32 LE version in 1..=3`).
pub(crate) fn verify_btrfs_send_stream(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BTRFS_SEND_SIZE,
        prefix.len() >= 17,
        starts(prefix, b"btrfs-stream\0"),
        matches!(le32(prefix, 13), Some(1..=3))
    );
    Some(Label::BtrfsSendStream)
}

/// Commodore 64 / C16 CCS64 Raw Datasette Tape Pulse Image (.tap).
pub(crate) fn verify_c64_tap(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_C64_TAP_SIZE
            && prefix.len() >= 20
            && (starts(prefix, b"C64-TAPE-RAW") || starts(prefix, b"C16-TAPE-RAW"))
            && u8_at(prefix, 12)? <= 2
    );
    let data_len = le32(prefix, 16)? as u64;
    require!(data_len >= 4 && 20u64.saturating_add(data_len) <= size);
    Some(Label::C64Tap)
}

/// Software Preservation Society (SPS / CAPS) Interchangeable Preservation Format Flux Image (.ipf).
pub(crate) fn verify_caps_ipf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CAPS_IPF_SIZE
            && prefix.len() >= 24
            && starts(prefix, b"CAPS")
            && be32(prefix, 4) == Some(12)
            && eq_at(prefix, 12, b"INFO")
            && be32(prefix, 16) == Some(96)
    );
    Some(Label::CapsIpf)
}

/// MAME / MESS Compressed Hunks of Data (CHD).
pub(crate) fn verify_chd(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_CHD_SIZE && prefix.len() >= MIN_CHD_SIZE as usize && starts(prefix, b"MComprHD")
    {
        let ver = be32(prefix, 12).unwrap_or(0);
        if ver <= 5 {
            return Some(Label::Chd);
        }
    }
    None
}

/// Commodore 1581 3.5" Floppy Disk Image (`.d81`).
pub(crate) fn verify_commodore_d81(prefix: &[u8], size: u64) -> Option<Label> {
    require!(matches!(size, 819_200 | 822_400), prefix.len() >= 16, starts(prefix, b"CBM\0"));
    Some(Label::C1581D81)
}

/// Commodore 1541 NIBTools Raw GCR Image (`.nib` / `.nbz`, `"MNIB-1541-RAW"` at `@0` or `@1` after `0x1F` RLE byte).
pub(crate) fn verify_commodore_nibtools(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_COMMODORE_NIBTOOLS_SIZE, prefix.len() >= 20);
    let is_nib = starts(prefix, b"MNIB-1541-RAW");
    let is_nbz = u8_at(prefix, 0)? == 0x1F && eq_at(prefix, 1, b"MNIB-1541-RAW");
    require!(is_nib || is_nbz);
    Some(Label::NibtoolsNib)
}

/// Linux Compressed ROM File System Image (.cramfs).
pub(crate) fn verify_cramfs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CRAMFS_SIZE
            && prefix.len() >= 32
            && get_slice(prefix, 16, 32)? == b"Compressed ROMFS"
    );
    Some(Label::Cramfs)
}

/// Compressed ISO (CSO/CISO) disc image.
pub(crate) fn verify_cso_container(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_CSO_CONTAINER_SIZE
        && prefix.len() >= MIN_CSO_CONTAINER_SIZE as usize
        && starts(prefix, b"CISO")
        && prefix.get(4..24)?.contains(&0)
    {
        let sector_size = le32(prefix, 16).unwrap_or(0);
        if matches!(sector_size, 0 | 512 | 1024 | 2048 | 4096 | 8192 | 16384 | 32768) {
            return Some(Label::Cso);
        }
    }
    None
}

/// Convergent CTOS / Burroughs BTOS Disk Volume (`.img` / `.ima`, 1981 Convergent Technologies).
///
/// Volume Home Block (`VHB`, sector 0 at `@0x00..0x200`):
/// - `@0x08`: `u16 LE sector_size == 512`
/// - `@0x14`: `u8 vol_name_len` (`1..=12`) + printable ASCII volume label at `@0x15..`
/// - `@0x00..0x74` (58 `u16 LE` words): wrapping 16-bit sum strictly equals `0xDB5D`.
pub(crate) fn verify_ctos_volume(prefix: &[u8], size: u64) -> Option<Label> {
    const SECT_SIZE_OFF: usize = 0x08; // @0x08: u16 LE sector size (512)
    const VLEN_OFF: usize = 0x14; // @0x14: u8 Pascal volume name length
    const VNAME_OFF: usize = 0x15; // @0x15: ASCII volume name
    const VHB_LEN: usize = CTOS_VHB_WORDS * 2; // 116 bytes

    require!(
        size >= MIN_CTOS_VOLUME_SIZE,
        size.is_multiple_of(512),
        prefix.len() >= 512,
        le16(prefix, SECT_SIZE_OFF) == Some(512)
    );
    let vlen = u8_at(prefix, VLEN_OFF)? as usize;
    require!(
        (1..=12).contains(&vlen),
        get_slice(prefix, VNAME_OFF, VNAME_OFF + vlen)?.iter().all(|&b| (0x20..=0x7E).contains(&b)),
        has_non_ascii_text_byte(prefix.get(..VHB_LEN)?)
    );

    let mut sum: u16 = 0;
    for idx in 0..CTOS_VHB_WORDS {
        sum = sum.wrapping_add(le16(prefix, idx * 2)?);
    }
    require!(sum == CTOS_VHB_CHECKSUM_TARGET);

    Some(Label::CtosVol)
}

/// DEC PDP-11 RT-11 Disk Filesystem Image (`.dsk`, 1973 DEC).
///
/// Header layout (Block 1 Home Block at `@0x200..0x400`):
/// - `@0x3D2..0x3D4`: `u16 LE` pack cluster size (typically `1`)
/// - `@0x3D4..0x3D6`: `u16 LE` first directory segment block number (typically `6`)
/// - `@0x3D6..0x3D8`: `u16 LE` PDP-11 Radix-50 system version word (e.g. `0x8E53` = `"V05"`)
/// - `@0x3D8..0x3E4`: 12-byte volume identification string
/// - `@0x3E4..0x3F0`: 12-byte owner name string
/// - `@0x3F0..0x3FC` (or `@0x3EC..0x3F8`): 12-byte system identification `"DECRT11A    "`
/// - `@0x3FE..0x400`: `u16 LE` additive 16-bit Home Block checksum (`0x0000` when omitted on some volumes).
pub(crate) fn verify_dec_rt11(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DEC_RT11_SIZE,
        prefix.len() >= DEC_RT11_HDR_OFFSET + DEC_RT11_HDR_LEN,
        eq_at(prefix, DEC_RT11_SYSID_ALT_OFFSET, DEC_RT11_SYSTEM_ID)
            || eq_at(prefix, DEC_RT11_SYSID_OFFSET, DEC_RT11_SYSTEM_ID),
        has_non_ascii_text_byte(
            prefix.get(DEC_RT11_HDR_OFFSET..DEC_RT11_HDR_OFFSET + DEC_RT11_HDR_LEN)?
        )
    );
    Some(Label::DecRt11)
}

/// DEC PDP-11 XXDP Diagnostic Filesystem Image (`.dsk`, 1973 Digital Equipment).
///
/// Header layout (512-byte block aligned):
/// - Block 0 `@0x00..0x0C`: PDP-11 boot words `[0x00A0, 0x0100..=0x01FF, 0x0006, 0x0000, 0x000A, 0x0000]`
/// - UFD directory block (block 3 at `@0x0600` for XXDP v1, or block `blk1[1]` for XXDP v2):
///   `u16 LE next_link < total_blocks` followed by valid 9-word RADIX-50 file entry (`f1`, `f2`, `ext`, `start`, `len`).
pub(crate) fn verify_dec_xxdp(prefix: &[u8], size: u64) -> Option<Label> {
    const HOME_BLK1_UFD_OFF: usize = 514; // @0x202: XXDP v2 UFD start block in block 1

    require!(
        size >= MIN_DEC_XXDP_SIZE,
        size.is_multiple_of(512),
        prefix.len() >= 2048,
        le16(prefix, 0) == Some(DEC_XXDP_BOOT_SIG),
        (0x0100..=0x01FF).contains(&le16(prefix, 2)?),
        le16(prefix, 4) == Some(6),
        le16(prefix, 6) == Some(0),
        le16(prefix, 8) == Some(10),
        le16(prefix, 10) == Some(0),
        has_non_ascii_text_byte(prefix.get(..512)?)
    );

    let total_blks = size / 512;
    let check_ufd_at = |ufd_blk: usize| -> Option<usize> {
        let off = ufd_blk.checked_mul(512)?;
        if off + 20 > prefix.len() {
            return None;
        }
        let link = le16(prefix, off)? as u64;
        let f1 = le16(prefix, off + 2)?;
        let f2 = le16(prefix, off + 4)?;
        let ext = le16(prefix, off + 6)?;
        let start = le16(prefix, off + 12)? as u64;
        let len = le16(prefix, off + 14)? as u64;
        if (link == 0 || link < total_blks)
            && (1..64_000).contains(&f1)
            && f2 < 64_000
            && (1..64_000).contains(&ext)
            && start >= 1
            && start < total_blks
            && len >= 1
            && start + len <= total_blks
        {
            Some(off)
        } else {
            None
        }
    };

    if check_ufd_at(3).is_none() {
        let v2_blk = le16(prefix, HOME_BLK1_UFD_OFF)? as usize;
        require!(v2_blk >= 1);
        check_ufd_at(v2_blk)?;
    }
    Some(Label::DecXxdp)
}

/// Micro System Designs DiskDupe Floppy Disk Image (`.ddi`).
///
/// Header layout:
/// - `@0..10`: `"IM\0\0\0\0\0\0\0\0"` (`49 4D 00 00 00 00 00 00 00 00`)
/// - `@10`: format density code (`2..=4`)
/// - `@11..100`: 89 zero bytes
/// - `@100..108`: track/side table starting with `01 01 00 00 00 01 01 02`.
pub(crate) fn verify_diskdupe_ddi(prefix: &[u8], size: u64) -> Option<Label> {
    const DDI_MAGIC: &[u8; 10] = bytes!(b"IM", be64_bytes!(0)); // @0..10: "IM" + 8 zero bytes
    const DDI_V5_MAGIC: &[u8; 21] = b"MSD Image Version 1 \x1A"; // @0..21: DiskDupe v5 header magic
    const DENSITY_OFFSET: usize = 10; // @10: u8 floppy density code (2..=4)
    const ZERO_PAD_OFFSET: usize = 11; // @11..100: 89 zero padding bytes
    const TRACK_MAP_OFFSET: usize = 100; // @100..108: initial track/head table entries
    const TRACK_MAP_MAGIC: &[u8; 8] = hex2bin!("01 01 00 00 00 01 01 02"); // @100..108

    if size >= 72
        && prefix.len() >= 48
        && starts(prefix, DDI_V5_MAGIC)
        && (1..=4).contains(&u8_at(prefix, 21)?)
        && u8_at(prefix, 22) == Some(0)
    {
        return Some(Label::DiskdupeDdi);
    }

    require!(
        size >= MIN_DISKDUPE_DDI_SIZE,
        prefix.len() >= 128,
        starts(prefix, DDI_MAGIC),
        (2..=4).contains(&u8_at(prefix, DENSITY_OFFSET)?),
        eq_at(prefix, ZERO_PAD_OFFSET, &[0u8; 89]),
        eq_at(prefix, TRACK_MAP_OFFSET, TRACK_MAP_MAGIC)
    );
    Some(Label::DiskdupeDdi)
}

/// Amiga Disk Masher System compressed floppy image (`.dms`).
pub(crate) fn verify_dms(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 56,
        prefix.len() >= 56,
        starts(prefix, b"DMS!"),
        prefix.get(4..16)?.contains(&0),
    );
    let _low_trk = be16(prefix, 10)?;
    let _high_trk = be16(prefix, 12)?;
    Some(Label::Dms)
}

/// Ray Arachelian Disk IMage (`.dim`, `"Disk IMage VER "`).
pub(crate) fn verify_dos_dim_archive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DOS_DIM_ARCHIVE_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"Disk IMage VER ")
    );
    Some(Label::DosDim)
}

/// EnCase / Expert Witness Forensic Disk Image (.E01 / .Ex01).
pub(crate) fn verify_encase_e01(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ENCASE_E01_SIZE
            && prefix.len() >= 16
            && (starts(prefix, b"EVF\x09\x0D\x0A\xFF\x00") || starts(prefix, b"EVF2\r\n\x81\0"))
    );
    Some(Label::EncaseE01)
}

/// Linux Ext2 / Ext3 / Ext4 superblock (`s_magic == 0xEF53` LE at `@0x438`) when the
/// 1024-byte boot-block preamble at `@0..0x400` is non-zero (e.g., GRUB Stage 1 boot code or sector markers).
pub(crate) fn verify_ext234_offset(prefix: &[u8], size: u64) -> Option<Label> {
    const INODES_COUNT_OFF: usize = EXT234_SB_OFFSET; // @0x400: u32 LE s_inodes_count
    const BLOCKS_COUNT_LO_OFF: usize = EXT234_SB_OFFSET + 0x04; // @0x404: u32 LE s_blocks_count_lo
    const FIRST_DATA_BLOCK_OFF: usize = EXT234_SB_OFFSET + 0x14; // @0x414: u32 LE s_first_data_block (0 or 1)
    const LOG_BLOCK_SIZE_OFF: usize = EXT234_SB_OFFSET + 0x18; // @0x418: u32 LE s_log_block_size (0..=6)
    const BLOCKS_PER_GROUP_OFF: usize = EXT234_SB_OFFSET + 0x20; // @0x420: u32 LE s_blocks_per_group
    const INODES_PER_GROUP_OFF: usize = EXT234_SB_OFFSET + 0x28; // @0x428: u32 LE s_inodes_per_group
    require!(
        size >= MIN_EXT234_VOLUME_SIZE,
        prefix.len() >= EXT234_MAGIC_OFFSET + 2,
        le16(prefix, EXT234_MAGIC_OFFSET) == Some(EXT234_SUPER_MAGIC),
        le32(prefix, INODES_COUNT_OFF)? > 0,
        le32(prefix, BLOCKS_COUNT_LO_OFF)? > 0,
        le32(prefix, FIRST_DATA_BLOCK_OFF)? <= 1,
        le32(prefix, LOG_BLOCK_SIZE_OFF)? <= 6,
        le32(prefix, BLOCKS_PER_GROUP_OFF)? > 0,
        le32(prefix, INODES_PER_GROUP_OFF)? > 0
    );
    Some(Label::Ext4)
}

/// Hercules Mainframe CKD / FBA DASD Image (`"CKD_P370"`, `"CKD_C370"`, `"CKD_S370"`, `"FBA_C370"`, `"FBA_S370"`).
pub(crate) fn verify_hercules_dasd_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HERCULES_DASD_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"CKD_P370")
            || starts(prefix, b"CKD_C370")
            || starts(prefix, b"CKD_S370")
            || starts(prefix, b"CKD_P064")
            || starts(prefix, b"CKD_C064")
            || starts(prefix, b"CKD_S064")
            || starts(prefix, b"FBA_P370")
            || starts(prefix, b"FBA_C370")
            || starts(prefix, b"FBA_S370")
            || starts(prefix, b"FBA_P064")
            || starts(prefix, b"FBA_C064")
            || starts(prefix, b"FBA_S064"),
        has_non_ascii_text_byte(prefix.get(8..24)?)
    );
    Some(Label::HerculesDasd)
}

/// HxC Floppy Emulator image (`HXCPICFE` / `HXCHFEV3` / `HXCSTREAM`).
pub(crate) fn verify_hfe(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 16, prefix.len() >= 16);
    if (starts(prefix, b"HXCPICFE") || starts(prefix, b"HXCHFEV3"))
        && (1..=120).contains(&u8_at(prefix, 9)?)
        && (1..=2).contains(&u8_at(prefix, 10)?)
    {
        return Some(Label::Hfe);
    }
    require!(starts(prefix, b"HXCSTREAM"));
    Some(Label::Hfe)
}

/// IBM 3740 / ECMA-58 Basic Exchange 8-Inch Floppy Image (`.img` / `.dsk`, 1973 IBM).
///
/// Track 0 EBCDIC label layout (128-byte sectors, exact total size `256,256` B):
/// - `@0x0200` (sector 5): EBCDIC `"ERMAP   "`
/// - `@0x0300` (sector 7): EBCDIC `"VOL1"` (or `@0x0380` `"HDR1"` / `"DDR1"`)
/// - `@0x0400` (sector 9): EBCDIC `"HDR1"` or `"DDR1"`.
pub(crate) fn verify_ibm_3740(prefix: &[u8], size: u64) -> Option<Label> {
    const ERMAP_OFF: usize = 0x0200; // @0x0200: track 0 sector 5
    const VOL1_OFF: usize = 0x0300; // @0x0300: track 0 sector 7
    const HDR0_OFF: usize = 0x0380; // @0x0380: track 0 sector 8
    const HDR1_OFF: usize = 0x0400; // @0x0400: track 0 sector 9

    require!(
        size == IBM_3740_DISK_SIZE,
        prefix.len() >= HDR1_OFF + 4,
        eq_at(prefix, ERMAP_OFF, IBM_3740_ERMAP_EBCDIC)
    );
    let has_vol1 = eq_at(prefix, VOL1_OFF, IBM_3740_VOL1_EBCDIC);
    let has_hdr0 = eq_at(prefix, HDR0_OFF, IBM_3740_HDR1_EBCDIC)
        || eq_at(prefix, HDR0_OFF, IBM_3740_DDR1_EBCDIC);
    let has_hdr1 = eq_at(prefix, HDR1_OFF, IBM_3740_HDR1_EBCDIC)
        || eq_at(prefix, HDR1_OFF, IBM_3740_DDR1_EBCDIC);
    require!(has_vol1 || has_hdr0, has_hdr1);

    Some(Label::Ibm3740)
}

/// IBM OS/2 SaveDskF / LoadDskF (`.dsk` / `.1dk` / `.2dk`) floppy disk image (`0x58AA` / `0x59AA` / `0x5AAA` LE):
/// validates FAT media descriptor (`0x00E0..=0x00FF` @2), sector size (`512` @4), FAT count (`1..=2` @10),
/// cylinders (`20..=200` @24), heads (`1..=2` @26), sectors/track (`8..=64` @28), and reserved zero dword (@30).
pub(crate) fn verify_ibm_savedskf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_IBM_SAVEDSKF_SIZE, prefix.len() >= 40);
    require!(u8_at(prefix, 0)? == 0xAA, matches!(u8_at(prefix, 1)?, 0x58..=0x5A));
    let media_type = le16(prefix, 2)?;
    let sector_size = le16(prefix, 4)?;
    let n_fats = u8_at(prefix, 10)?;
    let cylinders = le16(prefix, 24)?;
    let heads = le16(prefix, 26)?;
    let sectors_per_track = le16(prefix, 28)?;
    let reserved = le32(prefix, 30)?;
    require!(
        (0x00E0..=0x00FF).contains(&media_type),
        sector_size == 512,
        (1..=2).contains(&n_fats),
        (20..=200).contains(&cylinders),
        (1..=2).contains(&heads),
        (8..=64).contains(&sectors_per_track),
        reserved == 0
    );
    Some(Label::IbmSavedskf)
}

/// Dave Dunfield ImageDisk floppy image (`.imd`).
pub(crate) fn verify_imd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 32, prefix.len() >= 32, starts(prefix, b"IMD "));
    let scan_end = prefix.len().min(2048);
    let eof_pos = find_pos(prefix.get(..scan_end)?, &[0x1A])?;
    require!(eof_pos >= 4);
    Some(Label::Imd)
}

/// Standard IEEE 802.3 CRC-32 (`0xEDB88320` reflected).
fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

/// Linux Journalling Flash File System v2 (JFFS2) node.
pub(crate) fn verify_jffs2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 12, prefix.len() >= 12);
    let b0 = u8_at(prefix, 0)?;
    let b1 = u8_at(prefix, 1)?;
    let is_le = matches!(b0, 0x85 | 0x84) && b1 == 0x19;
    let is_be = b0 == 0x19 && matches!(b1, 0x85 | 0x84);
    require!(is_le || is_be);
    let (nodetype, totlen, hdr_crc) = if is_le {
        (le16(prefix, 2)?, le32(prefix, 4)?, le32(prefix, 8)?)
    } else {
        (be16(prefix, 2)?, be32(prefix, 4)?, be32(prefix, 8)?)
    };
    require!(
        totlen >= 12,
        (totlen as u64) <= size,
        matches!(nodetype, 0xE001 | 0xE002 | 0x2003 | 0x2004 | 0x2005 | 0xE006 | 0xE007),
    );
    let hdr = get_slice(prefix, 0, 8)?;
    let calc = crc32_ieee(hdr);
    require!(hdr_crc == calc || hdr_crc == (calc ^ 0x6522_DF69));
    Some(Label::Jffs2)
}

/// KryoFlux Raw Magnetic Flux Stream (`.raw`, `0x0D` OOB + `0x04` StreamInfo `"host_date="` / `"KryoFlux DiskSystem"`).
pub(crate) fn verify_kryoflux_stream(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KRYOFLUX_STREAM_SIZE,
        prefix.len() >= 32,
        u8_at(prefix, 0)? == 0x0D,
        matches!(u8_at(prefix, 1)?, 0x01..=0x04)
    );
    let info_len = le16(prefix, 2)? as usize;
    let win = get_slice(prefix, 0, prefix.len().min(256))?;
    require!(
        (8..=512).contains(&info_len),
        contains(win, b"KryoFlux") || contains(win, b"host_date=")
    );
    Some(Label::KryofluxRaw)
}

/// ARM Mbed `littlefs` Microcontroller Flash Filesystem Image (v1 / v2, 2017 ARM).
pub(crate) fn verify_littlefs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_LITTLEFS_SIZE && prefix.len() >= 56);
    // 1. littlefs v2 (`"littlefs"` at @8..16):
    if eq_at(prefix, 8, b"littlefs") {
        // Check metadata entry tag at @4..8 (`F0 0F FF F7` or masked `LFS_TYPE_CREATE` superblock tag)
        // and v2 superblock geometry at @20..44 (`version` major == 2 at @22..24).
        let ver_major = le16(prefix, 22)?;
        let block_size = le32(prefix, 24)?;
        let block_count = le32(prefix, 28)?;
        let name_max = le32(prefix, 32)?;
        let file_max = le32(prefix, 36)?;
        let attr_max = le32(prefix, 40)?;
        require!(
            ver_major == 2
                && block_size >= 128
                && block_count >= 2
                && (1..=1022).contains(&name_max)
                && file_max > 0
                && attr_max <= 1022
        );
        return Some(Label::Littlefs);
    }
    // 2. littlefs v1 (`"littlefs"` at @40..48):
    if eq_at(prefix, 40, b"littlefs") {
        let block_size = le32(prefix, 28)?;
        let block_count = le32(prefix, 32)?;
        let ver_major = le16(prefix, 38)?;
        require!(
            ver_major == 1
                && (u8_at(prefix, 16)? & 0x7F) == 0x2E
                && matches!(u8_at(prefix, 17)?, 0x14 | 0x18)
                && block_size >= 128
                && block_count >= 2
        );
        return Some(Label::Littlefs);
    }
    None
}

/// MAME Floppy Image (`MAMEFLOP`) mapped to HFE/floppy bitstream container.
pub(crate) fn verify_mame_mfi(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 16, prefix.len() >= 16, starts(prefix, b"MAMEFLOP"));
    let cyls = le32(prefix, 8)? & 0x3FFF_FFFF;
    let heads = le32(prefix, 12)?;
    require!((1..=168).contains(&cyls), (1..=2).contains(&heads));
    Some(Label::Hfe)
}

/// Microchip MPFS2 Embedded HTTP Read-Only Filesystem (`.mpfs`).
pub(crate) fn verify_microchip_mpfs(prefix: &[u8], size: u64) -> Option<Label> {
    const MPFS_MAGIC: &[u8; 4] = b"MPFS"; // @0: "MPFS" signature
    const MAJOR_OFF: usize = 4; // @4: u8 major version (2)
    const MINOR_OFF: usize = 5; // @5: u8 minor version (0 or 1)
    const NUM_FILES_OFF: usize = 6; // @6: u16 LE file count (1..=4096)
    require!(
        size >= MIN_MICROCHIP_MPFS_SIZE,
        prefix.len() >= 16,
        starts(prefix, MPFS_MAGIC),
        u8_at(prefix, MAJOR_OFF) == Some(2),
        matches!(u8_at(prefix, MINOR_OFF), Some(0 | 1))
    );
    let num_files = le16(prefix, NUM_FILES_OFF)?;
    require!((1..=4096).contains(&num_files));
    Some(Label::MicrochipMpfs)
}

/// Microsoft Hyper-V Virtual Hard Disk v2 (.vhdx).
pub(crate) fn verify_microsoft_vhdx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MICROSOFT_VHDX_SIZE
            && prefix.len() >= 16
            && starts(prefix, b"vhdxfile")
            && prefix.get(8..16)?.contains(&0)
    );
    Some(Label::Vhdx)
}

/// Applesauce MOOF: `"MOOF\xFF\n\r\n"` (high-bit + line-ending corruption detector), `u32 LE` CRC32, first chunk `"INFO"`.
pub(crate) fn verify_moof(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = b"MOOF\xFF\n\r\n"; // @0, 8 bytes
    const FIRST_CHUNK: usize = 12; // @12: chunk id "INFO"
    require!(size >= MIN_MOOF_SIZE, starts(prefix, MAGIC), eq_at(prefix, FIRST_CHUNK, b"INFO"));
    Some(Label::Moof)
}

/// MSX Cassette Tape Image (`.cas`, `1F A6 DE BA CC 13 7D 74`).
pub(crate) fn verify_msx_cas(prefix: &[u8], size: u64) -> Option<Label> {
    const MSX_CAS_SYNC: &[u8; 8] = hex2bin!("1F A6 DE BA CC 13 7D 74");
    require!(size >= MIN_MSX_CAS_SIZE, prefix.len() >= 16, starts(prefix, MSX_CAS_SYNC));
    Some(Label::MsxCas)
}

/// SNK Neo Geo Pocket / Pocket Color Cartridge ROM (.ngp / .ngc).
pub(crate) fn verify_neo_geo_pocket_rom(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_NEO_GEO_POCKET_SIZE && prefix.len() >= 64);

    let is_snk = starts(prefix, b"COPYRIGHT BY SNK CORPORATION")
        || starts(prefix, b" LICENSED BY SNK CORPORATION");
    require!(is_snk);

    let sys_code = u8_at(prefix, 0x23)?;
    // 0x00 = Monochrome NGP, 0x10 = Color NGPC
    require!(sys_code == 0x00 || sys_code == 0x10);

    // Reserved 16 bytes at 0x30..0x40 must be zero
    require!(get_slice(prefix, 0x30, 0x40)? == [0u8; 16]);

    Some(Label::NgpRom)
}

/// Nintendo 64 ROM Image (.z64 / .n64 / .v64).
pub(crate) fn verify_nintendo64_rom(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_N64_ROM_SIZE && prefix.len() >= 64);

    let is_be = starts(prefix, &[0x80, 0x37, 0x12, 0x40]);
    let is_swapped = starts(prefix, &[0x37, 0x80, 0x40, 0x12]);
    let is_le = starts(prefix, &[0x40, 0x12, 0x37, 0x80]);

    require!(is_be || is_swapped || is_le);

    // If big-endian (.z64), check internal game title at 0x20..0x34 (20 bytes)
    if is_be {
        let title = get_slice(prefix, 0x20, 0x34)?;
        let valid_chars = title.iter().all(|&b| (0x20..=0x7E).contains(&b) || b == 0);
        require!(valid_chars);
    }

    Some(Label::N64Rom)
}

/// Nintendo Famicom Disk System Disk Image (.fds).
pub(crate) fn verify_nintendo_fds_disk(prefix: &[u8], size: u64) -> Option<Label> {
    const FDS_BLOCK1_MAGIC: &[u8; 15] = b"\x01*NINTENDO-HVC*";
    require!(size >= MIN_FDS_DISK_SIZE, prefix.len() >= 32);
    if starts(prefix, b"FDS\x1A") {
        let num_sides = u8_at(prefix, 4)?;
        require!(
            size >= 65_516,
            (1..=16).contains(&num_sides),
            get_slice(prefix, 5, 16)?.iter().all(|&b| b == 0),
            get_slice(prefix, 16, 31)? == FDS_BLOCK1_MAGIC
        );
        return Some(Label::Fds);
    }
    require!(starts(prefix, FDS_BLOCK1_MAGIC));
    Some(Label::Fds)
}

/// Nintendo Entertainment System / Famicom ROM (.nes / iNES / NES 2.0).
pub(crate) fn verify_nintendo_nes_rom(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_NES_ROM_SIZE && prefix.len() >= 16);
    require!(starts(prefix, b"NES\x1A"));

    let prg_banks = u8_at(prefix, 4)? as u64;
    let chr_banks = u8_at(prefix, 5)? as u64;
    let flags6 = u8_at(prefix, 6)?;

    // Must have at least 1 PRG ROM bank (16 KB)
    require!(prg_banks >= 1);

    let has_trainer = (flags6 & 0x04) != 0;
    let expected_data_len =
        16 + (if has_trainer { 512 } else { 0 }) + prg_banks * 16384 + chr_banks * 8192;
    if expected_data_len > 16 {
        require!(size + 1024 >= expected_data_len);
    }

    Some(Label::NesRom)
}

/// Linux-NTFS `ntfsclone` Sparse Backup Image (`"\0ntfsclone-image"`).
pub(crate) fn verify_ntfsclone_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NTFSCLONE_SIZE,
        prefix.len() >= 18,
        starts(prefix, b"\x00ntfsclone-image"),
        u8_at(prefix, 16)? == 10
    );
    Some(Label::NtfscloneImg)
}

/// Oric `MFM_DISK`: `u32 LE sides` (1..=2), `u32 LE tracks` (1..=99), `u32 LE geometry` (1..=2), and the file size is
/// exactly `256 + sides * tracks * 6400`.
pub(crate) fn verify_oric_mfm_disk(prefix: &[u8], size: u64) -> Option<Label> {
    const SIDES: usize = 8; // @8: u32 LE
    const TRACKS: usize = 12; // @12: u32 LE
    const GEOMETRY: usize = 16; // @16: u32 LE
    require!(
        size >= MIN_ORIC_MFM_DISK_SIZE,
        starts(prefix, b"MFM_DISK") || starts(prefix, b"ORICDISK")
    );
    let sides = le32(prefix, SIDES)?;
    let tracks = le32(prefix, TRACKS)?;
    let geom = le32(prefix, GEOMETRY)?;
    require!((1..=2).contains(&sides), (1..=99).contains(&tracks));
    let track_size = if starts(prefix, b"ORICDISK") && geom > 2 {
        geom as u64 * 256
    } else {
        require!((1..=2).contains(&geom));
        ORIC_MFM_TRACK_SIZE
    };
    require!(size == ORIC_MFM_HEADER_SIZE + sides as u64 * tracks as u64 * track_size);
    Some(Label::OricDsk)
}

/// Parallels Desktop Hard Disk Image (`.hdd`, `"WithoutFreeSpace"` / `"WithouFreSpacExt"`).
pub(crate) fn verify_parallels_hdd_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PARALLELS_HDD_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"WithoutFreeSpace") || starts(prefix, b"WithouFreSpacExt"),
        le32(prefix, 16) == Some(2),
        le32(prefix, 20)? > 0,
        le32(prefix, 24)? > 0
    );
    Some(Label::ParallelsHdd)
}

/// NEC PC-98 Virtual Floppy Disk (`.fdd`, `"VFD1.00\0"`).
pub(crate) fn verify_pc98_vfd_fdd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PC98_VFD_SIZE, prefix.len() >= 16, starts(prefix, b"VFD1.0"));
    Some(Label::VfdFdd)
}

/// Pismo File Mount Compact ISO Archive (`.cfs` / `.ciso`, 2006 Pismo Technic, `"ci%p"`).
pub(crate) fn verify_pismo_compact_iso(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PISMO_CISO_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"ci%p"),
        eq_at(prefix, 8, b"\x01\x01\x01iso\0\0")
    );
    Some(Label::PismoCiso)
}

/// PowerISO Direct-Access-Archive Disc Image (.daa).
pub(crate) fn verify_poweriso_daa(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_POWERISO_DAA_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"DAA\0\0\0\0\0") || starts(prefix, b"DAAC\0\0\0\0")
    );
    Some(Label::PowerisoDaa)
}

/// Dark_AleX PlayStation Portable DAX Compressed ISO (.dax).
pub(crate) fn verify_psp_dax(prefix: &[u8], size: u64) -> Option<Label> {
    const UNPACKED_SIZE_OFF: usize = 4; // @4: u32 LE total uncompressed ISO byte size (> 0)
    const VERSION_OFF: usize = 8; // @8: u32 LE DAX format version (0 or 1)
    const NC_AREAS_OFF: usize = 12; // @12: u32 LE number of non-compressed sector ranges
    const RESERVED_OFF: usize = 16; // @16..32: 16 reserved zero bytes

    require!(
        size >= MIN_PSP_DAX_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"DAX\0"),
        eq_at(prefix, RESERVED_OFF, &[0u8; 16])
    );
    let unpacked_size = le32(prefix, UNPACKED_SIZE_OFF)?;
    let version = le32(prefix, VERSION_OFF)?;
    let nc_areas = le32(prefix, NC_AREAS_OFF)?;
    require!(unpacked_size > 0, version <= 1, version == 1 || nc_areas == 0);
    Some(Label::PspDax)
}

/// QEMU Copy-On-Write v1/v2/v3 Disk Image (`.qcow` / `.qcow2`, `"QFI\xFB"`).
pub(crate) fn verify_qcow2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_QCOW2_SIZE, prefix.len() >= 36, starts(prefix, b"QFI\xFB"));
    let ver = be32(prefix, 4)?;
    let valid_cluster_bits = match ver {
        1 => (9..=21).contains(&u8_at(prefix, 32)?) && (1..=21).contains(&u8_at(prefix, 33)?),
        2 | 3 => (9..=21).contains(&be32(prefix, 20)?),
        _ => false,
    };
    require!(valid_cluster_bits);
    Some(Label::Qcow2)
}

/// QEMU Enhanced Disk Image (`.qed`, `"QED\0"`).
pub(crate) fn verify_qemu_qed_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_QEMU_QED_SIZE, prefix.len() >= 16, starts(prefix, b"QED\0"));
    let cluster_size = le32(prefix, 4)?;
    let table_size = le32(prefix, 8)?;
    require!(
        cluster_size >= 4096 && cluster_size.is_power_of_two(),
        (1..=16).contains(&table_size)
    );
    Some(Label::QemuQed)
}

/// BlackBerry QNX `deflate` / `inflator` Block-Compressed Filesystem Stream (`.deflated`).
pub(crate) fn verify_qnx_deflate(prefix: &[u8], size: u64) -> Option<Label> {
    const QNX_DEFLATE_MAGIC: &[u8; 8] = b"iwlyfmbp"; // @0: 8-byte signature ("iwlyfmbp")
    const USIZE_OFF: usize = 8; // @8: u32 LE uncompressed file size (> 0)
    const BLKSIZE_OFF: usize = 12; // @12: u16 LE compression block size (4096, 8192, 16384, or 32768)
    const CMPTYPE_OFF: usize = 14; // @14: u8 compression algorithm (0=LZO, 1=UCL, 2=zlib)
    const FLAGS_OFF: usize = 15; // @15: u8 reserved flags (0)
    require!(size >= MIN_QNX_DEFLATE_SIZE, prefix.len() >= 20, starts(prefix, QNX_DEFLATE_MAGIC));
    let usize_val = le32(prefix, USIZE_OFF)?;
    let blksize = le16(prefix, BLKSIZE_OFF)?;
    let cmptype = u8_at(prefix, CMPTYPE_OFF)?;
    let flags = u8_at(prefix, FLAGS_OFF)?;
    require!(
        usize_val > 0,
        matches!(blksize, 4096 | 8192 | 16384 | 32768),
        cmptype <= 2,
        flags == 0
    );
    Some(Label::QnxDeflate)
}

/// Linux ROMFS Read-Only Filesystem Image (.romfs).
pub(crate) fn verify_romfs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ROMFS_SIZE
            && prefix.len() >= 20
            && starts(prefix, b"-rom1fs-")
            && be32(prefix, 8).unwrap_or(0) >= 32
            && prefix.get(8..16)?.contains(&0)
    );
    Some(Label::Romfs)
}

/// SAB Diskette Utility: `"SAB Diskette Utility"` terminated by NUL (or `" Version x.y"` in later releases).
pub(crate) fn verify_sabdu(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = b"SAB Diskette Utility"; // @0, 20 bytes
    const TERMINATOR: usize = 20; // @20: NUL or " Version x.y\0"
    require!(size >= MIN_SABDU_SIZE, prefix.len() >= 32, starts(prefix, MAGIC));
    require!(
        u8_at(prefix, TERMINATOR)? == 0x00
            || (eq_at(prefix, 20, b" Version ")
                && u8_at(prefix, 29)?.is_ascii_digit()
                && u8_at(prefix, 30)? == b'.'
                && u8_at(prefix, 31)?.is_ascii_digit()),
        prefix.get(20..prefix.len().min(64))?.contains(&0)
    );
    Some(Label::Sabdu)
}

/// SuperCard Pro magnetic flux image (`.scp`).
pub(crate) fn verify_scp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 16,
        prefix.len() >= 16,
        starts(prefix, b"SCP"),
        u8_at(prefix, 4)? <= 0x80,
        (1..=10).contains(&u8_at(prefix, 5)?),
        u8_at(prefix, 6)? <= u8_at(prefix, 7)?,
        u8_at(prefix, 7)? <= 168,
        has_non_ascii_text_byte(prefix.get(8..16)?),
    );
    Some(Label::Scp)
}

/// Headerless Sega Master System / Game Gear Z80 ROM: reset bootstrap (`F3 ED 56` = `DI; IM 1` + `C3`/`18`/`31` jump/SP load) and exact power-of-two/multiple-of-16KiB cartridge size.
pub(crate) fn verify_sega_sms_rom_bootstrap(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SEGA_SMS_ROM_SIZE
            && (size.is_multiple_of(8_192)
                || (size >= 8_704 && (size - 512).is_multiple_of(8_192)))
            && prefix.len() >= 16
            && starts(prefix, hex2bin!("F3 ED 56"))
            && matches!(u8_at(prefix, 3)?, 0xC3 | 0x18 | 0x31 | 0xAF | 0x3E | 0x21 | 0x00)
    );
    Some(Label::SmsRom)
}

/// SIMH / Ersatz-11 Magnetic Tape Image (`.tap`, 1993) Bob Supnik / John Wilson.
///
/// A headerless stream of records, each framed as `u32 LE length` + payload (+ pad byte for
/// odd lengths in SIMH) + the same `u32 LE length`; `0x00000000` is a tape mark and
/// `0xFFFFFFFF` end-of-medium. The verifier skips up to two leading tape marks, requires the
/// first record's leading/trailing lengths to agree, and then requires the next framing word
/// to be a tape mark, end-of-medium, or another fully self-consistent record (or exact EOF).
pub(crate) fn verify_simh_tape(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SIMH_TAPE_SIZE, prefix.len() >= MIN_SIMH_TAPE_SIZE as usize);
    let mut pos = 0usize;
    let mut marks = 0usize;
    while marks < MAX_SIMH_LEADING_TAPE_MARKS && le32(prefix, pos) == Some(SIMH_TAPE_MARK) {
        pos += SIMH_META_LEN;
        marks += 1;
    }
    let first_end = simh_record_end(prefix, pos)?;
    require!(first_end as u64 <= size);
    if first_end as u64 != size {
        match le32(prefix, first_end)? {
            SIMH_TAPE_MARK | SIMH_END_OF_MEDIUM => {}
            meta => {
                let len = meta & SIMH_RECORD_LEN_MASK;
                require!(meta & !SIMH_RECORD_LEN_MASK == 0, len >= MIN_SIMH_RECORD_LEN);
                // Fully verify the second record when it lies inside the prefix window.
                if first_end + 2 * SIMH_META_LEN + (len as usize) < prefix.len() {
                    simh_record_end(prefix, first_end)?;
                }
            }
        }
    }
    Some(Label::SimhTap)
}

/// Sinclair QL QDOS Filesystem Disk Image (`.img`, 1984 Sinclair, `"QL5A"` / `"QL5B"`).
pub(crate) fn verify_sinclair_qdos(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SINCLAIR_QDOS_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"QL5A") || starts(prefix, b"QL5B"),
        get_slice(prefix, 4, 14)?.iter().all(|&b| (0x20..=0x7E).contains(&b)),
        be16(prefix, 14)? > 0,
        be16(prefix, 24)? > 0,
        be16(prefix, 26)? > 0,
        be16(prefix, 28)? >= be16(prefix, 26)?,
        be16(prefix, 30)? > 0,
        has_non_ascii_text_byte(prefix.get(..32)?)
    );
    Some(Label::SinclairQdos)
}

/// Stacker v1.x–v4.x compressed volume: `"STACKER  version  "` followed by `'1'..='4'` and `"volume:"`.
pub(crate) fn verify_stacker(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8] = b"STACKER  version  "; // @0, 18 bytes
    require!(
        size >= MIN_STACKER_SIZE,
        starts(prefix, MAGIC),
        matches!(prefix.get(18), Some(b'1'..=b'4')),
        scan_range(prefix, 19, 36, b"volume:")
    );
    Some(Label::Stacker)
}

/// NEC PC-9801 T98-Next Floppy Image (`.nfd`, `"T98FDDIMAGE.R0\0\0"` / `"T98FDDIMAGE.R1\0\0"`).
pub(crate) fn verify_t98next_nfd_image(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_T98NEXT_NFD_SIZE,
        starts(prefix, b"T98FDDIMAGE.R0\0\0") || starts(prefix, b"T98FDDIMAGE.R1\0\0")
    );
    Some(Label::T98nextNfd)
}

/// Sydex TeleDisk floppy image (.td0).
pub(crate) fn verify_teledisk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_TELEDISK_SIZE && prefix.len() >= 12);
    require!(matches!(get_slice(prefix, 0, 2)?, b"TD" | b"td"));
    let version = u8_at(prefix, 4)?;
    let data_rate = u8_at(prefix, 5)?;
    let drive_type = u8_at(prefix, 6)?;
    let stepping = u8_at(prefix, 7)?;
    let surfaces = u8_at(prefix, 9)?;
    require!(
        (10..=21).contains(&version)
            && ((data_rate & 0x7F) <= 3)
            && drive_type <= 6
            && (stepping & 0x7C) == 0
            && matches!(surfaces, 1 | 2)
    );
    let expected_crc = le16(prefix, 10)?;
    let computed_crc = crc16_teledisk(get_slice(prefix, 0, 10)?);
    require!(expected_crc == computed_crc);
    Some(Label::Td0)
}

/// TP-LINK MiniFS Embedded Read-Only Filesystem (`.minifs`).
pub(crate) fn verify_tplink_minifs(prefix: &[u8], size: u64) -> Option<Label> {
    const MINIFS_MAGIC: &[u8; 16] = b"MINIFS\0\0\0\0\0\0\0\0\0\0"; // @0: 6B magic + 10B reserved zero padding
    const DIR_COUNT_OFF: usize = 16; // @16: u32 BE directory count (1..=4096)
    const FILE_COUNT_OFF: usize = 20; // @20: u32 BE file count (1..=65536)
    const RAW_SIZE_OFF: usize = 24; // @24: u32 BE uncompressed data size (> 0)
    const STR_SIZE_OFF: usize = 28; // @28: u32 BE string table size (> 0)
    require!(size >= MIN_TPLINK_MINIFS_SIZE, prefix.len() >= 36, starts(prefix, MINIFS_MAGIC));
    let dirs = be32(prefix, DIR_COUNT_OFF)?;
    let files = be32(prefix, FILE_COUNT_OFF)?;
    let raw_sz = be32(prefix, RAW_SIZE_OFF)?;
    let str_sz = be32(prefix, STR_SIZE_OFF)?;
    require!((1..=4096).contains(&dirs), (1..=65536).contains(&files), raw_sz > 0, str_sz > 0);
    Some(Label::TplinkMinifs)
}

/// Toshiba TROC Read-Only Embedded Filesystem (`.troc`).
pub(crate) fn verify_troc_fs(prefix: &[u8], size: u64) -> Option<Label> {
    const TROC_MAGIC: &[u8; 8] = bytes!(b"TROC", le32_bytes!(1)); // @0: "TROC" + version 1 LE
    const COUNT_OFF: usize = 8; // @8: u32 LE file count (1..=65536)
    const PATH_OFF: usize = 14; // @14: NUL-terminated path starting with '/' (0x2F)
    require!(size >= MIN_TROC_FS_SIZE, prefix.len() >= 24, starts(prefix, TROC_MAGIC));
    let count = le32(prefix, COUNT_OFF)?;
    let first_char = u8_at(prefix, PATH_OFF)?;
    require!((1..=65536).contains(&count), first_char == b'/');
    Some(Label::TrocFs)
}

/// Linux Unsorted Block Image / UBIFS Flash Filesystem (.ubi / .ubifs).
pub(crate) fn verify_ubifs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_UBIFS_SIZE && prefix.len() >= 24);
    let valid = if starts(prefix, b"UBI#") || starts(prefix, b"UBI!") {
        u8_at(prefix, 4)? == 1 && get_slice(prefix, 5, 8)? == [0, 0, 0]
    } else {
        starts(prefix, hex2bin!("31 18 10 06"))
            && le32(prefix, 16).unwrap_or(0) >= 24
            && u8_at(prefix, 20)? <= 12
    };
    require!(valid);
    Some(Label::Ubifs)
}

/// UCSD Pascal p-System disk volume (`.vol` / `.dsk`, 1977) UC San Diego / SofTech Microsystems.
///
/// Validates the 26-byte block-2 volume header at `@1024` (both LE and BE `u16` variants):
/// `dfirstblk == 0`, `dlastblk` in `{6, 10}`, `dfkind == 0`, 1..=7-byte printable volume name,
/// `deovblk >= 16`, `dnumfiles` in `1..=77`, and monotonic non-overlapping live file entries with
/// valid 1..=15-byte printable filenames and `dlastbyte` in `1..=512`.
pub(crate) fn verify_ucsd_vol(prefix: &[u8], size: u64) -> Option<Label> {
    const DIR_OFF: usize = 1024; // @1024: Block 2 UCSD directory header (26 bytes)
    const ENTRY_SIZE: usize = 26;
    require!(size >= MIN_UCSD_VOL_SIZE, prefix.len() >= DIR_OFF + ENTRY_SIZE * 2);

    let parse_dir = |big_endian: bool| -> Option<usize> {
        let u16_at = |o: usize| -> Option<u16> {
            if big_endian { be16(prefix, o) } else { le16(prefix, o) }
        };
        let dfirst = u16_at(DIR_OFF)?;
        let dlast = u16_at(DIR_OFF + 2)?;
        let dfkind = u16_at(DIR_OFF + 4)?;
        let vlen = u8_at(prefix, DIR_OFF + 6)? as usize;
        let deov = u16_at(DIR_OFF + 14)?;
        let dnum = u16_at(DIR_OFF + 16)? as usize;

        if dfirst != 0 || (dlast != 6 && dlast != 10) || dfkind != 0 {
            return None;
        }
        if !(1..=7).contains(&vlen)
            || !get_slice(prefix, DIR_OFF + 7, DIR_OFF + 7 + vlen)?
                .iter()
                .all(|&b| (0x21..=0x7E).contains(&b))
        {
            return None;
        }
        if !(16..=32768).contains(&deov) || !(1..=77).contains(&dnum) {
            return None;
        }
        let dir_end = DIR_OFF + (dnum + 1) * ENTRY_SIZE;
        if dir_end > prefix.len() || dir_end > (size as usize) {
            return None;
        }

        let mut prev_end = dlast;
        for slot in 1..=dnum {
            let eo = DIR_OFF + slot * ENTRY_SIZE;
            let fb = u16_at(eo)?;
            let lb = u16_at(eo + 2)?;
            let fk = u16_at(eo + 4)?;
            let nl = u8_at(prefix, eo + 6)? as usize;
            let lastb = u16_at(eo + 22)?;
            if fb < prev_end || fb >= lb || lb > deov || (fk & 0x0F) > 8 {
                return None;
            }
            if !(1..=15).contains(&nl)
                || !get_slice(prefix, eo + 7, eo + 7 + nl)?
                    .iter()
                    .all(|&b| (0x21..=0x7E).contains(&b))
            {
                return None;
            }
            if !(1..=512).contains(&lastb) {
                return None;
            }
            prev_end = lb;
        }
        Some(dir_end)
    };

    parse_dir(false).or_else(|| parse_dir(true))?;
    Some(Label::UcsdVol)
}

/// Tektronix 4404 / TSC UniFLEX filesystem disk image (`.img`, 1984) Tektronix / Technical Systems Consultants.
///
/// Validates the big-endian System Information Record (SIR) at block 1 (`@512`):
/// zero lock/modify bytes `@512..520`, non-zero timestamps `@520..528`, `s_isize` (`@528`),
/// 24-bit `s_fsize` (`@530..533`), `s_nfree <= s_fsize` (`@533..536`), `s_ninode <= s_isize * 8`
/// (`@536..538`), NUL-padded printable ASCII volume label `s_fname` (`@538..552`), and either a
/// `"4400P"` Tektronix boot signature or zeroed boot header at `@0..16`.
pub(crate) fn verify_uniflex_img(prefix: &[u8], size: u64) -> Option<Label> {
    const SIR_OFF: usize = 512; // @512: UniFLEX System Information Record (SIR)
    const LOCK_LEN: usize = 8; // @512..520: lock/modify bytes
    const ISIZE_OFF: usize = SIR_OFF + 16; // @528..530: u16 BE s_isize (inode blocks)
    const FSIZE_OFF: usize = SIR_OFF + 18; // @530..533: u24 BE s_fsize (total blocks - 1)
    const NFREE_OFF: usize = SIR_OFF + 21; // @533..536: u24 BE s_nfree
    const NINODE_OFF: usize = SIR_OFF + 24; // @536..538: u16 BE s_ninode
    const FNAME_OFF: usize = SIR_OFF + 26; // @538..552: 14-byte NUL-padded ASCII volume name
    const FPACK_OFF: usize = SIR_OFF + 40; // @552..566: 14-byte NUL-padded ASCII pack name
    const SIR_CHECK_END: usize = SIR_OFF + 54; // @566

    require!(
        size >= MIN_UNIFLEX_IMG_SIZE,
        prefix.len() >= SIR_CHECK_END,
        has_non_ascii_text_byte(prefix.get(SIR_OFF..SIR_CHECK_END)?)
    );

    let has_tek_boot = starts(prefix, TEKTRONIX_4404_BOOT_PREFIX);
    let zero_boot = get_slice(prefix, 0, 16)?.iter().all(|&b| b == 0);
    require!(has_tek_boot || zero_boot);

    // Unmounted clean UniFLEX SIR has zeroed 8-byte lock/flags block at @512..520
    require!(get_slice(prefix, SIR_OFF, SIR_OFF + LOCK_LEN)?.iter().all(|&b| b == 0));

    let isize = be16(prefix, ISIZE_OFF)? as u32;
    let fsize = ((u8_at(prefix, FSIZE_OFF)? as u32) << 16)
        | ((u8_at(prefix, FSIZE_OFF + 1)? as u32) << 8)
        | (u8_at(prefix, FSIZE_OFF + 2)? as u32);
    let nfree = ((u8_at(prefix, NFREE_OFF)? as u32) << 16)
        | ((u8_at(prefix, NFREE_OFF + 1)? as u32) << 8)
        | (u8_at(prefix, NFREE_OFF + 2)? as u32);
    let ninode = be16(prefix, NINODE_OFF)? as u32;

    require!(
        (1..=4096).contains(&isize),
        fsize > isize && fsize <= 1_000_000,
        nfree <= fsize,
        ninode <= isize * 8
    );

    // s_fname: >= 3 printable ASCII chars followed strictly by NUL padding
    let fname = get_slice(prefix, FNAME_OFF, FNAME_OFF + 14)?;
    let name_len = fname.iter().position(|&b| b == 0).unwrap_or(14);
    require!(
        name_len >= 3,
        fname[..name_len].iter().all(|&b| (0x20..=0x7E).contains(&b)),
        fname[name_len..].iter().all(|&b| b == 0)
    );
    // s_fpack: printable ASCII prefix + NUL padding (may be empty)
    let fpack = get_slice(prefix, FPACK_OFF, FPACK_OFF + 14)?;
    let pack_len = fpack.iter().position(|&b| b == 0).unwrap_or(14);
    require!(
        fpack[..pack_len].iter().all(|&b| (0x20..=0x7E).contains(&b)),
        fpack[pack_len..].iter().all(|&b| b == 0)
    );

    Some(Label::UniflexImg)
}

/// SCO UnixWare Boot File System Image (`.bfs`, `0x1BADFACE` LE).
pub(crate) fn verify_unixware_bfs(prefix: &[u8], size: u64) -> Option<Label> {
    const BFS_MAGIC_LE: &[u8; 4] = hex2bin!("CE FA AD 1B"); // 0x1BADFACE LE
    require!(
        size >= MIN_UNIXWARE_BFS_SIZE,
        prefix.len() >= 32,
        starts(prefix, BFS_MAGIC_LE),
        eq_at(prefix, 12, &[0xFF; 16])
    );
    let start = le32(prefix, 4)?;
    let end = le32(prefix, 8)?;
    require!(start >= 512, end > start);
    Some(Label::UnixwareBfs)
}

/// VMware Virtual Disk (`.vmdk`, `"KDMV"` or `"COWD"`).
pub(crate) fn verify_vmdk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VMDK_SIZE,
        prefix.len() >= 28,
        (starts(prefix, b"KDMV") && matches!(le32(prefix, 4), Some(1..=3)))
            || (starts(prefix, b"COWD") && le32(prefix, 4) == Some(1))
    );
    Some(Label::Vmdk)
}

/// Applesauce WOZ Apple II flux/bitstream floppy image (`WOZ1` / `WOZ2`).
pub(crate) fn verify_woz(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 8,
        prefix.len() >= 8,
        starts(prefix, b"WOZ1\xFF\n\r\n") || starts(prefix, b"WOZ2\xFF\n\r\n"),
    );
    Some(Label::Woz)
}

/// Xerox Alto Diablo 31 / Diablo 44 Disk Pack Image (`.dsk`, 1973) Xerox PARC.
///
/// Exact-size geometry (4,872 or 9,744 sectors of 534 bytes) plus the invariant that VDA 1 is
/// the `SysDir` leader page: `header[1] == 0x1000`, `label.filepage == 0`,
/// `label.fid == {1, 0x8000, ...}` and leader name `"SysDir."`. Both little-endian-word
/// (Salto / ContrAlto, `aar`) and big-endian-word dumps are recognised via the name byte order.
pub(crate) fn verify_xerox_alto_disk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size == ALTO_DIABLO31_IMAGE_SIZE || size == ALTO_DIABLO44_IMAGE_SIZE);
    let base = ALTO_SECTOR_SIZE * ALTO_SYSDIR_VDA;
    let word: fn(&[u8], usize) -> Option<u16> =
        if eq_at(prefix, base + ALTO_LEADER_NAME_OFFSET, ALTO_SYSDIR_NAME_SWAPPED) {
            |p, o| le16(p, o)
        } else if eq_at(prefix, base + ALTO_LEADER_NAME_OFFSET, ALTO_SYSDIR_NAME) {
            |p, o| be16(p, o)
        } else {
            return None;
        };
    require!(
        word(prefix, base + 2 * ALTO_HEADER_RDA_WORD) == Some(ALTO_SYSDIR_RDA),
        word(prefix, base + 2 * ALTO_LABEL_FILEPAGE_WORD) == Some(0),
        word(prefix, base + 2 * ALTO_LABEL_FID0_WORD) == Some(ALTO_FID_IN_USE),
        word(prefix, base + 2 * ALTO_LABEL_FID1_WORD) == Some(ALTO_FID_DIRECTORY)
    );
    Some(Label::AltoDsk)
}

/// SGI / Linux XFS Filesystem Superblock (`"XFSB"` at `@0..4`).
pub(crate) fn verify_xfs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_XFS_SIZE, prefix.len() >= 32, starts(prefix, b"XFSB"));
    let blocksize = be32(prefix, 4)?;
    require!(
        (512..=65536).contains(&blocksize) && blocksize.is_power_of_two(),
        be64(prefix, 8)? > 0
    );
    Some(Label::Xfs)
}

/// ZX Spectrum / Amstrad CPC Digital Tape Image (.tzx / .cdt).
pub(crate) fn verify_zx_tzx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ZX_TZX_SIZE
            && prefix.len() >= 11
            && starts(prefix, b"ZXTape!\x1A")
            && u8_at(prefix, 8)? == 1
            && u8_at(prefix, 9)? <= 50
    );
    Some(Label::ZxTzx)
}

/// MacBinary-wrapped Apple DiskCopy 6.0 NDIF image (`rohdddsk` / `dimgddsk` at `@65`).
pub(crate) fn verify_apple_ndif_offset(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 128,
        prefix.len() >= 128,
        u8_at(prefix, 0)? == 0,
        (1..=63).contains(&u8_at(prefix, 1)?),
        u8_at(prefix, 74)? == 0,
        u8_at(prefix, 82)? == 0,
        eq_at(prefix, 65, b"rohdddsk") || eq_at(prefix, 65, b"dimgddsk"),
    );
    Some(Label::Ndif)
}
