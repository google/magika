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

use anyhow::Result;

use crate::{ContentType, Input};

const PREFIX_LIMIT: usize = 4096;

/// Returns the content types of a file without false positives.
///
/// The returned list may have false negatives (the file should have a content type that's not
/// returned). In particular, the list may be empty, in which case the model will also try to
/// identify the file.
pub(crate) fn identify(file: &mut Input) -> Result<Vec<ContentType>> {
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
        assert_eq!(matched, 311);
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

        // 7. AutoIt text scripts (`autoit`) vs compiled `.a3x` / MZ `AU3!EA0` (`autoit_compiled`)
        let au3 = b"#include <MsgBoxConstants.au3>\nLocal $x = 1\n";
        assert_eq!(resolve_slice(au3), vec![ContentType::Autoit]);
        let mut a3x = vec![0u8; 32];
        let a3x_sig = b"\xA3\x48\x4B\xBE\x98\x6C\x4A\xA9\x99\x4C\x53\x0A\x86\xD6\x48\x7D";
        a3x[..16].copy_from_slice(a3x_sig);
        assert_eq!(resolve_slice(&a3x), vec![ContentType::AutoitCompiled]);
        let mut mz_au3 = vec![0u8; 64];
        mz_au3[..2].copy_from_slice(b"MZ");
        mz_au3[32..40].copy_from_slice(b"AU3!EA06");
        assert_eq!(resolve_slice(&mz_au3), vec![ContentType::AutoitCompiled]);
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
}
