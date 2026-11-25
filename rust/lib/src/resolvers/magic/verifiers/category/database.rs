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

//! Database, key-value store, and spatial index format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// PostgreSQL custom-format dump (`pg_dump -Fc`): `"PGDMP"` magic, major version 1, integer/offset
/// sizes in `1..=32`, and compression/format code `1 | 3 | 5`.
pub(crate) fn postgres_dump(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"PGDMP\x01"; // @0..6: PGDMP signature + vmajor = 1
    const VMINOR: usize = 6; // @6: u8 — minor archive version
    const MIN_LEN: usize = 9;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    let minor = u8_at(prefix, VMINOR)?;
    let valid = if minor == 0 {
        (1..=32).contains(&u8_at(prefix, 7)?) && matches!(u8_at(prefix, 8)?, 1 | 3 | 5)
    } else if minor <= 6 {
        prefix.len() >= 10
            && (1..=32).contains(&u8_at(prefix, 8)?)
            && matches!(u8_at(prefix, 9)?, 1 | 3 | 5)
    } else {
        prefix.len() >= 11
            && (1..=32).contains(&u8_at(prefix, 8)?)
            && (1..=32).contains(&u8_at(prefix, 9)?)
            && matches!(u8_at(prefix, 10)?, 1 | 3 | 5)
    };
    require!(valid);
    Some(Label::PostgresDump)
}

/// Redis RDB snapshot: `"REDIS"` followed by a nonzero 4-digit ASCII version (`"0001"`..`"9999"`)
/// and a binary RDB opcode byte at `@9`.
pub(crate) fn redis_rdb(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"REDIS"; // @0..5: Redis RDB signature
    const VERSION: usize = 5; // @5..9: 4-digit ASCII version
    const ZERO_VERSION: u32 = tag_u32(b"0000");
    const MIN_LEN: usize = 10;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        get_slice(prefix, VERSION, 9)?.iter().all(u8::is_ascii_digit),
        be32(prefix, VERSION)? != ZERO_VERSION,
        has_non_ascii_text_byte(&prefix[9..10]),
    );
    Some(Label::RedisRdb)
}

/// DuckDB database file: full 4 KiB main header with `"DUCK"` at `@8` and a 64-bit version word
/// (`1..=u32::MAX`) at `@12..20`.
pub(crate) fn duckdb(prefix: &[u8]) -> Option<Label> {
    const MAGIC_OFFSET: usize = 8; // @8..12: "DUCK" after 8-byte checksum
    const MAGIC: &[u8] = b"DUCK";
    const VERSION_LO: usize = 12; // @12: u32 LE — format version low word (≥ 1)
    const VERSION_HI: usize = 16; // @16: u32 LE — format version high word (0)
    const MIN_LEN: usize = 4096;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, MAGIC_OFFSET, MAGIC),
        le32(prefix, VERSION_LO)? >= 1,
        le32(prefix, VERSION_HI)? == 0,
    );
    Some(Label::Duckdb)
}

/// Microsoft Extensible Storage Engine (ESE / Jet Blue) database: magic `0x89ABCDEF` LE at `@4`,
/// nonzero `ulVersion` at `@8`, `attrib` `0` or `1` at `@12`, and zero word at `@132`.
pub(crate) fn ese(prefix: &[u8]) -> Option<Label> {
    const MAGIC_OFFSET: usize = 4; // @4..8: u32 LE — 0x89ABCDEF after 4-byte checksum
    const MAGIC: &[u8] = &0x89AB_CDEFu32.to_le_bytes();
    const VERSION: usize = 8; // @8: u32 LE — ulVersion (≥ 1)
    const ATTRIB: usize = 12; // @12: u32 LE — file type (0 = database, 1 = streaming file)
    const RESERVED_ZERO: usize = 132; // @132: u32 LE — zero field
    const MIN_LEN: usize = 668;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, MAGIC_OFFSET, MAGIC),
        le32(prefix, VERSION)? >= 1,
        le32(prefix, ATTRIB)? <= 1,
        le32(prefix, RESERVED_ZERO)? == 0,
    );
    Some(Label::Ese)
}

/// dBASE database table (`.dbf`): either a dBASE III/IV/V header (≥ 64 bytes, date in `@2..4`,
/// header size ≥ 64 at `@8`, record size ≥ 1 at `@10`, ASCII letter field name at `@32`, and
/// valid field type at `@43`) or a fixed-header dBASE II table (file size ≥ 521, `0x02` at `@0`,
/// record size `1..=0x03FF` at `@6`, field name at `@8`, type `'C'|'L'|'N'` at `@19`, length ≥ 1
/// at `@20`).
pub(crate) fn dbase(prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    static DBASE_II: Subformat = Subformat::with_ids("dBASE II", "x-fmt/8", Some("fdd000325"));
    static DBASE_III: Subformat = Subformat::with_ids("dBASE III", "x-fmt/9", Some("fdd000325"));
    static DBASE_IV: Subformat = Subformat::with_ids("dBASE IV", "x-fmt/10", Some("fdd000325"));
    const VERSION: usize = 0; // @0: u8 — dBASE version byte
    let v0 = u8_at(prefix, 0)?;

    if v0 == 0x02 {
        // dBASE II: fixed 521-byte header
        const DB2_MIN_PREFIX: usize = 24;
        const DB2_MIN_FILE_SIZE: u64 = 521;
        require!(
            prefix.len() >= DB2_MIN_PREFIX,
            ctx.size >= DB2_MIN_FILE_SIZE,
            (u8_at(prefix, 3)? == 0 && u8_at(prefix, 4)? == 0 && u8_at(prefix, 5)? == 0)
                || ((1..=12).contains(&u8_at(prefix, 3)?) && (1..=31).contains(&u8_at(prefix, 4)?)),
            le16(prefix, 6)? >= 1,
            u8_at(prefix, 7)? <= 3,
            u8_at(prefix, 8)?.is_ascii_alphabetic(),
            matches!(u8_at(prefix, 19)?, b'C' | b'L' | b'N'),
            u8_at(prefix, 20)? >= 1,
        );
        return with_subformat(Label::Dbase, &DBASE_II);
    }

    const DB3_MIN_PREFIX: usize = 64;
    require!(
        prefix.len() >= DB3_MIN_PREFIX,
        matches!(v0, 0x03 | 0x04 | 0x43 | 0x63 | 0x7B | 0x83 | 0x8B | 0x8E | 0xCB),
        (1..=12).contains(&u8_at(prefix, 2)?),
        (1..=31).contains(&u8_at(prefix, 3)?),
        le16(prefix, 8)? >= 64,
        le16(prefix, 10)? >= 1,
        u8_at(prefix, 32)?.is_ascii_alphabetic(),
    );
    let field_type = u8_at(prefix, 43)?;
    let valid_type = match v0 {
        0x8B | 0xCB | 0x8E | 0x7B => {
            matches!(field_type, b'C' | b'D' | b'F' | b'L' | b'M' | b'N')
        }
        0x04 | 0x43 | 0x63 => matches!(field_type, b'C' | b'D' | b'F' | b'L' | b'N'),
        0x03 | 0x83 => matches!(field_type, b'C' | b'D' | b'L' | b'M' | b'N'),
        _ => false,
    };
    require!(valid_type);
    let _ = VERSION;
    with_subformat(Label::Dbase, if matches!(v0, 0x03 | 0x83) { &DBASE_III } else { &DBASE_IV })
}

