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

//! Native executable, object, and debug symbol format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// Apple Mach-O thin binary: 32-bit (`0xFEEDFACE`, ≥ 28 bytes) or 64-bit (`0xFEEDFACF`, ≥ 32
/// bytes) in either byte order, nonzero `filetype` at `@12`, and consistent `ncmds`/`sizeofcmds`
/// at `@16` and `@20`.
pub(crate) fn macho(prefix: &[u8]) -> Option<Label> {
    const MH_MAGIC: u32 = 0xFEED_FACE; // 32-bit Mach-O magic
    const MH_MAGIC_64: u32 = 0xFEED_FACF; // 64-bit Mach-O magic
    const FILETYPE: usize = 12; // @12: u32 — Mach-O file type (≥ 1)
    const NCMDS: usize = 16; // @16: u32 — number of load commands
    const SIZEOFCMDS: usize = 20; // @20: u32 — total byte size of load commands
    const MIN_LEN_32: usize = 28;
    const MIN_LEN_64: usize = 32;

    require!(prefix.len() >= MIN_LEN_32);
    let (read_u32, is_64): (ReadU32, bool) = match le32(prefix, 0)? {
        MH_MAGIC => (le32, false),
        MH_MAGIC_64 => (le32, true),
        _ => match be32(prefix, 0)? {
            MH_MAGIC => (be32, false),
            MH_MAGIC_64 => (be32, true),
            _ => return None,
        },
    };
    require!(!is_64 || prefix.len() >= MIN_LEN_64, read_u32(prefix, FILETYPE)? >= 1);
    let (ncmds, sizeofcmds) = (read_u32(prefix, NCMDS)?, read_u32(prefix, SIZEOFCMDS)?);
    require!((ncmds == 0 && sizeofcmds == 0) || (ncmds >= 1 && sizeofcmds >= 8));
    static MACHO_32_LE: Subformat =
        Subformat::with_ids("Mach-O 32-bit (little-endian)", "fmt/692", None);
    static MACHO_32_BE: Subformat =
        Subformat::with_ids("Mach-O 32-bit (big-endian)", "fmt/692", None);
    static MACHO_64_LE: Subformat =
        Subformat::with_ids("Mach-O 64-bit (little-endian)", "fmt/693", None);
    static MACHO_64_BE: Subformat =
        Subformat::with_ids("Mach-O 64-bit (big-endian)", "fmt/693", None);
    let is_le = matches!(le32(prefix, 0)?, MH_MAGIC | MH_MAGIC_64);
    with_subformat(
        Label::Macho,
        match (is_64, is_le) {
            (false, true) => &MACHO_32_LE,
            (false, false) => &MACHO_32_BE,
            (true, true) => &MACHO_64_LE,
            (true, false) => &MACHO_64_BE,
        },
    )
}

