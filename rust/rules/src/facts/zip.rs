// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Bounded zip analysis: the end-of-central-directory record, the central directory names,
//! the stored `mimetype` entry and the first entry's contents, derived from the held blocks.
//!
//! Everything here is a pure function of `(first block, input size, tail block)`, so an
//! independent implementation can reproduce the facts and both views byte for byte. Every
//! line of the `zip_names` view is `\n` followed by a payload in which the bytes 0x00 and
//! 0x0A are replaced by 0x01; the `mimetype=<data>` line is sanitized like any other. The
//! tail block is the last 16 KiB of the input, extended backwards to the start of the
//! central directory when [`directory_start`] says so. The contract, in evaluation order:
//!
//! 1. **`mimetype` line.** If `first` starts with a local file header (`PK\x03\x04`) whose
//!    method (u16 at 8) is 0, name length (u16 at 26) is 8, name (at 30) is `mimetype`,
//!    uncompressed size (u32 at 22) is 1..=128, and whose data at `38 + extra_len` (u16 at
//!    28) lies entirely inside `first`, the line `\nmimetype=<data>` is written first, with
//!    `<data>` sanitized. It counts towards `names_len`, not `names_entries`, and needs no
//!    EOCD.
//! 2. **EOCD.** The last 22-byte window of `tail` and every earlier one is tried, walking
//!    backwards; the first `PK\x05\x06` whose comment length (u16 at 20) equals the bytes
//!    remaining after its 22 bytes is the EOCD. `tail` must end at the input's end and be
//!    no longer than the input, otherwise nothing is found. Without an EOCD the directory
//!    facts of steps 3 to 7 are zero and only the `mimetype` line may be in the view; the
//!    first entry of step 8 is analyzed regardless.
//! 3. **EOCD facts.** `entries` is the total entry count (u16 at 10), `comment_len` the
//!    comment length. `flags` bit 0 (zip64) is set by any sentinel: 0xFFFF in the disk
//!    (u16 at 4), directory disk (6), entries on this disk (8) or total (10) fields,
//!    0xFFFFFFFF in the directory size (u32 at 12) or offset (16), or a zip64 locator
//!    (`PK\x06\x07`, 20 bytes) ending exactly where the EOCD starts. Bit 1 (multi-disk)
//!    is set when the disk or directory disk is nonzero or the per-disk count differs
//!    from the total. Multi-disk ends the directory analysis. Zip64 does not: when the
//!    locator's zip64 end-of-central-directory record (`PK\x06\x06`) lies inside `tail`
//!    at the absolute offset the locator names (u64 at locator + 8), its 64-bit total
//!    entries (offset 32), directory size (40) and directory offset (48) override the
//!    32-bit fields, and the directory ends where that record begins. A zip64 archive
//!    whose record is not held, or that carries a sentinel without a locator, keeps bit 0
//!    and reports a plain invalid zip: `cd_size` and the `names_*` facts stay zero.
//! 4. **Directory placement.** `cd_size` is reported, saturated to a u32. The directory is
//!    anchored at its end, not at the declared offset: it is the `cd_size` bytes ending at
//!    `cd_end_abs`, the EOCD's absolute offset `eocd_abs = size - tail.len() + position`
//!    for a plain archive, or the zip64 end record's absolute offset for a zip64 one. So
//!    it starts at `cd_end_abs - cd_size`. This keeps self-extractors walkable, and it is
//!    deliberate: any bytes between the directory's end and the anchor (a `PK\x05\x05`
//!    digital signature record, say) shift the anchor into the directory, so such an
//!    archive reads as malformed and prepended. A reimplementation must mirror this. Bit 5
//!    (prepended data) is set when the declared offset plus `cd_size` is smaller than
//!    `cd_end_abs`. A `cd_size` beyond `cd_end_abs` sets bit 3 (malformed) and ends the
//!    analysis. The directory must lie entirely inside `tail` or, failing that, entirely
//!    inside `first`; otherwise bit 2 (not held) is set and the analysis ends.
//! 5. **Walk.** The directory is walked in two passes, each reading up to `entries` headers
//!    in order: the first pass writes the names that contain no `/`, the second the names
//!    that do, so top-level names such as `classes.dex` are never crowded out by nested
//!    ones. Each header needs `PK\x01\x02`, 46 bytes, and `46 + name_len (u16 at 28) +
//!    extra_len (30) + comment_len (32)` bytes inside the directory; any shortfall sets
//!    bit 3 and ends the walk, both passes, without writing that entry. Each name is
//!    written as `\n` followed by the name with bytes 0x00 and 0x0A replaced by 0x01. A
//!    name that would not leave room for the final terminator (`written + 1 + name_len +
//!    1 > 4096`) is not written; bit 4 (names truncated) is set and the walk ends, both
//!    passes. `names_entries` counts written names.
//! 6. **Terminator.** If any line was written, a final `\n` follows it. `names_len` is the
//!    total number of bytes written; bytes beyond it are left untouched (zero).
//! 7. `valid` is set iff an EOCD was found and bits 1 to 3 (multi-disk, not held, malformed)
//!    are clear; bit 0 (zip64) does not bar a walked archive.
//! 8. **First entry.** When `first` starts with a local file header whose flags (u16 at 6)
//!    have neither bit 0 (encrypted) nor bit 3 (sizes in a data descriptor) set, whose
//!    method (u16 at 8) is 0 (stored) or 8 (deflated), whose name (`name_len`, u16 at 26,
//!    bytes at 30) lies inside `first` with `name_len + 1 < ZIP_FIRST_ENTRY_BYTES`, and
//!    whose compressed data (u32 at 18 bytes, at `30 + name_len + extra_len` with
//!    `extra_len` the u16 at 28) lies entirely inside `first`, the `zip_first_entry` view
//!    receives the name (0x00 and 0x0A replaced by 0x01), `\n`, then the data: stored data
//!    copied, deflated data inflated as raw deflate (RFC 1951, no checksum), truncated to
//!    the space left in the view. Bit 6 (first entry filled) is set when the data reaches
//!    the end of the view, whatever follows it. Otherwise deflated data that does not
//!    decode to the end of its final block (an invalid stream, or compressed bytes ending
//!    early) is discarded: only the name line remains and bit 7 (first entry undecodable)
//!    is set. `first_entry_len` is the number of bytes written; the view is zero beyond
//!    it. Bits 6 and 7 never affect `valid`.

use std::cell::RefCell;

use miniz_oxide::inflate::core::inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
use miniz_oxide::inflate::core::{decompress, DecompressorOxide};
use miniz_oxide::inflate::TINFLStatus;

use super::{u16_at, u32_at, u64_at, ZIP_FIRST_ENTRY_BYTES, ZIP_NAMES_BYTES};

// =============================================================================
// ZIP record signatures, header lengths, and field offsets
// =============================================================================

/// Largest central directory read on its own when the tail window does not hold it.
pub(super) const ZIP_DIRECTORY_MAX_BYTES: u32 = 256 * 1024;

/// Longest stored `mimetype` payload copied into the view.
const MIMETYPE_MAX_LEN: u32 = 128;
const MIMETYPE_NAME: &[u8; 8] = b"mimetype";
const MIMETYPE_PREFIX: &[u8; 9] = b"mimetype=";

/// Name of the OOXML content-types manifest inspected by `first_entry` rules.
const CONTENT_TYPES_NAME: &[u8; 19] = b"[Content_Types].xml";
/// Maximum decompressed payload bytes of `[Content_Types].xml` in the 16 KiB `zip_first_entry` view
/// (`ZIP_FIRST_ENTRY_BYTES` minus `"<19-byte name>\n"`).
const OOXML_BODY_MAX_BYTES: usize = ZIP_FIRST_ENTRY_BYTES - (CONTENT_TYPES_NAME.len() + 1);

/// 4-byte record signatures (`PK\x03\x04`, `PK\x01\x02`, `PK\x05\x06`, `PK\x06\x07`, `PK\x06\x06`).
const LOCAL_SIGNATURE: &[u8; 4] = b"PK\x03\x04"; // Local file header signature
const CENTRAL_SIGNATURE: &[u8; 4] = b"PK\x01\x02"; // Central directory file header signature
const EOCD_SIGNATURE: &[u8; 4] = b"PK\x05\x06"; // End of central directory signature
const ZIP64_LOCATOR_SIGNATURE: &[u8; 4] = b"PK\x06\x07"; // ZIP64 EOCD locator signature
const ZIP64_EOCD_SIGNATURE: &[u8; 4] = b"PK\x06\x06"; // ZIP64 EOCD record signature

/// Local File Header (`PK\x03\x04`, 30 bytes) field offsets and bitflags.
const LOCAL_HEADER_LEN: usize = 30;
const LFH_FLAGS: usize = 6; // @6: u16 LE — general purpose bit flag
const LFH_METHOD: usize = 8; // @8: u16 LE — compression method
const LFH_COMP_SIZE: usize = 18; // @18: u32 LE — compressed size
const LFH_UNCOMP_SIZE: usize = 22; // @22: u32 LE — uncompressed size
const LFH_NAME_LEN: usize = 26; // @26: u16 LE — file name length
const LFH_EXTRA_LEN: usize = 28; // @28: u16 LE — extra field length