/// Microsoft Access Jet (`"Standard Jet DB\0"`) or ACE (`"Standard ACE DB\0"`) database (`access`,
/// 1992): `00 01 00 00` page header prefix at `@0..4` followed by the 16-byte engine signature at `@4..20`.
pub(crate) fn access(prefix: &[u8], size: u64) -> Option<Label> {
    static ACCESS_JET: Subformat =
        Subformat::with_fdd("Microsoft Access Jet database (.mdb)", "fdd000462");
    static ACCESS_ACE: Subformat =
        Subformat::with_ids("Microsoft Access ACE database (.accdb)", "fmt/275", Some("fdd000463"));
    const ACCESS_PAGE_PREFIX: &[u8; 4] = &[0x00, 0x01, 0x00, 0x00];
    const ACCESS_JET_DB: &[u8; 16] = b"Standard Jet DB\0";
    const ACCESS_ACE_DB: &[u8; 16] = b"Standard ACE DB\0";

    require!(
        size >= 64,
        prefix.len() >= 20,
        starts(prefix, ACCESS_PAGE_PREFIX),
        eq_at(prefix, 4, ACCESS_JET_DB) || eq_at(prefix, 4, ACCESS_ACE_DB),
    );
    with_subformat(
        Label::Access,
        if eq_at(prefix, 4, ACCESS_JET_DB) { &ACCESS_JET } else { &ACCESS_ACE },
    )
}

/// IBM DB2 database backup, check/index file, bind file, or conversion table (`db2`, 1993):
/// `"DB2BACKUP"`, `"\x7FDB2CK"`, `"\xFF\xFBIND\0\0\0"`, `"BIND V02"`, or `"SQLOCONVHD\0\0"` header.
pub(crate) fn db2(prefix: &[u8], size: u64) -> Option<Label> {
    const DB2_BACKUP_MAGIC: &[u8; 9] = b"DB2BACKUP";
    const DB2_CK_MAGIC: &[u8; 6] = b"\x7FDB2CK";
    const DB2_IND_MAGIC: &[u8; 8] = b"\xFF\xFBIND\0\0\0";
    const DB2_BIND_V02_MAGIC: &[u8; 8] = b"BIND V02";
    const DB2_SQLOCONVHD_MAGIC: &[u8; 12] = b"SQLOCONVHD\0\0";

    require!(
        size >= 32,
        prefix.len() >= 16,
        starts(prefix, DB2_CK_MAGIC)
            || starts(prefix, DB2_IND_MAGIC)
            || starts(prefix, DB2_SQLOCONVHD_MAGIC)
            || ((starts(prefix, DB2_BACKUP_MAGIC) || starts(prefix, DB2_BIND_V02_MAGIC))
                && has_non_ascii_text_byte(&prefix[8..16])),
    );
    Some(Label::Db2)
}

/// H2 Database Engine MVStore file (`.mv.db`, `h2`, 2014): `"H:2,block:"` key-value header line
/// containing `"version:"` followed by binary page data.
pub(crate) fn h2(prefix: &[u8], size: u64) -> Option<Label> {
    const H2_MVSTORE_MAGIC: &[u8] = b"H:2,block:";
    const H2_VERSION_KEY: &[u8] = b"version:";

    require!(
        size >= 32,
        prefix.len() >= 24,
        starts(prefix, H2_MVSTORE_MAGIC),
        contains(prefix, H2_VERSION_KEY),
        has_non_ascii_text_byte(&prefix[10..prefix.len().min(64)]),
    );
    Some(Label::H2)
}

/// Oracle Export dump file (`.dmp`, `oracle_db`, 1979): `"\x03\x00\x01EXPORT:"` binary export header
/// or `"EXPORT:V"` version banner with binary dump records.
pub(crate) fn oracle_dmp_text(prefix: &[u8], size: u64) -> Option<Label> {
    static ORACLE_EXPORT: Subformat = Subformat::new("Oracle Export dump");
    const ORACLE_EXP_BIN_MAGIC: &[u8; 10] = b"\x03\x00\x01EXPORT:";
    const ORACLE_EXP_TEXT_MAGIC: &[u8] = b"EXPORT:V";

    require!(
        size >= 16,
        prefix.len() >= 12,
        starts(prefix, ORACLE_EXP_BIN_MAGIC)
            || (starts(prefix, ORACLE_EXP_TEXT_MAGIC) && has_non_ascii_text_byte(prefix)),
    );
    with_subformat(Label::OracleDb, &ORACLE_EXPORT)
}

/// Protomaps PMTiles cloud-optimized map tile archive (`pmtiles`, 2021): `"PMTiles\x03"` or
/// `"PMTiles\x02"` magic at `@0..8` and ≥ 127-byte header.
pub(crate) fn pmtiles(prefix: &[u8], size: u64) -> Option<Label> {
    static PMTILES_V2: Subformat = Subformat::new("PMTiles v2");
    static PMTILES_V3: Subformat = Subformat::new("PMTiles v3");
    const PMTILES_V3_MAGIC: &[u8; 8] = b"PMTiles\x03";
    const PMTILES_V2_MAGIC: &[u8; 8] = b"PMTiles\x02";

    require!(
        size >= 127,
        prefix.len() >= 16,
        starts(prefix, PMTILES_V3_MAGIC) || starts(prefix, PMTILES_V2_MAGIC),
    );
    with_subformat(
        Label::Pmtiles,
        if starts(prefix, PMTILES_V3_MAGIC) { &PMTILES_V3 } else { &PMTILES_V2 },
    )
}

pub(crate) const REALM_TDB_MAGIC: &[u8; 4] = b"T-DB";

/// Realm Mobile Database (`realm`, 2014): `"T-DB"` magic at `@0..4` and top-ref index `0` or `1` at `@20`.
pub(crate) fn realm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 24,
        prefix.len() >= 24,
        starts(prefix, REALM_TDB_MAGIC),
        u8_at(prefix, 20)? <= 1,
    );
    Some(Label::Realm)
}

/// Redis 7.0+ Multi-Part Append-Only File manifest (`redis_aof`, 2022): UTF-8 manifest lines with
/// `"file "`, `" seq "`, and `" type "` tokens.
pub(crate) fn redis_aof_manifest(prefix: &[u8], size: u64) -> Option<Label> {
    static REDIS_AOF_MANIFEST: Subformat = Subformat::new("Redis AOF manifest");
    const AOF_MANIFEST_FILE_KEY: &[u8] = b"file ";
    const AOF_MANIFEST_SEQ_KEY: &[u8] = b" seq ";
    const AOF_MANIFEST_TYPE_KEY: &[u8] = b" type ";

    require!(
        size >= 20,
        prefix.len() >= 20,
        starts(prefix, AOF_MANIFEST_FILE_KEY)
            && contains(prefix, AOF_MANIFEST_SEQ_KEY)
            && contains(prefix, AOF_MANIFEST_TYPE_KEY),
        is_utf8_text_header(prefix, size as usize),
    );
    with_subformat(Label::RedisAof, &REDIS_AOF_MANIFEST)
}

/// WiredTiger storage engine data/log file (`wiredtiger`, 2012): `"WT_FILE\0"` turtle header,
/// `41 D8 01 00` B-Tree block header, or `E0 0D 00 00` write-ahead log header with version `1..=3`.
pub(crate) fn wiredtiger(prefix: &[u8], size: u64) -> Option<Label> {
    const WT_FILE_MAGIC: &[u8; 8] = b"WT_FILE\0";
    const WT_BTREE_MAGIC: &[u8] = &[0x41, 0xD8, 0x01, 0x00];
    const WT_LOG_MAGIC: &[u8] = &[0xE0, 0x0D, 0x00, 0x00];

    require!(
        size >= 16,
        prefix.len() >= 16,
        starts(prefix, WT_FILE_MAGIC)
            || (starts(prefix, WT_BTREE_MAGIC)
                && matches!(le16(prefix, 4)?, 1..=3)
                && le16(prefix, 6)? == 0)
            || (starts(prefix, WT_LOG_MAGIC) && matches!(le16(prefix, 4)?, 1..=3)),
    );
    Some(Label::Wiredtiger)
}

