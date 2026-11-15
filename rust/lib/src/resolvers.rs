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

mod magic;
mod metadata;

use std::collections::HashMap;

use anyhow::Result;

use crate::{ContentType, Input};

const PREFIX_LIMIT: usize = 4096;

/// Returns the content types and metadata of a file without false positives.
pub(crate) fn identify_with_metadata(
    file: &mut Input,
) -> Result<(Vec<ContentType>, HashMap<String, String>)> {
    let content_types = identify(file)?;
    let mut metadata = HashMap::new();
    if let Some(&content_type) = content_types.first() {
        let subformat = magic::take_subformat(content_type);
        metadata::populate(content_type, subformat, &mut metadata);
    }
    Ok((content_types, metadata))
}

/// Returns the content types of a file without false positives.
///
/// The returned list may have false negatives (the file should have a content type that's not
/// returned). In particular, the list may be empty, in which case the model will also try to
/// identify the file.
pub(crate) fn identify(file: &mut Input) -> Result<Vec<ContentType>> {
    magic::clear_subformat();
    let size = file.length();
    if size == 0 {
        return Ok(Vec::new());
    }
    let mut prefix_len = file.ensure_prefix(PREFIX_LIMIT)?;
    if size > prefix_len as u64 {
        let p = file.prefix(prefix_len);
        if p.starts_with(b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1")
            || (p.starts_with(b"{") && !p.starts_with(b"{\\rt"))
            || (size >= 0x8006 && p.starts_with(b"\0\0\0\0"))
        {
            prefix_len = file.ensure_prefix(65536)?;
        }
    }
    let mut tail_len = if (magic::wants_tail(file.prefix(prefix_len))
        || file.prefix(prefix_len).starts_with(b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1")
        || file.prefix(prefix_len).starts_with(b"MZ")
        || file.prefix(prefix_len).starts_with(b"ZM")
        || file.prefix(prefix_len).starts_with(b"ORC")
        || file.prefix(prefix_len).starts_with(b"#!"))
        && size > prefix_len as u64
    {
        file.ensure_suffix(size.min(magic::ZIP_TAIL_BYTES as u64) as usize)?
    } else {
        0
    };
    if tail_len > 0
        && let Some(start) = magic::directory_start(file.suffix(tail_len), size)
    {
        tail_len = file.ensure_suffix((size - start) as usize)?;
    }
    let prefix = file.prefix(prefix_len);
    let tail = if tail_len > 0 {
        Some(file.suffix(tail_len))
    } else if size <= prefix_len as u64 {
        Some(prefix)
    } else {
        None
    };
    let ctx = magic::Ctx { prefix, size, tail };
    if let Some(content_type) = magic::identify(&ctx) {
        return Ok(vec![content_type]);
    }
    if size > prefix_len as u64 && size >= 0x8006 {
        prefix_len = file.ensure_prefix(65536)?;
        let prefix = file.prefix(prefix_len);
        if let Some(ct) = magic::iso(prefix, size) {
            return Ok(vec![ct]);
        }
    }
    if tail_len == 0 && size > prefix_len as u64 {
        let footer_len = file.ensure_suffix(size.min(512) as usize)?;
        let prefix = file.prefix(prefix_len);
        let tail = Some(file.suffix(footer_len));
        if let Some(ct) = magic::dmg_koly(size, tail)
            .or_else(|| magic::leveldb(prefix, size, tail))
            .or_else(|| magic::rocksdb(prefix, size, tail))
            .or_else(|| magic::tga(prefix, size, tail))
        {
            return Ok(vec![ct]);
        }
    }
    Ok(Vec::new())
}

/// List of content types for which the resolvers have no false negatives.
///
/// If the resolvers don't return a match for a given file, and the model returns a content type
/// from this list, then that content type will be vetoed and "txt" or "unknown" will be used (based
/// on "is_text" from the content type that was vetoed).
#[cfg_attr(not(test), allow(dead_code))] // cannot use cfg(test) because of sync_kb.py
pub(crate) const VETOS: &[ContentType] = &[
    ContentType::_3dsm,
    ContentType::_3dsx,
    ContentType::_3gp,
    ContentType::Ace,
    ContentType::Ani,
    ContentType::Applebplist,
    ContentType::Appledouble,
    ContentType::Applesingle,
    ContentType::Arrow,
    ContentType::Au,
    ContentType::Avi,
    ContentType::Avif,
    ContentType::Avro,
    ContentType::Bam,
    ContentType::Beam,
    ContentType::Blend,
    ContentType::Bpg,
    ContentType::Bzip,
    ContentType::Collada,
    ContentType::Cram,
    ContentType::Cubin,
    ContentType::Dex,
    ContentType::Dicom,
    ContentType::Dsstore,
    ContentType::Duckdb,
    ContentType::Dwg,
    ContentType::Ese,
    ContentType::Fbx,
    ContentType::Fits,
    ContentType::Flv,
    ContentType::Geopackage,
    ContentType::Gguf,
    ContentType::Gltf,
    ContentType::Gpx,
    ContentType::Grib,
    ContentType::Gzip,
    ContentType::Hdf4,
    ContentType::Heif,
    ContentType::Hve,
    ContentType::Icc,
    ContentType::Icns,
    ContentType::Javabytecode,
    ContentType::Jng,
    ContentType::Keras,
    ContentType::Kml,
    ContentType::Lha,
    ContentType::LlvmBitcode,
    ContentType::Lnk,
    ContentType::Lrz,
    ContentType::Luabytecode,
    ContentType::Lz,
    ContentType::Lz4,
    ContentType::Mat,
    ContentType::Midi,
    ContentType::Minidump,
    ContentType::Mkv,
    ContentType::Mpegts,
    ContentType::Npy,
    ContentType::Nrrd,
    ContentType::One,
    ContentType::Osm,
    ContentType::Palmos,
    ContentType::Parquet,
    ContentType::Pbm,
    ContentType::Pcapng,
    ContentType::Pgp,
    ContentType::Ply,
    ContentType::Png,
    ContentType::PostgresDump,
    ContentType::Psd,
    ContentType::Qgis,
    ContentType::Qoi,
    ContentType::Rar,
    ContentType::RedisRdb,
    ContentType::Rzip,
    ContentType::Safetensors,
    ContentType::Sas,
    ContentType::Sevenzip,
    ContentType::Shapefile,
    ContentType::Sketchup,
    ContentType::Spirv,
    ContentType::Spss,
    ContentType::Squashfs,
    ContentType::Step,
    ContentType::Swf,
    ContentType::Uf2,
    ContentType::Vhd,
    ContentType::Vib,
    ContentType::Visio,
    ContentType::Wad,
    ContentType::Wasm,
    ContentType::Wav,
    ContentType::Webm,
    ContentType::Woff,
    ContentType::Woff2,
    ContentType::Xar,
    ContentType::Xcf,
    ContentType::Xcoff,
    ContentType::Zst,
];

#[cfg(test)]
mod tests {
    mod corpus;
    mod dataset;
    mod regressions;

    use std::collections::HashSet;

    use super::*;

    fn resolve_slice(bytes: &[u8]) -> Vec<ContentType> {
        let mut input = Input::from_slice(bytes);
        identify(&mut input).unwrap()
    }

    fn resolve_slice_with_metadata(bytes: &[u8]) -> (Vec<ContentType>, HashMap<String, String>) {
        let mut input = Input::from_slice(bytes);
        identify_with_metadata(&mut input).unwrap()
    }

    #[test]
    fn vetos_are_distinct_and_complete() {
        let resolvers_vetos: HashSet<ContentType> = VETOS.iter().copied().collect();
        assert_eq!(VETOS.len(), 99);
        assert_eq!(resolvers_vetos.len(), VETOS.len());
        assert!(VETOS.is_sorted_by_key(|ct| ct.info().label), "VETOS must be sorted by label");
    }

    #[test]
    fn resolvers_identify_basic_corpus() {
        let root = std::path::Path::new("../../tests_data/basic");
        let mut dirs: Vec<_> = std::fs::read_dir(root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        let mut matched = 0usize;
        for dir in dirs {
            let expected = dir.file_name().unwrap().to_str().unwrap();
            let mut entries: Vec<_> =
                std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
            entries.sort();
            for path in entries {
                if !path.is_file() {
                    continue;
                }
                let mut input = Input::from_file(std::fs::File::open(&path).unwrap()).unwrap();
                let got = identify(&mut input).unwrap();
                if let [ct] = got[..] {
                    assert_eq!(ct.info().label, expected, "mismatch on {}", path.display());
                    matched += 1;
                } else {
                    assert!(got.is_empty(), "multiple matches on {}", path.display());
                }
            }
        }
        assert_eq!(matched, 40);
    }

    #[test]
    fn resolvers_identify_fixtures() {
        let root = std::path::Path::new("../../tests_data/resolvers/fixtures");
        let mut dirs: Vec<_> = std::fs::read_dir(root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        let mut matched = 0usize;
        for dir in dirs {
            let mut entries: Vec<_> =
                std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
            entries.sort();
            for path in entries {
                if !path.is_file() {
                    continue;
                }
                let stem = path.file_stem().unwrap().to_str().unwrap();
                let expected = stem.strip_prefix('!').unwrap_or(stem);
                let mut input = Input::from_file(std::fs::File::open(&path).unwrap()).unwrap();
                let got = identify(&mut input).unwrap();
                assert_eq!(got.len(), 1, "expected 1 match on {}, got {got:?}", path.display());
                assert_eq!(got[0].info().label, expected, "mismatch on {}", path.display());
                matched += 1;
            }
        }
        assert_eq!(matched, 310);
    }

    #[test]
    fn resolvers_populate_standard_metadata() {
        let png = std::fs::File::open("../../tests_data/resolvers/fixtures/image/png.png").unwrap();
        let mut input = Input::from_file(png).unwrap();
        let (cts, meta) = identify_with_metadata(&mut input).unwrap();
        assert_eq!(cts, vec![ContentType::Png]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("PNG"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/11"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000153"));

        let pdf =
            std::fs::File::open("../../tests_data/resolvers/fixtures/document/pdf.pdf").unwrap();
        let mut input = Input::from_file(pdf).unwrap();
        let (cts, meta) = identify_with_metadata(&mut input).unwrap();
        assert_eq!(cts, vec![ContentType::Pdf]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("PDF 1.3"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/17"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000316"));

        // Non-matching inputs return empty metadata and do not leak prior subformat state.
        let (cts, meta) = resolve_slice_with_metadata(b"plain text without any file signature");
        assert!(cts.is_empty());
        assert!(meta.is_empty());
    }

    #[test]
    fn resolvers_fixtures_metadata_are_well_formed() {
        let root = std::path::Path::new("../../tests_data/resolvers/fixtures");
        let mut dirs: Vec<_> = std::fs::read_dir(root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        let mut with_puid = 0usize;
        let mut with_fdd = 0usize;
        let mut with_sub = 0usize;
        for dir in dirs {
            let mut entries: Vec<_> =
                std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
            entries.sort();
            for path in entries {
                if !path.is_file() {
                    continue;
                }
                let mut input = Input::from_file(std::fs::File::open(&path).unwrap()).unwrap();
                let (cts, meta) = identify_with_metadata(&mut input).unwrap();
                assert_eq!(cts.len(), 1, "expected 1 match on {}", path.display());
                for (k, v) in &meta {
                    assert!(
                        matches!(k.as_str(), "subformat" | "pronom_puid" | "loc_fdd"),
                        "unexpected metadata key {k:?} on {}",
                        path.display()
                    );
                    assert!(!v.is_empty(), "empty metadata value for {k:?} on {}", path.display());
                }
                if let Some(puid) = meta.get("pronom_puid") {
                    let rest = puid
                        .strip_prefix("fmt/")
                        .or_else(|| puid.strip_prefix("x-fmt/"))
                        .unwrap_or_else(|| panic!("invalid PUID {puid:?} on {}", path.display()));
                    assert!(
                        !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()),
                        "invalid PUID digits {puid:?} on {}",
                        path.display()
                    );
                    with_puid += 1;
                }
                if let Some(fdd) = meta.get("loc_fdd") {
                    let rest = fdd
                        .strip_prefix("fdd")
                        .unwrap_or_else(|| panic!("invalid FDD {fdd:?} on {}", path.display()));
                    assert!(
                        rest.len() == 6 && rest.bytes().all(|b| b.is_ascii_digit()),
                        "invalid FDD digits {fdd:?} on {}",
                        path.display()
                    );
                    with_fdd += 1;
                }
                if meta.contains_key("subformat") {
                    with_sub += 1;
                }
            }
        }
        assert!(with_puid >= 150, "expected broad PUID coverage, got {with_puid}");
        assert!(with_fdd >= 90, "expected broad FDD coverage, got {with_fdd}");
        assert!(with_sub >= 100, "expected broad subformat coverage, got {with_sub}");
    }

    #[test]
    fn resolvers_populate_subformat_overrides() {
        // GIF87a vs GIF89a
        let mut gif87a = *b"GIF87a\x01\x00\x01\x00\x00\x00\x00;";
        let (cts, meta) = resolve_slice_with_metadata(&gif87a);
        assert_eq!(cts, vec![ContentType::Gif]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("GIF 87a"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/3"));
        assert_eq!(meta.get("loc_fdd"), None);

        gif87a[3..6].copy_from_slice(b"89a");
        let (cts, meta) = resolve_slice_with_metadata(&gif87a);
        assert_eq!(cts, vec![ContentType::Gif]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("GIF 89a"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/4"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000133"));

        // PDF 1.4, 1.7, 2.0, and minimal header
        let (cts, meta) = resolve_slice_with_metadata(b"%PDF-1.4\n");
        assert_eq!(cts, vec![ContentType::Pdf]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("PDF 1.4"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/18"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000122"));

        let (cts, meta) = resolve_slice_with_metadata(b"%PDF-2.0\n");
        assert_eq!(cts, vec![ContentType::Pdf]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("PDF 2.0"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/1129"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000474"));

        let (cts, meta) = resolve_slice_with_metadata(b"%PDF\n1 0 obj<<>>endobj\n");
        assert_eq!(cts, vec![ContentType::Pdf]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("PDF (non-standard header)"));
        assert_eq!(meta.get("pronom_puid"), None);
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000030"));

        // PE32 vs tiny collapsed MZPE
        let (cts, meta) = resolve_slice_with_metadata(&build_pe32(0x0102, 0x1000));
        assert_eq!(cts, vec![ContentType::Pebin]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("PE32"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/899"));

        let mut mzpe = vec![0u8; 97];
        mzpe[..6].copy_from_slice(b"MZPE\0\0");
        mzpe[6..8].copy_from_slice(&0x014Cu16.to_le_bytes());
        let (cts, meta) = resolve_slice_with_metadata(&mzpe);
        assert_eq!(cts, vec![ContentType::Pebin]);
        assert_eq!(
            meta.get("subformat").map(String::as_str),
            Some("PE (tiny, header at offset 2)")
        );
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("x-fmt/411"));

        // TIFF vs BigTIFF
        let mut tiff = vec![0u8; 32];
        tiff[..8].copy_from_slice(b"II\x2A\x00\x08\x00\x00\x00");
        let (cts, meta) = resolve_slice_with_metadata(&tiff);
        assert_eq!(cts, vec![ContentType::Tiff]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("TIFF (little-endian)"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/353"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000022"));

        let mut bigtiff = vec![0u8; 32];
        bigtiff[..16].copy_from_slice(&[
            b'I', b'I', 0x2B, 0x00, 0x08, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]);
        let (cts, meta) = resolve_slice_with_metadata(&bigtiff);
        assert_eq!(cts, vec![ContentType::Tiff]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("BigTIFF (little-endian)"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/1917"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000328"));

        // DOCX vs DOCM
        let docx = archive(
            &[
                stored(
                    b"[Content_Types].xml",
                    b"<Types>application/vnd.openxmlformats-officedocument.wordprocessingml.\
                      document.main+xml</Types>",
                ),
                stored(b"word/document.xml", b"<w:document/>"),
            ],
            b"",
        );
        let (cts, meta) = resolve_slice_with_metadata(&docx);
        assert_eq!(cts, vec![ContentType::Docx]);
        assert_eq!(
            meta.get("subformat").map(String::as_str),
            Some("Word Open XML document (.docx)")
        );
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/412"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000397"));

        let docm = archive(
            &[
                stored(
                    b"[Content_Types].xml",
                    b"<Types>application/vnd.ms-word.document.macroEnabled.main+xml</Types>",
                ),
                stored(b"word/document.xml", b"<w:document/>"),
            ],
            b"",
        );
        let (cts, meta) = resolve_slice_with_metadata(&docm);
        assert_eq!(cts, vec![ContentType::Docx]);
        assert_eq!(
            meta.get("subformat").map(String::as_str),
            Some("Word Open XML macro-enabled document (.docm)")
        );
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/523"));
        assert_eq!(meta.get("loc_fdd"), None);

        // Stata 117 vs 118
        let (cts, meta) = resolve_slice_with_metadata(
            b"<stata_dta><header><release>117</release><byteorder>LSF</byteorder>",
        );
        assert_eq!(cts, vec![ContentType::Stata]);
        assert_eq!(
            meta.get("subformat").map(String::as_str),
            Some("Stata DTA 117 (little-endian)")
        );
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/1036"));
        assert_eq!(meta.get("loc_fdd"), None);

        let (cts, meta) = resolve_slice_with_metadata(
            b"<stata_dta><header><release>118</release><byteorder>MSF</byteorder>",
        );
        assert_eq!(cts, vec![ContentType::Stata]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("Stata DTA 118 (big-endian)"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/1037"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000471"));

        // NetCDF-3 classic vs 64-bit
        let mut nc = vec![0u8; 32];
        nc[..4].copy_from_slice(b"CDF\x01");
        let (cts, meta) = resolve_slice_with_metadata(&nc);
        assert_eq!(cts, vec![ContentType::Netcdf]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("NetCDF-3 classic"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/282"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000330"));

        nc[3] = 2;
        let (cts, meta) = resolve_slice_with_metadata(&nc);
        assert_eq!(cts, vec![ContentType::Netcdf]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("NetCDF-3 64-bit offset"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/283"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000330"));

        // ISO 9660 vs High Sierra
        let mut iso = vec![0u8; 0x8020];
        iso[0x8000..0x8006].copy_from_slice(b"\x01CD001");
        let (cts, meta) = resolve_slice_with_metadata(&iso);
        assert_eq!(cts, vec![ContentType::Iso]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("ISO 9660"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/468"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000348"));

        iso[0x8000..0x8006].fill(0);
        iso[0x8008..0x800E].copy_from_slice(b"\x01CDROM");
        let (cts, meta) = resolve_slice_with_metadata(&iso);
        assert_eq!(cts, vec![ContentType::Iso]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("High Sierra"));
        assert_eq!(meta.get("pronom_puid"), None);
        assert_eq!(meta.get("loc_fdd"), None);

        // GeoPackage 1.0 vs 1.2+
        let mut gpkg = vec![0u8; 100];
        gpkg[..16].copy_from_slice(b"SQLite format 3\0");
        gpkg[68..72].copy_from_slice(b"GP10");
        let (cts, meta) = resolve_slice_with_metadata(&gpkg);
        assert_eq!(cts, vec![ContentType::Geopackage]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("GeoPackage 1.0"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/1700"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000419"));

        gpkg[68..72].copy_from_slice(b"GPKG");
        let (cts, meta) = resolve_slice_with_metadata(&gpkg);
        assert_eq!(cts, vec![ContentType::Geopackage]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("GeoPackage 1.2+"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/1700"));
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000520"));

        // GGUF v1 / v2 / v3
        let mut gguf = vec![0u8; 24];
        gguf[..4].copy_from_slice(b"GGUF");
        for (ver, puid) in [(1u32, "fmt/2096"), (2, "fmt/2097"), (3, "fmt/2098")] {
            gguf[4..8].copy_from_slice(&ver.to_le_bytes());
            let (cts, meta) = resolve_slice_with_metadata(&gguf);
            assert_eq!(cts, vec![ContentType::Gguf]);
            assert_eq!(meta.get("pronom_puid").map(String::as_str), Some(puid));
            assert_eq!(meta.get("loc_fdd"), None);
        }

        // ASPRS LAS 1.2 vs 1.4
        let mut las = vec![0u8; 227];
        las[..4].copy_from_slice(b"LASF");
        las[24] = 1;
        las[25] = 2;
        let (cts, meta) = resolve_slice_with_metadata(&las);
        assert_eq!(cts, vec![ContentType::Las]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("ASPRS LAS 1.2"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/370"));
        assert_eq!(meta.get("loc_fdd"), None);

        las[25] = 4;
        let (cts, meta) = resolve_slice_with_metadata(&las);
        assert_eq!(cts, vec![ContentType::Las]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("ASPRS LAS 1.4"));
        assert_eq!(meta.get("pronom_puid"), None);
        assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000418"));
    }

    fn stored<'a>(name: &'a [u8], data: &'a [u8]) -> (&'a [u8], &'a [u8]) {
        (name, data)
    }

    fn archive(entries: &[(&[u8], &[u8])], comment: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut offsets = Vec::new();
        for &(name, data) in entries {
            offsets.push(out.len() as u32);
            out.extend_from_slice(b"PK\x03\x04");
            out.extend_from_slice(&[20, 0, 0, 0, 0, 0]);
            out.extend_from_slice(&[0; 8]);
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&[0, 0]);
            out.extend_from_slice(name);
            out.extend_from_slice(data);
        }
        let cd_offset = out.len() as u32;
        for (&(name, data), offset) in entries.iter().zip(&offsets) {
            out.extend_from_slice(b"PK\x01\x02");
            out.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0]);
            out.extend_from_slice(&[0; 8]);
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&[0; 12]);
            out.extend_from_slice(&offset.to_le_bytes());
            out.extend_from_slice(name);
        }
        let cd_size = out.len() as u32 - cd_offset;
        out.extend_from_slice(b"PK\x05\x06\x00\x00\x00\x00");
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&cd_size.to_le_bytes());
        out.extend_from_slice(&cd_offset.to_le_bytes());
        out.extend_from_slice(&(comment.len() as u16).to_le_bytes());
        out.extend_from_slice(comment);
        out
    }

    fn build_pe32(characteristics: u16, entry_point: u32) -> Vec<u8> {
        let mut out = vec![0u8; 0x80];
        out[..2].copy_from_slice(b"MZ");
        out[0x3c..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        out.extend_from_slice(b"PE\0\0");
        out.extend_from_slice(&0x014cu16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&[0; 12]);
        out.extend_from_slice(&0xe0u16.to_le_bytes());
        out.extend_from_slice(&characteristics.to_le_bytes());
        let mut opt = vec![0u8; 0xe0];
        opt[..2].copy_from_slice(&0x010bu16.to_le_bytes());
        opt[16..20].copy_from_slice(&entry_point.to_le_bytes());
        opt[68..70].copy_from_slice(&3u16.to_le_bytes());
        opt[92..96].copy_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&opt);
        let mut section = [0u8; 40];
        section[16..20].copy_from_slice(&0x200u32.to_le_bytes());
        section[20..24].copy_from_slice(&0x200u32.to_le_bytes());
        out.extend_from_slice(&section);
        out.resize(0x400, 0);
        out
    }

    #[test]
    fn resolvers_identify_archives_and_executables() {
        let docx = archive(
            &[
                stored(
                    b"[Content_Types].xml",
                    b"<Types>application/vnd.openxmlformats-officedocument.wordprocessingml.\
                      document.main+xml</Types>",
                ),
                stored(b"word/document.xml", b"<w:document/>"),
            ],
            b"",
        );
        assert_eq!(resolve_slice(&docx), vec![ContentType::Docx]);
        let jar = archive(
            &[stored(b"META-INF/MANIFEST.MF", &[b' '; 8192]), stored(b"A.class", b"")],
            b"",
        );
        assert_eq!(resolve_slice(&jar), vec![ContentType::Jar]);
        assert_eq!(resolve_slice(&build_pe32(0x0102, 0x1000)), vec![ContentType::Pebin]);
        assert_eq!(resolve_slice(&build_pe32(0x2102, 0)), Vec::<ContentType>::new());
    }

    #[test]
    fn a_tail_extends_back_to_a_large_central_directory() {
        let names: Vec<_> = (0..400).map(|i| format!("{i:0>60}")).collect();
        let entries: Vec<_> = names.iter().map(|name| stored(name.as_bytes(), b"")).collect();
        let bytes = archive(&entries, b"");
        let (prefix, size) = (&bytes[..PREFIX_LIMIT], bytes.len() as u64);
        let tail = &bytes[bytes.len() - magic::ZIP_TAIL_BYTES..];
        let start = magic::directory_start(tail, size).unwrap();
        assert!(start < size - tail.len() as u64);
        let valid =
            |tail| magic::identify(&magic::Ctx { prefix, size, tail: Some(tail) }).is_some();
        assert!(!valid(tail));
        assert!(valid(&bytes[start as usize..]));
    }

    #[test]
    fn magic32_shortcuts_and_expanded_magics() {
        // 1. Collapsed 4-byte MZ+PE overlap ("MZPE\0\0")
        let mut mzpe = vec![0u8; 97];
        mzpe[..6].copy_from_slice(b"MZPE\0\0");
        mzpe[6..8].copy_from_slice(&0x014Cu16.to_le_bytes());
        assert_eq!(resolve_slice(&mzpe), vec![ContentType::Pebin]);

        // 2. Reflective DLL stub ("MZAR") with PE header at e_lfanew = 0x80
        let mut mzar = build_pe32(0x2102, 0x1000);
        mzar[2..4].copy_from_slice(b"AR");
        assert_eq!(resolve_slice(&mzar), vec![ContentType::Pebin]);

        // 3. MS-DOS .COM executables via magic32 and magic16
        let mut com_reloc = vec![0u8; 64];
        com_reloc[..4].copy_from_slice(&[0x9C, 0x55, 0x56, 0x8C]);
        assert_eq!(resolve_slice(&com_reloc), vec![ContentType::Pebin]);

        let mut com_tpack = vec![0u8; 64];
        com_tpack[..8].copy_from_slice(&[0x68, 0x00, 0x01, 0xFD, 0x60, 0xBE, 0x00, 0x02]);
        assert_eq!(resolve_slice(&com_tpack), vec![ContentType::Pebin]);

        // 4. FreeBSD boot0 MBR prologue (FC 31 C0 8E) with 55 AA signature at @510
        let mut mbr = vec![0u8; 512];
        mbr[..4].copy_from_slice(&[0xFC, 0x31, 0xC0, 0x8E]);
        mbr[0x1BE] = 0x80;
        mbr[0x1BE + 4] = 0xA5; // FreeBSD slice type
        mbr[0x1BE + 8..0x1BE + 12].copy_from_slice(&63u32.to_le_bytes());
        mbr[0x1BE + 12..0x1BE + 16].copy_from_slice(&1000u32.to_le_bytes());
        mbr[510..512].copy_from_slice(&[0x55, 0xAA]);
        assert_eq!(resolve_slice(&mbr), vec![ContentType::Dosmbr]);

        // 5. El Torito IsoHybrid (ISOLINUX 33 ED 90 90 MBR at @0 + ISO 9660 PVD at @0x8000)
        // resolves to Iso via boot_or_isohybrid
        let mut isohybrid = vec![0u8; 0x8800];
        isohybrid[..512].copy_from_slice(&mbr);
        isohybrid[..4].copy_from_slice(&[0x33, 0xED, 0x90, 0x90]);
        isohybrid[0x8000..0x8007].copy_from_slice(b"\x01CD001\x01");
        isohybrid[0x8050..0x8054].copy_from_slice(&100u32.to_le_bytes());
        isohybrid[0x8054..0x8058].copy_from_slice(&100u32.to_be_bytes());
        isohybrid[0x8080..0x8082].copy_from_slice(&2048u16.to_le_bytes());
        isohybrid[0x8082..0x8084].copy_from_slice(&2048u16.to_be_bytes());
        assert_eq!(resolve_slice(&isohybrid), vec![ContentType::Iso]);

        // 6. ISO 9660 with non-zero sector-0 fill (AD AD AD AD)
        let mut iso_ad = isohybrid.clone();
        iso_ad[..512].fill(0xAD);
        assert_eq!(resolve_slice(&iso_ad), vec![ContentType::Iso]);
    }

    #[test]
    fn mid_file_bytes_not_mistaken_for_eof_footer_without_tail() {
        // leveldb footer: 48 bytes ending with 0xDB4775248B80FB57 LE
        let mut leveldb_prefix = vec![0u8; 48];
        leveldb_prefix[40..48].copy_from_slice(&[0x57, 0xFB, 0x80, 0x8B, 0x24, 0x75, 0x47, 0xDB]);
        assert_eq!(magic::leveldb(&leveldb_prefix, 100, None), None);
        assert_eq!(
            magic::leveldb(&leveldb_prefix, 48, Some(&leveldb_prefix)),
            Some(ContentType::Leveldb)
        );

        // rocksdb footer: 53 bytes ending with ROCKSDB_BLOCK_MAGIC_LE
        let mut rocksdb_prefix = vec![0u8; 53];
        rocksdb_prefix[45..53].copy_from_slice(&[0x75, 0x3F, 0x6B, 0x98, 0xF7, 0x85, 0xB2, 0x88]);
        assert_eq!(magic::rocksdb(&rocksdb_prefix, 100, None), None);
        assert_eq!(
            magic::rocksdb(&rocksdb_prefix, 53, Some(&rocksdb_prefix)),
            Some(ContentType::Rocksdb)
        );

        // orc: starts with b"ORC\x82", postscript at the end of prefix
        let mut orc_prefix = vec![0u8; 64];
        orc_prefix[..4].copy_from_slice(b"ORC\x82");
        orc_prefix[55..58].copy_from_slice(b"ORC");
        orc_prefix[63] = 4;
        assert_eq!(magic::orc(&orc_prefix, 1000, None), None);
        assert_eq!(magic::orc(&orc_prefix, 64, Some(&orc_prefix)), Some(ContentType::Orc));

        // pythonpar: shebang without local ZIP header, but EOCD in prefix
        let mut pypar_prefix = vec![0u8; 64];
        pypar_prefix[..22].copy_from_slice(b"#!/usr/bin/env python\n");
        pypar_prefix[30..34].copy_from_slice(b"PK\x05\x06");
        assert_eq!(magic::pythonpar(&pypar_prefix, 1000, None), None);
        assert_eq!(
            magic::pythonpar(&pypar_prefix, 64, Some(&pypar_prefix)),
            Some(ContentType::Pythonpar)
        );
    }

    #[test]
    fn segy_variants_and_false_positive_regressions() {
        // EBCDIC "C 1 CLIENT SAMPLE SEGY1234" (>= 16 alphanumeric EBCDIC bytes)
        let ebcdic_card: &[u8] = &[
            0xC3, 0x40, 0xF1, 0x40, 0xC3, 0xD3, 0xC9, 0xC5, 0xD5, 0xE3, 0x40, 0xE2, 0xC1, 0xD4,
            0xD7, 0xD3, 0xC5, 0x40, 0xE2, 0xC5, 0xC7, 0xE8, 0xF1, 0xF2, 0xF3, 0xF4,
        ];

        // 1. Standard big-endian EBCDIC textual header
        let mut ebcdic = vec![0u8; 3632];
        ebcdic[..3200].fill(0x40);
        ebcdic[..ebcdic_card.len()].copy_from_slice(ebcdic_card);
        ebcdic[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
        ebcdic[3220..3222].copy_from_slice(&1000u16.to_be_bytes());
        ebcdic[3224..3226].copy_from_slice(&5u16.to_be_bytes());
        let (cts, meta) = resolve_slice_with_metadata(&ebcdic);
        assert_eq!(cts, vec![ContentType::Segy]);
        assert_eq!(
            meta.get("subformat").map(String::as_str),
            Some("SEG-Y (EBCDIC textual header)")
        );
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/363"));

        // 2. Non-'C' EBCDIC textual header (leading EBCDIC space)
        let mut ebcdic_noc = ebcdic.clone();
        ebcdic_noc[0] = 0x40;
        assert_eq!(resolve_slice(&ebcdic_noc), vec![ContentType::Segy]);

        // 2b. EBCDIC textual header with decorative border lines ('----' = 0x60, '====' = 0x7E, '****' = 0x5C)
        let mut ebcdic_borders = ebcdic.clone();
        ebcdic_borders[80..160].fill(0x60);
        ebcdic_borders[160..240].fill(0x7E);
        ebcdic_borders[240..320].fill(0x5C);
        assert_eq!(resolve_slice(&ebcdic_borders), vec![ContentType::Segy]);

        // 3. Little-endian EBCDIC without Rev 2 endianness constant, across all Rev 0/1/2 format codes
        for fmt in [1u16, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 15, 16] {
            let mut le = ebcdic.clone();
            le[3216..3218].copy_from_slice(&4000u16.to_le_bytes());
            le[3220..3222].copy_from_slice(&250u16.to_le_bytes());
            le[3224..3226].copy_from_slice(&fmt.to_le_bytes());
            assert_eq!(resolve_slice(&le), vec![ContentType::Segy], "LE fmt {fmt}");
        }

        // 4. Rev 2 extended sample interval (f64 @3272) and extended sample count (i32 @3268)
        let mut rev2_ext = ebcdic.clone();
        rev2_ext[3216..3218].fill(0);
        rev2_ext[3220..3222].fill(0);
        rev2_ext[3224..3226].copy_from_slice(&10u16.to_be_bytes());
        rev2_ext[3268..3272].copy_from_slice(&100_000i32.to_be_bytes());
        rev2_ext[3272..3280].copy_from_slice(&2000.0f64.to_be_bytes());
        rev2_ext[3296..3300].copy_from_slice(&[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(resolve_slice(&rev2_ext), vec![ContentType::Segy]);

        // Negative extended sample count is rejected
        let mut rev2_neg_ns = rev2_ext.clone();
        rev2_neg_ns[3268..3272].copy_from_slice(&(-1i32).to_be_bytes());
        assert_eq!(resolve_slice(&rev2_neg_ns), Vec::<ContentType>::new());

        // Rev 2 extended fields override non-zero 16-bit fields when present
        let mut rev2_override = vec![0u8; 3600 + 240 + 10 * 4]; // 1 trace of 10 x 4B samples
        rev2_override[..24].copy_from_slice(b"C 1 SEGY REV 2 OVERRIDES");
        rev2_override[3216..3218].copy_from_slice(&1000u16.to_be_bytes());
        rev2_override[3220..3222].copy_from_slice(&u16::MAX.to_be_bytes()); // clamped 16-bit ns
        rev2_override[3224..3226].copy_from_slice(&5u16.to_be_bytes());
        rev2_override[3268..3272].copy_from_slice(&10i32.to_be_bytes()); // overrides ns16
        rev2_override[3272..3280].copy_from_slice(&2000.0f64.to_be_bytes()); // overrides dt16
        assert_eq!(resolve_slice(&rev2_override), vec![ContentType::Segy]);

        // Negative extended sample count or invalid extended interval is rejected even if 16-bit fields are non-zero
        let mut rev2_bad_ext_ns = ebcdic.clone();
        rev2_bad_ext_ns[3268..3272].copy_from_slice(&(-5i32).to_be_bytes());
        assert_eq!(resolve_slice(&rev2_bad_ext_ns), Vec::<ContentType>::new());
        let mut rev2_nan_dt = ebcdic.clone();
        rev2_nan_dt[3272..3280].copy_from_slice(&f64::NAN.to_be_bytes());
        assert_eq!(resolve_slice(&rev2_nan_dt), Vec::<ContentType>::new());

        // Rev 2 ext_headers == -1 with explicit first_trace_offset (@3520..3528)
        let mut rev2_fto = vec![0u8; 3600 + 3200 + 240 + 10 * 4];
        rev2_fto[..24].copy_from_slice(b"C 1 SEGY REV 2 FIRST TRC");
        rev2_fto[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
        rev2_fto[3220..3222].copy_from_slice(&10u16.to_be_bytes());
        rev2_fto[3224..3226].copy_from_slice(&5u16.to_be_bytes());
        rev2_fto[3504..3506].copy_from_slice(&(-1i16).to_be_bytes());
        rev2_fto[3520..3528].copy_from_slice(&(6800u64).to_be_bytes());
        assert_eq!(resolve_slice(&rev2_fto), vec![ContentType::Segy]);

        // Rev 2 format code 9 (8-byte signed int) exact trace geometry corroborates ASCII header
        let mut rev2_fmt9 = vec![0u8; 3600 + 240 + 10 * 8]; // 1 trace of 10 x 8B samples
        rev2_fmt9[..24].copy_from_slice(b"C 1 SEGY REV 2 INT64 FMT");
        rev2_fmt9[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
        rev2_fmt9[3220..3222].copy_from_slice(&10u16.to_be_bytes());
        rev2_fmt9[3224..3226].copy_from_slice(&9u16.to_be_bytes());
        assert_eq!(resolve_slice(&rev2_fmt9), vec![ContentType::Segy]);

        // 5. NUL-padded ASCII textual header corroborated by trace-0 header match
        let mut ascii_nul = vec![0u8; 3840];
        ascii_nul[..24].copy_from_slice(b"C 1 ASCII SEISMIC HEADER");
        ascii_nul[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
        ascii_nul[3220..3222].copy_from_slice(&100u16.to_be_bytes());
        ascii_nul[3224..3226].copy_from_slice(&1u16.to_be_bytes());
        ascii_nul[3714..3716].copy_from_slice(&100u16.to_be_bytes());
        ascii_nul[3716..3718].copy_from_slice(&2000u16.to_be_bytes());
        let (cts, meta) = resolve_slice_with_metadata(&ascii_nul);
        assert_eq!(cts, vec![ContentType::Segy]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("SEG-Y (ASCII textual header)"));

        // 6. Little-endian ASCII textual header corroborated by Rev 2 byte-order constant
        let mut ascii_le = vec![0u8; 3632];
        ascii_le[..3200].fill(b' ');
        ascii_le[..24].copy_from_slice(b"C 1 SEGY REV 2 LE HEADER");
        ascii_le[3216..3218].copy_from_slice(&4000u16.to_le_bytes());
        ascii_le[3220..3222].copy_from_slice(&500u16.to_le_bytes());
        ascii_le[3224..3226].copy_from_slice(&16u16.to_le_bytes());
        ascii_le[3296..3300].copy_from_slice(&[0x04, 0x03, 0x02, 0x01]);
        assert_eq!(resolve_slice(&ascii_le), vec![ContentType::Segy]);

        // 7. All-zero textual header corroborated by exact geometry + trace-0 match
        let mut empty_text = vec![0u8; 3600 + 240 + 40]; // 1 trace of 10 x 4B samples
        empty_text[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
        empty_text[3220..3222].copy_from_slice(&10u16.to_be_bytes());
        empty_text[3224..3226].copy_from_slice(&5u16.to_be_bytes());
        empty_text[3714..3716].copy_from_slice(&10u16.to_be_bytes());
        empty_text[3716..3718].copy_from_slice(&2000u16.to_be_bytes());
        let (cts, meta) = resolve_slice_with_metadata(&empty_text);
        assert_eq!(cts, vec![ContentType::Segy]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("SEG-Y (empty textual header)"));

        // Negative regressions:
        // a) Uncorroborated short prefix with sparse binary header
        let mut bogus_pnt = vec![0u8; 3632];
        bogus_pnt[..3].copy_from_slice(b"PNT");
        bogus_pnt[3216..3218].copy_from_slice(&0xC600u16.to_be_bytes());
        bogus_pnt[3220..3222].copy_from_slice(&0xEDFFu16.to_be_bytes());
        bogus_pnt[3224..3226].copy_from_slice(&1u16.to_be_bytes());
        assert_eq!(resolve_slice(&bogus_pnt), Vec::<ContentType>::new());

        // b) Non-zero unassigned binary header ranges (@3300..3500 and @3532..3600)
        for off in [3300usize, 3499, 3532, 3599] {
            let mut dirty = ebcdic.clone();
            dirty[off] = 0x01;
            assert_eq!(resolve_slice(&dirty), Vec::<ContentType>::new(), "dirty @{off}");
        }

        // c) Invalid Rev 2 byte-order constant
        let mut bad_em = ebcdic.clone();
        bad_em[3296..3300].copy_from_slice(&[0x01, 0x02, 0x03, 0x05]);
        assert_eq!(resolve_slice(&bad_em), Vec::<ContentType>::new());

        // d) Invalid format codes (0, 13, 14, 17)
        for fmt in [0u16, 13, 14, 17] {
            let mut bad_fmt = ebcdic.clone();
            bad_fmt[3224..3226].copy_from_slice(&fmt.to_be_bytes());
            assert_eq!(resolve_slice(&bad_fmt), Vec::<ContentType>::new(), "invalid fmt {fmt}");
        }

        // e) Out-of-range binary header control fields
        let mut bad_sort = ebcdic.clone();
        bad_sort[3228..3230].copy_from_slice(&10i16.to_be_bytes());
        assert_eq!(resolve_slice(&bad_sort), Vec::<ContentType>::new());

        let mut bad_meas = ebcdic.clone();
        bad_meas[3254..3256].copy_from_slice(&3u16.to_be_bytes());
        assert_eq!(resolve_slice(&bad_meas), Vec::<ContentType>::new());

        let mut bad_fixed = ebcdic.clone();
        bad_fixed[3502..3504].copy_from_slice(&2u16.to_be_bytes());
        assert_eq!(resolve_slice(&bad_fixed), Vec::<ContentType>::new());

        let mut bad_ext = ebcdic.clone();
        bad_ext[3504..3506].copy_from_slice(&5i16.to_be_bytes()); // exceeds 3632 B file size
        assert_eq!(resolve_slice(&bad_ext), Vec::<ContentType>::new());

        // f) Uncorroborated ASCII and uncorroborated all-zero textual headers
        let mut uncorr_ascii = ascii_le.clone();
        uncorr_ascii[3296..3300].fill(0);
        assert_eq!(resolve_slice(&uncorr_ascii), Vec::<ContentType>::new());

        let mut uncorr_zero = vec![0u8; 3632];
        uncorr_zero[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
        uncorr_zero[3220..3222].copy_from_slice(&10u16.to_be_bytes());
        uncorr_zero[3224..3226].copy_from_slice(&5u16.to_be_bytes());
        assert_eq!(resolve_slice(&uncorr_zero), Vec::<ContentType>::new());

        // g) Binary noise in textual header (e.g., bytecode/compiled binary), punctuation-only
        // header (e.g., EBCDIC space 0x40 / ASCII '@'), & pure printable ASCII
        let mut noisy = ebcdic.clone();
        noisy[..64].fill(0x01);
        assert_eq!(resolve_slice(&noisy), Vec::<ContentType>::new());
        let mut at_only = vec![0u8; 3600 + 240 + 40];
        at_only[..32].fill(0x40);
        at_only[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
        at_only[3220..3222].copy_from_slice(&10u16.to_be_bytes());
        at_only[3224..3226].copy_from_slice(&5u16.to_be_bytes());
        assert_eq!(resolve_slice(&at_only), Vec::<ContentType>::new());
        assert_eq!(resolve_slice(&vec![b'A'; 4096]), Vec::<ContentType>::new());

        // h) Uncorroborated NUL-padded high bytes without EBCDIC spaces, and UTF-16LE / 16-bit arrays
        let mut nul_high = vec![0u8; 3632];
        nul_high[..32].fill(0xC1); // EBCDIC 'A' without any 0x40 spaces or corroboration
        nul_high[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
        nul_high[3220..3222].copy_from_slice(&1000u16.to_be_bytes());
        nul_high[3224..3226].copy_from_slice(&5u16.to_be_bytes());
        assert_eq!(resolve_slice(&nul_high), Vec::<ContentType>::new());

        let mut utf16le = vec![0u8; 3600 + 240 + 10 * 4];
        for i in (0..400).step_by(2) {
            utf16le[i] = b'A';
        }
        utf16le[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
        utf16le[3220..3222].copy_from_slice(&10u16.to_be_bytes());
        utf16le[3224..3226].copy_from_slice(&5u16.to_be_bytes());
        assert_eq!(resolve_slice(&utf16le), Vec::<ContentType>::new());

        let mut u16_ebcdic = vec![0u8; 3632];
        for i in (0..400).step_by(4) {
            u16_ebcdic[i] = 0xC1;
            u16_ebcdic[i + 2] = 0x40;
        }
        u16_ebcdic[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
        u16_ebcdic[3220..3222].copy_from_slice(&1000u16.to_be_bytes());
        u16_ebcdic[3224..3226].copy_from_slice(&5u16.to_be_bytes());
        assert_eq!(resolve_slice(&u16_ebcdic), Vec::<ContentType>::new());

        // i) Exact NUL-transition boundary across all-NUL, all-non-NUL, and mixed 64-byte chunks
        let mut nul_boundary = vec![0u8; 3632];
        for chunk_idx in 0..40 {
            let fill = if chunk_idx % 2 == 0 { 0x00 } else { 0x40 };
            nul_boundary[chunk_idx * 64..(chunk_idx + 1) * 64].fill(fill);
        }
        nul_boundary[64..80].fill(0xC1); // 16 EBCDIC alnum in chunk 1 (20 chunk-boundary transitions)
        nul_boundary[2560..3200].fill(0x40);
        for k in 0..44 {
            nul_boundary[2560 + 2 * k] = 0x00;
            nul_boundary[2560 + 2 * k + 1] = 0xC1; // +44 transitions => 64 total (exact max)
        }
        nul_boundary[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
        nul_boundary[3220..3222].copy_from_slice(&1000u16.to_be_bytes());
        nul_boundary[3224..3226].copy_from_slice(&5u16.to_be_bytes());
        assert_eq!(resolve_slice(&nul_boundary), vec![ContentType::Segy]);
        nul_boundary[2560 + 88] = 0x00;
        nul_boundary[2560 + 89] = 0xC1; // 65 total transitions => rejected
        assert_eq!(resolve_slice(&nul_boundary), Vec::<ContentType>::new());
    }
}
