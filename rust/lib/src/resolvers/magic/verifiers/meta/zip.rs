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

//! PKZIP local-header and central-directory container disambiguator.

use miniz_oxide::inflate::TINFLStatus;
use miniz_oxide::inflate::core::inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
use miniz_oxide::inflate::core::{DecompressorOxide, decompress};

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// Size of the `zip_names` view.
pub(crate) const ZIP_NAMES_BYTES: usize = 4096;

/// Size of the `zip_first_entry` view.
pub(crate) const ZIP_FIRST_ENTRY_BYTES: usize = 16 * 1024;

/// Size of the trailing window that holds the central directory of most archives.
#[allow(dead_code)]
pub(crate) const ZIP_TAIL_BYTES: usize = 16 * 1024;

/// Largest central directory read on its own when the tail window does not hold it.
#[allow(dead_code)]
pub(crate) const ZIP_DIRECTORY_MAX_BYTES: u32 = 256 * 1024;

pub(crate) const ZIP_FLAG_ZIP64: u8 = 1 << 0;

pub(crate) const ZIP_FLAG_MULTIDISK: u8 = 1 << 1;

pub(crate) const ZIP_FLAG_CD_NOT_HELD: u8 = 1 << 2;

pub(crate) const ZIP_FLAG_MALFORMED: u8 = 1 << 3;

pub(crate) const ZIP_FLAG_NAMES_TRUNCATED: u8 = 1 << 4;

pub(crate) const ZIP_LOCAL_SIGNATURE: &[u8; 4] = b"PK\x03\x04";

pub(crate) const ZIP_CENTRAL_SIGNATURE: &[u8; 4] = b"PK\x01\x02";

pub(crate) const ZIP_EOCD_SIGNATURE: &[u8; 4] = b"PK\x05\x06";

pub(crate) const ZIP_ZIP64_LOCATOR_SIGNATURE: &[u8; 4] = b"PK\x06\x07";

pub(crate) const ZIP_ZIP64_EOCD_SIGNATURE: &[u8; 4] = b"PK\x06\x06";

const LINE_CONTENT_TYPES: &[u8] = b"\n[Content_Types].xml\n";

pub(crate) const ZIP_LOCAL_HEADER_LEN: usize = 30;

pub(crate) const ZIP_CENTRAL_HEADER_LEN: usize = 46;

pub(crate) const ZIP_EOCD_LEN: usize = 22;

pub(crate) const ZIP_ZIP64_LOCATOR_LEN: usize = 20;

pub(crate) const ZIP_ZIP64_EOCD_LEN: usize = 56;

pub(crate) const ZIP_MIMETYPE_MAX_LEN: u32 = 128;

/// Whether the prefix opens a zip archive (a local file header, or the end record of an empty
/// one).
pub(crate) fn wants_tail(prefix: &[u8]) -> bool {
    starts(prefix, ZIP_LOCAL_SIGNATURE) || starts(prefix, ZIP_EOCD_SIGNATURE)
}

/// Where the central directory starts, when `tail` (the last bytes of an input of `size` bytes)
/// holds an end record whose directory starts before it and spans at most
/// `ZIP_DIRECTORY_MAX_BYTES`.
#[allow(dead_code)]
pub(crate) fn directory_start(tail: &[u8], size: u64) -> Option<u64> {
    let eocd = find_zip_eocd(tail)?;
    let record = ZipEocd::decode(tail, eocd);
    let tail_start = size.checked_sub(tail.len() as u64)?;
    let eocd_abs = tail_start + eocd as u64;
    let (cd_size, cd_end_abs) = match zip64_directory(tail, eocd, tail_start) {
        Some((_, cd_size, _, cd_end_abs)) => (cd_size, cd_end_abs),
        None if record.flags != 0 => return None,
        None => (u64::from(record.cd_size), eocd_abs),
    };
    if cd_size > u64::from(ZIP_DIRECTORY_MAX_BYTES) {
        return None;
    }
    let start = cd_end_abs.checked_sub(cd_size)?;
    (start < tail_start).then_some(start)
}

pub(crate) struct ZipViewWriter<'a> {
    out: &'a mut [u8; ZIP_NAMES_BYTES],
    len: usize,
}

impl ZipViewWriter<'_> {
    fn fits(&self, prefix: &[u8], text: &[u8]) -> bool {
        let line = 1 + prefix.len() + text.len();
        self.len + line < ZIP_NAMES_BYTES
    }

    fn line(&mut self, prefix: &[u8], text: &[u8]) {
        debug_assert!(self.fits(prefix, text));
        self.out[self.len] = b'\n';
        self.len += 1;
        self.out[self.len..self.len + prefix.len()].copy_from_slice(prefix);
        self.len += prefix.len();
        for &byte in text {
            self.out[self.len] = if byte == 0 || byte == b'\n' { 1 } else { byte };
            self.len += 1;
        }
    }

    fn finish(self) -> usize {
        if self.len == 0 {
            return 0;
        }
        self.out[self.len] = b'\n';
        self.len + 1
    }
}

