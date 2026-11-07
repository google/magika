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

//! Tier 4 (`offset_probes`): length-gated signatures at fixed non-zero offsets or trailers.

use crate::ContentType as Label;
use super::*;

/// Tier 4 (`offset_probes`): probes fixed non-zero offsets and container trailers.
#[inline]
pub(crate) fn probe(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    let size = ctx.size;
    super::probe(|| ftyp(prefix))
        .or_else(|| super::probe(|| isobmff_mp4_or_qt(prefix, size)))
        .or_else(|| super::probe(|| ace(prefix)))
        .or_else(|| super::probe(|| dicom(prefix)))
        .or_else(|| super::probe(|| duckdb(prefix)))
        .or_else(|| super::probe(|| ese(prefix)))
        .or_else(|| super::probe(|| icc(prefix)))
        .or_else(|| super::probe(|| palmos(prefix)))
        .or_else(|| super::probe(|| osm_pbf(prefix)))
        .or_else(|| super::probe(|| lha(prefix, size)))
        .or_else(|| super::probe(|| tar(prefix, size)))
        .or_else(|| super::probe(|| iso(prefix, size)))
        .or_else(|| super::probe(|| dmg_koly(size, ctx.tail)))
        .or_else(|| super::probe(|| nintendo_3ds(prefix, size)))
        .or_else(|| super::probe(|| berkeleydb(prefix, size)))
        .or_else(|| super::probe(|| filemaker(prefix, size)))
        .or_else(|| super::probe(|| hfs(prefix, size)))
        .or_else(|| super::probe(|| iges(prefix, size)))
        .or_else(|| super::probe(|| leveldb(prefix, size, ctx.tail)))
        .or_else(|| super::probe(|| lmdb(prefix, size)))
        .or_else(|| super::probe(|| mysql_storage(prefix, size)))
        .or_else(|| super::probe(|| nifti(prefix, size)))
        .or_else(|| super::probe(|| oracle_dbf(prefix, size)))
        .or_else(|| super::probe(|| paradox(prefix, size)))
        .or_else(|| super::probe(|| pcd_kodak(prefix, size)))
        .or_else(|| super::probe(|| pythonpar(prefix, size, ctx.tail)))
        .or_else(|| super::probe(|| realm_offset(prefix, size)))
        .or_else(|| super::probe(|| rocksdb(prefix, size, ctx.tail)))
        .or_else(|| super::probe(|| tflite(prefix, size)))
        .or_else(|| super::probe(|| uefi_fv(prefix, size)))
}
