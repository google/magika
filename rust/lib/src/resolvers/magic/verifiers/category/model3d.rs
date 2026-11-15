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
    static BLEND_32_LE: Subformat =
        Subformat::with_ids("Blender 3D (32-bit, little-endian)", "fmt/902", Some("fdd000559"));
    static BLEND_32_BE: Subformat =
        Subformat::with_ids("Blender 3D (32-bit, big-endian)", "fmt/902", Some("fdd000559"));
    static BLEND_64_LE: Subformat =
        Subformat::with_ids("Blender 3D (64-bit, little-endian)", "fmt/903", Some("fdd000559"));
    static BLEND_64_BE: Subformat =
        Subformat::with_ids("Blender 3D (64-bit, big-endian)", "fmt/903", Some("fdd000559"));
    const MAGIC: &[u8] = b"BLENDER"; // @0..7: Blender file magic
    const PTR_SIZE: usize = 7; // @7: '_' (32-bit) or '-' (64-bit)
    const ENDIAN: usize = 8; // @8: 'v' (little-endian) or 'V' (big-endian)
    const VER_MAJOR: usize = 9; // @9: '1'..='4'
    const MIN_LEN: usize = 32; // 12-byte file header + first BHead

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    let ptr_size = u8_at(prefix, PTR_SIZE)?;
    let endian = u8_at(prefix, ENDIAN)?;
    require!(
        matches!(ptr_size, b'_' | b'-'),
        matches!(endian, b'v' | b'V'),
        matches!(u8_at(prefix, VER_MAJOR)?, b'1'..=b'4'),
        u8_at(prefix, 10)?.is_ascii_digit(),
        u8_at(prefix, 11)?.is_ascii_digit(),
        u8_at(prefix, if endian == b'v' { 19 } else { 16 })? == 0,
    );
    let sub = match (ptr_size, endian) {
        (b'_', b'v') => &BLEND_32_LE,
        (b'_', b'V') => &BLEND_32_BE,
        (b'-', b'v') => &BLEND_64_LE,
        (b'-', b'V') => &BLEND_64_BE,
        _ => unreachable!(),
    };
    with_subformat(Label::Blend, sub)
}

/// AutoCAD DWG drawing: `"AC10"` version prefix with known release code (`09`, `12`, `14`, `15`,
/// `18`, `21`, `24`, `27`, `32`), five zero bytes at `@6..11`, and ≥ 128-byte header.
pub(crate) fn dwg(prefix: &[u8]) -> Option<Label> {
    static DWG_R11_12: Subformat =
        Subformat::with_ids("AutoCAD DWG R11/12 (AC1009)", "fmt/32", Some("fdd000445"));
    static DWG_R13: Subformat =
        Subformat::with_ids("AutoCAD DWG R13 (AC1012)", "fmt/33", Some("fdd000445"));
    static DWG_R14: Subformat =
        Subformat::with_ids("AutoCAD DWG R14 (AC1014)", "fmt/34", Some("fdd000445"));
    static DWG_2000: Subformat =
        Subformat::with_ids("AutoCAD DWG 2000-2002 (AC1015)", "fmt/35", Some("fdd000445"));
    static DWG_2004: Subformat =
        Subformat::with_ids("AutoCAD DWG 2004-2005 (AC1018)", "fmt/36", Some("fdd000445"));
    static DWG_2007: Subformat =
        Subformat::with_ids("AutoCAD DWG 2007-2008 (AC1021)", "x-fmt/455", Some("fdd000445"));
    static DWG_2010: Subformat =
        Subformat::with_ids("AutoCAD DWG 2010-2012 (AC1024)", "fmt/434", Some("fdd000445"));
    static DWG_2013: Subformat =
        Subformat::with_ids("AutoCAD DWG 2013-2017 (AC1027)", "fmt/531", Some("fdd000445"));
    static DWG_2018: Subformat =
        Subformat::with_ids("AutoCAD DWG 2018+ (AC1032)", "fmt/1395", Some("fdd000445"));
    const MAGIC: &[u8] = b"AC10"; // @0..4: AutoCAD DWG version prefix
    const ZERO_PAD: usize = 6; // @6..11: 5 reserved zero bytes
    const MIN_LEN: usize = 128;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    let sub = match get_slice(prefix, 4, 6)? {
        b"09" => &DWG_R11_12,
        b"12" => &DWG_R13,
        b"14" => &DWG_R14,
        b"15" => &DWG_2000,
        b"18" => &DWG_2004,
        b"21" => &DWG_2007,
        b"24" => &DWG_2010,
        b"27" => &DWG_2013,
        b"32" => &DWG_2018,
        _ => return None,
    };
    require!(eq_at(prefix, ZERO_PAD, &[0; 5]));
    with_subformat(Label::Dwg, sub)
}

/// Autodesk FBX: binary `"Kaydara FBX Binary  \x00\x1a\x00"` with version `3000..=7700` at `@23`,
/// or ASCII export header comment `"; FBX X.Y.Z project file"`.
pub(crate) fn fbx(prefix: &[u8]) -> Option<Label> {
    static FBX_BINARY: Subformat = Subformat::with_ids("FBX binary", "fmt/1009", Some("fdd000558"));
    static FBX_ASCII: Subformat = Subformat::with_ids("FBX ASCII", "fmt/1010", Some("fdd000558"));
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
        return with_subformat(Label::Fbx, &FBX_BINARY);
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
    with_subformat(Label::Fbx, &FBX_ASCII)
}

/// Khronos binary glTF 2.0 (GLB): `"glTF"` + version 2, 4-byte-aligned total and chunk lengths,
/// `"JSON"` first chunk type at `@16`, and JSON object opening `{` after optional whitespace at `@20`.
pub(crate) fn glb(prefix: &[u8]) -> Option<Label> {
    static GLTF_BINARY: Subformat =
        Subformat::with_ids("glTF 2.0 binary (.glb)", "fmt/1316", Some("fdd000500"));
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
    with_subformat(Label::Gltf, &GLTF_BINARY)
}

/// Polygon File Format (PLY): `"ply"` line followed by `"format <encoding> 1.0"` and whitespace.
pub(crate) fn ply(prefix: &[u8]) -> Option<Label> {
    static PLY_ASCII: Subformat = Subformat::new("PLY ASCII");
    static PLY_BINARY_LE: Subformat = Subformat::new("PLY binary (little-endian)");
    static PLY_BINARY_BE: Subformat = Subformat::new("PLY binary (big-endian)");
    const HEADER_LF: &[u8] = b"ply\nformat ";
    const HEADER_CRLF: &[u8] = b"ply\r\nformat ";
    static ENCODINGS: [(&[u8], &Subformat); 3] = [
        (b"ascii 1.0", &PLY_ASCII),
        (b"binary_little_endian 1.0", &PLY_BINARY_LE),
        (b"binary_big_endian 1.0", &PLY_BINARY_BE),
    ];

    let rest = prefix.strip_prefix(HEADER_LF).or_else(|| prefix.strip_prefix(HEADER_CRLF))?;
    for &(enc, sub) in &ENCODINGS {
        if let Some(tail) = rest.strip_prefix(enc)
            && matches!(tail.first(), Some(b' ' | b'\t' | b'\r' | b'\n'))
        {
            return with_subformat(Label::Ply, sub);
        }
    }
    None
}

/// SketchUp model (`.skp`): UTF-16LE BOM + `"SketchUp Model"` (32 bytes) or legacy length-prefixed
/// ASCII `"\x0eSketchUp Model\x08"` (16 bytes).
pub(crate) fn sketchup(prefix: &[u8]) -> Option<Label> {
    static SKP_LEGACY: Subformat = Subformat::new("SketchUp (ASCII header)");
    static SKP_UNICODE: Subformat = Subformat::new("SketchUp (UTF-16 header)");
    const LEGACY: &[u8] = b"\x0eSketchUp Model\x08"; // @0..16: legacy ASCII SketchUp header
    const UNICODE: &[u8] = &[
        0xFF, 0xFE, 0xFF, 0x0E, b'S', 0, b'k', 0, b'e', 0, b't', 0, b'c', 0, b'h', 0, b'U', 0,
        b'p', 0, b' ', 0, b'M', 0, b'o', 0, b'd', 0, b'e', 0, b'l', 0,
    ]; // @0..32: UTF-16LE SketchUp header

    let matched = starts(prefix, LEGACY) || starts(prefix, UNICODE);
    require!(matched);
    with_subformat(Label::Sketchup, if starts(prefix, LEGACY) { &SKP_LEGACY } else { &SKP_UNICODE })
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
    static THREE_DS_MESH: Subformat = Subformat::new("3D Studio mesh (M3DMAGIC)");
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
    with_subformat(Label::_3dsm, &THREE_DS_MESH)
}

/// Legacy 1990 Autodesk 3D Studio DOS `.3ds` header (`"3DS\0"` at `@0..4`, `u32 LE` chunk length ≥ 16 at `@4..8`).
pub(crate) fn three_dsm_legacy(prefix: &[u8], size: u64) -> Option<Label> {
    static THREE_DS_DOS: Subformat = Subformat::new("3D Studio DOS (3DS header)");
    const LEGACY_3DS_MAGIC: &[u8; 4] = b"3DS\0";
    const MIN_LEN: usize = 16;

    require!(
        size >= MIN_LEN as u64,
        prefix.len() >= MIN_LEN,
        starts(prefix, LEGACY_3DS_MAGIC),
        le32(prefix, 4)? >= MIN_LEN as u32,
        get_slice(prefix, 8, MIN_LEN)?.contains(&0),
    );
    with_subformat(Label::_3dsm, &THREE_DS_DOS)
}

pub(crate) const STL_SOLID_PREFIX: &[u8; 6] = b"solid ";

/// 3D Systems Binary StereoLithography (`stlbinary`, 1987): 80-byte header + `u32 LE`
/// triangle count at `@80` validated by `crate::deep::cad::verify_stl_binary` or
/// exact `84 + num_triangles * 50 == size` geometry.
pub(crate) fn stl_binary(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 84, prefix.len() >= 84, !starts(prefix, STL_SOLID_PREFIX),);
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
    static DXF_BINARY: Subformat = Subformat::with_ids("DXF binary", "fmt/1394", None);
    const DXF_BINARY_MAGIC: &[u8; 22] = b"AutoCAD Binary DXF\r\n\x1A\x00";

    require!(size >= 22, prefix.len() >= 22, starts(prefix, DXF_BINARY_MAGIC),);
    with_subformat(Label::Dxf, &DXF_BINARY)
}

/// AutoCAD ASCII Drawing Exchange Format (`dxf`, 1982): group code `0` + `SECTION` +
/// group code `2` + `HEADER` / `ENTITIES` / `TABLES` / `BLOCKS`.
pub(crate) fn dxf_ascii(prefix: &[u8], size: u64) -> Option<Label> {
    static DXF_ASCII: Subformat = Subformat::with_ids("DXF ASCII", "fmt/63", Some("fdd000446"));
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
    with_subformat(Label::Dxf, &DXF_ASCII)
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
    static LAS_1_0: Subformat = Subformat::with_ids("ASPRS LAS 1.0", "fmt/368", None);
    static LAS_1_1: Subformat = Subformat::with_ids("ASPRS LAS 1.1", "fmt/369", None);
    static LAS_1_2: Subformat = Subformat::with_ids("ASPRS LAS 1.2", "fmt/370", None);
    static LAS_1_3: Subformat = Subformat::new("ASPRS LAS 1.3");
    static LAS_1_4: Subformat = Subformat::with_fdd("ASPRS LAS 1.4", "fdd000418");
    const LASF_MAGIC: &[u8; 4] = b"LASF";

    require!(size >= 227, prefix.len() >= 32, starts(prefix, LASF_MAGIC), u8_at(prefix, 24)? == 1);
    let sub = match u8_at(prefix, 25)? {
        0 => &LAS_1_0,
        1 => &LAS_1_1,
        2 => &LAS_1_2,
        3 => &LAS_1_3,
        4 => &LAS_1_4,
        _ => return None,
    };
    with_subformat(Label::Las, sub)
}

