// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Bounded PE analysis: the COFF header, the optional header's identifying fields, two data
//! directories and the section table, derived from the first block only.
//!
//! Everything here is a pure function of `(first block, input size)`, so an independent
//! implementation can reproduce the facts byte for byte. No other block is read: a PE
//! costs no read beyond the prefix every input pays. All integers are little-endian; a
//! field that does not lie entirely inside `first` reads as zero. The contract, in
//! evaluation order:
//!
//! 1. **Location.** `first` must start with `MZ`, `e_lfanew` (u32 at 0x3c) must be at
//!    least 4, `e_lfanew + 24` must not exceed `first.len()`, and `PE\0\0` must be at
//!    `e_lfanew`. Otherwise every fact is zero, `flags` included: a plain DOS executable
//!    is not a PE, and neither is one whose headers the first block does not hold.
//! 2. **COFF header** at `e_lfanew + 4`: `machine` (u16 at 0), `sections` (2), the
//!    optional header size `soh` (16) and `characteristics` (18). `is_dll` is
//!    `characteristics` bit 0x2000 and `is_executable_image` bit 0x0002.
//! 3. **Optional header** at `opt = e_lfanew + 24`: `magic` (u16 at 0), `subsystem`
//!    (68) and `dll_characteristics` (70). A `magic` other than 0x10b (PE32) or 0x20b
//!    (PE32+) sets bit 2 (unknown optional magic) and skips step 4.
//! 4. **Data directories.** The count is the u32 at `opt + 92` (PE32) or `opt + 108`
//!    (PE32+) and entry `i` is the `(u32 rva, u32 size)` pair at `opt + 96 + 8 * i` or
//!    `opt + 112 + 8 * i`. Entry `i` is read only when `i < count`, `i < 16`, it lies
//!    inside the declared optional header (`96 + 8 * (i + 1) <= soh` or `112 + ...`)
//!    and inside `first`; an unread entry is all zero. Entry 4 is the certificate table,
//!    whose `rva` is a file offset: `signed` is `rva != 0 && size != 0`, but when
//!    `size != 0` and `rva + size` (u64) exceeds `size` of the input, bit 1 (certificate
//!    table out of file) is set and `signed` cleared. Entry 14 is the CLR runtime header:
//!    `clr` is `rva != 0 && size != 0`, the same rule.
//! 5. **Section table** at `opt + soh`, 40 bytes per section for `sections` sections. If
//!    `opt + soh + 40 * sections > first.len()`, bit 0 (section table not held) is set
//!    and nothing is walked. Otherwise `raw_end` starts at the table's end
//!    `opt + soh + 40 * sections`, so that the headers are never overlay, and every
//!    header's extent `pointer_to_raw_data (u32 at 20) + size_of_raw_data (16)` is
//!    computed in u64: an extent beyond the input size sets bit 3 (section beyond file)
//!    and contributes nothing; each of the others raises `raw_end` to at least itself.
//! 6. **`valid`** is set iff `magic` is known, `soh` is at least 96 (PE32) or 112
//!    (PE32+), `sections` is 1..=96, bit 0 is clear and `opt + soh + 40 * sections` does
//!    not exceed the input size.
//! 7. **`overlay`** is `size - raw_end` when `valid`, and zero otherwise. A valid image
//!    whose sections carry no raw data thus reports the bytes after its headers, as
//!    pefile and LIEF do.
//!
//! | `flags` bit | meaning                                                  |
//! |-------------|----------------------------------------------------------|
//! | 0           | section table not held in the first block                |
//! | 1           | certificate table extends beyond the file (not `signed`) |
//! | 2           | unknown optional header magic                            |
//! | 3           | a section's raw data extends beyond the file             |

use super::{u16_at, u32_at};

/// Most sections a valid image declares; a real image has a handful.
const PE_MAX_SECTIONS: u16 = 96;

/// Position of `e_lfanew`, the offset of the PE signature, in the DOS header.
const E_LFANEW_AT: usize = 0x3c;
const PE_SIGNATURE: &[u8] = b"PE\0\0";
const COFF_HEADER_LEN: usize = 20;
const SECTION_HEADER_LEN: usize = 40;
/// Optional header magic of a PE32 image, and the length of its fixed fields.
const PE32_MAGIC: u16 = 0x10b;
const PE32_FIXED_LEN: u64 = 96;
/// Optional header magic of a PE32+ image, and the length of its fixed fields.
const PE32PLUS_MAGIC: u16 = 0x20b;
const PE32PLUS_FIXED_LEN: u64 = 112;
/// Most data directory entries an optional header declares.
const DIRECTORY_ENTRIES: u64 = 16;
/// The data directory entries holding the certificate table and the CLR runtime header.
const DIRECTORY_CERTIFICATE_TABLE: u64 = 4;
const DIRECTORY_CLR_RUNTIME_HEADER: u64 = 14;
/// COFF characteristics: the image is a DLL, and the image is executable (fully linked).
const CHARACTERISTIC_EXECUTABLE_IMAGE: u16 = 0x0002;
const CHARACTERISTIC_DLL: u16 = 0x2000;