/// Firebird / InterBase relational database (`firebird`, 2000): `01 00 39 30` backup header or
/// `01 00` database header page (`pag_type == 1`) with power-of-two page size `1024..=32768` at
/// `@16` and ODS major version `10..=14` at `@18`.
pub(crate) fn firebird(prefix: &[u8], size: u64) -> Option<Label> {
    static FIREBIRD_BACKUP: Subformat = Subformat::new("Firebird backup (gbak)");
    static FIREBIRD_DATABASE: Subformat = Subformat::new("Firebird database (ODS)");
    const FIREBIRD_GBK_MAGIC: &[u8] = &[0x01, 0x00, 0x39, 0x30];
    const FIREBIRD_ODS_HDR: &[u8] = &[0x01, 0x00];

    require!(
        size >= 64,
        prefix.len() >= 32,
        starts(prefix, FIREBIRD_GBK_MAGIC)
            || (starts(prefix, FIREBIRD_ODS_HDR)
                && matches!(le16(prefix, 16)?, 1024 | 2048 | 4096 | 8192 | 16384 | 32768)
                && matches!(le16(prefix, 18)? & 0x0FFF, 10..=14)),
    );
    with_subformat(
        Label::Firebird,
        if starts(prefix, FIREBIRD_GBK_MAGIC) { &FIREBIRD_BACKUP } else { &FIREBIRD_DATABASE },
    )
}

/// Redis Append-Only File RESP command stream (`redis_aof`, 2009): RESP array opening with
/// `*2\r\n$6\r\nSELECT\r\n` or `*3\r\n$3\r\nSET\r\n`.
pub(crate) fn redis_aof_resp(prefix: &[u8], size: u64) -> Option<Label> {
    static REDIS_AOF_RESP: Subformat = Subformat::new("Redis AOF (RESP commands)");
    const RESP_SELECT_UPPER: &[u8] = b"*2\r\n$6\r\nSELECT\r\n";
    const RESP_SELECT_LOWER: &[u8] = b"*2\r\n$6\r\nselect\r\n";
    const RESP_SET_UPPER: &[u8] = b"*3\r\n$3\r\nSET\r\n";
    const RESP_BULK_STR_DELIM: &[u8] = b"\r\n$";

    require!(
        size >= 24,
        prefix.len() >= 24,
        starts(prefix, RESP_SELECT_UPPER)
            || starts(prefix, RESP_SELECT_LOWER)
            || (starts(prefix, RESP_SET_UPPER) && contains(prefix, RESP_BULK_STR_DELIM)),
    );
    with_subformat(Label::RedisAof, &REDIS_AOF_RESP)
}

/// Realm Mobile Database (`realm`, 2014): 16-byte top-ref header with `"T-DB"` magic at `@16..20`
/// and active flag `0` or `1` at `@23`.
pub(crate) fn realm_offset(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 24,
        prefix.len() >= 24,
        eq_at(prefix, 16, REALM_TDB_MAGIC),
        u8_at(prefix, 23)? <= 1,
    );
    Some(Label::Realm)
}

/// Sleepycat / Oracle Berkeley DB (`berkeleydb`, 1996): Btree (`0x00053162`), Hash (`0x00061561`),
/// or Queue (`0x00042253`) magic at `@12..16` and version `1..=20` at `@16..20` (LE or BE).
pub(crate) fn berkeleydb(prefix: &[u8], size: u64) -> Option<Label> {
    static BDB_BTREE: Subformat = Subformat::new("Berkeley DB (Btree)");
    static BDB_HASH: Subformat = Subformat::new("Berkeley DB (Hash)");
    static BDB_QUEUE: Subformat = Subformat::new("Berkeley DB (Queue)");
    const BDB_BTREE_LE: &[u8] = &[0x62, 0x31, 0x05, 0x00];
    const BDB_BTREE_BE: &[u8] = &[0x00, 0x05, 0x31, 0x62];
    const BDB_HASH_LE: &[u8] = &[0x61, 0x15, 0x06, 0x00];
    const BDB_HASH_BE: &[u8] = &[0x00, 0x06, 0x15, 0x61];
    const BDB_QUEUE_LE: &[u8] = &[0x53, 0x22, 0x04, 0x00];
    const BDB_QUEUE_BE: &[u8] = &[0x00, 0x04, 0x22, 0x53];

    require!(size >= 64, prefix.len() >= 28);
    if eq_at(prefix, 12, BDB_BTREE_LE)
        || eq_at(prefix, 12, BDB_BTREE_BE)
        || eq_at(prefix, 12, BDB_HASH_LE)
        || eq_at(prefix, 12, BDB_HASH_BE)
        || eq_at(prefix, 12, BDB_QUEUE_LE)
        || eq_at(prefix, 12, BDB_QUEUE_BE)
    {
        let ver = le32(prefix, 16)?;
        require!((1..=20).contains(&ver) || (1..=20).contains(&ver.swap_bytes()));
        let sub = if eq_at(prefix, 12, BDB_BTREE_LE) || eq_at(prefix, 12, BDB_BTREE_BE) {
            &BDB_BTREE
        } else if eq_at(prefix, 12, BDB_HASH_LE) || eq_at(prefix, 12, BDB_HASH_BE) {
            &BDB_HASH
        } else {
            &BDB_QUEUE
        };
        return with_subformat(Label::Berkeleydb, sub);
    }
    None
}

/// Claris / FileMaker Pro database (`.fp7` / `.fmp12`, `filemaker`, 1985): `00 01 00 00` header
/// prefix at `@0..4` paired with `"HBAM2"` or `"HBAM7"` signature at `@525` (or `@14`).
pub(crate) fn filemaker(prefix: &[u8], size: u64) -> Option<Label> {
    const FILEMAKER_HDR_PREFIX: &[u8; 4] = &[0x00, 0x01, 0x00, 0x00];
    const FILEMAKER_HBAM2_TAG: &[u8; 5] = b"HBAM2";
    const FILEMAKER_HBAM7_TAG: &[u8; 5] = b"HBAM7";

    require!(
        size >= 544,
        prefix.len() >= 540,
        starts(prefix, FILEMAKER_HDR_PREFIX),
        eq_at(prefix, 525, FILEMAKER_HBAM2_TAG)
            || eq_at(prefix, 525, FILEMAKER_HBAM7_TAG)
            || eq_at(prefix, 14, FILEMAKER_HBAM2_TAG),
    );
    Some(Label::Filemaker)
}

/// Google LevelDB SSTable (`.ldb` / `.sst`, `leveldb`, 2011): 48-byte table footer ending with
/// 64-bit magic `0xDB4775248B80FB57` LE at `size - 8`.
pub(crate) fn leveldb(_prefix: &[u8], size: u64, tail: Option<&[u8]>) -> Option<Label> {
    const LEVELDB_FOOTER_MAGIC: &[u8] = &[0x57, 0xFB, 0x80, 0x8B, 0x24, 0x75, 0x47, 0xDB];
    let t = tail?;
    require!(size >= 48, t.len() >= 48, eq_at(t, t.len() - 8, LEVELDB_FOOTER_MAGIC),);
    Some(Label::Leveldb)
}

/// Symas Lightning Memory-Mapped Database (`lmdb`, 2011): `0xBEEFC0DE` LE magic (`DE C0 EF BE`) at
/// `@0` (lockfile), `@12` (32-bit meta page), or `@16` (64-bit meta page).
pub(crate) fn lmdb(prefix: &[u8], size: u64) -> Option<Label> {
    static LMDB_LOCK: Subformat = Subformat::new("LMDB lock file");
    static LMDB_DATA_32: Subformat = Subformat::new("LMDB data file (32-bit)");
    static LMDB_DATA_64: Subformat = Subformat::new("LMDB data file (64-bit)");
    const LMDB_BEEFC0DE_LE: &[u8] = &[0xDE, 0xC0, 0xEF, 0xBE];

    require!(
        size >= 24,
        prefix.len() >= 24,
        starts(prefix, LMDB_BEEFC0DE_LE)
            || eq_at(prefix, 12, LMDB_BEEFC0DE_LE)
            || eq_at(prefix, 16, LMDB_BEEFC0DE_LE),
    );
    let sub = if starts(prefix, LMDB_BEEFC0DE_LE) {
        &LMDB_LOCK
    } else if eq_at(prefix, 12, LMDB_BEEFC0DE_LE) {
        &LMDB_DATA_32
    } else {
        &LMDB_DATA_64
    };
    with_subformat(Label::Lmdb, sub)
}

