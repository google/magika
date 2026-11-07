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

//! Extensible Markup Language (`XML`) and HTML metaformat verifiers (`xml`, `html`, `appleplist`, `collada`, `gml`, `gpx`, `hta`, `jnlp`, `kml`, `mum`, `openvino_ir`, `rdf`, `scriptwsf`, `stata`, `svd`, `svg`, `xsd`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// Flags recorded in a single `memchr('<')` pass over the first 257 bytes for the four
/// floating-XML root-element rules (`collada`, `gpx`, `kml`, `osm`).
#[derive(Clone, Copy, Default)]
pub(crate) struct XmlRoots {
    pub(crate) collada: bool,
    pub(crate) gpx: bool,
    pub(crate) kml: bool,
    pub(crate) osm: bool,
}

/// Scans `0..=256` once for `<COLLADA`, `<gpx[ \t\r\n]`, `<kml[ \t\r\n>]`, and `<osm[ \t\r\n]`.
///
/// Precondition made explicit (relative to `.yar` text for `gpx`, `kml`, `osm`): a well-formed
/// XML document prelude (UTF-8 BOM, `<?xml` declaration, comments, processing instructions, or
/// XML whitespace) always begins with `<`, the UTF-8 BOM lead `0xEF`, or XML whitespace (`' '`,
/// `'\t'`, `'\r'`, `'\n'`), so non-XML inputs are skipped immediately without sweeping 257 bytes.
#[inline]
pub(crate) fn xml_roots(prefix: &[u8]) -> XmlRoots {
    const MAX_ROOT_OFFSET: usize = 256;
    const XML_DECL: &[u8] = b"<?xml";
    const COLLADA_TAG: &[u8] = b"<COLLADA";
    const GPX_TAG: &[u8] = b"<gpx";
    const KML_TAG: &[u8] = b"<kml";
    const OSM_TAG: &[u8] = b"<osm";

    let Some(&first) = prefix.first() else {
        return XmlRoots::default();
    };
    if !matches!(first, b'<' | 0xEF | b' ' | b'\t' | b'\r' | b'\n') {
        return XmlRoots::default();
    }
    let has_xml_decl = starts(prefix, XML_DECL);
    let mut roots = XmlRoots::default();
    let window = &prefix[..prefix.len().min(MAX_ROOT_OFFSET + 1)];
    for pos in memchr::memchr_iter(b'<', window) {
        match prefix.get(pos + 1) {
            Some(b'C') if has_xml_decl && eq_at(prefix, pos, COLLADA_TAG) => {
                roots.collada = true;
            }
            Some(b'g')
                if eq_at(prefix, pos, GPX_TAG)
                    && matches!(u8_at(prefix, pos + 4), Some(b' ' | b'\t' | b'\r' | b'\n')) =>
            {
                roots.gpx = true;
            }
            Some(b'k')
                if eq_at(prefix, pos, KML_TAG)
                    && matches!(
                        u8_at(prefix, pos + 4),
                        Some(b' ' | b'\t' | b'\r' | b'\n' | b'>')
                    ) =>
            {
                roots.kml = true;
            }
            Some(b'o')
                if eq_at(prefix, pos, OSM_TAG)
                    && matches!(u8_at(prefix, pos + 4), Some(b' ' | b'\t' | b'\r' | b'\n')) =>
            {
                roots.osm = true;
            }
            _ => {}
        }
    }
    roots
}

const PACKAGE_TAG: &[u8] = b"<package";