/// General-purpose bit flags (`@LFH_FLAGS`) and compression methods (`@LFH_METHOD`).
const GPBF_ENCRYPTED: u16 = 1 << 0; // bit 0: encrypted entry
const GPBF_DATA_DESCRIPTOR: u16 = 1 << 3; // bit 3: sizes/CRC deferred to data descriptor
const METHOD_STORED: u16 = 0; // stored (uncompressed)
const METHOD_DEFLATED: u16 = 8; // deflated

/// Central Directory File Header (`PK\x01\x02`, 46 bytes) field offsets.
const CENTRAL_HEADER_LEN: usize = 46;
const CDH_NAME_LEN: usize = 28; // @28: u16 LE — file name length
const CDH_EXTRA_LEN: usize = 30; // @30: u16 LE — extra field length
const CDH_COMMENT_LEN: usize = 32; // @32: u16 LE — file comment length

/// End of Central Directory record (`PK\x05\x06`, 22 bytes) field offsets.
const EOCD_LEN: usize = 22;
const EOCD_DISK_NUM: usize = 4; // @4: u16 LE — number of this disk
const EOCD_CD_DISK: usize = 6; // @6: u16 LE — disk where central directory starts
const EOCD_DISK_ENTRIES: usize = 8; // @8: u16 LE — number of CD records on this disk
const EOCD_TOTAL_ENTRIES: usize = 10; // @10: u16 LE — total number of CD records
const EOCD_CD_SIZE: usize = 12; // @12: u32 LE — byte size of central directory
const EOCD_CD_OFFSET: usize = 16; // @16: u32 LE — offset of start of central directory
const EOCD_COMMENT_LEN: usize = 20; // @20: u16 LE — ZIP file comment length

/// ZIP64 EOCD Locator (`PK\x06\x07`, 20 bytes) and ZIP64 EOCD Record (`PK\x06\x06`, 56 bytes).
const ZIP64_LOCATOR_LEN: usize = 20;
const ZIP64_LOC_EOCD_OFFSET: usize = 8; // @8: u64 LE — relative offset of ZIP64 EOCD record
const ZIP64_EOCD_LEN: usize = 56;
const ZIP64_EOCD_TOTAL_ENTRIES: usize = 32; // @32: u64 LE — total number of CD entries
const ZIP64_EOCD_CD_SIZE: usize = 40; // @40: u64 LE — byte size of central directory
const ZIP64_EOCD_CD_OFFSET: usize = 48; // @48: u64 LE — offset of start of central directory

/// The archive uses zip64 sentinels or carries a zip64 locator.
const FLAG_ZIP64: u8 = 1 << 0;
/// The archive spans several disks.
const FLAG_MULTIDISK: u8 = 1 << 1;
/// The central directory lies outside the two held blocks.
const FLAG_CD_NOT_HELD: u8 = 1 << 2;
/// The central directory does not parse as declared.
const FLAG_MALFORMED: u8 = 1 << 3;
/// At least one name did not fit the view.
const FLAG_NAMES_TRUNCATED: u8 = 1 << 4;
/// Bytes precede the archive: a self-extractor or an appended archive.
const FLAG_PREPENDED: u8 = 1 << 5;
/// The first entry's data reached the end of the `zip_first_entry` view.
#[cfg(test)]
const FLAG_FIRST_ENTRY_FILLED: u8 = 1 << 6;
/// The first entry is deflated but its data does not decode completely.
#[cfg(test)]
const FLAG_FIRST_ENTRY_UNDECODABLE: u8 = 1 << 7;

/// Bitflags for central-directory entry names and `mimetype=` lines matched by the enforced
/// ZIP cascade rules.
pub(crate) const SEEN_MODEL_WEIGHTS: u32 = 1 << 0;
pub(crate) const SEEN_METADATA_JSON: u32 = 1 << 1;
pub(crate) const SEEN_SUFFIX_QGS: u32 = 1 << 2;
pub(crate) const SEEN_VISIO_DOC: u32 = 1 << 3;
pub(crate) const SEEN_CONTENT_TYPES: u32 = 1 << 4;
pub(crate) const SEEN_XL_WORKBOOK: u32 = 1 << 5;
pub(crate) const SEEN_THREE_MF: u32 = 1 << 6;
pub(crate) const SEEN_ANDROID_MANIFEST: u32 = 1 << 7;
pub(crate) const SEEN_CLASSES_DEX: u32 = 1 << 8;
pub(crate) const SEEN_RESOURCES_ARSC: u32 = 1 << 9;
pub(crate) const SEEN_CONTAINER_XML: u32 = 1 << 10;
pub(crate) const SEEN_CONTENT_XML: u32 = 1 << 11;
pub(crate) const SEEN_MANIFEST_MF: u32 = 1 << 12;
pub(crate) const SEEN_SUFFIX_CLASS: u32 = 1 << 13;
pub(crate) const SEEN_DOC_KML: u32 = 1 << 14;
pub(crate) const SEEN_APPX_MANIFEST: u32 = 1 << 15;
pub(crate) const SEEN_APPX_BLOCKMAP: u32 = 1 << 16;
pub(crate) const SEEN_PPT_PRESENTATION: u32 = 1 << 17;
pub(crate) const MIMETYPE_EPUB: u32 = 1 << 18;
pub(crate) const MIMETYPE_ODP: u32 = 1 << 19;
pub(crate) const MIMETYPE_ODS: u32 = 1 << 20;
pub(crate) const MIMETYPE_ODT: u32 = 1 << 21;

/// Open Packaging Conventions (`[Content_Types].xml`) subtype classified from the first entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OoxmlKind {
    /// WordprocessingML document (`docx` / `docm`).
    Docx,
    /// WordprocessingML template (`dotx` / `dotm`).
    Dotx,
    /// SpreadsheetML binary workbook (`xlsb`).
    Xlsb,
}

/// The zip facts of one input, all zero when the input holds no usable archive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ZipFacts {
    pub valid: bool,
    pub flags: u8,
    pub entries: u16,
    pub names_entries: u16,
    pub names_len: u16,
    pub comment_len: u16,
    pub cd_size: u32,
    pub first_entry_len: u16,
}

#[inline]
fn classify_suffix(name: &[u8]) -> u32 {
    if name.ends_with(b".qgs") {
        SEEN_SUFFIX_QGS
    } else if name.ends_with(b".class") {
        SEEN_SUFFIX_CLASS
    } else {
        0
    }
}

#[inline]
fn classify_first_line(is_mimetype_prefix: bool, text: &[u8]) -> u32 {
    let body = if is_mimetype_prefix {
        text
    } else if let Some(rest) = text.strip_prefix(MIMETYPE_PREFIX) {
        rest
    } else {
        return 0;
    };
    let mask = match body {
        b"application/epub+zip" => MIMETYPE_EPUB,
        b"application/vnd.oasis.opendocument.presentation" => MIMETYPE_ODP,
        b"application/vnd.oasis.opendocument.spreadsheet" => MIMETYPE_ODS,
        b"application/vnd.oasis.opendocument.text" => MIMETYPE_ODT,
        _ => 0,
    };
    if is_mimetype_prefix && mask == 0 {
        return classify_suffix(body);
    }
    mask
}

#[inline]
fn classify_name(name: &[u8]) -> u32 {
    let exact = match name.len() {
        7 if name == b"doc.kml" => SEEN_DOC_KML,
        11 => match name {
            b"classes.dex" => SEEN_CLASSES_DEX,
            b"content.xml" => SEEN_CONTENT_XML,
            _ => 0,
        },
        13 if name == b"metadata.json" => SEEN_METADATA_JSON,
        14 if name == b"resources.arsc" => SEEN_RESOURCES_ARSC,
        15 if name == b"xl/workbook.xml" => SEEN_XL_WORKBOOK,
        16 => match name {
            b"model.weights.h5" => SEEN_MODEL_WEIGHTS,
            b"3D/3dmodel.model" => SEEN_THREE_MF,
            b"AppxManifest.xml" => SEEN_APPX_MANIFEST,
            b"AppxBlockMap.xml" => SEEN_APPX_BLOCKMAP,
            _ => 0,
        },
        18 if name == b"visio/document.xml" => SEEN_VISIO_DOC,
        19 => match name {
            b"[Content_Types].xml" => SEEN_CONTENT_TYPES,
            b"AndroidManifest.xml" => SEEN_ANDROID_MANIFEST,
            _ => 0,
        },
        20 => match name {
            b"META-INF/MANIFEST.MF" => SEEN_MANIFEST_MF,
            b"ppt/presentation.xml" => SEEN_PPT_PRESENTATION,
            _ => 0,
        },
        22 if name == b"META-INF/container.xml" => SEEN_CONTAINER_XML,
        _ => 0,
    };
    if exact != 0 {
        return exact;
    }
    classify_suffix(name)
}

