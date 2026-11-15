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

/// Society of Exploration Geophysicists SEG-Y seismic data (`segy`, 1975): 3200-byte EBCDIC/ASCII
/// textual reel header + 400-byte binary header with non-zero sample interval (`@3216`), sample
/// count (`@3220`), and data sample format code `1..=8` (`@3224`).
pub(crate) fn segy(prefix: &[u8], size: u64) -> Option<Label> {
    static SEGY_EBCDIC: Subformat = Subformat::new("SEG-Y (EBCDIC textual header)");
    static SEGY_ASCII: Subformat = Subformat::new("SEG-Y (ASCII textual header)");
    require!(size >= 3600, prefix.len() >= 3600);
    let interval = u16::from_be_bytes([prefix[3216], prefix[3217]]);
    let samples = u16::from_be_bytes([prefix[3220], prefix[3221]]);
    let fmt = u16::from_be_bytes([prefix[3224], prefix[3225]]);
    require!(
        interval > 0,
        samples > 0,
        matches!(fmt, 1..=8),
        prefix[0] == 0xC3 || prefix[0].is_ascii_graphic() || prefix[0] == b' ',
    );
    let _ = get_slice(prefix, 0, 4);
    let _ = get_slice(prefix, 3216, 3226);
    with_subformat(Label::Segy, if prefix[0] == 0xC3 { &SEGY_EBCDIC } else { &SEGY_ASCII })
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

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const CAPCOM_MT_GMD_MAGIC: &[u8; 4] = b"GMD\0";
const CLIPPER_NTX_MAGIC_CLIPPER5: &[u8; 2] = hex2bin!("06 00");
const CLIPPER_NTX_MAGIC_HUGE: &[u8; 2] = hex2bin!("26 00");
const CLIPPER_NTX_MAGIC_S87: &[u8; 2] = hex2bin!("03 00");
const CLIPPER_NTX_MAGIC_V53: &[u8; 2] = hex2bin!("06 01");
const HELPPC_INDEX_BANNER_PREFIX: &[u8; 8] = b" HelpPC ";
const HELPPC_INDEX_TERM: &[u8; 5] = hex2bin!("0D 0A 00 1A 09");
const IBUS_EMOJI_MAGIC: &[u8; 32] =
    bytes!(b"IBusEmojiData", &[0x00, 0x05, 0x00], b"IBusEmojiData", &[0x00, 0x00, 0x00]);
const IDD_KB_SIG: &[u8; 24] = b"IDD Knowledge Base File\x00";
const IDR_KB_SIG: &[u8; 24] = b"IDR Knowledge Base File\x00";
const INNODB_FIL_PAGE_OFFSET: usize = 4;
const INNODB_FIL_PAGE_TYPE: usize = 24;
const INNODB_FIL_PAGE_TYPE_FSP_HDR: u16 = 0x0008;
const KUDO_HSH_MAGIC: &[u8; 4] = b"FASH";
const KUDO_KDB_MAGIC: &[u8; 4] = b"NG\x00\x00";
const LUCENE_CODEC_MAGIC: &[u8; 4] = hex2bin!("3F D7 6C 17");
const LUCENE_NAME_LEN_OFFSET: usize = 4;
const MAX_BAI_BINS: u32 = 37_450;
const MAX_N_REF: u32 = 10_000_000;
const MIN_1C_ENTERPRISE_1CD_SIZE: u64 = 4096;
const MIN_ABSOLUTE_DB_QC_SIZE: u64 = 64;
const MIN_ACR5_ICDB_SIZE: u64 = 32;
const MIN_ADEXPLORER_SNAPSHOT_SIZE: u64 = 0x43E;
const MIN_ADOBE_ASN_SIF_SIZE: u64 = 32;
const MIN_AGS_TRA_SIZE: u64 = 24;
const MIN_APPLE_SPOTLIGHT_STORE_SIZE: u64 = 56;
const MIN_ASKSAM_SIZE: u64 = 8;
const MIN_BAI_SIZE: u64 = 8;
const MIN_BETHESDA_ESP_SIZE: u64 = 38;
const MIN_BEYONDTRUST_BRDF_RDF_SIZE: u64 = 32;
const MIN_BIOWARE_KEY_SIZE: u64 = 24;
const MIN_BIOWARE_TLK_SIZE: u64 = 20;
const MIN_BITTORRENT_BTDB_SIZE: u64 = 28;
const MIN_BOLT_DB_SIZE: u64 = 64;
const MIN_BORLAND_BDE_LOCALE_BTL_SIZE: u64 = 48;
const MIN_CAFFEINE_CSCHEME_SIZE: u64 = 28;
const MIN_CAPCOM_MT_GMD_SIZE: u64 = 48;
const MIN_CDTREE_CD_SIZE: u64 = 32;
const MIN_CEDICT_UNIT_U8_SIZE: u64 = 64;
const MIN_CERN_ZEBRA_RZ_SIZE: u64 = 1024;
const MIN_CHAPS_TIMETABLE_TT_SIZE: u64 = 512;
const MIN_CHESSBASE_TREE_CTB_SIZE: u64 = 64;
const MIN_CHROME_BDIC_SIZE: u64 = 32;
const MIN_CHROME_V4STORE_SIZE: u64 = 16;
const MIN_CHROME_VLNK_SIZE: u64 = 32;
const MIN_CHROMIUM_CACHE_BLOCKFILE_SIZE: u64 = 368;
const MIN_CLARION_MEM_SIZE: u64 = 262;
const MIN_CLIPPER_NTX_SIZE: u64 = 36;
const MIN_CONSULTANTPLUS_DT5_SIZE: u64 = 42;
const MIN_CONS_DICT_NX2_SIZE: u64 = 36;
const MIN_DATABENTO_DBN_V1_SIZE: u64 = 64;
const MIN_DBASE_LBL_SIZE: u64 = 64;
const MIN_DELPHI_CDS_SIZE: u64 = 48;
const MIN_DTCH_CHART_SIZE: u64 = 28;
const MIN_EPLAN_EOX_DB_SIZE: u64 = 64;
const MIN_ETHEREUM_ERA1_SIZE: u64 = 24;
const MIN_FBS_TIMETABLE_FPL_SIZE: u64 = 32;
const MIN_FOXPRO_FORM_SCT_SIZE: u64 = 536;
const MIN_GHIDRA_BF_SIZE: u64 = 64;
const MIN_GIT_PACK_REV_SIZE: u64 = 52;
const MIN_GNUPG_TRUSTDB_SIZE: u64 = 40;
const MIN_GSDD_GDD_SIZE: u64 = 28;
const MIN_HELPPC_INDEX_SIZE: u64 = 1160;
const MIN_HFST_OL_TRANSDUCER_SIZE: u64 = 64;
const MIN_HYPERPLANNING_HYP_SIZE: u64 = 64;
const MIN_IBUS_DICT_SIZE: u64 = 64;
const MIN_IDA_DATABASE_SIZE: u64 = 44;
const MIN_IDA_TIL_SIZE: u64 = 32;
const MIN_IDR_KB_SIZE: u64 = 293;
const MIN_IGO_POI_KNS_SIZE: u64 = 32;
const MIN_INFOTRIEVE_TPX_SIZE: u64 = 40;
const MIN_INMAGIC_DBTEXTWORKS_SIZE: u64 = 20;
const MIN_INNODB_TABLESPACE_SIZE: u64 = 16_384;
const MIN_IX_INDEX_SIZE: u64 = 28;
const MIN_JETBRAINS_PHM_AT_SIZE: u64 = 48;
const MIN_KEEPASS_SIZE: u64 = 64;
const MIN_KINDLE_DRMION_SIZE: u64 = 32;
const MIN_KINDLE_KFX_SIZE: u64 = 32;
const MIN_KUDO_HSH_SIZE: u64 = 32;
const MIN_KUDO_KDB_SIZE: u64 = 64;
const MIN_LIFETECH_SDS_SIZE: u64 = 8;
const MIN_LINGEA_LLD_SIZE: u64 = 44;
const MIN_LOCALE_STRNDX_SIZE: u64 = 36;
const MIN_LUCENE_CODEC_SIZE: u64 = 32;
const MIN_MALCAT_CSIG_V1_SIZE: u64 = 28;
const MIN_MALCAT_CSIG_V2_SIZE: u64 = 32;
const MIN_METATRADER_HC_SIZE: u64 = 40;
const MIN_MICROMERITICS_SUBSET_SIZE: u64 = 40;
const MIN_MLOCATE_DB_SIZE: u64 = 24;
const MIN_MSFS_BGL_SIZE: u64 = 76;
const MIN_MSTS_SIMIS_SIZE: u64 = 32;
const MIN_MSVC_IPDB_SIZE: u64 = 44;
const MIN_MSWORKS_WDB_SIZE: u64 = 32;
const MIN_MS_ANSWER_WIZARD_AW_SIZE: u64 = 48;
const MIN_NINTENDO_BMG_SIZE: u64 = 32;
const MIN_NINTENDO_MSBT_SIZE: u64 = 32;
const MIN_NOL_2900_LAYER_SIZE: u64 = 64;
const MIN_OPENFST_SIZE: u64 = 16;
const MIN_OUTLOOK_NK2_SIZE: u64 = 32;
const MIN_PDI2_DICTIONARY_SIZE: u64 = 44;
const MIN_PGP_LOCSTR_SIZE: u64 = 28;
const MIN_PREMIERE_PRMI_SIZE: u64 = 28;
const MIN_RAN_ONLINE_IMS_SIZE: u64 = 32;
const MIN_RE4_MDT_SIZE: u64 = 64;
const MIN_REDB_SIZE: u64 = 512;
const MIN_RPGMAKER_LCF_SIZE: u64 = 16;
const MIN_SAFARI_BINARYCOOKIES_SIZE: u64 = 24;
const MIN_SDC_GDB_SIZE: u64 = 28;
const MIN_SF3_SIZE: u64 = 24;
const MIN_STATISTICA_SCR_SIZE: u64 = 136;
const MIN_STATISTICA_STA_SIZE: u64 = 64;
const MIN_SYSTEMD_CATALOG_SIZE: u64 = 40;
const MIN_TDBD_DATABASE_SIZE: u64 = 32;
const MIN_TENSORSTORE_OCDBT_SIZE: u64 = 20;
const MIN_TOPSPEED_0205_TPS_SIZE: u64 = 64;
const MIN_VALVE_TOOLS_ASSET_INFO_SIZE: u64 = 32;
const MIN_VIM_SPELL_SPL_SIZE: u64 = 28;
const MIN_WARFRAME_TOC_SIZE: u64 = 32;
const MIN_WINBOX_CDB_SIZE: u64 = 16;
const MIN_WINDOWS_CACHES_DB_SIZE: u64 = 64;
const MIN_WITCHER3_W3STRINGS_SIZE: u64 = 28;
const MIN_WOW_DBC_SIZE: u64 = 24;
const MSFS_BGL_MAGIC: &[u8; 4] = hex2bin!("01 02 92 19");
const REDB_MAGIC: &[u8; 9] = bytes!(b"redb", &[0x1A, 0x0A, 0xA9, 0x0D, 0x0A]);
const REDB_PAGE_SIZE_OFFSET: usize = 12;
const REDB_VERSION_OFFSET: usize = 9;
const STATISTICA_SCR_MAGIC: &[u8; 4] = b"\x6F\xD9\xFF\xFF";
const VALVE_TOOLS_ASSET_INFO_V1_MAGIC: &[u8; 4] = hex2bin!("E8 AC CC C4");
const VALVE_TOOLS_ASSET_INFO_V2_MAGIC: &[u8; 4] = hex2bin!("E9 AC CC C4");
const WINBOX_CDB_MAGIC: &[u8; 4] = hex2bin!("0D F0 1D C0");
const WINBOX_M2_TAG: &[u8; 2] = b"M2";

fn is_valid_til_desc(prefix: &[u8], len_off: usize) -> bool {
    let Some(desc_len) = u8_at(prefix, len_off).map(|b| b as usize) else {
        return false;
    };
    if !(1..=250).contains(&desc_len)
        || !u8_at(prefix, len_off + 1).is_some_and(|b| b.is_ascii_graphic())
    {
        return false;
    }
    let check_end = (len_off + 1 + desc_len).min(prefix.len()).min(len_off + 17);
    get_slice(prefix, len_off + 1, check_end)
        .is_some_and(|s| s.iter().all(|&b| (0x20..=0x7E).contains(&b)))
}

/// 1C:Enterprise 8 Infobase database (`1Cv8.1CD`, `"1CDBMSV8"`).
///
/// Header layout (page 0):
/// - `@0..8`: `"1CDBMSV8"` (`8B` signature)
/// - `@8..12`: 4-byte BCD/binary version `[8, minor (1..=5), build (0..=64), 0]` (e.g., `08 02 0E 00` = v8.2.14.0, `08 03 08 00` = v8.3.8.0)
/// - `@12..16`: `u32 LE length_in_blocks` (`>= 1`)
/// - `@16..20`: `u32 LE` unknown constant (`== 1`)
pub(crate) fn verify_1c_enterprise_1cd(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC_1CD: &[u8; 9] = bytes!(b"1CDBMSV8", &[8]); // @0..9: "1CDBMSV8" + major version 8
    const MINOR_VER_OFFSET: usize = 9; // @9: minor version (1..=5, e.g., 2 for 8.2, 3 for 8.3)
    const BUILD_VER_OFFSET: usize = 10; // @10: format revision (0..=64)
    const VER_PAD_OFFSET: usize = 11; // @11: version padding byte (0)
    const NUM_BLOCKS_OFFSET: usize = 12; // @12..16: u32 LE total block count (> 0)
    const CONST1_OFFSET: usize = 16; // @16..20: u32 LE constant 1

    require!(
        size >= MIN_1C_ENTERPRISE_1CD_SIZE,
        prefix.len() >= 24,
        starts(prefix, MAGIC_1CD),
        (1..=5).contains(&u8_at(prefix, MINOR_VER_OFFSET)?),
        u8_at(prefix, BUILD_VER_OFFSET)? <= 64,
        u8_at(prefix, VER_PAD_OFFSET) == Some(0),
        le32(prefix, NUM_BLOCKS_OFFSET)? >= 1,
        le32(prefix, CONST1_OFFSET) == Some(1)
    );
    Some(Label::_1cEnterprise1cd)
}

/// ComponentAce Absolute Database File (`"ABS0LUTEDATABASE"`).
pub(crate) fn verify_absolute_db_qc(prefix: &[u8], size: u64) -> Option<Label> {
    const ABSOLUTE_DB_MAGIC: &[u8; 16] = b"ABS0LUTEDATABASE"; // @0..16

    require!(
        size >= MIN_ABSOLUTE_DB_QC_SIZE,
        prefix.len() >= 28,
        starts(prefix, ABSOLUTE_DB_MAGIC),
        (0x40..=0x200).contains(&le16(prefix, 16)?) // @16..18: header length
    );
    let ver = f64::from_le_bytes(get_slice(prefix, 18, 26)?.try_into().ok()?); // @18..26: IEEE-754 engine version
    let page_size = le16(prefix, 26)?; // @26..28: database page size
    require!(
        (1.0..=50.0).contains(&ver),
        (512..=32768).contains(&page_size),
        page_size.is_power_of_two()
    );
    Some(Label::AbsoluteDbQc)
}

/// ACDSee ACR5 Image Catalog Database (.icdb, ACDSee).
pub(crate) fn verify_acr5_icdb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ACR5_ICDB_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"ACR5ffffff&\x40\x00\x10*\x01\x04\0\x01\0")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::Acr5Icdb)
}