/// Extracts the uncompressed payload of an unencrypted stored (`method == 0`) first entry named
/// `"mimetype"` (up to `ZIP_MIMETYPE_MAX_LEN` bytes), used by EPUB and OASIS OpenDocument archives.
pub(crate) fn stored_mimetype(first: &[u8]) -> Option<&[u8]> {
    const MIMETYPE_NAME: &[u8] = b"mimetype";
    if !starts(first, ZIP_LOCAL_SIGNATURE) || le16(first, 8)? != 0 || le16(first, 26)? != 8 {
        return None;
    }
    let length = le32(first, 22)?;
    if !(1..=ZIP_MIMETYPE_MAX_LEN).contains(&length) {
        return None;
    }
    let extra_len = usize::from(le16(first, 28)?);
    let name_end = ZIP_LOCAL_HEADER_LEN + 8;
    if !eq_at(first, ZIP_LOCAL_HEADER_LEN, MIMETYPE_NAME) {
        return None;
    }
    let data_start = name_end.checked_add(extra_len)?;
    get_slice(first, data_start, data_start.checked_add(length as usize)?)
}

/// Scans backwards through `tail` for a PKZIP End of Central Directory (`PK\x05\x06`) record whose
/// 16-bit trailing comment length exactly reaches the end of the slice.
pub(crate) fn find_zip_eocd(tail: &[u8]) -> Option<usize> {
    let last = tail.len().checked_sub(ZIP_EOCD_LEN)?;
    (0..=last).rev().find(|&at| {
        tail[at..].starts_with(ZIP_EOCD_SIGNATURE)
            && le16(tail, at + 20).is_some_and(|comment_len| {
                usize::from(comment_len) == tail.len() - at - ZIP_EOCD_LEN
            })
            && eq_at(tail, at, ZIP_EOCD_SIGNATURE)
    })
}

pub(crate) struct ZipEocd {
    entries: u16,
    cd_size: u32,
    cd_offset: u32,
    flags: u8,
}

impl ZipEocd {
    fn decode(tail: &[u8], eocd: usize) -> Self {
        let record = &tail[eocd..eocd + ZIP_EOCD_LEN];
        let field = |at| le16(record, at).unwrap_or(0);
        let (disk, cd_disk, entries_disk, entries) = (field(4), field(6), field(8), field(10));
        let cd_size = le32(record, 12).unwrap_or(0);
        let cd_offset = le32(record, 16).unwrap_or(0);
        let _comment_len = field(20);
        let locator = eocd
            .checked_sub(ZIP_ZIP64_LOCATOR_LEN)
            .is_some_and(|at| tail[at..].starts_with(ZIP_ZIP64_LOCATOR_SIGNATURE));
        let mut flags = 0;
        if [disk, cd_disk, entries_disk, entries].contains(&u16::MAX)
            || cd_size == u32::MAX
            || cd_offset == u32::MAX
            || locator
        {
            flags |= ZIP_FLAG_ZIP64;
        }
        if disk != 0 || cd_disk != 0 || entries_disk != entries {
            flags |= ZIP_FLAG_MULTIDISK;
        }
        Self { entries, cd_size, cd_offset, flags }
    }
}

/// Resolves a ZIP64 End of Central Directory Locator (`PK\x06\x07`) and Record (`PK\x06\x06`)
/// immediately preceding `eocd`, returning `(entries, cd_size, cd_offset, record_abs)`.
pub(crate) fn zip64_directory(tail: &[u8], eocd: usize, tail_start: u64) -> Option<(u64, u64, u64, u64)> {
    let locator = eocd.checked_sub(ZIP_ZIP64_LOCATOR_LEN)?;
    if !eq_at(tail, locator, ZIP_ZIP64_LOCATOR_SIGNATURE) {
        return None;
    }
    let record_abs = le64(tail, locator + 8)?;
    let at = usize::try_from(record_abs.checked_sub(tail_start)?).ok()?;
    let record = get_slice(tail, at, at.checked_add(ZIP_ZIP64_EOCD_LEN)?)?;
    if !starts(record, ZIP_ZIP64_EOCD_SIGNATURE) {
        return None;
    }
    Some((le64(record, 32)?, le64(record, 40)?, le64(record, 48)?, record_abs))
}

/// Parses a single Central Directory File Header (`PK\x01\x02`) at `at`, returning the entry
/// filename slice and total record byte length (`46 + name_len + extra_len + comment_len`).
pub(crate) fn zip_entry_name(directory: &[u8], at: usize) -> Option<(&[u8], usize)> {
    let header = get_slice(directory, at, at.checked_add(ZIP_CENTRAL_HEADER_LEN)?)?;
    if !starts(header, ZIP_CENTRAL_SIGNATURE) {
        return None;
    }
    let name_len = usize::from(le16(header, 28)?);
    let trailer = usize::from(le16(header, 30)?) + usize::from(le16(header, 32)?);
    let name_start = at + ZIP_CENTRAL_HEADER_LEN;
    let name_end = name_start.checked_add(name_len)?;
    let entry_end = name_end.checked_add(trailer)?;
    if entry_end > directory.len() {
        return None;
    }
    Some((get_slice(directory, name_start, name_end)?, entry_end - at))
}

