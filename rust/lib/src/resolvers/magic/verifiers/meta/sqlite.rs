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

//! SQLite 3 database metaformat verifiers (`sqlite`, `geopackage`, `mbtiles`, `qgis`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

pub(crate) const SQLITE3_MAGIC: &[u8; 16] = b"SQLite format 3\0"; // @0..16: SQLite 3 header magic
const APP_GPKG: u32 = tag_u32(b"GPKG");
const APP_GP10: u32 = tag_u32(b"GP10");
const APP_GP11: u32 = tag_u32(b"GP11");
static GEOPACKAGE_1_0: Subformat = Subformat::with_fdd("GeoPackage 1.0", "fdd000419");
static GEOPACKAGE_1_1: Subformat = Subformat::new("GeoPackage 1.1");
static GEOPACKAGE_1_2: Subformat = Subformat::new("GeoPackage 1.2+");

/// GeoPackage version named by an already-validated `application_id`.
fn geopackage_subformat(app_id: u32) -> &'static Subformat {
    match app_id {
        APP_GP10 => &GEOPACKAGE_1_0,
        APP_GP11 => &GEOPACKAGE_1_1,
        _ => &GEOPACKAGE_1_2,
    }
}

/// OGC GeoPackage: SQLite 3 database header (`"SQLite format 3\0"`) whose `application_id` at
/// offset 68 is `"GPKG"`, `"GP10"`, or `"GP11"`.
pub(crate) fn geopackage(prefix: &[u8]) -> Option<Label> {
    const APPLICATION_ID: usize = 68; // @68: u32 BE — SQLite application_id field
    const MIN_LEN: usize = 100; // complete 100-byte SQLite database header

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, SQLITE3_MAGIC),
        matches!(be32(prefix, APPLICATION_ID)?, APP_GPKG | APP_GP10 | APP_GP11),
    );
    with_subformat(Label::Geopackage, geopackage_subformat(be32(prefix, APPLICATION_ID)?))
}

/// Validates a SQLite 3 database header (`"SQLite format 3\0"` at `@0..16`, valid power-of-two
/// page size at `@16`, format write/read versions `1|2` at `@18..20`, payload fractions
/// `64, 32, 32` at `@21..24`, schema format `0..=4` at `@44`, and text encoding `0..=3` at `@56`)
/// and dispatches on `application_id` at `@68` between `geopackage`, `qgis`, and `sqlite`.
pub(crate) fn sqlite_family(prefix: &[u8], size: u64) -> Option<Label> {
    static QGIS_SQLITE: Subformat = Subformat::new("QGIS SQLite database");
    const APP_ID: usize = 68;
    const QGIS: u32 = tag_u32(b"QGIS");
    const MPBX: u32 = tag_u32(b"MPBX");
    const MBTILES_TABLE_TILES: &[u8] = b"CREATE TABLE tiles";
    const MBTILES_VIEW_TILES: &[u8] = b"CREATE VIEW tiles";
    const MBTILES_TABLE_METADATA: &[u8] = b"CREATE TABLE metadata";
    const SQLITE_FRACTIONS: &[u8; 3] = &[64, 32, 32];

    require!(size >= 100, prefix.len() >= 100, starts(prefix, SQLITE3_MAGIC),);
    let app_id = be32(prefix, APP_ID)?;
    if matches!(app_id, APP_GPKG | APP_GP10) {
        return with_subformat(Label::Geopackage, geopackage_subformat(app_id));
    }
    if app_id == QGIS {
        return with_subformat(Label::Qgis, &QGIS_SQLITE);
    }
    if app_id == MPBX
        || ((contains(prefix, MBTILES_TABLE_TILES) || contains(prefix, MBTILES_VIEW_TILES))
            && contains(prefix, MBTILES_TABLE_METADATA))
    {
        return Some(Label::Mbtiles);
    }
    let page_raw = be16(prefix, 16)?;
    let page_size: u32 = if page_raw == 1 { 65536 } else { page_raw as u32 };
    require!(
        (512..=65536).contains(&page_size),
        page_size.is_power_of_two(),
        matches!(u8_at(prefix, 18)?, 1 | 2),
        matches!(u8_at(prefix, 19)?, 1 | 2),
        eq_at(prefix, 21, SQLITE_FRACTIONS),
        be32(prefix, 44)? <= 4,
        be32(prefix, 56)? <= 3,
    );
    Some(Label::Sqlite)
}

/// Disambiguates SQLite 3 database containers (`geopackage`, `qgis`, `mbtiles`, `sqlite`).
#[inline]
pub(crate) fn sqlite_or_geopackage(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| geopackage(prefix)).or_else(|| probe(|| sqlite_family(prefix, size)))
}
