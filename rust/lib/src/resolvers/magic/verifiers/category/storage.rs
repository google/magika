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