/// The section table does not lie inside the first block.
const FLAG_SECTIONS_NOT_HELD: u8 = 1 << 0;
/// The certificate table extends beyond the file.
const FLAG_CERTIFICATE_OUT_OF_FILE: u8 = 1 << 1;
/// The optional header magic is neither PE32 nor PE32+.
const FLAG_UNKNOWN_MAGIC: u8 = 1 << 2;
/// At least one section's raw data extends beyond the file.
const FLAG_SECTION_BEYOND_FILE: u8 = 1 << 3;

/// The PE facts of one input, all zero when the input holds no PE headers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct PeFacts {
    pub valid: bool,
    pub flags: u8,
    pub machine: u16,
    pub characteristics: u16,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub sections: u16,
    pub magic: u16,
    pub clr: bool,
    pub signed: bool,
    pub overlay: u64,
    pub is_dll: bool,
    pub is_executable_image: bool,
}

/// Whether the prefix opens a DOS or PE executable, the only inputs analyzed.
pub(super) fn wants(prefix: &[u8]) -> bool {
    prefix.starts_with(b"MZ")
}

/// Analyzes the headers held in `first` (the block at offset 0) of an input of `size`
/// bytes. Never panics; see the module documentation for the contract.
pub(super) fn analyze(first: &[u8], size: u64) -> PeFacts {
    let mut facts = PeFacts::default();
    let Some(coff) = locate(first) else { return facts };
    facts.machine = u16_at(first, coff).unwrap_or(0);
    facts.sections = u16_at(first, coff + 2).unwrap_or(0);
    let soh = u64::from(u16_at(first, coff + 16).unwrap_or(0));
    facts.characteristics = u16_at(first, coff + 18).unwrap_or(0);
    facts.is_dll = facts.characteristics & CHARACTERISTIC_DLL != 0;
    facts.is_executable_image = facts.characteristics & CHARACTERISTIC_EXECUTABLE_IMAGE != 0;
    let opt = coff + COFF_HEADER_LEN;
    facts.magic = u16_at(first, opt).unwrap_or(0);
    facts.subsystem = u16_at(first, opt + 68).unwrap_or(0);
    facts.dll_characteristics = u16_at(first, opt + 70).unwrap_or(0);
    let fixed = match facts.magic {
        PE32_MAGIC => Some(PE32_FIXED_LEN),
        PE32PLUS_MAGIC => Some(PE32PLUS_FIXED_LEN),
        _ => None,
    };
    match fixed {
        Some(fixed) => read_directories(first, size, opt, soh, fixed, &mut facts),
        None => facts.flags |= FLAG_UNKNOWN_MAGIC,
    }
    // The section table, addressed in u64 so that no declared size can overflow.
    let table = opt as u64 + soh;
    let table_end = table + SECTION_HEADER_LEN as u64 * u64::from(facts.sections);
    // The headers are data, not overlay: the raw extent starts where the table ends and
    // only grows past it with the sections' raw data.
    let mut raw_end = table_end;
    if table_end > first.len() as u64 {
        facts.flags |= FLAG_SECTIONS_NOT_HELD;
    } else {
        for index in 0..usize::from(facts.sections) {
            let header = table as usize + SECTION_HEADER_LEN * index;
            let raw_size = u64::from(u32_at(first, header + 16).unwrap_or(0));
            let raw_pointer = u64::from(u32_at(first, header + 20).unwrap_or(0));
            let extent = raw_pointer + raw_size;
            if extent > size {
                facts.flags |= FLAG_SECTION_BEYOND_FILE;
            } else {
                raw_end = raw_end.max(extent);
            }
        }
    }
    facts.valid = fixed.is_some_and(|fixed| soh >= fixed)
        && (1..=PE_MAX_SECTIONS).contains(&facts.sections)
        && facts.flags & FLAG_SECTIONS_NOT_HELD == 0
        && table_end <= size;
    if facts.valid {
        facts.overlay = size.saturating_sub(raw_end);
    }
    facts
}

