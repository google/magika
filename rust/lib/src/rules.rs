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

use std::sync::LazyLock;

use anyhow::{anyhow, Result};
use magika_rules::{Outcome, RuleSet};

use crate::ContentType;

// Rules scan the first block that feature extraction reads anyway, so they never read more.
const _: () = assert!(crate::model::CONFIG.block_size >= magika_rules::PREFIX_LIMIT);

/// Compiled format rules.
///
/// Rules decide a content type from the first bytes and the size of a file, only when every
/// matching rule agrees.
pub(crate) struct Rules {
    set: RuleSet,
    /// The content type of each label of `set`, by index.
    content_types: Vec<ContentType>,
}

impl Rules {
    /// Returns the rules bundled with Magika, compiled when `magika-rules` was built.
    fn bundled() -> &'static Self {
        static RULES: LazyLock<Rules> = LazyLock::new(|| Rules::new(RuleSet::bundled()).unwrap());
        &RULES
    }

    /// Wraps compiled rules, failing if a rule labels a content type that Magika does not know.
    fn new(set: RuleSet) -> Result<Self> {
        let content_types = set
            .labels()
            .iter()
            .map(|label| {
                ContentType::from_label(label)
                    .ok_or_else(|| anyhow!("rule label {label:?} is not a Magika content type"))
            })
            .collect::<Result<_>>()?;
        Ok(Rules { set, content_types })
    }

    /// Returns the content type the rules decide from the first block of a file and its size.
    ///
    /// The first block holds at least the first `PREFIX_LIMIT` bytes of the file, or the whole
    /// file if it is shorter.
    pub(crate) fn identify(first_block: &[u8], size: u64) -> Option<ContentType> {
        let rules = Self::bundled();
        let prefix = &first_block[..first_block.len().min(magika_rules::PREFIX_LIMIT)];
        match rules.set.scan(magika_rules::Input { prefix, size, tail: None }) {
            Outcome::Match(label) => Some(rules.content_types[label]),
            Outcome::NoMatch | Outcome::Conflict | Outcome::InsufficientInput => None,
        }
    }
}