/// NewTek LightWave 3D ASCII scene (`lightwave`, 1990): `"LWSC\n"`, `"LWSC\r\n"`, or `"LWS\n"` header.
pub(crate) fn lightwave_scene(prefix: &[u8], size: u64) -> Option<Label> {
    static LIGHTWAVE_SCENE: Subformat = Subformat::new("LightWave scene (ASCII)");
    const LWSC_LF_MAGIC: &[u8] = b"LWSC\n";
    const LWSC_CRLF_MAGIC: &[u8] = b"LWSC\r\n";
    const LWS_LF_MAGIC: &[u8] = b"LWS\n";

    require!(
        size >= 16,
        prefix.len() >= 12,
        starts(prefix, LWSC_LF_MAGIC)
            || starts(prefix, LWSC_CRLF_MAGIC)
            || starts(prefix, LWS_LF_MAGIC),
        is_utf8_text_header(prefix, size as usize),
    );
    with_subformat(Label::Lightwave, &LIGHTWAVE_SCENE)
}

/// Alias|Wavefront / Autodesk Maya Binary scene (`.mb`, `maya`, 1998): 32-bit `"FOR4"` or 64-bit
/// `"FOR8"` IFF container with `"MAYA"` or `"Maya"` form type at `@8` or `@12`.
pub(crate) fn maya_binary(prefix: &[u8], size: u64) -> Option<Label> {
    static MAYA_BINARY_32: Subformat =
        Subformat::with_ids("Maya Binary (32-bit)", "fmt/861", Some("fdd000605"));
    static MAYA_BINARY_64: Subformat =
        Subformat::with_ids("Maya Binary (64-bit)", "fmt/862", Some("fdd000605"));
    const MAYA_UPPER_FORM: &[u8; 4] = b"MAYA";
    const MAYA_MIXED_FORM: &[u8; 4] = b"Maya";

    require!(
        size >= 16,
        prefix.len() >= 16,
        u8_at(prefix, 4)? == 0,
        (starts(prefix, IFF_FOR4_MAGIC)
            && (eq_at(prefix, 8, MAYA_UPPER_FORM) || eq_at(prefix, 8, MAYA_MIXED_FORM)))
            || (starts(prefix, IFF_FOR8_MAGIC)
                && (eq_at(prefix, 12, MAYA_UPPER_FORM) || eq_at(prefix, 12, MAYA_MIXED_FORM))),
    );
    let sub = if starts(prefix, IFF_FOR4_MAGIC) { &MAYA_BINARY_32 } else { &MAYA_BINARY_64 };
    with_subformat(Label::Maya, sub)
}

/// Alias|Wavefront / Autodesk Maya ASCII scene (`.ma`, `maya`, 1998): `"//Maya ASCII"` header comment.
pub(crate) fn maya_ascii(prefix: &[u8], size: u64) -> Option<Label> {
    static MAYA_ASCII: Subformat = Subformat::with_ids("Maya ASCII", "fmt/863", Some("fdd000604"));
    const MAYA_ASCII_MAGIC: &[u8] = b"//Maya ASCII";

    require!(
        size >= 16,
        prefix.len() >= 16,
        starts(prefix, MAYA_ASCII_MAGIC),
        is_utf8_text_header(prefix, size as usize),
    );
    with_subformat(Label::Maya, &MAYA_ASCII)
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
    static USDC: Subformat = Subformat::new("USD crate (binary)");
    static USDA: Subformat = Subformat::with_ids("USD ASCII", "fmt/1905", Some("fdd000561"));
    const USDC_CRATE_MAGIC: &[u8; 8] = b"PXR-USDC";
    const USDA_ASCII_MAGIC: &[u8; 9] = b"#usda 1.0";

    require!(size >= 16, prefix.len() >= 14);
    if starts(prefix, USDC_CRATE_MAGIC) {
        require!(u8_at(prefix, 8)? <= 2);
        return with_subformat(Label::Usd, &USDC);
    }
    require!(starts(prefix, USDA_ASCII_MAGIC), is_utf8_text_header(prefix, size as usize),);
    with_subformat(Label::Usd, &USDA)
}

/// Kitware Visualization Toolkit legacy datafile (`.vtk`, `vtk`, 1993): `"# vtk DataFile Version "`
/// banner accompanied by `"ASCII"` or `"BINARY"` encoding keyword.
pub(crate) fn vtk(prefix: &[u8], size: u64) -> Option<Label> {
    static VTK_ASCII: Subformat = Subformat::new("VTK legacy (ASCII)");
    static VTK_BINARY: Subformat = Subformat::new("VTK legacy (binary)");
    const VTK_LEGACY_MAGIC: &[u8] = b"# vtk DataFile Version ";
    const VTK_ASCII_MODE: &[u8] = b"ASCII";
    const VTK_BINARY_MODE: &[u8] = b"BINARY";

    require!(size >= 32, prefix.len() >= 26, starts(prefix, VTK_LEGACY_MAGIC));
    let sub = if contains(prefix, VTK_ASCII_MODE) {
        &VTK_ASCII
    } else if contains(prefix, VTK_BINARY_MODE) {
        &VTK_BINARY
    } else {
        return None;
    };
    with_subformat(Label::Vtk, sub)
}

/// Pixologic / Maxon ZBrush document, tool, or project (`zbrush`, 1999): `"ZBr4"` or
/// `"ZBrush File.Copyright "` binary header, or `A8 00 00 00` `.zpr` header containing `"ZBrush"`.
pub(crate) fn zbrush(prefix: &[u8], size: u64) -> Option<Label> {
    static ZBRUSH_DOCUMENT: Subformat = Subformat::new("ZBrush document or tool");
    static ZBRUSH_PROJECT: Subformat = Subformat::new("ZBrush project (ZPR)");
    const ZBR4_MAGIC: &[u8; 4] = b"ZBr4";
    const ZBRUSH_COPYRIGHT_HDR: &[u8] = b"ZBrush File.Copyright ";
    const ZPR_HDR_LEN_MAGIC: &[u8] = &[0xA8, 0x00, 0x00, 0x00];
    const ZBRUSH_NAME_TAG: &[u8] = b"ZBrush";

    require!(
        size >= 32,
        prefix.len() >= 24,
        ((starts(prefix, ZBR4_MAGIC) || starts(prefix, ZBRUSH_COPYRIGHT_HDR))
            && has_non_ascii_text_byte(&prefix[4..24]))
            || (starts(prefix, ZPR_HDR_LEN_MAGIC) && contains(prefix, ZBRUSH_NAME_TAG)),
    );
    let is_project = starts(prefix, ZPR_HDR_LEN_MAGIC);
    with_subformat(Label::Zbrush, if is_project { &ZBRUSH_PROJECT } else { &ZBRUSH_DOCUMENT })
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
            && (contains(prefix, MTL_KD_LF)
                || contains(prefix, MTL_KA_LF)
                || contains(prefix, MTL_KD_CRLF)),
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

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const ATARI_CAD3D2_MAGIC: &[u8; 2] = hex2bin!("3D 02");
const BLIZZARD_M3_COUNT_OFFSET: usize = 8;
const BLIZZARD_M3_MODL_REF_OFFSET: usize = 12;
const BLIZZARD_M3_OFS_OFFSET: usize = 4;
const BLIZZARD_M3_PAD_OFFSET: usize = 24;
const BODYSLIDE_COUNT_OFFSET: usize = 8;
const BODYSLIDE_NAME_LEN_OFFSET: usize = 12;
const BODYSLIDE_OSD_MAGIC: &[u8; 8] = bytes!(&[0x00], b"DSO", &[0x01, 0x00, 0x00, 0x00]);
const CAPCOM_MT_MRL_MAGIC: &[u8; 4] = b"MRL\0";
const CESIUM_B3DM_MAGIC: &[u8; 8] = bytes!(b"b3dm", &[0x01, 0x00, 0x00, 0x00]);
const CESIUM_BYTE_LEN_OFFSET: usize = 8;
const CESIUM_FT_JSON_LEN_OFFSET: usize = 12;
const HOOPS_HSF_MAGIC: &[u8; 8] = b";; HSF V";
const JACKHAMMER_JMF_MAGIC: &[u8; 4] = b"JHMF";
const KN5_EXTRA_OFFSET: usize = 10;
const KN5_MAGIC: &[u8; 6] = b"sc6969";
const KN5_VERSION_OFFSET: usize = 6;
const KSP_MU_MAGIC: &[u8; 4] = hex2bin!("FF 2A 01 00");
const KSP_MU_NAME_LEN_OFFSET: usize = 8;
const KSP_MU_VERSION_OFFSET: usize = 4;
const LIBGDX_G3DB_MAGIC: &[u8; 16] =
    bytes!(b"{s", &[0x07], b"versionai", &[0x02, 0x00, 0x00, 0x00]);
const MIN_ARC_CASTPART_SIZE: u64 = 36;
const MIN_ASSETTO_CORSA_KN5_SIZE: u64 = 32;
const MIN_ASTM_E57_SIZE: u64 = 48;
const MIN_ATARI_CAD3D2_SIZE: u64 = 269;
const MIN_ATLUS_GFS_SIZE: u64 = 32;
const MIN_BIOWARE_FXM_SIZE: u64 = 48;
const MIN_BLIZZARD_M3_SIZE: u64 = 64;
const MIN_BODYSLIDE_OSD_SIZE: u64 = 24;
const MIN_BOHEMIA_P3D_SIZE: u64 = 32;
const MIN_BOHEMIA_RTM_SIZE: u64 = 32;
const MIN_BROTHER_PES_SIZE: u64 = 48;
const MIN_BSP_MAP_SIZE: u64 = 144;
const MIN_C3_MAXFILE_SIZE: u64 = 28;
const MIN_CAL3D_SIZE: u64 = 16;
const MIN_CAPCOM_MT_MRL_SIZE: u64 = 32;
const MIN_CESIUM_B3DM_SIZE: u64 = 28;
const MIN_CESIUM_GEOM_SIZE: u64 = 32;
const MIN_CTM2_MAP_SIZE: u64 = 40;
const MIN_D3D_INDEX_BUFFER_IB_SIZE: u64 = 64;
const MIN_DCS_WORLD_EDM_SIZE: u64 = 64;
const MIN_DIESEL_GAI_SIZE: u64 = 36;
const MIN_EA_EBO_SIZE: u64 = 28;
const MIN_EDF_MESH_SIZE: u64 = 48;
const MIN_EPLYBNDS_SIZE: u64 = 32;
const MIN_EZNEC_EZ_SIZE: u64 = 28;
const MIN_FACEGEN_TRI_SIZE: u64 = 36;
const MIN_FALCOM_MI_SIZE: u64 = 36;
const MIN_FILAMENT_MATERIAL_SIZE: u64 = 29;
const MIN_FLIGHTSIM_NAME_AIR_SIZE: u64 = 64;
const MIN_FLUENT_INTERP_IP_SIZE: u64 = 512;
const MIN_FROMSOFT_FLVER_SIZE: u64 = 128;
const MIN_FROMSOFT_TAE_SIZE: u64 = 64;
const MIN_GAMEBRYO_NIF_SIZE: u64 = 48;
const MIN_GEO_GTX_SIZE: u64 = 28;
const MIN_GTA_IFP_SIZE: u64 = 32;
const MIN_HAVOK_HKX_SIZE: u64 = 64;
const MIN_HOMEWORLD_MEX_SIZE: u64 = 28;
const MIN_HOOPS_HSF_SIZE: u64 = 32;
const MIN_IDSP_SPR_SIZE: u64 = 36;
const MIN_ILLUSION_I3D_SIZE: u64 = 32;
const MIN_JACKHAMMER_JMF_SIZE: u64 = 66;
const MIN_KOMPAS_3D_SIZE: u64 = 64;
const MIN_KSP_MU_SIZE: u64 = 24;
const MIN_LAYAMODEL_0501_LM_SIZE: u64 = 64;
const MIN_LFM2_LFMAT_SIZE: u64 = 28;
const MIN_LIBGDX_G3DB_SIZE: u64 = 32;
const MIN_LINEAGE2_VER111_USX_SIZE: u64 = 64;
const MIN_LIVE2D_CMO3_SIZE: u64 = 32;
const MIN_MAGICAVOXEL_SIZE: u64 = 28;
const MIN_MANDELBULB3D_M3P_SIZE: u64 = 64;
const MIN_MASTERCAM_V9_MC9_SIZE: u64 = 64;
const MIN_MDXELM_G2P_SIZE: u64 = 28;
const MIN_MMD_PMD_SIZE: u64 = 283;
const MIN_MMD_PMX_SIZE: u64 = 32;
const MIN_MMD_VMD_SIZE: u64 = 64;
const MIN_NAMCO_HBSS_SIZE: u64 = 64;
const MIN_NEXON_KARTRIDER_1S_SIZE: u64 = 28;
const MIN_NINTENDO_J3D_SIZE: u64 = 64;
const MIN_OGRE_MESH_SIZE: u64 = 32;
const MIN_PADS_PCB_SIZE: u64 = 36;
const MIN_PAINT4D_SIZE: u64 = 32;
const MIN_PANASONIC_S3D_SIZE: u64 = 28;
const MIN_PARADOX_MESH_SIZE: u64 = 32;
const MIN_PEPAKURA_PDI1_SIZE: u64 = 28;
const MIN_PRISTON_TALE_SMD_SIZE: u64 = 64;
const MIN_PROTEUS_ISIS_DSN_SIZE: u64 = 28;
const MIN_PROTEUS_PDSPART_SIZE: u64 = 32;
const MIN_QUAKE1_MDL_SIZE: u64 = 84;
const MIN_QUAKE2_MD2_SIZE: u64 = 68;
const MIN_RACER_DOF1_SIZE: u64 = 28;
const MIN_RE_ENGINE_CHAIN_SIZE: u64 = 32;
const MIN_RE_ENGINE_MCAMLIST_SIZE: u64 = 40;
const MIN_SCS_PMD_SIZE: u64 = 64;
const MIN_SCS_PMG_SIZE: u64 = 28;
const MIN_SCS_TOBJ_SIZE: u64 = 36;
const MIN_SEGA_DRAGON_GMD_SIZE: u64 = 48;
const MIN_SEMI_OASIS_SIZE: u64 = 24;
const MIN_SHOCKWAVE_W3D_SIZE: u64 = 24;
const MIN_SONY_PSP_GMO_SIZE: u64 = 32;
const MIN_STALKER_XRAY_OGF_SIZE: u64 = 64;
const MIN_STORM_GM_SIZE: u64 = 40;
const MIN_TANGOPRO_LIB_SIZE: u64 = 112;
const MIN_TRAINZ_IM_SIZE: u64 = 32;
const MIN_UNITY_ENLIGHTEN_CAW_SIZE: u64 = 36;
const MIN_UNITY_ENLIGHTEN_VIS_SIZE: u64 = 64;
const MIN_VALVE_ANI_SIZE: u64 = 80;
const MIN_VALVE_RMF_SIZE: u64 = 22;
const MIN_VALVE_STUDIO_MDL_SIZE: u64 = 148;
const MIN_VALVE_VVD_SIZE: u64 = 64;
const MIN_VAM_VAB_SIZE: u64 = 32;
const MIN_VCTR_VECTOR_SIZE: u64 = 48;
const MIN_VECTOR_FWK_SIZE: u64 = 32;
const MIN_VOXLAP_KV6_SIZE: u64 = 32;
const MIN_WARCRAFT3_MDX_SIZE: u64 = 24;
const MIN_WESTWOOD_VXL_SIZE: u64 = 64;
const MIN_WOW_M2_SIZE: u64 = 64;
const MIN_XPLANE_ACF_SIZE: u64 = 44;
const MIN_XPLANE_DSF_SIZE: u64 = 36;
const SHOCKWAVE_W3D_MAGIC: &[u8; 4] = b"IFX\0";
const TRAINZ_FILE_SIZE_OFFSET: usize = 4;
const TRAINZ_INFO_SIZE_OFFSET: usize = 16;
const TRAINZ_JIRF_MAGIC: &[u8; 4] = b"JIRF";
const WARCRAFT3_MDX_MAGIC: &[u8; 4] = b"MDLX";

/// ARC CAST Airport Simulation Component (CT COBJ2, .CASTPart).
pub(crate) fn verify_arc_castpart(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ARC_CASTPART_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"CT\x02\0\0\0\x05\0\0\0COBJ2")
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::ArcCastpart)
}

