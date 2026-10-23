// Copyright 2026 Google LLC
// SPDX-License-Identifier: Apache-2.0

//! Reference-only verifiers for the 52 `not-working` rules in `rulesets/notworking/formats.yar`.
//!
//! All 52 rules carry `enforced = false` and are **never** consulted by
//! [`super::identify`]. They are preserved here as executable documentation of why they remain
//! disabled:
//!
//! - **OLE2 compound files (`D0CF11E0A1B11AE1`)** → `doc`, `ppt`, `outlook`, `msi` (no enforced
//!   rule claims bare OLE2, so enabling any one mislabels every legacy Office document).
//! - **`ftyp` ISO-BMFF boxes** → `mp4`, `qt` overlap the enforced `3gp`, `avif`, and `heif`
//!   brands (and bare `"ftyp"` at `@4` matches every ISO-BMFF family).
//! - **EBML (`1A 45 DF A3`)** → `webm` checks only the 4-byte EBML opener without a `DocType`
//!   element, overlapping the enforced `mkv` verifier.
//! - **RIFF (`"RIFF"`)** → `webp` matches bare `"RIFF"` at `@0`, which would steal `avi`, `wav`,
//!   and `ani`.
//! - **`"MM"` (`4D 4D`)** → `tiff` overlaps Autodesk 3ds Max (`3dsm`).
//! - **ASF header GUID (`30 26 B2 75 ...`)** → `asf`, `wma`, and `wmv` share the same container
//!   GUID and are split by codec in Magika's taxonomy.
//! - **SQLite 3 header** → `sqlite` (`sqlite_v1`) matches generic SQLite databases before
//!   application subtypes (`geopackage`, etc.) can be separated.
//! - **Signature-less placeholders (`false`)** → `bzip3`, `coff`, `crt`, `iso`, `msi`, `otf`.
//! - **Short or unvalidated magics** → 2-byte `zlibstream` (`78 9C` / `78 01` / `78 DA` / `78 5E`),
//!   2-byte `wmf` (`01 00`), 2-byte `arj` (`60 EA`), 3-byte `lzx` (`"LZX"`), 3-byte `orc`
//!   (`"ORC"`), 2-byte `mp3` frame syncs, bare `"!<arch>"` in `deb`, etc.

use super::{be16, be32, eq_at, le32, scan_range, starts, u8_at};

/// A disabled reference rule: `(rule_id, label, verifier(prefix, size))`.
pub(crate) const DISABLED: &[(&str, &str, fn(&[u8], u64) -> bool)] = &[
    ("taxonomy_access", "access", access),
    ("taxonomy_aout", "aout", aout),
    ("taxonomy_arj", "arj", arj),
    ("taxonomy_berkeleydb", "berkeleydb", berkeleydb),
    ("taxonomy_bzip3", "bzip3", bzip3),
    ("taxonomy_cinema4d", "cinema4d", cinema4d),
    ("taxonomy_coff", "coff", coff),
    ("taxonomy_crt", "crt", crt),
    ("taxonomy_deb", "deb", deb),
    ("taxonomy_dmg", "dmg", dmg),
    ("taxonomy_doc", "doc", doc),
    ("taxonomy_elf", "elf", elf),
    ("taxonomy_filemaker", "filemaker", filemaker),
    ("taxonomy_flatgeobuf", "flatgeobuf", flatgeobuf),
    ("taxonomy_h5", "h5", h5),
    ("taxonomy_hwp", "hwp", hwp),
    ("taxonomy_iso", "iso", iso),
    ("taxonomy_javabytecode", "javabytecode", javabytecode),
    ("taxonomy_jpeg", "jpeg", jpeg),
    ("taxonomy_lha", "lha", lha),
    ("taxonomy_lightwave", "lightwave", lightwave),
    ("taxonomy_lmdb", "lmdb", lmdb),
    ("taxonomy_lzx", "lzx", lzx),
    ("taxonomy_mp3", "mp3", mp3),
    ("taxonomy_mp4", "mp4", mp4),
    ("taxonomy_msi", "msi", msi),
    ("taxonomy_otf", "otf", otf),
    ("taxonomy_outlook", "outlook", outlook),
    ("taxonomy_paradox", "paradox", paradox),
    ("taxonomy_png", "png", png),
    ("taxonomy_ppt", "ppt", ppt),
    ("taxonomy_pub", "pub", ms_pub),
    ("taxonomy_qt", "qt", qt),
    ("taxonomy_rdata", "rdata", rdata),
    ("taxonomy_rhinoceros", "rhinoceros", rhinoceros),
    ("taxonomy_squashfs", "squashfs", squashfs),
    ("taxonomy_tar", "tar", tar),
    ("taxonomy_tga", "tga", tga),
    ("taxonomy_tiff", "tiff", tiff),
    ("taxonomy_webm", "webm", webm),
    ("taxonomy_webp", "webp", webp),
    ("taxonomy_wim", "wim", wim),
    ("taxonomy_wma", "wma", wma),
    ("taxonomy_wmf", "wmf", wmf),
    ("taxonomy_wmv", "wmv", wmv),
    ("taxonomy_xls", "xls", xls),
    ("xz_v1", "xz", xz_v1),
    ("taxonomy_zlibstream", "zlibstream", zlibstream),
    ("sqlite_v1", "sqlite", sqlite_v1),
    ("taxonomy_arc", "arc", arc),
    ("taxonomy_orc", "orc", orc),
    ("taxonomy_asf", "asf", asf),
];

