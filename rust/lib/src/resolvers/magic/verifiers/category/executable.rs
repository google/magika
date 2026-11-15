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

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const AAPT2_COUNT_OFFSET: usize = 8;
const AAPT2_ENTRY_LEN_OFFSET: usize = 16;
const AAPT2_ENTRY_TYPE_OFFSET: usize = 12;
const AAPT2_FLAT_MAGIC: &[u8; 8] = bytes!(b"AAPT", &[0x01, 0x00, 0x00, 0x00]);
const ALTO_RUN_BLV_OFFSET: usize = 80;
const ALTO_RUN_RESERVED_LEN: usize = 20;
const ALTO_RUN_RESERVED_OFFSET: usize = 8;
const APOLLO_SYSBOOT_MAGIC: &[u8; 12] = b"SYSBOOT REV ";
const APPLE_NIBARCHIVE_MAGIC: &[u8; 14] = bytes!(b"NIBArchive", &[0x01, 0x00, 0x00, 0x00]);
const BORLAND_BGI_V2_MAGIC: &[u8; 4] = bytes!(b"pk", &[0x08, 0x08]);
const BORLAND_BGI_V3_HEADER_8: &[u8; 8] = bytes!(b"FBGD", &[0x08, 0x08, 0x08, 0x08]);
const BORLAND_TPU0_MAGIC: &[u8; 4] = b"TPU0";
const BORLAND_TPU5_MAGIC: &[u8; 4] = b"TPU5";
const BORLAND_TPU6_MAGIC: &[u8; 4] = b"TPU6";
const BORLAND_TPU9_MAGIC: &[u8; 4] = b"TPU9";
const BORLAND_TPUQ_MAGIC: &[u8; 4] = b"TPUQ";
const CLICKTEAM_PAME_MAGIC: &[u8; 4] = b"PAME";
const CLICKTEAM_PAMU_MAGIC: &[u8; 4] = b"PAMU";
const CTOS_RUN_MAGIC: &[u8; 2] = b"GW";
const CUDA_FATBIN_MAGIC_LE: &[u8; 4] = le32_bytes!(0xBA55_ED50);
const CUDA_FATBIN_WRAPPER_MAGIC_LE: &[u8; 4] = le32_bytes!(0x4662_43B1);
const DARPA_CGC_MAGIC: &[u8; 4] = bytes!(&[0x7F], b"CGC");
const DELPHI_DCU_D2005_MAGIC: &[u8; 4] = hex2bin!("0D 00 00 11");
const DELPHI_DCU_D2006_MAGIC: &[u8; 4] = hex2bin!("23 00 00 12");
const DELPHI_DCU_D2009_MAGIC: &[u8; 4] = hex2bin!("39 00 00 14");
const DELPHI_DCU_D2010_MAGIC: &[u8; 4] = hex2bin!("45 00 00 15");
const DELPHI_DCU_D2_MAGIC: &[u8; 4] = b"HSPP";
const DELPHI_DCU_D3_MAGIC: &[u8; 4] = hex2bin!("41 86 51 44");
const DELPHI_DCU_D4_MAGIC: &[u8; 4] = hex2bin!("D8 A6 68 47");
const DELPHI_DCU_D5_MAGIC: &[u8; 4] = hex2bin!("8B 14 1F F2");
const DELPHI_DCU_D6_MAGIC: &[u8; 4] = hex2bin!("DD 00 00 0E");
const DELPHI_DCU_D7_MAGIC: &[u8; 4] = hex2bin!("DF 00 00 0F");
const DELPHI_DCU_KYLIX1_MAGIC: &[u8; 4] = hex2bin!("8C 14 1F F2");
const HERMES_MAGIC_8: &[u8; 8] = le64_bytes!(0x1F19_03C1_03BC_1FC6);
const HPUX_SOM_VERSION_NEW: u32 = 87_102_412;
const HPUX_SOM_VERSION_OLD: u32 = 85_082_112;
const MIN_AAPT2_FLAT_SIZE: u64 = 32;
const MIN_ALTO_RUN_SIZE: u64 = 512;
const MIN_AMIGA_CDTV_RESIDENT_TM_SIZE: u64 = 32;
const MIN_ANDROID_ART_SIZE: u64 = 16;
const MIN_ANDROID_VDEX_SIZE: u64 = 24;
const MIN_APOLLO_SYSBOOT_SIZE: u64 = 512;
const MIN_APPLE_METALLIB_SIZE: u64 = 24;
const MIN_APPLE_NIBARCHIVE_SIZE: u64 = 50;
const MIN_ARM_VECTOR_ROM_SIZE: u64 = 48;
const MIN_AWARD_BIOS_STF_SIZE: u64 = 32;
const MIN_BINJA_BNTL_SIZE: u64 = 16;
const MIN_BORLAND_BGI_DRIVER_SIZE: u64 = 0xA0 + 1;
const MIN_BORLAND_DPMI_OVERLAY_OVY_SIZE: u64 = 48;
const MIN_BORLAND_TB_TBC_SIZE: u64 = 32;
const MIN_BORLAND_TDS_SIZE: u64 = 16;
const MIN_BORLAND_TPU_SIZE: u64 = 64;
const MIN_BORLAND_TP_OVR_SIZE: u64 = 32;
const MIN_CLANG_OFFLOAD_BUNDLE_SIZE: u64 = 20;
const MIN_CLICKTEAM_CCN_SIZE: u64 = 32;
const MIN_CTOS_RUN_SIZE: u64 = 512;
const MIN_CUDA_FATBIN_SIZE: u64 = 64;
const MIN_DARPA_CGC_SIZE: u64 = 52;
const MIN_DEFINICON_DSI32_E32_SIZE: u64 = 64;
const MIN_DELL_DIAGNOSTICS_MLM_SIZE: u64 = 32;
const MIN_DELPHI_DCU_SIZE: u64 = 24;
const MIN_DG_NOVA_RB_SIZE: u64 = 36;
const MIN_DOS16M_BW_SIZE: u64 = 48;
const MIN_FANUC_PMC_PATCH_SIZE: u64 = 64;
const MIN_FLATPAK_BUNDLE_SIZE: u64 = 32;
const MIN_GEOWORKS_GEOS_SIZE: u64 = 200;
const MIN_GOFF_OBJECT_SIZE: u64 = 160;
const MIN_HERMES_BYTECODE_SIZE: u64 = 128;
const MIN_HP2100_ABS_SIZE: u64 = 24;
const MIN_HP2100_REL_SIZE: u64 = 24;
const MIN_HP48_SIZE: u64 = 11;
const MIN_HPUX_SOM_SIZE: u64 = 128;
const MIN_HP_PALMTOP_EXM_SIZE: u64 = 64;
const MIN_IBMPC_OVL_SIZE: u64 = 32;
const MIN_LLVM_GSYM_SIZE: u64 = 40;
const MIN_LLVM_PROFDATA_SIZE: u64 = 24;
const MIN_MICROFOCUS_GNT_SIZE: u64 = 36;
const MIN_MOBICORE_MCLF_SIZE: u64 = 128;
const MIN_MRUBY_RITE_SIZE: u64 = 34;
const MIN_MVS_OBJECT_DECK_SIZE: u64 = 160;
const MIN_NASM_RDOFF_SIZE: u64 = 18;
const MIN_NOKIA_BB5_PPMG_SIZE: u64 = 64;
const MIN_NOVELL_NETWARE_RPL_ROM_SIZE: u64 = 32;
const MIN_OSASK_HRB_EXEC_SIZE: u64 = 32;
const MIN_PCVIC_SNAPSHOT_SIZE: u64 = 64;
const MIN_PDP11_LDA_SIZE: u64 = 14;
const MIN_PDP8_SV_SIZE: u64 = 1024;
const MIN_PDSE_PROGRAM_OBJECT_SIZE: u64 = 60;
const MIN_PERF_JITDUMP_SIZE: u64 = 40;
const MIN_PERQ_POS_SEG_SIZE: u64 = 512;
const MIN_PSION_SIBO_APP_SIZE: u64 = 64;
const MIN_PSP_PBP_SIZE: u64 = 60;
const MIN_PSX_EXE_SIZE: u64 = 2048;
const MIN_QUALCOMM_QCDT_SIZE: u64 = 32;
const MIN_RADDBG_RDI_SIZE: u64 = 56;
const MIN_SQUIRREL_CNUT_SIZE: u64 = 32;
const MIN_UBOOT_UIMAGE_SIZE: u64 = 64;
const MIN_UCLINUX_BFLT_SIZE: u64 = 64;
const MIN_VMS_EXE_SIZE: u64 = 1024;
const MIN_WAMR_AOT_SIZE: u64 = 28;
const MIN_XAMARIN_XALZ_SIZE: u64 = 32;
const MIN_XBOX_XBE_SIZE: u64 = 376;
const MOBICORE_MCLF_MAGIC: &[u8; 4] = b"MCLF";
const NIB_MINOR_OFFSET: usize = 14;
const NIB_OBJ_COUNT_OFFSET: usize = 18;
const NIB_OBJ_OFF_OFFSET: usize = 22;
const OS8_BLOCK_SIZE: usize = 512;
const PAPER_TAPE_MAX_LEADER: usize = 512;
const PERF_JITDUMP_HDR_SIZE_OFFSET: usize = 8;
const PERF_JITDUMP_LE_MAGIC: &[u8; 4] = b"DTiJ";
const PERF_JITDUMP_MACH_OFFSET: usize = 12;
const PERF_JITDUMP_VER_OFFSET: usize = 4;
const PSX_EXE_SIG: &[u8; 8] = b"PS-X EXE";
const PSX_SCE_EXE_SIG: &[u8; 8] = b"SCE EXE\x00";
const UBOOT_UIMAGE_MAGIC: &[u8; 4] = be32_bytes!(0x2705_1956);
const VMS_EXE_MAJOR_ID: &[u8; 2] = b"02";
const XAMARIN_XALZ_MAGIC: &[u8; 4] = b"XALZ";

/// Google Android AAPT2 Compiled Resource Container (`.flat`).
pub(crate) fn verify_aapt2_flat(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AAPT2_FLAT_SIZE,
        prefix.len() >= 24,
        starts(prefix, AAPT2_FLAT_MAGIC),
        (1..=65_536).contains(&le32(prefix, AAPT2_COUNT_OFFSET)?),
        le32(prefix, AAPT2_ENTRY_TYPE_OFFSET)? <= 1,
        le64(prefix, AAPT2_ENTRY_LEN_OFFSET)? >= 4
    );
    Some(Label::Aapt2Flat)
}

