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

//! Layered, magic-keyed native dispatch cascade for Magika's pre-ML format rules.

use super::facts::Facts;
use crate::ContentType as Label;

/// Concatenate byte-slice parts into a compile-time constant byte array reference.
macro_rules! bytes {
    ($($part:expr),+ $(,)?) => {{
        const LEN: usize = 0 $(+ { let s: &[u8] = $part; s.len() })+;
        const ARR: [u8; LEN] = {
            let mut out = [0u8; LEN];
            let mut pos = 0;
            $(
                {
                    let slice: &[u8] = $part;
                    let mut i = 0;
                    while i < slice.len() {
                        out[pos] = slice[i];
                        pos += 1;
                        i += 1;
                    }
                }
            )+
            out
        };
        &ARR
    }};
}

/// Parse a canonical `"XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"` GUID literal into its 16-byte
/// mixed-endian binary representation at compile time.
const fn parse_guid(s: &str) -> [u8; 16] {
    const fn nibble(b: u8) -> u8 {
        match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => panic!("invalid hex digit in GUID"),
        }
    }
    const fn byte(raw: &[u8], idx: usize) -> u8 {
        (nibble(raw[idx]) << 4) | nibble(raw[idx + 1])
    }
    let b = s.as_bytes();
    assert!(b.len() == 36, "GUID string must be 36 characters");
    assert!(
        b[8] == b'-' && b[13] == b'-' && b[18] == b'-' && b[23] == b'-',
        "GUID hyphens must be at 8, 13, 18, 23"
    );
    [
        // Data1 (u32 little-endian)
        byte(b, 6),
        byte(b, 4),
        byte(b, 2),
        byte(b, 0),
        // Data2 (u16 little-endian)
        byte(b, 11),
        byte(b, 9),
        // Data3 (u16 little-endian)
        byte(b, 16),
        byte(b, 14),
        // Data4 (8 bytes big-endian)
        byte(b, 19),
        byte(b, 21),
        byte(b, 24),
        byte(b, 26),
        byte(b, 28),
        byte(b, 30),
        byte(b, 32),
        byte(b, 34),
    ]
}

/// Compile-time 16-byte mixed-endian GUID slice from a string literal.
macro_rules! guid {
    ($s:expr) => {{
        const G: [u8; 16] = parse_guid($s);
        &G
    }};
}

/// Early-return `None` on the first false precondition.
macro_rules! require {
    ($($cond:expr),+ $(,)?) => {
        if !($($cond)&&+) {
            return None;
        }
    };
}

/// Big-endian `u32` representation of a 4-byte ASCII tag.
#[inline]
pub(crate) const fn tag_u32(tag: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*tag)
}

/// Whether `h` starts with `sig`.
#[inline]
pub(crate) fn starts(h: &[u8], sig: &[u8]) -> bool {
    h.starts_with(sig)
}

/// Whether `sig` appears at byte offset `off` inside `h`.
#[inline]
pub(crate) fn eq_at(h: &[u8], off: usize, sig: &[u8]) -> bool {
    h.get(off..).is_some_and(|s| s.starts_with(sig))
}

/// Bounds-checked `u8` read at `off`.
#[inline]
pub(crate) fn u8_at(h: &[u8], off: usize) -> Option<u8> {
    h.get(off).copied()
}

/// Bounds-checked little-endian `u16` read at `off`.
#[inline]
pub(crate) fn le16(h: &[u8], off: usize) -> Option<u16> {
    Some(u16::from_le_bytes(h.get(off..off.checked_add(2)?)?.try_into().ok()?))
}

/// Bounds-checked big-endian `u16` read at `off`.
#[inline]
pub(crate) fn be16(h: &[u8], off: usize) -> Option<u16> {
    Some(u16::from_be_bytes(h.get(off..off.checked_add(2)?)?.try_into().ok()?))
}

/// Bounds-checked little-endian `u32` read at `off`.
#[inline]
pub(crate) fn le32(h: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_le_bytes(h.get(off..off.checked_add(4)?)?.try_into().ok()?))
}

/// Bounds-checked big-endian `u32` read at `off`.
#[inline]
pub(crate) fn be32(h: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_be_bytes(h.get(off..off.checked_add(4)?)?.try_into().ok()?))
}

/// Whether `needle` occurs starting at some offset in `lo..=hi` within `h`.
#[inline]
pub(crate) fn scan_range(h: &[u8], lo: usize, hi: usize, needle: &[u8]) -> bool {
    let end = h.len().min(hi.saturating_add(needle.len()));
    h.get(lo..end).is_some_and(|w| memchr::memmem::find(w, needle).is_some())
}

/// Per-input scan context.
#[derive(Clone, Copy)]
pub(crate) struct Ctx<'a> {
    pub(crate) prefix: &'a [u8],
    pub(crate) size: u64,
    pub(crate) tail: Option<&'a [u8]>,
}

/// Tier 1 (MAGIC32): 4-byte magic at offset 0.
#[inline]
fn magic32(key: &[u8; 4], prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    match key {
        b"3DSX" => threedsx(prefix),    // Nintendo 3DS homebrew executable
        b"bpli" => applebplist(prefix), // Apple binary property list ("bplist...")
        &[0x00, 0x05, 0x16, 0x07] => appledouble(prefix), // AppleDouble resource file
        &[0x00, 0x05, 0x16, 0x00] => applesingle(prefix), // AppleSingle archive
        b".snd" | b"dns." => au(prefix), // Sun/NeXT or DEC AU audio
        b"RIFF" | b"RIFX" | b"RF64" => riff(prefix), // RIFF/RIFX/RF64 container (AVI/WAV/ANI)
        b"Obj\x01" => avro(prefix),     // Apache Avro Object Container File
        b"BAM\x01" => bam(prefix),      // Binary Alignment Map
        &[0x0F, 0x37, b'B', b'E'] | b"FOR1" => beam(prefix), // Erlang BEAM module
        b"BLEN" => blend(prefix),       // Blender project ("BLENDER...")
        b"BPG\xfb" => bpg(prefix),      // Better Portable Graphics
        b"CRAM" => cram(prefix),        // CRAM compressed alignment
        b"\x7fELF" => cubin(prefix),    // ELF image (CUDA cubin)
        b"dex\n" => dex(prefix),        // Dalvik Executable
        &[0x00, 0x00, 0x00, 0x01] => dsstore(prefix), // macOS .DS_Store Buddy allocator
        b"AC10" => dwg(prefix),         // AutoCAD DWG drawing
        b"Kayd" | b"; FB" => fbx(prefix), // Autodesk FBX (binary or ASCII)
        b"SIMP" => fits(prefix),        // FITS image/table ("SIMPLE  =...")
        b"FLV\x01" => flv(prefix),      // Adobe Flash Video v1
        b"SQLi" => geopackage(prefix),  // OGC GeoPackage on SQLite 3
        b"GGUF" => gguf(prefix),        // GGML Universal File
        b"glTF" => glb(prefix),         // Khronos binary glTF 2.0
        b"GRIB" => grib(prefix),        // WMO GRIB weather data
        &[0x0E, 0x03, 0x13, 0x01] => hdf4(prefix), // NCSA HDF4 data file
        b"regf" => hve(prefix),         // Windows registry hive base block
        b"icns" => icns(prefix),        // Apple Icon Image container
        b"\x8bJNG" => jng(prefix),      // JPEG Network Graphics
        b"BC\xc0\xde" | &[0xDE, 0xC0, 0x17, 0x0B] => llvm_bitcode(prefix, ctx), // LLVM bitcode (raw or wrapper)
        &[0x4C, 0x00, 0x00, 0x00] => lnk(prefix), // Windows Shell Link (.lnk)
        b"LRZI" => lrz(prefix),                   // Long Range ZIP
        b"\x1bLua" | b"\x1bLJ\x01" | b"\x1bLJ\x02" => luabytecode(prefix), // Lua / LuaJIT bytecode
        b"LZIP" => lzip(prefix, ctx),             // Lzip compressed member
        &[0x02 | 0x03, 0x21, 0x4C, 0x18] | &[0x04, 0x22, 0x4D, 0x18] => lz4(prefix, ctx), // LZ4 frame
        b"MATL" => mat(prefix),      // MATLAB v5 MAT-file ("MATLAB 5")
        b"MThd" => midi(prefix),     // Standard MIDI File
        b"MDMP" => minidump(prefix), // Windows Minidump
        b"\x93NUM" => npy(prefix),   // NumPy array ("\x93NUMPY")
        b"NRRD" => nrrd(prefix),     // Teem NRRD volume
        &[0xE4, 0x52, 0x5C, 0x7B] => one(prefix), // Microsoft OneNote section
        b"PAR1" => parquet(prefix),  // Apache Parquet column store
        b"----" => pem_or_pgp(prefix), // OpenPGP ASCII armor or RFC 7468 PEM
        b"ply\n" | b"ply\r" => ply(prefix), // Polygon File Format
        b"PGDM" => postgres_dump(prefix), // PostgreSQL custom dump ("PGDMP")
        b"8BPS" => psd(prefix),      // Adobe Photoshop PSD/PSB
        b"qoif" => qoi(prefix),      // Quite OK Image
        b"Rar!" | b"RE~^" => rar(prefix), // RAR archive (v4/v5 or legacy)
        b"REDI" => redis_rdb(prefix), // Redis RDB snapshot ("REDIS...")
        b"RZIP" => rzip(prefix),     // rzip compressed file
        &[0x37, 0x7A, 0xBC, 0xAF] => sevenzip(prefix), // 7-Zip archive
        &[0x00, 0x00, 0x27, 0x0A] => shapefile(prefix), // ESRI Shapefile (.shp/.shx, code 9994)
        &[0xFF, 0xFE, 0xFF, 0x0E] | b"\x0eSke" => sketchup(prefix), // SketchUp model
        &[0x03, 0x02, 0x23, 0x07] | &[0x07, 0x23, 0x02, 0x03] => spirv(prefix, ctx), // Khronos SPIR-V module
        b"$FL2" | b"$FL3" | &[0xC9, 0xC3, 0xE2, 0xC1] => spss(prefix), // SPSS SAV/ZSAV or portable
        b"ISO-" => step(prefix),          // ISO 10303-21 STEP exchange file
        b"UF2\n" => uf2(prefix),          // Microsoft UF2 bootloader image
        b"cone" => vhd(prefix),           // Microsoft VHD footer ("conectix")
        b"!<ar" => vib(prefix),           // VMware VIB in Unix ar ("!<arch>\n")
        b"IWAD" | b"PWAD" => wad(prefix), // Doom WAD archive
        b"\0asm" => wasm(prefix),         // WebAssembly binary module
        b"wOFF" => woff(prefix),          // W3C WOFF 1.0 font
        b"wOF2" => woff2(prefix),         // W3C WOFF 2.0 font
        b"xar!" => xar(prefix),           // eXtensible ARchive
        b"gimp" => xcf(prefix),           // GIMP XCF image ("gimp xcf ")
        &[0x22..=0x28, 0xB5, 0x2F, 0xFD] => zst(prefix), // Zstandard frame
        b"ARRO" => arrow(prefix),         // Apache Arrow IPC ("ARROW1\0\0")
        b"\x0a\x0d\x0d\x0a" => pcapng(prefix), // IETF pcapng Section Header Block
        b"MSCF" => cab(prefix),           // Microsoft Cabinet archive
        b"Cr24" => crx(prefix),           // Chrome extension package
        b"fLaC" => flac(prefix),          // Free Lossless Audio Codec
        b"GIF8" => gif(prefix),           // CompuServe GIF87a / GIF89a
        &[0x3F, 0x5F, 0x03, 0x00] => hlp(prefix), // Windows Help (.hlp)
        &[0x00, 0x00, 0x01, 0x00] => ico(prefix), // Windows Icon directory
        b"FORM" => ilbm(prefix),          // EA IFF 85 FORM ILBM/ACBM/PBM
        &[0x00, 0x00, 0x00, 0x0C] => jp2_or_jxl(prefix), // ISO-BMFF 12-byte signature box (JP2/JXL)
        &[0xFF, 0x80, 0xC9, 0xC7] => koala(prefix), // Commodore 64 Koala Painter image
        &[0xCE | 0xCF, 0xFA, 0xED, 0xFE] | &[0xFE, 0xED, 0xFA, 0xCE | 0xCF] => macho(prefix), // Apple Mach-O thin binary
        &[0x1A, 0x45, 0xDF, 0xA3] => mkv(prefix), // Matroska EBML container
        b"SZDD" => mscompress(prefix),            // Microsoft SZDD compressed file
        b"CDF\x01" | b"CDF\x02" => netcdf(prefix), // Unidata NetCDF classic / 64-bit
        b"OggS" => ogg(prefix),                   // Ogg bitstream page
        &[0xD4, 0xC3, 0xB2, 0xA1]
        | &[0x4D, 0x3C, 0xB2, 0xA1]
        | &[0xA1, 0xB2, 0xC3, 0xD4]
        | &[0xA1, 0xB2, 0x3C, 0x4D] => pcap(prefix), // libpcap capture file
        b"Micr" => pdb(prefix),                   // Microsoft PDB (MSF 7.00 / JG 2.00)
        &[0xED, 0xAB, 0xEE, 0xDB] => rpm(prefix), // RPM package lead
        b"<sta" => stata(prefix),                 // Stata XML data ("<stata_dta>...")
        b"%!PS" => postscript(prefix),            // Adobe PostScript DSC ("%!PS-Adobe")
        &[0x00, 0x00, 0x00, 0x00] => sas(prefix), // SAS7BDAT dataset (32-byte magic starting with 12 NULs)
        &[0x01, 0x00, 0x00, 0x00] => emf(prefix), // Microsoft Enhanced Metafile (EMR_HEADER = 1)
        b"d13:" | b"d4:i" | b"d8:a" | b"d7:c" => torrent(prefix), // BitTorrent metainfo bencoded dict
        b"PK\x03\x04" | b"PK\x05\x06" => zip(prefix, ctx), // PKZIP archive (local header or EOCD)
        _ => None,
    }
}

/// Tier 2: 2-byte magic at offset 0.
#[inline]
fn magic16(key: &[u8; 2], prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    match key {
        b"MM" => three_dsm(prefix),    // Autodesk 3ds Max M3DMAGIC (0x4D4D)
        b"BZ" => bzip(prefix),         // bzip2 stream ("BZh[1-9]")
        &[0x1F, 0x8B] => gzip(prefix), // RFC 1952 gzip member
        b"P1" | b"P2" | b"P3" | b"P4" | b"P5" | b"P6" => pbm(prefix), // Netpbm P1..P6 image
        b"FW" | b"CW" | b"ZW" => swf(prefix), // Adobe Flash SWF (FWS/CWS/ZWS)
        &[0x01, 0xF7 | 0xDF] => xcoff(prefix), // IBM AIX XCOFF64 (0x01F7) / XCOFF32 (0x01DF)
        b"BM" => bmp(prefix),          // Windows / OS/2 BMP image
        &[0x1F, 0x9D] => unixcompress(prefix), // Unix compress (LZW)
        &[0x00, 0x00..=0x02] => degas(prefix, ctx), // Atari ST DEGAS screen dump (resolution 0..=2)
        b"MZ" => pe(prefix, ctx),      // DOS MZ / Windows PE executable
        _ => None,
    }
}

/// A bounds-checked 32-bit integer reader (`le32` or `be32`).
type ReadU32 = fn(&[u8], usize) -> Option<u32>;

/// Tier 3: signatures at a fixed non-zero offset.
#[inline]
fn offset_probes(prefix: &[u8]) -> Option<Label> {
    ftyp(prefix)
        .or_else(|| ace(prefix))
        .or_else(|| dicom(prefix))
        .or_else(|| duckdb(prefix))
        .or_else(|| ese(prefix))
        .or_else(|| icc(prefix))
        .or_else(|| palmos(prefix))
        .or_else(|| osm_pbf(prefix))
}

/// Tier 4: structural and text heuristics without a fixed magic key.
#[inline]
fn heuristics(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    let xml = xml_roots(prefix);
    if xml.collada {
        return Some(Label::Collada);
    }
    if let Some(label) = gltf_json(prefix) {
        return Some(label);
    }
    if xml.gpx {
        return Some(Label::Gpx);
    }
    if xml.kml {
        return Some(Label::Kml);
    }
    if let Some(label) = mpegts(prefix) {
        return Some(label);
    }
    if let Some(label) = safetensors(prefix) {
        return Some(label);
    }
    if let Some(label) = dbase(prefix, ctx) {
        return Some(label);
    }
    if let Some(label) = intelhex(prefix) {
        return Some(label);
    }
    if xml.osm {
        return Some(Label::Osm);
    }
    pythonbytecode(prefix)
}

/// Runs the 4-tier cascade over `ctx`.
#[inline]
pub(crate) fn identify(ctx: &Ctx<'_>) -> Option<Label> {
    let prefix = ctx.prefix;
    if let Some(key) = prefix.first_chunk::<4>()
        && let Some(label) = magic32(key, prefix, ctx)
    {
        return Some(label);
    }
    if let Some(key) = prefix.first_chunk::<2>()
        && let Some(label) = magic16(key, prefix, ctx)
    {
        return Some(label);
    }
    if let Some(label) = offset_probes(prefix) {
        return Some(label);
    }
    heuristics(prefix, ctx)
}

// =============================================================================
// Tier 1 (MAGIC32) verifiers
// =============================================================================

/// Nintendo 3DS homebrew executable (3DSX): magic `"3DSX"`, header size (32 base or 44 extended),
/// relocation header size 8, version 0, flags 0, and 4-byte-aligned segment sizes.
fn threedsx(prefix: &[u8]) -> Option<Label> {
    const TAG: &[u8] = b"3DSX"; // @0..4: format signature
    const BASE_HDR: &[u8] = bytes!(TAG, &32u16.to_le_bytes(), &8u16.to_le_bytes(), &[0u8; 8]); // @0..16: 32-byte header
    const EXT_HDR: &[u8] = bytes!(TAG, &44u16.to_le_bytes(), &8u16.to_le_bytes(), &[0u8; 8]); // @0..16: 44-byte extended header
    const BASE_MIN_LEN: usize = 56; // base header (32) + 3 relocation headers (24)
    const EXT_MIN_LEN: usize = 68; // extended header (44) + 3 relocation headers (24)
    const CODE_SEG_SIZE: usize = 16; // @16: u32 LE — code segment size (nonzero, 4-byte aligned)
    const RODATA_SEG_SIZE: usize = 20; // @20: u32 LE — rodata segment size (4-byte aligned)
    const DATA_BSS_SIZE: usize = 24; // @24: u32 LE — data+bss segment size (4-byte aligned)
    const BSS_SIZE: usize = 28; // @28: u32 LE — bss size (4-byte aligned)

    let has_header = (starts(prefix, BASE_HDR) && prefix.len() >= BASE_MIN_LEN)
        || (starts(prefix, EXT_HDR) && prefix.len() >= EXT_MIN_LEN);
    require!(has_header);
    let code = le32(prefix, CODE_SEG_SIZE)?;
    require!(
        code >= 4,
        code % 4 == 0,
        le32(prefix, RODATA_SEG_SIZE)? % 4 == 0,
        le32(prefix, DATA_BSS_SIZE)? % 4 == 0,
        le32(prefix, BSS_SIZE)? % 4 == 0,
    );
    Some(Label::_3dsx)
}

/// Apple binary property list: `"bplist"` magic followed by classic `"0?"` (with 32-byte trailer),
/// modern `"10"`..`"19"`, or binary version words `00 00`, `00 01`, or `40 00`.
fn applebplist(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"bplist"; // @0..6: binary plist signature
    const MIN_LEN: usize = 8; // 6-byte magic + 2-byte version
    const CLASSIC_MIN_LEN: usize = 41; // 8-byte header + ≥1 object byte + 32-byte trailer
    const VER_MAJOR: usize = 6; // @6: u8 — major version byte
    const VER_MINOR: usize = 7; // @7: u8 — minor version byte

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    let (v0, v1) = (u8_at(prefix, VER_MAJOR)?, u8_at(prefix, VER_MINOR)?);
    let valid = (v0 == b'0' && prefix.len() >= CLASSIC_MIN_LEN)
        || (v0 == b'1' && v1.is_ascii_digit())
        || (v0 == 0x00 && matches!(v1, 0x00 | 0x01))
        || (v0 == 0x40 && v1 == 0x00);
    require!(valid);
    Some(Label::Applebplist)
}

