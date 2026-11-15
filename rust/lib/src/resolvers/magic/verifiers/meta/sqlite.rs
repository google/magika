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

//! SQLite 3 database metaformat verifiers (`sqlite`, `geopackage`, `mbtiles`, `qgis`) and
//! bounded `sqlite_schema` B-tree subformat identification.

use crate::resolvers::magic::*;
use crate::{ContentType as Label, Input};

pub(crate) const SQLITE3_MAGIC: &[u8; 16] = b"SQLite format 3\0"; // @0..16: SQLite 3 header magic
const AUTOCAD_FDO_PAGE_SIZE: u32 = 51200; // AutoCAD Map 3D FDO (.sdf) custom SQLite 3 page size (0xC800)
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

/// SQLite 3 database container: header `"SQLite format 3\0"` at `@0..16`, power-of-two page size
/// at `@16`, format write/read versions `1|2` at `@18..20`, payload fractions `64, 32, 32` at
/// `@21..24`, schema format `0..=4` at `@44`, and text encoding `0..=3` at `@56`; dispatches on
/// `application_id` at `@68` between `geopackage`, `qgis`, `mbtiles`, and `sqlite`.
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
        page_size.is_power_of_two() || page_size == AUTOCAD_FDO_PAGE_SIZE,
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
    if be32(prefix, 60)? == MONOTONE_USER_VERSION {
        return with_subformat(Label::Sqlite, &SUB_MONOTONE_REPO);
    }
    Some(Label::Sqlite)
}

/// Disambiguates SQLite 3 database containers (`geopackage`, `qgis`, `mbtiles`, `sqlite`).
#[inline]
pub(crate) fn sqlite_or_geopackage(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| geopackage(prefix)).or_else(|| probe(|| sqlite_family(prefix, size)))
}

// ============================================================================
// Bounded `sqlite_schema` B-tree loader & FNV-1a64 schema subformat matcher
// ============================================================================

const DB_HEADER_LEN: usize = 100; // @0..100: SQLite 3 database header
const PAGE_SIZE_OFF: usize = 16; // @16: u16 BE — page size in bytes (`1` encodes `65536`)
const RESERVED_SPACE_OFF: usize = 20; // @20: u8 — reserved bytes at the end of every page
const TEXT_ENCODING_OFF: usize = 56; // @56: u32 BE — 1=UTF-8, 2=UTF-16LE, 3=UTF-16BE
const MIN_USABLE_PAGE_SIZE: usize = 480; // SQLite 3 minimum usable page size

const BTREE_INTERIOR_TABLE: u8 = 0x05; // Table B-tree interior page (12-byte header)
const BTREE_LEAF_TABLE: u8 = 0x0D; // Table B-tree leaf page (8-byte header)
const BTREE_LEAF_HDR_LEN: usize = 8;
const BTREE_INTERIOR_HDR_LEN: usize = 12;

const MAX_VISITED_PAGES: usize = 64;
const MAX_BTREE_DEPTH: u8 = 16;
const MAX_STACK: usize = 128;
const MAX_CELLS_PER_PAGE: usize = 256;
const MAX_SCHEMA_ROWS: usize = 512;
const MAX_TABLES: usize = 256;
const ARENA_SIZE: usize = 4096;
const CELL_PREFIX_BUF_LEN: usize = 256;
const MAX_BOOKMARKS: usize = 3;
const MAX_SQL_LEN: usize = 4096;
const MAX_OVERFLOW_HOPS: usize = 4;

const FNV1A64_OFFSET_BASIS: u64 = 0xCBF2_9CE4_8422_2325;
const FNV1A64_PRIME: u64 = 0x0000_0100_0000_01B3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum TextEncoding {
    Utf8,
    Utf16Le,
    Utf16Be,
}

#[derive(Clone, Copy)]
struct TableSqlBookmark {
    page_file_offset: u64,
    cell_off: usize,
    rec_start: usize,
    sql_offset_in_rec: usize,
    sql_len: usize,
    payload_size: usize,
    local_payload: usize,
}

impl TableSqlBookmark {
    const EMPTY: Self = Self {
        page_file_offset: 0,
        cell_off: 0,
        rec_start: 0,
        sql_offset_in_rec: 0,
        sql_len: 0,
        payload_size: 0,
        local_payload: 0,
    };
}

/// Rolling window size for [`PageReader`] (one standard SQLite page).
const PAGE_READER_WINDOW: usize = 4096;
/// Hard ceiling on 4 KiB window refills per file (`128 * 4 KiB = 512 KiB` max I/O).
const MAX_WINDOW_READS: usize = 128;

/// Rolling 4 KiB stack-cached reader over [`Input`] for positioned B-tree page reads.
struct PageReader<'a, 'b> {
    input: &'a mut Input<'b>,
    win_start: u64,
    win_len: usize,
    window_reads: usize,
    buf: [u8; PAGE_READER_WINDOW],
}

impl<'a, 'b> PageReader<'a, 'b> {
    fn new(input: &'a mut Input<'b>) -> Self {
        Self {
            input,
            win_start: u64::MAX,
            win_len: 0,
            window_reads: 0,
            buf: [0u8; PAGE_READER_WINDOW],
        }
    }

    fn read_up_to(&mut self, mut offset: u64, mut dst: &mut [u8]) -> usize {
        let mut total_read = 0usize;
        while !dst.is_empty() {
            let win_end = self.win_start.saturating_add(self.win_len as u64);
            if offset < self.win_start || offset >= win_end {
                if self.window_reads >= MAX_WINDOW_READS {
                    break;
                }
                let aligned = offset & !((PAGE_READER_WINDOW - 1) as u64);
                let n = self.input.read_chunk_at(aligned, &mut self.buf);
                self.window_reads += 1;
                self.win_start = aligned;
                self.win_len = n;
                if offset >= aligned.saturating_add(n as u64) {
                    break;
                }
            }
            let rel = (offset - self.win_start) as usize;
            let take = (self.win_len - rel).min(dst.len());
            dst[..take].copy_from_slice(&self.buf[rel..rel + take]);
            offset += take as u64;
            dst = &mut dst[take..];
            total_read += take;
        }
        total_read
    }

    fn read_exact(&mut self, offset: u64, dst: &mut [u8]) -> bool {
        self.read_up_to(offset, dst) == dst.len()
    }

    fn read_be32(&mut self, offset: u64) -> Option<u32> {
        let mut b = [0u8; 4];
        self.read_exact(offset, &mut b).then(|| u32::from_be_bytes(b))
    }
}

const fn fnv1a64_step(mut h: u64, bytes: &[u8]) -> u64 {
    let mut i = 0;
    while i < bytes.len() {
        h ^= bytes[i] as u64;
        h = h.wrapping_mul(FNV1A64_PRIME);
        i += 1;
    }
    h
}

const fn fnv1a64(bytes: &[u8]) -> u64 {
    fnv1a64_step(FNV1A64_OFFSET_BASIS, bytes)
}

/// Decodes an SQLite 3 big-endian varint (1 to 9 bytes) at `off` in `buf`.
fn parse_varint(buf: &[u8], off: usize) -> Option<(u64, usize)> {
    let mut val = 0u64;
    for i in 0..9 {
        let b = *buf.get(off + i)?;
        if i == 8 {
            val = (val << 8) | u64::from(b);
            return Some((val, off + 9));
        }
        val = (val << 7) | u64::from(b & 0x7F);
        if b < 0x80 {
            return Some((val, off + i + 1));
        }
    }
    None
}

/// Byte length of an SQLite 3 record column for the given serial type code.
fn serial_type_len(st: u64) -> Option<usize> {
    match st {
        0 | 8 | 9 | 10 | 11 => Some(0),
        1 => Some(1),
        2 => Some(2),
        3 => Some(3),
        4 => Some(4),
        5 => Some(6),
        6 | 7 => Some(8),
        n if n >= 12 => usize::try_from((n - 12) / 2).ok(),
        _ => None,
    }
}

/// Computes local inline payload size `P_local` for a table B-tree leaf cell (SQLite 3 §1.6).
fn table_leaf_local_payload(payload_size: u64, usable_size: usize) -> usize {
    let max_local = usable_size - 35;
    if payload_size <= max_local as u64 {
        return payload_size as usize;
    }
    let min_local = ((usable_size - 12) * 32 / 255).saturating_sub(23);
    let denom = (usable_size - 4) as u64;
    let k = min_local as u64 + ((payload_size - min_local as u64) % denom);
    if k <= max_local as u64 { k as usize } else { min_local }
}

/// Returns `true` when `raw` decodes as ASCII `"table"` (ignoring trailing NUL padding).
fn is_table_type(raw: &[u8], enc: TextEncoding) -> bool {
    match enc {
        TextEncoding::Utf8 => {
            let mut end = raw.len();
            while end > 0 && raw[end - 1] == 0 {
                end -= 1;
            }
            raw[..end].eq_ignore_ascii_case(b"table")
        }
        TextEncoding::Utf16Le => {
            let mut end = raw.len() & !1;
            while end >= 2 && raw[end - 2] == 0 && raw[end - 1] == 0 {
                end -= 2;
            }
            end == 10
                && raw[1] == 0
                && raw[3] == 0
                && raw[5] == 0
                && raw[7] == 0
                && raw[9] == 0
                && [raw[0], raw[2], raw[4], raw[6], raw[8]].eq_ignore_ascii_case(b"table")
        }
        TextEncoding::Utf16Be => {
            let mut end = raw.len() & !1;
            while end >= 2 && raw[end - 2] == 0 && raw[end - 1] == 0 {
                end -= 2;
            }
            end == 10
                && raw[0] == 0
                && raw[2] == 0
                && raw[4] == 0
                && raw[6] == 0
                && raw[8] == 0
                && [raw[1], raw[3], raw[5], raw[7], raw[9]].eq_ignore_ascii_case(b"table")
        }
    }
}

/// Decodes raw SQLite text bytes into UTF-8 inside `out` (trims trailing NUL units).
/// Returns the written byte length, or `None` if `out` has insufficient capacity.
fn decode_text(raw: &[u8], enc: TextEncoding, out: &mut [u8]) -> Option<usize> {
    match enc {
        TextEncoding::Utf8 => {
            let mut end = raw.len();
            while end > 0 && raw[end - 1] == 0 {
                end -= 1;
            }
            if end > out.len() {
                return None;
            }
            out[..end].copy_from_slice(&raw[..end]);
            Some(end)
        }
        TextEncoding::Utf16Le | TextEncoding::Utf16Be => {
            let mut end = raw.len() & !1;
            while end >= 2 && raw[end - 2] == 0 && raw[end - 1] == 0 {
                end -= 2;
            }
            let units = raw[..end].as_chunks::<2>().0.iter().map(|&c| match enc {
                TextEncoding::Utf16Le => u16::from_le_bytes(c),
                TextEncoding::Utf16Be => u16::from_be_bytes(c),
                TextEncoding::Utf8 => unreachable!(),
            });
            let mut pos = 0usize;
            let mut enc_buf = [0u8; 4];
            for ch in char::decode_utf16(units) {
                let c = ch.unwrap_or('\u{FFFD}');
                let s = c.encode_utf8(&mut enc_buf).as_bytes();
                if pos + s.len() > out.len() {
                    return None;
                }
                out[pos..pos + s.len()].copy_from_slice(s);
                pos += s.len();
            }
            Some(pos)
        }
    }
}

/// Appends a normalized lowercase SQLite identifier to `arena`, escaping delimiter characters
/// (`\n`, `\r`, `|`, `\`) so single identifiers cannot forge multi-entry hashes.
fn push_normalized_ident(raw_utf8: &[u8], arena: &mut [u8], pos: &mut usize) -> Option<(u16, u16)> {
    let mut start_trim = 0usize;
    let mut end_trim = raw_utf8.len();
    while start_trim < end_trim && raw_utf8[start_trim].is_ascii_whitespace() {
        start_trim += 1;
    }
    while end_trim > start_trim && raw_utf8[end_trim - 1].is_ascii_whitespace() {
        end_trim -= 1;
    }
    let slice = &raw_utf8[start_trim..end_trim];
    if slice.is_empty() {
        return Some((*pos as u16, *pos as u16));
    }
    let start = *pos;
    for &b in slice {
        let lower = b.to_ascii_lowercase();
        match lower {
            b'\n' => {
                if *pos + 2 > arena.len() {
                    *pos = start;
                    return None;
                }
                arena[*pos] = b'\\';
                arena[*pos + 1] = b'n';
                *pos += 2;
            }
            b'\r' => {
                if *pos + 2 > arena.len() {
                    *pos = start;
                    return None;
                }
                arena[*pos] = b'\\';
                arena[*pos + 1] = b'r';
                *pos += 2;
            }
            b'|' | b'\\' => {
                if *pos + 2 > arena.len() {
                    *pos = start;
                    return None;
                }
                arena[*pos] = b'\\';
                arena[*pos + 1] = lower;
                *pos += 2;
            }
            0 => {}
            _ => {
                if *pos + 1 > arena.len() {
                    *pos = start;
                    return None;
                }
                arena[*pos] = lower;
                *pos += 1;
            }
        }
    }
    Some((start as u16, *pos as u16))
}

/// SQLite internal and Android locale bookkeeping tables excluded from schema fingerprinting.
fn is_bookkeeping_table(name_lower: &[u8]) -> bool {
    name_lower.starts_with(b"sqlite_") || name_lower == b"android_metadata"
}

