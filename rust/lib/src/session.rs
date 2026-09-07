// Copyright 2024 Google LLC
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

use std::path::Path;

use anyhow::Result;
use ndarray::ArrayView2;

use crate::{BackendInfo, Features, FeaturesOrRuled, FileType, Input, RulesMode, Runtime};

/// A Magika session to identify files.
pub struct Session {
    pub(crate) rules_mode: RulesMode,
    #[cfg(feature = "yara-rules")]
    pub(crate) ruleset: Option<crate::RuleSet>,
    #[cfg(test)]
    pub(crate) inference_runs: usize,
    pub(crate) inner: magika_tract_runtime::Session,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Session").field("backend", &self.backend_info()).finish()
    }
}

impl Session {
    /// Creates a default session.
    pub fn new() -> Result<Self> {
        Runtime::new()?.session()
    }

    /// Returns the resolved CPU or GPU implementation.
    pub fn backend_info(&self) -> BackendInfo {
        self.inner.backend_info().into()
    }

    /// Identifies a single file.
    pub fn identify_file(&mut self, file: impl AsRef<Path>) -> Result<FileType> {
        let file = file.as_ref();
        let metadata = std::fs::symlink_metadata(file)?;
        if metadata.is_dir() {
            Ok(FileType::Directory)
        } else if metadata.is_symlink() {
            Ok(FileType::Symlink)
        } else {
            self.identify_content(std::fs::File::open(file)?)
        }
    }

    /// Identifies a single file from its content.
    pub fn identify_content(&mut self, file: impl Input) -> Result<FileType> {
        #[cfg(feature = "yara-rules")]
        if self.rules_mode == RulesMode::Enforce {
            if let Some(rules) = &self.ruleset {
                let prepared = FeaturesOrRuled::extract_with_ruleset(file, rules)?;
                return self.identify_prepared(prepared);
            }
        }
        self.identify_prepared(FeaturesOrRuled::extract_with_rules(file, self.rules_mode)?)
    }

    fn identify_prepared(&mut self, prepared: FeaturesOrRuled) -> Result<FileType> {
        match prepared {
            FeaturesOrRuled::Ruled(content_type) => Ok(FileType::Ruled(content_type)),
            FeaturesOrRuled::Features(features) => self.identify_features(&features),
        }
    }

    /// Identifies a single file from its features.
    pub fn identify_features(&mut self, features: &Features) -> Result<FileType> {
        let results = self.identify_features_batch(std::slice::from_ref(features))?;
        let [result] = results.try_into().ok().unwrap();
        Ok(result)
    }

    /// Identifies multiple files in parallel from their features.
    pub fn identify_features_batch(&mut self, features: &[Features]) -> Result<Vec<FileType>> {
        if features.is_empty() {
            return Ok(Vec::new());
        }
        #[cfg(test)]
        {
            self.inference_runs += 1;
        }
        let input: Vec<_> =
            features.iter().flat_map(|features| features.0.iter().copied()).collect();
        let output = self.inner.run(&input, features.len())?;
        let output = ArrayView2::from_shape((features.len(), crate::model::NUM_LABELS), &output)?;
        Ok(FileType::convert(output.into_dyn()))
    }
}