/// AppleDouble resource file: magic `0x00051607`, version 1 (`0x00010000`) or 2 (`0x00020000`),
/// and complete 26-byte fixed header.
fn appledouble(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = &0x0005_1607u32.to_be_bytes(); // @0..4: AppleDouble magic
    const VERSION: usize = 4; // @4..8: u32 BE — version 1 or 2
    const V1: u32 = 0x0001_0000; // version 1.0
    const V2: u32 = 0x0002_0000; // version 2.0
    const MIN_LEN: usize = 26; // fixed 26-byte header

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(be32(prefix, VERSION)?, V1 | V2),
    );
    Some(Label::Appledouble)
}

/// AppleSingle archive: magic `0x00051600`, version 1 (`0x00010000`) or 2 (`0x00020000`),
/// and complete 26-byte fixed header.
fn applesingle(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = &0x0005_1600u32.to_be_bytes(); // @0..4: AppleSingle magic
    const VERSION: usize = 4; // @4..8: u32 BE — version 1 or 2
    const V1: u32 = 0x0001_0000; // version 1.0
    const V2: u32 = 0x0002_0000; // version 2.0
    const MIN_LEN: usize = 26; // fixed 26-byte header

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(be32(prefix, VERSION)?, V1 | V2),
    );
    Some(Label::Applesingle)
}

/// Sun/NeXT (`.snd`, big-endian) or DEC (`dns.`, little-endian) AU audio file: 24-byte header,
/// data offset ≥ 24, defined encoding code (1..=14 or 16..=27), nonzero sample rate and channels.
fn au(prefix: &[u8]) -> Option<Label> {
    const BIG_MAGIC: &[u8] = b".snd"; // @0..4: big-endian AU magic
    const LITTLE_MAGIC: &[u8] = b"dns."; // @0..4: little-endian AU magic
    const MIN_LEN: usize = 24; // 6 × u32 header
    const DATA_OFFSET: usize = 4; // @4: u32 — byte offset to audio data (≥ 24)
    const ENCODING: usize = 12; // @12: u32 — sample encoding format (1..=14 or 16..=27)
    const SAMPLE_RATE: usize = 16; // @16: u32 — sample rate in Hz (> 0)
    const CHANNELS: usize = 20; // @20: u32 — number of channels (> 0)

    require!(prefix.len() >= MIN_LEN);
    let read_u32: fn(&[u8], usize) -> Option<u32> = if starts(prefix, BIG_MAGIC) {
        be32
    } else if starts(prefix, LITTLE_MAGIC) {
        le32
    } else {
        return None;
    };
    let enc = read_u32(prefix, ENCODING)?;
    require!(
        read_u32(prefix, DATA_OFFSET)? >= MIN_LEN as u32,
        (1..=14).contains(&enc) || (16..=27).contains(&enc),
        read_u32(prefix, SAMPLE_RATE)? > 0,
        read_u32(prefix, CHANNELS)? > 0,
    );
    Some(Label::Au)
}

/// RIFF / RIFX / RF64 container overlap verifier: dispatches on the form type at offset 8
/// between AVI (`"AVI "` + `"LIST"`/`"hdrl"`), WAV (`"WAVE"`), and ANI (`"ACON"`).
fn riff(prefix: &[u8]) -> Option<Label> {
    const RIFF_TAG: &[u8] = b"RIFF"; // @0..4: little-endian RIFF container
    const RIFX_TAG: &[u8] = b"RIFX"; // @0..4: big-endian RIFX container
    const RF64_HDR: &[u8] = bytes!(b"RF64", &u32::MAX.to_le_bytes(), b"WAVEds64"); // @0..16: RF64 WAVE + ds64 chunk
    const CHUNK_SIZE: usize = 4; // @4: u32 — container chunk size
    const FORM_TYPE: usize = 8; // @8..12: 4-byte form type ("AVI ", "WAVE", "ACON")
    const AVI_LIST: &[u8] = b"AVI LIST"; // @8..16: AVI form + initial LIST chunk
    const LIST_SIZE: usize = 16; // @16: u32 LE — LIST/ds64 chunk size
    const HDRL_TAG: &[u8] = b"hdrl"; // @20..24: AVI header list type
    const AVI_MIN_LEN: usize = 24;
    const WAV_MIN_LEN: usize = 12;
    const RF64_MIN_LEN: usize = 48;

    require!(prefix.len() >= WAV_MIN_LEN);
    if starts(prefix, RIFF_TAG) {
        if prefix.len() >= AVI_MIN_LEN
            && eq_at(prefix, FORM_TYPE, AVI_LIST)
            && eq_at(prefix, 20, HDRL_TAG)
            && le32(prefix, CHUNK_SIZE)? >= 16
            && le32(prefix, LIST_SIZE)? >= 4
        {
            return Some(Label::Avi);
        }
        if eq_at(prefix, FORM_TYPE, b"WAVE") && le32(prefix, CHUNK_SIZE)? >= 4 {
            return Some(Label::Wav);
        }
        if be32(prefix, FORM_TYPE)? == tag_u32(b"ACON") {
            return Some(Label::Ani);
        }
    } else if starts(prefix, RIFX_TAG) {
        if eq_at(prefix, FORM_TYPE, b"WAVE") && be32(prefix, CHUNK_SIZE)? >= 4 {
            return Some(Label::Wav);
        }
    } else if prefix.len() >= RF64_MIN_LEN
        && starts(prefix, RF64_HDR)
        && le32(prefix, LIST_SIZE)? >= 28
    {
        return Some(Label::Wav);
    }
    None
}

/// Apache Avro Object Container File: `"Obj\x01"` magic and nonempty metadata map (byte 4 ≥ 1).
fn avro(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"Obj\x01"; // @0..4: Avro OCF version 1 magic
    const META_COUNT: usize = 4; // @4: u8 — first byte of zig-zag metadata block count (≥ 1)
    const MIN_LEN: usize = 21; // 4-byte magic + ≥1 metadata byte + 16-byte sync marker

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC), u8_at(prefix, META_COUNT)? >= 1);
    Some(Label::Avro)
}

/// Binary Alignment Map (BAM): `"BAM\x01"` magic and 8 bytes of header length / reference count.
fn bam(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"BAM\x01"; // @0..4: BAM v1 magic
    const MIN_LEN: usize = 12; // 4-byte magic + l_text (4) + n_ref (4)

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    Some(Label::Bam)
}

/// Erlang BEAM module: legacy `"\x0f7BEAM!"` header or EA IFF `"FOR1"` container with `"BEAM"` form.
fn beam(prefix: &[u8]) -> Option<Label> {
    const LEGACY_MAGIC: &[u8] = b"\x0f7BEAM!"; // @0..7: legacy BEAM header
    const IFF_FOR1: &[u8] = b"FOR1"; // @0..4: EA IFF 85 FOR1 chunk
    const BEAM_FORM: &[u8] = b"BEAM"; // @8..12: BEAM form identifier
    const FORM_OFFSET: usize = 8; // @8: form type offset
    const MIN_LEN: usize = 8;
    const IFF_MIN_LEN: usize = 12;

    require!(prefix.len() >= MIN_LEN);
    let matched = starts(prefix, LEGACY_MAGIC)
        || (prefix.len() >= IFF_MIN_LEN
            && starts(prefix, IFF_FOR1)
            && eq_at(prefix, FORM_OFFSET, BEAM_FORM));
    require!(matched);
    Some(Label::Beam)
}

/// Blender project file (pre-v5): `"BLENDER"` magic, pointer-width indicator (`_` or `-`),
/// endianness indicator (`v` or `V`), and three-digit version `100`..`499`.
fn blend(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"BLENDER"; // @0..7: Blender file magic
    const PTR_SIZE: usize = 7; // @7: '_' (32-bit) or '-' (64-bit)
    const ENDIAN: usize = 8; // @8: 'v' (little-endian) or 'V' (big-endian)
    const VER_MAJOR: usize = 9; // @9: '1'..='4'
    const MIN_LEN: usize = 32; // 12-byte file header + first BHead

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, PTR_SIZE)?, b'_' | b'-'),
        matches!(u8_at(prefix, ENDIAN)?, b'v' | b'V'),
        matches!(u8_at(prefix, VER_MAJOR)?, b'1'..=b'4'),
        u8_at(prefix, 10)?.is_ascii_digit(),
        u8_at(prefix, 11)?.is_ascii_digit(),
    );
    Some(Label::Blend)
}

/// Better Portable Graphics (BPG): `"BPG\xfb"` magic, valid pixel format / bit depth byte,
/// color space / flags byte, and two nonzero `ue7(32)` dimensions followed by at least one byte.
fn bpg(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"BPG\xfb"; // @0..4: BPG header magic
    const FORMAT_DEPTH: usize = 4; // @4: pixel_format (bits 7..5, 0..=5) | alpha1 | bit_depth_minus_8 (0..=6)
    const COLOR_FLAGS: usize = 5; // @5: color_space | extension_present | alpha2 | limited_range | animation
    const DIMS_OFFSET: usize = 6; // @6: width and height encoded as ue7(32) varints
    const MIN_LEN: usize = 9;

    /// Advance past one nonzero `ue7(32)` varint at `pos`: `[0x01..=0x7F]` or
    /// `[0x81..=0xFF][0x80..=0xFF]{0,2}[0x00..=0x7F]` or `[0x81..=0x8F][0x80..=0xFF]{3}[0x00..=0x7F]`.
    fn skip_ue7(h: &[u8], pos: usize) -> Option<usize> {
        let first = u8_at(h, pos)?;
        if first == 0x00 || first == 0x80 {
            return None;
        }
        if first < 0x80 {
            return Some(pos + 1);
        }
        let max_cont = if first <= 0x8F { 3 } else { 2 };
        for cont in 0..=max_cont {
            if u8_at(h, pos + 1 + cont)? < 0x80 {
                return Some(pos + 2 + cont);
            }
        }
        None
    }

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    let b4 = u8_at(prefix, FORMAT_DEPTH)?;
    let b5 = u8_at(prefix, COLOR_FLAGS)?;
    require!((b4 & 0x0F) <= 6, (b4 >> 4) <= 0x0B, b5 < 80, b4 >= 32 || b5 < 16,);
    let after_w = skip_ue7(prefix, DIMS_OFFSET)?;
    let after_h = skip_ue7(prefix, after_w)?;
    require!(after_h < prefix.len());
    Some(Label::Bpg)
}

/// CRAM compressed sequence alignment: `"CRAM"` magic, major/minor version `(1, 0)`, `(2, 0|1)`,
/// or `(3, 0|1)`, and complete 26-byte file definition header.
fn cram(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"CRAM"; // @0..4: CRAM format magic
    const MAJOR: usize = 4; // @4: u8 — major version (1..=3)
    const MINOR: usize = 5; // @5: u8 — minor version (0..=1)
    const MIN_LEN: usize = 26; // 26-byte CRAM file definition

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!((u8_at(prefix, MAJOR)?, u8_at(prefix, MINOR)?), (1, 0) | (2 | 3, 0 | 1)),
    );
    Some(Label::Cram)
}

/// NVIDIA CUDA binary (cubin): an ELF image (`"\x7fELF"`) whose `e_machine` at offset 18 is
/// `EM_CUDA` (`190`) in the endianness declared by `EI_DATA` at offset 5.
fn cubin(prefix: &[u8]) -> Option<Label> {
    const ELF_MAGIC: &[u8] = b"\x7fELF"; // @0..4: ELF magic
    const EI_DATA: usize = 5; // @5: u8 — 1 = ELFDATA2LSB, 2 = ELFDATA2MSB
    const E_MACHINE: usize = 18; // @18: u16 — ELF e_machine field
    const EM_CUDA: u16 = 190; // NVIDIA CUDA architecture
    const MIN_LEN: usize = 20;

    require!(prefix.len() >= MIN_LEN, starts(prefix, ELF_MAGIC));
    let is_cuda = match u8_at(prefix, EI_DATA)? {
        1 => le16(prefix, E_MACHINE)? == EM_CUDA,
        2 => be16(prefix, E_MACHINE)? == EM_CUDA,
        _ => false,
    };
    require!(is_cuda);
    Some(Label::Cubin)
}

/// Dalvik Executable (DEX): `"dex\n"` followed by `"[0-9]{3}\x00"`, matching header size
/// (112 standard, 120 for v041 container) and `ENDIAN_CONSTANT` (`0x12345678`) in either byte order.
fn dex(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"dex\n"; // @0..4: DEX magic prefix
    const VERSION: usize = 4; // @4..8: 3 ASCII digits + NUL
    const VER_041: u32 = tag_u32(b"041\0"); // DEX v041 container version
    const HEADER_SIZE: usize = 36; // @36: u32 — header size (112 or 120)
    const ENDIAN_TAG: usize = 40; // @40: u32 — endianness constant
    const ENDIAN_CONSTANT: u32 = 0x1234_5678; // standard DEX endian marker
    const STD_HDR_LEN: usize = 112;
    const V041_HDR_LEN: usize = 120;

    require!(
        prefix.len() >= STD_HDR_LEN,
        starts(prefix, MAGIC),
        prefix[4..7].iter().all(u8::is_ascii_digit),
        u8_at(prefix, 7)? == 0,
    );
    let expected_hdr = if be32(prefix, VERSION)? == VER_041 {
        require!(prefix.len() >= V041_HDR_LEN);
        V041_HDR_LEN as u32
    } else {
        STD_HDR_LEN as u32
    };
    let valid_endian = (le32(prefix, HEADER_SIZE)? == expected_hdr
        && le32(prefix, ENDIAN_TAG)? == ENDIAN_CONSTANT)
        || (be32(prefix, HEADER_SIZE)? == expected_hdr
            && be32(prefix, ENDIAN_TAG)? == ENDIAN_CONSTANT);
    require!(valid_endian);
    Some(Label::Dex)
}

/// macOS Finder `.DS_Store`: alignment word `1` + `"Bud1\0"` Buddy allocator magic and root block
/// offset / size bounds.
fn dsstore(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(&1u32.to_be_bytes(), b"Bud1\0"); // @0..9: alignment word + Bud1 + high zero byte
    const ROOT_OFFSET: usize = 8; // @8: u32 BE — allocator book-keeping offset (≥ 32)
    const ROOT_SIZE: usize = 12; // @12: u32 BE — allocator book-keeping size (≥ 12)
    const MIN_LEN: usize = 36;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, HEADER),
        be32(prefix, ROOT_OFFSET)? >= 32,
        be32(prefix, ROOT_SIZE)? >= 12,
    );
    Some(Label::Dsstore)
}

/// AutoCAD DWG drawing: `"AC10"` version prefix with known release code (`09`, `12`, `14`, `15`,
/// `18`, `21`, `24`, `27`, `32`), five zero bytes at `@6..11`, and ≥ 128-byte header.
fn dwg(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"AC10"; // @0..4: AutoCAD DWG version prefix
    const ZERO_PAD: usize = 6; // @6..11: 5 reserved zero bytes
    const MIN_LEN: usize = 128;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(
            &prefix[4..6],
            b"09" | b"12" | b"14" | b"15" | b"18" | b"21" | b"24" | b"27" | b"32"
        ),
        eq_at(prefix, ZERO_PAD, &[0; 5]),
    );
    Some(Label::Dwg)
}

/// Autodesk FBX: binary `"Kaydara FBX Binary  \x00\x1a\x00"` with version `3000..=7700` at `@23`,
/// or ASCII export header comment `"; FBX X.Y.Z project file"`.
fn fbx(prefix: &[u8]) -> Option<Label> {
    const BIN_MAGIC: &[u8] = b"Kaydara FBX Binary  \x00\x1a\x00"; // @0..23: binary FBX signature
    const BIN_VERSION: usize = 23; // @23: u32 LE — FBX version (3000..=7700)
    const BIN_MIN_LEN: usize = 27;
    const ASCII_PREFIX: &[u8] = b"; FBX "; // @0..6: ASCII FBX header comment start
    const ASCII_SUFFIX: &[u8] = b" project file"; // @11..24: ASCII FBX header comment tail
    const ASCII_MIN_LEN: usize = 24;

    if prefix.len() >= BIN_MIN_LEN
        && starts(prefix, BIN_MAGIC)
        && (3000..=7700).contains(&le32(prefix, BIN_VERSION)?)
    {
        return Some(Label::Fbx);
    }
    require!(
        prefix.len() >= ASCII_MIN_LEN,
        starts(prefix, ASCII_PREFIX),
        u8_at(prefix, 6)?.is_ascii_digit(),
        u8_at(prefix, 7)? == b'.',
        u8_at(prefix, 8)?.is_ascii_digit(),
        u8_at(prefix, 9)? == b'.',
        u8_at(prefix, 10)?.is_ascii_digit(),
        eq_at(prefix, 11, ASCII_SUFFIX),
    );
    Some(Label::Fbx)
}

/// Flexible Image Transport System (FITS): complete 2880-byte primary header block opening with a
/// `"SIMPLE  = "` card (`T` or `F`) and a `"BITPIX  = "` card at offset 80 or 160.
fn fits(prefix: &[u8]) -> Option<Label> {
    const BLOCK_LEN: usize = 2880; // one 36 × 80-byte FITS header block
    const CARD_LEN: usize = 80; // 80-byte FITS header card
    const SECOND_CARD_PAD: usize = 89; // @89..91: two spaces in second card value field

    fn match_simple(card: &[u8]) -> bool {
        let Some(rest) = card.strip_prefix(b"SIMPLE  =") else { return false };
        let spaces = rest.iter().take_while(|&&b| b == b' ').count();
        (1..=20).contains(&spaces)
            && matches!(rest.get(spaces), Some(b'T' | b'F'))
            && matches!(rest.get(spaces + 1), Some(b' ' | b'/'))
    }

    fn match_bitpix(card: &[u8]) -> bool {
        let Some(rest) = card.strip_prefix(b"BITPIX  =") else { return false };
        let spaces = rest.iter().take_while(|&&b| b == b' ').count();
        if !(1..=20).contains(&spaces) {
            return false;
        }
        let after_spaces = &rest[spaces..];
        for val in [b"8".as_slice(), b"16", b"32", b"64", b"-32", b"-64"] {
            if let Some(tail) = after_spaces.strip_prefix(val)
                && matches!(tail.first(), Some(b' ' | b'/'))
            {
                return true;
            }
        }
        false
    }

    require!(
        prefix.len() >= BLOCK_LEN,
        be16(prefix, SECOND_CARD_PAD)? == 0x2020,
        match_simple(prefix),
        match_bitpix(&prefix[CARD_LEN..]) || match_bitpix(&prefix[2 * CARD_LEN..]),
    );
    Some(Label::Fits)
}

/// Adobe Flash Video (FLV v1): `"FLV\x01"`, valid audio/video flags (`0x00 | 0x01 | 0x04 | 0x05`),
/// data offset 9, and initial `PreviousTagSize0` = 0.
fn flv(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"FLV\x01"; // @0..4: FLV version 1
    const FLAGS: usize = 4; // @4: u8 — audio (0x04) | video (0x01) flags
    const DATA_OFFSET: usize = 5; // @5: u32 BE — header length (9)
    const PREV_TAG_0: usize = 9; // @9: u32 BE — initial previous-tag size (0)
    const MIN_LEN: usize = 13;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, FLAGS)?, 0x00 | 0x01 | 0x04 | 0x05),
        be32(prefix, DATA_OFFSET)? == 9,
        be32(prefix, PREV_TAG_0)? == 0,
    );
    Some(Label::Flv)
}

/// OGC GeoPackage: SQLite 3 database header (`"SQLite format 3\0"`) whose `application_id` at
/// offset 68 is `"GPKG"`, `"GP10"`, or `"GP11"`.
fn geopackage(prefix: &[u8]) -> Option<Label> {
    const SQLITE3: &[u8] = b"SQLite format 3\0"; // @0..16: SQLite 3 header magic
    const APPLICATION_ID: usize = 68; // @68: u32 BE — SQLite application_id field
    const MIN_LEN: usize = 100; // complete 100-byte SQLite database header

    require!(prefix.len() >= MIN_LEN, starts(prefix, SQLITE3));
    let id = be32(prefix, APPLICATION_ID)?;
    require!(id == tag_u32(b"GPKG") || id == tag_u32(b"GP10") || id == tag_u32(b"GP11"));
    Some(Label::Geopackage)
}

