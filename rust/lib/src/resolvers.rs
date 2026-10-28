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

use anyhow::Result;

use crate::cached_input::CachedInput;
use crate::{ContentType, Input};

/// Returns the content types of a file without false positives.
///
/// The returned list may have false negatives (the file should have a content type that's not
/// returned). In particular, the list may be empty, in which case the rules and the model will also
/// try to identify the file.
pub(crate) fn identify(file: &mut CachedInput<impl Input>) -> Result<Vec<ContentType>> {
    let _ = file;
    Ok(Vec::new())
}

/// List of content types for which the resolvers have no false negatives.
///
/// If the resolvers don't return a match for a given file, and the model returns a content type
/// from this list, then that content type will be vetoed and "txt" or "unknown" will be used (based
/// on "is_text" from the content type that was vetoed).
pub(crate) const VETOS: &[ContentType] = &[];
