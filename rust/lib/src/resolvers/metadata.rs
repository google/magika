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

use std::collections::HashMap;

use crate::ContentType;
use crate::resolvers::magic::Subformat;

/// Populates the metadata of a content type identified by the resolvers.
///
/// Keys (no SemVer guarantees, see [`crate::RegularFileType::metadata`]):
/// - `subformat`: finer-grained variant already told apart by the verifiers;
/// - `pronom_puid`: UK National Archives PRONOM persistent unique identifier;
/// - `loc_fdd`: US Library of Congress format description document identifier.
///
/// Registry identifiers of the subformat take precedence over those of the content type.
pub(crate) fn populate(
    content_type: ContentType, subformat: Option<&Subformat>,
    metadata: &mut HashMap<String, String>,
) {
    let (mut pronom_puid, mut loc_fdd) = standard_ids(content_type);
    if let Some(subformat) = subformat {
        metadata.insert("subformat".to_string(), subformat.name.to_string());
        if subformat.pronom_puid.is_some() {
            pronom_puid = subformat.pronom_puid;
            loc_fdd = subformat.loc_fdd;
        } else if subformat.loc_fdd.is_some() {
            loc_fdd = subformat.loc_fdd;
        }
    }
    if let Some(puid) = pronom_puid {
        metadata.insert("pronom_puid".to_string(), puid.to_string());
    }
    if let Some(fdd) = loc_fdd {
        metadata.insert("loc_fdd".to_string(), fdd.to_string());
    }
}