/// OLE2 compound document header (`D0 CF 11 E0 A1 B1 1A E1`), shared by legacy Office formats.
const OLE2_MAGIC: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// Microsoft ASF Header Object GUID (`75B22630-668E-11CF-A6D9-00AA0062CE6C`), shared by `asf`,
/// `wma`, and `wmv`.
const ASF_HEADER_GUID: &[u8] = &[
    0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE, 0x6C,
];

/// Microsoft Access (`.mdb` / `.accdb` PRONOM signatures).
fn access(prefix: &[u8], _size: u64) -> bool {
    (prefix.len() >= 1035
        && starts(prefix, &[0x77, 0x2C, 0x53, 0x20])
        && matches!(prefix[1034], 0x66 | 0x68 | 0x69))
        || (prefix.len() >= 1035
            && starts(prefix, &[0x01, 0x00, 0x00, 0x00])
            && eq_at(prefix, 1030, b"Rich")
            && matches!(prefix[1034], 0x07..=0x09))
}

/// Unix `a.out` object/executable magic numbers (`OMAGIC` `0x0107`, `NMAGIC` `0x0108`, `ZMAGIC`
/// `0x010B` in either byte order) — 4-byte magic with 2 zero bytes is too broad without header
/// validation.
fn aout(prefix: &[u8], _size: u64) -> bool {
    prefix.len() >= 4
        && (matches!(le32(prefix, 0), Some(0x0107 | 0x0108 | 0x010B))
            || matches!(be32(prefix, 0), Some(0x0107 | 0x0108 | 0x010B)))
}

/// ARJ archive: bare 2-byte header ID `60 EA` at `@0`.
fn arj(prefix: &[u8], size: u64) -> bool {
    prefix.len() >= 2 && size >= 2 && starts(prefix, &[0x60, 0xEA])
}

/// Berkeley DB (btree `0x00053162`, hash `0x00061561`, queue `0x00042253`, log `0x00040988` at
/// `@0` or `@12` in either byte order).
fn berkeleydb(prefix: &[u8], _size: u64) -> bool {
    const MAGICS: &[&[u8; 4]] = &[
        &[0x61, 0x15, 0x06, 0x00],
        &[0x00, 0x06, 0x15, 0x61],
        &[0x53, 0x22, 0x04, 0x00],
        &[0x00, 0x04, 0x22, 0x53],
        &[0x00, 0x04, 0x09, 0x88],
        &[0x88, 0x09, 0x04, 0x00],
        &[0x62, 0x31, 0x05, 0x00],
        &[0x00, 0x05, 0x31, 0x62],
    ];
    starts(prefix, &[0x00, 0x06, 0x15, 0x61])
        || starts(prefix, &[0x62, 0x31, 0x05, 0x00])
        || starts(prefix, &[0x00, 0x05, 0x31, 0x62])
        || (prefix.len() >= 16 && MAGICS.iter().any(|m| eq_at(prefix, 12, *m)))
}