/// Microsoft Program Database (PDB): MSF 7.00 header (`"Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0"`)
/// with valid power-of-two page size at `@32` and free block map `1 | 2` at `@36`, legacy JG
/// 2.00 / NCB / 1.00 header, or .NET Portable PDB (`"BSJB"` with `#Pdb` / `"MPDB"`).
pub(crate) fn pdb(prefix: &[u8]) -> Option<Label> {
    const MODERN_MAGIC: &[u8] = b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0"; // @0..32: MSF 7.00 signature
    const OLD_MAGIC: &[u8] = b"Microsoft C/C++ program database 2.00\r\n\x1aJG\0\0"; // @0..44: MSF 2.00 signature
    const NCB_MAGIC: &[u8] = b"Microsoft C/C++ program database 2.00\r\n\x1a\0\0\0\0"; // @0..44: Visual C++ NCB signature
    const PDB1_MAGIC: &[u8] = b"Microsoft C/C++ program database 1.00\r\n\x1aJG\0\0"; // @0..44: PDB 1.00 signature
    const MPDB_MAGIC: &[u8] = b"MPDB";
    const BSJB_MAGIC: &[u8] = b"BSJB";
    const PDB_V1_0: &[u8] = b"PDB v1.0";
    const PDB_STREAM: &[u8] = b"#Pdb\0";
    const MODERN_PAGE_SIZE: usize = 32; // @32: u32 LE — page size (512..=32768)
    const MODERN_FPM: usize = 36; // @36: u32 LE — free block map block (1 or 2)
    const OLD_PAGE_SIZE: usize = 44; // @44: u32 LE — page size (512..=32768)
    const MODERN_MIN_LEN: usize = 56;
    const OLD_MIN_LEN: usize = 60;

    static EMBEDDED_PORTABLE_PDB: Subformat = Subformat::new("Embedded Portable PDB");
    static PORTABLE_PDB: Subformat = Subformat::new("Portable PDB");
    static MSF_7_00: Subformat = Subformat::with_ids("Microsoft PDB (MSF 7.00)", "fmt/1079", None);
    static MSF_2_00: Subformat = Subformat::with_ids("Microsoft PDB (MSF 2.00)", "fmt/1078", None);
    static NCB: Subformat = Subformat::new("Visual C++ NCB (MSF 2.00)");
    static PDB_1_00: Subformat = Subformat::new("Microsoft PDB 1.00");

    fn valid_page_size(n: u32) -> bool {
        matches!(n, 0x0200 | 0x0400 | 0x0800 | 0x1000 | 0x2000 | 0x4000 | 0x8000)
    }

    if prefix.len() >= 8 && starts(prefix, MPDB_MAGIC) {
        require!(le32(prefix, 4)? >= 64);
        return with_subformat(Label::Pdb, &EMBEDDED_PORTABLE_PDB);
    }
    if prefix.len() >= 32 && starts(prefix, BSJB_MAGIC) {
        let vlen = le32(prefix, 12)? as usize;
        require!(
            (4..=64).contains(&vlen),
            vlen.is_multiple_of(4),
            eq_at(prefix, 16, PDB_V1_0) || contains(prefix, PDB_STREAM),
        );
        return with_subformat(Label::Pdb, &PORTABLE_PDB);
    }

    let modern = prefix.len() >= MODERN_MIN_LEN
        && starts(prefix, MODERN_MAGIC)
        && valid_page_size(le32(prefix, MODERN_PAGE_SIZE)?)
        && matches!(le32(prefix, MODERN_FPM)?, 1 | 2);
    let old = prefix.len() >= OLD_MIN_LEN
        && (starts(prefix, OLD_MAGIC) || starts(prefix, NCB_MAGIC))
        && valid_page_size(le32(prefix, OLD_PAGE_SIZE)?);
    let pdb1 = prefix.len() >= MODERN_MIN_LEN && starts(prefix, PDB1_MAGIC);
    require!(modern || old || pdb1);
    let sub = if modern {
        &MSF_7_00
    } else if old && starts(prefix, OLD_MAGIC) {
        &MSF_2_00
    } else if old {
        &NCB
    } else {
        &PDB_1_00
    };
    with_subformat(Label::Pdb, sub)
}

/// IBM AIX XCOFF object/executable: magic `0x01F7` (XCOFF64) or `0x01DF` (XCOFF32) at `@0`,
/// section count `1..=1024` at `@2`, and a standard auxiliary header size at `@16` (`0` or `120`
/// for 64-bit; `0`, `28`, or `72` for 32-bit).
pub(crate) fn xcoff(prefix: &[u8]) -> Option<Label> {
    const MAGIC: usize = 0; // @0: u16 BE — 0x01F7 (XCOFF64) or 0x01DF (XCOFF32)
    const XCOFF64: u16 = 0x01F7; // 503
    const XCOFF32: u16 = 0x01DF; // 479
    const N_SCNS: usize = 2; // @2: u16 BE — number of sections (1..=1024)
    const OPT_HDR: usize = 16; // @16: u16 BE — optional/auxiliary header byte size
    const MIN_LEN: usize = 24;

    require!(prefix.len() >= MIN_LEN, (1..=1024).contains(&be16(prefix, N_SCNS)?),);
    static XCOFF_32: Subformat = Subformat::new("XCOFF32");
    static XCOFF_64: Subformat = Subformat::new("XCOFF64");
    let opt = be16(prefix, OPT_HDR)?;
    let magic = be16(prefix, MAGIC)?;
    let valid = match magic {
        XCOFF64 => matches!(opt, 0 | 120),
        XCOFF32 => matches!(opt, 0 | 28 | 72),
        _ => false,
    };
    require!(valid);
    with_subformat(Label::Xcoff, if magic == XCOFF64 { &XCOFF_64 } else { &XCOFF_32 })
}