/// GGML Universal File (GGUF): `"GGUF"` magic with version 1 (≥ 16 bytes) or version 2/3 (≥ 24
/// bytes) in either endianness.
fn gguf(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"GGUF"; // @0..4: GGUF magic
    const VERSION: usize = 4; // @4: u32 (LE or BE) — format version (1, 2, or 3)
    const V1_MIN_LEN: usize = 16; // magic (4) + version (4) + 2 × u32 counts (8)
    const V2_MIN_LEN: usize = 24; // magic (4) + version (4) + 2 × u64 counts (16)

    require!(prefix.len() >= V1_MIN_LEN, starts(prefix, MAGIC));
    let (v_le, v_be) = (le32(prefix, VERSION)?, be32(prefix, VERSION)?);
    let valid = v_le == 1
        || v_be == 1
        || (prefix.len() >= V2_MIN_LEN && (matches!(v_le, 2 | 3) || matches!(v_be, 2 | 3)));
    require!(valid);
    Some(Label::Gguf)
}

/// Khronos binary glTF 2.0 (GLB): `"glTF"` + version 2, 4-byte-aligned total and chunk lengths,
/// `"JSON"` first chunk type at `@16`, and JSON object opening `{` after optional whitespace at `@20`.
fn glb(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"glTF", &2u32.to_le_bytes()); // @0..8: glTF magic + version 2
    const TOTAL_LEN: usize = 8; // @8: u32 LE — total byte length (≥ 24, % 4 == 0)
    const CHUNK_LEN: usize = 12; // @12: u32 LE — JSON chunk length (≥ 4, % 4 == 0)
    const CHUNK_TYPE: usize = 16; // @16..20: "JSON" chunk type
    const JSON_DATA: usize = 20; // @20: start of JSON payload
    const MIN_LEN: usize = 24;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, HEADER),
        le32(prefix, TOTAL_LEN)? >= MIN_LEN as u32,
        le32(prefix, TOTAL_LEN)? % 4 == 0,
        le32(prefix, CHUNK_LEN)? >= 4,
        le32(prefix, CHUNK_LEN)? % 4 == 0,
        eq_at(prefix, CHUNK_TYPE, b"JSON"),
    );
    let ws = prefix[JSON_DATA..]
        .iter()
        .take(65)
        .take_while(|&&b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        .count();
    require!(ws <= 64, u8_at(prefix, JSON_DATA + ws)? == b'{');
    Some(Label::Gltf)
}

/// WMO GRIB meteorological data: `"GRIB"` indicator magic and edition 1 or 2 at offset 7.
fn grib(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"GRIB"; // @0..4: GRIB indicator section magic
    const EDITION: usize = 7; // @7: u8 — GRIB edition number (1 or 2)
    const MIN_LEN: usize = 16;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, EDITION)?, 1 | 2),
    );
    Some(Label::Grib)
}

/// NCSA Hierarchical Data Format v4 (HDF4): `0E 03 13 01` magic and positive signed 16-bit DD
/// count (`1..=32767`) at offset 4.
fn hdf4(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = &[0x0E, 0x03, 0x13, 0x01]; // @0..4: HDF4 magic (^N^C^S^A)
    const DD_COUNT: usize = 4; // @4: u16 BE — number of data descriptors in block (1..=32767)
    const MIN_LEN: usize = 22;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        (1..=32767).contains(&be16(prefix, DD_COUNT)?),
    );
    Some(Label::Hdf4)
}

/// Windows NT registry hive (`regf`): `"regf"` base block magic, major version 1, minor version
/// `0..=6`, primary file type `0`, and direct memory load format `1`.
fn hve(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"regf"; // @0..4: registry hive base block signature
    const MAJOR: usize = 20; // @20: u32 LE — major version (1)
    const MINOR: usize = 24; // @24: u32 LE — minor version (0..=6)
    const FILE_TYPE: usize = 28; // @28: u32 LE — file type (0 = primary)
    const FORMAT: usize = 32; // @32: u32 LE — format (1 = direct memory load)
    const MIN_LEN: usize = 48;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        le32(prefix, MAJOR)? == 1,
        le32(prefix, MINOR)? <= 6,
        le32(prefix, FILE_TYPE)? == 0,
        le32(prefix, FORMAT)? == 1,
    );
    Some(Label::Hve)
}

/// Apple Icon Image (`icns`): `"icns"` magic, container length (8 for an empty container, or
/// ≥ 16 with a first element of length ≥ 8 at offset 12).
fn icns(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"icns"; // @0..4: Apple icon container magic
    const FILE_LEN: usize = 4; // @4: u32 BE — total file length
    const FIRST_ELEM_LEN: usize = 12; // @12: u32 BE — first icon element length
    const EMPTY_LEN: usize = 8;
    const ELEM_MIN_LEN: usize = 16;

    require!(prefix.len() >= EMPTY_LEN, starts(prefix, MAGIC));
    let total = be32(prefix, FILE_LEN)?;
    let valid = total == EMPTY_LEN as u32
        || (prefix.len() >= ELEM_MIN_LEN
            && total >= ELEM_MIN_LEN as u32
            && be32(prefix, FIRST_ELEM_LEN)? >= EMPTY_LEN as u32);
    require!(valid);
    Some(Label::Icns)
}

/// JPEG Network Graphics (JNG): 8-byte JNG signature followed by a 16-byte `JHDR` chunk header.
fn jng(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"\x8bJNG\r\n\x1a\n", &16u32.to_be_bytes(), b"JHDR"); // @0..16: JNG sig + JHDR chunk

    require!(starts(prefix, HEADER));
    Some(Label::Jng)
}

/// LLVM bitcode: 4-byte-aligned file size and either a raw bitcode stream (`"BC\xc0\xde"` with
/// non-zero initial 2-bit abbrev ID) or a Darwin/Clang bitcode wrapper (`0x0B17C0DE` LE).
fn llvm_bitcode(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    const RAW_MAGIC: &[u8] = b"BC\xc0\xde"; // @0..4: raw LLVM bitcode stream magic
    const WRAPPER_MAGIC: &[u8] = &0x0B17_C0DEu32.to_le_bytes(); // @0..4: LLVM bitcode wrapper magic
    const FIRST_BYTE: usize = 4; // @4: low 2 bits are initial abbrev ID (must not be END_BLOCK = 0)
    const WRAP_OFFSET: usize = 8; // @8: u32 LE — bitcode offset (≥ 16)
    const WRAP_SIZE: usize = 12; // @12: u32 LE — bitcode size (≥ 4)

    require!(ctx.size.is_multiple_of(4));
    let raw =
        prefix.len() >= 8 && starts(prefix, RAW_MAGIC) && (u8_at(prefix, FIRST_BYTE)? & 0b11) != 0;
    let wrapper = prefix.len() >= 20
        && starts(prefix, WRAPPER_MAGIC)
        && le32(prefix, WRAP_OFFSET)? >= 16
        && le32(prefix, WRAP_SIZE)? >= 4;
    require!(raw || wrapper);
    Some(Label::LlvmBitcode)
}

/// Windows Shell Link (`.lnk`): 76-byte `ShellLinkHeader` opening with size `0x4C` (76) and
/// link CLSID `00021401-0000-0000-C000-000000000046`.
fn lnk(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] =
        bytes!(&76u32.to_le_bytes(), guid!("00021401-0000-0000-C000-000000000046"),); // @0..20: HeaderSize (76) + LinkCLSID
    const MIN_LEN: usize = 76;

    require!(prefix.len() >= MIN_LEN, starts(prefix, HEADER));
    Some(Label::Lnk)
}

/// Long Range ZIP (lrzip): `"LRZI"` magic, major version 0, minor version ≥ 1, and 24-byte header.
fn lrz(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"LRZI"; // @0..4: lrzip signature
    const MAJOR: usize = 4; // @4: u8 — major version (0)
    const MINOR: usize = 5; // @5: u8 — minor version (≥ 1)
    const MIN_LEN: usize = 24;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        u8_at(prefix, MAJOR)? == 0,
        u8_at(prefix, MINOR)? >= 1,
    );
    Some(Label::Lrz)
}

/// Lua (v2.4..v5.5) and LuaJIT (v1/v2) compiled bytecode chunks.
fn luabytecode(prefix: &[u8]) -> Option<Label> {
    const LUA_MAGIC: &[u8] = b"\x1bLua"; // @0..4: standard Lua bytecode signature
    const LUAJIT_MAGIC: &[u8] = b"\x1bLJ"; // @0..3: LuaJIT bytecode dump signature
    const LUAC_DATA: &[u8] = b"\x19\x93\r\n\x1a\n"; // Lua 5.2+ conversion-check tail
    const MIN_LEN: usize = 8;

    require!(prefix.len() >= MIN_LEN);
    if starts(prefix, LUA_MAGIC) {
        let ver = u8_at(prefix, 4)?;
        let ok = match ver {
            0x23 => prefix.len() >= 11 && matches!(be16(prefix, 5)?, 0x1234 | 0x3412),
            0x25 => {
                prefix.len() >= 14
                    && u8_at(prefix, 5)? == 2
                    && u8_at(prefix, 6)? == 4
                    && u8_at(prefix, 7)? >= 1
                    && matches!(be16(prefix, 8)?, 0x1234 | 0x3412)
            }
            0x31 => {
                matches!(u8_at(prefix, 5)?, 0x6C | 0x66 | 0x64 | 0x3F) && u8_at(prefix, 6)? >= 1
            }
            0x32 => true,
            0x40 => {
                prefix.len() >= 14
                    && u8_at(prefix, 5)? <= 1
                    && prefix[6..13].iter().all(|&b| b >= 1)
            }
            0x50 => {
                prefix.len() >= 15
                    && u8_at(prefix, 5)? <= 1
                    && prefix[6..14].iter().all(|&b| b >= 1)
            }
            0x51 => {
                prefix.len() >= 12
                    && u8_at(prefix, 5)? == 0
                    && u8_at(prefix, 6)? <= 1
                    && prefix[7..11].iter().all(|&b| b >= 1)
                    && matches!(
                        u8_at(prefix, 11)?,
                        0x00 | 0x01 | 0x02 | 0x04 | 0x08 | 0x82 | 0x84 | 0x88
                    )
            }
            0x52 => {
                prefix.len() >= 18
                    && u8_at(prefix, 5)? == 0
                    && u8_at(prefix, 6)? <= 1
                    && prefix[7..11].iter().all(|&b| b >= 1)
                    && matches!(
                        u8_at(prefix, 11)?,
                        0x00 | 0x01 | 0x02 | 0x04 | 0x08 | 0x82 | 0x84 | 0x88
                    )
                    && eq_at(prefix, 12, LUAC_DATA)
            }
            0x53 => {
                prefix.len() >= 17
                    && u8_at(prefix, 5)? == 0
                    && eq_at(prefix, 6, LUAC_DATA)
                    && prefix[12..17].iter().all(|&b| b >= 1)
            }
            0x54 => {
                prefix.len() >= 15
                    && u8_at(prefix, 5)? == 0
                    && eq_at(prefix, 6, LUAC_DATA)
                    && prefix[12..15].iter().all(|&b| b >= 1)
            }
            0x55 => {
                prefix.len() >= 13
                    && u8_at(prefix, 5)? == 0
                    && eq_at(prefix, 6, LUAC_DATA)
                    && u8_at(prefix, 12)? >= 1
            }
            _ => false,
        };
        require!(ok);
        return Some(Label::Luabytecode);
    }

    require!(starts(prefix, LUAJIT_MAGIC), matches!(u8_at(prefix, 3)?, 1 | 2));
    let flags = u8_at(prefix, 4)?;
    require!(flags <= 0x0F);
    let b5 = u8_at(prefix, 5)?;
    if (flags & 0x02) != 0 {
        // Stripped LuaJIT dump: ULEB128 proto length (1..=3 bytes) + proto flags/params/framesize
        let after = if (0x08..=0x7F).contains(&b5) {
            6
        } else if b5 >= 0x80 {
            let b6 = u8_at(prefix, 6)?;
            if (0x01..=0x7F).contains(&b6) {
                7
            } else if b6 >= 0x80 && (0x01..=0x7F).contains(&u8_at(prefix, 7)?) {
                8
            } else {
                return None;
            }
        } else {
            return None;
        };
        require!(
            u8_at(prefix, after)? <= 0x1F,
            u8_at(prefix, after + 1).is_some(),
            matches!(u8_at(prefix, after + 2)?, 0x01..=0xFA),
        );
    } else {
        // Named LuaJIT dump: ULEB128 name length (1..=2 bytes) + '@' or '=' + printable ASCII
        let after = if (0x02..=0x7F).contains(&b5) {
            6
        } else if b5 >= 0x80 && (0x01..=0x7F).contains(&u8_at(prefix, 6)?) {
            7
        } else {
            return None;
        };
        require!(
            matches!(u8_at(prefix, after)?, b'@' | b'='),
            matches!(u8_at(prefix, after + 1)?, 0x20..=0x7E),
        );
    }
    Some(Label::Luabytecode)
}

/// Lzip compressed file: `"LZIP"` magic, version 0 (≥ 18 bytes total) or 1 (≥ 26 bytes total),
/// and dictionary size exponent `12..=29` in the low 5 bits of byte 5.
fn lzip(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    const MAGIC: &[u8] = b"LZIP"; // @0..4: lzip magic
    const VERSION: usize = 4; // @4: u8 — version 0 or 1
    const DICT_SIZE: usize = 5; // @5: u8 — low 5 bits are log2(dict_size) in 12..=29
    const MIN_LEN: usize = 8;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        (12..=29).contains(&(u8_at(prefix, DICT_SIZE)? & 0x1F)),
    );
    let valid_ver = match u8_at(prefix, VERSION)? {
        0 => ctx.size >= 18,
        1 => ctx.size >= 26,
        _ => false,
    };
    require!(valid_ver);
    Some(Label::Lz)
}

/// LZ4 frame: legacy (`0x184C2102` / `0x184C2103` LE) or v1 (`0x184D2204` LE) with valid FLG/BD
/// descriptors and minimum frame size for the declared optional header fields.
fn lz4(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    const LEGACY_V02: &[u8] = &0x184C_2102u32.to_le_bytes(); // @0..4: legacy LZ4 magic (02 21 4C 18)
    const LEGACY_V03: &[u8] = &0x184C_2103u32.to_le_bytes(); // @0..4: LZ4 v1.3 magic (03 21 4C 18)
    const MODERN_MAGIC: &[u8] = &0x184D_2204u32.to_le_bytes(); // @0..4: LZ4 v1.4+ frame magic (04 22 4D 18)
    const FLG: usize = 4; // @4: u8 — frame descriptor flags
    const BD: usize = 5; // @5: u8 — block descriptor (0x40 | 0x50 | 0x60 | 0x70)
    const MIN_LEN: usize = 8;

    require!(prefix.len() >= MIN_LEN);
    if starts(prefix, LEGACY_V02) || starts(prefix, LEGACY_V03) {
        return Some(Label::Lz4);
    }
    require!(starts(prefix, MODERN_MAGIC));
    let (flg, bd) = (u8_at(prefix, FLG)?, u8_at(prefix, BD)?);
    // Version bits 7..6 must be 01 (0x40), reserved bit 1 must be 0, block max size in 4..=7.
    require!((flg & 0b1100_0010) == 0x40, matches!(bd, 0x40 | 0x50 | 0x60 | 0x70));
    let (min_prefix, min_size): (usize, u64) = match flg & 0b0000_1001 {
        0x00 => (7, 11),  // neither content size nor dict ID
        0x01 => (11, 15), // 4-byte dict ID
        0x08 => (15, 19), // 8-byte content size
        0x09 => (19, 23), // 8-byte content size + 4-byte dict ID
        _ => return None,
    };
    require!(prefix.len() >= min_prefix, ctx.size >= min_size);
    Some(Label::Lz4)
}

/// MATLAB v5 MAT-file: `"MATLAB 5"` text prefix and 128-byte header with version `0x0100` and
/// endianness indicator `"MI"` (little-endian) or `"IM"` (big-endian) at offset 124.
fn mat(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"MATLAB 5"; // @0..8: descriptive text prefix
    const VERSION_ENDIAN: usize = 124; // @124..128: version 0x0100 + endian tag
    const BE_TAG: &[u8] = bytes!(&0x0100u16.to_be_bytes(), b"MI"); // big-endian v5 indicator
    const LE_TAG: &[u8] = bytes!(&0x0100u16.to_le_bytes(), b"IM"); // little-endian v5 indicator
    const MIN_LEN: usize = 128;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        eq_at(prefix, VERSION_ENDIAN, BE_TAG) || eq_at(prefix, VERSION_ENDIAN, LE_TAG),
    );
    Some(Label::Mat)
}

/// Standard MIDI File (SMF): `"MThd"` chunk of length 6, valid format (`0` with 1 track, or
/// `1..=2` with ≥ 1 track), valid time division, and first `"MTrk"` track chunk of length ≥ 4.
fn midi(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"MThd", &6u32.to_be_bytes()); // @0..8: MThd tag + 6-byte header length
    const FORMAT: usize = 8; // @8: u16 BE — MIDI format (0, 1, or 2)
    const TRACKS: usize = 10; // @10: u16 BE — number of tracks
    const DIVISION: usize = 12; // @12: u16 BE — ticks per quarter-note or SMPTE timecode
    const MTRK_TAG: usize = 14; // @14..18: first track "MTrk" chunk magic
    const MTRK_LEN: usize = 18; // @18: u32 BE — first track length (≥ 4)
    const MIN_LEN: usize = 26;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, HEADER),
        eq_at(prefix, MTRK_TAG, b"MTrk"),
        be32(prefix, MTRK_LEN)? >= 4,
    );
    let (fmt, trk, div) = (be16(prefix, FORMAT)?, be16(prefix, TRACKS)?, be16(prefix, DIVISION)?);
    let valid_fmt = (fmt == 0 && trk == 1) || ((1..=2).contains(&fmt) && trk >= 1);
    let valid_div = (1..=32767).contains(&div)
        || (matches!(u8_at(prefix, 12)?, 226 | 227 | 231 | 232) && u8_at(prefix, 13)? >= 1);
    require!(valid_fmt, valid_div);
    Some(Label::Midi)
}

/// Windows Minidump (`MINIDUMP_HEADER`): `"MDMP"` + version `0xA793`, stream count `1..=255` at
/// `@8`, and stream directory RVA `32` immediately following the 32-byte header at `@12`.
fn minidump(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"MDMP", &0xA793u16.to_le_bytes()); // @0..6: MDMP signature + 0xA793 version
    const STREAM_COUNT: usize = 8; // @8: u32 LE — NumberOfStreams (1..=255)
    const STREAM_DIR_RVA: usize = 12; // @12: u32 LE — StreamDirectoryRva (32)
    const HEADER_LEN: usize = 32;

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, HEADER),
        (1..=255).contains(&le32(prefix, STREAM_COUNT)?),
        le32(prefix, STREAM_DIR_RVA)? == HEADER_LEN as u32,
    );
    Some(Label::Minidump)
}

/// NumPy serialized array (`.npy`): `"\x93NUMPY"` magic, version 1.0 (16-bit header length at `@8`)
/// or 2.0/3.0 (32-bit header length at `@8`), 16-byte total header alignment, and Python dict `{`.
fn npy(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"\x93NUMPY"; // @0..6: NumPy format signature
    const MAJOR: usize = 6; // @6: u8 — major version (1, 2, or 3)
    const MINOR: usize = 7; // @7: u8 — minor version (0)
    const HDR_LEN: usize = 8; // @8: u16 LE (v1) or u32 LE (v2/v3) — dictionary header length
    const MIN_LEN: usize = 64;

    fn match_dict(s: &[u8]) -> bool {
        if !starts(s, b"{") {
            return false;
        }
        let ws = s[1..].iter().take(17).take_while(|&&b| matches!(b, b' ' | b'\t')).count();
        ws <= 16 && matches!(s.get(1 + ws), Some(b'\'' | b'"'))
    }

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC), u8_at(prefix, MINOR)? == 0);
    let ok = match u8_at(prefix, MAJOR)? {
        1 => {
            let len = le16(prefix, HDR_LEN)?;
            len >= 54 && len % 16 == 6 && match_dict(&prefix[10..])
        }
        2 | 3 => {
            let len = le32(prefix, HDR_LEN)?;
            len >= 52 && len % 16 == 4 && match_dict(&prefix[12..])
        }
        _ => false,
    };
    require!(ok);
    Some(Label::Npy)
}

