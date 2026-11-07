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

//! ASN.1 BER/DER Universal SEQUENCE (`0x30`) disambiguator for X.509 Certificates (`crt`)
//! and Microsoft Authenticode Security Catalogs (`cat`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

const MIN_X509_CERT_SIZE: u64 = 128;
const MIN_CATALOG_SIZE: u64 = 128;

/// ASN.1 BER/DER Universal `INTEGER` tag (`0x02`).
const ASN1_TAG_INTEGER: u8 = 0x02;
/// ASN.1 BER/DER Universal `OBJECT IDENTIFIER` (`OID`) tag (`0x06`).
const ASN1_TAG_OID: u8 = 0x06;
/// ASN.1 BER/DER Universal constructed `SEQUENCE` tag (`0x30`).
pub(crate) const ASN1_TAG_SEQUENCE: u8 = 0x30;
/// ASN.1 BER/DER Context-Specific constructed `[0] EXPLICIT` tag (`0xA0`).
const ASN1_TAG_CONTEXT_0: u8 = 0xA0;

/// ASN.1 BER indefinite-length indicator byte (`0x80`).
const ASN1_LEN_INDEFINITE: u8 = 0x80;
/// High bit indicating ASN.1 BER/DER long-form length (`0x80`).
const ASN1_LEN_LONG_FORM_BIT: u8 = 0x80;
/// Low 7-bit mask for the octet count of an ASN.1 long-form length (`0x7F`).
const ASN1_LEN_OCTETS_MASK: u8 = 0x7F;

/// PKCS#7 `signedData` OID (`1.2.840.113549.1.7.2`).
const OID_PKCS7_SIGNED_DATA: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x02];
/// PKCS#7 `data` OID (`1.2.840.113549.1.7.1`).
const OID_PKCS7_DATA: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x01];
/// Microsoft Certificate Trust List (`szOID_CTL`, `1.3.6.1.4.1.311.10.1`).
const OID_MS_CTL: &[u8] = &[0x2B, 0x06, 0x01, 0x04, 0x01, 0x82, 0x37, 0x0A, 0x01];
/// Microsoft Catalog List (`szOID_CATALOG_LIST`, `1.3.6.1.4.1.311.12.1.1`).
const OID_MS_CATALOG_LIST: &[u8] = &[0x2B, 0x06, 0x01, 0x04, 0x01, 0x82, 0x37, 0x0C, 0x01, 0x01];

/// Parses a single ASN.1 BER/DER TLV header at `pos` inside `buf`, returning
/// `(tag, content_start, content_len, next_pos)`.
fn read_tlv(buf: &[u8], pos: usize) -> Option<(u8, usize, usize, usize)> {
    let tag = u8_at(buf, pos)?;
    let len_byte = u8_at(buf, pos + 1)?;
    if len_byte == ASN1_LEN_INDEFINITE {
        // Indefinite-length BER: content starts at pos + 2
        return Some((tag, pos + 2, buf.len().saturating_sub(pos + 2), buf.len()));
    }
    if (len_byte & ASN1_LEN_LONG_FORM_BIT) == 0 {
        let len = len_byte as usize;
        let start = pos + 2;
        let next = start.checked_add(len)?;
        return Some((tag, start, len, next));
    }
    let num_octets = (len_byte & ASN1_LEN_OCTETS_MASK) as usize;
    require!((1..=4).contains(&num_octets));
    let mut len = 0usize;
    for i in 0..num_octets {
        len = (len << 8) | (u8_at(buf, pos + 2 + i)? as usize);
    }
    let start = pos + 2 + num_octets;
    let next = start.checked_add(len)?;
    Some((tag, start, len, next))
}

/// Verifies an ASN.1 BER/DER Universal `SEQUENCE` (`0x30`) for Magika's `cat` (Microsoft
/// Security Catalog) and `crt` (X.509 Certificate or PKCS#7 `.p7b` certificate bundle).
pub(crate) fn verify_asn1_der(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_X509_CERT_SIZE,
        prefix.len() >= 32,
        u8_at(prefix, 0)? == ASN1_TAG_SEQUENCE,
    );

    let (root_tag, root_start, _root_len, _root_end) = read_tlv(prefix, 0)?;
    require!(root_tag == ASN1_TAG_SEQUENCE);

    let (c0_tag, c0_start, c0_len, c0_end) = read_tlv(prefix, root_start)?;

    // Branch 1: CMS / PKCS#7 ContentInfo starting with OID `signedData` (0x06)
    if c0_tag == ASN1_TAG_OID {
        let oid_slice = get_slice(prefix, c0_start, c0_start + c0_len)?;
        if oid_slice == OID_PKCS7_SIGNED_DATA {
            let (c1_tag, _c1_start, _c1_len, _) = read_tlv(prefix, c0_end)?;
            require!(c1_tag == ASN1_TAG_CONTEXT_0);
            if size >= MIN_CATALOG_SIZE
                && (contains(prefix, OID_MS_CATALOG_LIST) || contains(prefix, OID_MS_CTL))
            {
                return Some(Label::Cat);
            }
            if contains(prefix, OID_PKCS7_DATA) {
                return Some(Label::Crt);
            }
        }
        return None;
    }

    // Branch 2: X.509 Certificate (RFC 5280) — SEQUENCE { tbsCertificate SEQUENCE, signatureAlgorithm SEQUENCE, signatureValue BIT STRING }
    require!(c0_tag == ASN1_TAG_SEQUENCE);
    let (tbs0_tag, tbs0_start, _, tbs0_end) = read_tlv(prefix, c0_start)?;
    let valid_tbs = if tbs0_tag == ASN1_TAG_CONTEXT_0 {
        // v2/v3 explicit version [0] { INTEGER 1 | 2 } followed by serialNumber INTEGER (0x02)
        let (ver_tag, ver_start, ver_len, _) = read_tlv(prefix, tbs0_start)?;
        let (ser_tag, _, _, _) = read_tlv(prefix, tbs0_end)?;
        ver_tag == ASN1_TAG_INTEGER
            && ver_len == 1
            && matches!(u8_at(prefix, ver_start)?, 0..=2)
            && ser_tag == ASN1_TAG_INTEGER
    } else if tbs0_tag == ASN1_TAG_INTEGER {
        // v1 certificate: serialNumber INTEGER (0x02) followed by signature AlgorithmIdentifier SEQUENCE (0x30) containing OID (0x06)
        let (alg_tag, alg_start, _, _) = read_tlv(prefix, tbs0_end)?;
        alg_tag == ASN1_TAG_SEQUENCE && u8_at(prefix, alg_start) == Some(ASN1_TAG_OID)
    } else {
        false
    };
    require!(valid_tbs);

    if c0_end < prefix.len() {
        let (c1_tag, c1_start, _, _) = read_tlv(prefix, c0_end)?;
        require!(
            c1_tag == ASN1_TAG_SEQUENCE,
            u8_at(prefix, c1_start) == Some(ASN1_TAG_OID),
        );
    }

    Some(Label::Crt)
}