/// Reads the certificate table and CLR runtime header entries of the data directories
/// declared at `opt + fixed - 4`, each only when the count, the declared optional header
/// size and `first` all reach it.
fn read_directories(
    first: &[u8], size: u64, opt: usize, soh: u64, fixed: u64, facts: &mut PeFacts,
) {
    let count = u64::from(u32_at(first, opt + fixed as usize - 4).unwrap_or(0));
    let entry = |index: u64| -> Option<(u32, u32)> {
        if index >= count.min(DIRECTORY_ENTRIES) || fixed + 8 * (index + 1) > soh {
            return None;
        }
        let at = opt + (fixed + 8 * index) as usize;
        Some((u32_at(first, at)?, u32_at(first, at + 4)?))
    };
    if let Some((offset, length)) = entry(DIRECTORY_CERTIFICATE_TABLE) {
        if length != 0 {
            if u64::from(offset) + u64::from(length) > size {
                facts.flags |= FLAG_CERTIFICATE_OUT_OF_FILE;
            } else {
                // Offset zero is the DOS header, not a certificate: both halves count.
                facts.signed = offset != 0;
            }
        }
    }
    if let Some((rva, length)) = entry(DIRECTORY_CLR_RUNTIME_HEADER) {
        facts.clr = rva != 0 && length != 0;
    }
}

