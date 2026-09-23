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

use std::sync::Arc;

use anyhow::{anyhow, Result};
use magika_rules::{Outcome, RuleSet};

use crate::ContentType;

// Rules scan the first block that feature extraction reads anyway, so they never read more.
const _: () = assert!(crate::model::CONFIG.block_size >= magika_rules::PREFIX_LIMIT);

/// Compiled format rules.
///
/// Rules decide a content type from the first bytes and the size of a file, only when every
/// matching rule agrees. Cloning is cheap: clones share the compiled rules.
#[derive(Clone)]
pub struct Rules(Arc<Inner>);

struct Inner {
    set: RuleSet,
    /// The content type of each label of `set`, by index.
    content_types: Vec<ContentType>,
}

impl std::fmt::Debug for Rules {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Rules").finish_non_exhaustive()
    }
}

impl Rules {
    /// Loads the rules bundled with Magika, compiled when `magika-rules` was built.
    pub fn bundled() -> Result<Self> {
        Self::new(RuleSet::bundled())
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
        Ok(Rules(Arc::new(Inner { set, content_types })))
    }

    /// Returns the content type the rules decide from the first block of a file and its size.
    ///
    /// The first block holds at least the first `PREFIX_LIMIT` bytes of the file, or the whole
    /// file if it is shorter.
    pub(crate) fn identify(&self, first_block: &[u8], size: u64) -> Option<ContentType> {
        let prefix = &first_block[..first_block.len().min(magika_rules::PREFIX_LIMIT)];
        match self.0.set.scan(magika_rules::Input { prefix, size, tail: None }) {
            Outcome::Match(label) => Some(self.0.content_types[label]),
            Outcome::NoMatch | Outcome::Conflict | Outcome::InsufficientInput => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use magika_rules::Source;

    use super::*;

    fn rule(label: &str, magic: &str) -> RuleSet {
        let source = Source::parse(&format!(
            r#"rule r {{
                meta: label = "{label}" enforced = true class = "full" fp_rate = 0 fn_rate = 0
                strings: $a = "{magic}"
                condition: $a at 0
            }}"#
        ))
        .unwrap();
        RuleSet::compile(&source).unwrap()
    }

    #[test]
    fn bundled_labels_are_content_types() {
        let rules = Rules::bundled().unwrap();
        assert_eq!(rules.0.content_types.len(), rules.0.set.labels().len());
    }

    #[test]
    fn unknown_labels_are_rejected() {
        let error = Rules::new(rule("not-a-content-type", "x")).unwrap_err();
        assert_eq!(
            error.to_string(),
            r#"rule label "not-a-content-type" is not a Magika content type"#
        );
        assert!(Rules::new(rule("directory", "x")).is_err());
    }

    #[test]
    fn rules_decide_before_inference() {
        let rules = Rules::new(rule("png", "MAGIKA")).unwrap();
        let runtime = crate::Runtime::builder()
            .with_backend(crate::Backend::Cpu)
            .with_max_batch(1)
            .with_rules(rules)
            .build()
            .unwrap();
        let mut session = runtime.session().unwrap();
        let mut file = b"MAGIKA fn main() {}\n".repeat(500);
        let ruled = session.identify_content(&file[..]).unwrap();
        assert!(matches!(ruled, crate::FileType::Ruled(ContentType::Png)), "{ruled:?}");
        file[0] = b'#';
        let inferred = session.identify_content(&file[..]).unwrap();
        assert!(matches!(inferred, crate::FileType::Inferred(_)), "{inferred:?}");
    }

    #[test]
    fn a_rule_decision_reads_only_the_first_block() {
        struct Counting(Vec<u8>, Vec<u64>);
        impl crate::Input for Counting {
            fn length(&self) -> Result<u64> {
                Ok(self.0.len() as u64)
            }
            fn read_at(&mut self, buffer: &mut [u8], offset: u64) -> Result<()> {
                self.1.push(offset);
                let start = offset as usize;
                buffer.copy_from_slice(&self.0[start..][..buffer.len()]);
                Ok(())
            }
        }
        let rules = Rules::new(rule("png", "MAGIKA")).unwrap();
        let mut file = Counting(b"MAGIKA".repeat(10_000), Vec::new());
        let extracted =
            crate::FeaturesOrRuled::extract_with_rules(&mut file, Some(&rules)).unwrap();
        assert!(matches!(extracted, crate::FeaturesOrRuled::Ruled(ContentType::Png)));
        assert_eq!(file.1, [0]);
        file.0[0] = b'#';
        file.1.clear();
        let extracted =
            crate::FeaturesOrRuled::extract_with_rules(&mut file, Some(&rules)).unwrap();
        assert!(matches!(extracted, crate::FeaturesOrRuled::Features(_)));
        assert_eq!(file.1.len(), 2, "{:?}", file.1);
    }
}
