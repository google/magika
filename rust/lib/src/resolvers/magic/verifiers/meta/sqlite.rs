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

/// Well-known SQLite 3 `application_id` subformats (offset `@68`, big-endian `u32`).
fn sqlite_app_subformat(app_id: u32) -> Option<&'static Subformat> {
    const APP_FOSSIL_REPO: u32 = 0x0F05_5111;
    const APP_FOSSIL_CHECKOUT: u32 = 0x0F05_5112;
    const APP_FOSSIL_GLOBAL_CFG: u32 = 0x0F05_5113;
    const APP_FOSSIL_LEGACY: u32 = tag_u32(b"_FSL");
    const APP_AUDACITY_3: u32 = tag_u32(b"AUDY");
    const APP_ESRI_LOWER: u32 = tag_u32(b"Esri");
    const APP_ESRI_UPPER: u32 = tag_u32(b"ESRI");
    const APP_BENTLEY_BESQLITE: u32 = 0xBE7D_EC1A;
    const APP_BENTLEY_BEDB: u32 = tag_u32(b"BeDb");
    const APP_BENTLEY_BELN: u32 = tag_u32(b"BeLn");
    const APP_SQLAR: u32 = tag_u32(b"Sqlr");
    const APP_BINARY_NINJA: u32 = tag_u32(b"V35\0");
    const APP_CALIBRE: u32 = tag_u32(b"cali");
    const APP_BITCOIN_MAINNET: u32 = 0xF9BE_B4D9;
    const APP_BITCOIN_TESTNET: u32 = 0x0B11_0907;
    const APP_BITCOIN_REGTEST: u32 = 0xFABF_B5DA;
    const APP_MAPLE_WORKBOOK: u32 = 0x5CDE_09EF;
    const APP_GNOME_TRACKER: u32 = tag_u32(b"Spql");
    const APP_MONEY_MANAGER_EX: u32 = tag_u32(b"MMEX");
    const APP_TWINMOTION: u32 = tag_u32(b"TMPR");
    const APP_AMD_DXCACHE: u32 = 0xD3D1_2CDB;
    const APP_ACORN: u32 = tag_u32(b"Acrn");
    const APP_FORENSICSTORE: u32 = tag_u32(b"elem");
    const APP_CEWE_PHOTOBOOK: u32 = tag_u32(b"CEWE");
    const APP_BEEREF: u32 = 0x7ACC_CCCE;
    const APP_KSTARS: u32 = tag_u32(b"MQQX");
    const APP_TEXNICARD: u32 = 0x6A03_5744;

    static FOSSIL_REPO: Subformat = Subformat::new("Fossil SCM repository");
    static FOSSIL_CHECKOUT: Subformat = Subformat::new("Fossil SCM checkout");
    static FOSSIL_GLOBAL_CFG: Subformat = Subformat::new("Fossil SCM global configuration");
    static AUDACITY_3: Subformat =
        Subformat::with_ids("Audacity 3 project (.aup3)", "fmt/1826", None);
    static ESRI_ST_GEOMETRY: Subformat = Subformat::new("Esri ST_Geometry database");
    static BENTLEY_BESQLITE: Subformat = Subformat::new("Bentley BeSQLite database");
    static BENTLEY_BELN: Subformat = Subformat::new("Bentley localization database");
    static SQLAR: Subformat = Subformat::new("SQLite Archive (.sqlar)");
    static BINARY_NINJA: Subformat = Subformat::new("Binary Ninja analysis database (.bndb)");
    static CALIBRE: Subformat = Subformat::new("Calibre library database");
    static BITCOIN_WALLET: Subformat = Subformat::new("Bitcoin Core descriptor wallet");
    static MAPLE_WORKBOOK: Subformat = Subformat::new("Maple workbook (.maple)");
    static GNOME_TRACKER: Subformat = Subformat::new("GNOME Tracker SPARQL store");
    static MONEY_MANAGER_EX: Subformat = Subformat::new("Money Manager Ex database (.mmb)");
    static TWINMOTION: Subformat = Subformat::new("Twinmotion project (.tm)");
    static AMD_DXCACHE: Subformat = Subformat::new("AMD DirectX shader cache (.dxcache)");
    static ACORN: Subformat = Subformat::new("Acorn image (.acorn)");
    static FORENSICSTORE: Subformat = Subformat::new("ForensicStore container (.forensicstore)");
    static CEWE_PHOTOBOOK: Subformat = Subformat::new("CEWE Photobook project (.mcfx)");
    static BEEREF: Subformat = Subformat::new("BeeRef scene (.bee)");
    static KSTARS: Subformat = Subformat::new("KDE KStars catalog (.kscat)");
    static TEXNICARD: Subformat = Subformat::new("TeXnicard card database");

    match app_id {
        APP_FOSSIL_REPO | APP_FOSSIL_LEGACY => Some(&FOSSIL_REPO),
        APP_FOSSIL_CHECKOUT => Some(&FOSSIL_CHECKOUT),
        APP_FOSSIL_GLOBAL_CFG => Some(&FOSSIL_GLOBAL_CFG),
        APP_AUDACITY_3 => Some(&AUDACITY_3),
        APP_ESRI_LOWER | APP_ESRI_UPPER => Some(&ESRI_ST_GEOMETRY),
        APP_BENTLEY_BESQLITE | APP_BENTLEY_BEDB => Some(&BENTLEY_BESQLITE),
        APP_BENTLEY_BELN => Some(&BENTLEY_BELN),
        APP_SQLAR => Some(&SQLAR),
        APP_BINARY_NINJA => Some(&BINARY_NINJA),
        APP_CALIBRE => Some(&CALIBRE),
        APP_BITCOIN_MAINNET | APP_BITCOIN_TESTNET | APP_BITCOIN_REGTEST => Some(&BITCOIN_WALLET),
        APP_MAPLE_WORKBOOK => Some(&MAPLE_WORKBOOK),
        APP_GNOME_TRACKER => Some(&GNOME_TRACKER),
        APP_MONEY_MANAGER_EX => Some(&MONEY_MANAGER_EX),
        APP_TWINMOTION => Some(&TWINMOTION),
        APP_AMD_DXCACHE => Some(&AMD_DXCACHE),
        APP_ACORN => Some(&ACORN),
        APP_FORENSICSTORE => Some(&FORENSICSTORE),
        APP_CEWE_PHOTOBOOK => Some(&CEWE_PHOTOBOOK),
        APP_BEEREF => Some(&BEEREF),
        APP_KSTARS => Some(&KSTARS),
        APP_TEXNICARD => Some(&TEXNICARD),
        _ => None,
    }
}