pub(crate) const MZ_MAGIC: &[u8; 2] = b"MZ";

pub(crate) const ZM_MAGIC: &[u8; 2] = b"ZM";

pub(crate) const PE_MAGIC: &[u8; 4] = b"PE\0\0";

/// Checks whether `prefix` (of an input of `size` bytes) holds a valid Windows PE/COFF image
/// header with `IMAGE_FILE_EXECUTABLE_IMAGE` set, not a resource-only DLL (`is_dll &&
/// entry_point == 0`), Windows subsystem in `1..=16`, and a shipping Windows `Machine`
/// architecture.
pub(crate) fn is_windows_pebin(prefix: &[u8], size: u64) -> bool {
    if !starts(prefix, MZ_MAGIC) {
        return false;
    }
    let Some(e_lfanew) = le32(prefix, 0x3C) else {
        return false;
    };
    let headers_end = u64::from(e_lfanew) + 24;
    if e_lfanew < 4 || headers_end > prefix.len() as u64 {
        return false;
    }
    let sig = e_lfanew as usize;
    if !eq_at(prefix, sig, PE_MAGIC) {
        return false;
    }
    let coff = sig + 4;
    let machine = le16(prefix, coff).unwrap_or(0);
    let sections = le16(prefix, coff + 2).unwrap_or(0);
    let soh = u64::from(le16(prefix, coff + 16).unwrap_or(0));
    let characteristics = le16(prefix, coff + 18).unwrap_or(0);
    let is_dll = characteristics & 0x2000 != 0;
    let is_executable_image = characteristics & 0x0002 != 0;
    let opt = coff + 20;
    let magic = le16(prefix, opt).unwrap_or(0);
    let entry_point = le32(prefix, opt + 16).unwrap_or(0);
    let subsystem = le16(prefix, opt + 68).unwrap_or(0);
    let fixed = match magic {
        0x010B => 96u64,
        0x020B => 112u64,
        _ => return false,
    };
    let table_end = (opt as u64) + soh + 40 * u64::from(sections);
    soh >= fixed
        && (1..=96).contains(&sections)
        && table_end <= prefix.len() as u64
        && table_end <= size
        && is_executable_image
        && !(is_dll && entry_point == 0)
        && (1..=16).contains(&subsystem)
        && matches!(
            machine,
            0x014C
                | 0x0166
                | 0x0184
                | 0x01C0
                | 0x01C2
                | 0x01C4
                | 0x01F0
                | 0x01F1
                | 0x0200
                | 0x3A64
                | 0x5032
                | 0x5064
                | 0x8664
                | 0xA641
                | 0xA642
                | 0xA64E
                | 0xAA64
        )
}

static PE32: Subformat = Subformat::with_ids("PE32", "fmt/899", None);
static PE32_PLUS: Subformat = Subformat::with_ids("PE32+", "fmt/900", None);

/// Names the optional header magic (`0x010B` PE32 / `0x020B` PE32+) already validated by
/// [`is_windows_pebin`].
fn windows_pe_subformat(prefix: &[u8]) -> &'static Subformat {
    let magic = le32(prefix, 0x3C).and_then(|e_lfanew| le16(prefix, e_lfanew as usize + 24));
    if magic == Some(0x020B) { &PE32_PLUS } else { &PE32 }
}

/// Windows PE/COFF executable (`taxonomy_pebin`).
pub(crate) fn pe(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    require!(is_windows_pebin(prefix, ctx.size));
    with_subformat(Label::Pebin, windows_pe_subformat(prefix))
}

