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

//! ISO 9660 / ECMA-167 UDF optical disc metaformat verifiers (`iso`, `udf`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// ISO 9660 / High Sierra / ECMA-167 UDF Optical Disc Image (`iso`, 1988):
/// volume descriptor at sector 16 (`0x8000`) with `"CD001"`, `"BEA01"`, `"NSR02"`,
/// `"NSR03"`, or `"BOOT2"` at `0x8001` (or `"CDROM"` at `0x8009`).
pub(crate) fn iso(prefix: &[u8], size: u64) -> Option<Label> {
    const PVD_OFF: usize = 0x8000;
    const UDF_BEA01_ID: &[u8; 5] = b"BEA01";
    const UDF_NSR02_ID: &[u8; 5] = b"NSR02";
    const UDF_NSR03_ID: &[u8; 5] = b"NSR03";
    const ISO_CD001_ID: &[u8; 5] = b"CD001";
    const HIGH_SIERRA_CDROM_ID: &[u8; 5] = b"CDROM";

    require!(size >= (PVD_OFF + 6) as u64, prefix.len() >= PVD_OFF + 6);
    for off in [0x8000usize, 0x8800, 0x9000, 0x9800] {
        if prefix.len() >= off + 6
            && (eq_at(prefix, off + 1, UDF_BEA01_ID)
                || eq_at(prefix, off + 1, UDF_NSR02_ID)
                || eq_at(prefix, off + 1, UDF_NSR03_ID))
            && matches!(u8_at(prefix, off)?, 0 | 1 | 2 | 3 | 0xFF)
        {
            return Some(Label::Udf);
        }
    }
    let valid = (matches!(u8_at(prefix, PVD_OFF)?, 0 | 1 | 2 | 3 | 0xFF)
        && eq_at(prefix, PVD_OFF + 1, ISO_CD001_ID))
        || (prefix.len() >= PVD_OFF + 14
            && matches!(u8_at(prefix, PVD_OFF + 8)?, 0 | 1 | 2 | 3 | 0xFF)
            && eq_at(prefix, PVD_OFF + 9, HIGH_SIERRA_CDROM_ID));
    require!(valid);
    Some(Label::Iso)
}