/// Teem Nearly Raw Raster Data (NRRD): `"NRRD000"` + version `'1'`..=`'5'` followed by LF or CRLF.
fn nrrd(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"NRRD000"; // @0..7: NRRD version prefix
    const VERSION: usize = 7; // @7: ASCII digit '1'..='5'

    require!(starts(prefix, MAGIC), matches!(u8_at(prefix, VERSION)?, b'1'..=b'5'));
    let eol =
        u8_at(prefix, 8)? == b'\n' || (u8_at(prefix, 8)? == b'\r' && u8_at(prefix, 9)? == b'\n');
    require!(eol);
    Some(Label::Nrrd)
}

/// Microsoft OneNote revision store file (`.one`): 16-byte file-type GUID at offset 0.
fn one(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = guid!("7B5C52E4-D88C-4DA7-AEB1-5378D02996D3"); // @0..16: OneNote header GUID

    require!(starts(prefix, MAGIC));
    Some(Label::One)
}

/// Apache Parquet column store: `"PAR1"` leading magic and ≥ 12 bytes (header + 8-byte footer).
fn parquet(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"PAR1"; // @0..4: Parquet magic
    const MIN_LEN: usize = 12;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    Some(Label::Parquet)
}

/// OpenPGP ASCII armor (`"-----BEGIN PGP "`) or RFC 7468 PEM certificate/key encapsulation boundary.
fn pem_or_pgp(prefix: &[u8]) -> Option<Label> {
    const BEGIN: &[u8] = b"-----BEGIN "; // @0..11: encapsulation boundary opening
    const PGP_ARMOR: &[u8] = b"-----BEGIN PGP "; // @0..15: OpenPGP armor header
    const PEM_LABELS: &[&[u8]] = &[
        b"CERTIFICATE",
        b"TRUSTED CERTIFICATE",
        b"CERTIFICATE REQUEST",
        b"NEW CERTIFICATE REQUEST",
        b"X509 CRL",
        b"PUBLIC KEY",
        b"PRIVATE KEY",
        b"ENCRYPTED PRIVATE KEY",
        b"RSA PRIVATE KEY",
        b"RSA PUBLIC KEY",
        b"DSA PRIVATE KEY",
        b"DSA PARAMETERS",
        b"EC PRIVATE KEY",
        b"EC PARAMETERS",
        b"DH PARAMETERS",
        b"PKCS7",
        b"CMS",
    ];

    if starts(prefix, PGP_ARMOR) {
        return Some(Label::Pgp);
    }
    let rest = prefix.strip_prefix(BEGIN)?;
    for &label in PEM_LABELS {
        if let Some(after_label) = rest.strip_prefix(label)
            && let Some(after_dashes) = after_label.strip_prefix(b"-----")
            && (starts(after_dashes, b"\n") || starts(after_dashes, b"\r\n"))
        {
            return Some(Label::Pem);
        }
    }
    None
}

/// Polygon File Format (PLY): `"ply"` line followed by `"format <encoding> 1.0"` and whitespace.
fn ply(prefix: &[u8]) -> Option<Label> {
    const ENCODINGS: &[&[u8]] =
        &[b"ascii 1.0", b"binary_little_endian 1.0", b"binary_big_endian 1.0"];

    let rest =
        prefix.strip_prefix(b"ply\nformat ").or_else(|| prefix.strip_prefix(b"ply\r\nformat "))?;
    for &enc in ENCODINGS {
        if let Some(tail) = rest.strip_prefix(enc)
            && matches!(tail.first(), Some(b' ' | b'\t' | b'\r' | b'\n'))
        {
            return Some(Label::Ply);
        }
    }
    None
}

/// PostgreSQL custom-format dump (`pg_dump -Fc`): `"PGDMP"` magic, major version 1, integer/offset
/// sizes in `1..=32`, and compression/format code `1 | 3 | 5`.
fn postgres_dump(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"PGDMP\x01"; // @0..6: PGDMP signature + vmajor = 1
    const VMINOR: usize = 6; // @6: u8 — minor archive version
    const MIN_LEN: usize = 9;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    let minor = u8_at(prefix, VMINOR)?;
    let valid = if minor == 0 {
        (1..=32).contains(&u8_at(prefix, 7)?) && matches!(u8_at(prefix, 8)?, 1 | 3 | 5)
    } else if minor <= 6 {
        prefix.len() >= 10
            && (1..=32).contains(&u8_at(prefix, 8)?)
            && matches!(u8_at(prefix, 9)?, 1 | 3 | 5)
    } else {
        prefix.len() >= 11
            && (1..=32).contains(&u8_at(prefix, 8)?)
            && (1..=32).contains(&u8_at(prefix, 9)?)
            && matches!(u8_at(prefix, 10)?, 1 | 3 | 5)
    };
    require!(valid);
    Some(Label::PostgresDump)
}

/// Adobe Photoshop PSD (v1) / PSB (v2): `"8BPS"` signature, version 1 or 2, 6 reserved zero bytes,
/// `1..=56` channels, bounded dimensions, bit depth `1 | 8 | 16 | 32`, and color mode `0..=9`.
fn psd(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"8BPS"; // @0..4: Photoshop signature
    const VERSION: usize = 4; // @4: u16 BE — 1 = PSD, 2 = PSB
    const RESERVED: usize = 6; // @6..12: 6 reserved zero bytes
    const CHANNELS: usize = 12; // @12: u16 BE — channel count (1..=56)
    const HEIGHT: usize = 14; // @14: u32 BE — height in pixels
    const WIDTH: usize = 18; // @18: u32 BE — width in pixels
    const DEPTH: usize = 22; // @22: u16 BE — bits per channel (1, 8, 16, 32)
    const COLOR_MODE: usize = 24; // @24: u16 BE — color mode (0..=9)
    const HEADER_LEN: usize = 26;

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, MAGIC),
        eq_at(prefix, RESERVED, &[0; 6]),
        (1..=56).contains(&be16(prefix, CHANNELS)?),
        matches!(be16(prefix, DEPTH)?, 1 | 8 | 16 | 32),
        be16(prefix, COLOR_MODE)? <= 9,
    );
    let (ver, h, w) = (be16(prefix, VERSION)?, be32(prefix, HEIGHT)?, be32(prefix, WIDTH)?);
    let max_dim = match ver {
        1 => 30_000,
        2 => 300_000,
        _ => return None,
    };
    require!((1..=max_dim).contains(&h), (1..=max_dim).contains(&w));
    Some(Label::Psd)
}

/// Quite OK Image (QOI): `"qoif"` magic, nonzero width and height, 3 or 4 channels, and colorspace
/// `0` (sRGB) or `1` (linear).
fn qoi(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"qoif"; // @0..4: QOI header magic
    const WIDTH: usize = 4; // @4: u32 BE — width (> 0)
    const HEIGHT: usize = 8; // @8: u32 BE — height (> 0)
    const CHANNELS: usize = 12; // @12: u8 — 3 (RGB) or 4 (RGBA)
    const COLORSPACE: usize = 13; // @13: u8 — 0 or 1
    const MIN_LEN: usize = 23; // 14-byte header + ≥1 chunk byte + 8-byte end marker

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        be32(prefix, WIDTH)? > 0,
        be32(prefix, HEIGHT)? > 0,
        matches!(u8_at(prefix, CHANNELS)?, 3 | 4),
        u8_at(prefix, COLORSPACE)? <= 1,
    );
    Some(Label::Qoi)
}

/// RAR archive: RAR 1.5..4.x (`"Rar!\x1a\x07\x00"`), RAR 5.0 (`"Rar!\x1a\x07\x01\x00"`), or
/// pre-1.5 legacy (`"RE~^"`).
fn rar(prefix: &[u8]) -> Option<Label> {
    const RAR4: &[u8] = b"Rar!\x1a\x07\x00"; // @0..7: RAR v1.5–4.x signature
    const RAR5: &[u8] = b"Rar!\x1a\x07\x01\x00"; // @0..8: RAR v5.0 signature
    const LEGACY: &[u8] = b"RE~^"; // @0..4: RAR pre-1.5 signature
    const MIN_LEN: usize = 8;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, RAR4) || starts(prefix, RAR5) || starts(prefix, LEGACY),
    );
    Some(Label::Rar)
}

/// Redis RDB snapshot: `"REDIS"` followed by a nonzero 4-digit ASCII version (`"0001"`..`"9999"`).
fn redis_rdb(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"REDIS"; // @0..5: Redis RDB signature
    const VERSION: usize = 5; // @5..9: 4-digit ASCII version
    const MIN_LEN: usize = 9;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        prefix[VERSION..MIN_LEN].iter().all(u8::is_ascii_digit),
        be32(prefix, VERSION)? != tag_u32(b"0000"),
    );
    Some(Label::RedisRdb)
}

/// rzip compressed file: `"RZIP"` magic, major version 2, minor version `0..=1`, two 32-bit size
/// fields, and 10 reserved zero bytes at `@14..24`.
fn rzip(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"RZIP\x02"; // @0..5: RZIP magic + major version 2
    const MINOR: usize = 5; // @5: u8 — minor version (0 or 1)
    const RESERVED: usize = 14; // @14..24: 10 reserved zero bytes
    const HEADER_LEN: usize = 24;

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, MAGIC),
        u8_at(prefix, MINOR)? <= 1,
        eq_at(prefix, RESERVED, &[0; 10]),
    );
    Some(Label::Rzip)
}

/// 7-Zip archive: 6-byte signature `37 7A BC AF 27 1C`, major version 0, and 32-byte signature header.
fn sevenzip(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = b"7z\xbc\xaf\x27\x1c\x00"; // @0..7: 7z signature + major version 0
    const HEADER_LEN: usize = 32;

    require!(prefix.len() >= HEADER_LEN, starts(prefix, HEADER));
    Some(Label::Sevenzip)
}

/// ESRI Shapefile (`.shp` main or `.shx` index): file code `9994` BE at `@0`, 20 unused zero bytes
/// at `@4..24`, file length ≥ 54 words at `@24`, version `1000` LE at `@28`, valid shape type at
/// `@32`, and first record number `1` (`.shp`) or offset `50` words (`.shx`) BE at `@100`.
fn shapefile(prefix: &[u8]) -> Option<Label> {
    const FILE_CODE: u32 = 9994; // @0: u32 BE — ESRI shapefile code (0x0000270A)
    const UNUSED: usize = 4; // @4..24: 5 × u32 unused zero words
    const FILE_LEN_WORDS: usize = 24; // @24: u32 BE — file length in 16-bit words (≥ 54)
    const VERSION: usize = 28; // @28: u32 LE — shapefile version (1000)
    const SHAPE_TYPE: usize = 32; // @32: u32 LE — shape type code
    const FIRST_RECORD: usize = 100; // @100: u32 BE — record 1 (.shp) or 50-word header offset (.shx)
    const MIN_LEN: usize = 108;

    require!(
        prefix.len() >= MIN_LEN,
        be32(prefix, 0)? == FILE_CODE,
        eq_at(prefix, UNUSED, &[0; 20]),
        be32(prefix, FILE_LEN_WORDS)? >= 54,
        le32(prefix, VERSION)? == 1000,
        matches!(
            le32(prefix, SHAPE_TYPE)?,
            0 | 1 | 3 | 5 | 8 | 11 | 13 | 15 | 18 | 21 | 23 | 25 | 28 | 31
        ),
        matches!(be32(prefix, FIRST_RECORD)?, 1 | 50),
    );
    Some(Label::Shapefile)
}

/// SketchUp model (`.skp`): UTF-16LE BOM + `"SketchUp Model"` (32 bytes) or legacy length-prefixed
/// ASCII `"\x0eSketchUp Model\x08"` (16 bytes).
fn sketchup(prefix: &[u8]) -> Option<Label> {
    const LEGACY: &[u8] = b"\x0eSketchUp Model\x08"; // @0..16: legacy ASCII SketchUp header
    const UNICODE: &[u8] = &[
        0xFF, 0xFE, 0xFF, 0x0E, b'S', 0, b'k', 0, b'e', 0, b't', 0, b'c', 0, b'h', 0, b'U', 0,
        b'p', 0, b' ', 0, b'M', 0, b'o', 0, b'd', 0, b'e', 0, b'l', 0,
    ]; // @0..32: UTF-16LE SketchUp header

    let matched = starts(prefix, LEGACY) || starts(prefix, UNICODE);
    require!(matched);
    Some(Label::Sketchup)
}

/// Khronos SPIR-V binary module: 4-byte-aligned size, 5-word header with magic `0x07230203` in
/// either endianness, version `1.0`..=`1.6` (`0x00010000..=0x00010600` with low byte 0), nonzero
/// ID bound at `@12`, and reserved schema `0` at `@16`.
fn spirv(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    const MAGIC: u32 = 0x0723_0203; // @0: u32 — SPIR-V magic number
    const VERSION: usize = 4; // @4: u32 — version word (0x00010000..=0x00010600, step 0x100)
    const BOUND: usize = 12; // @12: u32 — ID bound (> 0)
    const SCHEMA: usize = 16; // @16: u32 — instruction schema (0)
    const HEADER_LEN: usize = 20;

    require!(prefix.len() >= HEADER_LEN, ctx.size.is_multiple_of(4));
    let read_u32: ReadU32 = if le32(prefix, 0)? == MAGIC {
        le32
    } else if be32(prefix, 0)? == MAGIC {
        be32
    } else {
        return None;
    };
    let ver = read_u32(prefix, VERSION)?;
    require!(
        (0x0001_0000..=0x0001_0600).contains(&ver),
        ver & 0xFF == 0,
        read_u32(prefix, BOUND)? > 0,
        read_u32(prefix, SCHEMA)? == 0,
    );
    Some(Label::Spirv)
}

/// SPSS system file (`.sav` / `.zsav`, `"$FL2"` or `"$FL3"` with layout code 2 or 3 at `@64` and
/// compression `0..=2` at `@72`) or SPSS portable file (EBCDIC `"SPSS"` = `C9 C3 E2 C1` at `@0`,
/// ≥ 464 bytes).
fn spss(prefix: &[u8]) -> Option<Label> {
    const SAV_FL2: &[u8] = b"$FL2"; // @0..4: SPSS SAV magic
    const SAV_FL3: &[u8] = b"$FL3"; // @0..4: SPSS ZSAV magic
    const PORTABLE_EBCDIC: &[u8] = &[0xC9, 0xC3, 0xE2, 0xC1]; // @0..4: EBCDIC "SPSS" in portable header
    const LAYOUT_CODE: usize = 64; // @64: u32 (LE or BE) — layout_code (2 or 3)
    const COMPRESSION: usize = 72; // @72: u32 (LE or BE) — compression switch (0..=2)
    const SAV_MIN_LEN: usize = 176;
    const PORTABLE_MIN_LEN: usize = 464;

    if prefix.len() >= SAV_MIN_LEN && (starts(prefix, SAV_FL2) || starts(prefix, SAV_FL3)) {
        let le_ok = matches!(le32(prefix, LAYOUT_CODE)?, 2 | 3) && le32(prefix, COMPRESSION)? <= 2;
        let be_ok = matches!(be32(prefix, LAYOUT_CODE)?, 2 | 3) && be32(prefix, COMPRESSION)? <= 2;
        require!(le_ok || be_ok);
        return Some(Label::Spss);
    }
    require!(prefix.len() >= PORTABLE_MIN_LEN, starts(prefix, PORTABLE_EBCDIC));
    Some(Label::Spss)
}

/// ISO 10303-21 STEP product data exchange file: opens with `"ISO-10303-21;"`.
fn step(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"ISO-10303-21;"; // @0..13: STEP Part 21 header keyword

    require!(starts(prefix, MAGIC));
    Some(Label::Step)
}

/// Microsoft USB Flashing Format (UF2): 512-byte block with starting magics `"UF2\n"` +
/// `0x9E5D5157` LE, payload size ≤ 476 at `@16`, and final magic `0x0AB16F30` LE at `@508`.
fn uf2(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"UF2\n", &0x9E5D_5157u32.to_le_bytes()); // @0..8: magicStart0 + magicStart1
    const PAYLOAD_SIZE: usize = 16; // @16: u32 LE — payloadSize (≤ 476)
    const MAGIC_END_OFFSET: usize = 508; // @508: u32 LE — magicEnd (0x0AB16F30)
    const MAGIC_END: u32 = 0x0AB1_6F30;
    const BLOCK_LEN: usize = 512;

    require!(
        prefix.len() >= BLOCK_LEN,
        starts(prefix, HEADER),
        le32(prefix, PAYLOAD_SIZE)? <= 476,
        le32(prefix, MAGIC_END_OFFSET)? == MAGIC_END,
    );
    Some(Label::Uf2)
}

/// Microsoft Virtual Hard Disk (VHD): 512-byte dynamic/differencing header copy of footer opening
/// with `"conectix"`, features `2 | 3` at `@8`, version `0x00010000` at `@12`, disk type `2..=4`
/// at `@60`, and saved state `0..=1` at `@84`.
fn vhd(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"conectix"; // @0..8: VHD cookie
    const FEATURES: usize = 8; // @8: u32 BE — features bitmask (2 or 3)
    const FORMAT_VERSION: usize = 12; // @12: u32 BE — file format version (0x00010000)
    const DISK_TYPE: usize = 60; // @60: u32 BE — disk type (2..=4)
    const SAVED_STATE: usize = 84; // @84: u8 — saved state flag (0 or 1)
    const FOOTER_LEN: usize = 512;

    require!(
        prefix.len() >= FOOTER_LEN,
        starts(prefix, MAGIC),
        matches!(be32(prefix, FEATURES)?, 2 | 3),
        be32(prefix, FORMAT_VERSION)? == 0x0001_0000,
        (2..=4).contains(&be32(prefix, DISK_TYPE)?),
        u8_at(prefix, SAVED_STATE)? <= 1,
    );
    Some(Label::Vhd)
}

/// VMware vSphere Installation Bundle (VIB): Unix `ar` archive (`"!<arch>\n"`) whose first member
/// is `"descriptor.xml"` at `@8` holding `"<vib version"` at `@68`.
fn vib(prefix: &[u8]) -> Option<Label> {
    const AR_MAGIC: &[u8] = b"!<arch>\n"; // @0..8: Unix ar archive signature
    const MEMBER_NAME: usize = 8; // @8..22: first member name "descriptor.xml"
    const PAYLOAD_OFFSET: usize = 68; // @68..80: 8-byte ar magic + 60-byte member header
    const MIN_LEN: usize = 80;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, AR_MAGIC),
        eq_at(prefix, MEMBER_NAME, b"descriptor.xml"),
        eq_at(prefix, PAYLOAD_OFFSET, b"<vib version"),
    );
    Some(Label::Vib)
}

/// Doom WAD archive: `"IWAD"` or `"PWAD"` magic plus lump count and directory offset (≥ 12 bytes).
fn wad(prefix: &[u8]) -> Option<Label> {
    const IWAD: &[u8] = b"IWAD"; // @0..4: internal WAD signature
    const PWAD: &[u8] = b"PWAD"; // @0..4: patch WAD signature
    const HEADER_LEN: usize = 12; // 4-byte magic + numlumps (4) + infotableofs (4)

    require!(prefix.len() >= HEADER_LEN, starts(prefix, IWAD) || starts(prefix, PWAD));
    Some(Label::Wad)
}

/// WebAssembly binary module: `"\0asm"` magic followed by version 1 (`0x00000001` LE).
fn wasm(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"\0asm", &1u32.to_le_bytes()); // @0..8: WASM magic + version 1

    require!(starts(prefix, HEADER));
    Some(Label::Wasm)
}