/// MySQL / MariaDB table definition (`.frm`, `FE FE 07 01`) or MyISAM index (`.MYI`, `FE FE 09`)
/// storage file (`mysql_storage`, 1995).
pub(crate) fn mysql_storage(prefix: &[u8], size: u64) -> Option<Label> {
    static MYSQL_FRM: Subformat =
        Subformat::with_ids("MySQL table definition (.frm)", "fmt/868", Some("fdd000623"));
    static MYISAM_MYI: Subformat = Subformat::with_ids("MyISAM index (.MYI)", "fmt/1197", None);
    const MYSQL_FRM_MAGIC: &[u8] = &[0xFE, 0xFE, 0x07, 0x01];
    const MYSQL_MYI_MAGIC: &[u8] = &[0xFE, 0xFE, 0x09];

    require!(
        size >= 64,
        prefix.len() >= 24,
        starts(prefix, MYSQL_FRM_MAGIC)
            || (starts(prefix, MYSQL_MYI_MAGIC) && matches!(u8_at(prefix, 3)?, 1..=12)),
    );
    with_subformat(
        Label::MysqlStorage,
        if starts(prefix, MYSQL_FRM_MAGIC) { &MYSQL_FRM } else { &MYISAM_MYI },
    )
}

/// Oracle Database datafile / controlfile (`.dbf` / `.ctl`, `oracle_db`, 1979): block header with
/// magic `0x7AC2F8F0` (LE or BE) at `@20..24` and ≥ 512-byte size.
pub(crate) fn oracle_dbf(prefix: &[u8], size: u64) -> Option<Label> {
    static ORACLE_DATAFILE: Subformat = Subformat::new("Oracle datafile / control file");
    const ORACLE_DBF_MAGIC_LE: &[u8] = &[0xF0, 0xF8, 0xC2, 0x7A];
    const ORACLE_DBF_MAGIC_BE: &[u8] = &[0x7A, 0xC2, 0xF8, 0xF0];

    require!(
        size >= 512,
        prefix.len() >= 32,
        eq_at(prefix, 20, ORACLE_DBF_MAGIC_LE) || eq_at(prefix, 20, ORACLE_DBF_MAGIC_BE),
    );
    with_subformat(Label::OracleDb, &ORACLE_DATAFILE)
}

/// Borland Paradox relational database table (`.db`, `paradox`, 1985): record size at `@0..2`,
/// 2 KiB-aligned header size at `@2..4`, file type `0` or `2` at `@4`, block size code `1..=4` at
/// `@5`, field count `>= 1` at `@33`, and version ID `3..=12` at `@57`.
pub(crate) fn paradox(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 96, prefix.len() >= 92);
    let rec_size = le16(prefix, 0)?;
    let hdr_size = le16(prefix, 2)?;
    let ftype = u8_at(prefix, 4)?;
    let bsize_code = u8_at(prefix, 5)?;
    let num_fields = u8_at(prefix, 33)?;
    let ver_id = u8_at(prefix, 57)?;
    require!(
        (1..=32768).contains(&rec_size),
        hdr_size >= 0x0800 && hdr_size % 0x0800 == 0,
        matches!(ftype, 0 | 2),
        (1..=4).contains(&bsize_code),
        num_fields >= 1,
        (3..=12).contains(&ver_id),
        u8_at(prefix, 88)? == 0 && u8_at(prefix, 91)? == 0,
    );
    Some(Label::Paradox)
}

/// Meta RocksDB SSTable (`.sst`, `rocksdb`, 2012): 53-byte table footer ending with 64-bit
/// BlockBased, PlainTable, or CuckooTable magic at `size - 8`.
pub(crate) fn rocksdb(_prefix: &[u8], size: u64, tail: Option<&[u8]>) -> Option<Label> {
    const ROCKSDB_BLOCK_MAGIC_LE: &[u8] = &[0x75, 0x3F, 0x6B, 0x98, 0xF7, 0x85, 0xB2, 0x88];
    const ROCKSDB_BLOCK_MAGIC_BE: &[u8] = &[0x88, 0xB2, 0x85, 0xF7, 0x98, 0x6B, 0x3F, 0x75];
    const ROCKSDB_PLAIN_MAGIC_LE: &[u8] = &[0xF7, 0xCF, 0xF4, 0x85, 0xB7, 0x41, 0xE2, 0x88];
    const ROCKSDB_CUCKOO_MAGIC_LE: &[u8] = &[0x64, 0x95, 0xBF, 0x63, 0x96, 0x22, 0x42, 0x82];

    let t = tail?;
    require!(
        size >= 53,
        t.len() >= 48,
        eq_at(t, t.len() - 8, ROCKSDB_BLOCK_MAGIC_LE)
            || eq_at(t, t.len() - 8, ROCKSDB_BLOCK_MAGIC_BE)
            || eq_at(t, t.len() - 8, ROCKSDB_PLAIN_MAGIC_LE)
            || eq_at(t, t.len() - 8, ROCKSDB_CUCKOO_MAGIC_LE),
    );
    Some(Label::Rocksdb)
}

/// ESRI Shapefile (`.shp` main or `.shx` index): file code `9994` BE at `@0`, 20 unused zero bytes
/// at `@4..24`, file length ≥ 54 words at `@24`, version `1000` LE at `@28`, valid shape type at
/// `@32`, and first record number `1` (`.shp`) or offset `50` words (`.shx`) BE at `@100`.
pub(crate) fn shapefile(prefix: &[u8]) -> Option<Label> {
    static SHAPEFILE_SHP: Subformat = Subformat::new("Esri Shapefile main file (.shp)");
    static SHAPEFILE_SHX: Subformat =
        Subformat::with_ids("Esri Shapefile index (.shx)", "fmt/277", None);
    const FILE_CODE: u32 = 9994; // @0: u32 BE — ESRI shapefile code (0x0000270A)
    const UNUSED: usize = 4; // @4..24: 5 × u32 unused zero words
    const FILE_LEN_WORDS: usize = 24; // @24: u32 BE — file length in 16-bit words (≥ 54)
    const VERSION: usize = 28; // @28: u32 LE — shapefile version (1000)
    const SHAPE_TYPE: usize = 32; // @32: u32 LE — shape type code
    const FIRST_RECORD: usize = 100; // @100: u32 BE — record 1 (.shp) or 50-word header offset (.shx)
    const MIN_LEN: usize = 108;

    require!(
        prefix.len() >= MIN_LEN,
        be32(prefix, 0)? == FILE_CODE,
        eq_at(prefix, UNUSED, &[0; 20]),
        be32(prefix, FILE_LEN_WORDS)? >= 54,
        le32(prefix, VERSION)? == 1000,
        matches!(
            le32(prefix, SHAPE_TYPE)?,
            0 | 1 | 3 | 5 | 8 | 11 | 13 | 15 | 18 | 21 | 23 | 25 | 28 | 31
        ),
        matches!(be32(prefix, FIRST_RECORD)?, 1 | 50),
    );
    with_subformat(
        Label::Shapefile,
        if be32(prefix, FIRST_RECORD)? == 1 { &SHAPEFILE_SHP } else { &SHAPEFILE_SHX },
    )
}