/// Walks the PKZIP Central Directory in two passes (top-level entries first, nested paths second),
/// populating `view` with newline-delimited entry names and returning `(valid_cd, has_dir_names)`.
pub(crate) fn walk_zip_directory(
    first: &[u8],
    size: u64,
    tail: &[u8],
    eocd: usize,
    view: &mut ZipViewWriter<'_>,
) -> (bool, bool) {
    let Some(tail_start) = size.checked_sub(tail.len() as u64) else {
        return (false, false);
    };
    let record = ZipEocd::decode(tail, eocd);
    if record.flags & ZIP_FLAG_MULTIDISK != 0 {
        return (false, false);
    }
    let eocd_abs = tail_start + eocd as u64;
    let (entries, cd_size, _cd_offset, cd_end_abs) = match zip64_directory(tail, eocd, tail_start) {
        Some(resolved) => resolved,
        None if record.flags & ZIP_FLAG_ZIP64 != 0 => return (false, false),
        None => (
            u64::from(record.entries),
            u64::from(record.cd_size),
            u64::from(record.cd_offset),
            eocd_abs,
        ),
    };
    let entries_u16 = entries.min(u64::from(u16::MAX)) as u16;
    let Some(cd_abs) = cd_end_abs.checked_sub(cd_size) else {
        return (false, false);
    };
    let directory = if cd_abs >= tail_start {
        let (start, end) = ((cd_abs - tail_start) as usize, (cd_end_abs - tail_start) as usize);
        let Some(slice) = tail.get(start..end) else {
            return (false, false);
        };
        slice
    } else if cd_end_abs <= first.len() as u64 {
        &first[cd_abs as usize..cd_end_abs as usize]
    } else {
        let _ = ZIP_FLAG_CD_NOT_HELD;
        return (false, false);
    };
    let mut flags = 0u8;
    let mut names_entries = 0u16;
    'passes: for nested in [false, true] {
        let mut at = 0;
        for _ in 0..entries {
            let Some((name, len)) = zip_entry_name(directory, at) else {
                flags |= ZIP_FLAG_MALFORMED;
                break 'passes;
            };
            at += len;
            if name.contains(&b'/') != nested {
                continue;
            }
            if !view.fits(b"", name) {
                flags |= ZIP_FLAG_NAMES_TRUNCATED;
                break 'passes;
            }
            view.line(b"", name);
            names_entries += 1;
        }
    }
    let valid = flags & ZIP_FLAG_MALFORMED == 0;
    let dir_names = valid && entries_u16 >= 1 && names_entries >= 1;
    (valid, dir_names)
}

/// Extracts or inflates (via `miniz_oxide`) the first local file entry (`PK\x03\x04`) into `out`
/// as `<entry_name>\n<uncompressed_bytes>`, returning the number of bytes written.
pub(crate) fn zip_first_entry(
    first: &[u8],
    out: &mut [u8; ZIP_FIRST_ENTRY_BYTES],
    inflater: &mut DecompressorOxide,
) -> usize {
    if !starts(first, ZIP_LOCAL_SIGNATURE) {
        return 0;
    }
    let (Some(flags), Some(method)) = (le16(first, 6), le16(first, 8)) else {
        return 0;
    };
    if flags & 0b1001 != 0 || !(method == 0 || method == 8) {
        return 0;
    }
    let Some(compressed) = le32(first, 18).and_then(|n| usize::try_from(n).ok()) else {
        return 0;
    };
    let (Some(name_len), Some(extra_len)) = (le16(first, 26), le16(first, 28)) else {
        return 0;
    };
    let (name_len, extra_len) = (usize::from(name_len), usize::from(extra_len));
    if name_len + 1 >= ZIP_FIRST_ENTRY_BYTES {
        return 0;
    }
    let Some(name) = get_slice(first, ZIP_LOCAL_HEADER_LEN, ZIP_LOCAL_HEADER_LEN + name_len) else {
        return 0;
    };
    let data_start = ZIP_LOCAL_HEADER_LEN + name_len + extra_len;
    let Some(data_end) = data_start.checked_add(compressed) else {
        return 0;
    };
    let Some(data) = get_slice(first, data_start, data_end) else {
        return 0;
    };
    for (slot, &byte) in out.iter_mut().zip(name) {
        *slot = if byte == 0 || byte == b'\n' { 1 } else { byte };
    }
    out[name.len()] = b'\n';
    let head = name.len() + 1;
    let space = &mut out[head..];
    let (written, complete) = if method == 8 {
        inflater.init();
        let inflate_flags = TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
        let (status, _, written) = decompress(inflater, data, space, 0, inflate_flags);
        (written, status == TINFLStatus::Done)
    } else {
        let written = data.len().min(space.len());
        space[..written].copy_from_slice(&data[..written]);
        (written, true)
    };
    if written == space.len() {
        return ZIP_FIRST_ENTRY_BYTES;
    }
    if !complete {
        space[..written].fill(0);
        return head;
    }
    head + written
}