/// Standalone Microsoft / DJGPP COFF Object File (`coff`): `Machine` (`0x014C` i386,
/// `0x8664` AMD64, `0xAA64` ARM64, `0xA641`/`0xA642`/`0xA64E` ARM64EC/ARM64X, `0x3A64` CHPE x86,
/// `0x01C0`/`0x01C4` ARM) at `@0`, `NumberOfSections` `1..=96` at `@2`,
/// `SizeOfOptionalHeader == 0` at `@16`, and a valid first section header at `@20..60`
/// with printable ASCII name (or a `/bigobj` / `ANON_OBJECT_HEADER` COFF object).
pub(crate) fn coff(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= 56
        && prefix.len() >= 56
        && starts(prefix, &[0x00, 0x00, 0xFF, 0xFF, 0x02, 0x00])
        && eq_at(
            prefix,
            12,
            &[
                0xC7, 0xA1, 0xBA, 0xD1, 0xEE, 0xBA, 0xA9, 0x4B, 0xAF, 0x20, 0xFA, 0xF6, 0x6A, 0xA4,
                0xDC, 0xB8,
            ],
        )
    {
        static COFF_BIGOBJ: Subformat = Subformat::new("COFF object (/bigobj)");
        return with_subformat(Label::Coff, &COFF_BIGOBJ);
    }
    const MIN_COFF_SIZE: u64 = 60; // 20B IMAGE_FILE_HEADER + 40B IMAGE_SECTION_HEADER

    require!(
        size >= MIN_COFF_SIZE,
        prefix.len() >= 60,
        matches!(
            le16(prefix, 0)?,
            0x014C | 0x8664 | 0xAA64 | 0xA641 | 0xA642 | 0xA64E | 0x3A64 | 0x01C0 | 0x01C4
        ),
    );
    let n_sects = le16(prefix, 2)?;
    let opt_hdr = le16(prefix, 16)?;
    require!((1..=96).contains(&n_sects), opt_hdr == 0);
    let sec_name = get_slice(prefix, 20, 28)?;
    let name_len = sec_name.iter().position(|&b| b == 0).unwrap_or(8);
    require!(
        name_len >= 2,
        sec_name[0] == b'.' || sec_name[0] == b'/' || sec_name[0] == b'_',
        sec_name[..name_len].iter().all(|&b| (0x20..=0x7E).contains(&b)),
        sec_name[name_len..].iter().all(|&b| b == 0),
    );
    static COFF_OBJECT: Subformat = Subformat::new("COFF object");
    with_subformat(Label::Coff, &COFF_OBJECT)
}

/// AutoIt v3 compiled script 16-byte signature (`A3 48 4B BE 98 6C 4A A9 99 4C 53 0A 86 D6 48 7D`).
pub(crate) const AUTOIT_V3_MAGIC: &[u8] = &[
    0xA3, 0x48, 0x4B, 0xBE, 0x98, 0x6C, 0x4A, 0xA9, 0x99, 0x4C, 0x53, 0x0A, 0x86, 0xD6, 0x48, 0x7D,
];

/// AutoIt v3 EA05/EA06 archive marker (`"AU3!EA0"`).
pub(crate) const AUTOIT_EA0_MAGIC: &[u8] = b"AU3!EA0";