/// Analyzes the archive held in `first` and `tail` without materializing the 4 KiB `zip_names`
/// buffer, computing all [`ZipFacts`] fields and the `seen` entry bitmask in place.
pub(super) fn analyze_facts(first: &[u8], size: u64, tail: &[u8]) -> (ZipFacts, u32) {
    let mut facts = ZipFacts::default();
    let mut view_len = 0usize;
    let mut seen = 0u32;
    if let Some(mimetype) = stored_mimetype(first) {
        view_len = 1 + MIMETYPE_PREFIX.len() + mimetype.len();
        seen |= classify_first_line(true, mimetype);
    }
    if let Some(eocd) = find_eocd(tail) {
        walk_directory_facts(first, size, tail, eocd, &mut view_len, &mut facts, &mut seen);
    }
    if view_len > 0 {
        facts.names_len = (view_len + 1) as u16;
    }
    (facts, seen)
}

/// Analyzes the archive held in `first` (the block at offset 0) and `tail` (the block ending
/// at `size`, the input length), writing the `zip_names` view into `names_out`. Never panics
/// and never reads outside the two blocks; see the module documentation for the contract.
#[cfg(test)]
pub(super) fn analyze(
    first: &[u8], size: u64, tail: &[u8], names_out: &mut [u8; ZIP_NAMES_BYTES],
) -> ZipFacts {
    let mut view = ViewWriter { out: names_out, len: 0 };
    let mut facts = ZipFacts::default();
    if let Some(mimetype) = stored_mimetype(first) {
        // The line always fits: it is at most 1 + 9 + 128 bytes into an empty view.
        view.line(MIMETYPE_PREFIX, mimetype);
    }
    if let Some(eocd) = find_eocd(tail) {
        walk_directory(first, size, tail, eocd, &mut view, &mut facts);
    }
    facts.names_len = view.finish();
    facts
}

/// Appends `\n`-led lines to the view and terminates the sequence once at the end.
#[cfg(test)]
struct ViewWriter<'a> {
    out: &'a mut [u8; ZIP_NAMES_BYTES],
    len: usize,
}

#[cfg(test)]
impl ViewWriter<'_> {
    /// Whether a line of `prefix` and `text` fits, keeping one byte for the terminator.
    fn fits(&self, prefix: &[u8], text: &[u8]) -> bool {
        // Line: the leading `\n`, the prefix and the text; the terminator needs one more.
        let line = 1 + prefix.len() + text.len();
        self.len + line < ZIP_NAMES_BYTES
    }

    /// Writes `\n`, `prefix` verbatim and `text` sanitized. The caller checks [`Self::fits`].
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

    /// Terminates a nonempty view and returns its length.
    fn finish(self) -> u16 {
        if self.len == 0 {
            return 0;
        }
        self.out[self.len] = b'\n';
        (self.len + 1) as u16
    }
}

/// The payload of a stored, short `mimetype` first entry, when `first` holds it entirely.
#[inline]
fn stored_mimetype(first: &[u8]) -> Option<&[u8]> {
    if !first.starts_with(LOCAL_SIGNATURE)
        || u16_at(first, LFH_METHOD)? != METHOD_STORED
        || u16_at(first, LFH_NAME_LEN)? != MIMETYPE_NAME.len() as u16
    {
        return None;
    }
    let length = u32_at(first, LFH_UNCOMP_SIZE)?;
    if !(1..=MIMETYPE_MAX_LEN).contains(&length) {
        return None;
    }
    let extra_len = usize::from(u16_at(first, LFH_EXTRA_LEN)?);
    let name_end = LOCAL_HEADER_LEN + MIMETYPE_NAME.len();
    if first.get(LOCAL_HEADER_LEN..name_end)? != MIMETYPE_NAME {
        return None;
    }
    let data_start = name_end.checked_add(extra_len)?;
    first.get(data_start..data_start.checked_add(length as usize)?)
}

/// Position in `tail` of the end-of-central-directory record whose comment reaches the end.
#[inline]
fn find_eocd(tail: &[u8]) -> Option<usize> {
    let last = tail.len().checked_sub(EOCD_LEN)?;
    if tail[last..].starts_with(EOCD_SIGNATURE) && u16_at(tail, last + EOCD_COMMENT_LEN) == Some(0)
    {
        return Some(last);
    }
    memchr::memmem::rfind_iter(tail, EOCD_SIGNATURE).find(|&at| {
        u16_at(tail, at + EOCD_COMMENT_LEN)
            .is_some_and(|comment_len| usize::from(comment_len) == tail.len() - at - EOCD_LEN)
    })
}

/// Decodes the EOCD at `eocd` in `tail` and returns the held central directory slice and entry count.
fn locate_directory<'a>(
    first: &'a [u8], size: u64, tail: &'a [u8], eocd: usize, facts: &mut ZipFacts,
) -> Option<(&'a [u8], u64)> {
    let tail_start = size.checked_sub(tail.len() as u64)?;
    let record = Eocd::decode(tail, eocd);
    facts.entries = record.entries;
    facts.comment_len = record.comment_len;
    if record.flags & FLAG_MULTIDISK != 0 {
        facts.flags |= record.flags;
        return None;
    }
    let eocd_abs = tail_start + eocd as u64;
    // The directory ends at the EOCD, or, for a zip64 archive, at the zip64 end record its
    // locator points to. Resolve the 64-bit entry count, size and offsets: a zip64 archive
    // whose end record is not held (or that carries a sentinel without a locator) keeps its
    // flag and reports a plain invalid zip, exactly as before zip64 was walked.
    let (entries, cd_size, cd_offset, cd_end_abs) = match zip64_directory(tail, eocd, tail_start) {
        Some(resolved) => {
            facts.flags |= FLAG_ZIP64;
            resolved
        }
        None if record.flags & FLAG_ZIP64 != 0 => {
            facts.flags |= record.flags;
            return None;
        }
        None => (
            u64::from(record.entries),
            u64::from(record.cd_size),
            u64::from(record.cd_offset),
            eocd_abs,
        ),
    };
    facts.cd_size = cd_size.min(u64::from(u32::MAX)) as u32;
    facts.entries = entries.min(u64::from(u16::MAX)) as u16;
    if cd_offset.saturating_add(cd_size) < cd_end_abs {
        facts.flags |= FLAG_PREPENDED;
    }
    let Some(cd_abs) = cd_end_abs.checked_sub(cd_size) else {
        facts.flags |= FLAG_MALFORMED;
        return None;
    };
    // The directory ends at `cd_end_abs`, so it is inside `tail` iff it starts there; `first`
    // starts at 0, so it is inside `first` iff it ends there.
    let directory = if cd_abs >= tail_start {
        let (start, end) = ((cd_abs - tail_start) as usize, (cd_end_abs - tail_start) as usize);
        match tail.get(start..end) {
            Some(slice) => slice,
            None => {
                facts.flags |= FLAG_MALFORMED;
                return None;
            }
        }
    } else if cd_end_abs <= first.len() as u64 {
        // Reached with production blocks only when a long archive comment keeps the end
        // record inside `first` while the 16 KiB tail starts after the directory.
        &first[cd_abs as usize..cd_end_abs as usize]
    } else {
        facts.flags |= FLAG_CD_NOT_HELD;
        return None;
    };
    Some((directory, entries))
}

/// Walks the central directory to compute [`ZipFacts`] and the `seen` bitmask without copying
/// entry names into a 4 KiB buffer.
fn walk_directory_facts(
    first: &[u8], size: u64, tail: &[u8], eocd: usize, view_len: &mut usize, facts: &mut ZipFacts,
    seen: &mut u32,
) {
    let Some((directory, entries)) = locate_directory(first, size, tail, eocd, facts) else {
        return;
    };
    // Single-pass fast path: walk `0..entries` accumulating `names_bytes` and `pass_seen`.
    // As long as `*view_len + names_bytes < ZIP_NAMES_BYTES`, every entry (both top-level and
    // nested) fits in the 4 KiB view without truncation, so `memchr(b'/', name)` is only needed
    // until the first top-level entry is found (when `*view_len == 0`).
    let mut at = 0;
    let mut names_bytes = 0usize;
    let mut pass_seen = 0u32;
    let mut first_top: Option<&[u8]> = None;
    let mut first_nested: Option<&[u8]> = None;
    let mut fits_all = true;
    for _ in 0..entries {
        let Some((name, len)) = entry_name(directory, at) else {
            fits_all = false;
            break;
        };
        at += len;
        names_bytes += 1 + name.len();
        if *view_len + names_bytes >= ZIP_NAMES_BYTES {
            fits_all = false;
            break;
        }
        pass_seen |= classify_name(name);
        if *view_len == 0 && first_top.is_none() {
            if memchr::memchr(b'/', name).is_none() {
                first_top = Some(name);
            } else if first_nested.is_none() {
                first_nested = Some(name);
            }
        }
    }
    if fits_all {
        if let Some(name) = first_top.or(first_nested) {
            *seen |= classify_first_line(false, name);
        }
        *view_len += names_bytes;
        facts.names_entries = entries as u16;
        *seen |= pass_seen;
        facts.valid = facts.flags & (FLAG_MULTIDISK | FLAG_CD_NOT_HELD | FLAG_MALFORMED) == 0;
        return;
    }

    // Fallback 2-pass walk when the entry names overflow the 4 KiB view (`FLAG_NAMES_TRUNCATED`).
    'passes: for nested in [false, true] {
        let mut at = 0;
        for _ in 0..entries {
            let Some((name, len)) = entry_name(directory, at) else {
                facts.flags |= FLAG_MALFORMED;
                break 'passes;
            };
            at += len;
            if memchr::memchr(b'/', name).is_some() != nested {
                continue;
            }
            if *view_len + 1 + name.len() >= ZIP_NAMES_BYTES {
                facts.flags |= FLAG_NAMES_TRUNCATED;
                break 'passes;
            }
            if *view_len == 0 {
                *seen |= classify_first_line(false, name);
            }
            *view_len += 1 + name.len();
            facts.names_entries += 1;
            *seen |= classify_name(name);
        }
    }
    facts.valid = facts.flags & (FLAG_MULTIDISK | FLAG_CD_NOT_HELD | FLAG_MALFORMED) == 0;
}