/// Placeholder (`condition: false`).
fn bzip3(_prefix: &[u8], _size: u64) -> bool {
    false
}

/// Maxon Cinema 4D (`FORM....MC4D`, `C4DC4D6` at `@1`, or `MC50` with `CAT5`/`DOK5`/`FCV5`/`PRF5`).
fn cinema4d(prefix: &[u8], _size: u64) -> bool {
    (prefix.len() >= 12 && starts(prefix, b"FORM") && eq_at(prefix, 8, b"MC4D"))
        || eq_at(prefix, 1, b"C4DC4D6")
        || (prefix.len() >= 12
            && starts(prefix, b"MC50")
            && (eq_at(prefix, 8, b"CAT5")
                || eq_at(prefix, 8, b"DOK5")
                || eq_at(prefix, 8, b"FCV5")
                || eq_at(prefix, 8, b"PRF5")))
}

/// Placeholder (`condition: false`).
fn coff(_prefix: &[u8], _size: u64) -> bool {
    false
}

/// Placeholder (`condition: false`).
fn crt(_prefix: &[u8], _size: u64) -> bool {
    false
}

/// Debian binary package (`!<arch>\ndebian-binary` / `debian-split`, or bare `"!<arch>"` which
/// matches every Unix `ar` archive).
fn deb(prefix: &[u8], size: u64) -> bool {
    (starts(prefix, b"!<arch>\ndebian")
        && (eq_at(prefix, 14, b"-binary") || eq_at(prefix, 14, b"-split")))
        || (prefix.len() >= 7 && size >= 7 && starts(prefix, b"!<arch>"))
}

/// Apple Disk Image (`"ER"` driver descriptor map at `@0` — 4-byte signature is too weak).
fn dmg(prefix: &[u8], _size: u64) -> bool {
    prefix.len() >= 4
        && starts(prefix, b"ER")
        && matches!(prefix[2], 0x00 | 0x02 | 0x04 | 0x06 | 0x08 | 0x0A | 0x0C | 0x0E)
        && prefix[3] == 0x00
}

/// Microsoft Word legacy `.doc`: includes bare OLE2 (`D0 CF 11 E0 A1 B1 1A E1`), which matches all
/// OLE2 compound documents (`xls`, `ppt`, `msi`, `outlook`, etc.).
fn doc(prefix: &[u8], size: u64) -> bool {
    let min = |n: usize| prefix.len() >= n && size >= n as u64;
    (prefix.len() >= 8
        && eq_at(prefix, 4, &[0, 0, 0, 0])
        && (starts(prefix, &[0xFE, 0x37, 0x00, 0x23])
            || starts(prefix, &[0xFE, 0x37, 0x00, 0x1C])
            || starts(prefix, &[0xFE, 0x34, 0x00, 0x00])
            || starts(prefix, &[0xFE, 0x32, 0x00, 0x00])))
        || (min(516) && eq_at(prefix, 512, &[0xEC, 0xA5, 0xC1, 0x00]))
        || (min(4) && starts(prefix, &[0xDB, 0xA5, 0x2D, 0x00]))
        || (min(8) && starts(prefix, &[0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0x00]))
        || (min(550) && (eq_at(prefix, 546, b"jbjb") || eq_at(prefix, 546, b"bjbj")))
        || (min(14) && starts(prefix, b"\\376\\067\\0\\043"))
        || (min(8) && starts(prefix, OLE2_MAGIC))
        || (min(4) && starts(prefix, b"\x0dDOC"))
        || (min(2140) && eq_at(prefix, 2112, b"Microsoft Word document data"))
        || (min(15) && starts(prefix, b"\\333\\245-\\0\\0\\0"))
        || (min(16) && starts(prefix, b"\\x31\\xbe\\x00\\x00"))
        || (min(5) && starts(prefix, b"PO^Q`"))
}