/// PRONOM PUID and LoC FDD of a content type, when the registries have one for the whole type.
pub(crate) const fn standard_ids(
    content_type: ContentType,
) -> (Option<&'static str>, Option<&'static str>) {
    match content_type {
        ContentType::Alembic => (None, Some("fdd000560")),
        ContentType::Ani => (Some("fmt/386"), None),
        ContentType::Apk => (Some("fmt/2041"), Some("fdd000592")),
        ContentType::Applebplist => (Some("fmt/984"), None),
        ContentType::Appledouble => (Some("fmt/503"), Some("fdd000625")),
        ContentType::Appleplist => (Some("fmt/979"), None),
        ContentType::Applesingle => (Some("fmt/968"), None),
        ContentType::Arcgrid => (None, Some("fdd000421")),
        ContentType::Arj => (Some("fmt/610"), None),
        ContentType::Arrow => (Some("fmt/2053"), None),
        ContentType::Asf => (Some("fmt/131"), Some("fdd000067")),
        ContentType::Au => (Some("x-fmt/139"), None),
        ContentType::Avi => (Some("fmt/5"), Some("fdd000059")),
        ContentType::Avif => (Some("fmt/2062"), Some("fdd000540")),
        ContentType::Avro => (Some("fmt/2029"), None),
        ContentType::Batch => (Some("x-fmt/413"), None),
        ContentType::Blend => (None, Some("fdd000559")),
        ContentType::Bpg => (Some("fmt/687"), None),
        ContentType::Bzip => (Some("x-fmt/268"), Some("fdd000600")),
        ContentType::Cab => (Some("x-fmt/414"), None),
        ContentType::Cbor => (None, Some("fdd000647")),
        ContentType::Chm => (Some("fmt/634"), None),
        ContentType::Collada => (Some("fmt/1209"), None),
        ContentType::Dbase => (None, Some("fdd000325")),
        ContentType::Dex => (Some("fmt/694"), None),
        ContentType::Dicom => (Some("fmt/574"), None),
        ContentType::Docx => (Some("fmt/412"), Some("fdd000397")),
        ContentType::Dotx => (Some("fmt/597"), None),
        ContentType::Dsstore => (Some("fmt/394"), Some("fdd000597")),
        ContentType::Dwg => (None, Some("fdd000445")),
        ContentType::Emf => (Some("fmt/345"), None),
        ContentType::Eml => (Some("fmt/278"), Some("fdd000393")),
        ContentType::Epub => (Some("fmt/483"), Some("fdd000310")),
        ContentType::Erdas => (None, Some("fdd000420")),
        ContentType::Fasta => (None, Some("fdd000622")),
        ContentType::Fastq => (Some("fmt/2118"), None),
        ContentType::Fbx => (None, Some("fdd000558")),
        ContentType::Fits => (Some("x-fmt/383"), Some("fdd000317")),
        ContentType::Flac => (Some("fmt/279"), Some("fdd000198")),
        ContentType::Flv => (Some("x-fmt/382"), Some("fdd000131")),
        ContentType::Fpx => (Some("x-fmt/56"), None),
        ContentType::Geojson => (Some("fmt/1367"), Some("fdd000382")),
        ContentType::Geopackage => (Some("fmt/1700"), Some("fdd000520")),
        ContentType::Gltf => (None, Some("fdd000498")),
        ContentType::Gml => (Some("x-fmt/227"), Some("fdd000296")),
        ContentType::Gzip => (Some("x-fmt/266"), Some("fdd000599")),
        ContentType::H5 => (None, Some("fdd000229")),
        ContentType::Hdf4 => (Some("fmt/1041"), Some("fdd000228")),
        ContentType::Heif => (Some("fmt/1101"), Some("fdd000525")),
        ContentType::Hlp => (Some("fmt/474"), None),
        ContentType::Html => (Some("fmt/96"), Some("fdd000475")),
        ContentType::Icns => (Some("fmt/1185"), None),
        ContentType::Ico => (Some("x-fmt/418"), None),
        ContentType::Ics => (Some("fmt/388"), Some("fdd000394")),
        ContentType::Internetshortcut => (Some("x-fmt/454"), None),
        ContentType::Iosapp => (None, Some("fdd000593")),
        ContentType::Ipynb => (Some("fmt/1119"), None),
        ContentType::Jar => (Some("x-fmt/412"), None),
        ContentType::Javabytecode => (Some("x-fmt/415"), Some("fdd000598")),
        ContentType::Jng => (Some("fmt/529"), None),
        ContentType::Jpeg => (Some("fmt/41"), Some("fdd000017")),
        ContentType::Json => (Some("fmt/817"), Some("fdd000381")),
        ContentType::Kml => (Some("fmt/244"), Some("fdd000340")),
        ContentType::Kmz => (Some("fmt/724"), Some("fdd000547")),
        ContentType::Koala => (Some("fmt/1780"), None),
        ContentType::Latex => (Some("fmt/280"), None),
        ContentType::Lha => (Some("fmt/626"), None),
        ContentType::Lnk => (Some("x-fmt/428"), Some("fdd000596")),
        ContentType::Mapinfo => (None, Some("fdd000300")),
        ContentType::Mat => (Some("fmt/806"), None),
        ContentType::Max => (Some("fmt/978"), None),
        ContentType::Mht => (Some("x-fmt/429"), None),
        ContentType::Midi => (Some("x-fmt/230"), Some("fdd000119")),
        ContentType::Mkv => (Some("fmt/569"), Some("fdd000342")),
        ContentType::Mp2 => (Some("fmt/198"), Some("fdd000338")),
        ContentType::Mp3 => (Some("fmt/134"), Some("fdd000105")),
        ContentType::Mp4 => (Some("fmt/199"), Some("fdd000155")),
        ContentType::Mscompress => (Some("fmt/462"), None),
        ContentType::Mtl => (Some("fmt/1211"), Some("fdd000508")),
        ContentType::Nifti => (Some("fmt/798"), None),
        ContentType::Npy => (Some("fmt/2112"), None),
        ContentType::Obj => (Some("fmt/1210"), Some("fdd000507")),
        ContentType::Odex => (Some("fmt/695"), None),
        ContentType::Odp => (None, Some("fdd000247")),
        ContentType::Ods => (None, Some("fdd000247")),
        ContentType::Odt => (None, Some("fdd000247")),
        ContentType::Ogg => (Some("fmt/944"), Some("fdd000026")),
        ContentType::One => (Some("fmt/637"), None),
        ContentType::Orc => (Some("fmt/2030"), None),
        ContentType::Otf => (Some("fmt/520"), None),
        ContentType::Parquet => (Some("fmt/2023"), Some("fdd000575")),
        ContentType::Pcap => (Some("fmt/779"), None),
        ContentType::Pcapng => (Some("fmt/780"), None),
        ContentType::Pdf => (None, Some("fdd000030")),
        ContentType::Php => (Some("x-fmt/169"), None),
        ContentType::Ply => (Some("fmt/831"), Some("fdd000501")),
        ContentType::Png => (Some("fmt/11"), Some("fdd000153")),
        ContentType::Postscript => (None, Some("fdd000029")),
        ContentType::Pptx => (Some("fmt/215"), Some("fdd000399")),
        ContentType::Proteindb => (Some("fmt/2009"), None),
        ContentType::Psd => (Some("x-fmt/92"), Some("fdd000523")),
        ContentType::Pytorch => (None, Some("fdd000644")),
        ContentType::Qt => (Some("x-fmt/384"), Some("fdd000052")),
        ContentType::Rar => (None, Some("fdd000450")),
        ContentType::Rdata => (None, Some("fdd000470")),
        ContentType::Rdf => (Some("fmt/875"), None),
        ContentType::Rhinoceros => (None, Some("fdd000601")),
        ContentType::Rtf => (None, Some("fdd000473")),
        ContentType::Sas => (Some("fmt/1015"), None),
        ContentType::Segy => (Some("fmt/363"), None),
        ContentType::Sevenzip => (Some("fmt/484"), Some("fdd000539")),
        ContentType::Shapefile => (Some("x-fmt/235"), Some("fdd000280")),
        ContentType::Sketchup => (Some("x-fmt/451"), None),
        ContentType::Spirv => (Some("fmt/1996"), None),
        ContentType::Spss => (Some("fmt/638"), Some("fdd000469")),
        ContentType::Sqlite => (Some("fmt/729"), Some("fdd000461")),
        ContentType::Srt => (Some("fmt/1218"), Some("fdd000569")),
        ContentType::Step => (Some("fmt/698"), Some("fdd000448")),
        ContentType::Stlbinary => (Some("fmt/865"), Some("fdd000505")),
        ContentType::Stltext => (Some("x-fmt/108"), Some("fdd000506")),
        ContentType::Svg => (Some("fmt/92"), Some("fdd000515")),
        ContentType::Swf => (None, Some("fdd000629")),
        ContentType::Tar => (Some("x-fmt/265"), Some("fdd000531")),
        ContentType::Tiff => (Some("fmt/353"), Some("fdd000022")),
        ContentType::Udf => (Some("fmt/1738"), None),
        ContentType::Unixcompress => (Some("fmt/1671"), None),
        ContentType::Usd => (None, Some("fdd000561")),
        ContentType::Vcard => (Some("fmt/395"), Some("fdd000616")),
        ContentType::VcfGenomics => (Some("fmt/908"), None),
        ContentType::Vcs => (Some("fmt/387"), None),
        ContentType::Visio => (Some("fmt/442"), None),
        ContentType::Vtt => (Some("fmt/1454"), Some("fdd000567")),
        ContentType::Webm => (Some("fmt/573"), Some("fdd000518")),
        ContentType::Webp => (None, Some("fdd000577")),
        ContentType::Wkt => (None, Some("fdd000548")),
        ContentType::Wma => (Some("fmt/132"), Some("fdd000027")),
        ContentType::Wmf => (Some("x-fmt/119"), None),
        ContentType::Wmv => (Some("fmt/133"), Some("fdd000091")),
        ContentType::Woff => (Some("fmt/616"), None),
        ContentType::Woff2 => (Some("fmt/1172"), None),
        ContentType::Xar => (Some("fmt/600"), None),
        ContentType::Xcf => (Some("fmt/615"), None),
        ContentType::Xlsb => (Some("fmt/595"), Some("fdd000512")),
        ContentType::Xlsx => (Some("fmt/214"), Some("fdd000398")),
        ContentType::Xml => (Some("fmt/101"), Some("fdd000075")),
        ContentType::Xsd => (Some("x-fmt/280"), Some("fdd000077")),
        ContentType::Xz => (Some("fmt/1098"), None),
        ContentType::Zip => (Some("x-fmt/263"), Some("fdd000354")),
        ContentType::_3dsm => (Some("x-fmt/19"), None),
        ContentType::_3gp => (Some("fmt/357"), None),
        ContentType::_3mf => (Some("fmt/829"), Some("fdd000557")),
        _ => (None, None),
    }
}
