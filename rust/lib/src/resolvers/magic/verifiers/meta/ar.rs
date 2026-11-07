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

//! Unix `ar` archive metaformat verifiers (`ar`, `deb`, `rlib`, `vib`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

pub(crate) const AR_MAGIC: &[u8; 8] = b"!<arch>\n";

/// Debian binary package (`deb`, 1995): Unix `!<arch>\n` archive opening with the
/// `"debian-binary"` member and format version `"2.0\n"` at `@68..72`.
pub(crate) fn deb(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_DEB_SIZE: u64 = 72; // 8B magic + 60B ar header + 4B "2.0\n"
    const DEB_MEMBER_STD: &[u8; 16] = b"debian-binary   ";
    const DEB_MEMBER_SLASH: &[u8; 16] = b"debian-binary/  ";
    const DEB_VER_2_0: &[u8; 6] = b"`\n2.0\n";

    require!(
        size >= MIN_DEB_SIZE,
        prefix.len() >= 72,
        starts(prefix, AR_MAGIC),
        eq_at(prefix, 8, DEB_MEMBER_STD) || eq_at(prefix, 8, DEB_MEMBER_SLASH),
        eq_at(prefix, 66, DEB_VER_2_0),
    );
    Some(Label::Deb)
}

/// VMware vSphere Installation Bundle (VIB): Unix `ar` archive (`"!<arch>\n"`) whose first member
/// is `"descriptor.xml"` at `@8` holding `"<vib version"` at `@68`.
pub(crate) fn vib(prefix: &[u8]) -> Option<Label> {
    const MEMBER_NAME_OFFSET: usize = 8; // @8..22: first member name "descriptor.xml"
    const MEMBER_NAME: &[u8] = b"descriptor.xml";
    const PAYLOAD_OFFSET: usize = 68; // @68..80: 8-byte ar magic + 60-byte member header
    const VIB_TAG: &[u8] = b"<vib version";
    const MIN_LEN: usize = 80;

    require!(
        prefix.len() >= MIN_LEN,
        starts(prefix, AR_MAGIC),
        eq_at(prefix, MEMBER_NAME_OFFSET, MEMBER_NAME),
        eq_at(prefix, PAYLOAD_OFFSET, VIB_TAG),
    );
    Some(Label::Vib)
}

/// Disambiguates Rust `.rlib` (`rust.metadata.bin` / `.rmeta` member) vs generic Unix `ar`
/// (`!<arch>\n` with `` `\n `` member header terminator at `@66..68`) after `vib` and `deb`.
pub(crate) fn rlib_or_ar(prefix: &[u8], size: u64) -> Option<Label> {
    const AR_FMAG: &[u8; 2] = b"`\n";
    const RLIB_METADATA_BIN: &[u8] = b"rust.metadata.bin";
    const RLIB_RMETA_EXT: &[u8] = b".rmeta";

    require!(
        size >= 68,
        prefix.len() >= 68,
        starts(prefix, AR_MAGIC),
        eq_at(prefix, 66, AR_FMAG),
    );
    let name_hdr = get_slice(prefix, 8, 24)?;
    require!(name_hdr.iter().all(|&b| (0x20..=0x7E).contains(&b)));
    if contains(prefix, RLIB_METADATA_BIN) || contains(prefix, RLIB_RMETA_EXT) {
        return Some(Label::Rlib);
    }
    Some(Label::Ar)
}

/// Disambiguates Unix `!<arch>\n` archives between VMware VIB (`vib`), Debian `.deb` (`deb`),
/// Rust `.rlib` (`rlib`), and generic Unix `ar` (`ar`).
#[inline]
pub(crate) fn vib_or_deb(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| vib(prefix))
        .or_else(|| probe(|| deb(prefix, size)))
        .or_else(|| probe(|| rlib_or_ar(prefix, size)))
}