/// Bare ELF magic `"\x7fELF"` (would override `cubin` or mislabel corrupted ELF fragments).
fn elf(prefix: &[u8], size: u64) -> bool {
    prefix.len() >= 4 && size >= 4 && starts(prefix, b"\x7fELF")
}

/// Claris / FileMaker Pro database signatures.
fn filemaker(prefix: &[u8], _size: u64) -> bool {
    const PRO35_HEAD: &[u8] =
        &[0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x05, 0x00, 0x02, 0x00, 0x02, 0xC0];
    const PRO7_TAIL: &[u8] = b"HBAM2101OCT99\xC1\x02H\x07Pro 7.0\xC0\xC0";
    const PRO12_TAIL: &[u8] = b"HBAM2125JAN11\xC1\x02H\x08Pro 12.0\xC0\xC0";

    (eq_at(prefix, 14, b"\xC0HBAM7") && eq_at(prefix, 14 + 6 + 505, PRO7_TAIL))
        || (starts(prefix, PRO35_HEAD)
            && (eq_at(prefix, 15 + 527, b"Pro 3") || eq_at(prefix, 15 + 527, b"Pro 5")))
        || (0..=14).any(|pos| {
            eq_at(prefix, pos, b"\xC0HBAM7")
                && scan_range(prefix, pos + 6, pos + 6 + 512, PRO12_TAIL)
        })
}

/// FlatGeobuf (`"fgb\x03fgb\x01"`).
fn flatgeobuf(prefix: &[u8], _size: u64) -> bool {
    starts(prefix, b"fgb\x03fgb\x01")
}

/// HDF5 superblock (`\x89HDF\r\n\x1a\n` at `@0`, `@1024`, or `@2048` — overlaps HDF5-based formats
/// such as NetCDF-4, `.keras`, and MATLAB v7.3).
fn h5(prefix: &[u8], size: u64) -> bool {
    const MAGIC: &[u8] = b"\x89HDF\r\n\x1a\n";
    (starts(prefix, MAGIC)
        && ((prefix.len() >= 8 && size >= 8) || matches!(u8_at(prefix, 8), Some(0..=2))))
        || (prefix.len() >= 1032 && eq_at(prefix, 1024, MAGIC))
        || (prefix.len() >= 2056 && eq_at(prefix, 2048, MAGIC))
}

/// Hancom Hangul Word Processor v3 (`"HWP Document File"`).
fn hwp(prefix: &[u8], size: u64) -> bool {
    (prefix.len() >= 17 && size >= 17 && starts(prefix, b"HWP Document File"))
        || starts(prefix, b"HWP Document File V")
}

/// Placeholder (`condition: false`).
fn iso(_prefix: &[u8], _size: u64) -> bool {
    false
}

/// Java class file (`CA FE BA BE` at `@0` — overlaps Mach-O universal fat binaries).
fn javabytecode(prefix: &[u8], size: u64) -> bool {
    prefix.len() >= 4 && size >= 4 && starts(prefix, &[0xCA, 0xFE, 0xBA, 0xBE])
}

/// JPEG image (`FF D8` at `@0`, or ISO-BMFF JP2 box `00 00 00 0C "jP  "` which overlaps `jp2`).
fn jpeg(prefix: &[u8], size: u64) -> bool {
    (prefix.len() >= 2 && size >= 2 && starts(prefix, &[0xFF, 0xD8]))
        || (prefix.len() >= 4 && starts(prefix, &[0xFF, 0xD8, 0xFF]))
        || (prefix.len() >= 8
            && size >= 8
            && starts(prefix, &[0x00, 0x00, 0x00, 0x0C, b'j', b'P', b' ', b' ']))
        || (prefix.len() >= 12 && size >= 12 && starts(prefix, b"\\377\\330\\377"))
}