/// Xerox Alto BCPL Save Version Executable (`.run`, 1974) Xerox PARC.
///
/// Big-endian `SV` image (`BCPLFILES.D`): `w0` (`startingAddress`) `== w42` (`BLV.startOfCode`),
/// `w1` (`nPages`) bounded by file size, `w2 == 0`, 20 zero bytes at `@8..28`, and monotonic
/// `BLV` bounds (`w40 < w41 < w42 < w43 == w44`).
pub(crate) fn verify_alto_run(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ALTO_RUN_SIZE,
        size.is_multiple_of(2),
        prefix.len() >= ALTO_RUN_BLV_OFFSET + 10
    );
    let w0 = be16(prefix, 0)?;
    let w1 = be16(prefix, 2)?;
    let w2 = be16(prefix, 4)?;
    require!(
        w0 >= 0x0100,
        w1 >= 1,
        (w1 as u64) * 512 <= size + 512,
        w2 == 0,
        get_slice(
            prefix,
            ALTO_RUN_RESERVED_OFFSET,
            ALTO_RUN_RESERVED_OFFSET + ALTO_RUN_RESERVED_LEN
        )?
        .iter()
        .all(|&b| b == 0)
    );
    let w40 = be16(prefix, ALTO_RUN_BLV_OFFSET)?;
    let w41 = be16(prefix, ALTO_RUN_BLV_OFFSET + 2)?;
    let w42 = be16(prefix, ALTO_RUN_BLV_OFFSET + 4)?;
    let w43 = be16(prefix, ALTO_RUN_BLV_OFFSET + 6)?;
    let w44 = be16(prefix, ALTO_RUN_BLV_OFFSET + 8)?;
    require!(w40 >= 0x0100, w40 < w41, w41 < w42, w0 == w42, w42 < w43, w43 == w44);
    Some(Label::AltoRun)
}

/// Commodore Amiga CDTV Resident Trademark / Device Module (`CDTV.TM`, 1991) Commodore-Amiga.
pub(crate) fn verify_amiga_cdtv_resident_tm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_AMIGA_CDTV_RESIDENT_TM_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"Nu\x00\x00cdtv ")
    );
    Some(Label::AmigaCdtvResidentTm)
}

/// Android Runtime (ART) Pre-Initialized Heap Image (`.art`, 2014) Google (`"art\n"`).
pub(crate) fn verify_android_art(prefix: &[u8], size: u64) -> Option<Label> {
    (size >= MIN_ANDROID_ART_SIZE
        && prefix.len() >= 8
        && starts(prefix, b"art\n")
        && get_slice(prefix, 4, 7)?.iter().all(|b| b.is_ascii_digit())
        && u8_at(prefix, 7)? == 0)
        .then_some(Label::Art)
}

/// Android Runtime (ART) Verifier Dependencies DEX Container (`.vdex`, 2017) Google (`"vdex"`).
pub(crate) fn verify_android_vdex(prefix: &[u8], size: u64) -> Option<Label> {
    (size >= MIN_ANDROID_VDEX_SIZE
        && prefix.len() >= 8
        && starts(prefix, b"vdex")
        && get_slice(prefix, 4, 7)?.iter().all(|b| b.is_ascii_digit())
        && u8_at(prefix, 7)? == 0)
        .then_some(Label::Vdex)
}

/// Apollo Domain/OS `SYSBOOT` Boot Tape Image (`.ct` / `.img`, 1985 Apollo Computer).
///
/// Header layout:
/// - `@0x00..0x0C`: three `u32 BE` memory addresses (`0 < load_addr < entry_addr < end_addr`)
/// - `@0x10..0x1C`: `"SYSBOOT REV "` ASCII magic
/// - `@0x1C..0x28`: printable ASCII revision string and target architecture tag (`M68K`, `A88K`, `PRIS`).
pub(crate) fn verify_apollo_sysboot(prefix: &[u8], size: u64) -> Option<Label> {
    const LOAD_OFF: usize = 0x00; // @0x00: u32 BE load address
    const ENTRY_OFF: usize = 0x04; // @0x04: u32 BE entry point
    const END_OFF: usize = 0x08; // @0x08: u32 BE end address
    const MAGIC_OFF: usize = 0x10; // @0x10: "SYSBOOT REV "
    const REV_OFF: usize = 0x1C; // @0x1C..0x20: 4-byte binary revision / zero padding
    const ARCH_OFF: usize = 0x20; // @0x20..0x28: ASCII CPU architecture tag (e.g. " M68K   ")
    const HDR_END: usize = 0x28; // @0x28: end of revision + CPU tag

    require!(
        size >= MIN_APOLLO_SYSBOOT_SIZE,
        prefix.len() >= HDR_END,
        eq_at(prefix, MAGIC_OFF, APOLLO_SYSBOOT_MAGIC),
        has_non_ascii_text_byte(prefix.get(..MAGIC_OFF)?)
    );
    let load = be32(prefix, LOAD_OFF)?;
    let entry = be32(prefix, ENTRY_OFF)?;
    let end = be32(prefix, END_OFF)?;
    require!(
        load > 0 && load < entry && entry < end,
        get_slice(prefix, REV_OFF, ARCH_OFF)?.iter().all(|&b| b == 0 || (0x20..=0x7E).contains(&b)),
        get_slice(prefix, ARCH_OFF, HDR_END)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::ApolloSysboot)
}

/// Apple Metal Compiled GPU Shader Library (`.metallib`, 2014) Apple (`"MTLB"`).
pub(crate) fn verify_apple_metallib(prefix: &[u8], size: u64) -> Option<Label> {
    (size >= MIN_APPLE_METALLIB_SIZE
        && prefix.len() >= 16
        && starts(prefix, b"MTLB")
        && has_non_ascii_text_byte(prefix.get(4..16)?))
    .then_some(Label::Metallib)
}

/// Apple macOS / iOS Compiled Interface Builder NIBArchive (`.nib`).
pub(crate) fn verify_apple_nibarchive(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APPLE_NIBARCHIVE_SIZE,
        prefix.len() >= 30,
        starts(prefix, APPLE_NIBARCHIVE_MAGIC),
        (8..=16).contains(&le32(prefix, NIB_MINOR_OFFSET)?),
        le32(prefix, NIB_OBJ_COUNT_OFFSET)? >= 1,
        le32(prefix, NIB_OBJ_OFF_OFFSET) == Some(50)
    );
    Some(Label::AppleNibarchive)
}

/// ARM32 Exception Vector Firmware ROM Image (.rom).
pub(crate) fn verify_arm_vector_rom(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ARM_VECTOR_ROM_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"\x58\xf0\x9f\xe5\x58\xf0\x9f\xe5\x58\xf0\x9f\xe5\x58\xf0\x9f\xe5\x58\xf0\x9f\xe5\0\0\0\0\x58\xf0\x9f\xe5\x58\xf0\x9f\xe5"),
        get_slice(prefix, 36, 48)? == b"\0\0\0\0\0\0\0\0\0\0\0\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::ArmVectorRom)
}

/// Award Software BIOS Setup Language Module (_XX_CODE.BIN, .stf).
pub(crate) fn verify_award_bios_stf(prefix: &[u8], size: u64) -> Option<Label> {
    const CODE_BIN_SUFFIX: &[u8; 13] = b"_CODE.BIN\0\0\0\0"; // @3..16: region-independent firmware image tag + 4 NULs

    require!(
        size >= MIN_AWARD_BIOS_STF_SIZE,
        prefix.len() >= 32,
        u8_at(prefix, 0)? == b'_',
        u8_at(prefix, 1)?.is_ascii_alphanumeric(),
        u8_at(prefix, 2)?.is_ascii_alphanumeric(),
        eq_at(prefix, 3, CODE_BIN_SUFFIX),
        has_non_ascii_text_byte(prefix.get(16..32)?)
    );
    Some(Label::AwardBiosStf)
}

/// Vector 35 Binary Ninja Type Library (.bntl).
pub(crate) fn verify_binja_bntl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BINJA_BNTL_SIZE, prefix.len() >= 12, starts(prefix, b"BNTL"));
    // After "BNTL" (@0..4) is either zlib (0x78 0x01/0x5E/0x9C/0xDA), zstd (28 B5 2F FD), or binary version (1..=8)
    let b4 = u8_at(prefix, 4)?;
    let b5 = u8_at(prefix, 5)?;
    let is_zlib = b4 == 0x78
        && matches!(b5, 0x01 | 0x5E | 0x9C | 0xDA)
        && has_non_ascii_text_byte(prefix.get(4..12)?);
    let is_zstd = get_slice(prefix, 4, 8)? == hex2bin!("28 B5 2F FD");
    let is_bin_ver = (1..=8).contains(&le32(prefix, 4).unwrap_or(0));
    require!(is_zlib || is_zstd || is_bin_ver);
    Some(Label::Bntl)
}

/// Borland Graphics Interface Video/Graphics Driver (`.bgi`, 1987–1992 Borland).
///
/// Validates both BGI v2 real-mode drivers (`"pk\x08\x08"`, `header_size + code_size == size`)
/// and BGI v3 DPMI16 protected-mode drivers (`"FBGD\x08\x08\x08\x08"`, `header_size + code_size + data_size == size`).
pub(crate) fn verify_borland_bgi_driver(prefix: &[u8], size: u64) -> Option<Label> {
    const COPYRIGHT_END: usize = 0x80; // @0..0x80: ASCII copyright text terminated by 0x1A
    const HDR_SIZE_OFF: usize = 0x80; // @0x80: u16 LE header_size (>= 0x00A0)
    const CODE_SIZE_OFF: usize = 0x84; // @0x84: u16 LE code_size (> 0)
    const DATA_SIZE_OFF: usize = 0x8C; // @0x8C: u16 LE data_size (v3 DPMI16 only)
    const MIN_HDR_SIZE: u16 = 0x00A0;

    require!(size >= MIN_BORLAND_BGI_DRIVER_SIZE, prefix.len() >= 0x8E);

    let (is_v3, scan_start) = if starts(prefix, BORLAND_BGI_V3_HEADER_8) {
        (true, 8usize)
    } else if starts(prefix, BORLAND_BGI_V2_MAGIC) {
        (false, 4usize)
    } else {
        return None;
    };

    get_slice(prefix, scan_start, COPYRIGHT_END)?.iter().position(|&b| b == 0x1A)?;
    let hdr_size = le16(prefix, HDR_SIZE_OFF)?;
    let code_size = le16(prefix, CODE_SIZE_OFF)?;
    require!(hdr_size >= MIN_HDR_SIZE, code_size > 0);

    if is_v3 {
        let data_size = le16(prefix, DATA_SIZE_OFF)?;
        require!((hdr_size as u64) + (code_size as u64) + (data_size as u64) == size);
    } else {
        require!((hdr_size as u64) + (code_size as u64) == size);
    }
    Some(Label::BorlandBgi)
}

/// Borland C++ / Pascal DPMI Protected-Mode Overlay (`.ovy`, 1991) Borland.
pub(crate) fn verify_borland_dpmi_overlay_ovy(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BORLAND_DPMI_OVERLAY_OVY_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"YO\x06\x00\x64\x00\x00\x00")
    );
    Some(Label::BorlandDpmiOverlayOvy)
}

/// Borland Turbo Basic Chain Overlay (`.tbc`, `"rz\x01\x00\x10\x01"`).
pub(crate) fn verify_borland_tb_tbc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BORLAND_TB_TBC_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"rz\x01\x00\x10\x01")
    );
    Some(Label::TbTbc)
}

/// Borland Turbo Pascal / Force Overlay (`.ovr`, `"TPOV"` or `"FBOV"`).
pub(crate) fn verify_borland_tp_ovr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BORLAND_TP_OVR_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"TPOV") || starts(prefix, b"FBOV"),
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::TpOvr)
}

