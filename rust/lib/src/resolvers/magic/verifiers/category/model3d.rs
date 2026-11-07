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

//! 3D CAD, scene, mesh, and point-cloud format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

const WAVEFRONT_COMMENT: &[u8] = b"# ";

/// Blender 3D scene file (`blend`, 1994): `"BLENDER"` magic, pointer size (`_` or `-`),
/// endianness indicator (`v` or `V`), and three-digit version `100`..`499`.
pub(crate) fn blend(prefix: &[u8]) -> Option<Label> {
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
        u8_at(prefix, 19)? == 0,
    );
    Some(Label::Blend)
}

/// AutoCAD DWG drawing: `"AC10"` version prefix with known release code (`09`, `12`, `14`, `15`,
/// `18`, `21`, `24`, `27`, `32`), five zero bytes at `@6..11`, and ≥ 128-byte header.
pub(crate) fn dwg(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"AC10"; // @0..4: AutoCAD DWG version prefix
    const VERSIONS: &[&[u8]] =
        &[b"09", b"12", b"14", b"15", b"18", b"21", b"24", b"27", b"32"];
    const ZERO_PAD: usize = 6; // @6..11: 5 reserved zero bytes
    const MIN_LEN: usize = 128;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        VERSIONS.contains(&get_slice(prefix, 4, 6)?),
        eq_at(prefix, ZERO_PAD, &[0; 5]),
    );
    Some(Label::Dwg)
}

/// Autodesk FBX: binary `"Kaydara FBX Binary  \x00\x1a\x00"` with version `3000..=7700` at `@23`,
/// or ASCII export header comment `"; FBX X.Y.Z project file"`.
pub(crate) fn fbx(prefix: &[u8]) -> Option<Label> {
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

/// Khronos binary glTF 2.0 (GLB): `"glTF"` + version 2, 4-byte-aligned total and chunk lengths,
/// `"JSON"` first chunk type at `@16`, and JSON object opening `{` after optional whitespace at `@20`.
pub(crate) fn glb(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(b"glTF", &2u32.to_le_bytes()); // @0..8: glTF magic + version 2
    const TOTAL_LEN: usize = 8; // @8: u32 LE — total byte length (≥ 24, % 4 == 0)
    const CHUNK_LEN: usize = 12; // @12: u32 LE — JSON chunk length (≥ 4, % 4 == 0)
    const CHUNK_TYPE: usize = 16; // @16..20: "JSON" chunk type
    const JSON_CHUNK: &[u8] = b"JSON";
    const JSON_DATA: usize = 20; // @20: start of JSON payload
    const MIN_LEN: usize = 24;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, HEADER),
        le32(prefix, TOTAL_LEN)? >= MIN_LEN as u32,
        le32(prefix, TOTAL_LEN)? % 4 == 0,
        le32(prefix, CHUNK_LEN)? >= 4,
        le32(prefix, CHUNK_LEN)? % 4 == 0,
        eq_at(prefix, CHUNK_TYPE, JSON_CHUNK),
    );
    let ws = prefix[JSON_DATA..]
        .iter()
        .take(65)
        .take_while(|&&b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        .count();
    require!(ws <= 64, u8_at(prefix, JSON_DATA + ws)? == b'{');
    Some(Label::Gltf)
}

/// Polygon File Format (PLY): `"ply"` line followed by `"format <encoding> 1.0"` and whitespace.
pub(crate) fn ply(prefix: &[u8]) -> Option<Label> {
    const HEADER_LF: &[u8] = b"ply\nformat ";
    const HEADER_CRLF: &[u8] = b"ply\r\nformat ";
    const ENCODINGS: &[&[u8]] =
        &[b"ascii 1.0", b"binary_little_endian 1.0", b"binary_big_endian 1.0"];

    let rest = prefix.strip_prefix(HEADER_LF).or_else(|| prefix.strip_prefix(HEADER_CRLF))?;
    for &enc in ENCODINGS {
        if let Some(tail) = rest.strip_prefix(enc)
            && matches!(tail.first(), Some(b' ' | b'\t' | b'\r' | b'\n'))
        {
            return Some(Label::Ply);
        }
    }
    None
}