/// LHA / LZH archive (`"-lh"` or `"-lz"` method tag at `@2`).
fn lha(prefix: &[u8], size: u64) -> bool {
    (prefix.len() >= 5 && size >= 5 && eq_at(prefix, 2, b"-lh"))
        || (prefix.len() >= 7
            && size >= 7
            && (eq_at(prefix, 2, b"-lzs-")
                || eq_at(prefix, 2, b"-lz5-")
                || eq_at(prefix, 2, b"-lz4-")))
        || (prefix.len() >= 7 && (eq_at(prefix, 2, b"-lh6-") || eq_at(prefix, 2, b"-lh7-")))
}

/// NewTek LightWave 3D object (`"FORM....LWOB"`).
fn lightwave(prefix: &[u8], _size: u64) -> bool {
    prefix.len() >= 12 && starts(prefix, b"FORM") && eq_at(prefix, 8, b"LWOB")
}

/// Symas LMDB (`0xBEEFC0DE` LE at `@16`).
fn lmdb(prefix: &[u8], _size: u64) -> bool {
    prefix.len() >= 20 && eq_at(prefix, 16, &[0xDE, 0xC0, 0xEF, 0xBE])
}

/// Amiga LZX archive (`"LZX"` at `@0` — 3 ASCII bytes are too short).
fn lzx(prefix: &[u8], _size: u64) -> bool {
    starts(prefix, b"LZX")
}

/// MP3 audio (`"ID3"` tag or 2-byte MPEG audio frame sync `FF xx` at `@0`).
fn mp3(prefix: &[u8], size: u64) -> bool {
    if prefix.len() >= 3 && size >= 3 && starts(prefix, b"ID3") {
        return true;
    }
    if prefix.len() >= 2 && prefix[0] == 0xFF {
        let b1 = prefix[1];
        if matches!(b1, 0xE2 | 0xE3 | 0xF2..=0xF7 | 0xFC | 0xFD) {
            return true;
        }
        if size >= 2
            && matches!(b1, 0xD0 | 0xD1 | 0xD6 | 0xD7 | 0xDA | 0xDB | 0xDE | 0xDF | 0xE0..=0xFF)
        {
            return true;
        }
    }
    false
}

/// MPEG-4 container (`ftyp` brands at `@4`/`@8` — overlaps `3gp`, `m4a`, etc.).
fn mp4(prefix: &[u8], size: u64) -> bool {
    const BRANDS_4: &[&[u8; 4]] = &[
        b"mp41", b"mmp4", b"NDXP", b"isml", b"XAVC", b"ARRI", b"NDXC", b"mp42", b"bbxm", b"NDSH",
        b"mobi", b"NDSP", b"niko", b"NDXH", b"NDXM", b"NDSS", b"dash", b"NDSC", b"avc1", b"NDSM",
    ];
    const BRANDS_3: &[&[u8; 3]] = &[b"iso", b"F4P", b"M4P", b"F4V"];

    if eq_at(prefix, 4, b"ftyp") {
        if BRANDS_4.iter().any(|b| eq_at(prefix, 8, *b))
            || BRANDS_3.iter().any(|b| eq_at(prefix, 8, *b))
            || eq_at(prefix, 8, b"mp41")
        {
            return true;
        }
        if size >= 12
            && (eq_at(prefix, 8, b"isom")
                || eq_at(prefix, 8, b"f4v ")
                || eq_at(prefix, 8, b"MSNV")
                || eq_at(prefix, 8, b"M4V ")
                || eq_at(prefix, 8, b"mp42")
                || eq_at(prefix, 8, b"3gp5"))
        {
            return true;
        }
    }
    (prefix.len() >= 8
        && size >= 8
        && starts(prefix, &[0x00, 0x00, 0x00, 0x1C, b'f', b't', b'y', b'p']))
        || (prefix.len() >= 31 && size >= 31 && eq_at(prefix, 20, b"iso2avc1mp4"))
}