#[cfg(test)]
mod rules_tests {
    use super::*;
    #[test]
    fn prepared_rule_results_never_invoke_inference() {
        let runtime =
            Runtime::builder().with_backend(crate::Backend::Cpu).with_max_batch(1).build().unwrap();
        let mut session = runtime.session().unwrap();
        let ruled = FeaturesOrRuled::extract_with_matcher(&b"known file"[..], |_, _| {
            Some(crate::ContentType::Png)
        })
        .unwrap();
        assert!(matches!(
            session.identify_prepared(ruled).unwrap(),
            FileType::Ruled(crate::ContentType::Png)
        ));
        assert_eq!(session.inference_runs, 0);
        session.identify_content(&b"fn main() { println!(\"hello\"); }"[..]).unwrap();
        assert_eq!(session.inference_runs, 1);
    }
    #[cfg(feature = "yara-rules")]
    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn loaded_rules_bypass_inference_and_preserve_fallback() {
        let pack = crate::RuleSet::from_source(
            r#"
            rule synthetic { meta: label = "png" enabled = true class = "full" fp_rate = 0 fn_rate = 0
                strings: $a = "MAGIKA_TEST_RULE"
                condition: $a at 0 and original_size >= 16 }
        "#,
        )
        .unwrap();
        let runtime = Runtime::builder()
            .with_backend(crate::Backend::Cpu)
            .with_max_batch(1)
            .with_rules_mode(RulesMode::Enforce)
            .with_ruleset(pack.clone())
            .build()
            .unwrap();
        let mut session = runtime.session().unwrap();
        struct PrefixOnly;
        impl Input for PrefixOnly {
            fn length(&self) -> Result<u64> {
                Ok(100_000)
            }
            fn read_at(&mut self, buffer: &mut [u8], offset: u64) -> Result<()> {
                assert_eq!(offset, 0, "a rule hit must not read the tail");
                assert_eq!(buffer.len(), 4096);
                buffer.fill(0);
                buffer[..17].copy_from_slice(b"MAGIKA_TEST_RULE!");
                Ok(())
            }
        }
        assert!(matches!(
            session.identify_content(PrefixOnly).unwrap(),
            FileType::Ruled(crate::ContentType::Png)
        ));
        assert_eq!(session.inference_runs, 0);
        let sample = &b"fn main() { println!(\"hello\"); }"[..];
        let fallback = session.identify_content(sample).unwrap();
        let mut baseline = Runtime::builder()
            .with_backend(crate::Backend::Cpu)
            .with_max_batch(1)
            .with_ruleset(pack)
            .build()
            .unwrap()
            .session()
            .unwrap();
        let expected = baseline.identify_content(sample).unwrap();
        assert_eq!(fallback.content_type(), expected.content_type());
        assert_eq!(fallback.score(), expected.score());
        assert_eq!(session.inference_runs, 1);
    }

    #[cfg(feature = "yara-rules")]
    #[test]
    #[ignore = "requires a native Vectorscan compiler library"]
    fn bundled_pack_bypasses_inference_and_preserves_fallback() {
        let pack = crate::RuleSet::from_source(crate::rules::DEFAULT_RULES).unwrap();
        let runtime = Runtime::builder()
            .with_backend(crate::Backend::Cpu)
            .with_max_batch(1)
            .with_rules_mode(RulesMode::Enforce)
            .with_ruleset(pack.clone())
            .build()
            .unwrap();
        let mut session = runtime.session().unwrap();
        let mut baseline = runtime.session().unwrap();
        baseline.rules_mode = RulesMode::Off;
        struct NoTail<'a>(&'a [u8]);
        impl Input for NoTail<'_> {
            fn length(&self) -> Result<u64> {
                Ok(self.0.len() as u64)
            }
            fn read_at(&mut self, buffer: &mut [u8], offset: u64) -> Result<()> {
                assert_eq!(offset, 0, "rule hit attempted a tail read");
                assert_eq!(buffer.len(), self.0.len().min(crate::rules::PREFIX_LIMIT));
                buffer.copy_from_slice(&self.0[..buffer.len()]);
                Ok(())
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests_data/basic");
        let (mut hits, mut fallbacks) = (0, 0);
        for path in ["wav/test.wav", "txt/lorem-big.txt"] {
            let bytes = std::fs::read(root.join(path)).unwrap();
            let data = bytes.as_slice();
            let before = session.inference_runs;
            if let Some(label) = pack
                .identify(&data[..data.len().min(crate::rules::PREFIX_LIMIT)], data.len() as u64)
            {
                assert!(
                    matches!(session.identify_content(NoTail(data)).unwrap(), FileType::Ruled(actual) if actual == label),
                    "{path}"
                );
                assert_eq!(session.inference_runs, before, "{path}");
                hits += 1;
            } else {
                let expected = FeaturesOrRuled::extract(data).unwrap();
                let actual = FeaturesOrRuled::extract_with_ruleset(data, &pack).unwrap();
                match (&expected, &actual) {
                    (FeaturesOrRuled::Features(a), FeaturesOrRuled::Features(b)) => {
                        assert_eq!(a.0, b.0, "{path}")
                    }
                    (FeaturesOrRuled::Ruled(a), FeaturesOrRuled::Ruled(b)) => {
                        assert_eq!(a, b, "{path}")
                    }
                    _ => panic!("fallback extraction changed: {path}"),
                }
                let expected = baseline.identify_prepared(expected).unwrap();
                let actual = session.identify_prepared(actual).unwrap();
                assert_eq!(actual.content_type(), expected.content_type(), "{path}");
                assert_eq!(actual.score(), expected.score(), "{path}");
                fallbacks += 1;
            }
        }
        assert!(hits > 0 && fallbacks > 0, "provide both rule hits and fallback inputs");
        eprintln!("bundled pack: {hits} hits without inference or tail reads; {fallbacks} exact fallbacks");
    }
}