/// Kunos Simulazioni Assetto Corsa KN5 3D Model file (`.kn5`).
pub(crate) fn verify_assetto_corsa_kn5(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ASSETTO_CORSA_KN5_SIZE,
        prefix.len() >= 18,
        starts(prefix, KN5_MAGIC),
        (5..=6).contains(&le32(prefix, KN5_VERSION_OFFSET)?),
        le32(prefix, KN5_EXTRA_OFFSET)? <= 1024
    );
    Some(Label::AssettoCorsaKn5)
}

/// ASTM E57 3D Point Cloud (`.e57`, `"ASTM-E57"` + `major == 1` + `minor == 0`).
pub(crate) fn verify_astm_e57_point_cloud(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ASTM_E57_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"ASTM-E57"),
        le32(prefix, 8) == Some(1),
        le32(prefix, 12) == Some(0),
        le64(prefix, 24)? >= 48
    );
    Some(Label::AstmE57)
}

/// Antic Software Atari ST Cyber Studio CAD-3D 2.0 3D Object (`.3d2`, 1987 Tom Hudson).
pub(crate) fn verify_atari_cad3d2(prefix: &[u8], size: u64) -> Option<Label> {
    const OBJ_COUNT_OFF: usize = 2; // @2..4: u16 BE object count (1..=40)
    const OBJ0_NAME_OFF: usize = 256; // @256..265: 8-char NUL-terminated ASCII name (9 bytes)
    const OBJ0_VERTS_OFF: usize = 265; // @265..267: u16 BE Object 0 vertex count
    const OBJ0_VERTS_DATA_OFF: usize = 267; // @267: vertex_count * 6 bytes (X,Y,Z fixed i16)

    require!(
        size >= MIN_ATARI_CAD3D2_SIZE,
        prefix.len() >= MIN_ATARI_CAD3D2_SIZE as usize,
        starts(prefix, ATARI_CAD3D2_MAGIC)
    );

    let obj_count = be16(prefix, OBJ_COUNT_OFF)?;
    require!((1..=40).contains(&obj_count));

    let name0 = get_slice(prefix, OBJ0_NAME_OFF, OBJ0_NAME_OFF + 9)?;
    let nul_pos = name0.iter().position(|&b| b == 0)?;
    require!(
        (1..=8).contains(&nul_pos),
        name0[..nul_pos].iter().all(|&b| (0x20..=0x7E).contains(&b))
    );

    let nverts = be16(prefix, OBJ0_VERTS_OFF)? as usize;
    let nf_off = OBJ0_VERTS_DATA_OFF.checked_add(nverts.checked_mul(6)?)?;
    require!((nf_off as u64).checked_add(2)? <= size);

    if nf_off + 2 <= prefix.len() {
        let nfaces = be16(prefix, nf_off)? as u64;
        let obj0_end = (nf_off as u64).checked_add(2)?.checked_add(nfaces.checked_mul(8)?)?;
        if obj_count == 1 {
            require!(obj0_end == size);
        } else {
            require!(obj0_end < size);
        }
    }

    Some(Label::AtariCad3d2)
}

/// Atlus Persona/Metaphor GFS0 3D Model Container (`.gmd` / `.gfs`, `"GFS0"`, 2016 Atlus).
pub(crate) fn verify_atlus_gfs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ATLUS_GFS_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"GFS0"),
        (0x0100_0000..=0x0250_0000).contains(&be32(prefix, 4)?),
        (1..=16).contains(&be32(prefix, 8)?),
        be32(prefix, 12) == Some(0)
    );
    Some(Label::AtlusGfs)
}

/// BioWare Eclipse / OC3 FaceFX Facial Animation (.fxm, 2009) BioWare Edmonton.
pub(crate) fn verify_bioware_fxm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BIOWARE_FXM_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"BIOE"),
        get_slice(prefix, 8, 28)? == b"\x10\0\0\0Bioware Edmonton",
        le32(prefix, 28) == Some(24)
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::BiowareFxm)
}

/// Blizzard Entertainment StarCraft II / Heroes of the Storm M3 3D Model (`.m3`).
pub(crate) fn verify_blizzard_m3(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BLIZZARD_M3_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"43DM") || starts(prefix, b"33DM"),
        le32(prefix, BLIZZARD_M3_OFS_OFFSET)? >= 32,
        le32(prefix, BLIZZARD_M3_COUNT_OFFSET)? >= 1,
        le32(prefix, BLIZZARD_M3_MODL_REF_OFFSET) == Some(1),
        le32(prefix, BLIZZARD_M3_PAD_OFFSET) == Some(0xAAAA_AAAA)
    );
    Some(Label::BlizzardM3)
}

/// BodySlide and Outfit Studio Slider Data file (`.osd`).
pub(crate) fn verify_bodyslide_osd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BODYSLIDE_OSD_SIZE,
        prefix.len() >= 20,
        starts(prefix, BODYSLIDE_OSD_MAGIC),
        le32(prefix, BODYSLIDE_COUNT_OFFSET)? >= 1
    );
    let name_len = u8_at(prefix, BODYSLIDE_NAME_LEN_OFFSET)? as usize;
    require!(
        (1..=120).contains(&name_len),
        13 + name_len.min(16) <= prefix.len(),
        get_slice(prefix, 13, 13 + name_len.min(16))?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::BodyslideOsd)
}

/// Bohemia Interactive Real Virtuality P3D 3D Model (`.p3d`, `"MLOD"`/`"ODOL"`, 2001 Bohemia Interactive).
pub(crate) fn verify_bohemia_p3d(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BOHEMIA_P3D_SIZE, prefix.len() >= 16);
    if starts(prefix, b"MLOD") {
        require!(
            le32(prefix, 4) == Some(0x101),
            (1..=128).contains(&le32(prefix, 8)?),
            eq_at(prefix, 12, b"P3DM")
        );
        return Some(Label::BohemiaP3d);
    }
    if starts(prefix, b"ODOL") {
        require!((7..=80).contains(&le32(prefix, 4)?), has_non_ascii_text_byte(prefix.get(..16)?));
        return Some(Label::BohemiaP3d);
    }
    None
}

/// Bohemia Interactive Real Virtuality RTM Skeletal Animation (`.rtm`, `"RTM_0101"`, 2001 Bohemia Interactive).
pub(crate) fn verify_bohemia_rtm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BOHEMIA_RTM_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"RTM_0101") || starts(prefix, b"RTM_0100"),
        (1..=100_000).contains(&le32(prefix, 20)?),
        (1..=2048).contains(&le32(prefix, 24)?)
    );
    Some(Label::BohemiaRtm)
}

/// Brother / Babylock / Bernina PES Embroidery Design (.pes).
pub(crate) fn verify_brother_pes(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BROTHER_PES_SIZE
            && prefix.len() >= 16
            && starts(prefix, b"#PES")
            && get_slice(prefix, 4, 8)?.iter().all(|b| b.is_ascii_digit())
            && has_non_ascii_text_byte(prefix.get(8..16)?)
    );
    let pec_off = le32(prefix, 8)? as u64;
    require!((16..size).contains(&pec_off));
    Some(Label::BrotherPes)
}

/// Valve Source / Quake II / III Binary Space Partitioning Map (.bsp).
pub(crate) fn verify_bsp_map(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BSP_MAP_SIZE
            && prefix.len() >= 24
            && matches!(get_slice(prefix, 0, 4)?, b"VBSP" | b"IBSP" | b"RBSP" | b"QBSP")
    );
    let version = le32(prefix, 4)?;
    let lump0_off = le32(prefix, 8)? as u64;
    let lump0_len = le32(prefix, 12)? as u64;
    require!((17..=48).contains(&version) && lump0_off + lump0_len <= size);
    Some(Label::Bsp)
}