/// Identifies a SQLite 3 database's application subformat by walking its `sqlite_schema` B-tree
/// (rooted at page 1) with strict page/row/byte caps and matching normalized table-set and
/// column-list FNV-1a64 hashes.
pub(crate) fn sqlite_schema_subformat(file: &mut Input<'_>) -> Option<&'static Subformat> {
    let mut reader = PageReader::new(file);
    let mut hdr = [0u8; DB_HEADER_LEN];
    require!(reader.read_exact(0, &mut hdr), starts(&hdr, SQLITE3_MAGIC));

    let page_raw = be16(&hdr, PAGE_SIZE_OFF)?;
    let page_size: usize = if page_raw == 1 { 65536 } else { page_raw as usize };
    require!(
        (512..=65536).contains(&page_size),
        page_size.is_power_of_two() || page_size == AUTOCAD_FDO_PAGE_SIZE as usize
    );

    let reserved = hdr[RESERVED_SPACE_OFF] as usize;
    let usable_size = page_size.checked_sub(reserved)?;
    require!(usable_size >= MIN_USABLE_PAGE_SIZE);

    let enc = match be32(&hdr, TEXT_ENCODING_OFF)? {
        0 | 1 => TextEncoding::Utf8,
        2 => TextEncoding::Utf16Le,
        3 => TextEncoding::Utf16Be,
        _ => return None,
    };

    let mut visited = [0u32; MAX_VISITED_PAGES];
    let mut visited_count = 0usize;
    let mut stack = [(0u32, 0u8); MAX_STACK];
    stack[0] = (1, 0);
    let mut stack_len = 1usize;

    let mut arena = [0u8; ARENA_SIZE];
    let mut arena_pos = 0usize;
    let mut t_spans = [(0u16, 0u16); MAX_TABLES];
    let mut t_count = 0usize;
    let mut bookmarks = [TableSqlBookmark::EMPTY; MAX_BOOKMARKS];
    let mut b_count = 0usize;

    let mut rows_inspected = 0usize;
    let mut schema_complete = true;
    let mut ptr_buf = [0u8; MAX_CELLS_PER_PAGE * 2];
    let mut cell_offs = [0u16; MAX_CELLS_PER_PAGE];
    let mut cell_buf = [0u8; CELL_PREFIX_BUF_LEN];
    let mut name_utf8 = [0u8; CELL_PREFIX_BUF_LEN];

    'walk: while stack_len > 0 {
        stack_len -= 1;
        let (pgno, depth) = stack[stack_len];
        if pgno == 0 || visited[..visited_count].contains(&pgno) {
            schema_complete = false;
            continue;
        }
        if visited_count >= MAX_VISITED_PAGES || depth >= MAX_BTREE_DEPTH {
            schema_complete = false;
            break;
        }
        visited[visited_count] = pgno;
        visited_count += 1;

        let Some(page_file_offset) = u64::from(pgno - 1).checked_mul(page_size as u64) else {
            schema_complete = false;
            continue;
        };
        let hdr_rel = if pgno == 1 { DB_HEADER_LEN } else { 0 };
        let hdr_off = page_file_offset + hdr_rel as u64;

        let mut ph = [0u8; BTREE_LEAF_HDR_LEN];
        if !reader.read_exact(hdr_off, &mut ph) {
            schema_complete = false;
            continue;
        }
        let page_type = ph[0];
        let cell_count = u16::from_be_bytes([ph[3], ph[4]]) as usize;
        if cell_count > MAX_CELLS_PER_PAGE {
            schema_complete = false;
        }
        let bounded_cells = cell_count.min(MAX_CELLS_PER_PAGE);

        match page_type {
            BTREE_INTERIOR_TABLE => {
                let Some(right_child) = reader.read_be32(hdr_off + BTREE_LEAF_HDR_LEN as u64)
                else {
                    schema_complete = false;
                    continue;
                };
                if right_child != 0 {
                    if stack_len < MAX_STACK {
                        stack[stack_len] = (right_child, depth + 1);
                        stack_len += 1;
                    } else {
                        schema_complete = false;
                    }
                }
                let ptr_array_off = hdr_off + BTREE_INTERIOR_HDR_LEN as u64;
                let ptr_bytes_want = bounded_cells * 2;
                let ptr_bytes_got =
                    reader.read_up_to(ptr_array_off, &mut ptr_buf[..ptr_bytes_want]);
                if ptr_bytes_got < ptr_bytes_want {
                    schema_complete = false;
                }
                let ptr_count = ptr_bytes_got / 2;
                for i in (0..ptr_count).rev() {
                    let cell_off =
                        u16::from_be_bytes([ptr_buf[i * 2], ptr_buf[i * 2 + 1]]) as usize;
                    if cell_off < hdr_rel + BTREE_INTERIOR_HDR_LEN || cell_off + 4 > usable_size {
                        schema_complete = false;
                        continue;
                    }
                    let Some(child_pg) = reader.read_be32(page_file_offset + cell_off as u64)
                    else {
                        schema_complete = false;
                        continue;
                    };
                    if child_pg != 0 {
                        if stack_len < MAX_STACK {
                            stack[stack_len] = (child_pg, depth + 1);
                            stack_len += 1;
                        } else {
                            schema_complete = false;
                        }
                    }
                }
            }
            BTREE_LEAF_TABLE => {
                let ptr_array_off = hdr_off + BTREE_LEAF_HDR_LEN as u64;
                let ptr_bytes_want = bounded_cells * 2;
                let ptr_bytes_got =
                    reader.read_up_to(ptr_array_off, &mut ptr_buf[..ptr_bytes_want]);
                if ptr_bytes_got < ptr_bytes_want {
                    schema_complete = false;
                }
                let ptr_count = ptr_bytes_got / 2;
                let mut valid_cells = 0usize;
                for i in 0..ptr_count {
                    let ptr = u16::from_be_bytes([ptr_buf[i * 2], ptr_buf[i * 2 + 1]]);
                    let cell_off = ptr as usize;
                    if cell_off < hdr_rel + BTREE_LEAF_HDR_LEN || cell_off >= usable_size {
                        schema_complete = false;
                        continue;
                    }
                    cell_offs[valid_cells] = ptr;
                    valid_cells += 1;
                }
                // Visit cells in ascending file-offset order so PageReader's 4 KiB rolling
                // window touches each 4 KiB slice of a large (8–64 KiB) page at most once.
                cell_offs[..valid_cells].sort_unstable();

                for &cell_ptr in &cell_offs[..valid_cells] {
                    if rows_inspected >= MAX_SCHEMA_ROWS || t_count >= MAX_TABLES {
                        schema_complete = false;
                        break 'walk;
                    }
                    let cell_off = cell_ptr as usize;
                    rows_inspected += 1;

                    let max_read = (usable_size - cell_off).min(CELL_PREFIX_BUF_LEN);
                    let got = reader
                        .read_up_to(page_file_offset + cell_off as u64, &mut cell_buf[..max_read]);
                    if got < 8 {
                        schema_complete = false;
                        continue;
                    }
                    let cell = &cell_buf[..got];
                    let Some((payload_size, pos1)) = parse_varint(cell, 0) else {
                        schema_complete = false;
                        continue;
                    };
                    let Some((_rowid, rec_start)) = parse_varint(cell, pos1) else {
                        schema_complete = false;
                        continue;
                    };
                    let local_payload = table_leaf_local_payload(payload_size, usable_size);
                    let rec_end = rec_start.saturating_add(local_payload).min(cell.len());
                    let Some((hdr_size_u64, mut hpos)) = parse_varint(cell, rec_start) else {
                        schema_complete = false;
                        continue;
                    };
                    let Ok(hdr_size) = usize::try_from(hdr_size_u64) else {
                        schema_complete = false;
                        continue;
                    };
                    if hdr_size < 2 || rec_start.saturating_add(hdr_size) > rec_end {
                        schema_complete = false;
                        continue;
                    }
                    let hdr_end = rec_start + hdr_size;
                    let mut col_lens = [0usize; 5];
                    let mut col_count = 0usize;
                    while hpos < hdr_end && col_count < 5 {
                        let Some((st, next)) = parse_varint(cell, hpos) else {
                            col_count = 0;
                            break;
                        };
                        let Some(clen) = serial_type_len(st) else {
                            col_count = 0;
                            break;
                        };
                        col_lens[col_count] = clen;
                        col_count += 1;
                        hpos = next;
                    }
                    if col_count < 2 {
                        continue;
                    }
                    let type_start = hdr_end;
                    let type_end = type_start.saturating_add(col_lens[0]);
                    let name_start = type_end;
                    let name_end = name_start.saturating_add(col_lens[1]);
                    if name_end > rec_end {
                        schema_complete = false;
                        continue;
                    }
                    if !is_table_type(&cell[type_start..type_end], enc) {
                        continue;
                    }
                    let Some(nlen) = decode_text(&cell[name_start..name_end], enc, &mut name_utf8)
                    else {
                        schema_complete = false;
                        continue;
                    };
                    let saved_pos = arena_pos;
                    let Some((s, e)) =
                        push_normalized_ident(&name_utf8[..nlen], &mut arena, &mut arena_pos)
                    else {
                        schema_complete = false;
                        break 'walk;
                    };
                    let norm_name = &arena[s as usize..e as usize];
                    if norm_name.is_empty() || is_bookkeeping_table(norm_name) {
                        arena_pos = saved_pos;
                        continue;
                    }
                    let already_seen = t_spans[..t_count]
                        .iter()
                        .any(|&(ps, pe)| &arena[ps as usize..pe as usize] == norm_name);
                    if already_seen {
                        arena_pos = saved_pos;
                        continue;
                    }
                    t_spans[t_count] = (s, e);
                    t_count += 1;

                    if b_count < MAX_BOOKMARKS && col_count == 5 && col_lens[4] > 0 {
                        let sql_off_in_rec =
                            hdr_size + col_lens[0] + col_lens[1] + col_lens[2] + col_lens[3];
                        if let Ok(psize) = usize::try_from(payload_size) {
                            bookmarks[b_count] = TableSqlBookmark {
                                page_file_offset,
                                cell_off,
                                rec_start,
                                sql_offset_in_rec: sql_off_in_rec,
                                sql_len: col_lens[4],
                                payload_size: psize,
                                local_payload,
                            };
                            b_count += 1;
                        }
                    }
                }
            }
            _ => {
                schema_complete = false;
            }
        }
    }

    if t_count == 0 {
        return None;
    }

    let spans = &mut t_spans[..t_count];
    spans.sort_unstable_by(|&(as_, ae), &(bs, be)| {
        arena[as_ as usize..ae as usize].cmp(&arena[bs as usize..be as usize])
    });

    let mut s1_hash = FNV1A64_OFFSET_BASIS;
    for (idx, &(s, e)) in spans.iter().enumerate() {
        if idx > 0 {
            s1_hash = fnv1a64_step(s1_hash, b"\n");
        }
        s1_hash = fnv1a64_step(s1_hash, &arena[s as usize..e as usize]);
    }

    if schema_complete {
        if t_count <= MAX_BOOKMARKS
            && b_count == t_count
            && is_tier3_candidate(s1_hash)
            && let Some(sub) = resolve_tier3(
                &mut reader,
                page_size,
                usable_size,
                enc,
                s1_hash,
                &bookmarks[..b_count],
            )
        {
            return Some(sub);
        }
        if let Some(sub) = match_exact_s1_hash(s1_hash, enc) {
            return Some(sub);
        }
    }

    match_anchor_tables(&arena, spans, enc)
}

// ============================================================================
// Subformat descriptors
// ============================================================================

static SUB_ACORN_IMAGE: Subformat = Subformat::new("Acorn image (.acorn)");
static SUB_ACTIVEPRESENTER: Subformat = Subformat::new("ActivePresenter project (.approj)");
static SUB_ADOBE_ACROBAT_SYNC: Subformat =
    Subformat::new("Adobe Acrobat Cloud Synchronizer database");
static SUB_ADOBE_CC_DATATABLE: Subformat = Subformat::new("Adobe Acrobat Creative Cloud cache");
static SUB_ADOBE_CC_PCD_MASTER: Subformat =
    Subformat::new("Adobe Creative Cloud PCD master database (pcd.db)");
static SUB_ADOBE_CC_PIM: Subformat =
    Subformat::new("Adobe Creative Cloud package manager database (pim.db)");
static SUB_ADOBE_CRCOMMON: Subformat = Subformat::new("Adobe Acrobat CRCommon cache");
static SUB_ADOBE_LIGHTROOM: Subformat = Subformat::new("Adobe Lightroom catalog (.lrcat)");
static SUB_ADOBE_LIGHTROOM_TEMP: Subformat =
    Subformat::new("Adobe Lightroom temporary import catalog");
static SUB_ADOBE_MEDIA_DB: Subformat =
    Subformat::new("Adobe Creative Suite / Creative Cloud Installer Media database (Media_db.db)");
static SUB_ADOBE_MUSE: Subformat = Subformat::new("Adobe Muse project (.muse)");
static SUB_ADOBE_OPM_DB: Subformat = Subformat::new("Adobe Creative Cloud OOBE policy database");
static SUB_ADOBE_PCD_CACHE: Subformat = Subformat::new("Adobe Creative Cloud PCD licensing cache");
static SUB_ADOBE_READER_MSGS: Subformat = Subformat::new("Adobe Acrobat Reader messaging cache");
static SUB_ADOBE_SHARED_DATA_EVENTS: Subformat =
    Subformat::new("Adobe Creative Cloud SharedDataEvents");
static SUB_AIRFLOW_DB: Subformat = Subformat::new("Apache Airflow metadata database");
static SUB_AMD_DXCACHE: Subformat = Subformat::new("AMD DirectX shader cache (.dxcache)");
static SUB_ANDROID_ACCOUNTS: Subformat =
    Subformat::new("Android AccountManager database (accounts.db)");
static SUB_ANDROID_AIRGUARD: Subformat =
    Subformat::new("AirGuard AirTag tracker detection database (attd_db)");
static SUB_ANDROID_BURNER: Subformat =
    Subformat::new("Burner Android communication database (burners.sqlite)");
static SUB_ANDROID_CALENDAR: Subformat =
    Subformat::new("Android CalendarProvider database (calendar.db)");
static SUB_ANDROID_CONTACTS: Subformat =
    Subformat::new("Android Contacts Provider database (contacts2.db)");
static SUB_ANDROID_DIGITAL_WELLBEING: Subformat =
    Subformat::new("Android Digital Wellbeing usage database (app_usage)");
static SUB_ANDROID_DOWNLOADS: Subformat =
    Subformat::new("Android DownloadProvider database (downloads.db)");
static SUB_ANDROID_KEYSTORE: Subformat = Subformat::new("Android Keystore 2.0 database");
static SUB_ANDROID_MEDIA: Subformat = Subformat::new("Android MediaProvider database");
static SUB_ANDROID_MMSSMS: Subformat =
    Subformat::new("Android Telephony MMS/SMS database (mmssms.db)");
static SUB_ANDROID_ROOM: Subformat = Subformat::new("Android Jetpack Room database");
static SUB_ANDROID_SETTINGS: Subformat =
    Subformat::new("Android SettingsProvider database (settings.db)");
static SUB_ANDROID_SIMPLESTORAGE: Subformat =
    Subformat::new("Android System Intelligence launch database (SimpleStorage)");
static SUB_ANDROID_TELEPHONY: Subformat =
    Subformat::new("Android Telephony Carriers database (telephony.db)");
static SUB_ANDROID_TURBO_BATTERY: Subformat =
    Subformat::new("Android Device Health Services battery database (turbo.db)");
static SUB_ANDROID_TV: Subformat = Subformat::new("Android TvProvider database");
static SUB_ANDROID_TWITTER: Subformat = Subformat::new("Twitter / X Android database (twitter.db)");
static SUB_ANDROID_VIBER: Subformat =
    Subformat::new("Viber Messenger Android database (viber_data)");
static SUB_ANDROID_WEBVIEW_CACHE: Subformat =
    Subformat::new("Android WebView HTTP Cache (webviewCache.db)");
static SUB_ANDROID_WEBVIEW_CREDS: Subformat =
    Subformat::new("Android WebView Credentials database (webview.db)");
static SUB_ANDROID_WORKMANAGER: Subformat = Subformat::new("AndroidX WorkManager database");
static SUB_ANGR_DB: Subformat = Subformat::new("angr binary analysis database (.adb)");
static SUB_ANKI_1_DECK: Subformat = Subformat::new("Anki 1.x flashcard deck (.anki)");
static SUB_ANKI_2_DECK: Subformat = Subformat::new("Anki flashcard deck (.anki2)");
static SUB_ANTIGRAVITY_CLI_SESSION: Subformat =
    Subformat::new("Google Antigravity CLI session database (session.db)");
static SUB_APPLE_ACCOUNTS3: Subformat =
    Subformat::new("Apple Accounts3 CoreData database (Accounts3.sqlite)");
static SUB_APPLE_ADDRESSBOOK: Subformat =
    Subformat::new("Apple AddressBook database (AddressBook.sqlitedb)");
static SUB_APPLE_APS_DB: Subformat =
    Subformat::new("Apple Push Notification Service database (aps.db)");
static SUB_APPLE_CALENDAR: Subformat =
    Subformat::new("Apple Calendar database (Calendar.sqlitedb)");
static SUB_APPLE_CALL_HISTORY: Subformat =
    Subformat::new("Apple iOS Call History database (CallHistory.storedata)");
static SUB_APPLE_CFURLCACHE: Subformat =
    Subformat::new("Apple CFURLCache network cache (Cache.db)");
static SUB_APPLE_COREDATA: Subformat = Subformat::new("Apple CoreData persistent store");
static SUB_APPLE_DATAUSAGE: Subformat =
    Subformat::new("Apple Data Usage CoreData database (DataUsage.sqlite)");