/// Borland Turbo Pascal Compiled Unit (`.tpu` / `.tpp` / `.tpw`, 1987–1992 Borland).
///
/// Validates the compiler version magic (`TPU0`, `TPU5`, `TPU6`, `TPU9`, `TPUQ`), reserved zero
/// dword at `@4..8`, 16-byte paragraph alignment, and exact paragraph-rounded sum of all stored
/// sections (`sym`, `code`, `const`, optional `const_reloc`, `reloc`, optional `browser`) == `size`.
pub(crate) fn verify_borland_tpu(prefix: &[u8], size: u64) -> Option<Label> {
    const RESERVED_ZERO_OFF: usize = 4; // @4..8: reserved u32 LE == 0 across all TP 4..7 units
    const RESERVED_ZERO: &[u8; 4] = &[0, 0, 0, 0];

    #[inline]
    fn round_para(n: u16) -> u64 {
        ((n as u64) + 15) & !15
    }

    require!(
        size >= MIN_BORLAND_TPU_SIZE,
        size.is_multiple_of(16),
        prefix.len() >= 44,
        eq_at(prefix, RESERVED_ZERO_OFF, RESERVED_ZERO)
    );

    let (sec_off, sec_count): (usize, usize) = match get_slice(prefix, 0, 4)? {
        m if m == BORLAND_TPU0_MAGIC => (0x14, 4), // @0x14..0x1C: sym, code, const, reloc
        m if m == BORLAND_TPU5_MAGIC => (0x18, 4), // @0x18..0x20: sym, code, const, reloc
        m if m == BORLAND_TPU6_MAGIC => (0x1A, 5), // @0x1A..0x24: + const_reloc
        m if m == BORLAND_TPU9_MAGIC => (0x1C, 5), // @0x1C..0x26: + const_reloc
        m if m == BORLAND_TPUQ_MAGIC => (0x1E, 6), // @0x1E..0x2A: + browser
        _ => return None,
    };

    let sym_size = le16(prefix, sec_off)?;
    require!(sym_size >= 40);
    let mut total: u64 = 0;
    for i in 0..sec_count {
        total += round_para(le16(prefix, sec_off + i * 2)?);
    }
    require!(total == size);

    Some(Label::BorlandTpu)
}

/// Clang Compressed (`"CCOB"`) or Uncompressed (`"__CLANG_OFFLOAD_BUNDLE__"`) Offload Bundle.
pub(crate) fn verify_clang_offload_bundle(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_CLANG_OFFLOAD_BUNDLE_SIZE, prefix.len() >= 20);
    if starts(prefix, b"CCOB") {
        let ver = le16(prefix, 4)?;
        let comp = le16(prefix, 6)?;
        let sz = le32(prefix, 8)?;
        require!((1..=3).contains(&ver), matches!(comp, 1 | 2), sz > 0);
        return Some(Label::ClangOffloadBundle);
    }
    if size >= 48
        && prefix.len() >= 32
        && starts(prefix, b"__CLANG_OFFLOAD_BUNDLE__")
        && (1..=1024).contains(&le64(prefix, 24)?)
    {
        return Some(Label::ClangOffloadBundle);
    }
    None
}

/// Clickteam Multimedia Fusion / Fusion 2.5 Sub-Application (`.ccn` / `.cca`).
///
/// Header layout:
/// - `@0..4`: `"PAMU"` (Unicode, Fusion 2.5 / MMF2 Unicode) or `"PAME"` (ANSI, MMF2)
/// - `@4..6`: `u16 LE` `runtime_version` (`0x0302` or `0x0304`)
/// - `@6..8`: `u16 LE` `runtime_subversion`
pub(crate) fn verify_clickteam_ccn(prefix: &[u8], size: u64) -> Option<Label> {
    const RUNTIME_VER_OFF: usize = 4; // @4..6: u16 LE runtime version (0x0302 or 0x0304)

    require!(
        size >= MIN_CLICKTEAM_CCN_SIZE,
        prefix.len() >= 16,
        starts(prefix, CLICKTEAM_PAMU_MAGIC) || starts(prefix, CLICKTEAM_PAME_MAGIC),
        matches!(le16(prefix, RUNTIME_VER_OFF)?, 0x0302 | 0x0304)
    );
    Some(Label::ClickteamCcn)
}

/// Convergent CTOS / Burroughs BTOS Executable (`.Run` / `.sys`, 1981 Convergent Technologies).
///
/// Header layout:
/// - `@0x00..0x02`: `"GW"` (`0x5747` LE) signature
/// - `@0x02`: `u16 LE format_version` (`4..=6`)
/// - `@0x04`: `u16 LE page_count` (`>= 1`, `page_count * 512 <= size`)
/// - `@0x0A`: `u32 LE reserved == 0`
/// - `@0x1E`: `u16 LE sentinel == 0xFFFF`.
pub(crate) fn verify_ctos_run(prefix: &[u8], size: u64) -> Option<Label> {
    const VER_OFF: usize = 0x02; // @0x02: u16 LE version (4..=6)
    const PAGES_OFF: usize = 0x04; // @0x04: u16 LE 512-byte page count
    const ZERO_OFF: usize = 0x0A; // @0x0A: u32 LE reserved zero
    const SENTINEL_OFF: usize = 0x1E; // @0x1E: u16 LE 0xFFFF marker

    require!(
        size >= MIN_CTOS_RUN_SIZE,
        prefix.len() >= SENTINEL_OFF + 2,
        starts(prefix, CTOS_RUN_MAGIC),
        (4..=6).contains(&le16(prefix, VER_OFF)?),
        le32(prefix, ZERO_OFF) == Some(0),
        le16(prefix, SENTINEL_OFF) == Some(0xFFFF)
    );
    let pages = le16(prefix, PAGES_OFF)? as u64;
    require!(
        pages >= 1,
        pages * 512 <= size,
        has_non_ascii_text_byte(prefix.get(..SENTINEL_OFF + 2)?)
    );
    Some(Label::CtosRun)
}

/// NVIDIA CUDA Fatbinary GPU Code Container (.fatbin).
pub(crate) fn verify_cuda_fatbin(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_CUDA_FATBIN_SIZE && prefix.len() >= 24);
    if starts(prefix, CUDA_FATBIN_MAGIC_LE) {
        let version = le16(prefix, 4)?;
        let hdr_size = le16(prefix, 6)?;
        let fat_size = le64(prefix, 8)?;
        require!((1..=2).contains(&version) && hdr_size >= 16 && fat_size >= 16);
    } else if starts(prefix, CUDA_FATBIN_WRAPPER_MAGIC_LE) {
        let version = le32(prefix, 4)?;
        require!((1..=2).contains(&version));
    } else {
        return None;
    }
    Some(Label::CudaFatbin)
}

/// DARPA Cyber Grand Challenge DECREE Executable (`.cgc`, 2014 DARPA).
///
/// Header layout (`CGC32_Ehdr`, 52 bytes):
/// - `@0..4`: `0x7F "CGC"` magic
/// - `@4`: `EI_CLASS == 1` (32-bit)
/// - `@5`: `EI_DATA == 1` (little-endian)
/// - `@6`: `EI_VERSION == 1`
/// - `@7`: `EI_OSABI == b'C'` (`0x43`, DECREE OSABI)
/// - `@8`: `EI_ABIVERSION == 1`
/// - `@9..16`: `e_ident` padding (unconstrained)
/// - `@16..18`: `u16 LE` `e_type` (`1..=4`)
/// - `@18..20`: `u16 LE` `e_machine == 3` (`EM_386`)
/// - `@20..24`: `u32 LE` `e_version == 1`
/// - `@40..42`: `u16 LE` `e_ehsize == 52`
/// - `@42..44`: `u16 LE` `e_phentsize == 32` (when `e_phnum > 0`)
/// - `@44..46`: `u16 LE` `e_phnum`.
pub(crate) fn verify_darpa_cgc(prefix: &[u8], size: u64) -> Option<Label> {
    const EI_CLASS_OFF: usize = 4; // @4: u8 ELFCLASS32 (1)
    const EI_DATA_OFF: usize = 5; // @5: u8 ELFDATA2LSB (1)
    const EI_VERSION_OFF: usize = 6; // @6: u8 EV_CURRENT (1)
    const EI_OSABI_OFF: usize = 7; // @7: u8 'C' (0x43, DECREE)
    const EI_ABIVER_OFF: usize = 8; // @8: u8 ABI version (1)
    const E_TYPE_OFF: usize = 16; // @16: u16 LE ELF type (1..=4)
    const E_MACHINE_OFF: usize = 18; // @18: u16 LE EM_386 (3)
    const E_VERSION_OFF: usize = 20; // @20: u32 LE version (1)
    const E_EHSIZE_OFF: usize = 40; // @40: u16 LE header size (52)
    const E_PHENTSIZE_OFF: usize = 42; // @42: u16 LE program header entry size (32)
    const E_PHNUM_OFF: usize = 44; // @44: u16 LE program header count

    require!(
        size >= MIN_DARPA_CGC_SIZE,
        prefix.len() >= 52,
        starts(prefix, DARPA_CGC_MAGIC),
        u8_at(prefix, EI_CLASS_OFF)? == 1,
        u8_at(prefix, EI_DATA_OFF)? == 1,
        u8_at(prefix, EI_VERSION_OFF)? == 1,
        u8_at(prefix, EI_OSABI_OFF)? == b'C',
        u8_at(prefix, EI_ABIVER_OFF)? == 1,
        (1..=4).contains(&le16(prefix, E_TYPE_OFF)?),
        le16(prefix, E_MACHINE_OFF)? == 3,
        le32(prefix, E_VERSION_OFF)? == 1,
        le16(prefix, E_EHSIZE_OFF)? == 52
    );
    let phnum = le16(prefix, E_PHNUM_OFF)?;
    require!(phnum == 0 || le16(prefix, E_PHENTSIZE_OFF)? == 32);

    Some(Label::DarpaCgc)
}

/// Returns `true` if `machine` is a known Windows PE/COFF `IMAGE_FILE_MACHINE_*` value.
#[inline]
const fn is_known_pe_machine(machine: u16) -> bool {
    matches!(machine, 0x014C | 0x8664 | 0xAA64 | 0xA641 | 0xA642 | 0xA64E | 0x3A64 | 0x01C0)
        || matches!(machine, 0x01C2 | 0x01C4 | 0x0200 | 0x0EBC | 0x5032 | 0x5064 | 0x5128)
        || matches!(machine, 0x6232 | 0x6264 | 0x0184 | 0x0284 | 0x0162 | 0x0166 | 0x0168)
        || matches!(machine, 0x0169 | 0x0266 | 0x0366 | 0x0466 | 0x01F0 | 0x01F1 | 0x01A2)
        || matches!(machine, 0x01A3 | 0x01A6 | 0x01A8 | 0x01D3 | 0x9041 | 0x0520)
}

/// Microsoft Windows Separate Debug Information file (`.dbg`, `"DI"`).
pub(crate) fn verify_dbg(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_LEN: usize = 48;
    require!(size >= MIN_LEN as u64, prefix.len() >= MIN_LEN, starts(prefix, b"DI"));
    let flags = le16(prefix, 2)?;
    let machine = le16(prefix, 4)?;
    require!(
        (flags & !0x8003) == 0,
        is_known_pe_machine(machine),
        prefix.get(6..MIN_LEN)?.contains(&0),
    );
    Some(Label::Dbg)
}