/// TQ Digital C3 Engine 3D Mesh / Physics / Motion Container (`.c3`, 2003, NetDragon / TQ Digital).
pub(crate) fn verify_c3_maxfile(prefix: &[u8], size: u64) -> Option<Label> {
    const C3_MAXFILE_MAGIC: &[u8; 16] = b"MAXFILE C3 00001"; // @0..16: C3 MAXFILE header magic + version

    require!(
        size >= MIN_C3_MAXFILE_SIZE,
        prefix.len() >= 28,
        starts(prefix, C3_MAXFILE_MAGIC),
        get_slice(prefix, 16, 20)?
            .iter()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || *b == b' '),
        has_non_ascii_text_byte(prefix.get(20..28)?)
    );
    Some(Label::C3Maxfile)
}

/// Cal3D 3D Character Skeleton / Mesh / Animation / Material (.csf / .cmf / .caf / .crf).
pub(crate) fn verify_cal3d(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CAL3D_SIZE
            && prefix.len() >= 12
            && matches!(get_slice(prefix, 0, 4)?, b"CSF\0" | b"CMF\0" | b"CAF\0" | b"CRF\0")
    );
    let version = le32(prefix, 4)?;
    let count_or_dur = le32(prefix, 8)?;
    require!((700..=1300).contains(&version) && count_or_dur > 0);
    Some(Label::Cal3d)
}

/// Capcom MT Framework Material Library (`.mrl`, `"MRL\0"`, 2006 Capcom).
pub(crate) fn verify_capcom_mt_mrl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CAPCOM_MT_MRL_SIZE,
        prefix.len() >= 24,
        starts(prefix, CAPCOM_MT_MRL_MAGIC),
        (1..=64).contains(&le32(prefix, 4)?),
        (1..=10_000).contains(&le32(prefix, 8)?),
        le32(prefix, 20)? >= 24
    );
    Some(Label::CapcomMtMrl)
}

/// OGC / Cesium 3D Tiles Batched 3D Model file (`.b3dm`).
pub(crate) fn verify_cesium_b3dm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CESIUM_B3DM_SIZE,
        prefix.len() >= 28,
        starts(prefix, CESIUM_B3DM_MAGIC),
        le32(prefix, CESIUM_BYTE_LEN_OFFSET)? >= 28,
        le32(prefix, CESIUM_FT_JSON_LEN_OFFSET)? <= le32(prefix, CESIUM_BYTE_LEN_OFFSET)?
    );
    Some(Label::CesiumB3dm)
}

/// Cesium 3D Tiles Geometry Tile (`.geom`) 28-byte header + JSON feature table.
pub(crate) fn verify_cesium_geom(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_CESIUM_GEOM_SIZE, prefix.len() >= 30, starts(prefix, b"geom\x01\0\0\0"));
    let byte_len = le32(prefix, 8)? as u64;
    let ft_json = le32(prefix, 12)? as u64;
    let ft_bin = le32(prefix, 16)? as u64;
    let bt_json = le32(prefix, 20)? as u64;
    let bt_bin = le32(prefix, 24)? as u64;
    let total = 28u64
        .checked_add(ft_json)?
        .checked_add(ft_bin)?
        .checked_add(bt_json)?
        .checked_add(bt_bin)?;
    require!(
        byte_len == size,
        total == size,
        ft_json >= 2,
        u8_at(prefix, 28)? == b'{',
        u8_at(prefix, 29).unwrap_or(0) == b'"'
            || u8_at(prefix, 29).unwrap_or(0) == b'}'
            || u8_at(prefix, 29).unwrap_or(0).is_ascii_whitespace()
    );
    Some(Label::CesiumGeom)
}

/// CTM v2 Tile Map (.ctm).
pub(crate) fn verify_ctm2_map(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CTM2_MAP_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"CTM\x02\x01\x0e\x06\x08\0\x01"),
        le64(prefix, 29) == Some(u64::MAX)
    );
    Some(Label::Ctm2Map)
}

/// Direct3D 32-Bit Triangle Mesh Index Buffer (.ib).
pub(crate) fn verify_d3d_index_buffer_ib(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_D3D_INDEX_BUFFER_IB_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 12)? == hex2bin!("00 00 00 00 01 00 00 00 02 00 00 00"),
        get_slice(prefix, 13, 16)? == hex2bin!("00 00 00"),
        get_slice(prefix, 17, 20)? == hex2bin!("00 00 00"),
        get_slice(prefix, 21, 24)? == hex2bin!("00 00 00"),
        get_slice(prefix, 25, 28)? == hex2bin!("00 00 00"),
        get_slice(prefix, 29, 32)? == hex2bin!("00 00 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::D3dIndexBufferIb)
}

/// DCS World / Eagle Dynamics 3D Model (`.edm`, `"EDM"` + `model::`).
pub(crate) fn verify_dcs_world_edm(prefix: &[u8], size: u64) -> Option<Label> {
    const EDM_PREFIX: &[u8; 3] = b"EDM"; // @0..3
    const EDM_MODEL_TAG: &[u8; 7] = b"model::"; // @9..16

    require!(
        size >= MIN_DCS_WORLD_EDM_SIZE,
        prefix.len() >= 16,
        starts(prefix, EDM_PREFIX),
        (6..=16).contains(&u8_at(prefix, 3)?), // @3: EDM format version
        u8_at(prefix, 4)? == 0,                // @4: high byte of u16 version
        (16..=65536).contains(&le32(prefix, 5)?), // @5..9: string table length
        eq_at(prefix, 9, EDM_MODEL_TAG)
    );
    Some(Label::DcsWorldEdm)
}

/// Grin / Overkill Diesel Engine Skeletal Animation (.gai).
pub(crate) fn verify_diesel_gai(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DIESEL_GAI_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"gai\0\x01\0\0\0\0\0\0\0\0\0\0\0"),
        le32(prefix, 16) == Some(20),
        le32(prefix, 20) == Some(20)
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::DieselGai)
}

/// Electronic Arts Sports EBO 3D Stadium/Player Mesh (.ebo, 2002) EA Sports.
pub(crate) fn verify_ea_ebo(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EA_EBO_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"EBO\0\x11\0\0\0"),
        le32(prefix, 8).unwrap_or(0) >= 0x60,
        le32(prefix, 16) == Some(0x60)
    );
    Some(Label::EaEbo)
}

/// EDF 3D Bounding/Scale Mesh (.edf).
pub(crate) fn verify_edf_mesh(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EDF_MESH_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"EDF\0"),
        le32(prefix, 28) == Some(1),
        le32(prefix, 32) == Some(0),
        get_slice(prefix, 36, 48)? == b"\0\0\x80?\0\0\x80?\0\0\x80?"
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::EdfMesh)
}

/// EPLYBNDS Polygon Bounding Model (`"EPLYBNDS"`).
pub(crate) fn verify_eplybnds_model(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EPLYBNDS_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"EPLYBNDS"),
        has_non_ascii_text_byte(prefix.get(8..16)?)
    );
    Some(Label::EplybndsModel)
}

/// EZNEC Antenna Modeling Description (.ez, Roy Lewallen W7EL).
pub(crate) fn verify_eznec_ez(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EZNEC_EZ_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"\x08\0\x01\0R"),
        get_slice(prefix, 9, 18)? == b"E\0\0\0\0\0\0\x80?"
    );
    require!(has_non_ascii_text_byte(prefix.get(..28)?));
    Some(Label::EznecEz)
}

/// Singular Inversions FaceGen / Creation Engine TRI Head Morph Mesh (.tri, 2003) Singular Inversions.
pub(crate) fn verify_facegen_tri(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FACEGEN_TRI_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"FRTRI003"),
        le32(prefix, 8).unwrap_or(0) > 0,
        le32(prefix, 12).unwrap_or(0) > 0
    );
    Some(Label::FacegenTri)
}

/// Nihon Falcom Trails / Kuro Engine Binary JSON Model Info (.mi).
pub(crate) fn verify_falcom_mi(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FALCOM_MI_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"JSON\0\0\0\0"),
        get_slice(prefix, 16, 21)? == b"\xff\xff\xff\xff\0",
        contains(prefix.get(21..prefix.len().min(40))?, b"Bounding\0")
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::FalcomMi)
}

/// Google Filament Compiled Material Package (`.filamat`, `"SREV_TAM\x04\0\0\0"`).
pub(crate) fn verify_filament_material(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FILAMENT_MATERIAL_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"SREV_TAM\x04\0\0\0"),
        (1..=200).contains(&le32(prefix, 12)?),
        eq_at(prefix, 16, b"TAEF_TAM\x01\0\0\0")
    );
    Some(Label::FilamentFilamat)
}

/// Flight Simulator Tagged Aircraft Descriptor (.air).
pub(crate) fn verify_flightsim_name_air(prefix: &[u8], size: u64) -> Option<Label> {
    let n = prefix.get(4).copied().unwrap_or(0) as usize;
    require!(
        size >= MIN_FLIGHTSIM_NAME_AIR_SIZE,
        prefix.len() >= 48,
        starts(prefix, hex2bin!("01 00 00 00")),
        (8..=36).contains(&n),
        get_slice(prefix, 5, 8)? == b"\0\0\0",
        get_slice(prefix, 8, 8 + n - 1)?
            .iter()
            .all(|&b| (0x20..=0x7E).contains(&b) || b == b'\r' || b == b'\n'),
        matches!(get_slice(prefix, 8 + n, 12 + n)?, b"\x02\0\0\0" | b"\x04\0\0\0")
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::FlightsimNameAir)
}

/// ANSYS Fluent Interpolation Field Data (.ip).
pub(crate) fn verify_fluent_interp_ip(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FLUENT_INTERP_IP_SIZE,
        prefix.len() >= 56,
        get_slice(prefix, 0, 4)? == hex2bin!("35 0A 32 0A"),
        get_slice(prefix, 5, 29)?
            == hex2bin!("30 30 30 0A 33 0A 70 72 65 73 73 75 72 65 0A 78 2D 76 65 6C 6F 63 69 74"),
        get_slice(prefix, 49, 51)? == hex2bin!("07 40")
    );
    require!(has_non_ascii_text_byte(prefix.get(..56)?));
    Some(Label::FluentInterpIp)
}

/// FromSoftware FLVER 3D Model (`.flver`, `"FLVER\0L\0"` / `"FLVER\0B\0"`, 2009 FromSoftware).
pub(crate) fn verify_fromsoft_flver(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FROMSOFT_FLVER_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"FLVER\0"),
        matches!(get_slice(prefix, 6, 8)?, b"L\0" | b"B\0")
    );
    let (ver, data_off) = if get_slice(prefix, 6, 8)? == b"L\0" {
        (le32(prefix, 8)?, le32(prefix, 12)? as u64)
    } else {
        (be32(prefix, 8)?, be32(prefix, 12)? as u64)
    };
    require!((0x10..=0x2_0020).contains(&ver), data_off >= 64);
    Some(Label::FromsoftFlver)
}

/// FromSoftware TimeAct Animation Event Table (`.tae`, `"TAE "`, 2009 FromSoftware).
pub(crate) fn verify_fromsoft_tae(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FROMSOFT_TAE_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"TAE "),
        u8_at(prefix, 4)? <= 1,
        u8_at(prefix, 5)? == 0,
        u8_at(prefix, 6)? <= 1,
        u8_at(prefix, 7)? == 0
    );
    let ver_ok = matches!(be32(prefix, 8)?, 0x0001_000B..=0x0001_000F)
        || matches!(le32(prefix, 8)?, 0x0001_000B..=0x0001_000F);
    let sz_ok = be32(prefix, 12)? >= 64 || le32(prefix, 12)? >= 64;
    require!(ver_ok, sz_ok);
    Some(Label::FromsoftTae)
}

