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

//! Network packet capture and peer-to-peer metainfo format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// IETF pcapng Section Header Block: block type `0x0A0D0D0A`, byte-order magic `0x1A2B3C4D` at `@8`
/// in either endianness with major version `1` at `@12`, and ≥ 28-byte minimum SHB.
pub(crate) fn pcapng(prefix: &[u8]) -> Option<Label> {
    const SHB_TYPE: &[u8] = b"\x0a\x0d\x0d\x0a"; // @0..4: Section Header Block type
    const BYTE_ORDER_MAGIC: u32 = 0x1A2B_3C4D; // @8: u32 — endianness marker
    const BYTE_ORDER_OFFSET: usize = 8;
    const MAJOR_VERSION: usize = 12; // @12: u16 — major version (1)
    const MIN_LEN: usize = 28;

    require!(prefix.len() >= MIN_LEN, starts(prefix, SHB_TYPE));
    let valid = (le32(prefix, BYTE_ORDER_OFFSET)? == BYTE_ORDER_MAGIC
        && le16(prefix, MAJOR_VERSION)? == 1)
        || (be32(prefix, BYTE_ORDER_OFFSET)? == BYTE_ORDER_MAGIC
            && be16(prefix, MAJOR_VERSION)? == 1);
    require!(valid);
    Some(Label::Pcapng)
}

/// libpcap capture file: microsecond (`0xA1B2C3D4`) or nanosecond (`0xA1B23C4D`) magic in either
/// byte order, version 2.4 at `@4..8`, and nonzero `snaplen` at `@16`.
pub(crate) fn pcap(prefix: &[u8]) -> Option<Label> {
    const MAGIC_USEC: u32 = 0xA1B2_C3D4; // microsecond-resolution pcap magic
    const MAGIC_NSEC: u32 = 0xA1B2_3C4D; // nanosecond-resolution pcap magic
    const MAJOR: usize = 4; // @4: u16 — major version (2)
    const MINOR: usize = 6; // @6: u16 — minor version (4)
    const SNAPLEN: usize = 16; // @16: u32 — maximum captured packet length (> 0)
    const HEADER_LEN: usize = 24;

    require!(prefix.len() >= HEADER_LEN);
    let le_ok = matches!(le32(prefix, 0)?, MAGIC_USEC | MAGIC_NSEC)
        && le16(prefix, MAJOR)? == 2
        && le16(prefix, MINOR)? == 4
        && le32(prefix, SNAPLEN)? > 0;
    let be_ok = matches!(be32(prefix, 0)?, MAGIC_USEC | MAGIC_NSEC)
        && be16(prefix, MAJOR)? == 2
        && be16(prefix, MINOR)? == 4
        && be32(prefix, SNAPLEN)? > 0;
    require!(le_ok || be_ok);
    Some(Label::Pcap)
}

/// BitTorrent metainfo file (`.torrent`, BEP 3/12/52): bencoded dictionary opening with a known
/// root key (`announce-list`, `info`, `announce`, or `comment`), corroborated either by a typed
/// tracker URL (`http`/`https`/`udp`/`ws`/`wss`) at `@0` or by `4:infod`, `12:piece lengthi...e`,
/// and `4:name...:` within the prefix.
pub(crate) fn torrent(prefix: &[u8]) -> Option<Label> {
    const MIN_LEN: usize = 20;

    /// Checks `([7-9]|[1-9][0-9]{1,3}):(https?|udp|wss?):\/\/[\x21-\xff]` at the start of `s`.
    fn has_tracker_url(s: &[u8]) -> bool {
        let Some(&first_digit) = s.first() else {
            return false;
        };
        if !matches!(first_digit, b'1'..=b'9') {
            return false;
        }
        let mut d = 1;
        while d < s.len() && s[d].is_ascii_digit() {
            d += 1;
        }
        let valid_len = (d == 1 && first_digit >= b'7') || (2..=4).contains(&d);
        if !valid_len || s.get(d) != Some(&b':') {
            return false;
        }
        let url = &s[d + 1..];
        const SCHEMES: &[&[u8]] = &[b"http://", b"https://", b"udp://", b"ws://", b"wss://"];
        for scheme in SCHEMES {
            if url.starts_with(scheme) {
                return matches!(url.get(scheme.len()), Some(0x21..=0xFF));
            }
        }
        false
    }

    /// Checks `12:piece lengthi[1-9][0-9]{0,18}e` starting in `0..=4060`.
    fn has_piece_length(prefix: &[u8]) -> bool {
        const TAG: &[u8] = b"12:piece lengthi";
        for pos in memchr::memmem::find_iter(prefix, TAG) {
            if pos > 4060 {
                break;
            }
            let s = &prefix[pos + TAG.len()..];
            let Some(&first) = s.first() else {
                continue;
            };
            if !matches!(first, b'1'..=b'9') {
                continue;
            }
            let mut d = 1;
            while d < s.len() && s[d].is_ascii_digit() {
                d += 1;
            }
            if d <= 19 && s.get(d) == Some(&b'e') {
                return true;
            }
        }
        false
    }

    /// Checks `4:name[1-9][0-9]{0,3}:` starting in `0..=4085`.
    fn has_name(prefix: &[u8]) -> bool {
        const TAG: &[u8] = b"4:name";
        for pos in memchr::memmem::find_iter(prefix, TAG) {
            if pos > 4085 {
                break;
            }
            let s = &prefix[pos + TAG.len()..];
            let Some(&first) = s.first() else {
                continue;
            };
            if !matches!(first, b'1'..=b'9') {
                continue;
            }
            let mut d = 1;
            while d < s.len() && s[d].is_ascii_digit() {
                d += 1;
            }
            if d <= 4 && s.get(d) == Some(&b':') {
                return true;
            }
        }
        false
    }

    const ROOT_ANNOUNCE_LIST: &[u8] = b"d13:announce-list";
    const ROOT_ANNOUNCE_LIST_LL: &[u8] = b"d13:announce-listll";
    const ROOT_INFO: &[u8] = b"d4:info";
    const ROOT_ANNOUNCE: &[u8] = b"d8:announce";
    const ROOT_COMMENT: &[u8] = b"d7:comment";
    const TAG_INFOD: &[u8] = b"4:infod";

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, ROOT_ANNOUNCE_LIST)
            || starts(prefix, ROOT_INFO)
            || starts(prefix, ROOT_ANNOUNCE)
            || starts(prefix, ROOT_COMMENT),
    );
    let valid = (starts(prefix, ROOT_ANNOUNCE) && has_tracker_url(&prefix[11..]))
        || (starts(prefix, ROOT_ANNOUNCE_LIST_LL) && has_tracker_url(&prefix[19..]))
        || (scan_range(prefix, 0, 4089, TAG_INFOD)
            && has_piece_length(prefix)
            && has_name(prefix));
    require!(valid);
    let _ = get_slice(prefix, 0, prefix.len().min(96));
    Some(Label::Torrent)
}