/// XML / HTML family disambiguator (`svg`, `appleplist`, `rdf`, `mum`, `html`, `xml`).
pub(crate) fn xml_or_html(prefix: &[u8], size: u64) -> Option<Label> {
    const PLIST_DOCTYPE: &[u8] = b"<!DOCTYPE plist";
    const PLIST_OPEN_ATTR: &[u8] = b"<plist ";
    const PLIST_OPEN_TAG: &[u8] = b"<plist>";
    const SVG_OPEN_TAG: &[u8] = b"<svg";
    const SVG_DOCTYPE: &[u8] = b"<!DOCTYPE svg";
    const RDF_QUALIFIED_TAG: &[u8] = b"<rdf:RDF";
    const RDF_BARE_TAG: &[u8] = b"<RDF";
    const RDF_XMLNS: &[u8] = b"xmlns:rdf";
    const MUM_ASSEMBLY_TAG: &[u8] = b"<assembly";
    const MUM_ASM_NS: &[u8] = b"urn:schemas-microsoft-com:asm";
    const HTML_DOCTYPE_UPPER: &[u8] = b"<!DOCTYPE html";
    const HTML_DOCTYPE_LOWER: &[u8] = b"<!doctype html";
    const HTML_TAG_LOWER: &[u8] = b"<html";
    const HTML_TAG_UPPER: &[u8] = b"<HTML";
    const HEAD_TAG_LOWER: &[u8] = b"<head";
    const HEAD_TAG_UPPER: &[u8] = b"<HEAD";
    const HTML_OPEN_ATTR: &[u8] = b"<html ";
    const HTML_OPEN_TAG: &[u8] = b"<html>";
    const STATA_DTA_TAG: &[u8] = b"<stata_dta>";
    const XML_DECL_SPACE: &[u8] = b"<?xml ";
    const XML_DECL_TAB: &[u8] = b"<?xml\t";
    const XML_DECL_CR: &[u8] = b"<?xml\r";
    const XML_DECL_LF: &[u8] = b"<?xml\n";
    const XMLNS_ATTR: &[u8] = b"xmlns";
    const XML_CLOSE_TAG: &[u8] = b"</";
    const XML_SELF_CLOSE_TAG: &[u8] = b"/>";

    require!(
        size >= 8,
        prefix.len() >= 8,
        !prefix[..prefix.len().min(256)].contains(&0),
    );
    let mut i = 0usize;
    if starts(prefix, UTF8_BOM) {
        i = 3;
    }
    while i < prefix.len().min(64) && matches!(u8_at(prefix, i), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        i += 1;
    }
    require!(u8_at(prefix, i) == Some(b'<'));

    // 1. Apple XML Property List (`appleplist`)
    if contains(prefix, PLIST_DOCTYPE)
        || ((contains(prefix, PLIST_OPEN_ATTR) || contains(prefix, PLIST_OPEN_TAG))
            && (contains(prefix, XML_CLOSE_TAG) || contains(prefix, XML_SELF_CLOSE_TAG)))
    {
        return Some(Label::Appleplist);
    }
    // 2. W3C Scalable Vector Graphics (`svg`)
    if contains(prefix, SVG_OPEN_TAG) || contains(prefix, SVG_DOCTYPE) {
        return Some(Label::Svg);
    }
    // 3. W3C Resource Description Framework (`rdf`)
    if contains(prefix, RDF_QUALIFIED_TAG) || (contains(prefix, RDF_BARE_TAG) && contains(prefix, RDF_XMLNS)) {
        return Some(Label::Rdf);
    }
    // 4. Windows Update Package Manifest (`mum`)
    if contains(prefix, MUM_ASSEMBLY_TAG) && contains(prefix, PACKAGE_TAG) && contains(prefix, MUM_ASM_NS) {
        return Some(Label::Mum);
    }
    // 5. HTML / XHTML (`html`)
    if eq_at(prefix, i, HTML_DOCTYPE_UPPER)
        || eq_at(prefix, i, HTML_DOCTYPE_LOWER)
        || eq_at(prefix, i, HTML_TAG_LOWER)
        || eq_at(prefix, i, HTML_TAG_UPPER)
        || eq_at(prefix, i, HEAD_TAG_LOWER)
        || eq_at(prefix, i, HEAD_TAG_UPPER)
        || contains(prefix, HTML_DOCTYPE_UPPER)
        || contains(prefix, HTML_OPEN_ATTR)
        || contains(prefix, HTML_OPEN_TAG)
    {
        return Some(Label::Html);
    }
    // 6. Generic XML (`xml`)
    if !eq_at(prefix, i, STATA_DTA_TAG)
        && (eq_at(prefix, i, XML_DECL_SPACE)
            || eq_at(prefix, i, XML_DECL_TAB)
            || eq_at(prefix, i, XML_DECL_CR)
            || eq_at(prefix, i, XML_DECL_LF)
            || (u8_at(prefix, i + 1).is_some_and(|b| b.is_ascii_alphabetic())
                && (contains(prefix, XMLNS_ATTR) || contains(prefix, XML_CLOSE_TAG) || contains(prefix, XML_SELF_CLOSE_TAG))))
    {
        return Some(Label::Xml);
    }
    None
}