/// Position of the COFF header in `first`, when the block holds `MZ`, a usable `e_lfanew`,
/// `PE\0\0` and the 20 COFF bytes after it.
fn locate(first: &[u8]) -> Option<usize> {
    if !wants(first) {
        return None;
    }
    let e_lfanew = u32_at(first, E_LFANEW_AT)?;
    let headers_end = u64::from(e_lfanew) + (PE_SIGNATURE.len() + COFF_HEADER_LEN) as u64;
    if e_lfanew < 4 || headers_end > first.len() as u64 {
        return None;
    }
    let signature = e_lfanew as usize;
    if first.get(signature..signature + PE_SIGNATURE.len())? != PE_SIGNATURE {
        return None;
    }
    Some(signature + PE_SIGNATURE.len())
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    /// A hand-built image: DOS header, `PE\0\0`, COFF header, optional header, section
    /// table and section data. Fields default to a consistent console executable.
    #[derive(Clone)]
    pub(crate) struct Image {
        pub e_lfanew: u32,
        pub machine: u16,
        pub characteristics: u16,
        pub magic: u16,
        pub subsystem: u16,
        pub dll_characteristics: u16,
        /// The data directories in index order, `(rva, size)`.
        pub directories: Vec<(u32, u32)>,
        /// The declared directory count, `directories.len()` by default.
        pub directory_count: Option<u32>,
        /// The optional header size, both declared and emitted: the emitted fields are cut
        /// or zero-padded to it. The natural size of the fields by default.
        pub size_of_optional_header: Option<u16>,
        /// The sections, `(pointer_to_raw_data, size_of_raw_data)`.
        pub sections: Vec<(u32, u32)>,
        /// The declared section count, `sections.len()` by default.
        pub section_count: Option<u16>,
        /// The file length, the end of the furthest section's raw data by default.
        pub len: Option<usize>,
    }

    pub(crate) fn pe32() -> Image {
        Image {
            e_lfanew: 0x40,
            machine: 0x14c,
            characteristics: 0x0102,
            magic: PE32_MAGIC,
            subsystem: 3,
            dll_characteristics: 0x8140,
            directories: vec![(0, 0); 16],
            directory_count: None,
            size_of_optional_header: None,
            sections: vec![(0x200, 0x200)],
            section_count: None,
            len: None,
        }
    }

    pub(crate) fn pe32plus() -> Image {
        Image { machine: 0x8664, magic: PE32PLUS_MAGIC, ..pe32() }
    }

    fn put_u16(bytes: &mut [u8], at: usize, value: u16) {
        bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut [u8], at: usize, value: u32) {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    impl Image {
        /// The bytes of the image. An `e_lfanew` below 64 overlaps the DOS header, whose
        /// `MZ` and `e_lfanew` are written last.
        pub(crate) fn build(&self) -> Vec<u8> {
            let mut out = vec![0; self.e_lfanew as usize];
            let fixed = if self.magic == PE32PLUS_MAGIC { 112 } else { 96 };
            let mut optional = vec![0; fixed + 8 * self.directories.len()];
            put_u16(&mut optional, 0, self.magic);
            put_u16(&mut optional, 68, self.subsystem);
            put_u16(&mut optional, 70, self.dll_characteristics);
            let count = self.directory_count.unwrap_or(self.directories.len() as u32);
            put_u32(&mut optional, fixed - 4, count);
            for (index, (rva, size)) in self.directories.iter().enumerate() {
                put_u32(&mut optional, fixed + 8 * index, *rva);
                put_u32(&mut optional, fixed + 8 * index + 4, *size);
            }
            // The declared size is the layout: the section table follows it directly.
            let soh = self.size_of_optional_header.unwrap_or(optional.len() as u16);
            optional.resize(usize::from(soh), 0);
            out.extend_from_slice(b"PE\0\0");
            let mut coff = [0; 20];
            put_u16(&mut coff, 0, self.machine);
            put_u16(&mut coff, 2, self.section_count.unwrap_or(self.sections.len() as u16));
            put_u16(&mut coff, 16, soh);
            put_u16(&mut coff, 18, self.characteristics);
            out.extend_from_slice(&coff);
            out.extend_from_slice(&optional);
            for (index, (pointer, size)) in self.sections.iter().enumerate() {
                let mut header = [0; 40];
                put_u32(&mut header, 8, *size);
                put_u32(&mut header, 12, 0x1000 * (index as u32 + 1));
                put_u32(&mut header, 16, *size);
                put_u32(&mut header, 20, *pointer);
                out.extend_from_slice(&header);
            }
            let len = match self.len {
                Some(len) => len,
                None => out
                    .len()
                    .max(self.sections.iter().map(|(p, s)| (*p + *s) as usize).max().unwrap_or(0)),
            };
            out.resize(len.max(0x40), 0xcc);
            out[..2].copy_from_slice(b"MZ");
            put_u32(&mut out, 0x3c, self.e_lfanew);
            out.truncate(len.max(2));
            out
        }
    }

    fn facts_of(bytes: &[u8]) -> PeFacts {
        analyze(&bytes[..bytes.len().min(4096)], bytes.len() as u64)
    }

    fn valid_pe32() -> PeFacts {
        PeFacts {
            valid: true,
            flags: 0,
            machine: 0x14c,
            characteristics: 0x0102,
            subsystem: 3,
            dll_characteristics: 0x8140,
            sections: 1,
            magic: PE32_MAGIC,
            clr: false,
            signed: false,
            overlay: 0,
            is_dll: false,
            is_executable_image: true,
        }
    }

    /// The invariants every analysis upholds, whatever the input.
    fn check(facts: &PeFacts, size: u64) {
        if facts.valid {
            assert_eq!(facts.flags & (FLAG_SECTIONS_NOT_HELD | FLAG_UNKNOWN_MAGIC), 0, "{facts:?}");
            assert!((1..=PE_MAX_SECTIONS).contains(&facts.sections), "{facts:?}");
            assert!([PE32_MAGIC, PE32PLUS_MAGIC].contains(&facts.magic), "{facts:?}");
        } else {
            assert_eq!(facts.overlay, 0, "{facts:?}");
        }
        assert!(facts.overlay <= size, "{facts:?}");
        assert_eq!(facts.is_dll, facts.characteristics & 0x2000 != 0, "{facts:?}");
        assert_eq!(facts.is_executable_image, facts.characteristics & 0x0002 != 0, "{facts:?}");
        assert!(!(facts.signed && facts.flags & FLAG_CERTIFICATE_OUT_OF_FILE != 0), "{facts:?}");
    }

    #[test]
    fn only_mz_prefixes_are_wanted() {
        assert!(wants(b"MZ"));
        assert!(wants(b"MZ\x90\x00"));
        for prefix in [&b""[..], b"M", b"ZM", b"PK\x03\x04", b"PE\0\0", b"\x7fELF"] {
            assert!(!wants(prefix), "{prefix:?}");
        }
    }

    #[test]
    fn plain_dos_executables_yield_nothing() {
        let mut dos = vec![0; 0x80];
        dos[..2].copy_from_slice(b"MZ");
        assert_eq!(facts_of(&dos), PeFacts::default());
        // `e_lfanew` pointing at bytes that are not a PE signature.
        put_u32(&mut dos, 0x3c, 0x40);
        assert_eq!(facts_of(&dos), PeFacts::default());
        dos[0x40..0x44].copy_from_slice(b"PE\0\x01");
        assert_eq!(facts_of(&dos), PeFacts::default());
        // Too short to hold `e_lfanew` at all, and not starting with `MZ`.
        assert_eq!(facts_of(b"MZ"), PeFacts::default());
        assert_eq!(facts_of(&[b'M'; 0x3f]), PeFacts::default());
        let mut foreign = pe32().build();
        foreign[1] = b'z';
        assert_eq!(facts_of(&foreign), PeFacts::default());
    }

    #[test]
    fn unknown_optional_magic_is_flagged_and_invalid() {
        for magic in [0, 0x107, 0x10a, 0x20a, 0xffff] {
            let mut image = Image { magic, ..pe32() };
            // Directories are not read behind an unknown magic, whatever they hold.
            image.directories[4] = (0x200, 0x100);
            image.directories[14] = (0x2000, 72);
            let expected = PeFacts {
                valid: false,
                flags: FLAG_UNKNOWN_MAGIC,
                magic,
                overlay: 0,
                ..valid_pe32()
            };
            assert_eq!(facts_of(&image.build()), expected, "{magic:#x}");
        }
    }

    #[test]
    fn hostile_e_lfanew_values_yield_nothing() {
        let bytes = pe32().build();
        let len = bytes.len() as u32;
        for e_lfanew in [0, 1, 3, 0x3c, 0x41, len - 24 + 1, len - 1] {
            let mut mutated = bytes.clone();
            put_u32(&mut mutated, 0x3c, e_lfanew);
            assert_eq!(facts_of(&mutated), PeFacts::default(), "{e_lfanew:#x}");
        }
        // An 8 KiB file: these offsets lie inside it, but the headers they address do
        // not lie inside its 4 KiB first block, the only bytes the analysis sees.
        for e_lfanew in [0xffff_ffff, 0xffff_ffe8, 0x8000_0000, 4096 - 23] {
            let mut mutated = bytes.clone();
            mutated.resize(8192, 0);
            put_u32(&mut mutated, 0x3c, e_lfanew);
            assert_eq!(facts_of(&mutated), PeFacts::default(), "{e_lfanew:#x}");
        }
        // Headers past the first block but inside the file are not held: nothing is known.
        let unheld = Image { e_lfanew: 5000, ..pe32() }.build();
        assert!(unheld.len() > 5000);
        assert_eq!(facts_of(&unheld), PeFacts::default());
        // Headers ending exactly at the block's end are held; nothing after them is.
        let edge = Image { e_lfanew: 4096 - 24, directories: vec![], ..pe32() }.build();
        let facts = facts_of(&edge);
        assert_eq!(facts.machine, 0x14c, "{facts:?}");
        assert_eq!(facts.flags, FLAG_SECTIONS_NOT_HELD | FLAG_UNKNOWN_MAGIC, "{facts:?}");
        assert!(!facts.valid, "{facts:?}");
    }

    #[test]
    fn e_lfanew_inside_the_dos_header_is_parsed_from_the_overlap() {
        // The DOS header's `e_lfanew` field lands on BaseOfCode (+20 of the optional
        // header): nothing the analysis reads.
        let bytes = Image { e_lfanew: 0x10, ..pe32() }.build();
        assert_eq!(&bytes[..2], b"MZ");
        assert_eq!(&bytes[0x10..0x14], b"PE\0\0");
        assert_eq!(u32_at(&bytes, 0x3c), Some(0x10));
        assert_eq!(facts_of(&bytes), valid_pe32());
        // Four is the least offset that keeps `MZ` out of the signature.
        let bytes = Image { e_lfanew: 4, ..pe32() }.build();
        assert_eq!(facts_of(&bytes), valid_pe32());
    }

    #[test]
    fn hostile_optional_header_sizes_are_invalid() {
        // Without any optional header the section table's first bytes read as the magic.
        let none = Image { size_of_optional_header: Some(0), ..pe32() }.build();
        let facts = facts_of(&none);
        assert_eq!(facts.flags, FLAG_UNKNOWN_MAGIC, "{facts:?}");
        assert_eq!((facts.valid, facts.magic, facts.overlay), (false, 0, 0), "{facts:?}");
        // One byte short of the fixed fields is invalid, the fixed fields alone are valid.
        for (image, short, enough) in [(pe32(), 95, 96), (pe32plus(), 111, 112)] {
            let short = Image { size_of_optional_header: Some(short), ..image.clone() }.build();
            let facts = facts_of(&short);
            assert_eq!((facts.valid, facts.flags, facts.overlay), (false, 0, 0), "{facts:?}");
            assert_eq!(facts.magic, image.magic);
            let enough = Image { size_of_optional_header: Some(enough), ..image.clone() }.build();
            let facts = facts_of(&enough);
            assert!(facts.valid && facts.flags == 0, "{facts:?}");
        }
        // A huge optional header pushes the section table out of the block.
        let huge = Image { size_of_optional_header: Some(0xffff), ..pe32() }.build();
        assert!(huge.len() > 0xffff);
        let facts = facts_of(&huge);
        assert_eq!(facts.flags, FLAG_SECTIONS_NOT_HELD, "{facts:?}");
        assert_eq!((facts.valid, facts.magic, facts.overlay), (false, PE32_MAGIC, 0), "{facts:?}");
    }

    #[test]
    fn hostile_section_counts_are_invalid() {
        let none = Image { section_count: Some(0), ..pe32() }.build();
        let facts = facts_of(&none);
        assert_eq!((facts.valid, facts.flags, facts.sections, facts.overlay), (false, 0, 0, 0));
        // The most sections allowed, all held in the block, are valid.
        let full = Image { sections: vec![(0x1000, 0x10); 96], directories: vec![], ..pe32() };
        let facts = facts_of(&full.build());
        assert_eq!((facts.valid, facts.flags, facts.sections, facts.overlay), (true, 0, 96, 0));
        // One more, still held in the block, is not.
        let mut over = full.clone();
        over.sections.push((0x1000, 0x10));
        assert_eq!(0x40 + 24 + 96 + 40 * 97, 4064);
        let facts = facts_of(&over.build());
        assert_eq!((facts.valid, facts.flags, facts.sections, facts.overlay), (false, 0, 97, 0));
        // A count beyond the block is not held.
        let unheld = Image { section_count: Some(0xffff), ..pe32() }.build();
        let facts = facts_of(&unheld);
        assert_eq!(facts.flags, FLAG_SECTIONS_NOT_HELD, "{facts:?}");
        assert_eq!((facts.valid, facts.sections, facts.overlay), (false, 0xffff, 0));
    }

    #[test]
    fn hostile_directory_counts_bound_the_entries_read() {
        let mut image = pe32();
        image.len = Some(0x600);
        image.directories[4] = (0x400, 0x200);
        image.directories[14] = (0x2000, 72);
        image.directory_count = Some(0);
        let expected = PeFacts { overlay: 0x200, ..valid_pe32() };
        assert_eq!(facts_of(&image.build()), expected);
        image.directory_count = Some(0xffff_ffff);
        assert_eq!(facts_of(&image.build()), PeFacts { signed: true, clr: true, ..expected });
    }

    #[test]
    fn section_table_cut_by_the_block_is_not_held() {
        // The optional header is held whole, the section table lies just beyond the block.
        let image = Image { e_lfanew: 4096 - 24 - 224, ..pe32() };
        let bytes = image.build();
        assert!(bytes.len() > 4096);
        let expected =
            PeFacts { valid: false, flags: FLAG_SECTIONS_NOT_HELD, overlay: 0, ..valid_pe32() };
        assert_eq!(facts_of(&bytes), expected);
        // With the table one byte short of held, the same; one byte earlier, held.
        let table = 4096 - 24 - 224 - 40;
        let cut = Image { e_lfanew: table + 1, ..pe32() }.build();
        assert_eq!(facts_of(&cut), expected);
        // The table ends with the block, which is the whole file: nothing is overlay.
        let held = Image { e_lfanew: table, ..pe32() }.build();
        assert_eq!(held.len(), 4096);
        assert_eq!(facts_of(&held), valid_pe32());
        // Being cut by a short file rather than the block is the same condition.
        let mut short = pe32().build();
        short.truncate(0x40 + 24 + 224 + 39);
        assert_eq!(facts_of(&short), expected);
    }

    #[test]
    fn sections_beyond_the_file_are_flagged_and_contribute_nothing() {
        let mut image = pe32();
        image.len = Some(0x400);
        image.sections = vec![(0x200, 0x200), (0x10000, 0x10)];
        let expected = PeFacts { flags: FLAG_SECTION_BEYOND_FILE, sections: 2, ..valid_pe32() };
        assert_eq!(facts_of(&image.build()), expected);
        // The extents are measured in u64: the largest u32 pair does not wrap around.
        image.sections[1] = (0xffff_ffff, 0xffff_ffff);
        assert_eq!(facts_of(&image.build()), expected);
        // Ending exactly at the file's end is inside it.
        image.sections[1] = (0x3f0, 0x10);
        assert_eq!(facts_of(&image.build()), PeFacts { sections: 2, ..valid_pe32() });
        // With every section beyond the file, everything after the headers is overlay.
        image.sections = vec![(0x10000, 0x10)];
        let facts = facts_of(&image.build());
        assert_eq!(
            (facts.valid, facts.flags, facts.overlay),
            (true, FLAG_SECTION_BEYOND_FILE, 0x400 - HEADERS_END)
        );
    }

    /// The end of the headers of a [`pe32`] image with one section: `e_lfanew`, the
    /// signature, the COFF header, the optional header and the section table.
    const HEADERS_END: u64 = 0x40 + 4 + 20 + 224 + 40;

    #[test]
    fn headers_are_never_overlay() {
        // Sections without raw data (a `.bss`) leave the headers as the only data:
        // the overlay is what follows them, as pefile and LIEF report it.
        let mut image = pe32();
        image.len = Some(0x400);
        image.sections = vec![(0, 0)];
        assert_eq!(
            facts_of(&image.build()),
            PeFacts { overlay: 0x400 - HEADERS_END, ..valid_pe32() }
        );
        // A second empty section lengthens the table by 40 bytes.
        image.sections = vec![(0, 0), (0, 0)];
        let expected = PeFacts { sections: 2, overlay: 0x400 - HEADERS_END - 40, ..valid_pe32() };
        assert_eq!(facts_of(&image.build()), expected);
        // A section whose raw data lies inside the headers does not shorten them.
        image.sections = vec![(0x10, 0x10), (0, 0)];
        assert_eq!(facts_of(&image.build()), expected);
        // A file ending exactly at the headers has no overlay at all.
        image.len = Some(HEADERS_END as usize + 40);
        assert_eq!(facts_of(&image.build()), PeFacts { sections: 2, ..valid_pe32() });
    }

    #[test]
    fn overlay_measures_the_bytes_after_the_last_section() {
        let mut bytes = pe32().build();
        bytes.extend_from_slice(&[b'!'; 1234]);
        assert_eq!(facts_of(&bytes), PeFacts { overlay: 1234, ..valid_pe32() });
        // Sections are not necessarily in file order: the furthest extent counts.
        let mut image = pe32plus();
        image.sections = vec![(0x400, 0x100), (0x200, 0x200)];
        image.len = Some(0x500 + 7);
        let expected = PeFacts {
            machine: 0x8664,
            magic: PE32PLUS_MAGIC,
            sections: 2,
            overlay: 7,
            ..valid_pe32()
        };
        assert_eq!(facts_of(&image.build()), expected);
        // The size is the input's, not the block's: a large payload is measured whole.
        let mut large = pe32().build();
        large.resize(1 << 20, 0);
        assert_eq!(facts_of(&large), PeFacts { overlay: (1 << 20) - 0x400, ..valid_pe32() });
    }

    #[test]
    fn every_byte_mutation_of_the_fixtures_is_survived() {
        let mut fixtures = vec![pe32().build(), pe32plus().build()];
        let mut paths: Vec<_> = std::fs::read_dir("../../tests_data/mitra/pebin")
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        paths.sort();
        assert!(!paths.is_empty(), "no fixture executables");
        for path in paths {
            fixtures.push(std::fs::read(&path).unwrap());
        }
        for original in &fixtures {
            assert!(facts_of(original).valid);
            let held = original.len().min(4096);
            for at in 0..held {
                for value in [0x00, 0xff, 0x50, original[at] ^ 0x01, original[at] ^ 0x80] {
                    let mut mutated = original.clone();
                    mutated[at] = value;
                    let size = mutated.len() as u64;
                    check(&analyze(&mutated[..held], size), size);
                    // Truncations, and sizes disagreeing with the block, are reachable too.
                    check(&analyze(&mutated[..at], at as u64), at as u64);
                    check(&analyze(&mutated[..held], 1 << 40), 1 << 40);
                    check(&analyze(&mutated[..held], 0), 0);
                }
            }
        }
    }

    #[test]
    fn minimal_pe32_yields_exact_facts() {
        let bytes = pe32().build();
        assert_eq!(bytes.len(), 0x400);
        assert_eq!(facts_of(&bytes), valid_pe32());
    }

    #[test]
    fn minimal_pe32plus_yields_exact_facts() {
        let image = Image { sections: vec![(0x200, 0x200), (0x400, 0x100)], ..pe32plus() };
        let bytes = image.build();
        assert_eq!(bytes.len(), 0x500);
        let expected =
            PeFacts { machine: 0x8664, magic: PE32PLUS_MAGIC, sections: 2, ..valid_pe32() };
        assert_eq!(facts_of(&bytes), expected);
        // The optional header is 16 bytes longer, so the section table moved: its last
        // header ends at 0x40 + 4 + 20 + 240 + 80 = 0x1a0 = 416.
        let end = 0x40 + 4 + 20 + 240 + 80;
        assert_eq!(&bytes[end - 40..end - 40 + 8], &[0; 8]);
        assert_eq!(u32_at(&bytes, end - 40 + 20), Some(0x400));
    }

    #[test]
    fn fixture_executables_yield_exact_facts() {
        // Values cross-checked with an independent Python decoding of the fixtures.
        let expected = PeFacts {
            valid: true,
            flags: 0,
            machine: 0x14c,
            characteristics: 0x0102,
            subsystem: 3,
            dll_characteristics: 0,
            sections: 1,
            magic: PE32_MAGIC,
            clr: false,
            signed: false,
            overlay: 0,
            is_dll: false,
            is_executable_image: true,
        };
        let pe32 = std::fs::read("../../tests_data/mitra/pebin/pe32.exe").unwrap();
        assert_eq!(pe32.len(), 1024);
        assert_eq!(facts_of(&pe32), expected);
        let pe64 = std::fs::read("../../tests_data/mitra/pebin/pe64.exe").unwrap();
        assert_eq!(pe64.len(), 1024);
        assert_eq!(facts_of(&pe64), PeFacts { machine: 0x8664, magic: PE32PLUS_MAGIC, ..expected });
    }

    #[test]
    fn dll_and_executable_image_bits_are_exposed_as_bytes() {
        for (characteristics, is_dll, is_executable_image) in [
            (0x0000, false, false),
            (0x0002, false, true),
            (0x2000, true, false),
            (0x2102, true, true),
            (0xfffd, true, false),
        ] {
            let bytes = Image { characteristics, ..pe32() }.build();
            let facts = facts_of(&bytes);
            assert_eq!(facts.characteristics, characteristics, "{characteristics:#x}");
            assert_eq!((facts.is_dll, facts.is_executable_image), (is_dll, is_executable_image));
            assert!(facts.valid, "the bits do not affect validity");
        }
    }

    #[test]
    fn clr_directory_marks_managed_images() {
        let mut image = pe32();
        image.directories[14] = (0x2000, 72);
        assert_eq!(facts_of(&image.build()), PeFacts { clr: true, ..valid_pe32() });
        // Either half being zero means no runtime header.
        image.directories[14] = (0x2000, 0);
        assert_eq!(facts_of(&image.build()), valid_pe32());
        image.directories[14] = (0, 72);
        assert_eq!(facts_of(&image.build()), valid_pe32());
        // The entry counts only when the declared count reaches it.
        image.directories[14] = (0x2000, 72);
        image.directory_count = Some(14);
        assert_eq!(facts_of(&image.build()), valid_pe32());
        image.directory_count = Some(15);
        assert_eq!(facts_of(&image.build()), PeFacts { clr: true, ..valid_pe32() });
        // ...and when the declared optional header size covers it: cut to 14 entries, the
        // bytes where entry 14 would be are the first section's name, whatever they hold.
        image.directory_count = None;
        image.size_of_optional_header = Some(96 + 8 * 14);
        let mut bytes = image.build();
        let name = 0x40 + 4 + 20 + 96 + 8 * 14;
        put_u32(&mut bytes, name, 0x2000);
        put_u32(&mut bytes, name + 4, 72);
        assert_eq!(facts_of(&bytes), valid_pe32());
        image.size_of_optional_header = Some(96 + 8 * 15);
        assert_eq!(facts_of(&image.build()), PeFacts { clr: true, ..valid_pe32() });
        // A PE32+ image keeps the same directory 16 bytes further along.
        let mut plus = pe32plus();
        plus.directories[14] = (0x2000, 72);
        let facts = facts_of(&plus.build());
        assert!(facts.clr && facts.valid, "{facts:?}");
    }

    #[test]
    fn certificate_directory_marks_signed_images_when_inside_the_file() {
        let mut image = pe32();
        image.len = Some(0x600);
        image.directories[4] = (0x400, 0x200);
        let signed = PeFacts { signed: true, overlay: 0x200, ..valid_pe32() };
        assert_eq!(facts_of(&image.build()), signed);
        // Ending exactly at the end of the file is inside it; one byte more is not.
        image.directories[4] = (0x400, 0x201);
        let out = PeFacts { signed: false, flags: FLAG_CERTIFICATE_OUT_OF_FILE, ..signed };
        assert_eq!(facts_of(&image.build()), out);
        image.directories[4] = (0xffff_ffff, 0xffff_ffff);
        assert_eq!(facts_of(&image.build()), out);
        // An empty table is not a certificate, wherever it claims to be.
        image.directories[4] = (0xffff_ffff, 0);
        assert_eq!(facts_of(&image.build()), PeFacts { signed: false, ..signed });
        // Nor is one at offset zero, the DOS header: both halves must be set, as for CLR.
        image.directories[4] = (0, 0x200);
        assert_eq!(facts_of(&image.build()), PeFacts { signed: false, ..signed });
        // The directory is only read when the count and the optional header reach it.
        image.directories[4] = (0x400, 0x200);
        image.directory_count = Some(4);
        assert_eq!(facts_of(&image.build()), PeFacts { signed: false, ..signed });
        image.directory_count = None;
        image.size_of_optional_header = Some(96 + 8 * 4);
        let mut bytes = image.build();
        let name = 0x40 + 4 + 20 + 96 + 8 * 4;
        put_u32(&mut bytes, name, 0x400);
        put_u32(&mut bytes, name + 4, 0x200);
        assert_eq!(facts_of(&bytes), PeFacts { signed: false, ..signed });
        // A PE32+ image keeps the same directory 16 bytes further along.
        let mut plus = pe32plus();
        plus.len = Some(0x600);
        plus.directories[4] = (0x400, 0x200);
        let facts = facts_of(&plus.build());
        assert!(facts.signed && facts.valid && facts.overlay == 0x200, "{facts:?}");
    }
}