/// Placeholder (`condition: false`).
fn msi(_prefix: &[u8], _size: u64) -> bool {
    false
}

/// Placeholder (`condition: false`).
fn otf(_prefix: &[u8], _size: u64) -> bool {
    false
}

/// Microsoft Outlook `.msg`: bare OLE2 magic `D0 CF 11 E0 A1 B1 1A E1` at `@0`.
fn outlook(prefix: &[u8], size: u64) -> bool {
    prefix.len() >= 8 && size >= 8 && starts(prefix, OLE2_MAGIC)
}

/// Borland Paradox database header.
fn paradox(prefix: &[u8], _size: u64) -> bool {
    if prefix.len() < 95 || !eq_at(prefix, 2, &[0x00, 0x08]) || !matches!(prefix[4], 0x00 | 0x02) {
        return false;
    }
    let b5 = prefix[5];
    let z88 = eq_at(prefix, 91, &[0, 0, 0, 0]);
    ((1..=4).contains(&b5)
        && ((matches!(prefix[57], 0x05..=0x09) && z88)
            || (eq_at(prefix, 37, &[0, 0, 0, 0]) && matches!(prefix[57], 0x03..=0x04))))
        || ((1..=0x20).contains(&b5) && matches!(prefix[57], 0x0A..=0x0C) && z88)
}

/// PNG image (`\x89PNG\r\n\x1a\n` or literal `"\\x89PNG"`).
fn png(prefix: &[u8], size: u64) -> bool {
    (prefix.len() >= 8 && size >= 8 && starts(prefix, b"\x89PNG\r\n\x1a\n"))
        || (prefix.len() >= 7 && size >= 7 && starts(prefix, b"\\x89PNG"))
}

/// Microsoft PowerPoint legacy `.ppt` subheader at `@512` (overlaps OLE2 sector payloads).
fn ppt(prefix: &[u8], size: u64) -> bool {
    prefix.len() >= 516
        && size >= 516
        && (eq_at(prefix, 512, &[0x0F, 0x00, 0xE8, 0x03])
            || eq_at(prefix, 512, &[0xA0, 0x46, 0x1D, 0xF0])
            || eq_at(prefix, 512, &[0xFD, 0xFF, 0xFF, 0xFF])
            || eq_at(prefix, 512, &[0x00, 0x6E, 0x1E, 0xF0]))
}

/// Microsoft Publisher (`E7 AC 2C 00` within `0..=8`).
fn ms_pub(prefix: &[u8], _size: u64) -> bool {
    scan_range(prefix, 0, 8, &[0xE7, 0xAC, 0x2C, 0x00])
}

/// Apple QuickTime (`ftyp`, `moov`, `free`, `mdat`, `pnot`, `skip`, `wide` at `@4` — bare `"ftyp"`
/// at `@4` matches every ISO-BMFF file).
fn qt(prefix: &[u8], size: u64) -> bool {
    const TAGS: &[&[u8; 4]] = &[b"free", b"mdat", b"moov", b"pnot", b"skip", b"wide"];
    eq_at(prefix, 4, b"ftyp")
        || starts(prefix, &[0x00, 0x00, 0x00, 0x08, b'w', b'i', b'd', b'e'])
        || (prefix.len() >= 8 && size >= 8 && TAGS.iter().any(|t| eq_at(prefix, 4, *t)))
        || (prefix.len() >= 9 && TAGS.iter().any(|t| eq_at(prefix, 4, *t) && prefix[8] == 0x00))
}

/// R serialized workspace (`"RDX2\n"` or `"RDX3\n"`).
fn rdata(prefix: &[u8], _size: u64) -> bool {
    starts(prefix, b"RDX2\n") || starts(prefix, b"RDX3\n")
}

