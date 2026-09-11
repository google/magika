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
//!    from the total. Either bit ends the directory analysis: `cd_size` and the `names_*`
//!    facts stay zero and no name is written.
//! 4. **Directory placement.** `cd_size` is reported. The directory is anchored at the
//!    EOCD, not at the declared offset: it is the `cd_size` bytes ending at the EOCD's
//!    absolute offset `eocd_abs = size - tail.len() + position`, so it starts at
//!    `eocd_abs - cd_size`. This keeps self-extractors walkable, and it is deliberate: any
//!    bytes between the directory's end and the EOCD (a `PK\x05\x05` digital signature
//!    record, say) shift the anchor into the directory, so such an archive reads as
//!    malformed and prepended. A reimplementation must mirror this. Bit 5 (prepended
//!    data) is set when the declared offset (u32 at 16) plus `cd_size` is smaller than
//!    `eocd_abs`. A `cd_size` beyond `eocd_abs` sets bit 3 (malformed) and ends the
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
//! 7. `valid` is set iff an EOCD was found and bits 0 to 3 are all clear.
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

use miniz_oxide::inflate::core::inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
use miniz_oxide::inflate::core::{decompress, DecompressorOxide};
use miniz_oxide::inflate::TINFLStatus;

use super::{u16_at, u32_at, ZIP_FIRST_ENTRY_BYTES, ZIP_NAMES_BYTES};

/// Largest central directory read on its own when the tail window does not hold it.
pub(super) const ZIP_DIRECTORY_MAX_BYTES: u32 = 256 * 1024;

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
const FLAG_FIRST_ENTRY_FILLED: u8 = 1 << 6;
/// The first entry is deflated but its data does not decode completely.
const FLAG_FIRST_ENTRY_UNDECODABLE: u8 = 1 << 7;

const LOCAL_SIGNATURE: &[u8] = b"PK\x03\x04";
const CENTRAL_SIGNATURE: &[u8] = b"PK\x01\x02";
const EOCD_SIGNATURE: &[u8] = b"PK\x05\x06";
const ZIP64_LOCATOR_SIGNATURE: &[u8] = b"PK\x06\x07";
const LOCAL_HEADER_LEN: usize = 30;
const CENTRAL_HEADER_LEN: usize = 46;
const EOCD_LEN: usize = 22;
const ZIP64_LOCATOR_LEN: usize = 20;
/// Longest stored `mimetype` payload copied into the view.
const MIMETYPE_MAX_LEN: u32 = 128;

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

/// Analyzes the archive held in `first` (the block at offset 0) and `tail` (the block ending
/// at `size`, the input length), writing the `zip_names` view into `names_out`. Never panics
/// and never reads outside the two blocks; see the module documentation for the contract.
pub(super) fn analyze(
    first: &[u8], size: u64, tail: &[u8], names_out: &mut [u8; ZIP_NAMES_BYTES],
) -> ZipFacts {
    let mut view = ViewWriter { out: names_out, len: 0 };
    let mut facts = ZipFacts::default();
    if let Some(mimetype) = stored_mimetype(first) {
        // The line always fits: it is at most 1 + 9 + 128 bytes into an empty view.
        view.line(b"mimetype=", mimetype);
    }
    if let Some(eocd) = find_eocd(tail) {
        walk_directory(first, size, tail, eocd, &mut view, &mut facts);
    }
    facts.names_len = view.finish();
    facts
}

/// Appends `\n`-led lines to the view and terminates the sequence once at the end.
struct ViewWriter<'a> {
    out: &'a mut [u8; ZIP_NAMES_BYTES],
    len: usize,
}

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
fn stored_mimetype(first: &[u8]) -> Option<&[u8]> {
    if !first.starts_with(LOCAL_SIGNATURE) || u16_at(first, 8)? != 0 || u16_at(first, 26)? != 8 {
        return None;
    }
    let length = u32_at(first, 22)?;
    if !(1..=MIMETYPE_MAX_LEN).contains(&length) {
        return None;
    }
    let extra_len = usize::from(u16_at(first, 28)?);
    let name_end = LOCAL_HEADER_LEN + 8;
    if first.get(LOCAL_HEADER_LEN..name_end)? != b"mimetype" {
        return None;
    }
    let data_start = name_end.checked_add(extra_len)?;
    first.get(data_start..data_start.checked_add(length as usize)?)
}

/// Position in `tail` of the end-of-central-directory record whose comment reaches the end.
fn find_eocd(tail: &[u8]) -> Option<usize> {
    let last = tail.len().checked_sub(EOCD_LEN)?;
    (0..=last).rev().find(|&at| {
        tail[at..].starts_with(EOCD_SIGNATURE)
            && u16_at(tail, at + 20)
                .is_some_and(|comment_len| usize::from(comment_len) == tail.len() - at - EOCD_LEN)
    })
}