/// Sysinternals Active Directory Explorer Snapshot database (`.dat`).
///
/// Header layout (`0x43E = 1086` bytes):
/// - `@0..10`: `"win-ad-ob\0"` ASCII signature
/// - `@10..14`: `u32 LE` marker (`0x00010001`)
/// - `@14..22`: `u64 LE` Windows `FILETIME` snapshot timestamp
/// - `@22..542`: `wchar_t optionalDescription[260]`
/// - `@542..1062`: `wchar_t server[260]`
/// - `@1062..1066` (`0x426`): `u32 LE numObjects` (`> 0`)
/// - `@1066..1070` (`0x42A`): `u32 LE numAttributes` (`> 0`)
/// - `@1070..1078` (`0x42E`): `u64 LE metadataOffset` (`>= 0x43E && <= size`)
/// - `@1078..1086` (`0x436`): `u64 LE treeviewOffset` (`<= size`)
pub(crate) fn verify_adexplorer_snapshot(prefix: &[u8], size: u64) -> Option<Label> {
    const ADEXP_MAGIC: &[u8; 14] = bytes!(b"win-ad-ob", &[0x00], le32_bytes!(0x0001_0001)); // @0..14: "win-ad-ob\0" + 0x00010001 LE
    const FILETIME_OFFSET: usize = 14; // @14..22: u64 LE Windows FILETIME
    const NUM_OBJECTS_OFFSET: usize = 0x426; // @1062..1066: u32 LE numObjects
    const NUM_ATTRS_OFFSET: usize = 0x42A; // @1066..1070: u32 LE numAttributes
    const METADATA_OFFSET: usize = 0x42E; // @1070..1078: u64 LE metadataOffset
    const TREEVIEW_OFFSET: usize = 0x436; // @1078..1086: u64 LE treeviewOffset

    require!(
        size >= MIN_ADEXPLORER_SNAPSHOT_SIZE,
        prefix.len() >= 0x43E,
        starts(prefix, ADEXP_MAGIC)
    );
    let ft = le64(prefix, FILETIME_OFFSET)?;
    let num_objects = le32(prefix, NUM_OBJECTS_OFFSET)?;
    let num_attrs = le32(prefix, NUM_ATTRS_OFFSET)?;
    let meta_off = le64(prefix, METADATA_OFFSET)?;
    let tree_off = le64(prefix, TREEVIEW_OFFSET)?;
    require!(
        (0x01B0_0000_0000_0000..=0x0250_0000_0000_0000).contains(&ft),
        num_objects > 0,
        num_attrs > 0,
        meta_off >= MIN_ADEXPLORER_SNAPSHOT_SIZE
            && (size == MIN_ADEXPLORER_SNAPSHOT_SIZE || meta_off <= size),
        tree_off >= MIN_ADEXPLORER_SNAPSHOT_SIZE
            && (size == MIN_ADEXPLORER_SNAPSHOT_SIZE || tree_off <= size)
    );
    Some(Label::AdexplorerDat)
}

/// Adobe Developer Association (ASN) Search Index File (`.sif`, 1995) Adobe.
pub(crate) fn verify_adobe_asn_sif(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_ADOBE_ASN_SIF_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"Adobe ASN SIF   \x01\x00\x00\x00")
    );
    Some(Label::AdobeAsnSif)
}

/// Adventure Game Studio Compiled Translation (`.tra`).
///
/// Header layout:
/// - `@0..15`: `"AGSTranslation\0"`
/// - `@15..19`: `u32 LE` `block_type` (`1` = content, `2` = header, `3` = settings)
/// - `@19..23`: `u32 LE` `block_size` (`>= 4`)
pub(crate) fn verify_ags_tra(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 15] = bytes!(b"AGSTranslation", &[0]); // @0..15: NUL-terminated signature
    const BLOCK_TYPE_OFF: usize = 15; // @15..19: u32 LE first block type (1..=3)
    const BLOCK_SIZE_OFF: usize = 19; // @19..23: u32 LE first block payload size

    require!(
        size >= MIN_AGS_TRA_SIZE,
        prefix.len() >= 23,
        starts(prefix, MAGIC),
        (1..=3).contains(&le32(prefix, BLOCK_TYPE_OFF)?),
        le32(prefix, BLOCK_SIZE_OFF)? >= 4
    );
    Some(Label::AgsTra)
}

/// Apple macOS / iOS Spotlight Metadata Store database (`store.db` `"8tsd"` or `dbStr` map header `"\0PataD\0\0"`).
pub(crate) fn verify_apple_spotlight_store(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_APPLE_SPOTLIGHT_STORE_SIZE, prefix.len() >= 56);
    if starts(prefix, b"8tsd") {
        let map_off = le32(prefix, 36)?;
        let map_size = le32(prefix, 40)?;
        let page_size = le32(prefix, 44)?;
        require!(
            map_off > 0,
            map_size > 0,
            (512..=65536).contains(&page_size),
            page_size % 512 == 0
        );
        return Some(Label::AppleSpotlightStore);
    }
    if starts(prefix, b"\0PataD\0\0") {
        let total = le32(prefix, 24)?;
        let used = le32(prefix, 28)?;
        require!(total > 0, used <= total);
        return Some(Label::AppleSpotlightStore);
    }
    None
}

/// askSam Free-Form Database (.ask, PRONOM fmt/2019).
pub(crate) fn verify_asksam(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_ASKSAM_SIZE
        && prefix.len() >= 8
        && (starts(prefix, b"askSam\x00\x00")
            || starts(prefix, b"askWiS\x00\x00")
            || starts(prefix, b"askw40\x00\x00")
            || starts(prefix, b"askwin\x00\x00"))
    {
        Some(Label::Asksam)
    } else {
        None
    }
}

/// SAMtools BAM index (`BAI\x01`): every reference needs at least `n_bin` + `n_intv` (8 bytes).
pub(crate) fn verify_bai(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BAI_SIZE, starts(prefix, b"BAI\x01"));
    let n_ref = le32(prefix, 4)?; // @4: u32 LE n_ref
    require!(n_ref <= MAX_N_REF, 8 + 8 * n_ref as u64 <= size);
    if n_ref > 0
        && let Some(n_bin) = le32(prefix, 8)
    {
        require!(n_bin <= MAX_BAI_BINS); // @8: u32 LE n_bin of reference 0
    }
    Some(Label::Bai)
}

/// Bethesda Game Studios Elder Scrolls / Fallout / Starfield Plugin (`.esp` / `.esm` / `.esl`, `"TES4"` / `"TES3"`).
pub(crate) fn verify_bethesda_plugin_esp(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_BETHESDA_ESP_SIZE, prefix.len() >= 32);
    if starts(prefix, b"TES4") {
        let data_size = le32(prefix, 4)? as u64;
        // Oblivion uses a 20-byte record header ("HEDR\x0C\x00" at @0x14);
        // Fallout 3/NV, Skyrim, Fallout 4, and Starfield use a 24-byte record header ("HEDR\x0C\x00" at @0x18).
        let hedr_off = if eq_at(prefix, 0x18, b"HEDR\x0C\x00") {
            0x18usize
        } else if eq_at(prefix, 0x14, b"HEDR\x0C\x00") {
            0x14usize
        } else {
            return None;
        };
        require!(
            data_size >= 18,
            (hedr_off as u64).checked_add(data_size)? <= size,
            prefix.len() >= hedr_off + 18
        );
        let ver_bits = le32(prefix, hedr_off + 6)?;
        let ver = f32::from_bits(ver_bits);
        require!(ver.is_finite() && (0.5..=3.0).contains(&ver));
        return Some(Label::BethesdaEsp);
    }
    if starts(prefix, b"TES3") {
        let data_size = le32(prefix, 4)? as u64;
        require!(
            data_size >= 306,
            16u64.checked_add(data_size)? <= size,
            eq_at(prefix, 0x10, b"HEDR\x2C\x01\x00\x00")
        );
        return Some(Label::BethesdaEsp);
    }
    None
}

/// BeyondTrust / Bomgar Remote Support Binary Resource Definition (`BRDF`, `.rdf`).
pub(crate) fn verify_beyondtrust_brdf_rdf(prefix: &[u8], size: u64) -> Option<Label> {
    const BRDF_HDR: &[u8; 16] = hex2bin!("42 52 44 46 80 00 0A 91 01 81 00 82 00 00 00 00");

    require!(
        size >= MIN_BEYONDTRUST_BRDF_RDF_SIZE,
        prefix.len() >= 23,
        starts(prefix, BRDF_HDR),
        u8_at(prefix, 20)? == 0x91,
        u8_at(prefix, 22)? == 0x83
    );
    Some(Label::BeyondtrustBrdfRdf)
}

/// BioWare Infinity / Aurora / Odyssey Engine Key Index (`.key`).
///
/// Header layout:
/// - `@0..8`: `"KEY V1  "` (Infinity / Aurora / KotOR) or `"KEY V1.1"` (The Witcher 1)
/// - `@8..12`: `u32 LE` `biff_count` (`1..=100_000`)
/// - `@12..16`: `u32 LE` `key_count` (`<= 10_000_000`)
/// - `@16..20`: `u32 LE` `biff_table_offset` (`>= 24`, `< file_size`)
pub(crate) fn verify_bioware_key(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC_V1: &[u8; 8] = b"KEY V1  "; // @0..8: Infinity / Aurora / Odyssey KEY V1
    const MAGIC_V11: &[u8; 8] = b"KEY V1.1"; // @0..8: The Witcher 1 KEY V1.1
    const BIFF_COUNT_OFF: usize = 8; // @8..12: u32 LE BIFF file count
    const KEY_COUNT_OFF: usize = 12; // @12..16: u32 LE resource key count
    const BIFF_OFF_OFF: usize = 16; // @16..20: u32 LE offset to BIFF entry table

    require!(
        size >= MIN_BIOWARE_KEY_SIZE,
        prefix.len() >= 24,
        starts(prefix, MAGIC_V1) || starts(prefix, MAGIC_V11),
        (1..=100_000).contains(&le32(prefix, BIFF_COUNT_OFF)?),
        le32(prefix, KEY_COUNT_OFF)? <= 10_000_000
    );
    let biff_off = le32(prefix, BIFF_OFF_OFF)? as u64;
    require!(biff_off >= 24, biff_off < size);
    Some(Label::BiowareKey)
}

/// BioWare Aurora / Odyssey / Eclipse / Mass Effect Talk Table (`.tlk`).
///
/// Supports:
/// - Aurora/Odyssey (`"TLK V3.0"` / `"TLK V4.0"`): `@8..12` `u32 LE` `language_id <= 256`,
///   `@16..20` `u32 LE` `string_entries_offset >= 20`.
/// - Mass Effect Huffman TLK (`"Tlk\0\x03\0\0\0"`): `@4..8` `u32 LE` `version == 3`,
///   `@8..12` `u32 LE` `min_version in 1..=10`.
pub(crate) fn verify_bioware_tlk(prefix: &[u8], size: u64) -> Option<Label> {
    const TLK_V30: &[u8; 8] = b"TLK V3.0"; // @0..8: Aurora / Odyssey / Witcher 1 TLK
    const TLK_V40: &[u8; 8] = b"TLK V4.0"; // @0..8: Dragon Age / Eclipse TLK
    const TLK_ME_V3: &[u8; 8] = bytes!(b"Tlk", &[0], le32_bytes!(3)); // @0..8: Mass Effect Huffman TLK v3
    const LANG_ID_OFF: usize = 8; // @8..12: u32 LE language ID (or min_version for ME)
    const STR_OFF_OFF: usize = 16; // @16..20: u32 LE string data offset

    require!(size >= MIN_BIOWARE_TLK_SIZE, prefix.len() >= 20);
    if starts(prefix, TLK_V30) || starts(prefix, TLK_V40) {
        let str_off = le32(prefix, STR_OFF_OFF)? as u64;
        require!(le32(prefix, LANG_ID_OFF)? <= 256, str_off >= 20, str_off <= size);
        return Some(Label::BiowareTlk);
    }
    if starts(prefix, TLK_ME_V3) {
        require!((1..=10).contains(&le32(prefix, LANG_ID_OFF)?));
        return Some(Label::BiowareTlk);
    }
    None
}

/// BitTorrent Speed / uTorrent Ledger Database (BTComm format 0, .btdb).
pub(crate) fn verify_bittorrent_btdb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BITTORRENT_BTDB_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"BTComm format 0\0\x10\0\x01\x01(@  ")
    );
    require!(has_non_ascii_text_byte(prefix.get(..28)?));
    Some(Label::BittorrentBtdb)
}

/// BoltDB / bbolt Go Key-Value Database (`.bolt` / `.bdb`, meta page magic `0xED0CDAED`).
pub(crate) fn verify_bolt_db(prefix: &[u8], size: u64) -> Option<Label> {
    const BOLT_DB_MAGIC: u32 = 0xED0C_DAED; // @16..20: meta page magic LE

    require!(
        size >= MIN_BOLT_DB_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 8)? == [0u8; 8], // @0..8: pgid 0 (meta page 0)
        le16(prefix, 8) == Some(4),           // @8..10: metaPageFlag (0x04)
        get_slice(prefix, 10, 16)? == [0u8; 6], // @10..16: count/overflow = 0
        le32(prefix, 16) == Some(BOLT_DB_MAGIC),
        le32(prefix, 20) == Some(2) // @20..24: BoltDB format version 2
    );
    let page_size = le32(prefix, 24)?;
    require!(
        (512..=65536).contains(&page_size),
        page_size.is_power_of_two(),
        get_slice(prefix, 28, 32)? == [0u8; 4]
    );
    Some(Label::BoltDb)
}

/// Borland Database Engine (BDE) Locale Driver (`.btl`, 1994) Borland.
pub(crate) fn verify_borland_bde_locale_btl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_BORLAND_BDE_LOCALE_BTL_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"BTLF01."),
        get_slice(prefix, 7, 16)?.contains(&0x00)
    );
    Some(Label::BorlandBdeLocaleBtl)
}

/// Caffeine.Scheme Database Schema (.cscheme).
pub(crate) fn verify_caffeine_cscheme(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CAFFEINE_CSCHEME_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"Caffeine.Scheme\0\x07\0\0\0")
    );
    Some(Label::CaffeineCscheme)
}

