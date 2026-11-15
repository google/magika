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
        let mut subformat = magic::take_subformat(content_type);
        if content_type == ContentType::Sqlite && subformat.is_none() {
            subformat = magic::sqlite_schema_subformat(file);
        }
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

    fn encode_sqlite_varint(mut v: u64, out: &mut Vec<u8>) {
        if v <= 0x7F {
            out.push(v as u8);
            return;
        }
        let mut tmp = [0u8; 9];
        let mut n = 0usize;
        while v > 0 && n < 8 {
            tmp[n] = (v & 0x7F) as u8;
            v >>= 7;
            n += 1;
        }
        for i in (0..n).rev() {
            let b = tmp[i] | if i > 0 { 0x80 } else { 0 };
            out.push(b);
        }
    }

    fn encode_sqlite_text(s: &str, enc: u32) -> Vec<u8> {
        match enc {
            2 => s.encode_utf16().flat_map(u16::to_le_bytes).collect(),
            3 => s.encode_utf16().flat_map(u16::to_be_bytes).collect(),
            _ => s.as_bytes().to_vec(),
        }
    }

    fn build_sqlite_schema_record(name: &str, sql: &str, enc: u32) -> Vec<u8> {
        let typ = encode_sqlite_text("table", enc);
        let nam = encode_sqlite_text(name, enc);
        let tbl = nam.clone();
        let sql_b = encode_sqlite_text(sql, enc);

        let mut hdr_body = Vec::new();
        encode_sqlite_varint((typ.len() as u64) * 2 + 13, &mut hdr_body);
        encode_sqlite_varint((nam.len() as u64) * 2 + 13, &mut hdr_body);
        encode_sqlite_varint((tbl.len() as u64) * 2 + 13, &mut hdr_body);
        encode_sqlite_varint(1, &mut hdr_body); // rootpage: 1-byte integer
        if sql_b.is_empty() {
            encode_sqlite_varint(0, &mut hdr_body); // NULL
        } else {
            encode_sqlite_varint((sql_b.len() as u64) * 2 + 13, &mut hdr_body);
        }

        let mut hdr_len_bytes = Vec::new();
        let mut total_hdr = hdr_body.len() + 1;
        loop {
            hdr_len_bytes.clear();
            encode_sqlite_varint(total_hdr as u64, &mut hdr_len_bytes);
            if hdr_len_bytes.len() + hdr_body.len() == total_hdr {
                break;
            }
            total_hdr = hdr_len_bytes.len() + hdr_body.len();
        }

        let mut rec = Vec::new();
        rec.extend_from_slice(&hdr_len_bytes);
        rec.extend_from_slice(&hdr_body);
        rec.extend_from_slice(&typ);
        rec.extend_from_slice(&nam);
        rec.extend_from_slice(&tbl);
        rec.push(2); // rootpage = 2
        rec.extend_from_slice(&sql_b);
        rec
    }

    fn init_sqlite_header(buf: &mut [u8], page_size: usize, enc: u32) {
        buf[..16].copy_from_slice(b"SQLite format 3\0");
        let pg_raw: u16 = if page_size == 65536 { 1 } else { page_size as u16 };
        buf[16..18].copy_from_slice(&pg_raw.to_be_bytes());
        buf[18] = 1; // write version
        buf[19] = 1; // read version
        buf[20] = 0; // reserved space
        buf[21..24].copy_from_slice(&[64, 32, 32]);
        buf[44..48].copy_from_slice(&4u32.to_be_bytes()); // schema format
        buf[56..60].copy_from_slice(&enc.to_be_bytes());
    }

    fn write_leaf_schema_page(page: &mut [u8], hdr_off: usize, enc: u32, tables: &[(&str, &str)]) {
        page[hdr_off] = 0x0D; // table B-tree leaf page
        page[hdr_off + 3..hdr_off + 5].copy_from_slice(&(tables.len() as u16).to_be_bytes());
        let mut cursor = page.len();
        for (idx, &(name, sql)) in tables.iter().enumerate() {
            let rec = build_sqlite_schema_record(name, sql, enc);
            let mut cell = Vec::new();
            encode_sqlite_varint(rec.len() as u64, &mut cell);
            encode_sqlite_varint((idx + 1) as u64, &mut cell);
            cell.extend_from_slice(&rec);
            cursor -= cell.len();
            page[cursor..cursor + cell.len()].copy_from_slice(&cell);
            let ptr_off = hdr_off + 8 + idx * 2;
            page[ptr_off..ptr_off + 2].copy_from_slice(&(cursor as u16).to_be_bytes());
        }
        page[hdr_off + 5..hdr_off + 7].copy_from_slice(&(cursor as u16).to_be_bytes());
    }

    fn build_sqlite_leaf_db(page_size: usize, enc: u32, tables: &[(&str, &str)]) -> Vec<u8> {
        let mut db = vec![0u8; page_size];
        init_sqlite_header(&mut db, page_size, enc);
        write_leaf_schema_page(&mut db, 100, enc, tables);
        db
    }

    fn build_sqlite_interior_db(
        page_size: usize, total_pages: usize, child_page: u32, enc: u32, tables: &[(&str, &str)],
    ) -> Vec<u8> {
        let mut db = vec![0u8; page_size * total_pages];
        init_sqlite_header(&mut db, page_size, enc);
        // Page 1: interior table B-tree page pointing to `child_page` via right_child.
        db[100] = 0x05;
        db[108..112].copy_from_slice(&child_page.to_be_bytes());
        let child_off = (child_page as usize - 1) * page_size;
        write_leaf_schema_page(&mut db[child_off..child_off + page_size], 0, enc, tables);
        db
    }

    fn build_sqlite_overflow_tier3_db(table_name: &str, sql: &str) -> Vec<u8> {
        let page_size = 512usize;
        let rec = build_sqlite_schema_record(table_name, sql, 1);
        let max_local = page_size - 35;
        assert!(rec.len() > max_local, "test SQL must spill to an overflow page");
        let min_local = ((page_size - 12) * 32 / 255).saturating_sub(23);
        let k = min_local + ((rec.len() - min_local) % (page_size - 4));
        let local_len = if k <= max_local { k } else { min_local };

        let mut db = vec![0u8; page_size * 2];
        init_sqlite_header(&mut db, page_size, 1);
        db[100] = 0x0D;
        db[103..105].copy_from_slice(&1u16.to_be_bytes());

        let mut cell = Vec::new();
        encode_sqlite_varint(rec.len() as u64, &mut cell);
        encode_sqlite_varint(1, &mut cell);
        cell.extend_from_slice(&rec[..local_len]);
        cell.extend_from_slice(&2u32.to_be_bytes()); // overflow page = 2

        let cell_off = page_size - cell.len();
        db[cell_off..page_size].copy_from_slice(&cell);
        db[108..110].copy_from_slice(&(cell_off as u16).to_be_bytes());
        db[105..107].copy_from_slice(&(cell_off as u16).to_be_bytes());

        // Page 2: overflow page with next_pg = 0 followed by remaining payload bytes.
        let rem = &rec[local_len..];
        assert!(rem.len() <= page_size - 4);
        db[page_size + 4..page_size + 4 + rem.len()].copy_from_slice(rem);
        db
    }

    #[test]
    fn resolvers_identify_sqlite_schema_subformats() {
        let dir = std::path::Path::new("../../tests_data/resolvers/subformats/sqlite");
        let expected: &[(&str, &str, Option<&str>, Option<&str>)] = &[
            (
                "!android_keystore.sqlite",
                "Android Keystore 2.0 database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "!bindiff_workspace.BinDiffWorkspace",
                "BinDiff Workspace database (.BinDiffWorkspace)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "!debuginfod_db.sqlite",
                "GNU elfutils debuginfod index",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("!fossil_bundle.sqlite", "Fossil SCM bundle", Some("fmt/729"), Some("fdd000461")),
            (
                "!fossil_global_cfg.sqlite",
                "Fossil SCM global configuration",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "!heimdal_krb5.sqlite",
                "Heimdal Kerberos 5 credential cache",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "!kyua_results.sqlite",
                "BSD Kyua test results database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "!linux_perf_db.sqlite",
                "Linux perf trace export database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "!makemandb.sqlite",
                "BSD makemandb manual page index",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "!matrix_rust_sdk.sqlite",
                "Matrix Rust SDK crypto store",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("!nix_fetcher_cache.sqlite", "Nix fetcher cache", Some("fmt/729"), Some("fdd000461")),
            (
                "!nix_store.sqlite",
                "Nix / GNU Guix store database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "!python_lockfile.sqlite",
                "Python lockfile.sqlitelockfile database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "!signal_desktop.sqlite",
                "Signal Desktop database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "!tensorboard_db.sqlite",
                "TensorFlow / TensorBoard summary database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("!texnicard_db.sqlite", "TeXnicard card database", Some("fmt/729"), Some("fdd000461")),
            ("acorn_image.acorn", "Acorn image (.acorn)", Some("fmt/729"), Some("fdd000461")),
            (
                "activepresenter.sqlite",
                "ActivePresenter project (.approj)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "adobe_acrobat_sync.sqlite",
                "Adobe Acrobat Cloud Synchronizer database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "adobe_cc_datatable.sqlite",
                "Adobe Acrobat Creative Cloud cache",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "adobe_cc_pcd_master.sqlite",
                "Adobe Creative Cloud PCD master database (pcd.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "adobe_cc_pim.sqlite",
                "Adobe Creative Cloud package manager database (pim.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "adobe_crcommon.sqlite",
                "Adobe Acrobat CRCommon cache",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "adobe_lightroom.sqlite",
                "Adobe Lightroom catalog (.lrcat)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "adobe_lightroom_temp.sqlite",
                "Adobe Lightroom temporary import catalog",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "adobe_media_db.sqlite",
                "Adobe Creative Suite / Creative Cloud Installer Media database (Media_db.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("adobe_muse.muse", "Adobe Muse project (.muse)", Some("fmt/729"), Some("fdd000461")),
            (
                "adobe_opm_db.sqlite",
                "Adobe Creative Cloud OOBE policy database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "adobe_pcd_cache.sqlite",
                "Adobe Creative Cloud PCD licensing cache",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "adobe_reader_msgs.sqlite",
                "Adobe Acrobat Reader messaging cache",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "adobe_shared_data_events.sqlite",
                "Adobe Creative Cloud SharedDataEvents",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "airflow_db.sqlite",
                "Apache Airflow metadata database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "amd_dxcache.sqlite",
                "AMD DirectX shader cache (.dxcache)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_accounts.sqlite",
                "Android AccountManager database (accounts.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_airguard.sqlite",
                "AirGuard AirTag tracker detection database (attd_db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_burner.sqlite",
                "Burner Android communication database (burners.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_calendar.sqlite",
                "Android CalendarProvider database (calendar.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_contacts.sqlite",
                "Android Contacts Provider database (contacts2.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_digital_wellbeing.sqlite",
                "Android Digital Wellbeing usage database (app_usage)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_downloads.sqlite",
                "Android DownloadProvider database (downloads.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_media.sqlite",
                "Android MediaProvider database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_mmssms.sqlite",
                "Android Telephony MMS/SMS database (mmssms.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_room.sqlite",
                "Android Jetpack Room database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_settings.sqlite",
                "Android SettingsProvider database (settings.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_simplestorage.sqlite",
                "Android System Intelligence launch database (SimpleStorage)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_telephony.sqlite",
                "Android Telephony Carriers database (telephony.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_turbo_battery.sqlite",
                "Android Device Health Services battery database (turbo.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_tv.sqlite",
                "Android TvProvider database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_twitter.sqlite",
                "Twitter / X Android database (twitter.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_viber.sqlite",
                "Viber Messenger Android database (viber_data)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_webview_cache.sqlite",
                "Android WebView HTTP Cache (webviewCache.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_webview_creds.sqlite",
                "Android WebView Credentials database (webview.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "android_workmanager.sqlite",
                "AndroidX WorkManager database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "angr_db.adb",
                "angr binary analysis database (.adb)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "anki_1_deck.sqlite",
                "Anki 1.x flashcard deck (.anki)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "anki_2_deck.sqlite",
                "Anki flashcard deck (.anki2)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "antigravity_cli_session.sqlite",
                "Google Antigravity CLI session database (session.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_accounts3.sqlite",
                "Apple Accounts3 CoreData database (Accounts3.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_addressbook.sqlite",
                "Apple AddressBook database (AddressBook.sqlitedb)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_aps_db.sqlite",
                "Apple Push Notification Service database (aps.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_calendar.sqlite",
                "Apple Calendar database (Calendar.sqlitedb)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_call_history.sqlite",
                "Apple iOS Call History database (CallHistory.storedata)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_cfurlcache.sqlite",
                "Apple CFURLCache network cache (Cache.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_coredata.sqlite",
                "Apple CoreData persistent store",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_datausage.sqlite",
                "Apple Data Usage CoreData database (DataUsage.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_doc_revisions.sqlite",
                "Apple macOS Document Revisions database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_fcp_event.fcpevent",
                "Apple Final Cut Pro / iMovie Event CoreData database (.fcpevent)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_healthkit.sqlite",
                "Apple iOS HealthKit database (healthdb_secure.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_imessage.sqlite",
                "Apple iMessage / SMS database (sms.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_ios_powerlog.sqlite",
                "Apple iOS Powerlog telemetry database (CurrentPowerlog.PLSQL)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_iphoto_lib.sqlite",
                "Apple iPhoto / Aperture library database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_iphoto_rootpixels.sqlite",
                "Apple iPhoto / Aperture RootPixels database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_itunes_extras.sqlite",
                "Apple iTunes Library Extras CDDB database (.itdb)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_knowledgec.sqlite",
                "Apple KnowledgeC CoreData database (knowledgeC.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_mail_envelope.sqlite",
                "Apple Mail Envelope Index database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_netusage.sqlite",
                "Apple Network Usage CoreData database (netusage.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_notes_legacy.sqlite",
                "Apple macOS Legacy Notes CoreData database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_notes_modern.sqlite",
                "Apple Notes CoreData database (NoteStore.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_notif_center.sqlite",
                "Apple macOS Notification Center database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_photos_preview.sqlite",
                "Apple Photos preview cache",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_quarantine.sqlite",
                "Apple macOS QuarantineEventsV2 database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_screentime.sqlite",
                "Apple Screen Time CoreData database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_tcc.sqlite",
                "Apple TCC permissions database (TCC.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "apple_webkit_db.sqlite",
                "Apple WebKit database info store (__WebKitDatabaseInfoTable__)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("apsim_db.sqlite", "APSIM simulator database", Some("fmt/729"), Some("fdd000461")),
            ("audacity_3.aup3", "Audacity 3 project (.aup3)", Some("fmt/1826"), None),
            (
                "autocad_dimpts.sqlite",
                "AutoCAD dimension points cache",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "autocad_fdo.sqlite",
                "AutoCAD Map 3D FDO spatial database (.sdf)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "autopsy_tsk.sqlite",
                "The Sleuth Kit / Autopsy forensic database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "balsamiq_bmpr.bmpr",
                "Balsamiq Mockups project (.bmpr)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("beeref_scene.bee", "BeeRef scene (.bee)", Some("fmt/729"), Some("fdd000461")),
            (
                "bentley_besqlite.sqlite",
                "Bentley BeSQLite database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "binary_ninja.bndb",
                "Binary Ninja analysis database (.bndb)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "bindiff_db.BinDiff",
                "BinDiff binary comparison database (.BinDiff)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "bitcoin_wallet.sqlite",
                "Bitcoin Core descriptor wallet",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "bnsql_export.sqlite",
                "Binary Ninja SQLite export database (BNSQL)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "calibre_library.sqlite",
                "Calibre library database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "cern_cms_conddb.sqlite",
                "CERN CMS CondDB database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "cewe_photobook.mcfx",
                "CEWE Photobook project (.mcfx)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "cherrytree_note.ctb",
                "CherryTree note document (.ctb)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromadb.sqlite",
                "ChromaDB vector database (chroma.sqlite3)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_affiliation.sqlite",
                "Chromium Password Affiliation database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_app_cache.sqlite",
                "Chromium HTML5 Application Cache",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_attribution.sqlite",
                "Chromium Attribution Reporting database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_cookies.sqlite",
                "Chromium Cookies database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_dips.sqlite",
                "Chromium Bounce Tracking Mitigations database (DIPS)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_ext_activity.sqlite",
                "Chromium Extension Activity database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_favicons.sqlite",
                "Chromium Favicons database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_fledge.sqlite",
                "Chromium Protected Audience database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_fps.sqlite",
                "Chromium First-Party Sets database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_history.sqlite",
                "Chromium History database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_history_embeddings.sqlite",
                "Chromium History Embeddings database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_history_index.sqlite",
                "Chromium full-text History Index database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_indexeddb.sqlite",
                "Chromium IndexedDB database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_localstorage.sqlite",
                "Chromium LocalStorage (ItemTable)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_login_data.sqlite",
                "Chromium Login Data database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_nel.sqlite",
                "Chromium Reporting and NEL database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_net_predictor.sqlite",
                "Chromium Network Action Predictor database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_origin_certs.sqlite",
                "Chromium Origin Bound Certs database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_power_bookmarks.sqlite",
                "Chromium Power Bookmarks database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_quota_manager.sqlite",
                "Chromium QuotaManager database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_shared_storage.sqlite",
                "Chromium Shared Storage database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_shortcuts.sqlite",
                "Chromium Shortcuts database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_top_sites.sqlite",
                "Chromium Top Sites database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_topics.sqlite",
                "Chromium Browsing Topics database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_trust_tokens.sqlite",
                "Chromium Trust Tokens database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_ukm.sqlite",
                "Chromium URL-Keyed Metrics database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "chromium_web_data.sqlite",
                "Chromium Web Data database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "clip_studio_paint.sqlite",
                "Clip Studio Paint artwork (.clip)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "colmap_db.sqlite",
                "COLMAP 3D reconstruction database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "corsair_icue.sqlite",
                "Corsair iCUE package registry",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "darktable_data.sqlite",
                "Darktable styles and presets database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "darktable_lib.sqlite",
                "Darktable photo library database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "daw_vst_cache.sqlite",
                "Audio DAW VST plugin cache",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "denon_marantz_dmu.sqlite",
                "Denon / Marantz media manual database (.dmu)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "dfanalyzer_zindex.sqlite",
                "DFAnalyzer zindex compressed file index",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "diaphora_db.diaphora",
                "Diaphora program diffing database (.diaphora)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("digikam_db.sqlite", "KDE digiKam photo database", Some("fmt/729"), Some("fdd000461")),
            (
                "drakon_diagram.sqlite",
                "DRAKON flowchart diagram (.drn)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "dropbox_sync_history.sqlite",
                "Dropbox sync history database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "embedded_media_lib.sqlite",
                "Embedded media player library",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "embedded_recent_play.sqlite",
                "Embedded media recent play database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "esri_st_geometry.sqlite",
                "Esri ST_Geometry database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("fail2ban_db.sqlite", "Fail2Ban ban database", Some("fmt/729"), Some("fdd000461")),
            (
                "far_manager_config.sqlite",
                "Far Manager v3 configuration database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "files_by_google.sqlite",
                "Files by Google Android master database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firebase_app_measurement.sqlite",
                "Google Play / Firebase App Measurement database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_addons.sqlite",
                "Mozilla Firefox Add-ons metadata database (addons.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_content_prefs.sqlite",
                "Mozilla Firefox Content Preferences database (content-prefs.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_cookies.sqlite",
                "Mozilla Firefox Cookies database (cookies.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_downloads.sqlite",
                "Mozilla Firefox Legacy Downloads database (downloads.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_extensions.sqlite",
                "Mozilla Firefox Extensions database (extensions.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_favicons.sqlite",
                "Mozilla Firefox Favicons database (favicons.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_form_history.sqlite",
                "Mozilla Firefox Form History database (formhistory.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_health_report.sqlite",
                "Mozilla Firefox Health Report database (healthreport.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_indexeddb.sqlite",
                "Mozilla Firefox IndexedDB database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_offline_cache.sqlite",
                "Mozilla Firefox Offline Application Cache",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_permissions.sqlite",
                "Mozilla Firefox Site Permissions database (permissions.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_places.sqlite",
                "Mozilla Firefox Places database (places.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_remote_settings.sqlite",
                "Mozilla Firefox Remote Settings database (kinto.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_signons.sqlite",
                "Mozilla Firefox Legacy Signons database (signons.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_storage.sqlite",
                "Mozilla Firefox Storage Manager database (storage.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_url_classifier.sqlite",
                "Mozilla Firefox SafeBrowsing URL Classifier database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "firefox_webappsstore.sqlite",
                "Mozilla Firefox DOM LocalStorage database (webappsstore.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "flare_shellcode_hashes.sqlite",
                "FLARE shellcode API hashes database (sc_hashes.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "forensicstore_elem.sqlite",
                "ForensicStore container (.forensicstore)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("fossil_checkout.sqlite", "Fossil SCM checkout", Some("fmt/729"), Some("fdd000461")),
            ("fossil_repo.fossil", "Fossil SCM repository", Some("fmt/729"), Some("fdd000461")),
            (
                "gameanalytics_cache.sqlite",
                "GameAnalytics SDK event cache",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "geopackage_schema.sqlite",
                "OGC GeoPackage (schema)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "ghidrasql_export.sqlite",
                "Ghidra SQLite export database (GhidraSQL)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "gnome_tracker_sparql.sqlite",
                "GNOME Tracker SPARQL store",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "gnome_zeitgeist.sqlite",
                "GNOME Zeitgeist activity database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "gnupg_keybox.sqlite",
                "GnuPG 2 Keybox keyring (pubring.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "gnupg_tofu.sqlite",
                "GnuPG 2 TOFU policy database (tofu.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "google_analytics_v4.sqlite",
                "Google Analytics Android SDK v4 database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "google_drive_snapshot.sqlite",
                "Google Drive Backup and Sync snapshot database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "google_hangouts.sqlite",
                "Google Hangouts Android conversations database (babel.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "google_safe_browsing.sqlite",
                "Google Safe Browsing chunk database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "grafana_db.sqlite",
                "Grafana database (grafana.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "home_assistant_db.sqlite",
                "Home Assistant recorder database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "idarling_db.sqlite",
                "IDArling collaborative IDA server database (idarling.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "idasql_export.sqlite",
                "IDA Pro SQLite export database (IDASQL)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "internet_archive_meta.sqlite",
                "Internet Archive S3 per-key metadata database (*_meta.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "ios_imo_hd.sqlite",
                "IMO HD Messenger iOS chat database (IMODb2.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "ios_instagram_direct.sqlite",
                "Instagram Direct iOS threads database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "ios_kik_messenger.sqlite",
                "Kik Messenger iOS chat database (kik.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "ios_manifest.sqlite",
                "Apple iOS Backup Manifest (Manifest.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "ios_twitter.sqlite",
                "Twitter / X iOS database (twitter.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "ipython_history.sqlite",
                "IPython shell history database (history.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "jellyfin_library.sqlite",
                "Jellyfin / Emby media library database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "jupyter_nbsig.sqlite",
                "Jupyter Notebook trust signatures database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "k3s_kine_db.sqlite",
                "K3s / Kine Kubernetes etcd-shim database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "kaspersky_kav_reports.sqlite",
                "Kaspersky Anti-Virus reports database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "kaspersky_kvdb.sqlite",
                "Kaspersky key-value settings database (.kvdb)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("kde_kstars.kscat", "KDE KStars catalog (.kscat)", Some("fmt/729"), Some("fdd000461")),
            (
                "kismet_capture.kismet",
                "Kismet wireless capture database (.kismet)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "kodi_videos.sqlite",
                "Kodi video library database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "langchain_cache.sqlite",
                "LangChain LLM cache database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "libchewing_phrase_db.sqlite",
                "libchewing user phrase database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "logitech_ghub.sqlite",
                "Logitech G HUB / Logi Options+ settings database (settings.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "macos_app_usage.sqlite",
                "Apple macOS Application Usage database (application_usage.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "mandoc_db.sqlite",
                "BSD mandoc manual page index",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("maple_workbook.maple", "Maple workbook (.maple)", Some("fmt/729"), Some("fdd000461")),
            ("mbtiles_schema.sqlite", "MBTiles (schema)", Some("fmt/729"), Some("fdd000461")),
            (
                "minecraft_coreprotect.sqlite",
                "Minecraft CoreProtect database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "minecraft_lwc.sqlite",
                "Minecraft LWC protection database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "minecraft_plotme.sqlite",
                "Minecraft PlotMe database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "mlflow_db.sqlite",
                "MLflow tracking database (mlflow.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "money_manager_ex.mmb",
                "Money Manager Ex database (.mmb)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "monotone_repo.mtn",
                "Monotone SCM repository (.mtn)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "mozilla_nss_cert9.sqlite",
                "Mozilla NSS Public Certificate store (cert9.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "mozilla_nss_key4.sqlite",
                "Mozilla NSS Private Key store (key4.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "nix_binary_cache.sqlite",
                "Nix binary cache metadata database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "office_c2r.sqlite",
                "Microsoft Office Click-to-Run database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "one_c_eventlog.sqlite",
                "1C:Enterprise 8 event log database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "opencode_session.sqlite",
                "OpenCode AI coding agent session database (opencode.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "opera_load_stats.sqlite",
                "Opera Load Statistics database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "opera_session.sqlite",
                "Opera Browser Session database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "optuna_db.sqlite",
                "Optuna study database (optuna.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "osgeo_proj.sqlite",
                "OSGeo PROJ geodetic registry (proj.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "paessler_prtg.sqlite",
                "Paessler PRTG Network Monitor database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "pdbsql_export.sqlite",
                "Microsoft PDB SQLite export database (PDBSQL)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "photothumb_cache.sqlite",
                "Photo thumbnail cache database (photothumb.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "plaso_storage.plaso",
                "Plaso forensic event storage (.plaso)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "plaso_winevt_rc.sqlite",
                "Plaso Windows Event Log resources database (winevt-rc.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "plex_library.sqlite",
                "Plex Media Server library database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "python_dbm_sqlite3.sqlite",
                "Python dbm.sqlite3 database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "qt_compressed_help.qch",
                "Qt Compressed Help documentation database (.qch)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "qt_help_collection.qhc",
                "Qt Help Collection index database (.qhc)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "quick_macros_2.sqlite",
                "Quick Macros 2 automation database (.qml / .qm2)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("reflet.reflet", "Reflet database (.reflet)", Some("fmt/729"), Some("fdd000461")),
            (
                "resonic_player.sqlite",
                "Resonic Player media database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "rpm_repo_metadata.sqlite",
                "RPM/YUM/DNF repository metadata database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "rti_apex_project.sqlite",
                "RTI Integration Designer APEX project database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "safari_history.sqlite",
                "Apple Safari History database (History.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "safari_storage_tracker.sqlite",
                "Apple Safari LocalStorage tracker (StorageTracker.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "safari_webpage_icons.sqlite",
                "Apple Safari WebpageIcons database (WebpageIcons.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "sdl_trados.sdltm",
                "SDL Trados translation memory (.sdltm)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "sidechannelmarvels_tracer.sqlite",
                "SideChannelMarvels TracerGrind / TracerPIN trace database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "sketch_doc.sketch",
                "Sketch design document (.sketch)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "skype_config_db.sqlite",
                "Microsoft Skype configuration database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "skype_main_db.sqlite",
                "Microsoft Skype main database (main.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "sourcetrail_db.srctrldb",
                "Sourcetrail source index database (.srctrldb)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "spatialite.sqlite",
                "SpatiaLite spatial database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            ("sqlar.sqlar", "SQLite Archive (.sqlar)", Some("fmt/729"), Some("fdd000461")),
            (
                "superantispyware_db.sqlite",
                "SUPERAntiSpyware configuration database (.db3)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "svn_rep_cache.sqlite",
                "Apache Subversion FSFS representation cache (rep-cache.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "svn_wc.sqlite",
                "Apache Subversion working copy database (wc.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "tango_android.sqlite",
                "Tango Messenger Android database (tc.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "thunderbird_global_msgs.sqlite",
                "Mozilla Thunderbird Global Messages database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "topcon_survey_job.tsj",
                "Topcon Survey Job database (.tsj)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "twinmotion_item.tmi",
                "Twinmotion library item (.tmi)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "twinmotion_project.tm",
                "Twinmotion project (.tm)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "unity_asset_versioning.sqlite",
                "Unity Editor Asset Versioning database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "unity_shader_cache.sqlite",
                "Unity Editor Shader Cache (ShaderCache.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "visual_paradigm.sqlite",
                "Visual Paradigm UML project (.vpp)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "visual_studio_storage_ide.sqlite",
                "Microsoft Visual Studio solution cache (storage.ide)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "vivaldi_calendar.sqlite",
                "Vivaldi Browser Calendar database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "vs_cpp_browse.sqlite",
                "Microsoft Visual Studio C++ browse database (.browse.VC.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "vs_slnx_index.sqlite",
                "Microsoft Visual Studio workspace index (slnx.sqlite)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "vscode_state.sqlite",
                "VS Code workspace state (state.vscdb)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "wazuh_inventory.sqlite",
                "Wazuh Syscollector inventory database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "whatsapp_axolotl.sqlite",
                "WhatsApp Signal-protocol key store (axolotl.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "whatsapp_contacts.sqlite",
                "WhatsApp Contacts database (wa.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "whatsapp_msgstore.sqlite",
                "WhatsApp Messages database (msgstore.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "windows_asimov.sqlite",
                "Microsoft Windows Asimov EventStore database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "windows_eventtranscript.sqlite",
                "Microsoft Windows EventTranscript database (EventTranscript.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "windows_staterepository.sqlite",
                "Microsoft Windows AppX StateRepository database (.srd)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "windows_timeline.sqlite",
                "Microsoft Windows Timeline database (ActivitiesCache.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "windows_update_store.sqlite",
                "Microsoft Windows Update Store database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "windows_wpn.sqlite",
                "Microsoft Windows Push Notification database (wpndatabase.db)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "wowool_nlp_model.sqlite",
                "Wowool NLP linguistic model database (.dom / .hmm)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "xklb_media.sqlite",
                "xklb media library database",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
            (
                "yandex_autofill.sqlite",
                "Yandex Browser autofill database (Ya Autofill Data)",
                Some("fmt/729"),
                Some("fdd000461"),
            ),
        ];

        let mut actual_files: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        actual_files.sort();
        let expected_files: Vec<String> =
            expected.iter().map(|(f, _, _, _)| (*f).to_string()).collect();
        assert_eq!(actual_files, expected_files);

        let mut failures: Vec<String> = Vec::new();
        for &(filename, expected_sub, expected_puid, expected_fdd) in expected {
            assert!(expected_sub.is_ascii(), "non-ASCII subformat {expected_sub:?}");
            let path = dir.join(filename);
            let mut file_input = Input::from_file(std::fs::File::open(&path).unwrap()).unwrap();
            let (cts, meta) = identify_with_metadata(&mut file_input).unwrap();
            if cts != vec![ContentType::Sqlite]
                || meta.get("subformat").map(String::as_str) != Some(expected_sub)
                || meta.get("pronom_puid").map(String::as_str) != expected_puid
                || meta.get("loc_fdd").map(String::as_str) != expected_fdd
            {
                failures.push(format!(
                    "FILE {filename}: cts={cts:?} sub={:?} (exp={expected_sub:?}) puid={:?} (exp={expected_puid:?}) fdd={:?} (exp={expected_fdd:?})",
                    meta.get("subformat"),
                    meta.get("pronom_puid"),
                    meta.get("loc_fdd")
                ));
                continue;
            }

            let bytes = std::fs::read(&path).unwrap();
            let (cts, meta) = resolve_slice_with_metadata(&bytes);
            if cts != vec![ContentType::Sqlite]
                || meta.get("subformat").map(String::as_str) != Some(expected_sub)
            {
                failures.push(format!(
                    "SLICE {filename}: cts={cts:?} sub={:?} (exp={expected_sub:?})",
                    meta.get("subformat")
                ));
            }
        }
        assert!(
            failures.is_empty(),
            "{} SQLite subformat fixture failure(s):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn resolvers_sqlite_schema_subformats_edge_cases() {
        // Generic existing SQLite fixtures remain ContentType::Sqlite without a subformat.
        for path in [
            "../../tests_data/basic/sqlite/weather_stations.sqlite",
            "../../tests_data/previous_missdetections/sqlite/test-gh-616.db",
            "../../tests_data/resolvers/fixtures/database/sqlite.sqlite",
        ] {
            let mut input = Input::from_file(std::fs::File::open(path).unwrap()).unwrap();
            let (cts, meta) = identify_with_metadata(&mut input).unwrap();
            assert_eq!(cts, vec![ContentType::Sqlite], "expected sqlite on {path}");
            assert_eq!(meta.get("subformat"), None, "unexpected subformat on {path}");
            assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/729"));
            assert_eq!(meta.get("loc_fdd").map(String::as_str), Some("fdd000461"));
        }

        // Generic schemas and Tier-3 table names with unrelated columns must not false-positive.
        for tables in [
            &[("users", "CREATE TABLE users(id INT, name TEXT)")][..],
            &[("cache", "CREATE TABLE cache(k TEXT, v BLOB)")],
            &[("files", "CREATE TABLE files(id INT, name TEXT)")],
            &[("main", "CREATE TABLE main(a INT, b INT)")],
            &[("Dict", "CREATE TABLE Dict(k TEXT, v TEXT)")],
            &[("ItemTable", "CREATE TABLE ItemTable(id INT, label TEXT)")],
            &[("sync_history", "CREATE TABLE sync_history(id INT, status TEXT)")],
            &[("global_config", "CREATE TABLE global_config(k TEXT, v TEXT)")],
            &[("application_usage", "CREATE TABLE application_usage(id INT, name TEXT)")],
            &[
                ("android_metadata", "CREATE TABLE android_metadata(locale TEXT)"),
                ("cache", "CREATE TABLE cache(k TEXT, v BLOB)"),
            ],
            &[
                ("data", "CREATE TABLE data(id INT, body BLOB)"),
                ("snapshots", "CREATE TABLE snapshots(id INT, ts INT)"),
            ],
            &[("cookies", "CREATE TABLE cookies(id INT, name TEXT)")],
            &[("messages", "CREATE TABLE messages(id INT, body TEXT)")],
            // Embedded newline in a single table name cannot spoof multi-table S1 ("cookies\nmeta").
            &[("cookies\nmeta", "CREATE TABLE \"cookies\nmeta\"(x INT)")],
            // Embedded pipe in a single quoted column name cannot spoof 5-column sqlar S4.
            &[("sqlar", "CREATE TABLE sqlar(\"name|mode|mtime|sz|data\" TEXT)")],
        ] {
            let db = build_sqlite_leaf_db(4096, 1, tables);
            let (cts, meta) = resolve_slice_with_metadata(&db);
            assert_eq!(cts, vec![ContentType::Sqlite]);
            assert_eq!(meta.get("subformat"), None, "unexpected match on {tables:?}");
        }

        // Tier-3 exact DDL & S4 matches across UTF-8 and UTF-16LE.
        type Tier3Case<'a> = (u32, &'a [(&'a str, &'a str)], &'a str);
        let tier3_cases: &[Tier3Case<'_>] = &[
            (
                1,
                &[(
                    "ItemTable",
                    "CREATE TABLE ItemTable (key TEXT UNIQUE ON CONFLICT REPLACE, value BLOB)",
                )],
                "VS Code workspace state (state.vscdb)",
            ),
            (
                2,
                &[("ItemTable", "CREATE TABLE ItemTable (key TEXT, value BLOB)")],
                "Chromium LocalStorage (ItemTable)",
            ),
            (
                1,
                &[(
                    "main",
                    "CREATE TABLE main(key BLOB PRIMARY KEY NOT NULL, value BLOB NOT NULL)",
                )],
                "Bitcoin Core descriptor wallet",
            ),
            (
                1,
                &[("cache", "CREATE TABLE cache(key PRIMARY KEY, value)")],
                "AMD DirectX shader cache (.dxcache)",
            ),
            (
                1,
                &[(
                    "Cache",
                    "CREATE TABLE Cache (\n    domain    text not null,\n    key       text not null,\n    value     text not null,\n    timestamp integer not null,\n    primary key (domain, key)\n)",
                )],
                "Nix fetcher cache",
            ),
            (
                1,
                &[(
                    "Files",
                    "CREATE TABLE Files(Filename TEXT PRIMARY KEY,Data BLOB,LastModified INTEGER)",
                )],
                "CEWE Photobook project (.mcfx)",
            ),
            (
                1,
                &[(
                    "locks",
                    "CREATE TABLE locks(   lock_file varchar(32),   unique_name varchar(32))",
                )],
                "Python lockfile.sqlitelockfile database",
            ),
            (
                2,
                &[("Origins", "CREATE TABLE Origins (origin TEXT, path TEXT)")],
                "Apple Safari LocalStorage tracker (StorageTracker.db)",
            ),
            (
                1,
                &[
                    (
                        "tObjDimPts",
                        "CREATE TABLE tObjDimPts(id INT, handle INT, dimblkname TEXT, pt1x REAL, pt1y REAL, pt2x REAL, pt2y REAL, pt3x REAL, pt3y REAL, pt4x REAL, pt4y REAL, pt5x REAL, pt5y REAL, textrot REAL, dimscale REAL, dimstyle TEXT, leaderlen REAL, oblique REAL)",
                    ),
                    (
                        "tables_version",
                        "CREATE TABLE tables_version(table_name TEXT, table_version INT)",
                    ),
                ],
                "AutoCAD dimension points cache",
            ),
            (
                1,
                &[(
                    "sync_history",
                    "CREATE TABLE sync_history (event_type TEXT NOT NULL,file_event_type TEXT,direction TEXT,file_id TEXT,local_path TEXT,other_user INTEGER,timestamp INTEGER NOT NULL)",
                )],
                "Dropbox sync history database",
            ),
        ];
        for &(enc, tables, expected_sub) in tier3_cases {
            let db = build_sqlite_leaf_db(4096, enc, tables);
            let (cts, meta) = resolve_slice_with_metadata(&db);
            assert_eq!(cts, vec![ContentType::Sqlite]);
            assert_eq!(
                meta.get("subformat").map(String::as_str),
                Some(expected_sub),
                "Tier-3 mismatch for enc={enc} tables={tables:?}"
            );
        }

        // Plaso storage without `event_tag` / `session_start` resolves via `event_data` +
        // `metadata` + `event_source` + `parser_count`.
        let legacy_plaso = build_sqlite_leaf_db(
            4096,
            1,
            &[
                ("event_data", "CREATE TABLE event_data(_identifier INT, _data BLOB)"),
                ("event_source", "CREATE TABLE event_source(_identifier INT, data_type TEXT)"),
                ("metadata", "CREATE TABLE metadata(key TEXT, value TEXT)"),
                ("parser_count", "CREATE TABLE parser_count(_identifier INT, name TEXT)"),
            ],
        );
        let (cts, meta) = resolve_slice_with_metadata(&legacy_plaso);
        assert_eq!(cts, vec![ContentType::Sqlite]);
        assert_eq!(
            meta.get("subformat").map(String::as_str),
            Some("Plaso forensic event storage (.plaso)")
        );

        // Audacity 3 schema overrides PRONOM PUID to fmt/1826 and clears SQLite's generic LoC FDD.
        let audacity = build_sqlite_leaf_db(
            4096,
            1,
            &[
                ("project", "CREATE TABLE project(id INT)"),
                ("sampleblocks", "CREATE TABLE sampleblocks(blockid INT)"),
            ],
        );
        let (cts, meta) = resolve_slice_with_metadata(&audacity);
        assert_eq!(cts, vec![ContentType::Sqlite]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("Audacity 3 project (.aup3)"));
        assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/1826"));
        assert_eq!(meta.get("loc_fdd"), None);

        // Encoding-gated UTF-16BE anchor match (`cachedplugins`).
        let vst_be =
            build_sqlite_leaf_db(4096, 3, &[("CachedPlugins", "CREATE TABLE CachedPlugins(id)")]);
        let (cts, meta) = resolve_slice_with_metadata(&vst_be);
        assert_eq!(cts, vec![ContentType::Sqlite]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("Audio DAW VST plugin cache"));
        let vst_utf8 =
            build_sqlite_leaf_db(4096, 1, &[("CachedPlugins", "CREATE TABLE CachedPlugins(id)")]);
        let (_, meta) = resolve_slice_with_metadata(&vst_utf8);
        assert_eq!(meta.get("subformat"), None);

        // Tier-3 DDL spanning an overflow page with SQL comments, quoted identifiers, and table
        // constraints still extracts ordered columns and resolves `sqlar`.
        let pad = " ".repeat(430);
        let overflow_sql = format!(
            "CREATE TABLE sqlar /* header comment {pad} */ (\n\
             -- line comment\n\
             \"name\" TEXT PRIMARY KEY, `mode` INT, [mtime] INT, sz INT, data BLOB,\n\
             UNIQUE (name))"
        );
        let overflow_db = build_sqlite_overflow_tier3_db("sqlar", &overflow_sql);
        let (cts, meta) = resolve_slice_with_metadata(&overflow_db);
        assert_eq!(cts, vec![ContentType::Sqlite]);
        assert_eq!(meta.get("subformat").map(String::as_str), Some("SQLite Archive (.sqlar)"));

        // Lazy Content::Partial (> 64 KiB file on disk) with child leaf page past offset 65536
        // exercises PageReader + Input::read_chunk_at without loading the full file into memory.
        let big_db = build_sqlite_interior_db(
            4096,
            25,
            20,
            1,
            &[
                ("cookies", "CREATE TABLE cookies(k TEXT)"),
                ("meta", "CREATE TABLE meta(k TEXT, v TEXT)"),
            ],
        );
        assert!(big_db.len() > 65536);
        let tmp_path = std::env::temp_dir().join(format!(
            "magika_sqlite_partial_test_{}_{:?}.db",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::write(&tmp_path, &big_db).unwrap();
        let file_res = {
            let mut input = Input::from_file(std::fs::File::open(&tmp_path).unwrap()).unwrap();
            identify_with_metadata(&mut input).unwrap()
        };
        let _ = std::fs::remove_file(&tmp_path);
        assert_eq!(file_res.0, vec![ContentType::Sqlite]);
        assert_eq!(
            file_res.1.get("subformat").map(String::as_str),
            Some("Chromium Cookies database")
        );

        // Adversarial: cyclic interior B-tree (page 1 -> page 2 -> page 1).
        let mut cyclic_btree = vec![0u8; 8192];
        init_sqlite_header(&mut cyclic_btree, 4096, 1);
        cyclic_btree[100] = 0x05;
        cyclic_btree[108..112].copy_from_slice(&2u32.to_be_bytes());
        cyclic_btree[4096] = 0x05;
        cyclic_btree[4096 + 8..4096 + 12].copy_from_slice(&1u32.to_be_bytes());
        let (cts, meta) = resolve_slice_with_metadata(&cyclic_btree);
        assert_eq!(cts, vec![ContentType::Sqlite]);
        assert_eq!(meta.get("subformat"), None);

        // Adversarial: cyclic overflow page chain on a Tier-3 candidate (page 2 -> page 2 when
        // the declared SQL length requires a second overflow page).
        let long_pad = " ".repeat(950);
        let long_sql = format!(
            "CREATE TABLE sqlar /* {long_pad} */ (name TEXT, mode INT, mtime INT, sz INT, data BLOB)"
        );
        let rec = build_sqlite_schema_record("sqlar", &long_sql, 1);
        let max_local = 512usize - 35;
        let min_local = ((512usize - 12) * 32 / 255).saturating_sub(23);
        let k = min_local + ((rec.len() - min_local) % (512 - 4));
        let local_len = if k <= max_local { k } else { min_local };
        assert!(rec.len() > local_len + 508, "must require >= 2 overflow pages");
        let mut cyclic_ovf = vec![0u8; 512 * 2];
        init_sqlite_header(&mut cyclic_ovf, 512, 1);
        cyclic_ovf[100] = 0x0D;
        cyclic_ovf[103..105].copy_from_slice(&1u16.to_be_bytes());
        let mut cell = Vec::new();
        encode_sqlite_varint(rec.len() as u64, &mut cell);
        encode_sqlite_varint(1, &mut cell);
        cell.extend_from_slice(&rec[..local_len]);
        cell.extend_from_slice(&2u32.to_be_bytes());
        let cell_off = 512 - cell.len();
        cyclic_ovf[cell_off..512].copy_from_slice(&cell);
        cyclic_ovf[108..110].copy_from_slice(&(cell_off as u16).to_be_bytes());
        cyclic_ovf[105..107].copy_from_slice(&(cell_off as u16).to_be_bytes());
        // Page 2 loops back to page 2.
        cyclic_ovf[512..516].copy_from_slice(&2u32.to_be_bytes());
        cyclic_ovf[516..1024].copy_from_slice(&rec[local_len..local_len + 508]);
        let (cts, meta) = resolve_slice_with_metadata(&cyclic_ovf);
        assert_eq!(cts, vec![ContentType::Sqlite]);
        assert_eq!(meta.get("subformat"), None);

        // Adversarial: malformed 9-byte 0xFF varints and out-of-bounds cell pointer.
        let mut corrupt_varint = vec![0u8; 1024];
        init_sqlite_header(&mut corrupt_varint, 1024, 1);
        corrupt_varint[100] = 0x0D;
        corrupt_varint[103..105].copy_from_slice(&2u16.to_be_bytes());
        corrupt_varint[108..110].copy_from_slice(&200u16.to_be_bytes());
        corrupt_varint[110..112].copy_from_slice(&1020u16.to_be_bytes());
        corrupt_varint[200..232].fill(0xFF);
        let (cts, meta) = resolve_slice_with_metadata(&corrupt_varint);
        assert_eq!(cts, vec![ContentType::Sqlite]);
        assert_eq!(meta.get("subformat"), None);

        // Incomplete walk (`schema_complete == false`): exact-only S1 does not fire when a
        // sibling page is missing, whereas an anchor-table match still resolves gracefully.
        let mut s1_only = build_sqlite_interior_db(
            4096,
            2,
            2,
            1,
            &[
                ("opm_data", "CREATE TABLE opm_data(k)"),
                ("opm_data_v2", "CREATE TABLE opm_data_v2(k)"),
                ("opm_meta", "CREATE TABLE opm_meta(k)"),
            ],
        );
        let (_, meta_complete) = resolve_slice_with_metadata(&s1_only);
        assert_eq!(
            meta_complete.get("subformat").map(String::as_str),
            Some("Adobe Creative Cloud OOBE policy database")
        );
        s1_only[103..105].copy_from_slice(&1u16.to_be_bytes());
        s1_only[112..114].copy_from_slice(&4000u16.to_be_bytes());
        s1_only[4000..4004].copy_from_slice(&999u32.to_be_bytes());
        let (_, meta_incomplete) = resolve_slice_with_metadata(&s1_only);
        assert_eq!(meta_incomplete.get("subformat"), None);

        let mut anchor_incomplete = build_sqlite_interior_db(
            4096,
            2,
            2,
            1,
            &[
                ("moz_places", "CREATE TABLE moz_places(id INT)"),
                ("moz_historyvisits", "CREATE TABLE moz_historyvisits(id INT)"),
            ],
        );
        anchor_incomplete[103..105].copy_from_slice(&1u16.to_be_bytes());
        anchor_incomplete[112..114].copy_from_slice(&4000u16.to_be_bytes());
        anchor_incomplete[4000..4004].copy_from_slice(&999u32.to_be_bytes());
        let (cts, meta_anchor) = resolve_slice_with_metadata(&anchor_incomplete);
        assert_eq!(cts, vec![ContentType::Sqlite]);
        assert_eq!(
            meta_anchor.get("subformat").map(String::as_str),
            Some("Mozilla Firefox Places database (places.sqlite)")
        );
    }
}
