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

    static PGP_ARMORED: Subformat = Subformat::new("OpenPGP ASCII armor");
    static PEM_CERTIFICATE: Subformat = Subformat::new("PEM certificate");
    static PEM_CERTIFICATE_REQUEST: Subformat = Subformat::new("PEM certificate request");
    static PEM_CRL: Subformat = Subformat::new("PEM X.509 CRL");
    static PEM_PUBLIC_KEY: Subformat = Subformat::new("PEM public key");
    static PEM_PRIVATE_KEY: Subformat = Subformat::new("PEM private key");
    static PEM_PARAMETERS: Subformat = Subformat::new("PEM parameters");
    static PEM_PKCS7: Subformat = Subformat::new("PEM PKCS #7 / CMS");

    if starts(prefix, PGP_ARMOR) {
        return with_subformat(Label::Pgp, &PGP_ARMORED);
    }
    const END_DASHES: &[u8] = b"-----";
    let rest = prefix.strip_prefix(BEGIN)?;
    for &label in PEM_LABELS {
        if let Some(after_label) = rest.strip_prefix(label)
            && let Some(after_dashes) = after_label.strip_prefix(END_DASHES)
            && (starts(after_dashes, b"\n") || starts(after_dashes, b"\r\n"))
        {
            let sub = match label {
                b"CERTIFICATE" | b"TRUSTED CERTIFICATE" => &PEM_CERTIFICATE,
                b"CERTIFICATE REQUEST" | b"NEW CERTIFICATE REQUEST" => &PEM_CERTIFICATE_REQUEST,
                b"X509 CRL" => &PEM_CRL,
                b"PUBLIC KEY" | b"RSA PUBLIC KEY" => &PEM_PUBLIC_KEY,
                b"DSA PARAMETERS" | b"EC PARAMETERS" | b"DH PARAMETERS" => &PEM_PARAMETERS,
                b"PKCS7" | b"CMS" => &PEM_PKCS7,
                _ => &PEM_PRIVATE_KEY,
            };
            return with_subformat(Label::Pem, sub);
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
    require!(matches!(ver, 3..=6), matches!(alg, 1 | 2 | 3 | 16 | 17 | 18 | 19 | 22 | 27 | 28),);
    static PGP_PUBLIC_KEY: Subformat = Subformat::new("OpenPGP public key (binary)");
    static PGP_SECRET_KEY: Subformat = Subformat::new("OpenPGP secret key (binary)");
    with_subformat(
        Label::Pgp,
        if matches!(b0, 0x99 | 0xC6) { &PGP_PUBLIC_KEY } else { &PGP_SECRET_KEY },
    )
}

// =============================================================================
// Additional Standalone Format Verifiers
// =============================================================================

const JUMBF_DESC_BOX_MAGIC: &[u8; 4] = b"jumd";
const JUMBF_SUPERBOX_MAGIC: &[u8; 4] = b"jumb";
const MIN_APPLE_KEYCHAIN_SIZE: u64 = 48;
const MIN_APPLE_SECURITYD_BLOB_SIZE: u64 = 48;
const MIN_CDES_ENCRYPTED_ENC_SIZE: u64 = 24;
const MIN_EASTWEST_TCHK_KEY_SIZE: u64 = 64;
const MIN_EXODUS_SECO_SIZE: u64 = 516;
const MIN_GNOME_KEYRING_SIZE: u64 = 48;
const MIN_JUMBF_C2PA_SIZE: u64 = 36;
const MIN_KDE_KWALLET_SIZE: u64 = 48;
const MIN_KERBEROS_KIRBI_SIZE: u64 = 64;
const MIN_MS_CERT_STORE_SST_SIZE: u64 = 64;
const MIN_SECURITY_BOX_ENCRYPTED_SIZE: u64 = 180;
const MIN_SUPF_510_INDEX_SIZE: u64 = 32;
const MIN_SUPP_510_PKG_SIZE: u64 = 46;
const MIN_TELEGRAM_TDF_SIZE: u64 = 28;
const MIN_WHATSAPP_CRYPT14_SIZE: u64 = 144;

/// Apple macOS Keychain Encrypted Credential Store (.keychain).
pub(crate) fn verify_apple_keychain(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_APPLE_KEYCHAIN_SIZE
            && prefix.len() >= 20
            && starts(prefix, b"kych")
            && be16(prefix, 4) == Some(1)
            && be32(prefix, 8).unwrap_or(0) >= 16
            && be32(prefix, 12).unwrap_or(0) >= 16
    );
    Some(Label::Keychain)
}