/// Capcom MT Framework GMD Localized String Table (`.gmd`, `"GMD\0"`, 2007 Capcom).
pub(crate) fn verify_capcom_mt_gmd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CAPCOM_MT_GMD_SIZE,
        prefix.len() >= 44,
        starts(prefix, CAPCOM_MT_GMD_MAGIC),
        le32(prefix, 4).is_some_and(|v| v > 0 && v <= 0x0003_FFFF),
        le32(prefix, 20)? >= 1,
        le32(prefix, 24)? >= 1
    );
    let name_len = le32(prefix, 36)? as usize;
    require!(
        (1..=256).contains(&name_len),
        40 + name_len <= prefix.len(),
        get_slice(prefix, 40, 40 + name_len)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::CapcomMtGmd)
}

/// YACU Data Record (.cd).
pub(crate) fn verify_cdtree_cd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CDTREE_CD_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"YACU\x02\0\x01\0\x03\0"),
        get_slice(prefix, 18, 24)? == b"\0\0\0\0\0\0"
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::CdtreeCd)
}

/// CC-CEDICT ASCII-0x1F Unit-Delimited Chinese Dictionary (.u8).
pub(crate) fn verify_cedict_unit_u8(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CEDICT_UNIT_U8_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 32)?
            == hex2bin!(
                "E5 91 B5 1F 61 31 0A 1F 28 70 68 6F 6E 65 74 69 63 20 70 61 72 74 69 63 6C 65 29 0A E9 98 BF 1F"
            )
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::CedictUnitU8)
}

/// CERNLIB ZEBRA RZ Direct-Access File / HBOOK / DELPHI CARGO Database (`.rz` / `.hbook` / `.dat`, 1985 CERN).
///
/// Record 1 header layout (16 32-bit words at `@0..64`, native LE or BE):
/// - `w[0] == 1` (`NREC1`, first directory record index)
/// - `w[2] == 0`, `w[3] == 1`, `w[4] == 1`, `w[5] == 25` (`0x19` root directory metadata pointer)
/// - `w[6]` in `26..=256`, `w[7..=11] == 0`, `w[12] == 2`
/// - `w[13]` (`NREC` in `1..=10_000_000`), `w[14]` (`LREC`, power of 2 in `256..=8192`), `w[15]` in `1..=16`
pub(crate) fn verify_cern_zebra_rz(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_BYTES: usize = 64; // @0..64: 16 32-bit control words in RZ Record 1

    require!(size >= MIN_CERN_ZEBRA_RZ_SIZE, prefix.len() >= HDR_BYTES);
    let is_be = if le32(prefix, 0) == Some(1) {
        false
    } else if be32(prefix, 0) == Some(1) {
        true
    } else {
        return None;
    };
    let word = |idx: usize| -> Option<u32> {
        let off = idx * 4;
        if is_be { be32(prefix, off) } else { le32(prefix, off) }
    };
    require!(
        word(0)? == 1,
        word(2)? == 0,
        word(3)? == 1,
        word(4)? == 1,
        word(5)? == 25,
        (26..=256).contains(&word(6)?),
        word(7)? == 0,
        word(8)? == 0,
        word(9)? == 0,
        word(10)? == 0,
        word(11)? == 0,
        word(12)? == 2,
    );
    let nrec = word(13)?;
    let lrec = word(14)?;
    let nrec_dir = word(15)?;
    require!(
        (1..=10_000_000).contains(&nrec),
        lrec.is_power_of_two(),
        (256..=8192).contains(&lrec),
        (1..=16).contains(&nrec_dir),
        size >= (lrec as u64) * 4,
    );
    Some(Label::CernZebraRz)
}

/// CHAPS TimeTable / TimeTable Extension Transit Schedule Database (`.tt`, CHAPS s.r.o.).
pub(crate) fn verify_chaps_timetable_tt(prefix: &[u8], size: u64) -> Option<Label> {
    const TIMETABLE_MAGIC: &[u8; 26] = b" TT | TimeTable | Version "; // @0..26: base TimeTable header
    const TIMETABLE_EXT_MAGIC: &[u8; 36] = b" TT | TimeTable Extension | Version "; // @0..36: Extension TimeTable header

    require!(
        size >= MIN_CHAPS_TIMETABLE_TT_SIZE,
        prefix.len() >= 80,
        starts(prefix, TIMETABLE_MAGIC) || starts(prefix, TIMETABLE_EXT_MAGIC),
        has_non_ascii_text_byte(prefix.get(..prefix.len().min(96))?)
    );
    Some(Label::ChapsTimetableTt)
}

/// ChessBase Opening Book Tree Table (.ctb).
pub(crate) fn verify_chessbase_tree_ctb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CHESSBASE_TREE_CTB_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 5)? == hex2bin!("00 0C 00 01 00"),
        get_slice(prefix, 8, 9)? == hex2bin!("00"),
        get_slice(prefix, 10, 11)? == hex2bin!("FF"),
        get_slice(prefix, 12, 32)?
            == hex2bin!("FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::ChessbaseTreeCtb)
}

/// Google Chrome Compiled Hunspell Spellcheck Dictionary (.bdic).
pub(crate) fn verify_chrome_bdic(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_CHROME_BDIC_SIZE && prefix.len() >= 20 && starts(prefix, b"BDic"));
    let version = le32(prefix, 4)?;
    let aff_offset = le32(prefix, 8)? as u64;
    let dic_offset = le32(prefix, 12)? as u64;
    require!(
        (1..=4).contains(&version)
            && aff_offset >= 16
            && dic_offset > aff_offset
            && dic_offset < size
    );
    Some(Label::ChromeBdic)
}

/// Google Chrome Safe Browsing V4 Store (`.store`, `kFileMagic = 0x600D71FE` LE = `FE 71 0D 60`).
pub(crate) fn verify_chrome_safebrowsing_store(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CHROME_V4STORE_SIZE,
        prefix.len() >= 16,
        starts(prefix, &[0xFE, 0x71, 0x0D, 0x60])
    );
    let version = le32(prefix, 4)?;
    let proto_size = le32(prefix, 8)? as u64;
    let valid_proto_size = (1..=size.saturating_sub(12)).contains(&proto_size)
        || matches!(u8_at(prefix, 8)?, 0x08 | 0x0A);
    require!((1..=16).contains(&version), valid_proto_size);
    Some(Label::ChromeV4store)
}

/// Google Chrome / Chromium Visited Links History (`"VLnk"`).
pub(crate) fn verify_chrome_visited_links(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_CHROME_VLNK_SIZE, prefix.len() >= 24, starts(prefix, b"VLnk"));
    let version = le32(prefix, 4)?;
    let table_len = le32(prefix, 8)?;
    let used_items = le32(prefix, 12)?;
    require!(
        (1..=4).contains(&version),
        (1..=16_777_216).contains(&table_len),
        used_items <= table_len,
        size >= 24 + (table_len as u64).min(1) * 8
    );
    Some(Label::ChromeVlnk)
}

/// Chromium Disk Cache Blockfile or Index (`data_0`..`data_3`, `index`, 2008 Google).
pub(crate) fn verify_chromium_cache_blockfile(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_CHROMIUM_CACHE_BLOCKFILE_SIZE && prefix.len() >= 32);
    let magic = le32(prefix, 0)?;
    let ver = le32(prefix, 4)?;
    require!(
        matches!(magic, 0xC104_CAC3 | 0xC103_CAC3)
            && matches!(ver, 0x0001_0000..=0x0001_0004 | 0x0002_0000 | 0x0002_0001 | 0x0003_0000)
    );
    Some(Label::ChromeCacheBlockfile)
}

/// Chromium Simple Cache Entry (`*_0`..`*_9`), top-level `index`, or `the-real-index` (`"oy retne"`, 2013 Google).
pub(crate) fn verify_chromium_simple_cache(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= 24 && prefix.len() >= 24 && eq_at(prefix, 0, hex2bin!("30 5C 72 A7 1B 6D FB FC")) {
        let ver = le32(prefix, 8)?;
        let key_len = le32(prefix, 12)? as u64;
        let is_top_index = size == 24 && key_len == 0 && le32(prefix, 16) == Some(0);
        let is_stream_entry = size >= 44
            && prefix.len() >= 28
            && (1..=65536).contains(&key_len)
            && 24 + key_len <= size;
        require!((1..=12).contains(&ver) && (is_top_index || is_stream_entry));
        return Some(Label::ChromeSimpleCache);
    }
    if size >= 40 && prefix.len() >= 20 && eq_at(prefix, 8, b"oy retne") {
        let payload_size = le32(prefix, 0)? as u64;
        let ver = le32(prefix, 16)?;
        require!(payload_size >= 32, 8 + payload_size <= size, (1..=16).contains(&ver));
        return Some(Label::ChromeSimpleCache);
    }
    if size >= 32 && prefix.len() >= 12 && starts(prefix, b"oy retne") {
        let ver = le32(prefix, 8)?;
        require!((1..=16).contains(&ver));
        return Some(Label::ChromeSimpleCache);
    }
    None
}

/// Barrington Systems Clarion for DOS Memo File (`.mem`, `"M3"`).
///
/// Layout: 6-byte header (`"M3"` signature + `u32 LE firstdel` 1-based free-list head, `0` if
/// none) followed by `N >= 1` fixed 256-byte blocks, each starting with `u32 LE nxtblk`
/// (`0..=N`, no self-loop) and 252 bytes of DOS text memo payload (`size == 6 + 256 * N`).
pub(crate) fn verify_clarion_mem(prefix: &[u8], size: u64) -> Option<Label> {
    const HDR_LEN: u64 = 6;
    const BLOCK_SIZE: u64 = 256;

    require!(
        size >= MIN_CLARION_MEM_SIZE,
        (size - HDR_LEN).is_multiple_of(BLOCK_SIZE),
        prefix.len() >= MIN_CLARION_MEM_SIZE as usize,
        starts(prefix, b"M3")
    );
    let num_blocks = (size - HDR_LEN) / BLOCK_SIZE;
    require!(num_blocks <= u32::MAX as u64);

    let firstdel = le32(prefix, 2)? as u64; // @2..6: 1-based first deleted block (0 = none)
    require!(firstdel <= num_blocks);

    // Number of blocks whose 4-byte nxtblk header fits in prefix: 6 + 256*(k-1) + 4 <= prefix.len().
    let vis_blocks =
        ((prefix.len() - HDR_LEN as usize + 252) / BLOCK_SIZE as usize).min(num_blocks as usize);
    require!(vis_blocks >= 1);
    for i in 0..vis_blocks {
        let blk_off = HDR_LEN as usize + i * BLOCK_SIZE as usize;
        let nxtblk = le32(prefix, blk_off)? as u64;
        let blk_id = (i + 1) as u64;
        require!(nxtblk <= num_blocks, nxtblk != blk_id);
    }

    // Walk the visible prefix of the free list starting at firstdel to reject pointer cycles.
    let mut cur = firstdel;
    for _ in 0..vis_blocks {
        if cur == 0 || cur > vis_blocks as u64 {
            break;
        }
        let blk_off = HDR_LEN as usize + (cur as usize - 1) * BLOCK_SIZE as usize;
        cur = le32(prefix, blk_off)? as u64;
        require!(cur != firstdel);
    }
    require!(cur == 0 || cur > vis_blocks as u64);

    // Block 1 payload (@10..262): Clarion DOS memos hold DOS text (printable ASCII, CP437 >= 0x80,
    // NUL/tab/CR/LF); require at least one printable ASCII byte and no binary C0 control bytes.
    let payload = get_slice(prefix, 10, 262)?;
    let mut has_printable = false;
    for &b in payload {
        if (0x20..=0x7E).contains(&b) {
            has_printable = true;
        } else if (b < 0x20 && !matches!(b, 0x00 | b'\t' | b'\n' | b'\r')) || b == 0x7F {
            return None;
        }
    }
    require!(has_printable);

    Some(Label::ClarionMem)
}

/// Nantucket / CA-Clipper NTX Database Index (`.ntx`, 1987, Nantucket / Computer Associates).
///
/// Validates the 1024-byte page-aligned B-tree header across Summer '87 (`0x0003`), Clipper 5.x
/// (`0x0006`), Huge-file (`0x0026`), and Clipper 5.3 (`0x0106`) signatures, accepting any non-zero
/// `index_version`, 1024-byte-aligned `first_node` / `next_unused_page`, `item_size == key_size + 8`,
/// `half_page == max_item / 2`, and printable ASCII index key expression at `@22`.
pub(crate) fn verify_clipper_ntx(prefix: &[u8], size: u64) -> Option<Label> {
    const VER_OFF: usize = 2; // @2..4: u16 LE index version / modification counter (>= 1)
    const ROOT_OFF: usize = 4; // @4..8: u32 LE root page offset (1024-byte aligned, >= 1024)
    const NEXT_UNUSED_OFF: usize = 8; // @8..12: u32 LE next unused page offset (0 or >= 1024, % 1024 == 0)
    const ITEM_SIZE_OFF: usize = 12; // @12..14: u16 LE B-tree item size (key_size + 8)
    const KEY_SIZE_OFF: usize = 14; // @14..16: u16 LE key size (1..=256)
    const KEY_DEC_OFF: usize = 16; // @16..18: u16 LE key decimal places (<= 32)
    const MAX_ITEM_OFF: usize = 18; // @18..20: u16 LE max items per page (even, 2..=512)
    const HALF_PAGE_OFF: usize = 20; // @20..22: u16 LE half_page == max_item / 2
    const EXPR_OFF: usize = 22; // @22: first byte of NUL-terminated ASCII key expression

    require!(
        size >= MIN_CLIPPER_NTX_SIZE,
        prefix.len() >= 36,
        starts(prefix, CLIPPER_NTX_MAGIC_CLIPPER5)
            || starts(prefix, CLIPPER_NTX_MAGIC_S87)
            || starts(prefix, CLIPPER_NTX_MAGIC_HUGE)
            || starts(prefix, CLIPPER_NTX_MAGIC_V53)
    );
    let version = le16(prefix, VER_OFF)?;
    let root = le32(prefix, ROOT_OFF)?;
    let next_unused = le32(prefix, NEXT_UNUSED_OFF)?;
    let item_size = le16(prefix, ITEM_SIZE_OFF)?;
    let key_size = le16(prefix, KEY_SIZE_OFF)?;
    let key_dec = le16(prefix, KEY_DEC_OFF)?;
    let max_item = le16(prefix, MAX_ITEM_OFF)?;
    let half_page = le16(prefix, HALF_PAGE_OFF)?;
    let expr0 = u8_at(prefix, EXPR_OFF)?;
    require!(
        version >= 1,
        root >= 1024 && root % 1024 == 0,
        next_unused == 0 || (next_unused >= 1024 && next_unused % 1024 == 0),
        (1..=256).contains(&key_size),
        item_size == key_size.checked_add(8)?,
        key_dec <= 32,
        (2..=512).contains(&max_item),
        half_page >= 1 && half_page == max_item / 2,
        (0x20..=0x7E).contains(&expr0)
    );
    Some(Label::ClipperNtx)
}