/// NetImmerse / Gamebryo 3D Model (.nif).
///
/// Supports:
/// - Standard `"Gamebryo File Format, Version ..."` / `"NetImmerse File Format, Version ..."` with `u32 LE` binary version after `\n` (`>= 4.0`).
/// - Versionless `"Gamebryo File Format\n"` (`@0..21`) with `u32 LE` binary version at `@21..25`.
/// - Legacy NetImmerse 2.3–3.1 multi-line copyright header (`"NetImmerse File Format, Version "` + `"Numerical Design Limited"` + binary scene graph).
pub(crate) fn verify_gamebryo_nif(prefix: &[u8], size: u64) -> Option<Label> {
    const GAMEBRYO_VER_PREFIX: &[u8; 30] = b"Gamebryo File Format, Version ";
    const GAMEBRYO_NL_PREFIX: &[u8; 21] = b"Gamebryo File Format\n";
    const NETIMMERSE_VER_PREFIX: &[u8; 32] = b"NetImmerse File Format, Version ";
    const NDL_COPYRIGHT: &[u8; 24] = b"Numerical Design Limited";

    require!(size >= MIN_GAMEBRYO_NIF_SIZE, prefix.len() >= 48);
    if starts(prefix, GAMEBRYO_NL_PREFIX) {
        let bin_ver = le32(prefix, GAMEBRYO_NL_PREFIX.len())?;
        require!((0x0200_0000..=0x3000_0000).contains(&bin_ver));
        return Some(Label::Nif);
    }
    let has_ver_prefix =
        starts(prefix, GAMEBRYO_VER_PREFIX) || starts(prefix, NETIMMERSE_VER_PREFIX);
    require!(has_ver_prefix);
    let scan_end = prefix.len().min(64);
    let nl_pos =
        get_slice(prefix, 30, scan_end)?.iter().position(|&b| b == b'\n').map(|p| 30 + p)?;
    require!(prefix.len() >= nl_pos + 5);
    let bin_ver = le32(prefix, nl_pos + 1)?;
    if (0x0200_0000..=0x3000_0000).contains(&bin_ver) {
        return Some(Label::Nif);
    }
    // Legacy NetImmerse 2.3 / 3.0 / 3.1 multi-line copyright header followed by binary node stream.
    require!(
        starts(prefix, NETIMMERSE_VER_PREFIX),
        contains(prefix.get(32..prefix.len().min(192))?, NDL_COPYRIGHT),
        has_non_ascii_text_byte(prefix.get(32..prefix.len().min(256))?)
    );
    Some(Label::Nif)
}

/// GEO 3D Voxel Chunk (.gtx).
pub(crate) fn verify_geo_gtx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GEO_GTX_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"GEO\0\0\x02\0\0\0\0\0\0\0\0\0"),
        (1..=256).contains(&le16(prefix, 15).unwrap_or(0)),
        (1..=256).contains(&le16(prefix, 17).unwrap_or(0)),
        (1..=256).contains(&le16(prefix, 19).unwrap_or(0))
    );
    Some(Label::GeoGtx)
}

/// Rockstar Games RenderWare GTA Animation Package (`.ifp`, `"ANPK"`/`"ANP3"`/`"ANCT"`, 2001 Rockstar Games).
pub(crate) fn verify_gta_ifp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_GTA_IFP_SIZE, prefix.len() >= 32);
    let ok = match get_slice(prefix, 0, 4)? {
        b"ANPK" => le32(prefix, 4)? >= 16 && eq_at(prefix, 8, b"INFO"),
        b"ANP3" => {
            le32(prefix, 4)? >= 24
                && u8_at(prefix, 8)?.is_ascii_graphic()
                && prefix.get(8..32)?.contains(&0)
                && has_non_ascii_text_byte(prefix.get(..32)?)
        }
        b"ANCT" => le32(prefix, 4)? >= 16 && eq_at(prefix, 8, b"BLOC"),
        _ => false,
    };
    require!(ok);
    Some(Label::GtaIfp)
}

/// Havok Physics / Animation Binary Packfile or Tagfile (`.hkx` / `.hkt`).
///
/// Header layouts:
/// - Binary Packfile: `@0..8` `57 E0 E0 57 10 C0 C0 10`, `@12..16` `u32` LE/BE `file_version` (`4..=12`)
/// - Binary Tagfile: `@0..8` `1E 0D B0 CA CE FA 11 D0`
pub(crate) fn verify_havok_hkx(prefix: &[u8], size: u64) -> Option<Label> {
    const PACKFILE_MAGIC: &[u8; 8] = hex2bin!("57 E0 E0 57 10 C0 C0 10"); // @0..8: Havok packfile signature
    const TAGFILE_MAGIC: &[u8; 8] = hex2bin!("1E 0D B0 CA CE FA 11 D0"); // @0..8: Havok tagfile signature
    const VERSION_OFF: usize = 12; // @12..16: u32 LE or BE packfile version (4..=12)

    require!(size >= MIN_HAVOK_HKX_SIZE, prefix.len() >= 16);
    if starts(prefix, PACKFILE_MAGIC) {
        let ver_le = le32(prefix, VERSION_OFF)?;
        let ver_be = be32(prefix, VERSION_OFF)?;
        require!((4..=12).contains(&ver_le) || (4..=12).contains(&ver_be));
        return Some(Label::HavokHkx);
    }
    if starts(prefix, TAGFILE_MAGIC) {
        return Some(Label::HavokHkx);
    }
    None
}

/// Relic Entertainment Homeworld Mesh Extension Data (`.mex`) file.
pub(crate) fn verify_homeworld_mex_data(prefix: &[u8], size: u64) -> Option<Label> {
    const MEX_MAGIC: &[u8; 10] = bytes!(b"mannngo", &[0x00, 0x02, 0x01]); // @0..10: "mannngo\0" + version 2.1

    require!(size >= MIN_HOMEWORLD_MEX_SIZE, prefix.len() >= 16, starts(prefix, MEX_MAGIC));
    Some(Label::HomeworldMex)
}

/// Tech Soft 3D HOOPS Stream File / SolidWorks eDrawings stream (`.hsf` / `.easm`).
pub(crate) fn verify_hoops_hsf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_HOOPS_HSF_SIZE,
        prefix.len() >= 32,
        starts(prefix, HOOPS_HSF_MAGIC),
        u8_at(prefix, 8)?.is_ascii_digit(),
        scan_range(prefix, 8, 64, b"\n"),
        has_non_ascii_text_byte(prefix.get(12..prefix.len().min(64))?)
    );
    Some(Label::HoopsHsf)
}

/// id Software Quake / Valve Half-Life IDSP 2D/3D Sprite (.spr, 1996) id Software / Valve.
/// - Quake 1 (`version == 1`): `@8..12` `type` (`0..=5`), `@12..16` `f32 LE boundingradius`, `@16..20` `width`, `@20..24` `height`, `@24..28` `numframes`.
/// - Half-Life (`version == 2`): `@8..12` `type` (`0..=5`), `@12..16` `texFormat` (`0..=4`), `@16..20` `boundingradius`, `@20..24` `width`, `@24..28` `height`, `@28..32` `numframes`.
pub(crate) fn verify_idsp_spr(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4..8: u32 LE sprite format version (1 = Quake, 2 = Half-Life)
    const TYPE_OFF: usize = 8; // @8..12: u32 LE sprite orientation type (0..=5)
    require!(
        size >= MIN_IDSP_SPR_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"IDSP"),
        le32(prefix, TYPE_OFF).unwrap_or(99) <= 5
    );
    let version = le32(prefix, VERSION_OFF)?;
    let (width, height) = match version {
        1 => {
            let radius = f32::from_bits(le32(prefix, 12)?);
            require!(radius.is_finite() && (0.0..=65536.0).contains(&radius));
            (le32(prefix, 16)?, le32(prefix, 20)?)
        }
        2 => {
            require!(le32(prefix, 12)? <= 4);
            (le32(prefix, 20)?, le32(prefix, 24)?)
        }
        _ => return None,
    };
    require!((1..=4096).contains(&width), (1..=4096).contains(&height));
    Some(Label::IdspSpr)
}

/// Illusion Softworks LS3D / I3D 3D Model (`.i3d`, Mafia / Hidden & Dangerous 2, 2002 Illusion Softworks).
pub(crate) fn verify_illusion_i3d(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ILLUSION_I3D_SIZE,
        prefix.len() >= 16,
        starts(prefix, hex2bin!("CD AB 0A 00 00 00 DD CC BB AA 00 AA"))
    );
    Some(Label::IllusionI3d)
}

/// J.A.C.K. / Jackhammer Binary Map (`.jmf`, 2006 Jackhammer team / Chain Studios).
///
/// Layout: `@0 "JHMF"`, `@4 u32 LE version` (121 / 122), `@8 u32 LE export_path_count`, then
/// `export_path_count` × (`u32 LE len` + `len`
/// printable path bytes, no NUL). For v121 the next `u32 LE` is the editor group count.
pub(crate) fn verify_jackhammer_jmf(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION: usize = 4; // @4: u32 LE format version
    const EXPORT_PATH_COUNT: usize = 8; // @8: u32 LE number of recent export (.map) paths
    const FIRST_EXPORT_PATH: usize = 12; // @12: first (u32 LE len + chars) export path
    const MIN_VERSION: u32 = 121; // J.A.C.K. 1.1 binary map
    const MAX_VERSION: u32 = 122; // J.A.C.K. 1.1.x with viewport background images
    const MAX_EXPORT_PATHS: u32 = 64;
    const MAX_EXPORT_PATH_LEN: usize = 1024;
    const MAX_GROUPS: u32 = 0x10000;

    require!(
        size >= MIN_JACKHAMMER_JMF_SIZE,
        prefix.len() >= 16,
        starts(prefix, JACKHAMMER_JMF_MAGIC)
    );
    let version = le32(prefix, VERSION)?;
    let path_count = le32(prefix, EXPORT_PATH_COUNT)?;
    require!((MIN_VERSION..=MAX_VERSION).contains(&version), path_count <= MAX_EXPORT_PATHS);

    let mut pos = FIRST_EXPORT_PATH;
    for _ in 0..path_count {
        let Some(len) = le32(prefix, pos) else {
            break; // path table extends beyond the prefix window
        };
        let len = len as usize;
        require!((1..=MAX_EXPORT_PATH_LEN).contains(&len));
        let Some(path) = get_slice(prefix, pos + 4, pos + 4 + len) else {
            break;
        };
        require!(path.iter().all(|&b| (0x20..=0x7E).contains(&b)));
        pos += 4 + len;
        require!(pos as u64 <= size);
    }
    if version == MIN_VERSION
        && let Some(groups) = le32(prefix, pos)
    {
        require!(groups <= MAX_GROUPS);
    }
    Some(Label::JackhammerJmf)
}

/// ASCON Kompas-3D CAD Drawing / Fragment / Model (`.cdw` / `.frw` / `.m3d`, 1989 ASCON).
pub(crate) fn verify_kompas_3d(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_KOMPAS_3D_SIZE && prefix.len() >= 32 && starts(prefix, b"KF"));
    let sub = le16(prefix, 2)?;
    let ver_major = u8_at(prefix, 5)?;
    let flags = le16(prefix, 6)?;
    let hdr_len_or_size = le32(prefix, 8)? as u64;
    let sec_count = le32(prefix, 12)?;
    let ver_a = le32(prefix, 16)?;
    let ver_b = le32(prefix, 20)?;
    require!(
        sub != 0
            && u8_at(prefix, 4)? == 0
            && (1..=32).contains(&ver_major)
            && flags == 0x1000
            && (64..=size).contains(&hdr_len_or_size)
            && (1..=65535).contains(&sec_count)
            && ((matches!(ver_a, 1 | 2) && ver_b == 0) || (ver_a == 0 && matches!(ver_b, 1 | 2)))
    );
    Some(Label::Kompas3d)
}

/// Squad Kerbal Space Program `.mu` Part Model file.
pub(crate) fn verify_ksp_mu(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KSP_MU_SIZE,
        prefix.len() >= 16,
        starts(prefix, KSP_MU_MAGIC),
        le32(prefix, KSP_MU_VERSION_OFFSET)? <= 8
    );
    let name_len = u8_at(prefix, KSP_MU_NAME_LEN_OFFSET)? as usize;
    require!(
        (1..=64).contains(&name_len),
        9 + name_len <= prefix.len(),
        get_slice(prefix, 9, 9 + name_len)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::KspMu)
}

