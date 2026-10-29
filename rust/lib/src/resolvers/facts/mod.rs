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

//! Facts about archives and executables, which verifiers read through [`Facts`].
//!
//! A zip archive is recognized by its central directory, at the end of the file, so its facts
//! need the input's tail as well as its prefix: see [`super::identify`]. An executable
//! is recognized from its headers in the prefix alone. Both analyses are pure functions of the
//! prefix, the size and the tail, specified in `zip.rs` and `pe.rs`, and run only for an input
//! that opens like an archive or an executable, and only when a verifier reads their facts.

mod pe;
mod zip;

use miniz_oxide::inflate::core::DecompressorOxide;

#[cfg(test)]
pub(crate) use self::pe::tests::{Image, pe32};
#[cfg(test)]
pub(crate) use self::zip::tests::{archive, stored};

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

    pub(crate) fn is_zip_valid(&self) -> bool {
        self.zip.valid
    }

    pub(crate) fn has_zip_dir_names(&self) -> bool {
        self.zip.valid && self.zip.entries >= 1 && self.zip.names_entries >= 1
    }

    pub(crate) fn names(&self) -> &[u8] {
        &self.names
    }

    pub(crate) fn first_entry(&self) -> &[u8] {
        &self.first_entry
    }

    pub(crate) fn is_windows_pebin(&self) -> bool {
        let p = &self.pe;
        p.valid
            && p.is_executable_image
            && !(p.is_dll && p.entry_point == 0)
            && (1..=16).contains(&p.subsystem)
            && matches!(
                p.machine,
                0x014C | 0x01C0 | 0x01C2 | 0x01C4 | 0x0200 | 0x5032 | 0x5064 | 0x8664 | 0xAA64
            )
    }

    #[cfg(test)]
    pub(crate) fn is_valid(&self) -> bool {
        self.zip.valid || self.pe.valid
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
