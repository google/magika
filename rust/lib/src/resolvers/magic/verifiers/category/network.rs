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
    static PCAPNG_LE: Subformat = Subformat::new("pcapng (little-endian)");
    static PCAPNG_BE: Subformat = Subformat::new("pcapng (big-endian)");
    const SHB_TYPE: &[u8] = b"\x0a\x0d\x0d\x0a"; // @0..4: Section Header Block type
    const BYTE_ORDER_MAGIC: u32 = 0x1A2B_3C4D; // @8: u32 — endianness marker
    const BYTE_ORDER_OFFSET: usize = 8;
    const MAJOR_VERSION: usize = 12; // @12: u16 — major version (1)
    const MIN_LEN: usize = 28;

    require!(prefix.len() >= MIN_LEN, starts(prefix, SHB_TYPE));
    let sub = if le32(prefix, BYTE_ORDER_OFFSET)? == BYTE_ORDER_MAGIC
        && le16(prefix, MAJOR_VERSION)? == 1
    {
        &PCAPNG_LE
    } else if be32(prefix, BYTE_ORDER_OFFSET)? == BYTE_ORDER_MAGIC
        && be16(prefix, MAJOR_VERSION)? == 1
    {
        &PCAPNG_BE
    } else {
        return None;
    };
    with_subformat(Label::Pcapng, sub)
}

/// libpcap capture file: microsecond (`0xA1B2C3D4`) or nanosecond (`0xA1B23C4D`) magic in either
/// byte order, version 2.4 at `@4..8`, and nonzero `snaplen` at `@16`.
pub(crate) fn pcap(prefix: &[u8]) -> Option<Label> {
    static PCAP_USEC_LE: Subformat = Subformat::new("libpcap (microsecond, little-endian)");
    static PCAP_NSEC_LE: Subformat = Subformat::new("libpcap (nanosecond, little-endian)");
    static PCAP_USEC_BE: Subformat = Subformat::new("libpcap (microsecond, big-endian)");
    static PCAP_NSEC_BE: Subformat = Subformat::new("libpcap (nanosecond, big-endian)");
    const MAGIC_USEC: u32 = 0xA1B2_C3D4; // microsecond-resolution pcap magic
    const MAGIC_NSEC: u32 = 0xA1B2_3C4D; // nanosecond-resolution pcap magic
    const MAJOR: usize = 4; // @4: u16 — major version (2)
    const MINOR: usize = 6; // @6: u16 — minor version (4)
    const SNAPLEN: usize = 16; // @16: u32 — maximum captured packet length (> 0)
    const HEADER_LEN: usize = 24;

    require!(prefix.len() >= HEADER_LEN);
    let le_magic = le32(prefix, 0)?;
    if matches!(le_magic, MAGIC_USEC | MAGIC_NSEC)
        && le16(prefix, MAJOR)? == 2
        && le16(prefix, MINOR)? == 4
        && le32(prefix, SNAPLEN)? > 0
    {
        return with_subformat(
            Label::Pcap,
            if le_magic == MAGIC_USEC { &PCAP_USEC_LE } else { &PCAP_NSEC_LE },
        );
    }
    let be_magic = be32(prefix, 0)?;
    require!(
        matches!(be_magic, MAGIC_USEC | MAGIC_NSEC),
        be16(prefix, MAJOR)? == 2,
        be16(prefix, MINOR)? == 4,
        be32(prefix, SNAPLEN)? > 0,
    );
    with_subformat(Label::Pcap, if be_magic == MAGIC_USEC { &PCAP_USEC_BE } else { &PCAP_NSEC_BE })
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
        || (scan_range(prefix, 0, 4089, TAG_INFOD) && has_piece_length(prefix) && has_name(prefix));
    require!(valid);
    let _ = get_slice(prefix, 0, prefix.len().min(96));
    Some(Label::Torrent)
}

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const MIN_METATRADER_SRV_SIZE: u64 = 64;
const MIN_MS_NETMON_NPB_SIZE: u64 = 32;
const MIN_SINGBOX_SRS_SIZE: u64 = 16;
const MIN_SNIFFER_DOS_SIZE: u64 = 12;
const MIN_TLS_SESSION_NET443_SIZE: u64 = 20;
const MIN_UA_RESUME1_SIZE: u64 = 36;
const MIN_VECTOR_BLF_SIZE: u64 = 144;

/// MetaQuotes MetaTrader Broker Server Definition (.srv).
pub(crate) fn verify_metatrader_srv(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_METATRADER_SRV_SIZE,
        prefix.len() >= 32,
        get_slice(prefix, 0, 14)? == hex2bin!("41 6C 70 61 72 69 2D 50 72 6F 2E 45 43 4E"),
        get_slice(prefix, 19, 32)? == hex2bin!("00 00 00 00 00 00 00 00 00 00 00 00 00")
    );
    require!(has_non_ascii_text_byte(prefix.get(..32)?));
    Some(Label::MetatraderSrv)
}

