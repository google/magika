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

use std::collections::HashSet;
use std::sync::LazyLock;

use anyhow::{Result, anyhow, bail, ensure};
use magika_rules::{Class, Outcome, RuleSet};

use crate::ContentType;

// Rules scan the first block that feature extraction reads anyway, so they never read more.
const _: () = assert!(crate::model::CONFIG.block_size >= magika_rules::PREFIX_LIMIT);

/// Compiled format rules.
///
/// Rules decide a content type from the first bytes and the size of a file, only when every
/// matching rule agrees.
pub(crate) struct Rules {
    set: RuleSet,
    /// The map from labels (index in `set.labels()`) to content types.
    content_types: Vec<ContentType>,
    /// The set of content types covered by rules without false negatives.
    vetos: HashSet<ContentType>,
}

impl Rules {
    /// Returns the rules bundled with Magika, compiled when `magika-rules` was built.
    fn bundled() -> &'static Self {
        static RULES: LazyLock<Rules> = LazyLock::new(|| Rules::new(RuleSet::bundled()).unwrap());
        &RULES
    }

    /// Wraps compiled rules, failing if a rule labels a content type that Magika does not know.
    fn new(set: RuleSet) -> Result<Self> {
        let mut content_types = Vec::with_capacity(set.labels().len());
        let mut vetos = HashSet::new();
        for label in set.labels() {
            let content_type = ContentType::from_label(label)
                .ok_or_else(|| anyhow!("rule label {label:?} is not a Magika content type"))?;
            content_types.push(content_type);
            let mut class = None;
            for rule in set.rules() {
                if !rule.enforced || rule.label.as_ref() != Some(label) {
                    continue;
                }
                if let Some(prev) = class.replace(rule.class) {
                    ensure!(
                        prev == rule.class,
                        "enforced rules for label {label:?} disagree on class: {prev:?} vs {:?}",
                        rule.class
                    );
                }
            }
            match class {
                Some(Class::Full) => drop(vetos.insert(content_type)),
                Some(Class::Partial) => (),
                _ => bail!("label {label:?} has no valid enforced rule class: {class:?}"),
            }
        }
        Ok(Rules { set, content_types, vetos })
    }

    /// Returns the content types of the rules that match a file.
    ///
    /// The first block holds at least the first `PREFIX_LIMIT` bytes of the file, or the whole
    /// file if it is shorter.
    pub(crate) fn identify(
        first_block: &[u8], size: u64, mut file: impl crate::Input,
    ) -> Result<Vec<ContentType>> {
        let rules = Self::bundled();
        let prefix = &first_block[..first_block.len().min(magika_rules::PREFIX_LIMIT)];
        let mut tail = None;
        let tail_len = rules.set.tail_len(prefix, size);
        if tail_len > 0 {
            let mut buf = vec![0; tail_len];
            file.read_at(&mut buf, size - tail_len as u64)?;
            if let Some(start) = rules.set.tail_start(&buf, size) {
                buf = vec![0; (size - start) as usize];
                file.read_at(&mut buf, start)?;
            }
            tail = Some(buf);
        }
        let tail = tail.as_deref();
        Ok(match rules.set.scan(magika_rules::Input { prefix, size, tail }) {
            Outcome::Match(labels) => labels.into_iter().map(|i| rules.content_types[i]).collect(),
            Outcome::InsufficientInput => Vec::new(),
        })
    }

    /// Returns whether the rules cover a content type with no false negatives.
    pub(crate) fn veto(content_type: ContentType) -> bool {
        Self::bundled().vetos.contains(&content_type)
    }
}