/// Björn Harrtell FlatGeobuf cloud-native vector geospatial format (`flatgeobuf`, 2018):
/// `"fgb\x03fgb"`, `"fgb\x02fgb"`, or `"fgb\x01fgb"` header magic at `@0..7` and patch byte `<= 3` at `@7`.
pub(crate) fn flatgeobuf(prefix: &[u8], size: u64) -> Option<Label> {
    static FGB_V1: Subformat = Subformat::new("FlatGeobuf v1");
    static FGB_V2: Subformat = Subformat::new("FlatGeobuf v2");
    static FGB_V3: Subformat = Subformat::new("FlatGeobuf v3");
    const FGB_V3_MAGIC: &[u8; 7] = b"fgb\x03fgb";
    const FGB_V2_MAGIC: &[u8; 7] = b"fgb\x02fgb";
    const FGB_V1_MAGIC: &[u8; 7] = b"fgb\x01fgb";

    require!(
        size >= 16,
        prefix.len() >= 12,
        starts(prefix, FGB_V3_MAGIC)
            || starts(prefix, FGB_V2_MAGIC)
            || starts(prefix, FGB_V1_MAGIC),
        u8_at(prefix, 7)? <= 3,
    );
    let sub = if starts(prefix, FGB_V3_MAGIC) {
        &FGB_V3
    } else if starts(prefix, FGB_V2_MAGIC) {
        &FGB_V2
    } else {
        &FGB_V1
    };
    with_subformat(Label::Flatgeobuf, sub)
}

const SEGY_TEXT_HEADER_LEN: usize = 3200; // @0..3200: 40 x 80-byte textual file header
const SEGY_BINARY_HEADER_END: usize = 3600; // @3200..3600: 400-byte binary file header
const SEGY_TRACE0_HEADER_END: usize = 3840; // @3600..3840: 240-byte first trace header
const SEGY_TRACE_HEADER_LEN: u64 = 240;

const SEGY_BIN_INTERVAL: usize = 3216; // @3216: i16/u16 — sample interval in microseconds
const SEGY_BIN_SAMPLES: usize = 3220; // @3220: u16 — samples per data trace
const SEGY_BIN_FORMAT: usize = 3224; // @3224: u16 — data sample format code
const SEGY_BIN_SORTING_CODE: usize = 3228; // @3228: i16 — trace sorting code (-1..=9)
const SEGY_BIN_MEASUREMENT_SYSTEM: usize = 3254; // @3254: u16 — measurement system (0..=2)
const SEGY_BIN_EXT_SAMPLES: usize = 3268; // @3268: i32 — Rev 2 extended sample count
const SEGY_BIN_EXT_INTERVAL: usize = 3272; // @3272: f64 — Rev 2 extended sample interval
const SEGY_BIN_ENDIAN_MAGIC: usize = 3296; // @3296..3300: Rev 2 byte-order constant
const SEGY_BIN_UNASSIGNED1_START: usize = 3300; // @3300..3500: 200 unassigned bytes
const SEGY_BIN_UNASSIGNED1_END: usize = 3500;
const SEGY_BIN_FIXED_LENGTH_FLAG: usize = 3502; // @3502: u16 — fixed-length trace flag (0..=1)
const SEGY_BIN_EXT_HEADERS: usize = 3504; // @3504: i16 — extended textual file header count
const SEGY_BIN_MAX_ADD_TR_HEADERS: usize = 3506; // @3506: u16 — Rev 2 max additional trace headers
const SEGY_BIN_FIRST_TRACE_OFFSET: usize = 3520; // @3520..3528: u64 — Rev 2 first trace byte offset
const SEGY_BIN_UNASSIGNED2_START: usize = 3532; // @3532..3600: 68 unassigned bytes
const SEGY_BIN_UNASSIGNED2_END: usize = 3600;
const SEGY_TRACE0_SAMPLES: usize = 3714; // @3714: u16 — samples in first trace (@3600 + 114)
const SEGY_TRACE0_INTERVAL: usize = 3716; // @3716: u16 — sample interval in first trace (@3600 + 116)

const SEGY_ENDIAN_UNSET: &[u8; 4] = &[0x00, 0x00, 0x00, 0x00];
const SEGY_ENDIAN_BE: &[u8; 4] = &[0x01, 0x02, 0x03, 0x04];
const SEGY_ENDIAN_LE: &[u8; 4] = &[0x04, 0x03, 0x02, 0x01];
const SEGY_MAX_BAD_TEXT_BYTES: usize = 16;
const SEGY_MIN_TEXT_CHARS: usize = 16;
const SEGY_MAX_NUL_TRANSITIONS: usize = 64;
/// Plausible Rev 2 extended sample intervals. The lower bound rejects subnormals, which is what
/// any small integer stored in `@3272..3280` decodes to; `Range::contains` also rejects NaN,
/// infinities, zero, and negatives.
const SEGY_EXT_INTERVAL_RANGE: core::ops::Range<f64> = 1e-9..1e9;
const SEGY_TEXT_CHUNK_LEN: usize = 64; // divides SEGY_TEXT_HEADER_LEN (3200 = 50 * 64)
const SEGY_SWAR_LANE_MASK: u64 = 0x1FF; // 9 bits per lane (max 64 per 64-byte chunk)
const SEGY_SWAR_HIGH_BITS: u64 = 0x8080_8080_8080_8080;
const SEGY_SWAR_LOW7_BITS: u64 = 0x7F7F_7F7F_7F7F_7F7F;
const SEGY_SWAR_PREV_NUL_BIT: u64 = 0x80; // bit 7 carry matching `zero >> 56`
const _: () = assert!(SEGY_TEXT_HEADER_LEN.is_multiple_of(SEGY_TEXT_CHUNK_LEN));

/// Sample width in bytes for SEG-Y Rev 0/1/2.0/2.1 data sample format codes (`1..=12, 15, 16`).
#[inline]
const fn segy_sample_bytes(fmt: u16) -> Option<u64> {
    match fmt {
        1 | 2 | 4 | 5 | 10 => Some(4),
        3 | 11 => Some(2),
        6 | 9 | 12 => Some(8),
        7 | 15 => Some(3),
        8 | 16 => Some(1),
        _ => None,
    }
}

/// EBCDIC (CP037 / CP500) letter or digit byte.
#[inline]
const fn is_ebcdic_alnum(b: u8) -> bool {
    matches!(
        b,
        0x81..=0x89
            | 0x91..=0x99
            | 0xA2..=0xA9
            | 0xC1..=0xC9
            | 0xD1..=0xD9
            | 0xE2..=0xE9
            | 0xF0..=0xF9
    )
}

/// Printable EBCDIC (CP037 / CP500) character or card-image control/padding byte.
#[inline]
const fn is_ebcdic_text_byte(b: u8) -> bool {
    is_ebcdic_alnum(b)
        || matches!(
            b,
            0x00 | 0x05
                | 0x0D
                | 0x15
                | 0x25
                | 0x40
                | 0x4A..=0x50
                | 0x5A..=0x61
                | 0x6A..=0x6F
                | 0x79..=0x7F
                | 0xA1
                | 0xAD
                | 0xB0
                | 0xBA
                | 0xBB
                | 0xBD
                | 0xC0
                | 0xD0
                | 0xE0
        )
}

/// Seven 9-bit byte-class counters packed into a `u64` for 64-byte chunked textual-header scanning:
/// - bits `0..9`: non-EBCDIC text byte (`ebcdic_bad`)
/// - bits `9..18`: EBCDIC space `0x40` (`ebcdic_space`)
/// - bits `18..27`: EBCDIC alphanumeric (`ebcdic_alnum`)
/// - bits `27..36`: printable ASCII excluding `0x40` (`ascii_print_non_at`)
/// - bits `36..45`: ASCII alphanumeric (`ascii_alnum`)
/// - bits `45..54`: non-ASCII text byte (`ascii_bad`)
/// - bits `54..63`: NUL byte `0x00` (`chunk_nuls`)
const SEGY_TEXT_SWAR_LUT: [u64; 256] = {
    let mut table = [0u64; 256];
    let mut i = 0usize;
    while i < 256 {
        let b = i as u8;
        let mut entry = 0u64;
        if !is_ebcdic_text_byte(b) {
            entry |= 1u64;
        } else if b == 0x40 {
            entry |= 1u64 << 9;
        } else if is_ebcdic_alnum(b) {
            entry |= 1u64 << 18;
        }

        if matches!(b, 0x20..=0x7E) {
            if b != 0x40 {
                entry |= 1u64 << 27;
            }
            if b.is_ascii_alphanumeric() {
                entry |= 1u64 << 36;
            }
        } else if !matches!(b, 0x00 | b'\t' | b'\n' | b'\r') {
            entry |= 1u64 << 45;
        }

        if b == 0x00 {
            entry |= 1u64 << 54;
        }

        table[i] = entry;
        i += 1;
    }
    table
};