/// Rhinoceros 3D model (`"3D Geometry File Format       "`).
fn rhinoceros(prefix: &[u8], _size: u64) -> bool {
    const HEAD: &[u8] = b"3D Geometry File Format       "; // 30 bytes
    starts(prefix, HEAD)
        && ((prefix.len() >= 31 && matches!(prefix[30], b'5'..=b'8'))
            || (prefix.len() >= 36
                && prefix[30] == b' '
                && matches!(prefix[31], b'1'..=b'4')
                && eq_at(prefix, 32, &[0x01, 0x00, 0x00, 0x00])))
}

/// SquashFS superblock (`"hsqs"` or `"sqsh"`).
fn squashfs(prefix: &[u8], size: u64) -> bool {
    prefix.len() >= 4 && size >= 4 && (starts(prefix, b"hsqs") || starts(prefix, b"sqsh"))
}

/// POSIX `ustar` archive (`"ustar"` at `@257`).
fn tar(prefix: &[u8], size: u64) -> bool {
    (prefix.len() >= 262 && size >= 262 && eq_at(prefix, 257, b"ustar"))
        || (prefix.len() >= 263 && eq_at(prefix, 257, b"ustar\0"))
}

/// Truevision TGA (literal `"\\0\\2"` at `@1`).
fn tga(prefix: &[u8], size: u64) -> bool {
    prefix.len() >= 5 && size >= 5 && eq_at(prefix, 1, b"\\0\\2")
}

/// TIFF image (`"MM\x00*"` / `"MM\x00+"` / `"II*\x00"` / `"II+\x00"` / `"I I"`).
fn tiff(prefix: &[u8], size: u64) -> bool {
    starts(prefix, b"II+\0")
        || (prefix.len() >= 3 && size >= 3 && starts(prefix, b"I I"))
        || (prefix.len() >= 4
            && size >= 4
            && (starts(prefix, b"MM\0*") || starts(prefix, b"MM\0+") || starts(prefix, b"II*\0")))
}

/// WebM video: bare EBML header `1A 45 DF A3` at `@0` without a `DocType` check (overlaps `mkv`).
fn webm(prefix: &[u8], size: u64) -> bool {
    prefix.len() >= 4 && size >= 4 && starts(prefix, &[0x1A, 0x45, 0xDF, 0xA3])
}

/// WebP image: `RIFF....WEBP` or bare `"RIFF"` at `@0` (which overlaps `avi`, `wav`, `ani`).
fn webp(prefix: &[u8], size: u64) -> bool {
    (prefix.len() >= 12 && starts(prefix, b"RIFF") && eq_at(prefix, 8, b"WEBP"))
        || (prefix.len() >= 4 && size >= 4 && starts(prefix, b"RIFF"))
}

/// Windows Imaging Format (literal `"MSWIM\\000\\000\\000"`).
fn wim(prefix: &[u8], size: u64) -> bool {
    prefix.len() >= 17 && size >= 17 && starts(prefix, b"MSWIM\\000\\000\\000")
}

/// Windows Media Audio: bare ASF Header Object GUID at `@0` (overlaps `asf` and `wmv`).
fn wma(prefix: &[u8], size: u64) -> bool {
    prefix.len() >= 16 && size >= 16 && starts(prefix, ASF_HEADER_GUID)
}

/// Windows Metafile: placeable `D7 CD C6 9A 00 00`, standard `01 00 09 00 00 03`, or bare 2-byte
/// `01 00` at `@0` (too broad).
fn wmf(prefix: &[u8], size: u64) -> bool {
    starts(prefix, &[0x01, 0x00, 0x09, 0x00, 0x00, 0x03])
        || starts(prefix, &[0xD7, 0xCD, 0xC6, 0x9A, 0x00, 0x00])
        || (prefix.len() >= 2 && size >= 2 && starts(prefix, &[0x01, 0x00]))
}

/// Windows Media Video: ASF Header Object GUID (or its first 8 bytes) at `@0` (overlaps `asf` and
/// `wma`).
fn wmv(prefix: &[u8], size: u64) -> bool {
    prefix.len() >= 8 && size >= 8 && starts(prefix, &ASF_HEADER_GUID[..8])
}