static SUB_APPLE_DOC_REVISIONS: Subformat =
    Subformat::new("Apple macOS Document Revisions database");
static SUB_APPLE_FCP_EVENT: Subformat =
    Subformat::new("Apple Final Cut Pro / iMovie Event CoreData database (.fcpevent)");
static SUB_APPLE_HEALTHKIT: Subformat =
    Subformat::new("Apple iOS HealthKit database (healthdb_secure.sqlite)");
static SUB_APPLE_IMESSAGE: Subformat = Subformat::new("Apple iMessage / SMS database (sms.db)");
static SUB_APPLE_IOS_POWERLOG: Subformat =
    Subformat::new("Apple iOS Powerlog telemetry database (CurrentPowerlog.PLSQL)");
static SUB_APPLE_IPHOTO_LIB: Subformat = Subformat::new("Apple iPhoto / Aperture library database");
static SUB_APPLE_IPHOTO_ROOTPIXELS: Subformat =
    Subformat::new("Apple iPhoto / Aperture RootPixels database");
static SUB_APPLE_ITUNES_EXTRAS: Subformat =
    Subformat::new("Apple iTunes Library Extras CDDB database (.itdb)");
static SUB_APPLE_KNOWLEDGEC: Subformat =
    Subformat::new("Apple KnowledgeC CoreData database (knowledgeC.db)");
static SUB_APPLE_MAIL_ENVELOPE: Subformat = Subformat::new("Apple Mail Envelope Index database");
static SUB_APPLE_NETUSAGE: Subformat =
    Subformat::new("Apple Network Usage CoreData database (netusage.sqlite)");
static SUB_APPLE_NOTES_LEGACY: Subformat =
    Subformat::new("Apple macOS Legacy Notes CoreData database");
static SUB_APPLE_NOTES_MODERN: Subformat =
    Subformat::new("Apple Notes CoreData database (NoteStore.sqlite)");
static SUB_APPLE_NOTIF_CENTER: Subformat =
    Subformat::new("Apple macOS Notification Center database");
static SUB_APPLE_PHOTOS_PREVIEW: Subformat = Subformat::new("Apple Photos preview cache");
static SUB_APPLE_QUARANTINE: Subformat = Subformat::new("Apple macOS QuarantineEventsV2 database");
static SUB_APPLE_SCREENTIME: Subformat = Subformat::new("Apple Screen Time CoreData database");
static SUB_APPLE_TCC: Subformat = Subformat::new("Apple TCC permissions database (TCC.db)");
static SUB_APPLE_WEBKIT_DB: Subformat =
    Subformat::new("Apple WebKit database info store (__WebKitDatabaseInfoTable__)");
static SUB_APSIM_DB: Subformat = Subformat::new("APSIM simulator database");
static SUB_AUDACITY_3: Subformat =
    Subformat::with_ids("Audacity 3 project (.aup3)", "fmt/1826", None);
static SUB_AUTOCAD_DIMPTS: Subformat = Subformat::new("AutoCAD dimension points cache");
static SUB_AUTOCAD_FDO: Subformat = Subformat::new("AutoCAD Map 3D FDO spatial database (.sdf)");
static SUB_AUTOPSY_TSK: Subformat = Subformat::new("The Sleuth Kit / Autopsy forensic database");
static SUB_BALSAMIQ_BMPR: Subformat = Subformat::new("Balsamiq Mockups project (.bmpr)");
static SUB_BEEREF_SCENE: Subformat = Subformat::new("BeeRef scene (.bee)");
static SUB_BENTLEY_BESQLITE: Subformat = Subformat::new("Bentley BeSQLite database");
static SUB_BINARY_NINJA: Subformat = Subformat::new("Binary Ninja analysis database (.bndb)");
static SUB_BINDIFF_DB: Subformat = Subformat::new("BinDiff binary comparison database (.BinDiff)");
static SUB_BINDIFF_WORKSPACE: Subformat =
    Subformat::new("BinDiff Workspace database (.BinDiffWorkspace)");
static SUB_BITCOIN_WALLET: Subformat = Subformat::new("Bitcoin Core descriptor wallet");
static SUB_BNSQL_EXPORT: Subformat = Subformat::new("Binary Ninja SQLite export database (BNSQL)");
static SUB_CALIBRE_LIBRARY: Subformat = Subformat::new("Calibre library database");
static SUB_CERN_CMS_CONDDB: Subformat = Subformat::new("CERN CMS CondDB database");
static SUB_CEWE_PHOTOBOOK: Subformat = Subformat::new("CEWE Photobook project (.mcfx)");
static SUB_CHERRYTREE_NOTE: Subformat = Subformat::new("CherryTree note document (.ctb)");
static SUB_CHROMADB: Subformat = Subformat::new("ChromaDB vector database (chroma.sqlite3)");
static SUB_CHROMIUM_AFFILIATION: Subformat =
    Subformat::new("Chromium Password Affiliation database");
static SUB_CHROMIUM_APP_CACHE: Subformat = Subformat::new("Chromium HTML5 Application Cache");
static SUB_CHROMIUM_ATTRIBUTION: Subformat =
    Subformat::new("Chromium Attribution Reporting database");
static SUB_CHROMIUM_COOKIES: Subformat = Subformat::new("Chromium Cookies database");
static SUB_CHROMIUM_DIPS: Subformat =
    Subformat::new("Chromium Bounce Tracking Mitigations database (DIPS)");
static SUB_CHROMIUM_EXT_ACTIVITY: Subformat =
    Subformat::new("Chromium Extension Activity database");
static SUB_CHROMIUM_FAVICONS: Subformat = Subformat::new("Chromium Favicons database");
static SUB_CHROMIUM_FLEDGE: Subformat = Subformat::new("Chromium Protected Audience database");
static SUB_CHROMIUM_FPS: Subformat = Subformat::new("Chromium First-Party Sets database");
static SUB_CHROMIUM_HISTORY: Subformat = Subformat::new("Chromium History database");
static SUB_CHROMIUM_HISTORY_EMBEDDINGS: Subformat =
    Subformat::new("Chromium History Embeddings database");
static SUB_CHROMIUM_HISTORY_INDEX: Subformat =
    Subformat::new("Chromium full-text History Index database");
static SUB_CHROMIUM_INDEXEDDB: Subformat = Subformat::new("Chromium IndexedDB database");
static SUB_CHROMIUM_LOCALSTORAGE: Subformat = Subformat::new("Chromium LocalStorage (ItemTable)");
static SUB_CHROMIUM_LOGIN_DATA: Subformat = Subformat::new("Chromium Login Data database");
static SUB_CHROMIUM_NEL: Subformat = Subformat::new("Chromium Reporting and NEL database");
static SUB_CHROMIUM_NET_PREDICTOR: Subformat =
    Subformat::new("Chromium Network Action Predictor database");
static SUB_CHROMIUM_ORIGIN_CERTS: Subformat =
    Subformat::new("Chromium Origin Bound Certs database");
static SUB_CHROMIUM_POWER_BOOKMARKS: Subformat =
    Subformat::new("Chromium Power Bookmarks database");
static SUB_CHROMIUM_QUOTA_MANAGER: Subformat = Subformat::new("Chromium QuotaManager database");
static SUB_CHROMIUM_SHARED_STORAGE: Subformat = Subformat::new("Chromium Shared Storage database");
static SUB_CHROMIUM_SHORTCUTS: Subformat = Subformat::new("Chromium Shortcuts database");
static SUB_CHROMIUM_TOPICS: Subformat = Subformat::new("Chromium Browsing Topics database");
static SUB_CHROMIUM_TOP_SITES: Subformat = Subformat::new("Chromium Top Sites database");
static SUB_CHROMIUM_TRUST_TOKENS: Subformat = Subformat::new("Chromium Trust Tokens database");
static SUB_CHROMIUM_UKM: Subformat = Subformat::new("Chromium URL-Keyed Metrics database");
static SUB_CHROMIUM_WEB_DATA: Subformat = Subformat::new("Chromium Web Data database");
static SUB_CLIP_STUDIO_PAINT: Subformat = Subformat::new("Clip Studio Paint artwork (.clip)");
static SUB_COLMAP_DB: Subformat = Subformat::new("COLMAP 3D reconstruction database");
static SUB_CORSAIR_ICUE: Subformat = Subformat::new("Corsair iCUE package registry");
static SUB_DARKTABLE_DATA: Subformat = Subformat::new("Darktable styles and presets database");
static SUB_DARKTABLE_LIB: Subformat = Subformat::new("Darktable photo library database");
static SUB_DAW_VST_CACHE: Subformat = Subformat::new("Audio DAW VST plugin cache");
static SUB_DEBUGINFOD_DB: Subformat = Subformat::new("GNU elfutils debuginfod index");
static SUB_DENON_MARANTZ_DMU: Subformat =
    Subformat::new("Denon / Marantz media manual database (.dmu)");
static SUB_DFANALYZER_ZINDEX: Subformat = Subformat::new("DFAnalyzer zindex compressed file index");
static SUB_DIAPHORA_DB: Subformat = Subformat::new("Diaphora program diffing database (.diaphora)");
static SUB_DIGIKAM_DB: Subformat = Subformat::new("KDE digiKam photo database");
static SUB_DRAKON_DIAGRAM: Subformat = Subformat::new("DRAKON flowchart diagram (.drn)");
static SUB_DROPBOX_SYNC_HISTORY: Subformat = Subformat::new("Dropbox sync history database");
static SUB_EMBEDDED_MEDIA_LIB: Subformat = Subformat::new("Embedded media player library");
static SUB_EMBEDDED_RECENT_PLAY: Subformat = Subformat::new("Embedded media recent play database");
static SUB_ESRI_ST_GEOMETRY: Subformat = Subformat::new("Esri ST_Geometry database");
static SUB_FAIL2BAN_DB: Subformat = Subformat::new("Fail2Ban ban database");
static SUB_FAR_MANAGER_CONFIG: Subformat = Subformat::new("Far Manager v3 configuration database");
static SUB_FILES_BY_GOOGLE: Subformat = Subformat::new("Files by Google Android master database");
static SUB_FIREBASE_APP_MEASUREMENT: Subformat =
    Subformat::new("Google Play / Firebase App Measurement database");
static SUB_FIREFOX_ADDONS: Subformat =
    Subformat::new("Mozilla Firefox Add-ons metadata database (addons.sqlite)");
static SUB_FIREFOX_CONTENT_PREFS: Subformat =
    Subformat::new("Mozilla Firefox Content Preferences database (content-prefs.sqlite)");
static SUB_FIREFOX_COOKIES: Subformat =
    Subformat::new("Mozilla Firefox Cookies database (cookies.sqlite)");
static SUB_FIREFOX_DOWNLOADS: Subformat =
    Subformat::new("Mozilla Firefox Legacy Downloads database (downloads.sqlite)");
static SUB_FIREFOX_EXTENSIONS: Subformat =
    Subformat::new("Mozilla Firefox Extensions database (extensions.sqlite)");
static SUB_FIREFOX_FAVICONS: Subformat =
    Subformat::new("Mozilla Firefox Favicons database (favicons.sqlite)");
static SUB_FIREFOX_FORM_HISTORY: Subformat =
    Subformat::new("Mozilla Firefox Form History database (formhistory.sqlite)");
static SUB_FIREFOX_HEALTH_REPORT: Subformat =
    Subformat::new("Mozilla Firefox Health Report database (healthreport.sqlite)");
static SUB_FIREFOX_INDEXEDDB: Subformat = Subformat::new("Mozilla Firefox IndexedDB database");
static SUB_FIREFOX_OFFLINE_CACHE: Subformat =
    Subformat::new("Mozilla Firefox Offline Application Cache");
static SUB_FIREFOX_PERMISSIONS: Subformat =
    Subformat::new("Mozilla Firefox Site Permissions database (permissions.sqlite)");
static SUB_FIREFOX_PLACES: Subformat =
    Subformat::new("Mozilla Firefox Places database (places.sqlite)");
static SUB_FIREFOX_REMOTE_SETTINGS: Subformat =
    Subformat::new("Mozilla Firefox Remote Settings database (kinto.sqlite)");
static SUB_FIREFOX_SIGNONS: Subformat =
    Subformat::new("Mozilla Firefox Legacy Signons database (signons.sqlite)");
static SUB_FIREFOX_STORAGE: Subformat =
    Subformat::new("Mozilla Firefox Storage Manager database (storage.sqlite)");
static SUB_FIREFOX_URL_CLASSIFIER: Subformat =
    Subformat::new("Mozilla Firefox SafeBrowsing URL Classifier database");
static SUB_FIREFOX_WEBAPPSSTORE: Subformat =
    Subformat::new("Mozilla Firefox DOM LocalStorage database (webappsstore.sqlite)");
static SUB_FLARE_SHELLCODE_HASHES: Subformat =
    Subformat::new("FLARE shellcode API hashes database (sc_hashes.db)");
static SUB_FORENSICSTORE_ELEM: Subformat =
    Subformat::new("ForensicStore container (.forensicstore)");
static SUB_FOSSIL_BUNDLE: Subformat = Subformat::new("Fossil SCM bundle");
static SUB_FOSSIL_CHECKOUT: Subformat = Subformat::new("Fossil SCM checkout");
static SUB_FOSSIL_GLOBAL_CFG: Subformat = Subformat::new("Fossil SCM global configuration");
static SUB_FOSSIL_REPO: Subformat = Subformat::new("Fossil SCM repository");
static SUB_GAMEANALYTICS_CACHE: Subformat = Subformat::new("GameAnalytics SDK event cache");
static SUB_GEOPACKAGE_SCHEMA: Subformat = Subformat::new("OGC GeoPackage (schema)");
static SUB_GHIDRASQL_EXPORT: Subformat =
    Subformat::new("Ghidra SQLite export database (GhidraSQL)");
static SUB_GNOME_TRACKER_SPARQL: Subformat = Subformat::new("GNOME Tracker SPARQL store");
static SUB_GNOME_ZEITGEIST: Subformat = Subformat::new("GNOME Zeitgeist activity database");
static SUB_GNUPG_KEYBOX: Subformat = Subformat::new("GnuPG 2 Keybox keyring (pubring.db)");
static SUB_GNUPG_TOFU: Subformat = Subformat::new("GnuPG 2 TOFU policy database (tofu.db)");
static SUB_GOOGLE_ANALYTICS_V4: Subformat =
    Subformat::new("Google Analytics Android SDK v4 database");
static SUB_GOOGLE_DRIVE_SNAPSHOT: Subformat =
    Subformat::new("Google Drive Backup and Sync snapshot database");
static SUB_GOOGLE_HANGOUTS: Subformat =
    Subformat::new("Google Hangouts Android conversations database (babel.db)");
static SUB_GOOGLE_SAFE_BROWSING: Subformat = Subformat::new("Google Safe Browsing chunk database");
static SUB_GRAFANA_DB: Subformat = Subformat::new("Grafana database (grafana.db)");
static SUB_HEIMDAL_KRB5: Subformat = Subformat::new("Heimdal Kerberos 5 credential cache");
static SUB_HOME_ASSISTANT_DB: Subformat = Subformat::new("Home Assistant recorder database");
static SUB_IDARLING_DB: Subformat =
    Subformat::new("IDArling collaborative IDA server database (idarling.db)");
static SUB_IDASQL_EXPORT: Subformat = Subformat::new("IDA Pro SQLite export database (IDASQL)");
static SUB_INTERNET_ARCHIVE_META: Subformat =
    Subformat::new("Internet Archive S3 per-key metadata database (*_meta.sqlite)");
