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

//! Tier 5 (`heuristics`): structural and text heuristics without a fixed offset-0 magic key.

use super::*;
use crate::ContentType as Label;

/// Last-resort heuristics evaluated only if no earlier magic or structural format matched.
#[inline]
fn heuristic_last(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    super::probe(|| pdf_or_ai_scan_1024(prefix, ctx.size))
}

/// Distinct Tier 5 heuristics that can be prioritized by file extension and skipped once tried.
#[derive(Clone, Copy)]
#[repr(u16)]
enum Heuristic {
    Xml,
    Json,
    Mpegts,
    Safetensors,
    Dbase,
    Intelhex,
    Pythonbytecode,
    StlBinary,
    Tga,
    Mht,
    Eml,
    Batch,
    Latex,
    Srt,
    M3u,
    Vtt,
    AutoitScript,
    Aout,
    Bson,
    Dosmbr,
    ImgFat12,
    Segy,
    Arcgrid,
    BplSource,
    LlvmIr,
    Mapinfo,
    Mtl,
    Obj,
    PcdAscii,
    Wkt,
    Rdp,
}

impl Heuristic {
    const COUNT: usize = Self::Rdp as usize + 1;
}

/// Fixed-size stack bitset tracking heuristics already executed during a scan.
#[derive(Default)]
struct Tried([u64; Heuristic::COUNT.div_ceil(64)]);

impl Tried {
    #[inline]
    fn insert(&mut self, h: Heuristic) {
        let idx = h as usize;
        self.0[idx / 64] |= 1u64 << (idx % 64);
    }

    #[inline]
    fn contains(&self, h: Heuristic) -> bool {
        let idx = h as usize;
        (self.0[idx / 64] & (1u64 << (idx % 64))) != 0
    }
}

/// Returns the ordered list of Tier 5 heuristics to try first for a lowercased file extension.
#[inline]
fn hints_for_ext(ext: &str) -> &'static [Heuristic] {
    use Heuristic::*;
    match ext {
        "dae" | "gpx" | "kml" | "osm" | "gml" | "hta" | "jnlp" | "wsf" | "scriptwsf" | "svd"
        | "xsd" | "plist" | "svg" | "rdf" | "mum" | "html" | "htm" | "xhtml" | "xht" | "xml" => {
            &[Xml]
        }
        "gltf" | "cdb" | "geojson" | "ipynb" | "json" => &[Json],
        "ts" | "m2ts" | "mts" | "m2t" | "tsa" | "tsv" => &[Mpegts],
        "safetensors" => &[Safetensors],
        "dbf" => &[Dbase],
        "hex" | "ihex" | "ihx" => &[Intelhex],
        "pyc" | "pyo" => &[Pythonbytecode],
        "stl" | "stlb" => &[StlBinary],
        "tga" | "icb" | "vda" | "vst" => &[Tga],
        "mht" | "mhtml" | "eml" => &[Mht, Eml],
        "bat" | "cmd" => &[Batch],
        "tex" | "ltx" | "sty" | "cls" | "bib" => &[Latex],
        "srt" => &[Srt],
        "m3u" | "m3u8" => &[M3u],
        "vtt" | "webvtt" => &[Vtt],
        "au3" => &[AutoitScript],
        "out" | "aout" => &[Aout],
        "bson" => &[Bson],
        "mbr" | "dosmbr" | "img" | "ima" | "flp" | "dsk" => &[Dosmbr, ImgFat12],
        "segy" | "sgy" => &[Segy],
        "asc" | "grd" | "agr" => &[Arcgrid],
        "bpl" => &[BplSource],
        "ll" => &[LlvmIr],
        "tab" | "mif" | "mid" | "map" => &[Mapinfo],
        "mtl" => &[Mtl],
        "obj" => &[Obj],
        "pcd" => &[PcdAscii],
        "wkt" | "prj" => &[Wkt],
        "rdp" => &[Rdp],
        _ => &[],
    }
}

/// Returns the first non-ASCII-whitespace byte in `prefix` (skipping an optional UTF-8 BOM,
/// scanning up to 64 bytes).
#[inline]
fn first_non_ws(prefix: &[u8]) -> Option<u8> {
    let mut ws_idx = if prefix.starts_with(UTF8_BOM) { 3 } else { 0 };
    while ws_idx < prefix.len().min(64) && matches!(prefix[ws_idx], b' ' | b'\t' | b'\r' | b'\n') {
        ws_idx += 1;
    }
    prefix.get(ws_idx).copied()
}

/// Returns `true` if the first 256 bytes of `prefix` contain no NUL byte.
#[inline]
fn no_nul_256(prefix: &[u8]) -> bool {
    memchr::memchr(0, &prefix[..prefix.len().min(256)]).is_none()
}