/// Microsoft Excel 5.0 (`"Microsoft Excel 5.0 Worksheet"` at `@2080`).
fn xls(prefix: &[u8], size: u64) -> bool {
    prefix.len() >= 2109 && size >= 2109 && eq_at(prefix, 2080, b"Microsoft Excel 5.0 Worksheet")
}

/// XZ compressed stream (`FD 37 7A 58 5A 00`, flags, CRC32, and 4-byte-aligned size ≥ 32).
fn xz_v1(prefix: &[u8], size: u64) -> bool {
    if prefix.len() < 12
        || size < 32
        || size % 4 != 0
        || !starts(prefix, bytes!(&[0xFD], b"7zXZ\0\0"))
    {
        return false;
    }
    matches!(
        (prefix[7], le32(prefix, 8)),
        (0, Some(0x41D9_12FF))
            | (1, Some(0x36DE_2269))
            | (4, Some(0x46B4_D6E6))
            | (10, Some(0xA10C_FBE1))
    )
}

/// Bare 2-byte zlib header (`78 01`, `78 5E`, `78 9C`, `78 DA` at `@0`).
fn zlibstream(prefix: &[u8], _size: u64) -> bool {
    prefix.len() >= 2 && prefix[0] == 0x78 && matches!(prefix[1], 0x01 | 0x5E | 0x9C | 0xDA)
}

/// SQLite 3 database header (`"SQLite format 3\0"` — overlaps SQLite-based application formats such
/// as `geopackage`).
fn sqlite_v1(prefix: &[u8], size: u64) -> bool {
    if prefix.len() < 100 || size < 512 || !starts(prefix, b"SQLite format 3\0") {
        return false;
    }
    let page_code = be16(prefix, 16).unwrap_or(0);
    let page_aligned = match page_code {
        512 | 1024 | 2048 | 4096 | 8192 | 16384 | 32768 => size % u64::from(page_code) == 0,
        1 => size % 65536 == 0,
        _ => false,
    };
    page_aligned
        && matches!(prefix[18], 1 | 2)
        && matches!(prefix[19], 1 | 2)
        && prefix[21] == 64
        && prefix[22] == 32
        && prefix[23] == 32
        && matches!(be32(prefix, 44), Some(1..=4))
        && matches!(be32(prefix, 56), Some(1..=3))
        && eq_at(prefix, 72, &[0u8; 20])
        && be32(prefix, 96).unwrap_or(0) > 0
}

/// SEA ARC archive (`0x1A` + compression method byte + ASCII filename bytes).
fn arc(prefix: &[u8], size: u64) -> bool {
    if prefix.len() < 4 || prefix[0] != 0x1A {
        return false;
    }
    let m = prefix[1];
    (matches!(m, 0x02..=0x04 | 0x06 | 0x08..=0x0A | 0x14 | 0x48)
        && prefix[2] <= 0x7F
        && prefix[3] <= 0x7F)
        || (size >= 4
            && matches!(m, 0x02..=0x04 | 0x06 | 0x08 | 0x09)
            && prefix[2] == 0
            && prefix[3] == 0)
}

/// Apache ORC (`"ORC"` at `@0` — 3 ASCII bytes without EOF postscript validation).
fn orc(prefix: &[u8], _size: u64) -> bool {
    starts(prefix, b"ORC")
}

/// Microsoft Advanced Systems Format (`ASF_Header_Object` GUID at `@0`, subobject count `1..=64` at
/// `@24`, reserved bytes `01 02` at `@28` — overlaps `wma` and `wmv`).
fn asf(prefix: &[u8], _size: u64) -> bool {
    prefix.len() >= 30
        && starts(prefix, ASF_HEADER_GUID)
        && matches!(le32(prefix, 24), Some(1..=64))
        && be16(prefix, 28) == Some(0x0102)
}