static SUB_IOS_IMO_HD: Subformat =
    Subformat::new("IMO HD Messenger iOS chat database (IMODb2.sqlite)");
static SUB_IOS_INSTAGRAM_DIRECT: Subformat =
    Subformat::new("Instagram Direct iOS threads database");
static SUB_IOS_KIK_MESSENGER: Subformat =
    Subformat::new("Kik Messenger iOS chat database (kik.sqlite)");
static SUB_IOS_MANIFEST: Subformat = Subformat::new("Apple iOS Backup Manifest (Manifest.db)");
static SUB_IOS_TWITTER: Subformat = Subformat::new("Twitter / X iOS database (twitter.db)");
static SUB_IPYTHON_HISTORY: Subformat =
    Subformat::new("IPython shell history database (history.sqlite)");
static SUB_JELLYFIN_LIBRARY: Subformat = Subformat::new("Jellyfin / Emby media library database");
static SUB_JUPYTER_NBSIG: Subformat = Subformat::new("Jupyter Notebook trust signatures database");
static SUB_K3S_KINE_DB: Subformat = Subformat::new("K3s / Kine Kubernetes etcd-shim database");
static SUB_KASPERSKY_KAV_REPORTS: Subformat =
    Subformat::new("Kaspersky Anti-Virus reports database");
static SUB_KASPERSKY_KVDB: Subformat =
    Subformat::new("Kaspersky key-value settings database (.kvdb)");
static SUB_KDE_KSTARS: Subformat = Subformat::new("KDE KStars catalog (.kscat)");
static SUB_KISMET_CAPTURE: Subformat = Subformat::new("Kismet wireless capture database (.kismet)");
static SUB_KODI_VIDEOS: Subformat = Subformat::new("Kodi video library database");
static SUB_KYUA_RESULTS: Subformat = Subformat::new("BSD Kyua test results database");
static SUB_LANGCHAIN_CACHE: Subformat = Subformat::new("LangChain LLM cache database");
static SUB_LIBCHEWING_PHRASE_DB: Subformat = Subformat::new("libchewing user phrase database");
static SUB_LINUX_PERF_DB: Subformat = Subformat::new("Linux perf trace export database");
static SUB_LOGITECH_GHUB: Subformat =
    Subformat::new("Logitech G HUB / Logi Options+ settings database (settings.db)");
static SUB_MACOS_APP_USAGE: Subformat =
    Subformat::new("Apple macOS Application Usage database (application_usage.sqlite)");
static SUB_MAKEMANDB: Subformat = Subformat::new("BSD makemandb manual page index");
static SUB_MANDOC_DB: Subformat = Subformat::new("BSD mandoc manual page index");
static SUB_MAPLE_WORKBOOK: Subformat = Subformat::new("Maple workbook (.maple)");
static SUB_MATRIX_RUST_SDK: Subformat = Subformat::new("Matrix Rust SDK crypto store");
static SUB_MBTILES_SCHEMA: Subformat = Subformat::new("MBTiles (schema)");
static SUB_MINECRAFT_COREPROTECT: Subformat = Subformat::new("Minecraft CoreProtect database");
static SUB_MINECRAFT_LWC: Subformat = Subformat::new("Minecraft LWC protection database");
static SUB_MINECRAFT_PLOTME: Subformat = Subformat::new("Minecraft PlotMe database");
static SUB_MLFLOW_DB: Subformat = Subformat::new("MLflow tracking database (mlflow.db)");
static SUB_MONEY_MANAGER_EX: Subformat = Subformat::new("Money Manager Ex database (.mmb)");
static SUB_MONOTONE_REPO: Subformat = Subformat::new("Monotone SCM repository (.mtn)");
static SUB_MOZILLA_NSS_CERT9: Subformat =
    Subformat::new("Mozilla NSS Public Certificate store (cert9.db)");
static SUB_MOZILLA_NSS_KEY4: Subformat = Subformat::new("Mozilla NSS Private Key store (key4.db)");
static SUB_NIX_BINARY_CACHE: Subformat = Subformat::new("Nix binary cache metadata database");
static SUB_NIX_FETCHER_CACHE: Subformat = Subformat::new("Nix fetcher cache");
static SUB_NIX_STORE: Subformat = Subformat::new("Nix / GNU Guix store database");
static SUB_OFFICE_C2R: Subformat = Subformat::new("Microsoft Office Click-to-Run database");
static SUB_ONE_C_EVENTLOG: Subformat = Subformat::new("1C:Enterprise 8 event log database");
static SUB_OPENCODE_SESSION: Subformat =
    Subformat::new("OpenCode AI coding agent session database (opencode.db)");
static SUB_OPERA_LOAD_STATS: Subformat = Subformat::new("Opera Load Statistics database");
static SUB_OPERA_SESSION: Subformat = Subformat::new("Opera Browser Session database");
static SUB_OPTUNA_DB: Subformat = Subformat::new("Optuna study database (optuna.db)");
static SUB_OSGEO_PROJ: Subformat = Subformat::new("OSGeo PROJ geodetic registry (proj.db)");
static SUB_PAESSLER_PRTG: Subformat = Subformat::new("Paessler PRTG Network Monitor database");
static SUB_PDBSQL_EXPORT: Subformat =
    Subformat::new("Microsoft PDB SQLite export database (PDBSQL)");
static SUB_PHOTOTHUMB_CACHE: Subformat =
    Subformat::new("Photo thumbnail cache database (photothumb.db)");
static SUB_PLASO_STORAGE: Subformat = Subformat::new("Plaso forensic event storage (.plaso)");
static SUB_PLASO_WINEVT_RC: Subformat =
    Subformat::new("Plaso Windows Event Log resources database (winevt-rc.db)");
static SUB_PLEX_LIBRARY: Subformat = Subformat::new("Plex Media Server library database");
static SUB_PYTHON_DBM_SQLITE3: Subformat = Subformat::new("Python dbm.sqlite3 database");
static SUB_PYTHON_LOCKFILE: Subformat = Subformat::new("Python lockfile.sqlitelockfile database");
static SUB_QT_COMPRESSED_HELP: Subformat =
    Subformat::new("Qt Compressed Help documentation database (.qch)");
static SUB_QT_HELP_COLLECTION: Subformat =
    Subformat::new("Qt Help Collection index database (.qhc)");
static SUB_QUICK_MACROS_2: Subformat =
    Subformat::new("Quick Macros 2 automation database (.qml / .qm2)");
static SUB_REFLET_DB: Subformat = Subformat::new("Reflet database (.reflet)");
static SUB_RESONIC_PLAYER: Subformat = Subformat::new("Resonic Player media database");
static SUB_RPM_REPO_METADATA: Subformat =
    Subformat::new("RPM/YUM/DNF repository metadata database");
static SUB_RTI_APEX_PROJECT: Subformat =
    Subformat::new("RTI Integration Designer APEX project database");
static SUB_SAFARI_HISTORY: Subformat = Subformat::new("Apple Safari History database (History.db)");
static SUB_SAFARI_STORAGE_TRACKER: Subformat =
    Subformat::new("Apple Safari LocalStorage tracker (StorageTracker.db)");
static SUB_SAFARI_WEBPAGE_ICONS: Subformat =
    Subformat::new("Apple Safari WebpageIcons database (WebpageIcons.db)");
static SUB_SDL_TRADOS: Subformat = Subformat::new("SDL Trados translation memory (.sdltm)");
static SUB_SIDECHANNELMARVELS_TRACER: Subformat =
    Subformat::new("SideChannelMarvels TracerGrind / TracerPIN trace database");
static SUB_SIGNAL_DESKTOP: Subformat = Subformat::new("Signal Desktop database");
static SUB_SKETCH_DOC: Subformat = Subformat::new("Sketch design document (.sketch)");
static SUB_SKYPE_CONFIG_DB: Subformat = Subformat::new("Microsoft Skype configuration database");
static SUB_SKYPE_MAIN_DB: Subformat = Subformat::new("Microsoft Skype main database (main.db)");
static SUB_SOURCETRAIL_DB: Subformat =
    Subformat::new("Sourcetrail source index database (.srctrldb)");
static SUB_SPATIALITE: Subformat = Subformat::new("SpatiaLite spatial database");
static SUB_SQLAR: Subformat = Subformat::new("SQLite Archive (.sqlar)");
static SUB_SUPERANTISPYWARE_DB: Subformat =
    Subformat::new("SUPERAntiSpyware configuration database (.db3)");
static SUB_SVN_REP_CACHE: Subformat =
    Subformat::new("Apache Subversion FSFS representation cache (rep-cache.db)");
static SUB_SVN_WC: Subformat = Subformat::new("Apache Subversion working copy database (wc.db)");
static SUB_TANGO_ANDROID: Subformat = Subformat::new("Tango Messenger Android database (tc.db)");
static SUB_TENSORBOARD_DB: Subformat = Subformat::new("TensorFlow / TensorBoard summary database");
static SUB_TEXNICARD_DB: Subformat = Subformat::new("TeXnicard card database");
static SUB_THUNDERBIRD_GLOBAL_MSGS: Subformat =
    Subformat::new("Mozilla Thunderbird Global Messages database");
static SUB_TOPCON_SURVEY_JOB: Subformat = Subformat::new("Topcon Survey Job database (.tsj)");
static SUB_TWINMOTION_ITEM: Subformat = Subformat::new("Twinmotion library item (.tmi)");
static SUB_TWINMOTION_PROJECT: Subformat = Subformat::new("Twinmotion project (.tm)");
static SUB_UNITY_ASSET_VERSIONING: Subformat =
    Subformat::new("Unity Editor Asset Versioning database");
static SUB_UNITY_SHADER_CACHE: Subformat =
    Subformat::new("Unity Editor Shader Cache (ShaderCache.db)");
static SUB_VISUAL_PARADIGM: Subformat = Subformat::new("Visual Paradigm UML project (.vpp)");
static SUB_VISUAL_STUDIO_STORAGE_IDE: Subformat =
    Subformat::new("Microsoft Visual Studio solution cache (storage.ide)");
static SUB_VIVALDI_CALENDAR: Subformat = Subformat::new("Vivaldi Browser Calendar database");
static SUB_VSCODE_STATE: Subformat = Subformat::new("VS Code workspace state (state.vscdb)");
static SUB_VS_CPP_BROWSE: Subformat =
    Subformat::new("Microsoft Visual Studio C++ browse database (.browse.VC.db)");
static SUB_VS_SLNX_INDEX: Subformat =
    Subformat::new("Microsoft Visual Studio workspace index (slnx.sqlite)");
static SUB_WAZUH_INVENTORY: Subformat = Subformat::new("Wazuh Syscollector inventory database");
static SUB_WHATSAPP_AXOLOTL: Subformat =
    Subformat::new("WhatsApp Signal-protocol key store (axolotl.db)");
static SUB_WHATSAPP_CONTACTS: Subformat = Subformat::new("WhatsApp Contacts database (wa.db)");
static SUB_WHATSAPP_MSGSTORE: Subformat =
    Subformat::new("WhatsApp Messages database (msgstore.db)");
static SUB_WINDOWS_ASIMOV: Subformat =
    Subformat::new("Microsoft Windows Asimov EventStore database");
static SUB_WINDOWS_EVENTTRANSCRIPT: Subformat =
    Subformat::new("Microsoft Windows EventTranscript database (EventTranscript.db)");
static SUB_WINDOWS_STATEREPOSITORY: Subformat =
    Subformat::new("Microsoft Windows AppX StateRepository database (.srd)");
static SUB_WINDOWS_TIMELINE: Subformat =
    Subformat::new("Microsoft Windows Timeline database (ActivitiesCache.db)");
static SUB_WINDOWS_UPDATE_STORE: Subformat =
    Subformat::new("Microsoft Windows Update Store database");
static SUB_WINDOWS_WPN: Subformat =
    Subformat::new("Microsoft Windows Push Notification database (wpndatabase.db)");
static SUB_WOWOOL_NLP_MODEL: Subformat =
    Subformat::new("Wowool NLP linguistic model database (.dom / .hmm)");
static SUB_XKLB_MEDIA: Subformat = Subformat::new("xklb media library database");
static SUB_YANDEX_AUTOFILL: Subformat =
    Subformat::new("Yandex Browser autofill database (Ya Autofill Data)");

// ============================================================================
// Tier 0: Header `application_id` (@68) and `user_version` (@60)
// ============================================================================

const MONOTONE_USER_VERSION: u32 = 0x5F4D_544E;

/// SQLite 3 subformat identified directly by its big-endian `application_id` at offset `@68`.
fn sqlite_app_subformat(app_id: u32) -> Option<&'static Subformat> {
    match app_id {
        0x4163726E => Some(&SUB_ACORN_IMAGE),
        0xD3D12CDB => Some(&SUB_AMD_DXCACHE),
        0x41554459 => Some(&SUB_AUDACITY_3),
        0x7ACCCCCE => Some(&SUB_BEEREF_SCENE),
        0x42654462 | 0x42654C6E | 0xBE7DEC1A => Some(&SUB_BENTLEY_BESQLITE),
        0x424E4442 | 0x56333500 => Some(&SUB_BINARY_NINJA),
        0x0B110907 | 0xF9BEB4D9 | 0xFABFB5DA => Some(&SUB_BITCOIN_WALLET),
        0x63616C69 => Some(&SUB_CALIBRE_LIBRARY),
        0x43455745 => Some(&SUB_CEWE_PHOTOBOOK),
        0x5E96730E => Some(&SUB_CORSAIR_ICUE),
        0x5A494458 => Some(&SUB_DFANALYZER_ZINDEX),
        0x45535249 | 0x45737269 => Some(&SUB_ESRI_ST_GEOMETRY),
        0x656C656D => Some(&SUB_FORENSICSTORE_ELEM),
        0x0F055112 => Some(&SUB_FOSSIL_CHECKOUT),
        0x0F055113 => Some(&SUB_FOSSIL_GLOBAL_CFG),
        0x0F055111 | 0x5F46534C => Some(&SUB_FOSSIL_REPO),
        0x5370716C => Some(&SUB_GNOME_TRACKER_SPARQL),
        0x004B4156 => Some(&SUB_KASPERSKY_KAV_REPORTS),
        0x4D515158 => Some(&SUB_KDE_KSTARS),
        0x43484557 => Some(&SUB_LIBCHEWING_PHRASE_DB),
        0x5CDE09EF => Some(&SUB_MAPLE_WORKBOOK),
        0x4D4D4558 => Some(&SUB_MONEY_MANAGER_EX),
        0x4D534442 | 0x70727467 => Some(&SUB_PAESSLER_PRTG),
        0x514D3200 => Some(&SUB_QUICK_MACROS_2),
        0x52464C54 => Some(&SUB_REFLET_DB),
        0x23502350 => Some(&SUB_RESONIC_PLAYER),
        0x000CA439 => Some(&SUB_RTI_APEX_PROJECT),
        0x53716C72 => Some(&SUB_SQLAR),
        0xFEEDABEE => Some(&SUB_TENSORBOARD_DB),
        0x6A035744 => Some(&SUB_TEXNICARD_DB),
        0x544D4C49 => Some(&SUB_TWINMOTION_ITEM),
        0x544D5052 => Some(&SUB_TWINMOTION_PROJECT),
        0x584B4C42 => Some(&SUB_XKLB_MEDIA),
        _ => None,
    }
}

// ============================================================================
// Tier 3: Disambiguated DDL & ordered column-tuple (`S4`) matching
// ============================================================================