/// NPB UTF-16LE Path Backup List (.npb).
pub(crate) fn verify_ms_netmon_npb(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_MS_NETMON_NPB_SIZE,
        prefix.len() >= 22,
        starts(prefix, b"N\0P\0B\0\0\0\x06\0\0\0\x01\0\0\0\xfd\x07\x1f\0\x01\0")
    );
    require!(has_non_ascii_text_byte(prefix.get(..22)?));
    Some(Label::MsNetmonNpb)
}

/// sing-box Binary Rule-Set Container (`SRS` v1–v5 + zlib stream, `.srs`).
pub(crate) fn verify_singbox_srs(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_SINGBOX_SRS_SIZE,
        prefix.len() >= 6,
        starts(prefix, b"SRS"),
        (1..=5).contains(&u8_at(prefix, 3)?),
        is_valid_zlib_header(u8_at(prefix, 4)?, u8_at(prefix, 5)?)
    );
    Some(Label::SingboxSrs)
}

/// Network General Sniffer DOS capture (TRSNIFWR).
pub(crate) fn verify_sniffer_dos(prefix: &[u8], size: u64) -> Option<Label> {
    if size >= MIN_SNIFFER_DOS_SIZE
        && prefix.len() >= 12
        && (starts(prefix, b"TRSNIFF data    \x1A")
            || ((starts(prefix, b"TRSNIFWR")
                || starts(prefix, b"TRSNIFF ")
                || starts(prefix, b"NETX"))
                && has_non_ascii_text_byte(prefix.get(4..prefix.len().min(18))?)))
    {
        Some(Label::Enc)
    } else {
        None
    }
}

/// Sun Microsystems Snoop packet capture (`.snoop`, RFC 1761).
pub(crate) fn verify_snoop(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 16, prefix.len() >= 16, starts(prefix, b"snoop\0\0\0"));
    Some(Label::Snoop)
}

/// ASN.1 DER SSL/TLS Session Cache Entry (.net_443, OpenSSL SSL_SESSION).
pub(crate) fn verify_tls_session_net443(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_TLS_SESSION_NET443_SIZE,
        prefix.len() >= 20,
        u8_at(prefix, 0)? == 0x30,
        u8_at(prefix, 1)? == 0x82,
        get_slice(prefix, 4, 16)? == b"\x02\x01\x01\x02\x02\x03\x04\x04\x02\x13\x01\x04"
    );
    require!(has_non_ascii_text_byte(prefix.get(..20)?));
    Some(Label::TlsSessionNet443)
}

/// UA Developments Patcher Download Resume State (UA-RESUME-1, .resume).
pub(crate) fn verify_ua_resume1(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_UA_RESUME1_SIZE,
        prefix.len() >= 36,
        starts(prefix, b"U\0A\0-\0R\0E\0S\0U\0M\0E\0-\x001\0\n\0h\0t\0t\0p\0")
    );
    require!(has_non_ascii_text_byte(prefix.get(..36)?));
    Some(Label::UaResume1)
}

/// Vector Informatik Binary Logging Format (`.blf`) CAN/LIN/FlexRay/Ethernet bus capture.
///
/// Header layout (`VBLFileStatisticsEx`):
/// - `@0..4`: `"LOGG"` (`0x47474F4C` LE file signature)
/// - `@4..8`: `u32 LE mStatisticsSize` (`144..=512`, typically `144` = `0x90`)
/// - `@8..12`: `u32 LE mApplicationId` / API version
/// - `@mStatisticsSize..mStatisticsSize+4`: First `VBLObjectHeaderBase` starting with `"LOBJ"` (`0x4A424F4C` LE).
pub(crate) fn verify_vector_blf(prefix: &[u8], size: u64) -> Option<Label> {
    const BLF_MAGIC: &[u8; 4] = b"LOGG"; // @0..4: VBLFileStatisticsEx signature
    const HDR_SIZE_OFFSET: usize = 4; // @4..8: u32 LE mStatisticsSize (>= 144)
    const LOBJ_MAGIC: &[u8; 4] = b"LOBJ"; // @mStatisticsSize..+4: VBLObjectHeaderBase signature

    require!(size >= MIN_VECTOR_BLF_SIZE, prefix.len() >= 144, starts(prefix, BLF_MAGIC));
    let hdr_size = le32(prefix, HDR_SIZE_OFFSET)? as usize;
    require!(
        (144..=512).contains(&hdr_size),
        (hdr_size as u64) <= size,
        size == hdr_size as u64
            || (prefix.len() >= hdr_size + 4 && eq_at(prefix, hdr_size, LOBJ_MAGIC))
    );
    Some(Label::VectorBlf)
}