/// CONS+ DICT(HUGE) Dictionary Database (.nx2).
pub(crate) fn verify_cons_dict_nx2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_CONS_DICT_NX2_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"\x42\x81\xf5\0\0\0"),
        get_slice(prefix, 16, 33)? == b"CONS+ DICT(HUGE)\0"
    );
    Some(Label::ConsDictNx2)
}

/// ConsultantPlus Information Bank Database (`.dt5`).
pub(crate) fn verify_consultantplus_dt5(prefix: &[u8], size: u64) -> Option<Label> {
    const DT5_MAGIC: &[u8; 6] = hex2bin!("D2 00 D2 00 00 00"); // @0..6: ConsultantPlus DT5 signature
    const NAME_START: usize = 8; // @8..40: NUL-padded ASCII `<BANK>.DT5` filename
    const NAME_END: usize = 40; // @40..42: CRLF before CP1251 bank title
    const CRLF: &[u8; 2] = b"\r\n";

    require!(
        size >= MIN_CONSULTANTPLUS_DT5_SIZE,
        prefix.len() >= 42,
        starts(prefix, DT5_MAGIC),
        eq_at(prefix, NAME_END, CRLF)
    );
    let name_field = get_slice(prefix, NAME_START, NAME_END)?;
    let nul_pos = name_field.iter().position(|&b| b == 0)?;
    require!(
        (5..=24).contains(&nul_pos),
        name_field[nul_pos - 4..nul_pos].eq_ignore_ascii_case(b".DT5"),
        name_field[..nul_pos - 4]
            .iter()
            .all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
        name_field[nul_pos..].iter().all(|&b| b == 0)
    );
    Some(Label::ConsultantplusDt5)
}

/// Databento Binary Encoding Market Data Stream (`.dbn`, `"DBN"` v1–v3).
pub(crate) fn verify_databento_dbn_v1(prefix: &[u8], size: u64) -> Option<Label> {
    const DBN_MAGIC_PREFIX: &[u8; 3] = b"DBN"; // @0..3

    require!(
        size >= MIN_DATABENTO_DBN_V1_SIZE,
        prefix.len() >= 24,
        starts(prefix, DBN_MAGIC_PREFIX),
        (1..=3).contains(&u8_at(prefix, 3)?), // @3: DBN format version
        (120..=0x0100_0000).contains(&le32(prefix, 4)?)  // @4..8: metadata length
    );
    let ds = get_slice(prefix, 8, 24)?; // @8..24: NUL-padded ASCII dataset identifier (e.g. "GLBX.MDP3")
    let ds_len = ds.iter().position(|&b| b == 0)?;
    require!(
        ds_len >= 1,
        ds[..ds_len].iter().all(|&b| matches!(b, b'A'..=b'Z' | b'0'..=b'9' | b'.' | b'_' | b'-')),
        ds[ds_len..].iter().all(|&b| b == 0)
    );
    Some(Label::DatabentoDbn)
}

/// Ashton-Tate dBASE IV Generic Label / Report Design File (`"dBASE IV Generic Design File"`).
pub(crate) fn verify_dbase_lbl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_DBASE_LBL_SIZE, starts(prefix, b"dBASE IV Generic Design File"));
    Some(Label::DbaseLbl)
}

/// Borland / Embarcadero Delphi ClientDataSet Binary Packet (MIDAS / DataSnap `.cds` / `.rds`).
pub(crate) fn verify_delphi_cds(prefix: &[u8], size: u64) -> Option<Label> {
    const DELPHI_CDS_HDR_PREFIX: &[u8; 12] = hex2bin!("96 19 E0 BD 01 00 00 00 18 00 00 00"); // @0..12: magic 0xBDE01996 + version 1 + header_size 24
    const COL_COUNT_OFF: usize = 12; // @12..14: u16 LE column count
    const PROPS_COUNT_OFF: usize = 18; // @18..22: u32 LE optional parameter count
    const META_SIZE_OFF: usize = 22; // @22..24: u16 LE header + schema metadata byte length
    const FIRST_COL_LEN_OFF: usize = 24; // @24: length-prefixed first column name

    require!(
        size >= MIN_DELPHI_CDS_SIZE,
        prefix.len() >= 32,
        starts(prefix, DELPHI_CDS_HDR_PREFIX)
    );
    let col_count = le16(prefix, COL_COUNT_OFF)?;
    let props_count = le32(prefix, PROPS_COUNT_OFF)?;
    let meta_size = le16(prefix, META_SIZE_OFF)?;
    let first_name_len = u8_at(prefix, FIRST_COL_LEN_OFF)? as usize;
    require!(
        col_count >= 1,
        props_count <= 0xFFFF,
        meta_size >= 24,
        (1..=64).contains(&first_name_len),
        prefix.len() >= FIRST_COL_LEN_OFF + 1 + first_name_len,
        get_slice(prefix, FIRST_COL_LEN_OFF + 1, FIRST_COL_LEN_OFF + 1 + first_name_len)?
            .iter()
            .all(|&b| b.is_ascii_graphic() || b == b' ')
    );
    Some(Label::DelphiCds)
}

/// DTCH Song Chart Database (.dtc).
pub(crate) fn verify_dtch_chart(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_DTCH_CHART_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"DTCH\x04\0"),
        get_slice(prefix, 6, 16)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
    );
    Some(Label::DtchChart)
}

/// EPLAN Electric P8 / EEC Engineering Object Database (.eox).
pub(crate) fn verify_eplan_eox_db(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EPLAN_EOX_DB_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 9)? == hex2bin!("FE 53 02 02 04 04 0F FF 00"),
        get_slice(prefix, 10, 13)? == hex2bin!("07 04 00"),
        get_slice(prefix, 15, 32)?
            == hex2bin!("00 00 00 02 00 00 00 00 00 00 00 00 00 00 00 00 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::EplanEoxDb)
}

/// Ethereum Era1 / e2store History Archive (`.era1` / `.era` / `.e2s`).
///
/// Header layout:
/// - `@0..8`: `65 32 00 00 00 00 00 00` (`u16 LE type = 0x3265` `"e2"`, `u32 LE length = 0`, `u16 LE reserved = 0`)
/// - `@8..10`: `u16 LE second_type` (`0x0001..=0x0010`, `0x3266`CompressedHeader, `0x3269` EpochIndex, etc.)
/// - `@10..14`: `u32 LE second_length` (`> 0` and `16 + second_length <= size`)
/// - `@14..16`: `u16 LE second_reserved` (`0`)
pub(crate) fn verify_ethereum_era1(prefix: &[u8], size: u64) -> Option<Label> {
    const ERA1_VERSION_TLV: &[u8; 8] = bytes!(b"e2", le16_bytes!(0), le32_bytes!(0)); // @0..8: e2store Version TLV
    const NEXT_TYPE_OFFSET: usize = 8; // @8..10: u16 LE second TLV type
    const NEXT_LEN_OFFSET: usize = 10; // @10..14: u32 LE second TLV payload length
    const NEXT_RES_OFFSET: usize = 14; // @14..16: u16 LE second TLV reserved (0)

    require!(
        size >= MIN_ETHEREUM_ERA1_SIZE,
        prefix.len() >= 16,
        starts(prefix, ERA1_VERSION_TLV),
        le16(prefix, NEXT_RES_OFFSET) == Some(0)
    );
    let next_type = le16(prefix, NEXT_TYPE_OFFSET)?;
    let next_len = le32(prefix, NEXT_LEN_OFFSET)? as u64;
    require!(
        matches!(next_type, 1..=32 | 0x3265..=0x326F | 0x0100..=0x010F),
        next_len > 0,
        16u64.saturating_add(next_len) <= size
    );
    Some(Label::EthereumEra1)
}

/// FPL Railway Timetable Route (.fpl).
pub(crate) fn verify_fbs_timetable_fpl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FBS_TIMETABLE_FPL_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"\x31\x01\x01\x01"),
        get_slice(prefix, 6, 10)? == b"\x03\xc6\x04\0",
        (4..=64).contains(&u8_at(prefix, 10)?)
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::FbsTimetableFpl)
}

/// Microsoft Visual FoxPro 3.0 Form/Class Memo (.sct / .vct).
pub(crate) fn verify_foxpro_form_sct(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_FOXPRO_FORM_SCT_SIZE,
        prefix.len() >= 536,
        get_slice(prefix, 4, 8)? == hex2bin!("00 00 00 01"),
        get_slice(prefix, 512, 536)? == b"\0\0\0\x01\0\0\0\x10VERSION =   3.00"
    );
    require!(has_non_ascii_text_byte(prefix.get(..536)?));
    Some(Label::FoxproFormSct)
}

/// NSA Ghidra Unpacked BufferFile (`.bf` / `.gbf`, `LocalBufferFile.MAGIC_NUMBER` `"/01,4),*"` + `fileId(8)` + `be32(16) == 1` + `blockSize(20) in 512..=65536`).
pub(crate) fn verify_ghidra_buffer_file(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_GHIDRA_BF_SIZE
        && prefix.len() >= 24
        && starts(prefix, b"/01,4),*")
        && be32(prefix, 16) == Some(1)
        && matches!(be32(prefix, 20), Some(512..=65536))
    {
        return Some(Label::GhidraBf);
    }
    require!(
        size >= MIN_GHIDRA_BF_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"/01*49,+") || starts(prefix, b"/01,4),*"),
        be32(prefix, 8) == Some(1),
        (512..=65536).contains(&be32(prefix, 12)?)
    );
    Some(Label::GhidraBf)
}

/// Git Packfile Reverse Index (.rev, 2021) Git Project.
pub(crate) fn verify_git_pack_rev(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GIT_PACK_REV_SIZE,
        prefix.len() >= 52,
        starts(prefix, b"RIDX\0\0\0\x01"),
        matches!(be32(prefix, 8), Some(1 | 2))
    );
    Some(Label::GitPackRev)
}

/// GnuPG Trust Database (`trustdb.gpg`, 1998 Werner Koch / Free Software Foundation).
pub(crate) fn verify_gnupg_trustdb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GNUPG_TRUSTDB_SIZE
            && size.is_multiple_of(40)
            && prefix.len() >= 40
            && starts(prefix, b"\x01gpg")
            && matches!(u8_at(prefix, 4)?, 2 | 3)
    );
    Some(Label::GpgTrustdb)
}

/// GSDD Display Descriptor Database (.gdd).
pub(crate) fn verify_gsdd_gdd(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GSDD_GDD_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"GSDDDgdt\0\0\0d"),
        get_slice(prefix, 20, 24)? == b"Dgcd"
    );
    Some(Label::GsddGdd)
}

/// David Jurgens HelpPC 2.10 Quick Reference Index (`.ndx`, 1991).
///
/// Header layout:
/// - `@0..8`: `b" HelpPC "` (beginning of an 80-byte banner line)
/// - `@80..85`: `0D 0A 00 1A 09` (CR LF NUL EOF + 0x09 tag)
/// - `@85`: `u8 num_files` (`1..=16`)
/// - `@86..88`: `u16 LE num_topics` (`>= 1`)
/// - `@98..1154`: 16 fixed 66-byte indexed `.TXT` file slots, followed by packed topic records.
pub(crate) fn verify_helppc_index(prefix: &[u8], size: u64) -> Option<Label> {
    const TERM_OFF: usize = 80; // @80..85: \r\n\0\x1A\x09
    const NUM_FILES_OFF: usize = 85; // @85: u8 file slot count (1..=16)
    const NUM_TOPICS_OFF: usize = 86; // @86..88: u16 LE indexed topic count (>= 1)
    require!(
        size >= MIN_HELPPC_INDEX_SIZE,
        prefix.len() >= NUM_TOPICS_OFF + 2,
        starts(prefix, HELPPC_INDEX_BANNER_PREFIX),
        eq_at(prefix, TERM_OFF, HELPPC_INDEX_TERM),
        (1..=16).contains(&u8_at(prefix, NUM_FILES_OFF)?),
        le16(prefix, NUM_TOPICS_OFF)? >= 1
    );
    Some(Label::HelppcIndex)
}

/// Helsinki Finite-State Transducer Optimized Lookup (`.hfstol`, `"HFST\0"`).
pub(crate) fn verify_hfst_ol_transducer(prefix: &[u8], size: u64) -> Option<Label> {
    const HFST_MAGIC: &[u8; 5] = b"HFST\0"; // @0..5
    const VERSION_KEY: &[u8; 8] = b"version\0";
    const HFST_OL_TAG: &[u8; 7] = b"HFST_OL";

    require!(size >= MIN_HFST_OL_TRANSDUCER_SIZE, prefix.len() >= 32, starts(prefix, HFST_MAGIC));
    let hdr_len = le16(prefix, 5)? as usize;
    require!((16..=4096).contains(&hdr_len), u8_at(prefix, 7)? == 0);
    let search_end = prefix.len().min(8 + hdr_len).min(256);
    let hdr = get_slice(prefix, 8, search_end)?;
    hdr.windows(VERSION_KEY.len()).position(|w| w == VERSION_KEY)?;
    hdr.windows(HFST_OL_TAG.len()).position(|w| w == HFST_OL_TAG)?;
    Some(Label::HfstOlTransducer)
}

/// Index Education HYPERPLANNING Schedule Database (`.hyp`).
pub(crate) fn verify_hyperplanning_hyp(prefix: &[u8], size: u64) -> Option<Label> {
    const HYPERPLANNING_PREFIX: &[u8; 2] = hex2bin!("48 08"); // @0..2
    const HYPERPLANNING_TAG: &[u8; 14] = b"\x0dHYPERPLANNING"; // @4..18

    require!(
        size >= MIN_HYPERPLANNING_HYP_SIZE,
        prefix.len() >= 32,
        starts(prefix, HYPERPLANNING_PREFIX),
        eq_at(prefix, 4, HYPERPLANNING_TAG),
        get_slice(prefix, 18, 32)? == [0u8; 14]
    );
    Some(Label::HyperplanningHyp)
}

/// IBus GVariant Emoji / Unicode Dictionary file (`.dict`).
pub(crate) fn verify_ibus_dict(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_IBUS_DICT_SIZE, prefix.len() >= 32, starts(prefix, IBUS_EMOJI_MAGIC));
    Some(Label::IbusDict)
}

/// Hex-Rays IDA Pro Disassembly Database (.idb / .i64).
pub(crate) fn verify_ida_database(prefix: &[u8], size: u64) -> Option<Label> {
    if size < MIN_IDA_DATABASE_SIZE || prefix.len() < 32 || le16(prefix, 4) != Some(0) {
        return None;
    }
    let sig = le32(prefix, 26);
    let ver = le16(prefix, 30).unwrap_or(0);
    let valid = if starts(prefix, b"IDA0") && sig == Some(0) {
        // Pre-5.0 IDA0 header: 5 monotonic u32 LE section offsets at 6..26, 0x00000000 at 26..30
        let id0_off = le32(prefix, 6).unwrap_or(0) as u64;
        let id1_off = le32(prefix, 10).unwrap_or(0) as u64;
        (22..=4096).contains(&id0_off) && id0_off < size && id1_off > id0_off
    } else if sig == Some(0xAABB_CCDD) {
        if (1..=6).contains(&ver) {
            true
        } else if starts(prefix, b"IDA2") && (900..=1200).contains(&ver) {
            // IDA 9.1+ V910 (0x038E = 910) inline sections header: compression (0=None, 2=Zlib, 3=Zstd) at 32
            let hdr_size = le64(prefix, 6).unwrap_or(0);
            let data_start = le64(prefix, 14).unwrap_or(0);
            prefix.len() >= 33
                && matches!(u8_at(prefix, 32)?, 0 | 2 | 3)
                && hdr_size >= 64
                && data_start >= hdr_size
        } else {
            false
        }
    } else if le32(prefix, 30) == Some(0xAABB_CCDD) {
        prefix.len() < 36 || (1..=6).contains(&le16(prefix, 34).unwrap_or(1))
    } else {
        prefix.len() >= 42 && le32(prefix, 38) == Some(0xAABB_CCDD)
    };
    require!(valid);
    Some(Label::Idb)
}

