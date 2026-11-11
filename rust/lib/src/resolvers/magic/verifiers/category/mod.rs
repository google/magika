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

//! Standalone category-grouped format verifiers.
//!
//! Formats that are subformats of a container or framing metaformat (`ar`, `asf`, `asn1`,
//! `ebml`, `elf`, `iff`, `iso`, `isobmff`, `json`, `ole2`, `pdf`, `riff`, `sqlite`, `xml`,
//! `zip`) live in `super::meta` instead, as metaformats take precedence over categories.

pub(crate) mod archive;
pub(crate) mod audio;
pub(crate) mod bio;
pub(crate) mod bytecode;
pub(crate) mod crypto;
pub(crate) mod database;
pub(crate) mod document;
pub(crate) mod embedded;
pub(crate) mod executable;
pub(crate) mod font;
pub(crate) mod image;
pub(crate) mod model3d;
pub(crate) mod network;
pub(crate) mod serialization;
pub(crate) mod storage;
pub(crate) mod system;
pub(crate) mod text;
pub(crate) mod video;

pub(crate) use self::archive::*;
pub(crate) use self::audio::*;
pub(crate) use self::bio::*;
pub(crate) use self::bytecode::*;
pub(crate) use self::crypto::*;
pub(crate) use self::database::*;
pub(crate) use self::document::*;
pub(crate) use self::embedded::*;
pub(crate) use self::executable::*;
pub(crate) use self::font::*;
pub(crate) use self::image::*;
pub(crate) use self::model3d::*;
pub(crate) use self::network::*;
pub(crate) use self::serialization::*;
pub(crate) use self::storage::*;
pub(crate) use self::system::*;
pub(crate) use self::text::*;
pub(crate) use self::video::*;