/// SketchUp model (`.skp`): UTF-16LE BOM + `"SketchUp Model"` (32 bytes) or legacy length-prefixed
/// ASCII `"\x0eSketchUp Model\x08"` (16 bytes).
pub(crate) fn sketchup(prefix: &[u8]) -> Option<Label> {
    const LEGACY: &[u8] = b"\x0eSketchUp Model\x08"; // @0..16: legacy ASCII SketchUp header
    const UNICODE: &[u8] = &[
        0xFF, 0xFE, 0xFF, 0x0E, b'S', 0, b'k', 0, b'e', 0, b't', 0, b'c', 0, b'h', 0, b'U', 0,
        b'p', 0, b' ', 0, b'M', 0, b'o', 0, b'd', 0, b'e', 0, b'l', 0,
    ]; // @0..32: UTF-16LE SketchUp header

    let matched = starts(prefix, LEGACY) || starts(prefix, UNICODE);
    require!(matched);
    Some(Label::Sketchup)
}

/// ISO 10303-21 STEP product data exchange file: opens with `"ISO-10303-21;"`.
pub(crate) fn step(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"ISO-10303-21;"; // @0..13: STEP Part 21 header keyword

    require!(starts(prefix, MAGIC));
    Some(Label::Step)
}

/// Autodesk 3ds Max (`.3ds`): root chunk `M3DMAGIC` (`0x4D4D` = `"MM"`) at `@0` with length ≥ 16
/// at `@2`, followed at `@6` by either `M3D_VERSION` (`0x0002`, length 10, version 3..=4 at `@12`)
/// or `MDATA` (`0x3D3D` = `"=="`, length ≥ 6 at `@8`).
pub(crate) fn three_dsm(prefix: &[u8]) -> Option<Label> {
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
        (MIN_LEN as u32..0x0800_0000).contains(&le32(prefix, ROOT_LEN)?),
        (eq_at(prefix, CHILD_OFFSET, VERSION_CHUNK)
            && matches!(le32(prefix, VERSION_VALUE)?, 3 | 4))
            || (eq_at(prefix, CHILD_OFFSET, EDITOR_MAGIC)
                && (6..0x0800_0000).contains(&le32(prefix, EDITOR_LEN)?)),
    );
    Some(Label::_3dsm)
}

/// Legacy 1990 Autodesk 3D Studio DOS `.3ds` header (`"3DS\0"` at `@0..4`, `u32 LE` chunk length ≥ 16 at `@4..8`).
pub(crate) fn three_dsm_legacy(prefix: &[u8], size: u64) -> Option<Label> {
    const LEGACY_3DS_MAGIC: &[u8; 4] = b"3DS\0";
    const MIN_LEN: usize = 16;

    require!(
        size >= MIN_LEN as u64,
        prefix.len() >= MIN_LEN,
        starts(prefix, LEGACY_3DS_MAGIC),
        le32(prefix, 4)? >= MIN_LEN as u32,
        get_slice(prefix, 8, MIN_LEN)?.iter().any(|&b| b == 0),
    );
    Some(Label::_3dsm)
}

pub(crate) const STL_SOLID_PREFIX: &[u8; 6] = b"solid ";

/// 3D Systems Binary StereoLithography (`stlbinary`, 1987): 80-byte header + `u32 LE`
/// triangle count at `@80` validated by `crate::deep::cad::verify_stl_binary` or
/// exact `84 + num_triangles * 50 == size` geometry.
pub(crate) fn stl_binary(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 84,
        prefix.len() >= 84,
        !starts(prefix, STL_SOLID_PREFIX),
    );
    let n_tri = le32(prefix, 80)? as u64;
    require!(n_tri <= 50_000_000, 84 + n_tri * 50 == size);
    let has_banner = starts(prefix, b"STLB ")
        || starts(prefix, b"STLEXP ")
        || starts(prefix, b"Exported from Blender")
        || starts(prefix, b"Blender Binary STL")
        || starts(prefix, b"STL generated by ")
        || starts(prefix, b"STL Exported by ")
        || starts(prefix, b"STL File created by ")
        || starts(prefix, b"Binary STL");
    require!(n_tri >= 1 || has_banner);
    Some(Label::Stlbinary)
}

/// 3D Systems ASCII StereoLithography (`stltext`, 1987): starts with `"solid "` and
/// contains `"facet normal"` and `"vertex"`, or disambiguates to `stlbinary` when a
/// Binary STL header starts with `"solid "`.
pub(crate) fn stl_ascii(prefix: &[u8], size: u64) -> Option<Label> {
    const STL_FACET_NORMAL: &[u8] = b"facet normal";
    const STL_VERTEX: &[u8] = b"vertex";

    if size >= 40
        && prefix.len() >= 32
        && starts(prefix, STL_SOLID_PREFIX)
        && contains(prefix, STL_FACET_NORMAL)
        && contains(prefix, STL_VERTEX)
    {
        return Some(Label::Stltext);
    }
    if size >= 134 && prefix.len() >= 84 {
        let n_tri = le32(prefix, 80)? as u64;
        if n_tri >= 1 && 84 + n_tri * 50 == size {
            return Some(Label::Stlbinary);
        }
    }
    None
}