/// Runs a single Tier 5 heuristic with its exact prefix pre-guard.
#[inline]
fn run_heuristic(h: Heuristic, prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    let size = ctx.size;
    match h {
        Heuristic::Xml => {
            require!(matches!(prefix.first(), Some(b'<' | 0xEF | b' ' | b'\t' | b'\r' | b'\n')));
            super::probe(|| xml_document(prefix, size))
        }
        Heuristic::Json => {
            require!(matches!(first_non_ws(prefix), Some(b'{' | b'[')));
            super::probe(|| gltf_json(prefix))
                .or_else(|| super::probe(|| json_subtypes(prefix, size)))
                .or_else(|| super::probe(|| ipynb_or_json(prefix, size)))
        }
        Heuristic::Mpegts => super::probe(|| mpegts(prefix)),
        Heuristic::Safetensors => super::probe(|| safetensors(prefix)),
        Heuristic::Dbase => super::probe(|| dbase(prefix, ctx)),
        Heuristic::Intelhex => super::probe(|| intelhex(prefix)),
        Heuristic::Pythonbytecode => super::probe(|| pythonbytecode(prefix)),
        Heuristic::StlBinary => super::probe(|| stl_binary(prefix, size)),
        Heuristic::Tga => super::probe(|| tga(prefix, size, ctx.tail)),
        Heuristic::Mht => {
            require!(no_nul_256(prefix));
            super::probe(|| mht(prefix, size))
        }
        Heuristic::Eml => {
            require!(no_nul_256(prefix));
            super::probe(|| eml(prefix, size))
        }
        Heuristic::Batch => {
            require!(no_nul_256(prefix));
            super::probe(|| batch(prefix, size))
        }
        Heuristic::Latex => {
            require!(no_nul_256(prefix));
            super::probe(|| latex(prefix, size))
        }
        Heuristic::Srt => {
            require!(no_nul_256(prefix));
            super::probe(|| srt(prefix, size))
        }
        Heuristic::M3u => {
            require!(no_nul_256(prefix));
            super::probe(|| m3u(prefix, size))
        }
        Heuristic::Vtt => {
            require!(no_nul_256(prefix));
            super::probe(|| vtt(prefix, size))
        }
        Heuristic::AutoitScript => {
            require!(no_nul_256(prefix));
            super::probe(|| autoit_script(prefix, size))
        }
        Heuristic::Aout => super::probe(|| aout(prefix, size)),
        Heuristic::Bson => super::probe(|| bson(prefix, size)),
        Heuristic::Dosmbr => super::probe(|| dosmbr(prefix, size)),
        Heuristic::ImgFat12 => super::probe(|| img_fat12(prefix, size)),
        Heuristic::Segy => super::probe(|| segy(prefix, size)),
        Heuristic::Arcgrid => {
            require!(no_nul_256(prefix));
            super::probe(|| arcgrid(prefix, size))
        }
        Heuristic::BplSource => {
            require!(no_nul_256(prefix));
            super::probe(|| bpl_source(prefix, size))
        }
        Heuristic::LlvmIr => {
            require!(no_nul_256(prefix));
            super::probe(|| llvm_ir(prefix, size))
        }
        Heuristic::Mapinfo => {
            require!(no_nul_256(prefix));
            super::probe(|| mapinfo(prefix, size))
        }
        Heuristic::Mtl => {
            require!(no_nul_256(prefix));
            super::probe(|| mtl(prefix, size))
        }
        Heuristic::Obj => {
            require!(no_nul_256(prefix));
            super::probe(|| obj(prefix, size))
        }
        Heuristic::PcdAscii => {
            require!(no_nul_256(prefix));
            super::probe(|| pcd_ascii(prefix, size))
        }
        Heuristic::Wkt => {
            require!(no_nul_256(prefix));
            super::probe(|| wkt(prefix, size))
        }
        Heuristic::Rdp => super::probe(|| rdp(prefix, size)),
    }
}

/// Evaluates the heuristic(s) matching `ctx.path`'s file extension first, recording each one in
/// `tried` so the fallback pass does not execute them twice.
#[inline]
fn probe_by_path(prefix: &[u8], ctx: &Ctx<'_>, tried: &mut Tried) -> Option<Label> {
    let ext_os = ctx.path?.extension()?.to_str()?;
    let bytes = ext_os.as_bytes();
    require!(!bytes.is_empty(), bytes.len() <= 16);
    let mut buf = [0u8; 16];
    for (dst, &src) in buf.iter_mut().zip(bytes) {
        *dst = src.to_ascii_lowercase();
    }
    let ext = core::str::from_utf8(&buf[..bytes.len()]).ok()?;
    for &h in hints_for_ext(ext) {
        if tried.contains(h) {
            continue;
        }
        tried.insert(h);
        if let Some(label) = run_heuristic(h, prefix, ctx) {
            return Some(label);
        }
    }
    None
}

/// Evaluates `f` only if `h` has not already been executed in this scan.
#[inline]
fn try_unseen(tried: &Tried, h: Heuristic, f: impl FnOnce() -> Option<Label>) -> Option<Label> {
    if tried.contains(h) { None } else { super::probe(f) }
}

