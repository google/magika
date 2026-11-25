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

//! SEG-Y (`segy`) verifier: accepted header variants and false-positive regressions.

use super::{resolve_slice, resolve_slice_with_metadata};
use crate::ContentType;

#[test]
fn segy_variants_and_false_positive_regressions() {
    // EBCDIC "C 1 CLIENT SAMPLE SEGY1234" (>= 16 alphanumeric EBCDIC bytes)
    let ebcdic_card: &[u8] = &[
        0xC3, 0x40, 0xF1, 0x40, 0xC3, 0xD3, 0xC9, 0xC5, 0xD5, 0xE3, 0x40, 0xE2, 0xC1, 0xD4, 0xD7,
        0xD3, 0xC5, 0x40, 0xE2, 0xC5, 0xC7, 0xE8, 0xF1, 0xF2, 0xF3, 0xF4,
    ];

    // 1. Standard big-endian EBCDIC textual header
    let mut ebcdic = vec![0u8; 3632];
    ebcdic[..3200].fill(0x40);
    ebcdic[..ebcdic_card.len()].copy_from_slice(ebcdic_card);
    ebcdic[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
    ebcdic[3220..3222].copy_from_slice(&1000u16.to_be_bytes());
    ebcdic[3224..3226].copy_from_slice(&5u16.to_be_bytes());
    let (cts, meta) = resolve_slice_with_metadata(&ebcdic);
    assert_eq!(cts, vec![ContentType::Segy]);
    assert_eq!(meta.get("subformat").map(String::as_str), Some("SEG-Y (EBCDIC textual header)"));
    assert_eq!(meta.get("pronom_puid").map(String::as_str), Some("fmt/363"));

    // 2. Non-'C' EBCDIC textual header (leading EBCDIC space)
    let mut ebcdic_noc = ebcdic.clone();
    ebcdic_noc[0] = 0x40;
    assert_eq!(resolve_slice(&ebcdic_noc), vec![ContentType::Segy]);

    // 2b. EBCDIC textual header with decorative border lines ('----' = 0x60, '====' = 0x7E, '****' = 0x5C)
    let mut ebcdic_borders = ebcdic.clone();
    ebcdic_borders[80..160].fill(0x60);
    ebcdic_borders[160..240].fill(0x7E);
    ebcdic_borders[240..320].fill(0x5C);
    assert_eq!(resolve_slice(&ebcdic_borders), vec![ContentType::Segy]);

    // 3. Little-endian EBCDIC without Rev 2 endianness constant, across all Rev 0/1/2 format codes
    for fmt in [1u16, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 15, 16] {
        let mut le = ebcdic.clone();
        le[3216..3218].copy_from_slice(&4000u16.to_le_bytes());
        le[3220..3222].copy_from_slice(&250u16.to_le_bytes());
        le[3224..3226].copy_from_slice(&fmt.to_le_bytes());
        assert_eq!(resolve_slice(&le), vec![ContentType::Segy], "LE fmt {fmt}");
    }

    // 4. Rev 2 extended sample interval (f64 @3272) and extended sample count (i32 @3268)
    let mut rev2_ext = ebcdic.clone();
    rev2_ext[3216..3218].fill(0);
    rev2_ext[3220..3222].fill(0);
    rev2_ext[3224..3226].copy_from_slice(&10u16.to_be_bytes());
    rev2_ext[3268..3272].copy_from_slice(&100_000i32.to_be_bytes());
    rev2_ext[3272..3280].copy_from_slice(&2000.0f64.to_be_bytes());
    rev2_ext[3296..3300].copy_from_slice(&[0x01, 0x02, 0x03, 0x04]);
    assert_eq!(resolve_slice(&rev2_ext), vec![ContentType::Segy]);

    // Negative extended sample count is rejected
    let mut rev2_neg_ns = rev2_ext.clone();
    rev2_neg_ns[3268..3272].copy_from_slice(&(-1i32).to_be_bytes());
    assert_eq!(resolve_slice(&rev2_neg_ns), Vec::<ContentType>::new());

    // Rev 2 extended fields override non-zero 16-bit fields when present
    let mut rev2_override = vec![0u8; 3600 + 240 + 10 * 4]; // 1 trace of 10 x 4B samples
    rev2_override[..24].copy_from_slice(b"C 1 SEGY REV 2 OVERRIDES");
    rev2_override[3216..3218].copy_from_slice(&1000u16.to_be_bytes());
    rev2_override[3220..3222].copy_from_slice(&u16::MAX.to_be_bytes()); // clamped 16-bit ns
    rev2_override[3224..3226].copy_from_slice(&5u16.to_be_bytes());
    rev2_override[3268..3272].copy_from_slice(&10i32.to_be_bytes()); // overrides ns16
    rev2_override[3272..3280].copy_from_slice(&2000.0f64.to_be_bytes()); // overrides dt16
    assert_eq!(resolve_slice(&rev2_override), vec![ContentType::Segy]);

    // Negative extended sample count or invalid extended interval is rejected even if 16-bit fields are non-zero
    let mut rev2_bad_ext_ns = ebcdic.clone();
    rev2_bad_ext_ns[3268..3272].copy_from_slice(&(-5i32).to_be_bytes());
    assert_eq!(resolve_slice(&rev2_bad_ext_ns), Vec::<ContentType>::new());
    let mut rev2_nan_dt = ebcdic.clone();
    rev2_nan_dt[3272..3280].copy_from_slice(&f64::NAN.to_be_bytes());
    assert_eq!(resolve_slice(&rev2_nan_dt), Vec::<ContentType>::new());

    // Rev 2 ext_headers == -1 with explicit first_trace_offset (@3520..3528)
    let mut rev2_fto = vec![0u8; 3600 + 3200 + 240 + 10 * 4];
    rev2_fto[..24].copy_from_slice(b"C 1 SEGY REV 2 FIRST TRC");
    rev2_fto[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
    rev2_fto[3220..3222].copy_from_slice(&10u16.to_be_bytes());
    rev2_fto[3224..3226].copy_from_slice(&5u16.to_be_bytes());
    rev2_fto[3504..3506].copy_from_slice(&(-1i16).to_be_bytes());
    rev2_fto[3520..3528].copy_from_slice(&(6800u64).to_be_bytes());
    assert_eq!(resolve_slice(&rev2_fto), vec![ContentType::Segy]);

    // Rev 2 format code 9 (8-byte signed int) exact trace geometry corroborates ASCII header
    let mut rev2_fmt9 = vec![0u8; 3600 + 240 + 10 * 8]; // 1 trace of 10 x 8B samples
    rev2_fmt9[..24].copy_from_slice(b"C 1 SEGY REV 2 INT64 FMT");
    rev2_fmt9[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
    rev2_fmt9[3220..3222].copy_from_slice(&10u16.to_be_bytes());
    rev2_fmt9[3224..3226].copy_from_slice(&9u16.to_be_bytes());
    assert_eq!(resolve_slice(&rev2_fmt9), vec![ContentType::Segy]);

    // 5. NUL-padded ASCII textual header corroborated by trace-0 header match
    let mut ascii_nul = vec![0u8; 3840];
    ascii_nul[..24].copy_from_slice(b"C 1 ASCII SEISMIC HEADER");
    ascii_nul[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
    ascii_nul[3220..3222].copy_from_slice(&100u16.to_be_bytes());
    ascii_nul[3224..3226].copy_from_slice(&1u16.to_be_bytes());
    ascii_nul[3714..3716].copy_from_slice(&100u16.to_be_bytes());
    ascii_nul[3716..3718].copy_from_slice(&2000u16.to_be_bytes());
    let (cts, meta) = resolve_slice_with_metadata(&ascii_nul);
    assert_eq!(cts, vec![ContentType::Segy]);
    assert_eq!(meta.get("subformat").map(String::as_str), Some("SEG-Y (ASCII textual header)"));

    // 6. Little-endian ASCII textual header corroborated by Rev 2 byte-order constant
    let mut ascii_le = vec![0u8; 3632];
    ascii_le[..3200].fill(b' ');
    ascii_le[..24].copy_from_slice(b"C 1 SEGY REV 2 LE HEADER");
    ascii_le[3216..3218].copy_from_slice(&4000u16.to_le_bytes());
    ascii_le[3220..3222].copy_from_slice(&500u16.to_le_bytes());
    ascii_le[3224..3226].copy_from_slice(&16u16.to_le_bytes());
    ascii_le[3296..3300].copy_from_slice(&[0x04, 0x03, 0x02, 0x01]);
    assert_eq!(resolve_slice(&ascii_le), vec![ContentType::Segy]);

    // 7. All-zero textual header corroborated by exact geometry + trace-0 match
    let mut empty_text = vec![0u8; 3600 + 240 + 40]; // 1 trace of 10 x 4B samples
    empty_text[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
    empty_text[3220..3222].copy_from_slice(&10u16.to_be_bytes());
    empty_text[3224..3226].copy_from_slice(&5u16.to_be_bytes());
    empty_text[3714..3716].copy_from_slice(&10u16.to_be_bytes());
    empty_text[3716..3718].copy_from_slice(&2000u16.to_be_bytes());
    let (cts, meta) = resolve_slice_with_metadata(&empty_text);
    assert_eq!(cts, vec![ContentType::Segy]);
    assert_eq!(meta.get("subformat").map(String::as_str), Some("SEG-Y (empty textual header)"));

    // Negative regressions:
    // a) Uncorroborated short prefix with sparse binary header
    let mut bogus_pnt = vec![0u8; 3632];
    bogus_pnt[..3].copy_from_slice(b"PNT");
    bogus_pnt[3216..3218].copy_from_slice(&0xC600u16.to_be_bytes());
    bogus_pnt[3220..3222].copy_from_slice(&0xEDFFu16.to_be_bytes());
    bogus_pnt[3224..3226].copy_from_slice(&1u16.to_be_bytes());
    assert_eq!(resolve_slice(&bogus_pnt), Vec::<ContentType>::new());

    // b) Non-zero unassigned binary header ranges (@3300..3500 and @3532..3600)
    for off in [3300usize, 3499, 3532, 3599] {
        let mut dirty = ebcdic.clone();
        dirty[off] = 0x01;
        assert_eq!(resolve_slice(&dirty), Vec::<ContentType>::new(), "dirty @{off}");
    }

    // c) Invalid Rev 2 byte-order constant
    let mut bad_em = ebcdic.clone();
    bad_em[3296..3300].copy_from_slice(&[0x01, 0x02, 0x03, 0x05]);
    assert_eq!(resolve_slice(&bad_em), Vec::<ContentType>::new());

    // d) Invalid format codes (0, 13, 14, 17)
    for fmt in [0u16, 13, 14, 17] {
        let mut bad_fmt = ebcdic.clone();
        bad_fmt[3224..3226].copy_from_slice(&fmt.to_be_bytes());
        assert_eq!(resolve_slice(&bad_fmt), Vec::<ContentType>::new(), "invalid fmt {fmt}");
    }

    // e) Out-of-range binary header control fields
    let mut bad_sort = ebcdic.clone();
    bad_sort[3228..3230].copy_from_slice(&10i16.to_be_bytes());
    assert_eq!(resolve_slice(&bad_sort), Vec::<ContentType>::new());

    let mut bad_meas = ebcdic.clone();
    bad_meas[3254..3256].copy_from_slice(&3u16.to_be_bytes());
    assert_eq!(resolve_slice(&bad_meas), Vec::<ContentType>::new());

    let mut bad_fixed = ebcdic.clone();
    bad_fixed[3502..3504].copy_from_slice(&2u16.to_be_bytes());
    assert_eq!(resolve_slice(&bad_fixed), Vec::<ContentType>::new());

    let mut bad_ext = ebcdic.clone();
    bad_ext[3504..3506].copy_from_slice(&5i16.to_be_bytes()); // exceeds 3632 B file size
    assert_eq!(resolve_slice(&bad_ext), Vec::<ContentType>::new());

    // f) Uncorroborated ASCII and uncorroborated all-zero textual headers
    let mut uncorr_ascii = ascii_le.clone();
    uncorr_ascii[3296..3300].fill(0);
    assert_eq!(resolve_slice(&uncorr_ascii), Vec::<ContentType>::new());

    let mut uncorr_zero = vec![0u8; 3632];
    uncorr_zero[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
    uncorr_zero[3220..3222].copy_from_slice(&10u16.to_be_bytes());
    uncorr_zero[3224..3226].copy_from_slice(&5u16.to_be_bytes());
    assert_eq!(resolve_slice(&uncorr_zero), Vec::<ContentType>::new());

    // g) Binary noise in textual header (e.g., bytecode/compiled binary), punctuation-only
    // header (e.g., EBCDIC space 0x40 / ASCII '@'), & pure printable ASCII
    let mut noisy = ebcdic.clone();
    noisy[..64].fill(0x01);
    assert_eq!(resolve_slice(&noisy), Vec::<ContentType>::new());
    let mut at_only = vec![0u8; 3600 + 240 + 40];
    at_only[..32].fill(0x40);
    at_only[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
    at_only[3220..3222].copy_from_slice(&10u16.to_be_bytes());
    at_only[3224..3226].copy_from_slice(&5u16.to_be_bytes());
    assert_eq!(resolve_slice(&at_only), Vec::<ContentType>::new());
    assert_eq!(resolve_slice(&vec![b'A'; 4096]), Vec::<ContentType>::new());

    // h) Uncorroborated NUL-padded high bytes without EBCDIC spaces, and UTF-16LE / 16-bit arrays
    let mut nul_high = vec![0u8; 3632];
    nul_high[..32].fill(0xC1); // EBCDIC 'A' without any 0x40 spaces or corroboration
    nul_high[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
    nul_high[3220..3222].copy_from_slice(&1000u16.to_be_bytes());
    nul_high[3224..3226].copy_from_slice(&5u16.to_be_bytes());
    assert_eq!(resolve_slice(&nul_high), Vec::<ContentType>::new());

    let mut utf16le = vec![0u8; 3600 + 240 + 10 * 4];
    for i in (0..400).step_by(2) {
        utf16le[i] = b'A';
    }
    utf16le[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
    utf16le[3220..3222].copy_from_slice(&10u16.to_be_bytes());
    utf16le[3224..3226].copy_from_slice(&5u16.to_be_bytes());
    assert_eq!(resolve_slice(&utf16le), Vec::<ContentType>::new());

    let mut u16_ebcdic = vec![0u8; 3632];
    for i in (0..400).step_by(4) {
        u16_ebcdic[i] = 0xC1;
        u16_ebcdic[i + 2] = 0x40;
    }
    u16_ebcdic[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
    u16_ebcdic[3220..3222].copy_from_slice(&1000u16.to_be_bytes());
    u16_ebcdic[3224..3226].copy_from_slice(&5u16.to_be_bytes());
    assert_eq!(resolve_slice(&u16_ebcdic), Vec::<ContentType>::new());

    // i) Exact NUL-transition boundary across all-NUL, all-non-NUL, and mixed 64-byte chunks
    let mut nul_boundary = vec![0u8; 3632];
    for chunk_idx in 0..40 {
        let fill = if chunk_idx % 2 == 0 { 0x00 } else { 0x40 };
        nul_boundary[chunk_idx * 64..(chunk_idx + 1) * 64].fill(fill);
    }
    nul_boundary[64..80].fill(0xC1); // 16 EBCDIC alnum in chunk 1 (20 chunk-boundary transitions)
    nul_boundary[2560..3200].fill(0x40);
    for k in 0..44 {
        nul_boundary[2560 + 2 * k] = 0x00;
        nul_boundary[2560 + 2 * k + 1] = 0xC1; // +44 transitions => 64 total (exact max)
    }
    nul_boundary[3216..3218].copy_from_slice(&2000u16.to_be_bytes());
    nul_boundary[3220..3222].copy_from_slice(&1000u16.to_be_bytes());
    nul_boundary[3224..3226].copy_from_slice(&5u16.to_be_bytes());
    assert_eq!(resolve_slice(&nul_boundary), vec![ContentType::Segy]);
    nul_boundary[2560 + 88] = 0x00;
    nul_boundary[2560 + 89] = 0xC1; // 65 total transitions => rejected
    assert_eq!(resolve_slice(&nul_boundary), Vec::<ContentType>::new());
}

#[test]
fn segy_extended_interval_rejects_subnormals_and_out_of_range_values() {
    // A sub-microsecond interval with the 16-bit field unset is what the Rev 2 field is for.
    let mut sub_micro = ebcdic_file(3632, 1000);
    set_be16(&mut sub_micro, INTERVAL, 0);
    sub_micro[EXT_INTERVAL..EXT_INTERVAL + 8].copy_from_slice(&0.05f64.to_be_bytes());
    assert_segy(&sub_micro, "0.05 us extended interval");

    // Small integers decode as subnormals; they must not stand in for a missing 16-bit
    // interval, nor pass next to a valid one.
    let out_of_range = [
        1u64,
        0x0000_0001_0000_0000,
        0x000F_FFFF_FFFF_FFFF,
        f64::MIN_POSITIVE.to_bits(),
        1e-10f64.to_bits(),
        1e9f64.to_bits(),
    ];
    for bits in out_of_range {
        for interval in [0u16, 2000] {
            let mut file = ebcdic_file(3632, 1000);
            set_be16(&mut file, INTERVAL, interval);
            file[EXT_INTERVAL..EXT_INTERVAL + 8].copy_from_slice(&bits.to_be_bytes());
            assert_rejected(&file, &format!("extended interval {bits:#018x}, dt16 {interval}"));
        }
    }
}

#[test]
fn segy_first_trace_offset_is_range_checked_and_overrides_the_header_count() {
    // Unknown extended header count (-1) and unknown offset (0): the EBCDIC text suffices.
    let mut unknown = ebcdic_file(3632, 1000);
    set_be16(&mut unknown, EXT_HEADERS, -1i16 as u16);
    assert_segy(&unknown, "count -1, offset 0");

    // A non-zero offset must lie in `3600..=size`, whatever the count says.
    let offsets = [
        (100u64, false),
        (3599, false),
        (3600, true),
        (3632, true),
        (3633, false),
        (u64::MAX, false),
    ];
    for count in [-1i16, 0] {
        for (offset, valid) in offsets {
            let mut file = ebcdic_file(3632, 1000);
            set_be16(&mut file, EXT_HEADERS, count as u16);
            file[FIRST_TRACE..FIRST_TRACE + 8].copy_from_slice(&offset.to_be_bytes());
            let case = format!("count {count}, offset {offset}");
            if valid {
                assert_segy(&file, &case);
            } else {
                assert_rejected(&file, &case);
            }
        }
    }

    // With count 0 the traces would start at @3600, where one 280-byte trace does not divide
    // the remaining 3,480 bytes: the ASCII header stays uncorroborated. The explicit offset
    // moves the start to @6800 and the geometry fits.
    let mut overridden = ascii_file(3600 + 3200 + 240 + 10 * 4, 10);
    assert_rejected(&overridden, "count 0, geometry from @3600");
    overridden[FIRST_TRACE..FIRST_TRACE + 8].copy_from_slice(&6800u64.to_be_bytes());
    assert_segy(&overridden, "count 0, geometry from offset 6800");
}

#[test]
fn segy_extended_textual_headers_shift_the_trace_geometry() {
    // One 3,200-byte extended header, then one trace of 10 x 4-byte samples.
    let mut file = ascii_file(3600 + 3200 + 240 + 10 * 4, 10);
    set_be16(&mut file, EXT_HEADERS, 1);
    assert_segy(&file, "1 extended header");

    // Two extended headers do not fit in 7,080 bytes, with or without an explicit offset.
    set_be16(&mut file, EXT_HEADERS, 2);
    assert_rejected(&file, "2 extended headers past EOF");
    file[FIRST_TRACE..FIRST_TRACE + 8].copy_from_slice(&6800u64.to_be_bytes());
    assert_rejected(&file, "2 extended headers past EOF, offset 6800");
}

#[test]
fn segy_additional_trace_headers_count_toward_the_trace_size() {
    // 520 bytes after the file header: two 240-byte trace headers and 10 x 4-byte samples.
    let mut file = ascii_file(3600 + 2 * 240 + 10 * 4, 10);
    assert_rejected(&file, "280-byte traces do not divide 520 bytes");
    set_be16(&mut file, MAX_ADD_TRACE_HEADERS, 1);
    assert_segy(&file, "1 additional trace header");
}

#[test]
fn segy_binary_header_is_read_in_one_byte_order() {
    let big_endian = ebcdic_file(3632, 1000);
    let mut little_endian = ebcdic_file(3632, 1000);
    for (offset, value) in [(INTERVAL, 2000u16), (SAMPLES, 1000), (FORMAT, 5)] {
        little_endian[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    // The Rev 2 byte-order constant selects the only order tried.
    for (header, name) in [(&big_endian, "BE"), (&little_endian, "LE")] {
        for (constant, constant_name) in [(ENDIAN_BE, "BE"), (ENDIAN_LE, "LE")] {
            let mut file = header.clone();
            file[ENDIAN..ENDIAN + 4].copy_from_slice(&constant);
            let case = format!("{name} header, {constant_name} constant");
            if name == constant_name {
                assert_segy(&file, &case);
            } else {
                assert_rejected(&file, &case);
            }
        }
    }

    // Without the constant, no field may be read in the other order: a little-endian
    // measurement system of 1 reads as 256 in a big-endian header.
    let mut mixed = big_endian.clone();
    set_be16(&mut mixed, MEASUREMENT_SYSTEM, 1);
    assert_segy(&mixed, "BE measurement system");
    mixed[MEASUREMENT_SYSTEM..MEASUREMENT_SYSTEM + 2].copy_from_slice(&1u16.to_le_bytes());
    assert_rejected(&mixed, "LE measurement system in a BE header");
}

#[test]
fn segy_empty_textual_header_needs_the_endian_constant_or_both_trace_checks() {
    // The byte-order constant alone: 32 trailing bytes hold no trace.
    let mut constant_only = zero_text_file(3632, 10);
    constant_only[ENDIAN..ENDIAN + 4].copy_from_slice(&ENDIAN_BE);
    let (content_types, metadata) = resolve_slice_with_metadata(&constant_only);
    assert_eq!(content_types, vec![ContentType::Segy]);
    assert_eq!(metadata.get("subformat").map(String::as_str), Some("SEG-Y (empty textual header)"));

    // Exact geometry (one 280-byte trace) without a matching first trace header.
    let geometry_only = zero_text_file(3600 + 240 + 10 * 4, 10);
    assert_rejected(&geometry_only, "geometry only");

    // A matching first trace header without exact geometry (one stray byte).
    let mut trace0_only = zero_text_file(3600 + 240 + 10 * 4 + 1, 10);
    set_be16(&mut trace0_only, TRACE0_SAMPLES, 10);
    assert_rejected(&trace0_only, "first trace header only");

    let mut both = geometry_only;
    set_be16(&mut both, TRACE0_SAMPLES, 10);
    assert_segy(&both, "geometry and first trace header");
}

const INTERVAL: usize = 3216;
const SAMPLES: usize = 3220;
const FORMAT: usize = 3224;
const MEASUREMENT_SYSTEM: usize = 3254;
const EXT_INTERVAL: usize = 3272;
const ENDIAN: usize = 3296;
const EXT_HEADERS: usize = 3504;
const MAX_ADD_TRACE_HEADERS: usize = 3506;
const FIRST_TRACE: usize = 3520;
const TRACE0_SAMPLES: usize = 3714;
const ENDIAN_BE: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
const ENDIAN_LE: [u8; 4] = [0x04, 0x03, 0x02, 0x01];

/// A `len`-byte file with an EBCDIC textual header, which needs no corroboration.
fn ebcdic_file(len: usize, samples: u16) -> Vec<u8> {
    let mut file = zero_text_file(len, samples);
    file[..3200].fill(0x40);
    file[..32].fill(0xC1);
    file
}

/// A `len`-byte file with a NUL-padded ASCII textual header, which is accepted only when the
/// byte-order constant, the trace geometry, or the first trace header corroborates it.
fn ascii_file(len: usize, samples: u16) -> Vec<u8> {
    let mut file = zero_text_file(len, samples);
    file[..24].copy_from_slice(b"C 1 ASCII SEISMIC HEADER");
    file
}

/// A `len`-byte file with an all-zero textual header and a big-endian binary header: 2 ms
/// sample interval, `samples` 4-byte IEEE samples per trace.
fn zero_text_file(len: usize, samples: u16) -> Vec<u8> {
    let mut file = vec![0u8; len];
    set_be16(&mut file, INTERVAL, 2000);
    set_be16(&mut file, SAMPLES, samples);
    set_be16(&mut file, FORMAT, 5);
    file
}

fn set_be16(file: &mut [u8], offset: usize, value: u16) {
    file[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

#[track_caller]
fn assert_segy(file: &[u8], case: &str) {
    assert_eq!(resolve_slice(file), vec![ContentType::Segy], "{case}");
}

#[track_caller]
fn assert_rejected(file: &[u8], case: &str) {
    assert_eq!(resolve_slice(file), Vec::<ContentType>::new(), "{case}");
}