/// AutoCAD Binary Drawing Exchange Format (`dxf`, 1988): `"AutoCAD Binary DXF\r\n\x1A\x00"`.
pub(crate) fn dxf_binary(prefix: &[u8], size: u64) -> Option<Label> {
    const DXF_BINARY_MAGIC: &[u8; 22] = b"AutoCAD Binary DXF\r\n\x1A\x00";

    require!(
        size >= 22,
        prefix.len() >= 22,
        starts(prefix, DXF_BINARY_MAGIC),
    );
    Some(Label::Dxf)
}

/// AutoCAD ASCII Drawing Exchange Format (`dxf`, 1982): group code `0` + `SECTION` +
/// group code `2` + `HEADER` / `ENTITIES` / `TABLES` / `BLOCKS`.
pub(crate) fn dxf_ascii(prefix: &[u8], size: u64) -> Option<Label> {
    const DXF_SEC_PADDED_LF: &[u8] = b"  0\nSECTION\n";
    const DXF_SEC_PADDED_CRLF: &[u8] = b"  0\r\nSECTION\r\n";
    const DXF_SEC_UNPADDED_LF: &[u8] = b"0\nSECTION\n";
    const DXF_SEC_UNPADDED_CRLF: &[u8] = b"0\r\nSECTION\r\n";
    const DXF_HEADER_TAG: &[u8] = b"HEADER";
    const DXF_ENTITIES_TAG: &[u8] = b"ENTITIES";
    const DXF_TABLES_TAG: &[u8] = b"TABLES";
    const DXF_BLOCKS_TAG: &[u8] = b"BLOCKS";

    require!(size >= 24, prefix.len() >= 20);
    let has_section = starts(prefix, DXF_SEC_PADDED_LF)
        || starts(prefix, DXF_SEC_PADDED_CRLF)
        || starts(prefix, DXF_SEC_UNPADDED_LF)
        || starts(prefix, DXF_SEC_UNPADDED_CRLF);
    require!(
        has_section,
        contains(prefix, DXF_HEADER_TAG)
            || contains(prefix, DXF_ENTITIES_TAG)
            || contains(prefix, DXF_TABLES_TAG)
            || contains(prefix, DXF_BLOCKS_TAG),
    );
    Some(Label::Dxf)
}

/// Sony Imageworks & ILM Alembic 3D scene cache (`alembic`, 2011): `"Ogawa\xFF\x00\x01"`,
/// `"Ogawa\x00\x00\x01"`, or `"Ogawa\x00\xFF\x00"` header magic at `@0..8` and root group offset `<= size` at `@8..16`.
pub(crate) fn alembic(prefix: &[u8], size: u64) -> Option<Label> {
    const OGAWA_FROZEN_MAGIC: &[u8; 8] = b"Ogawa\xFF\x00\x01";
    const OGAWA_UNFROZEN_MAGIC: &[u8; 8] = b"Ogawa\x00\x00\x01";
    const OGAWA_ALT_MAGIC: &[u8; 8] = b"Ogawa\x00\xFF\x00";

    require!(
        size >= 16,
        prefix.len() >= 16,
        starts(prefix, OGAWA_FROZEN_MAGIC)
            || starts(prefix, OGAWA_UNFROZEN_MAGIC)
            || starts(prefix, OGAWA_ALT_MAGIC),
        le64(prefix, 8)? <= size,
    );
    Some(Label::Alembic)
}

/// ASPRS LAS LiDAR point cloud (`las`, 2003): `"LASF"` signature at `@0..4`, version major `1` and
/// minor `0..=4` at `@24..26`, and ≥ 227-byte public header block.
pub(crate) fn las(prefix: &[u8], size: u64) -> Option<Label> {
    const LASF_MAGIC: &[u8; 4] = b"LASF";

    require!(
        size >= 227,
        prefix.len() >= 32,
        starts(prefix, LASF_MAGIC),
        u8_at(prefix, 24)? == 1,
        u8_at(prefix, 25)? <= 4,
    );
    Some(Label::Las)
}

