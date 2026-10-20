// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Facts about archives and executables, which verifiers read through [`Facts`].
//!
//! A zip archive is recognized by its central directory, at the end of the file, so its facts
//! need the input's tail as well as its prefix: see [`crate::RuleSet::tail_len`]. An executable
//! is recognized from its headers in the prefix alone. Both analyses are pure functions of the
//! prefix, the size and the tail, specified in `zip.rs` and `pe.rs`, and run only for an input
//! that opens like an archive or an executable, and only when a verifier reads their facts.

mod pe;
mod zip;

pub(crate) use self::pe::analyze_header as analyze_pe_header;
// Fixtures for the bundled rules tests in `lib.rs`.
#[cfg(all(test, feature = "bundled"))]
pub(crate) use self::pe::tests::{pe32, Image};
#[cfg(all(test, feature = "bundled"))]
pub(crate) use self::zip::tests::{archive, stored};
pub(crate) use self::zip::{
    classify_ooxml_first_entry, OoxmlKind, MIMETYPE_EPUB, MIMETYPE_ODP, MIMETYPE_ODS, MIMETYPE_ODT,
    SEEN_ANDROID_MANIFEST, SEEN_APPX_BLOCKMAP, SEEN_APPX_MANIFEST, SEEN_CLASSES_DEX,
    SEEN_CONTAINER_XML, SEEN_CONTENT_TYPES, SEEN_CONTENT_XML, SEEN_DOC_KML, SEEN_MANIFEST_MF,
    SEEN_METADATA_JSON, SEEN_MODEL_WEIGHTS, SEEN_PPT_PRESENTATION, SEEN_RESOURCES_ARSC,
    SEEN_SUFFIX_CLASS, SEEN_SUFFIX_QGS, SEEN_THREE_MF, SEEN_VISIO_DOC, SEEN_XL_WORKBOOK,
};

/// Size of the `zip_names` view.
const ZIP_NAMES_BYTES: usize = 4096;
/// Size of the `zip_first_entry` view.
const ZIP_FIRST_ENTRY_BYTES: usize = 16 * 1024;
/// Size of the trailing window that holds the central directory of most archives.
pub(crate) const ZIP_TAIL_BYTES: usize = 16 * 1024;

/// Whether the prefix opens a zip archive (a local file header, or the end record of an empty
/// one), the only inputs whose tail the facts read.
#[inline]
pub(crate) fn wants_tail(prefix: &[u8]) -> bool {
    matches!(prefix.first_chunk::<4>(), Some(b"PK\x03\x04" | b"PK\x05\x06"))
}

/// Where the central directory starts, when `tail` (the last bytes of an input of `size` bytes)
/// holds an end record whose directory starts before it and is small enough to read.
pub(crate) fn directory_start(tail: &[u8], size: u64) -> Option<u64> {
    zip::directory_start(tail, size)
}

/// The ZIP central-directory facts of one input.
#[derive(Clone, Copy)]
pub(crate) struct Facts {
    zip: zip::ZipFacts,
    zip_seen: u32,
}

impl Facts {
    /// Analyzes a ZIP input. `tail` holds its last bytes, and may be the prefix itself when it
    /// holds the whole input; without one, a zip archive's directory is unknown.
    #[inline]
    pub(crate) fn analyze(prefix: &[u8], size: u64, tail: Option<&[u8]>) -> Facts {
        let whole = size == prefix.len() as u64;
        let tail = tail.unwrap_or(if whole { prefix } else { &[] });
        let (zip, zip_seen) = zip::analyze_facts(prefix, size, tail);
        Facts { zip, zip_seen }
    }

    #[inline]
    pub(crate) fn zip_seen(&self) -> u32 {
        self.zip_seen
    }

    #[inline]
    pub(crate) fn is_valid(&self) -> bool {
        self.zip.valid
    }

    #[inline]
    pub(crate) fn has_dir_names(&self) -> bool {
        self.zip.valid && self.zip.entries >= 1 && self.zip.names_entries >= 1
    }
}

/// The little-endian u16 at `at`. A field not lying entirely inside `bytes` (or an `at` that
/// overflows) reads as `None`, never panics.
#[inline]
fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(at..at.checked_add(2)?)?.try_into().ok()?))
}

/// The little-endian u32 at `at`; see [`u16_at`].
#[inline]
fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at.checked_add(4)?)?.try_into().ok()?))
}

/// The little-endian u64 at `at`; see [`u16_at`].
#[inline]
fn u64_at(bytes: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(bytes.get(at..at.checked_add(8)?)?.try_into().ok()?))
}