/// Decodes the EOCD at `eocd` in `tail`, places the directory and walks its names.
#[cfg(test)]
fn walk_directory(
    first: &[u8], size: u64, tail: &[u8], eocd: usize, view: &mut ViewWriter<'_>,
    facts: &mut ZipFacts,
) {
    let Some((directory, entries)) = locate_directory(first, size, tail, eocd, facts) else {
        return;
    };
    // Top-level names first, then nested ones: both passes walk the same headers. A hostile
    // entry count is harmless: `entry_name` fails once `at` leaves the held directory, which
    // bounds the walk to the directory's own length no matter how large `entries` claims to be.
    'passes: for nested in [false, true] {
        let mut at = 0;
        for _ in 0..entries {
            let Some((name, len)) = entry_name(directory, at) else {
                facts.flags |= FLAG_MALFORMED;
                break 'passes;
            };
            at += len;
            if name.contains(&b'/') != nested {
                continue;
            }
            if !view.fits(b"", name) {
                facts.flags |= FLAG_NAMES_TRUNCATED;
                break 'passes;
            }
            view.line(b"", name);
            facts.names_entries += 1;
        }
    }
    facts.valid = facts.flags & (FLAG_MULTIDISK | FLAG_CD_NOT_HELD | FLAG_MALFORMED) == 0;
}

/// Resolves the directory of a zip64 archive. When the EOCD at `eocd` carries a zip64 locator
/// whose zip64 end-of-central-directory record lies inside `tail`, returns the total entry
/// count, the directory size, the directory offset field and the absolute offset at which the
/// directory ends (where the zip64 end record begins). `None` when there is no locator or its
/// record is not held, in which case the caller falls back to the 32-bit fields.
fn zip64_directory(tail: &[u8], eocd: usize, tail_start: u64) -> Option<(u64, u64, u64, u64)> {
    let locator = eocd.checked_sub(ZIP64_LOCATOR_LEN)?;
    if !tail[locator..].starts_with(ZIP64_LOCATOR_SIGNATURE) {
        return None;
    }
    let record_abs = u64_at(tail, locator + ZIP64_LOC_EOCD_OFFSET)?;
    let at = usize::try_from(record_abs.checked_sub(tail_start)?).ok()?;
    let record = tail.get(at..at.checked_add(ZIP64_EOCD_LEN)?)?;
    if !record.starts_with(ZIP64_EOCD_SIGNATURE) {
        return None;
    }
    Some((
        u64_at(record, ZIP64_EOCD_TOTAL_ENTRIES)?,
        u64_at(record, ZIP64_EOCD_CD_SIZE)?,
        u64_at(record, ZIP64_EOCD_CD_OFFSET)?,
        record_abs,
    ))
}

/// The name of the central directory header at `at` and the length of the whole entry,
/// name, extra field and comment included, when all of it lies inside the directory.
#[inline]
fn entry_name(directory: &[u8], at: usize) -> Option<(&[u8], usize)> {
    let header = directory.get(at..at.checked_add(CENTRAL_HEADER_LEN)?)?;
    if !header.starts_with(CENTRAL_SIGNATURE) {
        return None;
    }
    let name_len = usize::from(u16_at(header, CDH_NAME_LEN)?);
    let trailer =
        usize::from(u16_at(header, CDH_EXTRA_LEN)?) + usize::from(u16_at(header, CDH_COMMENT_LEN)?);
    let name_start = at + CENTRAL_HEADER_LEN;
    let name_end = name_start.checked_add(name_len)?;
    // The extra field and comment must fit as well, or the next header cannot be found.
    let entry_end = name_end.checked_add(trailer)?;
    if entry_end > directory.len() {
        return None;
    }
    Some((directory.get(name_start..name_end)?, entry_end - at))
}

/// The fields of an end-of-central-directory record that locate and qualify its directory.
struct Eocd {
    entries: u16,
    comment_len: u16,
    cd_size: u32,
    cd_offset: u32,
    /// `FLAG_ZIP64` and `FLAG_MULTIDISK`, as step 3 defines them.
    flags: u8,
}

impl Eocd {
    /// Decodes the record at `eocd` in `tail`, a position [`find_eocd`] returned.
    #[inline]
    fn decode(tail: &[u8], eocd: usize) -> Self {
        let record = &tail[eocd..eocd + EOCD_LEN];
        let field = |at| u16_at(record, at).unwrap_or(0);
        let (disk, cd_disk, entries_disk, entries) = (
            field(EOCD_DISK_NUM),
            field(EOCD_CD_DISK),
            field(EOCD_DISK_ENTRIES),
            field(EOCD_TOTAL_ENTRIES),
        );
        let cd_size = u32_at(record, EOCD_CD_SIZE).unwrap_or(0);
        let cd_offset = u32_at(record, EOCD_CD_OFFSET).unwrap_or(0);
        let locator = eocd
            .checked_sub(ZIP64_LOCATOR_LEN)
            .is_some_and(|at| tail[at..].starts_with(ZIP64_LOCATOR_SIGNATURE));
        let mut flags = 0;
        if [disk, cd_disk, entries_disk, entries].contains(&u16::MAX)
            || cd_size == u32::MAX
            || cd_offset == u32::MAX
            || locator
        {
            flags |= FLAG_ZIP64;
        }
        if disk != 0 || cd_disk != 0 || entries_disk != entries {
            flags |= FLAG_MULTIDISK;
        }
        Self { entries, comment_len: field(EOCD_COMMENT_LEN), cd_size, cd_offset, flags }
    }
}

