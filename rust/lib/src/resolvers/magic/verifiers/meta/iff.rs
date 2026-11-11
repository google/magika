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
    const LEGACY_MAGIC: &[u8] = b"\x0f7BEAM!"; // @0..7: legacy BEAM header
    const BEAM_FORM: &[u8] = b"BEAM"; // @8..12: BEAM form identifier
    const FORM_OFFSET: usize = 8; // @8: form type offset
    const MIN_LEN: usize = 8;
    const IFF_MIN_LEN: usize = 12;

    require!(prefix.len() >= MIN_LEN);
    let matched = starts(prefix, LEGACY_MAGIC)
        || (prefix.len() >= IFF_MIN_LEN
            && starts(prefix, IFF_FOR1_MAGIC)
            && u8_at(prefix, 4)? == 0
            && eq_at(prefix, FORM_OFFSET, BEAM_FORM));
    require!(matched);
    Some(Label::Beam)
}

/// Amiga EA IFF 85 bitmap image: `"FORM"` container magic at `@0` and `"ILBM"`, `"ACBM"`, or
/// `"PBM "` form type at `@8`.
pub(crate) fn ilbm(prefix: &[u8]) -> Option<Label> {
    const FORM_TYPE: usize = 8; // @8..12: IFF form type
    const FORM_ILBM: &[u8] = b"ILBM";
    const FORM_ACBM: &[u8] = b"ACBM";
    const FORM_PBM: &[u8] = b"PBM ";
    const MIN_LEN: usize = 12;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, IFF_FORM_MAGIC),
        u8_at(prefix, 4)? == 0,
        eq_at(prefix, FORM_TYPE, FORM_ILBM)
            || eq_at(prefix, FORM_TYPE, FORM_ACBM)
            || eq_at(prefix, FORM_TYPE, FORM_PBM),
    );
    Some(Label::Ilbm)
}

/// Disambiguates EA IFF 85 `FORM` 3D containers (`cinema4d` `MC4D`/`FRAY` vs `lightwave` `LWOB`/`LWLO`/`LWO2`/`LWO3`).
pub(crate) fn iff_cinema4d_or_lightwave(prefix: &[u8], size: u64) -> Option<Label> {
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
    if eq_at(prefix, 8, C4D_MC4D_FORM) || eq_at(prefix, 8, C4D_FRAY_FORM) {
        return Some(Label::Cinema4d);
    }
    if eq_at(prefix, 8, LW_LWOB_FORM)
        || eq_at(prefix, 8, LW_LWLO_FORM)
        || eq_at(prefix, 8, LW_LWO2_FORM)
        || eq_at(prefix, 8, LW_LWO3_FORM)
        || eq_at(prefix, 8, LW_LWSC_FORM)
    {
        return Some(Label::Lightwave);
    }
    None
}

/// Disambiguates EA IFF 85 `FORM` containers between Amiga `ilbm` and 3D `cinema4d`/`lightwave`.
#[inline]
pub(crate) fn ilbm_or_iff(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| ilbm(prefix)).or_else(|| probe(|| iff_cinema4d_or_lightwave(prefix, size)))
}