/// W3C Web Open Font Format 1.0 (WOFF): `"wOFF"` magic, known sfnt flavor at `@4`, length ≥ 64 at
/// `@8`, `1..=4095` tables at `@12`, reserved `0` at `@14`, and totalSfntSize ≥ 28 at `@16`.
fn woff(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"wOFF"; // @0..4: WOFF 1.0 signature
    const FLAVOR: usize = 4; // @4: u32 BE — sfnt flavor
    const LENGTH: usize = 8; // @8: u32 BE — total WOFF file size (≥ 64)
    const NUM_TABLES: usize = 12; // @12: u16 BE — number of tables (1..=4095)
    const RESERVED: usize = 14; // @14: u16 BE — reserved (0)
    const TOTAL_SFNT_SIZE: usize = 16; // @16: u32 BE — uncompressed sfnt size (≥ 28)
    const MIN_LEN: usize = 64; // 44-byte WOFF header + 1 × 20-byte table directory entry

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        be32(prefix, LENGTH)? >= MIN_LEN as u32,
        (1..=4095).contains(&be16(prefix, NUM_TABLES)?),
        be16(prefix, RESERVED)? == 0,
        be32(prefix, TOTAL_SFNT_SIZE)? >= 28,
    );
    let flavor = be32(prefix, FLAVOR)?;
    require!(
        flavor == 0x0001_0000
            || flavor == tag_u32(b"OTTO")
            || flavor == tag_u32(b"true")
            || flavor == tag_u32(b"typ1")
    );
    Some(Label::Woff)
}

/// W3C Web Open Font Format 2.0 (WOFF2): `"wOF2"` magic, known sfnt/collection flavor at `@4`,
/// length ≥ 51 at `@8`, `1..=4095` tables at `@12`, reserved `0` at `@14`, totalSfntSize ≥ 28 at
/// `@16`, and nonzero totalCompressedSize at `@20`.
fn woff2(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"wOF2"; // @0..4: WOFF 2.0 signature
    const FLAVOR: usize = 4; // @4: u32 BE — sfnt flavor
    const LENGTH: usize = 8; // @8: u32 BE — total WOFF2 file size (≥ 51)
    const NUM_TABLES: usize = 12; // @12: u16 BE — number of tables (1..=4095)
    const RESERVED: usize = 14; // @14: u16 BE — reserved (0)
    const TOTAL_SFNT_SIZE: usize = 16; // @16: u32 BE — uncompressed size (≥ 28)
    const TOTAL_COMP_SIZE: usize = 20; // @20: u32 BE — compressed stream size (≥ 1)
    const MIN_LEN: usize = 51; // 48-byte WOFF2 header + ≥ 2-byte table entry + ≥ 1 payload byte

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        be32(prefix, LENGTH)? >= MIN_LEN as u32,
        (1..=4095).contains(&be16(prefix, NUM_TABLES)?),
        be16(prefix, RESERVED)? == 0,
        be32(prefix, TOTAL_SFNT_SIZE)? >= 28,
        be32(prefix, TOTAL_COMP_SIZE)? >= 1,
    );
    let flavor = be32(prefix, FLAVOR)?;
    require!(
        flavor == 0x0001_0000
            || flavor == tag_u32(b"OTTO")
            || flavor == tag_u32(b"true")
            || flavor == tag_u32(b"typ1")
            || flavor == tag_u32(b"ttcf")
    );
    Some(Label::Woff2)
}

/// eXtensible ARchive (XAR): `"xar!"` magic, 28-byte header, and nonzero 64-bit compressed and
/// uncompressed TOC lengths at `@8..16` and `@16..24`.
fn xar(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"xar!"; // @0..4: XAR header magic
    const TOC_COMP_HI: usize = 8; // @8..16: u64 BE — compressed TOC length (> 0)
    const TOC_COMP_LO: usize = 12;
    const TOC_UNCOMP_HI: usize = 16; // @16..24: u64 BE — uncompressed TOC length (> 0)
    const TOC_UNCOMP_LO: usize = 20;
    const HEADER_LEN: usize = 28;

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, MAGIC),
        be32(prefix, TOC_COMP_HI)? > 0 || be32(prefix, TOC_COMP_LO)? > 0,
        be32(prefix, TOC_UNCOMP_HI)? > 0 || be32(prefix, TOC_UNCOMP_LO)? > 0,
    );
    Some(Label::Xar)
}

/// GIMP XCF image: `"gimp xcf "` followed by `"file\0"` or `"v[0-9]{3}\0"`, width/height words,
/// and base image type `0..=2` (RGB, grayscale, indexed) at `@22`.
fn xcf(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"gimp xcf "; // @0..9: XCF magic prefix
    const VERSION: usize = 9; // @9..14: "file\0" or "vNNN\0"
    const IMAGE_TYPE: usize = 22; // @22: u32 BE — base image type (0..=2)
    const MIN_LEN: usize = 26;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC), be32(prefix, IMAGE_TYPE)? <= 2);
    let ver_ok = eq_at(prefix, VERSION, b"file\0")
        || (u8_at(prefix, VERSION)? == b'v'
            && prefix[10..13].iter().all(u8::is_ascii_digit)
            && u8_at(prefix, 13)? == 0);
    require!(ver_ok);
    Some(Label::Xcf)
}

/// Zstandard compressed frame: legacy magics `0xFD2FB522..=0xFD2FB527` LE (≥ 8 bytes) or modern
/// `0xFD2FB528` LE (≥ 9 bytes) with reserved bit 3 of `Frame_Header_Descriptor` clear.
fn zst(prefix: &[u8]) -> Option<Label> {
    const MAGIC_TAIL: &[u8] = &[0xB5, 0x2F, 0xFD]; // @1..4: upper 3 bytes of Zstandard magic
    const FHD: usize = 4; // @4: u8 — Frame_Header_Descriptor (bit 3 reserved = 0)
    const MIN_LEN: usize = 8;

    require!(prefix.len() >= MIN_LEN, eq_at(prefix, 1, MAGIC_TAIL));
    let valid = match u8_at(prefix, 0)? {
        0x22..=0x27 => true,
        0x28 => prefix.len() >= 9 && (u8_at(prefix, FHD)? & 0x08) == 0,
        _ => false,
    };
    require!(valid);
    Some(Label::Zst)
}

/// Apache Arrow IPC file (Feather v2): `"ARROW1\0\0"` padded magic and ≥ 12 bytes.
fn arrow(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"ARROW1\0\0"; // @0..8: 8-byte padded Arrow IPC file magic
    const MIN_LEN: usize = 12;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    Some(Label::Arrow)
}

/// IETF pcapng Section Header Block: block type `0x0A0D0D0A`, byte-order magic `0x1A2B3C4D` at `@8`
/// in either endianness with major version `1` at `@12`, and ≥ 28-byte minimum SHB.
fn pcapng(prefix: &[u8]) -> Option<Label> {
    const SHB_TYPE: &[u8] = b"\x0a\x0d\x0d\x0a"; // @0..4: Section Header Block type
    const BYTE_ORDER_MAGIC: u32 = 0x1A2B_3C4D; // @8: u32 — endianness marker
    const BYTE_ORDER_OFFSET: usize = 8;
    const MAJOR_VERSION: usize = 12; // @12: u16 — major version (1)
    const MIN_LEN: usize = 28;

    require!(prefix.len() >= MIN_LEN, starts(prefix, SHB_TYPE));
    let valid = (le32(prefix, BYTE_ORDER_OFFSET)? == BYTE_ORDER_MAGIC
        && le16(prefix, MAJOR_VERSION)? == 1)
        || (be32(prefix, BYTE_ORDER_OFFSET)? == BYTE_ORDER_MAGIC
            && be16(prefix, MAJOR_VERSION)? == 1);
    require!(valid);
    Some(Label::Pcapng)
}

/// Microsoft Cabinet archive (`CFHEADER`): `"MSCF\0\0\0\0"`, `cbCabinet` ≥ 36 at `@8`, reserved2
/// `0` at `@12`, `coffFiles` ≥ 36 at `@16`, reserved3 `0` at `@20`, version 1.3 (`0x0103`) at
/// `@24`, `cFolders` ≥ 1 at `@26`, and `flags` ≤ 7 at `@30`.
fn cab(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"MSCF", &0u32.to_le_bytes()); // @0..8: MSCF signature + reserved1 = 0
    const CB_CABINET: usize = 8; // @8: u32 LE — archive size in bytes (≥ 36)
    const RESERVED2: usize = 12; // @12: u32 LE — reserved2 (0)
    const COFF_FILES: usize = 16; // @16: u32 LE — offset of first CFFILE entry (≥ 36)
    const RESERVED3: usize = 20; // @20: u32 LE — reserved3 (0)
    const VERSION: usize = 24; // @24: u16 LE — versionMinor=3, versionMajor=1 (0x0103 = 259)
    const C_FOLDERS: usize = 26; // @26: u16 LE — number of CFFOLDER entries (≥ 1)
    const FLAGS: usize = 30; // @30: u16 LE — option flags (≤ 7)
    const MIN_LEN: usize = 36;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, HEADER),
        le32(prefix, CB_CABINET)? >= MIN_LEN as u32,
        le32(prefix, RESERVED2)? == 0,
        le32(prefix, COFF_FILES)? >= MIN_LEN as u32,
        le32(prefix, RESERVED3)? == 0,
        le16(prefix, VERSION)? == 0x0103,
        le16(prefix, C_FOLDERS)? >= 1,
        le16(prefix, FLAGS)? <= 7,
    );
    Some(Label::Cab)
}

/// Google Chrome extension (`Cr24`): version 3 with nonzero header length at `@8`, or version 2
/// with public-key and signature lengths in `1..=65536` at `@8` and `@12`.
fn crx(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"Cr24"; // @0..4: CRX magic
    const VERSION: usize = 4; // @4: u32 LE — CRX format version (2 or 3)
    const LEN_A: usize = 8; // @8: u32 LE — CRX3 header length or CRX2 public key length
    const LEN_B: usize = 12; // @12: u32 LE — CRX2 signature length
    const MIN_LEN: usize = 12;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    let valid = match le32(prefix, VERSION)? {
        3 => le32(prefix, LEN_A)? >= 1,
        2 => {
            prefix.len() >= 16
                && (1..=65536).contains(&le32(prefix, LEN_A)?)
                && (1..=65536).contains(&le32(prefix, LEN_B)?)
        }
        _ => false,
    };
    require!(valid);
    Some(Label::Crx)
}

/// Free Lossless Audio Codec (FLAC): `"fLaC"` magic followed by a 34-byte `STREAMINFO` block
/// (type 0, optionally last block `0x80`), block sizes ≥ 16 at `@8` and `@10`, and sample rate /
/// channel word ≥ 4096 at `@18`.
fn flac(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"fLaC"; // @0..4: FLAC stream marker
    const BLOCK_HDR_TYPE: usize = 4; // @4: u8 — 0x00 or 0x80 (last-block flag + STREAMINFO type 0)
    const BLOCK_HDR_LEN: &[u8] = &[0x00, 0x00, 0x22]; // @5..8: u24 BE — STREAMINFO length = 34
    const MIN_BLOCK: usize = 8; // @8: u16 BE — minimum block size in samples (≥ 16)
    const MAX_BLOCK: usize = 10; // @10: u16 BE — maximum block size in samples (≥ 16)
    const RATE_CH_BPS: usize = 18; // @18: u32 BE — sample rate (20 bits) + channels + bps (≥ 4096)
    const MIN_LEN: usize = 42; // 4-byte magic + 4-byte metadata header + 34-byte STREAMINFO

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, BLOCK_HDR_TYPE)?, 0x00 | 0x80),
        eq_at(prefix, 5, BLOCK_HDR_LEN),
        be16(prefix, MIN_BLOCK)? >= 16,
        be16(prefix, MAX_BLOCK)? >= 16,
        be32(prefix, RATE_CH_BPS)? >= 4096,
    );
    Some(Label::Flac)
}

/// CompuServe GIF (`"GIF87a"` or `"GIF89a"`): nonzero width and height at `@6` and `@8`, and a
/// valid block introducer (`0x21` extension, `0x2C` image descriptor, or `0x3B` trailer) right
/// after the optional Global Color Table.
fn gif(prefix: &[u8]) -> Option<Label> {
    const GIF87A: &[u8] = b"GIF87a"; // @0..6: GIF87a header
    const GIF89A: &[u8] = b"GIF89a"; // @0..6: GIF89a header
    const WIDTH: usize = 6; // @6: u16 LE — logical screen width (> 0)
    const HEIGHT: usize = 8; // @8: u16 LE — logical screen height (> 0)
    const PACKED: usize = 10; // @10: u8 — bit 7 = GCT present, bits 2..0 = GCT size exponent
    const END_OF_LSD: usize = 13; // @13: first byte after 6-byte header + 7-byte Logical Screen Descriptor
    const MIN_LEN: usize = 14;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, GIF87A) || starts(prefix, GIF89A),
        le16(prefix, WIDTH)? > 0,
        le16(prefix, HEIGHT)? > 0,
    );
    let packed = u8_at(prefix, PACKED)?;
    let first_block =
        if packed < 0x80 { END_OF_LSD } else { END_OF_LSD + 3 * (1usize << ((packed & 0x07) + 1)) };
    require!(matches!(u8_at(prefix, first_block)?, 0x21 | 0x2C | 0x3B));
    Some(Label::Gif)
}

/// Windows Help (`.hlp`): magic `0x00035F3F` LE, directory start ≥ 16 at `@4`, no-free-chain
/// sentinel `00 00 FF FF FF FF` at `@6..12`, and entire file size ≥ 16 at `@12`.
fn hlp(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = &0x0003_5F3Fu32.to_le_bytes(); // @0..4: WinHelp magic (3F 5F 03 00)
    const DIR_START: usize = 4; // @4: u32 LE — offset of internal directory (≥ 16)
    const NO_FREE_CHAIN: &[u8] = &[0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF]; // @6..12: high bytes of DirStart + FirstFreeBlock = -1
    const FILE_SIZE: usize = 12; // @12: u32 LE — entire file size (≥ 16)
    const HEADER_LEN: usize = 16;

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, MAGIC),
        eq_at(prefix, 6, NO_FREE_CHAIN),
        le32(prefix, DIR_START)? >= HEADER_LEN as u32,
        le32(prefix, FILE_SIZE)? >= HEADER_LEN as u32,
    );
    Some(Label::Hlp)
}

/// Windows Icon (`ICONDIR` + first `ICONDIRENTRY`): reserved `0`, type `1` (`00 00 01 00`),
/// image count ≥ 1 at `@4`, reserved `0` at `@9`, bytes in resource ≥ 8 at `@14`, and image
/// offset ≥ 22 at `@18`.
fn ico(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = &[0x00, 0x00, 0x01, 0x00]; // @0..4: idReserved = 0, idType = 1 (icon)
    const ID_COUNT: usize = 4; // @4: u16 LE — number of images (≥ 1)
    const ENTRY_RESERVED: usize = 9; // @9: u8 — first ICONDIRENTRY.bReserved (0)
    const BYTES_IN_RES: usize = 14; // @14: u32 LE — resource byte size (≥ 8)
    const IMAGE_OFFSET: usize = 18; // @18: u32 LE — offset to image data (≥ 22)
    const MIN_LEN: usize = 22; // 6-byte ICONDIR + 16-byte ICONDIRENTRY

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, HEADER),
        le16(prefix, ID_COUNT)? >= 1,
        u8_at(prefix, ENTRY_RESERVED)? == 0,
        le32(prefix, BYTES_IN_RES)? >= 8,
        le32(prefix, IMAGE_OFFSET)? >= MIN_LEN as u32,
    );
    Some(Label::Ico)
}

/// Amiga EA IFF 85 bitmap image: `"FORM"` container magic at `@0` and `"ILBM"`, `"ACBM"`, or
/// `"PBM "` form type at `@8`.
fn ilbm(prefix: &[u8]) -> Option<Label> {
    const FORM: &[u8] = b"FORM"; // @0..4: EA IFF 85 FORM chunk
    const FORM_TYPE: usize = 8; // @8..12: IFF form type
    const MIN_LEN: usize = 12;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, FORM),
        eq_at(prefix, FORM_TYPE, b"ILBM")
            || eq_at(prefix, FORM_TYPE, b"ACBM")
            || eq_at(prefix, FORM_TYPE, b"PBM "),
    );
    Some(Label::Ilbm)
}

/// ISO-BMFF 12-byte signature box overlap verifier: dispatches between JPEG 2000 (`"jP  \r\n\x87\n"`
/// + `"ftypjp2 "`) and JPEG XL container (`"JXL \r\n\x87\n"` + `"ftypjxl "`).
fn jp2_or_jxl(prefix: &[u8]) -> Option<Label> {
    const JP2_SIG: &[u8] = bytes!(&12u32.to_be_bytes(), b"jP  \r\n\x87\n"); // @0..12: JPEG 2000 signature box
    const JXL_SIG: &[u8] = bytes!(&12u32.to_be_bytes(), b"JXL \r\n\x87\n"); // @0..12: JPEG XL signature box
    const FTYP_LEN: usize = 12; // @12: u32 BE — ftyp box length
    const FTYP_BRAND: usize = 16; // @16..24: "ftyp" + 4-byte major brand
    const JP2_MIN_LEN: usize = 28;
    const JXL_MIN_LEN: usize = 32;

    if prefix.len() >= JP2_MIN_LEN
        && starts(prefix, JP2_SIG)
        && eq_at(prefix, FTYP_BRAND, b"ftypjp2 ")
    {
        let ftyp_len = be32(prefix, FTYP_LEN)?;
        require!(ftyp_len >= 16, ftyp_len % 4 == 0);
        return Some(Label::Jp2);
    }
    if prefix.len() >= JXL_MIN_LEN
        && starts(prefix, JXL_SIG)
        && eq_at(prefix, FTYP_BRAND, b"ftypjxl ")
        && be32(prefix, FTYP_LEN)? >= 20
    {
        return Some(Label::Jxl);
    }
    None
}

/// Commodore 64 compressed Koala Painter image: fixed 4-byte header `FF 80 C9 C7` and ≥ 16 bytes.
fn koala(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = &[0xFF, 0x80, 0xC9, 0xC7]; // @0..4: Koala compressed load header
    const MIN_LEN: usize = 16;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    Some(Label::Koala)
}

/// Apple Mach-O thin binary: 32-bit (`0xFEEDFACE`, ≥ 28 bytes) or 64-bit (`0xFEEDFACF`, ≥ 32
/// bytes) in either byte order, nonzero `filetype` at `@12`, and consistent `ncmds`/`sizeofcmds`
/// at `@16` and `@20`.
fn macho(prefix: &[u8]) -> Option<Label> {
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
    Some(Label::Macho)
}

/// Matroska container (`.mkv`): EBML header `1A 45 DF A3`, non-zero/non-FF VINT byte at `@4`,
/// a `DocType` element (`42 82` + VINT size 8 + `"matroska"`) starting in `@5..=28`, and
/// `"matroska"` aligned at offset 8, 24, or 31.
fn mkv(prefix: &[u8]) -> Option<Label> {
    const EBML_MAGIC: &[u8] = &[0x1A, 0x45, 0xDF, 0xA3]; // @0..4: EBML Header element ID
    const VINT_FIRST: usize = 4; // @4: u8 — first byte of EBML header size VINT (1..=254)
    const DOCTYPE_ID: &[u8] = &[0x42, 0x82]; // DocType element ID
    const MATROSKA: &[u8] = b"matroska"; // 8-byte DocType string
    const MIN_LEN: usize = 16;

    fn has_doctype_at(h: &[u8], pos: usize) -> bool {
        if !eq_at(h, pos, DOCTYPE_ID) {
            return false;
        }
        let vint_start = pos + 2;
        // Size 8 encoded in 1..=8-byte EBML VINT: leading byte `(0x80 >> k)` followed by `k - 1`
        // zero bytes and final byte `0x08` (or single byte `0x88` for width 1).
        for width in 1..=8usize {
            let lead = 0x80u8 >> (width - 1);
            if width == 1 {
                if u8_at(h, vint_start) == Some(lead | 0x08) && eq_at(h, vint_start + 1, MATROSKA) {
                    return true;
                }
            } else if u8_at(h, vint_start) == Some(lead)
                && h.get(vint_start + 1..vint_start + width - 1)
                    .is_some_and(|zeros| zeros.iter().all(|&b| b == 0))
                && u8_at(h, vint_start + width - 1) == Some(0x08)
                && eq_at(h, vint_start + width, MATROSKA)
            {
                return true;
            }
        }
        false
    }

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, EBML_MAGIC),
        (1..=254).contains(&u8_at(prefix, VINT_FIRST)?),
        eq_at(prefix, 8, MATROSKA) || eq_at(prefix, 24, MATROSKA) || eq_at(prefix, 31, MATROSKA),
        (5..=28).any(|pos| has_doctype_at(prefix, pos)),
    );
    Some(Label::Mkv)
}