/// Validates a SQLite 3 database header (`"SQLite format 3\0"` at `@0..16`, valid power-of-two
/// page size at `@16`, format write/read versions `1|2` at `@18..20`, payload fractions
/// `64, 32, 32` at `@21..24`, schema format `0..=4` at `@44`, and text encoding `0..=3` at `@56`)
/// and dispatches on `application_id` at `@68` between `geopackage`, `qgis`, `mbtiles`, and `sqlite`.
pub(crate) fn sqlite_family(prefix: &[u8], size: u64) -> Option<Label> {
    static QGIS_SQLITE: Subformat = Subformat::new("QGIS SQLite database");
    static MBTILES_HEADER: Subformat = Subformat::new("MBTiles (MPBX header)");
    static MBTILES_SCHEMA: Subformat = Subformat::new("MBTiles (schema)");
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
    if app_id == MPBX {
        return with_subformat(Label::Mbtiles, &MBTILES_HEADER);
    }
    if (contains(prefix, MBTILES_TABLE_TILES) || contains(prefix, MBTILES_VIEW_TILES))
        && contains(prefix, MBTILES_TABLE_METADATA)
    {
        return with_subformat(Label::Mbtiles, &MBTILES_SCHEMA);
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
    if app_id != 0
        && let Some(sub) = sqlite_app_subformat(app_id)
    {
        return with_subformat(Label::Sqlite, sub);
    }
    Some(Label::Sqlite)
}

/// Disambiguates SQLite 3 database containers (`geopackage`, `qgis`, `mbtiles`, `sqlite`).
#[inline]
pub(crate) fn sqlite_or_geopackage(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| geopackage(prefix)).or_else(|| probe(|| sqlite_family(prefix, size)))
}