struct SegyBinMatch {
    geom_ok: bool,
    trace0_match: bool,
}

fn check_segy_bin_header(prefix: &[u8], size: u64, big_endian: bool) -> Option<SegyBinMatch> {
    let rd16 = |off| if big_endian { be16(prefix, off) } else { le16(prefix, off) };
    let rd32 = |off| if big_endian { be32(prefix, off) } else { le32(prefix, off) };
    let rd64 = |off| if big_endian { be64(prefix, off) } else { le64(prefix, off) };

    let fmt = rd16(SEGY_BIN_FORMAT)?;
    let bps = segy_sample_bytes(fmt)?;

    let dt16 = rd16(SEGY_BIN_INTERVAL)?;
    let dt_ext_bits = rd64(SEGY_BIN_EXT_INTERVAL)?;
    if dt_ext_bits != 0 {
        require!(SEGY_EXT_INTERVAL_RANGE.contains(&f64::from_bits(dt_ext_bits)));
    } else {
        require!(dt16 != 0);
    }

    let ns16 = rd16(SEGY_BIN_SAMPLES)?;
    let ext_ns = rd32(SEGY_BIN_EXT_SAMPLES)? as i32;
    let ns = if ext_ns != 0 {
        require!(ext_ns > 0);
        ext_ns as u64
    } else {
        require!(ns16 > 0);
        u64::from(ns16)
    };

    let sorting_code = rd16(SEGY_BIN_SORTING_CODE)? as i16;
    let meas = rd16(SEGY_BIN_MEASUREMENT_SYSTEM)?;
    let fixed = rd16(SEGY_BIN_FIXED_LENGTH_FLAG)?;
    let ext_headers = rd16(SEGY_BIN_EXT_HEADERS)? as i16;
    require!((-1..=9).contains(&sorting_code), meas <= 2, fixed <= 1, ext_headers >= -1);

    // Extended textual headers declared by a positive count must fit in the file, wherever the
    // traces start.
    let implied_end = if ext_headers > 0 {
        let ext_bytes = (ext_headers as u64).checked_mul(SEGY_TEXT_HEADER_LEN as u64)?;
        let end = (SEGY_BINARY_HEADER_END as u64).checked_add(ext_bytes)?;
        require!(end <= size);
        Some(end)
    } else if ext_headers == 0 {
        Some(SEGY_BINARY_HEADER_END as u64)
    } else {
        None
    };
    // A non-zero Rev 2 first-trace offset overrides the offset implied by the count; zero means
    // unknown.
    let first_trace = rd64(SEGY_BIN_FIRST_TRACE_OFFSET)?;
    let header_end = if first_trace != 0 {
        require!((SEGY_BINARY_HEADER_END as u64..=size).contains(&first_trace));
        Some(first_trace)
    } else {
        implied_end
    };

    let mut geom_ok = false;
    if let Some(start) = header_end {
        let max_add_tr = u64::from(rd16(SEGY_BIN_MAX_ADD_TR_HEADERS)?);
        let trace_size = max_add_tr
            .checked_add(1)
            .and_then(|n| n.checked_mul(SEGY_TRACE_HEADER_LEN))
            .and_then(|hdr| ns.checked_mul(bps).and_then(|s| hdr.checked_add(s)));
        if let Some(trace_size) = trace_size {
            let rem = size - start;
            if rem >= trace_size && rem.is_multiple_of(trace_size) {
                geom_ok = true;
            }
        }
    }

    let mut trace0_match = false;
    if header_end == Some(SEGY_BINARY_HEADER_END as u64)
        && size >= SEGY_TRACE0_HEADER_END as u64
        && prefix.len() >= SEGY_TRACE0_HEADER_END
    {
        let tr_ns = rd16(SEGY_TRACE0_SAMPLES)?;
        let tr_dt = rd16(SEGY_TRACE0_INTERVAL)?;
        trace0_match = (ns16 > 0 && tr_ns == ns16) || (dt16 != 0 && tr_dt == dt16);
    }

    Some(SegyBinMatch { geom_ok, trace0_match })
}

/// Society of Exploration Geophysicists SEG-Y seismic data (`segy`, 1975, Rev 0 / 1 / 2.0 / 2.1):
/// 3200-byte EBCDIC, ASCII, or zero textual file header followed by a 400-byte binary file header
/// (`@3200..3600`) with consistent byte order, valid sample format code (`1..=12 | 15 | 16`),
/// non-zero sample interval and sample count, valid control enums, and zeroed unassigned ranges
/// (`@3300..3500` and `@3532..3600`).
pub(crate) fn segy(prefix: &[u8], size: u64) -> Option<Label> {
    static SEGY_EBCDIC: Subformat = Subformat::new("SEG-Y (EBCDIC textual header)");
    static SEGY_ASCII: Subformat = Subformat::new("SEG-Y (ASCII textual header)");
    static SEGY_EMPTY: Subformat = Subformat::new("SEG-Y (empty textual header)");
    require!(size >= SEGY_BINARY_HEADER_END as u64, prefix.len() >= SEGY_BINARY_HEADER_END);

    // 1. Rev 2 byte-order constant @3296..3300
    let em = get_slice(prefix, SEGY_BIN_ENDIAN_MAGIC, SEGY_BIN_ENDIAN_MAGIC + 4)?;
    let (probe_be, probe_le, has_endian_magic) = if em == SEGY_ENDIAN_UNSET {
        (true, true, false)
    } else if em == SEGY_ENDIAN_BE {
        (true, false, true)
    } else if em == SEGY_ENDIAN_LE {
        (false, true, true)
    } else {
        return None;
    };

    // 2. Binary file header validation under a single consistent endianness
    let bin_match = if probe_be { check_segy_bin_header(prefix, size, true) } else { None }
        .or_else(|| if probe_le { check_segy_bin_header(prefix, size, false) } else { None })?;

    // 3. Unassigned binary header padding (@3300..3500 and @3532..3600)
    require!(
        is_all_zeros(get_slice(prefix, SEGY_BIN_UNASSIGNED1_START, SEGY_BIN_UNASSIGNED1_END)?),
        is_all_zeros(get_slice(prefix, SEGY_BIN_UNASSIGNED2_START, SEGY_BIN_UNASSIGNED2_END)?),
    );

    // 4. Textual file header (@0..3200): EBCDIC, corroborated ASCII, or corroborated all-zero
    let hdr = get_slice(prefix, 0, SEGY_TEXT_HEADER_LEN)?;
    if is_all_zeros(hdr) {
        require!(has_endian_magic || (bin_match.geom_ok && bin_match.trace0_match));
        return with_subformat(Label::Segy, &SEGY_EMPTY);
    }

    let mut ebcdic_bad = 0usize;
    let mut ebcdic_space = 0usize;
    let mut ebcdic_alnum = 0usize;
    let mut ascii_print_non_at = 0usize;
    let mut ascii_alnum = 0usize;
    let mut ascii_bad = 0usize;
    let mut nul_transitions = 0usize;
    let mut prev_nul_bit = 0u64;

    for chunk in hdr.as_chunks::<SEGY_TEXT_CHUNK_LEN>().0 {
        let mut acc0 = 0u64;
        let mut acc1 = 0u64;
        let mut acc2 = 0u64;
        let mut acc3 = 0u64;
        for &[b0, b1, b2, b3] in chunk.as_chunks::<4>().0 {
            acc0 += SEGY_TEXT_SWAR_LUT[usize::from(b0)];
            acc1 += SEGY_TEXT_SWAR_LUT[usize::from(b1)];
            acc2 += SEGY_TEXT_SWAR_LUT[usize::from(b2)];
            acc3 += SEGY_TEXT_SWAR_LUT[usize::from(b3)];
        }
        let acc = acc0 + acc1 + acc2 + acc3;
        ebcdic_bad += (acc & SEGY_SWAR_LANE_MASK) as usize;
        ebcdic_space += ((acc >> 9) & SEGY_SWAR_LANE_MASK) as usize;
        ebcdic_alnum += ((acc >> 18) & SEGY_SWAR_LANE_MASK) as usize;
        ascii_print_non_at += ((acc >> 27) & SEGY_SWAR_LANE_MASK) as usize;
        ascii_alnum += ((acc >> 36) & SEGY_SWAR_LANE_MASK) as usize;
        ascii_bad += ((acc >> 45) & SEGY_SWAR_LANE_MASK) as usize;
        let chunk_nuls = ((acc >> 54) & SEGY_SWAR_LANE_MASK) as usize;
        if ebcdic_bad > SEGY_MAX_BAD_TEXT_BYTES && ascii_bad > SEGY_MAX_BAD_TEXT_BYTES {
            return None;
        }

        if chunk_nuls == 0 {
            nul_transitions += usize::from(prev_nul_bit != 0);
            prev_nul_bit = 0;
        } else if chunk_nuls == SEGY_TEXT_CHUNK_LEN {
            prev_nul_bit = SEGY_SWAR_PREV_NUL_BIT;
        } else {
            for &word in chunk.as_chunks::<8>().0 {
                let w = u64::from_le_bytes(word);
                let non_zero =
                    (((w & !SEGY_SWAR_HIGH_BITS) + SEGY_SWAR_LOW7_BITS) | w) & SEGY_SWAR_HIGH_BITS;
                let zero = non_zero ^ SEGY_SWAR_HIGH_BITS;
                let prev_is_nul = (zero << 8) | prev_nul_bit;
                nul_transitions += (prev_is_nul & non_zero).count_ones() as usize;
                prev_nul_bit = zero >> 56;
            }
        }
        if nul_transitions > SEGY_MAX_NUL_TRANSITIONS {
            return None;
        }
    }

    let corroborated = has_endian_magic || bin_match.geom_ok || bin_match.trace0_match;
    let is_ebcdic = ebcdic_bad <= SEGY_MAX_BAD_TEXT_BYTES
        && ebcdic_alnum >= SEGY_MIN_TEXT_CHARS
        && (ebcdic_space + ebcdic_alnum) > ascii_print_non_at
        && (ebcdic_space >= SEGY_MIN_TEXT_CHARS || corroborated);
    if is_ebcdic {
        return with_subformat(Label::Segy, &SEGY_EBCDIC);
    }

    let is_ascii =
        ascii_bad <= SEGY_MAX_BAD_TEXT_BYTES && ascii_alnum >= SEGY_MIN_TEXT_CHARS && corroborated;
    require!(is_ascii);
    with_subformat(Label::Segy, &SEGY_ASCII)
}