/// Microsoft `compress.exe` SZDD archive: `"SZDD\x88\xf0\x27\x33"` magic, compression mode `'A'`
/// or `'B'` at `@8`, and 14-byte header.
fn mscompress(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"SZDD\x88\xf0\x27\x33"; // @0..8: SZDD signature
    const MODE: usize = 8; // @8: u8 — compression mode ('A' or 'B')
    const HEADER_LEN: usize = 14;

    require!(
        prefix.len() >= HEADER_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, MODE)?, b'A' | b'B'),
    );
    Some(Label::Mscompress)
}

/// Unidata NetCDF classic (`"CDF\x01"`) or 64-bit offset (`"CDF\x02"`): ≥ 32-byte header with
/// either `NC_DIMENSION` tag `10` at `@8` or zero tag at `@12`.
fn netcdf(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"CDF"; // @0..3: NetCDF magic prefix
    const VERSION: usize = 3; // @3: u8 — 1 (classic) or 2 (64-bit offset)
    const DIM_TAG: usize = 8; // @8: u32 BE — NC_DIMENSION tag (10) or ABSENT (0)
    const ZERO_FIELD: usize = 12; // @12: u32 BE — nelems when absent (0)
    const MIN_LEN: usize = 32;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, VERSION)?, 1 | 2),
        be32(prefix, ZERO_FIELD)? == 0 || be32(prefix, DIM_TAG)? == 10,
    );
    Some(Label::Netcdf)
}

/// Ogg bitstream (`"OggS"`): version 0, BOS flag (`0x02`, optionally EOS `0x06`) at `@5`, initial
/// page sequence number `0` at `@18`, and ≥ 1 segment at `@26`.
fn ogg(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"OggS\x00"; // @0..5: Ogg capture pattern + stream_structure_version 0
    const HEADER_TYPE: usize = 5; // @5: u8 — BOS (0x02) or BOS|EOS (0x06)
    const PAGE_SEQ_NO: usize = 18; // @18: u32 LE — initial page sequence number (0)
    const PAGE_SEGMENTS: usize = 26; // @26: u8 — number of segment table entries (≥ 1)
    const MIN_LEN: usize = 28; // 27-byte Ogg page header + ≥ 1 segment table byte

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, HEADER_TYPE)?, 0x02 | 0x06),
        le32(prefix, PAGE_SEQ_NO)? == 0,
        u8_at(prefix, PAGE_SEGMENTS)? >= 1,
    );
    Some(Label::Ogg)
}

/// libpcap capture file: microsecond (`0xA1B2C3D4`) or nanosecond (`0xA1B23C4D`) magic in either
/// byte order, version 2.4 at `@4..8`, and nonzero `snaplen` at `@16`.
fn pcap(prefix: &[u8]) -> Option<Label> {
    const MAGIC_USEC: u32 = 0xA1B2_C3D4; // microsecond-resolution pcap magic
    const MAGIC_NSEC: u32 = 0xA1B2_3C4D; // nanosecond-resolution pcap magic
    const MAJOR: usize = 4; // @4: u16 — major version (2)
    const MINOR: usize = 6; // @6: u16 — minor version (4)
    const SNAPLEN: usize = 16; // @16: u32 — maximum captured packet length (> 0)
    const HEADER_LEN: usize = 24;

    require!(prefix.len() >= HEADER_LEN);
    let le_ok = matches!(le32(prefix, 0)?, MAGIC_USEC | MAGIC_NSEC)
        && le16(prefix, MAJOR)? == 2
        && le16(prefix, MINOR)? == 4
        && le32(prefix, SNAPLEN)? > 0;
    let be_ok = matches!(be32(prefix, 0)?, MAGIC_USEC | MAGIC_NSEC)
        && be16(prefix, MAJOR)? == 2
        && be16(prefix, MINOR)? == 4
        && be32(prefix, SNAPLEN)? > 0;
    require!(le_ok || be_ok);
    Some(Label::Pcap)
}

/// Microsoft Program Database (PDB): MSF 7.00 header (`"Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0"`)
/// with valid power-of-two page size at `@32` and free block map `1 | 2` at `@36`, or legacy JG
/// 2.00 header (`"Microsoft C/C++ program database 2.00\r\n\x1aJG\0\0"`) with page size at `@44`.
fn pdb(prefix: &[u8]) -> Option<Label> {
    const MODERN_MAGIC: &[u8] = b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0"; // @0..32: MSF 7.00 signature
    const OLD_MAGIC: &[u8] = b"Microsoft C/C++ program database 2.00\r\n\x1aJG\0\0"; // @0..44: MSF 2.00 signature
    const MODERN_PAGE_SIZE: usize = 32; // @32: u32 LE — page size (512..=32768)
    const MODERN_FPM: usize = 36; // @36: u32 LE — free block map block (1 or 2)
    const OLD_PAGE_SIZE: usize = 44; // @44: u32 LE — page size (512..=32768)
    const MODERN_MIN_LEN: usize = 56;
    const OLD_MIN_LEN: usize = 60;

    fn valid_page_size(n: u32) -> bool {
        matches!(n, 0x0200 | 0x0400 | 0x0800 | 0x1000 | 0x2000 | 0x4000 | 0x8000)
    }

    let modern = prefix.len() >= MODERN_MIN_LEN
        && starts(prefix, MODERN_MAGIC)
        && valid_page_size(le32(prefix, MODERN_PAGE_SIZE)?)
        && matches!(le32(prefix, MODERN_FPM)?, 1 | 2);
    let old = prefix.len() >= OLD_MIN_LEN
        && starts(prefix, OLD_MAGIC)
        && valid_page_size(le32(prefix, OLD_PAGE_SIZE)?);
    require!(modern || old);
    Some(Label::Pdb)
}

/// RPM package: 96-byte lead (`ED AB EE DB`, major version `3 | 4`, minor `0`, type `0 | 1` at `@6`)
/// followed at `@96` by the 16-byte signature header magic (`8E AD E8 01 00 00 00 00`).
fn rpm(prefix: &[u8]) -> Option<Label> {
    const LEAD_MAGIC: &[u8] = &[0xED, 0xAB, 0xEE, 0xDB]; // @0..4: RPM lead magic
    const MAJOR: usize = 4; // @4: u8 — major version (3 or 4)
    const MINOR: usize = 5; // @5: u8 — minor version (0)
    const PACKAGE_TYPE: usize = 6; // @6: u16 BE — 0 (binary) or 1 (source)
    const SIG_HEADER: &[u8] = &[0x8E, 0xAD, 0xE8, 0x01, 0x00, 0x00, 0x00, 0x00]; // @96..104: header v1 magic + reserved
    const SIG_OFFSET: usize = 96; // @96: end of 96-byte RPM lead
    const MIN_LEN: usize = 112; // 96-byte lead + 16-byte signature header

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, LEAD_MAGIC),
        matches!(u8_at(prefix, MAJOR)?, 3 | 4),
        u8_at(prefix, MINOR)? == 0,
        be16(prefix, PACKAGE_TYPE)? <= 1,
        eq_at(prefix, SIG_OFFSET, SIG_HEADER),
    );
    Some(Label::Rpm)
}

/// Stata `.dta` XML dataset: `"<stata_dta><header><release>"` + 3 ASCII digits +
/// `"</release><byteorder>"` + `"LSF</byteorder>"` or `"MSF</byteorder>"`.
fn stata(prefix: &[u8]) -> Option<Label> {
    const PREFIX: &[u8] = b"<stata_dta><header><release>"; // @0..28: opening tags
    const RELEASE_DIGITS: usize = 28; // @28..31: 3-digit release code
    const MID: &[u8] = b"</release><byteorder>"; // @31..52: closing release + opening byteorder
    const BYTEORDER: usize = 52; // @52..67: "LSF</byteorder>" or "MSF</byteorder>"
    const MIN_LEN: usize = 67;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, PREFIX),
        prefix[RELEASE_DIGITS..31].iter().all(u8::is_ascii_digit),
        eq_at(prefix, 31, MID),
        eq_at(prefix, BYTEORDER, b"LSF</byteorder>")
            || eq_at(prefix, BYTEORDER, b"MSF</byteorder>"),
    );
    Some(Label::Stata)
}

/// Adobe PostScript Document Structuring Conventions: opens with `"%!PS-Adobe"`.
fn postscript(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"%!PS-Adobe"; // @0..10: conforming PostScript DSC header

    require!(starts(prefix, MAGIC));
    Some(Label::Postscript)
}

// =============================================================================
// Tier 2: 2-byte magic verifiers (MAGIC16)
// =============================================================================

/// Autodesk 3ds Max (`.3ds`): root chunk `M3DMAGIC` (`0x4D4D` = `"MM"`) at `@0` with length ≥ 16
/// at `@2`, followed at `@6` by either `M3D_VERSION` (`0x0002`, length 10, version 3..=4 at `@12`)
/// or `MDATA` (`0x3D3D` = `"=="`, length ≥ 6 at `@8`).
fn three_dsm(prefix: &[u8]) -> Option<Label> {
    const ROOT_MAGIC: &[u8] = b"MM"; // @0..2: u16 LE — M3DMAGIC (0x4D4D)
    const ROOT_LEN: usize = 2; // @2: u32 LE — root chunk byte length (≥ 16)
    const CHILD_OFFSET: usize = 6; // @6: first child chunk header
    const VERSION_CHUNK: &[u8] = bytes!(&2u16.to_le_bytes(), &10u32.to_le_bytes()); // @6..12: M3D_VERSION (0x0002) + length 10
    const VERSION_VALUE: usize = 12; // @12: u32 LE — 3DS version (3 or 4)
    const EDITOR_MAGIC: &[u8] = b"=="; // @6..8: u16 LE — MDATA (0x3D3D)
    const EDITOR_LEN: usize = 8; // @8: u32 LE — MDATA chunk length (≥ 6)
    const MIN_LEN: usize = 16;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, ROOT_MAGIC),
        le32(prefix, ROOT_LEN)? >= MIN_LEN as u32,
        (eq_at(prefix, CHILD_OFFSET, VERSION_CHUNK)
            && matches!(le32(prefix, VERSION_VALUE)?, 3 | 4))
            || (eq_at(prefix, CHILD_OFFSET, EDITOR_MAGIC) && le32(prefix, EDITOR_LEN)? >= 6),
    );
    Some(Label::_3dsm)
}

/// bzip2 compressed stream: `"BZh"` + block-size digit `'1'..='9'`, followed at `@4` by either the
/// 6-byte block header magic (`0x314159265359`, π in BCD, total prefix ≥ 20) or the 10-byte empty
/// end-of-stream marker (`0x177245385090`, √π in BCD, plus 4 zero CRC bytes, total prefix ≥ 14).
fn bzip(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"BZh"; // @0..3: bzip2 stream magic + 'h' (Huffman)
    const LEVEL: usize = 3; // @3: u8 — block size '1'..='9' (100..900 KiB)
    const PAYLOAD: usize = 4; // @4: first block or end-of-stream header
    // BCD digits of π (3.14159265359) — kept in hex because ASCII "1AY&SY" is coincidental.
    const BLOCK_MAGIC: &[u8] = &[0x31, 0x41, 0x59, 0x26, 0x53, 0x59];
    // BCD digits of √π (1.77245385090) followed by 4 zero stream-CRC bytes.
    const EMPTY_STREAM: &[u8] = bytes!(&[0x17, 0x72, 0x45, 0x38, 0x50, 0x90], &0u32.to_be_bytes());
    const MIN_EMPTY_LEN: usize = 14;
    const MIN_BLOCK_LEN: usize = 20;

    require!(
        prefix.len() >= MIN_EMPTY_LEN,
        starts(prefix, MAGIC),
        matches!(u8_at(prefix, LEVEL)?, b'1'..=b'9'),
        eq_at(prefix, PAYLOAD, EMPTY_STREAM)
            || (prefix.len() >= MIN_BLOCK_LEN && eq_at(prefix, PAYLOAD, BLOCK_MAGIC)),
    );
    Some(Label::Bzip)
}

/// RFC 1952 gzip member: magic `1F 8B`, compression method `08` (DEFLATE), reserved flag bits
/// `5..7` clear at `@3` (`<= 31`), and at least a 20-byte member (10-byte header + 2-byte minimum
/// deflate block + 8-byte trailer).
fn gzip(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = &[0x1F, 0x8B, 0x08]; // @0..3: ID1=0x1F, ID2=0x8B, CM=8 (DEFLATE)
    const FLG: usize = 3; // @3: u8 — member flags (bits 5..7 reserved zero → <= 0x1F)
    const MIN_LEN: usize = 20; // 10-byte header + 2-byte deflate block + 8-byte CRC32/ISIZE trailer

    require!(prefix.len() >= MIN_LEN, starts(prefix, HEADER), u8_at(prefix, FLG)? <= 0x1F,);
    Some(Label::Gzip)
}

/// Netpbm image (`"P1"`..`"P6"`): 1..=8 whitespace bytes, up to 2 `#...\n` comment lines (each ≤
/// 128 comment bytes followed by `\n` and 0..=4 whitespace bytes), width (1..=6 ASCII digits),
/// 1..=4 whitespace bytes, height (1..=6 ASCII digits), and a trailing whitespace byte.
fn pbm(prefix: &[u8]) -> Option<Label> {
    #[inline]
    fn is_ws(b: u8) -> bool {
        matches!(b, b' ' | b'\t' | b'\r' | b'\n')
    }

    #[inline]
    fn take_run(s: &[u8], i: &mut usize, pred: fn(u8) -> bool) -> usize {
        let start = *i;
        while *i < s.len() && pred(s[*i]) {
            *i += 1;
        }
        *i - start
    }

    require!(prefix.len() >= 7, prefix[0] == b'P', matches!(prefix[1], b'1'..=b'6'));
    let mut i = 2;

    // [ \t\r\n]{1,8}
    let ws0 = take_run(prefix, &mut i, is_ws);
    require!((1..=8).contains(&ws0));

    // (#[^\n]{0,128}\n[ \t\r\n]{0,4}){0,2}
    for _ in 0..2 {
        if u8_at(prefix, i) != Some(b'#') {
            break;
        }
        i += 1;
        let comment_len = take_run(prefix, &mut i, |b| b != b'\n');
        require!(comment_len <= 128, u8_at(prefix, i) == Some(b'\n'));
        i += 1;
        let ws_after = take_run(prefix, &mut i, is_ws);
        require!(ws_after <= 4);
    }

    // [0-9]{1,6}[ \t\r\n]{1,4}[0-9]{1,6}[ \t\r\n]
    let width_digits = take_run(prefix, &mut i, |b| b.is_ascii_digit());
    require!((1..=6).contains(&width_digits));
    let ws_mid = take_run(prefix, &mut i, is_ws);
    require!((1..=4).contains(&ws_mid));
    let height_digits = take_run(prefix, &mut i, |b| b.is_ascii_digit());
    require!((1..=6).contains(&height_digits), is_ws(u8_at(prefix, i)?),);
    Some(Label::Pbm)
}

/// Adobe Flash SWF (`"FWS"`, `"CWS"`, or `"ZWS"`): version 1..=50 at `@3`, declared file length
/// ≥ 15 at `@4`, and compression-specific header fields at `@8`.
fn swf(prefix: &[u8]) -> Option<Label> {
    const VERSION: usize = 3; // @3: u8 — SWF format version (1..=50)
    const FILE_LENGTH: usize = 4; // @4: u32 LE — uncompressed file length (≥ 15)
    const PAYLOAD: usize = 8; // @8: RECT (FWS), zlib header (CWS), or LZMA header (ZWS)
    const MIN_LEN: usize = 15;

    require!(prefix.len() >= MIN_LEN, le32(prefix, FILE_LENGTH)? >= MIN_LEN as u32);
    let ver = u8_at(prefix, VERSION)?;
    require!((1..=50).contains(&ver));

    let valid = if starts(prefix, b"FWS") {
        let rect0 = u8_at(prefix, PAYLOAD)?;
        let rect1 = u8_at(prefix, PAYLOAD + 1)?;
        (8..=248).contains(&rect0) && rect0 % 8 == 0 && rect1 <= 31
    } else if starts(prefix, b"CWS") {
        ver >= 6
            && u8_at(prefix, PAYLOAD)? == 0x78
            && matches!(u8_at(prefix, PAYLOAD + 1)?, 0x01 | 0x5E | 0x9C | 0xDA)
    } else if starts(prefix, b"ZWS") {
        const LZMA_PROPS: usize = 12; // @12: u8 — LZMA lc/lp/pb properties (0x5D = 93)
        const LZMA_DICT: usize = 13; // @13: u32 LE — LZMA dictionary size (4 KiB .. 1 GiB)
        prefix.len() >= 17
            && ver >= 13
            && le32(prefix, PAYLOAD)? >= 5
            && u8_at(prefix, LZMA_PROPS)? == 0x5D
            && (4096..=1_073_741_824).contains(&le32(prefix, LZMA_DICT)?)
    } else {
        false
    };
    require!(valid);
    Some(Label::Swf)
}

/// IBM AIX XCOFF object/executable: magic `0x01F7` (XCOFF64) or `0x01DF` (XCOFF32) at `@0`,
/// section count `1..=1024` at `@2`, and a standard auxiliary header size at `@16` (`0` or `120`
/// for 64-bit; `0`, `28`, or `72` for 32-bit).
fn xcoff(prefix: &[u8]) -> Option<Label> {
    const MAGIC: usize = 0; // @0: u16 BE — 0x01F7 (XCOFF64) or 0x01DF (XCOFF32)
    const XCOFF64: u16 = 0x01F7; // 503
    const XCOFF32: u16 = 0x01DF; // 479
    const N_SCNS: usize = 2; // @2: u16 BE — number of sections (1..=1024)
    const OPT_HDR: usize = 16; // @16: u16 BE — optional/auxiliary header byte size
    const MIN_LEN: usize = 24;

    require!(prefix.len() >= MIN_LEN, (1..=1024).contains(&be16(prefix, N_SCNS)?),);
    let opt = be16(prefix, OPT_HDR)?;
    let valid = match be16(prefix, MAGIC)? {
        XCOFF64 => matches!(opt, 0 | 120),
        XCOFF32 => matches!(opt, 0 | 28 | 72),
        _ => false,
    };
    require!(valid);
    Some(Label::Xcoff)
}

/// Windows or OS/2 BMP image (`"BM"`): zero reserved words at `@6..10`, file size and pixel
/// offset at `@2` and `@10`, followed at `@14` by either a 12-byte OS/2 `BITMAPCOREHEADER` or an
/// uncompressed Windows DIB header (40, 52, 56, 108, or 124 bytes).
fn bmp(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"BM"; // @0..2: BITMAPFILEHEADER magic
    const FILE_SIZE: usize = 2; // @2: u32 LE — bfSize (≥ 26)
    const RESERVED: usize = 6; // @6: u32 LE — bfReserved1 + bfReserved2 (0)
    const OFF_BITS: usize = 10; // @10: u32 LE — bfOffBits (pixel array offset)
    const DIB_SIZE: usize = 14; // @14: u32 LE — DIB header size
    const MIN_CORE_LEN: usize = 26; // 14-byte file header + 12-byte BITMAPCOREHEADER
    const MIN_WIN_LEN: usize = 54; // 14-byte file header + 40-byte BITMAPINFOHEADER

    require!(
        prefix.len() >= MIN_CORE_LEN,
        starts(prefix, MAGIC),
        le32(prefix, RESERVED)? == 0,
        le32(prefix, FILE_SIZE)? >= MIN_CORE_LEN as u32,
    );
    let off_bits = le32(prefix, OFF_BITS)?;
    require!(off_bits >= MIN_CORE_LEN as u32);
    let dib_size = le32(prefix, DIB_SIZE)?;
    let valid = if dib_size == 12 {
        // OS/2 BITMAPCOREHEADER: u16 width @18, u16 height @20, u16 planes @22, u16 bpp @24
        le16(prefix, 18)? >= 1
            && le16(prefix, 20)? >= 1
            && le16(prefix, 22)? == 1
            && matches!(le16(prefix, 24)?, 1 | 4 | 8 | 24)
    } else {
        // Windows BITMAPINFOHEADER / V2 / V3 / V4 / V5
        prefix.len() >= MIN_WIN_LEN
            && off_bits >= MIN_WIN_LEN as u32
            && matches!(dib_size, 40 | 52 | 56 | 108 | 124)
            && (1..=0x7FFF_FFFF).contains(&le32(prefix, 18)?)
            && le32(prefix, 22)? >= 1
            && le16(prefix, 26)? == 1
            && matches!(le16(prefix, 28)?, 1 | 4 | 8 | 16 | 24 | 32)
            && le32(prefix, 30)? == 0
    };
    require!(valid);
    Some(Label::Bmp)
}