/// Checks for AutoIt v3 compiled executable signature (`\xA3\x48\x4B\xBE...` or `"AU3!EA0"`)
/// before falling back to Windows PE/DOS MZ (`pebin`).
pub(crate) fn pe_or_autoit(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    const KKRUNCHY_TAG: &[u8] = b"KKRUNCHY";
    const DOS32_ADAM_MAGIC: &[u8; 4] = b"Adam";
    const TMT_STUB_TAG: &[u8; 3] = b"TMT";
    static AUTOIT_EXE: Subformat = Subformat::new("AutoIt v3 compiled executable");
    static PE_TINY: Subformat =
        Subformat::with_ids("PE (tiny, header at offset 2)", "x-fmt/411", None);
    static PE_NONSTANDARD: Subformat =
        Subformat::with_ids("PE (non-standard header)", "x-fmt/411", None);
    static NE: Subformat = Subformat::with_ids("NE executable", "x-fmt/410", None);
    static LE: Subformat = Subformat::new("LE executable");
    static LX: Subformat = Subformat::new("OS/2 LX executable");
    static W3: Subformat = Subformat::new("Windows W3 executable");
    static W4: Subformat = Subformat::new("Windows W4 executable");
    static DOS32_ADAM: Subformat = Subformat::new("DOS32 Adam executable");
    static DOS_MZ: Subformat = Subformat::with_ids("MS-DOS MZ executable", "x-fmt/409", None);
    static DOS_ZM: Subformat = Subformat::with_ids("MS-DOS ZM executable", "x-fmt/409", None);

    if contains(prefix, AUTOIT_V3_MAGIC) || contains(prefix, AUTOIT_EA0_MAGIC) {
        return with_subformat(Label::Autoit, &AUTOIT_EXE);
    }
    if let Some(t) = ctx.tail
        && (contains(t, AUTOIT_V3_MAGIC) || contains(t, AUTOIT_EA0_MAGIC))
    {
        return with_subformat(Label::Autoit, &AUTOIT_EXE);
    }
    if is_windows_pebin(prefix, ctx.size) {
        return with_subformat(Label::Pebin, windows_pe_subformat(prefix));
    }
    if prefix.len() >= 0x28
        && eq_at(prefix, 2, PE_MAGIC)
        && matches!(le16(prefix, 6), Some(0x014C | 0x8664 | 0xAA64))
    {
        return with_subformat(Label::Pebin, &PE_TINY);
    }
    if prefix.len() >= 0x40 {
        let e_lfanew = le32(prefix, 0x3C).unwrap_or(0) as usize;
        if e_lfanew >= 4 && eq_at(prefix, e_lfanew, PE_MAGIC) {
            let coff = e_lfanew + 4;
            let sections = le16(prefix, coff + 2).unwrap_or(1);
            let soh = le16(prefix, coff + 16).unwrap_or(1);
            let is_reflective_stub = starts(prefix, b"MZAR\x55\x48\x89\xE5")
                || starts(prefix, b"MZ\xE8\x00\x00\x00\x00")
                || starts(prefix, b"MZRE\xE8\x00\x00\x00\x00")
                || starts(prefix, b"MZRE\x55\x48\x89\xE5")
                || starts(prefix, b"MZER\xE8\x00\x00\x00\x00")
                || starts(prefix, b"MZER\x55");
            if (e_lfanew <= 0x10
                || is_reflective_stub
                || contains(prefix, KKRUNCHY_TAG)
                || (sections == 0 && soh == 0))
                && matches!(le16(prefix, coff), Some(0x014C | 0x8664 | 0xAA64))
            {
                return with_subformat(Label::Pebin, &PE_NONSTANDARD);
            }
            return None;
        }
        if e_lfanew >= 0x40 && e_lfanew + 4 <= prefix.len() && starts(prefix, MZ_MAGIC) {
            let ext_sig = get_slice(prefix, e_lfanew, e_lfanew + 2)?;
            if matches!(ext_sig, b"NE" | b"LE" | b"LX" | b"W3" | b"W4") {
                let sub = match ext_sig {
                    b"NE" => &NE,
                    b"LE" => &LE,
                    b"LX" => &LX,
                    b"W3" => &W3,
                    _ => &W4,
                };
                return with_subformat(Label::Pebin, sub);
            }
        }
    }
    if prefix.len() >= 0x36 && starts(prefix, DOS32_ADAM_MAGIC) && eq_at(prefix, 0x22, TMT_STUB_TAG)
    {
        let _ = get_slice(prefix, 0, 0x36);
        return with_subformat(Label::Pebin, &DOS32_ADAM);
    }
    // Also support classic DOS MZ / Win16 NE / OS/2 LE/LX executables mapped to `pebin`
    require!(
        ctx.size >= 28,
        prefix.len() >= 28,
        starts(prefix, MZ_MAGIC) || starts(prefix, ZM_MAGIC),
    );
    let e_cblp = le16(prefix, 2)?;
    let e_cp = le16(prefix, 4)?;
    let e_cparhdr = le16(prefix, 8)?;
    require!(
        e_cblp <= 512,
        (1..=16384).contains(&e_cp),
        e_cparhdr >= 2 || starts(prefix, b"MZ\x00\x00")
    );
    with_subformat(Label::Pebin, if starts(prefix, MZ_MAGIC) { &DOS_MZ } else { &DOS_ZM })
}

