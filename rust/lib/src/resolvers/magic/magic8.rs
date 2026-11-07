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

//! Tier 3 (`magic8`): 1-byte stream dispatch at offset 0.

use crate::ContentType as Label;
use super::*;

/// Tier 3 (`magic8`): dispatches on the leading byte `b0` at offset 0.
#[inline]
pub(crate) fn probe(b0: u8, prefix: &[u8], ctx: &Ctx<'_>) -> Option<Label> {
    let size = ctx.size;
    match b0 {
        0xFF => stream_ff(prefix, size), // 0xFF-prefixed streams (JPEG, J2K, JXL codestream, MP2, MP3)
        0x99 | 0x95 | 0xC6 | 0xC5 => openpgp_binary(prefix, size), // RFC 4880 OpenPGP Binary Keyring
        0x1A => arc(prefix, size), // System Enhancement Associates ARC Archive (1985) Thom Henderson / SEA
        0x80 | 0x82 => omf_or_msgpack(prefix, size), // Intel 8086 OMF Object / MessagePack FixMap
        0xF0 => omf_intel(prefix, size), // Intel / Microsoft Relocatable OMF Static Library (.lib) (1981) Intel / Microsoft
        0x81 | 0x83..=0x8F => msgpack(prefix, size), // MessagePack FixMap (2008) Sadayuki Furuhashi
        0x0A => protobuf(prefix, size), // Protocol Buffers FileDescriptorSet / FileDescriptorProto (2008) Google
        b'B' | b'G' | b'P' => printfox(prefix, size), // C64 Printfox / Pagefox RLE Bitmap (1986) Scanntronik
        b'>' => fasta(prefix, size), // FASTA Nucleotide / Peptide Sequence (1985) William R. Pearson
        b'@' => fastq(prefix, size), // FASTQ Sequencing Reads with Phred Quality (2000) Sanger Institute
        _ => None,
    }
}