/// Definicon DSI-32 NS32032 Executable (`.e32`, 1985 Definicon Systems coprocessor `a.out`-variant).
pub(crate) fn verify_definicon_dsi32_e32(prefix: &[u8], size: u64) -> Option<Label> {
    const DSI32_MAGIC: u32 = 0x0001_1E14; // @0..4: Definicon DSI-32 executable magic
    const MAX_ADDR_24BIT: u32 = 0x0100_0000; // NS32032 24-bit linear address space

    require!(size >= MIN_DEFINICON_DSI32_E32_SIZE, prefix.len() >= 32);
    let magic = le32(prefix, 0)?;
    let seg_end = le32(prefix, 4)?;
    let mem_top = le32(prefix, 8)?;
    let text_end = le32(prefix, 12)?;
    let data_end = le32(prefix, 16)?;
    let sec_cnt_a = le32(prefix, 20)?;
    let sec_cnt_b = le32(prefix, 24)?;
    let hdr_align = le32(prefix, 28)?;
    require!(
        magic == DSI32_MAGIC,
        text_end > 0,
        text_end <= data_end,
        data_end <= seg_end,
        seg_end <= mem_top,
        mem_top < MAX_ADDR_24BIT,
        (1..=256).contains(&sec_cnt_a),
        (sec_cnt_a..=512).contains(&sec_cnt_b),
        (0x10..=0x4000).contains(&hdr_align),
        hdr_align % 16 == 0
    );
    Some(Label::DefiniconDsi32E32)
}

/// Dell Modular System Diagnostics Module (`.mlm`, 1994) Dell.
pub(crate) fn verify_dell_diagnostics_mlm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DELL_DIAGNOSTICS_MLM_SIZE,
        prefix.len() >= 24,
        starts(prefix, &[0x7F, 0x18, 0x00, 0x00]),
        eq_at(prefix, 11, b"DellDiag.exe\x00")
    );
    Some(Label::DellDiagnosticsMlm)
}

/// Borland / Embarcadero Delphi Compiled Unit (`.dcu`, 1996 Borland).
///
/// Header layout:
/// - `@0..4`: 4-byte Delphi/Kylix version magic
/// - `@4..8`: `u32 LE` `FSize` — exact file size in bytes (`FSize == size`)
/// - `@8..12`: `u32 LE` `FTime` — DOS timestamp
/// - `@12..16`: unit flags and start of tagged DCU stream.
pub(crate) fn verify_delphi_dcu(prefix: &[u8], size: u64) -> Option<Label> {
    const FSIZE_OFF: usize = 4; // @4: u32 LE exact file byte length

    require!(size >= MIN_DELPHI_DCU_SIZE, prefix.len() >= 16);
    let magic4: &[u8; 4] = get_slice(prefix, 0, 4)?.try_into().ok()?;
    let known_magic = matches!(
        magic4,
        DELPHI_DCU_D2_MAGIC
            | DELPHI_DCU_D3_MAGIC
            | DELPHI_DCU_D4_MAGIC
            | DELPHI_DCU_D5_MAGIC
            | DELPHI_DCU_KYLIX1_MAGIC
            | DELPHI_DCU_D6_MAGIC
            | DELPHI_DCU_D7_MAGIC
            | DELPHI_DCU_D2005_MAGIC
            | DELPHI_DCU_D2006_MAGIC
            | DELPHI_DCU_D2009_MAGIC
            | DELPHI_DCU_D2010_MAGIC
    );
    require!(
        known_magic,
        le32(prefix, FSIZE_OFF)? as u64 == size,
        has_non_ascii_text_byte(prefix.get(..8)?)
    );

    Some(Label::DelphiDcu)
}

pub(crate) fn verify_dg_nova_rb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_DG_NOVA_RB_SIZE);
    let mut p = 0usize;
    while p < prefix.len() && p < PAPER_TAPE_MAX_LEADER && u8_at(prefix, p)? == 0 {
        p += 1;
    }
    let mut blocks = 0usize;
    while blocks < 3 {
        let btype = le16(prefix, p)?;
        let neg_cnt = le16(prefix, p + 2)?;
        let nwords = 0u16.wrapping_sub(neg_cnt) as usize;
        require!((1..=12).contains(&btype), (1..=64).contains(&nwords));
        if blocks == 0 {
            // Standard DG Nova .RB begins with a TITLE block (type 7, count -3) or valid header block.
            require!(matches!(btype, 1..=9));
        }
        let rec_len = (6 + nwords) * 2;
        require!(p + rec_len <= prefix.len());
        let mut sum = 0u16;
        for i in 0..(6 + nwords) {
            sum = sum.wrapping_add(le16(prefix, p + i * 2)?);
        }
        require!(sum == 0);
        blocks += 1;
        p += rec_len;
    }
    Some(Label::DgNovaRb)
}

/// Rational Systems / Tenberry Software DOS/16M Protected-Mode `BW` Executable (`.exp` / `.exe`, 1987).
pub(crate) fn verify_dos16m_bw(prefix: &[u8], size: u64) -> Option<Label> {
    const BW_MAGIC: &[u8; 2] = b"BW"; // @0..2: "BW" signature
    const REM_OFFSET: usize = 2; // @2..4: u16 LE bytes on last 512B page (< 512)
    const PAGES_OFFSET: usize = 4; // @4..6: u16 LE 512B page count (>= 1)
    const NEXT_HDR_OFFSET: usize = 28; // @28..32: u32 LE spliced payload offset / end of BW image

    require!(size >= MIN_DOS16M_BW_SIZE, prefix.len() >= 36, starts(prefix, BW_MAGIC));
    let rem = le16(prefix, REM_OFFSET)? as u64;
    let pages = le16(prefix, PAGES_OFFSET)? as u64;
    let next_hdr = le32(prefix, NEXT_HDR_OFFSET)? as u64;
    let calc_size = if rem == 0 { pages * 512 } else { (pages.saturating_sub(1)) * 512 + rem };
    require!(
        rem < 512,
        pages >= 1,
        (calc_size >= 48 && calc_size <= size) || (next_hdr >= 48 && next_hdr <= size),
        has_non_ascii_text_byte(prefix.get(2..32)?)
    );
    Some(Label::Dos16mBw)
}

/// Fanuc PMC CNC Machine Controller Patch (.bfd/.kcf).
pub(crate) fn verify_fanuc_pmc_patch(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FANUC_PMC_PATCH_SIZE,
        prefix.len() >= 56,
        get_slice(prefix, 0, 6)? == hex2bin!("00 00 00 00 00 00"),
        get_slice(prefix, 8, 14)? == hex2bin!("00 00 00 02 00 00"),
        get_slice(prefix, 16, 22)? == hex2bin!("00 00 00 09 00 00"),
        get_slice(prefix, 24, 34)? == hex2bin!("00 00 00 01 16 00 00 00 00 10"),
        get_slice(prefix, 35, 45)? == hex2bin!("00 45 34 30 34 30 38 39 2E 50"),
        get_slice(prefix, 47, 56)? == hex2bin!("00 00 00 00 00 00 00 40 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..56)?));
    Some(Label::FanucPmcPatch)
}

/// Flatpak Single-File Application Bundle (`.flatpak` / `.xdgapp`, `"flatpak\0\x01\0\x89\xe5"` / `"xdg-app\0\x01\0\x89\xe5"`).
pub(crate) fn verify_flatpak_bundle(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FLATPAK_BUNDLE_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"flatpak\0") || starts(prefix, b"xdg-app\0"),
        starts(prefix.get(8..)?, &[0x01, 0x00, 0x89, 0xE5]) || be32(prefix, 8) == Some(1)
    );
    Some(Label::FlatpakBundle)
}

/// PC/GEOS Geoworks GeoFile Executable or VM Document (`.geo` / `.vm`, 1990 Geoworks).
pub(crate) fn verify_geoworks_geos(prefix: &[u8], size: u64) -> Option<Label> {
    const GEOS_V1_MAGIC: &[u8; 4] = hex2bin!("C7 45 C1 53"); // @0..4: GEOS 1.x magic ('G'|0x80, 'E', 'A'|0x80, 'S')
    const GEOS_V2_MAGIC: &[u8; 4] = hex2bin!("C7 45 CF 53"); // @0..4: GEOS 2.x magic ('G'|0x80, 'E', 'O'|0x80, 'S')
    const V1_TYPE_OFFSET: usize = 48; // @48..50: GeosFileType in v1 (1..=6)
    const V2_TYPE_OFFSET: usize = 4; // @4..6: GeosFileType in v2 (1..=6)

    require!(
        size >= MIN_GEOWORKS_GEOS_SIZE,
        prefix.len() >= 128,
        (starts(prefix, GEOS_V2_MAGIC) && (1..=16).contains(&le16(prefix, V2_TYPE_OFFSET)?))
            || (starts(prefix, GEOS_V1_MAGIC)
                && ((1..=16).contains(&le16(prefix, 40)?)
                    || (1..=16).contains(&le16(prefix, V1_TYPE_OFFSET)?)))
    );
    Some(Label::GeoworksGeos)
}

/// IBM z/OS Generalized Object File Format (GOFF).
pub(crate) fn verify_goff_object(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GOFF_OBJECT_SIZE,
        prefix.len() >= 84,
        u8_at(prefix, 0)? == 0x03,
        (u8_at(prefix, 1).unwrap_or(0) & 0xF0) == 0xF0 || u8_at(prefix, 1).unwrap_or(0) == 0x0F,
        u8_at(prefix, 2)? == 0x00,
        u8_at(prefix, 3)? <= 0x02
    );
    // Check either fixed-80 record at @80 (`u8_at(prefix, 80)? == 0x03`) or RDW-framed record
    let second_ok = u8_at(prefix, 80)? == 0x03
        && matches!(u8_at(prefix, 81)? & 0xF0, 0x00 | 0x10 | 0x20 | 0x30 | 0x40 | 0xF0);
    require!(second_ok);
    Some(Label::Goff)
}

/// React Native Hermes JS bytecode (.hbc).
pub(crate) fn verify_hermes_bytecode(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HERMES_BYTECODE_SIZE && prefix.len() >= 44 && starts(prefix, HERMES_MAGIC_8)
    );
    let version = le32(prefix, 8)?;
    let file_length = le32(prefix, 32)? as u64;
    let function_count = le32(prefix, 40)?;
    require!(
        (1..=200).contains(&version)
            && file_length >= MIN_HERMES_BYTECODE_SIZE
            && file_length <= size
            && function_count > 0
    );
    Some(Label::Hermes)
}

pub(crate) fn verify_hp2100_abs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_HP2100_ABS_SIZE);
    let mut p = 0usize;
    while p < prefix.len() && p < PAPER_TAPE_MAX_LEADER && u8_at(prefix, p)? == 0 {
        p += 1;
    }
    let mut recs = 0usize;
    let mut any_nonzero = false;
    while recs < 3 {
        let gap_start = p;
        while p < prefix.len() && p - gap_start < 16 && u8_at(prefix, p)? == 0 {
            p += 1;
        }
        require!(p + 8 <= prefix.len());
        let cnt = u8_at(prefix, p)? as usize;
        require!(cnt >= 1, u8_at(prefix, p + 1)? == 0);
        let rec_len = 6 + cnt * 2;
        require!(p + rec_len <= prefix.len());
        let mut sum = be16(prefix, p + 2)?;
        if sum != 0 {
            any_nonzero = true;
        }
        for i in 0..cnt {
            let w = be16(prefix, p + 4 + i * 2)?;
            if w != 0 {
                any_nonzero = true;
            }
            sum = sum.wrapping_add(w);
        }
        let csum = be16(prefix, p + 4 + cnt * 2)?;
        require!(sum == csum);
        recs += 1;
        p += rec_len;
    }
    require!(any_nonzero);
    Some(Label::Hp2100Abs)
}