/// NewTek LightWave 3D ASCII scene (`lightwave`, 1990): `"LWSC\n"`, `"LWSC\r\n"`, or `"LWS\n"` header.
pub(crate) fn lightwave_scene(prefix: &[u8], size: u64) -> Option<Label> {
    const LWSC_LF_MAGIC: &[u8] = b"LWSC\n";
    const LWSC_CRLF_MAGIC: &[u8] = b"LWSC\r\n";
    const LWS_LF_MAGIC: &[u8] = b"LWS\n";

    require!(
        size >= 16,
        prefix.len() >= 12,
        starts(prefix, LWSC_LF_MAGIC) || starts(prefix, LWSC_CRLF_MAGIC) || starts(prefix, LWS_LF_MAGIC),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Lightwave)
}

/// Alias|Wavefront / Autodesk Maya Binary scene (`.mb`, `maya`, 1998): 32-bit `"FOR4"` or 64-bit
/// `"FOR8"` IFF container with `"MAYA"` or `"Maya"` form type at `@8` or `@12`.
pub(crate) fn maya_binary(prefix: &[u8], size: u64) -> Option<Label> {
    const MAYA_UPPER_FORM: &[u8; 4] = b"MAYA";
    const MAYA_MIXED_FORM: &[u8; 4] = b"Maya";

    require!(
        size >= 16,
        prefix.len() >= 16,
        u8_at(prefix, 4)? == 0,
        (starts(prefix, IFF_FOR4_MAGIC) && (eq_at(prefix, 8, MAYA_UPPER_FORM) || eq_at(prefix, 8, MAYA_MIXED_FORM)))
            || (starts(prefix, IFF_FOR8_MAGIC)
                && (eq_at(prefix, 12, MAYA_UPPER_FORM) || eq_at(prefix, 12, MAYA_MIXED_FORM))),
    );
    Some(Label::Maya)
}

/// Alias|Wavefront / Autodesk Maya ASCII scene (`.ma`, `maya`, 1998): `"//Maya ASCII"` header comment.
pub(crate) fn maya_ascii(prefix: &[u8], size: u64) -> Option<Label> {
    const MAYA_ASCII_MAGIC: &[u8] = b"//Maya ASCII";

    require!(
        size >= 16,
        prefix.len() >= 16,
        starts(prefix, MAYA_ASCII_MAGIC),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Maya)
}

/// Robert McNeel & Associates Rhinoceros openNURBS 3D model (`.3dm`, `rhinoceros`, 1998):
/// `"3D Geometry File Format "` 24-byte header followed by binary version padding.
pub(crate) fn rhinoceros(prefix: &[u8], size: u64) -> Option<Label> {
    const RHINO_3DM_MAGIC: &[u8; 24] = b"3D Geometry File Format ";

    require!(
        size >= 32,
        prefix.len() >= 32,
        starts(prefix, RHINO_3DM_MAGIC),
        has_non_ascii_text_byte(&prefix[24..32]),
    );
    Some(Label::Rhinoceros)
}

/// Pixar Universal Scene Description (`usd`, 2016): `"PXR-USDC"` binary crate (with major version
/// `<= 2` at `@8`) or `"#usda 1.0"` ASCII scene header.
pub(crate) fn usd(prefix: &[u8], size: u64) -> Option<Label> {
    const USDC_CRATE_MAGIC: &[u8; 8] = b"PXR-USDC";
    const USDA_ASCII_MAGIC: &[u8; 9] = b"#usda 1.0";

    require!(size >= 16, prefix.len() >= 14);
    if starts(prefix, USDC_CRATE_MAGIC) {
        require!(u8_at(prefix, 8)? <= 2);
        return Some(Label::Usd);
    }
    require!(
        starts(prefix, USDA_ASCII_MAGIC),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Usd)
}

/// Kitware Visualization Toolkit legacy datafile (`.vtk`, `vtk`, 1993): `"# vtk DataFile Version "`
/// banner accompanied by `"ASCII"` or `"BINARY"` encoding keyword.
pub(crate) fn vtk(prefix: &[u8], size: u64) -> Option<Label> {
    const VTK_LEGACY_MAGIC: &[u8] = b"# vtk DataFile Version ";
    const VTK_ASCII_MODE: &[u8] = b"ASCII";
    const VTK_BINARY_MODE: &[u8] = b"BINARY";

    require!(
        size >= 32,
        prefix.len() >= 26,
        starts(prefix, VTK_LEGACY_MAGIC),
        contains(prefix, VTK_ASCII_MODE) || contains(prefix, VTK_BINARY_MODE),
    );
    Some(Label::Vtk)
}