/// Hex-Rays IDA Pro Type Information Library (.til).
pub(crate) fn verify_ida_til(prefix: &[u8], size: u64) -> Option<Label> {
    if size < MIN_IDA_TIL_SIZE || prefix.len() < 16 || !starts(prefix, b"IDATIL") {
        return None;
    }
    let format = le32(prefix, 6).unwrap_or(0);
    let valid_desc = if (1..=15).contains(&format) {
        is_valid_til_desc(prefix, 10)
    } else if (16..=30).contains(&format) {
        (le16(prefix, 12) == Some(0) && is_valid_til_desc(prefix, 14))
            || is_valid_til_desc(prefix, 10)
    } else {
        false
    };
    require!(valid_desc);
    Some(Label::Idatil)
}

/// Interactive Delphi Reconstructor Knowledge Base (`.kb`, 2009 crypto2011).
///
/// Header layout:
/// - `@0..24`: `"IDR Knowledge Base File\0"` paired with `@289..293 u32 LE == 2`, or
///   `"IDD Knowledge Base File\0"` paired with `@289..293 u32 LE == 1`.
/// - `@24`: `_isMSIL` flag in `0..=1`.
pub(crate) fn verify_idr_kb(prefix: &[u8], size: u64) -> Option<Label> {
    const IS_MSIL_OFF: usize = 24; // @24: u8 _isMSIL boolean (0 = native Win32, 1 = .NET MSIL)
    const KB_VER_OFF: usize = 289; // @289: u32 LE kbVer (2 for IDR, 1 for IDD)

    require!(
        size >= MIN_IDR_KB_SIZE,
        prefix.len() >= KB_VER_OFF + 4,
        u8_at(prefix, IS_MSIL_OFF)? <= 1
    );
    let kb_ver = le32(prefix, KB_VER_OFF)?;
    let valid_pair =
        (starts(prefix, IDR_KB_SIG) && kb_ver == 2) || (starts(prefix, IDD_KB_SIG) && kb_ver == 1);
    require!(valid_pair);

    Some(Label::IdrKb)
}

/// NNG iGO Navigation KML POI Search Index (`.kns`).
pub(crate) fn verify_igo_poi_kns(prefix: &[u8], size: u64) -> Option<Label> {
    const IGO_KNS_MAGIC: &[u8; 14] = hex2bin!("01 00 00 00 00 00 0E 00 00 00 FF FF FF 7F"); // @0..14: iGO KNS index magic + sentinel
    const NAME_LEN_OFF: usize = 14; // @14..18: u32 LE UTF-16LE category name length (code units)
    const VERSION_OFF: usize = 18; // @18..22: u32 LE format version (1)
    const ENTRY_CNT_OFF: usize = 22; // @22..26: u32 LE indexed POI entry count
    const NAME_OFF: usize = 26; // @26..: UTF-16LE NUL-terminated POI category name

    require!(size >= MIN_IGO_POI_KNS_SIZE, prefix.len() >= 28, starts(prefix, IGO_KNS_MAGIC));
    let name_len = le32(prefix, NAME_LEN_OFF)? as u64;
    let version = le32(prefix, VERSION_OFF)?;
    let entry_cnt = le32(prefix, ENTRY_CNT_OFF)?;
    let first_ch = le16(prefix, NAME_OFF)?;
    require!(
        (2..=256).contains(&name_len),
        version == 1,
        (1..=1_000_000).contains(&entry_cnt),
        (NAME_OFF as u64).checked_add(name_len.checked_mul(2)?)? <= size,
        first_ch != 0
    );
    Some(Label::IgoPoiKns)
}

/// INFOTrieve Technologies TPX Record (.tpx).
pub(crate) fn verify_infotrieve_tpx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_INFOTRIEVE_TPX_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"\xc5\x04H\0INFOTrieve Technologies Inc.\0")
    );
    require!(has_non_ascii_text_byte(prefix.get(..40)?));
    Some(Label::InfotrieveTpx)
}

/// Inmagic DB/TextWorks database family file (`.acf`/`.btx`/`.dbo`/`.dbr`/`.dbs`/`.ixl`/`.occ`/`.sdo`/`.tba`/`.tbu`).
///
/// Unified 16-byte ASCII header + binary body (v3.0 & v7.0):
/// - `0..4`: 3-letter uppercase type tag + `' '` (`ACF `, `BTX `, `DBO `, `DBR `, `DBS `, `IXL `, `OCC `, `SDO `, `TBA `, `TBU `)
/// - `4..8`: 3-digit format revision (`001`..`999`) + `' '`
/// - `8..16`: creation/revision date `MM/DD/YY` (`01..=12`, `'/'`, `01..=31`, `'/'`, `00..=99`)
/// - `16..20`: binary LE16/LE32 header fields containing at least one non-ASCII/control byte
pub(crate) fn verify_inmagic_dbtextworks(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_INMAGIC_DBTEXTWORKS_SIZE, prefix.len() >= 20);
    require!(matches!(
        get_slice(prefix, 0, 4)?,
        b"ACF "
            | b"BTX "
            | b"DBO "
            | b"DBR "
            | b"DBS "
            | b"IXL "
            | b"OCC "
            | b"SDO "
            | b"TBA "
            | b"TBU "
    ));
    require!(
        u8_at(prefix, 4)?.is_ascii_digit(),
        u8_at(prefix, 5)?.is_ascii_digit(),
        u8_at(prefix, 6)?.is_ascii_digit(),
        u8_at(prefix, 7)? == b' ',
        u8_at(prefix, 10)? == b'/',
        u8_at(prefix, 13)? == b'/'
    );
    let parse_2d = |a: u8, b: u8| -> Option<u8> {
        if a.is_ascii_digit() && b.is_ascii_digit() {
            Some((a - b'0') * 10 + (b - b'0'))
        } else {
            None
        }
    };
    let mm = parse_2d(u8_at(prefix, 8)?, u8_at(prefix, 9)?)?;
    let dd = parse_2d(u8_at(prefix, 11)?, u8_at(prefix, 12)?)?;
    let _yy = parse_2d(u8_at(prefix, 14)?, u8_at(prefix, 15)?)?;
    require!(
        (1..=12).contains(&mm),
        (1..=31).contains(&dd),
        has_non_ascii_text_byte(prefix.get(16..20)?)
    );

    Some(Label::InmagicDbtext)
}

/// MySQL / MariaDB InnoDB tablespace (`.ibd` / `ibdata1` / `undo_*` / `mysql.ibd`,
/// 2001 Innobase Oy / Oracle MySQL), page 0 (all fields big-endian):
///
/// ```text
/// @0   u32 FIL_PAGE_SPACE_OR_CHKSUM   page checksum (any value)
/// @4   u32 FIL_PAGE_OFFSET            page number == 0
/// @8   u32 FIL_PAGE_PREV              MySQL 8.0+: FIL_PAGE_SRV_VERSION (80043 = 8.0.43); 5.x: unused
/// @12  u32 FIL_PAGE_NEXT              MySQL 8.0+: FIL_PAGE_SPACE_VERSION (1); 5.x: unused
/// @16  u64 FIL_PAGE_LSN               newest modification LSN (!= 0)
/// @24  u16 FIL_PAGE_TYPE              FIL_PAGE_TYPE_FSP_HDR (0x0008)
/// @26  u64 FIL_PAGE_FILE_FLUSH_LSN
/// @34  u32 FIL_PAGE_SPACE_ID          tablespace id
/// @38  u32 FSP_SPACE_ID               == FIL_PAGE_SPACE_ID
/// @42  u32 FSP_NOT_USED               == 0
/// @46  u32 FSP_SIZE                   tablespace size in pages (>= 4, FSP_SIZE * page_size <= file size)
/// @54  u32 FSP_SPACE_FLAGS            bit 0 POST_ANTELOPE, bits 1..4 ZIP_SSIZE, bit 5 ATOMIC_BLOBS,
///                                     bits 6..9 PAGE_SSIZE (0 = 16 KiB, 3..7 = 4..64 KiB)
/// @page_size-4 u32 FIL_PAGE_END_LSN_OLD_CHKSUM low 32 bits == low 32 bits of FIL_PAGE_LSN
/// ```
///
/// Page-aligned tablespace with matching header/trailer LSN; excludes `ROW_FORMAT=COMPRESSED`
/// (`ZIP_SSIZE != 0`) and `full_crc32` (bit 4) tablespaces.
pub(crate) fn verify_innodb_tablespace(prefix: &[u8], size: u64) -> Option<Label> {
    const FIL_PAGE_LSN: usize = 16; // @16: u64 newest modification LSN
    const FIL_PAGE_SPACE_ID: usize = 34; // @34: u32 tablespace id (FIL header copy)
    const FSP_SPACE_ID: usize = 38; // @38: u32 tablespace id (FSP header, FIL_PAGE_DATA + 0)
    const FSP_NOT_USED: usize = 42; // @42: u32 reserved, always 0
    const FSP_SIZE: usize = 46; // @46: u32 tablespace size in pages
    const FSP_SPACE_FLAGS: usize = 54; // @54: u32 FSP_SPACE_FLAGS
    const FSP_FLAGS_ZIP_SSIZE_MASK: u32 = 0xF << 1; // bits 1..4 (also covers MariaDB full_crc32 bit 4)
    const FSP_FLAGS_PAGE_SSIZE_SHIFT: u32 = 6; // bits 6..9
    const UNIV_PAGE_SIZE_ORIG: usize = 16_384; // PAGE_SSIZE == 0: default 16 KiB page
    const MIN_PAGES: u64 = 4;
    require!(
        size >= MIN_INNODB_TABLESPACE_SIZE,
        be32(prefix, INNODB_FIL_PAGE_OFFSET) == Some(0),
        be16(prefix, INNODB_FIL_PAGE_TYPE) == Some(INNODB_FIL_PAGE_TYPE_FSP_HDR)
    );
    let flags = be32(prefix, FSP_SPACE_FLAGS)?;
    require!(flags & FSP_FLAGS_ZIP_SSIZE_MASK == 0);
    let page_size = match (flags >> FSP_FLAGS_PAGE_SSIZE_SHIFT) & 0xF {
        0 => UNIV_PAGE_SIZE_ORIG,
        ssize @ 3..=7 => 512usize << ssize,
        _ => return None,
    };
    let lsn = be64(prefix, FIL_PAGE_LSN)?;
    let space_id = be32(prefix, FIL_PAGE_SPACE_ID)?;
    let fsp_size = be32(prefix, FSP_SIZE)? as u64;
    require!(
        size.is_multiple_of(page_size as u64),
        size >= MIN_PAGES * page_size as u64,
        lsn != 0,
        be32(prefix, FSP_SPACE_ID) == Some(space_id),
        be32(prefix, FSP_NOT_USED) == Some(0),
        fsp_size >= MIN_PAGES && fsp_size * page_size as u64 <= size,
        be32(prefix, page_size - 4) == Some(lsn as u32)
    );
    Some(Label::InnodbTablespace)
}

/// IX Binary Database Index (.ix).
pub(crate) fn verify_ix_index(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_IX_INDEX_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"IX\x01\x01\xff\xff\xff\xff\0\0\0\0\0\0\0\0"),
        le32(prefix, 16).unwrap_or(0) > 0
    );
    Some(Label::IxIndex)
}

/// JetBrains IntelliJ / Kotlin `PersistentHashMapValueStorage` (`.values.at`).
pub(crate) fn verify_jetbrains_phm_at(prefix: &[u8], size: u64) -> Option<Label> {
    const PHM_MAGIC: &[u8; 48] = b"/\0Header Record For PersistentHashMapValueStorag";

    require!(size >= MIN_JETBRAINS_PHM_AT_SIZE, prefix.len() >= 48);
    if starts(prefix, PHM_MAGIC) {
        return Some(Label::JetbrainsPhmAt);
    }
    if prefix.len() >= 52 && eq_at(prefix, 4, PHM_MAGIC) {
        return Some(Label::JetbrainsPhmAt);
    }
    None
}

/// KeePass Password Safe Encrypted Database (.kdb / .kdbx).
pub(crate) fn verify_keepass(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KEEPASS_SIZE
            && prefix.len() >= 16
            && matches!(le32(prefix, 4), Some(0xB54B_FB65..=0xB54B_FB67))
    );
    Some(Label::Keepass)
}

/// Amazon Kindle KFX E-Book Container (`.kfx`, `"CONT"` + `le16(4) in 1..=2` + container info bounds or `E0 01 00 EA` Ion marker).
pub(crate) fn verify_kindle_kfx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KINDLE_KFX_SIZE,
        prefix.len() >= 18,
        starts(prefix, b"CONT"),
        matches!(le16(prefix, 4), Some(1 | 2))
    );
    let win = get_slice(prefix, 0, prefix.len().min(64))?;
    let has_ion_in_prefix = win.windows(4).any(|w| w == [0xE0, 0x01, 0x00, 0xEA]);
    let hdr_len = le32(prefix, 6)? as u64;
    let info_off = le32(prefix, 10)? as u64;
    let info_len = le32(prefix, 14)? as u64;
    let valid_cont_geom = hdr_len >= 18
        && hdr_len <= size
        && info_off >= 18
        && info_len > 0
        && info_off.saturating_add(info_len) <= hdr_len;
    require!(has_ion_in_prefix || valid_cont_geom);
    Some(Label::KindleKfx)
}

/// Amazon Kindle KFX DRM-Protected Ion Content (`\xEADRMION\xEE`).
pub(crate) fn verify_kindle_kfx_drm(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_KINDLE_DRMION_SIZE, starts(prefix, b"\xEADRMION\xEE"));
    Some(Label::KindleDrmion)
}