/// Android APK local-header detection in the 4 KiB prefix (`taxonomy_apk`): unencrypted stored or
/// deflated first entry with nonzero sizes or data-descriptor flag, opening either with
/// `classes.dex` plus a local `AndroidManifest.xml` entry in `41..=4047` or with
/// `AndroidManifest.xml` plus a local `classes.dex` (`49..=4055`) or `resources.arsc`
/// (`49..=4052`) entry.
pub(crate) fn apk_prefix(prefix: &[u8]) -> Option<Label> {
    const FLAGS: usize = 6; // @6: u16 LE — general purpose bit flag (bit 0 = encrypted, bit 3 = data descriptor)
    const METHOD: usize = 8; // @8: u16 LE — compression method (0 = stored, 8 = deflated)
    const COMP_SIZE: usize = 18; // @18: u32 LE — compressed size
    const UNCOMP_SIZE: usize = 22; // @22: u32 LE — uncompressed size
    const NAME_LEN: usize = 26; // @26: u16 LE — file name length
    const NAME_OFFSET: usize = 30; // @30: file name bytes
    const CLASSES_DEX: &[u8] = b"classes.dex";
    const ANDROID_MANIFEST: &[u8] = b"AndroidManifest.xml";
    const RESOURCES_ARSC: &[u8] = b"resources.arsc";
    const MIN_LEN: usize = 41;

    /// Checks for `{ 50 4B 03 04 [22] <name_len u16 LE> ?? ?? <name> }` starting in `lo..=hi`.
    fn has_local_entry(prefix: &[u8], lo: usize, hi: usize, name: &[u8]) -> bool {
        let name_len = name.len() as u16;
        for name_pos in memchr::memmem::find_iter(prefix, name) {
            let Some(hdr_pos) = name_pos.checked_sub(NAME_OFFSET) else {
                continue;
            };
            if hdr_pos > hi {
                break;
            }
            if hdr_pos >= lo
                && eq_at(prefix, hdr_pos, ZIP_LOCAL_SIGNATURE)
                && le16(prefix, hdr_pos + NAME_LEN) == Some(name_len)
            {
                return true;
            }
        }
        false
    }

    require!(prefix.len() >= MIN_LEN, starts(prefix, ZIP_LOCAL_SIGNATURE));
    let flags = le16(prefix, FLAGS)?;
    let method = le16(prefix, METHOD)?;
    require!(
        flags % 2 == 0,
        matches!(method, 0 | 8),
        (le32(prefix, COMP_SIZE)? >= 1 && le32(prefix, UNCOMP_SIZE)? >= 1) || (flags & 0x08 != 0),
    );
    let name_len = le16(prefix, NAME_LEN)?;
    let valid = (name_len == 11
        && eq_at(prefix, NAME_OFFSET, CLASSES_DEX)
        && has_local_entry(prefix, 41, 4047, ANDROID_MANIFEST))
        || (prefix.len() >= 49
            && name_len == 19
            && eq_at(prefix, NAME_OFFSET, ANDROID_MANIFEST)
            && (has_local_entry(prefix, 49, 4055, CLASSES_DEX)
                || has_local_entry(prefix, 49, 4052, RESOURCES_ARSC)));
    require!(valid);
    Some(Label::Apk)
}

/// EPUB OCF local-header detection in the 4 KiB prefix (`taxonomy_epub`): unencrypted stored
/// first entry `"mimetype"` (`name_len == 8`, `extra_len == 0`) with 20-byte payload
/// `"application/epub+zip"` (either declared in the local header or deferred to a data descriptor
/// at `@58`).
pub(crate) fn epub_prefix(prefix: &[u8]) -> Option<Label> {
    const FLAGS: usize = 6; // @6: u16 LE — general purpose bit flag
    const METHOD: usize = 8; // @8: u16 LE — compression method (0 = stored)
    const COMP_SIZE: usize = 18; // @18: u32 LE — compressed size (20, or 0 with data descriptor)
    const UNCOMP_SIZE: usize = 22; // @22: u32 LE — uncompressed size (20, or 0 with data descriptor)
    const NAME_LEN: usize = 26; // @26: u16 LE — file name length (8 = "mimetype")
    const EXTRA_LEN: usize = 28; // @28: u16 LE — extra field length (0)
    const PAYLOAD_OFFSET: usize = 30; // @30..58: "mimetypeapplication/epub+zip"
    const PAYLOAD: &[u8] = b"mimetypeapplication/epub+zip";
    const DESCRIPTOR_OFFSET: usize = 58; // @58..74: PK\x07\x08 + CRC32("application/epub+zip") + 20 + 20
    const DESCRIPTOR: &[u8] = bytes!(
        b"PK\x07\x08",
        &0x2CAB_616Fu32.to_le_bytes(),
        &20u32.to_le_bytes(),
        &20u32.to_le_bytes(),
    );
    const MIN_LEN: usize = 58;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, ZIP_LOCAL_SIGNATURE),
        eq_at(prefix, PAYLOAD_OFFSET, PAYLOAD),
        le16(prefix, METHOD)? == 0,
        le16(prefix, NAME_LEN)? == 8,
        le16(prefix, EXTRA_LEN)? == 0,
    );
    let flags = le16(prefix, FLAGS)?;
    require!(flags % 2 == 0);
    let comp = le32(prefix, COMP_SIZE)?;
    let uncomp = le32(prefix, UNCOMP_SIZE)?;
    require!(
        (comp == 20 && uncomp == 20)
            || (prefix.len() >= 74
                && flags % 16 == 8
                && comp == 0
                && uncomp == 0
                && eq_at(prefix, DESCRIPTOR_OFFSET, DESCRIPTOR))
    );
    Some(Label::Epub)
}

