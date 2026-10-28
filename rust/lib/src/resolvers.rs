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

mod facts;
mod magic;

use anyhow::Result;

use crate::cached_input::CachedInput;
use crate::{ContentType, Input};

const PREFIX_LIMIT: usize = 4096;

/// Returns the content types of a file without false positives.
///
/// The returned list may have false negatives (the file should have a content type that's not
/// returned). In particular, the list may be empty, in which case the model will also try to
/// identify the file.
pub(crate) fn identify(file: &mut CachedInput<impl Input>) -> Result<Vec<ContentType>> {
    let size = file.length();
    if size == 0 {
        return Ok(Vec::new());
    }
    let prefix_len = file.ensure_prefix(PREFIX_LIMIT)?;
    let mut tail_len = if facts::wants_tail(file.prefix(prefix_len)) && size > prefix_len as u64 {
        file.ensure_suffix(size.min(facts::ZIP_TAIL_BYTES as u64) as usize)?
    } else {
        0
    };
    if tail_len > 0
        && let Some(start) = facts::directory_start(file.suffix(tail_len), size)
    {
        tail_len = file.ensure_suffix((size - start) as usize)?;
    }
    let prefix = file.prefix(prefix_len);
    let tail = (tail_len > 0).then(|| file.suffix(tail_len));
    let ctx = magic::Ctx { prefix, size, tail };
    Ok(match magic::identify(&ctx) {
        Some(content_type) => vec![content_type],
        None => Vec::new(),
    })
}

/// List of content types for which the resolvers have no false negatives.
///
/// If the resolvers don't return a match for a given file, and the model returns a content type
/// from this list, then that content type will be vetoed and "txt" or "unknown" will be used (based
/// on "is_text" from the content type that was vetoed).
pub(crate) const VETOS: &[ContentType] = &[
    ContentType::Keras,
    ContentType::Qgis,
    ContentType::Visio,
    ContentType::Xlsx,
    ContentType::_3dsm,
    ContentType::_3dsx,
    ContentType::_3gp,
    ContentType::Ace,
    ContentType::Applebplist,
    ContentType::Appledouble,
    ContentType::Applesingle,
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
    ContentType::Jng,
    ContentType::Kml,
    ContentType::LlvmBitcode,
    ContentType::Lnk,
    ContentType::Lrz,
    ContentType::Luabytecode,
    ContentType::Lz,
    ContentType::Lz4,
    ContentType::Mat,
    ContentType::Midi,
    ContentType::Minidump,
    ContentType::Mpegts,
    ContentType::Npy,
    ContentType::Nrrd,
    ContentType::One,
    ContentType::Palmos,
    ContentType::Parquet,
    ContentType::Pbm,
    ContentType::Pgp,
    ContentType::Ply,
    ContentType::PostgresDump,
    ContentType::Psd,
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
    ContentType::Step,
    ContentType::Swf,
    ContentType::Uf2,
    ContentType::Vhd,
    ContentType::Vib,
    ContentType::Wad,
    ContentType::Wasm,
    ContentType::Wav,
    ContentType::Woff,
    ContentType::Woff2,
    ContentType::Xar,
    ContentType::Xcf,
    ContentType::Zst,
    ContentType::Ani,
    ContentType::Arrow,
    ContentType::Pcapng,
    ContentType::Xcoff,
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn resolve_slice(bytes: &[u8]) -> Vec<ContentType> {
        let mut input = CachedInput::new(bytes).unwrap();
        identify(&mut input).unwrap()
    }

    #[test]
    fn vetos_are_distinct_and_complete() {
        let resolvers_vetos: HashSet<ContentType> = VETOS.iter().copied().collect();
        assert_eq!(VETOS.len(), 93);
        assert_eq!(resolvers_vetos.len(), VETOS.len());
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
                let mut input = CachedInput::new(std::fs::File::open(&path).unwrap()).unwrap();
                let got = identify(&mut input).unwrap();
                if let [ct] = got[..] {
                    assert_eq!(ct.info().label, expected, "mismatch on {}", path.display());
                    matched += 1;
                } else {
                    assert!(got.is_empty(), "multiple matches on {}", path.display());
                }
            }
        }
        assert_eq!(matched, 14);
    }

    #[test]
    fn notworking_verifiers_remain_disabled_in_enforced_scan() {
        assert_eq!(magic::notworking::DISABLED.len(), 52);
        let find_verifier = |id: &str| {
            magic::notworking::DISABLED
                .iter()
                .find_map(|&(rid, _, f)| (rid == id).then_some(f))
                .unwrap_or_else(|| panic!("missing disabled verifier {id}"))
        };

        let mut ole2 = vec![0u8; 520];
        ole2[..8].copy_from_slice(b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1");
        ole2[512..516].copy_from_slice(b"\xFD\xFF\xFF\xFF");
        assert!(find_verifier("taxonomy_doc")(&ole2, ole2.len() as u64));
        assert!(find_verifier("taxonomy_ppt")(&ole2, ole2.len() as u64));
        assert!(find_verifier("taxonomy_outlook")(&ole2, ole2.len() as u64));
        assert_eq!(resolve_slice(&ole2), Vec::<ContentType>::new());

        let mut asf = vec![0u8; 64];
        asf[..16]
            .copy_from_slice(b"\x30\x26\xB2\x75\x8E\x66\xCF\x11\xA6\xD9\x00\xAA\x00\x62\xCE\x6C");
        asf[24] = 1;
        asf[28..30].copy_from_slice(&[0x01, 0x02]);
        assert!(find_verifier("taxonomy_asf")(&asf, asf.len() as u64));
        assert!(find_verifier("taxonomy_wma")(&asf, asf.len() as u64));
        assert!(find_verifier("taxonomy_wmv")(&asf, asf.len() as u64));
        assert_eq!(resolve_slice(&asf), Vec::<ContentType>::new());

        let mut ebml = vec![0u8; 64];
        ebml[..4].copy_from_slice(b"\x1A\x45\xDF\xA3");
        assert!(find_verifier("taxonomy_webm")(&ebml, ebml.len() as u64));
        assert_eq!(resolve_slice(&ebml), Vec::<ContentType>::new());
    }

    #[test]
    fn facts_resolvers_identify_archives_and_executables() {
        use facts::{archive, pe32, stored};
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
        assert_eq!(resolve_slice(&pe32().build()), vec![ContentType::Pebin]);
        let resources = facts::Image { characteristics: 0x2102, entry_point: 0, ..pe32() };
        assert_eq!(resolve_slice(&resources.build()), Vec::<ContentType>::new());
    }

    #[test]
    fn a_tail_extends_back_to_a_large_central_directory() {
        use facts::{archive, stored};
        let names: Vec<_> = (0..400).map(|i| format!("{i:0>60}")).collect();
        let entries: Vec<_> = names.iter().map(|name| stored(name.as_bytes(), b"")).collect();
        let bytes = archive(&entries, b"");
        let (prefix, size) = (&bytes[..PREFIX_LIMIT], bytes.len() as u64);
        let tail = &bytes[bytes.len() - facts::ZIP_TAIL_BYTES..];
        let start = facts::directory_start(tail, size).unwrap();
        assert!(start < size - tail.len() as u64);
        let valid = |tail| facts::Facts::analyze(prefix, size, Some(tail)).is_valid();
        assert!(!valid(tail));
        assert!(valid(&bytes[start as usize..]));
    }
}