/// Esri ARC/INFO ASCII Grid (`arcgrid`, 1991): `"ncols "` header followed by `nrows`,
/// `xllcorner`/`xllcenter`, and `cellsize` raster grid parameters.
pub(crate) fn arcgrid(prefix: &[u8], size: u64) -> Option<Label> {
    const ARCGRID_NCOLS_LOWER: &[u8] = b"ncols ";
    const ARCGRID_NCOLS_UPPER: &[u8] = b"NCOLS ";
    const ARCGRID_NCOLS_TAB: &[u8] = b"ncols\t";
    const ARCGRID_NROWS_LF_LOWER: &[u8] = b"\nnrows ";
    const ARCGRID_NROWS_LF_UPPER: &[u8] = b"\nNROWS ";
    const ARCGRID_NROWS_CRLF_LOWER: &[u8] = b"\r\nnrows ";
    const ARCGRID_XLLCORNER_LOWER: &[u8] = b"xllcorner";
    const ARCGRID_XLLCORNER_UPPER: &[u8] = b"XLLCORNER";
    const ARCGRID_XLLCENTER_LOWER: &[u8] = b"xllcenter";
    const ARCGRID_CELLSIZE_LOWER: &[u8] = b"cellsize";
    const ARCGRID_CELLSIZE_UPPER: &[u8] = b"CELLSIZE";

    require!(
        size >= 32,
        prefix.len() >= 32,
        (starts(prefix, ARCGRID_NCOLS_LOWER)
            || starts(prefix, ARCGRID_NCOLS_UPPER)
            || starts(prefix, ARCGRID_NCOLS_TAB))
            && (contains(prefix, ARCGRID_NROWS_LF_LOWER)
                || contains(prefix, ARCGRID_NROWS_LF_UPPER)
                || contains(prefix, ARCGRID_NROWS_CRLF_LOWER))
            && (contains(prefix, ARCGRID_XLLCORNER_LOWER)
                || contains(prefix, ARCGRID_XLLCORNER_UPPER)
                || contains(prefix, ARCGRID_XLLCENTER_LOWER))
            && (contains(prefix, ARCGRID_CELLSIZE_LOWER)
                || contains(prefix, ARCGRID_CELLSIZE_UPPER)),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Arcgrid)
}

/// MapInfo TAB geospatial table definition (`mapinfo`, 1990): `"!table"` header line followed by `"!version "`.
pub(crate) fn mapinfo(prefix: &[u8], size: u64) -> Option<Label> {
    const MAPINFO_TABLE_CRLF: &[u8] = b"!table\r\n";
    const MAPINFO_TABLE_LF: &[u8] = b"!table\n";
    const MAPINFO_VERSION_KW: &[u8] = b"!version ";

    require!(
        size >= 20,
        prefix.len() >= 20,
        (starts(prefix, MAPINFO_TABLE_CRLF) || starts(prefix, MAPINFO_TABLE_LF))
            && contains(prefix, MAPINFO_VERSION_KW),
        is_utf8_text_header(prefix, size as usize),
    );
    Some(Label::Mapinfo)
}

/// Apache Avro Object Container File: `"Obj\x01"` magic and nonempty metadata map (byte 4 ≥ 1).
pub(crate) fn avro(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"Obj\x01"; // @0..4: Avro OCF version 1 magic
    const META_COUNT: usize = 4; // @4: u8 — first byte of zig-zag metadata block count (≥ 1)
    const MIN_LEN: usize = 21; // 4-byte magic + ≥1 metadata byte + 16-byte sync marker

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC), u8_at(prefix, META_COUNT)? >= 1);
    Some(Label::Avro)
}

/// MATLAB v5 MAT-file: `"MATLAB 5"` text prefix and 128-byte header with version `0x0100` and
/// endianness indicator `"MI"` (little-endian) or `"IM"` (big-endian) at offset 124.
pub(crate) fn mat(prefix: &[u8]) -> Option<Label> {
    static MAT5_BE: Subformat = Subformat::new("MAT-file Level 5 (big-endian)");
    static MAT5_LE: Subformat = Subformat::new("MAT-file Level 5 (little-endian)");
    const MAGIC: &[u8] = b"MATLAB 5"; // @0..8: descriptive text prefix
    const VERSION_ENDIAN: usize = 124; // @124..128: version 0x0100 + endian tag
    const BE_TAG: &[u8] = bytes!(&0x0100u16.to_be_bytes(), b"MI"); // big-endian v5 indicator
    const LE_TAG: &[u8] = bytes!(&0x0100u16.to_le_bytes(), b"IM"); // little-endian v5 indicator
    const MIN_LEN: usize = 128;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        eq_at(prefix, VERSION_ENDIAN, BE_TAG) || eq_at(prefix, VERSION_ENDIAN, LE_TAG),
    );
    with_subformat(
        Label::Mat,
        if eq_at(prefix, VERSION_ENDIAN, BE_TAG) { &MAT5_BE } else { &MAT5_LE },
    )
}