/// ZIP container subtypes (`PK\x03\x04` or `PK\x05\x06`): checks the prefix-only APK/EPUB local
/// headers first (so tail-less archives still resolve), then inspects the central-directory
/// `names` and `first_entry` views.
pub(crate) fn zip(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    if let Some(label) = apk_prefix(prefix) {
        return Some(label);
    }
    if let Some(label) = epub_prefix(prefix) {
        return Some(label);
    }

    const MIMETYPE_PREFIX: &[u8] = b"mimetype=";
    const LINE_MODEL_WEIGHTS: &[u8] = b"\nmodel.weights.h5\n";
    const LINE_METADATA_JSON: &[u8] = b"\nmetadata.json\n";
    const SUFFIX_QGS: &[u8] = b".qgs\n";
    const LINE_VISIO_DOC: &[u8] = b"\nvisio/document.xml\n";
    const LINE_XL_WORKBOOK: &[u8] = b"\nxl/workbook.xml\n";
    const LINE_3MF_MODEL: &[u8] = b"\n3D/3dmodel.model\n";
    const LINE_ANDROID_MANIFEST: &[u8] = b"\nAndroidManifest.xml\n";
    const LINE_CLASSES_DEX: &[u8] = b"\nclasses.dex\n";
    const LINE_RESOURCES_ARSC: &[u8] = b"\nresources.arsc\n";
    const MIME_EPUB: &[u8] = b"\nmimetype=application/epub+zip\n";
    const LINE_CONTAINER_XML: &[u8] = b"\nMETA-INF/container.xml\n";
    const MIME_ODP: &[u8] = b"\nmimetype=application/vnd.oasis.opendocument.presentation\n";
    const MIME_ODS: &[u8] = b"\nmimetype=application/vnd.oasis.opendocument.spreadsheet\n";
    const MIME_ODT: &[u8] = b"\nmimetype=application/vnd.oasis.opendocument.text\n";
    const LINE_CONTENT_XML: &[u8] = b"\ncontent.xml\n";
    const LINE_MANIFEST_MF: &[u8] = b"\nMETA-INF/MANIFEST.MF\n";
    const SUFFIX_CLASS: &[u8] = b".class\n";
    const LINE_DOC_KML: &[u8] = b"\ndoc.kml\n";
    const LINE_APPX_MANIFEST: &[u8] = b"\nAppxManifest.xml\n";
    const LINE_APPX_BLOCKMAP: &[u8] = b"\nAppxBlockMap.xml\n";
    const LINE_PPT_PRESENTATION: &[u8] = b"\nppt/presentation.xml\n";
    const FIRST_CONTENT_TYPES: &[u8] = b"[Content_Types].xml\n";
    const CT_DOCX_MAIN: &[u8] =
        b"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";
    const CT_DOCM_MAIN: &[u8] = b"application/vnd.ms-word.document.macroEnabled.main+xml";
    const CT_DOTX_MAIN: &[u8] =
        b"application/vnd.openxmlformats-officedocument.wordprocessingml.template.main+xml";
    const CT_DOTM_MAIN: &[u8] =
        b"application/vnd.ms-word.template.macroEnabledTemplate.main+xml";
    const CT_XLSB_MAIN: &[u8] = b"application/vnd.ms-excel.sheet.binary.macroEnabled.main";

    let mut names_buf = Box::new([0u8; ZIP_NAMES_BYTES]);
    let mut first_entry_buf = Box::new([0u8; ZIP_FIRST_ENTRY_BYTES]);
    let (names_len, first_entry_len, valid, dir_names) = if wants_tail(prefix) {
        let whole = ctx.size == prefix.len() as u64;
        let tail = ctx.tail.unwrap_or(if whole { prefix } else { &[] });
        let mut view = ZipViewWriter { out: &mut names_buf, len: 0 };
        if let Some(mimetype) = stored_mimetype(prefix) {
            view.line(MIMETYPE_PREFIX, mimetype);
        }
        let (valid, dir_names) = match find_zip_eocd(tail) {
            Some(eocd) => walk_zip_directory(prefix, ctx.size, tail, eocd, &mut view),
            None => (false, false),
        };
        let names_len = view.finish();
        let mut inflater = Box::<DecompressorOxide>::default();
        let first_len = zip_first_entry(prefix, &mut first_entry_buf, &mut inflater);
        (names_len, first_len, valid, dir_names)
    } else {
        (0, 0, false, false)
    };
    let names = &names_buf[..names_len];
    let first_entry = &first_entry_buf[..first_entry_len];
    let has_line = |line: &[u8]| memchr::memmem::find(names, line).is_some();

    // Full rules
    if valid {
        if has_line(LINE_MODEL_WEIGHTS) && has_line(LINE_METADATA_JSON) {
            return Some(Label::Keras);
        }
        if memchr::memmem::find(names, SUFFIX_QGS).is_some() {
            return Some(Label::Qgis);
        }
        if has_line(LINE_VISIO_DOC) {
            return Some(Label::Visio);
        }
        if dir_names && has_line(LINE_CONTENT_TYPES) && has_line(LINE_XL_WORKBOOK) {
            return Some(Label::Xlsx);
        }
        if has_line(LINE_3MF_MODEL) {
            return Some(Label::_3mf);
        }
    }

    // Partial central-directory rules
    if dir_names
        && has_line(LINE_ANDROID_MANIFEST)
        && (has_line(LINE_CLASSES_DEX) || has_line(LINE_RESOURCES_ARSC))
    {
        return Some(Label::Apk);
    }
    if starts(names, MIME_EPUB) && has_line(LINE_CONTAINER_XML) {
        return Some(Label::Epub);
    }
    if starts(names, MIME_ODP) && has_line(LINE_CONTENT_XML) {
        return Some(Label::Odp);
    }
    if starts(names, MIME_ODS) && has_line(LINE_CONTENT_XML) {
        return Some(Label::Ods);
    }
    if starts(names, MIME_ODT) && has_line(LINE_CONTENT_XML) {
        return Some(Label::Odt);
    }
    if dir_names
        && has_line(LINE_MANIFEST_MF)
        && memchr::memmem::find(names, SUFFIX_CLASS).is_some()
    {
        return Some(Label::Jar);
    }
    if valid && has_line(LINE_DOC_KML) {
        return Some(Label::Kmz);
    }
    if valid && has_line(LINE_APPX_MANIFEST) && has_line(LINE_APPX_BLOCKMAP) {
        return Some(Label::Msix);
    }
    if dir_names && has_line(LINE_CONTENT_TYPES) && has_line(LINE_PPT_PRESENTATION) {
        return Some(Label::Pptx);
    }

    // First-entry rules
    if starts(first_entry, FIRST_CONTENT_TYPES) {
        if memchr::memmem::find(first_entry, CT_DOCX_MAIN).is_some()
            || memchr::memmem::find(first_entry, CT_DOCM_MAIN).is_some()
        {
            return Some(Label::Docx);
        }
        if memchr::memmem::find(first_entry, CT_DOTX_MAIN).is_some()
            || memchr::memmem::find(first_entry, CT_DOTM_MAIN).is_some()
        {
            return Some(Label::Dotx);
        }
        if memchr::memmem::find(first_entry, CT_XLSB_MAIN).is_some() {
            return Some(Label::Xlsb);
        }
    }
    let first_is_ct = starts(first_entry, FIRST_CONTENT_TYPES);
    zip_subtypes_and_fallback(prefix, ctx.size, names, valid, dir_names, first_is_ct)
}

