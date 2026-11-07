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

//! Cross-category magic-key collision disambiguators.

use super::super::*;
use crate::ContentType as Label;

/// Evaluates `0xFF`-prefixed streams (`jpeg`, `jp2`, `jxl`, `mp3`).
pub(crate) fn stream_ff(prefix: &[u8], size: u64) -> Option<Label> {
    const JPEG_SOI_LOW: u8 = 0xD8;
    const J2K_SOC_LOW: u8 = 0x4F;
    const JXL_SIG_LOW: u8 = 0x0A;

    require!(size >= 8, prefix.len() >= 8, u8_at(prefix, 0)? == 0xFF);
    let b1 = u8_at(prefix, 1)?;
    if b1 == JPEG_SOI_LOW {
        return jpeg(prefix, size);
    }
    if b1 == J2K_SOC_LOW
        && size >= 16
        && eq_at(prefix, 0, &[0xFF, 0x4F, 0xFF, 0x51])
        && be16(prefix, 4).unwrap_or(0) >= 38
    {
        return Some(Label::Jp2);
    }
    if b1 == JXL_SIG_LOW {
        return verify_jxl_codestream(prefix, size);
    }
    if verify_mpeg_audio_layer2(prefix, 0) {
        return Some(Label::Mp2);
    }
    if verify_mpeg_audio_frames(prefix, 0) {
        return Some(Label::Mp3);
    }
    None
}

/// Disambiguates `CA FE BA BE` at `@0` between Apple Mach-O Universal Fat Binary (`macho`)
/// and Sun / Oracle JVM `.class` bytecode (`javabytecode`, 1995).
pub(crate) fn cafebabe(prefix: &[u8], size: u64) -> Option<Label> {
    const CAFEBABE_MAGIC: &[u8] = &[0xCA, 0xFE, 0xBA, 0xBE];
    require!(size >= 24, prefix.len() >= 10, starts(prefix, CAFEBABE_MAGIC));
    let nfat_or_minor_major = be32(prefix, 4)?;
    // Mach-O Universal Fat binary: nfat_arch in 1..=32 and valid Mach-O cpu_type at @8
    if (1..=32).contains(&nfat_or_minor_major) && prefix.len() >= 28 {
        let cpu_type = be32(prefix, 8)?;
        if matches!(
            cpu_type & !0x0100_0000,
            7 | 12 | 18 // CPU_TYPE_X86 (7), CPU_TYPE_ARM (12), CPU_TYPE_POWERPC (18)
        ) {
            return Some(Label::Macho);
        }
    }
    // Java .class file: major_version in 45..=80 at @6, constant_pool_count >= 1 at @8
    let major = be16(prefix, 6)?;
    let cp_count = be16(prefix, 8)?;
    require!((45..=80).contains(&major), cp_count >= 1);
    Some(Label::Javabytecode)
}

/// Disambiguates `00 00` between Atari ST DEGAS low-res image (`degas`) and COFF `/bigobj` (`coff`).
#[inline]
pub(crate) fn degas_or_coff(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    probe(|| degas(prefix, ctx)).or_else(|| probe(|| coff(prefix, ctx.size)))
}

/// Disambiguates `0x80`/`0x82` between Intel 8086 Relocatable OMF (`omf`) and MessagePack (`msgpack`).
#[inline]
pub(crate) fn omf_or_msgpack(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| omf_intel(prefix, size)).or_else(|| probe(|| msgpack(prefix, size)))
}

/// Disambiguates `00 01 00 00` between TrueType 1.0 `sfnt` font (`ttf`) and Microsoft Access Jet/ACE DB (`access`).
#[inline]
pub(crate) fn ttf_or_access(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| ttf(prefix, size)).or_else(|| probe(|| access(prefix, size)))
}

/// Disambiguates x86 MBR & El Torito IsoHybrid sector-0 prologues between ISO 9660 / UDF (`iso` / `udf`),
/// Master Boot Record (`dosmbr`), and MS-DOS `.COM` (`pebin`).
#[inline]
pub(crate) fn boot_or_isohybrid(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| iso(prefix, size))
        .or_else(|| probe(|| dosmbr(prefix, size)))
        .or_else(|| probe(|| dos_com(prefix, size)))
}

/// Disambiguates `00 00 00 00` at `@0` across formats whose header begins with a 32-bit zero word or null preamble.
pub(crate) fn null_prefixed(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    let size = ctx.size;
    probe(|| sas(prefix))
        .or_else(|| probe(|| duckdb(prefix)))
        .or_else(|| probe(|| mpegts(prefix)))
        .or_else(|| probe(|| stl_binary(prefix, size)))
        .or_else(|| probe(|| dicom(prefix)))
        .or_else(|| probe(|| iso(prefix, size)))
        .or_else(|| probe(|| dosmbr(prefix, size)))
        .or_else(|| probe(|| hfs(prefix, size)))
        .or_else(|| probe(|| pcd_kodak(prefix, size)))
}

/// Disambiguates `00 00 00 01` at `@0` between macOS `.DS_Store` (`dsstore`) and 64-bit ISO-BMFF boxes (`isobmff`).
#[inline]
pub(crate) fn dsstore_or_isobmff(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| dsstore(prefix)).or_else(|| probe(|| isobmff(prefix, size)))
}

/// Disambiguates `00 00 02 00` at `@0` between Windows Cursor (`ico`) and uncompressed true-color TGA (`tga`).
#[inline]
pub(crate) fn ico_or_tga(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    probe(|| ico(prefix)).or_else(|| probe(|| tga(prefix, ctx.size, ctx.tail)))
}

/// Disambiguates `b"----"` at `@0` between PEM / OpenPGP ASCII armor and Shapr3D Binary STL.
#[inline]
pub(crate) fn pem_or_stl(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| pem_or_pgp(prefix)).or_else(|| probe(|| stl_binary(prefix, size)))
}

/// Disambiguates `b"Auto"` at `@0` between AutoCAD Binary DXF (`dxf`) and AutoCAD Binary STL (`stlbinary`).
#[inline]
pub(crate) fn dxf_or_stl(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| dxf_binary(prefix, size)).or_else(|| probe(|| stl_binary(prefix, size)))
}