/// Apple SecurityServer CommonBlob Key/Database Store (`0xFADE0711` BE).
pub(crate) fn verify_apple_securityd_blob(prefix: &[u8], size: u64) -> Option<Label> {
    const SSBLOB_MAGIC: &[u8; 4] = hex2bin!("FA DE 07 11"); // @0..4: CommonBlob::magic (0xFADE0711 BE)
    const VERSION_OFF: usize = 4; // @4..8: u32 BE blobVersion (0x100, 0x101, 0x200)
    const START_CRYPTO_OFF: usize = 8; // @8..12: u32 BE startCryptoBlob (for DbBlob / KeyBlob)
    const TOTAL_LEN_OFF: usize = 12; // @12..16: u32 BE totalLength (for DbBlob / KeyBlob)
    require!(
        size >= MIN_APPLE_SECURITYD_BLOB_SIZE,
        prefix.len() >= 16,
        starts(prefix, SSBLOB_MAGIC),
        matches!(be32(prefix, VERSION_OFF), Some(0x0000_0100 | 0x0000_0101 | 0x0000_0200))
    );
    let valid_layout = if size == 48 {
        true
    } else {
        let start_crypto = be32(prefix, START_CRYPTO_OFF)? as u64;
        let total_len = be32(prefix, TOTAL_LEN_OFF)? as u64;
        start_crypto >= 108 && total_len > start_crypto && total_len <= size
    };
    require!(valid_layout);
    Some(Label::AppleSecuritydBlob)
}

/// `{{CDES}}` Encrypted File Container (`.enc`) header (`"{{CDES}}        "` + encrypted blocks).
pub(crate) fn verify_cdes_encrypted_enc(prefix: &[u8], size: u64) -> Option<Label> {
    const CDES_MAGIC: &[u8; 16] = b"{{CDES}}        "; // @0..16: 8-byte tag + 8 space padding bytes
    const PAYLOAD_OFFSET: usize = 16; // @16..: DES-encrypted payload blocks

    require!(
        size >= MIN_CDES_ENCRYPTED_ENC_SIZE,
        prefix.len() >= 24,
        starts(prefix, CDES_MAGIC),
        has_non_ascii_text_byte(prefix.get(PAYLOAD_OFFSET..prefix.len().min(48))?)
    );
    Some(Label::CdesEncryptedEnc)
}

/// Microsoft CryptoAPI RSA key BLOB (`PUBLICKEY` resources of installers, `.snk` strong-name
/// key files): `BLOBHEADER {u8 bType, u8 bVersion = 2, u16 reserved = 0, u32 aiKeyAlg}`,
/// `RSAPUBKEY {"RSA1"|"RSA2", u32 bitlen, u32 pubexp}`, then the modulus (and private parts).
/// The length closes on the bit length: `20 + bitlen/8` (public) or `20 + 9·bitlen/16` (private).
pub(crate) fn verify_cryptoapi_keyblob(prefix: &[u8], size: u64) -> Option<Label> {
    const B_TYPE: usize = 0; // @0: u8 bType (6 PUBLICKEYBLOB, 7 PRIVATEKEYBLOB)
    const AI_KEY_ALG: usize = 4; // @4: u32 aiKeyAlg
    const RSA_MAGIC: usize = 8; // @8: "RSA1" / "RSA2"
    const RSA_BITLEN: usize = 12; // @12: u32 bitlen
    const RSA_PUBEXP: usize = 16; // @16: u32 public exponent
    const PUBLICKEYBLOB: u8 = 0x06;
    const PRIVATEKEYBLOB: u8 = 0x07;
    const CALG_RSA_SIGN: u32 = 0x0000_2400;
    const CALG_RSA_KEYX: u32 = 0x0000_A400;
    const HEADERS_SIZE: u64 = 20;
    const MIN_KEYBLOB_SIZE: u64 = HEADERS_SIZE + 384 / 8;
    require!(size >= MIN_KEYBLOB_SIZE);
    let kind = u8_at(prefix, B_TYPE)?;
    require!(matches!(le32(prefix, AI_KEY_ALG)?, CALG_RSA_SIGN | CALG_RSA_KEYX));
    let bits = u64::from(le32(prefix, RSA_BITLEN)?);
    require!((384..=16384).contains(&bits), bits % 8 == 0, le32(prefix, RSA_PUBEXP)? % 2 == 1);
    let end = match kind {
        PUBLICKEYBLOB if eq_at(prefix, RSA_MAGIC, b"RSA1") => HEADERS_SIZE + bits / 8,
        PRIVATEKEYBLOB if eq_at(prefix, RSA_MAGIC, b"RSA2") => HEADERS_SIZE + 9 * bits / 16,
        _ => return None,
    };
    require!(size == end);
    Some(Label::CryptoapiKeyblob)
}