/// Unix `compress` (LZW, `.Z`): magic `1F 9D` at `@0`, maxbits `9..=16` in bits `0..4` of `@2`,
/// reserved bits `5..6` clear, optional block-mode bit `7` (`0x80`).
fn unixcompress(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = &[0x1F, 0x9D]; // @0..2: Unix compress LZW magic
    const FLAGS: usize = 2; // @2: u8 — bit 7 = block mode, bits 6..5 = 0, bits 4..0 = maxbits (9..=16)
    const MIN_LEN: usize = 8;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        (9..=16).contains(&(u8_at(prefix, FLAGS)? & !0x80)),
    );
    Some(Label::Unixcompress)
}

// =============================================================================
// Tier 3: Fixed non-zero offset probes
// =============================================================================

/// ISO Base Media File Format `ftyp` box at `@4` (3GPP, AVIF, or HEIF/HEVC): 4-byte-aligned box
/// length ≥ 16 at `@0`, `"ftyp"` at `@4`, and a 4-byte major brand at `@8`.
fn ftyp(prefix: &[u8]) -> Option<Label> {
    const BOX_LEN: usize = 0; // @0: u32 BE — ftyp box byte length (≥ 16, multiple of 4)
    const BOX_TYPE: &[u8] = b"ftyp"; // @4..8: ISO-BMFF file-type box tag
    const MAJOR_BRAND: usize = 8; // @8..12: 4-byte major brand code
    const MIN_LEN: usize = 16; // 8-byte box header + 4-byte major brand + 4-byte minor version

    require!(prefix.len() >= MIN_LEN, eq_at(prefix, 4, BOX_TYPE));
    let box_len = be32(prefix, BOX_LEN)?;
    require!(box_len >= MIN_LEN as u32, box_len % 4 == 0);
    let brand: &[u8; 4] = prefix[MAJOR_BRAND..MAJOR_BRAND + 4].try_into().ok()?;
    Some(match brand {
        b"3ge6" | b"3ge7" | b"3ge9" | b"3gg6" | b"3gg9" | b"3gh9" | b"3gm9" | b"3gmA" | b"3gp1"
        | b"3gp2" | b"3gp3" | b"3gp4" | b"3gp5" | b"3gp6" | b"3gp7" | b"3gp8" | b"3gp9"
        | b"3gr6" | b"3gr9" | b"3gs6" | b"3gs7" | b"3gs9" | b"3gt8" | b"3gt9" | b"3gtv" => {
            Label::_3gp
        }
        b"avif" | b"avis" => Label::Avif,
        b"heic" | b"heix" | b"heim" | b"heis" | b"hevc" | b"hevx" | b"hevm" | b"hevs" => {
            Label::Heif
        }
        _ => return None,
    })
}

/// WinACE archive (`"**ACE**"` at `@7`): header size ≥ 27 at `@2`, header type `0` at `@4`, even
/// header flags at `@5`, and at least 31 bytes in the prefix.
fn ace(prefix: &[u8]) -> Option<Label> {
    const HDR_SIZE: usize = 2; // @2: u16 LE — header size after the 4-byte CRC+size prefix (≥ 27)
    const HDR_TYPE: usize = 4; // @4: u8 — 0 = MAIN_BLK
    const HDR_FLAGS: usize = 5; // @5: u8 — bit 0 (extendable) must be 0 on MAIN_BLK
    const MAGIC_OFFSET: usize = 7; // @7..14: "**ACE**"
    const MAGIC: &[u8] = b"**ACE**";
    const MIN_LEN: usize = 31;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, MAGIC_OFFSET, MAGIC),
        le16(prefix, HDR_SIZE)? >= 27,
        u8_at(prefix, HDR_TYPE)? == 0,
        u8_at(prefix, HDR_FLAGS)? % 2 == 0,
    );
    Some(Label::Ace)
}

/// DICOM medical image (PS3.10): 128-byte preamble followed by `"DICM"` at `@128`.
fn dicom(prefix: &[u8]) -> Option<Label> {
    const MAGIC_OFFSET: usize = 128; // @128..132: "DICM" after 128-byte preamble
    const MAGIC: &[u8] = b"DICM";
    const MIN_LEN: usize = 132;

    require!(prefix.len() >= MIN_LEN, eq_at(prefix, MAGIC_OFFSET, MAGIC),);
    Some(Label::Dicom)
}

/// DuckDB database file: full 4 KiB main header with `"DUCK"` at `@8` and a 64-bit version word
/// (`1..=u32::MAX`) at `@12..20`.
fn duckdb(prefix: &[u8]) -> Option<Label> {
    const MAGIC_OFFSET: usize = 8; // @8..12: "DUCK" after 8-byte checksum
    const MAGIC: &[u8] = b"DUCK";
    const VERSION_LO: usize = 12; // @12: u32 LE — format version low word (≥ 1)
    const VERSION_HI: usize = 16; // @16: u32 LE — format version high word (0)
    const MIN_LEN: usize = 4096;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, MAGIC_OFFSET, MAGIC),
        le32(prefix, VERSION_LO)? >= 1,
        le32(prefix, VERSION_HI)? == 0,
    );
    Some(Label::Duckdb)
}

/// Microsoft Extensible Storage Engine (ESE / Jet Blue) database: magic `0x89ABCDEF` LE at `@4`,
/// nonzero `ulVersion` at `@8`, `attrib` `0` or `1` at `@12`, and zero word at `@132`.
fn ese(prefix: &[u8]) -> Option<Label> {
    const MAGIC_OFFSET: usize = 4; // @4..8: u32 LE — 0x89ABCDEF after 4-byte checksum
    const MAGIC: &[u8] = &0x89AB_CDEFu32.to_le_bytes();
    const VERSION: usize = 8; // @8: u32 LE — ulVersion (≥ 1)
    const ATTRIB: usize = 12; // @12: u32 LE — file type (0 = database, 1 = streaming file)
    const RESERVED_ZERO: usize = 132; // @132: u32 LE — zero field
    const MIN_LEN: usize = 668;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, MAGIC_OFFSET, MAGIC),
        le32(prefix, VERSION)? >= 1,
        le32(prefix, ATTRIB)? <= 1,
        le32(prefix, RESERVED_ZERO)? == 0,
    );
    Some(Label::Ese)
}

/// International Color Consortium (ICC) color profile: 128-byte profile header + 4-byte tag count,
/// signature `"acsp"` at `@36`, and a known device/profile class at `@12`.
fn icc(prefix: &[u8]) -> Option<Label> {
    const DEVICE_CLASS: usize = 12; // @12..16: 4-byte profile/device class signature
    const MAGIC_OFFSET: usize = 36; // @36..40: "acsp" profile file signature
    const MAGIC: &[u8] = b"acsp";
    const MIN_LEN: usize = 132; // 128-byte header + 4-byte tag count

    require!(prefix.len() >= MIN_LEN, eq_at(prefix, MAGIC_OFFSET, MAGIC),);
    let dev: &[u8; 4] = prefix[DEVICE_CLASS..DEVICE_CLASS + 4].try_into().ok()?;
    require!(matches!(
        dev,
        b"\0\0\0\0"
            | b"scnr"
            | b"mntr"
            | b"prtr"
            | b"link"
            | b"abst"
            | b"spac"
            | b"nmcl"
            | b"cenc"
            | b"mid "
            | b"mlnk"
            | b"mvis"
    ));
    Some(Label::Icc)
}

/// Palm OS PRC/PDB application database: NUL terminator at `@31`, type `"appl"` at `@60`, four
/// printable ASCII creator bytes at `@64..68`, and nonzero record count at `@76`.
fn palmos(prefix: &[u8]) -> Option<Label> {
    const NAME_NUL: usize = 31; // @31: u8 — NUL terminator of 32-byte database name
    const DB_TYPE: usize = 60; // @60..64: "appl" database type
    const CREATOR: usize = 64; // @64..68: 4-byte printable ASCII creator ID
    const NUM_RECORDS: usize = 76; // @76: u16 BE — number of records/resources (≥ 1)
    const MIN_LEN: usize = 78;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, DB_TYPE, b"appl"),
        u8_at(prefix, NAME_NUL)? == 0,
        be16(prefix, NUM_RECORDS)? >= 1,
        prefix[CREATOR..CREATOR + 4].iter().all(|&b| (0x20..=0x7E).contains(&b)),
    );
    Some(Label::Palmos)
}

/// SAS7BDAT dataset: 32-byte binary magic at `@0`, alignment/format byte `<= 1` at `@37`, and
/// `"SAS FILE"` at `@84` inside a ≥ 288-byte header.
fn sas(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(
        &[0u8; 12],
        &[
            0xC2, 0xEA, 0x81, 0x60, 0xB3, 0x14, 0x11, 0xCF, 0xBD, 0x92, 0x08, 0x00, 0x09, 0xC7,
            0x31, 0x8C, 0x18, 0x1F, 0x10, 0x11,
        ]
    ); // @0..32: 12 zero bytes + 20-byte SAS7BDAT magic
    const ALIGN_FLAG: usize = 37; // @37: u8 — 0 or 1
    const FILE_TAG_OFFSET: usize = 84; // @84..92: "SAS FILE"
    const FILE_TAG: &[u8] = b"SAS FILE";
    const MIN_LEN: usize = 288;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, FILE_TAG_OFFSET, FILE_TAG),
        starts(prefix, HEADER),
        u8_at(prefix, ALIGN_FLAG)? <= 1,
    );
    Some(Label::Sas)
}

/// Microsoft Enhanced Metafile (MS-EMF): `EMR_HEADER` record type `1` at `@0`, signature `" EMF"`
/// and version `0x00010000` at `@40..48`, aligned header/file sizes, ≥ 2 records, and consistent
/// base/extension1/extension2 descriptor offsets (88, 100, or 108 bytes).
fn emf(prefix: &[u8]) -> Option<Label> {
    const EMR_HEADER: &[u8] = &1u32.to_le_bytes(); // @0..4: u32 LE — iType = EMR_HEADER (1)
    const HEADER_SIZE: usize = 4; // @4: u32 LE — nSize of EMR_HEADER record (≥ 88, multiple of 4)
    const SIGNATURE_OFFSET: usize = 40; // @40..48: dSignature (" EMF") + nVersion (0x00010000)
    const SIGNATURE: &[u8] = bytes!(b" EMF", &0x0001_0000u32.to_le_bytes());
    const N_BYTES: usize = 48; // @48: u32 LE — metafile byte size (≥ header + 20-byte EOF, multiple of 4)
    const N_RECORDS: usize = 52; // @52: u32 LE — number of records (≥ 2)
    const N_DESC: usize = 60; // @60: u32 LE — nDescription length
    const OFF_DESC: usize = 64; // @64: u32 LE — offDescription offset
    const OFF_PFD: usize = 92; // @92: u32 LE — offPixelFormat offset (HeaderExtension1)
    const MIN_LEN: usize = 88;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, SIGNATURE_OFFSET, SIGNATURE),
        starts(prefix, EMR_HEADER),
    );
    let hdr_size = le32(prefix, HEADER_SIZE)?;
    let n_bytes = le32(prefix, N_BYTES)?;
    let off_desc = le32(prefix, OFF_DESC)?;
    require!(
        hdr_size >= 88,
        hdr_size % 4 == 0,
        n_bytes >= 108,
        n_bytes % 4 == 0,
        le32(prefix, N_RECORDS)? >= 2,
    );
    let valid_layout = ((hdr_size == 88 && off_desc == 0) || off_desc == 88)
        || (prefix.len() >= 100
            && hdr_size >= 100
            && n_bytes >= 120
            && ((hdr_size == 100 && le32(prefix, N_DESC)? == 0 && le32(prefix, OFF_PFD)? == 0)
                || off_desc == 100
                || le32(prefix, OFF_PFD)? == 100))
        || (prefix.len() >= 108
            && hdr_size >= 108
            && n_bytes >= 128
            && ((hdr_size == 108 && le32(prefix, N_DESC)? == 0 && le32(prefix, OFF_PFD)? == 0)
                || off_desc == 108
                || le32(prefix, OFF_PFD)? == 108));
    require!(valid_layout);
    Some(Label::Emf)
}

/// OpenStreetMap PBF binary stream: 4-byte big-endian `BlobHeader` length with zero high half-word
/// at `@0..2`, followed by protobuf field 1 (`0x0A`, length 9, `"OSMHeader"`) at `@4..15`.
fn osm_pbf(prefix: &[u8]) -> Option<Label> {
    const LEN_HI: &[u8] = &[0x00, 0x00]; // @0..2: high 16 bits of BlobHeader length (0)
    const BLOB_TYPE_OFFSET: usize = 4; // @4..15: protobuf tag 0x0A, length 9, "OSMHeader"
    const BLOB_TYPE: &[u8] = b"\x0a\x09OSMHeader";
    const MIN_LEN: usize = 16;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, LEN_HI),
        eq_at(prefix, BLOB_TYPE_OFFSET, BLOB_TYPE),
    );
    Some(Label::Osm)
}

// =============================================================================
// Tier 4: Structural and text heuristics
// =============================================================================

/// Flags recorded in a single `memchr('<')` pass over the first 257 bytes for the four
/// floating-XML root-element rules (`collada`, `gpx`, `kml`, `osm`).
#[derive(Clone, Copy, Default)]
struct XmlRoots {
    collada: bool,
    gpx: bool,
    kml: bool,
    osm: bool,
}

/// Scans `0..=256` once for `<COLLADA`, `<gpx[ \t\r\n]`, `<kml[ \t\r\n>]`, and `<osm[ \t\r\n]`.
///
/// Precondition made explicit (relative to `.yar` text for `gpx`, `kml`, `osm`): a well-formed
/// XML document prelude (UTF-8 BOM, `<?xml` declaration, comments, processing instructions, or
/// XML whitespace) always begins with `<`, the UTF-8 BOM lead `0xEF`, or XML whitespace (`' '`,
/// `'\t'`, `'\r'`, `'\n'`), so non-XML inputs are skipped immediately without sweeping 257 bytes.
#[inline]
fn xml_roots(prefix: &[u8]) -> XmlRoots {
    const MAX_ROOT_OFFSET: usize = 256;

    let Some(&first) = prefix.first() else {
        return XmlRoots::default();
    };
    if !matches!(first, b'<' | 0xEF | b' ' | b'\t' | b'\r' | b'\n') {
        return XmlRoots::default();
    }
    let has_xml_decl = starts(prefix, b"<?xml");
    let mut roots = XmlRoots::default();
    let window = &prefix[..prefix.len().min(MAX_ROOT_OFFSET + 1)];
    for pos in memchr::memchr_iter(b'<', window) {
        match prefix.get(pos + 1) {
            Some(b'C') if has_xml_decl && eq_at(prefix, pos, b"<COLLADA") => {
                roots.collada = true;
            }
            Some(b'g')
                if eq_at(prefix, pos, b"<gpx")
                    && matches!(u8_at(prefix, pos + 4), Some(b' ' | b'\t' | b'\r' | b'\n')) =>
            {
                roots.gpx = true;
            }
            Some(b'k')
                if eq_at(prefix, pos, b"<kml")
                    && matches!(
                        u8_at(prefix, pos + 4),
                        Some(b' ' | b'\t' | b'\r' | b'\n' | b'>')
                    ) =>
            {
                roots.kml = true;
            }
            Some(b'o')
                if eq_at(prefix, pos, b"<osm")
                    && matches!(u8_at(prefix, pos + 4), Some(b' ' | b'\t' | b'\r' | b'\n')) =>
            {
                roots.osm = true;
            }
            _ => {}
        }
    }
    roots
}

/// Khronos glTF 2.0 JSON asset: anchored at `@0`, either opens with `"accessors": [` and
/// `"componentType"` in `0..=1024`, or opens with a flat `"asset": { ... }` object followed by a
/// glTF top-level property or a `KHR_`/`EXT_` `"extensionsUsed"` list.
fn gltf_json(prefix: &[u8]) -> Option<Label> {
    #[inline]
    fn is_ws(b: u8) -> bool {
        matches!(b, b' ' | b'\t' | b'\r' | b'\n')
    }

    #[inline]
    fn skip_ws(s: &[u8], i: &mut usize, max: usize) -> bool {
        let start = *i;
        let limit = s.len().min(start + max + 1);
        while *i < limit && is_ws(s[*i]) {
            *i += 1;
        }
        *i - start <= max
    }

    let first = *prefix.first()?;
    require!(is_ws(first) || first == b'{');

    let mut i = 0;
    require!(skip_ws(prefix, &mut i, 64), u8_at(prefix, i) == Some(b'{'),);
    i += 1;
    require!(skip_ws(prefix, &mut i, 64));

    if eq_at(prefix, i, b"\"accessors\"") {
        i += b"\"accessors\"".len();
        require!(skip_ws(prefix, &mut i, 16), u8_at(prefix, i) == Some(b':'),);
        i += 1;
        require!(
            skip_ws(prefix, &mut i, 16),
            u8_at(prefix, i) == Some(b'['),
            scan_range(prefix, 0, 1024, b"\"componentType\""),
        );
        return Some(Label::Gltf);
    }

    require!(eq_at(prefix, i, b"\"asset\""));
    i += b"\"asset\"".len();
    require!(skip_ws(prefix, &mut i, 16), u8_at(prefix, i) == Some(b':'),);
    i += 1;
    require!(skip_ws(prefix, &mut i, 16), u8_at(prefix, i) == Some(b'{'),);
    i += 1;
    let body_start = i;
    while i < prefix.len() && prefix[i] != b'{' && prefix[i] != b'}' {
        i += 1;
    }
    require!(i - body_start <= 512, u8_at(prefix, i) == Some(b'}'),);
    i += 1;
    require!(skip_ws(prefix, &mut i, 16), u8_at(prefix, i) == Some(b','),);
    i += 1;
    require!(skip_ws(prefix, &mut i, 16));

    const GLTF_KEYS: &[&[u8]] = &[
        b"\"scene\"",
        b"\"scenes\"",
        b"\"nodes\"",
        b"\"accessors\"",
        b"\"bufferViews\"",
        b"\"buffers\"",
        b"\"meshes\"",
    ];
    if GLTF_KEYS.iter().any(|k| eq_at(prefix, i, k)) {
        return Some(Label::Gltf);
    }

    require!(eq_at(prefix, i, b"\"extensionsUsed\""));
    i += b"\"extensionsUsed\"".len();
    require!(skip_ws(prefix, &mut i, 16), u8_at(prefix, i) == Some(b':'),);
    i += 1;
    require!(skip_ws(prefix, &mut i, 16), u8_at(prefix, i) == Some(b'['),);
    i += 1;
    require!(
        skip_ws(prefix, &mut i, 64),
        eq_at(prefix, i, b"\"KHR_") || eq_at(prefix, i, b"\"EXT_"),
    );
    Some(Label::Gltf)
}

/// Hugging Face `safetensors` model file: 64-bit little-endian header length `< 4 GiB` (high
/// dword `0` at `@4`), JSON object opening `{"` at `@8`, and `"\"dtype\""` within `@10..=512`.
fn safetensors(prefix: &[u8]) -> Option<Label> {
    const LEN_HI: usize = 4; // @4: u32 LE — high 32 bits of 64-bit JSON header length (0)
    const JSON_OPEN: usize = 8; // @8..10: "{\"" opening of JSON header
    const DTYPE_MIN: usize = 10;
    const DTYPE_MAX: usize = 512;
    const MIN_LEN: usize = 16;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, JSON_OPEN, b"{\""),
        le32(prefix, LEN_HI)? == 0,
        scan_range(prefix, DTYPE_MIN, DTYPE_MAX, b"\"dtype\""),
    );
    Some(Label::Safetensors)
}