const S1_SQLAR: u64 = fnv1a64(b"sqlar");
const S1_FILES: u64 = fnv1a64(b"files");
const S1_FILES_PROPERTIES: u64 = fnv1a64(b"files\nproperties");
const S1_CACHE: u64 = fnv1a64(b"cache");
const S1_MAIN: u64 = fnv1a64(b"main");
const S1_DICT: u64 = fnv1a64(b"dict");
const S1_ITEMTABLE: u64 = fnv1a64(b"itemtable");
const S1_DATATABLE: u64 = fnv1a64(b"datatable");
const S1_LOCKS: u64 = fnv1a64(b"locks");
const S1_ORIGINS: u64 = fnv1a64(b"origins");
const S1_TABLES_VERSION_TOBJDIMPTS: u64 = fnv1a64(b"tables_version\ntobjdimpts");
const S1_METADATA_PAYLOAD: u64 = fnv1a64(b"metadata\npayload");
const S1_THUMB: u64 = fnv1a64(b"thumb");
const S1_SETTINGS: u64 = fnv1a64(b"settings");
const S1_STORAGE: u64 = fnv1a64(b"storage");
const S1_MESSAGES: u64 = fnv1a64(b"messages");
const S1_SYNC_HISTORY: u64 = fnv1a64(b"sync_history");
const S1_APPLICATION_USAGE: u64 = fnv1a64(b"application_usage");
const S1_DATA_SNAPSHOTS: u64 = fnv1a64(b"data\nsnapshots");
const S1_ITEMS_TEXTS: u64 = fnv1a64(b"items\ntexts");
const S1_GLOBAL_CONFIG: u64 = fnv1a64(b"global_config");
const S1_CAT_CATALOGS: u64 = fnv1a64(b"cat\ncatalogs");
const S1_CONFIG_RESULTS_UNMATCHED: u64 = fnv1a64(b"config\nresults\nunmatched");

fn is_tier3_candidate(s1_hash: u64) -> bool {
    matches!(
        s1_hash,
        S1_SQLAR
            | S1_FILES
            | S1_FILES_PROPERTIES
            | S1_CACHE
            | S1_MAIN
            | S1_DICT
            | S1_ITEMTABLE
            | S1_DATATABLE
            | S1_LOCKS
            | S1_ORIGINS
            | S1_TABLES_VERSION_TOBJDIMPTS
            | S1_METADATA_PAYLOAD
            | S1_THUMB
            | S1_SETTINGS
            | S1_STORAGE
            | S1_MESSAGES
            | S1_SYNC_HISTORY
            | S1_APPLICATION_USAGE
            | S1_DATA_SNAPSHOTS
            | S1_ITEMS_TEXTS
            | S1_GLOBAL_CONFIG
            | S1_CAT_CATALOGS
            | S1_CONFIG_RESULTS_UNMATCHED
    )
}

/// Reads the `sql` column of a bookmarked `sqlite_schema` leaf cell, chasing up to
/// [`MAX_OVERFLOW_HOPS`] overflow pages when the statement spills out of the leaf page.
fn read_bookmark_sql(
    reader: &mut PageReader<'_, '_>, page_size: usize, usable_size: usize, bm: TableSqlBookmark,
    out: &mut [u8],
) -> Option<usize> {
    if bm.sql_len == 0 || bm.sql_len > out.len() {
        return None;
    }
    let avail_on_page = usable_size.saturating_sub(bm.cell_off.saturating_add(bm.rec_start));
    let local_rec_len = bm.local_payload.min(avail_on_page);
    let mut written = 0usize;

    if bm.sql_offset_in_rec < local_rec_len {
        let take = (local_rec_len - bm.sql_offset_in_rec).min(bm.sql_len);
        let file_off =
            bm.page_file_offset + (bm.cell_off + bm.rec_start + bm.sql_offset_in_rec) as u64;
        if !reader.read_exact(file_off, &mut out[..take]) {
            return None;
        }
        written += take;
    }

    if written < bm.sql_len && bm.payload_size > bm.local_payload {
        let ptr_off_in_page =
            bm.cell_off.saturating_add(bm.rec_start).saturating_add(bm.local_payload);
        if ptr_off_in_page + 4 > usable_size {
            return None;
        }
        let mut ovf_pg = reader.read_be32(bm.page_file_offset + ptr_off_in_page as u64)?;
        let mut skip = bm.sql_offset_in_rec.saturating_sub(local_rec_len);
        let chunk_cap = usable_size.saturating_sub(4);
        if chunk_cap == 0 {
            return None;
        }
        let mut seen = [0u32; MAX_OVERFLOW_HOPS];
        let mut hops = 0usize;
        while ovf_pg > 1 && written < bm.sql_len && hops < MAX_OVERFLOW_HOPS {
            if seen[..hops].contains(&ovf_pg) {
                return None;
            }
            seen[hops] = ovf_pg;
            hops += 1;

            let ovf_off = u64::from(ovf_pg - 1).checked_mul(page_size as u64)?;
            let next_pg = reader.read_be32(ovf_off)?;
            if skip >= chunk_cap {
                skip -= chunk_cap;
            } else {
                let take = (chunk_cap - skip).min(bm.sql_len - written);
                if !reader.read_exact(ovf_off + 4 + skip as u64, &mut out[written..written + take])
                {
                    return None;
                }
                written += take;
                skip = 0;
            }
            ovf_pg = next_pg;
        }
    }

    (written == bm.sql_len).then_some(written)
}