pub(crate) fn verify_hp2100_rel(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_HP2100_REL_SIZE);
    let mut p = 0usize;
    while p < prefix.len() && p < PAPER_TAPE_MAX_LEADER && u8_at(prefix, p)? == 0 {
        p += 1;
    }
    let mut recs = 0usize;
    let mut any_nonzero = false;
    while recs < 3 {
        let gap_start = p;
        while p < prefix.len() && p - gap_start < 16 && u8_at(prefix, p)? == 0 {
            p += 1;
        }
        require!(p + 8 <= prefix.len());
        let cnt = u8_at(prefix, p)? as usize;
        require!(cnt >= 4, u8_at(prefix, p + 1)? == 0);
        let rec_len = cnt * 2;
        require!(p + rec_len <= prefix.len());
        let ident = be16(prefix, p + 2)?;
        let csum = be16(prefix, p + 4)?;
        let rtype = ident >> 13;
        if recs == 0 {
            require!((1..=6).contains(&rtype));
        }
        let mut sum = ident;
        for i in 0..(cnt - 3) {
            let w = be16(prefix, p + 6 + i * 2)?;
            if w != 0 {
                any_nonzero = true;
            }
            sum = sum.wrapping_add(w);
        }
        require!(sum == csum, csum != 0);
        recs += 1;
        p += rec_len;
    }
    require!(any_nonzero);
    Some(Label::Hp2100Rel)
}

/// HP 48 / 49 binary transfer object: `"HPHP48-"` or `"HPHP49-"` + ROM revision letter.
pub(crate) fn verify_hp48_binary(prefix: &[u8], size: u64) -> Option<Label> {
    const MODEL: usize = 5; // @5: '8' (HP 48) or '9' (HP 49)
    const ROM_REVISION: usize = 7; // @7: uppercase letter
    require!(
        size >= MIN_HP48_SIZE,
        prefix.len() >= 11,
        starts(prefix, b"HPHP4"),
        eq_at(prefix, 6, b"-")
    );
    require!(
        matches!(u8_at(prefix, MODEL)?, b'8' | b'9'),
        u8_at(prefix, ROM_REVISION)?.is_ascii_uppercase(),
        get_slice(prefix, 8, prefix.len().min(16))?
            .iter()
            .any(|&b| (b < 0x20 && !matches!(b, b'\t' | b'\n' | b'\r')) || b >= 0x7F)
    );
    Some(Label::Hp48Bin)
}

/// HP 95LX / 100LX / 200LX System Manager Executable (`.exm`, `"DL"` or `"HS\x00\x00"`).
pub(crate) fn verify_hp_palmtop_exm(prefix: &[u8], size: u64) -> Option<Label> {
    const EXM_MAGIC: &[u8; 4] = b"HS\0\0"; // @0..4: "HS" + 0x0000
    const PAGES_OFFSET: usize = 4; // @4..6: u16 LE page/paragraph count
    const HDR_PARA_OFFSET: usize = 8; // @8..10: u16 LE header size in paragraphs (>= 2)

    require!(size >= MIN_HP_PALMTOP_EXM_SIZE, prefix.len() >= 32);
    if starts(prefix, b"DL") {
        let rem = le16(prefix, 2)? as u64;
        let pages = le16(prefix, PAGES_OFFSET)? as u64;
        let relocs = le16(prefix, 6)?;
        let hdr_para = le16(prefix, HDR_PARA_OFFSET)?;
        let reloc_off = le16(prefix, 24)?;
        let calc_size = if rem == 0 { pages * 512 } else { pages.saturating_sub(1) * 512 + rem };
        require!(
            rem < 512,
            pages >= 1,
            (2..=256).contains(&hdr_para),
            relocs == 0
                || (reloc_off >= 0x1C
                    && (reloc_off as u32).saturating_add((relocs as u32) * 4)
                        <= (hdr_para as u32) * 16),
            calc_size == size
        );
        return Some(Label::HpPalmtopExm);
    }
    require!(
        starts(prefix, EXM_MAGIC),
        le16(prefix, PAGES_OFFSET)? >= 1,
        (2..=256).contains(&le16(prefix, HDR_PARA_OFFSET)?)
    );
    Some(Label::HpPalmtopExm)
}

/// HP-UX PA-RISC / M68k System Object Model (SOM) executable or object (1985) Hewlett-Packard.
///
/// Validates big-endian `system_id` (`0x020B` PA 1.0, `0x020C` M68k, `0x0210` PA 1.1, `0x0214` PA 2.0),
/// `a_magic` (`0x0106..=0x010E`), decimal `<filehdr.h>` `version_id` (`85082112` or `87102412`),
/// `som_length >= 128`, and the exact 32-bit header XOR checksum (`words[0..31] == words[31]`).
pub(crate) fn verify_hpux_som(prefix: &[u8], size: u64) -> Option<Label> {
    const SYS_ID_OFF: usize = 0; // @0..2: u16 BE system_id
    const MAGIC_OFF: usize = 2; // @2..4: u16 BE a_magic
    const VERSION_OFF: usize = 4; // @4..8: u32 BE version_id (85082112 or 87102412)
    const AUX_LOC_OFF: usize = 28; // @28..32: u32 BE aux_header_location
    const SOM_LEN_OFF: usize = 36; // @36..40: u32 BE som_length
    const HDR_WORDS: usize = 32; // @0..128: 32 × u32 BE header words (word 31 = XOR checksum)

    require!(size >= MIN_HPUX_SOM_SIZE, prefix.len() >= HDR_WORDS * 4);
    let sys_id = be16(prefix, SYS_ID_OFF)?;
    let a_magic = be16(prefix, MAGIC_OFF)?;
    let version = be32(prefix, VERSION_OFF)?;
    let aux_loc = be32(prefix, AUX_LOC_OFF)?;
    let som_len = be32(prefix, SOM_LEN_OFF)?;
    require!(
        matches!(sys_id, 0x020B | 0x020C | 0x0210 | 0x0214),
        matches!(a_magic, 0x0106 | 0x0107 | 0x0108 | 0x010B | 0x010D | 0x010E),
        version == HPUX_SOM_VERSION_OLD || version == HPUX_SOM_VERSION_NEW,
        aux_loc == 0 || aux_loc >= MIN_HPUX_SOM_SIZE as u32,
        som_len >= MIN_HPUX_SOM_SIZE as u32
    );

    let mut xor_acc: u32 = 0;
    for i in 0..(HDR_WORDS - 1) {
        xor_acc ^= be32(prefix, i * 4)?;
    }
    let stored_chk = be32(prefix, (HDR_WORDS - 1) * 4)?;
    require!(xor_acc == stored_chk);

    Some(Label::HpuxSom)
}

/// IBM PC Language Overlay (`.ovl`, `"Overlay for the IBM PC V"`).
pub(crate) fn verify_ibmpc_ovl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_IBMPC_OVL_SIZE, starts(prefix, b"Overlay for the IBM PC V"));
    Some(Label::IbmpcOvl)
}

/// LLVM GSYM Debug Symbol File.
pub(crate) fn verify_llvm_gsym(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_LLVM_GSYM_SIZE, prefix.len() >= 40);
    let is_le = starts(prefix, b"MYSG");
    let is_be = starts(prefix, b"GSYM");
    require!(is_le || is_be);
    let rd16 =
        |off: usize| -> Option<u16> { if is_le { le16(prefix, off) } else { be16(prefix, off) } };
    let rd32 =
        |off: usize| -> Option<u32> { if is_le { le32(prefix, off) } else { be32(prefix, off) } };
    let ver = rd16(4)?;
    let addr_off_size = u8_at(prefix, 6)?;
    let num_addrs = rd32(16)?;
    require!(matches!(addr_off_size, 1 | 2 | 4 | 8), num_addrs > 0);
    if ver == 1 {
        require!(size >= 48, prefix.len() >= 48);
        let uuid_size = u8_at(prefix, 7)? as usize;
        let strtab_off = rd32(20)? as u64;
        let strtab_size = rd32(24)? as u64;
        require!(
            uuid_size <= 20,
            get_slice(prefix, 28 + uuid_size, 48)?.iter().all(|&b| b == 0),
            strtab_off >= 48,
            strtab_size >= 1,
            strtab_off.saturating_add(strtab_size) <= size
        );
    } else if ver == 2 {
        // GSYM HeaderV2 (20B) + first 20B GlobalData entry at @20..40
        let strtab_enc = u8_at(prefix, 7)?;
        let gd_type = rd32(20)?;
        require!(strtab_enc <= 4, gd_type <= 16);
    } else {
        return None;
    }
    Some(Label::Gsym)
}

/// LLVM Indexed / Raw / Sample Coverage Profile Data (`.profdata` / `.profraw`).
pub(crate) fn verify_llvm_profile_data(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LLVM_PROFDATA_SIZE,
        starts(prefix, b"\xFFlprofi\x81")
            || starts(prefix, b"\x81iforpl\xFF")
            || starts(prefix, b"\xFFlprofr\x81")
            || starts(prefix, b"\xFFlprofR\x81")
            || starts(prefix, b"\x81rforpl\xFF")
            || starts(prefix, b"\x81Rforpl\xFF")
            || starts(prefix, b"\x80mforpl\xFF")
            || starts(prefix, hex2bin!("84 E4 D0 B1 F4 C9 94 A8 53 67"))
            || starts(prefix, hex2bin!("82 E4 D0 B1 F4 C9 94 A8 53 67"))
    );
    Some(Label::LlvmProfdata)
}

/// Micro Focus COBOL Generated Native Code (.gnt, Micro Focus).
pub(crate) fn verify_microfocus_gnt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MICROFOCUS_GNT_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\x39\x03\x03\0\0\0\0\0"),
        get_slice(prefix, 16, 26)? == b"\0\0\x04\x03.text\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::MicrofocusGnt)
}