/// MS-DOS / CP/M-80 `.COM` flat binary executable (`ORG 0x0100`, mapped to `pebin`):
/// verifies well-known x86 real-mode entry prologues and `.COM` executable packers
/// (Paragraph Reloc, TheDraw, Turbo Pascal 3.0, DOS `INT 21h AH=30h`, ICE, T-PACK, SHRINK,
/// Compact Peichl, and UPX `dos/com`).
pub(crate) fn dos_com(prefix: &[u8], size: u64) -> Option<Label> {
    const MAX_DOS_COM_SIZE: u64 = 65_280;
    const ICE_STUB: &[u8] = &[
        0xBE, 0x26, 0x01, 0x8B, 0xFE, 0x8B, 0x0E, 0x08, 0x01, 0x8B, 0x16, 0x02, 0x01, 0xB8, 0x77,
        0x01,
    ];
    const TPACK_STUB: &[u8] = &[0x68, 0x00, 0x01, 0xFD, 0x60, 0xBE];
    const SHRINK_STUB: &[u8] = &[
        0x50, 0x9C, 0xFC, 0xBE, 0x1F, 0x01, 0xBF, 0x00, 0x7E, 0x57, 0xB9, 0x34, 0x00, 0xF3, 0xA4,
        0x8B,
    ];
    const COMPACT_PEICHL_STUB: &[u8] = &[
        0x81, 0xFC, 0xE8, 0x80, 0x77, 0x04, 0xB4, 0x4C, 0xCD, 0x21, 0xFC, 0xBF, 0x00, 0x41, 0xBE,
        0x00, 0x01, 0xB9,
    ];

    static COM_ICE: Subformat = Subformat::new("MS-DOS COM (ICE packed)");
    static COM_TPACK: Subformat = Subformat::new("MS-DOS COM (T-PACK packed)");
    static COM_SHRINK: Subformat = Subformat::new("MS-DOS COM (SHRINK packed)");
    static COM_COMPACT: Subformat = Subformat::new("MS-DOS COM (COMPACT packed)");
    static COM_UPX: Subformat = Subformat::new("MS-DOS COM (UPX packed)");
    static COM_PARAGRAPH_RELOC: Subformat = Subformat::new("MS-DOS COM (paragraph relocation)");
    static COM_THEDRAW: Subformat = Subformat::new("TheDraw ANSI art COM");
    static COM_TURBO_PASCAL_DOS: Subformat = Subformat::new("Turbo Pascal 3.0 COM (MS-DOS)");
    static COM_TURBO_PASCAL_CPM: Subformat = Subformat::new("Turbo Pascal 3.0 COM (CP/M-80)");
    static COM_DOS_VERSION_CHECK: Subformat = Subformat::new("MS-DOS COM (DOS version check)");

    require!((16..=MAX_DOS_COM_SIZE).contains(&size), prefix.len() >= 16);
    if prefix.len() >= 512 && prefix[510] == 0x55 && prefix[511] == 0xAA {
        return None;
    }
    if size >= 64 && starts(prefix, &[0xEB, 0x0A]) && eq_at(prefix, 12, ICE_STUB) {
        return with_subformat(Label::Pebin, &COM_ICE);
    }
    if size >= 64 && starts(prefix, TPACK_STUB) {
        return with_subformat(Label::Pebin, &COM_TPACK);
    }
    if size >= 64 && starts(prefix, SHRINK_STUB) {
        return with_subformat(Label::Pebin, &COM_SHRINK);
    }
    if size >= 48 && starts(prefix, COMPACT_PEICHL_STUB) {
        return with_subformat(Label::Pebin, &COM_COMPACT);
    }
    if size >= 56
        && prefix.len() >= 34
        && starts(prefix, &[0x81, 0xFC])
        && eq_at(prefix, 4, &[0x77, 0x02, 0xCD, 0x20, 0xB9])
        && u8_at(prefix, 11) == Some(0xBE)
        && u8_at(prefix, 14) == Some(0xBF)
        && eq_at(prefix, 17, &[0xFD, 0xF3, 0xA4, 0xFC])
    {
        return with_subformat(Label::Pebin, &COM_UPX);
    }
    if size >= 32
        && (starts(prefix, &[0x9C, 0x55, 0x56, 0x8C])
            || starts(prefix, &[0xEB, 0x3D, 0x90, 0x00])
            || starts(prefix, &[0xE9, 0x79, 0x2C, 0x90])
            || starts(prefix, &[0xE9, 0x87, 0x2B, 0x90]))
    {
        let sub = if starts(prefix, &[0x9C, 0x55, 0x56, 0x8C]) {
            &COM_PARAGRAPH_RELOC
        } else if starts(prefix, &[0xEB, 0x3D, 0x90, 0x00]) {
            &COM_THEDRAW
        } else {
            &COM_TURBO_PASCAL_DOS
        };
        return with_subformat(Label::Pebin, sub);
    }
    if size >= 64
        && prefix.len() >= 64
        && starts(prefix, &[0xC3, 0xE2, 0x20, 0xCD])
        && contains(get_slice(prefix, 4, 64)?, b"Copyright")
    {
        return with_subformat(Label::Pebin, &COM_TURBO_PASCAL_CPM);
    }
    if starts(prefix, &[0xB4, 0x30, 0xCD, 0x21])
        && matches!(
            u8_at(prefix, 4)?,
            0x3C | 0x72 | 0x73 | 0x74 | 0x75 | 0x80 | 0x86 | 0x8A | 0xA2 | 0xA3
        )
    {
        return with_subformat(Label::Pebin, &COM_DOS_VERSION_CHECK);
    }
    None
}

