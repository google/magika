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

//! Format verifiers organized into `meta/` (container/framing metaformats) and
//! `category/` (standalone format categories), plus `overlap` for cross-category
//! magic-key collisions.
//!
//! **Metaformat precedence over categories**: whenever a format is a subformat of a
//! container or framing metaformat (`ar`, `asf`, `asn1`, `ebml`, `elf`, `iff`, `iso`,
//! `isobmff`, `json`, `ole2`, `pdf`, `riff`, `sqlite`, `xml`, `zip`), its verifier MUST
//! be placed in `meta/<metaformat>.rs` rather than `category/<category>.rs`.

pub(crate) mod category;
pub(crate) mod meta;
pub(crate) mod overlap;

pub(crate) use self::category::*;
pub(crate) use self::meta::*;
pub(crate) use self::overlap::*;