/// Strips SQL `--` line comments and `/* ... */` block comments outside quoted literals.
fn strip_sql_comments(src: &[u8], out: &mut [u8]) -> usize {
    let mut i = 0usize;
    let mut w = 0usize;
    let mut quote: u8 = 0;
    while i < src.len() && w < out.len() {
        let b = src[i];
        if quote != 0 {
            out[w] = b;
            w += 1;
            i += 1;
            if quote == b']' {
                if b == b']' {
                    quote = 0;
                }
            } else if b == quote {
                if i < src.len() && src[i] == quote && w < out.len() {
                    out[w] = src[i];
                    w += 1;
                    i += 1;
                } else {
                    quote = 0;
                }
            }
            continue;
        }
        if b == b'-' && i + 1 < src.len() && src[i + 1] == b'-' {
            i += 2;
            while i < src.len() && src[i] != b'\n' {
                i += 1;
            }
            if w < out.len() {
                out[w] = b' ';
                w += 1;
            }
            continue;
        }
        if b == b'/' && i + 1 < src.len() && src[i + 1] == b'*' {
            i += 2;
            while i + 1 < src.len() && !(src[i] == b'*' && src[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(src.len());
            if w < out.len() {
                out[w] = b' ';
                w += 1;
            }
            continue;
        }
        match b {
            b'\'' | b'"' | b'`' => quote = b,
            b'[' => quote = b']',
            _ => {}
        }
        out[w] = b;
        w += 1;
        i += 1;
    }
    w
}

/// Extracts the leading SQL identifier from a single column-definition item, lowercased into `out`.
fn extract_leading_column_ident(item: &[u8], out: &mut [u8]) -> usize {
    let mut i = 0usize;
    while i < item.len() && item[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= item.len() {
        return 0;
    }
    let mut w = 0usize;
    match item[i] {
        q @ (b'"' | b'`' | b'\'') => {
            i += 1;
            while i < item.len() && w < out.len() {
                if item[i] == q {
                    if i + 1 < item.len() && item[i + 1] == q {
                        out[w] = q;
                        w += 1;
                        i += 2;
                        continue;
                    }
                    break;
                }
                out[w] = item[i].to_ascii_lowercase();
                w += 1;
                i += 1;
            }
        }
        b'[' => {
            i += 1;
            while i < item.len() && item[i] != b']' && w < out.len() {
                out[w] = item[i].to_ascii_lowercase();
                w += 1;
                i += 1;
            }
        }
        _ => {
            while i < item.len() && w < out.len() {
                let b = item[i];
                if b.is_ascii_alphanumeric() || b == b'_' || b == b'$' || b >= 0x80 {
                    out[w] = b.to_ascii_lowercase();
                    w += 1;
                    i += 1;
                } else {
                    break;
                }
            }
        }
    }
    w
}

/// Extracts ordered lowercase column names (`"col1|col2|..."`) from a `CREATE TABLE` DDL statement
/// into `col_arena`, skipping table-level SQL constraints.
fn extract_ordered_columns(
    sql: &[u8], col_arena: &mut [u8], pos: &mut usize,
) -> Option<(u16, u16)> {
    let mut clean = [0u8; MAX_SQL_LEN];
    let clen = strip_sql_comments(sql, &mut clean);
    let clean = &clean[..clen];

    // Locate the first top-level `(` opening the column list.
    let mut open_idx = None;
    let mut quote: u8 = 0;
    for (idx, &b) in clean.iter().enumerate() {
        if quote != 0 {
            if (quote == b']' && b == b']') || (quote != b']' && b == quote) {
                quote = 0;
            }
            continue;
        }
        match b {
            b'\'' | b'"' | b'`' => quote = b,
            b'[' => quote = b']',
            b'(' => {
                open_idx = Some(idx);
                break;
            }
            _ => {}
        }
    }
    let body_start = open_idx? + 1;

    // Locate the matching top-level `)`.
    let mut depth = 1usize;
    let mut body_end = None;
    quote = 0;
    let mut i = body_start;
    while i < clean.len() {
        let b = clean[i];
        if quote != 0 {
            if quote == b']' {
                if b == b']' {
                    quote = 0;
                }
            } else if b == quote {
                if i + 1 < clean.len() && clean[i + 1] == quote {
                    i += 2;
                    continue;
                }
                quote = 0;
            }
            i += 1;
            continue;
        }
        match b {
            b'\'' | b'"' | b'`' => quote = b,
            b'[' => quote = b']',
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    body_end = Some(i);
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    let body = &clean[body_start..body_end?];

    let tuple_start = *pos;
    let mut col_count = 0usize;
    let mut ident_buf = [0u8; 128];
    let mut item_start = 0usize;
    depth = 0;
    quote = 0;
    let mut j = 0usize;
    while j <= body.len() {
        let at_split = if j == body.len() {
            true
        } else {
            let b = body[j];
            if quote != 0 {
                if quote == b']' {
                    if b == b']' {
                        quote = 0;
                    }
                } else if b == quote {
                    if j + 1 < body.len() && body[j + 1] == quote {
                        j += 2;
                        continue;
                    }
                    quote = 0;
                }
                false
            } else {
                match b {
                    b'\'' | b'"' | b'`' => {
                        quote = b;
                        false
                    }
                    b'[' => {
                        quote = b']';
                        false
                    }
                    b'(' => {
                        depth += 1;
                        false
                    }
                    b')' => {
                        depth = depth.saturating_sub(1);
                        false
                    }
                    b',' if depth == 0 => true,
                    _ => false,
                }
            }
        };

        if at_split {
            let item = &body[item_start..j];
            let ilen = extract_leading_column_ident(item, &mut ident_buf);
            let ident = &ident_buf[..ilen];
            let is_constraint =
                matches!(ident, b"constraint" | b"primary" | b"unique" | b"check" | b"foreign");
            if ilen > 0 && !is_constraint {
                if col_count > 0 {
                    if *pos >= col_arena.len() {
                        *pos = tuple_start;
                        return None;
                    }
                    col_arena[*pos] = b'|';
                    *pos += 1;
                }
                let Some((cs, ce)) = push_normalized_ident(ident, col_arena, pos) else {
                    *pos = tuple_start;
                    return None;
                };
                if cs == ce {
                    if col_count > 0 {
                        *pos -= 1;
                    }
                } else {
                    col_count += 1;
                }
            }
            item_start = j + 1;
        }
        j += 1;
    }

    (col_count > 0).then_some((tuple_start as u16, *pos as u16))
}

/// Disambiguates single- and two-table SQLite schemas using exact `CREATE TABLE` DDL hashes and
/// ordered column-tuple (`S4`) hashes. Kept out-of-line so its stack buffers stay off the Tier-2
/// hot path.
#[inline(never)]
fn resolve_tier3(
    reader: &mut PageReader<'_, '_>, page_size: usize, usable_size: usize, enc: TextEncoding,
    s1_hash: u64, bookmarks: &[TableSqlBookmark],
) -> Option<&'static Subformat> {
    if bookmarks.is_empty() {
        return None;
    }

    let mut raw_sql = [0u8; MAX_SQL_LEN];
    let mut decoded_sql = [0u8; MAX_SQL_LEN];
    let mut col_arena = [0u8; 2048];
    let mut col_pos = 0usize;
    let mut c_spans = [(0u16, 0u16); MAX_BOOKMARKS];
    let mut c_count = 0usize;
    let mut single_sql_hash = 0u64;

    for &bm in bookmarks {
        let rlen = read_bookmark_sql(reader, page_size, usable_size, bm, &mut raw_sql)?;
        let sql_bytes: &[u8] = match enc {
            TextEncoding::Utf8 => {
                let mut end = rlen;
                while end > 0 && raw_sql[end - 1] == 0 {
                    end -= 1;
                }
                &raw_sql[..end]
            }
            TextEncoding::Utf16Le | TextEncoding::Utf16Be => {
                let dlen = decode_text(&raw_sql[..rlen], enc, &mut decoded_sql)?;
                &decoded_sql[..dlen]
            }
        };
        if bookmarks.len() == 1 {
            single_sql_hash = fnv1a64(sql_bytes);
        }
        if let Some(span) = extract_ordered_columns(sql_bytes, &mut col_arena, &mut col_pos)
            && c_count < MAX_BOOKMARKS
        {
            c_spans[c_count] = span;
            c_count += 1;
        }
    }

    let c_slice = &mut c_spans[..c_count];
    c_slice.sort_unstable_by(|&(as_, ae), &(bs, be)| {
        col_arena[as_ as usize..ae as usize].cmp(&col_arena[bs as usize..be as usize])
    });

    let mut s4_hash = 0u64;
    if c_count > 0 {
        let mut h = FNV1A64_OFFSET_BASIS;
        let mut prev: Option<&[u8]> = None;
        let mut first = true;
        for &(s, e) in c_slice.iter() {
            let slice = &col_arena[s as usize..e as usize];
            if prev == Some(slice) {
                continue;
            }
            prev = Some(slice);
            if !first {
                h = fnv1a64_step(h, b"\n");
            }
            h = fnv1a64_step(h, slice);
            first = false;
        }
        s4_hash = h;
    }

    match s1_hash {
        S1_SQLAR => {
            if matches!(s4_hash, 0xDDDE8A609AFC204C) {
                return Some(&SUB_SQLAR);
            }
            None
        }
        S1_FILES => {
            if matches!(s4_hash, 0xBDF55702EE34BDE0) {
                return Some(&SUB_IOS_MANIFEST);
            }
            if matches!(single_sql_hash, 0x869A75BCD4241A8C)
                || matches!(s4_hash, 0xD25449ADD81A89B5)
            {
                return Some(&SUB_CEWE_PHOTOBOOK);
            }
            if matches!(s4_hash, 0x12A652406AD42FE6) {
                return Some(&SUB_WOWOOL_NLP_MODEL);
            }
            None
        }
        S1_FILES_PROPERTIES => {
            if matches!(s4_hash, 0x8634A2786ABF44B8) {
                return Some(&SUB_IOS_MANIFEST);
            }
            None
        }
        S1_CACHE => {
            if matches!(s4_hash, 0xEEC3A6306C02C86C | 0x494469C91DDE8C24) {
                return Some(&SUB_ANDROID_WEBVIEW_CACHE);
            }
            if matches!(single_sql_hash, 0x3EBB950732867F9D) {
                return Some(&SUB_AMD_DXCACHE);
            }
            if matches!(single_sql_hash, 0xA38BCCEC92AD6E1E) {
                return Some(&SUB_NIX_FETCHER_CACHE);
            }
            None
        }
        S1_MAIN => {
            if matches!(single_sql_hash, 0x51E0AA2AE7035674) {
                return Some(&SUB_BITCOIN_WALLET);
            }
            None
        }
        S1_DICT => {
            if matches!(single_sql_hash, 0x4F8A5141A5D184B3 | 0x2876DB76EC2DFCCB) {
                return Some(&SUB_PYTHON_DBM_SQLITE3);
            }
            None
        }
        S1_ITEMTABLE => {
            if matches!(single_sql_hash, 0x7D3CC36CA4AEC9D7) {
                return Some(&SUB_VSCODE_STATE);
            }
            if matches!(single_sql_hash, 0x9C674A793C621464)
                || (matches!(enc, TextEncoding::Utf16Le) && matches!(s4_hash, 0xAF491765864F27C1))
            {
                return Some(&SUB_CHROMIUM_LOCALSTORAGE);
            }
            None
        }
        S1_DATATABLE => {
            if matches!(s4_hash, 0xD869E351293B3CCD) {
                return Some(&SUB_ADOBE_CC_DATATABLE);
            }
            if matches!(s4_hash, 0xCC32B7894B55E504 | 0xD5C9DA0085E8AA29) {
                return Some(&SUB_ADOBE_CRCOMMON);
            }
            None
        }
        S1_LOCKS => {
            if matches!(single_sql_hash, 0xA0FBE6EC8C48393D) {
                return Some(&SUB_PYTHON_LOCKFILE);
            }
            None
        }
        S1_ORIGINS => {
            if (matches!(enc, TextEncoding::Utf16Le | TextEncoding::Utf8)
                && matches!(s4_hash, 0x3327F11462DDF072))
            {
                return Some(&SUB_SAFARI_STORAGE_TRACKER);
            }
            None
        }
        S1_TABLES_VERSION_TOBJDIMPTS => {
            if matches!(s4_hash, 0xD95D4273E9C19730 | 0x864E4F984E7033E6 | 0xF0616EA420C485AC) {
                return Some(&SUB_AUTOCAD_DIMPTS);
            }
            None
        }
        S1_METADATA_PAYLOAD => {
            if matches!(s4_hash, 0x6834462040BD0A8F) {
                return Some(&SUB_SKETCH_DOC);
            }
            None
        }
        S1_THUMB => {
            if matches!(s4_hash, 0xD0358A580F58A5AB) {
                return Some(&SUB_PHOTOTHUMB_CACHE);
            }
            None
        }
        S1_SETTINGS => {
            if matches!(s4_hash, 0x00DDEE0EF78E672F) {
                return Some(&SUB_SUPERANTISPYWARE_DB);
            }
            None
        }
        S1_STORAGE => {
            if matches!(s4_hash, 0x9BB92FFF285F19D1) {
                return Some(&SUB_KASPERSKY_KVDB);
            }
            None
        }
        S1_MESSAGES => {
            if matches!(s4_hash, 0x153092B036325D5D) {
                return Some(&SUB_FIREBASE_APP_MEASUREMENT);
            }
            None
        }
        S1_SYNC_HISTORY => {
            if matches!(s4_hash, 0x91E616D5CD0E5D35 | 0x8F58741CDA13D2A8 | 0x6E369E13283A7808) {
                return Some(&SUB_DROPBOX_SYNC_HISTORY);
            }
            None
        }
        S1_APPLICATION_USAGE => {
            if matches!(s4_hash, 0x75BEC6BC9E986D4A) {
                return Some(&SUB_MACOS_APP_USAGE);
            }
            None
        }
        S1_DATA_SNAPSHOTS => {
            if matches!(s4_hash, 0x2A25612CE850B20C) {
                return Some(&SUB_LOGITECH_GHUB);
            }
            None
        }
        S1_ITEMS_TEXTS => {
            if matches!(s4_hash, 0x1A8AB686B74B00DE) {
                return Some(&SUB_QUICK_MACROS_2);
            }
            None
        }
        S1_GLOBAL_CONFIG => {
            if matches!(s4_hash, 0x6834462040BD0A8F) {
                return Some(&SUB_FOSSIL_GLOBAL_CFG);
            }
            None
        }
        S1_CAT_CATALOGS => {
            if matches!(s4_hash, 0xBDCB06F068450A32 | 0xBDB36E6D143200E2) {
                return Some(&SUB_KDE_KSTARS);
            }
            None
        }
        S1_CONFIG_RESULTS_UNMATCHED => {
            if matches!(s4_hash, 0x651AE3C21F150272) {
                return Some(&SUB_DIAPHORA_DB);
            }
            None
        }
        _ => None,
    }
}

// ============================================================================
// Tier 2A: Exact normalized table-set (`S1`) FNV-1a64 matching
// ============================================================================

fn match_exact_s1_hash(s1_hash: u64, enc: TextEncoding) -> Option<&'static Subformat> {
    match s1_hash {
        0xA52210B92CA60067 => Some(&SUB_CHROMIUM_COOKIES),
        0x654118A25DE22EE6 => Some(&SUB_CHROMIUM_LOGIN_DATA),
        0x84B08C281B20ABBE => Some(&SUB_CHROMIUM_WEB_DATA),
        0x4C52C0F3FB60DB07 => Some(&SUB_CHROMIUM_TOP_SITES),
        0xC8A44084EA76078E => Some(&SUB_CHROMIUM_TOP_SITES),
        0x7CB59862A26FD325 => Some(&SUB_CHROMIUM_QUOTA_MANAGER),
        0x3415C7CF3C2CFD35 => Some(&SUB_CHROMIUM_APP_CACHE),
        0xA805876848ED0869 => Some(&SUB_CHROMIUM_UKM),
        0xFC964EBF8A1C8F17 => Some(&SUB_CHROMIUM_ORIGIN_CERTS),
        0x70786A81662CBC86 => Some(&SUB_CHROMIUM_HISTORY_INDEX),
        0x315FA3697B2A6203 => Some(&SUB_YANDEX_AUTOFILL),
        0xDFAC2890AB9F02F6 => Some(&SUB_GOOGLE_SAFE_BROWSING),
        0x2C78E7894A9E611D => Some(&SUB_MOZILLA_NSS_KEY4),
        0xF7BC6DB2468DD90A => Some(&SUB_MOZILLA_NSS_KEY4),
        0x096D265CFF297F27 => Some(&SUB_MOZILLA_NSS_CERT9),
        0xD27050E4A099D38A => Some(&SUB_MOZILLA_NSS_CERT9),
        0x06966A9C783EBC06 => Some(&SUB_FIREFOX_STORAGE),
        0x454BBCCF8210E1AD => Some(&SUB_FIREFOX_COOKIES),
        0x7A3774EA9701EF28 => Some(&SUB_FIREFOX_FORM_HISTORY),
        0xF97D89082A0D7D8F => Some(&SUB_FIREFOX_FORM_HISTORY),
        0x2722E189760EF119 => Some(&SUB_FIREFOX_PERMISSIONS),
        0x1B46F9099FF16605 => Some(&SUB_FIREFOX_PERMISSIONS),
        0xFD945D07B0AF2EEC => Some(&SUB_FIREFOX_PERMISSIONS),
        0x51415FF224DA94C0 => Some(&SUB_FIREFOX_WEBAPPSSTORE),
        0x4E666BC51E193872 => Some(&SUB_FIREFOX_WEBAPPSSTORE),
        0x2475491B54A84824 => Some(&SUB_FIREFOX_CONTENT_PREFS),
        0x4938187B1093CB2C => Some(&SUB_FIREFOX_REMOTE_SETTINGS),
        0x7FF1D794447EAC6C => Some(&SUB_FIREFOX_OFFLINE_CACHE),
        0x9D4247AD51F9B4B1 => Some(&SUB_FIREFOX_URL_CLASSIFIER),
        0x7C93D94277997606 => Some(&SUB_OPERA_SESSION),
        0x3E89EC82BF2B7B27 => Some(&SUB_VIVALDI_CALENDAR),
        0x1866414160732BC5 => Some(&SUB_ACORN_IMAGE),
        0xE60A8222DB95A03A => Some(&SUB_AUDACITY_3),
        0xCAC40C9F16BF9F10 => Some(&SUB_AUDACITY_3),
        0xE31003883F914C8E => Some(&SUB_BEEREF_SCENE),
        0x6E38509392B758A0 => Some(&SUB_FOSSIL_BUNDLE),
        0x955AC41C0E74D8D2 => Some(&SUB_TEXNICARD_DB),
        0x27DCC0EA793AD0EC => Some(&SUB_ADOBE_MUSE),
        0x7CFA92C235A9E0B9 => Some(&SUB_BALSAMIQ_BMPR),
        0xCA4D0746D3F20FFD => Some(&SUB_BALSAMIQ_BMPR),
        0x0197F240F59181C5 => Some(&SUB_ADOBE_SHARED_DATA_EVENTS),
        0xBF4D9BFE774BFEFD => Some(&SUB_ADOBE_PCD_CACHE),
        0x8E76D70A33F58794 => Some(&SUB_ADOBE_OPM_DB),
        0x62C16B277F25CEDA => Some(&SUB_ANKI_2_DECK),
        0xA9C535A52F0F3237 => Some(&SUB_CHERRYTREE_NOTE),
        0x4FB14A343A813493 => Some(&SUB_UNITY_ASSET_VERSIONING),
        0xDA33D3E9DF07AD47 => Some(&SUB_UNITY_SHADER_CACHE),
        0x3B05E3163085E11D => Some(&SUB_AMD_DXCACHE),
        0x533D5568338E97D6 => Some(&SUB_AMD_DXCACHE),
        0xED05C0B3CBE4C925 => Some(&SUB_VISUAL_STUDIO_STORAGE_IDE),
        0x5CEB3D5767499687 => Some(&SUB_VISUAL_STUDIO_STORAGE_IDE),
        0xD553EE3807E09851 => Some(&SUB_VISUAL_STUDIO_STORAGE_IDE),
        0x21779140A31E076F => Some(&SUB_RPM_REPO_METADATA),
        0x3DCD660373D3DEAF => Some(&SUB_RPM_REPO_METADATA),
        0xE7AD28ABE0FFABB4 => Some(&SUB_APPLE_APS_DB),
        0x2D1E8112FE386967 => Some(&SUB_APPLE_IPHOTO_ROOTPIXELS),
        0x79D31756DBFDA495 => Some(&SUB_APPLE_ITUNES_EXTRAS),
        0xAD7E3D19EBFD1E4D => Some(&SUB_SUPERANTISPYWARE_DB),
        0x33535D898D15D7CA => Some(&SUB_GOOGLE_ANALYTICS_V4),
        0x91C8796D96B1649E => Some(&SUB_QT_HELP_COLLECTION),
        0xE4B552318C16AF6C => Some(&SUB_TANGO_ANDROID),
        0x02DF5FCCC948C77C => Some(&SUB_IDARLING_DB),
        0xDBD3DFB3F9C1B58E => Some(&SUB_BINDIFF_WORKSPACE),
        0xA4EC888AE5601A47 => Some(&SUB_PAESSLER_PRTG),
        0xD81254624B772135 => Some(&SUB_XKLB_MEDIA),
        0x9BB26546F74CB2C4 => Some(&SUB_DRAKON_DIAGRAM),
        0x091972D829151A46 => Some(&SUB_APSIM_DB),
        0xEA2A07B00BC8260A => Some(&SUB_ACTIVEPRESENTER),
        0x93527EEEF7E9028B if matches!(enc, TextEncoding::Utf16Be) => {
            Some(&SUB_EMBEDDED_RECENT_PLAY)
        }
        0x442E5A60D871B3CB => Some(&SUB_MBTILES_SCHEMA),
        0xC6690A6617F03A8C => Some(&SUB_MBTILES_SCHEMA),
        0x76C7FB40D68CF75B => Some(&SUB_MBTILES_SCHEMA),
        0xFCCCD8B4EF619CB9 => Some(&SUB_FAR_MANAGER_CONFIG),
        0xB35CA94AE216055C => Some(&SUB_GAMEANALYTICS_CACHE),
        0xCF52ABFE92AD4559 => Some(&SUB_SKYPE_CONFIG_DB),
        0xE1DDC1172D034FC1 => Some(&SUB_SKYPE_CONFIG_DB),
        0xF9247C61411E4F33 => Some(&SUB_MINECRAFT_PLOTME),
        0x9DE7370B59DCCCD9 => Some(&SUB_MINECRAFT_LWC),
        _ => None,
    }
}

// ============================================================================
// Tier 2B: Sorted-span anchor-table matching
// ============================================================================

fn match_anchor_tables(
    arena: &[u8], spans: &[(u16, u16)], enc: TextEncoding,
) -> Option<&'static Subformat> {
    let has = |needle: &[u8]| -> bool {
        spans.binary_search_by(|&(s, e)| arena[s as usize..e as usize].cmp(needle)).is_ok()
    };

    if has(b"gpkg_contents") && (has(b"gpkg_spatial_ref_sys") || has(b"gpkg_geometry_columns")) {
        return Some(&SUB_GEOPACKAGE_SCHEMA);
    }
    if has(b"geometry_columns") && has(b"spatial_ref_sys") {
        return Some(&SUB_SPATIALITE);
    }
    if has(b"geodetic_crs") && has(b"projected_crs") && has(b"prime_meridian") {
        return Some(&SUB_OSGEO_PROJ);
    }
    if has(b"gdb_columnregistry") && has(b"gdb_itemrelationships") {
        return Some(&SUB_ESRI_ST_GEOMETRY);
    }
    if has(b"be_local") && has(b"be_prop") {
        return Some(&SUB_BENTLEY_BESQLITE);
    }
    if has(b"fdo_master") && has(b"schema") {
        return Some(&SUB_AUTOCAD_FDO);
    }
    if has(b"tblantennamodels") && has(b"tblcodedefs") {
        return Some(&SUB_TOPCON_SURVEY_JOB);
    }
    if has(b"cameras") && has(b"keypoints") && has(b"two_view_geometries") {
        return Some(&SUB_COLMAP_DB);
    }
    if has(b"objecttodatadependencies") && has(b"projectinfo") {
        return Some(&SUB_TWINMOTION_PROJECT);
    }
    if has(b"objecttodatadependencies") && has(b"libraryiteminfo") {
        return Some(&SUB_TWINMOTION_ITEM);
    }
    if has(b"moz_places") || (has(b"moz_bookmarks") && has(b"moz_historyvisits")) {
        return Some(&SUB_FIREFOX_PLACES);
    }
    if has(b"moz_cookies") {
        return Some(&SUB_FIREFOX_COOKIES);
    }
    if has(b"moz_formhistory") {
        return Some(&SUB_FIREFOX_FORM_HISTORY);
    }
    if has(b"moz_perms") || has(b"moz_hosts") {
        return Some(&SUB_FIREFOX_PERMISSIONS);
    }
    if has(b"moz_icons") && has(b"moz_pages_w_icons") {
        return Some(&SUB_FIREFOX_FAVICONS);
    }
    if has(b"moz_logins") && has(b"moz_disabledhosts") {
        return Some(&SUB_FIREFOX_SIGNONS);
    }
    if has(b"moz_downloads") {
        return Some(&SUB_FIREFOX_DOWNLOADS);
    }
    if has(b"addon") && has(b"addon_locale") && has(b"locale") && has(b"targetapplication") {
        return Some(&SUB_FIREFOX_EXTENSIONS);
    }
    if has(b"addon") && has(b"developer") && has(b"screenshot") {
        return Some(&SUB_FIREFOX_ADDONS);
    }
    if has(b"daily_counters") && has(b"daily_discrete_numeric") {
        return Some(&SUB_FIREFOX_HEALTH_REPORT);
    }
    if has(b"object_data") && has(b"object_store") {
        return Some(&SUB_FIREFOX_INDEXEDDB);
    }
    if has(b"nssprivate") {
        return Some(&SUB_MOZILLA_NSS_KEY4);
    }
    if has(b"nsspublic") {
        return Some(&SUB_MOZILLA_NSS_CERT9);
    }
    if has(b"attributedefinitions") && has(b"conversationstext") {
        return Some(&SUB_THUNDERBIRD_GLOBAL_MSGS);
    }
    if has(b"urls")
        && has(b"visits")
        && (has(b"keyword_search_terms") || has(b"downloads") || has(b"segments"))
    {
        return Some(&SUB_CHROMIUM_HISTORY);
    }
    if has(b"logins")
        && has(b"meta")
        && (has(b"ie7_logins") || has(b"stats") || has(b"insecure_credentials"))
    {
        return Some(&SUB_CHROMIUM_LOGIN_DATA);
    }
    if has(b"autofill") && has(b"meta") {
        return Some(&SUB_CHROMIUM_WEB_DATA);
    }
    if has(b"cookies") && has(b"meta") {
        return Some(&SUB_CHROMIUM_COOKIES);
    }
    if has(b"favicons") && has(b"icon_mapping") && (has(b"favicon_bitmaps") || has(b"meta")) {
        return Some(&SUB_CHROMIUM_FAVICONS);
    }
    if has(b"thumbnails") && has(b"meta") {
        return Some(&SUB_CHROMIUM_TOP_SITES);
    }
    if has(b"evictioninfotable") && has(b"origininfotable") {
        return Some(&SUB_CHROMIUM_QUOTA_MANAGER);
    }
    if has(b"omni_box_shortcuts") {
        return Some(&SUB_CHROMIUM_SHORTCUTS);
    }
    if has(b"network_action_predictor") {
        return Some(&SUB_CHROMIUM_NET_PREDICTOR);
    }
    if has(b"nel_policies") {
        return Some(&SUB_CHROMIUM_NEL);
    }
    if has(b"trust_tokens_issuer_config") {
        return Some(&SUB_CHROMIUM_TRUST_TOKENS);
    }
    if has(b"origin_bound_certs") {
        return Some(&SUB_CHROMIUM_ORIGIN_CERTS);
    }
    if has(b"bounces") && has(b"meta") {
        return Some(&SUB_CHROMIUM_DIPS);
    }
    if has(b"browser_context_sets_version") && has(b"browser_context_sites_to_clear") {
        return Some(&SUB_CHROMIUM_FPS);
    }
    if has(b"sources") && has(b"rate_limits") && (has(b"reports") || has(b"dedup_keys")) {
        return Some(&SUB_CHROMIUM_ATTRIBUTION);
    }
    if has(b"interest_groups") && has(b"join_history") && has(b"bid_history") {
        return Some(&SUB_CHROMIUM_FLEDGE);
    }
    if has(b"values_mapping") && has(b"per_origin_mapping") && has(b"budget_mapping") {
        return Some(&SUB_CHROMIUM_SHARED_STORAGE);
    }
    if has(b"browsing_topics_api_usages") {
        return Some(&SUB_CHROMIUM_TOPICS);
    }
    if has(b"passages") && has(b"embeddings") && has(b"meta") {
        return Some(&SUB_CHROMIUM_HISTORY_EMBEDDINGS);
    }
    if has(b"saves") && has(b"blobs") && has(b"sync_metadata") {
        return Some(&SUB_CHROMIUM_POWER_BOOKMARKS);
    }
    if has(b"indexed_db_metadata") && has(b"object_stores") && has(b"overflow_blob_chunks") {
        return Some(&SUB_CHROMIUM_INDEXEDDB);
    }
    if has(b"eq_classes") && has(b"eq_class_members") {
        return Some(&SUB_CHROMIUM_AFFILIATION);
    }
    if has(b"activitylog_compressed") && has(b"string_ids") && has(b"url_ids") {
        return Some(&SUB_CHROMIUM_EXT_ACTIVITY);
    }
    if has(b"load_statistics") && has(b"redirect_statistics") {
        return Some(&SUB_OPERA_LOAD_STATS);
    }
    if has(b"history_items")
        && has(b"history_visits")
        && (has(b"history_tombstones") || has(b"history_client_versions"))
    {
        return Some(&SUB_SAFARI_HISTORY);
    }
    if has(b"icondatabaseinfo") && has(b"pageurl") {
        return Some(&SUB_SAFARI_WEBPAGE_ICONS);
    }
    if has(b"__webkitdatabaseinfotable__") {
        return Some(&SUB_APPLE_WEBKIT_DB);
    }
    if (has(b"chat_message_join") && has(b"message_attachment_join"))
        || (has(b"msg_group") && has(b"msg_pieces"))
        || (has(b"chat")
            && has(b"chat_handle_join")
            && has(b"chat_message_join")
            && has(b"handle")
            && has(b"message"))
    {
        return Some(&SUB_APPLE_IMESSAGE);
    }
    if has(b"abperson") && has(b"abmultivalue") {
        return Some(&SUB_APPLE_ADDRESSBOOK);
    }
    if has(b"calendaritem") && has(b"calendar") && has(b"location") {
        return Some(&SUB_APPLE_CALENDAR);
    }
    if has(b"addresses")
        && has(b"attachments")
        && has(b"mailboxes")
        && (has(b"duplicates_unread_count") || has(b"ews_folders"))
    {
        return Some(&SUB_APPLE_MAIL_ENVELOPE);
    }
    if has(b"cfurl_cache_response") && has(b"cfurl_cache_blob_data") {
        return Some(&SUB_APPLE_CFURLCACHE);
    }
    if has(b"access") && has(b"access_overrides") && (has(b"active_policy") || has(b"policies")) {
        return Some(&SUB_APPLE_TCC);
    }
    if has(b"lsquarantineevent") {
        return Some(&SUB_APPLE_QUARANTINE);
    }
    if (has(b"dbinfo") && has(b"delivered") && has(b"displayed") && has(b"snoozed"))
        || ((has(b"app_info") || has(b"app"))
            && (has(b"notifications")
                || has(b"presented_alerts")
                || has(b"delivered")
                || has(b"displayed"))
            && has(b"record"))
    {
        return Some(&SUB_APPLE_NOTIF_CENTER);
    }
    if has(b"files") && has(b"generations") && has(b"storage") {
        return Some(&SUB_APPLE_DOC_REVISIONS);
    }
    if (has(b"quantity_samples")
        && has(b"workouts")
        && (has(b"activity_caches") || has(b"key_value_secure") || has(b"clinical_accounts")))
        || (has(b"workout_sessions") && has(b"source_devices") && has(b"logical_sources"))
        || (has(b"alarm_events") && has(b"datatype_source_order") && has(b"cloud_sync_stores"))
    {
        return Some(&SUB_APPLE_HEALTHKIT);
    }
    if spans.iter().any(|&(s, e)| {
        let t = &arena[s as usize..e as usize];
        t.starts_with(b"placcountingoperator_")
            || t.starts_with(b"plbatteryagent_")
            || t.starts_with(b"plprocessmonitoragent_")
            || t.starts_with(b"plapplicationagent_")
    }) {
        return Some(&SUB_APPLE_IOS_POWERLOG);
    }
    if (has(b"array_virtualreader") && has(b"ridlist_virtualreader"))
        || has(b"rkalbumchange")
        || has(b"rkdetectedface")
    {
        return Some(&SUB_APPLE_IPHOTO_LIB);
    }
    if has(b"deletedfacepreview") && has(b"facepreview") {
        return Some(&SUB_APPLE_PHOTOS_PREVIEW);
    }
    if has(b"zcallrecord") && has(b"zcalldbproperties") {
        return Some(&SUB_APPLE_CALL_HISTORY);
    }
    if has(b"zwaaxolotlidentity") && has(b"zwaaxolotlsession") {
        return Some(&SUB_WHATSAPP_AXOLOTL);
    }
    if has(b"zkikmessage") && has(b"zkikchat") && has(b"zkikuser") {
        return Some(&SUB_IOS_KIK_MESSENGER);
    }
    if has(b"zimochatmsg") && has(b"zimocontact") {
        return Some(&SUB_IOS_IMO_HD);
    }
    if has(b"zicnotedata") && has(b"ziccloudsyncingobject") {
        return Some(&SUB_APPLE_NOTES_MODERN);
    }
    if has(b"znotebody") && (has(b"zofflineaction") || (has(b"znote") && has(b"zfolder"))) {
        return Some(&SUB_APPLE_NOTES_LEGACY);
    }
    if (has(b"zcontextualkeypath") && has(b"zstructuredmetadata"))
        || (has(b"zobject") && has(b"zsource") && has(b"zstructuredmetadata"))
    {
        return Some(&SUB_APPLE_KNOWLEDGEC);
    }
    if has(b"zcredentialitem")
        && (has(b"zaccountproperty") || (has(b"zaccount") && has(b"zaccounttype")))
    {
        return Some(&SUB_APPLE_ACCOUNTS3);
    }
    if (has(b"zblueprint") && has(b"zblueprintconfiguration"))
        || (has(b"zusageblock") && has(b"zusagetimeditem"))
        || has(b"zrmcloudactivation")
    {
        return Some(&SUB_APPLE_SCREENTIME);
    }
    if has(b"znetworkattachment")
        && (has(b"zliverouteperf") || (has(b"zliveusage") && has(b"zprocess")))
    {
        return Some(&SUB_APPLE_NETUSAGE);
    }
    if has(b"zliveusage") && has(b"zprocess") {
        return Some(&SUB_APPLE_DATAUSAGE);
    }
    if has(b"zcatalogroot") && has(b"zcollectionmd") {
        return Some(&SUB_APPLE_FCP_EVENT);
    }
    if has(b"z_metadata") && has(b"z_primarykey") {
        return Some(&SUB_APPLE_COREDATA);
    }
    if has(b"workspec") && has(b"worktag") && has(b"workname") {
        return Some(&SUB_ANDROID_WORKMANAGER);
    }
    if has(b"echoapplaunchmetricsevents")
        || (has(b"pecanconversationthreadevents") && has(b"livetranslatechipevents"))
        || (has(b"safecommsentities") && has(b"room_master_table"))
    {
        return Some(&SUB_ANDROID_SIMPLESTORAGE);
    }
    if has(b"app_usage_periods") && has(b"app_usage_period_time_blocks") && has(b"component_events")
    {
        return Some(&SUB_ANDROID_DIGITAL_WELLBEING);
    }
    if has(b"beacon") && has(b"device") && has(b"feedback") && has(b"notification") && has(b"scan")
    {
        return Some(&SUB_ANDROID_AIRGUARD);
    }
    if has(b"files_master_table")
        && has(b"files_metadata_table")
        && has(b"files_classification_table")
    {
        return Some(&SUB_FILES_BY_GOOGLE);
    }
    if has(b"battery_event") && has(b"charging_places") {
        return Some(&SUB_ANDROID_TURBO_BATTERY);
    }
    if has(b"raw_events") && has(b"raw_events_metadata") && has(b"event_filters") {
        return Some(&SUB_FIREBASE_APP_MEASUREMENT);
    }
    if has(b"room_master_table") {
        return Some(&SUB_ANDROID_ROOM);
    }
    if (has(b"agg_exceptions") && has(b"contacts") && has(b"calls"))
        || (has(b"contacts") && has(b"data") && has(b"mimetypes") && has(b"raw_contacts"))
    {
        return Some(&SUB_ANDROID_CONTACTS);
    }
    if has(b"canonical_addresses") && has(b"pdu") && has(b"sms") {
        return Some(&SUB_ANDROID_MMSSMS);
    }
    if has(b"carriers") && has(b"siminfo") {
        return Some(&SUB_ANDROID_TELEPHONY);
    }
    if has(b"downloads") && has(b"request_headers") {
        return Some(&SUB_ANDROID_DOWNLOADS);
    }
    if has(b"attendees") && has(b"calendaralerts") && has(b"calendars") {
        return Some(&SUB_ANDROID_CALENDAR);
    }
    if has(b"bluetooth_devices") && has(b"bookmarks") && has(b"secure") && has(b"system") {
        return Some(&SUB_ANDROID_SETTINGS);
    }
    if has(b"accounts") && has(b"authtokens") && has(b"grants") {
        return Some(&SUB_ANDROID_ACCOUNTS);
    }
    if has(b"formdata") && has(b"formurl") && has(b"httpauth") {
        return Some(&SUB_ANDROID_WEBVIEW_CREDS);
    }
    if has(b"blobentry") && has(b"keyentry") && has(b"keyparameter") {
        return Some(&SUB_ANDROID_KEYSTORE);
    }
    if has(b"album_art") && has(b"audio_playlists_map") && has(b"files") {
        return Some(&SUB_ANDROID_MEDIA);
    }
    if has(b"channels") && has(b"programs") && has(b"watched_programs") {
        return Some(&SUB_ANDROID_TV);
    }
    if has(b"activity") && has(b"activityoperation") {
        return Some(&SUB_WINDOWS_TIMELINE);
    }
    if has(b"events_persisted") && (has(b"provider_groups") || has(b"producers")) {
        return Some(&SUB_WINDOWS_EVENTTRANSCRIPT);
    }
    if has(b"handlerassets") && has(b"notificationhandler") && has(b"wnspushchannel") {
        return Some(&SUB_WINDOWS_WPN);
    }
    if (has(b"application") && (has(b"appexecutionalias") || has(b"applicationfamilypolicy")))
        || (has(b"appxmanifest") && has(b"contentgroup") && has(b"packagesourceuri"))
    {
        return Some(&SUB_WINDOWS_STATEREPOSITORY);
    }
    if has(b"algorithms") && has(b"appstoskus") && has(b"blocksource") {
        return Some(&SUB_OFFICE_C2R);
    }
    if has(b"actionrecords") && has(b"completedupdates") {
        return Some(&SUB_WINDOWS_UPDATE_STORE);
    }
    if has(b"asimovevents") {
        return Some(&SUB_WINDOWS_ASIMOV);
    }
    if has(b"filescannercache") && has(b"filesystementity") && has(b"providerentity") {
        return Some(&SUB_VS_SLNX_INDEX);
    }
    if has(b"assoc_spans") && has(b"base_class_parents") && has(b"code_items") {
        return Some(&SUB_VS_CPP_BROWSE);
    }
    if has(b"actual_node") && has(b"nodes") && has(b"wcroot") {
        return Some(&SUB_SVN_WC);
    }
    if has(b"rep_cache") {
        return Some(&SUB_SVN_REP_CACHE);
    }
    if has(b"blob") && has(b"delta") && has(b"rcvfrom") {
        return Some(&SUB_FOSSIL_REPO);
    }
    if has(b"vfile") && has(b"vmerge") && has(b"vvar") {
        return Some(&SUB_FOSSIL_CHECKOUT);
    }
    if has(b"revision_ancestry") && has(b"revision_certs") && has(b"rosters") {
        return Some(&SUB_MONOTONE_REPO);
    }
    if (has(b"analysis_cache") && has(b"file_section")) || (has(b"snapshots") && has(b"raw")) {
        return Some(&SUB_BINARY_NINJA);
    }
    if has(b"metadata")
        && has(b"file")
        && has(b"function")
        && has(b"basicblock")
        && has(b"instruction")
    {
        return Some(&SUB_BINDIFF_DB);
    }
    if (has(b"functions")
        && has(b"basic_blocks")
        && has(b"bb_instructions")
        && has(b"function_bblocks"))
        || (has(b"config") && has(b"results") && has(b"unmatched"))
    {
        return Some(&SUB_DIAPHORA_DB);
    }
    if (has(b"ida_info") || has(b"ida_config"))
        && has(b"funcs")
        && has(b"segments")
        && (has(b"xrefs")
            || has(b"netnode_kv")
            || has(b"fchunks")
            || has(b"function_chunks")
            || has(b"instructions"))
    {
        return Some(&SUB_IDASQL_EXPORT);
    }
    if has(b"funcs")
        && has(b"blocks")
        && (has(b"pcode_ops")
            || has(b"pcode_varnodes")
            || (has(b"decomp_lvars") && has(b"decomp_tokens"))
            || (has(b"memory_blocks") && has(b"analysis_passes")))
    {
        return Some(&SUB_GHIDRASQL_EXPORT);
    }
    if has(b"funcs")
        && has(b"blocks")
        && has(b"cfg_edges")
        && (has(b"mlil")
            || has(b"mlil_operands")
            || has(b"hlil_vars")
            || (has(b"byte_search") && has(b"binary")))
    {
        return Some(&SUB_BNSQL_EXPORT);
    }
    if has(b"compilands")
        && has(b"udts")
        && has(b"udt_records")
        && (has(b"udt_fields")
            || has(b"udt_methods")
            || has(b"enum_records")
            || has(b"line_numbers"))
    {
        return Some(&SUB_PDBSQL_EXPORT);
    }
    if has(b"symbol_hashes") && has(b"source_libs") && has(b"hash_types") {
        return Some(&SUB_FLARE_SHELLCODE_HASHES);
    }
    if has(b"info") && has(b"lib") && has(b"bbl") && has(b"ins") && has(b"mem") && has(b"thread") {
        return Some(&SUB_SIDECHANNELMARVELS_TRACER);
    }
    if has(b"element")
        && has(b"element_component")
        && has(b"edge")
        && has(b"node")
        && has(b"symbol")
        && has(b"source_location")
        && has(b"occurrence")
    {
        return Some(&SUB_SOURCETRAIL_DB);
    }
    if has(b"information")
        && has(b"objects")
        && has(b"knowledgebases")
        && has(b"cfgs")
        && has(b"functions")
        && (has(b"callgraphs") || has(b"structured_code") || has(b"decompilation_caches"))
    {
        return Some(&SUB_ANGR_DB);
    }
    if has(b"steps")
        && has(b"trajectory_meta")
        && (has(b"trajectory_metadata_blob") || has(b"gen_metadata") || has(b"executor_metadata"))
    {
        return Some(&SUB_ANTIGRAVITY_CLI_SESSION);
    }
    if has(b"session")
        && has(b"message")
        && has(b"part")
        && has(b"project")
        && (has(b"todo") || has(b"session_share") || has(b"permission"))
    {
        return Some(&SUB_OPENCODE_SESSION);
    }
    if has(b"contentsfiltertable") && has(b"filedatatable") && has(b"namespacetable") {
        return Some(&SUB_QT_COMPRESSED_HELP);
    }
    if has(b"filterattributetable")
        && has(b"filternametable")
        && has(b"filtertable")
        && has(b"foldertable")
        && has(b"namespacetable")
        && has(b"settingstable")
    {
        return Some(&SUB_QT_HELP_COLLECTION);
    }
    if has(b"s3api_per_key_metadata") && has(b"s3item_history") {
        return Some(&SUB_INTERNET_ARCHIVE_META);
    }
    if has(b"validpaths") && has(b"refs") && has(b"derivationoutputs") {
        return Some(&SUB_NIX_STORE);
    }
    if has(b"binarycaches") && has(b"nars") && has(b"realisations") {
        return Some(&SUB_NIX_BINARY_CACHE);
    }
    if has(b"buildids10_buildids") && has(b"buildids10_fileparts") {
        return Some(&SUB_DEBUGINFOD_DB);
    }
    if has(b"selected_events") && has(b"comm_threads") && has(b"dsos") {
        return Some(&SUB_LINUX_PERF_DB);
    }
    if has(b"ultimately_trusted_keys") && has(b"bindings") && has(b"signatures") {
        return Some(&SUB_GNUPG_TOFU);
    }
    if has(b"pubkey") && has(b"fingerprint") && has(b"userid") && (has(b"ublob") || has(b"issuer"))
    {
        return Some(&SUB_GNUPG_KEYBOX);
    }
    if has(b"master") && has(b"caches") && has(b"credentials") {
        return Some(&SUB_HEIMDAL_KRB5);
    }
    if has(b"test_programs") && has(b"test_cases") && has(b"test_results") {
        return Some(&SUB_KYUA_RESULTS);
    }
    if has(b"mandb") && has(b"mandb_meta") && has(b"mandb_links") {
        return Some(&SUB_MAKEMANDB);
    }
    if has(b"mpages") && has(b"mlinks") {
        return Some(&SUB_MANDOC_DB);
    }
    if has(b"nbsignatures") {
        return Some(&SUB_JUPYTER_NBSIG);
    }
    if has(b"sessions") && has(b"history") && has(b"output_history") {
        return Some(&SUB_IPYTHON_HISTORY);
    }
    if has(b"tsk_db_info") && has(b"tsk_objects") && has(b"tsk_image_info") {
        return Some(&SUB_AUTOPSY_TSK);
    }
    if (has(b"winevtrc_eventlog_provider")
        && (has(b"winevtrc_message_string")
            || (has(b"winevtrc_message_file") && has(b"winevtrc_message_table"))))
        || (has(b"event_log_providers") && has(b"message_files") && has(b"message_tables"))
    {
        return Some(&SUB_PLASO_WINEVT_RC);
    }
    if has(b"event_data")
        && has(b"metadata")
        && (has(b"session_start")
            || has(b"event_tag")
            || (has(b"event_source") && has(b"parser_count")))
    {
        return Some(&SUB_PLASO_STORAGE);
    }
    if has(b"alerts") && has(b"datasources") && has(b"devices") && has(b"kismet") && has(b"packets")
    {
        return Some(&SUB_KISMET_CAPTURE);
    }
    if has(b"dashboard") && has(b"data_source") && has(b"org_user") {
        return Some(&SUB_GRAFANA_DB);
    }
    if has(b"states") && has(b"state_attributes") && has(b"event_data") {
        return Some(&SUB_HOME_ASSISTANT_DB);
    }
    if has(b"dag") && has(b"dag_run") && has(b"task_instance") && has(b"xcom") {
        return Some(&SUB_AIRFLOW_DB);
    }
    if has(b"kine") {
        return Some(&SUB_K3S_KINE_DB);
    }
    if has(b"fail2bandb") && has(b"jails") && has(b"bans") {
        return Some(&SUB_FAIL2BAN_DB);
    }
    if has(b"sys_osinfo") && has(b"sys_hwinfo") && has(b"sys_programs") {
        return Some(&SUB_WAZUH_INVENTORY);
    }
    if has(b"tensors") && has(b"tensorstrings") && has(b"runs") && has(b"experiments") {
        return Some(&SUB_TENSORBOARD_DB);
    }
    if has(b"experiments") && has(b"runs") && has(b"latest_metrics") && has(b"params") {
        return Some(&SUB_MLFLOW_DB);
    }
    if has(b"studies") && has(b"trials") && has(b"trial_params") && has(b"trial_values") {
        return Some(&SUB_OPTUNA_DB);
    }
    if has(b"full_llm_cache") || has(b"full_md5_llm_cache") {
        return Some(&SUB_LANGCHAIN_CACHE);
    }
    if has(b"acquire_write") && has(b"embedding_fulltext_search") && has(b"tenants") {
        return Some(&SUB_CHROMADB);
    }
    if (has(b"taginventory_table") && has(b"coral_sqlite_fk"))
        || (has(b"payload")
            && has(b"iov")
            && has(b"tag")
            && has(b"global_tag")
            && has(b"global_tag_map"))
    {
        return Some(&SUB_CERN_CMS_CONDDB);
    }
    if has(b"actor") && has(b"interpretation") && has(b"manifestation") {
        return Some(&SUB_GNOME_ZEITGEIST);
    }
    if has(b"adobe_imagedevelopsettings") && has(b"aglibraryfile") {
        return Some(&SUB_ADOBE_LIGHTROOM);
    }
    if has(b"adobe_variables") && has(b"filespecsneedingmetadata") {
        return Some(&SUB_ADOBE_LIGHTROOM_TEMP);
    }
    if has(b"applet_registration_info") && has(b"package_installation_info") {
        return Some(&SUB_ADOBE_CC_PIM);
    }
    if has(b"collection_data") && has(b"pcd_meta") && has(b"ribs_collection") {
        return Some(&SUB_ADOBE_CC_PCD_MASTER);
    }
    if has(b"payloaddata") && has(b"payloads") && has(b"suitepayloads") && has(b"suites") {
        return Some(&SUB_ADOBE_MEDIA_DB);
    }
    if has(b"resource_group_nodes") && has(b"resource_request_operations") {
        return Some(&SUB_ADOBE_ACROBAT_SYNC);
    }
    if has(b"controlmessages") && has(b"ipmmessage") {
        return Some(&SUB_ADOBE_READER_MSGS);
    }
    if has(b"canvaspreview") && has(b"layerthumbnail") && has(b"mipmap") {
        return Some(&SUB_CLIP_STUDIO_PAINT);
    }
    if has(b"maplecode") && has(b"workbookmodel") {
        return Some(&SUB_MAPLE_WORKBOOK);
    }
    if has(b"billsdeposits") && has(b"budgetsplittransactions") {
        return Some(&SUB_MONEY_MANAGER_EX);
    }
    if (has(b"books_authors_link") && has(b"books_tags_link"))
        || (has(b"annotations_dirtied") && has(b"annotations_fts"))
    {
        return Some(&SUB_CALIBRE_LIBRARY);
    }
    if has(b"cardmodels") && has(b"deckvars") {
        return Some(&SUB_ANKI_1_DECK);
    }
    if has(b"film_rolls") && has(b"images") && has(b"history") {
        return Some(&SUB_DARKTABLE_LIB);
    }
    if has(b"style_items") && has(b"styles") && has(b"presets") {
        return Some(&SUB_DARKTABLE_DATA);
    }
    if has(b"albumroots") && has(b"albums") && has(b"imagetags") {
        return Some(&SUB_DIGIKAM_DB);
    }
    if has(b"library_sections") && has(b"metadata_items") && has(b"media_parts") {
        return Some(&SUB_PLEX_LIBRARY);
    }
    if (has(b"typedbaseitems") || has(b"baseitems")) && has(b"ancestorids") && has(b"itemvalues") {
        return Some(&SUB_JELLYFIN_LIBRARY);
    }
    if (has(b"actor_link") && has(b"director_link") && has(b"movielinktvshow"))
        || (has(b"movie")
            && has(b"episode")
            && has(b"tvshow")
            && has(b"streamdetails")
            && has(b"movielinktvshow"))
    {
        return Some(&SUB_KODI_VIDEOS);
    }
    if has(b"conversations") && has(b"messages") && has(b"unprocessed") && has(b"identitykeys") {
        return Some(&SUB_SIGNAL_DESKTOP);
    }
    if has(b"identities") && has(b"prekeys") && has(b"sender_keys") && has(b"signed_prekeys") {
        return Some(&SUB_WHATSAPP_AXOLOTL);
    }
    if has(b"olm_sessions") && has(b"inbound_group_sessions") && has(b"tracked_users") {
        return Some(&SUB_MATRIX_RUST_SDK);
    }
    if has(b"appschemaversion") && has(b"callmembers") && has(b"chatmembers") {
        return Some(&SUB_SKYPE_MAIN_DB);
    }
    if (has(b"chat") && has(b"message") && has(b"message_media"))
        || (has(b"chat_list")
            && has(b"messages")
            && (has(b"media_refs") || has(b"message_thumbnails") || has(b"receipts")))
    {
        return Some(&SUB_WHATSAPP_MSGSTORE);
    }
    if has(b"wa_contacts")
        && (has(b"wa_contact_capabilities") || has(b"wa_vnames") || has(b"wa_biz_profiles"))
    {
        return Some(&SUB_WHATSAPP_CONTACTS);
    }
    if has(b"conversations")
        && has(b"conversation_participants")
        && has(b"messages")
        && (has(b"mms_notification_inds") || has(b"sticker_albums") || has(b"merged_contacts"))
    {
        return Some(&SUB_GOOGLE_HANGOUTS);
    }
    if has(b"burners")
        && has(b"connections")
        && has(b"contacts")
        && has(b"messages")
        && has(b"subscriptions")
    {
        return Some(&SUB_ANDROID_BURNER);
    }
    if has(b"conversations")
        && has(b"games")
        && has(b"likes")
        && has(b"messages")
        && has(b"profiles")
        && has(b"receipts")
        && has(b"sms")
    {
        return Some(&SUB_TANGO_ANDROID);
    }
    if has(b"statuses")
        && has(b"status_groups")
        && has(b"timeline")
        && has(b"users")
        && (has(b"moments_guide") || has(b"user_groups") || has(b"business_profiles"))
    {
        return Some(&SUB_ANDROID_TWITTER);
    }
    if has(b"statuses")
        && has(b"statusesshadow")
        && has(b"users")
        && has(b"usersshadow")
        && (has(b"myretweets") || has(b"listsshadow"))
    {
        return Some(&SUB_IOS_TWITTER);
    }
    if has(b"vibernumbers") && has(b"phonebookcontact") && has(b"phonebookdata") && has(b"calls") {
        return Some(&SUB_ANDROID_VIBER);
    }
    if has(b"inbox_metadata")
        && has(b"messages")
        && has(b"mutations")
        && has(b"quick_reply")
        && has(b"thread_client_state")
        && has(b"threads")
    {
        return Some(&SUB_IOS_INSTAGRAM_DIRECT);
    }
    if has(b"cloud_entry")
        && has(b"local_entry")
        && (has(b"overlay_status") || (has(b"cloud_relations") && has(b"local_relations")))
    {
        return Some(&SUB_GOOGLE_DRIVE_SNAPSHOT);
    }
    if has(b"fuzzy_data") && has(b"date_attributes") && has(b"picklist_values") {
        return Some(&SUB_SDL_TRADOS);
    }
    if has(b"diagram") && has(b"diagram_element") && has(b"project_info") {
        return Some(&SUB_VISUAL_PARADIGM);
    }
    if has(b"appcodes") && has(b"computercodes") && has(b"eventlog") {
        return Some(&SUB_ONE_C_EVENTLOG);
    }
    if has(b"postponed_pkg_operations") && has(b"registry_meta") {
        return Some(&SUB_CORSAIR_ICUE);
    }
    if has(b"items") && has(b"texts") && (has(b"xsett") || has(b"xremoved")) {
        return Some(&SUB_QUICK_MACROS_2);
    }
    if has(b"co_block") && has(b"co_chat") && has(b"co_container") {
        return Some(&SUB_MINECRAFT_COREPROTECT);
    }
    if has(b"audio_manuals") && has(b"editions_manuals") {
        return Some(&SUB_DENON_MARANTZ_DMU);
    }
    if has(b"album_artwork") && has(b"genre") && has(b"audio") {
        return Some(&SUB_EMBEDDED_MEDIA_LIB);
    }
    if matches!(enc, TextEncoding::Utf16Le | TextEncoding::Utf16Be) && has(b"cachedplugins") {
        return Some(&SUB_DAW_VST_CACHE);
    }

    None
}