/// Pixologic / Maxon ZBrush document, tool, or project (`zbrush`, 1999): `"ZBr4"` or
/// `"ZBrush File.Copyright "` binary header, or `A8 00 00 00` `.zpr` header containing `"ZBrush"`.
pub(crate) fn zbrush(prefix: &[u8], size: u64) -> Option<Label> {
    const ZBR4_MAGIC: &[u8; 4] = b"ZBr4";
    const ZBRUSH_COPYRIGHT_HDR: &[u8] = b"ZBrush File.Copyright ";
    const ZPR_HDR_LEN_MAGIC: &[u8] = &[0xA8, 0x00, 0x00, 0x00];
    const ZBRUSH_NAME_TAG: &[u8] = b"ZBrush";

    require!(
        size >= 32,
        prefix.len() >= 24,
        ((starts(prefix, ZBR4_MAGIC) || starts(prefix, ZBRUSH_COPYRIGHT_HDR))
            && has_non_ascii_text_byte(&prefix[4..24]))
            || (starts(prefix, ZPR_HDR_LEN_MAGIC)
                && contains(prefix, ZBRUSH_NAME_TAG)),
    );
    Some(Label::Zbrush)
}

/// ASME Y14.26M Initial Graphics Exchange Specification (`iges`, 1980): 80-column fixed card record
/// with printable ASCII columns `1..=72` and Start section sequence `"S      1"` or `"S0000001"` at `@72..80`.
pub(crate) fn iges(prefix: &[u8], size: u64) -> Option<Label> {
    const IGES_START_SEC_SPACES: &[u8; 8] = b"S      1";
    const IGES_START_SEC_ZEROS: &[u8; 8] = b"S0000001";

    require!(
        size >= 80,
        prefix.len() >= 80,
        eq_at(prefix, 72, IGES_START_SEC_SPACES) || eq_at(prefix, 72, IGES_START_SEC_ZEROS),
        prefix[..72].iter().all(|&b| (0x20..=0x7E).contains(&b)),
    );
    let _ = get_slice(prefix, 0, 80);
    Some(Label::Iges)
}

/// Alias|Wavefront Material Template Library (`.mtl`, `mtl`, 1990): `"# "` or `"newmtl "` header
/// with `"newmtl "` and diffuse/ambient reflectance (`Kd ` / `Ka `) statements.
pub(crate) fn mtl(prefix: &[u8], size: u64) -> Option<Label> {
    const MTL_NEWMTL_KW: &[u8] = b"newmtl ";
    const MTL_KD_LF: &[u8] = b"\nKd ";
    const MTL_KA_LF: &[u8] = b"\nKa ";
    const MTL_KD_CRLF: &[u8] = b"\r\nKd ";

    require!(
        size >= 16,
        prefix.len() >= 16,
        (starts(prefix, WAVEFRONT_COMMENT) || starts(prefix, MTL_NEWMTL_KW))
            && contains(prefix, MTL_NEWMTL_KW)
            && (contains(prefix, MTL_KD_LF) || contains(prefix, MTL_KA_LF) || contains(prefix, MTL_KD_CRLF)),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Mtl)
}

/// Alias|Wavefront 3D Object mesh (`.obj`, `obj`, 1990): comment/material/object header followed by
/// vertex (`\nv `) and face/normal (`\nf ` / `\nvn `) records.
pub(crate) fn obj(prefix: &[u8], size: u64) -> Option<Label> {
    const OBJ_MTLLIB_KW: &[u8] = b"mtllib ";
    const OBJ_OBJECT_KW: &[u8] = b"o ";
    const OBJ_VERTEX_START: &[u8] = b"v ";
    const OBJ_VERTEX_LF: &[u8] = b"\nv ";
    const OBJ_VERTEX_CRLF: &[u8] = b"\r\nv ";
    const OBJ_FACE_LF: &[u8] = b"\nf ";
    const OBJ_FACE_CRLF: &[u8] = b"\r\nf ";
    const OBJ_VNORMAL_LF: &[u8] = b"\nvn ";

    require!(
        size >= 16,
        prefix.len() >= 16,
        (starts(prefix, WAVEFRONT_COMMENT)
            || starts(prefix, OBJ_MTLLIB_KW)
            || starts(prefix, OBJ_OBJECT_KW)
            || starts(prefix, OBJ_VERTEX_START))
            && (contains(prefix, OBJ_VERTEX_LF) || contains(prefix, OBJ_VERTEX_CRLF))
            && (contains(prefix, OBJ_FACE_LF)
                || contains(prefix, OBJ_FACE_CRLF)
                || contains(prefix, OBJ_VNORMAL_LF)),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Obj)
}