/// Standalone AutoIt v3 compiled script (`\xA3\x48\x4B\xBE\x98\x6C\x4A\xA9...` at `@0`).
pub(crate) fn autoit_compiled(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 24, prefix.len() >= 20, starts(prefix, AUTOIT_V3_MAGIC));
    static AUTOIT_A3X: Subformat = Subformat::new("AutoIt v3 compiled script (A3X)");
    with_subformat(Label::Autoit, &AUTOIT_A3X)
}

/// AutoIt v3 Script (`autoit`): `#cs`, `#include <`, `AutoItSetOption(`, or `Opt("` script markers.
pub(crate) fn autoit_script(prefix: &[u8], size: u64) -> Option<Label> {
    const AU3_COMMENT_START: &[u8] = b"#cs";
    const AU3_INCLUDE_OPEN: &[u8] = b"#include <";
    const AU3_SET_OPTION: &[u8] = b"AutoItSetOption(";
    const AU3_MSGBOX: &[u8] = b"MsgBox(";
    const AU3_ENDFUNC: &[u8] = b"EndFunc";
    const AU3_LOCAL_VAR: &[u8] = b"Local $";
    const AU3_GLOBAL_VAR: &[u8] = b"Global $";
    const AU3_COMMENT_END: &[u8] = b"#ce";

    require!(
        size >= 16,
        prefix.len() >= 16,
        (starts(prefix, AU3_COMMENT_START)
            || starts(prefix, AU3_INCLUDE_OPEN)
            || contains(prefix, AU3_SET_OPTION)
            || contains(prefix, AU3_MSGBOX))
            && (contains(prefix, AU3_ENDFUNC)
                || contains(prefix, AU3_LOCAL_VAR)
                || contains(prefix, AU3_GLOBAL_VAR)
                || contains(prefix, AU3_COMMENT_END)),
        is_utf8_text_header(prefix, size as usize),
    );
    static AUTOIT_SCRIPT: Subformat = Subformat::new("AutoIt v3 script");
    with_subformat(Label::Autoit, &AUTOIT_SCRIPT)
}

/// Intel 8086 Relocatable Object Module Format (`omf`, 1981): `THEADR` (`0x80`), `LHEADR` (`0x82`),
/// or `LIBHDR` (`0xF0`) record with length-prefixed printable module name or power-of-two library page.
pub(crate) fn omf_intel(prefix: &[u8], size: u64) -> Option<Label> {
    static OMF_LIBRARY: Subformat = Subformat::new("OMF library");
    static OMF_OBJECT: Subformat = Subformat::new("OMF object module");

    require!(size >= 16, prefix.len() >= 12);
    if u8_at(prefix, 0)? == 0xF0 {
        let rec_len = le16(prefix, 1)? as usize;
        let page_size = rec_len + 3;
        require!(
            (16..=32768).contains(&page_size),
            page_size.is_power_of_two(),
            (page_size as u64) < size,
        );
        if page_size + 4 <= prefix.len() {
            let next_type = u8_at(prefix, page_size)?;
            let next_len = le16(prefix, page_size + 1)? as usize;
            let name_len = u8_at(prefix, page_size + 3)? as usize;
            require!(
                matches!(next_type, 0x80 | 0x82),
                next_len == name_len + 2,
                (1..=128).contains(&name_len),
            );
        } else {
            let pad = get_slice(prefix, 11, prefix.len().min(page_size))?;
            require!(pad.iter().all(|&b| b == 0));
        }
        return with_subformat(Label::Omf, &OMF_LIBRARY);
    }
    require!(matches!(u8_at(prefix, 0)?, 0x80 | 0x82));
    let rec_len = le16(prefix, 1)? as usize;
    let name_len = u8_at(prefix, 3)? as usize;
    require!(
        rec_len >= 3,
        rec_len == name_len + 2,
        (1..=128).contains(&name_len),
        prefix.len() >= 4 + rec_len,
    );
    let name = get_slice(prefix, 4, 4 + name_len)?;
    require!(name.iter().all(|&b| (0x20..=0x7E).contains(&b)));
    let next_tag = u8_at(prefix, 3 + rec_len)?;
    require!(matches!(next_tag, 0x88 | 0x8A | 0x96 | 0x98 | 0x99 | 0x9A | 0xA0 | 0xB0));
    with_subformat(Label::Omf, &OMF_OBJECT)
}

