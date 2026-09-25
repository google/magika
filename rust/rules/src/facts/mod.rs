// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Facts about archives and executables, which rules read by name.
//!
//! A zip archive is recognized by its central directory, at the end of the file, so its facts
//! need the input's tail as well as its prefix: see [`crate::RuleSet::tail_len`]. An executable
//! is recognized from its headers in the prefix alone. Both analyses are pure functions of the
//! prefix, the size and the tail, specified in `zip.rs` and `pe.rs`, and run only for an input
//! that opens like an archive or an executable, and only when a rule reads their facts.

mod pe;
mod zip;

use miniz_oxide::inflate::core::DecompressorOxide;

#[cfg(test)]
pub(crate) use pe::tests::{pe32, Image};
#[cfg(test)]
pub(crate) use zip::tests::{archive, stored};

use crate::ir::{Fact, View};

/// Size of the `zip_names` view.
const ZIP_NAMES_BYTES: usize = 4096;
/// Size of the `zip_first_entry` view.
const ZIP_FIRST_ENTRY_BYTES: usize = 16 * 1024;
/// Size of the trailing window that holds the central directory of most archives.
pub(crate) const ZIP_TAIL_BYTES: usize = 16 * 1024;

/// Whether the prefix opens a zip archive (a local file header, or the end record of an empty
/// one), the only inputs whose tail the facts read.
pub(crate) fn wants_tail(prefix: &[u8]) -> bool {
    prefix.starts_with(b"PK\x03\x04") || prefix.starts_with(b"PK\x05\x06")
}

/// Where the central directory starts, when `tail` (the last bytes of an input of `size` bytes)
/// holds an end record whose directory starts before it and is small enough to read.
pub(crate) fn directory_start(tail: &[u8], size: u64) -> Option<u64> {
    zip::directory_start(tail, size)
}

/// The facts and views of one input.
pub(crate) struct Facts {
    zip: zip::ZipFacts,
    pe: pe::PeFacts,
    names: Vec<u8>,
    first_entry: Vec<u8>,
}

impl Facts {
    /// Analyzes an input. `tail` holds its last bytes, and may be the prefix itself when it holds
    /// the whole input; without one, a zip archive's directory is unknown.
    pub(crate) fn analyze(prefix: &[u8], size: u64, tail: Option<&[u8]>) -> Facts {
        let mut facts = Facts {
            zip: zip::ZipFacts::default(),
            pe: pe::PeFacts::default(),
            names: Vec::new(),
            first_entry: Vec::new(),
        };
        if wants_tail(prefix) {
            let whole = size == prefix.len() as u64;
            let tail = tail.unwrap_or(if whole { prefix } else { &[] });
            let mut names = Box::new([0; ZIP_NAMES_BYTES]);
            facts.zip = zip::analyze(prefix, size, tail, &mut names);
            let mut first_entry = Box::new([0; ZIP_FIRST_ENTRY_BYTES]);
            let mut inflater = Box::<DecompressorOxide>::default();
            let (len, flags) = zip::first_entry(prefix, &mut first_entry, &mut inflater);
            facts.zip.first_entry_len = len;
            facts.zip.flags |= flags;
            facts.names = names[..usize::from(facts.zip.names_len)].to_vec();
            facts.first_entry = first_entry[..usize::from(len)].to_vec();
        } else if pe::wants(prefix) {
            facts.pe = pe::analyze(prefix, size);
        }
        facts
    }

    pub(crate) fn int(&self, fact: Fact) -> u64 {
        let (zip, pe) = (&self.zip, &self.pe);
        match fact {
            Fact::ZipValid => u64::from(zip.valid),
            Fact::ZipFlags => u64::from(zip.flags),
            Fact::ZipEntries => u64::from(zip.entries),
            Fact::ZipNamesEntries => u64::from(zip.names_entries),
            Fact::ZipNamesLen => u64::from(zip.names_len),
            Fact::ZipCommentLen => u64::from(zip.comment_len),
            Fact::ZipCdSize => u64::from(zip.cd_size),
            Fact::ZipFirstEntryLen => u64::from(zip.first_entry_len),
            Fact::PeValid => u64::from(pe.valid),
            Fact::PeFlags => u64::from(pe.flags),
            Fact::PeMachine => u64::from(pe.machine),
            Fact::PeCharacteristics => u64::from(pe.characteristics),
            Fact::PeSubsystem => u64::from(pe.subsystem),
            Fact::PeEntryPoint => u64::from(pe.entry_point),
            Fact::PeDllCharacteristics => u64::from(pe.dll_characteristics),
            Fact::PeSections => u64::from(pe.sections),
            Fact::PeMagic => u64::from(pe.magic),
            Fact::PeClr => u64::from(pe.clr),
            Fact::PeSigned => u64::from(pe.signed),
            Fact::PeOverlay => pe.overlay,
            Fact::PeIsDll => u64::from(pe.is_dll),
            Fact::PeIsExecutableImage => u64::from(pe.is_executable_image),
        }
    }

    pub(crate) fn view(&self, view: View) -> &[u8] {
        match view {
            View::ZipNames => &self.names,
            View::ZipFirstEntry => &self.first_entry,
        }
    }
}

/// The little-endian u16 at `at`. A field not lying entirely inside `bytes` (or an `at` that
/// overflows) reads as `None`, never panics.
fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(at..at.checked_add(2)?)?.try_into().ok()?))
}

/// The little-endian u32 at `at`; see [`u16_at`].
fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at.checked_add(4)?)?.try_into().ok()?))
}

/// The little-endian u64 at `at`; see [`u16_at`].
fn u64_at(bytes: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(bytes.get(at..at.checked_add(8)?)?.try_into().ok()?))
}