/// Classifies additional ZIP-backed formats in Magika's taxonomy (`npz`, `nupkg`,
/// `pytorch`, `xpi`, plus Central-Directory / Local-File-Header OOXML fallbacks) and
/// returns `Label::Zip` for valid generic ZIP archives.
pub(crate) fn zip_subtypes_and_fallback(
    prefix: &[u8],
    size: u64,
    names: &[u8],
    valid: bool,
    dir_names: bool,
    first_is_ct: bool,
) -> Option<Label> {
    const ZIP_EMPTY_ARCHIVE: &[u8; 22] = bytes!(ZIP_EOCD_SIGNATURE, &[0u8; 18]);

    const CD_WORD_DOC: &[u8] = b"\nword/document.xml\n";
    const CD_XL_WORKBOOK_BIN: &[u8] = b"\nxl/workbook.bin\n";
    const CD_NUSPEC_EXT: &[u8] = b".nuspec\n";
    const CD_NPY_EXT: &[u8] = b".npy\n";
    const CD_PAYLOAD_DIR: &[u8] = b"\nPayload/";
    const CD_IOS_APP_DIR: &[u8] = b".app/";
    const CD_MAFF_INDEX_RDF: &[u8] = b"/index.rdf\n";
    const CD_PT2_ARCHIVE_FORMAT: &[u8] = b"\narchive_format\n";
    const CD_PT2_MODELS_DIR: &[u8] = b"\nmodels/";
    const CD_SH3D_HOME: &[u8] = b"\nHome\n";
    const CD_SH3D_HOME_XML: &[u8] = b"\nHome.xml\n";
    const CD_SKOPS_SCHEMA_ROOT: &[u8] = b"\nschema.json\n";
    const CD_SKOPS_SCHEMA_SUB: &[u8] = b"/schema.json\n";
    const CD_TMDX_DOC_TMD: &[u8] = b"\ndocument.tmd\n";
    const CD_TMDX_DOC_XML: &[u8] = b"\ntmd/document.xml\n";
    const CD_TORCHSCRIPT_CONST_SUB: &[u8] = b"/constants.pkl\n";
    const CD_TORCHSCRIPT_CONST_ROOT: &[u8] = b"\nconstants.pkl\n";
    const CD_TORCHSCRIPT_CODE_DIR: &[u8] = b"/code/";
    const CD_PYTORCH_DATA_SUB: &[u8] = b"/data.pkl\n";
    const CD_PYTORCH_DATA_ROOT: &[u8] = b"\ndata.pkl\n";
    const CD_XPI_INSTALL_RDF: &[u8] = b"\ninstall.rdf\n";
    const CD_XPI_CHROME_MANIFEST: &[u8] = b"\nchrome.manifest\n";

    const LFH_NPY_EXT: &[u8] = b".npy";
    const LFH_NUSPEC_EXT: &[u8] = b".nuspec";
    const LFH_PAYLOAD_DIR: &[u8] = b"Payload/";
    const LFH_MAFF_INDEX_RDF: &[u8] = b"/index.rdf";
    const LFH_PT2_ARCHIVE_FORMAT: &[u8] = b"archive_format";
    const LFH_PT2_MODELS_DIR: &[u8] = b"models/";
    const LFH_SH3D_HOME: &[u8] = b"Home";
    const LFH_SH3D_HOME_XML: &[u8] = b"Home.xml";
    const LFH_SKOPS_SCHEMA_ROOT: &[u8] = b"schema.json";
    const LFH_SKOPS_SCHEMA_SUB: &[u8] = b"/schema.json";
    const LFH_TMDX_DOC_TMD: &[u8] = b"document.tmd";
    const LFH_TMDX_DOC_XML: &[u8] = b"tmd/document.xml";
    const LFH_TORCHSCRIPT_CONST_SUB: &[u8] = b"/constants.pkl";
    const LFH_TORCHSCRIPT_CONST_ROOT: &[u8] = b"constants.pkl";
    const LFH_PYTORCH_DATA_SUB: &[u8] = b"/data.pkl";
    const LFH_PYTORCH_DATA_ROOT: &[u8] = b"data.pkl";
    const LFH_PYTORCH_VERSION_SUB: &[u8] = b"/version";
    const LFH_PYTORCH_BYTEORDER: &[u8] = b"byteorder";
    const LFH_XPI_INSTALL_RDF: &[u8] = b"install.rdf";
    const LFH_XPI_CHROME_MANIFEST: &[u8] = b"chrome.manifest";
    const LFH_CONTENT_TYPES: &[u8] = b"[Content_Types].xml";

    // Check Central Directory `names` view
    if !names.is_empty() {
        if dir_names
            && !first_is_ct
            && memchr::memmem::find(names, LINE_CONTENT_TYPES).is_some()
        {
            if memchr::memmem::find(names, CD_WORD_DOC).is_some() {
                return Some(Label::Docx);
            }
            if memchr::memmem::find(names, CD_XL_WORKBOOK_BIN).is_some() {
                return Some(Label::Xlsb);
            }
        }
        if memchr::memmem::find(names, CD_NUSPEC_EXT).is_some() {
            return Some(Label::Nupkg);
        }
        if memchr::memmem::find(names, CD_NPY_EXT).is_some() {
            return Some(Label::Npz);
        }
        if memchr::memmem::find(names, CD_PAYLOAD_DIR).is_some()
            && memchr::memmem::find(names, CD_IOS_APP_DIR).is_some()
        {
            return Some(Label::Iosapp);
        }
        if memchr::memmem::find(names, CD_MAFF_INDEX_RDF).is_some() {
            return Some(Label::Maff);
        }
        if memchr::memmem::find(names, CD_PT2_ARCHIVE_FORMAT).is_some()
            && memchr::memmem::find(names, CD_PT2_MODELS_DIR).is_some()
        {
            return Some(Label::PytorchExport);
        }
        if memchr::memmem::find(names, CD_SH3D_HOME).is_some()
            || memchr::memmem::find(names, CD_SH3D_HOME_XML).is_some()
        {
            return Some(Label::Sh3d);
        }
        if memchr::memmem::find(names, CD_SKOPS_SCHEMA_ROOT).is_some()
            || memchr::memmem::find(names, CD_SKOPS_SCHEMA_SUB).is_some()
        {
            return Some(Label::Skops);
        }
        if memchr::memmem::find(names, CD_TMDX_DOC_TMD).is_some()
            || memchr::memmem::find(names, CD_TMDX_DOC_XML).is_some()
        {
            return Some(Label::Tmdx);
        }
        if (memchr::memmem::find(names, CD_TORCHSCRIPT_CONST_SUB).is_some()
            || memchr::memmem::find(names, CD_TORCHSCRIPT_CONST_ROOT).is_some())
            && memchr::memmem::find(names, CD_TORCHSCRIPT_CODE_DIR).is_some()
        {
            return Some(Label::Torchscript);
        }
        if memchr::memmem::find(names, CD_PYTORCH_DATA_SUB).is_some()
            || memchr::memmem::find(names, CD_PYTORCH_DATA_ROOT).is_some()
        {
            return Some(Label::Pytorch);
        }
        if memchr::memmem::find(names, CD_XPI_INSTALL_RDF).is_some()
            || memchr::memmem::find(names, CD_XPI_CHROME_MANIFEST).is_some()
        {
            return Some(Label::Xpi);
        }
    }

    // Check Local File Header filename in `prefix` when Central Directory is truncated or single-entry
    if size >= 30 && prefix.len() >= 34 && starts(prefix, ZIP_LOCAL_SIGNATURE) {
        let ver = le16(prefix, 4).unwrap_or(0);
        let name_len = le16(prefix, 26).unwrap_or(0) as usize;
        if (10..=63).contains(&(ver & 0xFF)) && (1..=512).contains(&name_len) {
            if let Some(first_name) = get_slice(prefix, 30, 30 + name_len.min(prefix.len() - 30)) {
                if first_name.ends_with(LFH_NPY_EXT) {
                    return Some(Label::Npz);
                }
                if first_name.ends_with(LFH_NUSPEC_EXT) {
                    return Some(Label::Nupkg);
                }
                if first_name.starts_with(LFH_PAYLOAD_DIR) && contains(prefix, CD_IOS_APP_DIR) {
                    return Some(Label::Iosapp);
                }
                if first_name.ends_with(LFH_MAFF_INDEX_RDF) || contains(prefix, LFH_MAFF_INDEX_RDF) {
                    return Some(Label::Maff);
                }
                if first_name == LFH_PT2_ARCHIVE_FORMAT
                    || (contains(prefix, LFH_PT2_ARCHIVE_FORMAT) && contains(prefix, LFH_PT2_MODELS_DIR))
                {
                    return Some(Label::PytorchExport);
                }
                if first_name == LFH_SH3D_HOME || first_name == LFH_SH3D_HOME_XML {
                    return Some(Label::Sh3d);
                }
                if first_name == LFH_SKOPS_SCHEMA_ROOT || first_name.ends_with(LFH_SKOPS_SCHEMA_SUB) {
                    return Some(Label::Skops);
                }
                if first_name == LFH_TMDX_DOC_TMD || first_name == LFH_TMDX_DOC_XML {
                    return Some(Label::Tmdx);
                }
                if first_name.ends_with(LFH_TORCHSCRIPT_CONST_SUB)
                    || first_name == LFH_TORCHSCRIPT_CONST_ROOT
                    || (contains(prefix, LFH_TORCHSCRIPT_CONST_ROOT) && contains(prefix, CD_TORCHSCRIPT_CODE_DIR))
                {
                    return Some(Label::Torchscript);
                }
                if first_name.ends_with(LFH_PYTORCH_DATA_SUB)
                    || first_name == LFH_PYTORCH_DATA_ROOT
                    || first_name.ends_with(LFH_PYTORCH_VERSION_SUB)
                {
                    if contains(prefix, LFH_PYTORCH_DATA_ROOT) || contains(prefix, LFH_PYTORCH_BYTEORDER) {
                        return Some(Label::Pytorch);
                    }
                }
                if first_name == LFH_XPI_INSTALL_RDF || first_name == LFH_XPI_CHROME_MANIFEST {
                    return Some(Label::Xpi);
                }
            }
            if contains(prefix, ZIP_ZIP64_EOCD_SIGNATURE)
                && contains(prefix, ZIP_ZIP64_LOCATOR_SIGNATURE)
                && !contains(prefix, LFH_CONTENT_TYPES)
            {
                return Some(Label::Zip);
            }
        }
        if (10..=63).contains(&(ver & 0xFF))
            && name_len == 0
            && contains(prefix, ZIP_CENTRAL_SIGNATURE)
            && contains(prefix, ZIP_EOCD_SIGNATURE)
        {
            return Some(Label::Zip);
        }
    }

    if valid
        || (size >= 22
            && prefix.len() >= 22
            && starts(prefix, ZIP_EMPTY_ARCHIVE))
    {
        return Some(Label::Zip);
    }
    None
}