/// EastWest Soundsonline TChk License Key File (.key).
pub(crate) fn verify_eastwest_tchk_key(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EASTWEST_TCHK_KEY_SIZE,
        prefix.len() >= 48,
        get_slice(prefix, 3, 8)? == hex2bin!("00 31 50 62 5B"),
        get_slice(prefix, 11, 12)? == hex2bin!("00"),
        get_slice(prefix, 23, 48)?
            == hex2bin!(
                "00 40 00 00 00 01 00 00 00 01 00 00 00 54 43 68 6B 00 00 00 00 31 50 62 5B"
            )
    );
    require!(has_non_ascii_text_byte(prefix.get(..48)?));
    Some(Label::EastwestTchkKey)
}

/// Exodus Movement `SECO` v0 encrypted wallet container (`seed.seco`, `info.seco`, `passphrase.seco`).
///
/// Header layout:
/// - `0..4`: `"SECO"` (`53 45 43 4F`)
/// - `4..8`: `u32be` format version (`0`)
/// - `8..12`: `u32be` reserved (`0`)
/// - `12..224`: 3 `u8`-length-prefixed ASCII strings (`versionTag` starting with `"seco-"`, `appName`, `appVersion`), NUL-padded to `224` (`0xE0`)
/// - `224..256`: 32B `SHA-256(file[256..])`
/// - `256..512`: 256B metadata (32B salt, `u32be` scrypt `N`/`r`/`p` at `288..300`, 32B NUL-padded cipher name at `300..332`, IVs/tags/encrypted blobKey, NUL padding)
/// - `512..516`: `u32be` `blob_len` (`516 + blob_len == size`)
pub(crate) fn verify_exodus_seco(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_EXODUS_SECO_SIZE,
        prefix.len() >= 332,
        starts(prefix, b"SECO"),
        be32(prefix, 4) == Some(0),
        be32(prefix, 8) == Some(0)
    );

    // Parse 3 Pascal strings inside the 224-byte header
    let mut pos = 12usize;
    let vtag_len = u8_at(prefix, pos)? as usize;
    pos += 1;
    require!((5..=64).contains(&vtag_len), pos + vtag_len <= 222);
    let vtag = get_slice(prefix, pos, pos + vtag_len)?;
    require!(vtag.starts_with(b"seco-"), vtag.iter().all(|&b| (0x20..=0x7E).contains(&b)));
    pos += vtag_len;

    let app_len = u8_at(prefix, pos)? as usize;
    pos += 1;
    require!(app_len <= 64, pos + app_len <= 223);
    require!(get_slice(prefix, pos, pos + app_len)?.iter().all(|&b| (0x20..=0x7E).contains(&b)));
    pos += app_len;

    let aver_len = u8_at(prefix, pos)? as usize;
    pos += 1;
    require!(aver_len <= 64, pos + aver_len <= 224);
    require!(get_slice(prefix, pos, pos + aver_len)?.iter().all(|&b| (0x20..=0x7E).contains(&b)));
    let hdr_end = pos + aver_len;
    require!(get_slice(prefix, hdr_end, 224)?.iter().all(|&b| b == 0));

    // Validate scrypt parameters (N power-of-2, r, p) and NUL-padded ASCII cipher name at 288..332
    let scrypt_n = be32(prefix, 288)?;
    let scrypt_r = be32(prefix, 292)?;
    let scrypt_p = be32(prefix, 296)?;
    require!(
        scrypt_n.is_power_of_two(),
        (1024..=16_777_216).contains(&scrypt_n),
        (1..=64).contains(&scrypt_r),
        (1..=64).contains(&scrypt_p)
    );
    let cipher_slot = get_slice(prefix, 300, 332)?;
    let cipher_len = cipher_slot.iter().position(|&b| b == 0).unwrap_or(32);
    require!(
        (3..=31).contains(&cipher_len),
        cipher_slot[..cipher_len]
            .iter()
            .all(|&b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
        cipher_slot[cipher_len..].iter().all(|&b| b == 0)
    );

    if prefix.len() >= 516 {
        let blob_len = be32(prefix, 512)? as u64;
        require!(516u64.checked_add(blob_len) == Some(size));
    }

    Some(Label::ExodusSeco)
}

