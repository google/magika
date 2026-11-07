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

//! Microsoft OLE2 Compound File Binary (MS-CFB) container subtype resolver for Magika's
//! OLE2-backed content types (`doc`, `xls`, `ppt`, `visio`, `msi`, `outlook`, `thumbsdb`,
//! `sketchup`, `3dsm`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// 8-byte OLE2 Compound File Binary header signature (`D0 CF 11 E0 A1 B1 1A E1`).
pub(crate) const OLE2_MAGIC: &[u8; 8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

const OLE2_HEADER_SIZE: usize = 512;
const OLE2_DIR_ENTRY_SIZE: usize = 128;
const OLE2_OFF_SECTOR_SHIFT: usize = 30; // @30: u16 LE — sector shift (9 = 512 B, 12 = 4096 B)
const OLE2_OFF_NUM_FAT_SECTORS: usize = 44; // @44: u32 LE — number of FAT sectors
const OLE2_OFF_FIRST_DIR_SECTOR: usize = 48; // @48: u32 LE — first directory sector ID
const OLE2_OFF_HEADER_DIFAT: usize = 76; // @76..512: 109 inline DIFAT sector IDs
const OLE2_HEADER_DIFAT_COUNT: usize = 109;

const OLE2_DIR_OFF_NAME_LEN: usize = 64; // @64: u16 LE — name byte length (including UTF-16 NUL)
const OLE2_DIR_OFF_OBJ_TYPE: usize = 66; // @66: u8 — 0=unallocated, 1=storage, 2=stream, 5=root
const OLE2_DIR_OFF_CLSID: usize = 80; // @80..96: 16-byte mixed-endian CLSID

const FREESECT: u32 = 0xFFFF_FFFF;
const ENDOFCHAIN: u32 = 0xFFFF_FFFE;
const MAXREGSECT: u32 = 0xFFFF_FFF0;

const CLSID_WORD_97: &[u8; 16] = guid!("00020906-0000-0000-C000-000000000046");
const CLSID_WORD_95: &[u8; 16] = guid!("00020900-0000-0000-C000-000000000046");
const CLSID_EXCEL_97: &[u8; 16] = guid!("00020820-0000-0000-C000-000000000046");
const CLSID_EXCEL_95: &[u8; 16] = guid!("00020810-0000-0000-C000-000000000046");
const CLSID_EXCEL_CHART: &[u8; 16] = guid!("00020821-0000-0000-C000-000000000046");
const CLSID_PPT_97: &[u8; 16] = guid!("64818D10-4F9B-11CF-86EA-00AA00B929E8");
const CLSID_PPT_95: &[u8; 16] = guid!("64818D10-4F9B-11CF-86EA-00AA004863B6");
const CLSID_PPT_SHOW: &[u8; 16] = guid!("00044851-0000-0000-C000-000000000046");
const CLSID_VISIO_2003: &[u8; 16] = guid!("00021A14-0000-0000-C000-000000000046");
const CLSID_VISIO_2000: &[u8; 16] = guid!("00021A13-0000-0000-C000-000000000046");
const CLSID_MSI_PACKAGE: &[u8; 16] = guid!("000C1084-0000-0000-C000-000000000046");
const CLSID_MSI_PATCH: &[u8; 16] = guid!("000C1086-0000-0000-C000-000000000046");
const CLSID_MSI_TRANSFORM: &[u8; 16] = guid!("000C1082-0000-0000-C000-000000000046");
const CLSID_MSI_TRANSFORM_OLD: &[u8; 16] = guid!("000C1080-0000-0000-C000-000000000046");
const CLSID_OUTLOOK_MSG: &[u8; 16] = guid!("00020D0B-0000-0000-C000-000000000046");
const CLSID_PUBLISHER: &[u8; 16] = guid!("00021201-0000-0000-C000-000000000046");
const CLSID_FLASHPIX: &[u8; 16] = guid!("56616700-C154-11CE-8553-00AA00A1F95B");
const CLSID_3DS_MAX: &[u8; 16] = guid!("1CDD8C7B-81C0-45A0-9FED-04143144CC1E");

const STREAM_WORD_DOCUMENT: &str = "WordDocument";
const STREAM_1TABLE: &str = "1Table";
const STREAM_0TABLE: &str = "0Table";
const STREAM_WORKBOOK: &str = "Workbook";
const STREAM_WORKBOOK_UPPER: &str = "WORKBOOK";
const STREAM_WORKBOOK_LOWER: &str = "workbook";
const STREAM_WORKBOOK_CAMEL: &str = "WorkBook";
const STREAM_BOOK: &str = "Book";
const STREAM_PPT_DOCUMENT: &str = "PowerPoint Document";
const STREAM_PP40: &str = "PP40";
const STREAM_CURRENT_USER: &str = "Current User";
const STREAM_VISIO_DOCUMENT: &str = "VisioDocument";
const STREAM_VISIO_INFORMATION: &str = "VisioInformation";
const STREAM_SKETCHUP_MODEL: &str = "SketchUpModel";
const STREAM_OUTLOOK_PROPS: &str = "__properties_version1.0";
const STREAM_OUTLOOK_SUBSTG_PREFIX: &str = "__substg1.0_";
const STREAM_OUTLOOK_NAMEID_PREFIX: &str = "__nameid_version1.0";
const STREAM_QUILL: &str = "Quill";
const STREAM_QUILL_SUB: &str = "QuillSub";
const STREAM_ESCHER_STM: &str = "EscherStm";
const STREAM_FLASHPIX: &str = "Flashpix";
const STREAM_GLOBAL_INFO: &str = "Global Info";
const STREAM_DATA_OBJECT_STORE: &str = "Data Object Store 000001";
const STREAM_HWP_HEADER: &str = "FileHeader";
const STREAM_HWP_DOCINFO: &str = "DocInfo";
const STREAM_PUB_OBJECTS: &str = "Objects";
const STREAM_PUB_CONTENTS: &str = "Contents";
const STREAM_CATALOG: &str = "Catalog";
const STREAM_THUMB_1: &str = "1";
const STREAM_THUMB_2: &str = "2";
const STREAM_THUMB_REV: &str = "rev";
const STREAM_SCENE: &str = "Scene";
const STREAM_CLASS_DATA: &str = "ClassData";
const STREAM_VIDEO_POST_QUEUE: &str = "VideoPostQueue";
const STREAM_VIDEO_POST: &str = "VideoPost";
const STREAM_FILE_ASSET_META2: &str = "FileAssetMetaData2";
const STREAM_DLL_DIRECTORY: &str = "DllDirectory";

/// Decodes an MSI base-64 compressed UTF-16LE stream name (characters in `0x3800..0x4840`).
fn is_msi_encoded_name(utf16_units: &[u16]) -> bool {
    !utf16_units.is_empty() && utf16_units.contains(&0x4840)
}

fn classify_clsid(clsid: &[u8]) -> Option<Label> {
    if clsid == CLSID_WORD_97 || clsid == CLSID_WORD_95 {
        Some(Label::Doc)
    } else if clsid == CLSID_EXCEL_97 || clsid == CLSID_EXCEL_95 || clsid == CLSID_EXCEL_CHART {
        Some(Label::Xls)
    } else if clsid == CLSID_PPT_97 || clsid == CLSID_PPT_95 || clsid == CLSID_PPT_SHOW {
        Some(Label::Ppt)
    } else if clsid == CLSID_VISIO_2003 || clsid == CLSID_VISIO_2000 {
        Some(Label::Visio)
    } else if clsid == CLSID_MSI_PACKAGE || clsid == CLSID_MSI_PATCH {
        Some(Label::Msi)
    } else if clsid == CLSID_MSI_TRANSFORM || clsid == CLSID_MSI_TRANSFORM_OLD {
        Some(Label::Mst)
    } else if clsid == CLSID_OUTLOOK_MSG {
        Some(Label::Outlook)
    } else if clsid == CLSID_PUBLISHER {
        Some(Label::Pub)
    } else if clsid == CLSID_FLASHPIX {
        Some(Label::Fpx)
    } else if clsid == CLSID_3DS_MAX {
        Some(Label::Max)
    } else {
        None
    }
}

/// Resolves an absolute file offset range `[abs_off .. abs_off + len]` into a slice within
/// either `prefix` or `tail`.
#[inline]
fn slice_at<'a>(
    prefix: &'a [u8],
    size: u64,
    tail: Option<&'a [u8]>,
    abs_off: usize,
    len: usize,
) -> Option<&'a [u8]> {
    let end = abs_off.checked_add(len)?;
    if end <= prefix.len() {
        return get_slice(prefix, abs_off, end);
    }
    if let Some(t) = tail {
        let tail_start = (size as usize).saturating_sub(t.len());
        if abs_off >= tail_start && end <= size as usize {
            let rel = abs_off - tail_start;
            return get_slice(t, rel, rel + len);
        }
    }
    None
}