/// Imspace Kudo Image Browser Catalog Database (`.kdb`, 1992, Imspace Systems).
///
/// Header layout: `@0..4 = b"NG\0\0"`, `@4..6 u16 BE version == 1`, `@6..8 u16 LE header_size == 24`,
/// `@8..18 == [0; 10]`, `@18..20 u16 LE == 1`, `@20..24 == [0; 4]`.
pub(crate) fn verify_kudo_catalog_db(prefix: &[u8], size: u64) -> Option<Label> {
    const VER_OFF: usize = 4; // @4..6: u16 BE version (1)
    const HDR_SZ_OFF: usize = 6; // @6..8: u16 LE header size (0x0018 = 24)
    const ZERO10_OFF: usize = 8; // @8..18: 10 zero bytes
    const FLAG1_OFF: usize = 18; // @18..20: u16 LE == 1
    const ZERO4_OFF: usize = 20; // @20..24: u32 LE == 0

    require!(
        size >= MIN_KUDO_KDB_SIZE,
        prefix.len() >= 24,
        starts(prefix, KUDO_KDB_MAGIC),
        be16(prefix, VER_OFF)? == 1,
        le16(prefix, HDR_SZ_OFF)? == 24,
        get_slice(prefix, ZERO10_OFF, FLAG1_OFF)? == [0u8; 10],
        le16(prefix, FLAG1_OFF)? == 1,
        le32(prefix, ZERO4_OFF)? == 0
    );
    Some(Label::KudoCatalogDb)
}

/// Imspace Kudo Image Browser Hash Index (`.hsh`, 1992, Imspace Systems).
///
/// Header layout: `@0..4 = b"FASH"`, `@4..8 u32 LE file_size == size`, `@8..12 == 0`,
/// `@12..16 u32 LE file_size == size`, `@16..20 u32 LE entry_count >= 1`,
/// `@20..24 u32 LE hash_bits <= 24`, `@24..28 u32 LE version == 1`.
pub(crate) fn verify_kudo_hash_index(prefix: &[u8], size: u64) -> Option<Label> {
    const FILE_SZ1_OFF: usize = 4; // @4..8: u32 LE total file size
    const ZERO_OFF: usize = 8; // @8..12: u32 LE reserved zero
    const FILE_SZ2_OFF: usize = 12; // @12..16: u32 LE duplicated total file size
    const ENTRIES_OFF: usize = 16; // @16..20: u32 LE indexed entry count (>= 1)
    const HASH_BITS_OFF: usize = 20; // @20..24: u32 LE hash table order (<= 24)
    const VER_OFF: usize = 24; // @24..28: u32 LE format version (1)

    require!(
        size >= MIN_KUDO_HSH_SIZE,
        prefix.len() >= 28,
        starts(prefix, KUDO_HSH_MAGIC),
        le32(prefix, FILE_SZ1_OFF)? as u64 == size,
        le32(prefix, ZERO_OFF)? == 0,
        le32(prefix, FILE_SZ2_OFF)? as u64 == size,
        le32(prefix, ENTRIES_OFF)? >= 1,
        le32(prefix, HASH_BITS_OFF)? <= 24,
        le32(prefix, VER_OFF)? == 1
    );
    Some(Label::KudoHashIndex)
}

/// Applied Biosystems / Life Technologies SDS Real-Time PCR Data (.sds, PRONOM fmt/586).
pub(crate) fn verify_lifetech_sds(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_LIFETECH_SDS_SIZE && prefix.len() >= 8 && starts(prefix, b"SDS2\x00\x02\x00\x02")
    {
        Some(Label::Sds)
    } else {
        None
    }
}

/// Lingea Lexicon Bilingual Dictionary Database (.lld, 2002) Lingea.
pub(crate) fn verify_lingea_lld(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LINGEA_LLD_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"Dict: Mini.  \n(c) "),
        u8_at(prefix, 36)? == 0
    );
    Some(Label::LingeaLld)
}

/// Sequential Offset/Length Localization String Index (`.strndx`) 12-byte record chain.
pub(crate) fn verify_locale_strndx(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_LOCALE_STRNDX_SIZE,
        size.is_multiple_of(12),
        prefix.len() >= 36,
        le32(prefix, 0) == Some(1),
        le32(prefix, 4) == Some(0)
    );
    let len0 = le32(prefix, 8)?;
    let id1 = le32(prefix, 12)?;
    let off1 = le32(prefix, 16)?;
    let len1 = le32(prefix, 20)?;
    let id2 = le32(prefix, 24)?;
    let off2 = le32(prefix, 28)?;
    let len2 = le32(prefix, 32)?;
    require!(
        (1..=65536).contains(&len0),
        id1 == 2,
        off1 == len0,
        (1..=65536).contains(&len1),
        id2 == 3,
        off2 == len0.checked_add(len1)?,
        (1..=65536).contains(&len2)
    );
    Some(Label::LocaleStrndx)
}

/// Apache Lucene Codec Segment / Index file (`.si` / `.cfe` / `.cfs` / `.fnm` / `.pos` / `.tim` / `.tip` / `.dvd` / `.dvm`).
pub(crate) fn verify_lucene_si(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_LUCENE_CODEC_SIZE, prefix.len() >= 24, starts(prefix, LUCENE_CODEC_MAGIC));
    let name_len = u8_at(prefix, LUCENE_NAME_LEN_OFFSET)? as usize;
    require!(
        (6..=64).contains(&name_len),
        5 + name_len.min(16) <= prefix.len(),
        eq_at(prefix, 5, b"Lucene")
            || get_slice(prefix, 5, 5 + name_len.min(16))?
                .iter()
                .all(|&b| b.is_ascii_alphanumeric() || b == b'_')
    );
    Some(Label::LuceneSi)
}

/// Malcat Constant & Kesakode Signature Database (`.csig`, 2020 / 2024) Malcat / Issue9.
pub(crate) fn verify_malcat_csig(prefix: &[u8], size: u64) -> Option<Label> {
    let hdr_base = if starts(prefix, b"!MCCSTS!") {
        require!(size >= MIN_MALCAT_CSIG_V2_SIZE && prefix.len() >= 24);
        8usize
    } else if starts(prefix, b"MCST") {
        require!(size >= MIN_MALCAT_CSIG_V1_SIZE && prefix.len() >= 20);
        4usize
    } else {
        return None;
    };

    let version = u8_at(prefix, hdr_base)?;
    let flags = u8_at(prefix, hdr_base + 1)?;
    let num_categories = le32(prefix, hdr_base + 2)?;
    let num_names = le32(prefix, hdr_base + 6)?;
    let num_entries = le32(prefix, hdr_base + 10)?;
    let first_cat_len = u8_at(prefix, hdr_base + 14)? as usize;
    let cat_start = hdr_base + 15;

    require!(
        version == 1
            && flags <= 1
            && (1..=10_000).contains(&num_categories)
            && (1..=10_000_000).contains(&num_names)
            && (1..=50_000_000).contains(&num_entries)
            && (1..=64).contains(&first_cat_len)
            && prefix.len() >= cat_start + first_cat_len
            && get_slice(prefix, cat_start, cat_start + first_cat_len)?
                .iter()
                .all(|&b| (0x20..=0x7E).contains(&b))
    );
    let _ = eq_at(prefix, cat_start, get_slice(prefix, cat_start, cat_start + first_cat_len)?);

    Some(Label::MalcatCsig)
}

/// Metakit v4 embedded database (`JL\x1A\x00` / `LJ\x1A\x00` / `JL\x1A\x80` / `LJ\x1A\x80`).
pub(crate) fn verify_metakit(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 24,
        prefix.len() >= 8,
        starts(prefix, b"JL\x1A\x00")
            || starts(prefix, b"LJ\x1A\x00")
            || starts(prefix, b"JL\x1A\x80")
            || starts(prefix, b"LJ\x1A\x80"),
    );
    let declared_len = be32(prefix, 4)? as u64;
    require!(declared_len >= 24, declared_len <= size);
    if declared_len <= prefix.len() as u64 {
        let ftr_off = (declared_len as usize).checked_sub(16)?;
        require!(
            be32(prefix, ftr_off)? == 0x8000_0000,
            be32(prefix, ftr_off + 4)? == (declared_len - 16) as u32,
            u8_at(prefix, ftr_off + 8)? == 0x80,
        );
        let toc_size = (be32(prefix, ftr_off + 8)? & 0x00FF_FFFF) as usize;
        let toc_rel = be32(prefix, ftr_off + 12)? as usize;
        require!(
            toc_size > 0,
            toc_rel >= 8,
            toc_rel.checked_add(toc_size).is_some_and(|te| te <= declared_len as usize - 16),
        );
    } else {
        require!(declared_len == size);
    }
    Some(Label::Metakit)
}

/// HC 0x01F6 UTF-16LE Container (.hc).
pub(crate) fn verify_metatrader_hc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_METATRADER_HC_SIZE,
        prefix.len() >= 40,
        starts(
            prefix,
            b"\xf6\x01\0\0C\0o\0p\0y\0r\0i\0g\0h\0t\0 \x002\x000\x000\x000\0-\x002\x000\0"
        )
    );
    require!(has_non_ascii_text_byte(prefix.get(..40)?));
    Some(Label::MetatraderHc)
}

/// Micromeritics Instrument Subset Database (.smp / .mth / .anc) Micromeritics.
pub(crate) fn verify_micromeritics_subset(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MICROMERITICS_SUBSET_SIZE,
        prefix.len() >= 40,
        starts(prefix, b"^vMIC##&&FS\x000200"),
        get_slice(prefix, 16, 20)? == get_slice(prefix, 20, 24)?,
        get_slice(prefix, 32, 38)? == b"SUBSET"
    );
    require!(has_non_ascii_text_byte(prefix.get(..40)?));
    Some(Label::MicromeriticsSubset)
}

/// MikroTik RouterOS WinBox Address Book Database (`.cdb`, 2005) MikroTik.
pub(crate) fn verify_mikrotik_winbox_cdb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_WINBOX_CDB_SIZE,
        prefix.len() >= 10,
        starts(prefix, WINBOX_CDB_MAGIC),
        eq_at(prefix, 8, WINBOX_M2_TAG),
    );
    let rec0_len = le32(prefix, 4)? as u64;
    require!(rec0_len >= 4, 8u64.saturating_add(rec0_len) <= size);
    Some(Label::WinboxCdb)
}

/// Linux `mlocate` file system index database (`mlocate.db`, `"\0mlocate"`).
pub(crate) fn verify_mlocate_db(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MLOCATE_DB_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"\0mlocate"),
        u8_at(prefix, 12)? == 0,
        u8_at(prefix, 13)? <= 1,
        eq_at(prefix, 14, b"\0\0/")
    );
    Some(Label::MlocateDb)
}

/// Microsoft Office Answer Wizard Database (`.aw`, Office 97–2003).
pub(crate) fn verify_ms_answer_wizard_aw(prefix: &[u8], size: u64) -> Option<Label> {
    const AW_PREFIX: &[u8; 2] = hex2bin!("8A 01"); // @0..2 and @46..48: Answer Wizard record marker
    const AW_TAG: &[u8; 7] = hex2bin!("00 00 00 E1 08 00 00"); // @3..10: invariant header signature
    const AW_ZERO_PAD: &[u8; 6] = &[0u8; 6]; // @16..22: reserved zero padding

    require!(
        size >= MIN_MS_ANSWER_WIZARD_AW_SIZE,
        prefix.len() >= 48,
        starts(prefix, AW_PREFIX),
        matches!(u8_at(prefix, 2)?, 0x09 | 0x0A), // 0x09 = Office 97/2000, 0x0A = Office XP/2003
        eq_at(prefix, 3, AW_TAG),
        eq_at(prefix, 16, AW_ZERO_PAD),
        eq_at(prefix, 46, AW_PREFIX)
    );
    Some(Label::MsAnswerWizardAw)
}

/// Microsoft Flight Simulator Scenery Database (`.bgl`, FS2004/FSX/P3D/MSFS, 2003 Microsoft).
pub(crate) fn verify_msfs_bgl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MSFS_BGL_SIZE,
        prefix.len() >= 24,
        starts(prefix, MSFS_BGL_MAGIC),
        le32(prefix, 4) == Some(0x38),
        eq_at(prefix, 16, hex2bin!("03 18 05 08")),
        (1..=1024).contains(&le32(prefix, 20)?)
    );
    Some(Label::MsfsBgl)
}

/// Microsoft Train Simulator Kuju SIMIS File (`"SIMISA@F"` binary, `"SIMISA@@@@@@@@@@JINX0"` ASCII, unpacked `"JINX0...______"`, or UTF-16LE `"\u{FEFF}SIMISA@@@@@@@@@@JINX0"`).
pub(crate) fn verify_msts_simis_binary(prefix: &[u8], size: u64) -> Option<Label> {
    const SIMIS_UTF16LE_MAGIC: &[u8; 44] = bytes!(
        hex2bin!("FF FE"),
        b"S\0I\0M\0I\0S\0A\0@\0@\0@\0@\0@\0@\0@\0@\0@\0@\0J\0I\0N\0X\x00",
        b"0\0"
    ); // @0..44: UTF-16LE BOM + "SIMISA@@@@@@@@@@JINX0"
    const SIMIS_ASCII_MAGIC: &[u8; 21] = b"SIMISA@@@@@@@@@@JINX0"; // @0..21: ASCII uncompressed SIMIS header
    const SIMIS_JINX_MAGIC: &[u8; 5] = b"JINX0"; // @0..5: unpacked SIMIS header prefix ("JINX0s1b______\r\n")
    const SIMIS_JINX_UNDERSCORES_OFF: usize = 8; // @8..14: 6 underscores in unpacked SIMIS header

    if size >= 64 && prefix.len() >= 64 && starts(prefix, SIMIS_UTF16LE_MAGIC) {
        return Some(Label::MstsSimis);
    }
    if size >= MIN_MSTS_SIMIS_SIZE && prefix.len() >= 32 && starts(prefix, SIMIS_ASCII_MAGIC) {
        return Some(Label::MstsSimis);
    }
    if size >= MIN_MSTS_SIMIS_SIZE
        && prefix.len() >= 24
        && starts(prefix, SIMIS_JINX_MAGIC)
        && eq_at(prefix, SIMIS_JINX_UNDERSCORES_OFF, b"______")
        && has_non_ascii_text_byte(prefix.get(14..prefix.len().min(64))?)
    {
        return Some(Label::MstsSimis);
    }
    require!(
        size >= MIN_MSTS_SIMIS_SIZE,
        prefix.len() >= 16,
        starts(prefix, b"SIMISA@F"),
        has_non_ascii_text_byte(prefix.get(8..prefix.len().min(32))?)
    );
    Some(Label::MstsSimis)
}

/// Microsoft Visual C++ Incremental Linker Program Database (`.ipdb`).
pub(crate) fn verify_msvc_ipdb(prefix: &[u8], size: u64) -> Option<Label> {
    const MSVC_IPDB_MAGIC: &[u8; 4] = hex2bin!("48 2E 51 7F"); // @0..4: 0x7F512E48 IPDB signature
    const VERSION_OFF: usize = 4; // @4..8: u32 LE IPDB format version (0x041B VS2019, 0x041F VS2022)
    const TAG_13_OFF: usize = 12; // @12..16: constant 0x13 header tag
    const ZERO_24_OFF: usize = 24; // @24..28: reserved zero field
    const PAGE_TAG_V19_OFF: usize = 36; // @36..40: 0x10 tag in v0x041B
    const PAGE_TAG_V22_OFF: usize = 40; // @40..44: 0x10 tag in v0x041F

    require!(
        size >= MIN_MSVC_IPDB_SIZE,
        prefix.len() >= 44,
        starts(prefix, MSVC_IPDB_MAGIC),
        (0x0400..=0x0500).contains(&le32(prefix, VERSION_OFF)?),
        le32(prefix, TAG_13_OFF) == Some(0x13),
        le32(prefix, ZERO_24_OFF) == Some(0),
        le32(prefix, PAGE_TAG_V19_OFF) == Some(0x10)
            || le32(prefix, PAGE_TAG_V22_OFF) == Some(0x10)
    );
    Some(Label::MsvcIpdb)
}

