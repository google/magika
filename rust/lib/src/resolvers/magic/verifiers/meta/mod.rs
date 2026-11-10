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

//! Container and framing metaformat verifiers.
//!
//! **Precedence rule**: metaformat verifiers take precedence over standalone category
//! verifiers (`super::category`). Whenever a format is a subformat of a container or
//! framing metaformat (`ar`, `asf`, `asn1`, `ebml`, `elf`, `iff`, `iso`, `isobmff`,
//! `json`, `ole2`, `pdf`, `riff`, `sqlite`, `xml`, `zip`), its verifier belongs here
//! under `meta/` rather than under `category/`.

pub(crate) mod ar;
pub(crate) mod asf;
pub(crate) mod asn1;
pub(crate) mod ebml;
pub(crate) mod elf;
pub(crate) mod iff;
pub(crate) mod iso;
pub(crate) mod isobmff;
pub(crate) mod json;
pub(crate) mod ole2;
pub(crate) mod pdf;
pub(crate) mod riff;
pub(crate) mod sqlite;
pub(crate) mod xml;
pub(crate) mod zip;

pub(crate) use self::ar::*;
pub(crate) use self::asf::*;
pub(crate) use self::asn1::*;
pub(crate) use self::ebml::*;
pub(crate) use self::elf::*;
pub(crate) use self::iff::*;
pub(crate) use self::iso::*;
pub(crate) use self::isobmff::*;
pub(crate) use self::json::*;
pub(crate) use self::ole2::*;
pub(crate) use self::pdf::*;
pub(crate) use self::riff::*;
pub(crate) use self::sqlite::*;
pub(crate) use self::xml::*;
pub(crate) use self::zip::*;