/// Tier 5 (`heuristics`): evaluates structural and text heuristics in deterministic priority order.
#[inline]
pub(crate) fn probe(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    let mut tried = Tried::default();
    if let Some(label) = probe_by_path(prefix, ctx, &mut tried) {
        return Some(label);
    }
    let size = ctx.size;

    let first_non_ws = first_non_ws(prefix);

    if !tried.contains(Heuristic::Xml)
        && matches!(prefix.first(), Some(b'<' | 0xEF | b' ' | b'\t' | b'\r' | b'\n'))
    {
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
            return with_subformat(Label::Osm, &OSM_XML);
        }
        if first_non_ws == Some(b'<')
            && let Some(label) = super::probe(|| xml_subtypes(prefix, size))
                .or_else(|| super::probe(|| xml_or_html(prefix, size)))
        {
            return Some(label);
        }
    }
    if !tried.contains(Heuristic::Json)
        && matches!(first_non_ws, Some(b'{' | b'['))
        && let Some(label) = super::probe(|| gltf_json(prefix))
            .or_else(|| super::probe(|| json_subtypes(prefix, size)))
            .or_else(|| super::probe(|| ipynb_or_json(prefix, size)))
    {
        return Some(label);
    }
    if let Some(label) = try_unseen(&tried, Heuristic::Mpegts, || mpegts(prefix))
        .or_else(|| try_unseen(&tried, Heuristic::Safetensors, || safetensors(prefix)))
        .or_else(|| try_unseen(&tried, Heuristic::Dbase, || dbase(prefix, ctx)))
        .or_else(|| try_unseen(&tried, Heuristic::Intelhex, || intelhex(prefix)))
        .or_else(|| try_unseen(&tried, Heuristic::Pythonbytecode, || pythonbytecode(prefix)))
        .or_else(|| try_unseen(&tried, Heuristic::StlBinary, || stl_binary(prefix, size)))
        .or_else(|| try_unseen(&tried, Heuristic::Tga, || tga(prefix, size, ctx.tail)))
    {
        return Some(label);
    }
    let no_nul = no_nul_256(prefix);
    if no_nul
        && let Some(label) = try_unseen(&tried, Heuristic::Mht, || mht(prefix, size))
            .or_else(|| try_unseen(&tried, Heuristic::Eml, || eml(prefix, size)))
            .or_else(|| try_unseen(&tried, Heuristic::Batch, || batch(prefix, size)))
            .or_else(|| try_unseen(&tried, Heuristic::Latex, || latex(prefix, size)))
            .or_else(|| try_unseen(&tried, Heuristic::Srt, || srt(prefix, size)))
            .or_else(|| try_unseen(&tried, Heuristic::M3u, || m3u(prefix, size)))
            .or_else(|| try_unseen(&tried, Heuristic::Vtt, || vtt(prefix, size)))
            .or_else(|| try_unseen(&tried, Heuristic::AutoitScript, || autoit_script(prefix, size)))
    {
        return Some(label);
    }
    if let Some(label) = try_unseen(&tried, Heuristic::Aout, || aout(prefix, size))
        .or_else(|| try_unseen(&tried, Heuristic::Bson, || bson(prefix, size)))
        .or_else(|| try_unseen(&tried, Heuristic::Dosmbr, || dosmbr(prefix, size)))
        .or_else(|| try_unseen(&tried, Heuristic::ImgFat12, || img_fat12(prefix, size)))
        .or_else(|| try_unseen(&tried, Heuristic::Segy, || segy(prefix, size)))
    {
        return Some(label);
    }
    if no_nul
        && let Some(label) = try_unseen(&tried, Heuristic::Arcgrid, || arcgrid(prefix, size))
            .or_else(|| try_unseen(&tried, Heuristic::BplSource, || bpl_source(prefix, size)))
            .or_else(|| try_unseen(&tried, Heuristic::LlvmIr, || llvm_ir(prefix, size)))
            .or_else(|| try_unseen(&tried, Heuristic::Mapinfo, || mapinfo(prefix, size)))
            .or_else(|| try_unseen(&tried, Heuristic::Mtl, || mtl(prefix, size)))
            .or_else(|| try_unseen(&tried, Heuristic::Obj, || obj(prefix, size)))
            .or_else(|| try_unseen(&tried, Heuristic::PcdAscii, || pcd_ascii(prefix, size)))
            .or_else(|| try_unseen(&tried, Heuristic::Wkt, || wkt(prefix, size)))
    {
        return Some(label);
    }
    try_unseen(&tried, Heuristic::Rdp, || rdp(prefix, size)).or_else(|| heuristic_last(prefix, ctx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tried_bitset_tracks_and_ext_hints_are_valid() {
        let mut tried = Tried::default();
        assert!(!tried.contains(Heuristic::Xml));
        assert!(!tried.contains(Heuristic::Rdp));
        tried.insert(Heuristic::Xml);
        tried.insert(Heuristic::Rdp);
        assert!(tried.contains(Heuristic::Xml));
        assert!(tried.contains(Heuristic::Rdp));
        assert!(!tried.contains(Heuristic::Json));

        assert!(hints_for_ext("img").len() >= 2);
        assert!(hints_for_ext("eml").len() >= 2);
        assert!(hints_for_ext("unknown_ext").is_empty());
    }
}