/// XML-based specialized formats (`gml`, `hta`, `jnlp`, `openvino_ir`, `scriptwsf`, `svd`, `xsd`).
pub(crate) fn xml_subtypes(prefix: &[u8], size: u64) -> Option<Label> {
    const GML_XMLNS: &[u8] = b"xmlns:gml=";
    const GML_TAG_PREFIX: &[u8] = b"<gml:";
    const HTA_APP_LOWER: &[u8] = b"<hta:application";
    const HTA_APP_UPPER: &[u8] = b"<HTA:APPLICATION";
    const JNLP_OPEN_ATTR: &[u8] = b"<jnlp ";
    const JNLP_OPEN_TAG: &[u8] = b"<jnlp>";
    const OPENVINO_NET_TAG: &[u8] = b"<net ";
    const OPENVINO_LAYERS_TAG: &[u8] = b"<layers>";
    const OPENVINO_LAYER_TAG: &[u8] = b"<layer ";
    const WSF_JOB_TAG: &[u8] = b"<job";
    const WSF_SCRIPT_TAG: &[u8] = b"<script";
    const SVD_DEVICE_TAG: &[u8] = b"<device";
    const SVD_PERIPHERALS_TAG: &[u8] = b"<peripherals>";
    const SVD_PERIPHERAL_TAG: &[u8] = b"<peripheral>";
    const XSD_XS_SCHEMA: &[u8] = b"<xs:schema";
    const XSD_XSD_SCHEMA: &[u8] = b"<xsd:schema";
    const XSD_BARE_SCHEMA: &[u8] = b"<schema";
    const XSD_NAMESPACE_URI: &[u8] = b"http://www.w3.org/2001/XMLSchema";

    require!(
        size >= 16,
        prefix.len() >= 16,
        !prefix[..prefix.len().min(256)].contains(&0),
    );
    let mut i = 0usize;
    if starts(prefix, UTF8_BOM) {
        i = 3;
    }
    while i < prefix.len().min(64) && matches!(prefix.get(i), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        i += 1;
    }
    require!(prefix.get(i) == Some(&b'<'));

    // 1. Geography Markup Language (`gml`)
    if contains(prefix, GML_XMLNS) || contains(prefix, GML_TAG_PREFIX) {
        let _ = get_slice(prefix, 0, i + 4);
        return Some(Label::Gml);
    }
    // 2. Microsoft HTML Application (`hta`)
    if contains(prefix, HTA_APP_LOWER) || contains(prefix, HTA_APP_UPPER) {
        let _ = get_slice(prefix, 0, i + 4);
        return Some(Label::Hta);
    }
    // 3. Java Network Launch Protocol (`jnlp`)
    if contains(prefix, JNLP_OPEN_ATTR) || contains(prefix, JNLP_OPEN_TAG) {
        let _ = get_slice(prefix, 0, i + 4);
        return Some(Label::Jnlp);
    }
    // 4. OpenVINO Intermediate Representation (`openvino_ir`)
    if contains(prefix, OPENVINO_NET_TAG)
        && contains(prefix, OPENVINO_LAYERS_TAG)
        && contains(prefix, OPENVINO_LAYER_TAG)
    {
        let _ = get_slice(prefix, 0, i + 4);
        return Some(Label::OpenvinoIr);
    }
    // 5. Windows Script File (`scriptwsf`)
    if (contains(prefix, WSF_JOB_TAG) || contains(prefix, PACKAGE_TAG))
        && contains(prefix, WSF_SCRIPT_TAG)
    {
        let _ = get_slice(prefix, 0, i + 4);
        return Some(Label::Scriptwsf);
    }
    // 6. ARM CMSIS System View Description (`svd`)
    if contains(prefix, SVD_DEVICE_TAG)
        && contains(prefix, SVD_PERIPHERALS_TAG)
        && contains(prefix, SVD_PERIPHERAL_TAG)
    {
        let _ = get_slice(prefix, 0, i + 4);
        return Some(Label::Svd);
    }
    // 7. W3C XML Schema Definition (`xsd`)
    if contains(prefix, XSD_XS_SCHEMA)
        || contains(prefix, XSD_XSD_SCHEMA)
        || (contains(prefix, XSD_BARE_SCHEMA) && contains(prefix, XSD_NAMESPACE_URI))
    {
        let _ = get_slice(prefix, 0, i + 4);
        return Some(Label::Xsd);
    }
    None
}

/// Stata `.dta` XML dataset: `"<stata_dta><header><release>"` + 3 ASCII digits +
/// `"</release><byteorder>"` + `"LSF</byteorder>"` or `"MSF</byteorder>"`.
pub(crate) fn stata(prefix: &[u8]) -> Option<Label> {
    const PREFIX: &[u8] = b"<stata_dta><header><release>"; // @0..28: opening tags
    const RELEASE_DIGITS: usize = 28; // @28..31: 3-digit release code
    const MID: &[u8] = b"</release><byteorder>"; // @31..52: closing release + opening byteorder
    const BYTEORDER: usize = 52; // @52..67: "LSF</byteorder>" or "MSF</byteorder>"
    const LSF_TAG: &[u8] = b"LSF</byteorder>";
    const MSF_TAG: &[u8] = b"MSF</byteorder>";
    const MIN_LEN: usize = 67;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, PREFIX),
        get_slice(prefix, RELEASE_DIGITS, 31)?.iter().all(u8::is_ascii_digit),
        eq_at(prefix, 31, MID),
        eq_at(prefix, BYTEORDER, LSF_TAG) || eq_at(prefix, BYTEORDER, MSF_TAG),
    );
    Some(Label::Stata)
}

/// Fast-path Tier 1 (`magic32`) XML / HTML document and subformat dispatcher.
pub(crate) fn xml_document(prefix: &[u8], size: u64) -> Option<Label> {
    let xml = xml_roots(prefix);
    if xml.collada {
        return Some(Label::Collada);
    }
    if xml.gpx {
        return Some(Label::Gpx);
    }
    if xml.kml {
        return Some(Label::Kml);
    }
    if xml.osm {
        return Some(Label::Osm);
    }
    probe(|| xml_subtypes(prefix, size)).or_else(|| probe(|| xml_or_html(prefix, size)))
}