/// MPEG-2 Transport Stream (`.ts` or Blu-ray `.m2ts`): 21 consecutive 188-byte packets starting
/// at `@0` (prefix ≥ 3948) or 192-byte packets with 4-byte timecode starting at `@4` (prefix ≥
/// 4032), each with sync byte `0x47`, `transport_error_indicator == 0`, and nonzero
/// `adaptation_field_control`.
fn mpegts(prefix: &[u8]) -> Option<Label> {
    const SYNC: u8 = 0x47; // MPEG-TS sync byte
    const PACKETS: usize = 21;
    const TS_STRIDE: usize = 188;
    const M2TS_STRIDE: usize = 192;
    const TS_MIN_LEN: usize = 3948; // 21 * 188
    const M2TS_MIN_LEN: usize = 4032; // 21 * 192

    #[inline]
    fn valid_packets(prefix: &[u8], start: usize, stride: usize) -> bool {
        for k in 0..PACKETS {
            let p = start + k * stride;
            if prefix[p] != SYNC || prefix[p + 1] & 0x80 != 0 || prefix[p + 3] & 0x30 == 0 {
                return false;
            }
        }
        true
    }

    if prefix.len() >= TS_MIN_LEN && prefix[0] == SYNC && valid_packets(prefix, 0, TS_STRIDE) {
        return Some(Label::Mpegts);
    }
    if prefix.len() >= M2TS_MIN_LEN && prefix[4] == SYNC && valid_packets(prefix, 4, M2TS_STRIDE) {
        return Some(Label::Mpegts);
    }
    None
}

/// dBASE database table (`.dbf`): either a dBASE III/IV/V header (≥ 64 bytes, date in `@2..4`,
/// header size ≥ 64 at `@8`, record size ≥ 1 at `@10`, ASCII letter field name at `@32`, and
/// valid field type at `@43`) or a fixed-header dBASE II table (file size ≥ 521, `0x02` at `@0`,
/// record size `1..=0x03FF` at `@6`, field name at `@8`, type `'C'|'L'|'N'` at `@19`, length ≥ 1
/// at `@20`).
fn dbase(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    const VERSION: usize = 0; // @0: u8 — dBASE version byte
    let v0 = *prefix.first()?;

    if v0 == 0x02 {
        // dBASE II: fixed 521-byte header
        const DB2_MIN_PREFIX: usize = 24;
        const DB2_MIN_FILE_SIZE: u64 = 521;
        require!(
            prefix.len() >= DB2_MIN_PREFIX,
            ctx.size >= DB2_MIN_FILE_SIZE,
            (prefix[3] == 0 && prefix[4] == 0 && prefix[5] == 0)
                || ((1..=12).contains(&prefix[3]) && (1..=31).contains(&prefix[4])),
            le16(prefix, 6)? >= 1,
            prefix[7] <= 3,
            prefix[8].is_ascii_alphabetic(),
            matches!(prefix[19], b'C' | b'L' | b'N'),
            prefix[20] >= 1,
        );
        return Some(Label::Dbase);
    }

    const DB3_MIN_PREFIX: usize = 64;
    require!(
        prefix.len() >= DB3_MIN_PREFIX,
        matches!(v0, 0x03 | 0x04 | 0x43 | 0x63 | 0x7B | 0x83 | 0x8B | 0x8E | 0xCB),
        (1..=12).contains(&prefix[2]),
        (1..=31).contains(&prefix[3]),
        le16(prefix, 8)? >= 64,
        le16(prefix, 10)? >= 1,
        prefix[32].is_ascii_alphabetic(),
    );
    let field_type = prefix[43];
    let valid_type = match v0 {
        0x8B | 0xCB | 0x8E | 0x7B => {
            matches!(field_type, b'C' | b'D' | b'F' | b'L' | b'M' | b'N')
        }
        0x04 | 0x43 | 0x63 => matches!(field_type, b'C' | b'D' | b'F' | b'L' | b'N'),
        0x03 | 0x83 => matches!(field_type, b'C' | b'D' | b'L' | b'M' | b'N'),
        _ => false,
    };
    require!(valid_type);
    let _ = VERSION;
    Some(Label::Dbase)
}

/// Atari ST DEGAS / DEGAS Elite uncompressed screen dump (`.pi1`/`.pi2`/`.pi3`): resolution word
/// `0..=2` BE at `@0` and exact file size `32034` (raw) or `32066` (with animation footer).
fn degas(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    const RESOLUTION: usize = 0; // @0: u16 BE — 0 (low), 1 (medium), 2 (high)
    const MIN_LEN: usize = 34; // 2-byte resolution + 32-byte palette
    const RAW_SIZE: u64 = 32034;
    const ANIM_SIZE: u64 = 32066;

    require!(
        prefix.len() >= MIN_LEN,
        matches!(ctx.size, RAW_SIZE | ANIM_SIZE),
        be16(prefix, RESOLUTION)? <= 2,
    );
    Some(Label::Degas)
}

/// Intel HEX object file: two consecutive leading records `:[0-9A-F]{6}0[0-5][0-9A-F]{2,512}\r?\n`
/// and `:[0-9A-F]{6}0[0-5]`.
fn intelhex(prefix: &[u8]) -> Option<Label> {
    #[inline]
    fn is_upper_hex(b: u8) -> bool {
        matches!(b, b'0'..=b'9' | b'A'..=b'F')
    }

    #[inline]
    fn record_head(s: &[u8], at: usize) -> bool {
        s.len() >= at + 9
            && s[at] == b':'
            && s[at + 1..at + 7].iter().copied().all(is_upper_hex)
            && s[at + 7] == b'0'
            && matches!(s[at + 8], b'0'..=b'5')
    }

    require!(record_head(prefix, 0));
    let mut i = 9;
    while i < prefix.len() && is_upper_hex(prefix[i]) {
        i += 1;
    }
    let hex_digits = i - 9;
    require!((2..=512).contains(&hex_digits));
    if u8_at(prefix, i) == Some(b'\r') {
        i += 1;
    }
    require!(u8_at(prefix, i) == Some(b'\n'));
    i += 1;
    require!(record_head(prefix, i));
    Some(Label::Intelhex)
}

/// CPython bytecode (`.pyc`): one of the 89 historical magic values at `@0..4` paired with the
/// marshal `TYPE_CODE` tag (`'C'` at `@8` for Python 1.0..1.2, `'c'` at `@8` for 1.5..3.2, `'c'`
/// at `@12` for 3.3..3.3a, or `'c'`/`0xE3` at `@12` for 3.4..3.6).
fn pythonbytecode(prefix: &[u8]) -> Option<Label> {
    const MIN_LEN: usize = 9;

    require!(prefix.len() >= MIN_LEN);
    // All 89 magics end in either 0x99 0x00 (Python 1.0..1.2) or 0x0D 0x0A ("\r\n").
    let marker = (prefix[2], prefix[3]);
    if marker == (0x99, 0x00) {
        require!(prefix[1] == 0x09, matches!(prefix[0], 0x02 | 0x03), u8_at(prefix, 8)? == 0x43,);
        return Some(Label::Pythonbytecode);
    }
    require!(marker == (0x0D, 0x0A));
    let ver = le16(prefix, 0)?;

    // 59 timestamp-only magics (marshal 'c' = 0x63 at @8)
    const TIMESTAMP_VERSIONS: &[u16] = &[
        0xF303, 0x1704, 0xF304, 0x0C09, 0xF30A, 0x0C13, 0x0C1D, 0x0C1F, 0x0C27, 0xEB2A, 0xEB2B,
        0xED2D, 0xED2E, 0x0C3B, 0xF23B, 0xF23C, 0x0C45, 0xF245, 0x0C4F, 0x0C58, 0xF259, 0x0C62,
        0xF263, 0x0C6C, 0xF26D, 0xF26E, 0x0C76, 0xF277, 0x0C80, 0xF281, 0xC687, 0xC688, 0x2E89,
        0xF28B, 0xF28C, 0xF295, 0x4E99, 0xF29F, 0xF2A9, 0xF2B3, 0xF2B4, 0x0BB8, 0x0BC2, 0xF2C7,
        0x0BCC, 0xF2D1, 0xF2D2, 0x0BD6, 0xF2DB, 0x0BE0, 0xF2E5, 0x0BEA, 0xF2EF, 0x0BF4, 0x0BF5,
        0xF2F9, 0xC4FC, 0xC4FD, 0x0BFF,
    ];
    if u8_at(prefix, 8) == Some(0x63) && TIMESTAMP_VERSIONS.contains(&ver) {
        return Some(Label::Pythonbytecode);
    }

    // 3 sized-header magics (marshal 'c' = 0x63 at @12)
    if matches!(ver, 0x0C8A | 0x0C94 | 0x0C9E) && u8_at(prefix, 12) == Some(0x63) {
        return Some(Label::Pythonbytecode);
    }

    // 25 reference-enabled magics (marshal 'c' = 0x63 or FLAG_REF|'c' = 0xE3 at @12)
    const REFERENCE_VERSIONS: &[u16] = &[
        0x0D02, 0x0D0C, 0x0D16, 0x0D17, 0x0D20, 0x0D21, 0x0D2A, 0x0D2B, 0x0D2C, 0x0D2D, 0x0D2F,
        0x0D30, 0x0D31, 0x0D32, 0x0D33, 0x0D3E, 0x0D3F, 0x0CB2, 0x0CBC, 0x0CC6, 0x0CD0, 0x0CDA,
        0x0CE4, 0x0CEE, 0x0CF8,
    ];
    require!(matches!(u8_at(prefix, 12), Some(0x63 | 0xE3)), REFERENCE_VERSIONS.contains(&ver),);
    Some(Label::Pythonbytecode)
}

/// BitTorrent metainfo file (`.torrent`, BEP 3/12/52): bencoded dictionary opening with a known
/// root key (`announce-list`, `info`, `announce`, or `comment`), corroborated either by a typed
/// tracker URL (`http`/`https`/`udp`/`ws`/`wss`) at `@0` or by `4:infod`, `12:piece lengthi...e`,
/// and `4:name...:` within the prefix.
fn torrent(prefix: &[u8]) -> Option<Label> {
    const MIN_LEN: usize = 20;

    /// Checks `([7-9]|[1-9][0-9]{1,3}):(https?|udp|wss?):\/\/[\x21-\xff]` at the start of `s`.
    fn has_tracker_url(s: &[u8]) -> bool {
        let Some(&first_digit) = s.first() else {
            return false;
        };
        if !matches!(first_digit, b'1'..=b'9') {
            return false;
        }
        let mut d = 1;
        while d < s.len() && s[d].is_ascii_digit() {
            d += 1;
        }
        let valid_len = (d == 1 && first_digit >= b'7') || (2..=4).contains(&d);
        if !valid_len || s.get(d) != Some(&b':') {
            return false;
        }
        let url = &s[d + 1..];
        const SCHEMES: &[&[u8]] = &[b"http://", b"https://", b"udp://", b"ws://", b"wss://"];
        for scheme in SCHEMES {
            if url.starts_with(scheme) {
                return matches!(url.get(scheme.len()), Some(0x21..=0xFF));
            }
        }
        false
    }

    /// Checks `12:piece lengthi[1-9][0-9]{0,18}e` starting in `0..=4060`.
    fn has_piece_length(prefix: &[u8]) -> bool {
        const TAG: &[u8] = b"12:piece lengthi";
        for pos in memchr::memmem::find_iter(prefix, TAG) {
            if pos > 4060 {
                break;
            }
            let s = &prefix[pos + TAG.len()..];
            let Some(&first) = s.first() else {
                continue;
            };
            if !matches!(first, b'1'..=b'9') {
                continue;
            }
            let mut d = 1;
            while d < s.len() && s[d].is_ascii_digit() {
                d += 1;
            }
            if d <= 19 && s.get(d) == Some(&b'e') {
                return true;
            }
        }
        false
    }

    /// Checks `4:name[1-9][0-9]{0,3}:` starting in `0..=4085`.
    fn has_name(prefix: &[u8]) -> bool {
        const TAG: &[u8] = b"4:name";
        for pos in memchr::memmem::find_iter(prefix, TAG) {
            if pos > 4085 {
                break;
            }
            let s = &prefix[pos + TAG.len()..];
            let Some(&first) = s.first() else {
                continue;
            };
            if !matches!(first, b'1'..=b'9') {
                continue;
            }
            let mut d = 1;
            while d < s.len() && s[d].is_ascii_digit() {
                d += 1;
            }
            if d <= 4 && s.get(d) == Some(&b':') {
                return true;
            }
        }
        false
    }

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, b"d13:announce-list")
            || starts(prefix, b"d4:info")
            || starts(prefix, b"d8:announce")
            || starts(prefix, b"d7:comment"),
    );
    let valid = (starts(prefix, b"d8:announce") && has_tracker_url(&prefix[11..]))
        || (starts(prefix, b"d13:announce-listll") && has_tracker_url(&prefix[19..]))
        || (scan_range(prefix, 0, 4089, b"4:infod")
            && has_piece_length(prefix)
            && has_name(prefix));
    require!(valid);
    Some(Label::Torrent)
}

// =============================================================================
// Facts-based (Deep) verifiers: ZIP and PE
// =============================================================================

/// Android APK local-header detection in the 4 KiB prefix (`taxonomy_apk`): unencrypted stored or
/// deflated first entry with nonzero sizes or data-descriptor flag, opening either with
/// `classes.dex` plus a local `AndroidManifest.xml` entry in `41..=4047` or with
/// `AndroidManifest.xml` plus a local `classes.dex` (`49..=4055`) or `resources.arsc`
/// (`49..=4052`) entry.
fn apk_prefix(prefix: &[u8]) -> Option<Label> {
    const LOCAL_SIG: &[u8] = b"PK\x03\x04"; // @0..4: ZIP local file header signature
    const FLAGS: usize = 6; // @6: u16 LE — general purpose bit flag (bit 0 = encrypted, bit 3 = data descriptor)
    const METHOD: usize = 8; // @8: u16 LE — compression method (0 = stored, 8 = deflated)
    const COMP_SIZE: usize = 18; // @18: u32 LE — compressed size
    const UNCOMP_SIZE: usize = 22; // @22: u32 LE — uncompressed size
    const NAME_LEN: usize = 26; // @26: u16 LE — file name length
    const NAME_OFFSET: usize = 30; // @30: file name bytes
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
                && eq_at(prefix, hdr_pos, LOCAL_SIG)
                && le16(prefix, hdr_pos + NAME_LEN) == Some(name_len)
            {
                return true;
            }
        }
        false
    }

    require!(prefix.len() >= MIN_LEN, starts(prefix, LOCAL_SIG));
    let flags = le16(prefix, FLAGS)?;
    let method = le16(prefix, METHOD)?;
    require!(
        flags % 2 == 0,
        matches!(method, 0 | 8),
        (le32(prefix, COMP_SIZE)? >= 1 && le32(prefix, UNCOMP_SIZE)? >= 1) || (flags & 0x08 != 0),
    );
    let name_len = le16(prefix, NAME_LEN)?;
    let valid = (name_len == 11
        && eq_at(prefix, NAME_OFFSET, b"classes.dex")
        && has_local_entry(prefix, 41, 4047, b"AndroidManifest.xml"))
        || (prefix.len() >= 49
            && name_len == 19
            && eq_at(prefix, NAME_OFFSET, b"AndroidManifest.xml")
            && (has_local_entry(prefix, 49, 4055, b"classes.dex")
                || has_local_entry(prefix, 49, 4052, b"resources.arsc")));
    require!(valid);
    Some(Label::Apk)
}

/// EPUB OCF local-header detection in the 4 KiB prefix (`taxonomy_epub`): unencrypted stored
/// first entry `"mimetype"` (`name_len == 8`, `extra_len == 0`) with 20-byte payload
/// `"application/epub+zip"` (either declared in the local header or deferred to a data descriptor
/// at `@58`).
fn epub_prefix(prefix: &[u8]) -> Option<Label> {
    const LOCAL_SIG: &[u8] = b"PK\x03\x04"; // @0..4: ZIP local file header signature
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
        starts(prefix, LOCAL_SIG),
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
/// `names` and `first_entry` views from [`Facts`].
fn zip(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    if let Some(label) = apk_prefix(prefix) {
        return Some(label);
    }
    if let Some(label) = epub_prefix(prefix) {
        return Some(label);
    }

    let facts = Facts::analyze(prefix, ctx.size, ctx.tail);
    let names = facts.names();
    let first_entry = facts.first_entry();
    let has_line = |line: &[u8]| memchr::memmem::find(names, line).is_some();
    let valid = facts.is_zip_valid();
    let dir_names = facts.has_zip_dir_names();

    // Full rules
    if valid {
        if has_line(b"\nmodel.weights.h5\n") && has_line(b"\nmetadata.json\n") {
            return Some(Label::Keras);
        }
        if memchr::memmem::find(names, b".qgs\n").is_some() {
            return Some(Label::Qgis);
        }
        if has_line(b"\nvisio/document.xml\n") {
            return Some(Label::Visio);
        }
        if dir_names && has_line(b"\n[Content_Types].xml\n") && has_line(b"\nxl/workbook.xml\n") {
            return Some(Label::Xlsx);
        }
        if has_line(b"\n3D/3dmodel.model\n") {
            return Some(Label::_3mf);
        }
    }

    // Partial central-directory rules
    if dir_names
        && has_line(b"\nAndroidManifest.xml\n")
        && (has_line(b"\nclasses.dex\n") || has_line(b"\nresources.arsc\n"))
    {
        return Some(Label::Apk);
    }
    if starts(names, b"\nmimetype=application/epub+zip\n")
        && has_line(b"\nMETA-INF/container.xml\n")
    {
        return Some(Label::Epub);
    }
    if starts(names, b"\nmimetype=application/vnd.oasis.opendocument.presentation\n")
        && has_line(b"\ncontent.xml\n")
    {
        return Some(Label::Odp);
    }
    if starts(names, b"\nmimetype=application/vnd.oasis.opendocument.spreadsheet\n")
        && has_line(b"\ncontent.xml\n")
    {
        return Some(Label::Ods);
    }
    if starts(names, b"\nmimetype=application/vnd.oasis.opendocument.text\n")
        && has_line(b"\ncontent.xml\n")
    {
        return Some(Label::Odt);
    }
    if dir_names
        && has_line(b"\nMETA-INF/MANIFEST.MF\n")
        && memchr::memmem::find(names, b".class\n").is_some()
    {
        return Some(Label::Jar);
    }
    if valid && has_line(b"\ndoc.kml\n") {
        return Some(Label::Kmz);
    }
    if valid && has_line(b"\nAppxManifest.xml\n") && has_line(b"\nAppxBlockMap.xml\n") {
        return Some(Label::Msix);
    }
    if dir_names && has_line(b"\n[Content_Types].xml\n") && has_line(b"\nppt/presentation.xml\n") {
        return Some(Label::Pptx);
    }

    // First-entry rules
    if starts(first_entry, b"[Content_Types].xml\n") {
        if memchr::memmem::find(
            first_entry,
            b"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
        )
        .is_some()
            || memchr::memmem::find(
                first_entry,
                b"application/vnd.ms-word.document.macroEnabled.main+xml",
            )
            .is_some()
        {
            return Some(Label::Docx);
        }
        if memchr::memmem::find(
            first_entry,
            b"application/vnd.openxmlformats-officedocument.wordprocessingml.template.main+xml",
        )
        .is_some()
            || memchr::memmem::find(
                first_entry,
                b"application/vnd.ms-word.template.macroEnabledTemplate.main+xml",
            )
            .is_some()
        {
            return Some(Label::Dotx);
        }
        if memchr::memmem::find(
            first_entry,
            b"application/vnd.ms-excel.sheet.binary.macroEnabled.main",
        )
        .is_some()
        {
            return Some(Label::Xlsb);
        }
    }
    None
}

/// Windows PE/COFF executable (`taxonomy_pebin`): valid PE image with `IMAGE_FILE_EXECUTABLE_IMAGE`
/// set, not a resource-only DLL (`is_dll && entry_point == 0`), Windows subsystem in `1..=16`,
/// and a shipping Windows `Machine` architecture.
fn pe(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    require!(Facts::analyze(prefix, ctx.size, None).is_windows_pebin());
    Some(Label::Pebin)
}