/// LayaAir 3D Engine Binary Mesh (`.lm`, `"LAYAMODEL:"`).
pub(crate) fn verify_layamodel_0501_lm(prefix: &[u8], size: u64) -> Option<Label> {
    const LAYAMODEL_PREFIX: &[u8; 10] = b"LAYAMODEL:"; // @2..12

    require!(size >= MIN_LAYAMODEL_0501_LM_SIZE, prefix.len() >= 16);
    let str_len = le16(prefix, 0)? as usize;
    require!(
        (12..=40).contains(&str_len),
        prefix.len() >= 2 + str_len,
        eq_at(prefix, 2, LAYAMODEL_PREFIX),
        get_slice(prefix, 12, 2 + str_len)?.iter().all(u8::is_ascii_graphic)
    );
    Some(Label::LayaairMeshLm)
}

/// LFM2 3D Material Definition (.lfmat).
pub(crate) fn verify_lfm2_lfmat(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LFM2_LFMAT_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"LFM2\x02\0\0\0 \0\0\0")
    );
    Some(Label::Lfm2Lfmat)
}

/// libGDX Binary 3D Model file (`.g3db`).
pub(crate) fn verify_libgdx_g3db(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_LIBGDX_G3DB_SIZE, prefix.len() >= 24, starts(prefix, LIBGDX_G3DB_MAGIC));
    Some(Label::LibgdxG3db)
}

/// NCSoft Lineage II Encrypted Unreal Engine Package (UTF-16LE `"Lineage2VerNNN"`).
pub(crate) fn verify_lineage2_ver111_usx(prefix: &[u8], size: u64) -> Option<Label> {
    const LINEAGE2_VER_UTF16LE: &[u8; 22] = b"L\0i\0n\0e\0a\0g\0e\x002\0V\0e\0r\0"; // @0..22

    require!(
        size >= MIN_LINEAGE2_VER111_USX_SIZE,
        prefix.len() >= 28,
        starts(prefix, LINEAGE2_VER_UTF16LE),
        [22usize, 24, 26].into_iter().all(|o| u8_at(prefix, o).unwrap_or(0).is_ascii_digit()
            && u8_at(prefix, o + 1).unwrap_or(0) == 0)
    );
    Some(Label::Lineage2UnrealPkg)
}

/// Live2D Cubism 3+ Editor Model Project (.cmo3 / .can3, 2017) Live2D.
pub(crate) fn verify_live2d_cmo3(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LIVE2D_CMO3_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"CAFF\0\0\0----\0\0\0")
    );
    Some(Label::Live2dCmo3)
}

/// ephtracy MagicaVoxel Model (`.vox`, `"VOX "` + `150 | 200` + `"MAIN"`).
pub(crate) fn verify_magicavoxel(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MAGICAVOXEL_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"VOX "),
        matches!(le32(prefix, 4), Some(150 | 200)),
        eq_at(prefix, 8, b"MAIN"),
        le32(prefix, 12) == Some(0)
    );
    Some(Label::MagicavoxelVox)
}

/// Mandelbulb 3D Fractal Parameter Preset (.m3p).
pub(crate) fn verify_mandelbulb3d_m3p(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MANDELBULB3D_M3P_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 32)?
            == hex2bin!(
                "2C 00 00 00 E0 01 00 00 68 01 00 00 3C 00 00 00 01 00 00 00 00 00 00 00 00 00 00 C0 00 00 00 00"
            )
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::Mandelbulb3dM3p)
}

/// CNC Software Mastercam v9 CAD/CAM Part (.mc9).
pub(crate) fn verify_mastercam_v9_mc9(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MASTERCAM_V9_MC9_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 12)? == hex2bin!("78 00 00 00 00 02 00 00 00 00 00 00"),
        get_slice(prefix, 18, 20)? == hex2bin!("00 00"),
        get_slice(prefix, 24, 26)? == hex2bin!("00 00"),
        get_slice(prefix, 27, 32)? == hex2bin!("00 FF FF FF 7F")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::MastercamV9Mc9)
}

/// MDXELM DTPS 3D Part Element (.g2p).
pub(crate) fn verify_mdxelm_g2p(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MDXELM_G2P_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"_MDXELM_DTPS\x03\0\0\0"),
        le32(prefix, 16).unwrap_or(0) > 0
    );
    Some(Label::MdxelmG2p)
}

/// MikuMikuDance PMD 1.0 Polygon Model Data (`.pmd`, `"Pmd\x00\x00\x80\x3F"`, 2008 Yu Higuchi).
pub(crate) fn verify_mmd_pmd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MMD_PMD_SIZE,
        prefix.len() >= 16,
        starts(prefix, hex2bin!("50 6D 64 00 00 80 3F"))
    );
    Some(Label::MmdPmd)
}

/// MikuMikuDance PMX 2.0/2.1 Polygon Model eXtended (`.pmx`, `"PMX "`, 2010 Yu Higuchi).
pub(crate) fn verify_mmd_pmx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MMD_PMX_SIZE,
        prefix.len() >= 17,
        starts(prefix, b"PMX "),
        eq_at(prefix, 4, hex2bin!("00 00 00 40")) || eq_at(prefix, 4, hex2bin!("66 66 06 40")),
        u8_at(prefix, 8)? == 8,
        u8_at(prefix, 9)? <= 1
    );
    Some(Label::MmdPmx)
}

/// MikuMikuDance Vocaloid Motion Data Animation (`.vmd`, 2008 Yu Higuchi).
pub(crate) fn verify_mmd_vmd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MMD_VMD_SIZE,
        prefix.len() >= 30,
        starts(prefix, b"Vocaloid Motion Data 0002\0")
            || starts(prefix, b"Vocaloid Motion Data file\0")
    );
    Some(Label::MmdVmd)
}

/// Bandai Namco Smash Bros. Ultimate NuLibrary Binary Resource (`"HBSS"`).
pub(crate) fn verify_namco_nu_binary(prefix: &[u8], size: u64) -> Option<Label> {
    const HBSS_MAGIC: &[u8; 16] = bytes!(b"HBSS", le32_bytes!(0x40), le64_bytes!(0)); // @0..16: "HBSS" + header size 0x40 + 8 zero bytes
    const INNER_TAG_RANGE: core::ops::Range<usize> = 16..20; // @16..20: 4-char uppercase block tag (MINA, MATL, MESH, MODL, SKEL, SHDR, HLPB)
    const MAJOR_VER_OFFSET: usize = 20; // @20..22: u16 LE major version (1 or 2)

    require!(
        size >= MIN_NAMCO_HBSS_SIZE,
        prefix.len() >= 24,
        starts(prefix, HBSS_MAGIC),
        get_slice(prefix, INNER_TAG_RANGE.start, INNER_TAG_RANGE.end)?
            .iter()
            .all(|&b| b.is_ascii_uppercase() || b.is_ascii_digit()),
        matches!(le16(prefix, MAJOR_VER_OFFSET), Some(1 | 2))
    );
    Some(Label::NamcoHbss)
}

/// Nexon Crazyracing KartRider 3D Model (.1s).
pub(crate) fn verify_nexon_kartrider_1s(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NEXON_KARTRIDER_1S_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"\xaa\x47\x3c\x03\x07\x0e\0\0\0\0\0\0\x01\0\0\0\xaa\x47\x3c\x03\x07\x0e\x01\0\x07\0\0\0")
    );
    require!(has_non_ascii_text_byte(prefix.get(..28)?));
    Some(Label::NexonKartrider1s)
}

/// Nintendo GameCube / Wii JSystem 3D Model / Animation (.bmd / .bdl / .bck / .btk / .brk).
pub(crate) fn verify_nintendo_j3d(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_NINTENDO_J3D_SIZE
            && prefix.len() >= 32
            && (starts(prefix, b"J3D1") || starts(prefix, b"J3D2"))
            && get_slice(prefix, 4, 8)?.iter().all(|b| b.is_ascii_alphanumeric())
            && be32(prefix, 8)? as u64 == size
            && (1..=32).contains(&be32(prefix, 12)?)
    );
    Some(Label::J3d)
}

/// OGRE 3D Binary Mesh (`.mesh`, `00 10 "[MeshSerializer_v1."`).
pub(crate) fn verify_ogre_3d_mesh(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_OGRE_MESH_SIZE, starts(prefix, b"\x00\x10[MeshSerializer_v1."));
    Some(Label::OgreMesh)
}

/// Mentor Graphics PADS Layout Binary PCB Design (.pcb).
pub(crate) fn verify_pads_pcb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PADS_PCB_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\0\xff\x27\x20\x01\0\0\0\0\0"),
        get_slice(prefix, 12, 14)? == b"\x01\0",
        get_slice(prefix, 26, 34)? == b"\x4b\0\0\0\xb0\x04\0\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::PadsPcb)
}

/// 4D Paint: `"4D Paint Project File"` (full) or `"4D Paint demo version"` (demo builds).
pub(crate) fn verify_paint4d(prefix: &[u8], size: u64) -> Option<Label> {
    const KIND: usize = 9; // @9: "Project File" / "demo version"
    require!(size >= MIN_PAINT4D_SIZE, prefix.len() >= 24, starts(prefix, b"4D Paint "));
    require!(
        eq_at(prefix, KIND, b"Project File") || eq_at(prefix, KIND, b"demo version"),
        has_non_ascii_text_byte(prefix.get(21..prefix.len().min(32))?)
    );
    Some(Label::Paint4d)
}

/// Panasonic ISL_PANA_3DMAP 3D Calibration Map (.s3d) Panasonic.
pub(crate) fn verify_panasonic_s3d(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PANASONIC_S3D_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"ISL_PANA_3DMAP\0\0\x01\0\0\0"),
        le32(prefix, 20).unwrap_or(0) > 0
    );
    Some(Label::PanasonicS3d)
}

/// Paradox Interactive Clausewitz / Jomini Engine Binary 3D Mesh (`.mesh`, `"@@b@"`, 2013 Paradox Interactive).
pub(crate) fn verify_paradox_mesh(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PARADOX_MESH_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"@@b@!\x08pdxasset"),
        has_non_ascii_text_byte(prefix.get(..20)?)
    );
    Some(Label::ParadoxMesh)
}

/// Tama Software Pepakura Designer Legacy v1 3D Papercraft Model (.pdo, 2002) Tama Software.
pub(crate) fn verify_pepakura_pdi1(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PEPAKURA_PDI1_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"PDI1 \0\0\0"),
        (1..=16).contains(&le32(prefix, 8).unwrap_or(0)),
        le32(prefix, 12).unwrap_or(0) > 0
    );
    Some(Label::PepakuraPdi1)
}

/// Triglow Pictures Priston Tale 3D Model or Motion (`.smd` / `.smb`) file.
pub(crate) fn verify_priston_tale_smd_model(prefix: &[u8], size: u64) -> Option<Label> {
    const SMD_MODEL_MAGIC: &[u8; 24] = bytes!(b"SMD Model data Ver 0.62", &[0x00]); // @0..24: model header
    const SMD_MOTION_MAGIC: &[u8; 24] = b"SMD Motion data Ver 0.62"; // @0..24: motion header

    require!(
        size >= MIN_PRISTON_TALE_SMD_SIZE,
        prefix.len() >= 28,
        starts(prefix, SMD_MODEL_MAGIC) || starts(prefix, SMD_MOTION_MAGIC)
    );
    Some(Label::PristonTaleSmd)
}

/// Labcenter Proteus ISIS Circuit Schematic (.dsn, 1990) Labcenter Electronics.
pub(crate) fn verify_proteus_isis_dsn(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PROTEUS_ISIS_DSN_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"ISIS SCHEMATIC FILE\x1a\0\0")
    );
    Some(Label::ProteusIsisDsn)
}

/// Labcenter Proteus CAD Part Descriptor (.pdspart) Labcenter Electronics.
pub(crate) fn verify_proteus_pdspart(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PROTEUS_PDSPART_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"Proteus Part Descriptor 1.0\n"),
        has_non_ascii_text_byte(prefix.get(28..32)?)
    );
    Some(Label::ProteusPdspart)
}

