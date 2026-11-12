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

/// Tier 5 (`heuristics`): evaluates structural and text heuristics in deterministic priority order.
#[inline]
pub(crate) fn probe(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    let size = ctx.size;

    let mut ws_idx = if prefix.starts_with(UTF8_BOM) { 3 } else { 0 };
    while ws_idx < prefix.len().min(64) && matches!(prefix[ws_idx], b' ' | b'\t' | b'\r' | b'\n') {
        ws_idx += 1;
    }
    let first_non_ws = prefix.get(ws_idx).copied();

    if matches!(prefix.first(), Some(b'<' | 0xEF | b' ' | b'\t' | b'\r' | b'\n')) {
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
    if matches!(first_non_ws, Some(b'{' | b'['))
        && let Some(label) = super::probe(|| gltf_json(prefix))
            .or_else(|| super::probe(|| json_subtypes(prefix, size)))
            .or_else(|| super::probe(|| ipynb_or_json(prefix, size)))
    {
        return Some(label);
    }
    if let Some(label) = super::probe(|| mpegts(prefix))
        .or_else(|| super::probe(|| safetensors(prefix)))
        .or_else(|| super::probe(|| dbase(prefix, ctx)))
        .or_else(|| super::probe(|| intelhex(prefix)))
        .or_else(|| super::probe(|| pythonbytecode(prefix)))
        .or_else(|| super::probe(|| stl_binary(prefix, size)))
        .or_else(|| super::probe(|| tga(prefix, size, ctx.tail)))
    {
        return Some(label);
    }
    let no_nul_256 = memchr::memchr(0, &prefix[..prefix.len().min(256)]).is_none();
    if no_nul_256
        && let Some(label) = super::probe(|| mht(prefix, size))
            .or_else(|| super::probe(|| eml(prefix, size)))
            .or_else(|| super::probe(|| batch(prefix, size)))
            .or_else(|| super::probe(|| latex(prefix, size)))
            .or_else(|| super::probe(|| srt(prefix, size)))
            .or_else(|| super::probe(|| m3u(prefix, size)))
            .or_else(|| super::probe(|| vtt(prefix, size)))
            .or_else(|| super::probe(|| autoit_script(prefix, size)))
    {
        return Some(label);
    }
    if let Some(label) = super::probe(|| aout(prefix, size))
        .or_else(|| super::probe(|| bson(prefix, size)))
        .or_else(|| super::probe(|| dosmbr(prefix, size)))
        .or_else(|| super::probe(|| img_fat12(prefix, size)))
        .or_else(|| super::probe(|| segy(prefix, size)))
    {
        return Some(label);
    }
    if no_nul_256
        && let Some(label) = super::probe(|| arcgrid(prefix, size))
            .or_else(|| super::probe(|| bpl_source(prefix, size)))
            .or_else(|| super::probe(|| llvm_ir(prefix, size)))
            .or_else(|| super::probe(|| mapinfo(prefix, size)))
            .or_else(|| super::probe(|| mtl(prefix, size)))
            .or_else(|| super::probe(|| obj(prefix, size)))
            .or_else(|| super::probe(|| pcd_ascii(prefix, size)))
            .or_else(|| super::probe(|| wkt(prefix, size)))
    {
        return Some(label);
    }
    super::probe(|| rdp(prefix, size)).or_else(|| heuristic_last(prefix, ctx))
}