/// GNOME Keyring Binary Keystore (`.keyring`, `"GnomeKeyring\n\r\0\n"`).
pub(crate) fn verify_gnome_keyring(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_GNOME_KEYRING_SIZE,
        prefix.len() >= 28,
        starts(prefix, b"GnomeKeyring\n\r\0\n"),
        u8_at(prefix, 16)? == 0,
        u8_at(prefix, 17)? == 0,
        u8_at(prefix, 18)? <= 1,
        u8_at(prefix, 19)? <= 1
    );
    let name_len = be32(prefix, 20)?;
    require!(name_len == 0xFFFF_FFFF || name_len <= 256);
    Some(Label::GnomeKeyring)
}

/// ISO/IEC 19566-5 JPEG Universal Metadata Box Format (JUMBF) / C2PA Manifest Store (`.jumbf` / `.c2pa`).
///
/// Header layout:
/// - `@0..4`: `u32 BE superbox_size` (`>= 36`, `superbox_size <= size`)
/// - `@4..8`: `"jumb"` (JUMBF superbox FourCC)
/// - `@8..12`: `u32 BE jumd_size` (`25..=superbox_size`, so `@8 == 0x00`)
/// - `@12..16`: `"jumd"` (JUMBF description box FourCC)
/// - `@16..32`: 16-byte JUMBF content type UUID (e.g. `"c2pa\x00\x11\x00\x10\x80\x00\x00\xAA\x00\x38\x9B\x71"`).
pub(crate) fn verify_jumbf_c2pa(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_JUMBF_C2PA_SIZE,
        prefix.len() >= 36,
        eq_at(prefix, 4, JUMBF_SUPERBOX_MAGIC),
        eq_at(prefix, 12, JUMBF_DESC_BOX_MAGIC)
    );
    let super_sz = be32(prefix, 0)? as u64;
    let jumd_sz = be32(prefix, 8)? as u64;
    require!((36..=size).contains(&super_sz), (25..=super_sz.saturating_sub(8)).contains(&jumd_sz));
    Some(Label::JumbfC2pa)
}

/// KDE KWallet Binary Keystore (`.kwl`, `"KWALLET\n\r\0\r\n"`).
pub(crate) fn verify_kde_kwallet(prefix: &[u8], size: u64) -> Option<Label> {
    require!(
        size >= MIN_KDE_KWALLET_SIZE,
        prefix.len() >= 24,
        starts(prefix, b"KWALLET\n\r\0\r\n"),
        u8_at(prefix, 12)? == 0,
        u8_at(prefix, 13)? <= 1,
        u8_at(prefix, 14)? <= 3,
        u8_at(prefix, 15)? <= 3,
        (1..=64).contains(&be32(prefix, 16)?)
    );
    Some(Label::KdeKwallet)
}

/// Kerberos v5 `KRB-CRED` ASN.1 BER/DER credential ticket (`.kirbi`).
///
/// Supports both standard DER length encoding (`76 81` / `76 82`) and Windows MS-ASN1
/// 4-byte BER length encoding (`76 84 00 00 <hi> <lo>`).
pub(crate) fn verify_kerberos_kirbi(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_KERBEROS_KIRBI_SIZE, prefix.len() >= 48, u8_at(prefix, 0)? == 0x76);
    let (app_len_bytes, app_content_len) = parse_asn1_ber_len(prefix.get(1..)?)?;
    let seq_off = 1 + app_len_bytes;
    require!((seq_off + app_content_len) as u64 == size, u8_at(prefix, seq_off)? == 0x30);
    let (seq_len_bytes, seq_content_len) = parse_asn1_ber_len(prefix.get(seq_off + 1..)?)?;
    let pvno_off = seq_off + 1 + seq_len_bytes;
    require!(
        seq_len_bytes + 1 + seq_content_len == app_content_len,
        u8_at(prefix, pvno_off)? == 0xA0
    );
    let (pvno_len_bytes, pvno_len) = parse_asn1_ber_len(prefix.get(pvno_off + 1..)?)?;
    let pvno_val_off = pvno_off + 1 + pvno_len_bytes;
    require!(pvno_len == 3, eq_at(prefix, pvno_val_off, &[0x02, 0x01, 0x05]));
    let msg_off = pvno_val_off + 3;
    require!(u8_at(prefix, msg_off)? == 0xA1);
    let (msg_len_bytes, msg_len) = parse_asn1_ber_len(prefix.get(msg_off + 1..)?)?;
    let msg_val_off = msg_off + 1 + msg_len_bytes;
    require!(msg_len == 3, eq_at(prefix, msg_val_off, &[0x02, 0x01, 0x16]));
    let tickets_off = msg_val_off + 3;
    require!(u8_at(prefix, tickets_off)? == 0xA2);
    Some(Label::KerberosKirbi)
}