/// Trustonic Kinibi / G&D MobiCore ARM TrustZone TEE Loadable Binary (`MCLF`).
///
/// Header layout (`mclfHeaderV2_t`):
/// - `@0..4`: `"MCLF"` magic
/// - `@4..6`: `u16 LE` `minor_version` (`0..=6`)
/// - `@6..8`: `u16 LE` `major_version` (`== 2`)
/// - `@8..12`: `u32 LE` `flags` (`<= 0xFFFF`)
/// - `@12..16`: `u32 LE` `mem_type` (`0..=2`: internal preferred / internal / external)
/// - `@16..20`: `u32 LE` `service_type` (`1..=5`: driver / SP trustlet / system trustlet / middleware / no service)
/// - `@20..24`: `u32 LE` `num_instances` (`1..=256` or `0xFFFF_FFFF` unlimited)
/// - `@44..48`: `u32 LE` `num_threads` (`1..=256`)
/// - `@52..56`: `u32 LE` `text_len` (`1..=64 MiB`)
/// - `@60..64`: `u32 LE` `data_len` (`<= 64 MiB`).
pub(crate) fn verify_mobicore_mclf(prefix: &[u8], size: u64) -> Option<Label> {
    const MINOR_VER_OFF: usize = 4; // @4..6: u16 LE minor version (0..=6)
    const MAJOR_VER_OFF: usize = 6; // @6..8: u16 LE major version (2)
    const FLAGS_OFF: usize = 8; // @8..12: u32 LE service header flags
    const MEM_TYPE_OFF: usize = 12; // @12..16: u32 LE memory type (0..=2)
    const SERVICE_TYPE_OFF: usize = 16; // @16..20: u32 LE service type (1..=5)
    const NUM_INSTANCES_OFF: usize = 20; // @20..24: u32 LE max concurrent instances
    const NUM_THREADS_OFF: usize = 44; // @44..48: u32 LE thread count (1..=256)
    const TEXT_LEN_OFF: usize = 52; // @52..56: u32 LE .text segment byte length
    const DATA_LEN_OFF: usize = 60; // @60..64: u32 LE .data segment byte length
    const MAX_SEGMENT_LEN: u32 = 64 * 1024 * 1024;
    const UNLIMITED_INSTANCES: u32 = u32::MAX;

    require!(
        size >= MIN_MOBICORE_MCLF_SIZE,
        prefix.len() >= 64,
        starts(prefix, MOBICORE_MCLF_MAGIC),
        le16(prefix, MINOR_VER_OFF)? <= 6,
        le16(prefix, MAJOR_VER_OFF)? == 2,
        le32(prefix, FLAGS_OFF)? <= 0xFFFF,
        le32(prefix, MEM_TYPE_OFF)? <= 2,
        (1..=5).contains(&le32(prefix, SERVICE_TYPE_OFF)?),
        (1..=256).contains(&le32(prefix, NUM_THREADS_OFF)?),
        (1..=MAX_SEGMENT_LEN).contains(&le32(prefix, TEXT_LEN_OFF)?),
        le32(prefix, DATA_LEN_OFF)? <= MAX_SEGMENT_LEN
    );
    let num_instances = le32(prefix, NUM_INSTANCES_OFF)?;
    require!((1..=256).contains(&num_instances) || num_instances == UNLIMITED_INSTANCES);

    Some(Label::MobicoreMclf)
}

/// mruby RiteVM Compiled Bytecode (`.mrb`, 2012) binary.
pub(crate) fn verify_mruby_rite(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC_OFFSET: usize = 0; // @0..4: "RITE"
    const VER_RANGE: core::ops::Range<usize> = 4..8; // @4..8: 4 ASCII digits ("0001".."0300")

    require!(
        size >= MIN_MRUBY_RITE_SIZE,
        prefix.len() >= 28,
        eq_at(prefix, MAGIC_OFFSET, b"RITE"),
        get_slice(prefix, VER_RANGE.start, VER_RANGE.end)?.iter().all(|b| b.is_ascii_digit())
    );

    // mruby 3.x ("0300"+) removed the 2-byte header CRC at @8..10, placing bin_size at @8..12,
    // compiler_name at @12..16, compiler_version at @16..20, and section ID at @20..24.
    // mruby 1.x/2.x ("0001".."0299") keeps 2-byte CRC at @8..10, bin_size at @10..14,
    // compiler_name at @14..18, compiler_version at @18..22, and section ID at @22..26.
    let is_v3 = u8_at(prefix, 4)? >= b'0' && u8_at(prefix, 5)? >= b'3';
    let (size_off, comp_off, cver_off, sec_off) = if is_v3 {
        (8usize, 12usize, 16usize, 20usize)
    } else {
        (10usize, 14usize, 18usize, 22usize)
    };

    let bin_size = be32(prefix, size_off)? as u64;
    require!(
        bin_size >= MIN_MRUBY_RITE_SIZE && bin_size <= size,
        get_slice(prefix, comp_off, comp_off + 4)?
            .iter()
            .all(|&b| b.is_ascii_graphic() || b == b' '),
        get_slice(prefix, cver_off, cver_off + 4)?.iter().all(|b| b.is_ascii_digit()),
        eq_at(prefix, sec_off, b"IREP")
            || eq_at(prefix, sec_off, b"LVAR")
            || eq_at(prefix, sec_off, b"DBG\0")
    );
    Some(Label::MrubyRiteMrb)
}

/// IBM System/370 & z/OS MVS Object Deck (ESD/TXT/RLD/END).
pub(crate) fn verify_mvs_object_deck(prefix: &[u8], size: u64) -> Option<Label> {
    const ESD: &[u8] = hex2bin!("C5 E2 C4"); // EBCDIC "ESD"
    const SYM: &[u8] = hex2bin!("E2 E8 D4"); // EBCDIC "SYM"
    const TXT: &[u8] = hex2bin!("E3 E7 E3"); // EBCDIC "TXT"
    const RLD: &[u8] = hex2bin!("D9 D3 C4"); // EBCDIC "RLD"
    const END: &[u8] = hex2bin!("C5 D5 C4"); // EBCDIC "END"
    require!(
        size >= MIN_MVS_OBJECT_DECK_SIZE,
        size.is_multiple_of(80),
        prefix.len() >= 160,
        u8_at(prefix, 0)? == 0x02,
        (get_slice(prefix, 1, 4)? == ESD || get_slice(prefix, 1, 4)? == SYM),
        get_slice(prefix, 4, 10)? == [0x40; 6],
        u8_at(prefix, 80)? == 0x02,
        (get_slice(prefix, 81, 84)? == ESD
            || get_slice(prefix, 81, 84)? == SYM
            || get_slice(prefix, 81, 84)? == TXT
            || get_slice(prefix, 81, 84)? == RLD
            || get_slice(prefix, 81, 84)? == END)
    );
    Some(Label::MvsObj)
}

/// NASM RDOFF Relocatable Dynamic Object (`"RDOFF2"` / `"RDOFF1"`).
pub(crate) fn verify_nasm_rdoff(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NASM_RDOFF_SIZE,
        prefix.len() >= 10,
        starts(prefix, b"RDOFF2") || starts(prefix, b"RDOFF1"),
        prefix.get(6..10)?.contains(&0)
    );
    let obj_len = le32(prefix, 6)? as u64;
    require!(obj_len > 0 && obj_len <= size);
    Some(Label::NasmRdoff)
}

/// Nokia BB5 PPM Language Pack Firmware image (`"\x04BB5\0"` + `"BB5 ALGORITHM\0"`).
pub(crate) fn verify_nokia_bb5_ppmg(prefix: &[u8], size: u64) -> Option<Label> {
    const BB5_TAG: &[u8; 5] = b"\x04BB5\0";
    const BB5_ALG_TAG: &[u8; 14] = b"BB5 ALGORITHM\0";

    require!(
        size >= MIN_NOKIA_BB5_PPMG_SIZE,
        prefix.len() >= 48,
        (0x20..=0x1000).contains(&le32(prefix, 0)?), // @0..4: outer block length
        (0x10..=0x1000).contains(&le32(prefix, 4)?)  // @4..8: inner header length
    );
    get_slice(prefix, 12, prefix.len().min(48))?
        .windows(BB5_TAG.len())
        .position(|w| w == BB5_TAG)?;
    get_slice(prefix, 16, prefix.len().min(64))?
        .windows(BB5_ALG_TAG.len())
        .position(|w| w == BB5_ALG_TAG)?;
    Some(Label::NokiaBb5Ppm)
}

/// Novell NetWare Remote Program Load (RPL) Boot ROM (`.rom` / `.rpl`, 1989) Novell.
pub(crate) fn verify_novell_netware_rpl_rom(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NOVELL_NETWARE_RPL_ROM_SIZE,
        prefix.len() >= 16,
        starts(prefix, &[0xFA, 0xE9]),
        eq_at(prefix, 4, b"Netw.RpL")
    );
    Some(Label::NovellNetwareRplRom)
}

/// OSASK / Haribote OS Compressed Application Binary (`.hrb`, Tek1/2/5 `"OSASKCMP"`).
pub(crate) fn verify_osask_hrb_exec(prefix: &[u8], size: u64) -> Option<Label> {
    const OSASKCMP_SIG: &[u8; 15] = hex2bin!("FF FF FF 01 00 00 00 4F 53 41 53 4B 43 4D 50"); // @1..16

    require!(
        size >= MIN_OSASK_HRB_EXEC_SIZE,
        prefix.len() >= 16,
        matches!(u8_at(prefix, 0)?, 0x83 | 0x85 | 0x89), // Tek1 (0x83), Tek2 (0x85), Tek5 (0x89)
        eq_at(prefix, 1, OSASKCMP_SIG)
    );
    Some(Label::OsaskHrbExec)
}

/// Commodore VIC-20 Cartridge ROM / PCVIC Snapshot (.prg / .crt / .20 / .a0).
pub(crate) fn verify_pcvic_snapshot(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PCVIC_SNAPSHOT_SIZE
            && prefix.len() >= 22
            && starts(prefix, b"PCVIC system snapshot\0")
    );
    Some(Label::Vic20Rom)
}

pub(crate) fn verify_pdp11_lda(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PDP11_LDA_SIZE);
    let mut p = 0usize;
    while p < prefix.len() && p < PAPER_TAPE_MAX_LEADER && u8_at(prefix, p)? == 0 {
        p += 1;
    }
    let mut blocks = 0usize;
    let mut has_data_block = false;
    while blocks < 2 {
        let gap_start = p;
        while p < prefix.len() && p - gap_start < 64 && u8_at(prefix, p)? == 0 {
            p += 1;
        }
        require!(le16(prefix, p) == Some(1));
        let bc = le16(prefix, p + 2)? as usize;
        require!(bc >= 6, p + bc < prefix.len());
        let sum = get_slice(prefix, p, p + bc + 1)?.iter().fold(0u8, |a, &b| a.wrapping_add(b));
        require!(sum == 0);
        if bc > 6 {
            has_data_block = true;
        }
        blocks += 1;
        p += bc + 1;
    }
    require!(has_data_block);
    Some(Label::Pdp11Lda)
}

/// DEC PDP-8 OS/8 Save Core Image executable (`.sv`, 1971) Digital Equipment.
///
/// Validates 512-byte block alignment, strictly 12-bit LE words (`< 0x1000`) throughout the
/// 512-byte Core Control Block (CCB), valid `CIF CDF` starting field instruction (`w[1]`),
/// segment count `nseg = (0x1000 - w[0]) & 0x0FFF` in `1..=62`, and exact segment page-to-block
/// count matching `size / 512`.
pub(crate) fn verify_pdp8_sv(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PDP8_SV_SIZE,
        size.is_multiple_of(OS8_BLOCK_SIZE as u64),
        prefix.len() >= OS8_BLOCK_SIZE,
        has_non_ascii_text_byte(prefix.get(..64)?)
    );

    let mut ccb = [0u16; 256];
    for (i, slot) in ccb.iter_mut().enumerate() {
        let w = le16(prefix, i * 2)?;
        if w >= 0x1000 {
            return None;
        }
        *slot = w;
    }

    let nseg = (0x1000u16.wrapping_sub(ccb[0]) & 0x0FFF) as usize;
    // w[1]: starting field CIF/CDF instruction (62n1, 62n2, or 62n3 octal)
    let start_field_op = ccb[1] & 0o7707;
    require!((1..=62).contains(&nseg), matches!(start_field_op, 0o6201..=0o6203));

    let mut total_blocks: u64 = 1; // Block 0 = CCB
    for seg in 0..nseg {
        let ctrl = ccb[5 + 2 * seg];
        require!((ctrl & 0o0007) == 0);
        let mut pages = ((ctrl >> 6) & 0x3F) as u64;
        if pages == 0 {
            pages = 32;
        }
        total_blocks += pages.div_ceil(2);
    }
    require!(total_blocks * (OS8_BLOCK_SIZE as u64) == size);

    Some(Label::Pdp8Sv)
}