/// id Software Quake 1 3D Model (.mdl).
pub(crate) fn verify_quake1_mdl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_QUAKE1_MDL_SIZE && prefix.len() >= 84);
    require!(starts(prefix, b"IDPO"));

    let version = le32(prefix, 4)?;
    require!(version == 6);

    let num_skins = i32::from_le_bytes(get_slice(prefix, 48, 52)?.try_into().ok()?);
    let skin_width = i32::from_le_bytes(get_slice(prefix, 52, 56)?.try_into().ok()?);
    let skin_height = i32::from_le_bytes(get_slice(prefix, 56, 60)?.try_into().ok()?);
    let num_verts = i32::from_le_bytes(get_slice(prefix, 60, 64)?.try_into().ok()?);
    let num_tris = i32::from_le_bytes(get_slice(prefix, 64, 68)?.try_into().ok()?);
    let num_frames = i32::from_le_bytes(get_slice(prefix, 68, 72)?.try_into().ok()?);
    let synctype = i32::from_le_bytes(get_slice(prefix, 72, 76)?.try_into().ok()?);

    require!((1..=256).contains(&num_skins));
    require!(skin_width > 0 && skin_width <= 4096);
    require!(skin_height > 0 && skin_height <= 4096);
    require!(num_verts > 0 && num_verts <= 4096);
    require!(num_tris > 0 && num_tris <= 8192);
    require!(num_frames > 0 && num_frames <= 1024);
    require!(synctype == 0 || synctype == 1);

    Some(Label::QuakeMdl)
}

/// id Software Quake II 3D Model (.md2).
pub(crate) fn verify_quake2_md2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_QUAKE2_MD2_SIZE && prefix.len() >= 68);
    require!(starts(prefix, b"IDP2"));

    let version = le32(prefix, 4)?;
    require!(version == 8);

    let skin_width = i32::from_le_bytes(get_slice(prefix, 8, 12)?.try_into().ok()?);
    let skin_height = i32::from_le_bytes(get_slice(prefix, 12, 16)?.try_into().ok()?);
    let frame_size = i32::from_le_bytes(get_slice(prefix, 16, 20)?.try_into().ok()?);
    let num_skins = i32::from_le_bytes(get_slice(prefix, 20, 24)?.try_into().ok()?);
    let num_xyz = i32::from_le_bytes(get_slice(prefix, 24, 28)?.try_into().ok()?);
    let num_st = i32::from_le_bytes(get_slice(prefix, 28, 32)?.try_into().ok()?);
    let num_tris = i32::from_le_bytes(get_slice(prefix, 32, 36)?.try_into().ok()?);
    let num_glcmds = i32::from_le_bytes(get_slice(prefix, 36, 40)?.try_into().ok()?);
    let num_frames = i32::from_le_bytes(get_slice(prefix, 40, 44)?.try_into().ok()?);
    let ofs_end = i32::from_le_bytes(get_slice(prefix, 64, 68)?.try_into().ok()?);

    require!((0..=4096).contains(&skin_width));
    require!((0..=4096).contains(&skin_height));
    require!(frame_size >= 0);
    require!((0..=256).contains(&num_skins));
    require!((0..=8192).contains(&num_xyz));
    require!((0..=8192).contains(&num_st));
    require!((0..=16384).contains(&num_tris));
    require!(num_glcmds >= 0);
    require!((0..=2048).contains(&num_frames));
    if ofs_end > 0 {
        require!((ofs_end as u64) <= size + 1024);
    }

    Some(Label::Md2)
}

/// Racer Simulation Dynamic Object Format 3D Model (.dof, 2001) Ruud van Gaal.
pub(crate) fn verify_racer_dof1(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RACER_DOF1_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"DOF1"),
        le32(prefix, 4).unwrap_or(0) > 0,
        matches!(get_slice(prefix, 8, 12)?, b"MATS" | b"GHDR" | b"GEOB")
    );
    Some(Label::RacerDof1)
}

/// Capcom RE Engine Physics Chain (`.chain.*`, `chan`).
pub(crate) fn verify_re_engine_chain(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RE_ENGINE_CHAIN_SIZE,
        prefix.len() >= 32,
        eq_at(prefix, 4, b"chan\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"),
        (1..=100).contains(&le32(prefix, 0).unwrap_or(0)),
        le32(prefix, 24).unwrap_or(0) >= 32
    );
    Some(Label::ReEngineChain)
}

/// Capcom RE Engine Motion Camera List (`clst`, `.mcamlist.*`).
pub(crate) fn verify_re_engine_mcamlist(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RE_ENGINE_MCAMLIST_SIZE,
        prefix.len() >= 40,
        eq_at(prefix, 4, b"clst\0\0\0\0\0\0\0\0"),
        (1..=100).contains(&le32(prefix, 0).unwrap_or(0)),
        (0x20..=0x400).contains(&le64(prefix, 16).unwrap_or(0)),
        le32(prefix, 32).unwrap_or(0) > 0
    );
    Some(Label::ReEngineMcamlist)
}

/// SCS Software Prism3D 3D Model Descriptor (`.pmd`, version 4).
pub(crate) fn verify_scs_pmd(prefix: &[u8], size: u64) -> Option<Label> {
    const PMD_VERSION: u32 = 4; // @0..4: Prism3D PMD format version 4
    const HDR_SIZE: u32 = 64; // @36..40: look_offset == 0x40 (fixed 64-byte header)

    require!(
        size >= MIN_SCS_PMD_SIZE,
        prefix.len() >= 64,
        le32(prefix, 0) == Some(PMD_VERSION),
        le32(prefix, 36) == Some(HDR_SIZE)
    );
    let mat_cnt = le32(prefix, 4)?;
    let look_cnt = le32(prefix, 8)?;
    let piece_cnt = le32(prefix, 12)?;
    let var_cnt = le32(prefix, 16)?;
    let part_cnt = le32(prefix, 20)?;
    let attr_cnt = le32(prefix, 24)?;
    let attr_val_sz = le32(prefix, 28)?;
    let mat_blk_sz = le32(prefix, 32)?;
    let look_off = HDR_SIZE;
    let var_off = le32(prefix, 40)?;
    let part_attr_off = le32(prefix, 44)?;
    let attr_val_off = le32(prefix, 48)?;
    let attr_off = le32(prefix, 52)?;
    let mat_off_tbl = le32(prefix, 56)?;
    let mat_data_off = le32(prefix, 60)?;

    let exp_var_off = look_off.checked_add(look_cnt.checked_mul(8)?)?;
    let exp_part_attr_off = var_off.checked_add(var_cnt.checked_mul(8)?)?;
    let exp_attr_off = part_attr_off.checked_add(part_cnt.checked_mul(8)?)?;
    let exp_mat_off_tbl = attr_val_off.checked_add(attr_val_sz)?;
    let exp_mat_data_off =
        mat_off_tbl.checked_add(look_cnt.checked_mul(mat_cnt)?.checked_mul(4)?)?;
    let exp_end = (mat_data_off as u64).checked_add(mat_blk_sz as u64)?;

    require!(
        (1..=4096).contains(&look_cnt),
        (1..=4096).contains(&mat_cnt),
        piece_cnt <= 65536,
        var_cnt <= 4096,
        part_cnt <= 4096,
        attr_cnt <= 65536,
        var_off == exp_var_off,
        part_attr_off == exp_part_attr_off,
        attr_off == exp_attr_off,
        attr_val_off >= attr_off,
        mat_off_tbl == exp_mat_off_tbl,
        mat_data_off == exp_mat_data_off,
        exp_end == size
    );
    Some(Label::ScsPmd)
}

/// SCS Software Prism3D Engine 3D Model Geometry (.pmg, 2012) SCS Software.
pub(crate) fn verify_scs_pmg(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SCS_PMG_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"\x15gmP"),
        le32(prefix, 4).unwrap_or(0) > 0,
        le32(prefix, 8).unwrap_or(0) > 0,
        le64(prefix, 16) == Some(0)
    );
    Some(Label::ScsPmg)
}

/// SCS Software Prism3D Engine Texture Object Descriptor (.tobj, 2012) SCS Software.
pub(crate) fn verify_scs_tobj(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SCS_TOBJ_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\x01\x0a\xb1\x70"),
        get_slice(prefix, 4, 20)? == [0u8; 16],
        get_slice(prefix, 20, 24)? == b"\x01\0\x02\0"
    );
    Some(Label::ScsTobj)
}

/// Sega / Ryu Ga Gotoku Studio Dragon Engine 3D Model (`.gmd`, `"GSGM"`, 2016 Sega).
pub(crate) fn verify_sega_dragon_engine_gmd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SEGA_DRAGON_GMD_SIZE,
        prefix.len() >= 20,
        starts(prefix, b"GSGM"),
        le32(prefix, 4)? <= 16,
        le16(prefix, 8)? <= 8,
        le16(prefix, 10)? <= 8,
        le32(prefix, 12)? >= 48,
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::SegaGsgm)
}

/// SEMI OASIS IC Layout Stream (`.oas`, `"%SEMI-OASIS\r\n\x01\x031.0"`).
pub(crate) fn verify_semi_oasis_stream(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SEMI_OASIS_SIZE, starts(prefix, b"%SEMI-OASIS\r\n\x01\x031.0"));
    Some(Label::SemiOasis)
}

/// Macromedia / Intel Shockwave 3D Scene (`.w3d`, `"IFX\0"`, 2001 Macromedia / Intel).
pub(crate) fn verify_shockwave_w3d(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SHOCKWAVE_W3D_SIZE,
        prefix.len() >= 20,
        starts(prefix, SHOCKWAVE_W3D_MAGIC),
        (4..=64).contains(&le32(prefix, 4)?),
        le32(prefix, 8)? <= 0x1000,
        le32(prefix, 12)? >= 24,
        u8_at(prefix, 19)? == 0xFF
    );
    Some(Label::ShockwaveW3d)
}

/// Sony PlayStation Portable GMO 3D Model (`.gmo`, 2004 Sony, `"OMG.00.1PSP\0\0\0\0\0"`).
///
/// Header layout:
/// - `@0..16`: `"OMG.00.1PSP\0\0\0\0\0"`
/// - `@16..18`: `u16 LE` root chunk type (`0x0002` = `GMO_FILE`)
/// - `@18..20`: `u16 LE` root chunk args offset (`8..=64`, typically `0x10`)
/// - `@20..24`: `u32 LE` root chunk next offset / size (`>= 16`)
pub(crate) fn verify_sony_psp_gmo(prefix: &[u8], size: u64) -> Option<Label> {
    const GMO_MAGIC: &[u8; 16] = b"OMG.00.1PSP\0\0\0\0\0"; // @0..16
    const CHUNK_TYPE_OFF: usize = 16; // @16..18: u16 LE GMO_FILE (0x0002)
    const ARGS_OFF: usize = 18; // @18..20: u16 LE chunk args offset (8..=64)
    const NEXT_OFF: usize = 20; // @20..24: u32 LE next chunk offset (>= 16)

    require!(
        size >= MIN_SONY_PSP_GMO_SIZE,
        prefix.len() >= 24,
        starts(prefix, GMO_MAGIC),
        le16(prefix, CHUNK_TYPE_OFF) == Some(0x0002),
        (8..=64).contains(&le16(prefix, ARGS_OFF)?),
        le32(prefix, NEXT_OFF)? >= 16
    );
    Some(Label::SonyPspGmo)
}

/// GSC Game World X-Ray Engine Compiled 3D Mesh (.ogf, v4).
pub(crate) fn verify_stalker_xray_ogf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_STALKER_XRAY_OGF_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 12)? == hex2bin!("01 00 00 00 2C 00 00 00 04 0A 00 00"),
        get_slice(prefix, 15, 17)? == hex2bin!("BE 00"),
        get_slice(prefix, 20, 21)? == hex2bin!("00"),
        get_slice(prefix, 27, 29)? == hex2bin!("3E 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::StalkerXrayOgf)
}

/// Akella Storm Engine 2.x 3D Geometry Model (.gm, 2003) Akella.
pub(crate) fn verify_storm_gm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_STORM_GM_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"50.1\x02\0\0\0"),
        (1..=16).contains(&le32(prefix, 16).unwrap_or(0)),
        le32(prefix, 20) == le32(prefix, 16),
        le32(prefix, 32) == Some(1)
    );
    require!(has_non_ascii_text_byte(prefix.get(..40)?));
    Some(Label::StormGm)
}