/// Microsoft Serialized Certificate Store (`.sst`, `\0\0\0\0CERT` + `WINE_CERT_PROP_HEADER`).
pub(crate) fn verify_ms_cert_store_sst(prefix: &[u8], size: u64) -> Option<Label> {
    const SST_MAGIC: &[u8; 8] = b"\x00\x00\x00\x00CERT"; // @0..8: u32 LE version (0) + "CERT"
    const PROP_ID_OFF: usize = 8; // @8..12: u32 LE propID (1..=0xFFFF)
    const PROP_UNK_OFF: usize = 12; // @12..16: u32 LE unknown (== 1)
    const PROP_CB_OFF: usize = 16; // @16..20: u32 LE cb (1..=16 MB)
    require!(
        size >= MIN_MS_CERT_STORE_SST_SIZE,
        prefix.len() >= 24,
        starts(prefix, SST_MAGIC),
        (1..=0xFFFF).contains(&le32(prefix, PROP_ID_OFF)?),
        le32(prefix, PROP_UNK_OFF) == Some(1),
        (1..=16 * 1024 * 1024).contains(&le32(prefix, PROP_CB_OFF)?)
    );
    Some(Label::MsCertStoreSst)
}

/// Stormshield / Arkoon Security BOX Encrypted Container (`.box`, `application/x-sbox-file`).
pub(crate) fn verify_security_box_encrypted(prefix: &[u8], size: u64) -> Option<Label> {
    const SBLIGHT_MAGIC: &[u8; 49] = b"SBLIGHT20\r\nTrademark: Encrypted with Security BOX"; // @0..49
    const SBOX_MIME: &[u8; 37] = b"Content-Type: application/x-sbox-file";

    let window = get_slice(prefix, 0, prefix.len().min(256))?;
    require!(
        size >= MIN_SECURITY_BOX_ENCRYPTED_SIZE,
        window.len() >= 140,
        starts(window, SBLIGHT_MAGIC),
        contains(window, SBOX_MIME)
    );
    Some(Label::SblightBox)
}

/// Apple FairPlay DRM f-Class Key Container (`.supf`).
pub(crate) fn verify_supf_510_index(prefix: &[u8], size: u64) -> Option<Label> {
    const FAIRPLAY_SUPF_REC_HDR: &[u8; 8] = hex2bin!("01 00 00 0C 00 00 00 00"); // @16..24

    require!(
        size >= MIN_SUPF_510_INDEX_SIZE,
        prefix.len() >= 24,
        (1..=10).contains(&u8_at(prefix, 0)?), // @0: FairPlay format version (3, 5, ...)
        get_slice(prefix, 1, 4)?.iter().all(u8::is_ascii_digit) // @1..4: ASCII build tag ("510", "612", ...)
    );
    let hdr_len = be32(prefix, 4)?;
    require!(
        hdr_len >= 24,
        be32(prefix, 8) == Some(1),
        be32(prefix, 12) == Some(hdr_len - 8),
        eq_at(prefix, 16, FAIRPLAY_SUPF_REC_HDR)
    );
    Some(Label::AppleFairplaySupf)
}

/// Apple FairPlay DRM p-Class Key Container (`.supp` v1 / `.v4.supp` v4).
pub(crate) fn verify_supp_510_pkg(prefix: &[u8], size: u64) -> Option<Label> {
    const DER_SEQUENCE_LONG: &[u8; 2] = hex2bin!("30 82"); // DER X.509 certificate SEQUENCE tag

    require!(size >= MIN_SUPP_510_PKG_SIZE, prefix.len() >= 46);
    let hdr_len: usize = match u8_at(prefix, 0)? {
        1 => 28, // v1: 1B ver + 3B ASCII build + 20B SHA-1 + 4B BE key count
        4 => 8,  // v4: 1B ver + 3B ASCII build + 4B BE key count
        _ => return None,
    };
    require!(get_slice(prefix, 1, 4)?.iter().all(u8::is_ascii_digit));
    let count = be32(prefix, hdr_len - 4)? as usize;
    require!((1..=4096).contains(&count));
    let cert_off = hdr_len.checked_add(count.checked_mul(32)?)?;
    require!(
        cert_off + 6 <= prefix.len(),
        (64..=16384).contains(&be32(prefix, cert_off)?),
        eq_at(prefix, cert_off + 4, DER_SEQUENCE_LONG)
    );
    Some(Label::AppleFairplaySupp)
}