/// Microsoft Works Database v1-2: record stream starting `20 54 02 00 00 00 05 54 02 00`
/// (record type `0x5420`, length 2, value 0; record type `0x5405`, length 2).
pub(crate) fn verify_msworks_wdb(prefix: &[u8], size: u64) -> Option<Label> {
    const PREAMBLE: &[u8] = hex2bin!("20 54 02 00 00 00 05 54 02 00"); // @0, 10 bytes
    require!(size >= MIN_MSWORKS_WDB_SIZE, starts(prefix, PREAMBLE));
    Some(Label::MsworksWdb)
}

/// Nintendo JSystem Binary Message Group (`.bmg`, GameCube / Wii / DS / 3DS).
///
/// Header layout:
/// - `@0..8`: `"MESGbmg1"` (big-endian) or `"GSEM1gmb"` (little-endian)
/// - `@8..12`: `u32` `file_size` (`>= 32`)
/// - `@12..16`: `u32` `section_count` (`1..=64`)
/// - `@16`: `u8` `text_encoding` (`0..=4`)
pub(crate) fn verify_nintendo_bmg(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC_BE: &[u8; 8] = b"MESGbmg1"; // @0..8: GameCube / Wii BE BMG
    const MAGIC_LE: &[u8; 8] = b"GSEM1gmb"; // @0..8: DS / 3DS LE BMG
    const FILE_SIZE_OFF: usize = 8; // @8..12: u32 file size
    const SECT_COUNT_OFF: usize = 12; // @12..16: u32 section count
    const ENC_OFF: usize = 16; // @16: encoding (0..=4)

    require!(size >= MIN_NINTENDO_BMG_SIZE, prefix.len() >= 32);
    let (decl_size, sect_count) = if starts(prefix, MAGIC_BE) {
        (be32(prefix, FILE_SIZE_OFF)?, be32(prefix, SECT_COUNT_OFF)?)
    } else if starts(prefix, MAGIC_LE) {
        (le32(prefix, FILE_SIZE_OFF)?, le32(prefix, SECT_COUNT_OFF)?)
    } else {
        return None;
    };
    require!(decl_size >= 32, (1..=64).contains(&sect_count), u8_at(prefix, ENC_OFF)? <= 4);
    Some(Label::NintendoBmg)
}

/// Nintendo LMS Binary Message container (`.msbt`, `.msbf`, `.msbp`).
///
/// Header layout:
/// - `@0..8`: `"MsgStdBn"`, `"MsgPrjBn"`, or `"MsgFlwBn"`
/// - `@8..10`: BOM (`FF FE` LE or `FE FF` BE)
/// - `@10..12`: `00 00` padding
/// - `@12`: `u8` text encoding (`0` = UTF-8, `1` = UTF-16, `2` = UTF-32)
/// - `@13`: `u8` version (`3`)
pub(crate) fn verify_nintendo_msbt(prefix: &[u8], size: u64) -> Option<Label> {
    const MSBT_MAGIC: &[u8; 8] = b"MsgStdBn"; // @0..8: MSBT text table
    const MSBP_MAGIC: &[u8; 8] = b"MsgPrjBn"; // @0..8: MSBP project
    const MSBF_MAGIC: &[u8; 8] = b"MsgFlwBn"; // @0..8: MSBF flow chart
    const BOM_OFF: usize = 8; // @8..10: byte-order mark
    const BOM_LE: &[u8; 2] = hex2bin!("FF FE");
    const BOM_BE: &[u8; 2] = hex2bin!("FE FF");
    const PAD_OFF: usize = 10; // @10..12: zero padding
    const ENC_OFF: usize = 12; // @12: encoding (0..=2)
    const VER_OFF: usize = 13; // @13: format version (3)

    require!(
        size >= MIN_NINTENDO_MSBT_SIZE,
        prefix.len() >= 32,
        starts(prefix, MSBT_MAGIC) || starts(prefix, MSBP_MAGIC) || starts(prefix, MSBF_MAGIC),
        eq_at(prefix, BOM_OFF, BOM_LE) || eq_at(prefix, BOM_OFF, BOM_BE),
        le16(prefix, PAD_OFF)? == 0,
        u8_at(prefix, ENC_OFF)? <= 2,
        u8_at(prefix, VER_OFF)? == 3
    );
    Some(Label::NintendoMsbt)
}

/// Cadcorp SIS Named Object Library (`.nol`).
pub(crate) fn verify_nol_2900_layer(prefix: &[u8], size: u64) -> Option<Label> {
    const CADCORP_NOL_SIG: &[u8; 8] = hex2bin!("01 88 BF FF FF FF 00 00"); // @8..16

    require!(
        size >= MIN_NOL_2900_LAYER_SIZE,
        prefix.len() >= 27,
        get_slice(prefix, 0, 4)? == [0u8; 4],
        (0x20..=0x40).contains(&le16(prefix, 4)?), // @4..6: SIS format version (0x27, 0x29, ...)
        le16(prefix, 6) == Some(1),
        eq_at(prefix, 8, CADCORP_NOL_SIG),
        (1..=100_000).contains(&le32(prefix, 16)?), // @16..20: entry count
        (0x81..=0xBF).contains(&u8_at(prefix, 20)?), // @20: UTF-16LE class name length tag
        eq_at(prefix, 21, b"A\0"), // @21..23: Cadcorp class prefix ("AFilter", "APrj", ...)
        u8_at(prefix, 24)? == 0,
        u8_at(prefix, 26)? == 0
    );
    Some(Label::CadcorpSisNol)
}

/// OpenFst Weighted Finite-State Transducer (`.fst`), FST Archive (`.far`),
/// OpenFst Compact Compress stream (`0x6ECC2532` / `0x6ECC1932`), OpenFst EncodeTable (`0x7ED96B4A` / `0x7ED96D4A`),
/// or Google Zuckerli Huffman FST (`8F 30 91 B2 78 2B 9B 47 01`).
pub(crate) fn verify_openfst_transducer(prefix: &[u8], size: u64) -> Option<Label> {
    const OPENFST_MAGIC: &[u8; 4] = hex2bin!("D6 FD B2 7E"); // 0x7EB2FDD6 LE
    const FAR_STLIST_MAGIC: &[u8; 4] = hex2bin!("5C F3 B2 7E"); // 0x7EB2F35C LE
    const FAR_STTABLE_MAGIC: &[u8; 4] = hex2bin!("5C 51 56 00"); // 0x0056515C LE
    const OPENFST_COMPRESS_MAGIC: &[u8; 4] = hex2bin!("32 25 CC 6E"); // 0x6ECC2532 LE
    const OPENFST_COMPRESS_LEGACY_MAGIC: &[u8; 4] = hex2bin!("32 19 CC 6E"); // 0x6ECC1932 LE
    const OPENFST_ENCODE_MAGIC: &[u8; 4] = hex2bin!("4A 6B D9 7E"); // 0x7ED96B4A LE
    const OPENFST_ENCODE_HEADER_MAGIC: &[u8; 4] = hex2bin!("4A 6D D9 7E"); // 0x7ED96D4A LE
    require!(size >= MIN_OPENFST_SIZE, prefix.len() >= 16);
    if starts(prefix, OPENFST_MAGIC) || starts(prefix, OPENFST_ENCODE_HEADER_MAGIC) {
        let len = le32(prefix, 4)? as usize;
        require!(
            (1..=64).contains(&len),
            prefix.len() >= 8 + len,
            get_slice(prefix, 8, 8 + len)?.iter().all(|&b| (0x20..=0x7E).contains(&b))
        );
        return Some(Label::Openfst);
    }
    if (starts(prefix, FAR_STLIST_MAGIC) || starts(prefix, FAR_STTABLE_MAGIC))
        && le32(prefix, 4) == Some(1)
    {
        return Some(Label::Openfst);
    }
    if (starts(prefix, OPENFST_COMPRESS_MAGIC) || starts(prefix, OPENFST_COMPRESS_LEGACY_MAGIC))
        && (1..=100_000_000).contains(&le64(prefix, 4)?)
    {
        return Some(Label::Openfst);
    }
    if starts(prefix, OPENFST_ENCODE_MAGIC)
        && (1..=31).contains(&le32(prefix, 4)?)
        && (1..=100_000_000).contains(&le64(prefix, 8)?)
    {
        return Some(Label::Openfst);
    }
    if starts(prefix, hex2bin!("8F 30 91 B2 78 2B 9B 47 01")) {
        return Some(Label::Openfst);
    }
    None
}

/// Microsoft Outlook Nickname AutoComplete Cache (`.nk2`, `0xBAADF00D` LE).
pub(crate) fn verify_outlook_nk2(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_OUTLOOK_NK2_SIZE,
        prefix.len() >= 32,
        starts(prefix, hex2bin!("0D F0 AD BA")),
        matches!(le32(prefix, 4), Some(10..=14)),
        (1..=0x0010_0000).contains(&le32(prefix, 12)?)
    );
    Some(Label::OutlookNk2)
}

/// Password Safe v2/v3 encrypted database (`.psafe3` / `.dat`).
pub(crate) fn verify_passwordsafe(prefix: &[u8], size: u64) -> Option<Label> {
    if starts(prefix, b"PWS3") {
        require!(size >= 40, prefix.len() >= 40, has_non_ascii_text_byte(prefix.get(4..40)?),);
        return Some(Label::Passwordsafe);
    }
    require!(
        size >= 12,
        prefix.len() >= 12,
        starts(prefix, b"PWS2"),
        has_non_ascii_text_byte(prefix.get(4..12)?),
    );
    Some(Label::Passwordsafe)
}

/// PDI2 Bilingual Dictionary Database (.pdi).
pub(crate) fn verify_pdi2_dictionary(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PDI2_DICTIONARY_SIZE,
        prefix.len() >= 44,
        starts(prefix, b"0|PDI2|"),
        has_non_ascii_text_byte(prefix.get(7..44)?)
    );
    Some(Label::Pdi2Dictionary)
}

/// PGP Desktop Compiled Localization String Table (.str, 2005) PGP.
pub(crate) fn verify_pgp_locstr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PGP_LOCSTR_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"PGPlocstr\n"),
        u8_at(prefix, 10)? == 0xC4
    );
    Some(Label::PgpLocstr)
}

/// PRMI FF63FF29 Raster Image (.prmi).
pub(crate) fn verify_premiere_prmi(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_PREMIERE_PRMI_SIZE,
        prefix.len() >= 28,
        starts(
            prefix,
            b"\xff\x63\xff\x29\0\x02\x64\x34\xe0\x34\0\x80\x80\0\0\x80\0\x80\0\0\xff\xe7\0\xa4"
        )
    );
    require!(has_non_ascii_text_byte(prefix.get(..28)?));
    Some(Label::PremierePrmi)
}

/// Min Communications Ran Online GLITEM_MIX Item Crafting Table (.ims).
pub(crate) fn verify_ran_online_ims(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_RAN_ONLINE_IMS_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"GLITEM_MIX\0\0\0\0\0\0\0\0\0\0\0\0\0\0")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::RanOnlineIms)
}

/// Capcom Resident Evil 4 Event Message Table (`.mdt`).
pub(crate) fn verify_re4_mdt(prefix: &[u8], size: u64) -> Option<Label> {
    const RE4_MDT_HDR: &[u8; 8] = hex2bin!("06 00 00 00 24 00 00 00"); // @0..8: lang_count=6, first_offset=0x24
    const FIRST_OFF: u32 = 0x24;

    require!(size >= MIN_RE4_MDT_SIZE, prefix.len() >= 36, starts(prefix, RE4_MDT_HDR));
    let mut prev = FIRST_OFF;
    for idx in 1..8usize {
        let off = le32(prefix, 4 + idx * 4)?;
        require!(off >= prev, off % 4 == 0, (off as u64) < size);
        prev = off;
    }
    require!(prev > FIRST_OFF);
    Some(Label::Re4Mdt)
}

/// `redb` Embedded Key-Value Database file (`.redb`).
pub(crate) fn verify_redb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_REDB_SIZE,
        prefix.len() >= 24,
        starts(prefix, REDB_MAGIC),
        (1..=16).contains(&u8_at(prefix, REDB_VERSION_OFFSET)?)
    );
    let page_size = le32(prefix, REDB_PAGE_SIZE_OFFSET)?;
    require!((512..=65_536).contains(&page_size) && page_size.is_power_of_two());
    Some(Label::Redb)
}

/// ASCII / Enterbrain RPG Maker 2000 / 2003 LCF Database / Map / Save (.ldb / .lmt / .lmu / .lsd).
pub(crate) fn verify_rpgmaker_lcf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_RPGMAKER_LCF_SIZE && prefix.len() >= 16);
    let magic_end = if starts(prefix, b"\x0BLcfDataBase") || starts(prefix, b"\x0BLcfSaveData") {
        12
    } else if starts(prefix, b"\x0ALcfMapTree") || starts(prefix, b"\x0ALcfMapUnit") {
        11
    } else {
        return None;
    };
    let first_chunk_id = u8_at(prefix, magic_end)?;
    require!((1..=0x7F).contains(&first_chunk_id));
    Some(Label::Ldb)
}

/// Apple Safari / CFNetwork Binary Cookies store (`.binarycookies`, `"cook"`).
pub(crate) fn verify_safari_binarycookies(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_SAFARI_BINARYCOOKIES_SIZE, prefix.len() >= 20, starts(prefix, b"cook"));
    let num_pages = be32(prefix, 4)? as usize;
    require!((1..=65536).contains(&num_pages));
    let first_page_size = be32(prefix, 8)? as u64;
    let first_page_off = 8usize.saturating_add(num_pages.saturating_mul(4));
    require!(first_page_size >= 8, (first_page_off as u64) + first_page_size <= size);
    if first_page_off + 4 <= prefix.len() {
        require!(eq_at(prefix, first_page_off, hex2bin!("00 00 01 00")));
    }
    Some(Label::SafariBinarycookies)
}

/// Esri Smart Data Compression (SDC FORMAT 1.00) Geospatial Database (.gdb, Esri).
pub(crate) fn verify_sdc_gdb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SDC_GDB_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"SDC FORMAT 1.00\0"),
        le32(prefix, 20).unwrap_or(0) > 0
    );
    require!(has_non_ascii_text_byte(prefix.get(..28)?));
    Some(Label::SdcGdb)
}