/// IBM z/OS PDSE Program Object (IEWPLMH).
pub(crate) fn verify_pdse_program_object(prefix: &[u8], size: u64) -> Option<Label> {
    const IEWPLMH: &[u8] = hex2bin!("C9 C5 E6 D7 D3 D4 C8 40"); // EBCDIC "IEWPLMH "
    const IEWPMAR: &[u8] = hex2bin!("C9 C5 E6 D7 D4 C1 D9 40"); // EBCDIC "IEWPMAR "
    require!(size >= MIN_PDSE_PROGRAM_OBJECT_SIZE, prefix.len() >= 36);
    if starts(prefix, IEWPLMH) {
        let hdr_len = be32(prefix, 8)?;
        let pm_ver = u8_at(prefix, 12)?;
        let total_size = be32(prefix, 16)?;
        let entry_count = be32(prefix, 20)?;
        let entry0_type = be32(prefix, 24)?;
        let entry0_off = be32(prefix, 28)?;
        require!(
            (1..=8).contains(&pm_ver),
            get_slice(prefix, 13, 16)? == [0, 0, 0],
            (2..=64).contains(&entry_count),
            hdr_len == 24 + entry_count * 12,
            total_size >= hdr_len,
            entry0_type == 0x0001_0000,
            entry0_off == hdr_len
        );
        return Some(Label::PdseProg);
    }
    require!(starts(prefix, IEWPMAR));
    let hdr_len = be16(prefix, 8)?;
    let pm_ver = u8_at(prefix, 10)?;
    require!((28..=512).contains(&hdr_len), (1..=8).contains(&pm_ver));
    Some(Label::PdseProg)
}

/// Linux `perf` JITDump Trace file (`jit-*.dump`).
pub(crate) fn verify_perf_jitdump(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PERF_JITDUMP_SIZE,
        prefix.len() >= 24,
        starts(prefix, PERF_JITDUMP_LE_MAGIC),
        (1..=2).contains(&le32(prefix, PERF_JITDUMP_VER_OFFSET)?),
        le32(prefix, PERF_JITDUMP_HDR_SIZE_OFFSET) == Some(40),
        le32(prefix, PERF_JITDUMP_MACH_OFFSET)? > 0
    );
    Some(Label::PerfJitdump)
}

/// Three Rivers PERQ POS / CMU Accent Q-Code Segment (`.SEG`, 1980 Three Rivers Computer).
///
/// Header layout:
/// - `@0x00`: `u8 kind` (`0..=3` for POS v3; `0x16 | 0x17` for Accent v4)
/// - `@0x01`: `u8 version` (`3` for PERQ POS, `4` for CMU Accent)
/// - `@0x02..0x0A`: 8-byte ASCII segment name (alphanumeric/underscore/space, leading alphanumeric)
/// - `@0x0A`: `u8 path_len` (`1..=80`) Pascal length of source path string
/// - `@0x0B..0x0B+path_len`: printable ASCII source path containing a delimiter (`.`, `:`, `>`, or `/`).
pub(crate) fn verify_perq_pos_seg(prefix: &[u8], size: u64) -> Option<Label> {
    const NAME_OFF: usize = 0x02; // @0x02: 8-byte segment identifier
    const PATH_LEN_OFF: usize = 0x0A; // @0x0A: Pascal length of source path
    const PATH_OFF: usize = 0x0B; // @0x0B: source path characters

    require!(size >= MIN_PERQ_POS_SEG_SIZE, prefix.len() >= 24);
    let kind = u8_at(prefix, 0)?;
    let ver = u8_at(prefix, 1)?;
    require!((ver == 3 && kind <= 3) || (ver == 4 && matches!(kind, 0x16 | 0x17)));
    let name = get_slice(prefix, NAME_OFF, PATH_LEN_OFF)?;
    let plen = u8_at(prefix, PATH_LEN_OFF)? as usize;
    require!(
        name[0].is_ascii_alphanumeric(),
        name.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b' '),
        (1..=80).contains(&plen),
        PATH_OFF + plen <= prefix.len(),
        has_non_ascii_text_byte(prefix.get(..PATH_OFF)?)
    );
    let path = get_slice(prefix, PATH_OFF, PATH_OFF + plen)?;
    require!(
        path.iter().all(|&b| (0x20..=0x7E).contains(&b)),
        path.iter().any(|&b| matches!(b, b'.' | b':' | b'>' | b'/'))
    );
    Some(Label::PerqSeg)
}

/// Sony PlayStation Portable Executable Package (.pbp / EBOOT.PBP).
pub(crate) fn verify_playstation_psp_pbp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_PSP_PBP_SIZE && prefix.len() >= 40);
    require!(starts(prefix, b"\x00PBP"));

    // Version: typically 0x00010000 or 0x00010001 (1.0 or 1.1)
    let version = le32(prefix, 4)?;
    require!(version == 0x0001_0000 || version == 0x0001_0001);

    // 8 section offsets starting at offset 8 (PARAM.SFO, ICON0.PNG, ICON1.PMF, PIC0.PNG, PIC1.PNG, SND0.AT3, PSP.DATA, PSAR.DATA)
    let mut prev_offset = 40u32;
    for i in 0..8 {
        let ofs = 8 + i * 4;
        let sec_offset = le32(prefix, ofs)?;
        require!(sec_offset >= prev_offset);
        require!((sec_offset as u64) <= size);
        prev_offset = sec_offset;
    }

    // First section is PARAM.SFO at offset get_slice(prefix, 8, 12)?
    let sfo_offset = le32(prefix, 8)? as usize;
    if prefix.len() >= sfo_offset + 4 {
        require!(get_slice(prefix, sfo_offset, sfo_offset + 4)? == b"\x00PSF");
    }

    Some(Label::Pbp)
}

/// Psion Series 3 / 3a / 3c / Siena SIBO Executable (`"ImageFileType**\0"`).
pub(crate) fn verify_psion_sibo_app(prefix: &[u8], size: u64) -> Option<Label> {
    const SIBO_IMG_MAGIC: &[u8; 16] = b"ImageFileType**\0"; // @0..16: 16-byte SIBO image signature

    require!(
        size >= MIN_PSION_SIBO_APP_SIZE,
        prefix.len() >= 32,
        starts(prefix, SIBO_IMG_MAGIC),
        has_non_ascii_text_byte(prefix.get(16..32)?)
    );
    Some(Label::PsionSiboApp)
}

/// Sony PlayStation 1 PS-X Executable (`PS-X EXE` / `SCE EXE `).
pub(crate) fn verify_psx_exe(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PSX_EXE_SIZE,
        prefix.len() >= 32,
        starts(prefix, PSX_EXE_SIG) || starts(prefix, PSX_SCE_EXE_SIG),
        le32(prefix, 16)? >= 0x8000_0000,
    );
    Some(Label::PsxExe)
}

/// Qualcomm Bootloader Device Tree Table Image (QCDT).
pub(crate) fn verify_qualcomm_qcdt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_QUALCOMM_QCDT_SIZE, prefix.len() >= 32, starts(prefix, b"QCDT"));
    let ver = le32(prefix, 4)?;
    let num_dtbs = le32(prefix, 8)?;
    require!((1..=3).contains(&ver), (1..=4096).contains(&num_dtbs));
    let entry_size = match ver {
        1 => 20usize,
        2 => 24usize,
        _ => 40usize,
    };
    let dtb_off_pos = 12 + entry_size - 8;
    if dtb_off_pos + 8 <= prefix.len() {
        let dtb_off = le32(prefix, dtb_off_pos)? as u64;
        let dtb_sz = le32(prefix, dtb_off_pos + 4)? as u64;
        require!(
            dtb_off >= 32,
            dtb_sz >= 40,
            dtb_off & 0x1FF == 0,
            dtb_off.saturating_add(dtb_sz) <= size
        );
    }
    Some(Label::Qcdt)
}

/// RAD Debugger Debug Information (`.rdi`, 2024) Ryan Fleury / Epic Games & RAD Game Tools.
pub(crate) fn verify_raddbg_rdi(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_RADDBG_RDI_SIZE && prefix.len() >= 32 && starts(prefix, b"raddbg\0\0"));
    let encoding_version = le32(prefix, 8)?;
    require!((1..=1000).contains(&encoding_version));

    let word12 = le32(prefix, 12)?;
    let (data_section_off, data_section_count) = if word12 >= 20 {
        // v0.9.20+ 32-bit header layout: u32 encoding_version (@8), u32 data_section_off (@12), u32 data_section_count (@16)
        let sec_cnt = le32(prefix, 16)? as u64;
        (word12 as u64, sec_cnt)
    } else if word12 == 0 {
        // v0.9.0..v0.9.15 64-bit header layout: u64 encoding_version (@8), u64 data_section_off (@16), u64 data_section_count (@24)
        let sec_off = le64(prefix, 16)?;
        let sec_cnt = le64(prefix, 24)?;
        (sec_off, sec_cnt)
    } else {
        return None;
    };

    require!(
        (20..=4096).contains(&data_section_off)
            && (1..=65536).contains(&data_section_count)
            && data_section_off + data_section_count.saturating_mul(24) <= size
    );

    Some(Label::RaddbgRdi)
}

/// Squirrel VM Compiled Closure Bytecode (`.cnut` / `.nut`, 2003) stream.
///
/// Supports both Squirrel 2.x (`PART` at `@10..14` immediately after `sizeof(SQChar)`) and
/// Squirrel 3.x (`sizeof(SQInteger)` at `@10..14` + `sizeof(SQFloat)` at `@14..18` + `PART` at `@18..22`).
pub(crate) fn verify_squirrel_cnut(prefix: &[u8], size: u64) -> Option<Label> {
    const STREAM_TAG: &[u8; 2] = hex2bin!("FA FA"); // @0..2: SQ_BYTECODE_STREAM_TAG (0xFAFA)
    const HEAD_LE: &[u8; 4] = b"RIQS"; // @2..6: SQ_CLOSURESTREAM_HEAD little-endian
    const HEAD_BE: &[u8; 4] = b"SQIR"; // @2..6: SQ_CLOSURESTREAM_HEAD big-endian
    const PART_LE: &[u8; 4] = b"TRAP"; // SQ_CLOSURESTREAM_PART little-endian
    const PART_BE: &[u8; 4] = b"PART"; // SQ_CLOSURESTREAM_PART big-endian
    const HEAD_OFFSET: usize = 2; // @2..6: closure stream head tag
    const CHAR_SIZE_OFFSET: usize = 6; // @6..10: sizeof(SQChar) (1, 2, or 4)
    const V2_PART_OFFSET: usize = 10; // @10..14: Squirrel 2.x PART tag
    const INT_SIZE_OFFSET: usize = 10; // @10..14: Squirrel 3.x sizeof(SQInteger) (4 or 8)
    const FLOAT_SIZE_OFFSET: usize = 14; // @14..18: Squirrel 3.x sizeof(SQFloat) (4 or 8)
    const V3_PART_OFFSET: usize = 18; // @18..22: Squirrel 3.x PART tag

    require!(size >= MIN_SQUIRREL_CNUT_SIZE, prefix.len() >= 18, starts(prefix, STREAM_TAG));
    let is_le = eq_at(prefix, HEAD_OFFSET, HEAD_LE);
    let is_be = eq_at(prefix, HEAD_OFFSET, HEAD_BE);
    require!(is_le || is_be);
    let read_u32 =
        |off: usize| -> Option<u32> { if is_le { le32(prefix, off) } else { be32(prefix, off) } };
    let part_tag = if is_le { PART_LE } else { PART_BE };
    let char_sz = read_u32(CHAR_SIZE_OFFSET)?;
    require!(matches!(char_sz, 1 | 2 | 4));
    let is_v2 = eq_at(prefix, V2_PART_OFFSET, part_tag);
    let is_v3 = prefix.len() >= 22
        && eq_at(prefix, V3_PART_OFFSET, part_tag)
        && matches!(read_u32(INT_SIZE_OFFSET), Some(4 | 8))
        && matches!(read_u32(FLOAT_SIZE_OFFSET), Some(4 | 8));
    require!(is_v2 || is_v3);
    Some(Label::SquirrelCnut)
}