/// Self-executing Python Archive (`.par` / `.pyz`, `pythonpar`, 2015): `#!` Python shebang header
/// followed by a PKZIP local header (`PK\x03\x04`) or trailing End of Central Directory (`PK\x05\x06`).
pub(crate) fn pythonpar(prefix: &[u8], size: u64, tail: Option<&[u8]>) -> Option<Label> {
    const SHEBANG_MAGIC: &[u8; 2] = b"#!";
    const PYTHON_INTERP: &[u8; 6] = b"python";

    require!(
        size >= 64,
        prefix.len() >= 32,
        starts(prefix, SHEBANG_MAGIC) && contains(prefix, PYTHON_INTERP),
    );
    require!(
        contains(prefix, ZIP_LOCAL_SIGNATURE)
            || tail.is_some_and(|t| contains(t, ZIP_EOCD_SIGNATURE))
    );
    Some(Label::Pythonpar)
}

/// Research Unix / BSD `a.out` executable (`aout`, 1971): `OMAGIC` (`0x0107`), `NMAGIC` (`0x0108`),
/// `ZMAGIC` (`0x010B`), or `QMAGIC` (`0x00CC`) header (LE or BE) with valid `a_text`, `a_data`, and `a_bss` sizes.
pub(crate) fn aout(prefix: &[u8], size: u64) -> Option<Label> {
    const OMAGIC: u32 = 0x0107;
    const NMAGIC: u32 = 0x0108;
    const ZMAGIC: u32 = 0x010B;
    const QMAGIC: u32 = 0x00CC;

    require!(size >= 16, prefix.len() >= 16);
    let m_be = u32::from_be_bytes([prefix[0], prefix[1], prefix[2], prefix[3]]);
    let m_le = u32::from_le_bytes([prefix[0], prefix[1], prefix[2], prefix[3]]);
    let is_be = matches!(m_be & 0xFFFF, OMAGIC | NMAGIC | ZMAGIC | QMAGIC) && (m_be >> 16) <= 300;
    let is_le = matches!(m_le & 0xFFFF, OMAGIC | NMAGIC | ZMAGIC | QMAGIC) && (m_le >> 16) <= 300;
    require!(is_be || is_le);
    let (t_sz, d_sz, b_sz) = if is_be {
        (be32(prefix, 4)?, be32(prefix, 8)?, be32(prefix, 12)?)
    } else {
        (le32(prefix, 4)?, le32(prefix, 8)?, le32(prefix, 12)?)
    };
    require!(
        t_sz > 0 && t_sz <= 64 * 1024 * 1024,
        d_sz <= 64 * 1024 * 1024,
        b_sz <= 64 * 1024 * 1024,
    );
    let _ = get_slice(prefix, 0, 16);
    static AOUT_OMAGIC: Subformat = Subformat::new("a.out OMAGIC");
    static AOUT_NMAGIC: Subformat = Subformat::new("a.out NMAGIC");
    static AOUT_ZMAGIC: Subformat = Subformat::new("a.out ZMAGIC");
    static AOUT_QMAGIC: Subformat = Subformat::new("a.out QMAGIC");
    let sub = match (if is_be { m_be } else { m_le }) & 0xFFFF {
        OMAGIC => &AOUT_OMAGIC,
        NMAGIC => &AOUT_NMAGIC,
        ZMAGIC => &AOUT_ZMAGIC,
        _ => &AOUT_QMAGIC,
    };
    with_subformat(Label::Aout, sub)
}

/// Disambiguates `MZ`/`ZM` executables between Windows PE (`pebin`), AutoIt v3 (`autoit`), and DOS MZ (`pebin`).
#[inline]
pub(crate) fn pe_or_dos_mz(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    probe(|| pe(prefix, ctx)).or_else(|| probe(|| pe_or_autoit(prefix, ctx)))
}