/// The absolute offset at which to start reading so that the central directory is held,
/// when `tail` (the block ending at `size`) holds the end record but not the directory:
/// the archive is single-disk, not zip64, and its directory, anchored at the end record as
/// in step 4, starts before `tail` and spans at most `ZIP_DIRECTORY_MAX_BYTES`. Reading
/// `[start, size - tail.len())` ahead of `tail` then gives the analysis a held directory.
pub(super) fn directory_start(tail: &[u8], size: u64) -> Option<u64> {
    let eocd = find_eocd(tail)?;
    let record = Eocd::decode(tail, eocd);
    let tail_start = size.checked_sub(tail.len() as u64)?;
    let eocd_abs = tail_start + eocd as u64;
    // A zip64 archive extends from its zip64 end record when that record is held in `tail`;
    // otherwise only a plain, single-disk archive is followed.
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

/// Writes the `zip_first_entry` view of step 8 and returns its length and flag bits.
#[cfg(test)]
pub(super) fn first_entry(
    first: &[u8], out: &mut [u8; ZIP_FIRST_ENTRY_BYTES], inflater: &mut DecompressorOxide,
) -> (u16, u8) {
    let Some((name, data, deflated)) = first_entry_parts(first) else { return (0, 0) };
    for (slot, &byte) in out.iter_mut().zip(name) {
        *slot = if byte == 0 || byte == b'\n' { 1 } else { byte };
    }
    out[name.len()] = b'\n';
    let head = name.len() + 1;
    let space = &mut out[head..];
    let (written, complete) = if deflated {
        inflater.init();
        let flags = TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
        let (status, _, written) = decompress(inflater, data, space, 0, flags);
        (written, status == TINFLStatus::Done)
    } else {
        let written = data.len().min(space.len());
        space[..written].copy_from_slice(&data[..written]);
        (written, true)
    };
    if written == space.len() {
        return (ZIP_FIRST_ENTRY_BYTES as u16, FLAG_FIRST_ENTRY_FILLED);
    }
    if !complete {
        space[..written].fill(0);
        return (head as u16, FLAG_FIRST_ENTRY_UNDECODABLE);
    }
    ((head + written) as u16, 0)
}

/// Scans a `[Content_Types].xml` payload in a single `memmem` pass over `".main"` to classify
/// the OOXML package subtype (`Docx`, `Dotx`, or `Xlsb`), preserving rule priority (`Docx > Dotx > Xlsb`).
#[inline]
fn scan_ooxml_content_types(xml: &[u8]) -> Option<OoxmlKind> {
    const DOCX_STD: &[u8] =
        b"application/vnd.openxmlformats-officedocument.wordprocessingml.document";
    const DOCX_MACRO: &[u8] = b"application/vnd.ms-word.document.macroEnabled";
    const DOTX_STD: &[u8] =
        b"application/vnd.openxmlformats-officedocument.wordprocessingml.template";
    const DOTX_MACRO: &[u8] = b"application/vnd.ms-word.template.macroEnabledTemplate";
    const XLSB_MAIN: &[u8] = b"application/vnd.ms-excel.sheet.binary.macroEnabled";

    let mut saw_dotx = false;
    let mut saw_xlsb = false;
    for pos in memchr::memmem::find_iter(xml, b".main") {
        let before = &xml[..pos];
        let after = &xml[pos + 5..];
        if after.starts_with(b"+xml") {
            if before.ends_with(DOCX_STD) || before.ends_with(DOCX_MACRO) {
                return Some(OoxmlKind::Docx);
            }
            if before.ends_with(DOTX_STD) || before.ends_with(DOTX_MACRO) {
                saw_dotx = true;
            }
        }
        if before.ends_with(XLSB_MAIN) {
            saw_xlsb = true;
        }
    }
    if saw_dotx {
        Some(OoxmlKind::Dotx)
    } else if saw_xlsb {
        Some(OoxmlKind::Xlsb)
    } else {
        None
    }
}

/// Scratch buffer and reusable DEFLATE state per thread, avoiding `16 KiB` + `11 KiB` heap
/// allocations and zero-fills on every OOXML archive.
struct InflateScratch {
    buf: [u8; OOXML_BODY_MAX_BYTES],
    inflater: DecompressorOxide,
}

thread_local! {
    static INFLATE_SCRATCH: RefCell<Box<InflateScratch>> = RefCell::new(Box::new(InflateScratch {
        buf: [0u8; OOXML_BODY_MAX_BYTES],
        inflater: DecompressorOxide::default(),
    }));
}

/// Classifies the first local entry of `first` when it is `[Content_Types].xml`, scanning stored
/// entries in place with zero copies and inflating deflated entries into thread-local scratch.
pub(crate) fn classify_ooxml_first_entry(first: &[u8]) -> Option<OoxmlKind> {
    let (name, data, deflated) = first_entry_parts(first)?;
    if name != CONTENT_TYPES_NAME {
        return None;
    }
    if !deflated {
        let len = data.len().min(OOXML_BODY_MAX_BYTES);
        return scan_ooxml_content_types(&data[..len]);
    }
    INFLATE_SCRATCH.with(|cell| {
        let mut scratch = cell.borrow_mut();
        let InflateScratch { buf, inflater } = &mut **scratch;
        inflater.init();
        let flags = TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
        let (status, _, written) = decompress(inflater, data, buf, 0, flags);
        if written < OOXML_BODY_MAX_BYTES && status != TINFLStatus::Done {
            return None;
        }
        scan_ooxml_content_types(&buf[..written])
    })
}

/// The name, the compressed data and whether it is deflated, for a first local entry that
/// step 8 accepts.
#[inline]
fn first_entry_parts(first: &[u8]) -> Option<(&[u8], &[u8], bool)> {
    if !first.starts_with(LOCAL_SIGNATURE) {
        return None;
    }
    let flags = u16_at(first, LFH_FLAGS)?;
    let method = u16_at(first, LFH_METHOD)?;
    if flags & (GPBF_ENCRYPTED | GPBF_DATA_DESCRIPTOR) != 0
        || !(method == METHOD_STORED || method == METHOD_DEFLATED)
    {
        return None;
    }
    let compressed = usize::try_from(u32_at(first, LFH_COMP_SIZE)?).ok()?;
    let name_len = usize::from(u16_at(first, LFH_NAME_LEN)?);
    let extra_len = usize::from(u16_at(first, LFH_EXTRA_LEN)?);
    if name_len + 1 >= ZIP_FIRST_ENTRY_BYTES {
        return None;
    }
    let name = first.get(LOCAL_HEADER_LEN..LOCAL_HEADER_LEN + name_len)?;
    let data_start = LOCAL_HEADER_LEN + name_len + extra_len;
    let data = first.get(data_start..data_start.checked_add(compressed)?)?;
    Some((name, data, method == METHOD_DEFLATED))
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    pub(crate) struct Entry {
        name: Vec<u8>,
        data: Vec<u8>,
        method: u16,
    }

    pub(crate) fn stored(name: &[u8], data: &[u8]) -> Entry {
        Entry { name: name.to_vec(), data: data.to_vec(), method: 0 }
    }

    /// A single-disk archive: local entries, the central directory, the EOCD and its comment.
    pub(crate) fn archive(entries: &[Entry], comment: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut offsets = Vec::new();
        for entry in entries {
            offsets.push(out.len() as u32);
            out.extend_from_slice(b"PK\x03\x04");
            out.extend_from_slice(&[20, 0, 0, 0]); // version, flags
            out.extend_from_slice(&entry.method.to_le_bytes());
            out.extend_from_slice(&[0; 8]); // time, date, crc
            out.extend_from_slice(&(entry.data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(entry.data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(entry.name.len() as u16).to_le_bytes());
            out.extend_from_slice(&[0, 0]); // extra length
            out.extend_from_slice(&entry.name);
            out.extend_from_slice(&entry.data);
        }
        let cd_offset = out.len() as u32;
        for (entry, offset) in entries.iter().zip(&offsets) {
            out.extend_from_slice(b"PK\x01\x02");
            out.extend_from_slice(&[20, 0, 20, 0, 0, 0]); // made by, needed, flags
            out.extend_from_slice(&entry.method.to_le_bytes());
            out.extend_from_slice(&[0; 8]); // time, date, crc
            out.extend_from_slice(&(entry.data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(entry.data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(entry.name.len() as u16).to_le_bytes());
            out.extend_from_slice(&[0, 0, 0, 0]); // extra length, comment length
            out.extend_from_slice(&[0; 8]); // disk, internal and external attributes
            out.extend_from_slice(&offset.to_le_bytes());
            out.extend_from_slice(&entry.name);
        }
        let cd_size = out.len() as u32 - cd_offset;
        out.extend_from_slice(b"PK\x05\x06");
        out.extend_from_slice(&[0, 0, 0, 0]); // disk, central directory disk
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&cd_size.to_le_bytes());
        out.extend_from_slice(&cd_offset.to_le_bytes());
        out.extend_from_slice(&(comment.len() as u16).to_le_bytes());
        out.extend_from_slice(comment);
        out
    }

    /// Offset of the EOCD record of an archive built by [`archive`] with `comment` bytes.
    fn eocd_at(bytes: &[u8], comment_len: usize) -> usize {
        bytes.len() - EOCD_LEN - comment_len
    }

    fn put_u16(bytes: &mut [u8], at: usize, value: u16) {
        bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut [u8], at: usize, value: u32) {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    /// Analyzes a small input held entirely in one block, as `first` and `tail` alike.
    fn small(bytes: &[u8]) -> (ZipFacts, Box<[u8; ZIP_NAMES_BYTES]>) {
        let mut names = Box::new([0; ZIP_NAMES_BYTES]);
        let facts = analyze(bytes, bytes.len() as u64, bytes, &mut names);
        assert_eq!(analyze_facts(bytes, bytes.len() as u64, bytes).0, facts);
        (facts, names)
    }

    /// Analyzes with the bounded blocks the pipeline holds: 4096 bytes at 0, 16 KiB at the end.
    fn blocks(bytes: &[u8]) -> (ZipFacts, Box<[u8; ZIP_NAMES_BYTES]>) {
        let mut names = Box::new([0; ZIP_NAMES_BYTES]);
        let first = &bytes[..bytes.len().min(4096)];
        let tail = &bytes[bytes.len().saturating_sub(16 * 1024)..];
        let facts = analyze(first, bytes.len() as u64, tail, &mut names);
        assert_eq!(analyze_facts(first, bytes.len() as u64, tail).0, facts);
        (facts, names)
    }

    fn view(names: &[u8; ZIP_NAMES_BYTES], len: usize) -> &[u8] {
        assert!(names[len..].iter().all(|x| *x == 0), "bytes beyond the view are untouched");
        &names[..len]
    }

    /// Grows the central directory of a single-entry archive built by [`archive`] without a
    /// comment by `pad` bytes of extra field on that entry, keeping the EOCD consistent.
    fn pad_directory(bytes: &mut Vec<u8>, pad: u16) {
        let eocd = eocd_at(bytes, 0);
        let cd_size = u32_at(bytes, eocd + 12).unwrap();
        let cd = eocd - cd_size as usize;
        assert_eq!(u16_at(bytes, eocd + 10), Some(1), "single entry");
        assert_eq!(u16_at(bytes, cd + 30), Some(0), "no extra field yet");
        put_u16(bytes, cd + 30, pad);
        bytes.splice(eocd..eocd, std::iter::repeat_n(0, usize::from(pad)));
        put_u32(bytes, eocd + usize::from(pad) + 12, cd_size + u32::from(pad));
    }

    /// The invariants every analysis upholds, whatever the input.
    fn check(facts: &ZipFacts, names: &[u8; ZIP_NAMES_BYTES]) {
        let names_len = usize::from(facts.names_len);
        assert!(names_len <= ZIP_NAMES_BYTES, "{facts:?}");
        assert!(facts.names_entries <= facts.entries, "{facts:?}");
        if names_len > 0 {
            assert_eq!((names[0], names[names_len - 1]), (b'\n', b'\n'), "{facts:?}");
        }
        if facts.valid {
            // A valid archive may be zip64 (bit 0); only multi-disk, not-held and malformed bar it.
            assert_eq!(facts.flags & 0b1110, 0, "{facts:?}");
        }
    }

    #[test]
    fn minimal_archive_yields_exact_facts_and_view() {
        let bytes = archive(&[stored(b"a.txt", b"hello"), stored(b"dir/b.bin", b"")], b"");
        let (facts, names) = small(&bytes);
        assert_eq!(
            facts,
            ZipFacts {
                valid: true,
                flags: 0,
                entries: 2,
                names_entries: 2,
                names_len: 17,
                comment_len: 0,
                cd_size: 2 * 46 + 5 + 9,
                first_entry_len: 0,
            }
        );
        assert_eq!(view(&names, 17), b"\na.txt\ndir/b.bin\n");
    }

    #[test]
    fn archive_comment_is_measured_and_eocd_found_before_it() {
        for comment in [&b""[..], b"a", b"ab", b"abc", b"abcd", b"a comment"] {
            let bytes = archive(&[stored(b"x", b"1")], comment);
            let (facts, names) = small(&bytes);
            assert!(facts.valid, "comment len {}", comment.len());
            assert_eq!(facts.comment_len as usize, comment.len());
            assert_eq!(facts.entries, 1);
            assert_eq!(view(&names, 3), b"\nx\n");
        }
    }

    #[test]
    fn nested_directory_entry_sets_first_line_when_no_top_level_entries_exist() {
        let bytes = archive(
            &[
                stored(b"mimetype=application/epub+zip", b""),
                stored(b"META-INF/container.xml", b"<container/>"),
            ],
            b"",
        );
        let (facts, names) = small(&bytes);
        let (fast_facts, signals) = analyze_facts(&bytes, bytes.len() as u64, &bytes);
        assert_eq!(fast_facts, facts);
        assert_ne!(signals & MIMETYPE_EPUB, 0);
        assert_ne!(signals & SEEN_CONTAINER_XML, 0);
        assert_eq!(
            view(&names, facts.names_len as usize),
            b"\nmimetype=application/epub+zip\nMETA-INF/container.xml\n"
        );
    }

    #[test]
    fn empty_archive_is_valid_with_an_empty_view() {
        let bytes = archive(&[], b"");
        let (facts, names) = small(&bytes);
        assert_eq!(
            facts,
            ZipFacts { valid: true, flags: 0, entries: 0, cd_size: 0, ..ZipFacts::default() }
        );
        assert_eq!(view(&names, 0), b"");
    }

    #[test]
    fn stored_mimetype_first_entry_adds_a_line_before_the_names() {
        let bytes =
            archive(&[stored(b"mimetype", b"application/epub+zip"), stored(b"OEBPS/x", b"")], b"");
        let (facts, names) = small(&bytes);
        let expected = b"\nmimetype=application/epub+zip\nmimetype\nOEBPS/x\n";
        assert!(facts.valid);
        assert_eq!(facts.names_entries, 2);
        assert_eq!(facts.names_len as usize, expected.len());
        assert_eq!(view(&names, expected.len()), expected);
        // The line is derived from the first block alone: a truncated archive keeps it.
        let (facts, names) = small(&bytes[..60]);
        assert_eq!(facts, ZipFacts { names_len: 31, ..ZipFacts::default() });
        assert_eq!(view(&names, 31), b"\nmimetype=application/epub+zip\n");
        // Only a stored, 1..=128 byte, in-block payload qualifies.
        let mut deflated = bytes.clone();
        put_u16(&mut deflated, 8, 8);
        assert_eq!(view(&small(&deflated).1, 18), b"\nmimetype\nOEBPS/x\n");
        let mut empty = bytes.clone();
        put_u32(&mut empty, 22, 0);
        assert_eq!(view(&small(&empty).1, 18), b"\nmimetype\nOEBPS/x\n");
        let mut huge = bytes.clone();
        put_u32(&mut huge, 22, 129);
        assert_eq!(view(&small(&huge).1, 18), b"\nmimetype\nOEBPS/x\n");
        let mut beyond = bytes.clone();
        put_u16(&mut beyond, 28, 4096);
        assert_eq!(view(&small(&beyond).1, 18), b"\nmimetype\nOEBPS/x\n");
        let mut other = bytes.clone();
        other[30..38].copy_from_slice(b"mimetypX");
        // Only the local header was renamed: the directory still lists `mimetype`.
        assert_eq!(view(&small(&other).1, 18), b"\nmimetype\nOEBPS/x\n");
        let mut sanitized = bytes.clone();
        sanitized[38] = b'\n';
        sanitized[39] = 0;
        assert_eq!(view(&small(&sanitized).1, 49)[..12], *b"\nmimetype=\x01\x01");
    }

    #[test]
    fn eocd_with_a_mismatched_comment_length_is_rejected() {
        let mut bytes = archive(&[stored(b"a", b"1")], b"");
        let eocd = eocd_at(&bytes, 0);
        put_u16(&mut bytes, eocd + 20, 5);
        let (facts, names) = small(&bytes);
        assert_eq!(facts, ZipFacts::default());
        assert_eq!(view(&names, 0), b"");
        // A shorter comment than declared is rejected too; a longer one is not an EOCD.
        let mut bytes = archive(&[stored(b"a", b"1")], b"abc");
        let eocd = eocd_at(&bytes, 3);
        put_u16(&mut bytes, eocd + 20, 2);
        assert_eq!(small(&bytes).0, ZipFacts::default());
    }

    #[test]
    fn eocd_at_the_very_start_of_the_tail_uses_the_first_block_directory() {
        let bytes = archive(&[stored(b"a", b"1"), stored(b"bb", b"22")], b"");
        let mut names = Box::new([0; ZIP_NAMES_BYTES]);
        let tail = &bytes[bytes.len() - EOCD_LEN..];
        let facts = analyze(&bytes, bytes.len() as u64, tail, &mut names);
        assert!(facts.valid, "{facts:?}");
        assert_eq!(facts.names_entries, 2);
        assert_eq!(view(&names, 6), b"\na\nbb\n");
        // Without the first block, the directory is held by neither block.
        let mut names = Box::new([0; ZIP_NAMES_BYTES]);
        let facts = analyze(&bytes[..4], bytes.len() as u64, tail, &mut names);
        assert_eq!(
            facts,
            ZipFacts {
                valid: false,
                flags: FLAG_CD_NOT_HELD,
                entries: 2,
                cd_size: 2 * 46 + 3,
                ..ZipFacts::default()
            }
        );
        assert_eq!(view(&names, 0), b"");
    }

    #[test]
    fn directory_filling_the_tail_window_exactly_is_held() {
        // The directory ends at the EOCD, which ends the 16 KiB tail: it is held iff it
        // starts at or after the tail's first byte.
        let mut held = archive(&[stored(b"a", b"1")], b"");
        pad_directory(&mut held, (16 * 1024 - EOCD_LEN - 47) as u16);
        assert!(held.len() > 16 * 1024);
        let (facts, names) = blocks(&held);
        assert_eq!(
            facts,
            ZipFacts {
                valid: true,
                flags: 0,
                entries: 1,
                names_entries: 1,
                names_len: 3,
                comment_len: 0,
                cd_size: (16 * 1024 - EOCD_LEN) as u32,
                first_entry_len: 0,
            }
        );
        assert_eq!(view(&names, 3), b"\na\n");
        let mut unheld = archive(&[stored(b"a", b"1")], b"");
        pad_directory(&mut unheld, (16 * 1024 - EOCD_LEN - 47 + 1) as u16);
        let (facts, names) = blocks(&unheld);
        assert_eq!(
            facts,
            ZipFacts {
                valid: false,
                flags: FLAG_CD_NOT_HELD,
                entries: 1,
                cd_size: (16 * 1024 - EOCD_LEN + 1) as u32,
                ..ZipFacts::default()
            }
        );
        assert_eq!(view(&names, 0), b"");
    }

    #[test]
    fn long_comment_keeps_the_eocd_and_directory_in_the_first_block() {
        // With the production blocks, the tail starts inside the local entries while the
        // comment pushes the EOCD, and so the directory, into the first block.
        let entries: Vec<_> = (b'a'..=b'h').map(|name| stored(&[name], b"1")).collect();
        let bytes = archive(&entries, &[b'c'; 16_000]);
        let eocd_abs = 8 * 32 + 8 * 47;
        assert_eq!(bytes.len(), eocd_abs + EOCD_LEN + 16_000);
        assert!(bytes.len() > 16 * 1024 && eocd_abs < 4096);
        let (facts, names) = blocks(&bytes);
        assert_eq!(
            facts,
            ZipFacts {
                valid: true,
                flags: 0,
                entries: 8,
                names_entries: 8,
                names_len: 17,
                comment_len: 16_000,
                cd_size: 8 * 47,
                first_entry_len: 0,
            }
        );
        assert_eq!(view(&names, 17), b"\na\nb\nc\nd\ne\nf\ng\nh\n");
    }

    #[test]
    fn directory_larger_than_the_tail_window_is_not_held() {
        let entries: Vec<_> =
            (0..400).map(|i| stored(format!("{i:0>40}").as_bytes(), b"")).collect();
        let bytes = archive(&entries, b"");
        let (facts, names) = blocks(&bytes);
        assert_eq!(
            facts,
            ZipFacts {
                valid: false,
                flags: FLAG_CD_NOT_HELD,
                entries: 400,
                cd_size: 400 * 86,
                ..ZipFacts::default()
            }
        );
        assert_eq!(view(&names, 0), b"");
    }

    #[test]
    fn directory_size_beyond_the_file_is_malformed() {
        let mut bytes = archive(&[stored(b"a", b"1")], b"");
        let eocd = eocd_at(&bytes, 0);
        put_u32(&mut bytes, eocd + 12, 0xffff_0000);
        let (facts, names) = small(&bytes);
        assert_eq!(
            facts,
            ZipFacts {
                valid: false,
                flags: FLAG_MALFORMED,
                entries: 1,
                cd_size: 0xffff_0000,
                ..ZipFacts::default()
            }
        );
        assert_eq!(view(&names, 0), b"");
        // One byte too many is malformed as well; the exact size is fine.
        let mut bytes = archive(&[stored(b"a", b"1")], b"");
        put_u32(&mut bytes, eocd + 12, eocd as u32 + 1);
        assert_eq!(small(&bytes).0.flags, FLAG_MALFORMED);
        put_u32(&mut bytes, eocd + 12, eocd as u32);
        assert_eq!(small(&bytes).0.flags, FLAG_MALFORMED, "a directory spanning the entries");
    }

    #[test]
    fn directory_offset_overflow_does_not_disturb_the_walk() {
        let mut bytes = archive(&[stored(b"a", b"1")], b"");
        let eocd = eocd_at(&bytes, 0);
        put_u32(&mut bytes, eocd + 16, 0xffff_fffe);
        let (facts, names) = small(&bytes);
        assert_eq!(facts.flags, 0);
        assert!(facts.valid);
        assert_eq!(view(&names, 3), b"\na\n");
    }

    #[test]
    fn name_running_past_the_directory_end_is_malformed() {
        let mut bytes = archive(&[stored(b"first", b"1"), stored(b"second", b"2")], b"");
        let eocd = eocd_at(&bytes, 0);
        let cd = eocd - (2 * 46 + 5 + 6);
        put_u16(&mut bytes, cd + 46 + 5 + 28, 0xffff);
        let (facts, names) = small(&bytes);
        assert_eq!(
            facts,
            ZipFacts {
                valid: false,
                flags: FLAG_MALFORMED,
                entries: 2,
                names_entries: 1,
                names_len: 7,
                comment_len: 0,
                cd_size: 2 * 46 + 5 + 6,
                first_entry_len: 0,
            }
        );
        assert_eq!(view(&names, 7), b"\nfirst\n");
        // A bad signature stops the walk the same way.
        let mut bytes = archive(&[stored(b"first", b"1"), stored(b"second", b"2")], b"");
        bytes[cd + 46 + 5] = b'Q';
        let (facts, names) = small(&bytes);
        assert_eq!(facts.flags, FLAG_MALFORMED);
        assert_eq!(view(&names, 7), b"\nfirst\n");
        // An extra or comment length past the end drops that entry without writing it.
        let mut bytes = archive(&[stored(b"first", b"1"), stored(b"second", b"2")], b"");
        put_u16(&mut bytes, cd + 46 + 5 + 32, 1);
        let (facts, names) = small(&bytes);
        assert_eq!((facts.flags, facts.names_entries), (FLAG_MALFORMED, 1));
        assert_eq!(view(&names, 7), b"\nfirst\n");
    }

    #[test]
    fn names_are_sanitized_of_newlines_and_nuls() {
        let bytes = archive(&[stored(b"a\nb\0c", b""), stored(b"\n", b"")], b"");
        let (facts, names) = small(&bytes);
        assert!(facts.valid);
        assert_eq!(view(&names, 9), b"\na\x01b\x01c\n\x01\n");
    }

    #[test]
    fn every_entry_is_walked_and_truncation_only_flags_a_full_view() {
        let entries: Vec<_> = (0..100).map(|i| stored(format!("n{i}").as_bytes(), b"")).collect();
        let mut bytes = archive(&entries, b"");
        let (facts, names) = small(&bytes);
        assert!(facts.valid, "{facts:?}");
        assert_eq!((facts.flags, facts.entries, facts.names_entries), (0, 100, 100));
        let expected: Vec<u8> =
            (0..100).flat_map(|i| format!("\nn{i}").into_bytes()).chain([b'\n']).collect();
        assert_eq!(facts.names_len as usize, expected.len());
        assert_eq!(view(&names, expected.len()), expected);
        // A count beyond the headers present runs off the directory: malformed, names kept.
        let eocd = eocd_at(&bytes, 0);
        put_u16(&mut bytes, eocd + 8, 0xfffe);
        put_u16(&mut bytes, eocd + 10, 0xfffe);
        let (facts, names) = small(&bytes);
        assert!(!facts.valid);
        assert_eq!((facts.flags, facts.names_entries), (FLAG_MALFORMED, 100));
        assert_eq!(view(&names, expected.len()), expected);
        // Long names fill the view: whole names are dropped and the truncation flagged.
        let entries: Vec<_> =
            (0..100).map(|i| stored(format!("{i:0>60}").as_bytes(), b"")).collect();
        let bytes = archive(&entries, b"");
        let (facts, names) = small(&bytes);
        assert!(facts.valid);
        assert_eq!(facts.flags, FLAG_NAMES_TRUNCATED);
        assert_eq!(facts.names_entries, 67);
        assert_eq!(facts.names_len, 67 * 61 + 1);
        let expected: Vec<u8> =
            (0..67).flat_map(|i| format!("\n{i:0>60}").into_bytes()).chain([b'\n']).collect();
        assert_eq!(view(&names, expected.len()), expected);
        // A name filling the view exactly, terminator included, still fits.
        let name = vec![b'z'; ZIP_NAMES_BYTES - 2];
        let bytes = archive(&[stored(&name, b""), stored(b"q", b"")], b"");
        let (facts, names) = small(&bytes);
        assert_eq!((facts.flags, facts.names_entries), (FLAG_NAMES_TRUNCATED, 1));
        assert_eq!(facts.names_len as usize, ZIP_NAMES_BYTES);
        assert_eq!(names[0], b'\n');
        assert_eq!(names[ZIP_NAMES_BYTES - 1], b'\n');
        assert!(names[1..ZIP_NAMES_BYTES - 1].iter().all(|x| *x == b'z'));
    }

    #[test]
    fn top_level_names_are_written_before_nested_ones() {
        let nested: Vec<_> =
            (0..150).map(|i| format!("res/drawable/icon_{i:0>40}.png").into_bytes()).collect();
        let mut entries: Vec<_> = nested.iter().map(|name| stored(name, b"")).collect();
        entries.push(stored(b"classes.dex", b""));
        entries.push(stored(b"AndroidManifest.xml", b""));
        let (facts, names) = small(&archive(&entries, b""));
        assert!(facts.valid, "{facts:?}");
        assert_eq!(facts.flags, FLAG_NAMES_TRUNCATED);
        let text = view(&names, facts.names_len as usize);
        assert!(text.starts_with(b"\nclasses.dex\nAndroidManifest.xml\nres/drawable/icon_"));
    }

    #[test]
    fn directory_start_reaches_a_directory_beyond_the_tail_window() {
        let entries: Vec<_> =
            (0..400).map(|i| stored(format!("{i:0>60}").as_bytes(), b"")).collect();
        let bytes = archive(&entries, b"");
        let size = bytes.len() as u64;
        let eocd = eocd_at(&bytes, 0);
        let cd_size = u32::from_le_bytes(bytes[eocd + 12..eocd + 16].try_into().unwrap());
        let (facts, _) = blocks(&bytes);
        assert_eq!(facts.flags & FLAG_CD_NOT_HELD, FLAG_CD_NOT_HELD, "{facts:?}");
        let tail = &bytes[bytes.len() - 16 * 1024..];
        let start = directory_start(tail, size).unwrap();
        assert_eq!(start, eocd as u64 - u64::from(cd_size));
        let mut names = Box::new([0; ZIP_NAMES_BYTES]);
        let facts = analyze(&bytes[..4096], size, &bytes[start as usize..], &mut names);
        assert!(facts.valid, "{facts:?}");
        // A held directory needs no second read, nor does one beyond the cap.
        assert_eq!(directory_start(&bytes, size), None);
        let mut record = bytes[eocd..].to_vec();
        let far = 1_u64 << 30;
        put_u32(&mut record, 12, ZIP_DIRECTORY_MAX_BYTES);
        let expected = far - EOCD_LEN as u64 - u64::from(ZIP_DIRECTORY_MAX_BYTES);
        assert_eq!(directory_start(&record, far), Some(expected));
        put_u32(&mut record, 12, ZIP_DIRECTORY_MAX_BYTES + 1);
        assert_eq!(directory_start(&record, far), None);
    }

    fn deflated(name: &[u8], content: &[u8]) -> Entry {
        let data = miniz_oxide::deflate::compress_to_vec(content, 6);
        Entry { name: name.to_vec(), data, method: 8 }
    }

    /// The `zip_first_entry` view of an input, from its first 4096 bytes as the pipeline holds.
    fn first_view(bytes: &[u8]) -> (u16, u8, Box<[u8; ZIP_FIRST_ENTRY_BYTES]>) {
        let mut out = Box::new([0; ZIP_FIRST_ENTRY_BYTES]);
        let mut inflater = DecompressorOxide::new();
        let (len, flags) = first_entry(&bytes[..bytes.len().min(4096)], &mut out, &mut inflater);
        (len, flags, out)
    }

    #[test]
    fn first_entry_view_holds_the_name_and_the_inflated_data() {
        let content = br#"<Types><Override ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.template.main+xml"/></Types>"#;
        let entries =
            [deflated(b"[Content_Types].xml", content), stored(b"word/document.xml", b"")];
        let (len, flags, out) = first_view(&archive(&entries, b""));
        let expected = [b"[Content_Types].xml\n".as_slice(), content].concat();
        assert_eq!((usize::from(len), flags), (expected.len(), 0));
        assert_eq!(&out[..expected.len()], expected);
        assert!(out[expected.len()..].iter().all(|x| *x == 0));
        let (len, flags, out) = first_view(&archive(&[stored(b"mimetype", b"text")], b""));
        assert_eq!((&out[..usize::from(len)], flags), (b"mimetype\ntext".as_slice(), 0));
    }

    #[test]
    fn first_entry_view_only_holds_what_it_reads_exactly() {
        let content = b"hello hello hello hello";
        let good = archive(&[deflated(b"a\nb", content)], b"");
        let (len, flags, out) = first_view(&good);
        let expected = [b"a\x01b\n".as_slice(), content].concat();
        assert_eq!((&out[..usize::from(len)], flags), (expected.as_slice(), 0));
        // Encrypted, data-descriptor and deflate64 entries, or data beyond the block: nothing.
        for (at, value) in [(6, 1_u16), (6, 8), (8, 9)] {
            let mut bytes = good.clone();
            put_u16(&mut bytes, at, value);
            assert_eq!(first_view(&bytes).0, 0, "field {at} = {value}");
        }
        let mut long = good.clone();
        put_u32(&mut long, 18, 5000);
        assert_eq!(first_view(&long).0, 0);
        // A reserved block type keeps only the name line.
        let mut corrupt = good.clone();
        corrupt[30 + 3] = 0xff;
        let (len, flags, out) = first_view(&corrupt);
        assert_eq!((len, flags), (4, FLAG_FIRST_ENTRY_UNDECODABLE));
        assert!(out[4..].iter().all(|x| *x == 0));
        // Compressed bytes ending before the final block are undecodable as well.
        let mut cut = good.clone();
        let compressed = u32::from_le_bytes(cut[18..22].try_into().unwrap());
        put_u32(&mut cut, 18, compressed / 2);
        assert_eq!(first_view(&cut).1, FLAG_FIRST_ENTRY_UNDECODABLE);
        // Data reaching the end of the view fills it.
        let big = archive(&[deflated(b"big", &vec![b'x'; 2 * ZIP_FIRST_ENTRY_BYTES])], b"");
        let (len, flags, _) = first_view(&big);
        assert_eq!((usize::from(len), flags), (ZIP_FIRST_ENTRY_BYTES, FLAG_FIRST_ENTRY_FILLED));
    }

    #[test]
    fn zip64_sentinels_and_locator_report_a_plain_invalid_zip() {
        let base = archive(&[stored(b"a", b"1")], b"");
        let eocd = eocd_at(&base, 0);
        let expected = |entries| ZipFacts {
            valid: false,
            flags: FLAG_ZIP64,
            entries,
            cd_size: 0,
            ..ZipFacts::default()
        };
        let mut total = base.clone();
        put_u16(&mut total, eocd + 8, 0xffff);
        put_u16(&mut total, eocd + 10, 0xffff);
        assert_eq!(small(&total), (expected(0xffff), Box::new([0; ZIP_NAMES_BYTES])));
        let mut size = base.clone();
        put_u32(&mut size, eocd + 12, 0xffff_ffff);
        assert_eq!(small(&size).0, expected(1));
        let mut offset = base.clone();
        put_u32(&mut offset, eocd + 16, 0xffff_ffff);
        assert_eq!(small(&offset).0, expected(1));
        let mut disk = base.clone();
        put_u16(&mut disk, eocd + 4, 0xffff);
        assert_eq!(small(&disk).0.flags & FLAG_ZIP64, FLAG_ZIP64);
        // A zip64 locator immediately before a plain EOCD marks zip64 as well.
        let mut located = base[..eocd].to_vec();
        located.extend_from_slice(b"PK\x06\x07");
        located.extend_from_slice(&[0; 16]);
        located.extend_from_slice(&base[eocd..]);
        assert_eq!(small(&located).0, expected(1));
    }

    #[test]
    fn multi_disk_archives_are_invalid() {
        let base = archive(&[stored(b"a", b"1")], b"");
        let eocd = eocd_at(&base, 0);
        let expected =
            ZipFacts { valid: false, flags: FLAG_MULTIDISK, entries: 1, ..ZipFacts::default() };
        let mut disk = base.clone();
        put_u16(&mut disk, eocd + 4, 1);
        assert_eq!(small(&disk), (expected, Box::new([0; ZIP_NAMES_BYTES])));
        let mut cd_disk = base.clone();
        put_u16(&mut cd_disk, eocd + 6, 1);
        assert_eq!(small(&cd_disk).0, expected);
        let mut split = base.clone();
        put_u16(&mut split, eocd + 8, 0);
        assert_eq!(small(&split).0, expected);
    }

    #[test]
    fn prepended_data_is_flagged_and_still_walked() {
        let mut bytes = vec![b'M'; 100];
        bytes.extend(archive(&[stored(b"payload.bin", b"xyz")], b""));
        let (facts, names) = small(&bytes);
        assert_eq!(
            facts,
            ZipFacts {
                valid: true,
                flags: FLAG_PREPENDED,
                entries: 1,
                names_entries: 1,
                names_len: 13,
                comment_len: 0,
                cd_size: 46 + 11,
                first_entry_len: 0,
            }
        );
        assert_eq!(view(&names, 13), b"\npayload.bin\n");
        // The same archive behind a large stub, with only the bounded blocks held.
        let mut bytes = vec![b'M'; 100_000];
        bytes.extend(archive(&[stored(b"payload.bin", b"xyz")], b""));
        let (facts, names) = blocks(&bytes);
        assert_eq!((facts.valid, facts.flags), (true, FLAG_PREPENDED));
        assert_eq!(view(&names, 13), b"\npayload.bin\n");
    }

    #[test]
    fn short_or_foreign_tails_yield_nothing() {
        let bytes = archive(&[stored(b"a", b"1")], b"");
        for tail in [&b""[..], &b"PK\x05\x06"[..], &bytes[bytes.len() - 21..]] {
            let mut names = Box::new([0; ZIP_NAMES_BYTES]);
            let facts = analyze(&bytes, bytes.len() as u64, tail, &mut names);
            assert_eq!(facts, ZipFacts::default(), "{tail:?}");
            assert_eq!(view(&names, 0), b"");
        }
        // A tail longer than the file cannot be placed and yields nothing.
        let mut names = Box::new([0; ZIP_NAMES_BYTES]);
        assert_eq!(analyze(&bytes, 10, &bytes, &mut names), ZipFacts::default());
        assert_eq!(small(b"PK\x03\x04 not an archive").0, ZipFacts::default());
    }

    #[test]
    fn every_byte_mutation_of_the_fixtures_is_survived() {
        for name in ["simple.zip", "zip64.zip", "volumecomment.zip", "filecomment.zip"] {
            let path = format!("../../tests_data/mitra/zip/{name}");
            let original = std::fs::read(&path).unwrap();
            let (facts, _) = blocks(&original);
            assert!(facts.entries > 0, "{name}: {facts:?}");
            for at in 0..original.len() {
                for value in [0x00, 0xff, 0x50, original[at] ^ 0x01, original[at] ^ 0x80] {
                    let mut mutated = original.clone();
                    mutated[at] = value;
                    let (facts, names) = blocks(&mutated);
                    check(&facts, &names);
                    let (facts, names) = small(&mutated);
                    check(&facts, &names);
                    // Truncations and the two-block split are reachable states as well.
                    let mut names = Box::new([0; ZIP_NAMES_BYTES]);
                    let facts =
                        analyze(&mutated[..at], mutated.len() as u64, &mutated[at..], &mut names);
                    check(&facts, &names);
                }
            }
        }
    }
}