/// ACCEL EDA / TangoPRO Component Library V2.0 (`.lib`).
pub(crate) fn verify_tangopro_lib(prefix: &[u8], size: u64) -> Option<Label> {
    const TANGOPRO_WORDS: &[u8; 28] = hex2bin!(
        "56 00 00 00 01 00 00 00 02 00 00 00 41 00 00 00 4C 00 00 00 49 00 00 00 02 00 00 00"
    ); // @4..32: u32 LE tag words ('V', 1, 2, 'A', 'L', 'I', 2)
    const TANGOPRO_BANNER: &[u8; 16] = b"TangoPRO Library"; // @92..108: ASCII library banner
    const BANNER_OFF: usize = 92;

    require!(
        size >= MIN_TANGOPRO_LIB_SIZE,
        prefix.len() >= 108,
        le32(prefix, 0)? <= 0xFFFF,
        eq_at(prefix, 4, TANGOPRO_WORDS),
        eq_at(prefix, BANNER_OFF, TANGOPRO_BANNER)
    );
    Some(Label::TangoproLib)
}

/// Auran / N3V Games Trainz Indexed Mesh or Animation file (`.im` / `.kin`).
pub(crate) fn verify_trainz_im(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TRAINZ_IM_SIZE,
        prefix.len() >= 24,
        starts(prefix, TRAINZ_JIRF_MAGIC),
        le32(prefix, TRAINZ_FILE_SIZE_OFFSET)? >= 16,
        eq_at(prefix, 8, b"IDXMINFO") || eq_at(prefix, 8, b"ANIMINFO"),
        (4..=4096).contains(&le32(prefix, TRAINZ_INFO_SIZE_OFFSET)?)
    );
    Some(Label::TrainzIm)
}

/// Unity Engine Enlighten Precomputed GI Cluster/Albedo Workspace (.caw / .sse).
pub(crate) fn verify_unity_enlighten_caw(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UNITY_ENLIGHTEN_CAW_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\x74\x51\x10\xad"),
        (1..=64).contains(&le32(prefix, 4).unwrap_or(0)),
        (1..=256).contains(&le32(prefix, 24).unwrap_or(0))
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::UnityEnlightenCaw)
}

/// Unity Engine Enlighten Precomputed Scene Visibility Data (`GEVS`, `.vis`).
pub(crate) fn verify_unity_enlighten_vis(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_UNITY_ENLIGHTEN_VIS_SIZE, prefix.len() >= 32, eq_at(prefix, 4, b"GEVS"));
    let payload_len = le32(prefix, 0)? as u64;
    let version = le32(prefix, 8)?;
    require!(
        payload_len.checked_add(4) == Some(size),
        (1..=16).contains(&version),
        le32(prefix, 28) == Some(0x1B)
    );
    Some(Label::UnityEnlightenVis)
}

/// Valve Source Engine Demand-Loaded Animation Sequence (`.ani`, `"IDAG"`, 2004 Valve).
pub(crate) fn verify_valve_ani(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VALVE_ANI_SIZE,
        prefix.len() >= 80,
        starts(prefix, b"IDAG"),
        (44..=54).contains(&le32(prefix, 4)?),
        le32(prefix, 76)? >= 80,
        has_non_ascii_text_byte(prefix.get(..16)?)
    );
    Some(Label::ValveAni)
}

/// Valve Hammer Editor / Worldcraft binary Rich Map Format file (`.rmf`) at any stage.
pub(crate) fn verify_valve_hammer_rmf_with_stage(prefix: &[u8], size: u64) -> Option<Label> {
    const RMF_MAGIC: &[u8; 3] = b"RMF"; // @4..7
    const CMAPWORLD_MAGIC: &[u8; 11] = b"\x0ACMapWorld\0"; // @11 + 140 * num_visgroups
    const VISGROUP_STRIDE: u64 = 140;

    require!(size >= MIN_VALVE_RMF_SIZE, prefix.len() >= 22, eq_at(prefix, 4, RMF_MAGIC));
    let ver_bits = le32(prefix, 0)?;
    let ver = f32::from_bits(ver_bits);
    require!(ver.is_finite(), (0.5..=10.0).contains(&ver));

    let num_visgroups = le32(prefix, 7)? as u64;
    require!(num_visgroups <= 4096);
    let world_off_u64 = 11u64.checked_add(num_visgroups.checked_mul(VISGROUP_STRIDE)?)?;
    require!(world_off_u64.checked_add(11)? <= size);

    if world_off_u64 + 11 <= prefix.len() as u64 {
        let world_off = world_off_u64 as usize;
        require!(eq_at(prefix, world_off, CMAPWORLD_MAGIC));
    } else {
        // Large visgroup table extending beyond prefix: verify visgroup[0] NUL-terminated name at @11..139
        require!(prefix.len() >= 151);
        let vg0_name = get_slice(prefix, 11, 139)?;
        let nul_pos = vg0_name.iter().position(|&b| b == 0)?;
        require!(nul_pos >= 1, vg0_name[..nul_pos].iter().all(|&b| (0x20..=0x7E).contains(&b)));
    }
    Some(Label::ValveHammerRmf)
}

/// Valve GoldSrc / Source Studio 3D Model (.mdl).
pub(crate) fn verify_valve_studio_mdl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_VALVE_STUDIO_MDL_SIZE && prefix.len() >= 148);
    require!(starts(prefix, b"IDST"));

    let version = i32::from_le_bytes(get_slice(prefix, 4, 8)?.try_into().ok()?);
    // GoldSrc uses version 10; Source Engine uses versions 44..=54 (44, 48, 49, 53, etc.)
    require!(version == 10 || (44..=54).contains(&version));

    let file_length = i32::from_le_bytes(get_slice(prefix, 72, 76)?.try_into().ok()?);
    if file_length > 0 {
        require!((file_length as u64) <= size + 1024);
    }

    Some(Label::ValveMdl)
}

/// Valve Source Engine StudioMDL Vertex Data (.vvd).
pub(crate) fn verify_valve_vvd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_VALVE_VVD_SIZE && prefix.len() >= 64 && starts(prefix, b"IDSV"));
    let version = le32(prefix, 4)?;
    let num_lods = le32(prefix, 12)?;
    let vertex_start = le32(prefix, 52)? as u64;
    let tangent_start = le32(prefix, 56)? as u64;
    require!(
        version == 4
            && (1..=8).contains(&num_lods)
            && vertex_start >= 64
            && vertex_start <= size
            && tangent_start <= size
    );
    Some(Label::Vvd)
}

/// Virt-A-Mate DynamicStore Binary Mesh (.vab, 2018) MeshedVR.
pub(crate) fn verify_vam_vab(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VAM_VAB_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"\x0cDynamicStore\x031.0")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::VamVab)
}

/// vctr Binary-Framed JSON Vector (.vctr).
pub(crate) fn verify_vctr_vector(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VCTR_VECTOR_SIZE,
        prefix.len() >= 48,
        starts(prefix, b"vctr\x01\0\0\0"),
        le32(prefix, 8).unwrap_or(0) >= 48,
        get_slice(prefix, 44, 48)? == b"{\"RE"
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::VctrVector)
}

/// VecTor Panel PV19 Finite Element Workspace (.fwk).
pub(crate) fn verify_vector_fwk(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VECTOR_FWK_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"\x0aPanel PV19\x06VecTor\x0c")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::VectorFwk)
}

/// Ken Silverman Voxlap Engine KV6 Voxel Sprite (.kv6, 2000) Ken Silverman.
pub(crate) fn verify_voxlap_kv6(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VOXLAP_KV6_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"Kvxl"),
        (1..=2048).contains(&le32(prefix, 4).unwrap_or(0)),
        (1..=2048).contains(&le32(prefix, 8).unwrap_or(0)),
        (1..=2048).contains(&le32(prefix, 12).unwrap_or(0)),
        (1..=10_000_000).contains(&le32(prefix, 28).unwrap_or(0))
    );
    Some(Label::VoxlapKv6)
}

/// Blizzard Entertainment Warcraft III Binary 3D Model (`.mdx`).
///
/// Header layout:
/// - `@0..4`: `"MDLX"` magic
/// - `@4..8`: `"VERS"` version chunk tag
/// - `@8..12`: `u32 LE` `chunk_size` (`4`)
/// - `@12..16`: `u32 LE` `format_version` (`800` classic RoC/TFT, `900`–`1200` Reforged)
pub(crate) fn verify_warcraft3_mdx(prefix: &[u8], size: u64) -> Option<Label> {
    const VERS_TAG: &[u8; 4] = b"VERS"; // @4..8: version chunk tag
    const VERS_OFF: usize = 4;
    const CHUNK_LEN_OFF: usize = 8; // @8..12: u32 LE chunk length (4)
    const FORMAT_VER_OFF: usize = 12; // @12..16: u32 LE format version (800..=1200)

    require!(
        size >= MIN_WARCRAFT3_MDX_SIZE,
        prefix.len() >= 16,
        starts(prefix, WARCRAFT3_MDX_MAGIC),
        eq_at(prefix, VERS_OFF, VERS_TAG),
        le32(prefix, CHUNK_LEN_OFF)? == 4,
        (800..=1200).contains(&le32(prefix, FORMAT_VER_OFF)?)
    );
    Some(Label::Warcraft3Mdx)
}

/// Westwood Studios Command & Conquer Voxel Animation (`.vxl`, Tiberian Sun / Red Alert 2).
///
/// Header layout:
/// - `@0..16`: `"Voxel Animation\0"`
/// - `@16..20`: `u32 LE` `palette_count` (`1..=256`)
/// - `@20..24`: `u32 LE` `limb_count` (`1..=1024`)
pub(crate) fn verify_westwood_vxl(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 16] = bytes!(b"Voxel Animation", &[0]); // @0..16: NUL-terminated VXL signature
    const PAL_COUNT_OFF: usize = 16; // @16..20: u32 LE palette count (1..=256)
    const LIMB_COUNT_OFF: usize = 20; // @20..24: u32 LE limb/section count (1..=1024)

    require!(
        size >= MIN_WESTWOOD_VXL_SIZE,
        prefix.len() >= 24,
        starts(prefix, MAGIC),
        (1..=256).contains(&le32(prefix, PAL_COUNT_OFF)?),
        (1..=1024).contains(&le32(prefix, LIMB_COUNT_OFF)?)
    );
    Some(Label::WestwoodVxl)
}

/// Blizzard Entertainment World of Warcraft M2 3D Model (`.m2`, `"MD21"`/`"MD20"`, 2004 Blizzard Entertainment).
pub(crate) fn verify_wow_m2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_WOW_M2_SIZE, prefix.len() >= 16);
    if starts(prefix, b"MD21") {
        require!(
            le32(prefix, 4)? >= 48,
            eq_at(prefix, 8, b"MD20"),
            (256..=300).contains(&le32(prefix, 12)?)
        );
        return Some(Label::WowM2);
    }
    if starts(prefix, b"MD20") {
        require!(
            (256..=300).contains(&le32(prefix, 4)?),
            has_non_ascii_text_byte(prefix.get(..16)?)
        );
        return Some(Label::WowM2);
    }
    None
}

/// Laminar Research X-Plane Binary Aircraft Model (.acf, v7/v8).
pub(crate) fn verify_xplane_acf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_XPLANE_ACF_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"\x61\0\0\x02X"),
        get_slice(prefix, 5, 11)?.iter().all(|b| b.is_ascii_alphanumeric() || *b == b' '),
        get_slice(prefix, 5, 11)?.iter().filter(|b| b.is_ascii_digit()).count() >= 3,
        get_slice(prefix, 11, 44)?.iter().all(|&b| b == 0)
    );
    require!(has_non_ascii_text_byte(prefix.get(..44)?));
    Some(Label::XplaneAcf)
}

/// Laminar Research X-Plane Distribution Scenery Format (`.dsf`) file.
pub(crate) fn verify_xplane_dsf_scenery(prefix: &[u8], size: u64) -> Option<Label> {
    const XPLANE_DSF_MAGIC: &[u8; 16] = bytes!(b"XPLNEDSF", le32_bytes!(1), b"DAEH"); // @0..16: "XPLNEDSF" + v1 LE + "DAEH" head atom

    require!(size >= MIN_XPLANE_DSF_SIZE, prefix.len() >= 20, starts(prefix, XPLANE_DSF_MAGIC));
    Some(Label::XplaneDsf)
}
