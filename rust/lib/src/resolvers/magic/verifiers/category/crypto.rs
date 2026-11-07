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

//! Cryptographic key, certificate, and armor format verifiers.

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// OpenPGP ASCII armor (`"-----BEGIN PGP "`) or RFC 7468 PEM certificate/key encapsulation boundary.
pub(crate) fn pem_or_pgp(prefix: &[u8]) -> Option<Label> {
    const BEGIN: &[u8] = b"-----BEGIN "; // @0..11: encapsulation boundary opening
    const PGP_ARMOR: &[u8] = b"-----BEGIN PGP "; // @0..15: OpenPGP armor header
    const PEM_LABELS: &[&[u8]] = &[
        b"CERTIFICATE",
        b"TRUSTED CERTIFICATE",
        b"CERTIFICATE REQUEST",
        b"NEW CERTIFICATE REQUEST",
        b"X509 CRL",
        b"PUBLIC KEY",
        b"PRIVATE KEY",
        b"ENCRYPTED PRIVATE KEY",
        b"RSA PRIVATE KEY",
        b"RSA PUBLIC KEY",
        b"DSA PRIVATE KEY",
        b"DSA PARAMETERS",
        b"EC PRIVATE KEY",
        b"EC PARAMETERS",
        b"DH PARAMETERS",
        b"PKCS7",
        b"CMS",
    ];

    if starts(prefix, PGP_ARMOR) {
        return Some(Label::Pgp);
    }
    const END_DASHES: &[u8] = b"-----";
    let rest = prefix.strip_prefix(BEGIN)?;
    for &label in PEM_LABELS {
        if let Some(after_label) = rest.strip_prefix(label)
            && let Some(after_dashes) = after_label.strip_prefix(END_DASHES)
            && (starts(after_dashes, b"\n") || starts(after_dashes, b"\r\n"))
        {
            return Some(Label::Pem);
        }
    }
    None
}

/// OpenPGP Binary Public/Secret Keyring (`pgp`, RFC 4880): packet tag `0x99`, `0x95`,
/// `0xC6`, or `0xC5` at `@0`, followed by key packet version `3..=6` and public-key
/// algorithm ID (`1|2|3|16|17|18|19|22|27|28`).
pub(crate) fn openpgp_binary(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= 32, prefix.len() >= 12);
    let b0 = u8_at(prefix, 0)?;
    let hdr_len = if matches!(b0, 0x99 | 0x95) {
        let pkt_len = be16(prefix, 1)? as usize;
        require!(pkt_len >= 6);
        3usize
    } else if matches!(b0, 0xC6 | 0xC5) {
        let l0 = u8_at(prefix, 1)?;
        if l0 < 192 {
            require!(l0 >= 6);
            2usize
        } else if l0 < 224 {
            3usize
        } else {
            return None;
        }
    } else {
        return None;
    };
    let ver = u8_at(prefix, hdr_len)?;
    let alg = u8_at(prefix, hdr_len + 5)?;
    require!(
        matches!(ver, 3..=6),
        matches!(alg, 1 | 2 | 3 | 16 | 17 | 18 | 19 | 22 | 27 | 28),
    );
    Some(Label::Pgp)
}
