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

//! Electronic Arts Interchange File Format (`IFF` `FORM` / `FOR1` / `FOR4`) metaformat verifiers (`beam`, `cinema4d`, `ilbm`, `lightwave`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

pub(crate) const IFF_FORM_MAGIC: &[u8; 4] = b"FORM";
pub(crate) const IFF_FOR1_MAGIC: &[u8; 4] = b"FOR1";
pub(crate) const IFF_FOR4_MAGIC: &[u8; 4] = b"FOR4";
pub(crate) const IFF_FOR8_MAGIC: &[u8; 4] = b"FOR8";

/// Erlang BEAM module: legacy `"\x0f7BEAM!"` header or EA IFF `"FOR1"` container with `"BEAM"` form.
pub(crate) fn beam(prefix: &[u8]) -> Option<Label> {
    static BEAM_LEGACY: Subformat = Subformat::new("Erlang BEAM (legacy)");
    static BEAM_IFF: Subformat = Subformat::new("Erlang BEAM (IFF FOR1)");
    const LEGACY_MAGIC: &[u8] = b"\x0f7BEAM!"; // @0..7: legacy BEAM header
    const BEAM_FORM: &[u8] = b"BEAM"; // @8..12: BEAM form identifier
    const FORM_OFFSET: usize = 8; // @8: form type offset
    const MIN_LEN: usize = 8;
    const IFF_MIN_LEN: usize = 12;

    require!(prefix.len() >= MIN_LEN);
    if starts(prefix, LEGACY_MAGIC) {
        return with_subformat(Label::Beam, &BEAM_LEGACY);
    }
    require!(
        prefix.len() >= IFF_MIN_LEN,
        starts(prefix, IFF_FOR1_MAGIC),
        u8_at(prefix, 4)? == 0,
        eq_at(prefix, FORM_OFFSET, BEAM_FORM),
    );
    with_subformat(Label::Beam, &BEAM_IFF)
}

/// Amiga EA IFF 85 bitmap image: `"FORM"` container magic at `@0` and `"ILBM"`, `"ACBM"`, or
/// `"PBM "` form type at `@8`.
pub(crate) fn ilbm(prefix: &[u8]) -> Option<Label> {
    static IFF_ILBM: Subformat = Subformat::with_ids("IFF ILBM", "fmt/338", None);
    static IFF_ACBM: Subformat = Subformat::new("IFF ACBM");
    static IFF_PBM: Subformat = Subformat::new("IFF PBM");
    const FORM_TYPE: usize = 8; // @8..12: IFF form type
    const FORM_ILBM: &[u8] = b"ILBM";
    const FORM_ACBM: &[u8] = b"ACBM";
    const FORM_PBM: &[u8] = b"PBM ";
    const MIN_LEN: usize = 12;

    require!(prefix.len() >= MIN_LEN, starts(prefix, IFF_FORM_MAGIC), u8_at(prefix, 4)? == 0);
    let sub = if eq_at(prefix, FORM_TYPE, FORM_ILBM) {
        &IFF_ILBM
    } else if eq_at(prefix, FORM_TYPE, FORM_ACBM) {
        &IFF_ACBM
    } else if eq_at(prefix, FORM_TYPE, FORM_PBM) {
        &IFF_PBM
    } else {
        return None;
    };
    with_subformat(Label::Ilbm, sub)
}

/// Disambiguates EA IFF 85 `FORM` 3D containers (`cinema4d` `MC4D`/`FRAY` vs `lightwave` `LWOB`/`LWLO`/`LWO2`/`LWO3`).
pub(crate) fn iff_cinema4d_or_lightwave(prefix: &[u8], size: u64) -> Option<Label> {
    static C4D_MC4D: Subformat = Subformat::new("Cinema 4D (MC4D)");
    static C4D_FRAY: Subformat = Subformat::new("Cinema 4D (FRAY)");
    static LW_LWOB: Subformat = Subformat::with_ids("LightWave Object (LWOB)", "fmt/1205", None);
    static LW_LWLO: Subformat =
        Subformat::with_ids("LightWave Layered Object (LWLO)", "fmt/1205", None);
    static LW_LWO2: Subformat = Subformat::with_ids("LightWave Object v2 (LWO2)", "fmt/1205", None);
    static LW_LWO3: Subformat = Subformat::with_ids("LightWave Object v3 (LWO3)", "fmt/1205", None);
    static LW_LWSC: Subformat = Subformat::new("LightWave Scene (IFF LWSC)");
    const C4D_MC4D_FORM: &[u8; 4] = b"MC4D";
    const C4D_FRAY_FORM: &[u8; 4] = b"FRAY";
    const LW_LWOB_FORM: &[u8; 4] = b"LWOB";
    const LW_LWLO_FORM: &[u8; 4] = b"LWLO";
    const LW_LWO2_FORM: &[u8; 4] = b"LWO2";
    const LW_LWO3_FORM: &[u8; 4] = b"LWO3";
    const LW_LWSC_FORM: &[u8; 4] = b"LWSC";

    require!(
        size >= 12,
        prefix.len() >= 12,
        starts(prefix, IFF_FORM_MAGIC),
        (4..0x0800_0000).contains(&be32(prefix, 4)?),
    );
    if eq_at(prefix, 8, C4D_MC4D_FORM) {
        return with_subformat(Label::Cinema4d, &C4D_MC4D);
    }
    if eq_at(prefix, 8, C4D_FRAY_FORM) {
        return with_subformat(Label::Cinema4d, &C4D_FRAY);
    }
    let lw_sub = if eq_at(prefix, 8, LW_LWOB_FORM) {
        &LW_LWOB
    } else if eq_at(prefix, 8, LW_LWLO_FORM) {
        &LW_LWLO
    } else if eq_at(prefix, 8, LW_LWO2_FORM) {
        &LW_LWO2
    } else if eq_at(prefix, 8, LW_LWO3_FORM) {
        &LW_LWO3
    } else if eq_at(prefix, 8, LW_LWSC_FORM) {
        &LW_LWSC
    } else {
        return None;
    };
    with_subformat(Label::Lightwave, lw_sub)
}

/// Disambiguates EA IFF 85 `FORM` containers between Amiga `ilbm` and 3D `cinema4d`/`lightwave`.
#[inline]
pub(crate) fn ilbm_or_iff(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| ilbm(prefix)).or_else(|| probe(|| iff_cinema4d_or_lightwave(prefix, size)))
}
