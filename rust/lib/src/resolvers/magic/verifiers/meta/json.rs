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

//! JSON document and schema metaformat verifiers (`json`, `castledb`, `geojson`, `gltf`, `ipynb`, `tensorflowjs`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// JSON / Jupyter Notebook disambiguator (`ipynb` vs `json`).
pub(crate) fn ipynb_or_json(prefix: &[u8], size: u64) -> Option<Label> {
    const IPYNB_NBFORMAT_KEY: &[u8] = b"\"nbformat\"";
    const IPYNB_CELLS_KEY: &[u8] = b"\"cells\"";
    const IPYNB_WORKSHEETS_KEY: &[u8] = b"\"worksheets\"";
    const IPYNB_METADATA_KEY: &[u8] = b"\"metadata\"";

    require!(size >= 8, prefix.len() >= 8);
    let mut i = 0usize;
    if starts(prefix, UTF8_BOM) {
        i = 3;
    }
    while i < prefix.len().min(64) && matches!(u8_at(prefix, i), Some(b' ' | b'\t' | b'\r' | b'\n'))
    {
        i += 1;
    }
    let open = u8_at(prefix, i)?;
    require!(open == b'{' || open == b'[');
    if open == b'{'
        && contains(prefix, IPYNB_NBFORMAT_KEY)
        && (contains(prefix, IPYNB_CELLS_KEY) || contains(prefix, IPYNB_WORKSHEETS_KEY))
        && contains(prefix, IPYNB_METADATA_KEY)
    {
        return Some(Label::Ipynb);
    }
    // Validate JSON structure: '{' followed by whitespace and '"' or '}', or '[' followed by valid value
    let mut j = i + 1;
    while j < prefix.len().min(i + 64)
        && matches!(u8_at(prefix, j), Some(b' ' | b'\t' | b'\r' | b'\n'))
    {
        j += 1;
    }
    let next = u8_at(prefix, j)?;
    if open == b'{' {
        require!((next == b'"' && contains(prefix, b":")) || (next == b'}' && size <= 64));
        return Some(Label::Json);
    }
    if open == b'[' {
        require!(
            matches!(next, b'{' | b'"' | b'[' | b']')
                && (contains(prefix, b":") || contains(prefix, b",") || next == b']')
        );
        return Some(Label::Json);
    }
    None
}

/// JSON-based specialized formats (`castledb`, `geojson`, `tensorflowjs`).
pub(crate) fn json_subtypes(prefix: &[u8], size: u64) -> Option<Label> {
    const CASTLEDB_SHEETS_KEY: &[u8] = b"\"sheets\"";
    const CASTLEDB_COLUMNS_KEY: &[u8] = b"\"columns\"";
    const CASTLEDB_LINES_KEY: &[u8] = b"\"lines\"";
    const GEOJSON_FEATURE_COLLECTION: &[u8] = b"\"FeatureCollection\"";
    const GEOJSON_GEOMETRY_COLLECTION: &[u8] = b"\"GeometryCollection\"";
    const GEOJSON_FEATURE: &[u8] = b"\"Feature\"";
    const GEOJSON_GEOMETRY_KEY: &[u8] = b"\"geometry\"";
    const GEOJSON_FEATURES_KEY: &[u8] = b"\"features\"";
    const GEOJSON_COORDINATES_KEY: &[u8] = b"\"coordinates\"";
    const TFJS_MODEL_TOPOLOGY_KEY: &[u8] = b"\"modelTopology\"";
    const TFJS_WEIGHTS_MANIFEST_KEY: &[u8] = b"\"weightsManifest\"";

    require!(size >= 24, prefix.len() >= 24);
    let mut i = 0usize;
    if starts(prefix, UTF8_BOM) {
        i = 3;
    }
    while i < prefix.len().min(64) && matches!(prefix.get(i), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        i += 1;
    }
    require!(prefix.get(i) == Some(&b'{'));

    // 1. CastleDB (`castledb`)
    if contains(prefix, CASTLEDB_SHEETS_KEY)
        && contains(prefix, CASTLEDB_COLUMNS_KEY)
        && contains(prefix, CASTLEDB_LINES_KEY)
    {
        let _ = get_slice(prefix, 0, i + 1);
        return Some(Label::Castledb);
    }
    // 2. GeoJSON (`geojson`)
    if (contains(prefix, GEOJSON_FEATURE_COLLECTION)
        || contains(prefix, GEOJSON_GEOMETRY_COLLECTION)
        || (contains(prefix, GEOJSON_FEATURE) && contains(prefix, GEOJSON_GEOMETRY_KEY)))
        && (contains(prefix, GEOJSON_FEATURES_KEY) || contains(prefix, GEOJSON_COORDINATES_KEY))
    {
        let _ = get_slice(prefix, 0, i + 1);
        return Some(Label::Geojson);
    }
    // 3. TensorFlow.js Model JSON (`tensorflowjs`)
    if contains(prefix, TFJS_MODEL_TOPOLOGY_KEY) && contains(prefix, TFJS_WEIGHTS_MANIFEST_KEY) {
        let _ = get_slice(prefix, 0, i + 1);
        return Some(Label::Tensorflowjs);
    }
    None
}