/// Telegram Desktop `TDF$` encrypted data file (`key_datas`, `map0`, `settings0`, session files).
///
/// Header layout:
/// - `0..4`: `"TDF$"` (`54 44 46 24`)
/// - `4..8`: `u32le` `AppVersion` (`1..=99_999_999`, e.g., `6020` = v0.6.20, `7004002` = v7.4.2)
/// - `8..(size-16)`: Qt `QDataStream` sequence of length-prefixed `QByteArray`s (`u32be` length + bytes, or `0xFFFF_FFFF` null)
/// - `(size-16)..size`: 16-byte `MD5(payload || u32le(payload.len()) || u32le(version) || "TDF$")`
pub(crate) fn verify_telegram_tdf(prefix: &[u8], size: u64) -> Option<Label> {
    require!(size >= MIN_TELEGRAM_TDF_SIZE, prefix.len() >= 12, starts(prefix, b"TDF$"));
    let version = le32(prefix, 4)?;
    // AppVersion encodes major*1_000_000 + minor*1_000 + patch.
    require!((1..=99_999_999).contains(&version));

    const MAX_CHUNKS: usize = 16;
    let payload_end = size.checked_sub(16)?;
    let mut pos: u64 = 8;
    let mut chunks = 0usize;
    while pos + 4 <= payload_end && chunks < MAX_CHUNKS {
        let pos_usize = usize::try_from(pos).ok()?;
        if pos_usize + 4 > prefix.len() {
            break;
        }
        let qba_len = be32(prefix, pos_usize)?;
        chunks += 1;
        if qba_len == 0xFFFF_FFFF {
            pos = pos.checked_add(4)?;
        } else {
            let next = pos.checked_add(4)?.checked_add(qba_len as u64)?;
            require!(next <= payload_end);
            pos = next;
        }
    }
    require!(chunks >= 1);
    // When the entire payload is inside the prefix buffer and we didn't hit the chunk cap,
    // require exact landing on the 16-byte MD5 trailer boundary.
    if payload_end <= prefix.len() as u64 && chunks < MAX_CHUNKS {
        require!(pos == payload_end);
    }

    Some(Label::TelegramTdf)
}

/// WhatsApp Encrypted Msgstore Database Backup (`.crypt14`): cleartext 2-byte protobuf
/// length varint (`@0 >= 0x80`, `@1 == 0x01`) and `KeyData` protobuf framing.
pub(crate) fn verify_whatsapp_crypt14(prefix: &[u8], size: u64) -> Option<Label> {
    // @2..15: field 1 key_version=0 (`08 00`), field 2 KeyData len 77 (`12 4D`),
    // cipher_version `\x00\x01` (`0A 02 00 01`), key_version_str "2" (`12 01 32`), server_salt len 32 (`1A 20`).
    const KEYDATA_PREFIX_13: &[u8; 13] = hex2bin!("08 00 12 4D 0A 02 00 01 12 01 32 1A 20");
    const ACCOUNT_HASH_TAG: &[u8; 2] = hex2bin!("22 10"); // @47..49: KeyData.account_hash (16B)
    const IV_TAG: &[u8; 2] = hex2bin!("2A 10"); // @65..67: KeyData.iv (16B)
    const BACKUP_METADATA_TAG: u8 = 0x22; // @83: BackupHeader.backup_metadata tag (field 4)

    require!(
        size >= MIN_WHATSAPP_CRYPT14_SIZE,
        prefix.len() >= 85,
        u8_at(prefix, 0)? >= 0x80,
        u8_at(prefix, 1)? == 0x01
    );
    let hdr_len = ((u8_at(prefix, 0)? & 0x7F) as usize) | ((u8_at(prefix, 1)? as usize) << 7);
    require!(
        size >= (2 + hdr_len) as u64,
        eq_at(prefix, 2, KEYDATA_PREFIX_13),
        eq_at(prefix, 47, ACCOUNT_HASH_TAG),
        eq_at(prefix, 65, IV_TAG),
        u8_at(prefix, 83)? == BACKUP_METADATA_TAG
    );
    Some(Label::WhatsappCrypt14)
}