/// Borland Turbo Debugger / Delphi TD32 symbol container (`FB52` / `FB09` / `FB0A` / `FB0B`).
pub(crate) fn verify_tds(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BORLAND_TDS_SIZE, prefix.len() >= 16);
    if starts(prefix, hex2bin!("FB 52")) {
        return Some(Label::Tds);
    }
    if starts(prefix, b"FB09") || starts(prefix, b"FB0A") || starts(prefix, b"FB0B") {
        let lfo_dir = le32(prefix, 4)? as usize;
        require!(lfo_dir >= 8);
        let valid_at_8 = if lfo_dir == 8 {
            le16(prefix, 8)? == 16 && le16(prefix, 10)? == 12
        } else {
            le32(prefix, 8)? == 0
                && (1..=256).contains(&le16(prefix, 12)?)
                && (eq_at(prefix, 14, b"CV") || eq_at(prefix, 14, b"VC"))
        };
        require!(valid_at_8);
        return Some(Label::Tds);
    }
    None
}

/// Das U-Boot Legacy `uImage` Firmware/Kernel Image (2000) DENX (`0x27051956` BE).
///
/// Header layout (`struct legacy_img_hdr`):
/// - `@0..4`: `0x27051956` BE (`IH_MAGIC`)
/// - `@12..16`: `u32 BE` `ih_size` (`<= 1 GiB` sanity bound)
/// - `@28`: `u8` `ih_os` (`0..=32`)
/// - `@29`: `u8` `ih_arch` (`0..=32`)
/// - `@30`: `u8` `ih_type` (`1..=45`; `0` is `IH_TYPE_INVALID`)
/// - `@31`: `u8` `ih_comp` (`0..=6`: none/gzip/bzip2/lzma/lzo/lz4/zstd)
/// - `@32..64`: `ih_name[32]` (image name)
pub(crate) fn verify_uboot_uimage(prefix: &[u8], size: u64) -> Option<Label> {
    const IH_SIZE_OFFSET: usize = 12; // @12..16: u32 BE payload data size
    const IH_OS_OFFSET: usize = 28; // @28: u8 operating system code (0..=32)
    const IH_ARCH_OFFSET: usize = 29; // @29: u8 CPU architecture code (0..=32)
    const IH_TYPE_OFFSET: usize = 30; // @30: u8 image type code (1..=45)
    const IH_COMP_OFFSET: usize = 31; // @31: u8 compression algorithm code (0..=6)
    const MAX_UIMAGE_DATA_SIZE: u32 = 0x4000_0000; // 1 GiB sanity ceiling
    const MAX_IH_OS: u8 = 32;
    const MAX_IH_ARCH: u8 = 32;
    const MAX_IH_TYPE: u8 = 45;
    const MAX_IH_COMP: u8 = 6;

    require!(
        size >= MIN_UBOOT_UIMAGE_SIZE,
        prefix.len() > IH_COMP_OFFSET,
        starts(prefix, UBOOT_UIMAGE_MAGIC),
        be32(prefix, IH_SIZE_OFFSET)? <= MAX_UIMAGE_DATA_SIZE,
        u8_at(prefix, IH_OS_OFFSET)? <= MAX_IH_OS,
        u8_at(prefix, IH_ARCH_OFFSET)? <= MAX_IH_ARCH,
        (1..=MAX_IH_TYPE).contains(&u8_at(prefix, IH_TYPE_OFFSET)?),
        u8_at(prefix, IH_COMP_OFFSET)? <= MAX_IH_COMP
    );
    Some(Label::Uboot)
}

/// uClinux Binary Flat Format Executable (bFLT).
pub(crate) fn verify_uclinux_bflt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_UCLINUX_BFLT_SIZE && prefix.len() >= 64 && starts(prefix, b"bFLT"));
    let rev = be32(prefix, 4).unwrap_or(0);
    let entry = be32(prefix, 8).unwrap_or(0);
    let data_start = be32(prefix, 12).unwrap_or(0);
    let data_end = be32(prefix, 16).unwrap_or(0);
    require!(matches!(rev, 2 | 4) && entry >= 64 && data_start >= entry && data_end >= data_start);
    Some(Label::UclinuxBflt)
}

/// UEFI Terse Executable (`"VZ"`).
pub(crate) fn verify_uefi_te(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 40, prefix.len() >= 40, starts(prefix, b"VZ"));
    let machine = le16(prefix, 2)?;
    let num_sections = u8_at(prefix, 4)?;
    let subsystem = u8_at(prefix, 5)?;
    let stripped_size = le16(prefix, 6)?;
    require!(
        is_known_pe_machine(machine),
        (1..=96).contains(&num_sections),
        subsystem <= 16,
        stripped_size >= 0x20,
    );
    Some(Label::UefiTe)
}

/// DEC VAX/VMS Executable / Shareable Image (`.EXE`, 1977 Digital Equipment).
///
/// Image Header (`IHD`) layout at block 0 (`512`-byte aligned file):
/// - `@0x00`: `u16 LE ihd$w_size` (`0x60..=512`, even, total fixed + sub-descriptor header bytes)
/// - `@0x02`: `u16 LE ihd$w_activoff` (`0x28..ihd_size`, activation descriptor offset)
/// - `@0x04`: `u16 LE ihd$w_symdbgoff` (`activoff..ihd_size`, debugger symbol descriptor offset)
/// - `@0x06`: `u16 LE ihd$w_imgidoff` (`symdbgoff..ihd_size`, image ID block offset)
/// - `@0x0C..0x10`: 4-byte ASCII `ihd$w_majorid` + `ihd$w_minorid` (`"02"` + 2 digits, e.g. `"0205"`)
/// - `@0x10`: `u8 ihd$b_hdrblkcnt` (`1..=32`, header block count; `hdrblkcnt * 512 <= size`)
/// - `@0x11`: `u8 ihd$b_imgtype` (`1` = executable, `2` = shareable, `3` = system/UWSS).
pub(crate) fn verify_vms_exe(prefix: &[u8], size: u64) -> Option<Label> {
    const IHD_SIZE_OFF: usize = 0x00; // @0x00: u16 LE header size
    const ACTIV_OFF: usize = 0x02; // @0x02: u16 LE activation offset
    const SYMDBG_OFF: usize = 0x04; // @0x04: u16 LE debug/symbol offset
    const IMGID_OFF: usize = 0x06; // @0x06: u16 LE image ID offset
    const MAJOR_ID_OFF: usize = 0x0C; // @0x0C: 2-byte ASCII major ID ("02")
    const MINOR_ID_OFF: usize = 0x0E; // @0x0E: 2-byte ASCII minor ID ("05")
    const HDRBLKCNT_OFF: usize = 0x10; // @0x10: u8 header block count
    const IMGTYPE_OFF: usize = 0x11; // @0x11: u8 image type (1..=3)

    require!(
        size >= MIN_VMS_EXE_SIZE,
        size.is_multiple_of(512),
        prefix.len() >= 32,
        eq_at(prefix, MAJOR_ID_OFF, VMS_EXE_MAJOR_ID),
        u8_at(prefix, MINOR_ID_OFF)?.is_ascii_digit(),
        u8_at(prefix, MINOR_ID_OFF + 1)?.is_ascii_digit()
    );
    let ihd_size = le16(prefix, IHD_SIZE_OFF)? as usize;
    let activoff = le16(prefix, ACTIV_OFF)? as usize;
    let symdbgoff = le16(prefix, SYMDBG_OFF)? as usize;
    let imgidoff = le16(prefix, IMGID_OFF)? as usize;
    let hdrblkcnt = u8_at(prefix, HDRBLKCNT_OFF)? as u64;
    let imgtype = u8_at(prefix, IMGTYPE_OFF)?;
    require!(
        (0x60..=512).contains(&ihd_size),
        ihd_size.is_multiple_of(2),
        (0x28..ihd_size).contains(&activoff),
        (activoff..ihd_size).contains(&symdbgoff),
        (symdbgoff..ihd_size).contains(&imgidoff),
        (1..=32).contains(&hdrblkcnt),
        hdrblkcnt * 512 <= size,
        (1..=3).contains(&imgtype),
        has_non_ascii_text_byte(prefix.get(..32)?)
    );
    Some(Label::VmsExe)
}

/// Bytecode Alliance WebAssembly Micro Runtime AOT Module (.aot).
pub(crate) fn verify_wamr_aot(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_WAMR_AOT_SIZE, prefix.len() >= 28, starts(prefix, b"\0aot"));
    let ver = le32(prefix, 4)?;
    let sec_type = le32(prefix, 8)?;
    let sec_size = le32(prefix, 12)?;
    let bin_type = le16(prefix, 16)?;
    let abi_type = le16(prefix, 18)?;
    let e_type = le16(prefix, 20)?;
    let e_version = le32(prefix, 24)?;
    require!(
        (1..=10).contains(&ver),
        matches!(sec_type, 0 | 1 | 100),
        (12..=512).contains(&sec_size),
        bin_type <= 7,
        abi_type <= 7,
        (1..=4).contains(&e_type),
        e_version == 1
    );
    Some(Label::WamrAot)
}

/// Xamarin.Android LZ4-Compressed Managed Assembly (`XALZ`, 2020 Microsoft / Xamarin).
pub(crate) fn verify_xamarin_xalz(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_XAMARIN_XALZ_SIZE && prefix.len() >= 16 && starts(prefix, XAMARIN_XALZ_MAGIC)
    );
    let uncompressed_len = le32(prefix, 8)? as u64;
    // Uncompressed payload is a .NET PE assembly (>= 512 bytes, and LZ4 literal run starts with `MZ`)
    require!(
        (512..=512 * 1024 * 1024).contains(&uncompressed_len)
            && get_slice(prefix, 12, 16)?.windows(2).any(|w| w == b"MZ")
    );
    Some(Label::Xalz)
}

/// Microsoft Original Xbox Executable (.xbe).
pub(crate) fn verify_xbox_xbe(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_XBOX_XBE_SIZE && prefix.len() >= 276 && starts(prefix, b"XBEH"));
    let base_addr = le32(prefix, 260).unwrap_or(0);
    let size_headers = le32(prefix, 264).unwrap_or(0);
    let size_image = le32(prefix, 268).unwrap_or(0);
    let size_img_hdr = le32(prefix, 272).unwrap_or(0);
    require!(
        base_addr > 0
            && (base_addr & 0xFFF) == 0
            && size_headers >= 376
            && size_image >= size_headers
            && size_img_hdr >= 376
    );
    Some(Label::XboxXbe)
}