/// Khronos glTF 2.0 JSON asset: anchored at `@0`, either opens with `"accessors": [` and
/// `"componentType"` in `0..=1024`, or opens with a flat `"asset": { ... }` object followed by a
/// glTF top-level property or a `KHR_`/`EXT_` `"extensionsUsed"` list.
pub(crate) fn gltf_json(prefix: &[u8]) -> Option<Label> {
    const KEY_ACCESSORS: &[u8] = b"\"accessors\"";
    const KEY_COMPONENT_TYPE: &[u8] = b"\"componentType\"";
    const KEY_ASSET: &[u8] = b"\"asset\"";
    const KEY_EXTENSIONS_USED: &[u8] = b"\"extensionsUsed\"";
    const EXT_KHR: &[u8] = b"\"KHR_";
    const EXT_EXT: &[u8] = b"\"EXT_";
    const GLTF_KEYS: &[&[u8]] = &[
        b"\"scene\"",
        b"\"scenes\"",
        b"\"nodes\"",
        KEY_ACCESSORS,
        b"\"bufferViews\"",
        b"\"buffers\"",
        b"\"meshes\"",
    ];

    #[inline]
    fn is_ws(b: u8) -> bool {
        matches!(b, b' ' | b'\t' | b'\r' | b'\n')
    }

    #[inline]
    fn skip_ws(s: &[u8], i: &mut usize, max: usize) -> bool {
        let start = *i;
        let limit = s.len().min(start + max + 1);
        while *i < limit && is_ws(s[*i]) {
            *i += 1;
        }
        *i - start <= max
    }

    let first = *prefix.first()?;
    require!(is_ws(first) || first == b'{');

    let mut i = 0;
    require!(skip_ws(prefix, &mut i, 64), u8_at(prefix, i) == Some(b'{'),);
    i += 1;
    require!(skip_ws(prefix, &mut i, 64));

    if eq_at(prefix, i, KEY_ACCESSORS) {
        i += KEY_ACCESSORS.len();
        require!(skip_ws(prefix, &mut i, 16), u8_at(prefix, i) == Some(b':'),);
        i += 1;
        require!(
            skip_ws(prefix, &mut i, 16),
            u8_at(prefix, i) == Some(b'['),
            scan_range(prefix, 0, 1024, KEY_COMPONENT_TYPE),
        );
        return Some(Label::Gltf);
    }

    require!(eq_at(prefix, i, KEY_ASSET));
    i += KEY_ASSET.len();
    require!(skip_ws(prefix, &mut i, 16), u8_at(prefix, i) == Some(b':'),);
    i += 1;
    require!(skip_ws(prefix, &mut i, 16), u8_at(prefix, i) == Some(b'{'),);
    i += 1;
    let body_start = i;
    while i < prefix.len() && prefix[i] != b'{' && prefix[i] != b'}' {
        i += 1;
    }
    require!(i - body_start <= 512, u8_at(prefix, i) == Some(b'}'),);
    i += 1;
    require!(skip_ws(prefix, &mut i, 16), u8_at(prefix, i) == Some(b','),);
    i += 1;
    require!(skip_ws(prefix, &mut i, 16));

    if GLTF_KEYS.iter().any(|k| eq_at(prefix, i, k)) {
        return Some(Label::Gltf);
    }

    require!(eq_at(prefix, i, KEY_EXTENSIONS_USED));
    i += KEY_EXTENSIONS_USED.len();
    require!(skip_ws(prefix, &mut i, 16), u8_at(prefix, i) == Some(b':'),);
    i += 1;
    require!(skip_ws(prefix, &mut i, 16), u8_at(prefix, i) == Some(b'['),);
    i += 1;
    require!(skip_ws(prefix, &mut i, 64), eq_at(prefix, i, EXT_KHR) || eq_at(prefix, i, EXT_EXT),);
    Some(Label::Gltf)
}