/// Shinmera Simple File Format 3 (`.sf3`, `81 53 46 33 00 E0 D0 0D 0A 0A`).
pub(crate) fn verify_simple_file_format_3(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SF3_SIZE,
        prefix.len() >= 12,
        starts(prefix, b"\x81SF3\x00\xE0\xD0\r\n\n"),
        (1..=9).contains(&u8_at(prefix, 10)?)
    );
    Some(Label::ShinmeraSf3)
}

/// SQLite 2.x B-tree database file.
pub(crate) fn verify_sqlite2(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 48] = b"** This file contains an SQLite 2.1 database **\0";
    const BTREE_MAGIC: u32 = 0xDAE3_7528;
    const OFF_BTREE_MAGIC: usize = 48;
    require!(
        size >= 52,
        prefix.len() >= 52,
        starts(prefix, MAGIC),
        le32(prefix, OFF_BTREE_MAGIC) == Some(BTREE_MAGIC)
            || be32(prefix, OFF_BTREE_MAGIC) == Some(BTREE_MAGIC),
    );
    Some(Label::Sqlite2)
}

/// SQLite 2/3 rollback journal (`.sqlite-journal`).
pub(crate) fn verify_sqlite_journal(prefix: &[u8], size: u64) -> Option<Label> {
    const SQLITE3_JOURNAL: &[u8; 8] = &[0xD9, 0xD5, 0x05, 0xF9, 0x20, 0xA1, 0x63, 0xD7];
    const SQLITE2_JOURNAL: &[u8; 8] = &[0xD9, 0xD5, 0x05, 0xF9, 0x20, 0xA1, 0x63, 0xD6];
    if (size >= 28 && prefix.len() >= 8 && starts(prefix, SQLITE3_JOURNAL))
        || (size >= 20 && prefix.len() >= 20 && starts(prefix, SQLITE2_JOURNAL))
    {
        return Some(Label::SqliteJournal);
    }
    None
}

/// SQLite 3 Write-Ahead Log (`-wal`).
pub(crate) fn verify_sqlite_wal(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= 32,
        prefix.len() >= 32,
        starts(prefix, &[0x37, 0x7F, 0x06, 0x82]) || starts(prefix, &[0x37, 0x7F, 0x06, 0x83]),
        be32(prefix, 4)? == 3_007_000,
    );
    Some(Label::SqliteWal)
}

/// StatSoft Statistica Scrollsheet (`.scr`, 1991 StatSoft).
///
/// Header & chained section directory:
/// - `0..4`: `6F D9 FF FF`
/// - `4..6`: `u16le` `1`; `6..8`: `u16le` `2`; `8..12`: `u32le` `0`
/// - `12..16`: `u32le` header size (`>= 0x40`, typically `0x88`)
/// - `16..40`: 3 contiguous `(offset, length)` `u32le` section descriptors with `off0 == hdr_size`,
///   `off0 + len0 == off1`, `off1 + len1 == off2`, and `off2 + len2 <= size`
pub(crate) fn verify_statistica_scr(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_STATISTICA_SCR_SIZE,
        prefix.len() >= 40,
        starts(prefix, STATISTICA_SCR_MAGIC),
        le16(prefix, 4) == Some(1),
        le16(prefix, 6) == Some(2),
        le32(prefix, 8) == Some(0)
    );
    let hdr_sz = le32(prefix, 12)?;
    let off0 = le32(prefix, 16)?;
    let len0 = le32(prefix, 20)?;
    let off1 = le32(prefix, 24)?;
    let len1 = le32(prefix, 28)?;
    let off2 = le32(prefix, 32)?;
    let len2 = le32(prefix, 36)?;
    require!(
        hdr_sz >= 0x40,
        (hdr_sz as u64) < size,
        off0 == hdr_sz,
        len0 > 0,
        off0.checked_add(len0) == Some(off1),
        off1.checked_add(len1) == Some(off2),
        (off2 as u64).checked_add(len2 as u64)? <= size
    );

    Some(Label::StatisticaScr)
}

/// StatSoft Statistica Spreadsheet (`.sta`, 1991 StatSoft / CSS: Complete Statistical System).
///
/// Header layout:
/// - `0..4`: `"CSS "` (`43 53 53 20`)
/// - `4..8`: negative `i32le` format/version tag (`u8_at(prefix, 4)? in 0x60..=0x7F`, `get_slice(prefix, 5, 8)? == [0xD9, 0xFF, 0xFF]`)
/// - `8..12`: `u32le` case count (`1..=10_000_000`)
/// - `12..16`: `u32le` variable count (`1..=1_000_000`)
/// - `16..20`: `u32le` variable name length (`1..=64`)
/// - `20..24`: `u32le` header/case-name width (`<= 64`)
pub(crate) fn verify_statistica_sta(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_STATISTICA_STA_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"CSS "),
        (0x60..=0x7F).contains(&u8_at(prefix, 4)?),
        u8_at(prefix, 5)? == 0xD9,
        u8_at(prefix, 6)? == 0xFF,
        u8_at(prefix, 7)? == 0xFF
    );
    let ncases = le32(prefix, 8)?;
    let nvars = le32(prefix, 12)?;
    let vname_len = le32(prefix, 16)?;
    let cname_len = le32(prefix, 20)?;
    require!(
        (1..=10_000_000).contains(&ncases),
        (1..=1_000_000).contains(&nvars),
        (1..=64).contains(&vname_len),
        cname_len <= 64
    );

    Some(Label::StatisticaSta)
}

/// Linux `systemd` Binary Journal Message Catalog (`"RHHHKSLP"`).
pub(crate) fn verify_systemd_catalog(prefix: &[u8], size: u64) -> Option<Label> {
    const CATALOG_MAGIC: &[u8; 8] = b"RHHHKSLP"; // @0..8: CatalogHeader signature
    const FLAGS_OFF: usize = 8; // @8..16: u32 LE incompatible_flags (0) + u32 LE compatible_flags (0)
    const HDR_SIZE_OFF: usize = 16; // @16..24: u64 LE header_size (40..=size)
    const ITEM_SIZE_OFF: usize = 32; // @32..40: u64 LE catalog_item_size (56 on systemd; >= 16)
    require!(
        size >= MIN_SYSTEMD_CATALOG_SIZE,
        prefix.len() >= 40,
        starts(prefix, CATALOG_MAGIC),
        le64(prefix, FLAGS_OFF) == Some(0),
        (40..=size).contains(&le64(prefix, HDR_SIZE_OFF)?),
        (16..=4096).contains(&le64(prefix, ITEM_SIZE_OFF)?)
    );
    Some(Label::SystemdCatalog)
}

/// TDBD Binary Database (.tdbd).
pub(crate) fn verify_tdbd_database(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TDBD_DATABASE_SIZE,
        prefix.len() >= 32,
        starts(prefix, b"\x81\x75\xe4\x39\x01\0\0\0\0 \0\0")
    );
    Some(Label::TdbdDatabase)
}

/// Google TensorStore OCDBT B+Tree Node / Manifest (.ocdbt).
pub(crate) fn verify_tensorstore_ocdbt(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TENSORSTORE_OCDBT_SIZE
            && prefix.len() >= 12
            && u8_at(prefix, 8)? == 0
            && le32(prefix, 4).unwrap_or(0) >= 20
    );
    Some(Label::Ocdbt)
}

/// SoftVelocity Clarion TopSpeed Database (`.tps`).
pub(crate) fn verify_topspeed_0205_tps(prefix: &[u8], size: u64) -> Option<Label> {
    const TOPSPEED_MAGIC: &[u8; 4] = b"tOpS"; // @14..18

    require!(
        size >= MIN_TOPSPEED_0205_TPS_SIZE,
        prefix.len() >= 20,
        get_slice(prefix, 0, 4)? == [0u8; 4], // @0..4: offset 0
        le16(prefix, 4) == Some(0x0200)       // @4..6: 512-byte header size
    );
    let fsize = le32(prefix, 6)?; // @6..10: declared file size
    require!(
        fsize >= 512,
        le32(prefix, 10) == Some(fsize), // @10..14: mirror file size
        eq_at(prefix, 14, TOPSPEED_MAGIC),
        le16(prefix, 18) == Some(0)
    );
    Some(Label::TopspeedTps)
}

/// Valve Source 2 Tools Asset Info Index (`readonly_tools_asset_info.bin`, 2014 Valve).
///
/// Header layout:
/// - `@0..4`: `E8 AC CC C4` (`0xC4CCACE8`, v1 `version` `8..=10`) or `E9 AC CC C4` (`0xC4CCACE9`, v2 `version` `11..=16`)
/// - `@4..8`: `u32 LE` `version`
/// - `@8..12`: `u32 LE` `file_count` (`<= 10_000_000`)
/// - `@12..16`: `u32 LE` `has_edit_info` (`0..=1`)
/// - `@16..20`: `u32 LE` `mods_count` (`<= 4096`)
pub(crate) fn verify_valve_tools_asset_info(prefix: &[u8], size: u64) -> Option<Label> {
    const VERSION_OFF: usize = 4; // @4..8: u32 LE format version
    const FILE_COUNT_OFF: usize = 8; // @8..12: u32 LE indexed asset count
    const EDIT_INFO_OFF: usize = 12; // @12..16: u32 LE edit-info boolean (0..=1)
    const MODS_COUNT_OFF: usize = 16; // @16..20: u32 LE mod directory count (<= 4096)

    require!(size >= MIN_VALVE_TOOLS_ASSET_INFO_SIZE, prefix.len() >= 20);
    let version = le32(prefix, VERSION_OFF)?;
    let magic_ver_ok = (starts(prefix, VALVE_TOOLS_ASSET_INFO_V1_MAGIC)
        && (8..=10).contains(&version))
        || (starts(prefix, VALVE_TOOLS_ASSET_INFO_V2_MAGIC) && (11..=16).contains(&version));
    require!(
        magic_ver_ok,
        le32(prefix, FILE_COUNT_OFF)? <= 10_000_000,
        le32(prefix, EDIT_INFO_OFF)? <= 1,
        le32(prefix, MODS_COUNT_OFF)? <= 4096
    );
    Some(Label::ValveToolsAssetInfo)
}

/// Vim Compiled Spell Checking Dictionary (.spl, 2006) Bram Moolenaar.
pub(crate) fn verify_vim_spell_spl(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_VIM_SPELL_SPL_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"VIMspell"),
        matches!(u8_at(prefix, 8)?, b'1' | b'2'),
        get_slice(prefix, 18, 28)? == [0u8; 10]
    );
    Some(Label::VimSpellSpl)
}

/// Digital Extremes Evolution Engine (Warframe) Cache TOC Index (`.toc`).
pub(crate) fn verify_warframe_toc(prefix: &[u8], size: u64) -> Option<Label> {
    const WARFRAME_TOC_MAGIC: &[u8; 4] = hex2bin!("4E C6 67 18"); // @0..4: Evolution Engine TOC magic
    const VERSION_OFF: usize = 4; // @4..8: u32 LE TOC format version (e.g., 20 = 0x14)
    const RESERVED_OFF: usize = 8; // @8..16: reserved zero u64
    const FILETIME_HI_OFF: usize = 20; // @20..24: Windows FILETIME upper 32 bits (~2000..2050)

    require!(
        size >= MIN_WARFRAME_TOC_SIZE,
        prefix.len() >= 32,
        starts(prefix, WARFRAME_TOC_MAGIC),
        (1..=64).contains(&le32(prefix, VERSION_OFF)?),
        le64(prefix, RESERVED_OFF) == Some(0),
        (0x01C0_0000..=0x01F0_0000).contains(&le32(prefix, FILETIME_HI_OFF)?)
    );
    Some(Label::WarframeToc)
}

/// Microsoft Windows Property / App Cache Database (`{GUID}.*.ver0x*.db`).
pub(crate) fn verify_windows_caches_db(prefix: &[u8], size: u64) -> Option<Label> {
    const CACHES_DB_MAGIC: &[u8; 8] = hex2bin!("00 AC AC 00 60 00 00 00"); // @0..8: magic 0x00ACAC00 + 0x60 header size
    const TABLE_OFF_FIELD: &[u8; 8] = hex2bin!("50 00 00 00 00 00 00 00"); // @24..32: fixed u64 table offset 0x50
    const LCID_OFF: usize = 20; // @20..24: Windows LCID (e.g., 0x0409 en-US)

    require!(
        size >= MIN_WINDOWS_CACHES_DB_SIZE,
        prefix.len() >= 32,
        starts(prefix, CACHES_DB_MAGIC),
        get_slice(prefix, 9, 12)? == [0, 0, 0],
        le16(prefix, 14) == Some(1),
        le16(prefix, 18) == Some(1),
        (1..=0x7FFF).contains(&le16(prefix, LCID_OFF)?),
        le16(prefix, LCID_OFF + 2) == Some(0),
        eq_at(prefix, 24, TABLE_OFF_FIELD)
    );
    Some(Label::WindowsCachesDb)
}

/// CD Projekt RED REDengine 3 / The Witcher 3 Localization Strings (`.w3strings`, 2015).
pub(crate) fn verify_witcher3_w3strings(prefix: &[u8], size: u64) -> Option<Label> {
    const MAGIC: &[u8; 4] = b"RTSW"; // @0..4: "RTSW" ("WSTR" reversed)
    const VERSION_OFF: usize = 4; // @4..8: u32 LE REDengine version (100..=1000; upper 2 bytes are 0x0000)

    require!(
        size >= MIN_WITCHER3_W3STRINGS_SIZE,
        prefix.len() >= 28,
        starts(prefix, MAGIC),
        (100..=1000).contains(&le32(prefix, VERSION_OFF)?)
    );
    Some(Label::Witcher3W3strings)
}

/// Blizzard Entertainment World of Warcraft Client Database (`.dbc`/`.db2`, `WDBC`/`WDB2`..`WDC5`, 2004 Blizzard Entertainment).
pub(crate) fn verify_wow_dbc(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_WOW_DBC_SIZE, prefix.len() >= 20);
    let tag = get_slice(prefix, 0, 4)?;
    if tag == b"WDBC" {
        let rec_count = le32(prefix, 4)? as u64;
        let field_count = le32(prefix, 8)?;
        let rec_size = le32(prefix, 12)? as u64;
        let str_size = le32(prefix, 16)? as u64;
        require!(
            (1..=512).contains(&field_count),
            (4..=65536).contains(&rec_size),
            str_size >= 1,
            20u64.saturating_add(rec_count.saturating_mul(rec_size)).saturating_add(str_size)
                == size
        );
        return Some(Label::WowDbc);
    }
    if matches!(tag, b"WDB2" | b"WDB5" | b"WDB6" | b"WDC1" | b"WDC2" | b"WDC3" | b"WDC4" | b"WDC5")
    {
        let field_count = le32(prefix, 8)?;
        let rec_size = le32(prefix, 12)?;
        require!(
            (1..=512).contains(&field_count),
            (1..=65536).contains(&rec_size),
            has_non_ascii_text_byte(prefix.get(..20)?)
        );
        return Some(Label::WowDbc);
    }
    None
}