/// Resolves an OLE2 Compound File Binary container into a Magika [`Label`] by walking its
/// directory entries (root CLSID and UTF-16LE stream/storage names).
pub(crate) fn verify_ole2(prefix: &[u8], size: u64, tail: Option<&[u8]>) -> Option<Label> {
    require!(
        size >= OLE2_HEADER_SIZE as u64,
        prefix.len() >= OLE2_HEADER_SIZE,
        starts(prefix, OLE2_MAGIC),
    );

    let sector_shift = le16(prefix, OLE2_OFF_SECTOR_SHIFT)? as usize;
    require!(sector_shift == 9 || sector_shift == 12);
    let sector_size = 1usize << sector_shift;

    let num_fat_sectors = le32(prefix, OLE2_OFF_NUM_FAT_SECTORS)? as usize;
    let first_dir_sector = le32(prefix, OLE2_OFF_FIRST_DIR_SECTOR)?;
    require!(first_dir_sector < MAXREGSECT);

    let mut fat_sector_ids = Vec::with_capacity(num_fat_sectors.min(OLE2_HEADER_DIFAT_COUNT));
    for i in 0..OLE2_HEADER_DIFAT_COUNT.min(num_fat_sectors.max(1)) {
        let off = OLE2_OFF_HEADER_DIFAT + i * 4;
        if let Some(sec_id) = le32(prefix, off) {
            if sec_id < MAXREGSECT {
                fat_sector_ids.push(sec_id);
            } else if sec_id == FREESECT || sec_id == ENDOFCHAIN {
                break;
            }
        }
    }

    let get_fat_next = |sec_id: u32| -> Option<u32> {
        let entries_per_fat = sector_size / 4;
        let fat_idx = (sec_id as usize) / entries_per_fat;
        let fat_sub = (sec_id as usize) % entries_per_fat;
        let fat_sec_id = *fat_sector_ids.get(fat_idx)?;
        let entry_abs = (fat_sec_id as usize).checked_add(1)?.checked_mul(sector_size)?.checked_add(fat_sub * 4)?;
        let bytes = slice_at(prefix, size, tail, entry_abs, 4)?;
        Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    };

    let mut dir_sectors = Vec::with_capacity(16);
    let mut cur_dir = first_dir_sector;
    for _ in 0..64 {
        if cur_dir >= MAXREGSECT {
            break;
        }
        dir_sectors.push(cur_dir);
        match get_fat_next(cur_dir) {
            Some(next) if next != cur_dir && next < MAXREGSECT => cur_dir = next,
            _ => break,
        }
    }

    let mut clsid_hit: Option<Label> = None;
    let mut has_catalog = false;
    let mut has_thumb_num = false;
    let mut has_scene = false;
    let mut has_3dsm_marker = false;
    let mut has_max_marker = false;
    let mut has_hwp_header = false;
    let mut has_hwp_docinfo = false;
    let mut has_pub_objects = false;
    let mut has_pub_contents = false;

    let entries_per_sector = sector_size / OLE2_DIR_ENTRY_SIZE;
    for dir_sec in dir_sectors {
        let Some(dir_abs) = (dir_sec as usize).checked_add(1).and_then(|s| s.checked_mul(sector_size)) else {
            continue;
        };
        for i in 0..entries_per_sector {
            let entry_abs = dir_abs + i * OLE2_DIR_ENTRY_SIZE;
            let Some(entry) = slice_at(prefix, size, tail, entry_abs, OLE2_DIR_ENTRY_SIZE) else {
                break;
            };
            let obj_type = entry[OLE2_DIR_OFF_OBJ_TYPE];
            if !matches!(obj_type, 1 | 2 | 5) {
                continue;
            }
            if obj_type == 5 || obj_type == 1 {
                let clsid = &entry[OLE2_DIR_OFF_CLSID..OLE2_DIR_OFF_CLSID + 16];
                if let Some(lbl) = classify_clsid(clsid) {
                    clsid_hit = Some(lbl);
                }
            }
            let name_len = u16::from_le_bytes([
                entry[OLE2_DIR_OFF_NAME_LEN],
                entry[OLE2_DIR_OFF_NAME_LEN + 1],
            ]) as usize;
            if !(2..=64).contains(&name_len) || name_len % 2 != 0 {
                continue;
            }
            let raw_name = &entry[..name_len - 2];
            let mut utf16 = [0u16; 32];
            let unit_count = raw_name.len() / 2;
            for (idx, chunk) in raw_name.chunks_exact(2).enumerate() {
                utf16[idx] = u16::from_le_bytes([chunk[0], chunk[1]]);
            }
            let units = &utf16[..unit_count];
            if is_msi_encoded_name(units) {
                return Some(clsid_hit.unwrap_or(Label::Msi));
            }
            let name = String::from_utf16_lossy(units);
            match name.as_str() {
                STREAM_WORD_DOCUMENT | STREAM_1TABLE | STREAM_0TABLE => return Some(Label::Doc),
                STREAM_WORKBOOK
                | STREAM_WORKBOOK_UPPER
                | STREAM_WORKBOOK_LOWER
                | STREAM_WORKBOOK_CAMEL
                | STREAM_BOOK => {
                    return Some(Label::Xls);
                }
                STREAM_PPT_DOCUMENT | STREAM_PP40 | STREAM_CURRENT_USER => return Some(Label::Ppt),
                STREAM_VISIO_DOCUMENT | STREAM_VISIO_INFORMATION => return Some(Label::Visio),
                STREAM_SKETCHUP_MODEL => return Some(Label::Sketchup),
                STREAM_OUTLOOK_PROPS => return Some(Label::Outlook),
                STREAM_QUILL | STREAM_QUILL_SUB | STREAM_ESCHER_STM => return Some(Label::Pub),
                STREAM_FLASHPIX | STREAM_GLOBAL_INFO | STREAM_DATA_OBJECT_STORE => {
                    return Some(Label::Fpx);
                }
                STREAM_HWP_HEADER => has_hwp_header = true,
                STREAM_HWP_DOCINFO => has_hwp_docinfo = true,
                STREAM_PUB_OBJECTS => has_pub_objects = true,
                STREAM_PUB_CONTENTS => has_pub_contents = true,
                STREAM_CATALOG => has_catalog = true,
                STREAM_THUMB_1 | STREAM_THUMB_2 | STREAM_THUMB_REV => has_thumb_num = true,
                STREAM_SCENE => has_scene = true,
                STREAM_CLASS_DATA => has_3dsm_marker = true,
                STREAM_VIDEO_POST_QUEUE
                | STREAM_VIDEO_POST
                | STREAM_FILE_ASSET_META2
                | STREAM_DLL_DIRECTORY => {
                    has_max_marker = true;
                }
                s if s.starts_with(STREAM_OUTLOOK_SUBSTG_PREFIX)
                    || s.starts_with(STREAM_OUTLOOK_NAMEID_PREFIX) =>
                {
                    return Some(Label::Outlook);
                }
                _ => {}
            }
        }
    }

    if has_hwp_header && has_hwp_docinfo {
        return Some(Label::Hwp);
    }
    if has_pub_objects && has_pub_contents {
        return Some(Label::Pub);
    }
    if has_scene && has_max_marker {
        return Some(Label::Max);
    }
    if has_scene && has_3dsm_marker {
        return Some(Label::_3dsm);
    }
    if has_catalog && has_thumb_num {
        return Some(Label::Thumbsdb);
    }
    clsid_hit
}