/// Apache Parquet column store: `"PAR1"` leading magic, ≥ 12 bytes, and binary payload/footer bytes.
pub(crate) fn parquet(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"PAR1"; // @0..4: Parquet magic
    const MIN_LEN: usize = 12;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, MAGIC),
        has_non_ascii_text_byte(&prefix[4..MIN_LEN]),
    );
    Some(Label::Parquet)
}

/// SPSS system file (`.sav` / `.zsav`, `"$FL2"` or `"$FL3"` with layout code 2 or 3 at `@64` and
/// compression `0..=2` at `@72`) or SPSS portable file (EBCDIC `"SPSS"` = `C9 C3 E2 C1` at `@0`,
/// ≥ 464 bytes).
pub(crate) fn spss(prefix: &[u8]) -> Option<Label> {
    static SPSS_SAV: Subformat = Subformat::new("SPSS system file (.sav)");
    static SPSS_ZSAV: Subformat = Subformat::new("SPSS compressed system file (.zsav)");
    static SPSS_POR: Subformat =
        Subformat::with_ids("SPSS portable file (.por)", "fmt/997", Some("fdd000468"));
    const SAV_FL2: &[u8] = b"$FL2"; // @0..4: SPSS SAV magic
    const SAV_FL3: &[u8] = b"$FL3"; // @0..4: SPSS ZSAV magic
    const PORTABLE_EBCDIC: &[u8] = &[0xC9, 0xC3, 0xE2, 0xC1]; // @0..4: EBCDIC "SPSS" in portable header
    const LAYOUT_CODE: usize = 64; // @64: u32 (LE or BE) — layout_code (2 or 3)
    const COMPRESSION: usize = 72; // @72: u32 (LE or BE) — compression switch (0..=2)
    const SAV_MIN_LEN: usize = 176;
    const PORTABLE_MIN_LEN: usize = 464;

    if prefix.len() >= SAV_MIN_LEN && (starts(prefix, SAV_FL2) || starts(prefix, SAV_FL3)) {
        let le_ok = matches!(le32(prefix, LAYOUT_CODE)?, 2 | 3) && le32(prefix, COMPRESSION)? <= 2;
        let be_ok = matches!(be32(prefix, LAYOUT_CODE)?, 2 | 3) && be32(prefix, COMPRESSION)? <= 2;
        require!(le_ok || be_ok);
        return with_subformat(
            Label::Spss,
            if starts(prefix, SAV_FL2) { &SPSS_SAV } else { &SPSS_ZSAV },
        );
    }
    require!(prefix.len() >= PORTABLE_MIN_LEN, starts(prefix, PORTABLE_EBCDIC));
    with_subformat(Label::Spss, &SPSS_POR)
}

/// Apache Arrow IPC file (Feather v2): `"ARROW1\0\0"` padded magic and ≥ 12 bytes.
pub(crate) fn arrow(prefix: &[u8]) -> Option<Label> {
    const MAGIC: &[u8] = b"ARROW1\0\0"; // @0..8: 8-byte padded Arrow IPC file magic
    const MIN_LEN: usize = 12;

    require!(prefix.len() >= MIN_LEN, starts(prefix, MAGIC));
    Some(Label::Arrow)
}

/// SAS7BDAT dataset: 32-byte binary magic at `@0`, alignment/format byte `<= 1` at `@37`, and
/// `"SAS FILE"` at `@84` inside a ≥ 288-byte header.
pub(crate) fn sas(prefix: &[u8]) -> Option<Label> {
    const HEADER: &[u8] = bytes!(
        &[0u8; 12],
        &[
            0xC2, 0xEA, 0x81, 0x60, 0xB3, 0x14, 0x11, 0xCF, 0xBD, 0x92, 0x08, 0x00, 0x09, 0xC7,
            0x31, 0x8C, 0x18, 0x1F, 0x10, 0x11,
        ]
    ); // @0..32: 12 zero bytes + 20-byte SAS7BDAT magic
    const ALIGN_FLAG: usize = 37; // @37: u8 — 0 or 1
    const FILE_TAG_OFFSET: usize = 84; // @84..92: "SAS FILE"
    const FILE_TAG: &[u8] = b"SAS FILE";
    const MIN_LEN: usize = 288;

    require!(
        prefix.len() >= MIN_LEN,
        eq_at(prefix, FILE_TAG_OFFSET, FILE_TAG),
        starts(prefix, HEADER),
        u8_at(prefix, ALIGN_FLAG)? <= 1,
    );
    Some(Label::Sas)
}

/// NASA Common Data Format (`CD F3 00 01` v3 / `CD F2 06 00` v2.6), RIFF `DMPR` CDF, or Unidata
/// NetCDF-5 64-bit data (`"CDF\x05"`) scientific dataset (`cdf`, 1985).
pub(crate) fn cdf(prefix: &[u8], size: u64) -> Option<Label> {
    static CDF_RIFF_DMPR: Subformat = Subformat::new("CDF (RIFF DMPR)");
    static NASA_CDF_V3: Subformat =
        Subformat::with_ids("NASA CDF 3.x", "fmt/1950", Some("fdd000226"));
    static NASA_CDF_V2_6: Subformat =
        Subformat::with_ids("NASA CDF 2.6/2.7", "fmt/1949", Some("fdd000226"));
    static NETCDF_5: Subformat = Subformat::new("NetCDF-5 (64-bit data)");
    const DMPR_FORM: &[u8; 4] = b"DMPR";
    const NASA_CDF3_MAGIC: &[u8] = &[0xCD, 0xF3, 0x00, 0x01];
    const NASA_CDF2_MAGIC: &[u8] = &[0xCD, 0xF2, 0x06, 0x00];
    const NASA_CDF_UNCOMPRESSED: &[u8] = &[0x00, 0x00, 0xFF, 0xFF];
    const NASA_CDF_COMPRESSED: &[u8] = &[0xFF, 0xFF, 0x00, 0x00];
    const NETCDF_64BIT_DATA_MAGIC: &[u8; 4] = b"CDF\x05";

    require!(size >= 12, prefix.len() >= 12);
    if starts(prefix, RIFF_MAGIC) && eq_at(prefix, 8, DMPR_FORM) {
        require!((4..0x0800_0000).contains(&le32(prefix, 4)?));
        return with_subformat(Label::Cdf, &CDF_RIFF_DMPR);
    }
    if starts(prefix, NASA_CDF3_MAGIC) || starts(prefix, NASA_CDF2_MAGIC) {
        require!(eq_at(prefix, 4, NASA_CDF_UNCOMPRESSED) || eq_at(prefix, 4, NASA_CDF_COMPRESSED));
        return with_subformat(
            Label::Cdf,
            if starts(prefix, NASA_CDF3_MAGIC) { &NASA_CDF_V3 } else { &NASA_CDF_V2_6 },
        );
    }
    require!(
        starts(prefix, NETCDF_64BIT_DATA_MAGIC),
        matches!(be32(prefix, 8).unwrap_or(0), 0 | 0x0000_000A | 0x0000_000B),
    );
    with_subformat(Label::Cdf, &NETCDF_5)
}

/// Apache Optimized Row Columnar (`orc`, 2013): `"ORC"` prefix followed by binary stream bytes and
/// a valid trailing `PostScript` footer containing `"ORC"` and bounded `postScriptLength` at `size - 1`.
pub(crate) fn orc(prefix: &[u8], size: u64, tail: Option<&[u8]>) -> Option<Label> {
    const ORC_MAGIC: &[u8; 3] = b"ORC";

    require!(
        size >= 8,
        prefix.len() >= 4,
        starts(prefix, ORC_MAGIC),
        has_non_ascii_text_byte(&prefix[3..4]),
    );
    let t = tail?;
    require!(!t.is_empty());
    let ps_len = u8_at(t, t.len() - 1)? as usize;
    require!(ps_len >= 1 && ps_len + 4 <= t.len());
    if contains(t, ORC_MAGIC)
        && (u8_at(prefix, 3)? == 0x82 || u8_at(prefix, 3)? == 0x0A || ps_len <= 64)
    {
        return Some(Label::Orc);
    }
    None
}
