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

//! Microsoft Advanced Systems Format (`ASF`) metaformat verifiers (`asf`, `wma`, `wmv`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// Microsoft Advanced Systems Format (`asf`, `wma`, `wmv`).
pub(crate) fn asf_family(prefix: &[u8], size: u64) -> Option<Label> {
    const ASF_HDR: &[u8; 16] = guid!("75B22630-668E-11CF-A6D9-00AA0062CE6C");
    const ASF_AUDIO: &[u8; 16] = guid!("F8699E40-5B4D-11CF-A8FD-00805F5C442B");
    const ASF_VIDEO: &[u8; 16] = guid!("BC19EFC0-5B4D-11CF-A8FD-00805F5C442B");

    require!(
        size >= 30,
        prefix.len() >= 30,
        starts(prefix, ASF_HDR),
        u8_at(prefix, 28)? == 0x01,
        u8_at(prefix, 29)? == 0x02,
    );
    let has_video = contains(prefix, ASF_VIDEO);
    let has_audio = contains(prefix, ASF_AUDIO);
    if has_video {
        return Some(Label::Wmv);
    }
    if has_audio {
        return Some(Label::Wma);
    }
    Some(Label::Asf)
}