/// Decodes the EOCD at `eocd` in `tail`, places the directory and walks its names.
fn walk_directory(
    first: &[u8], size: u64, tail: &[u8], eocd: usize, view: &mut ViewWriter<'_>,
    facts: &mut ZipFacts,
) {
    let Some(tail_start) = size.checked_sub(tail.len() as u64) else { return };
    let record = Eocd::decode(tail, eocd);
    facts.entries = record.entries;
    facts.comment_len = record.comment_len;
    if record.flags != 0 {
        facts.flags |= record.flags;
        return;
    }
    facts.cd_size = record.cd_size;
    let eocd_abs = tail_start + eocd as u64;
    if u64::from(record.cd_offset) + u64::from(record.cd_size) < eocd_abs {
        facts.flags |= FLAG_PREPENDED;
    }
    let Some(cd_abs) = eocd_abs.checked_sub(u64::from(record.cd_size)) else {
        facts.flags |= FLAG_MALFORMED;
        return;
    };
    // The directory ends at the EOCD, so it is inside `tail` iff it starts there; `first`
    // starts at 0, so it is inside `first` iff it ends there.
    let directory = if cd_abs >= tail_start {
        &tail[(cd_abs - tail_start) as usize..eocd]
    } else if eocd_abs <= first.len() as u64 {
        // Reached with production blocks only when a long archive comment keeps the EOCD
        // inside `first` while the 16 KiB tail starts after the directory.
        &first[cd_abs as usize..eocd_abs as usize]
    } else {
        facts.flags |= FLAG_CD_NOT_HELD;
        return;
    };
    // Top-level names first, then nested ones: both passes walk the same headers.
    'passes: for nested in [false, true] {
        let mut at = 0;
        for _ in 0..record.entries {
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
    facts.valid =
        facts.flags & (FLAG_ZIP64 | FLAG_MULTIDISK | FLAG_CD_NOT_HELD | FLAG_MALFORMED) == 0;
}

/// The name of the central directory header at `at` and the length of the whole entry,
/// name, extra field and comment included, when all of it lies inside the directory.
fn entry_name(directory: &[u8], at: usize) -> Option<(&[u8], usize)> {
    let header = directory.get(at..at.checked_add(CENTRAL_HEADER_LEN)?)?;
    if !header.starts_with(CENTRAL_SIGNATURE) {
        return None;
    }
    let name_len = usize::from(u16_at(header, 28)?);
    let trailer = usize::from(u16_at(header, 30)?) + usize::from(u16_at(header, 32)?);
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
    fn decode(tail: &[u8], eocd: usize) -> Self {
        let record = &tail[eocd..eocd + EOCD_LEN];
        let field = |at| u16_at(record, at).unwrap_or(0);
        let (disk, cd_disk, entries_disk, entries) = (field(4), field(6), field(8), field(10));
        let cd_size = u32_at(record, 12).unwrap_or(0);
        let cd_offset = u32_at(record, 16).unwrap_or(0);
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
        Self { entries, comment_len: field(20), cd_size, cd_offset, flags }
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
    if record.flags != 0 || record.cd_size > ZIP_DIRECTORY_MAX_BYTES {
        return None;
    }
    let start = (tail_start + eocd as u64).checked_sub(u64::from(record.cd_size))?;
    (start < tail_start).then_some(start)
}

/// Writes the `zip_first_entry` view of step 8 and returns its length and flag bits.
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

/// The name, the compressed data and whether it is deflated, for a first local entry that
/// step 8 accepts.
fn first_entry_parts(first: &[u8]) -> Option<(&[u8], &[u8], bool)> {
    if !first.starts_with(LOCAL_SIGNATURE) {
        return None;
    }
    let flags = u16_at(first, 6)?;
    let method = u16_at(first, 8)?;
    if flags & 0b1001 != 0 || !(method == 0 || method == 8) {
        return None;
    }
    let compressed = usize::try_from(u32_at(first, 18)?).ok()?;
    let name_len = usize::from(u16_at(first, 26)?);
    let extra_len = usize::from(u16_at(first, 28)?);
    if name_len + 1 >= ZIP_FIRST_ENTRY_BYTES {
        return None;
    }
    let name = first.get(LOCAL_HEADER_LEN..LOCAL_HEADER_LEN + name_len)?;
    let data_start = LOCAL_HEADER_LEN + name_len + extra_len;
    let data = first.get(data_start..data_start.checked_add(compressed)?)?;
    Some((name, data, method == 8))
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
        (facts, names)
    }

    /// Analyzes with the bounded blocks the pipeline holds: 4096 bytes at 0, 16 KiB at the end.
    fn blocks(bytes: &[u8]) -> (ZipFacts, Box<[u8; ZIP_NAMES_BYTES]>) {
        let mut names = Box::new([0; ZIP_NAMES_BYTES]);
        let first = &bytes[..bytes.len().min(4096)];
        let tail = &bytes[bytes.len().saturating_sub(16 * 1024)..];
        let facts = analyze(first, bytes.len() as u64, tail, &mut names);
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
            assert_eq!(facts.flags & 0b1111, 0, "{facts:?}");
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
        let bytes = archive(&[stored(b"x", b"1")], b"a comment");
        let (facts, names) = small(&bytes);
        assert!(facts.valid);
        assert_eq!(facts.comment_len, 9);
        assert_eq!(facts.entries, 1);
        assert_eq!(view(&names, 3), b"\nx\n");
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
